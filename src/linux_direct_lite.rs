use std::sync::{Arc, Mutex};

use cosmic_text::{
    Align, Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, SwashCache,
    Weight, Wrap,
};
use tiny_skia::{
    FillRule, Mask, Paint, Path, PathBuilder, PixmapMut, Rect as SkRect, Stroke, Transform,
};

use crate::linux_direct_icons::{draw_normalized_icon, LinuxIconCanvas};
use crate::linux_direct_menu::{
    LinuxDirectMenuSurface, LinuxMenuCanvas, LinuxMenuTextMetrics, LinuxMenuTextPlacement,
};
use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawImageCommand, NativeDrawPlan, NativeDrawTextCommand, NativeImageInterpolation,
    NativeStyleResolver, Rect, Size, TextStyle, TextWeight, TextWrap, VerticalAlign,
};

#[derive(Clone)]
pub(crate) struct LinuxLiteTextSystem {
    inner: Arc<Mutex<LinuxLiteTextState>>,
    ui_family: Arc<str>,
    ui_scale: f32,
}

struct LinuxLiteTextState {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl LinuxLiteTextSystem {
    pub(crate) fn new(configured_font: &str) -> Self {
        let (family, logical_pixels) = parse_configured_font(configured_font);
        Self {
            inner: Arc::new(Mutex::new(LinuxLiteTextState {
                font_system: FontSystem::new(),
                swash_cache: SwashCache::new(),
            })),
            ui_family: Arc::from(family),
            ui_scale: (logical_pixels / 14.0).clamp(0.75, 3.0),
        }
    }

    pub(crate) fn ui_family(&self) -> &str {
        &self.ui_family
    }

    pub(crate) const fn ui_scale(&self) -> f32 {
        self.ui_scale
    }

