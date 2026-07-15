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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

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
    Title,
    TitleLarge,
    Display,
    Button,
    Icon,
    Monospace,
}

impl TextRole {
    /// Returns the platform-independent Fluent type-ramp size in device-independent pixels.
    pub const fn size(self) -> f32 {
        match self {
            Self::Caption => 12.0,
            Self::Body | Self::Button => 14.0,
            Self::Icon => 16.0,
            Self::Monospace => 13.0,
            Self::BodyLarge => 18.0,
            Self::Subtitle => 20.0,
            Self::Title => 28.0,
            Self::TitleLarge => 40.0,
            Self::Display => 68.0,
        }
    }

    /// Returns the Fluent type-ramp line box in device-independent pixels.
    pub const fn line_height(self) -> f32 {
        match self {
            Self::Caption => 16.0,
            Self::Body | Self::Button | Self::Icon => 20.0,
            Self::Monospace => 18.0,
            Self::BodyLarge => 24.0,
            Self::Subtitle => 28.0,
            Self::Title => 36.0,
            Self::TitleLarge => 52.0,
            Self::Display => 92.0,
        }
    }

    pub const fn default_weight(self) -> TextWeight {
        match self {
            Self::Subtitle | Self::Title | Self::TitleLarge | Self::Display => TextWeight::Semibold,
            _ => TextWeight::Regular,
        }
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
            weight: role.default_weight(),
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
}

impl Default for NativeDrawPlan {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            theme_mode: ZsuiThemeMode::System,
        }
    }
}

impl NativeDrawPlan {
    pub fn new(commands: impl IntoIterator<Item = NativeDrawCommand>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
            theme_mode: ZsuiThemeMode::System,
        }
    }

    pub fn theme_mode(mut self, theme_mode: ZsuiThemeMode) -> Self {
        self.theme_mode = theme_mode;
        self
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
            TextWeight::Semibold
        );
    }
}
