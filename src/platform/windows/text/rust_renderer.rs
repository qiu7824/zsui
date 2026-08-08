use std::{ffi::c_void, fmt, mem::size_of, ptr::null_mut};

use windows_sys::Win32::{
    Foundation::HANDLE,
    Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
        RGBQUAD, SRCCOPY,
    },
};

use crate::{
    rust_text_renderer::{ZsRustTextEngine, ZsTextLineMetricPolicy, ZsTextRasterProfile},
    Size, TextRun, TextStyle,
};

/// Per-window compositor state. Font discovery and glyph allocations are lazy,
/// and the top-down DIB grows only to the largest text box seen by this window.
pub(crate) struct WindowsRustTextState {
    engine: Option<ZsRustTextEngine>,
    bitmap: Option<WindowsRustTextBitmap>,
    profile: ZsTextRasterProfile,
    stats: WindowsRustTextStats,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct WindowsRustTextStats {
    pub(crate) engine_initializations: u64,
    pub(crate) draw_attempts: u64,
    pub(crate) draw_successes: u64,
    pub(crate) fallback_count: u64,
}

impl Default for WindowsRustTextState {
    fn default() -> Self {
        Self {
            engine: None,
            bitmap: None,
            profile: ZsTextRasterProfile::subpixel_rgb(),
            stats: WindowsRustTextStats::default(),
        }
    }
}

impl fmt::Debug for WindowsRustTextState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WindowsRustTextState")
            .field("initialized", &self.engine.is_some())
            .field(
                "bitmap_size",
                &self
                    .bitmap
                    .as_ref()
                    .map(|bitmap| (bitmap.width, bitmap.height)),
            )
            .field("profile", &self.profile)
            .field("stats", &self.stats())
            .finish()
    }
}

impl WindowsRustTextState {
    pub(crate) fn draw_text(
        &mut self,
        destination: HDC,
        run: &TextRun,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> bool {
        self.stats.draw_attempts = self.stats.draw_attempts.saturating_add(1);
        if destination.is_null()
            || run.text.is_empty()
            || run.bounds.width <= 0
            || run.bounds.height <= 0
        {
            self.stats.fallback_count = self.stats.fallback_count.saturating_add(1);
            return false;
        }
        let drawn = self
            .try_draw_text(destination, run, style, dpi_scale)
            .is_ok();
        if drawn {
            self.stats.draw_successes = self.stats.draw_successes.saturating_add(1);
        } else {
            self.stats.fallback_count = self.stats.fallback_count.saturating_add(1);
        }
        drawn
    }

    fn try_draw_text(
        &mut self,
        destination: HDC,
        run: &TextRun,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> Result<(), String> {
        let was_uninitialized = self.engine.is_none();
        let engine = self.engine.get_or_insert_with(|| {
            ZsRustTextEngine::new()
                .with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite)
        });
        if was_uninitialized {
            self.stats.engine_initializations = self.stats.engine_initializations.saturating_add(1);
        }
        let layout = engine.layout(
            &run.text,
            style,
            run.bounds.width,
            run.bounds.height,
            dpi_scale,
        );
        // Character-safe ellipsis is not approximated. Until the Rust layout
        // owns that operation, the host falls through to DirectWrite/GDI.
        if style.ellipsis && layout.overflows_horizontally() {
            return Err("Rust text ellipsis is not implemented".into());
        }

        let width = run.bounds.width as u32;
        let height = run.bounds.height as u32;
        let bitmap = ensure_bitmap(&mut self.bitmap, destination, width, height)?;
        unsafe {
            GdiFlush();
            if BitBlt(
                bitmap.dc,
                0,
                0,
                width as i32,
                height as i32,
                destination,
                run.bounds.x,
                run.bounds.y,
                SRCCOPY,
            ) == 0
            {
                return Err("could not copy the native background into the Rust text DIB".into());
            }
        }
        let bitmap_stride = bitmap.stride;
        let pixels = bitmap.pixels_mut()?;
        engine.composite_bgra(
            &layout,
            pixels,
            width,
            height,
            bitmap_stride,
            style.color,
            self.profile,
        )?;
        unsafe {
            GdiFlush();
            if BitBlt(
                destination,
                run.bounds.x,
                run.bounds.y,
                width as i32,
                height as i32,
                bitmap.dc,
                0,
                0,
                SRCCOPY,
            ) == 0
            {
                return Err("could not present the Rust text DIB".into());
            }
        }
        Ok(())
    }

    pub(crate) fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: i32,
        dpi_scale: f32,
    ) -> Option<Size> {
        let was_uninitialized = self.engine.is_none();
        let engine = self.engine.get_or_insert_with(|| {
            ZsRustTextEngine::new()
                .with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite)
        });
        if was_uninitialized {
            self.stats.engine_initializations = self.stats.engine_initializations.saturating_add(1);
        }
        Some(engine.measure(text, style, max_width, dpi_scale))
    }

    pub(crate) const fn stats(&self) -> WindowsRustTextStats {
        self.stats
    }
}