    fn measure(&self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Size {
        if text.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        let Some(mut state) = self.inner.lock().ok() else {
            return Size {
                width: 0,
                height: 0,
            };
        };
        let mut buffer = make_buffer(
            &mut state.font_system,
            text,
            style,
            max_width,
            None,
            style.wrap,
            None,
        );
        buffer.shape_until_scroll(&mut state.font_system, true);
        measure_buffer(&buffer)
    }

    fn draw(
        &self,
        pixmap: &mut PixmapMut<'_>,
        text: &str,
        style: &TextStyle,
        bounds: Rect,
        clip: Rect,
        scale_factor: f32,
        origin_y: i32,
    ) {
        if text.is_empty() || bounds.width <= 0 || bounds.height <= 0 {
            return;
        }
        let Some(mut state) = self.inner.lock().ok() else {
            return;
        };
        let scaled_style = scaled_text_style(style, scale_factor);
        let width = bounds.width.max(0) as f32 * scale_factor;
        let height = bounds.height.max(0) as f32 * scale_factor;
        let alignment = match style.horizontal_align {
            HorizontalAlign::Start => Some(Align::Left),
            HorizontalAlign::Center => Some(Align::Center),
            HorizontalAlign::End => Some(Align::Right),
        };
        let constrain_width = style.wrap == TextWrap::Word || style.ellipsis;
        let mut buffer = make_buffer(
            &mut state.font_system,
            text,
            &scaled_style,
            constrain_width.then_some(width),
            (style.wrap == TextWrap::Word).then_some(height),
            style.wrap,
            constrain_width.then_some(alignment).flatten(),
        );
        buffer.shape_until_scroll(&mut state.font_system, true);
        let measured = measure_buffer(&buffer);
        let mut x = bounds.x as f32 * scale_factor;
        if !constrain_width {
            x += match style.horizontal_align {
                HorizontalAlign::Start => 0.0,
                HorizontalAlign::Center => (width - measured.width as f32).max(0.0) / 2.0,
                HorizontalAlign::End => (width - measured.width as f32).max(0.0),
            };
        }
        let mut y = bounds.y.saturating_add(origin_y) as f32 * scale_factor;
        y += match style.vertical_align {
            VerticalAlign::Start => 0.0,
            VerticalAlign::Center => (height - measured.height as f32).max(0.0) / 2.0,
            VerticalAlign::End => (height - measured.height as f32).max(0.0),
        };
        let clip = intersect_rect(
            clip,
            Rect {
                x: bounds.x,
                y: bounds.y.saturating_add(origin_y),
                width: bounds.width,
                height: bounds.height,
            },
        );
        if clip.width <= 0 || clip.height <= 0 {
            return;
        }
        let clip_left = (clip.x as f32 * scale_factor).floor() as i32;
        let clip_top = (clip.y as f32 * scale_factor).floor() as i32;
        let clip_right = ((clip.x + clip.width) as f32 * scale_factor).ceil() as i32;
        let clip_bottom = ((clip.y + clip.height) as f32 * scale_factor).ceil() as i32;
        let frame_width = pixmap.width() as i32;
        let frame_height = pixmap.height() as i32;
        let pixels = pixmap.data_mut();
        let cosmic_color = cosmic_color(style.color);
        let LinuxLiteTextState {
            font_system,
            swash_cache,
        } = &mut *state;
        buffer.draw(
            font_system,
            swash_cache,
            cosmic_color,
            |gx, gy, width, height, color| {
                let base_x = x.round() as i32 + gx;
                let base_y = y.round() as i32 + gy;
                for row in 0..height as i32 {
                    let py = base_y + row;
                    if py < clip_top || py >= clip_bottom || py < 0 || py >= frame_height {
                        continue;
                    }
                    for column in 0..width as i32 {
                        let px = base_x + column;
                        if px < clip_left || px >= clip_right || px < 0 || px >= frame_width {
                            continue;
                        }
                        let offset = (py as usize * frame_width as usize + px as usize) * 4;
                        blend_swapped_pixel(&mut pixels[offset..offset + 4], color);
                    }
                }
            },
        );
    }
}

impl LinuxMenuTextMetrics for LinuxLiteTextSystem {
    fn measure_menu_text(&self, text: &str) -> (i32, i32) {
        let mut style = TextStyle::line(
            self.ui_family().to_string(),
            14.0 * self.ui_scale(),
            Color::rgb(0, 0, 0),
        );
        style.line_height = 20.0 * self.ui_scale();
        style.ellipsis = false;
        let size = self.measure(text, &style, None);
        (size.width, size.height)
    }
}

pub(crate) fn render_linux_direct_lite_frame(
    frame: &mut [u32],
    plan: &NativeDrawPlan,
    menu_surface: Option<&LinuxDirectMenuSurface>,
    width: u32,
    height: u32,
    scale_factor: f64,
    theme_dark: bool,
    text_system: &LinuxLiteTextSystem,
) -> Result<(), String> {
    if !cfg!(target_endian = "little") {
        return Err("the pure-Rust Linux renderer currently requires little-endian pixels".into());
    }
    let expected_len = width as usize * height as usize;
    if frame.len() != expected_len {
        return Err(format!(
            "software frame length {} does not match {width}x{height}",
            frame.len()
        ));
    }
    // SAFETY: u32 pixels are contiguous and the byte slice has the exact same
    // lifetime and allocation. On little-endian Linux softbuffer consumes
    // B,G,R,X bytes; the renderer swaps red/blue so tiny-skia's R,G,B,A bytes
    // are the same in-memory representation without a second frame buffer.
    let bytes =
        unsafe { std::slice::from_raw_parts_mut(frame.as_mut_ptr().cast::<u8>(), frame.len() * 4) };
    let mut pixmap = PixmapMut::from_bytes(bytes, width, height)
        .ok_or_else(|| "could not bind tiny-skia to the software buffer".to_string())?;
    let palette = NativeDrawPalette::for_mode(plan.theme_mode, theme_dark);
    pixmap.fill(sk_color(palette.surface));
    let content_offset = menu_surface.map_or(0, LinuxDirectMenuSurface::content_offset_y);
    let logical_width = (width as f64 / scale_factor.max(0.1)).ceil() as i32;
    let logical_height = (height as f64 / scale_factor.max(0.1)).ceil() as i32;
    let mut sink = LinuxLiteDrawSink::new(
        pixmap,
        palette,
        plan.typography_scale(),
        scale_factor as f32,
        text_system.clone(),
        Rect {
            x: 0,
            y: content_offset,
            width: logical_width,
            height: logical_height.saturating_sub(content_offset),
        },
        content_offset,
    );
    sink.draw_plan(plan);
    if let Some(menu) = menu_surface {
        sink.reset_for_menu(Rect {
            x: 0,
            y: 0,
            width: logical_width,
            height: logical_height,
        });
        menu.draw(&mut sink, palette);
    }
    Ok(())
}

struct LinuxLiteDrawSink<'a> {
    pixmap: PixmapMut<'a>,
    palette: NativeDrawPalette,
    style_resolver: NativeDrawTextStyleResolver,
    text_system: LinuxLiteTextSystem,
    scale_factor: f32,
    origin_y: i32,
    base_clip: Rect,
    clip_stack: Vec<Rect>,
    mask: Option<Mask>,
}

impl<'a> LinuxLiteDrawSink<'a> {
    fn new(
        pixmap: PixmapMut<'a>,
        palette: NativeDrawPalette,
        typography_scale: f32,
        scale_factor: f32,
        text_system: LinuxLiteTextSystem,
        base_clip: Rect,
        origin_y: i32,
    ) -> Self {
        let profile = linux_lite_typography_profile(typography_scale, &text_system);
        Self {
            pixmap,
            palette,
            style_resolver: NativeDrawTextStyleResolver::from_profile(profile, palette),
            text_system,
            scale_factor: scale_factor.max(0.1),
            origin_y,
            base_clip,
            clip_stack: Vec::new(),
            mask: None,
        }
    }

