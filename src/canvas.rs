use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    Color, ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand,
    NativeDrawPlan, NativeIconColorMode, Rect, SemanticTextStyle, WidgetId, ZsIcon,
    ZsPointerButton, ZsPointerModifiers,
};

/// Lifecycle phase for a Canvas pointer capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsCanvasPointerPhase {
    Pressed,
    Moved,
    Released,
    Cancelled,
}

/// A typed pointer event expressed in the Canvas' local DP coordinate space.
///
/// Drag positions remain unbounded after capture; use `inside` to distinguish
/// movement inside the final Canvas bounds from movement outside them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasPointerEvent {
    pub widget: WidgetId,
    pub phase: ZsCanvasPointerPhase,
    pub position: ZsCanvasPoint,
    pub button: ZsPointerButton,
    pub modifiers: ZsPointerModifiers,
    pub inside: bool,
}

impl ZsCanvasPointerEvent {
    pub const fn new(
        widget: WidgetId,
        phase: ZsCanvasPointerPhase,
        position: ZsCanvasPoint,
        button: ZsPointerButton,
        modifiers: ZsPointerModifiers,
        inside: bool,
    ) -> Self {
        Self {
            widget,
            phase,
            position,
            button,
            modifiers,
            inside,
        }
    }
}

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
    /// Text in an explicit brand color, for example on a dark navigation
    /// rail or an avatar. High-contrast mode falls back to `style.color`.
    ColoredText {
        text: String,
        rect: ZsCanvasRect,
        style: SemanticTextStyle,
        color: Color,
    },
    /// A theme-aware icon tinted with an explicit brand color. High-contrast
    /// mode falls back to `fallback`.
    ColoredIcon {
        icon: ZsIcon,
        rect: ZsCanvasRect,
        color: Color,
        fallback: ColorRole,
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

    /// Text drawn in `color`; `style.color` remains the high-contrast fallback.
    pub fn colored_text(
        text: impl Into<String>,
        rect: ZsCanvasRect,
        style: SemanticTextStyle,
        color: Color,
    ) -> Self {
        Self::ColoredText {
            text: text.into(),
            rect,
            style,
            color,
        }
    }

    /// An icon tinted with `color`; `fallback` is used in high contrast.
    pub const fn colored_icon(
        icon: ZsIcon,
        rect: ZsCanvasRect,
        color: Color,
        fallback: ColorRole,
    ) -> Self {
        Self::ColoredIcon {
            icon,
            rect,
            color,
            fallback,
        }
    }
}

/// Immutable custom-drawing content retained by a Canvas View node.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasScene {
    primitives: Vec<ZsCanvasPrimitive>,
    /// Natural content height requested by a size-aware Canvas. Layout uses it
    /// when the node declares no height, for example inside a Scroll.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    extent_height: Option<Dp>,
    /// Rectangles that report pointer hover through `on_canvas_hover`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    hover_regions: Vec<ZsCanvasHoverRegion>,
}

/// A rectangle of a Canvas scene that reports pointer hover.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasHoverRegion {
    pub id: u64,
    pub rect: ZsCanvasRect,
}

impl ZsCanvasScene {
    pub const fn new() -> Self {
        Self {
            primitives: Vec::new(),
            extent_height: None,
            hover_regions: Vec::new(),
        }
    }

    /// Declares the natural height of this scene's content in local DP.
    ///
    /// Only size-aware canvases created with [`crate::canvas_with`] consult it:
    /// layout asks the builder for the scene at the available width and uses
    /// this height when the node has no explicit height.
    pub fn with_extent_height(mut self, height: Dp) -> Self {
        self.extent_height = Some(Dp::new(sanitized_extent(height.0)));
        self
    }

    pub fn set_extent_height(&mut self, height: Dp) {
        self.extent_height = Some(Dp::new(sanitized_extent(height.0)));
    }

    pub fn extent_height(&self) -> Option<Dp> {
        self.extent_height
    }

    /// Declares a hover region, for example one per tab, row or button
    /// drawn in this scene. Later regions sit above earlier ones.
    pub fn with_hover_region(mut self, id: u64, rect: ZsCanvasRect) -> Self {
        self.push_hover_region(id, rect);
        self
    }

