use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand, NativeDrawPlan,
    NativeIconColorMode, Rect, SemanticTextStyle, ZsIcon,
};

/// A point in a Canvas' local device-independent coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasPoint {
    pub x: Dp,
    pub y: Dp,
}

impl ZsCanvasPoint {
    pub const fn new(x: Dp, y: Dp) -> Self {
        Self { x, y }
    }
}

/// A rectangle in a Canvas' local device-independent coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasRect {
    pub x: Dp,
    pub y: Dp,
    pub width: Dp,
    pub height: Dp,
}

impl ZsCanvasRect {
    pub const fn new(x: Dp, y: Dp, width: Dp, height: Dp) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Backend-neutral Canvas primitives.
///
/// Coordinates and widths are declared in [`Dp`], while colors and text use
/// semantic roles. Native backends receive the same translated draw plan and
/// retain ownership of rasterization and system typography.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ZsCanvasPrimitive {
    FillRect {
        rect: ZsCanvasRect,
        fill: NativeDrawFill,
    },
    StrokeRect {
        rect: ZsCanvasRect,
        stroke: NativeDrawFill,
        width: Dp,
    },
    StrokeArc {
        rect: ZsCanvasRect,
        stroke: NativeDrawFill,
        width: Dp,
        start_degrees: i16,
        sweep_degrees: i16,
    },
    FillTriangle {
        points: [ZsCanvasPoint; 3],
        fill: NativeDrawFill,
    },
    RoundRect {
        rect: ZsCanvasRect,
        fill: NativeDrawFill,
        stroke: Option<NativeDrawFill>,
        radius: Dp,
    },
    RoundFill {
        rect: ZsCanvasRect,
        fill: NativeDrawFill,
        radius: Dp,
    },
    Text {
        text: String,
        rect: ZsCanvasRect,
        style: SemanticTextStyle,
    },
    Icon {
        icon: ZsIcon,
        rect: ZsCanvasRect,
        color: ColorRole,
    },
}

impl ZsCanvasPrimitive {
    pub fn fill_rect(rect: ZsCanvasRect, fill: NativeDrawFill) -> Self {
        Self::FillRect { rect, fill }
    }

    pub fn round_fill(rect: ZsCanvasRect, fill: NativeDrawFill, radius: Dp) -> Self {
        Self::RoundFill { rect, fill, radius }
    }

    pub fn text(text: impl Into<String>, rect: ZsCanvasRect, style: SemanticTextStyle) -> Self {
        Self::Text {
            text: text.into(),
            rect,
            style,
        }
    }

    pub const fn icon(icon: ZsIcon, rect: ZsCanvasRect, color: ColorRole) -> Self {
        Self::Icon { icon, rect, color }
    }
}

/// Immutable custom-drawing content retained by a Canvas View node.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasScene {
    primitives: Vec<ZsCanvasPrimitive>,
}

impl ZsCanvasScene {
    pub const fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    pub fn with(mut self, primitive: ZsCanvasPrimitive) -> Self {
        self.primitives.push(primitive);
        self
    }

    pub fn push(&mut self, primitive: ZsCanvasPrimitive) {
        self.primitives.push(primitive);
    }

    pub fn primitives(&self) -> &[ZsCanvasPrimitive] {
        &self.primitives
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }
}

impl FromIterator<ZsCanvasPrimitive> for ZsCanvasScene {
    fn from_iter<T: IntoIterator<Item = ZsCanvasPrimitive>>(iter: T) -> Self {
        Self {
            primitives: iter.into_iter().collect(),
        }
    }
}

/// Converts a local-DP Canvas scene into the shared native draw protocol.
///
/// The generated clip is always balanced and confines every primitive to the
/// Canvas' final layout bounds on Win32, AppKit and Linux renderers.
pub fn zs_canvas_native_draw_plan(bounds: Rect, scene: &ZsCanvasScene, dpi: Dpi) -> NativeDrawPlan {
    let mut plan = NativeDrawPlan::default();
    plan.push(NativeDrawCommand::PushClip { rect: bounds });
    for primitive in scene.primitives() {
        plan.push(canvas_primitive_to_native(bounds, primitive, dpi));
    }
    plan.push(NativeDrawCommand::PopClip);
    plan
}

