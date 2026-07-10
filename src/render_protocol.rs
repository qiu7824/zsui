use serde::{Deserialize, Serialize};

use crate::{
    geometry::{Rect, Size},
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
    Title,
    Button,
    Icon,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorRole {
    PrimaryText,
    SecondaryText,
    Accent,
    Surface,
    Control,
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
    pub const fn body() -> Self {
        Self {
            role: TextRole::Body,
            color: ColorRole::PrimaryText,
            weight: TextWeight::Regular,
            horizontal_align: HorizontalAlign::Start,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub size: f32,
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
    fn draw_text(&mut self, run: &TextRun, style: &TextStyle);
    fn push_clip(&mut self, rect: Rect);
    fn pop_clip(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererHostOperation {
    FillRect,
    StrokeRect,
    DrawText,
    PushClip,
    PopClip,
}

impl RendererHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::FillRect => "fill_rect",
            Self::StrokeRect => "stroke_rect",
            Self::DrawText => "draw_text",
            Self::PushClip => "push_clip",
            Self::PopClip => "pop_clip",
        }
    }
}

pub const REQUIRED_RENDERER_HOST_OPERATIONS: [RendererHostOperation; 5] = [
    RendererHostOperation::FillRect,
    RendererHostOperation::StrokeRect,
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
}

impl NativeDrawIconCommand {
    pub const fn new(icon: ZsIcon, bounds: Rect, color_mode: NativeIconColorMode) -> Self {
        Self {
            icon,
            bounds,
            color_mode,
        }
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
    Icon(NativeDrawIconCommand),
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
            Self::RoundRect { .. } => NativeDrawCommandOperation::RoundRect,
            Self::RoundFill { .. } => NativeDrawCommandOperation::RoundFill,
            Self::Text(_) => NativeDrawCommandOperation::DrawText,
            Self::Icon(_) => NativeDrawCommandOperation::DrawIcon,
            Self::PushClip { .. } => NativeDrawCommandOperation::PushClip,
            Self::PopClip => NativeDrawCommandOperation::PopClip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeDrawCommandOperation {
    FillRect,
    StrokeRect,
    RoundRect,
    RoundFill,
    DrawText,
    DrawIcon,
    PushClip,
    PopClip,
}

impl NativeDrawCommandOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::FillRect => "draw_fill_rect",
            Self::StrokeRect => "draw_stroke_rect",
            Self::RoundRect => "draw_round_rect",
            Self::RoundFill => "draw_round_fill",
            Self::DrawText => "draw_text",
            Self::DrawIcon => "draw_icon",
            Self::PushClip => "push_clip",
            Self::PopClip => "pop_clip",
        }
    }
}

pub const REQUIRED_NATIVE_DRAW_COMMAND_OPERATIONS: [NativeDrawCommandOperation; 8] = [
    NativeDrawCommandOperation::FillRect,
    NativeDrawCommandOperation::StrokeRect,
    NativeDrawCommandOperation::RoundRect,
    NativeDrawCommandOperation::RoundFill,
    NativeDrawCommandOperation::DrawText,
    NativeDrawCommandOperation::DrawIcon,
    NativeDrawCommandOperation::PushClip,
    NativeDrawCommandOperation::PopClip,
];

pub fn required_native_draw_command_operation_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_DRAW_COMMAND_OPERATIONS
        .iter()
        .map(|operation| operation.operation_name())
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawPlan {
    pub commands: Vec<NativeDrawCommand>,
}

impl NativeDrawPlan {
    pub fn new(commands: impl IntoIterator<Item = NativeDrawCommand>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
        }
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
        ]);

        assert_eq!(plan.command_count(), 4);
        assert_eq!(plan.text_count(), 1);
        assert_eq!(plan.icon_count(), 1);

        let mut sink = RecordingDrawSink::default();
        sink.draw_plan(&plan);
        assert_eq!(
            sink.commands,
            vec![
                NativeDrawCommandOperation::FillRect,
                NativeDrawCommandOperation::RoundRect,
                NativeDrawCommandOperation::DrawText,
                NativeDrawCommandOperation::DrawIcon,
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
                "draw_round_rect",
                "draw_round_fill",
                "draw_text",
                "draw_icon",
                "push_clip",
                "pop_clip",
            ]
        );
    }
}