    pub fn push_hover_region(&mut self, id: u64, rect: ZsCanvasRect) {
        self.hover_regions.push(ZsCanvasHoverRegion { id, rect });
    }

    pub fn hover_regions(&self) -> &[ZsCanvasHoverRegion] {
        &self.hover_regions
    }

    /// The topmost hover region containing `point`, in local DP.
    pub fn hover_region_at(&self, point: ZsCanvasPoint) -> Option<u64> {
        self.hover_regions
            .iter()
            .rev()
            .find(|region| {
                let rect = region.rect;
                point.x.0 >= rect.x.0
                    && point.y.0 >= rect.y.0
                    && point.x.0 < rect.x.0 + rect.width.0
                    && point.y.0 < rect.y.0 + rect.height.0
            })
            .map(|region| region.id)
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
            extent_height: None,
            hover_regions: Vec::new(),
        }
    }
}

fn sanitized_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Final size of a Canvas in its own local DP coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsCanvasSize {
    pub width: Dp,
    pub height: Dp,
}

impl ZsCanvasSize {
    pub const fn new(width: Dp, height: Dp) -> Self {
        Self { width, height }
    }
}

/// Layout-time context handed to a size-aware Canvas builder.
///
/// It carries the Canvas' final local size and measures text with the same
/// bounded, backend-produced text measurements that ordinary labels use.
/// Measuring with `max_width` equal to the width of the Text primitive that is
/// later drawn guarantees the geometry converges on the native text engine:
/// the first frame may use the framework estimate, every later layout uses the
/// renderer's own measurement.
pub struct ZsCanvasLayoutContext<'a> {
    size: ZsCanvasSize,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&'a crate::view::ViewTextMeasurements>,
}

impl<'a> ZsCanvasLayoutContext<'a> {
    pub(crate) fn new(
        size: ZsCanvasSize,
        dpi: Dpi,
        typography_scale: f32,
        measurements: Option<&'a crate::view::ViewTextMeasurements>,
    ) -> Self {
        Self {
            size,
            dpi,
            typography_scale,
            measurements,
        }
    }

    /// A context without native measurements, for previews and tests. Text
    /// measurement falls back to the framework estimate.
    pub fn detached(size: ZsCanvasSize) -> ZsCanvasLayoutContext<'static> {
        ZsCanvasLayoutContext {
            size,
            dpi: Dpi::standard(),
            typography_scale: 1.0,
            measurements: None,
        }
    }

    /// Final local size of the Canvas. During natural-height measurement the
    /// height is zero and only the width is meaningful.
    pub fn size(&self) -> ZsCanvasSize {
        self.size
    }

    /// Measures `text` in local DP. `max_width` only applies to
    /// [`crate::TextWrap::Word`] styles; other styles measure a single line.
    pub fn measure_text(
        &self,
        text: &str,
        style: SemanticTextStyle,
        max_width: Option<Dp>,
    ) -> ZsCanvasSize {
        let max_px = max_width
            .filter(|_| style.wrap == crate::TextWrap::Word)
            .map(|width| canvas_non_negative_px(width, self.dpi))
            .unwrap_or(0);
        if let Some(size) = self
            .measurements
            .and_then(|measurements| measurements.measure(text, style, max_px))
        {
            let scale = self.dpi.scale_factor().max(f32::EPSILON);
            return ZsCanvasSize::new(
                Dp::new(size.width.max(0) as f32 / scale),
                Dp::new(size.height.max(0) as f32 / scale),
            );
        }
        self.estimate_text(text, style, max_width)
    }

    fn estimate_text(
        &self,
        text: &str,
        style: SemanticTextStyle,
        max_width: Option<Dp>,
    ) -> ZsCanvasSize {
        let metrics = style
            .role
            .metrics_for(crate::ZsTypographyPlatformStyle::current());
        let scale = self.typography_scale.max(0.0);
        // Framework width units: one unit per Latin cell, two per CJK cell.
        let unit = metrics.size * 0.5 * scale;
        let weight = match style.weight {
            crate::TextWeight::Semibold | crate::TextWeight::Bold => 1.08,
            crate::TextWeight::Medium => 1.04,
            crate::TextWeight::Automatic | crate::TextWeight::Regular => 1.0,
        };
        let line_height = metrics.line_height * scale;
        let wrap_width = max_width
            .filter(|_| style.wrap == crate::TextWrap::Word)
            .map(|width| width.0.max(1.0));
        let mut widest = 0.0_f32;
        let mut rows = 0_u32;
        for line in text.split('\n') {
            let width =
                crate::widget_render::zs_estimated_text_width_units(line) as f32 * unit * weight;
            match wrap_width {
                Some(limit) if width > limit => {
                    rows += (width / limit).ceil().max(1.0) as u32;
                    widest = widest.max(limit);
                }
                _ => {
                    rows += 1;
                    widest = widest.max(width);
                }
            }
        }
        ZsCanvasSize::new(
            Dp::new(widest.ceil()),
            Dp::new(line_height * rows.max(1) as f32),
        )
    }
}

