#![allow(non_snake_case)]

#[cfg(feature = "rust-text-proof")]
use std::sync::{Arc, Mutex};
use std::{ffi::c_void, fmt, sync::OnceLock};
#[cfg(feature = "rust-text-proof")]
use std::{mem::size_of, ptr::null_mut};

use windows::core::{implement, Interface, HRESULT, PCWSTR};
use windows::Win32::{
    Foundation::COLORREF,
    Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteBitmapRenderTarget, IDWriteFactory, IDWriteGdiInterop,
        IDWriteInlineObject, IDWritePixelSnapping_Impl, IDWriteRenderingParams, IDWriteTextLayout,
        IDWriteTextLayout4, IDWriteTextRenderer, IDWriteTextRenderer_Impl,
        DWRITE_AUTOMATIC_FONT_AXES_OPTICAL_SIZE, DWRITE_FACTORY_TYPE_SHARED,
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_GLYPH_RUN,
        DWRITE_GLYPH_RUN_DESCRIPTION, DWRITE_LINE_METRICS, DWRITE_LINE_SPACING_METHOD_UNIFORM,
        DWRITE_MATRIX, DWRITE_MEASURING_MODE, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
        DWRITE_PARAGRAPH_ALIGNMENT_FAR, DWRITE_PARAGRAPH_ALIGNMENT_NEAR, DWRITE_STRIKETHROUGH,
        DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
        DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TEXT_METRICS, DWRITE_TRIMMING,
        DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_UNDERLINE, DWRITE_WORD_WRAPPING_NO_WRAP,
        DWRITE_WORD_WRAPPING_WRAP,
    },
};
#[cfg(feature = "rust-text-proof")]
use windows_sys::Win32::{
    Foundation::HANDLE,
    Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ, RGBQUAD,
    },
};
use windows_sys::Win32::{
    Globalization::GetUserDefaultLocaleName,
    Graphics::Gdi::{BitBlt, GdiFlush, HDC, SRCCOPY},
};

#[cfg(feature = "rust-text-proof")]
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFontFace3, IDWriteLocalizedStrings, DWRITE_GLYPH_OFFSET,
    DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
};

#[cfg(feature = "rust-text-proof")]
use crate::rust_text_renderer::{ZsTextGlyphProof, ZsTextLineProof, ZsTextProof};
#[cfg(feature = "rust-text-proof")]
use crate::Rect;
use crate::{
    Color, HorizontalAlign, Size, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};

static DIRECTWRITE_SYSTEM: OnceLock<Option<DirectWriteSystem>> = OnceLock::new();

struct DirectWriteSystem {
    factory: IDWriteFactory,
    gdi_interop: IDWriteGdiInterop,
    rendering_params: IDWriteRenderingParams,
    locale: Vec<u16>,
}

