//! Optional ZSUI-owned Rust text pipeline.
//!
//! The production module owns shaping, compact layout and pixel rasterization.
//! Framework-development proof output is compiled only by `rust-text-proof`.
//! Platform hosts may composite the result into their native surface without
//! exposing raw handles through this API.

use std::{collections::HashMap, fmt, mem::size_of, ops::Range, sync::Arc};

use cosmic_text::{
    fontdb, Align, Attrs, Buffer, CacheKey, CacheKeyFlags, Family, FontSystem, LayoutGlyph,
    Metrics, Shaping, Weight, Wrap,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "rust-text-proof")]
use swash::zeno::{Command, PathData as _};
use swash::{
    scale::{
        image::{Content, Image},
        Render, ScaleContext, Source, StrikeWith,
    },
    zeno::{Angle, Format, Transform, Vector},
    Setting, Tag,
};

use crate::{Color, HorizontalAlign, Size, TextStyle, TextWeight, TextWrap, VerticalAlign};

#[cfg(feature = "rust-text-proof")]
const TEXT_PROOF_SCHEMA: &str = "zsui.text-proof/v1";
const DEFAULT_GLYPH_CACHE_LIMIT: usize = 2_048;
const DEFAULT_GLYPH_CACHE_BYTE_LIMIT: usize = 4 * 1024 * 1024;
const DEFAULT_LAYOUT_CACHE_LIMIT: usize = 256;
const DEFAULT_LAYOUT_CACHE_BYTE_LIMIT: usize = 2 * 1024 * 1024;

/// Line-baseline policy is platform-owned even when shaping and rasterization
/// are shared. It prevents a portable renderer from imposing one platform's
/// vertical typography convention on every desktop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsTextLineMetricPolicy {
    /// Centers the font's glyph box inside the requested line height.
    #[default]
    CenteredGlyphBox,
    /// Uses ascent/(ascent+descent), matching DirectWrite uniform line spacing.
    WindowsDirectWrite,
}

/// Pixel mask requested from the ZSUI Rust rasterizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsTextRasterMode {
    /// One coverage value per pixel. Suitable for high-DPI and rotated output.
    Grayscale,
    /// Independent red, green and blue coverage values for an RGB stripe panel.
    SubpixelRgb,
}

/// Explicit compositor calibration. Values are recorded in proof output so a
/// visual baseline never silently depends on hidden tuning constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ZsTextRasterProfile {
    pub mode: ZsTextRasterMode,
    pub gamma: f32,
    pub contrast: f32,
}

/// Physical-pixel clip used when a backend composites a retained text layout
/// directly into a larger native software surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsTextPixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ZsTextPixelRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn full(width: u32, height: u32) -> Self {
        Self::new(0, 0, width, height)
    }

    fn clamped_edges(self, width: u32, height: u32) -> (i32, i32, i32, i32) {
        let surface_right = i32::try_from(width).unwrap_or(i32::MAX);
        let surface_bottom = i32::try_from(height).unwrap_or(i32::MAX);
        let right = i64::from(self.x)
            .saturating_add(i64::from(self.width))
            .clamp(0, i64::from(surface_right)) as i32;
        let bottom = i64::from(self.y)
            .saturating_add(i64::from(self.height))
            .clamp(0, i64::from(surface_bottom)) as i32;
        (
            self.x.clamp(0, surface_right),
            self.y.clamp(0, surface_bottom),
            right,
            bottom,
        )
    }
}

impl ZsTextRasterProfile {
    pub const fn grayscale() -> Self {
        Self {
            mode: ZsTextRasterMode::Grayscale,
            gamma: 1.0,
            contrast: 1.0,
        }
    }

    pub const fn subpixel_rgb() -> Self {
        Self {
            mode: ZsTextRasterMode::SubpixelRgb,
            gamma: 1.0,
            contrast: 1.0,
        }
    }

    fn adjusted_coverage(self, value: u8) -> u8 {
        let normalized = f32::from(value) / 255.0;
        let gamma = if self.gamma.is_finite() {
            self.gamma.clamp(0.1, 4.0)
        } else {
            1.0
        };
        let contrast = if self.contrast.is_finite() {
            self.contrast.clamp(0.0, 4.0)
        } else {
            1.0
        };
        (normalized
            .powf(gamma)
            .mul_add(contrast, 0.0)
            .clamp(0.0, 1.0)
            * 255.0)
            .round() as u8
    }
}

/// Stable, serializable glyph geometry emitted by the Rust text pipeline.
#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTextGlyphProof {
    pub cluster_start: usize,
    pub cluster_end: usize,
    pub glyph_id: u16,
    pub font_family: String,
    pub postscript_name: String,
    pub face_index: u32,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    /// Shaper placement along the advance axis before origin composition.
    pub offset_x_px: f32,
    /// Shaper placement toward the font ascender before origin composition.
    pub offset_y_px: f32,
    pub advance_px: f32,
    pub font_size_px: f32,
    pub rtl: bool,
}

#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTextLineProof {
    pub line_index: usize,
    pub top_px: f32,
    pub baseline_px: f32,
    pub height_px: f32,
    pub width_px: f32,
    pub rtl: bool,
    pub glyphs: Vec<ZsTextGlyphProof>,
}

/// Deterministic geometry layer used before pixel comparison.
#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTextProof {
    pub schema: String,
    pub backend: String,
    pub requested_font_family: String,
    pub dpi_scale: f32,
    pub width_px: i32,
    pub height_px: i32,
    pub content_width_px: i32,
    pub content_height_px: i32,
    pub overflow_x: bool,
    pub overflow_y: bool,
    pub lines: Vec<ZsTextLineProof>,
    pub errors: Vec<String>,
}

/// Geometry comparison is intentionally separate from pixel comparison.
/// Glyph IDs are meaningful only when both backends resolved the same face.
#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTextGeometryDiff {
    pub reference_backend: String,
    pub candidate_backend: String,
    pub line_count_equal: bool,
    pub glyph_count_equal: bool,
    pub compared_glyphs: usize,
    pub matching_glyph_ids: usize,
    pub matching_font_faces: usize,
    pub max_origin_delta_px: f32,
    pub max_advance_delta_px: f32,
    pub tolerance_px: f32,
    pub within_tolerance: bool,
}

#[cfg(feature = "rust-text-proof")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsTextPixelDiff {
    pub pixel_count: usize,
    pub different_pixel_count: usize,
    pub different_pixel_ratio: f64,
    pub mean_absolute_channel_delta: f64,
    pub max_channel_delta: u8,
    pub channel_threshold: u8,
}

#[cfg(feature = "rust-text-proof")]
pub fn compare_text_bgra_pixels(
    reference: &[u8],
    candidate: &[u8],
    channel_threshold: u8,
) -> Result<ZsTextPixelDiff, String> {
    if reference.len() != candidate.len() || reference.len() % 4 != 0 {
        return Err("pixel proof buffers must have equal BGRA8 lengths".into());
    }
    let pixel_count = reference.len() / 4;
    let mut different_pixel_count = 0usize;
    let mut total_delta = 0u64;
    let mut max_channel_delta = 0u8;
    for (reference, candidate) in reference.chunks_exact(4).zip(candidate.chunks_exact(4)) {
        let mut pixel_max = 0u8;
        for channel in 0..3 {
            let delta = reference[channel].abs_diff(candidate[channel]);
            total_delta = total_delta.saturating_add(u64::from(delta));
            pixel_max = pixel_max.max(delta);
            max_channel_delta = max_channel_delta.max(delta);
        }
        different_pixel_count += usize::from(pixel_max > channel_threshold);
    }
    Ok(ZsTextPixelDiff {
        pixel_count,
        different_pixel_count,
        different_pixel_ratio: if pixel_count == 0 {
            0.0
        } else {
            different_pixel_count as f64 / pixel_count as f64
        },
        mean_absolute_channel_delta: if pixel_count == 0 {
            0.0
        } else {
            total_delta as f64 / (pixel_count * 3) as f64
        },
        max_channel_delta,
        channel_threshold,
    })
}

#[cfg(feature = "rust-text-proof")]
pub fn text_bgra_difference_image(reference: &[u8], candidate: &[u8]) -> Result<Vec<u8>, String> {
    if reference.len() != candidate.len() || reference.len() % 4 != 0 {
        return Err("pixel proof buffers must have equal BGRA8 lengths".into());
    }
    let mut difference = Vec::with_capacity(reference.len());
    for (reference, candidate) in reference.chunks_exact(4).zip(candidate.chunks_exact(4)) {
        let delta = (0..3)
            .map(|channel| reference[channel].abs_diff(candidate[channel]))
            .max()
            .unwrap_or(0);
        difference.extend_from_slice(&[
            255u8.saturating_sub(delta),
            255u8.saturating_sub(delta),
            255,
            255,
        ]);
    }
    Ok(difference)
}

