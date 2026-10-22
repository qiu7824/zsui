use super::*;
use crate::symbolic_icons::{draw_normalized_icon, SymbolicIconCanvas};

#[link(name = "gdiplus")]
unsafe extern "system" {
    fn GdipFlush(graphics: *mut c_void, intention: i32) -> i32;
    fn GdipStartPathFigure(path: *mut c_void) -> i32;
    fn GdipResetPath(path: *mut c_void) -> i32;
    fn GdipAddPathLine(path: *mut c_void, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
    fn GdipAddPathBezier(
        path: *mut c_void,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> i32;
    fn GdipAddPathRectangle(path: *mut c_void, x: f32, y: f32, width: f32, height: f32) -> i32;
    fn GdipSetPenWidth(pen: *mut c_void, width: f32) -> i32;
    fn GdipSetPenLineJoin(pen: *mut c_void, join: i32) -> i32;
    fn GdipSetPenStartCap(pen: *mut c_void, cap: i32) -> i32;
    fn GdipSetPenEndCap(pen: *mut c_void, cap: i32) -> i32;
    fn GdipSetClipRect(
        graphics: *mut c_void,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        mode: i32,
    ) -> i32;
}

pub(super) fn draw(dc: HDC, command: &NativeDrawIconCommand, color: Color) -> bool {
    if dc.is_null() || ensure_gdiplus_startup().is_none() {
        return false;
    }
    let bounds = command.fitted_bounds(Size {
        width: 16,
        height: 16,
    });
    if bounds.width <= 0 || bounds.height <= 0 {
        return false;
    }
    unsafe {
        GdiFlush();
    }
    let mut canvas = Canvas {
        graphics: std::ptr::null_mut(),
        path: std::ptr::null_mut(),
        brush: std::ptr::null_mut(),
        pen: std::ptr::null_mut(),
        bounds,
        current: None,
        scale: bounds.width as f32 / 16.0,
        ok: true,
    };
    unsafe {
        if GdipCreateFromHDC(dc, &mut canvas.graphics) != 0
            || GdipCreatePath(GDIP_FILL_MODE_ALTERNATE, &mut canvas.path) != 0
            || GdipCreateSolidFill(color_to_argb(color), &mut canvas.brush) != 0
            || GdipCreatePen1(
                color_to_argb(color),
                1.35 * canvas.scale,
                GDIP_UNIT_PIXEL,
                &mut canvas.pen,
            ) != 0
        {
            return false;
        }
        if canvas.graphics.is_null()
            || canvas.path.is_null()
            || canvas.brush.is_null()
            || canvas.pen.is_null()
        {
            return false;
        }
        canvas.ok &= GdipSetSmoothingMode(canvas.graphics, GDIP_SMOOTHING_MODE_ANTI_ALIAS) == 0;
        canvas.ok &= GdipSetClipRect(
            canvas.graphics,
            bounds.x as f32,
            bounds.y as f32,
            bounds.width as f32,
            bounds.height as f32,
            1,
        ) == 0;
        canvas.ok &= GdipSetPenLineJoin(canvas.pen, 2) == 0;
        canvas.ok &= GdipSetPenStartCap(canvas.pen, 2) == 0;
        canvas.ok &= GdipSetPenEndCap(canvas.pen, 2) == 0;
    }
    draw_normalized_icon(&mut canvas, command.icon);
    unsafe {
        canvas.ok &= GdipFlush(canvas.graphics, 1) == 0;
    }
    canvas.ok
}

struct Canvas {
    graphics: *mut c_void,
    path: *mut c_void,
    brush: *mut c_void,
    pen: *mut c_void,
    bounds: Rect,
    current: Option<(f32, f32)>,
    scale: f32,
    ok: bool,
}

impl Canvas {
    fn point(&self, x: f64, y: f64) -> (f32, f32) {
        (
            self.bounds.x as f32 + x as f32 * self.scale,
            self.bounds.y as f32 + y as f32 * self.scale,
        )
    }
    fn reset(&mut self) {
        unsafe {
            self.ok &= GdipResetPath(self.path) == 0;
        }
        self.current = None;
    }
}

impl SymbolicIconCanvas for Canvas {
    fn new_sub_path(&mut self) {
        unsafe {
            self.ok &= GdipStartPathFigure(self.path) == 0;
        }
        self.current = None;
    }
    fn move_to(&mut self, x: f64, y: f64) {
        self.new_sub_path();
        self.current = Some(self.point(x, y));
    }
    fn line_to(&mut self, x: f64, y: f64) {
        let end = self.point(x, y);
        if let Some(start) = self.current {
            unsafe {
                self.ok &= GdipAddPathLine(self.path, start.0, start.1, end.0, end.1) == 0;
            }
        }
        self.current = Some(end);
    }
    fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        let a = self.point(x1, y1);
        let b = self.point(x2, y2);
        let end = self.point(x, y);
        if let Some(start) = self.current {
            unsafe {
                self.ok &= GdipAddPathBezier(
                    self.path, start.0, start.1, a.0, a.1, b.0, b.1, end.0, end.1,
                ) == 0;
            }
        }
        self.current = Some(end);
    }
    fn arc(&mut self, x: f64, y: f64, radius: f64, start: f64, end: f64) {
        crate::symbolic_icons::trace_arc(self, x, y, radius, start, end);
    }
    fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let p = self.point(x, y);
        unsafe {
            self.ok &= GdipAddPathRectangle(
                self.path,
                p.0,
                p.1,
                width as f32 * self.scale,
                height as f32 * self.scale,
            ) == 0;
        }
        self.current = None;
    }
    fn set_line_width(&mut self, width: f64) {
        unsafe {
            self.ok &= GdipSetPenWidth(self.pen, width as f32 * self.scale) == 0;
        }
    }
    fn close_path(&mut self) {
        unsafe {
            self.ok &= GdipClosePathFigure(self.path) == 0;
        }
    }
    fn stroke(&mut self) {
        unsafe {
            self.ok &= GdipDrawPath(self.graphics, self.pen, self.path) == 0;
        }
        self.reset();
    }
    fn fill(&mut self) {
        unsafe {
            self.ok &= GdipFillPath(self.graphics, self.brush, self.path) == 0;
        }
        self.reset();
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            for (operation, status) in [
                (
                    "icon path",
                    if self.path.is_null() {
                        0
                    } else {
                        GdipDeletePath(self.path)
                    },
                ),
                (
                    "icon pen",
                    if self.pen.is_null() {
                        0
                    } else {
                        GdipDeletePen(self.pen)
                    },
                ),
                (
                    "icon brush",
                    if self.brush.is_null() {
                        0
                    } else {
                        GdipDeleteBrush(self.brush)
                    },
                ),
                (
                    "icon graphics",
                    if self.graphics.is_null() {
                        0
                    } else {
                        GdipDeleteGraphics(self.graphics)
                    },
                ),
            ] {
                if status != 0 {
                    {
                        use std::io::Write as _;
                        let _ = writeln!(
                            std::io::stderr(),
                            "ZSUI {operation} cleanup failed: GDI+ status {status}"
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Graphics::Gdi::CreateDIBSection;

    struct Surface {
        dc: HDC,
        bitmap: HGDIOBJ,
        old: HGDIOBJ,
        bits: *mut u8,
    }
    impl Surface {
        fn new() -> Self {
            unsafe {
                let dc = CreateCompatibleDC(std::ptr::null_mut());
                assert!(!dc.is_null());
                let mut info: BITMAPINFO = std::mem::zeroed();
                info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
                info.bmiHeader.biWidth = 48;
                info.bmiHeader.biHeight = -48;
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
                assert!(!bitmap.is_null() && !bits.is_null());
                let old = SelectObject(dc, bitmap as _);
                Self {
                    dc,
                    bitmap: bitmap as _,
                    old,
                    bits: bits.cast(),
                }
            }
        }
        fn bytes(&mut self) -> &mut [u8] {
            unsafe { std::slice::from_raw_parts_mut(self.bits, 48 * 48 * 4) }
        }
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

    #[test]
    fn all_semantic_icons_render_on_gdi_without_system_fonts() {
        let mut surface = Surface::new();
        let mut atlas = vec![255_u8; 9 * 48 * 5 * 48 * 4];
        for (index, icon) in crate::ZsIcon::ALL.into_iter().enumerate() {
            surface.bytes().fill(255);
            let command = NativeDrawIconCommand::new(
                icon,
                Rect {
                    x: 8,
                    y: 8,
                    width: 32,
                    height: 32,
                },
                NativeIconColorMode::ThemeAware,
            );
            assert!(
                draw(surface.dc, &command, Color::rgb(30, 80, 160)),
                "{icon:?}"
            );
            unsafe {
                GdiFlush();
            }
            let mut changed = 0;
            for (pixel_index, pixel) in surface.bytes().chunks_exact(4).enumerate() {
                let x = pixel_index % 48;
                let y = pixel_index / 48;
                if pixel[..3] != [255, 255, 255] {
                    changed += 1;
                    assert!(
                        (8..40).contains(&x) && (8..40).contains(&y),
                        "{icon:?} escaped its glyph bounds"
                    );
                }
                let target = (((index / 9) * 48 + y) * (9 * 48) + (index % 9) * 48 + x) * 4;
                atlas[target..target + 4].copy_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
            }
            assert!(changed > 5, "{icon:?} produced an empty fallback");
        }
        if let Some(directory) = std::env::var_os("ZSUI_ICON_PROOF_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let file = std::fs::File::create(directory.join("windows-symbolic-icons.png")).unwrap();
            let mut encoder = png::Encoder::new(file, 9 * 48, 5 * 48);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&atlas)
                .unwrap();
        }
    }
}