impl DirectWriteSystem {
    fn initialize() -> Option<Self> {
        let factory =
            unsafe { DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) }.ok()?;
        let gdi_interop = unsafe { factory.GetGdiInterop() }.ok()?;
        let rendering_params = unsafe { factory.CreateRenderingParams() }.ok()?;
        Some(Self {
            factory,
            gdi_interop,
            rendering_params,
            locale: user_locale_name(),
        })
    }

    fn shared() -> Option<&'static Self> {
        DIRECTWRITE_SYSTEM
            .get_or_init(DirectWriteSystem::initialize)
            .as_ref()
    }

    fn text_layout(
        &self,
        text: &str,
        style: &TextStyle,
        max_width_px: i32,
        max_height_px: i32,
        dpi_scale: f32,
    ) -> windows::core::Result<IDWriteTextLayout> {
        let family = wide_null(&style.font_family);
        let text = text.encode_utf16().collect::<Vec<_>>();
        let format = unsafe {
            self.factory.CreateTextFormat(
                PCWSTR(family.as_ptr()),
                None,
                directwrite_weight(style.weight),
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size.max(1.0),
                PCWSTR(self.locale.as_ptr()),
            )?
        };
        unsafe {
            format.SetTextAlignment(match style.horizontal_align {
                HorizontalAlign::Start => DWRITE_TEXT_ALIGNMENT_LEADING,
                HorizontalAlign::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
                HorizontalAlign::End => DWRITE_TEXT_ALIGNMENT_TRAILING,
            })?;
            format.SetParagraphAlignment(match style.vertical_align {
                VerticalAlign::Start => DWRITE_PARAGRAPH_ALIGNMENT_NEAR,
                VerticalAlign::Center => DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                VerticalAlign::End => DWRITE_PARAGRAPH_ALIGNMENT_FAR,
            })?;
            format.SetWordWrapping(match style.wrap {
                TextWrap::NoWrap => DWRITE_WORD_WRAPPING_NO_WRAP,
                TextWrap::Word => DWRITE_WORD_WRAPPING_WRAP,
            })?;
            if style.ellipsis && style.wrap == TextWrap::NoWrap {
                let sign = self.factory.CreateEllipsisTrimmingSign(&format)?;
                let trimming = DWRITE_TRIMMING {
                    granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                    delimiter: 0,
                    delimiterCount: 0,
                };
                format.SetTrimming(&trimming, &sign)?;
            }
        }

        let scale = dpi_scale.max(0.5);
        let max_width = if max_width_px > 0 {
            max_width_px as f32 / scale
        } else {
            32_767.0 / scale
        };
        let max_height = if max_height_px > 0 {
            max_height_px as f32 / scale
        } else {
            32_767.0 / scale
        };
        let layout = unsafe {
            self.factory.CreateTextLayout(
                &text,
                &format,
                max_width.max(1.0),
                max_height.max(1.0),
            )?
        };
        if style.line_height > 0.0 {
            let mut line_count = 0;
            let _ = unsafe { layout.GetLineMetrics(None, &mut line_count) };
            if line_count > 0 {
                let mut lines = vec![DWRITE_LINE_METRICS::default(); line_count as usize];
                if unsafe { layout.GetLineMetrics(Some(&mut lines), &mut line_count) }.is_ok() {
                    if let Some(natural) = lines.first() {
                        let target_height = style.line_height.max(style.size).max(1.0);
                        let baseline = if natural.height > 0.0 {
                            natural.baseline * target_height / natural.height
                        } else {
                            target_height * 0.8
                        };
                        let _ = unsafe {
                            layout.SetLineSpacing(
                                DWRITE_LINE_SPACING_METHOD_UNIFORM,
                                target_height,
                                baseline,
                            )
                        };
                    }
                }
            }
        }
        if let Ok(layout4) = layout.cast::<IDWriteTextLayout4>() {
            let _ =
                unsafe { layout4.SetAutomaticFontAxes(DWRITE_AUTOMATIC_FONT_AXES_OPTICAL_SIZE) };
        }
        Ok(layout)
    }
}

/// Reusable software DirectWrite surface owned by the existing per-window GDI cache.
///
/// The bitmap grows only to the largest text run seen by the window. It avoids a
/// second full-window backing store and keeps the buffered Win32 paint path intact.
#[derive(Default)]
pub(crate) struct WindowsDirectWriteState {
    bitmap_target: Option<IDWriteBitmapRenderTarget>,
    width: u32,
    height: u32,
}