    fn reset_for_menu(&mut self, clip: Rect) {
        self.origin_y = 0;
        self.base_clip = clip;
        self.clip_stack.clear();
        self.rebuild_mask();
    }

    fn current_clip(&self) -> Rect {
        self.clip_stack.last().copied().unwrap_or(self.base_clip)
    }

    fn physical_rect(&self, rect: Rect) -> Option<SkRect> {
        SkRect::from_xywh(
            rect.x as f32 * self.scale_factor,
            rect.y.saturating_add(self.origin_y) as f32 * self.scale_factor,
            rect.width.max(0) as f32 * self.scale_factor,
            rect.height.max(0) as f32 * self.scale_factor,
        )
    }

    fn paint(color: Color) -> Paint<'static> {
        let mut paint = Paint::default();
        paint.set_color(sk_color(color));
        paint
    }

    fn fill_rect_color(&mut self, rect: Rect, color: Color) {
        let Some(rect) = self.physical_rect(rect) else {
            return;
        };
        self.pixmap.fill_rect(
            rect,
            &Self::paint(color),
            Transform::identity(),
            self.mask.as_ref(),
        );
    }

    fn round_path(&self, rect: Rect, radius: f32) -> Option<Path> {
        let rect = self.physical_rect(rect)?;
        rounded_path(rect, radius.max(0.0) * self.scale_factor)
    }

    fn fill_round_color(&mut self, rect: Rect, radius: f32, color: Color) {
        let Some(path) = self.round_path(rect, radius) else {
            return;
        };
        self.pixmap.fill_path(
            &path,
            &Self::paint(color),
            FillRule::Winding,
            Transform::identity(),
            self.mask.as_ref(),
        );
    }

    fn stroke_path_color(&mut self, path: &Path, color: Color, width: f32) {
        let stroke = Stroke {
            width: width.max(0.5) * self.scale_factor,
            ..Stroke::default()
        };
        self.pixmap.stroke_path(
            path,
            &Self::paint(color),
            &stroke,
            Transform::identity(),
            self.mask.as_ref(),
        );
    }

    fn draw_text_style(&mut self, text: &str, bounds: Rect, style: &TextStyle) {
        let clip = self.current_clip();
        self.text_system.draw(
            &mut self.pixmap,
            text,
            style,
            bounds,
            clip,
            self.scale_factor,
            self.origin_y,
        );
    }

    fn draw_text_command(&mut self, command: &NativeDrawTextCommand) {
        let style = self.style_resolver.resolve_text_style(command.style);
        self.draw_text_style(&command.text, command.bounds, &style);
    }

    fn draw_icon(&mut self, command: &NativeDrawIconCommand) {
        let color = self.palette.resolve(command.color);
        let scale = self.scale_factor;
        let origin_y = self.origin_y;
        let mask = self.mask.as_ref();
        let mut canvas = TinyIconCanvas::new(
            &mut self.pixmap,
            mask,
            command.bounds,
            origin_y,
            scale,
            color,
        );
        draw_normalized_icon(&mut canvas, command.icon);
    }

