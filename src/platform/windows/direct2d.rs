//! Optional Direct2D rasterization into a window-owned premultiplied DIB.
use super::*;
use crate::NativeDrawCommand;
use windows::Win32::Graphics::{
    Direct2D::Common::*, Direct2D::*, Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
};
use windows_numerics::Vector2;
use windows_sys::Win32::Graphics::Gdi::*;

fn failure(error: windows_core::Error) -> ZsuiError {
    ZsuiError::host("windows_direct2d", error.to_string())
}

pub(super) struct DibSurface {
    pub(super) dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u8,
    width: u32,
    height: u32,
    length: usize,
}

impl DibSurface {
    fn new(width: u32, height: u32) -> ZsuiResult<Self> {
        let length = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4))
            .filter(|v| *v > 0 && *v <= 256 * 1024 * 1024)
            .ok_or_else(|| {
                ZsuiError::invalid_spec(
                    "window.surface",
                    "surface must be nonempty and at most 256 MiB",
                )
            })?;
        if width > i32::MAX as u32 || height > i32::MAX as u32 {
            return Err(ZsuiError::invalid_spec(
                "window.surface",
                "surface dimensions exceed Win32 limits",
            ));
        }
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            if dc.is_null() {
                return Err(ZsuiError::host(
                    "CreateCompatibleDC",
                    "could not allocate a surface DC",
                ));
            }
            let mut info: BITMAPINFO = zeroed();
            info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            info.bmiHeader.biWidth = width as i32;
            info.bmiHeader.biHeight = -(height as i32);
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB;
            let mut bits = null_mut();
            let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
            if bitmap.is_null() || bits.is_null() {
                DeleteDC(dc);
                return Err(ZsuiError::host(
                    "CreateDIBSection",
                    "could not allocate a surface bitmap",
                ));
            }
            let previous = SelectObject(dc, bitmap);
            if previous.is_null() || previous as isize == -1 {
                DeleteObject(bitmap);
                DeleteDC(dc);
                return Err(ZsuiError::host(
                    "SelectObject",
                    "could not select a surface bitmap",
                ));
            }
            Ok(Self {
                dc,
                bitmap,
                previous,
                bits: bits.cast(),
                width,
                height,
                length,
            })
        }
    }
    fn pixels_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.bits, self.length) }
    }
    fn pixels(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bits, self.length) }
    }
}

impl Drop for DibSurface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}

pub(super) struct Direct2dPainter {
    // COM resources are UI-thread owned and outlive the bitmap/DC they bind.
    target: ID2D1DCRenderTarget,
    factory: ID2D1Factory,
    surface: DibSurface,
}

fn color(color: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: f32::from(color.r) / 255.0,
        g: f32::from(color.g) / 255.0,
        b: f32::from(color.b) / 255.0,
        a: f32::from(color.a) / 255.0,
    }
}
fn rect(rect: crate::Rect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: rect.x as f32,
        top: rect.y as f32,
        right: rect.x.saturating_add(rect.width) as f32,
        bottom: rect.y.saturating_add(rect.height) as f32,
    }
}
fn fill(
    palette: WindowsGdiPalette,
    fill: crate::NativeDrawFill,
    high_contrast: bool,
) -> D2D1_COLOR_F {
    let value = match fill {
        crate::NativeDrawFill::Color(value) => value,
        crate::NativeDrawFill::Role(role) => palette.resolve(role),
        crate::NativeDrawFill::RoleWithAlpha { role, alpha } => {
            let mut value = palette.resolve(role);
            let alpha = if high_contrast {
                match alpha {
                    0 => 0,
                    1..=20 => 64,
                    21..=63 => 112,
                    _ => alpha,
                }
            } else {
                alpha
            };
            value.a = ((u16::from(value.a) * u16::from(alpha) + 127) / 255) as u8;
            value
        }
    };
    color(value)
}