impl fmt::Debug for WindowsDirectWriteState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WindowsDirectWriteState")
            .field("initialized", &self.bitmap_target.is_some())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl WindowsDirectWriteState {
    pub(crate) fn draw_text(
        &mut self,
        destination: HDC,
        run: &TextRun,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> bool {
        if destination.is_null() || run.text.is_empty() {
            return false;
        }
        self.try_draw_text(destination, run, style, dpi_scale)
            .is_ok()
    }

    fn try_draw_text(
        &mut self,
        destination: HDC,
        run: &TextRun,
        style: &TextStyle,
        dpi_scale: f32,
    ) -> windows::core::Result<()> {
        let system = DirectWriteSystem::shared()
            .ok_or_else(|| windows::core::Error::from_hresult(HRESULT(0x8000_4005_u32 as i32)))?;
        let width = run.bounds.width.max(1) as u32;
        let height = run.bounds.height.max(1) as u32;
        self.ensure_target(system, width, height)?;
        let target = self.bitmap_target.as_ref().expect("target was ensured");
        unsafe { target.SetPixelsPerDip(dpi_scale.max(0.5))? };

        let layout = system.text_layout(
            &run.text,
            style,
            run.bounds.width,
            run.bounds.height,
            dpi_scale,
        )?;
        let memory_dc = unsafe { target.GetMemoryDC() };
        unsafe {
            GdiFlush();
            if BitBlt(
                memory_dc.0,
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
                return Err(windows::core::Error::from_thread());
            }
        }

        let renderer: IDWriteTextRenderer = BitmapTextRenderer {
            target: target.clone(),
            rendering_params: system.rendering_params.clone(),
            color: directwrite_color(style.color),
            pixels_per_dip: dpi_scale.max(0.5),
        }
        .into();
        let drawing_context = (&renderer as *const IDWriteTextRenderer).cast::<c_void>();
        unsafe {
            layout.Draw(Some(drawing_context), &renderer, 0.0, 0.0)?;
            GdiFlush();
            if BitBlt(
                destination,
                run.bounds.x,
                run.bounds.y,
                width as i32,
                height as i32,
                memory_dc.0,
                0,
                0,
                SRCCOPY,
            ) == 0
            {
                return Err(windows::core::Error::from_thread());
            }
        }
        Ok(())
    }

    fn ensure_target(
        &mut self,
        system: &DirectWriteSystem,
        width: u32,
        height: u32,
    ) -> windows::core::Result<()> {
        let required_width = self.width.max(width);
        let required_height = self.height.max(height);
        if let Some(target) = &self.bitmap_target {
            if required_width != self.width || required_height != self.height {
                unsafe { target.Resize(required_width, required_height)? };
                self.width = required_width;
                self.height = required_height;
            }
            return Ok(());
        }
        self.bitmap_target = Some(unsafe {
            system
                .gdi_interop
                .CreateBitmapRenderTarget(None, required_width, required_height)?
        });
        self.width = required_width;
        self.height = required_height;
        Ok(())
    }
}

/// Renders the actual DirectWrite bitmap target into a safe top-down BGRA8
/// buffer for pixel regression. This is an oracle path, not the Rust renderer.
#[cfg(feature = "rust-text-proof")]
pub fn directwrite_text_bgra(
    text: &str,
    style: &TextStyle,
    width: u32,
    height: u32,
    dpi_scale: f32,
    background: Color,
) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return Some(Vec::new());
    }
    let mut dib = DirectWriteProofDib::new(width, height)?;
    for pixel in dib.pixels_mut()?.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[background.b, background.g, background.r, background.a]);
    }
    let run = TextRun {
        text: text.to_string(),
        bounds: Rect {
            x: 0,
            y: 0,
            width: width as i32,
            height: height as i32,
        },
    };
    let mut renderer = WindowsDirectWriteState::default();
    renderer.draw_text(dib.dc, &run, style, dpi_scale).then(|| {
        dib.pixels_mut()
            .map(|pixels| pixels.to_vec())
            .unwrap_or_default()
    })
}

#[cfg(feature = "rust-text-proof")]
struct DirectWriteProofDib {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u8,
    width: u32,
    height: u32,
}