    fn draw_image(&mut self, command: &NativeDrawImageCommand) {
        if command.bounds.width <= 0
            || command.bounds.height <= 0
            || command.source.width <= 0
            || command.source.height <= 0
        {
            return;
        }
        let clip = intersect_rect(
            self.current_clip(),
            Rect {
                x: command.bounds.x,
                y: command.bounds.y.saturating_add(self.origin_y),
                width: command.bounds.width,
                height: command.bounds.height,
            },
        );
        if clip.width <= 0 || clip.height <= 0 {
            return;
        }
        let scale = self.scale_factor;
        let dst_left = (command.bounds.x as f32 * scale).floor() as i32;
        let dst_top =
            (command.bounds.y.saturating_add(self.origin_y) as f32 * scale).floor() as i32;
        let dst_right = ((command.bounds.x + command.bounds.width) as f32 * scale).ceil() as i32;
        let dst_bottom = ((command.bounds.y + self.origin_y + command.bounds.height) as f32 * scale)
            .ceil() as i32;
        let clip_left = (clip.x as f32 * scale).floor() as i32;
        let clip_top = (clip.y as f32 * scale).floor() as i32;
        let clip_right = ((clip.x + clip.width) as f32 * scale).ceil() as i32;
        let clip_bottom = ((clip.y + clip.height) as f32 * scale).ceil() as i32;
        let source_width = command.frame.width() as i32;
        let source_height = command.frame.height() as i32;
        let source = command.frame.premultiplied_bgra8();
        let frame_width = self.pixmap.width() as i32;
        let frame_height = self.pixmap.height() as i32;
        let target = self.pixmap.data_mut();
        let span_x = (dst_right - dst_left).max(1) as f32;
        let span_y = (dst_bottom - dst_top).max(1) as f32;
        for y in dst_top.max(clip_top).max(0)..dst_bottom.min(clip_bottom).min(frame_height) {
            for x in dst_left.max(clip_left).max(0)..dst_right.min(clip_right).min(frame_width) {
                let fx = (x - dst_left) as f32 / span_x;
                let fy = (y - dst_top) as f32 / span_y;
                let sx = command.source.x as f32 + fx * command.source.width as f32;
                let sy = command.source.y as f32 + fy * command.source.height as f32;
                let (sx, sy) = match command.interpolation {
                    NativeImageInterpolation::Nearest | NativeImageInterpolation::Smooth => (
                        sx.floor().clamp(0.0, (source_width - 1).max(0) as f32) as i32,
                        sy.floor().clamp(0.0, (source_height - 1).max(0) as f32) as i32,
                    ),
                };
                let source_offset = (sy as usize * source_width as usize + sx as usize) * 4;
                let target_offset = (y as usize * frame_width as usize + x as usize) * 4;
                blend_premultiplied_bgra(
                    &mut target[target_offset..target_offset + 4],
                    &source[source_offset..source_offset + 4],
                );
            }
        }
    }

    fn push_clip(&mut self, rect: Rect) {
        let rect = Rect {
            y: rect.y.saturating_add(self.origin_y),
            ..rect
        };
        self.clip_stack
            .push(intersect_rect(self.current_clip(), rect));
        self.rebuild_mask();
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
        self.rebuild_mask();
    }

    fn rebuild_mask(&mut self) {
        if self.clip_stack.is_empty() && self.base_clip.x == 0 && self.base_clip.y == 0 {
            self.mask = None;
            return;
        }
        let Some(mut mask) = Mask::new(self.pixmap.width(), self.pixmap.height()) else {
            self.mask = None;
            return;
        };
        let clip = self.current_clip();
        if clip.width > 0 && clip.height > 0 {
            if let Some(rect) = SkRect::from_xywh(
                clip.x as f32 * self.scale_factor,
                clip.y as f32 * self.scale_factor,
                clip.width as f32 * self.scale_factor,
                clip.height as f32 * self.scale_factor,
            ) {
                let path = PathBuilder::from_rect(rect);
                mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
            }
        }
        self.mask = Some(mask);
    }
}

impl NativeDrawCommandSink for LinuxLiteDrawSink<'_> {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                self.fill_rect_color(*rect, self.palette.resolve_source_fill(*fill));
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => {
                if let Some(rect) = self.physical_rect(*rect) {
                    let path = PathBuilder::from_rect(rect);
                    self.stroke_path_color(
                        &path,
                        self.palette.resolve_source_fill(*stroke),
                        *width as f32,
                    );
                }
            }
            NativeDrawCommand::StrokeArc {
                rect,
                stroke,
                width,
                start_degrees,
                sweep_degrees,
            } => {
                if let Some(path) = arc_path(
                    *rect,
                    self.origin_y,
                    self.scale_factor,
                    *start_degrees as f32,
                    *sweep_degrees as f32,
                ) {
                    self.stroke_path_color(
                        &path,
                        self.palette.resolve_source_fill(*stroke),
                        *width as f32,
                    );
                }
            }
            NativeDrawCommand::FillTriangle { points, fill } => {
                let mut path = PathBuilder::new();
                path.move_to(
                    points[0].x as f32 * self.scale_factor,
                    points[0].y.saturating_add(self.origin_y) as f32 * self.scale_factor,
                );
                for point in &points[1..] {
                    path.line_to(
                        point.x as f32 * self.scale_factor,
                        point.y.saturating_add(self.origin_y) as f32 * self.scale_factor,
                    );
                }
                path.close();
                if let Some(path) = path.finish() {
                    self.pixmap.fill_path(
                        &path,
                        &Self::paint(self.palette.resolve_source_fill(*fill)),
                        FillRule::Winding,
                        Transform::identity(),
                        self.mask.as_ref(),
                    );
                }
            }
            NativeDrawCommand::RoundRect {
                rect,
                fill,
                stroke,
                radius,
            } => {
                self.fill_round_color(
                    *rect,
                    *radius as f32,
                    self.palette.resolve_source_fill(*fill),
                );
                if let (Some(stroke), Some(path)) = (stroke, self.round_path(*rect, *radius as f32))
                {
                    self.stroke_path_color(&path, self.palette.resolve_source_fill(*stroke), 1.0);
                }
            }
            NativeDrawCommand::RoundFill { rect, fill, radius } => self.fill_round_color(
                *rect,
                *radius as f32,
                self.palette.resolve_source_fill(*fill),
            ),
            NativeDrawCommand::Text(command) => self.draw_text_command(command),
            #[cfg(feature = "password-box")]
            NativeDrawCommand::SecureText(command) => {
                let rendered = command.rendered_text();
                self.draw_text_command(&NativeDrawTextCommand::new(
                    rendered.as_str(),
                    command.bounds,
                    command.style,
                ));
            }
            NativeDrawCommand::Icon(command) => self.draw_icon(command),
            NativeDrawCommand::Image(command) => self.draw_image(command),
            NativeDrawCommand::PushClip { rect } => self.push_clip(*rect),
            NativeDrawCommand::PopClip => self.pop_clip(),
        }
    }
}

