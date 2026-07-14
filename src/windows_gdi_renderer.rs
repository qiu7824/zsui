use std::{ffi::c_void, io::Cursor, ptr::null, sync::OnceLock};

use crate::{
    native_icons::{WINDOWS_FLUENT_ICON_FONT_FAMILY, WINDOWS_MDL2_ICON_FONT_FAMILY},
    Color, ColorRole, HorizontalAlign, NativeDrawCommand, NativeDrawCommandOperation,
    NativeDrawCommandSink, NativeDrawFill, NativeDrawIconCommand, NativeDrawPlan,
    NativeDrawTextCommand, NativeIconColorMode, NativeStyleResolver, Rect, Renderer,
    SemanticTextStyle, Size, TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
    ZsIcon,
};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::{
        CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
        GetStockObject, GetTextFaceW, IntersectClipRect, Polygon, RestoreDC, RoundRect, SaveDC,
        SelectObject, SetBkMode, SetBrushOrgEx, SetStretchBltMode, SetTextColor, StretchDIBits,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH,
        DIB_RGB_COLORS, FF_DONTCARE, HALFTONE, HDC, HGDIOBJ, NULL_PEN, OUT_DEFAULT_PRECIS,
        PS_SOLID, SRCCOPY,
    },
};
#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
use windows_sys::Win32::{
    Globalization::{
        ScriptStringAnalyse, ScriptStringCPtoX, ScriptStringFree, ScriptString_pSize, SSA_BREAK,
        SSA_FALLBACK, SSA_GLYPHS, SSA_LINK,
    },
    Graphics::Gdi::{CreateCompatibleDC, DeleteDC},
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
static WINDOWS_SYSTEM_ICON_FONT: OnceLock<WindowsSystemIconFont> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsSystemIconFont {
    SegoeFluentIcons,
    SegoeMdl2Assets,
    Unavailable,
}

impl WindowsSystemIconFont {
    pub const fn font_family(self) -> Option<&'static str> {
        match self {
            Self::SegoeFluentIcons => Some(WINDOWS_FLUENT_ICON_FONT_FAMILY),
            Self::SegoeMdl2Assets => Some(WINDOWS_MDL2_ICON_FONT_FAMILY),
            Self::Unavailable => None,
        }
    }

    pub const fn glyph(self, icon: ZsIcon) -> Option<&'static str> {
        match self {
            Self::SegoeFluentIcons => Some(icon.windows_fluent_glyph()),
            Self::SegoeMdl2Assets => Some(icon.windows_mdl2_glyph()),
            Self::Unavailable => None,
        }
    }
}

pub const fn select_windows_system_icon_font(
    fluent_available: bool,
    mdl2_available: bool,
) -> WindowsSystemIconFont {
    if fluent_available {
        WindowsSystemIconFont::SegoeFluentIcons
    } else if mdl2_available {
        WindowsSystemIconFont::SegoeMdl2Assets
    } else {
        WindowsSystemIconFont::Unavailable
    }
}

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
    fn GdipDrawArcI(
        graphics: *mut c_void,
        pen: *mut c_void,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: f32,
        sweep_angle: f32,
    ) -> i32;
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