#[cfg(feature = "rust-text-proof")]
impl DirectWriteProofDib {
    fn new(width: u32, height: u32) -> Option<Self> {
        let width_i32 = i32::try_from(width).ok()?;
        let height_i32 = i32::try_from(height).ok()?;
        let image_bytes = width.checked_mul(height)?.checked_mul(4)?;
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
        let dc = unsafe { CreateCompatibleDC(null_mut()) };
        if dc.is_null() {
            return None;
        }
        let mut bits = null_mut::<c_void>();
        let bitmap = unsafe {
            CreateDIBSection(
                dc,
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
            return None;
        }
        let previous = unsafe { SelectObject(dc, bitmap as HGDIOBJ) };
        if previous.is_null() {
            unsafe {
                DeleteObject(bitmap as HGDIOBJ);
                DeleteDC(dc);
            }
            return None;
        }
        Some(Self {
            dc,
            bitmap,
            previous,
            bits: bits.cast(),
            width,
            height,
        })
    }

    fn pixels_mut(&mut self) -> Option<&mut [u8]> {
        let len = self.width.checked_mul(self.height)?.checked_mul(4)? as usize;
        (!self.bits.is_null()).then(|| {
            // SAFETY: the DIB owns exactly width * height * 4 bytes and lives
            // for the duration of the returned mutable borrow.
            unsafe { std::slice::from_raw_parts_mut(self.bits, len) }
        })
    }
}

#[cfg(feature = "rust-text-proof")]
impl Drop for DirectWriteProofDib {
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

pub(crate) fn measure_text(
    text: &str,
    style: &TextStyle,
    max_width: i32,
    dpi_scale: f32,
) -> Option<Size> {
    if text.is_empty() {
        return Some(Size {
            width: 0,
            height: 0,
        });
    }
    let system = DirectWriteSystem::shared()?;
    // Alignment belongs to the eventual paint box. Measuring against the
    // deliberately large probe box with centered/far paragraph alignment would
    // turn that probe height into the widget's intrinsic height.
    let mut measure_style = style.clone();
    measure_style.horizontal_align = HorizontalAlign::Start;
    measure_style.vertical_align = VerticalAlign::Start;
    measure_style.ellipsis = false;
    let layout = system
        .text_layout(text, &measure_style, max_width, 0, dpi_scale)
        .ok()?;
    let mut metrics = DWRITE_TEXT_METRICS::default();
    unsafe { layout.GetMetrics(&mut metrics) }.ok()?;
    let scale = dpi_scale.max(0.5);
    Some(Size {
        width: (metrics.widthIncludingTrailingWhitespace * scale).ceil() as i32,
        height: (metrics.height * scale).ceil() as i32,
    })
}

#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone)]
struct CapturedDirectWriteGlyphRun {
    baseline_x: f32,
    baseline_y: f32,
    font_size: f32,
    font_family: String,
    postscript_name: String,
    face_index: u32,
    bidi_level: u32,
    glyph_ids: Vec<u16>,
    advances: Vec<f32>,
    offsets: Vec<DWRITE_GLYPH_OFFSET>,
    clusters_utf16: Vec<(usize, usize)>,
}

/// Produces the DirectWrite side of a backend-neutral geometry proof without
/// exposing COM objects. UTF-16 clusters are converted back to UTF-8 byte
/// offsets so the result can be compared with the Rust shaper.
#[cfg(feature = "rust-text-proof")]
pub fn directwrite_text_proof(
    text: &str,
    style: &TextStyle,
    width_px: i32,
    height_px: i32,
    dpi_scale: f32,
) -> Option<ZsTextProof> {
    let scale = dpi_scale.max(0.5);
    if text.is_empty() {
        return Some(ZsTextProof {
            schema: "zsui.text-proof/v1".into(),
            backend: "windows-directwrite".into(),
            requested_font_family: style.font_family.clone(),
            dpi_scale: scale,
            width_px: width_px.max(0),
            height_px: height_px.max(0),
            content_width_px: 0,
            content_height_px: 0,
            overflow_x: false,
            overflow_y: false,
            lines: Vec::new(),
            errors: Vec::new(),
        });
    }
    let system = DirectWriteSystem::shared()?;
    let layout = system
        .text_layout(text, style, width_px, height_px, scale)
        .ok()?;
    let captured = Arc::new(Mutex::new(Vec::new()));
    let renderer: IDWriteTextRenderer = DirectWriteProofRenderer {
        captured: Arc::clone(&captured),
        pixels_per_dip: scale,
    }
    .into();
    unsafe { layout.Draw(None, &renderer, 0.0, 0.0) }.ok()?;

    let mut metrics = DWRITE_TEXT_METRICS::default();
    unsafe { layout.GetMetrics(&mut metrics) }.ok()?;
    let line_metrics = directwrite_line_metrics(&layout);
    let captured = captured.lock().ok()?.clone();
    let utf16_to_utf8 = utf16_to_utf8_offsets(text);
    let mut lines = Vec::<ZsTextLineProof>::new();
    for run in captured {
        let line_index = lines
            .iter()
            .position(|line| (line.baseline_px / scale - run.baseline_y).abs() < 0.01)
            .unwrap_or_else(|| {
                let index = lines.len();
                let metric = line_metrics.get(index).copied().unwrap_or_default();
                lines.push(ZsTextLineProof {
                    line_index: index,
                    top_px: (run.baseline_y - metric.baseline) * scale,
                    baseline_px: run.baseline_y * scale,
                    height_px: metric.height.max(run.font_size) * scale,
                    width_px: 0.0,
                    rtl: run.bidi_level % 2 == 1,
                    glyphs: Vec::new(),
                });
                index
            });
        let right_to_left = run.bidi_level % 2 == 1;
        let mut advance_cursor = 0.0f32;
        for glyph_index in 0..run.glyph_ids.len() {
            let advance = run.advances.get(glyph_index).copied().unwrap_or(0.0);
            let offset = run.offsets.get(glyph_index).copied().unwrap_or_default();
            let origin_x = if right_to_left {
                run.baseline_x - advance_cursor - advance - offset.advanceOffset
            } else {
                run.baseline_x + advance_cursor + offset.advanceOffset
            };
            let origin_y = run.baseline_y - offset.ascenderOffset;
            let (cluster_start_utf16, cluster_end_utf16) = run
                .clusters_utf16
                .get(glyph_index)
                .copied()
                .unwrap_or((0, 0));
            let cluster_start = utf16_to_utf8
                .get(cluster_start_utf16)
                .copied()
                .unwrap_or(text.len());
            let cluster_end = utf16_to_utf8
                .get(cluster_end_utf16)
                .copied()
                .unwrap_or(text.len());
            lines[line_index].glyphs.push(ZsTextGlyphProof {
                cluster_start,
                cluster_end,
                glyph_id: run.glyph_ids[glyph_index],
                font_family: run.font_family.clone(),
                postscript_name: run.postscript_name.clone(),
                face_index: run.face_index,
                origin_x_px: origin_x * scale,
                origin_y_px: origin_y * scale,
                offset_x_px: offset.advanceOffset * scale,
                offset_y_px: offset.ascenderOffset * scale,
                advance_px: advance * scale,
                font_size_px: run.font_size * scale,
                rtl: run.bidi_level % 2 == 1,
            });
            advance_cursor += advance;
        }
        lines[line_index].width_px = lines[line_index].width_px.max(advance_cursor.abs() * scale);
    }
    for line in &mut lines {
        let left = line
            .glyphs
            .iter()
            .map(|glyph| glyph.origin_x_px.min(glyph.origin_x_px + glyph.advance_px))
            .fold(f32::INFINITY, f32::min);
        let right = line
            .glyphs
            .iter()
            .map(|glyph| glyph.origin_x_px.max(glyph.origin_x_px + glyph.advance_px))
            .fold(f32::NEG_INFINITY, f32::max);
        if left.is_finite() && right.is_finite() {
            line.width_px = right - left;
        }
    }
    let content_width_px = (metrics.widthIncludingTrailingWhitespace * scale).ceil() as i32;
    let content_height_px = (metrics.height * scale).ceil() as i32;
    Some(ZsTextProof {
        schema: "zsui.text-proof/v1".into(),
        backend: "windows-directwrite".into(),
        requested_font_family: style.font_family.clone(),
        dpi_scale: scale,
        width_px: width_px.max(0),
        height_px: height_px.max(0),
        content_width_px,
        content_height_px,
        overflow_x: width_px > 0 && content_width_px > width_px,
        overflow_y: height_px > 0 && content_height_px > height_px,
        lines,
        errors: Vec::new(),
    })
}

#[cfg(feature = "rust-text-proof")]
fn directwrite_line_metrics(layout: &IDWriteTextLayout) -> Vec<DWRITE_LINE_METRICS> {
    let mut count = 0;
    let _ = unsafe { layout.GetLineMetrics(None, &mut count) };
    if count == 0 {
        return Vec::new();
    }
    let mut metrics = vec![DWRITE_LINE_METRICS::default(); count as usize];
    if unsafe { layout.GetLineMetrics(Some(&mut metrics), &mut count) }.is_err() {
        return Vec::new();
    }
    metrics.truncate(count as usize);
    metrics
}

#[cfg(feature = "rust-text-proof")]
#[implement(IDWriteTextRenderer)]
struct DirectWriteProofRenderer {
    captured: Arc<Mutex<Vec<CapturedDirectWriteGlyphRun>>>,
    pixels_per_dip: f32,
}

#[cfg(feature = "rust-text-proof")]
impl IDWritePixelSnapping_Impl for DirectWriteProofRenderer_Impl {
    fn IsPixelSnappingDisabled(
        &self,
        _client_drawing_context: *const c_void,
    ) -> windows::core::Result<windows::core::BOOL> {
        Ok(false.into())
    }