impl LinuxMenuCanvas for LinuxLiteDrawSink<'_> {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.fill_rect_color(rect, color);
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: f64, color: Color) {
        self.fill_round_color(rect, radius as f32, color);
    }

    fn stroke_round_rect(&mut self, rect: Rect, radius: f64, color: Color, width: f32) {
        if let Some(path) = self.round_path(rect, radius as f32) {
            self.stroke_path_color(&path, color, width);
        }
    }

    fn draw_text(
        &mut self,
        text: &str,
        bounds: Rect,
        color: Color,
        placement: LinuxMenuTextPlacement,
    ) {
        let mut style = TextStyle::line(
            self.text_system.ui_family().to_string(),
            14.0 * self.text_system.ui_scale(),
            color,
        );
        style.line_height = 20.0 * self.text_system.ui_scale();
        style.ellipsis = true;
        style.horizontal_align = match placement {
            LinuxMenuTextPlacement::Start => HorizontalAlign::Start,
            LinuxMenuTextPlacement::Center => HorizontalAlign::Center,
            LinuxMenuTextPlacement::End => HorizontalAlign::End,
        };
        self.draw_text_style(text, bounds, &style);
    }
}

struct TinyIconCanvas<'a, 'b> {
    pixmap: &'a mut PixmapMut<'b>,
    mask: Option<&'a Mask>,
    builder: PathBuilder,
    has_path: bool,
    current: (f32, f32),
    origin_x: f32,
    origin_y: f32,
    scale_x: f32,
    scale_y: f32,
    paint: Paint<'static>,
    stroke: Stroke,
}

impl<'a, 'b> TinyIconCanvas<'a, 'b> {
    fn new(
        pixmap: &'a mut PixmapMut<'b>,
        mask: Option<&'a Mask>,
        bounds: Rect,
        origin_y: i32,
        scale_factor: f32,
        color: Color,
    ) -> Self {
        let scale_x = bounds.width.max(1) as f32 * scale_factor / 16.0;
        let scale_y = bounds.height.max(1) as f32 * scale_factor / 16.0;
        Self {
            pixmap,
            mask,
            builder: PathBuilder::new(),
            has_path: false,
            current: (0.0, 0.0),
            origin_x: bounds.x as f32 * scale_factor,
            origin_y: bounds.y.saturating_add(origin_y) as f32 * scale_factor,
            scale_x,
            scale_y,
            paint: LinuxLiteDrawSink::paint(color),
            stroke: Stroke {
                width: 1.35 * scale_x.min(scale_y),
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Stroke::default()
            },
        }
    }

    fn point(&self, x: f64, y: f64) -> (f32, f32) {
        (
            self.origin_x + x as f32 * self.scale_x,
            self.origin_y + y as f32 * self.scale_y,
        )
    }

    fn take_path(&mut self) -> Option<Path> {
        self.has_path = false;
        self.current = (0.0, 0.0);
        std::mem::replace(&mut self.builder, PathBuilder::new()).finish()
    }
}

impl LinuxIconCanvas for TinyIconCanvas<'_, '_> {
    fn new_sub_path(&mut self) {
        self.has_path = false;
    }