unsafe fn draw_arc_antialiased(
    hdc: HDC,
    rect: RECT,
    color: Color,
    width: i32,
    start_degrees: i16,
    sweep_degrees: i16,
) -> bool {
    if ensure_gdiplus_startup().is_none()
        || hdc.is_null()
        || rect.right <= rect.left
        || rect.bottom <= rect.top
        || sweep_degrees == 0
    {
        return false;
    }
    let mut graphics = std::ptr::null_mut();
    if GdipCreateFromHDC(hdc, &mut graphics) != 0 || graphics.is_null() {
        return false;
    }
    let _ = GdipSetSmoothingMode(graphics, GDIP_SMOOTHING_MODE_ANTI_ALIAS);
    let mut pen = std::ptr::null_mut();
    let created = GdipCreatePen1(
        color_to_argb(color),
        width.max(1) as f32,
        GDIP_UNIT_PIXEL,
        &mut pen,
    ) == 0
        && !pen.is_null();
    let drawn = created
        && GdipDrawArcI(
            graphics,
            pen,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            f32::from(start_degrees),
            f32::from(sweep_degrees),
        ) == 0;
    if !pen.is_null() {
        let _ = GdipDeletePen(pen);
    }
    let _ = GdipDeleteGraphics(graphics);
    drawn
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

    fn stroke_arc(
        &mut self,
        rect: Rect,
        color: Color,
        width: i32,
        start_degrees: i16,
        sweep_degrees: i16,
    ) {
        if !self.has_dc() {
            return;
        }
        let _ = unsafe {
            draw_arc_antialiased(
                self.dc,
                to_win_rect(rect),
                color,
                width,
                start_degrees,
                sweep_degrees,
            )
        };
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

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
pub(crate) fn shape_windows_gdi_text_line(
    text: &str,
) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
    use crate::native_input_visuals::{
        NativeShapedTextCaret, NativeShapedTextCluster, NativeShapedTextLine,
    };

    if text.is_empty() {
        return None;
    }
    let dc = WindowsGdiOwnedMemoryDc::new()?;
    let style = WindowsGdiStyleResolver::default().resolve_text_style(SemanticTextStyle::body());
    with_font(dc.0, &style, |font_dc| {
        let wide = text.encode_utf16().collect::<Vec<_>>();
        let character_count = i32::try_from(wide.len()).ok()?;
        let glyph_capacity = character_count
            .saturating_add(character_count / 2)
            .saturating_add(16);
        let mut analysis = std::ptr::null_mut();
        let result = unsafe {
            ScriptStringAnalyse(
                font_dc,
                wide.as_ptr().cast(),
                character_count,
                glyph_capacity,
                -1,
                SSA_GLYPHS | SSA_FALLBACK | SSA_LINK | SSA_BREAK,
                0,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                &mut analysis,
            )
        };
        if result < 0 || analysis.is_null() {
            return None;
        }
        let analysis = WindowsUniscribeAnalysis(analysis);
        let size = unsafe { ScriptString_pSize(analysis.0) };
        if size.is_null() {
            return None;
        }
        let boundaries = crate::native_text_edit::grapheme_boundaries(text);
        let utf16_offsets = boundaries
            .iter()
            .map(|index| {
                i32::try_from(
                    text.chars()
                        .take(*index)
                        .map(char::len_utf16)
                        .sum::<usize>(),
                )
                .ok()
            })
            .collect::<Option<Vec<_>>>()?;
        let mut clusters = Vec::with_capacity(boundaries.len().saturating_sub(1));
        for (scalar, utf16) in boundaries.windows(2).zip(utf16_offsets.windows(2)) {
            let mut start_x = 0;
            let mut end_x = 0;
            let start_result = unsafe { ScriptStringCPtoX(analysis.0, utf16[0], 0, &mut start_x) };
            let end_result =
                unsafe { ScriptStringCPtoX(analysis.0, utf16[1].saturating_sub(1), 1, &mut end_x) };
            if start_result < 0 || end_result < 0 {
                return None;
            }
            clusters.push(NativeShapedTextCluster {
                start: scalar[0],
                end: scalar[1],
                start_x,
                end_x,
            });
        }
        let mut carets = Vec::with_capacity(boundaries.len());
        for (position, (index, utf16)) in boundaries
            .iter()
            .copied()
            .zip(utf16_offsets.iter().copied())
            .enumerate()
        {
            let mut leading = 0;
            let leading_result = if utf16 < character_count {
                unsafe { ScriptStringCPtoX(analysis.0, utf16, 0, &mut leading) }
            } else {
                unsafe {
                    ScriptStringCPtoX(
                        analysis.0,
                        character_count.saturating_sub(1),
                        1,
                        &mut leading,
                    )
                }
            };
            if leading_result < 0 {
                return None;
            }
            let mut trailing = leading;
            if position > 0 {
                let previous_utf16 = utf16_offsets[position - 1];
                let trailing_result = unsafe {
                    ScriptStringCPtoX(
                        analysis.0,
                        utf16.saturating_sub(1).max(previous_utf16),
                        1,
                        &mut trailing,
                    )
                };
                if trailing_result < 0 {
                    return None;
                }
            }
            carets.push(NativeShapedTextCaret {
                index,
                primary_x: leading,
                secondary_x: if position == 0 { leading } else { trailing },
            });
        }
        let width = unsafe { (*size).cx };
        NativeShapedTextLine::new(width, clusters, carets)
    })
}

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
struct WindowsGdiOwnedMemoryDc(HDC);

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
impl WindowsGdiOwnedMemoryDc {
    fn new() -> Option<Self> {
        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        (!dc.is_null()).then_some(Self(dc))
    }
}

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
impl Drop for WindowsGdiOwnedMemoryDc {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DeleteDC(self.0);
            }
        }
    }
}

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
struct WindowsUniscribeAnalysis(*mut c_void);