impl Direct2dPainter {
    #[cfg(all(test, feature = "native-smoke"))]
    pub(super) fn proof_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.surface.width || y >= self.surface.height {
            return None;
        }
        let start = ((y * self.surface.width + x) * 4) as usize;
        Some(self.surface.pixels()[start..start + 4].try_into().ok()?)
    }
    pub(super) fn new(width: u32, height: u32, hardware: bool) -> ZsuiResult<Self> {
        unsafe {
            let factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).map_err(failure)?;
            let properties = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: if hardware {
                    D2D1_RENDER_TARGET_TYPE_HARDWARE
                } else {
                    D2D1_RENDER_TARGET_TYPE_SOFTWARE
                },
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let target = factory.CreateDCRenderTarget(&properties).map_err(failure)?;
            Ok(Self {
                factory,
                target,
                surface: DibSurface::new(width, height)?,
            })
        }
    }

    pub(super) fn render(
        &mut self,
        width: u32,
        height: u32,
        plan: Option<&NativeDrawPlan>,
        palette: WindowsGdiPalette,
        high_contrast: bool,
        dpi: crate::Dpi,
        resources: WindowsGdiResourceCache,
        transparent: bool,
    ) -> ZsuiResult<()> {
        if self.surface.width != width || self.surface.height != height {
            self.surface = DibSurface::new(width, height)?;
        }
        let target = &self.target;
        unsafe {
            target
                .BindDC(
                    windows::Win32::Graphics::Gdi::HDC(self.surface.dc),
                    &windows::Win32::Foundation::RECT {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    },
                )
                .map_err(failure)?;
            target.BeginDraw();
            target.Clear(Some(&color(if transparent {
                Color::rgba(0, 0, 0, 0)
            } else {
                palette.surface
            })));
        }
        let mut clips = 0usize;
        let result = (|| -> ZsuiResult<()> {
            let Some(plan) = plan else {
                return Ok(());
            };
            let brush = unsafe { target.CreateSolidColorBrush(&color(palette.primary_text), None) }
                .map_err(failure)?;
            for command in &plan.commands {
                unsafe {
                    match command {
                        NativeDrawCommand::FillRect {
                            rect: bounds,
                            fill: ink,
                        } => {
                            brush.SetColor(&fill(palette, *ink, high_contrast));
                            target.FillRectangle(&rect(*bounds), &brush);
                        }
                        NativeDrawCommand::StrokeRect {
                            rect: bounds,
                            stroke,
                            width,
                        } => {
                            brush.SetColor(&fill(palette, *stroke, high_contrast));
                            target.DrawRectangle(
                                &rect(*bounds),
                                &brush,
                                (*width).max(1) as f32,
                                None,
                            );
                        }
                        NativeDrawCommand::RoundFill {
                            rect: bounds,
                            fill: ink,
                            radius,
                        } => {
                            brush.SetColor(&fill(palette, *ink, high_contrast));
                            target.FillRoundedRectangle(
                                &D2D1_ROUNDED_RECT {
                                    rect: rect(*bounds),
                                    radiusX: (*radius).max(0) as f32,
                                    radiusY: (*radius).max(0) as f32,
                                },
                                &brush,
                            );
                        }
                        NativeDrawCommand::RoundRect {
                            rect: bounds,
                            fill: ink,
                            stroke,
                            radius,
                        } => {
                            let shape = D2D1_ROUNDED_RECT {
                                rect: rect(*bounds),
                                radiusX: (*radius).max(0) as f32,
                                radiusY: (*radius).max(0) as f32,
                            };
                            brush.SetColor(&fill(palette, *ink, high_contrast));
                            target.FillRoundedRectangle(&shape, &brush);
                            if let Some(stroke) = stroke {
                                brush.SetColor(&fill(palette, *stroke, high_contrast));
                                target.DrawRoundedRectangle(&shape, &brush, 1.0, None);
                            }
                        }
                        NativeDrawCommand::FillTriangle { points, fill: ink } => {
                            let path = self.factory.CreatePathGeometry().map_err(failure)?;
                            let sink = path.Open().map_err(failure)?;
                            sink.BeginFigure(
                                Vector2 {
                                    X: points[0].x as f32,
                                    Y: points[0].y as f32,
                                },
                                D2D1_FIGURE_BEGIN_FILLED,
                            );
                            for point in &points[1..] {
                                sink.AddLine(Vector2 {
                                    X: point.x as f32,
                                    Y: point.y as f32,
                                });
                            }
                            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                            sink.Close().map_err(failure)?;
                            brush.SetColor(&fill(palette, *ink, high_contrast));
                            target.FillGeometry(&path, &brush, None);
                        }
                        NativeDrawCommand::StrokeArc {
                            rect: bounds,
                            stroke,
                            width,
                            start_degrees,
                            sweep_degrees,
                        } => {
                            let path = self.factory.CreatePathGeometry().map_err(failure)?;
                            let sink = path.Open().map_err(failure)?;
                            let steps = (i32::from(*sweep_degrees).unsigned_abs() / 2).max(1);
                            for step in 0..=steps {
                                let angle = (f32::from(*start_degrees)
                                    + f32::from(*sweep_degrees) * step as f32 / steps as f32)
                                    .to_radians();
                                let point = Vector2 {
                                    X: bounds.x as f32
                                        + bounds.width as f32 * (0.5 + 0.5 * angle.cos()),
                                    Y: bounds.y as f32
                                        + bounds.height as f32 * (0.5 + 0.5 * angle.sin()),
                                };
                                if step == 0 {
                                    sink.BeginFigure(point, D2D1_FIGURE_BEGIN_HOLLOW);
                                } else {
                                    sink.AddLine(point);
                                }
                            }
                            sink.EndFigure(D2D1_FIGURE_END_OPEN);
                            sink.Close().map_err(failure)?;
                            brush.SetColor(&fill(palette, *stroke, high_contrast));
                            target.DrawGeometry(&path, &brush, (*width).max(1) as f32, None);
                        }
                        NativeDrawCommand::PushClip { rect: bounds } => {
                            target.PushAxisAlignedClip(&rect(*bounds), D2D1_ANTIALIAS_MODE_ALIASED);
                            clips += 1;
                        }
                        NativeDrawCommand::PopClip => {
                            if clips == 0 {
                                return Err(ZsuiError::invalid_spec(
                                    "draw_plan.clip",
                                    "unbalanced clip pop",
                                ));
                            }
                            target.PopAxisAlignedClip();
                            clips -= 1;
                        }
                        NativeDrawCommand::Image(image) => {
                            let bitmap = self.bitmap(
                                image.frame.width(),
                                image.frame.height(),
                                image.frame.premultiplied_bgra8(),
                            )?;
                            let interpolation = if image.interpolation
                                == crate::NativeImageInterpolation::Nearest
                            {
                                D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR
                            } else {
                                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR
                            };
                            target.DrawBitmap(
                                &bitmap,
                                Some(&rect(image.bounds)),
                                1.0,
                                interpolation,
                                Some(&rect(image.source)),
                            );
                        }
                        // Keep exact HarfRust/Swash geometry and native semantic icons.
                        // Two opaque mats recover coverage without guessing GDI alpha.
                        _ => {
                            let Some(bounds) = command.paint_bounds() else {
                                continue;
                            };
                            if bounds.width <= 0 || bounds.height <= 0 {
                                continue;
                            }
                            let pixels = text_or_icon_pixels(
                                command,
                                bounds,
                                palette,
                                high_contrast,
                                dpi,
                                resources.clone(),
                            )?;
                            let bitmap =
                                self.bitmap(bounds.width as u32, bounds.height as u32, &pixels)?;
                            target.DrawBitmap(
                                &bitmap,
                                Some(&rect(bounds)),
                                1.0,
                                D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                                None,
                            );
                        }
                    }
                }
            }
            if clips != 0 {
                return Err(ZsuiError::invalid_spec(
                    "draw_plan.clip",
                    "unbalanced clip push",
                ));
            }
            Ok(())
        })();
        unsafe {
            for _ in 0..clips {
                target.PopAxisAlignedClip();
            }
        }
        let end = unsafe { target.EndDraw(None, None) }.map_err(failure);
        result.and(end)
    }

    unsafe fn bitmap(&self, width: u32, height: u32, pixels: &[u8]) -> ZsuiResult<ID2D1Bitmap> {
        self.target
            .CreateBitmap(
                D2D_SIZE_U { width, height },
                Some(pixels.as_ptr().cast()),
                width * 4,
                &D2D1_BITMAP_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                },
            )
            .map_err(failure)
    }

    pub(super) unsafe fn present(
        &self,
        hwnd: HWND,
        target: HDC,
        transparent: bool,
    ) -> ZsuiResult<()> {
        if transparent {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetWindowRect, UpdateLayeredWindow, ULW_ALPHA,
            };
            let mut bounds: RECT = zeroed();
            GetWindowRect(hwnd, &mut bounds);
            let destination = POINT {
                x: bounds.left,
                y: bounds.top,
            };
            let size = windows_sys::Win32::Foundation::SIZE {
                cx: self.surface.width as i32,
                cy: self.surface.height as i32,
            };
            let origin = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            if UpdateLayeredWindow(
                hwnd,
                null_mut(),
                &destination,
                &size,
                self.surface.dc,
                &origin,
                0,
                &blend,
                ULW_ALPHA,
            ) == 0
            {
                return Err(ZsuiError::host(
                    "UpdateLayeredWindow",
                    format!("Windows error {}", GetLastError()),
                ));
            }
        } else if BitBlt(
            target,
            0,
            0,
            self.surface.width as i32,
            self.surface.height as i32,
            self.surface.dc,
            0,
            0,
            SRCCOPY,
        ) == 0
        {
            return Err(ZsuiError::host("Direct2D.present", "BitBlt failed"));
        }
        Ok(())
    }

    pub(super) unsafe fn copy_to_dc(&self, target: HDC) -> ZsuiResult<()> {
        if BitBlt(
            target,
            0,
            0,
            self.surface.width as i32,
            self.surface.height as i32,
            self.surface.dc,
            0,
            0,
            SRCCOPY,
        ) == 0
        {
            return Err(ZsuiError::host("Direct2D.capture", "BitBlt failed"));
        }
        Ok(())
    }
}