#[cfg(feature = "rust-text-proof")]
pub fn compare_text_geometry(
    reference: &ZsTextProof,
    candidate: &ZsTextProof,
    tolerance_px: f32,
) -> ZsTextGeometryDiff {
    let tolerance = if tolerance_px.is_finite() {
        tolerance_px.max(0.0)
    } else {
        0.0
    };
    let reference_glyphs = reference
        .lines
        .iter()
        .flat_map(|line| {
            line.glyphs.iter().map(|glyph| ProofGlyphAtLine {
                line_index: line.line_index,
                glyph,
            })
        })
        .collect::<Vec<_>>();
    let candidate_glyphs = candidate
        .lines
        .iter()
        .flat_map(|line| {
            line.glyphs.iter().map(|glyph| ProofGlyphAtLine {
                line_index: line.line_index,
                glyph,
            })
        })
        .collect::<Vec<_>>();
    let compared = reference_glyphs.len().min(candidate_glyphs.len());
    let matching_glyph_ids =
        pair_proof_glyphs(&reference_glyphs, &candidate_glyphs, |left, right| {
            same_cluster(left, right) && left.glyph.glyph_id == right.glyph.glyph_id
        })
        .len();
    let matching_font_faces =
        pair_proof_glyphs(&reference_glyphs, &candidate_glyphs, |left, right| {
            same_cluster(left, right)
                && left.glyph.postscript_name == right.glyph.postscript_name
                && left.glyph.face_index == right.glyph.face_index
        })
        .len();
    // Only compare placement for the same logical cluster, glyph and face.
    // Zipping globally makes one omitted trailing-space glyph shift every
    // subsequent comparison and turns a local wrap difference into hundreds
    // of pixels of fictitious geometry drift.
    let geometry_pairs = pair_proof_glyphs(&reference_glyphs, &candidate_glyphs, |left, right| {
        same_cluster(left, right)
            && left.glyph.glyph_id == right.glyph.glyph_id
            && left.glyph.postscript_name == right.glyph.postscript_name
            && left.glyph.face_index == right.glyph.face_index
    });
    let geometry_match_count = geometry_pairs.len();
    let mut max_origin_delta_px = 0.0f32;
    let mut max_advance_delta_px = 0.0f32;
    for (reference, candidate) in geometry_pairs {
        let dx = reference.origin_x_px - candidate.origin_x_px;
        let dy = reference.origin_y_px - candidate.origin_y_px;
        max_origin_delta_px = max_origin_delta_px.max(dx.hypot(dy));
        max_advance_delta_px =
            max_advance_delta_px.max((reference.advance_px - candidate.advance_px).abs());
    }
    let line_count_equal = reference.lines.len() == candidate.lines.len();
    let reference_count = reference
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    let candidate_count = candidate
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    let glyph_count_equal = reference_count == candidate_count;
    ZsTextGeometryDiff {
        reference_backend: reference.backend.clone(),
        candidate_backend: candidate.backend.clone(),
        line_count_equal,
        glyph_count_equal,
        compared_glyphs: compared,
        matching_glyph_ids,
        matching_font_faces,
        max_origin_delta_px,
        max_advance_delta_px,
        tolerance_px: tolerance,
        within_tolerance: line_count_equal
            && glyph_count_equal
            && matching_glyph_ids == compared
            && matching_font_faces == compared
            && geometry_match_count == compared
            && max_origin_delta_px <= tolerance
            && max_advance_delta_px <= tolerance,
    }
}

#[cfg(feature = "rust-text-proof")]
#[derive(Clone, Copy)]
struct ProofGlyphAtLine<'a> {
    line_index: usize,
    glyph: &'a ZsTextGlyphProof,
}

#[cfg(feature = "rust-text-proof")]
fn same_cluster(left: &ProofGlyphAtLine<'_>, right: &ProofGlyphAtLine<'_>) -> bool {
    left.line_index == right.line_index
        && left.glyph.cluster_start == right.glyph.cluster_start
        && left.glyph.cluster_end == right.glyph.cluster_end
}

#[cfg(feature = "rust-text-proof")]
fn pair_proof_glyphs<'a>(
    reference: &[ProofGlyphAtLine<'a>],
    candidate: &[ProofGlyphAtLine<'a>],
    matches: impl Fn(&ProofGlyphAtLine<'_>, &ProofGlyphAtLine<'_>) -> bool,
) -> Vec<(&'a ZsTextGlyphProof, &'a ZsTextGlyphProof)> {
    let mut used = vec![false; candidate.len()];
    let mut pairs = Vec::with_capacity(reference.len().min(candidate.len()));
    for reference_glyph in reference {
        let best = candidate
            .iter()
            .enumerate()
            .filter(|(index, candidate_glyph)| {
                !used[*index] && matches(reference_glyph, candidate_glyph)
            })
            .min_by(|(_, left), (_, right)| {
                proof_origin_distance(reference_glyph.glyph, left.glyph)
                    .total_cmp(&proof_origin_distance(reference_glyph.glyph, right.glyph))
            })
            .map(|(index, _)| index);
        if let Some(index) = best {
            used[index] = true;
            pairs.push((reference_glyph.glyph, candidate[index].glyph));
        }
    }
    pairs
}

#[cfg(feature = "rust-text-proof")]
fn proof_origin_distance(left: &ZsTextGlyphProof, right: &ZsTextGlyphProof) -> f32 {
    let dx = left.origin_x_px - right.origin_x_px;
    let dy = left.origin_y_px - right.origin_y_px;
    dx.mul_add(dx, dy * dy)
}

/// Blue is the reference geometry, magenta is the candidate. Origins are
/// circles and advances are horizontal ticks, so shaping/placement drift is
/// visible without conflating it with antialiasing.
#[cfg(feature = "rust-text-proof")]
pub fn text_geometry_overlay_svg(reference: &ZsTextProof, candidate: &ZsTextProof) -> String {
    let width = reference
        .width_px
        .max(reference.content_width_px)
        .max(candidate.width_px)
        .max(candidate.content_width_px)
        .max(1);
    let height = reference
        .height_px
        .max(reference.content_height_px)
        .max(candidate.height_px)
        .max(candidate.content_height_px)
        .max(1);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">\n<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n"
    );
    append_geometry_layer(&mut svg, reference, "#0A84FF");
    append_geometry_layer(&mut svg, candidate, "#D000D0");
    svg.push_str("</svg>\n");
    svg
}

#[cfg(feature = "rust-text-proof")]
fn append_geometry_layer(svg: &mut String, proof: &ZsTextProof, color: &str) {
    for line in &proof.lines {
        svg.push_str(&format!(
            "<line x1=\"0\" y1=\"{:.3}\" x2=\"{}\" y2=\"{:.3}\" stroke=\"{color}\" stroke-opacity=\"0.22\" stroke-width=\"0.5\"/>\n",
            line.baseline_px,
            proof.width_px.max(proof.content_width_px).max(1),
            line.baseline_px
        ));
        for glyph in &line.glyphs {
            svg.push_str(&format!(
                "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"1.25\" fill=\"none\" stroke=\"{color}\" stroke-width=\"0.65\"/><line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"{color}\" stroke-width=\"0.65\"/>\n",
                glyph.origin_x_px,
                glyph.origin_y_px,
                glyph.origin_x_px,
                glyph.origin_y_px + 2.0,
                glyph.origin_x_px + glyph.advance_px,
                glyph.origin_y_px + 2.0
            ));
        }
    }
}

/// Compact shaped glyph geometry retained by production layouts.
#[derive(Debug, Clone)]
pub struct ZsTextGlyphLayout {
    pub cluster_start: usize,
    pub cluster_end: usize,
    pub glyph_id: u16,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub advance_px: f32,
    pub font_size_px: f32,
    pub rtl: bool,
    key: CacheKey,
    synthetic_bold_px: f32,
    raster_origin_x_px: i32,
    raster_origin_y_px: i32,
}

/// One retained line referring to a contiguous range in the layout glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct ZsTextLineLayout {
    pub line_index: usize,
    pub top_px: f32,
    pub baseline_px: f32,
    pub height_px: f32,
    pub width_px: f32,
    pub rtl: bool,
    pub glyph_range: Range<usize>,
}

/// Owned result reused by measure, paint, caret/proof and outline export.
#[derive(Debug, Clone)]
pub struct ZsRustTextLayout {
    size: Size,
    overflow_x: bool,
    overflow_y: bool,
    #[cfg(feature = "rust-text-proof")]
    proof: ZsTextProof,
    lines: Vec<ZsTextLineLayout>,
    glyphs: Vec<ZsTextGlyphLayout>,
}

impl ZsRustTextLayout {
    pub const fn size(&self) -> Size {
        self.size
    }

    #[cfg(feature = "rust-text-proof")]
    pub const fn proof(&self) -> &ZsTextProof {
        &self.proof
    }

    #[cfg(feature = "rust-text-proof")]
    pub fn proof_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.proof)
    }

    pub const fn overflows_horizontally(&self) -> bool {
        self.overflow_x
    }

    pub const fn overflows_vertically(&self) -> bool {
        self.overflow_y
    }

    pub fn lines(&self) -> &[ZsTextLineLayout] {
        &self.lines
    }

    pub fn glyphs(&self) -> &[ZsTextGlyphLayout] {
        &self.glyphs
    }

    pub fn glyphs_for_line(&self, line_index: usize) -> Option<&[ZsTextGlyphLayout]> {
        let line = self
            .lines
            .iter()
            .find(|line| line.line_index == line_index)?;
        self.glyphs.get(line.glyph_range.clone())
    }
}

/// Bounded-cache telemetry for framework diagnostics and performance tests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ZsTextCacheStats {
    pub layout_hits: u64,
    pub layout_misses: u64,
    pub layout_evictions: u64,
    pub glyph_hits: u64,
    pub glyph_misses: u64,
    pub glyph_evictions: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ZsTextLayoutCacheKey {
    text: String,
    font_family: String,
    size_bits: u32,
    line_height_bits: u32,
    weight: u8,
    horizontal_align: u8,
    vertical_align: u8,
    wrap: u8,
    width_px: i32,
    height_px: i32,
    dpi_scale_bits: u32,
    line_metric_policy: u8,
}

impl ZsTextLayoutCacheKey {
    fn new(
        text: &str,
        style: &TextStyle,
        width_px: i32,
        height_px: i32,
        dpi_scale: f32,
        line_metric_policy: ZsTextLineMetricPolicy,
    ) -> Self {
        Self {
            text: text.to_owned(),
            font_family: style.font_family.clone(),
            size_bits: style.size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            weight: text_weight_key(style.weight),
            horizontal_align: horizontal_align_key(style.horizontal_align),
            vertical_align: vertical_align_key(style.vertical_align),
            wrap: text_wrap_key(style.wrap),
            width_px,
            height_px,
            dpi_scale_bits: normalized_scale(dpi_scale).to_bits(),
            line_metric_policy: line_metric_policy_key(line_metric_policy),
        }
    }

    fn estimated_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.text.capacity())
            .saturating_add(self.font_family.capacity())
    }
}

struct CachedTextLayout {
    layout: Arc<ZsRustTextLayout>,
    estimated_bytes: usize,
    last_used: u64,
}

struct CachedGlyphImage {
    image: Option<Arc<Image>>,
    bytes: usize,
    last_used: u64,
}