#[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
impl Drop for WindowsUniscribeAnalysis {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ScriptStringFree(&mut self.0);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsGdiPalette {
    pub primary_text: Color,
    pub secondary_text: Color,
    pub disabled_text: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub control: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl WindowsGdiPalette {
    pub const fn resolve(self, role: ColorRole) -> Color {
        match role {
            ColorRole::PrimaryText => self.primary_text,
            ColorRole::SecondaryText => self.secondary_text,
            ColorRole::DisabledText => self.disabled_text,
            ColorRole::Accent => self.accent,
            ColorRole::AccentText => self.accent_text,
            ColorRole::Surface => self.surface,
            ColorRole::SurfaceRaised => self.surface_raised,
            ColorRole::Control => self.control,
            ColorRole::Border => self.border,
            ColorRole::Success => self.success,
            ColorRole::Warning => self.warning,
            ColorRole::Danger => self.danger,
        }
    }

    pub const fn resolve_fill(self, fill: NativeDrawFill) -> Color {
        self.resolve_fill_with_contrast(fill, false)
    }

    pub const fn resolve_fill_with_contrast(
        self,
        fill: NativeDrawFill,
        high_contrast: bool,
    ) -> Color {
        match fill {
            NativeDrawFill::Color(color) => color,
            NativeDrawFill::Role(role) => self.resolve(role),
            NativeDrawFill::RoleWithAlpha { role, alpha } => {
                let alpha = if high_contrast {
                    high_contrast_alpha(alpha)
                } else {
                    alpha
                };
                blend_color(self.resolve(role), self.surface, alpha)
            }
        }
    }
}

const fn high_contrast_alpha(alpha: u8) -> u8 {
    match alpha {
        0 => 0,
        1..=20 => 64,
        21..=63 => 112,
        alpha => alpha,
    }
}

const fn blend_color(foreground: Color, background: Color, alpha: u8) -> Color {
    const fn channel(foreground: u8, background: u8, alpha: u8) -> u8 {
        let alpha = alpha as u32;
        (((foreground as u32 * alpha) + (background as u32 * (255 - alpha)) + 127) / 255) as u8
    }

    Color {
        r: channel(foreground.r, background.r, alpha),
        g: channel(foreground.g, background.g, alpha),
        b: channel(foreground.b, background.b, alpha),
        a: 255,
    }
}

impl Default for WindowsGdiPalette {
    fn default() -> Self {
        Self::from_theme(&crate::ZsuiTheme::light())
    }
}

impl WindowsGdiPalette {
    pub fn from_theme(theme: &crate::ZsuiTheme) -> Self {
        Self {
            primary_text: theme.colors.text_primary,
            secondary_text: theme.colors.text_secondary,
            disabled_text: blend_color(theme.colors.text_secondary, theme.colors.surface, 96),
            accent: theme.colors.accent,
            accent_text: theme.colors.accent_text,
            surface: theme.colors.surface,
            surface_raised: theme.colors.surface_raised,
            control: theme.colors.control,
            border: theme.colors.border,
            success: theme.colors.success,
            warning: theme.colors.warning,
            danger: theme.colors.danger,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowsGdiStyleResolver {
    pub font_family: String,
    pub icon_font_family: String,
    pub palette: WindowsGdiPalette,
}

impl WindowsGdiStyleResolver {
    pub fn new(font_family: impl Into<String>, palette: WindowsGdiPalette) -> Self {
        Self {
            font_family: font_family.into(),
            icon_font_family: WINDOWS_FLUENT_ICON_FONT_FAMILY.to_string(),
            palette,
        }
    }

    pub fn with_icon_font_family(mut self, font_family: impl Into<String>) -> Self {
        self.icon_font_family = font_family.into();
        self
    }
}

impl Default for WindowsGdiStyleResolver {
    fn default() -> Self {
        Self::new("Segoe UI Variable Text", WindowsGdiPalette::default())
    }
}

impl NativeStyleResolver for WindowsGdiStyleResolver {
    fn resolve_text_style(&self, style: SemanticTextStyle) -> TextStyle {
        let size = match style.role {
            crate::TextRole::Body => 14.0,
            crate::TextRole::Caption => 12.0,
            crate::TextRole::BodyLarge => 18.0,
            crate::TextRole::Subtitle => 20.0,
            crate::TextRole::Title => 28.0,
            crate::TextRole::Display => 44.0,
            crate::TextRole::Button => 14.0,
            crate::TextRole::Icon => 16.0,
            crate::TextRole::Monospace => 13.0,
        };
        TextStyle {
            font_family: match style.role {
                crate::TextRole::Monospace => "Consolas".to_string(),
                crate::TextRole::Icon => self.icon_font_family.clone(),
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
    system_icon_font: WindowsSystemIconFont,
    operation_log: Vec<NativeDrawCommandOperation>,
    high_contrast: bool,
}

impl WindowsGdiDrawSink {
    pub fn new(dc: HDC) -> Self {
        Self::with_palette(dc, WindowsGdiPalette::default())
    }

    pub fn with_palette(dc: HDC, palette: WindowsGdiPalette) -> Self {
        Self::with_palette_and_contrast(dc, palette, false)
    }

    pub fn with_palette_and_contrast(
        dc: HDC,
        palette: WindowsGdiPalette,
        high_contrast: bool,
    ) -> Self {
        let system_icon_font = detect_windows_system_icon_font(dc);
        let icon_font_family = system_icon_font
            .font_family()
            .unwrap_or(WINDOWS_MDL2_ICON_FONT_FAMILY);
        Self {
            renderer: WindowsGdiRenderer::new(dc),
            palette,
            style_resolver: WindowsGdiStyleResolver::new("Segoe UI Variable Text", palette)
                .with_icon_font_family(icon_font_family),
            system_icon_font,
            operation_log: Vec::new(),
            high_contrast,
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
        let fill = self
            .palette
            .resolve_fill_with_contrast(fill, self.high_contrast);
        let stroke_color = stroke.map(|stroke| {
            self.palette
                .resolve_fill_with_contrast(stroke, self.high_contrast)
        });
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
        if command.color_mode == NativeIconColorMode::Original && self.draw_original_icon(command) {
            return;
        }
        if self.draw_system_icon(command) {
            return;
        }
        let _ = self.draw_original_icon(command);
    }

    fn draw_original_icon(&mut self, command: &NativeDrawIconCommand) -> bool {
        let Some(bytes) = command.icon.png_24_bytes() else {
            return false;
        };
        let Some((width, height, bgra)) = decode_png_to_bgra(bytes) else {
            return false;
        };
        stretch_top_down_32bpp(self.renderer.hdc(), command.bounds, width, height, &bgra);
        true
    }

    fn draw_system_icon(&mut self, command: &NativeDrawIconCommand) -> bool {
        let Some(font_family) = self.system_icon_font.font_family() else {
            return false;
        };
        let Some(glyph) = self.system_icon_font.glyph(command.icon) else {
            return false;
        };
        let size = command.bounds.width.min(command.bounds.height).max(1) as f32;
        let style = TextStyle {
            font_family: font_family.to_string(),
            size,
            weight: TextWeight::Regular,
            color: self.palette.resolve(command.color),
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: false,
        };
        self.renderer.draw_text(
            &TextRun {
                text: glyph.to_string(),
                bounds: command.bounds,
            },
            &style,
        );
        true
    }
}

fn detect_windows_system_icon_font(dc: HDC) -> WindowsSystemIconFont {
    if dc.is_null() {
        return WindowsSystemIconFont::Unavailable;
    }
    if let Some(font) = WINDOWS_SYSTEM_ICON_FONT.get() {
        return *font;
    }
    let selected = select_windows_system_icon_font(
        windows_gdi_font_family_available(dc, WINDOWS_FLUENT_ICON_FONT_FAMILY),
        windows_gdi_font_family_available(dc, WINDOWS_MDL2_ICON_FONT_FAMILY),
    );
    let _ = WINDOWS_SYSTEM_ICON_FONT.set(selected);
    selected
}

fn windows_gdi_font_family_available(dc: HDC, family: &str) -> bool {
    if dc.is_null() {
        return false;
    }
    let style = TextStyle::line(family, 16.0, Color::rgb(0, 0, 0));
    let Some(font) = WindowsGdiOwnedObject::font(&style) else {
        return false;
    };
    let Some(_selected) = WindowsGdiSelectedObject::select(dc, font.object()) else {
        return false;
    };
    let mut selected_family = [0_u16; 64];
    let length = unsafe {
        GetTextFaceW(
            dc,
            selected_family.len() as i32,
            selected_family.as_mut_ptr(),
        )
    };
    if length <= 0 {
        return false;
    }
    let end = selected_family
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(selected_family.len());
    String::from_utf16_lossy(&selected_family[..end]).eq_ignore_ascii_case(family)
}

impl NativeDrawCommandSink for WindowsGdiDrawSink {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        self.operation_log.push(command.operation());
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                self.renderer.fill_rect(
                    *rect,
                    self.palette
                        .resolve_fill_with_contrast(*fill, self.high_contrast),
                );
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => self.renderer.stroke_rect(
                *rect,
                self.palette
                    .resolve_fill_with_contrast(*stroke, self.high_contrast),
                *width,
            ),
            NativeDrawCommand::StrokeArc {
                rect,
                stroke,
                width,
                start_degrees,
                sweep_degrees,
            } => self.renderer.stroke_arc(
                *rect,
                self.palette
                    .resolve_fill_with_contrast(*stroke, self.high_contrast),
                *width,
                *start_degrees,
                *sweep_degrees,
            ),
            NativeDrawCommand::FillTriangle { points, fill } => {
                if self.renderer.hdc().is_null() {
                    return;
                }
                let Some(brush) = WindowsGdiOwnedObject::solid_brush(
                    self.palette
                        .resolve_fill_with_contrast(*fill, self.high_contrast),
                ) else {
                    return;
                };
                let _selected_brush =
                    WindowsGdiSelectedObject::select(self.renderer.hdc(), brush.object());
                let _selected_pen = WindowsGdiSelectedObject::select(self.renderer.hdc(), unsafe {
                    GetStockObject(NULL_PEN)
                });
                let points = points.map(|point| POINT {
                    x: point.x,
                    y: point.y,
                });
                unsafe {
                    Polygon(self.renderer.hdc(), points.as_ptr(), points.len() as i32);
                }
            }
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
            #[cfg(feature = "password-box")]
            NativeDrawCommand::SecureText(command) => {
                let rendered = command.rendered_text();
                self.draw_text_command(&NativeDrawTextCommand::new(
                    rendered.as_str(),
                    command.bounds,
                    command.style,
                ));
            }
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
        TextWeight::Semibold => 600,
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

    #[cfg(all(feature = "text-input-core", feature = "windows-win32"))]
    #[test]
    fn uniscribe_shaping_reports_proportional_and_bidirectional_geometry() {
        let proportional = shape_windows_gdi_text_line("iiiiWW")
            .expect("Uniscribe should shape Segoe UI text on Windows");
        assert_eq!(proportional.clusters.len(), 6);
        assert!(
            proportional.clusters[0].end_x - proportional.clusters[0].start_x
                < proportional.clusters[4].end_x - proportional.clusters[4].start_x
        );

        let mixed = shape_windows_gdi_text_line("abc אבג 123")
            .expect("Uniscribe should shape mixed-direction text");
        assert!(
            mixed
                .clusters
                .iter()
                .any(|cluster| cluster.start_x > cluster.end_x),
            "RTL clusters should retain their visual direction"
        );
        assert!(
            mixed
                .carets
                .iter()
                .any(|caret| caret.primary_x != caret.secondary_x),
            "a bidi boundary should expose primary and secondary caret positions"
        );

        let visual = shape_windows_gdi_text_line("abאב")
            .expect("Uniscribe should expose visual caret order");
        let mut visual_carets = visual
            .carets
            .iter()
            .map(|caret| (caret.primary_x, caret.index))
            .collect::<Vec<_>>();
        visual_carets.sort_unstable();
        assert_eq!(
            visual_carets
                .into_iter()
                .map(|(_, index)| index)
                .collect::<Vec<_>>(),
            vec![0, 1, 4, 3, 2]
        );
    }

    #[test]
    fn icon_text_role_uses_fluent_icon_font() {
        let style = WindowsGdiStyleResolver::default().resolve_text_style(SemanticTextStyle {
            role: crate::TextRole::Icon,
            color: ColorRole::PrimaryText,
            weight: TextWeight::Regular,
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: false,
        });

        assert_eq!(style.font_family, "Segoe Fluent Icons");
        assert_eq!(style.size, 16.0);
    }

    #[test]
    fn system_icon_font_prefers_fluent_then_mdl2() {
        assert_eq!(
            select_windows_system_icon_font(true, true),
            WindowsSystemIconFont::SegoeFluentIcons
        );
        assert_eq!(
            select_windows_system_icon_font(false, true),
            WindowsSystemIconFont::SegoeMdl2Assets
        );
        assert_eq!(
            select_windows_system_icon_font(false, false),
            WindowsSystemIconFont::Unavailable
        );
        assert_eq!(
            WindowsSystemIconFont::SegoeMdl2Assets.font_family(),
            Some(WINDOWS_MDL2_ICON_FONT_FAMILY)
        );
        assert_eq!(
            WindowsSystemIconFont::SegoeMdl2Assets.glyph(ZsIcon::Save),
            Some(ZsIcon::Save.windows_mdl2_glyph())
        );
    }

    #[test]
    fn icon_text_role_accepts_detected_mdl2_font() {
        let style = WindowsGdiStyleResolver::default()
            .with_icon_font_family(WINDOWS_MDL2_ICON_FONT_FAMILY)
            .resolve_text_style(SemanticTextStyle {
                role: crate::TextRole::Icon,
                color: ColorRole::PrimaryText,
                weight: TextWeight::Regular,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: VerticalAlign::Center,
                wrap: TextWrap::NoWrap,
                ellipsis: false,
            });

        assert_eq!(style.font_family, WINDOWS_MDL2_ICON_FONT_FAMILY);
    }

    #[test]
    fn fluent_type_ramp_uses_windows_11_sizes() {
        let resolver = WindowsGdiStyleResolver::default();
        let mut semantic = SemanticTextStyle::body();

        assert_eq!(resolver.resolve_text_style(semantic).size, 14.0);
        semantic.role = crate::TextRole::Caption;
        assert_eq!(resolver.resolve_text_style(semantic).size, 12.0);
        semantic.role = crate::TextRole::Subtitle;
        assert_eq!(resolver.resolve_text_style(semantic).size, 20.0);
        semantic.role = crate::TextRole::Title;
        assert_eq!(resolver.resolve_text_style(semantic).size, 28.0);
    }

    #[test]
    fn gdi_palette_is_resolved_from_shared_theme_tokens() {
        let theme = crate::ZsuiTheme::light();
        let palette = WindowsGdiPalette::from_theme(&theme);

        assert_eq!(palette.surface, theme.colors.surface);
        assert_eq!(palette.surface_raised, theme.colors.surface_raised);
        assert_eq!(palette.border, theme.colors.border);
        assert_eq!(palette.accent_text, theme.colors.accent_text);
    }

    #[test]
    fn role_alpha_is_composited_before_gdi_drops_alpha() {
        let palette = WindowsGdiPalette::default();

        assert_eq!(
            palette.resolve_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 0,
            }),
            palette.surface
        );
        assert_eq!(
            palette.resolve_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 255,
            }),
            palette.accent
        );
        let subtle = palette.resolve_fill(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Accent,
            alpha: 32,
        });
        assert!(subtle.r < palette.surface.r);
        assert!(subtle.b > palette.surface.b - 12);
        assert_eq!(subtle.a, 255);

        let high_contrast_hover = palette.resolve_fill_with_contrast(
            NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: 14,
            },
            true,
        );
        assert_eq!(high_contrast_hover, Color::rgb(189, 189, 189));
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