struct WindowsRustTextBitmap {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u8,
    width: u32,
    height: u32,
    stride: usize,
}

// SAFETY: the memory DC, selected bitmap and DIB pointer are one owned unit.
// WindowsRustTextBitmap is reachable only through the renderer resource-cache
// mutex, so no two threads can select, draw into or destroy the GDI objects at
// the same time. The DIB pointer is never exposed beyond a borrow of `self`.
unsafe impl Send for WindowsRustTextBitmap {}

impl WindowsRustTextBitmap {
    fn new(reference: HDC, width: u32, height: u32) -> Result<Self, String> {
        let width_i32 = i32::try_from(width).map_err(|_| "text DIB width is too large")?;
        let height_i32 = i32::try_from(height).map_err(|_| "text DIB height is too large")?;
        let stride = (width as usize)
            .checked_mul(4)
            .ok_or("text DIB stride overflow")?;
        let image_bytes = stride
            .checked_mul(height as usize)
            .ok_or("text DIB byte size overflow")?;
        let image_bytes = u32::try_from(image_bytes).map_err(|_| "text DIB exceeds 4 GiB")?;
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width_i32,
                biHeight: -height_i32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: image_bytes,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        let dc = unsafe { CreateCompatibleDC(reference) };
        if dc.is_null() {
            return Err("could not create the Rust text memory DC".into());
        }
        let mut bits = null_mut::<c_void>();
        let bitmap = unsafe {
            CreateDIBSection(
                reference,
                &info,
                DIB_RGB_COLORS,
                &mut bits,
                null_mut::<c_void>() as HANDLE,
                0,
            )
        };
        if bitmap.is_null() || bits.is_null() {
            unsafe {
                DeleteDC(dc);
            }
            return Err("could not allocate the Rust text DIB".into());
        }
        let previous = unsafe { SelectObject(dc, bitmap as HGDIOBJ) };
        if previous.is_null() {
            unsafe {
                DeleteObject(bitmap as HGDIOBJ);
                DeleteDC(dc);
            }
            return Err("could not select the Rust text DIB".into());
        }
        Ok(Self {
            dc,
            bitmap,
            previous,
            bits: bits.cast(),
            width,
            height,
            stride,
        })
    }

    fn pixels_mut(&mut self) -> Result<&mut [u8], String> {
        let len = self
            .stride
            .checked_mul(self.height as usize)
            .ok_or("text DIB slice length overflow")?;
        if self.bits.is_null() {
            return Err("text DIB has no pixel pointer".into());
        }
        // SAFETY: CreateDIBSection owns at least stride * height bytes until
        // this object is dropped, and the mutable borrow is tied to self.
        Ok(unsafe { std::slice::from_raw_parts_mut(self.bits, len) })
    }
}

impl Drop for WindowsRustTextBitmap {
    fn drop(&mut self) {
        unsafe {
            if !self.dc.is_null() && !self.previous.is_null() {
                SelectObject(self.dc, self.previous);
            }
            if !self.bitmap.is_null() {
                DeleteObject(self.bitmap as HGDIOBJ);
            }
            if !self.dc.is_null() {
                DeleteDC(self.dc);
            }
        }
    }
}

fn ensure_bitmap<'a>(
    bitmap: &'a mut Option<WindowsRustTextBitmap>,
    reference: HDC,
    width: u32,
    height: u32,
) -> Result<&'a mut WindowsRustTextBitmap, String> {
    let required_width = bitmap
        .as_ref()
        .map_or(width, |value| value.width.max(width));
    let required_height = bitmap
        .as_ref()
        .map_or(height, |value| value.height.max(height));
    let replace = bitmap
        .as_ref()
        .is_none_or(|value| value.width < required_width || value.height < required_height);
    if replace {
        *bitmap = Some(WindowsRustTextBitmap::new(
            reference,
            required_width,
            required_height,
        )?);
    }
    bitmap
        .as_mut()
        .ok_or_else(|| "Rust text DIB was not initialized".into())
}
