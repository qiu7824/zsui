use std::{
    ffi::c_void,
    fmt,
    mem::{size_of, zeroed},
    ptr::null_mut,
};

use windows_sys::Win32::{
    Foundation::{BOOL, HANDLE, POINT, RECT},
    Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetClipBox,
        GetCurrentObject, GetGraphicsMode, GetMapMode, GetObjectW, LPtoDP, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIBSECTION, DIB_RGB_COLORS, GM_COMPATIBLE, HBITMAP,
        HDC, HGDIOBJ, MM_TEXT, NULLREGION, OBJ_BITMAP, RGBQUAD, SIMPLEREGION, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
        SystemParametersInfoW, FE_FONTSMOOTHINGCLEARTYPE, FE_FONTSMOOTHINGORIENTATIONBGR,
        FE_FONTSMOOTHINGORIENTATIONRGB, SPI_GETFONTSMOOTHING, SPI_GETFONTSMOOTHINGORIENTATION,
        SPI_GETFONTSMOOTHINGTYPE,
    },
};

use crate::{
    rust_text_renderer::{
        ZsRustTextEngine, ZsTextLineMetricPolicy, ZsTextPixelRect, ZsTextRasterProfile,
    },
    Size, TextRun, TextStyle,
};

/// Per-window compositor state. Font discovery and glyph allocations are lazy.
/// Text normally composites straight into the destination DIB section; the
/// private top-down DIB exists only for device-dependent targets and grows to
/// the largest text box drawn into one.
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
        let raster_profile = windows_rust_text_raster_profile(style.semantic_role, self.profile);

        // Buffered paint and band surfaces are DIB sections: composite into
        // their pixels in place instead of copying through a private bitmap.
        if let Some(mut surface) = unsafe { SelectedDibSurface::of(destination) } {
            let origin = surface.to_device(run.bounds.x, run.bounds.y);
            let Some(clip) = surface.clip_to(origin, run.bounds.width, run.bounds.height) else {
                return Ok(());
            };
            let (width, height, stride) = (surface.width, surface.height, surface.stride);
            unsafe {
                GdiFlush();
            }
            // SAFETY: the DIB stays selected into `destination` for this whole
            // call, GDI has been flushed and nothing else touches its memory
            // until the slice is dropped at the end of this block.
            let pixels = unsafe { surface.pixels_mut() };
            return engine.composite_bgra_clipped(
                &layout,
                pixels,
                width,
                height,
                stride,
                origin.0,
                origin.1,
                clip,
                style.color,
                raster_profile,
            );
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
            raster_profile,
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

    pub(crate) const fn stats(&self) -> WindowsRustTextStats {
        self.stats
    }
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

/// The 32-bit top-down DIB section currently selected into a device context,
/// with the DC's logical-to-device offset and rectangular clip resolved.
struct SelectedDibSurface {
    bits: *mut u8,
    width: u32,
    height: u32,
    stride: usize,
    device_offset: (i32, i32),
    /// Device-space left, top, right and bottom edges.
    clip: (i32, i32, i32, i32),
}

impl SelectedDibSurface {
    /// Returns `None` whenever writing the bits could differ from what GDI
    /// itself would produce: a device-dependent bitmap, another pixel layout,
    /// a scaling or world transform, or a non-rectangular clip region.
    unsafe fn of(dc: HDC) -> Option<Self> {
        if dc.is_null() || GetMapMode(dc) != MM_TEXT || GetGraphicsMode(dc) != GM_COMPATIBLE as i32
        {
            return None;
        }
        let bitmap = GetCurrentObject(dc, OBJ_BITMAP as u32);
        if bitmap.is_null() {
            return None;
        }
        let mut section: DIBSECTION = zeroed();
        let section_size = size_of::<DIBSECTION>() as i32;
        if GetObjectW(
            bitmap,
            section_size,
            (&mut section as *mut DIBSECTION).cast(),
        ) != section_size
        {
            return None;
        }
        let header = &section.dsBmih;
        if section.dsBm.bmBits.is_null()
            || section.dsBm.bmBitsPixel != 32
            || header.biHeight >= 0
            || header.biCompression != BI_RGB
        {
            return None;
        }
        let width = u32::try_from(section.dsBm.bmWidth).ok()?;
        let height = section.dsBm.bmHeight.unsigned_abs();
        let stride = usize::try_from(section.dsBm.bmWidthBytes).ok()?;
        if stride < width as usize * 4 {
            return None;
        }

        let mut points = [POINT { x: 0, y: 0 }, POINT { x: 1, y: 1 }];
        if LPtoDP(dc, points.as_mut_ptr(), points.len() as i32) == 0
            || points[1].x - points[0].x != 1
            || points[1].y - points[0].y != 1
        {
            return None;
        }
        let device_offset = (points[0].x, points[0].y);
        let mut clip_box: RECT = zeroed();
        let clip = match GetClipBox(dc, &mut clip_box) {
            SIMPLEREGION => (
                clip_box.left.saturating_add(device_offset.0).max(0),
                clip_box.top.saturating_add(device_offset.1).max(0),
                clip_box
                    .right
                    .saturating_add(device_offset.0)
                    .min(i32::try_from(width).ok()?),
                clip_box
                    .bottom
                    .saturating_add(device_offset.1)
                    .min(i32::try_from(height).ok()?),
            ),
            NULLREGION => (0, 0, 0, 0),
            _ => return None,
        };
        Some(Self {
            bits: section.dsBm.bmBits.cast(),
            width,
            height,
            stride,
            device_offset,
            clip,
        })
    }

    const fn to_device(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x.saturating_add(self.device_offset.0),
            y.saturating_add(self.device_offset.1),
        )
    }

    /// The visible part of a box at a device origin, or `None` when GDI
    /// would clip all of it away.
    fn clip_to(&self, origin: (i32, i32), width: i32, height: i32) -> Option<ZsTextPixelRect> {
        let left = origin.0.max(self.clip.0);
        let top = origin.1.max(self.clip.1);
        let right = origin.0.saturating_add(width).min(self.clip.2);
        let bottom = origin.1.saturating_add(height).min(self.clip.3);
        (right > left && bottom > top)
            .then(|| ZsTextPixelRect::new(left, top, (right - left) as u32, (bottom - top) as u32))
    }

    /// # Safety
    ///
    /// The DIB must still be selected into the DC it was read from, and GDI
    /// must have been flushed so no batched drawing writes it concurrently.
    unsafe fn pixels_mut(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.bits, self.stride * self.height as usize)
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
