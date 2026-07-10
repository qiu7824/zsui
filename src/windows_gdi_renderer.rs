use std::{ffi::c_void, io::Cursor, ptr::null, sync::OnceLock};

use crate::{
    Color, ColorRole, HorizontalAlign, NativeDrawCommand, NativeDrawCommandOperation,
    NativeDrawCommandSink, NativeDrawFill, NativeDrawIconCommand, NativeDrawPlan,
    NativeDrawTextCommand, NativeIconColorMode, NativeStyleResolver, Rect, Renderer,
    SemanticTextStyle, Size, TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::{
        CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
        GetStockObject, IntersectClipRect, RestoreDC, RoundRect, SaveDC, SelectObject, SetBkMode,
        SetBrushOrgEx, SetStretchBltMode, SetTextColor, StretchDIBits, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH,
        DIB_RGB_COLORS, FF_DONTCARE, HALFTONE, HDC, HGDIOBJ, NULL_PEN, OUT_DEFAULT_PRECIS,
        PS_SOLID, SRCCOPY,
    },
};

#[allow(non_camel_case_types)]
type HPAINTBUFFER = *mut c_void;

#[repr(C)]
#[allow(non_snake_case)]
struct BpPaintParams {
    cbSize: u32,
    dwFlags: u32,
    prcExclude: *const RECT,
    pBlendFunction: *const c_void,
}

const BPBF_TOPDOWNDIB: u32 = 2;
static BUFFERED_PAINT_INIT: OnceLock<()> = OnceLock::new();
static GDIP_TOKEN: OnceLock<Option<usize>> = OnceLock::new();

#[link(name = "uxtheme")]
unsafe extern "system" {
    fn BufferedPaintInit() -> i32;
    fn BeginBufferedPaint(
        hdcTarget: HDC,
        prcTarget: *const RECT,
        dwFormat: u32,
        pPaintParams: *const BpPaintParams,
        phdc: *mut HDC,
    ) -> HPAINTBUFFER;
    fn EndBufferedPaint(hBufferedPaint: HPAINTBUFFER, fUpdateTarget: i32) -> i32;
}

const TRANSPARENT: i32 = 1;
const DT_LEFT: u32 = 0x0000;
const DT_CENTER: u32 = 0x0001;
const DT_RIGHT: u32 = 0x0002;
const DT_VCENTER: u32 = 0x0004;
const DT_BOTTOM: u32 = 0x0008;
const DT_WORDBREAK: u32 = 0x0010;
const DT_SINGLELINE: u32 = 0x0020;
const DT_CALCRECT: u32 = 0x0400;
const DT_NOPREFIX: u32 = 0x0800;
const DT_END_ELLIPSIS: u32 = 0x0000_8000;
const CLEARTYPE_QUALITY: u32 = 5;
const GDIP_SMOOTHING_MODE_ANTI_ALIAS: i32 = 4;
const GDIP_UNIT_PIXEL: i32 = 2;
const GDIP_FILL_MODE_ALTERNATE: i32 = 0;

#[repr(C)]
struct GdiplusStartupInput {
    gdiplus_version: u32,
    debug_event_callback: *const c_void,
    suppress_background_thread: i32,
    suppress_external_codecs: i32,
}