/// Pure Rust shaping, retained layout and glyph-raster state owned by ZSUI.
///
/// Create one per application or renderer cache. The glyph cache is bounded;
/// clearing it never invalidates an already-owned [`ZsRustTextLayout`].
pub struct ZsRustTextEngine {
    font_system: FontSystem,
    scale_context: ScaleContext,
    layout_cache: HashMap<ZsTextLayoutCacheKey, CachedTextLayout>,
    layout_cache_limit: usize,
    layout_cache_byte_limit: usize,
    layout_cache_bytes: usize,
    glyph_cache: HashMap<(CacheKey, ZsTextRasterMode, u32), CachedGlyphImage>,
    glyph_cache_limit: usize,
    glyph_cache_byte_limit: usize,
    glyph_cache_bytes: usize,
    cache_clock: u64,
    cache_stats: ZsTextCacheStats,
    line_metric_policy: ZsTextLineMetricPolicy,
}

impl fmt::Debug for ZsRustTextEngine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZsRustTextEngine")
            .field("locale", &self.font_system.locale())
            .field("layout_cache_len", &self.layout_cache.len())
            .field("layout_cache_bytes", &self.layout_cache_bytes)
            .field("glyph_cache_len", &self.glyph_cache.len())
            .field("glyph_cache_bytes", &self.glyph_cache_bytes)
            .field("glyph_cache_limit", &self.glyph_cache_limit)
            .field("cache_stats", &self.cache_stats)
            .field("line_metric_policy", &self.line_metric_policy)
            .finish_non_exhaustive()
    }
}