    fn GetCurrentTransform(
        &self,
        _client_drawing_context: *const c_void,
        transform: *mut DWRITE_MATRIX,
    ) -> windows::core::Result<()> {
        if transform.is_null() {
            return Err(windows::core::Error::from_hresult(HRESULT(
                0x8000_4003_u32 as i32,
            )));
        }
        unsafe {
            transform.write(DWRITE_MATRIX {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                dx: 0.0,
                dy: 0.0,
            });
        }
        Ok(())
    }

    fn GetPixelsPerDip(
        &self,
        _client_drawing_context: *const c_void,
    ) -> windows::core::Result<f32> {
        Ok(self.pixels_per_dip)
    }
}

#[cfg(feature = "rust-text-proof")]
impl IDWriteTextRenderer_Impl for DirectWriteProofRenderer_Impl {
    fn DrawGlyphRun(
        &self,
        _client_drawing_context: *const c_void,
        baseline_origin_x: f32,
        baseline_origin_y: f32,
        _measuring_mode: DWRITE_MEASURING_MODE,
        glyph_run: *const DWRITE_GLYPH_RUN,
        glyph_run_description: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        let Some(run) = (unsafe { glyph_run.as_ref() }) else {
            return Ok(());
        };
        let glyph_count = run.glyphCount as usize;
        if glyph_count == 0 || run.glyphIndices.is_null() {
            return Ok(());
        }
        let glyph_ids =
            unsafe { std::slice::from_raw_parts(run.glyphIndices, glyph_count) }.to_vec();
        let advances = if run.glyphAdvances.is_null() {
            vec![0.0; glyph_count]
        } else {
            unsafe { std::slice::from_raw_parts(run.glyphAdvances, glyph_count) }.to_vec()
        };
        let offsets = if run.glyphOffsets.is_null() {
            vec![DWRITE_GLYPH_OFFSET::default(); glyph_count]
        } else {
            unsafe { std::slice::from_raw_parts(run.glyphOffsets, glyph_count) }.to_vec()
        };
        let clusters_utf16 = directwrite_glyph_clusters(glyph_run_description, glyph_count);
        let (font_family, postscript_name, face_index) = run
            .fontFace
            .as_ref()
            .as_ref()
            .map(|face| directwrite_face_identity(face))
            .unwrap_or_else(|| (String::new(), String::new(), 0));
        if let Ok(mut captured) = self.captured.lock() {
            captured.push(CapturedDirectWriteGlyphRun {
                baseline_x: baseline_origin_x,
                baseline_y: baseline_origin_y,
                font_size: run.fontEmSize,
                font_family,
                postscript_name,
                face_index,
                bidi_level: run.bidiLevel,
                glyph_ids,
                advances,
                offsets,
                clusters_utf16,
            });
        }
        Ok(())
    }

