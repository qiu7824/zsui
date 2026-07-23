use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::{
    geometry::{Rect, Size},
    style::ZsuiThemeMode,
    ZsIcon,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parses the canonical platform-independent `#RRGGBBAA` representation.
    ///
    /// Lowercase, shorthand and RGB-only forms are intentionally rejected so
    /// serialized UI state has one deterministic spelling.
    pub fn parse_hex_rgba(value: &str) -> Option<Self> {
        fn nibble(value: u8) -> Option<u8> {
            match value {
                b'0'..=b'9' => Some(value - b'0'),
                b'A'..=b'F' => Some(value - b'A' + 10),
                _ => None,
            }
        }

        let bytes = value.as_bytes();
        if bytes.len() != 9 || bytes[0] != b'#' {
            return None;
        }
        let channel =
            |offset: usize| Some((nibble(bytes[offset])? << 4) | nibble(bytes[offset + 1])?);
        Some(Self::rgba(
            channel(1)?,
            channel(3)?,
            channel(5)?,
            channel(7)?,
        ))
    }

    /// Returns the canonical uppercase `#RRGGBBAA` representation.
    pub fn hex_rgba(self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextWeight {
    /// Resolve the role's native default weight in the active backend.
    Automatic,
    Regular,
    Medium,
    Semibold,
    Bold,
}

/// Native desktop typography family used to resolve semantic text roles.
///
/// Applications keep using [`TextRole`]. The active backend selects this
/// profile so a semantic body, caption or title does not inherit Windows
/// Fluent point sizes on AppKit or GTK.
pub type ZsTypographyPlatformStyle = crate::ZsPlatformStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HorizontalAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextWrap {
    NoWrap,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextRole {
    Body,
    Caption,
    BodyLarge,
    Subtitle,
    /// Native application/window title used by framework chrome.
    ///
    /// This is intentionally separate from content `Title`: desktop shells
    /// commonly use a compact title ramp (ZSClip uses a 24/32 Windows title)
    /// while document content may legitimately request a larger heading.
    WindowTitle,
    Title,
    TitleLarge,
    Display,
    Button,
    Icon,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTypographyMetrics {
    pub size: f32,
    pub line_height: f32,
    pub default_weight: TextWeight,
}

impl ZsTypographyMetrics {
    pub const fn new(size: f32, line_height: f32, default_weight: TextWeight) -> Self {
        Self {
            size,
            line_height,
            default_weight,
        }
    }
}

/// Resolved metrics for the platform UI font used by native proof and paint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NativeFontMetrics {
    pub size: f32,
    pub line_height: f32,
    pub ascent: Option<f32>,
    pub descent: Option<f32>,
    pub leading: Option<f32>,
}

impl NativeFontMetrics {
    pub const fn from_typography(metrics: ZsTypographyMetrics) -> Self {
        Self {
            size: metrics.size,
            line_height: metrics.line_height,
            ascent: None,
            descent: None,
            leading: None,
        }
    }

    pub const fn with_vertical_metrics(mut self, ascent: f32, descent: f32, leading: f32) -> Self {
        self.ascent = Some(ascent);
        self.descent = Some(descent);
        self.leading = Some(leading);
        self
    }
}

/// Backend-resolved typography profile shared by native layout, paint and proof.
///
/// Applications keep declaring semantic [`TextRole`] values. Each backend owns
/// the actual system families and text rasterizer recorded here, while the
/// framework keeps the role ramp and the live accessibility scale consistent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeTypographyProfile {
    pub platform: ZsTypographyPlatformStyle,
    pub source: String,
    pub configured_ui_font: Option<String>,
    pub ui_font_family: String,
    pub small_font_family: String,
    pub display_font_family: String,
    pub monospace_font_family: String,
    pub icon_font_family: String,
    pub typography_scale: f32,
    pub body_metrics: NativeFontMetrics,
    pub rasterization: String,
}

impl NativeTypographyProfile {
    pub fn new(
        platform: ZsTypographyPlatformStyle,
        source: impl Into<String>,
        ui_font_family: impl Into<String>,
        monospace_font_family: impl Into<String>,
        icon_font_family: impl Into<String>,
        typography_scale: f32,
        rasterization: impl Into<String>,
    ) -> Self {
        let ui_font_family = ui_font_family.into();
        let typography_scale = normalized_typography_scale(typography_scale);
        Self {
            platform,
            source: source.into(),
            configured_ui_font: None,
            small_font_family: ui_font_family.clone(),
            display_font_family: ui_font_family.clone(),
            ui_font_family,
            monospace_font_family: monospace_font_family.into(),
            icon_font_family: icon_font_family.into(),
            typography_scale,
            body_metrics: NativeFontMetrics::from_typography(scaled_typography_metrics(
                TextRole::Body.metrics_for(platform),
                typography_scale,
            )),
            rasterization: rasterization.into(),
        }
    }

    pub fn fallback(platform: ZsTypographyPlatformStyle, typography_scale: f32) -> Self {
        let fallback =
            crate::platform_component_profile::PlatformTypographyProfile::for_platform(platform)
                .fallback();
        Self::new(
            platform,
            fallback.source,
            fallback.ui_font_family,
            fallback.monospace_font_family,
            fallback.icon_font_family,
            typography_scale,
            fallback.rasterization,
        )
        .with_role_families(fallback.small_font_family, fallback.display_font_family)
    }

    pub fn with_configured_ui_font(mut self, configured_ui_font: impl Into<String>) -> Self {
        self.configured_ui_font = Some(configured_ui_font.into());
        self
    }

    pub fn with_role_families(
        mut self,
        small_font_family: impl Into<String>,
        display_font_family: impl Into<String>,
    ) -> Self {
        self.small_font_family = small_font_family.into();
        self.display_font_family = display_font_family.into();
        self
    }

    pub fn with_typography_scale(mut self, typography_scale: f32) -> Self {
        let typography_scale = normalized_typography_scale(typography_scale);
        let previous_scale = self.typography_scale.max(0.001);
        let vertical_scale = typography_scale / previous_scale;
        self.typography_scale = typography_scale;
        self.body_metrics = NativeFontMetrics {
            size: TextRole::Body.metrics_for(self.platform).size * typography_scale,
            line_height: TextRole::Body.metrics_for(self.platform).line_height * typography_scale,
            ascent: self.body_metrics.ascent.map(|value| value * vertical_scale),
            descent: self
                .body_metrics
                .descent
                .map(|value| value * vertical_scale),
            leading: self
                .body_metrics
                .leading
                .map(|value| value * vertical_scale),
        };
        self
    }

    pub fn with_body_vertical_metrics(mut self, ascent: f32, descent: f32, leading: f32) -> Self {
        self.body_metrics = self
            .body_metrics
            .with_vertical_metrics(ascent, descent, leading);
        self
    }

    pub fn metrics_for(&self, role: TextRole) -> ZsTypographyMetrics {
        scaled_typography_metrics(role.metrics_for(self.platform), self.typography_scale)
    }

    pub fn font_family_for(&self, role: TextRole) -> &str {
        match role {
            TextRole::Monospace => &self.monospace_font_family,
            TextRole::Icon => &self.icon_font_family,
            TextRole::Caption => &self.small_font_family,
            TextRole::Subtitle
            | TextRole::WindowTitle
            | TextRole::Title
            | TextRole::TitleLarge
            | TextRole::Display => &self.display_font_family,
            _ => &self.ui_font_family,
        }
    }
}

fn normalized_typography_scale(scale: f32) -> f32 {
    f32::from(normalize_typography_scale_per_mille(scale)) / 1_000.0
}

fn scaled_typography_metrics(
    metrics: ZsTypographyMetrics,
    typography_scale: f32,
) -> ZsTypographyMetrics {
    ZsTypographyMetrics::new(
        metrics.size * typography_scale,
        metrics.line_height * typography_scale,
        metrics.default_weight,
    )
}

impl TextRole {
    /// Returns the Windows Fluent fallback size in device-independent pixels.
    ///
    /// Native framework layout and render code should use [`Self::metrics_for`]
    /// so AppKit and GTK do not inherit this Windows type ramp.
    pub const fn size(self) -> f32 {
        match self {
            Self::Caption => 12.0,
            Self::Body | Self::Button => 14.0,
            Self::Icon => 16.0,
            Self::Monospace => 13.0,
            Self::BodyLarge => 18.0,
            Self::Subtitle => 20.0,
            Self::WindowTitle => 24.0,
            Self::Title => 28.0,
            Self::TitleLarge => 40.0,
            Self::Display => 68.0,
        }
    }

    /// Returns the Windows Fluent fallback line box in device-independent pixels.
    ///
    /// Native framework layout and render code should use [`Self::metrics_for`].
    pub const fn line_height(self) -> f32 {
        match self {
            Self::Caption => 16.0,
            Self::Body | Self::Button | Self::Icon => 20.0,
            Self::Monospace => 18.0,
            Self::BodyLarge => 24.0,
            Self::Subtitle => 28.0,
            Self::WindowTitle => 32.0,
            Self::Title => 36.0,
            Self::TitleLarge => 52.0,
            Self::Display => 92.0,
        }
    }

    pub const fn default_weight(self) -> TextWeight {
        match self {
            Self::Subtitle | Self::WindowTitle | Self::Title | Self::TitleLarge | Self::Display => {
                TextWeight::Semibold
            }
            _ => TextWeight::Regular,
        }
    }

    /// Resolves one semantic role through the native desktop type system.
    ///
    /// The macOS values are the AppKit text-style sizes and line heights from
    /// Apple's macOS typography table. GTK values follow the documented
    /// libadwaita relative type classes over the standard 14-logical-pixel UI
    /// base; the GTK renderer can still select the actual configured family.
    pub const fn metrics_for(self, platform: ZsTypographyPlatformStyle) -> ZsTypographyMetrics {
        crate::platform_component_profile::PlatformTypographyProfile::for_platform(platform)
            .metrics(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorRole {
    PrimaryText,
    SecondaryText,
    DisabledText,
    Accent,
    AccentText,
    Surface,
    SurfaceRaised,
    Control,
    Border,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticTextStyle {
    pub role: TextRole,
    pub color: ColorRole,
    pub weight: TextWeight,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
    pub ellipsis: bool,
}

impl SemanticTextStyle {
    pub const fn for_role(role: TextRole) -> Self {
        Self {
            role,
            color: ColorRole::PrimaryText,
            weight: TextWeight::Automatic,
            horizontal_align: HorizontalAlign::Start,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: true,
        }
    }

    pub const fn body() -> Self {
        Self::for_role(TextRole::Body)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub size: f32,
    #[serde(default)]
    pub line_height: f32,
    /// Original semantic role retained for native text-style APIs.
    #[serde(default)]
    pub semantic_role: Option<TextRole>,
    pub weight: TextWeight,
    pub color: Color,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
    pub ellipsis: bool,
}

impl TextStyle {
    pub fn line(font_family: impl Into<String>, size: f32, color: Color) -> Self {
        Self {
            font_family: font_family.into(),
            size,
            line_height: 0.0,
            semantic_role: None,
            weight: TextWeight::Regular,
            color,
            horizontal_align: HorizontalAlign::Start,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: true,
        }
    }
}

pub trait NativeStyleResolver {
    fn resolve_text_style(&self, style: SemanticTextStyle) -> TextStyle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeStyleHostOperation {
    ResolveTextStyle,
}

impl NativeStyleHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::ResolveTextStyle => "resolve_text_style",
        }
    }
}

pub const REQUIRED_NATIVE_STYLE_HOST_OPERATIONS: [NativeStyleHostOperation; 1] =
    [NativeStyleHostOperation::ResolveTextStyle];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub bounds: Rect,
}

pub trait TextLayout {
    fn measure(&self, text: &str, style: &TextStyle, max_width: i32) -> Size;
    fn layout_runs(&self, text: &str, style: &TextStyle, bounds: Rect) -> Vec<TextRun>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextLayoutHostOperation {
    Measure,
    LayoutRuns,
}

impl TextLayoutHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Measure => "measure",
            Self::LayoutRuns => "layout_runs",
        }
    }
}

pub const REQUIRED_TEXT_LAYOUT_HOST_OPERATIONS: [TextLayoutHostOperation; 2] = [
    TextLayoutHostOperation::Measure,
    TextLayoutHostOperation::LayoutRuns,
];

pub trait Renderer {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn stroke_rect(&mut self, rect: Rect, color: Color, width: i32);
    fn stroke_arc(
        &mut self,
        rect: Rect,
        color: Color,
        width: i32,
        start_degrees: i16,
        sweep_degrees: i16,
    );
    fn draw_text(&mut self, run: &TextRun, style: &TextStyle);
    fn push_clip(&mut self, rect: Rect);
    fn pop_clip(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererHostOperation {
    FillRect,
    StrokeRect,
    StrokeArc,
    DrawText,
    PushClip,
    PopClip,
}

impl RendererHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::FillRect => "fill_rect",
            Self::StrokeRect => "stroke_rect",
            Self::StrokeArc => "stroke_arc",
            Self::DrawText => "draw_text",
            Self::PushClip => "push_clip",
            Self::PopClip => "pop_clip",
        }
    }
}

pub const REQUIRED_RENDERER_HOST_OPERATIONS: [RendererHostOperation; 6] = [
    RendererHostOperation::FillRect,
    RendererHostOperation::StrokeRect,
    RendererHostOperation::StrokeArc,
    RendererHostOperation::DrawText,
    RendererHostOperation::PushClip,
    RendererHostOperation::PopClip,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeDrawFill {
    Color(Color),
    Role(ColorRole),
    RoleWithAlpha { role: ColorRole, alpha: u8 },
}

impl NativeDrawFill {
    pub const fn role(role: ColorRole) -> Self {
        Self::Role(role)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawTextCommand {
    pub text: String,
    pub bounds: Rect,
    pub style: SemanticTextStyle,
}

#[cfg(feature = "password-box")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawSecureTextCommand {
    #[serde(skip, default)]
    value: crate::ZsPassword,
    pub bounds: Rect,
    pub style: SemanticTextStyle,
    pub revealed: bool,
}

#[cfg(feature = "password-box")]
impl NativeDrawSecureTextCommand {
    pub fn new(
        value: crate::ZsPassword,
        bounds: Rect,
        style: SemanticTextStyle,
        revealed: bool,
    ) -> Self {
        Self {
            value,
            bounds,
            style,
            revealed,
        }
    }

    pub fn character_count(&self) -> usize {
        self.value.char_count()
    }

    #[allow(dead_code)]
    pub(crate) fn rendered_text(&self) -> zeroize::Zeroizing<String> {
        zeroize::Zeroizing::new(if self.revealed {
            self.value.as_str().to_owned()
        } else {
            crate::mask_password(self.value.as_str())
        })
    }

    pub(crate) fn replace_value(&mut self, value: crate::ZsPassword) {
        self.value = value;
    }
}

impl NativeDrawTextCommand {
    pub fn new(text: impl Into<String>, bounds: Rect, style: SemanticTextStyle) -> Self {
        Self {
            text: text.into(),
            bounds,
            style,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeIconColorMode {
    ThemeAware,
    Original,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawIconCommand {
    pub icon: ZsIcon,
    pub bounds: Rect,
    pub color_mode: NativeIconColorMode,
    pub color: ColorRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsImageFrameId(pub u64);

impl ZsImageFrameId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsImageFrame {
    id: ZsImageFrameId,
    width: u32,
    height: u32,
    #[serde(with = "arc_bytes")]
    premultiplied_bgra8: Arc<[u8]>,
}

impl fmt::Debug for ZsImageFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZsImageFrame")
            .field("id", &self.id)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("decoded_bytes", &self.premultiplied_bgra8.len())
            .finish()
    }
}

impl ZsImageFrame {
    pub fn from_rgba8(
        id: ZsImageFrameId,
        width: u32,
        height: u32,
        rgba8: impl Into<Vec<u8>>,
    ) -> crate::ZsuiResult<Self> {
        let rgba8 = rgba8.into();
        validate_image_buffer(width, height, rgba8.len())?;
        let mut bgra = Vec::with_capacity(rgba8.len());
        for pixel in rgba8.chunks_exact(4) {
            let alpha = u16::from(pixel[3]);
            let premultiply = |channel: u8| ((u16::from(channel) * alpha + 127) / 255) as u8;
            bgra.extend_from_slice(&[
                premultiply(pixel[2]),
                premultiply(pixel[1]),
                premultiply(pixel[0]),
                pixel[3],
            ]);
        }
        Ok(Self {
            id,
            width,
            height,
            premultiplied_bgra8: Arc::from(bgra),
        })
    }

    pub fn from_premultiplied_bgra8(
        id: ZsImageFrameId,
        width: u32,
        height: u32,
        premultiplied_bgra8: impl Into<Vec<u8>>,
    ) -> crate::ZsuiResult<Self> {
        let pixels = premultiplied_bgra8.into();
        validate_image_buffer(width, height, pixels.len())?;
        Ok(Self {
            id,
            width,
            height,
            premultiplied_bgra8: Arc::from(pixels),
        })
    }

    pub const fn id(&self) -> ZsImageFrameId {
        self.id
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn premultiplied_bgra8(&self) -> &[u8] {
        &self.premultiplied_bgra8
    }

    pub fn decoded_bytes(&self) -> usize {
        self.premultiplied_bgra8.len()
    }
}

fn validate_image_buffer(width: u32, height: u32, actual: usize) -> crate::ZsuiResult<()> {
    let expected = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            crate::ZsuiError::invalid_spec("image_frame.dimensions", "image dimensions overflow")
        })?;
    if width == 0 || height == 0 || actual != expected {
        return Err(crate::ZsuiError::invalid_spec(
            "image_frame.pixels",
            format!("expected {expected} BGRA/RGBA bytes for {width}x{height}, received {actual}"),
        ));
    }
    Ok(())
}

mod arc_bytes {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(bytes: &Arc<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<[u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer).map(Arc::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeImageInterpolation {
    Nearest,
    Smooth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawImageCommand {
    pub frame: ZsImageFrame,
    pub source: Rect,
    pub bounds: Rect,
    pub interpolation: NativeImageInterpolation,
}

impl NativeDrawImageCommand {
    pub const fn new(frame: ZsImageFrame, source: Rect, bounds: Rect) -> Self {
        Self {
            frame,
            source,
            bounds,
            interpolation: NativeImageInterpolation::Smooth,
        }
    }

    pub const fn interpolation(mut self, interpolation: NativeImageInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }
}

impl NativeDrawIconCommand {
    pub const fn new(icon: ZsIcon, bounds: Rect, color_mode: NativeIconColorMode) -> Self {
        Self {
            icon,
            bounds,
            color_mode,
            color: ColorRole::PrimaryText,
        }
    }

    pub const fn with_color(mut self, color: ColorRole) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeDrawCommand {
    FillRect {
        rect: Rect,
        fill: NativeDrawFill,
    },
    StrokeRect {
        rect: Rect,
        stroke: NativeDrawFill,
        width: i32,
    },
    StrokeArc {
        rect: Rect,
        stroke: NativeDrawFill,
        width: i32,
        start_degrees: i16,
        sweep_degrees: i16,
    },
    FillTriangle {
        points: [crate::Point; 3],
        fill: NativeDrawFill,
    },
    RoundRect {
        rect: Rect,
        fill: NativeDrawFill,
        stroke: Option<NativeDrawFill>,
        radius: i32,
    },
    RoundFill {
        rect: Rect,
        fill: NativeDrawFill,
        radius: i32,
    },
    Text(NativeDrawTextCommand),
    #[cfg(feature = "password-box")]
    SecureText(NativeDrawSecureTextCommand),
    Icon(NativeDrawIconCommand),
    Image(NativeDrawImageCommand),
    PushClip {
        rect: Rect,
    },
    PopClip,
}

impl NativeDrawCommand {
    pub const fn operation(&self) -> NativeDrawCommandOperation {
        match self {
            Self::FillRect { .. } => NativeDrawCommandOperation::FillRect,
            Self::StrokeRect { .. } => NativeDrawCommandOperation::StrokeRect,
            Self::StrokeArc { .. } => NativeDrawCommandOperation::StrokeArc,
            Self::FillTriangle { .. } => NativeDrawCommandOperation::FillTriangle,
            Self::RoundRect { .. } => NativeDrawCommandOperation::RoundRect,
            Self::RoundFill { .. } => NativeDrawCommandOperation::RoundFill,
            Self::Text(_) => NativeDrawCommandOperation::DrawText,
            #[cfg(feature = "password-box")]
            Self::SecureText(_) => NativeDrawCommandOperation::DrawText,
            Self::Icon(_) => NativeDrawCommandOperation::DrawIcon,
            Self::Image(_) => NativeDrawCommandOperation::DrawImage,
            Self::PushClip { .. } => NativeDrawCommandOperation::PushClip,
            Self::PopClip => NativeDrawCommandOperation::PopClip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeDrawCommandOperation {
    FillRect,
    StrokeRect,
    StrokeArc,
    FillTriangle,
    RoundRect,
    RoundFill,
    DrawText,
    DrawIcon,
    DrawImage,
    PushClip,
    PopClip,
}

impl NativeDrawCommandOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::FillRect => "draw_fill_rect",
            Self::StrokeRect => "draw_stroke_rect",
            Self::StrokeArc => "draw_stroke_arc",
            Self::FillTriangle => "draw_fill_triangle",
            Self::RoundRect => "draw_round_rect",
            Self::RoundFill => "draw_round_fill",
            Self::DrawText => "draw_text",
            Self::DrawIcon => "draw_icon",
            Self::DrawImage => "draw_image",
            Self::PushClip => "push_clip",
            Self::PopClip => "pop_clip",
        }
    }
}

pub const REQUIRED_NATIVE_DRAW_COMMAND_OPERATIONS: [NativeDrawCommandOperation; 11] = [
    NativeDrawCommandOperation::FillRect,
    NativeDrawCommandOperation::StrokeRect,
    NativeDrawCommandOperation::StrokeArc,
    NativeDrawCommandOperation::FillTriangle,
    NativeDrawCommandOperation::RoundRect,
    NativeDrawCommandOperation::RoundFill,
    NativeDrawCommandOperation::DrawText,
    NativeDrawCommandOperation::DrawIcon,
    NativeDrawCommandOperation::DrawImage,
    NativeDrawCommandOperation::PushClip,
    NativeDrawCommandOperation::PopClip,
];

pub fn required_native_draw_command_operation_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_DRAW_COMMAND_OPERATIONS
        .iter()
        .map(|operation| operation.operation_name())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawPlan {
    pub commands: Vec<NativeDrawCommand>,
    pub theme_mode: ZsuiThemeMode,
    /// Per-mille scale resolved from the target desktop's configured UI font.
    ///
    /// Integer storage keeps draw plans deterministic and comparable while
    /// allowing AppKit and GTK to feed their runtime font setting into both
    /// layout and final native text shaping.
    #[serde(default = "default_typography_scale_per_mille")]
    typography_scale_per_mille: u16,
}

pub(crate) const fn default_typography_scale_per_mille() -> u16 {
    1_000
}

pub(crate) fn normalize_typography_scale_per_mille(scale: f32) -> u16 {
    if !scale.is_finite() {
        return default_typography_scale_per_mille();
    }
    (scale.clamp(0.75, 3.0) * 1_000.0).round() as u16
}

impl Default for NativeDrawPlan {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            theme_mode: ZsuiThemeMode::System,
            typography_scale_per_mille: default_typography_scale_per_mille(),
        }
    }
}

impl NativeDrawPlan {
    pub fn new(commands: impl IntoIterator<Item = NativeDrawCommand>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
            theme_mode: ZsuiThemeMode::System,
            typography_scale_per_mille: default_typography_scale_per_mille(),
        }
    }

    pub fn theme_mode(mut self, theme_mode: ZsuiThemeMode) -> Self {
        self.theme_mode = theme_mode;
        self
    }

    pub fn typography_scale(&self) -> f32 {
        f32::from(if self.typography_scale_per_mille == 0 {
            default_typography_scale_per_mille()
        } else {
            self.typography_scale_per_mille
        }) / 1_000.0
    }

    pub(crate) fn set_typography_scale(&mut self, scale: f32) {
        self.typography_scale_per_mille = normalize_typography_scale_per_mille(scale);
    }

    pub fn push(&mut self, command: NativeDrawCommand) {
        self.commands.push(command);
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn text_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, NativeDrawCommand::Text(_)))
            .count()
    }

    pub fn icon_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, NativeDrawCommand::Icon(_)))
            .count()
    }

    pub fn image_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, NativeDrawCommand::Image(_)))
            .count()
    }
}