impl Default for ZsRustTextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ZsRustTextEngine {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            scale_context: ScaleContext::new(),
            layout_cache: HashMap::new(),
            layout_cache_limit: DEFAULT_LAYOUT_CACHE_LIMIT,
            layout_cache_byte_limit: DEFAULT_LAYOUT_CACHE_BYTE_LIMIT,
            layout_cache_bytes: 0,
            glyph_cache: HashMap::new(),
            glyph_cache_limit: DEFAULT_GLYPH_CACHE_LIMIT,
            glyph_cache_byte_limit: DEFAULT_GLYPH_CACHE_BYTE_LIMIT,
            glyph_cache_bytes: 0,
            cache_clock: 0,
            cache_stats: ZsTextCacheStats::default(),
            line_metric_policy: ZsTextLineMetricPolicy::CenteredGlyphBox,
        }
    }

    pub fn with_glyph_cache_limit(mut self, limit: usize) -> Self {
        self.glyph_cache_limit = limit.max(1);
        self.enforce_glyph_cache_limits();
        self
    }

    pub fn with_glyph_cache_byte_limit(mut self, limit: usize) -> Self {
        self.glyph_cache_byte_limit = limit.max(1);
        self.enforce_glyph_cache_limits();
        self
    }

    pub fn with_layout_cache_limits(mut self, entries: usize, bytes: usize) -> Self {
        self.layout_cache_limit = entries.max(1);
        self.layout_cache_byte_limit = bytes.max(1);
        self.enforce_layout_cache_limits();
        self
    }

    pub const fn with_line_metric_policy(mut self, policy: ZsTextLineMetricPolicy) -> Self {
        self.line_metric_policy = policy;
        self
    }

    pub const fn line_metric_policy(&self) -> ZsTextLineMetricPolicy {
        self.line_metric_policy
    }

    pub const fn glyph_cache_limit(&self) -> usize {
        self.glyph_cache_limit
    }

    pub fn glyph_cache_len(&self) -> usize {
        self.glyph_cache.len()
    }

    pub const fn glyph_cache_bytes(&self) -> usize {
        self.glyph_cache_bytes
    }

    pub fn layout_cache_len(&self) -> usize {
        self.layout_cache.len()
    }

    pub const fn layout_cache_bytes(&self) -> usize {
        self.layout_cache_bytes
    }

    pub const fn cache_stats(&self) -> ZsTextCacheStats {
        self.cache_stats
    }

    pub fn clear_glyph_cache(&mut self) {
        self.glyph_cache.clear();
        self.glyph_cache_bytes = 0;
    }

    pub fn clear_layout_cache(&mut self) {
        self.layout_cache.clear();
        self.layout_cache_bytes = 0;
    }

    /// Shapes and lays out one text box. Box dimensions and returned geometry
    /// are physical pixels; `TextStyle` sizes remain logical desktop pixels.
    pub fn layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        width_px: i32,
        height_px: i32,
        dpi_scale: f32,
    ) -> Arc<ZsRustTextLayout> {
        let cache_key = ZsTextLayoutCacheKey::new(
            text,
            style,
            width_px,
            height_px,
            dpi_scale,
            self.line_metric_policy,
        );
        if let Some(layout) = self.cached_layout(&cache_key) {
            return layout;
        }
        if text.is_empty() {
            let layout = Arc::new(empty_layout(style, width_px, height_px, dpi_scale));
            self.insert_layout(cache_key, Arc::clone(&layout));
            return layout;
        }

        let scale = normalized_scale(dpi_scale);
        let width_dp = (width_px > 0).then_some(width_px as f32 / scale);
        let height_dp = (height_px > 0).then_some(height_px as f32 / scale);
        let line_height = style.line_height.max(style.size).max(1.0);
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(style.size.max(1.0), line_height),
        );
        buffer.set_wrap(if style.wrap == TextWrap::Word {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        });
        buffer.set_size(
            width_dp,
            (style.wrap == TextWrap::Word)
                .then_some(height_dp)
                .flatten(),
        );
        let family = if style.font_family.eq_ignore_ascii_case("monospace") {
            Family::Monospace
        } else if style.font_family.trim().is_empty() {
            Family::SansSerif
        } else {
            Family::Name(&style.font_family)
        };
        let requested_weight = cosmic_weight(style.weight);
        // cosmic-text only treats a named family as its primary face when an
        // exact weight exists. DirectWrite instead selects the closest face
        // inside that family (for example Consolas 600 -> Consolas Bold), so
        // resolve the concrete face weight before shaping. Without this step,
        // a missing 600 face can incorrectly turn into a Segoe UI fallback.
        let resolved_weight =
            resolve_primary_family_weight(&self.font_system, family, requested_weight);
        let attrs = Attrs::new().family(family).weight(resolved_weight);
        let alignment = match style.horizontal_align {
            HorizontalAlign::Start => Some(Align::Left),
            HorizontalAlign::Center => Some(Align::Center),
            HorizontalAlign::End => Some(Align::Right),
        };
        buffer.set_text(text, &attrs, Shaping::Advanced, alignment);
        buffer.shape_until_scroll(&mut self.font_system, true);
        let trailing_space_template = (self.line_metric_policy
            == ZsTextLineMetricPolicy::WindowsDirectWrite
            && style.wrap == TextWrap::Word)
            .then(|| {
                shape_ascii_space_template(
                    &mut self.font_system,
                    Metrics::new(style.size.max(1.0), line_height),
                    &attrs,
                )
            })
            .flatten();

        let content_height_dp = buffer
            .layout_runs()
            .map(|run| run.line_top + run.line_height)
            .fold(0.0f32, f32::max);
        let box_height_dp = height_dp.unwrap_or(content_height_dp);
        let vertical_offset_dp = match style.vertical_align {
            VerticalAlign::Start => 0.0,
            VerticalAlign::Center => (box_height_dp - content_height_dp).max(0.0) / 2.0,
            VerticalAlign::End => (box_height_dp - content_height_dp).max(0.0),
        };

        #[cfg(feature = "rust-text-proof")]
        let mut proof_lines = Vec::new();
        let mut lines = Vec::new();
        let mut glyphs = Vec::new();
        let mut corrected_content_width_dp = 0.0f32;
        let line_byte_offsets = text_line_byte_offsets(text);
        let mut windows_paragraph_baseline_offsets = HashMap::<usize, f32>::new();
        for (visual_line_index, run) in buffer.layout_runs().enumerate() {
            let line_byte_offset = line_byte_offsets.get(run.line_i).copied().unwrap_or(0);
            let trailing_space_ranges = trailing_space_template
                .as_ref()
                .filter(|_| !run.rtl)
                .map_or_else(Vec::new, |_| {
                    omitted_trailing_ascii_space_ranges(run.text, run.glyphs)
                });
            let trailing_space_width_dp = trailing_space_template
                .as_ref()
                .map_or(0.0, |glyph| glyph.w * trailing_space_ranges.len() as f32);
            let glyph_start = glyphs.len();
            let line_baseline_dp = match self.line_metric_policy {
                ZsTextLineMetricPolicy::CenteredGlyphBox => run.line_y,
                ZsTextLineMetricPolicy::WindowsDirectWrite => {
                    let candidate = windows_line_baseline(
                        &mut self.font_system,
                        run.glyphs,
                        run.line_top,
                        run.line_height,
                    )
                    .unwrap_or(run.line_y);
                    let baseline_offset = windows_paragraph_baseline_offsets
                        .entry(run.line_i)
                        .or_insert(candidate - run.line_top);
                    run.line_top + *baseline_offset
                }
            };
            let baseline_px = (vertical_offset_dp + line_baseline_dp) * scale;
            let raster_baseline_px = match self.line_metric_policy {
                ZsTextLineMetricPolicy::CenteredGlyphBox => baseline_px,
                ZsTextLineMetricPolicy::WindowsDirectWrite => baseline_px.round(),
            };
            let mut placement_corrections =
                if self.line_metric_policy == ZsTextLineMetricPolicy::WindowsDirectWrite {
                    windows_gpos_kern_corrections(&mut self.font_system, run.glyphs)
                } else {
                    vec![ZsGlyphPlacementCorrection::default(); run.glyphs.len()]
                };
            if self.line_metric_policy == ZsTextLineMetricPolicy::WindowsDirectWrite {
                for (glyph, correction) in run.glyphs.iter().zip(&mut placement_corrections) {
                    let cluster_start = line_byte_offset.saturating_add(glyph.start);
                    let cluster_end = line_byte_offset.saturating_add(glyph.end);
                    let cluster_is_whitespace =
                        text.get(cluster_start..cluster_end).is_some_and(|cluster| {
                            !cluster.is_empty() && cluster.chars().all(char::is_whitespace)
                        });
                    let strength = windows_synthetic_bold_strength_dp(
                        &mut self.font_system,
                        glyph,
                        requested_weight,
                        cluster_is_whitespace,
                    );
                    correction.synthetic_bold_dp = strength;
                    correction.x_advance_dp += strength;
                }
            }
            let shaped_advance_correction_dp = placement_corrections
                .iter()
                .map(|correction| correction.x_advance_dp)
                .sum::<f32>();
            let line_advance_correction_dp = shaped_advance_correction_dp + trailing_space_width_dp;
            let alignment_correction_dp = if width_dp.is_some() {
                match style.horizontal_align {
                    HorizontalAlign::Start => 0.0,
                    HorizontalAlign::Center => -line_advance_correction_dp / 2.0,
                    HorizontalAlign::End => -line_advance_correction_dp,
                }
            } else {
                0.0
            };
            let mut cumulative_advance_correction_dp = alignment_correction_dp;
            #[cfg(feature = "rust-text-proof")]
            let mut proof_glyphs = Vec::with_capacity(run.glyphs.len());
            for (glyph, correction) in run.glyphs.iter().zip(&placement_corrections) {
                let corrected_x_dp = cumulative_advance_correction_dp + correction.x_placement_dp;
                let physical = glyph.physical((corrected_x_dp * scale, raster_baseline_px), scale);
                let cluster_start = line_byte_offset.saturating_add(glyph.start);
                let cluster_end = line_byte_offset.saturating_add(glyph.end);
                // Retain the shaper's exact placement for hit testing, vector
                // output and geometry proof. Raster cache bins remain separate
                // integer/subpixel coordinates and may be quantized for reuse.
                let origin_x_px = (glyph.x + glyph.font_size * glyph.x_offset + corrected_x_dp)
                    .mul_add(scale, 0.0);
                let origin_y_px =
                    (glyph.y - glyph.font_size * glyph.y_offset).mul_add(scale, raster_baseline_px);
                let advance_px = (glyph.w + correction.x_advance_dp) * scale;
                let font_size_px = glyph.font_size * scale;
                let rtl = glyph.level.is_rtl();
                #[cfg(feature = "rust-text-proof")]
                let (font_family, postscript_name, face_index) = self
                    .font_system
                    .db()
                    .face(physical.cache_key.font_id)
                    .map(|face| {
                        (
                            face.families
                                .first()
                                .map(|family| family.0.clone())
                                .unwrap_or_default(),
                            face.post_script_name.clone(),
                            face.index,
                        )
                    })
                    .unwrap_or_default();
                #[cfg(feature = "rust-text-proof")]
                proof_glyphs.push(ZsTextGlyphProof {
                    cluster_start,
                    cluster_end,
                    glyph_id: glyph.glyph_id,
                    font_family,
                    postscript_name,
                    face_index,
                    origin_x_px,
                    origin_y_px,
                    offset_x_px: (glyph.font_size * glyph.x_offset + correction.x_placement_dp)
                        * scale,
                    offset_y_px: glyph.font_size * glyph.y_offset * scale,
                    advance_px,
                    font_size_px,
                    rtl,
                });
                glyphs.push(ZsTextGlyphLayout {
                    cluster_start,
                    cluster_end,
                    glyph_id: glyph.glyph_id,
                    origin_x_px,
                    origin_y_px,
                    advance_px,
                    font_size_px,
                    rtl,
                    key: physical.cache_key,
                    synthetic_bold_px: correction.synthetic_bold_dp * scale,
                    raster_origin_x_px: physical.x,
                    raster_origin_y_px: physical.y,
                });
                cumulative_advance_correction_dp += correction.x_advance_dp;
            }
            if let Some(template) = trailing_space_template.as_ref() {
                for (space_index, &(space_start, space_end)) in
                    trailing_space_ranges.iter().enumerate()
                {
                    let mut glyph = template.clone();
                    glyph.start = space_start;
                    glyph.end = space_end;
                    glyph.x =
                        run.line_w + shaped_advance_correction_dp + glyph.w * space_index as f32;
                    let physical = glyph
                        .physical((alignment_correction_dp * scale, raster_baseline_px), scale);
                    let cluster_start = line_byte_offset.saturating_add(space_start);
                    let cluster_end = line_byte_offset.saturating_add(space_end);
                    let origin_x_px =
                        (glyph.x + glyph.font_size * glyph.x_offset + alignment_correction_dp)
                            * scale;
                    let origin_y_px = (glyph.y - glyph.font_size * glyph.y_offset)
                        .mul_add(scale, raster_baseline_px);
                    let advance_px = glyph.w * scale;
                    let font_size_px = glyph.font_size * scale;
                    #[cfg(feature = "rust-text-proof")]
                    let (font_family, postscript_name, face_index) = self
                        .font_system
                        .db()
                        .face(physical.cache_key.font_id)
                        .map(|face| {
                            (
                                face.families
                                    .first()
                                    .map(|family| family.0.clone())
                                    .unwrap_or_default(),
                                face.post_script_name.clone(),
                                face.index,
                            )
                        })
                        .unwrap_or_default();
                    #[cfg(feature = "rust-text-proof")]
                    proof_glyphs.push(ZsTextGlyphProof {
                        cluster_start,
                        cluster_end,
                        glyph_id: glyph.glyph_id,
                        font_family,
                        postscript_name,
                        face_index,
                        origin_x_px,
                        origin_y_px,
                        offset_x_px: glyph.font_size * glyph.x_offset * scale,
                        offset_y_px: glyph.font_size * glyph.y_offset * scale,
                        advance_px,
                        font_size_px,
                        rtl: false,
                    });
                    glyphs.push(ZsTextGlyphLayout {
                        cluster_start,
                        cluster_end,
                        glyph_id: glyph.glyph_id,
                        origin_x_px,
                        origin_y_px,
                        advance_px,
                        font_size_px,
                        rtl: false,
                        key: physical.cache_key,
                        synthetic_bold_px: 0.0,
                        raster_origin_x_px: physical.x,
                        raster_origin_y_px: physical.y,
                    });
                }
            }
            let corrected_line_width_dp = run.line_w + line_advance_correction_dp;
            corrected_content_width_dp = corrected_content_width_dp.max(corrected_line_width_dp);
            lines.push(ZsTextLineLayout {
                line_index: visual_line_index,
                top_px: (vertical_offset_dp + run.line_top) * scale,
                baseline_px,
                height_px: run.line_height * scale,
                width_px: corrected_line_width_dp * scale,
                rtl: run.rtl,
                glyph_range: glyph_start..glyphs.len(),
            });
            #[cfg(feature = "rust-text-proof")]
            proof_lines.push(ZsTextLineProof {
                line_index: visual_line_index,
                top_px: (vertical_offset_dp + run.line_top) * scale,
                baseline_px,
                height_px: run.line_height * scale,
                width_px: corrected_line_width_dp * scale,
                rtl: run.rtl,
                glyphs: proof_glyphs,
            });
        }

        let content_width_px = (corrected_content_width_dp * scale).ceil().max(0.0) as i32;
        let content_height_px = (content_height_dp * scale).ceil().max(0.0) as i32;
        let overflow_x = width_px > 0 && content_width_px > width_px;
        let overflow_y = height_px > 0 && content_height_px > height_px;
        let layout = Arc::new(ZsRustTextLayout {
            size: Size {
                width: content_width_px,
                height: content_height_px,
            },
            overflow_x,
            overflow_y,
            #[cfg(feature = "rust-text-proof")]
            proof: ZsTextProof {
                schema: TEXT_PROOF_SCHEMA.into(),
                backend: "zsui-rust-harfrust-swash".into(),
                requested_font_family: style.font_family.clone(),
                dpi_scale: scale,
                width_px: width_px.max(0),
                height_px: height_px.max(0),
                content_width_px,
                content_height_px,
                overflow_x,
                overflow_y,
                lines: proof_lines,
                errors: Vec::new(),
            },
            lines,
            glyphs,
        });
        self.insert_layout(cache_key, Arc::clone(&layout));
        layout
    }

    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width_px: i32,
        dpi_scale: f32,
    ) -> Size {
        let mut measure_style = style.clone();
        measure_style.horizontal_align = HorizontalAlign::Start;
        measure_style.vertical_align = VerticalAlign::Start;
        measure_style.ellipsis = false;
        self.layout(text, &measure_style, max_width_px, 0, dpi_scale)
            .size()
    }

    fn cached_layout(&mut self, key: &ZsTextLayoutCacheKey) -> Option<Arc<ZsRustTextLayout>> {
        let clock = self.next_cache_clock();
        if let Some(entry) = self.layout_cache.get_mut(key) {
            entry.last_used = clock;
            self.cache_stats.layout_hits = self.cache_stats.layout_hits.saturating_add(1);
            return Some(Arc::clone(&entry.layout));
        }
        self.cache_stats.layout_misses = self.cache_stats.layout_misses.saturating_add(1);
        None
    }

    fn insert_layout(&mut self, key: ZsTextLayoutCacheKey, layout: Arc<ZsRustTextLayout>) {
        let estimated_bytes = key
            .estimated_bytes()
            .saturating_add(estimate_layout_bytes(&layout));
        if estimated_bytes > self.layout_cache_byte_limit {
            return;
        }
        let last_used = self.next_cache_clock();
        if let Some(previous) = self.layout_cache.insert(
            key,
            CachedTextLayout {
                layout,
                estimated_bytes,
                last_used,
            },
        ) {
            self.layout_cache_bytes = self
                .layout_cache_bytes
                .saturating_sub(previous.estimated_bytes);
        }
        self.layout_cache_bytes = self.layout_cache_bytes.saturating_add(estimated_bytes);
        self.enforce_layout_cache_limits();
    }

    fn enforce_layout_cache_limits(&mut self) {
        while self.layout_cache.len() > self.layout_cache_limit
            || self.layout_cache_bytes > self.layout_cache_byte_limit
        {
            let Some(oldest) = self
                .layout_cache
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(entry) = self.layout_cache.remove(&oldest) {
                self.layout_cache_bytes = self
                    .layout_cache_bytes
                    .saturating_sub(entry.estimated_bytes);
                self.cache_stats.layout_evictions =
                    self.cache_stats.layout_evictions.saturating_add(1);
            }
        }
    }

    /// Composites a layout into a top-down BGRA8 buffer.
    pub fn composite_bgra(
        &mut self,
        layout: &ZsRustTextLayout,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        stride: usize,
        color: Color,
        profile: ZsTextRasterProfile,
    ) -> Result<(), String> {
        self.composite_bgra_clipped(
            layout,
            pixels,
            width,
            height,
            stride,
            0,
            0,
            ZsTextPixelRect::full(width, height),
            color,
            profile,
        )
    }

    /// Composites a retained layout at a physical-pixel origin while honoring
    /// the backend's clip. This lets a native host draw into its final buffered
    /// surface without allocating one temporary bitmap per text command.
    #[allow(clippy::too_many_arguments)]
    pub fn composite_bgra_clipped(
        &mut self,
        layout: &ZsRustTextLayout,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        stride: usize,
        origin_x_px: i32,
        origin_y_px: i32,
        clip: ZsTextPixelRect,
        color: Color,
        profile: ZsTextRasterProfile,
    ) -> Result<(), String> {
        let row_bytes = width as usize * 4;
        if stride < row_bytes || pixels.len() < stride.saturating_mul(height as usize) {
            return Err("BGRA surface dimensions do not match its backing slice".into());
        }
        let clip_edges = clip.clamped_edges(width, height);
        if clip_edges.0 >= clip_edges.2 || clip_edges.1 >= clip_edges.3 {
            return Ok(());
        }
        for glyph in &layout.glyphs {
            let Some(image) = self.glyph_image(glyph.key, profile.mode, glyph.synthetic_bold_px)
            else {
                continue;
            };
            let left = origin_x_px.saturating_add(
                glyph
                    .raster_origin_x_px
                    .saturating_add(image.placement.left),
            );
            let top = origin_y_px
                .saturating_add(glyph.raster_origin_y_px.saturating_sub(image.placement.top));
            composite_image_bgra_region(
                pixels, width, height, stride, left, top, clip_edges, &image, color, profile,
            );
        }
        Ok(())
    }

    /// Exports hinted-independent glyph outlines plus baselines and advances.
    /// Pixel differences still require PNG comparison; SVG isolates font
    /// selection, shaping and placement from antialiasing and color blending.
    #[cfg(feature = "rust-text-proof")]
    pub fn outline_svg(&mut self, layout: &ZsRustTextLayout) -> String {
        let width = layout
            .proof
            .width_px
            .max(layout.proof.content_width_px)
            .max(1);
        let height = layout
            .proof
            .height_px
            .max(layout.proof.content_height_px)
            .max(1);
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">\n<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n"
        );
        for line in &layout.proof.lines {
            svg.push_str(&format!(
                "<line x1=\"0\" y1=\"{:.3}\" x2=\"{}\" y2=\"{:.3}\" stroke=\"#0A84FF\" stroke-opacity=\"0.35\" stroke-width=\"0.5\"/>\n",
                line.baseline_px, width, line.baseline_px
            ));
        }
        for glyph in &layout.glyphs {
            let Some(commands) = self.outline_commands(glyph.key, glyph.synthetic_bold_px) else {
                continue;
            };
            let path = svg_path(&commands);
            let x = glyph.origin_x_px;
            let y = glyph.origin_y_px;
            svg.push_str(&format!(
                "<path d=\"{path}\" transform=\"translate({x:.3} {y:.3}) scale(1 -1)\" fill=\"#D000D0\" fill-opacity=\"0.42\" stroke=\"#850085\" stroke-width=\"0.2\"/>\n"
            ));
        }
        svg.push_str("</svg>\n");
        svg
    }

    fn glyph_image(
        &mut self,
        key: CacheKey,
        mode: ZsTextRasterMode,
        synthetic_bold_px: f32,
    ) -> Option<Arc<Image>> {
        let synthetic_bold_bits = synthetic_bold_px.max(0.0).to_bits();
        let cache_key = (key, mode, synthetic_bold_bits);
        let clock = self.next_cache_clock();
        if let Some(entry) = self.glyph_cache.get_mut(&cache_key) {
            entry.last_used = clock;
            self.cache_stats.glyph_hits = self.cache_stats.glyph_hits.saturating_add(1);
            return entry.image.as_ref().map(Arc::clone);
        }
        self.cache_stats.glyph_misses = self.cache_stats.glyph_misses.saturating_add(1);
        let image = render_glyph(
            &mut self.font_system,
            &mut self.scale_context,
            key,
            mode,
            synthetic_bold_px,
        )
        .map(Arc::new);
        let bytes = image.as_ref().map_or(0, |image| image.data.len());
        if bytes <= self.glyph_cache_byte_limit {
            let last_used = self.next_cache_clock();
            self.glyph_cache.insert(
                cache_key,
                CachedGlyphImage {
                    image: image.as_ref().map(Arc::clone),
                    bytes,
                    last_used,
                },
            );
            self.glyph_cache_bytes = self.glyph_cache_bytes.saturating_add(bytes);
            self.enforce_glyph_cache_limits();
        }
        image
    }

    fn enforce_glyph_cache_limits(&mut self) {
        while self.glyph_cache.len() > self.glyph_cache_limit
            || self.glyph_cache_bytes > self.glyph_cache_byte_limit
        {
            let Some(oldest) = self
                .glyph_cache
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            if let Some(entry) = self.glyph_cache.remove(&oldest) {
                self.glyph_cache_bytes = self.glyph_cache_bytes.saturating_sub(entry.bytes);
                self.cache_stats.glyph_evictions =
                    self.cache_stats.glyph_evictions.saturating_add(1);
            }
        }
    }

    fn next_cache_clock(&mut self) -> u64 {
        self.cache_clock = self.cache_clock.wrapping_add(1);
        self.cache_clock
    }

    #[cfg(feature = "rust-text-proof")]
    fn outline_commands(&mut self, key: CacheKey, synthetic_bold_px: f32) -> Option<Vec<Command>> {
        let font = self.font_system.get_font(key.font_id, key.font_weight)?;
        let mut scaler = self
            .scale_context
            .builder(font.as_swash())
            .size(f32::from_bits(key.font_size_bits))
            .hint(false)
            .build();
        let mut outline = scaler
            .scale_outline(key.glyph_id)
            .or_else(|| scaler.scale_color_outline(key.glyph_id))?;
        if key.flags.contains(CacheKeyFlags::FAKE_ITALIC) {
            outline.transform(&Transform::skew(
                Angle::from_degrees(14.0),
                Angle::from_degrees(0.0),
            ));
        }
        if synthetic_bold_px > 0.0 {
            outline.embolden(synthetic_bold_px, synthetic_bold_px);
        }
        let commands = outline.path().commands().collect();
        Some(commands)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ZsGlyphPlacementCorrection {
    x_placement_dp: f32,
    x_advance_dp: f32,
    synthetic_bold_dp: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct ZsGposPairValues {
    first_x_placement: i16,
    first_x_advance: i16,
    second_x_placement: i16,
    second_x_advance: i16,
}

/// Reconciles the two GPOS `kern` differences observable in DirectWrite:
/// mixed Han/Kana runs can miss a script-specific pair in HarfRust, while RTL
/// pairs retain their advance adjustment but DirectWrite omits the matching
/// x-placement. Corrections are table-derived and applied only when the current
/// shaping geometry proves that the value is missing or is the RTL placement.
fn windows_gpos_kern_corrections(
    font_system: &mut FontSystem,
    glyphs: &[LayoutGlyph],
) -> Vec<ZsGlyphPlacementCorrection> {
    let mut corrections = vec![ZsGlyphPlacementCorrection::default(); glyphs.len()];
    let mut span_start = 0usize;
    while span_start < glyphs.len() {
        let rtl = glyphs[span_start].level.is_rtl();
        let font_id = glyphs[span_start].font_id;
        let font_weight = glyphs[span_start].font_weight;
        let mut span_end = span_start + 1;
        while span_end < glyphs.len()
            && glyphs[span_end].level.is_rtl() == rtl
            && glyphs[span_end].font_id == font_id
            && glyphs[span_end].font_weight == font_weight
        {
            span_end += 1;
        }
        if span_end.saturating_sub(span_start) < 2 {
            span_start = span_end;
            continue;
        }

        let Some(face_index) = font_system.db().face(font_id).map(|face| face.index) else {
            span_start = span_end;
            continue;
        };
        let Some(font) = font_system.get_font(font_id, font_weight) else {
            span_start = span_end;
            continue;
        };
        let Ok(face) = ttf_parser::Face::parse(font.data(), face_index) else {
            span_start = span_end;
            continue;
        };
        let units_per_em = f32::from(face.units_per_em()).max(1.0);
        let mut desired = vec![ZsGlyphPlacementCorrection::default(); span_end - span_start];
        for index in span_start..span_end - 1 {
            let left = &glyphs[index];
            let right = &glyphs[index + 1];
            let Some(values) = gpos_kern_pair_values(
                &face,
                ttf_parser::GlyphId(left.glyph_id),
                ttf_parser::GlyphId(right.glyph_id),
            ) else {
                continue;
            };
            let left_scale = left.font_size / units_per_em;
            let right_scale = right.font_size / units_per_em;
            let left_index = index - span_start;
            let right_index = left_index + 1;
            desired[left_index].x_placement_dp += f32::from(values.first_x_placement) * left_scale;
            desired[left_index].x_advance_dp += f32::from(values.first_x_advance) * left_scale;
            desired[right_index].x_placement_dp +=
                f32::from(values.second_x_placement) * right_scale;
            desired[right_index].x_advance_dp += f32::from(values.second_x_advance) * right_scale;
        }

        const GEOMETRY_EPSILON_DP: f32 = 1.0 / 256.0;
        for (offset, glyph) in glyphs[span_start..span_end].iter().enumerate() {
            let wanted = desired[offset];
            let current_placement = glyph.font_size * glyph.x_offset;
            if rtl
                && wanted.x_placement_dp.abs() > GEOMETRY_EPSILON_DP
                && (current_placement - wanted.x_placement_dp).abs() <= GEOMETRY_EPSILON_DP
            {
                corrections[span_start + offset].x_placement_dp = -current_placement;
            } else if !rtl
                && wanted.x_placement_dp.abs() > GEOMETRY_EPSILON_DP
                && current_placement.abs() <= GEOMETRY_EPSILON_DP
            {
                corrections[span_start + offset].x_placement_dp = wanted.x_placement_dp;
            }
            let Some(nominal_units) = face.glyph_hor_advance(ttf_parser::GlyphId(glyph.glyph_id))
            else {
                continue;
            };
            let nominal_advance = f32::from(nominal_units) * glyph.font_size / units_per_em;
            let current_advance_delta = glyph.w - nominal_advance;
            if !rtl
                && wanted.x_advance_dp.abs() > GEOMETRY_EPSILON_DP
                && current_advance_delta.abs() <= GEOMETRY_EPSILON_DP
            {
                corrections[span_start + offset].x_advance_dp = wanted.x_advance_dp;
            }
        }
        span_start = span_end;
    }
    corrections
}

fn gpos_kern_pair_values(
    face: &ttf_parser::Face<'_>,
    first: ttf_parser::GlyphId,
    second: ttf_parser::GlyphId,
) -> Option<ZsGposPairValues> {
    let gpos = face.tables().gpos?;
    let kern_tag = ttf_parser::Tag::from_bytes(b"kern");
    let mut lookup_indices = Vec::<u16>::new();
    for feature in gpos.features {
        if feature.tag != kern_tag {
            continue;
        }
        for index in 0..feature.lookup_indices.len() {
            let Some(lookup_index) = feature.lookup_indices.get(index) else {
                continue;
            };
            if !lookup_indices.contains(&lookup_index) {
                lookup_indices.push(lookup_index);
            }
        }
    }
    let mut result = ZsGposPairValues::default();
    let mut found = false;
    for lookup_index in lookup_indices {
        let Some(lookup) = gpos.lookups.get(lookup_index) else {
            continue;
        };
        for subtable in lookup
            .subtables
            .into_iter::<ttf_parser::gpos::PositioningSubtable<'_>>()
        {
            let ttf_parser::gpos::PositioningSubtable::Pair(pair) = subtable else {
                continue;
            };
            let values = match pair {
                ttf_parser::gpos::PairAdjustment::Format1 { coverage, sets } => coverage
                    .get(first)
                    .and_then(|coverage_index| sets.get(coverage_index))
                    .and_then(|set| set.get(second)),
                ttf_parser::gpos::PairAdjustment::Format2 {
                    coverage,
                    classes,
                    matrix,
                } => {
                    if coverage.contains(first) {
                        matrix.get((classes.0.get(first), classes.1.get(second)))
                    } else {
                        None
                    }
                }
            };
            let Some((first_value, second_value)) = values else {
                continue;
            };
            result.first_x_placement = result
                .first_x_placement
                .saturating_add(first_value.x_placement);
            result.first_x_advance = result.first_x_advance.saturating_add(first_value.x_advance);
            result.second_x_placement = result
                .second_x_placement
                .saturating_add(second_value.x_placement);
            result.second_x_advance = result
                .second_x_advance
                .saturating_add(second_value.x_advance);
            found = true;
            break;
        }
    }
    found.then_some(result)
}

/// DirectWrite exposes a simulated bold face when a family (including a
/// fallback family) has no semibold-or-heavier face. Preserve that requested
/// intent after font fallback instead of silently painting the regular face.
/// The 2% em expansion is also the advance added by DirectWrite's simulation.
fn windows_synthetic_bold_strength_dp(
    font_system: &mut FontSystem,
    glyph: &LayoutGlyph,
    requested_weight: Weight,
    cluster_is_whitespace: bool,
) -> f32 {
    const SEMIBOLD_WEIGHT: u16 = 600;
    const SYNTHETIC_BOLD_EM: f32 = 0.02;
    const ADVANCE_EPSILON_DP: f32 = 1.0 / 256.0;

    if requested_weight.0 < SEMIBOLD_WEIGHT
        || glyph.w <= ADVANCE_EPSILON_DP
        || cluster_is_whitespace
    {
        return 0.0;
    }
    let Some((actual_weight, family_name)) = font_system.db().face(glyph.font_id).map(|face| {
        (
            face.weight,
            face.families.first().map(|family| family.0.clone()),
        )
    }) else {
        return 0.0;
    };
    if actual_weight.0 >= SEMIBOLD_WEIGHT {
        return 0.0;
    }
    if let Some(family_name) = family_name {
        let families = [Family::Name(&family_name)];
        let query = fontdb::Query {
            families: &families,
            weight: requested_weight,
            ..fontdb::Query::default()
        };
        if font_system
            .db()
            .query(&query)
            .and_then(|id| font_system.db().face(id))
            .is_some_and(|face| face.weight.0 >= SEMIBOLD_WEIGHT)
        {
            return 0.0;
        }
    }
    let has_weight_axis = font_system
        .get_font(glyph.font_id, glyph.font_weight)
        .is_some_and(|font| {
            font.as_swash()
                .variations()
                .find_by_tag(Tag::from_be_bytes(*b"wght"))
                .is_some()
        });
    if has_weight_axis {
        return 0.0;
    }
    glyph.font_size.max(0.0) * SYNTHETIC_BOLD_EM
}

fn render_glyph(
    font_system: &mut FontSystem,
    context: &mut ScaleContext,
    key: CacheKey,
    mode: ZsTextRasterMode,
    synthetic_bold_px: f32,
) -> Option<Image> {
    let font = font_system.get_font(key.font_id, key.font_weight)?;
    let weight_axis = font
        .as_swash()
        .variations()
        .find_by_tag(Tag::from_be_bytes(*b"wght"));
    let mut scaler = context
        .builder(font.as_swash())
        .size(f32::from_bits(key.font_size_bits))
        .hint(!key.flags.contains(CacheKeyFlags::DISABLE_HINTING));
    if let Some(axis) = weight_axis {
        scaler = scaler.variations(std::iter::once(Setting {
            tag: Tag::from_be_bytes(*b"wght"),
            value: f32::from(key.font_weight.0).clamp(axis.min_value(), axis.max_value()),
        }));
    }
    let mut scaler = scaler.build();
    let offset = if key.flags.contains(CacheKeyFlags::PIXEL_FONT) {
        Vector::new(
            key.x_bin.as_float().round() + 1.0,
            key.y_bin.as_float().round(),
        )
    } else {
        Vector::new(key.x_bin.as_float(), key.y_bin.as_float())
    };
    Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ])
    .format(match mode {
        ZsTextRasterMode::Grayscale => Format::Alpha,
        ZsTextRasterMode::SubpixelRgb => Format::Subpixel,
    })
    .offset(offset)
    .transform(
        key.flags
            .contains(CacheKeyFlags::FAKE_ITALIC)
            .then(|| Transform::skew(Angle::from_degrees(14.0), Angle::from_degrees(0.0))),
    )
    .embolden(synthetic_bold_px.max(0.0))
    .render(&mut scaler, key.glyph_id)
}

