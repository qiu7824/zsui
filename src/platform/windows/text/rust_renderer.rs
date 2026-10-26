use std::{ffi::c_void, fmt, mem::size_of, ptr::null_mut};

use windows_sys::Win32::{
    Foundation::{BOOL, HANDLE, RECT},
    Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetClipBox,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
        NULLREGION, RGBQUAD, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
        SystemParametersInfoW, FE_FONTSMOOTHINGCLEARTYPE, FE_FONTSMOOTHINGORIENTATIONBGR,
        FE_FONTSMOOTHINGORIENTATIONRGB, SPI_GETFONTSMOOTHING, SPI_GETFONTSMOOTHINGORIENTATION,
        SPI_GETFONTSMOOTHINGTYPE,
    },
};

use crate::{
    rust_text_renderer::{ZsRustTextEngine, ZsTextLineMetricPolicy, ZsTextRasterProfile},
    Rect, Size, TextRun, TextStyle, ZsTextPixelRect,
};

const TEXT_TILE_WIDTH: u32 = 512;
const TEXT_TILE_HEIGHT: u32 = 64;

/// Per-window compositor state. Ordinary painting borrows the final surface;
/// arbitrary native DCs use a reusable scratch tile capped at 128 KiB.
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
    pub(crate) direct_draws: u64,
    pub(crate) bitmap_bytes: usize,
    pub(crate) max_bitmap_bytes: usize,
}