unsafe fn text_or_icon_pixels(
    command: &NativeDrawCommand,
    bounds: crate::Rect,
    palette: WindowsGdiPalette,
    high_contrast: bool,
    dpi: crate::Dpi,
    resources: WindowsGdiResourceCache,
) -> ZsuiResult<Vec<u8>> {
    let mut black = DibSurface::new(bounds.width as u32, bounds.height as u32)?;
    let mut white = DibSurface::new(bounds.width as u32, bounds.height as u32)?;
    black.pixels_mut().fill(0);
    white.pixels_mut().fill(255);
    for surface in [&black, &white] {
        SetViewportOrgEx(surface.dc, -bounds.x, -bounds.y, null_mut());
        let mut sink = WindowsGdiDrawSink::with_palette_contrast_dpi_and_resources(
            surface.dc,
            palette,
            high_contrast,
            dpi,
            resources.clone(),
        );
        crate::NativeDrawCommandSink::draw_command(&mut sink, command);
    }
    GdiFlush();
    let mut pixels = black.pixels().to_vec();
    for (out, white) in pixels
        .chunks_exact_mut(4)
        .zip(white.pixels().chunks_exact(4))
    {
        let transparency = (0..3)
            .map(|i| white[i].saturating_sub(out[i]))
            .max()
            .unwrap_or(255);
        let alpha = 255 - transparency;
        for channel in &mut out[..3] {
            *channel = (*channel).min(alpha);
        }
        out[3] = alpha;
    }
    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn software_surface_preserves_premultiplied_alpha_and_clips() {
        let mut painter = Direct2dPainter::new(24, 24, false).unwrap();
        let plan = NativeDrawPlan::new([
            NativeDrawCommand::PushClip {
                rect: crate::Rect {
                    x: 4,
                    y: 4,
                    width: 12,
                    height: 12,
                },
            },
            NativeDrawCommand::RoundFill {
                rect: crate::Rect {
                    x: 0,
                    y: 0,
                    width: 24,
                    height: 24,
                },
                fill: crate::NativeDrawFill::Color(Color::rgba(200, 80, 40, 128)),
                radius: 4,
            },
            NativeDrawCommand::PopClip,
        ]);
        painter
            .render(
                24,
                24,
                Some(&plan),
                WindowsGdiPalette::default(),
                false,
                crate::Dpi::standard(),
                WindowsGdiResourceCache::default(),
                true,
            )
            .unwrap();
        let pixels = painter.surface.pixels();
        assert_eq!(&pixels[..4], &[0, 0, 0, 0]);
        let center = &pixels[(8 * 24 + 8) * 4..(8 * 24 + 8) * 4 + 4];
        assert!((i32::from(center[3]) - 128).abs() <= 1, "alpha {center:?}");
        assert!(
            (i32::from(center[2]) - 100).abs() <= 1,
            "premultiplied red {center:?}"
        );
    }
}