#[allow(clippy::too_many_arguments)]
fn composite_image_bgra_region(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    stride: usize,
    left: i32,
    top: i32,
    clip_edges: (i32, i32, i32, i32),
    image: &Image,
    color: Color,
    profile: ZsTextRasterProfile,
) {
    let channels = if image.content == Content::Mask { 1 } else { 4 };
    for row in 0..image.placement.height as i32 {
        let y = top.saturating_add(row);
        if y < clip_edges.1 || y >= clip_edges.3 || y < 0 || y >= height as i32 {
            continue;
        }
        for column in 0..image.placement.width as i32 {
            let x = left.saturating_add(column);
            if x < clip_edges.0 || x >= clip_edges.2 || x < 0 || x >= width as i32 {
                continue;
            }
            let source_offset =
                ((row as usize * image.placement.width as usize) + column as usize) * channels;
            let destination_offset = y as usize * stride + x as usize * 4;
            let destination = &mut pixels[destination_offset..destination_offset + 4];
            match image.content {
                Content::Mask => {
                    let coverage = multiply_alpha(
                        profile.adjusted_coverage(image.data[source_offset]),
                        color.a,
                    );
                    destination[0] = blend_channel(destination[0], color.b, coverage);
                    destination[1] = blend_channel(destination[1], color.g, coverage);
                    destination[2] = blend_channel(destination[2], color.r, coverage);
                    destination[3] = destination[3].max(coverage);
                }
                Content::SubpixelMask => {
                    let red = multiply_alpha(
                        profile.adjusted_coverage(image.data[source_offset]),
                        color.a,
                    );
                    let green = multiply_alpha(
                        profile.adjusted_coverage(image.data[source_offset + 1]),
                        color.a,
                    );
                    let blue = multiply_alpha(
                        profile.adjusted_coverage(image.data[source_offset + 2]),
                        color.a,
                    );
                    destination[0] = blend_channel(destination[0], color.b, blue);
                    destination[1] = blend_channel(destination[1], color.g, green);
                    destination[2] = blend_channel(destination[2], color.r, red);
                    destination[3] = destination[3].max(red.max(green).max(blue));
                }
                Content::Color => {
                    let alpha = multiply_alpha(image.data[source_offset + 3], color.a);
                    destination[0] =
                        blend_channel(destination[0], image.data[source_offset + 2], alpha);
                    destination[1] =
                        blend_channel(destination[1], image.data[source_offset + 1], alpha);
                    destination[2] =
                        blend_channel(destination[2], image.data[source_offset], alpha);
                    destination[3] = destination[3].max(alpha);
                }
            }
        }
    }
}

