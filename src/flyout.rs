use serde::{Deserialize, Serialize};

use crate::{
    Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Point, Rect, ZsPlatformStyle,
};

/// Preferred side of an anchored Flyout surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsFlyoutPlacement {
    #[default]
    Auto,
    Top,
    Bottom,
    Left,
    Right,
}

/// Platform-neutral size and placement for arbitrary Flyout content.
///
/// `content_width` and `content_height` describe the content area. ZSUI adds
/// the target platform's native popover padding, radius, gap and optional
/// arrow without requiring platform branches in application code.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsFlyoutSpec {
    content_width: Dp,
    content_height: Dp,
    preferred_placement: ZsFlyoutPlacement,
}

impl ZsFlyoutSpec {
    pub fn new(content_width: Dp, content_height: Dp) -> Self {
        Self {
            content_width: sanitize_extent(content_width),
            content_height: sanitize_extent(content_height),
            preferred_placement: ZsFlyoutPlacement::Auto,
        }
    }

    pub const fn preferred_placement(mut self, placement: ZsFlyoutPlacement) -> Self {
        self.preferred_placement = placement;
        self
    }

    pub const fn content_width(self) -> Dp {
        self.content_width
    }

    pub const fn content_height(self) -> Dp {
        self.content_height
    }

    pub const fn placement(self) -> ZsFlyoutPlacement {
        self.preferred_placement
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsFlyoutDismissReason {
    LightDismiss,
    EscapeKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsFlyoutState {
    pub open: bool,
    pub target: crate::WidgetId,
}

pub type ZsFlyoutPlatformStyle = ZsPlatformStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsFlyoutRenderPlan {
    pub viewport: Rect,
    pub target: Rect,
    pub surface: Rect,
    pub content: Rect,
    pub tail: Option<[Point; 3]>,
    pub placement: ZsFlyoutPlacement,
    pub radius: i32,
    pub shadow_offset: i32,
    pub shadow_alpha: u8,
    pub platform: ZsFlyoutPlatformStyle,
}

/// Resolves one platform-adaptive Flyout surface from the same application
/// declaration used on Windows, macOS and Linux.
pub fn zs_flyout_render_plan(
    viewport: Rect,
    target: Rect,
    spec: ZsFlyoutSpec,
    platform: ZsFlyoutPlatformStyle,
    dpi: Dpi,
) -> ZsFlyoutRenderPlan {
    let profile =
        crate::platform_component_profile::PlatformComponentProfile::for_style(platform).flyout;
    let margin = profile.viewport_margin.to_px(dpi).round_i32().max(0);
    let padding = profile.content_padding.to_px(dpi).round_i32().max(0);
    let gap = profile.target_gap.to_px(dpi).round_i32().max(0);
    let tail_size = if profile.draws_tail() {
        profile.tail_size.to_px(dpi).round_i32().max(1)
    } else {
        0
    };
    let maximum_width = viewport
        .width
        .saturating_sub(margin.saturating_mul(2))
        .max(1);
    let maximum_height = viewport
        .height
        .saturating_sub(margin.saturating_mul(2))
        .max(1);
    let requested_content_width = spec.content_width().to_px(dpi).round_i32().max(1);
    let requested_content_height = spec.content_height().to_px(dpi).round_i32().max(1);
    let surface_width = requested_content_width
        .saturating_add(padding.saturating_mul(2))
        .min(maximum_width)
        .max(1);
    let surface_height = requested_content_height
        .saturating_add(padding.saturating_mul(2))
        .min(maximum_height)
        .max(1);
    let placement = resolve_placement(
        viewport,
        target,
        surface_width,
        surface_height,
        margin,
        gap.saturating_add(tail_size),
        spec.placement(),
        profile.automatic_placement,
    );
    let desired = desired_surface_origin(
        target,
        surface_width,
        surface_height,
        gap,
        tail_size,
        placement,
        profile.aligns_to_leading_edge(),
    );
    let minimum_x = viewport.x.saturating_add(margin);
    let minimum_y = viewport.y.saturating_add(margin);
    let maximum_x = viewport
        .x
        .saturating_add(viewport.width)
        .saturating_sub(margin)
        .saturating_sub(surface_width)
        .max(minimum_x);
    let maximum_y = viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(margin)
        .saturating_sub(surface_height)
        .max(minimum_y);
    let surface = Rect {
        x: desired.x.clamp(minimum_x, maximum_x),
        y: desired.y.clamp(minimum_y, maximum_y),
        width: surface_width,
        height: surface_height,
    };
    let content = Rect {
        x: surface.x.saturating_add(padding),
        y: surface.y.saturating_add(padding),
        width: surface
            .width
            .saturating_sub(padding.saturating_mul(2))
            .max(0),
        height: surface
            .height
            .saturating_sub(padding.saturating_mul(2))
            .max(0),
    };
    let radius = profile.surface_radius.to_px(dpi).round_i32().max(0);

    ZsFlyoutRenderPlan {
        viewport,
        target,
        surface,
        content,
        tail: profile
            .draws_tail()
            .then(|| flyout_tail(surface, target, placement, tail_size, gap, radius)),
        placement,
        radius,
        shadow_offset: profile.shadow_offset.to_px(dpi).round_i32().max(0),
        shadow_alpha: profile.shadow_alpha,
        platform,
    }
}

pub fn zs_flyout_native_draw_plan(plan: &ZsFlyoutRenderPlan) -> NativeDrawPlan {
    let mut output = NativeDrawPlan::default();
    if plan.shadow_alpha > 0 {
        output.push(NativeDrawCommand::RoundFill {
            rect: Rect {
                x: plan.surface.x.saturating_add(plan.shadow_offset),
                y: plan.surface.y.saturating_add(plan.shadow_offset),
                ..plan.surface
            },
            fill: NativeDrawFill::RoleWithAlpha {
                role: crate::ColorRole::PrimaryText,
                alpha: plan.shadow_alpha,
            },
            radius: plan.radius,
        });
    }
    if let Some(points) = plan.tail {
        output.push(NativeDrawCommand::FillTriangle {
            points,
            fill: NativeDrawFill::role(crate::ColorRole::SurfaceRaised),
        });
    }
    output.push(NativeDrawCommand::RoundRect {
        rect: plan.surface,
        fill: NativeDrawFill::role(crate::ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::role(crate::ColorRole::Border)),
        radius: plan.radius,
    });
    output
}

fn sanitize_extent(value: Dp) -> Dp {
    if value.0.is_finite() {
        Dp::new(value.0.max(1.0))
    } else {
        Dp::new(1.0)
    }
}

fn resolve_placement(
    viewport: Rect,
    target: Rect,
    surface_width: i32,
    surface_height: i32,
    margin: i32,
    separation: i32,
    preferred: ZsFlyoutPlacement,
    automatic: ZsFlyoutPlacement,
) -> ZsFlyoutPlacement {
    let preferred = if preferred == ZsFlyoutPlacement::Auto {
        automatic
    } else {
        preferred
    };
    let order = placement_order(preferred);
    let viewport_right = viewport
        .x
        .saturating_add(viewport.width)
        .saturating_sub(margin);
    let viewport_bottom = viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(margin);
    let available = |placement| match placement {
        ZsFlyoutPlacement::Top => target.y.saturating_sub(viewport.y.saturating_add(margin)),
        ZsFlyoutPlacement::Bottom => {
            viewport_bottom.saturating_sub(target.y.saturating_add(target.height))
        }
        ZsFlyoutPlacement::Left => target.x.saturating_sub(viewport.x.saturating_add(margin)),
        ZsFlyoutPlacement::Right => {
            viewport_right.saturating_sub(target.x.saturating_add(target.width))
        }
        ZsFlyoutPlacement::Auto => 0,
    };
    let required = |placement| match placement {
        ZsFlyoutPlacement::Top | ZsFlyoutPlacement::Bottom => {
            surface_height.saturating_add(separation)
        }
        ZsFlyoutPlacement::Left | ZsFlyoutPlacement::Right => {
            surface_width.saturating_add(separation)
        }
        ZsFlyoutPlacement::Auto => i32::MAX,
    };
    order
        .into_iter()
        .find(|placement| available(*placement) >= required(*placement))
        .unwrap_or_else(|| {
            order
                .into_iter()
                .max_by_key(|placement| available(*placement))
                .unwrap_or(ZsFlyoutPlacement::Bottom)
        })
}

fn placement_order(preferred: ZsFlyoutPlacement) -> [ZsFlyoutPlacement; 4] {
    use ZsFlyoutPlacement::{Bottom, Left, Right, Top};
    match preferred {
        Top => [Top, Bottom, Right, Left],
        Bottom => [Bottom, Top, Right, Left],
        Left => [Left, Right, Bottom, Top],
        Right => [Right, Left, Bottom, Top],
        ZsFlyoutPlacement::Auto => [Bottom, Top, Right, Left],
    }
}

fn desired_surface_origin(
    target: Rect,
    width: i32,
    height: i32,
    gap: i32,
    tail: i32,
    placement: ZsFlyoutPlacement,
    leading_edge: bool,
) -> Point {
    let center_x = target.x.saturating_add(target.width / 2);
    let center_y = target.y.saturating_add(target.height / 2);
    let vertical_x = if leading_edge {
        target.x
    } else {
        center_x.saturating_sub(width / 2)
    };
    let horizontal_y = if leading_edge {
        target.y
    } else {
        center_y.saturating_sub(height / 2)
    };
    match placement {
        ZsFlyoutPlacement::Top => Point {
            x: vertical_x,
            y: target
                .y
                .saturating_sub(gap)
                .saturating_sub(tail)
                .saturating_sub(height),
        },
        ZsFlyoutPlacement::Bottom => Point {
            x: vertical_x,
            y: target
                .y
                .saturating_add(target.height)
                .saturating_add(gap)
                .saturating_add(tail),
        },
        ZsFlyoutPlacement::Left => Point {
            x: target
                .x
                .saturating_sub(gap)
                .saturating_sub(tail)
                .saturating_sub(width),
            y: horizontal_y,
        },
        ZsFlyoutPlacement::Right => Point {
            x: target
                .x
                .saturating_add(target.width)
                .saturating_add(gap)
                .saturating_add(tail),
            y: horizontal_y,
        },
        ZsFlyoutPlacement::Auto => unreachable!("automatic placement is resolved"),
    }
}

fn flyout_tail(
    surface: Rect,
    target: Rect,
    placement: ZsFlyoutPlacement,
    size: i32,
    gap: i32,
    radius: i32,
) -> [Point; 3] {
    let half = (size / 2).max(1);
    let surface_right = surface.x.saturating_add(surface.width);
    let surface_bottom = surface.y.saturating_add(surface.height);
    let target_center_x = target.x.saturating_add(target.width / 2);
    let target_center_y = target.y.saturating_add(target.height / 2);
    let inset = radius.saturating_add(half).max(size);
    match placement {
        ZsFlyoutPlacement::Top => {
            let center = clamped_tail_center(target_center_x, surface.x, surface.width, inset);
            [
                Point {
                    x: center - half,
                    y: surface_bottom,
                },
                Point {
                    x: center + half,
                    y: surface_bottom,
                },
                Point {
                    x: target_center_x,
                    y: target.y.saturating_sub(gap),
                },
            ]
        }
        ZsFlyoutPlacement::Bottom => {
            let center = clamped_tail_center(target_center_x, surface.x, surface.width, inset);
            [
                Point {
                    x: center - half,
                    y: surface.y,
                },
                Point {
                    x: center + half,
                    y: surface.y,
                },
                Point {
                    x: target_center_x,
                    y: target.y.saturating_add(target.height).saturating_add(gap),
                },
            ]
        }
        ZsFlyoutPlacement::Left => {
            let center = clamped_tail_center(target_center_y, surface.y, surface.height, inset);
            [
                Point {
                    x: surface_right,
                    y: center - half,
                },
                Point {
                    x: surface_right,
                    y: center + half,
                },
                Point {
                    x: target.x.saturating_sub(gap),
                    y: target_center_y,
                },
            ]
        }
        ZsFlyoutPlacement::Right => {
            let center = clamped_tail_center(target_center_y, surface.y, surface.height, inset);
            [
                Point {
                    x: surface.x,
                    y: center - half,
                },
                Point {
                    x: surface.x,
                    y: center + half,
                },
                Point {
                    x: target.x.saturating_add(target.width).saturating_add(gap),
                    y: target_center_y,
                },
            ]
        }
        ZsFlyoutPlacement::Auto => unreachable!("automatic placement is resolved"),
    }
}

fn clamped_tail_center(target: i32, start: i32, extent: i32, requested_inset: i32) -> i32 {
    let inset = requested_inset.min(extent.max(0) / 2).max(0);
    target.clamp(
        start.saturating_add(inset),
        start.saturating_add(extent).saturating_sub(inset),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Rect = Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };
    const TARGET: Rect = Rect {
        x: 360,
        y: 260,
        width: 80,
        height: 32,
    };

    #[test]
    fn flyout_spec_sanitizes_non_finite_and_negative_content_extents() {
        let spec = ZsFlyoutSpec::new(Dp::new(f32::NAN), Dp::new(-20.0));
        assert_eq!(spec.content_width(), Dp::new(1.0));
        assert_eq!(spec.content_height(), Dp::new(1.0));
    }

    #[test]
    fn flyout_profiles_keep_platform_composition_out_of_application_code() {
        let spec = ZsFlyoutSpec::new(Dp::new(220.0), Dp::new(120.0));
        let windows = zs_flyout_render_plan(
            VIEWPORT,
            TARGET,
            spec,
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_flyout_render_plan(
            VIEWPORT,
            TARGET,
            spec,
            ZsPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_flyout_render_plan(
            VIEWPORT,
            TARGET,
            spec,
            ZsPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.placement, ZsFlyoutPlacement::Bottom);
        assert_eq!(macos.placement, ZsFlyoutPlacement::Right);
        assert_eq!(gtk.placement, ZsFlyoutPlacement::Bottom);
        assert!(windows.tail.is_none());
        assert!(macos.tail.is_some());
        assert!(gtk.tail.is_some());
        assert_ne!(windows.radius, macos.radius);
        assert_ne!(macos.radius, gtk.radius);
    }

    #[test]
    fn flyout_flips_and_clamps_at_window_edges() {
        let target = Rect {
            x: 740,
            y: 550,
            width: 48,
            height: 32,
        };
        let plan = zs_flyout_render_plan(
            VIEWPORT,
            target,
            ZsFlyoutSpec::new(Dp::new(260.0), Dp::new(160.0))
                .preferred_placement(ZsFlyoutPlacement::Bottom),
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );

        assert_eq!(plan.placement, ZsFlyoutPlacement::Top);
        assert!(plan.surface.x >= VIEWPORT.x);
        assert!(plan.surface.y >= VIEWPORT.y);
        assert!(plan.surface.x + plan.surface.width <= VIEWPORT.width);
        assert!(plan.surface.y + plan.surface.height <= VIEWPORT.height);
    }

    #[test]
    fn flyout_draw_plan_uses_semantic_surface_border_and_platform_tail() {
        let plan = zs_flyout_render_plan(
            VIEWPORT,
            TARGET,
            ZsFlyoutSpec::new(Dp::new(200.0), Dp::new(100.0)),
            ZsPlatformStyle::Macos,
            Dpi::standard(),
        );
        let draw = zs_flyout_native_draw_plan(&plan);

        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::FillTriangle {
                fill: NativeDrawFill::Role(crate::ColorRole::SurfaceRaised),
                ..
            }
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect {
                fill: NativeDrawFill::Role(crate::ColorRole::SurfaceRaised),
                stroke: Some(NativeDrawFill::Role(crate::ColorRole::Border)),
                ..
            }
        )));
    }

    #[test]
    fn tiny_gtk_viewport_keeps_tail_geometry_total_and_non_panicking() {
        let viewport = Rect {
            x: 10,
            y: 20,
            width: 24,
            height: 24,
        };
        let plan = zs_flyout_render_plan(
            viewport,
            Rect {
                x: 18,
                y: 28,
                width: 4,
                height: 4,
            },
            ZsFlyoutSpec::new(Dp::new(1.0), Dp::new(1.0)),
            ZsPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(plan.surface.width, 1);
        assert_eq!(plan.surface.height, 1);
        assert!(plan.tail.is_some());
    }
}
