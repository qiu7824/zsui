use serde::{Deserialize, Serialize};

use crate::geometry::{Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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