fn blend_channel(destination: u8, source: u8, alpha: u8) -> u8 {
    let alpha = u32::from(alpha);
    (((u32::from(source) * alpha) + (u32::from(destination) * (255 - alpha)) + 127) / 255) as u8
}

fn multiply_alpha(left: u8, right: u8) -> u8 {
    ((u16::from(left) * u16::from(right) + 127) / 255) as u8
}

fn cosmic_weight(weight: TextWeight) -> Weight {
    match weight {
        TextWeight::Automatic | TextWeight::Regular => Weight::NORMAL,
        TextWeight::Medium => Weight::MEDIUM,
        TextWeight::Semibold => Weight::SEMIBOLD,
        TextWeight::Bold => Weight::BOLD,
    }
}

const fn text_weight_key(weight: TextWeight) -> u8 {
    match weight {
        TextWeight::Automatic => 0,
        TextWeight::Regular => 1,
        TextWeight::Medium => 2,
        TextWeight::Semibold => 3,
        TextWeight::Bold => 4,
    }
}

const fn horizontal_align_key(align: HorizontalAlign) -> u8 {
    match align {
        HorizontalAlign::Start => 0,
        HorizontalAlign::Center => 1,
        HorizontalAlign::End => 2,
    }
}

const fn vertical_align_key(align: VerticalAlign) -> u8 {
    match align {
        VerticalAlign::Start => 0,
        VerticalAlign::Center => 1,
        VerticalAlign::End => 2,
    }
}