    fn DrawUnderline(
        &self,
        _client_drawing_context: *const c_void,
        _baseline_origin_x: f32,
        _baseline_origin_y: f32,
        _underline: *const DWRITE_UNDERLINE,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn DrawStrikethrough(
        &self,
        _client_drawing_context: *const c_void,
        _baseline_origin_x: f32,
        _baseline_origin_y: f32,
        _strikethrough: *const DWRITE_STRIKETHROUGH,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn DrawInlineObject(
        &self,
        _client_drawing_context: *const c_void,
        _origin_x: f32,
        _origin_y: f32,
        _inline_object: windows::core::Ref<'_, IDWriteInlineObject>,
        _is_sideways: windows::core::BOOL,
        _is_right_to_left: windows::core::BOOL,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "rust-text-proof")]
fn directwrite_glyph_clusters(
    description: *const DWRITE_GLYPH_RUN_DESCRIPTION,
    glyph_count: usize,
) -> Vec<(usize, usize)> {
    let mut clusters = vec![(usize::MAX, 0); glyph_count];
    let Some(description) = (unsafe { description.as_ref() }) else {
        return vec![(0, 0); glyph_count];
    };
    if !description.clusterMap.is_null() {
        let cluster_map = unsafe {
            std::slice::from_raw_parts(description.clusterMap, description.stringLength as usize)
        };
        for (text_offset, glyph_index) in cluster_map.iter().copied().enumerate() {
            let glyph_index = usize::from(glyph_index).min(glyph_count.saturating_sub(1));
            clusters[glyph_index].0 = clusters[glyph_index]
                .0
                .min(description.textPosition as usize + text_offset);
            clusters[glyph_index].1 = clusters[glyph_index]
                .1
                .max(description.textPosition as usize + text_offset + 1);
        }
    }
    let mut last = (
        description.textPosition as usize,
        description.textPosition as usize,
    );
    for cluster in &mut clusters {
        if cluster.0 == usize::MAX {
            *cluster = last;
        } else {
            last = *cluster;
        }
    }
    clusters
}

#[cfg(feature = "rust-text-proof")]
fn directwrite_face_identity(
    face: &windows::Win32::Graphics::DirectWrite::IDWriteFontFace,
) -> (String, String, u32) {
    let face_index = unsafe { face.GetIndex() };
    let Ok(face3) = face.cast::<IDWriteFontFace3>() else {
        return (String::new(), String::new(), face_index);
    };
    let family = unsafe { face3.GetFamilyNames() }
        .ok()
        .and_then(localized_first_string)
        .unwrap_or_default();
    let mut strings = None;
    let mut exists = false.into();
    let postscript = unsafe {
        face3.GetInformationalStrings(
            DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
            &mut strings,
            &mut exists,
        )
    }
    .ok()
    .filter(|_| exists.as_bool())
    .and_then(|_| strings)
    .and_then(localized_first_string)
    .unwrap_or_else(|| family.clone());
    (family, postscript, face_index)
}

#[cfg(feature = "rust-text-proof")]
fn localized_first_string(strings: IDWriteLocalizedStrings) -> Option<String> {
    if unsafe { strings.GetCount() } == 0 {
        return None;
    }
    let length = unsafe { strings.GetStringLength(0) }.ok()?;
    let mut wide = vec![0_u16; length as usize + 1];
    unsafe { strings.GetString(0, &mut wide) }.ok()?;
    wide.truncate(length as usize);
    String::from_utf16(&wide).ok()
}

#[cfg(feature = "rust-text-proof")]
fn utf16_to_utf8_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (byte_index, character) in text.char_indices() {
        let next = byte_index + character.len_utf8();
        for _ in 0..character.len_utf16() {
            offsets.push(next);
        }
    }
    offsets
}

#[implement(IDWriteTextRenderer)]
struct BitmapTextRenderer {
    target: IDWriteBitmapRenderTarget,
    rendering_params: IDWriteRenderingParams,
    color: COLORREF,
    pixels_per_dip: f32,
}

impl IDWritePixelSnapping_Impl for BitmapTextRenderer_Impl {
    fn IsPixelSnappingDisabled(
        &self,
        _client_drawing_context: *const c_void,
    ) -> windows::core::Result<windows::core::BOOL> {
        Ok(false.into())
    }