#[link(name = "gdiplus")]
unsafe extern "system" {
    fn GdiplusStartup(
        token: *mut usize,
        input: *const GdiplusStartupInput,
        output: *mut c_void,
    ) -> i32;
    fn GdipCreateFromHDC(hdc: HDC, graphics: *mut *mut c_void) -> i32;
    fn GdipDeleteGraphics(graphics: *mut c_void) -> i32;
    fn GdipSetSmoothingMode(graphics: *mut c_void, smoothing_mode: i32) -> i32;
    fn GdipCreateSolidFill(color: u32, brush: *mut *mut c_void) -> i32;
    fn GdipDeleteBrush(brush: *mut c_void) -> i32;
    fn GdipCreatePen1(color: u32, width: f32, unit: i32, pen: *mut *mut c_void) -> i32;
    fn GdipDeletePen(pen: *mut c_void) -> i32;
    fn GdipCreatePath(fill_mode: i32, path: *mut *mut c_void) -> i32;
    fn GdipDeletePath(path: *mut c_void) -> i32;
    fn GdipAddPathArcI(
        path: *mut c_void,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: f32,
        sweep_angle: f32,
    ) -> i32;
    fn GdipClosePathFigure(path: *mut c_void) -> i32;
    fn GdipFillPath(graphics: *mut c_void, brush: *mut c_void, path: *mut c_void) -> i32;
    fn GdipDrawPath(graphics: *mut c_void, pen: *mut c_void, path: *mut c_void) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsNoFlickerPaintStrategy {
    pub suppress_erase_background: bool,
    pub preferred_target: &'static str,
    pub fallback_target: &'static str,
    pub present_operation: &'static str,
}

pub const fn windows_no_flicker_paint_strategy() -> WindowsNoFlickerPaintStrategy {
    WindowsNoFlickerPaintStrategy {
        suppress_erase_background: true,
        preferred_target: "uxtheme_buffered_top_down_dib",
        fallback_target: "direct_gdi_hdc",
        present_operation: "EndBufferedPaint(update_target=true)",
    }
}

pub struct WindowsBufferedPaint {
    buffer: HPAINTBUFFER,
    dc: HDC,
    update_target: bool,
}

impl WindowsBufferedPaint {
    pub unsafe fn begin(target: HDC, rect: &RECT) -> Option<Self> {
        ensure_buffered_paint();
        let mut paint_dc: HDC = std::ptr::null_mut();
        let buffer = BeginBufferedPaint(target, rect, BPBF_TOPDOWNDIB, null(), &mut paint_dc);
        if buffer.is_null() || paint_dc.is_null() {
            None
        } else {
            Some(Self {
                buffer,
                dc: paint_dc,
                update_target: true,
            })
        }
    }

    pub const fn hdc(&self) -> HDC {
        self.dc
    }

    pub fn set_update_target(&mut self, update_target: bool) {
        self.update_target = update_target;
    }
}

impl Drop for WindowsBufferedPaint {
    fn drop(&mut self) {
        if !self.buffer.is_null() {
            unsafe {
                EndBufferedPaint(self.buffer, if self.update_target { 1 } else { 0 });
            }
        }
    }
}

unsafe fn ensure_buffered_paint() {
    BUFFERED_PAINT_INIT.get_or_init(|| {
        let _ = BufferedPaintInit();
    });
}

fn ensure_gdiplus_startup() -> Option<usize> {
    *GDIP_TOKEN.get_or_init(|| unsafe {
        let mut token = 0usize;
        let input = GdiplusStartupInput {
            gdiplus_version: 1,
            debug_event_callback: null(),
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        };
        if GdiplusStartup(&mut token, &input, std::ptr::null_mut()) == 0 {
            Some(token)
        } else {
            None
        }
    })
}

fn color_to_argb(color: Color) -> u32 {
    ((color.a as u32) << 24) | ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
}

unsafe fn build_gdiplus_round_path(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
) -> *mut c_void {
    let mut path = std::ptr::null_mut();
    if GdipCreatePath(GDIP_FILL_MODE_ALTERNATE, &mut path) != 0 || path.is_null() {
        return std::ptr::null_mut();
    }
    let w = right - left;
    let h = bottom - top;
    let r = radius.min(w / 2).min(h / 2).max(1);
    let d = r * 2;
    let ok = GdipAddPathArcI(path, left, top, d, d, 180.0, 90.0) == 0
        && GdipAddPathArcI(path, right - d, top, d, d, 270.0, 90.0) == 0
        && GdipAddPathArcI(path, right - d, bottom - d, d, d, 0.0, 90.0) == 0
        && GdipAddPathArcI(path, left, bottom - d, d, d, 90.0, 90.0) == 0
        && GdipClosePathFigure(path) == 0;
    if !ok {
        let _ = GdipDeletePath(path);
        return std::ptr::null_mut();
    }
    path
}

unsafe fn draw_round_rect_antialiased(
    hdc: HDC,
    rect: RECT,
    fill: Color,
    stroke: Option<Color>,
    radius: i32,
) -> bool {
    if ensure_gdiplus_startup().is_none() {
        return false;
    }
    if hdc.is_null() || rect.right <= rect.left || rect.bottom <= rect.top {
        return true;
    }
    let mut graphics = std::ptr::null_mut();
    if GdipCreateFromHDC(hdc, &mut graphics) != 0 || graphics.is_null() {
        return false;
    }
    let _ = GdipSetSmoothingMode(graphics, GDIP_SMOOTHING_MODE_ANTI_ALIAS);
    let path =
        build_gdiplus_round_path(rect.left, rect.top, rect.right, rect.bottom, radius.max(1));
    if path.is_null() {
        let _ = GdipDeleteGraphics(graphics);
        return false;
    }

    let mut ok = true;
    let mut brush = std::ptr::null_mut();
    if GdipCreateSolidFill(color_to_argb(fill), &mut brush) == 0 && !brush.is_null() {
        ok &= GdipFillPath(graphics, brush, path) == 0;
        let _ = GdipDeleteBrush(brush);
    } else {
        ok = false;
    }
    if let Some(stroke) = stroke.filter(|stroke| *stroke != fill) {
        let mut pen = std::ptr::null_mut();
        if GdipCreatePen1(color_to_argb(stroke), 1.0, GDIP_UNIT_PIXEL, &mut pen) == 0
            && !pen.is_null()
        {
            ok &= GdipDrawPath(graphics, pen, path) == 0;
            let _ = GdipDeletePen(pen);
        }
    }
    let _ = GdipDeletePath(path);
    let _ = GdipDeleteGraphics(graphics);
    ok
}

struct WindowsGdiOwnedObject {
    object: HGDIOBJ,
}

impl WindowsGdiOwnedObject {
    fn from_raw(object: HGDIOBJ) -> Option<Self> {
        if object.is_null() {
            None
        } else {
            Some(Self { object })
        }
    }

    fn solid_brush(color: Color) -> Option<Self> {
        Self::from_raw(unsafe { CreateSolidBrush(to_colorref(color)) as HGDIOBJ })
    }

    fn pen(color: Color) -> Option<Self> {
        Self::from_raw(unsafe { CreatePen(PS_SOLID, 1, to_colorref(color)) as HGDIOBJ })
    }

    fn font(style: &TextStyle) -> Option<Self> {
        let family = to_wide(&style.font_family);
        Self::from_raw(unsafe {
            CreateFontW(
                -style.size.round().max(1.0) as i32,
                0,
                0,
                0,
                font_weight(style.weight),
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                family.as_ptr(),
            ) as HGDIOBJ
        })
    }

    fn object(&self) -> HGDIOBJ {
        self.object
    }
}

impl Drop for WindowsGdiOwnedObject {
    fn drop(&mut self) {
        if !self.object.is_null() {
            unsafe {
                DeleteObject(self.object);
            }
        }
    }
}

struct WindowsGdiSelectedObject {
    dc: HDC,
    old: HGDIOBJ,
}

impl WindowsGdiSelectedObject {
    fn select(dc: HDC, object: HGDIOBJ) -> Option<Self> {
        if dc.is_null() || object.is_null() {
            return None;
        }
        let old = unsafe { SelectObject(dc, object) };
        if old.is_null() {
            None
        } else {
            Some(Self { dc, old })
        }
    }
}

impl Drop for WindowsGdiSelectedObject {
    fn drop(&mut self) {
        if !self.dc.is_null() && !self.old.is_null() {
            unsafe {
                SelectObject(self.dc, self.old);
            }
        }
    }
}

pub struct WindowsGdiRenderer {
    dc: HDC,
    clip_stack: Vec<i32>,
}

impl WindowsGdiRenderer {
    pub fn new(dc: HDC) -> Self {
        Self {
            dc,
            clip_stack: Vec::new(),
        }
    }

    pub fn hdc(&self) -> HDC {
        self.dc
    }

    fn has_dc(&self) -> bool {
        !self.dc.is_null()
    }
}

impl Drop for WindowsGdiRenderer {
    fn drop(&mut self) {
        while let Some(saved) = self.clip_stack.pop() {
            unsafe {
                RestoreDC(self.dc, saved);
            }
        }
    }
}

impl Renderer for WindowsGdiRenderer {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        if !self.has_dc() {
            return;
        }
        let rect = to_win_rect(rect);
        if let Some(brush) = WindowsGdiOwnedObject::solid_brush(color) {
            unsafe {
                FillRect(self.dc, &rect, brush.object() as _);
            }
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: i32) {
        if !self.has_dc() {
            return;
        }
        let mut rect = to_win_rect(rect);
        if let Some(brush) = WindowsGdiOwnedObject::solid_brush(color) {
            for _ in 0..width.max(1) {
                if rect.right <= rect.left || rect.bottom <= rect.top {
                    break;
                }
                unsafe {
                    FrameRect(self.dc, &rect, brush.object() as _);
                }
                rect.left += 1;
                rect.top += 1;
                rect.right -= 1;
                rect.bottom -= 1;
            }
        }
    }

    fn draw_text(&mut self, run: &TextRun, style: &TextStyle) {
        if !self.has_dc() || run.text.is_empty() {
            return;
        }
        with_font(self.dc, style, |dc| {
            let mut rect = to_win_rect(run.bounds);
            let text = to_wide(&run.text);
            unsafe {
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, to_colorref(style.color));
                DrawTextW(dc, text.as_ptr(), -1, &mut rect, text_flags(style, false));
            }
        });
    }

    fn push_clip(&mut self, rect: Rect) {
        if !self.has_dc() {
            return;
        }
        let saved = unsafe { SaveDC(self.dc) };
        if saved != 0 {
            let rect = to_win_rect(rect);
            unsafe {
                IntersectClipRect(self.dc, rect.left, rect.top, rect.right, rect.bottom);
            }
            self.clip_stack.push(saved);
        }
    }

    fn pop_clip(&mut self) {
        if !self.has_dc() {
            return;
        }
        if let Some(saved) = self.clip_stack.pop() {
            unsafe {
                RestoreDC(self.dc, saved);
            }
        }
    }
}