    fn move_to(&mut self, x: f64, y: f64) {
        let point = self.point(x, y);
        self.builder.move_to(point.0, point.1);
        self.current = point;
        self.has_path = true;
    }

    fn line_to(&mut self, x: f64, y: f64) {
        let point = self.point(x, y);
        if self.has_path {
            self.builder.line_to(point.0, point.1);
        } else {
            self.builder.move_to(point.0, point.1);
            self.has_path = true;
        }
        self.current = point;
    }

    fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        let first = self.point(x1, y1);
        let second = self.point(x2, y2);
        let end = self.point(x, y);
        self.builder
            .cubic_to(first.0, first.1, second.0, second.1, end.0, end.1);
        self.current = end;
        self.has_path = true;
    }

    fn arc(&mut self, x: f64, y: f64, radius: f64, start: f64, end: f64) {
        let steps = (((end - start).abs() / std::f64::consts::TAU) * 24.0)
            .ceil()
            .max(4.0) as usize;
        for index in 0..=steps {
            let angle = start + (end - start) * index as f64 / steps as f64;
            let px = x + radius * angle.cos();
            let py = y + radius * angle.sin();
            if index == 0 && !self.has_path {
                self.move_to(px, py);
            } else {
                self.line_to(px, py);
            }
        }
    }

    fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close_path();
    }

    fn set_line_width(&mut self, width: f64) {
        self.stroke.width = width.max(0.5) as f32 * self.scale_x.min(self.scale_y);
    }

    fn close_path(&mut self) {
        self.builder.close();
    }

    fn stroke(&mut self) {
        if let Some(path) = self.take_path() {
            self.pixmap.stroke_path(
                &path,
                &self.paint,
                &self.stroke,
                Transform::identity(),
                self.mask,
            );
        }
    }

    fn fill(&mut self) {
        if let Some(path) = self.take_path() {
            self.pixmap.fill_path(
                &path,
                &self.paint,
                FillRule::Winding,
                Transform::identity(),
                self.mask,
            );
        }
    }
}

pub(crate) fn linux_lite_typography_profile(
    typography_scale: f32,
    text_system: &LinuxLiteTextSystem,
) -> crate::NativeTypographyProfile {
    let family = text_system.ui_family().to_string();
    let mut profile = crate::NativeTypographyProfile::new(
        crate::ZsTypographyPlatformStyle::Gtk,
        "fontdb_cosmic_text",
        family.clone(),
        "Monospace",
        family,
        typography_scale,
        "cosmic_text_swash_tiny_skia_softbuffer",
    )
    .with_configured_ui_font(crate::linux_direct::linux_direct_configured_font_name());
    let body = profile.body_metrics;
    let style = TextStyle {
        font_family: profile.ui_font_family.clone(),
        size: body.size,
        line_height: body.line_height,
        semantic_role: Some(crate::TextRole::Body),
        weight: TextWeight::Regular,
        color: Color::rgb(0, 0, 0),
        horizontal_align: HorizontalAlign::Start,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: false,
    };
    if let Ok(mut state) = text_system.inner.lock() {
        let mut buffer = make_buffer(
            &mut state.font_system,
            "Hg",
            &style,
            None,
            None,
            TextWrap::NoWrap,
            None,
        );
        buffer.shape_until_scroll(&mut state.font_system, true);
        if let Some(run) = buffer.layout_runs().next() {
            let ascent = run.line_y - run.line_top;
            let descent = (run.line_top + run.line_height - run.line_y).max(0.0);
            let leading = (body.line_height - ascent - descent).max(0.0);
            profile = profile.with_body_vertical_metrics(ascent, descent, leading);
        }
    }
    profile
}