    fn GetCurrentTransform(
        &self,
        _client_drawing_context: *const c_void,
        transform: *mut DWRITE_MATRIX,
    ) -> windows::core::Result<()> {
        if transform.is_null() {
            return Err(windows::core::Error::from_hresult(HRESULT(
                0x8000_4003_u32 as i32,
            )));
        }
        unsafe {
            transform.write(DWRITE_MATRIX {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                dx: 0.0,
                dy: 0.0,
            });
        }
        Ok(())
    }

    fn GetPixelsPerDip(
        &self,
        _client_drawing_context: *const c_void,
    ) -> windows::core::Result<f32> {
        Ok(self.pixels_per_dip)
    }
}

impl IDWriteTextRenderer_Impl for BitmapTextRenderer_Impl {
    fn DrawGlyphRun(
        &self,
        _client_drawing_context: *const c_void,
        baseline_origin_x: f32,
        baseline_origin_y: f32,
        measuring_mode: DWRITE_MEASURING_MODE,
        glyph_run: *const DWRITE_GLYPH_RUN,
        _glyph_run_description: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        if glyph_run.is_null() {
            return Ok(());
        }
        unsafe {
            self.target.DrawGlyphRun(
                baseline_origin_x,
                baseline_origin_y,
                measuring_mode,
                glyph_run,
                &self.rendering_params,
                self.color,
                None,
            )
        }
    }