impl Default for WindowsRustTextState {
    fn default() -> Self {
        Self {
            engine: None,
            bitmap: None,
            profile: windows_system_text_raster_profile(),
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
    #[cfg(any(feature = "windows-win32", test))]
    pub(crate) fn release_idle_memory(&mut self) {
        self.bitmap = None;
        self.engine = None;
    }

    #[cfg(test)]
    pub(crate) fn has_retained_resources(&self) -> bool {
        self.engine.is_some() || self.bitmap.is_some()
    }

    pub(crate) fn draw_text_buffered(
        &mut self,
        surface: &mut crate::windows_gdi_renderer::WindowsBufferedTextSurface<'_>,
        clip: ZsTextPixelRect,
        run: &TextRun,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> bool {
        if run.text.is_empty() || run.bounds.width <= 0 || run.bounds.height <= 0 {
            return false;
        }
        self.stats.draw_attempts = self.stats.draw_attempts.saturating_add(1);
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
        if style.ellipsis && layout.overflows_horizontally() {
            self.stats.fallback_count = self.stats.fallback_count.saturating_add(1);
            return false;
        }
        let profile = windows_rust_text_raster_profile(style.semantic_role, self.profile);
        if surface
            .composite(engine, &layout, run.bounds, clip, style.color, profile)
            .is_ok()
        {
            self.stats.draw_successes = self.stats.draw_successes.saturating_add(1);
            self.stats.direct_draws = self.stats.direct_draws.saturating_add(1);
            true
        } else {
            self.stats.fallback_count = self.stats.fallback_count.saturating_add(1);
            false
        }
    }

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
        let Some(visible) = visible_text_rect(destination, run.bounds)? else {
            return Ok(());
        };
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

        let raster_profile = windows_rust_text_raster_profile(style.semantic_role, self.profile);
        let start_x = visible.x.saturating_sub(run.bounds.x) as u32;
        let start_y = visible.y.saturating_sub(run.bounds.y) as u32;
        let end_x = start_x + visible.width as u32;
        let end_y = start_y + visible.height as u32;
        for y in (start_y..end_y).step_by(TEXT_TILE_HEIGHT as usize) {
            for x in (start_x..end_x).step_by(TEXT_TILE_WIDTH as usize) {
                let width = (end_x - x).min(TEXT_TILE_WIDTH);
                let height = (end_y - y).min(TEXT_TILE_HEIGHT);
                let bitmap = ensure_bitmap(&mut self.bitmap, destination, width, height)?;
                self.stats.max_bitmap_bytes = self
                    .stats
                    .max_bitmap_bytes
                    .max(bitmap.stride * bitmap.height as usize);
                let left = run.bounds.x.saturating_add(x as i32);
                let top = run.bounds.y.saturating_add(y as i32);
                unsafe {
                    GdiFlush();
                    if BitBlt(
                        bitmap.dc,
                        0,
                        0,
                        width as i32,
                        height as i32,
                        destination,
                        left,
                        top,
                        SRCCOPY,
                    ) == 0
                    {
                        return Err(
                            "could not copy the native background into the Rust text tile".into(),
                        );
                    }
                }
                let stride = bitmap.stride;
                let pixels = bitmap.pixels_mut()?;
                engine.composite_bgra_clipped(
                    &layout,
                    pixels,
                    width,
                    height,
                    stride,
                    -(x as i32),
                    -(y as i32),
                    ZsTextPixelRect::full(width, height),
                    style.color,
                    raster_profile,
                )?;
                unsafe {
                    GdiFlush();
                    if BitBlt(
                        destination,
                        left,
                        top,
                        width as i32,
                        height as i32,
                        bitmap.dc,
                        0,
                        0,
                        SRCCOPY,
                    ) == 0
                    {
                        return Err("could not present the Rust text tile".into());
                    }
                }
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

    #[cfg(feature = "text-input-core")]
    pub(crate) fn shape_line(
        &mut self,
        text: &str,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
        let was_uninitialized = self.engine.is_none();
        let engine = self.engine.get_or_insert_with(|| {
            ZsRustTextEngine::new()
                .with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite)
        });
        if was_uninitialized {
            self.stats.engine_initializations = self.stats.engine_initializations.saturating_add(1);
        }
        let layout = engine.layout(text, style, 0, 0, dpi_scale);
        crate::rust_text_renderer::native_shaped_line_from_layout(
            text,
            &layout,
            style.size * dpi_scale,
        )
    }

    pub(crate) fn stats(&self) -> WindowsRustTextStats {
        let mut stats = self.stats;
        stats.bitmap_bytes = self
            .bitmap
            .as_ref()
            .map_or(0, |bitmap| bitmap.stride * bitmap.height as usize);
        stats
    }
}

fn visible_text_rect(dc: HDC, bounds: Rect) -> Result<Option<Rect>, String> {
    let mut clip = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    match unsafe { GetClipBox(dc, &mut clip) } {
        0 => return Err("could not query the native text clip".into()),
        NULLREGION => return Ok(None),
        _ => {}
    }
    let left = bounds.x.max(clip.left);
    let top = bounds.y.max(clip.top);
    let right = bounds.x.saturating_add(bounds.width).min(clip.right);
    let bottom = bounds.y.saturating_add(bounds.height).min(clip.bottom);
    Ok((left < right && top < bottom).then_some(Rect {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    }))
}

const fn windows_rust_text_raster_profile(
    role: Option<crate::TextRole>,
    text_profile: ZsTextRasterProfile,
) -> ZsTextRasterProfile {
    if matches!(role, Some(crate::TextRole::Icon)) {
        ZsTextRasterProfile::grayscale()
    } else {
        text_profile
    }
}

/// Follows the user's font-smoothing choice, as GDI and DirectWrite do.
/// Settings that cannot be read keep the ClearType RGB default.
fn windows_system_text_raster_profile() -> ZsTextRasterProfile {
    let mut smoothing: BOOL = 1;
    let mut smoothing_type = FE_FONTSMOOTHINGCLEARTYPE;
    let mut orientation = FE_FONTSMOOTHINGORIENTATIONRGB;
    unsafe {
        SystemParametersInfoW(
            SPI_GETFONTSMOOTHING,
            0,
            (&mut smoothing as *mut BOOL).cast(),
            0,
        );
        SystemParametersInfoW(
            SPI_GETFONTSMOOTHINGTYPE,
            0,
            (&mut smoothing_type as *mut u32).cast(),
            0,
        );
        SystemParametersInfoW(
            SPI_GETFONTSMOOTHINGORIENTATION,
            0,
            (&mut orientation as *mut u32).cast(),
            0,
        );
    }
    windows_text_raster_profile(smoothing != 0, smoothing_type, orientation)
}

/// ClearType selects the panel's stripe order; standard or disabled smoothing
/// keeps one coverage value per pixel, which also suits remote sessions.
const fn windows_text_raster_profile(
    smoothing: bool,
    smoothing_type: u32,
    orientation: u32,
) -> ZsTextRasterProfile {
    if !smoothing || smoothing_type != FE_FONTSMOOTHINGCLEARTYPE {
        ZsTextRasterProfile::grayscale()
    } else if orientation == FE_FONTSMOOTHINGORIENTATIONBGR {
        ZsTextRasterProfile::subpixel_bgr()
    } else {
        ZsTextRasterProfile::subpixel_rgb()
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
    if width == 0 || height == 0 || width > TEXT_TILE_WIDTH || height > TEXT_TILE_HEIGHT {
        return Err("Rust text scratch tile exceeds its fixed capacity".into());
    }
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

#[cfg(test)]
#[path = "rust_renderer_memory_tests.rs"]
mod memory_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_text_renderer::ZsTextRasterMode;

    #[test]
    fn semantic_icons_use_grayscale_without_changing_ui_text_subpixel_mode() {
        let text = ZsTextRasterProfile::subpixel_rgb();

        assert_eq!(
            windows_rust_text_raster_profile(Some(crate::TextRole::Icon), text).mode,
            ZsTextRasterMode::Grayscale
        );
        assert_eq!(
            windows_rust_text_raster_profile(Some(crate::TextRole::Body), text).mode,
            ZsTextRasterMode::SubpixelRgb
        );
    }

    #[test]
    fn text_raster_profile_follows_system_font_smoothing() {
        use windows_sys::Win32::UI::WindowsAndMessaging::FE_FONTSMOOTHINGSTANDARD;

        let mode = |smoothing, smoothing_type, orientation| {
            windows_text_raster_profile(smoothing, smoothing_type, orientation).mode
        };
        assert_eq!(
            mode(
                true,
                FE_FONTSMOOTHINGCLEARTYPE,
                FE_FONTSMOOTHINGORIENTATIONRGB
            ),
            ZsTextRasterMode::SubpixelRgb
        );
        assert_eq!(
            mode(
                true,
                FE_FONTSMOOTHINGCLEARTYPE,
                FE_FONTSMOOTHINGORIENTATIONBGR
            ),
            ZsTextRasterMode::SubpixelBgr
        );
        assert_eq!(
            mode(
                true,
                FE_FONTSMOOTHINGSTANDARD,
                FE_FONTSMOOTHINGORIENTATIONRGB
            ),
            ZsTextRasterMode::Grayscale
        );
        assert_eq!(
            mode(
                false,
                FE_FONTSMOOTHINGCLEARTYPE,
                FE_FONTSMOOTHINGORIENTATIONRGB
            ),
            ZsTextRasterMode::Grayscale
        );
    }
}