#[cfg(feature = "text-input-core")]
pub(crate) fn shape_linux_lite_text_line(
    text_system: &LinuxLiteTextSystem,
    text: &str,
) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
    use crate::native_input_visuals::{
        NativeShapedTextCaret, NativeShapedTextCluster, NativeShapedTextLine,
    };

    if text.is_empty() {
        return None;
    }
    let body = crate::TextRole::Body.metrics_for(crate::ZsTypographyPlatformStyle::Gtk);
    let mut style = TextStyle::line(
        text_system.ui_family().to_string(),
        body.size * text_system.ui_scale(),
        Color::rgb(0, 0, 0),
    );
    style.line_height = body.line_height * text_system.ui_scale();
    style.ellipsis = false;
    let mut state = text_system.inner.lock().ok()?;
    let mut buffer = make_buffer(
        &mut state.font_system,
        text,
        &style,
        None,
        None,
        TextWrap::NoWrap,
        None,
    );
    buffer.shape_until_scroll(&mut state.font_system, true);
    let run = buffer.layout_runs().next()?;
    let boundaries = crate::native_text_edit::grapheme_boundaries(text);
    let byte_offsets = boundaries
        .iter()
        .map(|index| crate::native_text_edit::char_to_byte_index(text, *index))
        .collect::<Vec<_>>();
    let mut clusters = Vec::with_capacity(boundaries.len().saturating_sub(1));
    for (logical, bytes) in boundaries.windows(2).zip(byte_offsets.windows(2)) {
        let mut left = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        let mut rtl = false;
        for glyph in run
            .glyphs
            .iter()
            .filter(|glyph| glyph.end > bytes[0] && glyph.start < bytes[1])
        {
            let cluster_boundaries = byte_offsets
                .iter()
                .copied()
                .filter(|offset| *offset >= glyph.start && *offset <= glyph.end)
                .collect::<Vec<_>>();
            let part_count = cluster_boundaries.len().saturating_sub(1).max(1) as f32;
            let part = cluster_boundaries
                .windows(2)
                .position(|pair| pair[0] <= bytes[0] && pair[1] >= bytes[1])
                .unwrap_or(0) as f32;
            let glyph_left = glyph.x + glyph.w * part / part_count;
            let glyph_right = glyph.x + glyph.w * (part + 1.0) / part_count;
            left = left.min(glyph_left.min(glyph_right));
            right = right.max(glyph_left.max(glyph_right));
            rtl = glyph.level.is_rtl();
        }
        if !left.is_finite() || !right.is_finite() {
            let fallback = clusters
                .last()
                .map(|cluster: &NativeShapedTextCluster| cluster.end_x)
                .unwrap_or(0);
            left = fallback as f32;
            right = left + style.size.max(1.0) * 0.5;
        }
        let (start_x, end_x) = if rtl {
            (right.round() as i32, left.round() as i32)
        } else {
            (left.round() as i32, right.round() as i32)
        };
        clusters.push(NativeShapedTextCluster {
            start: logical[0],
            end: logical[1],
            start_x,
            end_x,
        });
    }
    let mut carets = Vec::with_capacity(boundaries.len());
    for (position, index) in boundaries.iter().copied().enumerate() {
        let previous = position
            .checked_sub(1)
            .and_then(|previous| clusters.get(previous))
            .map(|cluster| cluster.end_x);
        let next = clusters.get(position).map(|cluster| cluster.start_x);
        let primary = previous.or(next).unwrap_or(0);
        let secondary = next.or(previous).unwrap_or(primary);
        carets.push(NativeShapedTextCaret {
            index,
            primary_x: primary,
            secondary_x: secondary,
        });
    }
    NativeShapedTextLine::new(run.line_w.ceil() as i32, clusters, carets)
}

fn make_buffer(
    font_system: &mut FontSystem,
    text: &str,
    style: &TextStyle,
    width: Option<f32>,
    height: Option<f32>,
    wrap: TextWrap,
    alignment: Option<Align>,
) -> Buffer {
    let size = style.size.max(1.0);
    let line_height = style.line_height.max(size);
    let mut buffer = Buffer::new(font_system, Metrics::new(size, line_height));
    buffer.set_wrap(
        font_system,
        if wrap == TextWrap::Word {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        },
    );
    buffer.set_size(font_system, width, height);
    let attrs = Attrs::new()
        .family(if style.font_family.eq_ignore_ascii_case("monospace") {
            Family::Monospace
        } else if style.font_family.trim().is_empty() {
            Family::SansSerif
        } else {
            Family::Name(&style.font_family)
        })
        .weight(match style.weight {
            TextWeight::Automatic | TextWeight::Regular => Weight::NORMAL,
            TextWeight::Medium => Weight::MEDIUM,
            TextWeight::Semibold => Weight::SEMIBOLD,
            TextWeight::Bold => Weight::BOLD,
        });
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced, alignment);
    buffer
}

fn measure_buffer(buffer: &Buffer) -> Size {
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = height.max(run.line_top + run.line_height);
    }
    Size {
        width: width.ceil().max(0.0) as i32,
        height: height.ceil().max(0.0) as i32,
    }
}

fn scaled_text_style(style: &TextStyle, scale: f32) -> TextStyle {
    let mut scaled = style.clone();
    scaled.size = (style.size * scale).max(1.0);
    scaled.line_height = (style.line_height.max(style.size) * scale).max(scaled.size);
    scaled
}