fn canvas_primitive_to_native(
    bounds: Rect,
    primitive: &ZsCanvasPrimitive,
    dpi: Dpi,
) -> NativeDrawCommand {
    match primitive {
        ZsCanvasPrimitive::FillRect { rect, fill } => NativeDrawCommand::FillRect {
            rect: canvas_rect_to_native(bounds, *rect, dpi),
            fill: *fill,
        },
        ZsCanvasPrimitive::StrokeRect {
            rect,
            stroke,
            width,
        } => NativeDrawCommand::StrokeRect {
            rect: canvas_rect_to_native(bounds, *rect, dpi),
            stroke: *stroke,
            width: canvas_non_negative_px(*width, dpi).max(1),
        },
        ZsCanvasPrimitive::StrokeArc {
            rect,
            stroke,
            width,
            start_degrees,
            sweep_degrees,
        } => NativeDrawCommand::StrokeArc {
            rect: canvas_rect_to_native(bounds, *rect, dpi),
            stroke: *stroke,
            width: canvas_non_negative_px(*width, dpi).max(1),
            start_degrees: *start_degrees,
            sweep_degrees: *sweep_degrees,
        },
        ZsCanvasPrimitive::FillTriangle { points, fill } => NativeDrawCommand::FillTriangle {
            points: points.map(|point| canvas_point_to_native(bounds, point, dpi)),
            fill: *fill,
        },
        ZsCanvasPrimitive::RoundRect {
            rect,
            fill,
            stroke,
            radius,
        } => NativeDrawCommand::RoundRect {
            rect: canvas_rect_to_native(bounds, *rect, dpi),
            fill: *fill,
            stroke: *stroke,
            radius: canvas_non_negative_px(*radius, dpi),
        },
        ZsCanvasPrimitive::RoundFill { rect, fill, radius } => NativeDrawCommand::RoundFill {
            rect: canvas_rect_to_native(bounds, *rect, dpi),
            fill: *fill,
            radius: canvas_non_negative_px(*radius, dpi),
        },
        ZsCanvasPrimitive::Text { text, rect, style } => {
            NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                text,
                canvas_rect_to_native(bounds, *rect, dpi),
                *style,
            ))
        }
        ZsCanvasPrimitive::Icon { icon, rect, color } => NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                *icon,
                canvas_rect_to_native(bounds, *rect, dpi),
                NativeIconColorMode::ThemeAware,
            )
            .with_color(*color),
        ),
    }
}

fn canvas_point_to_native(bounds: Rect, point: ZsCanvasPoint, dpi: Dpi) -> crate::Point {
    crate::Point {
        x: bounds.x.saturating_add(canvas_signed_px(point.x, dpi)),
        y: bounds.y.saturating_add(canvas_signed_px(point.y, dpi)),
    }
}

fn canvas_rect_to_native(bounds: Rect, rect: ZsCanvasRect, dpi: Dpi) -> Rect {
    Rect {
        x: bounds.x.saturating_add(canvas_signed_px(rect.x, dpi)),
        y: bounds.y.saturating_add(canvas_signed_px(rect.y, dpi)),
        width: canvas_non_negative_px(rect.width, dpi),
        height: canvas_non_negative_px(rect.height, dpi),
    }
}

fn canvas_signed_px(value: Dp, dpi: Dpi) -> i32 {
    if value.0.is_finite() {
        value.to_px(dpi).round_i32()
    } else {
        0
    }
}

fn canvas_non_negative_px(value: Dp, dpi: Dpi) -> i32 {
    canvas_signed_px(value, dpi).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_translates_local_dp_and_balances_clip() {
        let scene = ZsCanvasScene::new()
            .with(ZsCanvasPrimitive::fill_rect(
                ZsCanvasRect::new(Dp::new(4.0), Dp::new(6.0), Dp::new(20.0), Dp::new(10.0)),
                NativeDrawFill::role(ColorRole::Accent),
            ))
            .with(ZsCanvasPrimitive::text(
                "Canvas",
                ZsCanvasRect::new(Dp::new(8.0), Dp::new(20.0), Dp::new(80.0), Dp::new(24.0)),
                SemanticTextStyle::body(),
            ));
        let plan = zs_canvas_native_draw_plan(
            Rect {
                x: 100,
                y: 40,
                width: 200,
                height: 100,
            },
            &scene,
            Dpi::new(192.0),
        );

        assert_eq!(plan.commands.len(), 4);
        assert_eq!(
            plan.commands.first(),
            Some(&NativeDrawCommand::PushClip {
                rect: Rect {
                    x: 100,
                    y: 40,
                    width: 200,
                    height: 100,
                },
            })
        );
        assert!(matches!(
            &plan.commands[1],
            NativeDrawCommand::FillRect {
                rect: Rect {
                    x: 108,
                    y: 52,
                    width: 40,
                    height: 20,
                },
                ..
            }
        ));
        assert_eq!(plan.commands.last(), Some(&NativeDrawCommand::PopClip));
    }

    #[test]
    fn canvas_sanitizes_non_finite_and_negative_extents() {
        let scene = ZsCanvasScene::new().with(ZsCanvasPrimitive::RoundFill {
            rect: ZsCanvasRect::new(
                Dp::new(f32::NAN),
                Dp::new(f32::INFINITY),
                Dp::new(-20.0),
                Dp::new(12.0),
            ),
            fill: NativeDrawFill::role(ColorRole::Control),
            radius: Dp::new(-4.0),
        });
        let plan = zs_canvas_native_draw_plan(
            Rect {
                x: 5,
                y: 7,
                width: 80,
                height: 40,
            },
            &scene,
            Dpi::standard(),
        );

        assert!(matches!(
            &plan.commands[1],
            NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: 5,
                    y: 7,
                    width: 0,
                    height: 12,
                },
                radius: 0,
                ..
            }
        ));
    }
}
