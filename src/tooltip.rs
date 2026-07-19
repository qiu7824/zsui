use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill, NativeDrawPlan,
    NativeDrawTextCommand, Point, Rect, SemanticTextStyle, TextRole, TextWeight, TextWrap,
    VerticalAlign, ViewInteractionPlan, ViewTooltipTarget, WidgetId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTooltipPlacement {
    Auto,
    Top,
    Bottom,
    Left,
    Right,
}

impl Default for ZsTooltipPlacement {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTooltipSpec {
    pub text: String,
    pub placement: ZsTooltipPlacement,
    /// Overrides the host hover delay. Intended for deterministic previews and tests.
    pub open_delay_ms: Option<u64>,
}

impl ZsTooltipSpec {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placement: ZsTooltipPlacement::Auto,
            open_delay_ms: None,
        }
    }

    pub const fn placement(mut self, placement: ZsTooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub const fn open_delay_ms(mut self, open_delay_ms: u64) -> Self {
        self.open_delay_ms = Some(open_delay_ms);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

impl From<&str> for ZsTooltipSpec {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for ZsTooltipSpec {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTooltipPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

impl ZsTooltipPlatformStyle {
    pub const fn current() -> Self {
        crate::platform_experience::PlatformExperience::current_or_desktop_fallback()
            .select_desktop(Self::Windows, Self::Macos, Self::Gtk, Self::Windows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTooltipMetrics {
    pub maximum_width: Dp,
    pub horizontal_padding: Dp,
    pub vertical_padding: Dp,
    pub gap: Dp,
    pub radius: Dp,
    pub line_height: Dp,
    pub average_character_width: Dp,
}

impl ZsTooltipMetrics {
    pub const fn for_platform(platform: ZsTooltipPlatformStyle) -> Self {
        match platform {
            ZsTooltipPlatformStyle::Windows => Self {
                maximum_width: Dp::new(320.0),
                horizontal_padding: Dp::new(8.0),
                vertical_padding: Dp::new(6.0),
                gap: Dp::new(8.0),
                radius: Dp::new(4.0),
                line_height: Dp::new(16.0),
                average_character_width: Dp::new(6.5),
            },
            ZsTooltipPlatformStyle::Macos => Self {
                maximum_width: Dp::new(300.0),
                horizontal_padding: Dp::new(7.0),
                vertical_padding: Dp::new(5.0),
                gap: Dp::new(6.0),
                radius: Dp::new(5.0),
                line_height: Dp::new(15.0),
                average_character_width: Dp::new(6.2),
            },
            ZsTooltipPlatformStyle::Gtk => Self {
                maximum_width: Dp::new(320.0),
                horizontal_padding: Dp::new(8.0),
                vertical_padding: Dp::new(6.0),
                gap: Dp::new(8.0),
                radius: Dp::new(6.0),
                line_height: Dp::new(17.0),
                average_character_width: Dp::new(6.7),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTooltipRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub radius: i32,
    pub placement: ZsTooltipPlacement,
    pub platform: ZsTooltipPlatformStyle,
}

pub fn zs_tooltip_render_plan(
    spec: &ZsTooltipSpec,
    owner: Rect,
    anchor: Point,
    viewport: Rect,
    platform: ZsTooltipPlatformStyle,
    dpi: Dpi,
) -> ZsTooltipRenderPlan {
    let metrics = ZsTooltipMetrics::for_platform(platform);
    let horizontal_padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(1);
    let vertical_padding = metrics.vertical_padding.to_px(dpi).round_i32().max(1);
    let gap = metrics.gap.to_px(dpi).round_i32().max(1);
    let line_height = metrics.line_height.to_px(dpi).round_i32().max(1);
    let character_width = metrics
        .average_character_width
        .to_px(dpi)
        .round_i32()
        .max(1);
    let maximum_width = metrics.maximum_width.to_px(dpi).round_i32().max(1);
    let maximum_text_width = maximum_width.saturating_sub(horizontal_padding * 2).max(1);
    let maximum_columns = (maximum_text_width / character_width).max(1) as usize;
    let (columns, lines) = tooltip_text_shape(&spec.text, maximum_columns);
    let width = (columns as i32)
        .saturating_mul(character_width)
        .saturating_add(horizontal_padding * 2)
        .min(maximum_width)
        .max(horizontal_padding * 2 + character_width);
    let height = (lines as i32)
        .saturating_mul(line_height)
        .saturating_add(vertical_padding * 2)
        .max(line_height + vertical_padding * 2);

    let placement = match spec.placement {
        ZsTooltipPlacement::Auto => auto_placement(owner, anchor, viewport, width, height, gap),
        placement => placement,
    };
    let bounds = clamp_rect(
        placed_bounds(placement, owner, anchor, width, height, gap),
        viewport,
    );
    ZsTooltipRenderPlan {
        bounds,
        text_bounds: Rect {
            x: bounds.x.saturating_add(horizontal_padding),
            y: bounds.y.saturating_add(vertical_padding),
            width: bounds.width.saturating_sub(horizontal_padding * 2).max(0),
            height: bounds.height.saturating_sub(vertical_padding * 2).max(0),
        },
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        placement,
        platform,
    }
}

pub fn zs_tooltip_native_draw_plan(
    plan: &ZsTooltipRenderPlan,
    spec: &ZsTooltipSpec,
) -> NativeDrawPlan {
    let text_style = SemanticTextStyle {
        role: TextRole::Caption,
        color: ColorRole::PrimaryText,
        weight: TextWeight::Regular,
        horizontal_align: HorizontalAlign::Start,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::Word,
        ellipsis: false,
    };
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.text.clone(),
            plan.text_bounds,
            text_style,
        )),
    ])
}

fn tooltip_text_shape(text: &str, maximum_columns: usize) -> (usize, usize) {
    let mut maximum_line = 0usize;
    let mut lines = 0usize;
    for source_line in text.lines().chain(text.is_empty().then_some("")) {
        let columns =
            crate::widget_render::zs_estimated_text_flow_units(source_line).max(0) as usize;
        let wrapped_lines = columns.max(1).div_ceil(maximum_columns);
        lines = lines.saturating_add(wrapped_lines);
        maximum_line = maximum_line.max(columns.min(maximum_columns).max(1));
    }
    (maximum_line.max(1), lines.max(1))
}

fn auto_placement(
    owner: Rect,
    anchor: Point,
    viewport: Rect,
    width: i32,
    height: i32,
    gap: i32,
) -> ZsTooltipPlacement {
    if anchor.y.saturating_sub(gap).saturating_sub(height) >= viewport.y {
        ZsTooltipPlacement::Top
    } else if owner
        .y
        .saturating_add(owner.height)
        .saturating_add(gap)
        .saturating_add(height)
        <= viewport.y.saturating_add(viewport.height)
    {
        ZsTooltipPlacement::Bottom
    } else if owner
        .x
        .saturating_add(owner.width)
        .saturating_add(gap)
        .saturating_add(width)
        <= viewport.x.saturating_add(viewport.width)
    {
        ZsTooltipPlacement::Right
    } else {
        ZsTooltipPlacement::Left
    }
}

fn placed_bounds(
    placement: ZsTooltipPlacement,
    owner: Rect,
    anchor: Point,
    width: i32,
    height: i32,
    gap: i32,
) -> Rect {
    match placement {
        ZsTooltipPlacement::Top | ZsTooltipPlacement::Auto => Rect {
            x: anchor.x.saturating_sub(width / 2),
            y: anchor.y.saturating_sub(gap).saturating_sub(height),
            width,
            height,
        },
        ZsTooltipPlacement::Bottom => Rect {
            x: anchor.x.saturating_sub(width / 2),
            y: owner.y.saturating_add(owner.height).saturating_add(gap),
            width,
            height,
        },
        ZsTooltipPlacement::Left => Rect {
            x: owner.x.saturating_sub(gap).saturating_sub(width),
            y: owner.y.saturating_add((owner.height - height) / 2),
            width,
            height,
        },
        ZsTooltipPlacement::Right => Rect {
            x: owner.x.saturating_add(owner.width).saturating_add(gap),
            y: owner.y.saturating_add((owner.height - height) / 2),
            width,
            height,
        },
    }
}

fn clamp_rect(rect: Rect, viewport: Rect) -> Rect {
    let width = rect.width.min(viewport.width.max(0)).max(0);
    let height = rect.height.min(viewport.height.max(0)).max(0);
    let maximum_x = viewport
        .x
        .saturating_add(viewport.width.saturating_sub(width));
    let maximum_y = viewport
        .y
        .saturating_add(viewport.height.saturating_sub(height));
    Rect {
        x: rect.x.clamp(viewport.x, maximum_x),
        y: rect.y.clamp(viewport.y, maximum_y),
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZsTooltipTiming {
    pub open_delay: Duration,
    pub visible_duration: Duration,
}

impl Default for ZsTooltipTiming {
    fn default() -> Self {
        Self {
            open_delay: Duration::from_millis(500),
            visible_duration: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ZsTooltipRuntime {
    timing: ZsTooltipTiming,
    pending_widget: Option<WidgetId>,
    show_at: Option<Instant>,
    visible_widget: Option<WidgetId>,
    hide_at: Option<Instant>,
    anchor: Point,
}

impl Default for ZsTooltipRuntime {
    fn default() -> Self {
        Self::new(ZsTooltipTiming::default())
    }
}

impl ZsTooltipRuntime {
    pub(crate) fn new(timing: ZsTooltipTiming) -> Self {
        Self {
            timing,
            pending_widget: None,
            show_at: None,
            visible_widget: None,
            hide_at: None,
            anchor: Point { x: 0, y: 0 },
        }
    }

    pub(crate) fn pointer_moved(
        &mut self,
        interaction: &ViewInteractionPlan,
        point: Point,
        now: Instant,
    ) -> bool {
        let target = interaction.tooltip_target_at(point);
        let next_widget = target.as_ref().map(|target| target.widget);
        if next_widget == self.visible_widget {
            return false;
        }
        if next_widget == self.pending_widget {
            return false;
        }
        let was_visible = self.visible_widget.take().is_some();
        self.hide_at = None;
        self.pending_widget = next_widget;
        self.anchor = point;
        self.show_at = target.map(|target| {
            now + Duration::from_millis(
                target
                    .spec
                    .open_delay_ms
                    .unwrap_or(self.timing.open_delay.as_millis() as u64),
            )
        });
        was_visible
    }

    pub(crate) fn focus_widget(
        &mut self,
        interaction: &ViewInteractionPlan,
        widget: WidgetId,
        now: Instant,
    ) -> bool {
        let Some(target) = interaction.tooltip_for_widget(widget) else {
            return self.dismiss();
        };
        let changed = self.visible_widget != Some(widget);
        self.pending_widget = None;
        self.show_at = None;
        self.visible_widget = Some(widget);
        self.hide_at = Some(now + self.timing.visible_duration);
        self.anchor = Point {
            x: target.bounds.x.saturating_add(target.bounds.width / 2),
            y: target.bounds.y,
        };
        changed
    }

    pub(crate) fn dismiss(&mut self) -> bool {
        let was_visible = self.visible_widget.take().is_some();
        self.pending_widget = None;
        self.show_at = None;
        self.hide_at = None;
        was_visible
    }

    pub(crate) fn refresh(&mut self, now: Instant) -> bool {
        if self.show_at.is_some_and(|deadline| now >= deadline) {
            self.visible_widget = self.pending_widget.take();
            self.show_at = None;
            self.hide_at = self
                .visible_widget
                .map(|_| now + self.timing.visible_duration);
            return self.visible_widget.is_some();
        }
        if self.hide_at.is_some_and(|deadline| now >= deadline) {
            self.visible_widget = None;
            self.hide_at = None;
            return true;
        }
        false
    }

    pub(crate) fn poll_interval_ms(&self, now: Instant) -> Option<u64> {
        self.show_at
            .into_iter()
            .chain(self.hide_at)
            .min()
            .map(|deadline| {
                deadline
                    .saturating_duration_since(now)
                    .as_millis()
                    .clamp(1, u64::MAX as u128) as u64
            })
    }

    pub(crate) fn visible_target(
        &self,
        interaction: &ViewInteractionPlan,
    ) -> Option<ViewTooltipTarget> {
        interaction.tooltip_for_widget(self.visible_widget?)
    }

    pub(crate) const fn anchor(&self) -> Point {
        self.anchor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> ViewTooltipTarget {
        ViewTooltipTarget {
            widget: WidgetId::new(7),
            bounds: Rect {
                x: 40,
                y: 40,
                width: 80,
                height: 32,
            },
            spec: ZsTooltipSpec::new("保存文档"),
        }
    }

    #[test]
    fn auto_placement_flips_and_clamps_inside_viewport() {
        let target = target();
        let viewport = Rect {
            x: 0,
            y: 20,
            width: 160,
            height: 120,
        };
        let plan = zs_tooltip_render_plan(
            &target.spec,
            target.bounds,
            Point { x: 80, y: 42 },
            viewport,
            ZsTooltipPlatformStyle::Windows,
            Dpi::standard(),
        );
        assert_eq!(plan.placement, ZsTooltipPlacement::Bottom);
        assert!(viewport.contains(Point {
            x: plan.bounds.x,
            y: plan.bounds.y,
        }));
        assert!(plan.bounds.x + plan.bounds.width <= viewport.x + viewport.width);
    }

    #[test]
    fn text_shape_uses_unicode_display_units_without_counting_combining_marks() {
        assert_eq!(tooltip_text_shape("AB", 20), (2, 1));
        assert_eq!(tooltip_text_shape("中文", 20), (4, 1));
        assert_eq!(tooltip_text_shape("e\u{301}", 20), (1, 1));
        assert_eq!(tooltip_text_shape("中文测试", 4), (4, 2));
    }

    #[test]
    fn runtime_delays_hover_but_opens_keyboard_focus_immediately() {
        let target = target();
        let interaction = ViewInteractionPlan {
            hit_targets: Vec::new(),
            tooltip_targets: vec![target.clone()],
        };
        let start = Instant::now();
        let mut runtime = ZsTooltipRuntime::new(ZsTooltipTiming {
            open_delay: Duration::from_millis(500),
            visible_duration: Duration::from_secs(5),
        });
        assert!(!runtime.pointer_moved(&interaction, Point { x: 60, y: 50 }, start));
        assert!(!runtime.refresh(start + Duration::from_millis(499)));
        assert!(runtime.refresh(start + Duration::from_millis(500)));
        assert!(runtime.visible_target(&interaction).is_some());
        assert!(runtime.dismiss());
        assert!(runtime.focus_widget(&interaction, target.widget, start));
        assert!(runtime.visible_target(&interaction).is_some());
    }

    #[test]
    fn native_plan_is_noninteractive_semantic_overlay() {
        let target = target();
        let render = zs_tooltip_render_plan(
            &target.spec,
            target.bounds,
            Point { x: 80, y: 40 },
            Rect {
                x: 0,
                y: 0,
                width: 400,
                height: 300,
            },
            ZsTooltipPlatformStyle::Windows,
            Dpi::standard(),
        );
        let draw = zs_tooltip_native_draw_plan(&render, &target.spec);
        assert_eq!(draw.commands.len(), 2);
        assert!(matches!(
            draw.commands.first(),
            Some(NativeDrawCommand::RoundRect {
                fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                ..
            })
        ));
    }
}