fn parse_configured_font(configured: &str) -> (String, f32) {
    let configured = configured.trim().trim_matches('\'');
    let mut parts = configured.rsplitn(2, char::is_whitespace);
    let possible_size = parts.next().unwrap_or_default();
    let family = parts.next().unwrap_or(configured).trim();
    let point_size = possible_size.parse::<f32>().ok().filter(|size| *size > 0.0);
    let family = if point_size.is_some() && !family.is_empty() {
        family
    } else {
        configured
    };
    (
        if family.is_empty() { "Sans" } else { family }.to_string(),
        point_size.map(|size| size * (96.0 / 72.0)).unwrap_or(14.0),
    )
}

fn cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.b, color.g, color.r, color.a)
}

fn sk_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(color.b, color.g, color.r, color.a)
}

fn blend_swapped_pixel(target: &mut [u8], source: CosmicColor) {
    let alpha = u16::from(source.a());
    let inverse = 255 - alpha;
    for (target, source) in target[..3]
        .iter_mut()
        .zip([source.r(), source.g(), source.b()])
    {
        *target = ((u16::from(source) * alpha + u16::from(*target) * inverse + 127) / 255) as u8;
    }
    target[3] = 255;
}

fn blend_premultiplied_bgra(target: &mut [u8], source: &[u8]) {
    let alpha = u16::from(source[3]);
    let inverse = 255 - alpha;
    for channel in 0..3 {
        target[channel] = (u16::from(source[channel])
            + (u16::from(target[channel]) * inverse + 127) / 255)
            .min(255) as u8;
    }
    target[3] = 255;
}

fn intersect_rect(a: Rect, b: Rect) -> Rect {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = a.x.saturating_add(a.width).min(b.x.saturating_add(b.width));
    let bottom =
        a.y.saturating_add(a.height)
            .min(b.y.saturating_add(b.height));
    Rect {
        x: left,
        y: top,
        width: right.saturating_sub(left).max(0),
        height: bottom.saturating_sub(top).max(0),
    }
}

fn rounded_path(rect: SkRect, radius: f32) -> Option<Path> {
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    if radius <= 0.0 {
        return Some(PathBuilder::from_rect(rect));
    }
    let k = 0.552_284_8;
    let left = rect.left();
    let top = rect.top();
    let right = rect.right();
    let bottom = rect.bottom();
    let mut path = PathBuilder::new();
    path.move_to(left + radius, top);
    path.line_to(right - radius, top);
    path.cubic_to(
        right - radius + radius * k,
        top,
        right,
        top + radius - radius * k,
        right,
        top + radius,
    );
    path.line_to(right, bottom - radius);
    path.cubic_to(
        right,
        bottom - radius + radius * k,
        right - radius + radius * k,
        bottom,
        right - radius,
        bottom,
    );
    path.line_to(left + radius, bottom);
    path.cubic_to(
        left + radius - radius * k,
        bottom,
        left,
        bottom - radius + radius * k,
        left,
        bottom - radius,
    );
    path.line_to(left, top + radius);
    path.cubic_to(
        left,
        top + radius - radius * k,
        left + radius - radius * k,
        top,
        left + radius,
        top,
    );
    path.close();
    path.finish()
}

fn arc_path(
    rect: Rect,
    origin_y: i32,
    scale: f32,
    start_degrees: f32,
    sweep_degrees: f32,
) -> Option<Path> {
    let cx = (rect.x as f32 + rect.width as f32 / 2.0) * scale;
    let cy = (rect.y.saturating_add(origin_y) as f32 + rect.height as f32 / 2.0) * scale;
    let radius = rect.width.min(rect.height).max(0) as f32 * scale / 2.0;
    if radius <= 0.0 || sweep_degrees == 0.0 {
        return None;
    }
    let steps = ((sweep_degrees.abs() / 360.0) * 48.0).ceil().max(4.0) as usize;
    let mut path = PathBuilder::new();
    for index in 0..=steps {
        let angle = (start_degrees + sweep_degrees * index as f32 / steps as f32).to_radians();
        let x = cx + angle.cos() * radius;
        let y = cy + angle.sin() * radius;
        if index == 0 {
            path.move_to(x, y);
        } else {
            path.line_to(x, y);
        }
    }
    path.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_font_parser_keeps_multi_word_family() {
        assert_eq!(
            parse_configured_font("Ubuntu Sans 11"),
            ("Ubuntu Sans".to_string(), 11.0 * 96.0 / 72.0)
        );
    }

    #[test]
    fn clipping_intersection_never_returns_negative_size() {
        assert_eq!(
            intersect_rect(
                Rect {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 10,
                },
                Rect {
                    x: 20,
                    y: 20,
                    width: 5,
                    height: 5,
                },
            ),
            Rect {
                x: 20,
                y: 20,
                width: 0,
                height: 0,
            }
        );
    }
}
