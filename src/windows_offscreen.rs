//! Memory-DC rendering for visual tests. No native window or input is created.
use super::*;

/// A top-down RGBA image rendered entirely off screen.
#[derive(Debug, Clone)]
pub struct WindowsOffscreenFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub icon_sources: Vec<&'static str>,
}

/// Renders an existing plan without creating, focusing or controlling a window.
/// `symbolic_icons` forces the built-in vector fallback for optical comparison.
pub fn render_windows_offscreen(
    plan: &NativeDrawPlan,
    width: u32,
    height: u32,
    dpi: Dpi,
    symbolic_icons: bool,
) -> crate::ZsuiResult<WindowsOffscreenFrame> {
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|bytes| *bytes > 0 && *bytes <= 256 * 1024 * 1024)
        .ok_or_else(|| {
            crate::ZsuiError::invalid_spec(
                "offscreen.size",
                "surface must be nonempty and at most 256 MiB",
            )
        })?;
    if width > i32::MAX as u32 || height > i32::MAX as u32 {
        return Err(crate::ZsuiError::invalid_spec(
            "offscreen.size",
            "surface dimension exceeds native limits",
        ));
    }
    struct Surface {
        dc: HDC,
        bitmap: HGDIOBJ,
        old: HGDIOBJ,
    }
    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                SelectObject(self.dc, self.old);
                DeleteObject(self.bitmap);
                DeleteDC(self.dc);
            }
        }
    }
    unsafe {
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        if dc.is_null() {
            return Err(crate::ZsuiError::host(
                "offscreen",
                "CreateCompatibleDC failed",
            ));
        }
        let mut info: BITMAPINFO = std::mem::zeroed();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width as i32;
        info.bmiHeader.biHeight = -(height as i32);
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            dc,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        if bitmap.is_null() || bits.is_null() {
            DeleteDC(dc);
            return Err(crate::ZsuiError::host(
                "offscreen",
                "CreateDIBSection failed",
            ));
        }
        let surface = Surface {
            dc,
            bitmap,
            old: SelectObject(dc, bitmap),
        };
        let palette = match plan.theme_mode {
            crate::ZsuiThemeMode::Dark => WindowsGdiPalette::from_theme(&crate::ZsuiTheme::dark()),
            crate::ZsuiThemeMode::HighContrast => {
                WindowsGdiPalette::from_theme(&crate::ZsuiTheme::high_contrast(true))
            }
            _ => WindowsGdiPalette::default(),
        };
        let mut sink = WindowsGdiDrawSink::with_palette_and_dpi(dc, palette, dpi);
        sink.high_contrast = plan.theme_mode == crate::ZsuiThemeMode::HighContrast;
        let mut sources = Vec::new();
        sink.renderer.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: width as i32,
                height: height as i32,
            },
            palette.surface,
        );
        for command in &plan.commands {
            if let NativeDrawCommand::Icon(icon) = command {
                if symbolic_icons {
                    symbolic_icons::draw(
                        dc,
                        icon,
                        icon.custom_color
                            .unwrap_or_else(|| palette.resolve(icon.color)),
                    );
                    sources.push("builtin-symbolic");
                } else {
                    sources.push(sink.draw_icon_command_source(icon));
                }
                continue;
            }
            #[cfg(feature = "custom-icons")]
            if matches!(command, NativeDrawCommand::VectorIcon(_)) {
                sources.push("svg-vector");
            }
            sink.draw_command(command);
        }
        GdiFlush();
        let mut rgba = std::slice::from_raw_parts(bits.cast::<u8>(), bytes).to_vec();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        drop(sink);
        drop(surface);
        Ok(WindowsOffscreenFrame {
            width,
            height,
            rgba,
            icon_sources: sources,
        })
    }
}

#[cfg(test)]
mod damage_tests {
    use super::*;
    #[test]
    fn changed_pixels_stay_inside_incremental_damage_for_insert_delete_overlap_and_clip() {
        use crate::render_protocol::{native_draw_plan_damage, NativeDrawPlanDamage};
        let fill = |rect, color| NativeDrawCommand::RoundFill {
            rect,
            fill: NativeDrawFill::Color(color),
            radius: 4,
        };
        let base = fill(
            Rect {
                x: 8,
                y: 8,
                width: 60,
                height: 40,
            },
            Color::rgb(40, 70, 120),
        );
        let overlay = fill(
            Rect {
                x: 22,
                y: 20,
                width: 22,
                height: 22,
            },
            Color::rgba(220, 50, 30, 128),
        );
        let before = NativeDrawPlan::new([base.clone()]);
        let after = NativeDrawPlan::new([base.clone(), overlay.clone()]);
        let clipped = NativeDrawPlan::new([
            base.clone(),
            NativeDrawCommand::PushClip {
                rect: Rect {
                    x: 28,
                    y: 25,
                    width: 10,
                    height: 10,
                },
            },
            overlay.clone(),
            NativeDrawCommand::PopClip,
        ]);
        let reordered = NativeDrawPlan::new([overlay, base]);
        for (before, after) in [
            (&before, &after),
            (&after, &before),
            (&after, &clipped),
            (&after, &reordered),
        ] {
            let damage = native_draw_plan_damage(before, after);
            let before = render_windows_offscreen(before, 96, 72, Dpi::standard(), false).unwrap();
            let after = render_windows_offscreen(after, 96, 72, Dpi::standard(), false).unwrap();
            let mut differences = 0;
            for (index, (left, right)) in before
                .rgba
                .chunks_exact(4)
                .zip(after.rgba.chunks_exact(4))
                .enumerate()
            {
                if left != right {
                    differences += 1;
                    let point = crate::Point {
                        x: (index % 96) as i32,
                        y: (index / 96) as i32,
                    };
                    assert!(
                        match damage {
                            NativeDrawPlanDamage::Full => true,
                            NativeDrawPlanDamage::Area(rect) => rect.contains(point),
                            NativeDrawPlanDamage::Unchanged => false,
                        },
                        "pixel {point:?} escaped {damage:?}"
                    );
                }
            }
            assert!(differences > 0);
        }
    }
}