const fn text_wrap_key(wrap: TextWrap) -> u8 {
    match wrap {
        TextWrap::NoWrap => 0,
        TextWrap::Word => 1,
    }
}

const fn line_metric_policy_key(policy: ZsTextLineMetricPolicy) -> u8 {
    match policy {
        ZsTextLineMetricPolicy::CenteredGlyphBox => 0,
        ZsTextLineMetricPolicy::WindowsDirectWrite => 1,
    }
}

fn estimate_layout_bytes(layout: &ZsRustTextLayout) -> usize {
    let bytes = size_of::<ZsRustTextLayout>()
        .saturating_add(
            layout
                .lines
                .capacity()
                .saturating_mul(size_of::<ZsTextLineLayout>()),
        )
        .saturating_add(
            layout
                .glyphs
                .capacity()
                .saturating_mul(size_of::<ZsTextGlyphLayout>()),
        );
    #[cfg(feature = "rust-text-proof")]
    let bytes = {
        let mut bytes = bytes
            .saturating_add(layout.proof.schema.capacity())
            .saturating_add(layout.proof.backend.capacity())
            .saturating_add(layout.proof.requested_font_family.capacity())
            .saturating_add(
                layout
                    .proof
                    .lines
                    .capacity()
                    .saturating_mul(size_of::<ZsTextLineProof>()),
            );
        for line in &layout.proof.lines {
            bytes = bytes.saturating_add(
                line.glyphs
                    .capacity()
                    .saturating_mul(size_of::<ZsTextGlyphProof>()),
            );
            for glyph in &line.glyphs {
                bytes = bytes
                    .saturating_add(glyph.font_family.capacity())
                    .saturating_add(glyph.postscript_name.capacity());
            }
        }
        bytes
    };
    bytes
}

fn resolve_primary_family_weight(
    font_system: &FontSystem,
    family: Family<'_>,
    requested_weight: Weight,
) -> Weight {
    let families = [family];
    let query = fontdb::Query {
        families: &families,
        weight: requested_weight,
        ..fontdb::Query::default()
    };
    font_system
        .db()
        .query(&query)
        .and_then(|id| font_system.db().face(id))
        .map_or(requested_weight, |face| face.weight)
}

fn windows_line_baseline(
    font_system: &mut FontSystem,
    glyphs: &[LayoutGlyph],
    line_top: f32,
    line_height: f32,
) -> Option<f32> {
    let mut max_before_baseline = 0.0f32;
    let mut max_after_baseline = 0.0f32;
    for glyph in glyphs {
        let face_index = font_system.db().face(glyph.font_id)?.index;
        let font = font_system.get_font(glyph.font_id, glyph.font_weight)?;
        let (units_per_em, ascent, descent, line_gap) =
            windows_design_line_metrics(font.data(), face_index).unwrap_or_else(|| {
                let metrics = font.metrics();
                (
                    f32::from(metrics.units_per_em).max(1.0),
                    metrics.ascent.max(0.0),
                    (-metrics.descent).max(0.0),
                    metrics.leading.max(0.0),
                )
            });
        let scale = glyph.font_size / units_per_em.max(1.0);
        let half_gap = line_gap.max(0.0) * scale / 2.0;
        max_before_baseline = max_before_baseline.max(ascent.max(0.0) * scale + half_gap);
        max_after_baseline = max_after_baseline.max(descent.max(0.0) * scale + half_gap);
    }
    let metric_height = max_before_baseline + max_after_baseline;
    (metric_height > 0.0).then_some(line_top + line_height * max_before_baseline / metric_height)
}

fn windows_design_line_metrics(data: &[u8], face_index: u32) -> Option<(f32, f32, f32, f32)> {
    let face = ttf_parser::Face::parse(data, face_index).ok()?;
    let units_per_em = f32::from(face.units_per_em()).max(1.0);
    let os2 = face.tables().os2?;
    if os2.use_typographic_metrics() {
        Some((
            units_per_em,
            f32::from(os2.typographic_ascender()),
            f32::from(-os2.typographic_descender()),
            f32::from(os2.typographic_line_gap()),
        ))
    } else {
        Some((
            units_per_em,
            f32::from(os2.windows_ascender()),
            f32::from(-os2.windows_descender()),
            0.0,
        ))
    }
}

fn normalized_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.clamp(0.5, 8.0)
    } else {
        1.0
    }
}

fn text_line_byte_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    offsets.extend(
        text.char_indices()
            .filter_map(|(index, character)| (character == '\n').then_some(index + 1)),
    );
    offsets
}

/// Cosmic Text intentionally removes break-separating ASCII spaces from a
/// wrapped visual run. DirectWrite retains them as zero-ink glyphs so caret,
/// selection and hit testing cover every source byte. Reinsert only the gap
/// immediately following this run; later runs keep their own glyph ranges.
fn omitted_trailing_ascii_space_ranges(
    line_text: &str,
    glyphs: &[LayoutGlyph],
) -> Vec<(usize, usize)> {
    let Some(start) = glyphs.iter().map(|glyph| glyph.end).max() else {
        return Vec::new();
    };
    let Some(suffix) = line_text.get(start..) else {
        return Vec::new();
    };
    suffix
        .char_indices()
        .take_while(|(_, character)| *character == ' ')
        .map(|(offset, character)| {
            let range_start = start + offset;
            (range_start, range_start + character.len_utf8())
        })
        .collect()
}

fn shape_ascii_space_template(
    font_system: &mut FontSystem,
    metrics: Metrics,
    attrs: &Attrs<'_>,
) -> Option<LayoutGlyph> {
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_wrap(Wrap::None);
    buffer.set_size(None, None);
    // A surrounding visible glyph prevents the shaper from trimming the space.
    buffer.set_text("x x", attrs, Shaping::Advanced, Some(Align::Left));
    buffer.shape_until_scroll(font_system, true);
    let mut glyph = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .find(|glyph| glyph.start == 1 && glyph.end == 2)?
        .clone();
    glyph.x = 0.0;
    glyph.start = 0;
    glyph.end = 1;
    Some(glyph)
}

fn empty_layout(
    style: &TextStyle,
    width_px: i32,
    height_px: i32,
    dpi_scale: f32,
) -> ZsRustTextLayout {
    #[cfg(not(feature = "rust-text-proof"))]
    let _ = (style, width_px, height_px, dpi_scale);
    ZsRustTextLayout {
        size: Size {
            width: 0,
            height: 0,
        },
        overflow_x: false,
        overflow_y: false,
        #[cfg(feature = "rust-text-proof")]
        proof: ZsTextProof {
            schema: TEXT_PROOF_SCHEMA.into(),
            backend: "zsui-rust-harfrust-swash".into(),
            requested_font_family: style.font_family.clone(),
            dpi_scale: normalized_scale(dpi_scale),
            width_px: width_px.max(0),
            height_px: height_px.max(0),
            content_width_px: 0,
            content_height_px: 0,
            overflow_x: false,
            overflow_y: false,
            lines: Vec::new(),
            errors: Vec::new(),
        },
        lines: Vec::new(),
        glyphs: Vec::new(),
    }
}