pub trait NativeDrawCommandSink {
    fn draw_command(&mut self, command: &NativeDrawCommand);

    fn draw_plan(&mut self, plan: &NativeDrawPlan) {
        for command in &plan.commands {
            self.draw_command(command);
        }
    }
}

#[cfg(test)]
mod draw_command_tests {
    use super::*;

    #[derive(Default)]
    struct RecordingDrawSink {
        commands: Vec<NativeDrawCommandOperation>,
    }

    impl NativeDrawCommandSink for RecordingDrawSink {
        fn draw_command(&mut self, command: &NativeDrawCommand) {
            self.commands.push(command.operation());
        }
    }

    #[test]
    fn color_rgba_text_has_one_canonical_representation() {
        let color = Color::rgba(32, 96, 160, 224);
        assert_eq!(color.hex_rgba(), "#2060A0E0");
        assert_eq!(Color::parse_hex_rgba("#2060A0E0"), Some(color));
        assert_eq!(Color::parse_hex_rgba("#2060a0e0"), None);
        assert_eq!(Color::parse_hex_rgba("#2060A0"), None);
    }

    #[test]
    fn native_draw_plan_keeps_self_draw_command_shape() {
        let rect = Rect {
            x: 8,
            y: 12,
            width: 120,
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
                radius: 8,
            },
            NativeDrawCommand::Text(NativeDrawTextCommand::new(
                "Search",
                rect,
                SemanticTextStyle::body(),
            )),
            NativeDrawCommand::Icon(NativeDrawIconCommand::new(
                ZsIcon::Search,
                rect,
                NativeIconColorMode::ThemeAware,
            )),
            NativeDrawCommand::Image(NativeDrawImageCommand::new(
                ZsImageFrame::from_rgba8(ZsImageFrameId::new(7), 1, 1, vec![255, 0, 0, 255])
                    .unwrap(),
                Rect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                rect,
            )),
        ]);

        assert_eq!(plan.command_count(), 5);
        assert_eq!(plan.text_count(), 1);
        assert_eq!(plan.icon_count(), 1);
        assert_eq!(plan.image_count(), 1);

        let mut sink = RecordingDrawSink::default();
        sink.draw_plan(&plan);
        assert_eq!(
            sink.commands,
            vec![
                NativeDrawCommandOperation::FillRect,
                NativeDrawCommandOperation::RoundRect,
                NativeDrawCommandOperation::DrawText,
                NativeDrawCommandOperation::DrawIcon,
                NativeDrawCommandOperation::DrawImage,
            ]
        );
    }

    #[test]
    fn native_draw_command_operation_names_are_stable() {
        assert_eq!(
            required_native_draw_command_operation_names(),
            vec![
                "draw_fill_rect",
                "draw_stroke_rect",
                "draw_stroke_arc",
                "draw_fill_triangle",
                "draw_round_rect",
                "draw_round_fill",
                "draw_text",
                "draw_icon",
                "draw_image",
                "push_clip",
                "pop_clip",
            ]
        );
    }

    #[test]
    fn image_frame_is_premultiplied_once_and_clones_share_storage() {
        let frame =
            ZsImageFrame::from_rgba8(ZsImageFrameId::new(11), 1, 1, vec![255, 64, 0, 128]).unwrap();
        assert_eq!(frame.premultiplied_bgra8(), &[0, 32, 128, 128]);
        let clone = frame.clone();
        assert_eq!(
            frame.premultiplied_bgra8().as_ptr(),
            clone.premultiplied_bgra8().as_ptr()
        );
        let json = serde_json::to_string(&frame).unwrap();
        let restored: ZsImageFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, frame);
    }

    #[test]
    fn semantic_roles_follow_the_fluent_type_ramp() {
        assert_eq!(
            (TextRole::Caption.size(), TextRole::Caption.line_height()),
            (12.0, 16.0)
        );
        assert_eq!(
            (TextRole::Body.size(), TextRole::Body.line_height()),
            (14.0, 20.0)
        );
        assert_eq!(
            (
                TextRole::BodyLarge.size(),
                TextRole::BodyLarge.line_height()
            ),
            (18.0, 24.0)
        );
        assert_eq!(
            (TextRole::Subtitle.size(), TextRole::Subtitle.line_height()),
            (20.0, 28.0)
        );
        assert_eq!(
            (
                TextRole::WindowTitle.size(),
                TextRole::WindowTitle.line_height()
            ),
            (24.0, 32.0)
        );
        assert_eq!(
            (TextRole::Title.size(), TextRole::Title.line_height()),
            (28.0, 36.0)
        );
        assert_eq!(
            (
                TextRole::TitleLarge.size(),
                TextRole::TitleLarge.line_height()
            ),
            (40.0, 52.0)
        );
        assert_eq!(
            (TextRole::Display.size(), TextRole::Display.line_height()),
            (68.0, 92.0)
        );
        assert_eq!(
            SemanticTextStyle::for_role(TextRole::Title).weight,
            TextWeight::Automatic
        );
    }

    #[test]
    fn semantic_roles_resolve_through_native_desktop_type_ramps() {
        let windows = TextRole::Body.metrics_for(ZsTypographyPlatformStyle::Windows);
        let macos_body = TextRole::Body.metrics_for(ZsTypographyPlatformStyle::Macos);
        let macos_caption = TextRole::Caption.metrics_for(ZsTypographyPlatformStyle::Macos);
        let macos_title = TextRole::Title.metrics_for(ZsTypographyPlatformStyle::Macos);
        let windows_title = TextRole::WindowTitle.metrics_for(ZsTypographyPlatformStyle::Windows);
        let gtk_caption = TextRole::Caption.metrics_for(ZsTypographyPlatformStyle::Gtk);

        assert_eq!((windows.size, windows.line_height), (14.0, 20.0));
        assert_eq!(
            (
                macos_body.size,
                macos_body.line_height,
                macos_body.default_weight
            ),
            (13.0, 16.0, TextWeight::Regular)
        );
        assert_eq!(
            (macos_caption.size, macos_caption.line_height),
            (10.0, 13.0)
        );
        assert_eq!(
            (
                macos_title.size,
                macos_title.line_height,
                macos_title.default_weight
            ),
            (22.0, 26.0, TextWeight::Regular)
        );
        assert_eq!(
            (
                windows_title.size,
                windows_title.line_height,
                windows_title.default_weight
            ),
            (24.0, 32.0, TextWeight::Semibold)
        );
        assert_eq!((gtk_caption.size, gtk_caption.line_height), (11.5, 16.0));
    }

    #[test]
    fn native_typography_profile_owns_role_families_scale_and_vertical_metrics() {
        let profile = NativeTypographyProfile::new(
            ZsTypographyPlatformStyle::Macos,
            "appkit",
            ".AppleSystemUIFont",
            "Menlo",
            ".AppleSystemUIFont",
            1.25,
            "coretext",
        )
        .with_role_families("System Small", "System Display")
        .with_body_vertical_metrics(12.0, 3.0, 1.0);

        assert_eq!(profile.font_family_for(TextRole::Caption), "System Small");
        assert_eq!(profile.font_family_for(TextRole::Title), "System Display");
        assert_eq!(profile.font_family_for(TextRole::Monospace), "Menlo");
        assert_eq!(profile.metrics_for(TextRole::Body).size, 16.25);
        assert_eq!(profile.body_metrics.line_height, 20.0);
        assert_eq!(profile.body_metrics.ascent, Some(12.0));

        let scaled = profile.with_typography_scale(1.5);
        assert_eq!(scaled.body_metrics.size, 19.5);
        assert!((scaled.body_metrics.ascent.expect("ascent") - 14.4).abs() < 0.0001);
    }

    #[test]
    fn native_draw_plan_typography_scale_is_deterministic_and_backward_compatible() {
        let legacy: NativeDrawPlan =
            serde_json::from_str(r#"{"commands":[],"theme_mode":"System"}"#).unwrap();
        assert_eq!(legacy.typography_scale(), 1.0);

        let mut plan = NativeDrawPlan::default();
        plan.set_typography_scale(1.375);
        assert_eq!(plan.typography_scale_per_mille, 1_375);
        assert_eq!(plan.typography_scale(), 1.375);
        plan.set_typography_scale(f32::NAN);
        assert_eq!(plan.typography_scale(), 1.0);
    }
}