    fn DrawUnderline(
        &self,
        _client_drawing_context: *const c_void,
        _baseline_origin_x: f32,
        _baseline_origin_y: f32,
        _underline: *const DWRITE_UNDERLINE,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn DrawStrikethrough(
        &self,
        _client_drawing_context: *const c_void,
        _baseline_origin_x: f32,
        _baseline_origin_y: f32,
        _strikethrough: *const DWRITE_STRIKETHROUGH,
        _client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn DrawInlineObject(
        &self,
        client_drawing_context: *const c_void,
        origin_x: f32,
        origin_y: f32,
        inline_object: windows::core::Ref<'_, IDWriteInlineObject>,
        is_sideways: windows::core::BOOL,
        is_right_to_left: windows::core::BOOL,
        client_drawing_effect: windows::core::Ref<'_, windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        let inline_object = inline_object.ok()?;
        if client_drawing_context.is_null() {
            return Ok(());
        }
        let renderer = unsafe { &*(client_drawing_context.cast::<IDWriteTextRenderer>()) };
        unsafe {
            inline_object.Draw(
                Some(client_drawing_context),
                renderer,
                origin_x,
                origin_y,
                is_sideways.as_bool(),
                is_right_to_left.as_bool(),
                client_drawing_effect.as_ref(),
            )
        }
    }
}

fn directwrite_weight(weight: TextWeight) -> DWRITE_FONT_WEIGHT {
    DWRITE_FONT_WEIGHT(match weight {
        TextWeight::Automatic | TextWeight::Regular => 400,
        TextWeight::Medium => 500,
        TextWeight::Semibold => 600,
        TextWeight::Bold => 700,
    })
}

fn directwrite_color(color: Color) -> COLORREF {
    COLORREF((color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16))
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

fn user_locale_name() -> Vec<u16> {
    let mut locale = vec![0_u16; 85];
    let length = unsafe { GetUserDefaultLocaleName(locale.as_mut_ptr(), locale.len() as i32) };
    if length > 1 {
        locale.truncate(length as usize);
        locale
    } else {
        wide_null("en-US")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directwrite_mappings_preserve_public_text_weights() {
        assert_eq!(directwrite_weight(TextWeight::Regular).0, 400);
        assert_eq!(directwrite_weight(TextWeight::Medium).0, 500);
        assert_eq!(directwrite_weight(TextWeight::Semibold).0, 600);
        assert_eq!(directwrite_weight(TextWeight::Bold).0, 700);
    }

    #[test]
    fn colorref_uses_windows_channel_order() {
        assert_eq!(
            directwrite_color(Color::rgba(0x12, 0x34, 0x56, 0xff)).0,
            0x563412
        );
    }

    #[test]
    fn empty_text_has_zero_measurement() {
        let style = TextStyle::line("Segoe UI", 14.0, Color::rgba(0, 0, 0, 0xff));
        assert_eq!(
            measure_text("", &style, 320, 1.0),
            Some(Size {
                width: 0,
                height: 0
            })
        );
    }

    #[test]
    fn measurement_is_intrinsic_even_when_paint_alignment_is_centered() {
        let style = TextStyle::line("Segoe UI", 14.0, Color::rgba(0, 0, 0, 0xff));
        let measured = measure_text("发票助手 / Invoice Assistant", &style, 800, 1.0)
            .expect("DirectWrite should be available on Windows");
        assert!(measured.width > 100 && measured.width < 800, "{measured:?}");
        assert!(measured.height > 8 && measured.height < 80, "{measured:?}");
    }
}