type ZsCanvasBuildFn = dyn Fn(&ZsCanvasLayoutContext<'_>) -> ZsCanvasScene + Send + Sync;

/// Rebuilds a Canvas scene from the Canvas' final layout size.
///
/// Layout invokes it once the Canvas has bounds (and once more at the
/// available width when the node needs a natural height), so applications can
/// right-align, wrap and size custom drawing without platform handles.
#[derive(Clone)]
pub struct ZsCanvasBuilder {
    build: Arc<ZsCanvasBuildFn>,
}

impl ZsCanvasBuilder {
    pub fn new(
        build: impl Fn(&ZsCanvasLayoutContext<'_>) -> ZsCanvasScene + Send + Sync + 'static,
    ) -> Self {
        Self {
            build: Arc::new(build),
        }
    }

    pub fn build(&self, context: &ZsCanvasLayoutContext<'_>) -> ZsCanvasScene {
        (self.build)(context)
    }
}

impl fmt::Debug for ZsCanvasBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZsCanvasBuilder")
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
        ZsCanvasPrimitive::ColoredText {
            text,
            rect,
            style,
            color,
        } => NativeDrawCommand::Text(
            crate::NativeDrawTextCommand::new(
                text,
                canvas_rect_to_native(bounds, *rect, dpi),
                *style,
            )
            .with_color(*color),
        ),
        ZsCanvasPrimitive::ColoredIcon {
            icon,
            rect,
            color,
            fallback,
        } => NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                *icon,
                canvas_rect_to_native(bounds, *rect, dpi),
                NativeIconColorMode::ThemeAware,
            )
            .with_color(*fallback)
            .with_custom_color(*color),
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

pub(crate) fn zs_canvas_pointer_event(
    widget: WidgetId,
    bounds: Rect,
    point: crate::Point,
    dpi: Dpi,
    phase: ZsCanvasPointerPhase,
    button: ZsPointerButton,
    modifiers: ZsPointerModifiers,
) -> ZsCanvasPointerEvent {
    let scale = dpi.scale_factor();
    ZsCanvasPointerEvent::new(
        widget,
        phase,
        ZsCanvasPoint::new(
            Dp::new((point.x.saturating_sub(bounds.x)) as f32 / scale),
            Dp::new((point.y.saturating_sub(bounds.y)) as f32 / scale),
        ),
        button,
        modifiers,
        bounds.contains(point),
    )
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

    #[test]
    fn canvas_pointer_events_use_local_dp_and_preserve_outside_drag_positions() {
        let event = zs_canvas_pointer_event(
            WidgetId::new(7),
            Rect {
                x: 100,
                y: 40,
                width: 200,
                height: 100,
            },
            crate::Point { x: 80, y: 260 },
            Dpi::new(192.0),
            ZsCanvasPointerPhase::Moved,
            ZsPointerButton::Secondary,
            ZsPointerModifiers::new(true, false, true, false),
        );

        assert_eq!(
            event.position,
            ZsCanvasPoint::new(Dp::new(-10.0), Dp::new(110.0))
        );
        assert!(!event.inside);
        assert_eq!(event.button, ZsPointerButton::Secondary);
        assert!(event.modifiers.shift);
        assert!(event.modifiers.alt);
    }
}