pub struct WindowsGdiTextLayout {
    dc: HDC,
}

impl WindowsGdiTextLayout {
    pub fn new(dc: HDC) -> Self {
        Self { dc }
    }
}

impl TextLayout for WindowsGdiTextLayout {
    fn measure(&self, text: &str, style: &TextStyle, max_width: i32) -> Size {
        if self.dc.is_null() || text.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        let mut measured = Size {
            width: 0,
            height: 0,
        };
        with_font(self.dc, style, |dc| {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: if max_width > 0 { max_width } else { 32_767 },
                bottom: 0,
            };
            let wide = to_wide(text);
            unsafe {
                DrawTextW(dc, wide.as_ptr(), -1, &mut rect, text_flags(style, true));
            }
            measured = Size {
                width: (rect.right - rect.left).max(0),
                height: (rect.bottom - rect.top).max(0),
            };
        });
        measured
    }

    fn layout_runs(&self, text: &str, _style: &TextStyle, bounds: Rect) -> Vec<TextRun> {
        if text.is_empty() {
            Vec::new()
        } else {
            vec![TextRun {
                text: text.to_string(),
                bounds,
            }]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsGdiPalette {
    pub primary_text: Color,
    pub secondary_text: Color,
    pub accent: Color,
    pub surface: Color,
    pub control: Color,
    pub danger: Color,
}

impl WindowsGdiPalette {
    pub const fn resolve(self, role: ColorRole) -> Color {
        match role {
            ColorRole::PrimaryText => self.primary_text,
            ColorRole::SecondaryText => self.secondary_text,
            ColorRole::Accent => self.accent,
            ColorRole::Surface => self.surface,
            ColorRole::Control => self.control,
            ColorRole::Danger => self.danger,
        }
    }

    pub const fn resolve_fill(self, fill: NativeDrawFill) -> Color {
        match fill {
            NativeDrawFill::Color(color) => color,
            NativeDrawFill::Role(role) => self.resolve(role),
            NativeDrawFill::RoleWithAlpha { role, alpha } => Color {
                a: alpha,
                ..self.resolve(role)
            },
        }
    }
}

impl Default for WindowsGdiPalette {
    fn default() -> Self {
        Self {
            primary_text: Color {
                r: 32,
                g: 32,
                b: 32,
                a: 255,
            },
            secondary_text: Color {
                r: 96,
                g: 96,
                b: 96,
                a: 255,
            },
            accent: Color {
                r: 0,
                g: 120,
                b: 212,
                a: 255,
            },
            surface: Color {
                r: 248,
                g: 248,
                b: 248,
                a: 255,
            },
            control: Color {
                r: 238,
                g: 238,
                b: 238,
                a: 255,
            },
            danger: Color {
                r: 196,
                g: 43,
                b: 28,
                a: 255,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowsGdiStyleResolver {
    pub font_family: String,
    pub palette: WindowsGdiPalette,
}

impl WindowsGdiStyleResolver {
    pub fn new(font_family: impl Into<String>, palette: WindowsGdiPalette) -> Self {
        Self {
            font_family: font_family.into(),
            palette,
        }
    }
}

impl Default for WindowsGdiStyleResolver {
    fn default() -> Self {
        Self::new("Segoe UI", WindowsGdiPalette::default())
    }
}

impl NativeStyleResolver for WindowsGdiStyleResolver {
    fn resolve_text_style(&self, style: SemanticTextStyle) -> TextStyle {
        let size = match style.role {
            crate::TextRole::Body => 14.0,
            crate::TextRole::Caption => 12.0,
            crate::TextRole::Title => 18.0,
            crate::TextRole::Button => 14.0,
            crate::TextRole::Icon => 16.0,
            crate::TextRole::Monospace => 13.0,
        };
        TextStyle {
            font_family: match style.role {
                crate::TextRole::Monospace => "Consolas".to_string(),
                crate::TextRole::Icon => "Segoe MDL2 Assets".to_string(),
                _ => self.font_family.clone(),
            },
            size,
            weight: style.weight,
            color: self.palette.resolve(style.color),
            horizontal_align: style.horizontal_align,
            vertical_align: style.vertical_align,
            wrap: style.wrap,
            ellipsis: style.ellipsis,
        }
    }
}

pub struct WindowsGdiDrawSink {
    renderer: WindowsGdiRenderer,
    palette: WindowsGdiPalette,
    style_resolver: WindowsGdiStyleResolver,
    operation_log: Vec<NativeDrawCommandOperation>,
}

impl WindowsGdiDrawSink {
    pub fn new(dc: HDC) -> Self {
        Self::with_palette(dc, WindowsGdiPalette::default())
    }

    pub fn with_palette(dc: HDC, palette: WindowsGdiPalette) -> Self {
        Self {
            renderer: WindowsGdiRenderer::new(dc),
            palette,
            style_resolver: WindowsGdiStyleResolver::new("Segoe UI", palette),
            operation_log: Vec::new(),
        }
    }

    pub fn hdc(&self) -> HDC {
        self.renderer.hdc()
    }

    pub fn operation_log(&self) -> &[NativeDrawCommandOperation] {
        &self.operation_log
    }

    pub fn draw_native_plan(&mut self, plan: &NativeDrawPlan) {
        self.draw_plan(plan);
    }

    fn draw_round_rect(
        &mut self,
        rect: Rect,
        fill: NativeDrawFill,
        stroke: Option<NativeDrawFill>,
        radius: i32,
    ) {
        if self.renderer.hdc().is_null() {
            return;
        }
        let rect = to_win_rect(rect);
        let fill = self.palette.resolve_fill(fill);
        let stroke_color = stroke.map(|stroke| self.palette.resolve_fill(stroke));
        if unsafe {
            draw_round_rect_antialiased(self.renderer.hdc(), rect, fill, stroke_color, radius)
        } {
            return;
        }
        let Some(fill_brush) = WindowsGdiOwnedObject::solid_brush(fill) else {
            return;
        };
        let stroke_pen = stroke_color.and_then(WindowsGdiOwnedObject::pen);
        let pen = stroke_pen
            .as_ref()
            .map(WindowsGdiOwnedObject::object)
            .unwrap_or_else(|| unsafe { GetStockObject(NULL_PEN) });
        let _selected_brush =
            WindowsGdiSelectedObject::select(self.renderer.hdc(), fill_brush.object());
        let _selected_pen = WindowsGdiSelectedObject::select(self.renderer.hdc(), pen);
        unsafe {
            RoundRect(
                self.renderer.hdc(),
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
                radius.max(1) * 2,
                radius.max(1) * 2,
            );
        }
    }

    fn draw_text_command(&mut self, command: &NativeDrawTextCommand) {
        let style = self.style_resolver.resolve_text_style(command.style);
        let run = TextRun {
            text: command.text.clone(),
            bounds: command.bounds,
        };
        self.renderer.draw_text(&run, &style);
    }

    fn draw_icon_command(&mut self, command: &NativeDrawIconCommand) {
        if self.renderer.hdc().is_null() {
            return;
        }
        let _mode = match command.color_mode {
            NativeIconColorMode::ThemeAware | NativeIconColorMode::Original => command.color_mode,
        };
        let Some(bytes) = command.icon.png_24_bytes() else {
            return;
        };
        let Some((width, height, bgra)) = decode_png_to_bgra(bytes) else {
            return;
        };
        stretch_top_down_32bpp(self.renderer.hdc(), command.bounds, width, height, &bgra);
    }
}

impl NativeDrawCommandSink for WindowsGdiDrawSink {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        self.operation_log.push(command.operation());
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                self.renderer
                    .fill_rect(*rect, self.palette.resolve_fill(*fill));
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => self
                .renderer
                .stroke_rect(*rect, self.palette.resolve_fill(*stroke), *width),
            NativeDrawCommand::RoundRect {
                rect,
                fill,
                stroke,
                radius,
            } => self.draw_round_rect(*rect, *fill, *stroke, *radius),
            NativeDrawCommand::RoundFill { rect, fill, radius } => {
                self.draw_round_rect(*rect, *fill, None, *radius);
            }
            NativeDrawCommand::Text(command) => self.draw_text_command(command),
            NativeDrawCommand::Icon(command) => self.draw_icon_command(command),
            NativeDrawCommand::PushClip { rect } => self.renderer.push_clip(*rect),
            NativeDrawCommand::PopClip => self.renderer.pop_clip(),
        }
    }
}

fn with_font<R>(dc: HDC, style: &TextStyle, f: impl FnOnce(HDC) -> R) -> R {
    let font = WindowsGdiOwnedObject::font(style);
    let _selected_font = font
        .as_ref()
        .and_then(|font| WindowsGdiSelectedObject::select(dc, font.object()));
    let result = f(dc);
    result
}

fn font_weight(weight: TextWeight) -> i32 {
    match weight {
        TextWeight::Regular => 400,
        TextWeight::Medium => 500,
        TextWeight::Bold => 700,
    }
}

fn text_flags(style: &TextStyle, measure: bool) -> u32 {
    let mut flags = DT_NOPREFIX;
    flags |= match style.horizontal_align {
        HorizontalAlign::Start => DT_LEFT,
        HorizontalAlign::Center => DT_CENTER,
        HorizontalAlign::End => DT_RIGHT,
    };
    flags |= match style.wrap {
        TextWrap::NoWrap => DT_SINGLELINE,
        TextWrap::Word => DT_WORDBREAK,
    };
    if style.wrap == TextWrap::NoWrap {
        flags |= match style.vertical_align {
            VerticalAlign::Start => 0,
            VerticalAlign::Center => DT_VCENTER,
            VerticalAlign::End => DT_BOTTOM,
        };
        if style.ellipsis {
            flags |= DT_END_ELLIPSIS;
        }
    }
    if measure {
        flags |= DT_CALCRECT;
    }
    flags
}

fn to_win_rect(rect: Rect) -> RECT {
    RECT {
        left: rect.x,
        top: rect.y,
        right: rect.x + rect.width.max(0),
        bottom: rect.y + rect.height.max(0),
    }
}

pub fn rect_from_win(rect: RECT) -> Rect {
    Rect {
        x: rect.left,
        y: rect.top,
        width: (rect.right - rect.left).max(0),
        height: (rect.bottom - rect.top).max(0),
    }
}

fn to_colorref(color: Color) -> u32 {
    (color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16)
}

pub fn color_from_colorref(color: u32) -> Color {
    Color {
        r: (color & 0xff) as u8,
        g: ((color >> 8) & 0xff) as u8,
        b: ((color >> 16) & 0xff) as u8,
        a: 255,
    }
}

fn stretch_top_down_32bpp(dc: HDC, rect: Rect, src_width: i32, src_height: i32, bgra_bits: &[u8]) {
    if dc.is_null()
        || rect.width <= 0
        || rect.height <= 0
        || src_width <= 0
        || src_height <= 0
        || bgra_bits.is_empty()
    {
        return;
    }
    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = src_width;
    info.bmiHeader.biHeight = -src_height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    unsafe {
        SetStretchBltMode(dc, HALFTONE);
        SetBrushOrgEx(dc, 0, 0, std::ptr::null_mut::<POINT>());
        StretchDIBits(
            dc,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            0,
            0,
            src_width,
            src_height,
            bgra_bits.as_ptr() as _,
            &info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

fn decode_png_to_bgra(bytes: &'static [u8]) -> Option<(i32, i32, Vec<u8>)> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    let frame = &buffer[..info.buffer_size()];
    let mut bgra = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);
    match info.color_type {
        png::ColorType::Rgba => {
            for chunk in frame.chunks_exact(4) {
                bgra.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
            }
        }
        png::ColorType::Rgb => {
            for chunk in frame.chunks_exact(3) {
                bgra.extend_from_slice(&[chunk[2], chunk[1], chunk[0], 255]);
            }
        }
        _ => return None,
    }
    Some((info.width as i32, info.height as i32, bgra))
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZsIcon;

    fn style() -> TextStyle {
        TextStyle::line(
            "Segoe UI",
            14.0,
            Color {
                r: 1,
                g: 2,
                b: 3,
                a: 255,
            },
        )
    }

    #[test]
    fn color_and_rect_conversion_match_gdi_contract() {
        assert_eq!(
            to_colorref(Color {
                r: 0x11,
                g: 0x22,
                b: 0x33,
                a: 0x44,
            }),
            0x0033_2211
        );
        let win = to_win_rect(Rect {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        });
        assert_eq!((win.left, win.top, win.right, win.bottom), (10, 20, 40, 60));
        assert_eq!(
            rect_from_win(win),
            Rect {
                x: 10,
                y: 20,
                width: 30,
                height: 40,
            }
        );
        assert_eq!(
            color_from_colorref(0x0033_2211),
            Color {
                r: 0x11,
                g: 0x22,
                b: 0x33,
                a: 255,
            }
        );
    }

    #[test]
    fn gdi_owned_object_rejects_null_handles_for_raii_safety() {
        assert!(WindowsGdiOwnedObject::from_raw(std::ptr::null_mut()).is_none());
        assert!(
            WindowsGdiSelectedObject::select(std::ptr::null_mut(), std::ptr::null_mut()).is_none()
        );
    }

    #[test]
    fn text_flags_follow_style_protocol() {
        let mut value = style();
        value.horizontal_align = HorizontalAlign::Center;
        value.vertical_align = VerticalAlign::Center;
        assert_eq!(
            text_flags(&value, false),
            DT_NOPREFIX | DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS
        );

        value.wrap = TextWrap::Word;
        value.ellipsis = false;
        assert_eq!(
            text_flags(&value, true),
            DT_NOPREFIX | DT_CENTER | DT_WORDBREAK | DT_CALCRECT
        );
    }

    #[test]
    fn icon_text_role_uses_mdl2_fallback_font() {
        let style = WindowsGdiStyleResolver::default().resolve_text_style(SemanticTextStyle {
            role: crate::TextRole::Icon,
            color: ColorRole::PrimaryText,
            weight: TextWeight::Regular,
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: false,
        });

        assert_eq!(style.font_family, "Segoe MDL2 Assets");
        assert_eq!(style.size, 16.0);
    }

    #[test]
    fn no_flicker_strategy_uses_buffered_paint_foundation() {
        let strategy = windows_no_flicker_paint_strategy();

        assert!(strategy.suppress_erase_background);
        assert_eq!(strategy.preferred_target, "uxtheme_buffered_top_down_dib");
        assert_eq!(strategy.fallback_target, "direct_gdi_hdc");
        assert_eq!(
            strategy.present_operation,
            "EndBufferedPaint(update_target=true)"
        );
    }

    #[test]
    fn windows_gdi_draw_sink_accepts_native_draw_plan_without_hdc() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 32,
            height: 32,
        };
        let plan = NativeDrawPlan::new([
            NativeDrawCommand::FillRect {
                rect,
                fill: NativeDrawFill::Role(ColorRole::Surface),
            },
            NativeDrawCommand::RoundRect {
                rect,
                fill: NativeDrawFill::Role(ColorRole::Control),
                stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                radius: 6,
            },
            NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                "hello",
                rect,
                SemanticTextStyle::body(),
            )),
            NativeDrawCommand::Icon(crate::NativeDrawIconCommand::new(
                ZsIcon::Search,
                rect,
                NativeIconColorMode::ThemeAware,
            )),
            NativeDrawCommand::PushClip { rect },
            NativeDrawCommand::PopClip,
        ]);
        let mut sink = WindowsGdiDrawSink::new(std::ptr::null_mut());

        sink.draw_native_plan(&plan);

        assert_eq!(
            sink.operation_log(),
            &[
                NativeDrawCommandOperation::FillRect,
                NativeDrawCommandOperation::RoundRect,
                NativeDrawCommandOperation::DrawText,
                NativeDrawCommandOperation::DrawIcon,
                NativeDrawCommandOperation::PushClip,
                NativeDrawCommandOperation::PopClip,
            ]
        );
    }

    #[test]
    fn png_icons_decode_to_bgra_for_gdi_stretch() {
        let (width, height, bgra) =
            decode_png_to_bgra(ZsIcon::Search.png_24_bytes().expect("search icon")).unwrap();
        assert_eq!((width, height), (24, 24));
        assert_eq!(bgra.len(), 24 * 24 * 4);
    }
}