#[cfg(feature = "rust-text-proof")]
fn svg_path(commands: &[Command]) -> String {
    let mut path = String::new();
    for command in commands {
        match command {
            Command::MoveTo(point) => path.push_str(&format!("M{:.3},{:.3}", point.x, point.y)),
            Command::LineTo(point) => path.push_str(&format!("L{:.3},{:.3}", point.x, point.y)),
            Command::CurveTo(first, second, point) => path.push_str(&format!(
                "C{:.3},{:.3} {:.3},{:.3} {:.3},{:.3}",
                first.x, first.y, second.x, second.y, point.x, point.y
            )),
            Command::QuadTo(control, point) => path.push_str(&format!(
                "Q{:.3},{:.3} {:.3},{:.3}",
                control.x, control.y, point.x, point.y
            )),
            Command::Close => path.push('Z'),
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    fn text_engine_test_guard() -> MutexGuard<'static, ()> {
        static TEST_LOCK: Mutex<()> = Mutex::new(());
        TEST_LOCK.lock().expect("text engine test lock")
    }

    #[test]
    fn subpixel_compositor_maps_rgb_masks_to_bgra_channels() {
        let image = Image {
            content: Content::SubpixelMask,
            placement: swash::zeno::Placement {
                left: 0,
                top: 0,
                width: 1,
                height: 1,
            },
            data: vec![255, 0, 0, 255],
            ..Image::default()
        };
        let mut pixel = [255, 255, 255, 255];
        composite_image_bgra_region(
            &mut pixel,
            1,
            1,
            4,
            0,
            0,
            (0, 0, 1, 1),
            &image,
            Color::rgb(0, 0, 0),
            ZsTextRasterProfile::subpixel_rgb(),
        );
        assert_eq!(pixel, [255, 255, 0, 255]);
    }

    #[test]
    fn clipped_compositor_applies_backend_origin_without_a_temporary_surface() {
        let image = Image {
            content: Content::Mask,
            placement: swash::zeno::Placement {
                left: 0,
                top: 0,
                width: 2,
                height: 1,
            },
            data: vec![255, 255],
            ..Image::default()
        };
        let mut pixels = [255_u8; 4 * 4];
        composite_image_bgra_region(
            &mut pixels,
            4,
            1,
            16,
            1,
            0,
            (2, 0, 3, 1),
            &image,
            Color::rgb(0, 0, 0),
            ZsTextRasterProfile::grayscale(),
        );

        assert_eq!(&pixels[4..8], &[255, 255, 255, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 0, 255]);
        assert_eq!(&pixels[12..16], &[255, 255, 255, 255]);
    }

    #[cfg(feature = "rust-text-proof")]
    #[test]
    fn layout_reuses_one_result_for_json_and_svg_geometry() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new().with_glyph_cache_limit(8);
        let mut style = TextStyle::line("sans-serif", 14.0, Color::rgb(20, 20, 20));
        style.ellipsis = false;
        let layout = engine.layout("ZSUI 字体", &style, 320, 48, 1.0);
        assert_eq!(layout.proof().schema, TEXT_PROOF_SCHEMA);
        assert!(!layout.proof().lines.is_empty());
        assert!(layout.proof_json().unwrap().contains("glyph_id"));
        let svg = engine.outline_svg(&layout);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<line"));
    }

    #[test]
    fn empty_text_has_zero_intrinsic_size() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new();
        let style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        let layout = engine.layout("", &style, 120, 32, 1.25);
        assert_eq!(
            layout.size(),
            Size {
                width: 0,
                height: 0,
            }
        );
        #[cfg(feature = "rust-text-proof")]
        assert!(layout.proof().lines.is_empty());
    }

    #[test]
    fn repeated_layout_reuses_the_retained_result() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new();
        let style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        let first = engine.layout("retained text", &style, 320, 48, 1.0);
        let second = engine.layout("retained text", &style, 320, 48, 1.0);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(engine.layout_cache_len(), 1);
        assert_eq!(engine.cache_stats().layout_hits, 1);
        assert!(!first.lines().is_empty());
        assert!(!first.glyphs().is_empty());
        assert!(first
            .glyphs()
            .iter()
            .all(|glyph| glyph.cluster_start <= glyph.cluster_end));

        let key = first.glyphs()[0].key;
        let synthetic_bold_px = first.glyphs()[0].synthetic_bold_px;
        let first_image = engine.glyph_image(key, ZsTextRasterMode::Grayscale, synthetic_bold_px);
        let second_image = engine.glyph_image(key, ZsTextRasterMode::Grayscale, synthetic_bold_px);
        if let (Some(first_image), Some(second_image)) = (first_image, second_image) {
            assert!(Arc::ptr_eq(&first_image, &second_image));
        }
        assert_eq!(engine.cache_stats().glyph_hits, 1);
    }

    #[test]
    fn wrapped_visual_lines_have_unique_indices_and_glyph_ranges() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new();
        let mut style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        style.wrap = TextWrap::Word;
        let layout = engine.layout("one two three four five", &style, 48, 256, 1.0);

        assert!(layout.lines().len() > 1);
        for (expected_index, line) in layout.lines().iter().enumerate() {
            assert_eq!(line.line_index, expected_index);
            assert_eq!(
                layout
                    .glyphs_for_line(expected_index)
                    .map(|glyphs| glyphs.len()),
                Some(line.glyph_range.len())
            );
        }
    }

    #[cfg(feature = "rust-text-proof")]
    #[test]
    fn geometry_comparison_does_not_cascade_after_one_unmatched_glyph() {
        let reference = proof_with_glyphs(vec![
            proof_glyph(0, 10, 0.0),
            proof_glyph(1, 11, 10.0),
            proof_glyph(2, 12, 20.0),
        ]);
        let candidate = proof_with_glyphs(vec![proof_glyph(0, 10, 0.0), proof_glyph(2, 12, 20.0)]);

        let diff = compare_text_geometry(&reference, &candidate, 0.25);
        assert!(!diff.glyph_count_equal);
        assert_eq!(diff.compared_glyphs, 2);
        assert_eq!(diff.matching_glyph_ids, 2);
        assert_eq!(diff.matching_font_faces, 2);
        assert_eq!(diff.max_origin_delta_px, 0.0);
        assert!(!diff.within_tolerance);
    }

    #[test]
    fn paint_color_does_not_invalidate_text_geometry() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new();
        let light = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        let mut dark = light.clone();
        dark.color = Color::rgb(255, 255, 255);
        let first = engine.layout("shared geometry", &light, 320, 48, 1.0);
        let second = engine.layout("shared geometry", &dark, 320, 48, 1.0);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn layout_cache_respects_entry_and_byte_limits() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new().with_layout_cache_limits(1, 64 * 1024);
        let style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        let _ = engine.layout("first", &style, 320, 48, 1.0);
        let _ = engine.layout("second", &style, 320, 48, 1.0);
        assert_eq!(engine.layout_cache_len(), 1);
        assert!(engine.layout_cache_bytes() <= 64 * 1024);
        assert!(engine.cache_stats().layout_evictions >= 1);
    }

    #[cfg(feature = "rust-text-proof")]
    fn proof_glyph(cluster_start: usize, glyph_id: u16, origin_x_px: f32) -> ZsTextGlyphProof {
        ZsTextGlyphProof {
            cluster_start,
            cluster_end: cluster_start + 1,
            glyph_id,
            font_family: "Test UI".into(),
            postscript_name: "TestUI-Regular".into(),
            face_index: 0,
            origin_x_px,
            origin_y_px: 12.0,
            offset_x_px: 0.0,
            offset_y_px: 0.0,
            advance_px: 10.0,
            font_size_px: 14.0,
            rtl: false,
        }
    }

    #[cfg(feature = "rust-text-proof")]
    fn proof_with_glyphs(glyphs: Vec<ZsTextGlyphProof>) -> ZsTextProof {
        ZsTextProof {
            schema: TEXT_PROOF_SCHEMA.into(),
            backend: "test".into(),
            requested_font_family: "Test UI".into(),
            dpi_scale: 1.0,
            width_px: 100,
            height_px: 20,
            content_width_px: 30,
            content_height_px: 20,
            overflow_x: false,
            overflow_y: false,
            lines: vec![ZsTextLineProof {
                line_index: 0,
                top_px: 0.0,
                baseline_px: 12.0,
                height_px: 20.0,
                width_px: 30.0,
                rtl: false,
                glyphs,
            }],
            errors: Vec::new(),
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_gpos_fallback_reads_kana_pair_adjustment() {
        let mut font_system = FontSystem::new();
        let face_info = font_system
            .db()
            .faces()
            .find(|face| face.post_script_name == "YuGothicUI-Regular")
            .cloned()
            .expect("Yu Gothic UI Regular should be installed on Windows");
        let font = font_system
            .get_font(face_info.id, face_info.weight)
            .expect("Yu Gothic UI font data");
        let face = ttf_parser::Face::parse(font.data(), face_info.index)
            .expect("Yu Gothic UI OpenType face");
        let values = gpos_kern_pair_values(
            &face,
            ttf_parser::GlyphId(23_780),
            ttf_parser::GlyphId(23_801),
        )
        .expect("Kana kern pair");
        assert_eq!(values.first_x_advance, -82);
        assert_eq!(values.first_x_placement, 0);
        assert_eq!(values.second_x_advance, 0);
        assert_eq!(values.second_x_placement, 0);
    }

    #[cfg(windows)]
    #[test]
    fn windows_synthetic_bold_is_retained_only_for_visible_glyphs_without_a_bold_face() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new()
            .with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
        let mut emoji_style = TextStyle::line("Segoe UI Emoji", 20.0, Color::rgb(0, 0, 0));
        emoji_style.weight = TextWeight::Semibold;
        let emoji = engine.layout("😀 😀", &emoji_style, 320, 48, 1.0);
        assert!(emoji
            .glyphs()
            .iter()
            .any(|glyph| glyph.synthetic_bold_px > 0.0));
        assert!(emoji.glyphs().iter().all(|glyph| {
            let whitespace = "😀 😀"
                .get(glyph.cluster_start..glyph.cluster_end)
                .is_some_and(|cluster| cluster.chars().all(char::is_whitespace));
            !whitespace || glyph.synthetic_bold_px == 0.0
        }));

        let mut cjk_style = TextStyle::line("Microsoft YaHei UI", 20.0, Color::rgb(0, 0, 0));
        cjk_style.weight = TextWeight::Bold;
        let cjk = engine.layout("中文", &cjk_style, 320, 48, 1.0);
        assert!(cjk
            .glyphs()
            .iter()
            .all(|glyph| glyph.synthetic_bold_px == 0.0));
    }

    #[cfg(windows)]
    #[test]
    fn windows_wrapped_layout_retains_break_spaces_and_one_paragraph_baseline_phase() {
        let _guard = text_engine_test_guard();
        let mut engine = ZsRustTextEngine::new()
            .with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
        let text = "ZSUI 原生文字 — 日本語 — 한국어 — مرحبا — שלום — नमस्ते — 👋🏽";
        let mut style = TextStyle::line("Segoe UI", 28.0, Color::rgb(0, 0, 0));
        style.line_height = 36.0;
        style.weight = TextWeight::Bold;
        style.wrap = TextWrap::Word;
        let layout = engine.layout(text, &style, 720, 320, 2.0);

        assert!(layout.lines().len() >= 2);
        assert!(layout.lines().iter().any(|line| {
            line.glyph_range
                .end
                .checked_sub(1)
                .and_then(|index| layout.glyphs().get(index))
                .is_some_and(|glyph| text.get(glyph.cluster_start..glyph.cluster_end) == Some(" "))
        }));
        let first_origins = layout
            .lines()
            .iter()
            .filter_map(|line| layout.glyphs().get(line.glyph_range.start))
            .map(|glyph| glyph.origin_y_px)
            .collect::<Vec<_>>();
        for baselines in first_origins.windows(2) {
            assert!((baselines[1] - baselines[0] - 72.0).abs() <= 0.001);
        }
    }
}
