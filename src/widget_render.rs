#[cfg(feature = "tree")]
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[cfg(feature = "color-picker")]
use crate::Point;
#[cfg(any(
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "date-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast",
    feature = "toggle-button"
))]
use crate::TextRole;
#[cfg(feature = "auto-suggest")]
use crate::ZsAutoSuggestion;
#[cfg(feature = "date-picker")]
use crate::ZsDate;
use crate::{Color, ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect};
#[cfg(any(
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "date-picker",
    feature = "table",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast"
))]
use crate::{HorizontalAlign, TextWeight};
#[cfg(all(
    feature = "grid-view",
    not(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "combo",
        feature = "date-picker",
        feature = "info-bar",
        feature = "teaching-tip",
        feature = "table",
        feature = "time-picker",
        feature = "toast",
        feature = "tree"
    ))
))]
use crate::{NativeDrawIconCommand, NativeIconColorMode};
#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "table",
    feature = "time-picker",
    feature = "toast",
    feature = "tree"
))]
use crate::{NativeDrawIconCommand, NativeIconColorMode, ZsIcon};
#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "number-box",
    feature = "table",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast",
    feature = "toggle-button",
    feature = "tree"
))]
use crate::{NativeDrawTextCommand, SemanticTextStyle};
#[cfg(feature = "time-picker")]
use crate::{ZsClockFormat, ZsMinuteIncrement, ZsTime};
#[cfg(feature = "color-picker")]
use crate::{ZsColorChannel, ZsColorPickerState, ZsHsvColor};

#[cfg(feature = "info-bar")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsInfoBarPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "info-bar")]
impl ZsInfoBarPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "info-bar")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsInfoBarMetrics {
    pub minimum_height: Dp,
    pub horizontal_padding: Dp,
    pub vertical_padding: Dp,
    pub content_gap: Dp,
    pub control_gap: Dp,
    pub control_height: Dp,
    pub icon_size: Dp,
    pub accent_width: Dp,
    pub surface_radius: Dp,
    pub control_radius: Dp,
    pub title_line_height: Dp,
    pub message_line_height: Dp,
    pub average_character_width: Dp,
}

#[cfg(feature = "info-bar")]
impl ZsInfoBarMetrics {
    pub const fn for_platform(platform: ZsInfoBarPlatformStyle) -> Self {
        match platform {
            ZsInfoBarPlatformStyle::Windows => Self {
                minimum_height: Dp::new(64.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(10.0),
                content_gap: Dp::new(12.0),
                control_gap: Dp::new(8.0),
                control_height: Dp::new(32.0),
                icon_size: Dp::new(20.0),
                accent_width: Dp::new(4.0),
                surface_radius: Dp::new(4.0),
                control_radius: Dp::new(4.0),
                title_line_height: Dp::new(20.0),
                message_line_height: Dp::new(18.0),
                average_character_width: Dp::new(7.2),
            },
            ZsInfoBarPlatformStyle::Macos => Self {
                minimum_height: Dp::new(56.0),
                horizontal_padding: Dp::new(14.0),
                vertical_padding: Dp::new(8.0),
                content_gap: Dp::new(10.0),
                control_gap: Dp::new(6.0),
                control_height: Dp::new(28.0),
                icon_size: Dp::new(18.0),
                accent_width: Dp::new(0.0),
                surface_radius: Dp::new(8.0),
                control_radius: Dp::new(6.0),
                title_line_height: Dp::new(18.0),
                message_line_height: Dp::new(16.0),
                average_character_width: Dp::new(6.8),
            },
            ZsInfoBarPlatformStyle::Gtk => Self {
                minimum_height: Dp::new(48.0),
                horizontal_padding: Dp::new(12.0),
                vertical_padding: Dp::new(6.0),
                content_gap: Dp::new(10.0),
                control_gap: Dp::new(8.0),
                control_height: Dp::new(34.0),
                icon_size: Dp::new(18.0),
                accent_width: Dp::new(0.0),
                surface_radius: Dp::new(0.0),
                control_radius: Dp::new(17.0),
                title_line_height: Dp::new(18.0),
                message_line_height: Dp::new(16.0),
                average_character_width: Dp::new(7.2),
            },
        }
    }
}

#[cfg(feature = "info-bar")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsInfoBarRenderPlan {
    pub surface: Rect,
    pub accent_bounds: Option<Rect>,
    pub icon_bounds: Rect,
    pub title_bounds: Option<Rect>,
    pub message_bounds: Option<Rect>,
    pub action_bounds: Option<Rect>,
    pub close_bounds: Option<Rect>,
    pub focused_control: Option<crate::ZsInfoBarControl>,
    pub surface_radius: i32,
    pub control_radius: i32,
    pub platform: ZsInfoBarPlatformStyle,
}

#[cfg(feature = "info-bar")]
pub fn zs_info_bar_render_plan(
    bounds: Rect,
    spec: &crate::ZsInfoBarSpec,
    focused_control: Option<crate::ZsInfoBarControl>,
    platform: ZsInfoBarPlatformStyle,
    dpi: Dpi,
) -> ZsInfoBarRenderPlan {
    let metrics = ZsInfoBarMetrics::for_platform(platform);
    let horizontal_padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let vertical_padding = metrics.vertical_padding.to_px(dpi).round_i32().max(0);
    let content_gap = metrics.content_gap.to_px(dpi).round_i32().max(0);
    let control_gap = metrics.control_gap.to_px(dpi).round_i32().max(0);
    let control_height = metrics.control_height.to_px(dpi).round_i32().max(1);
    let icon_size = metrics.icon_size.to_px(dpi).round_i32().max(1);
    let accent_width = metrics.accent_width.to_px(dpi).round_i32().max(0);
    let title_line_height = metrics.title_line_height.to_px(dpi).round_i32().max(1);
    let message_line_height = metrics.message_line_height.to_px(dpi).round_i32().max(1);
    let character_width = metrics
        .average_character_width
        .to_px(dpi)
        .round_i32()
        .max(1);
    let surface = Rect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(0),
        height: bounds.height.max(0),
    };
    let accent_bounds = (accent_width > 0).then_some(Rect {
        x: surface.x,
        y: surface.y,
        width: accent_width.min(surface.width),
        height: surface.height,
    });
    let surface_right = surface.x.saturating_add(surface.width);
    let content_left = surface
        .x
        .saturating_add(horizontal_padding)
        .saturating_add(accent_width)
        .min(surface_right);
    let icon_width = icon_size.min(surface_right.saturating_sub(content_left).max(0));
    let icon_bounds = Rect {
        x: content_left,
        y: surface
            .y
            .saturating_add((surface.height.saturating_sub(icon_size)) / 2),
        width: icon_width,
        height: icon_size.min(surface.height),
    };
    let mut trailing_x = surface_right
        .saturating_sub(horizontal_padding)
        .max(surface.x);
    let close_bounds = spec.is_closable().then(|| {
        let width = control_height.min(trailing_x.saturating_sub(surface.x).max(0));
        trailing_x = trailing_x.saturating_sub(width).max(surface.x);
        Rect {
            x: trailing_x,
            y: surface
                .y
                .saturating_add((surface.height.saturating_sub(control_height)) / 2),
            width,
            height: control_height.min(surface.height),
        }
    });
    if close_bounds.is_some() {
        trailing_x = trailing_x.saturating_sub(control_gap).max(surface.x);
    }
    let action_bounds = spec.action_label().map(|label| {
        let desired_width = (label.chars().count() as i32)
            .saturating_mul(character_width)
            .saturating_add(horizontal_padding)
            .max(control_height);
        let width = desired_width.min(trailing_x.saturating_sub(surface.x).max(0));
        trailing_x = trailing_x.saturating_sub(width).max(surface.x);
        Rect {
            x: trailing_x,
            y: surface
                .y
                .saturating_add((surface.height.saturating_sub(control_height)) / 2),
            width,
            height: control_height.min(surface.height),
        }
    });
    if action_bounds.is_some() {
        trailing_x = trailing_x.saturating_sub(content_gap).max(surface.x);
    }
    let text_left = icon_bounds
        .x
        .saturating_add(icon_bounds.width)
        .saturating_add(content_gap);
    let text_width = trailing_x.saturating_sub(text_left).max(0);
    let has_title = spec.title_text().is_some();
    let has_message = !spec.message().trim().is_empty();
    let total_text_height = match (has_title, has_message) {
        (true, true) => title_line_height.saturating_add(message_line_height),
        (true, false) => title_line_height,
        (false, true) => message_line_height,
        (false, false) => 0,
    }
    .min(surface.height.saturating_sub(vertical_padding * 2).max(0));
    let text_top = surface
        .y
        .saturating_add((surface.height.saturating_sub(total_text_height)) / 2);
    let title_bounds = has_title.then_some(Rect {
        x: text_left,
        y: text_top,
        width: text_width,
        height: if has_message {
            title_line_height.min(total_text_height)
        } else {
            total_text_height
        },
    });
    let message_bounds = has_message.then_some(Rect {
        x: text_left,
        y: text_top.saturating_add(if has_title {
            title_line_height.min(total_text_height)
        } else {
            0
        }),
        width: text_width,
        height: if has_title {
            total_text_height.saturating_sub(title_line_height.min(total_text_height))
        } else {
            total_text_height
        },
    });

    ZsInfoBarRenderPlan {
        surface,
        accent_bounds,
        icon_bounds,
        title_bounds,
        message_bounds,
        action_bounds,
        close_bounds,
        focused_control: focused_control.filter(|control| spec.has_control(*control)),
        surface_radius: metrics.surface_radius.to_px(dpi).round_i32().max(0),
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(0),
        platform,
    }
}

#[cfg(feature = "info-bar")]
pub fn zs_info_bar_native_draw_plan(
    plan: &ZsInfoBarRenderPlan,
    spec: &crate::ZsInfoBarSpec,
) -> NativeDrawPlan {
    let severity_role = match spec.info_bar_severity() {
        crate::ZsInfoBarSeverity::Informational => ColorRole::Accent,
        crate::ZsInfoBarSeverity::Success => ColorRole::Success,
        crate::ZsInfoBarSeverity::Warning => ColorRole::Warning,
        crate::ZsInfoBarSeverity::Error => ColorRole::Danger,
    };
    let surface_alpha = match plan.platform {
        ZsInfoBarPlatformStyle::Windows => 24,
        ZsInfoBarPlatformStyle::Macos => 18,
        ZsInfoBarPlatformStyle::Gtk => 28,
    };
    let mut commands = vec![
        NativeDrawCommand::RoundRect {
            rect: plan.surface,
            fill: NativeDrawFill::RoleWithAlpha {
                role: severity_role,
                alpha: surface_alpha,
            },
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.surface_radius,
        },
        NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                spec.info_bar_severity().icon(),
                plan.icon_bounds,
                NativeIconColorMode::ThemeAware,
            )
            .with_color(severity_role),
        ),
    ];
    if let Some(accent_bounds) = plan.accent_bounds {
        commands.push(NativeDrawCommand::FillRect {
            rect: accent_bounds,
            fill: NativeDrawFill::Role(severity_role),
        });
    }
    if let (Some(title), Some(bounds)) = (spec.title_text(), plan.title_bounds) {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            title,
            bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: ColorRole::PrimaryText,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    if let Some(bounds) = plan.message_bounds {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.message(),
            bounds,
            SemanticTextStyle {
                role: TextRole::Caption,
                color: ColorRole::SecondaryText,
                weight: TextWeight::Regular,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    if let (Some(label), Some(bounds)) = (spec.action_label(), plan.action_bounds) {
        let focused = plan.focused_control == Some(crate::ZsInfoBarControl::Action);
        commands.push(NativeDrawCommand::RoundRect {
            rect: bounds,
            fill: if plan.platform == ZsInfoBarPlatformStyle::Windows {
                NativeDrawFill::Role(ColorRole::Control)
            } else {
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 0,
                }
            },
            stroke: focused.then_some(NativeDrawFill::Role(ColorRole::Accent)),
            radius: plan.control_radius,
        });
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            bounds,
            SemanticTextStyle {
                role: TextRole::Button,
                color: ColorRole::Accent,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    if let Some(bounds) = plan.close_bounds {
        let focused = plan.focused_control == Some(crate::ZsInfoBarControl::Close);
        commands.push(NativeDrawCommand::RoundRect {
            rect: bounds,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: 0,
            },
            stroke: focused.then_some(NativeDrawFill::Role(ColorRole::Accent)),
            radius: plan.control_radius,
        });
        let inset = (bounds.width.min(bounds.height) / 4).max(1);
        commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
            ZsIcon::Close,
            Rect {
                x: bounds.x.saturating_add(inset),
                y: bounds.y.saturating_add(inset),
                width: bounds.width.saturating_sub(inset * 2),
                height: bounds.height.saturating_sub(inset * 2),
            },
            NativeIconColorMode::ThemeAware,
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "teaching-tip")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTeachingTipPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "teaching-tip")]
impl ZsTeachingTipPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "teaching-tip")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTeachingTipMetrics {
    pub minimum_width: Dp,
    pub maximum_width: Dp,
    pub viewport_margin: Dp,
    pub horizontal_padding: Dp,
    pub vertical_padding: Dp,
    pub content_gap: Dp,
    pub control_gap: Dp,
    pub control_height: Dp,
    pub title_line_height: Dp,
    pub subtitle_height: Dp,
    pub average_character_width: Dp,
    pub surface_radius: Dp,
    pub control_radius: Dp,
    pub tail_size: Dp,
    pub target_gap: Dp,
}

#[cfg(feature = "teaching-tip")]
impl ZsTeachingTipMetrics {
    pub const fn for_platform(platform: ZsTeachingTipPlatformStyle) -> Self {
        match platform {
            ZsTeachingTipPlatformStyle::Windows => Self {
                minimum_width: Dp::new(240.0),
                maximum_width: Dp::new(320.0),
                viewport_margin: Dp::new(12.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(14.0),
                content_gap: Dp::new(8.0),
                control_gap: Dp::new(12.0),
                control_height: Dp::new(32.0),
                title_line_height: Dp::new(22.0),
                subtitle_height: Dp::new(40.0),
                average_character_width: Dp::new(7.2),
                surface_radius: Dp::new(8.0),
                control_radius: Dp::new(4.0),
                tail_size: Dp::new(12.0),
                target_gap: Dp::new(4.0),
            },
            ZsTeachingTipPlatformStyle::Macos => Self {
                minimum_width: Dp::new(220.0),
                maximum_width: Dp::new(300.0),
                viewport_margin: Dp::new(12.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(12.0),
                content_gap: Dp::new(6.0),
                control_gap: Dp::new(10.0),
                control_height: Dp::new(28.0),
                title_line_height: Dp::new(20.0),
                subtitle_height: Dp::new(36.0),
                average_character_width: Dp::new(6.8),
                surface_radius: Dp::new(10.0),
                control_radius: Dp::new(6.0),
                tail_size: Dp::new(10.0),
                target_gap: Dp::new(4.0),
            },
            ZsTeachingTipPlatformStyle::Gtk => Self {
                minimum_width: Dp::new(240.0),
                maximum_width: Dp::new(320.0),
                viewport_margin: Dp::new(12.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(12.0),
                content_gap: Dp::new(6.0),
                control_gap: Dp::new(10.0),
                control_height: Dp::new(34.0),
                title_line_height: Dp::new(20.0),
                subtitle_height: Dp::new(38.0),
                average_character_width: Dp::new(7.2),
                surface_radius: Dp::new(12.0),
                control_radius: Dp::new(8.0),
                tail_size: Dp::new(10.0),
                target_gap: Dp::new(4.0),
            },
        }
    }
}

#[cfg(feature = "teaching-tip")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTeachingTipRenderPlan {
    pub viewport: Rect,
    pub target: Rect,
    pub surface: Rect,
    pub tail: [crate::Point; 3],
    pub title_bounds: Rect,
    pub subtitle_bounds: Option<Rect>,
    pub action_bounds: Option<Rect>,
    pub close_bounds: Rect,
    pub focused_control: crate::ZsTeachingTipControl,
    pub placement: crate::ZsTeachingTipPlacement,
    pub surface_radius: i32,
    pub control_radius: i32,
    pub platform: ZsTeachingTipPlatformStyle,
}

#[cfg(feature = "teaching-tip")]
pub fn zs_teaching_tip_render_plan(
    viewport: Rect,
    target: Rect,
    spec: &crate::ZsTeachingTipSpec,
    focused_control: crate::ZsTeachingTipControl,
    platform: ZsTeachingTipPlatformStyle,
    dpi: Dpi,
) -> ZsTeachingTipRenderPlan {
    let metrics = ZsTeachingTipMetrics::for_platform(platform);
    let margin = metrics.viewport_margin.to_px(dpi).round_i32().max(0);
    let horizontal_padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let vertical_padding = metrics.vertical_padding.to_px(dpi).round_i32().max(0);
    let content_gap = metrics.content_gap.to_px(dpi).round_i32().max(0);
    let control_gap = metrics.control_gap.to_px(dpi).round_i32().max(0);
    let control_height = metrics.control_height.to_px(dpi).round_i32().max(1);
    let title_height = metrics.title_line_height.to_px(dpi).round_i32().max(1);
    let subtitle_height = metrics.subtitle_height.to_px(dpi).round_i32().max(1);
    let character_width = metrics
        .average_character_width
        .to_px(dpi)
        .round_i32()
        .max(1);
    let tail_size = metrics.tail_size.to_px(dpi).round_i32().max(1);
    let target_gap = metrics.target_gap.to_px(dpi).round_i32().max(0);
    let minimum_width = metrics.minimum_width.to_px(dpi).round_i32().max(1);
    let maximum_width = metrics
        .maximum_width
        .to_px(dpi)
        .round_i32()
        .max(minimum_width);
    let viewport_inner_width = viewport
        .width
        .saturating_sub(margin.saturating_mul(2))
        .max(1);
    let longest_text = spec
        .title()
        .chars()
        .count()
        .max(spec.subtitle().chars().count()) as i32;
    let desired_width = longest_text
        .saturating_mul(character_width)
        .saturating_add(horizontal_padding.saturating_mul(2))
        .saturating_add(control_height);
    let surface_width = desired_width
        .max(minimum_width)
        .min(maximum_width)
        .min(viewport_inner_width)
        .max(1);
    let has_subtitle = !spec.subtitle().trim().is_empty();
    let has_action = spec.action_label().is_some();
    let surface_height = vertical_padding
        .saturating_mul(2)
        .saturating_add(title_height)
        .saturating_add(if has_subtitle {
            content_gap.saturating_add(subtitle_height)
        } else {
            0
        })
        .saturating_add(if has_action {
            control_gap.saturating_add(control_height)
        } else {
            0
        })
        .max(control_height.saturating_add(vertical_padding.saturating_mul(2)));
    let placement = teaching_tip_resolve_placement(
        viewport,
        target,
        surface_width,
        surface_height,
        tail_size,
        target_gap,
        spec.placement(),
    );
    let viewport_right = viewport.x.saturating_add(viewport.width);
    let viewport_bottom = viewport.y.saturating_add(viewport.height);
    let minimum_x = viewport.x.saturating_add(margin);
    let maximum_x = viewport_right
        .saturating_sub(margin)
        .saturating_sub(surface_width)
        .max(minimum_x);
    let minimum_y = viewport.y.saturating_add(margin);
    let maximum_y = viewport_bottom
        .saturating_sub(margin)
        .saturating_sub(surface_height)
        .max(minimum_y);
    let target_center_x = target.x.saturating_add(target.width / 2);
    let target_center_y = target.y.saturating_add(target.height / 2);
    let (desired_x, desired_y) = match placement {
        crate::ZsTeachingTipPlacement::Top => (
            target_center_x.saturating_sub(surface_width / 2),
            target
                .y
                .saturating_sub(target_gap)
                .saturating_sub(tail_size)
                .saturating_sub(surface_height),
        ),
        crate::ZsTeachingTipPlacement::Bottom => (
            target_center_x.saturating_sub(surface_width / 2),
            target
                .y
                .saturating_add(target.height)
                .saturating_add(target_gap)
                .saturating_add(tail_size),
        ),
        crate::ZsTeachingTipPlacement::Left => (
            target
                .x
                .saturating_sub(target_gap)
                .saturating_sub(tail_size)
                .saturating_sub(surface_width),
            target_center_y.saturating_sub(surface_height / 2),
        ),
        crate::ZsTeachingTipPlacement::Right => (
            target
                .x
                .saturating_add(target.width)
                .saturating_add(target_gap)
                .saturating_add(tail_size),
            target_center_y.saturating_sub(surface_height / 2),
        ),
        crate::ZsTeachingTipPlacement::Auto => unreachable!("placement is resolved"),
    };
    let surface = Rect {
        x: desired_x.clamp(minimum_x, maximum_x),
        y: desired_y.clamp(minimum_y, maximum_y),
        width: surface_width,
        height: surface_height,
    };
    let tail = teaching_tip_tail(surface, target, placement, tail_size, target_gap);
    let close_bounds = Rect {
        x: surface
            .x
            .saturating_add(surface.width)
            .saturating_sub(horizontal_padding)
            .saturating_sub(control_height),
        y: surface.y.saturating_add(vertical_padding),
        width: control_height,
        height: control_height,
    };
    let title_bounds = Rect {
        x: surface.x.saturating_add(horizontal_padding),
        y: surface.y.saturating_add(vertical_padding),
        width: close_bounds
            .x
            .saturating_sub(content_gap)
            .saturating_sub(surface.x.saturating_add(horizontal_padding))
            .max(1),
        height: title_height,
    };
    let subtitle_bounds = has_subtitle.then_some(Rect {
        x: surface.x.saturating_add(horizontal_padding),
        y: title_bounds
            .y
            .saturating_add(title_bounds.height)
            .saturating_add(content_gap),
        width: surface
            .width
            .saturating_sub(horizontal_padding.saturating_mul(2))
            .max(1),
        height: subtitle_height,
    });
    let action_bounds = spec.action_label().map(|label| {
        let desired = (label.chars().count() as i32)
            .saturating_mul(character_width)
            .saturating_add(horizontal_padding)
            .max(control_height);
        let width = desired.min(
            surface
                .width
                .saturating_sub(horizontal_padding.saturating_mul(2))
                .max(1),
        );
        Rect {
            x: surface
                .x
                .saturating_add(surface.width)
                .saturating_sub(horizontal_padding)
                .saturating_sub(width),
            y: surface
                .y
                .saturating_add(surface.height)
                .saturating_sub(vertical_padding)
                .saturating_sub(control_height),
            width,
            height: control_height,
        }
    });

    ZsTeachingTipRenderPlan {
        viewport,
        target,
        surface,
        tail,
        title_bounds,
        subtitle_bounds,
        action_bounds,
        close_bounds,
        focused_control: if spec.has_control(focused_control) {
            focused_control
        } else {
            spec.initial_control()
        },
        placement,
        surface_radius: metrics.surface_radius.to_px(dpi).round_i32().max(0),
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(0),
        platform,
    }
}

#[cfg(feature = "teaching-tip")]
fn teaching_tip_resolve_placement(
    viewport: Rect,
    target: Rect,
    surface_width: i32,
    surface_height: i32,
    tail_size: i32,
    target_gap: i32,
    preferred: crate::ZsTeachingTipPlacement,
) -> crate::ZsTeachingTipPlacement {
    use crate::ZsTeachingTipPlacement::{Auto, Bottom, Left, Right, Top};
    let required_vertical = surface_height
        .saturating_add(tail_size)
        .saturating_add(target_gap);
    let required_horizontal = surface_width
        .saturating_add(tail_size)
        .saturating_add(target_gap);
    let viewport_right = viewport.x.saturating_add(viewport.width);
    let viewport_bottom = viewport.y.saturating_add(viewport.height);
    let available = |placement| match placement {
        Top => target.y.saturating_sub(viewport.y),
        Bottom => viewport_bottom.saturating_sub(target.y.saturating_add(target.height)),
        Left => target.x.saturating_sub(viewport.x),
        Right => viewport_right.saturating_sub(target.x.saturating_add(target.width)),
        Auto => 0,
    };
    let fits = |placement| match placement {
        Top | Bottom => available(placement) >= required_vertical,
        Left | Right => available(placement) >= required_horizontal,
        Auto => false,
    };
    let order = match preferred {
        Auto => [Top, Bottom, Right, Left],
        Top => [Top, Bottom, Right, Left],
        Bottom => [Bottom, Top, Right, Left],
        Left => [Left, Right, Top, Bottom],
        Right => [Right, Left, Top, Bottom],
    };
    order
        .into_iter()
        .find(|placement| fits(*placement))
        .unwrap_or_else(|| {
            order
                .into_iter()
                .max_by_key(|placement| available(*placement))
                .unwrap_or(Top)
        })
}

#[cfg(feature = "teaching-tip")]
fn teaching_tip_tail(
    surface: Rect,
    target: Rect,
    placement: crate::ZsTeachingTipPlacement,
    size: i32,
    gap: i32,
) -> [crate::Point; 3] {
    let half = (size / 2).max(1);
    let surface_right = surface.x.saturating_add(surface.width);
    let surface_bottom = surface.y.saturating_add(surface.height);
    let target_center_x = target.x.saturating_add(target.width / 2);
    let target_center_y = target.y.saturating_add(target.height / 2);
    match placement {
        crate::ZsTeachingTipPlacement::Top => {
            let center = target_center_x.clamp(
                surface.x.saturating_add(size),
                surface_right.saturating_sub(size),
            );
            [
                crate::Point {
                    x: center.saturating_sub(half),
                    y: surface_bottom,
                },
                crate::Point {
                    x: center.saturating_add(half),
                    y: surface_bottom,
                },
                crate::Point {
                    x: target_center_x,
                    y: target.y.saturating_sub(gap),
                },
            ]
        }
        crate::ZsTeachingTipPlacement::Bottom => {
            let center = target_center_x.clamp(
                surface.x.saturating_add(size),
                surface_right.saturating_sub(size),
            );
            [
                crate::Point {
                    x: center.saturating_sub(half),
                    y: surface.y,
                },
                crate::Point {
                    x: center.saturating_add(half),
                    y: surface.y,
                },
                crate::Point {
                    x: target_center_x,
                    y: target.y.saturating_add(target.height).saturating_add(gap),
                },
            ]
        }
        crate::ZsTeachingTipPlacement::Left => {
            let center = target_center_y.clamp(
                surface.y.saturating_add(size),
                surface_bottom.saturating_sub(size),
            );
            [
                crate::Point {
                    x: surface_right,
                    y: center.saturating_sub(half),
                },
                crate::Point {
                    x: surface_right,
                    y: center.saturating_add(half),
                },
                crate::Point {
                    x: target.x.saturating_sub(gap),
                    y: target_center_y,
                },
            ]
        }
        crate::ZsTeachingTipPlacement::Right => {
            let center = target_center_y.clamp(
                surface.y.saturating_add(size),
                surface_bottom.saturating_sub(size),
            );
            [
                crate::Point {
                    x: surface.x,
                    y: center.saturating_sub(half),
                },
                crate::Point {
                    x: surface.x,
                    y: center.saturating_add(half),
                },
                crate::Point {
                    x: target.x.saturating_add(target.width).saturating_add(gap),
                    y: target_center_y,
                },
            ]
        }
        crate::ZsTeachingTipPlacement::Auto => unreachable!("placement is resolved"),
    }
}

#[cfg(feature = "teaching-tip")]
pub fn zs_teaching_tip_native_draw_plan(
    plan: &ZsTeachingTipRenderPlan,
    spec: &crate::ZsTeachingTipSpec,
) -> NativeDrawPlan {
    let shadow_alpha = match plan.platform {
        ZsTeachingTipPlatformStyle::Windows => 32,
        ZsTeachingTipPlatformStyle::Macos => 26,
        ZsTeachingTipPlatformStyle::Gtk => 34,
    };
    let shadow_surface = Rect {
        x: plan.surface.x.saturating_sub(4),
        y: plan.surface.y.saturating_add(3),
        width: plan.surface.width.saturating_add(8),
        height: plan.surface.height.saturating_add(5),
    };
    let shadow_tail = plan.tail.map(|point| crate::Point {
        x: point.x,
        y: point.y.saturating_add(3),
    });
    let mut commands = vec![
        NativeDrawCommand::RoundFill {
            rect: shadow_surface,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: shadow_alpha,
            },
            radius: plan.surface_radius.saturating_add(4),
        },
        NativeDrawCommand::FillTriangle {
            points: shadow_tail,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: shadow_alpha,
            },
        },
        NativeDrawCommand::FillTriangle {
            points: plan.tail,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        },
        NativeDrawCommand::RoundRect {
            rect: plan.surface,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.surface_radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.title(),
            plan.title_bounds,
            SemanticTextStyle {
                role: TextRole::Subtitle,
                color: ColorRole::PrimaryText,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )),
    ];
    if let Some(bounds) = plan.subtitle_bounds {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.subtitle(),
            bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: ColorRole::SecondaryText,
                weight: TextWeight::Regular,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Start,
                wrap: crate::TextWrap::Word,
                ellipsis: true,
            },
        )));
    }
    if let (Some(label), Some(bounds)) = (spec.action_label(), plan.action_bounds) {
        let focused = plan.focused_control == crate::ZsTeachingTipControl::Action;
        let native_filled = plan.platform != ZsTeachingTipPlatformStyle::Windows;
        commands.push(NativeDrawCommand::RoundRect {
            rect: bounds,
            fill: if native_filled {
                NativeDrawFill::Role(ColorRole::Accent)
            } else {
                NativeDrawFill::Role(ColorRole::Control)
            },
            stroke: focused.then_some(NativeDrawFill::Role(ColorRole::Accent)),
            radius: plan.control_radius,
        });
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            bounds,
            SemanticTextStyle {
                role: TextRole::Button,
                color: if native_filled {
                    ColorRole::AccentText
                } else {
                    ColorRole::Accent
                },
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    commands.push(NativeDrawCommand::RoundRect {
        rect: plan.close_bounds,
        fill: NativeDrawFill::RoleWithAlpha {
            role: ColorRole::PrimaryText,
            alpha: 0,
        },
        stroke: (plan.focused_control == crate::ZsTeachingTipControl::Close)
            .then_some(NativeDrawFill::Role(ColorRole::Accent)),
        radius: plan.control_radius,
    });
    let inset = (plan.close_bounds.width.min(plan.close_bounds.height) / 4).max(1);
    commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
        ZsIcon::Close,
        Rect {
            x: plan.close_bounds.x.saturating_add(inset),
            y: plan.close_bounds.y.saturating_add(inset),
            width: plan
                .close_bounds
                .width
                .saturating_sub(inset.saturating_mul(2)),
            height: plan
                .close_bounds
                .height
                .saturating_sub(inset.saturating_mul(2)),
        },
        NativeIconColorMode::ThemeAware,
    )));
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "toast")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsToastPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "toast")]
impl ZsToastPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "toast")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsToastMetrics {
    pub maximum_width: Dp,
    pub viewport_margin: Dp,
    pub bottom_margin: Dp,
    pub horizontal_padding: Dp,
    pub vertical_padding: Dp,
    pub control_gap: Dp,
    pub control_height: Dp,
    pub surface_radius: Dp,
    pub control_radius: Dp,
    pub line_height: Dp,
    pub average_character_width: Dp,
}

#[cfg(feature = "toast")]
impl ZsToastMetrics {
    pub const fn for_platform(platform: ZsToastPlatformStyle) -> Self {
        match platform {
            ZsToastPlatformStyle::Windows => Self {
                maximum_width: Dp::new(420.0),
                viewport_margin: Dp::new(24.0),
                bottom_margin: Dp::new(24.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(12.0),
                control_gap: Dp::new(8.0),
                control_height: Dp::new(32.0),
                surface_radius: Dp::new(8.0),
                control_radius: Dp::new(4.0),
                line_height: Dp::new(20.0),
                average_character_width: Dp::new(7.2),
            },
            ZsToastPlatformStyle::Macos => Self {
                maximum_width: Dp::new(380.0),
                viewport_margin: Dp::new(20.0),
                bottom_margin: Dp::new(20.0),
                horizontal_padding: Dp::new(14.0),
                vertical_padding: Dp::new(10.0),
                control_gap: Dp::new(6.0),
                control_height: Dp::new(28.0),
                surface_radius: Dp::new(10.0),
                control_radius: Dp::new(6.0),
                line_height: Dp::new(18.0),
                average_character_width: Dp::new(6.8),
            },
            ZsToastPlatformStyle::Gtk => Self {
                maximum_width: Dp::new(440.0),
                viewport_margin: Dp::new(24.0),
                bottom_margin: Dp::new(24.0),
                horizontal_padding: Dp::new(16.0),
                vertical_padding: Dp::new(10.0),
                control_gap: Dp::new(8.0),
                control_height: Dp::new(34.0),
                surface_radius: Dp::new(12.0),
                control_radius: Dp::new(17.0),
                line_height: Dp::new(20.0),
                average_character_width: Dp::new(7.2),
            },
        }
    }
}

#[cfg(feature = "toast")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsToastRenderPlan {
    pub surface: Rect,
    pub message_bounds: Rect,
    pub action_bounds: Option<Rect>,
    pub close_bounds: Rect,
    pub focused_control: crate::ZsToastControl,
    pub surface_radius: i32,
    pub control_radius: i32,
    pub platform: ZsToastPlatformStyle,
}

#[cfg(feature = "toast")]
pub fn zs_toast_render_plan(
    viewport: Rect,
    spec: &crate::ZsToastSpec,
    focused_control: crate::ZsToastControl,
    platform: ZsToastPlatformStyle,
    dpi: Dpi,
) -> ZsToastRenderPlan {
    let metrics = ZsToastMetrics::for_platform(platform);
    let viewport_margin = metrics.viewport_margin.to_px(dpi).round_i32().max(0);
    let bottom_margin = metrics.bottom_margin.to_px(dpi).round_i32().max(0);
    let horizontal_padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let vertical_padding = metrics.vertical_padding.to_px(dpi).round_i32().max(0);
    let control_gap = metrics.control_gap.to_px(dpi).round_i32().max(0);
    let control_height = metrics.control_height.to_px(dpi).round_i32().max(1);
    let line_height = metrics.line_height.to_px(dpi).round_i32().max(1);
    let character_width = metrics
        .average_character_width
        .to_px(dpi)
        .round_i32()
        .max(1);
    let maximum_width = metrics.maximum_width.to_px(dpi).round_i32().max(1);
    let available_width = viewport
        .width
        .saturating_sub(viewport_margin.saturating_mul(2))
        .max(1);
    let close_width = control_height;
    let action_width = spec.action_label().map(|label| {
        (label.chars().count() as i32)
            .saturating_mul(character_width)
            .saturating_add(horizontal_padding)
            .max(control_height)
    });
    let controls_width = close_width
        .saturating_add(control_gap)
        .saturating_add(action_width.map(|width| width + control_gap).unwrap_or(0));
    let desired_message_width = (spec.message().chars().count() as i32)
        .saturating_mul(character_width)
        .clamp(scale(96, dpi), scale(280, dpi));
    let desired_width = horizontal_padding
        .saturating_mul(2)
        .saturating_add(desired_message_width)
        .saturating_add(controls_width);
    let surface_width = desired_width.min(maximum_width).min(available_width).max(1);
    let surface_height = control_height
        .max(line_height)
        .saturating_add(vertical_padding.saturating_mul(2));
    let surface = Rect {
        x: viewport.x + (viewport.width - surface_width) / 2,
        y: viewport
            .y
            .saturating_add(viewport.height)
            .saturating_sub(bottom_margin)
            .saturating_sub(surface_height),
        width: surface_width,
        height: surface_height,
    };
    let controls_y = surface.y + (surface.height - control_height) / 2;
    let close_bounds = Rect {
        x: surface
            .x
            .saturating_add(surface.width)
            .saturating_sub(horizontal_padding)
            .saturating_sub(close_width),
        y: controls_y,
        width: close_width,
        height: control_height,
    };
    let action_bounds = action_width.map(|width| Rect {
        x: close_bounds
            .x
            .saturating_sub(control_gap)
            .saturating_sub(width),
        y: controls_y,
        width,
        height: control_height,
    });
    let message_right = action_bounds
        .map(|bounds| bounds.x)
        .unwrap_or(close_bounds.x)
        .saturating_sub(control_gap);
    let message_x = surface.x.saturating_add(horizontal_padding);
    let message_bounds = Rect {
        x: message_x,
        y: surface.y.saturating_add(vertical_padding),
        width: message_right.saturating_sub(message_x).max(1),
        height: surface
            .height
            .saturating_sub(vertical_padding.saturating_mul(2)),
    };
    ZsToastRenderPlan {
        surface,
        message_bounds,
        action_bounds,
        close_bounds,
        focused_control,
        surface_radius: metrics.surface_radius.to_px(dpi).round_i32().max(0),
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(0),
        platform,
    }
}

#[cfg(feature = "toast")]
pub fn zs_toast_native_draw_plan(
    plan: &ZsToastRenderPlan,
    spec: &crate::ZsToastSpec,
) -> NativeDrawPlan {
    let shadow_alpha = match plan.platform {
        ZsToastPlatformStyle::Windows => 30,
        ZsToastPlatformStyle::Macos => 24,
        ZsToastPlatformStyle::Gtk => 34,
    };
    let shadow = Rect {
        x: plan.surface.x.saturating_sub(3),
        y: plan.surface.y.saturating_add(2),
        width: plan.surface.width.saturating_add(6),
        height: plan.surface.height.saturating_add(4),
    };
    let mut commands = vec![
        NativeDrawCommand::RoundFill {
            rect: shadow,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: shadow_alpha,
            },
            radius: plan.surface_radius.saturating_add(3),
        },
        NativeDrawCommand::RoundRect {
            rect: plan.surface,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.surface_radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.message(),
            plan.message_bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: ColorRole::PrimaryText,
                weight: if plan.platform == ZsToastPlatformStyle::Gtk {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )),
    ];
    if let (Some(label), Some(bounds)) = (spec.action_label(), plan.action_bounds) {
        let focused = plan.focused_control == crate::ZsToastControl::Action;
        commands.push(NativeDrawCommand::RoundRect {
            rect: bounds,
            fill: if plan.platform == ZsToastPlatformStyle::Windows {
                NativeDrawFill::Role(ColorRole::Control)
            } else {
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 0,
                }
            },
            stroke: focused.then_some(NativeDrawFill::Role(ColorRole::Accent)),
            radius: plan.control_radius,
        });
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            bounds,
            SemanticTextStyle {
                role: TextRole::Button,
                color: ColorRole::Accent,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    commands.push(NativeDrawCommand::RoundRect {
        rect: plan.close_bounds,
        fill: NativeDrawFill::RoleWithAlpha {
            role: ColorRole::PrimaryText,
            alpha: 0,
        },
        stroke: (plan.focused_control == crate::ZsToastControl::Close)
            .then_some(NativeDrawFill::Role(ColorRole::Accent)),
        radius: plan.control_radius,
    });
    let icon_inset = (plan.close_bounds.width.min(plan.close_bounds.height) / 4).max(1);
    let icon_bounds = Rect {
        x: plan.close_bounds.x.saturating_add(icon_inset),
        y: plan.close_bounds.y.saturating_add(icon_inset),
        width: plan.close_bounds.width.saturating_sub(icon_inset * 2),
        height: plan.close_bounds.height.saturating_sub(icon_inset * 2),
    };
    commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
        ZsIcon::Close,
        icon_bounds,
        NativeIconColorMode::ThemeAware,
    )));
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsBreadcrumbPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "breadcrumb")]
impl ZsBreadcrumbPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsBreadcrumbMetrics {
    pub control_height: Dp,
    pub horizontal_padding: Dp,
    pub minimum_item_width: Dp,
    pub separator_width: Dp,
    pub icon_size: Dp,
    pub radius: Dp,
    pub character_width: Dp,
    pub popup_row_height: Dp,
    pub popup_padding: Dp,
}

#[cfg(feature = "breadcrumb")]
impl ZsBreadcrumbMetrics {
    pub const fn for_platform(platform: ZsBreadcrumbPlatformStyle) -> Self {
        match platform {
            ZsBreadcrumbPlatformStyle::Windows => Self {
                control_height: Dp::new(32.0),
                horizontal_padding: Dp::new(8.0),
                minimum_item_width: Dp::new(32.0),
                separator_width: Dp::new(20.0),
                icon_size: Dp::new(16.0),
                radius: Dp::new(4.0),
                character_width: Dp::new(7.2),
                popup_row_height: Dp::new(32.0),
                popup_padding: Dp::new(4.0),
            },
            ZsBreadcrumbPlatformStyle::Macos => Self {
                control_height: Dp::new(24.0),
                horizontal_padding: Dp::new(7.0),
                minimum_item_width: Dp::new(28.0),
                separator_width: Dp::new(16.0),
                icon_size: Dp::new(13.0),
                radius: Dp::new(5.0),
                character_width: Dp::new(6.6),
                popup_row_height: Dp::new(24.0),
                popup_padding: Dp::new(4.0),
            },
            ZsBreadcrumbPlatformStyle::Gtk => Self {
                control_height: Dp::new(34.0),
                horizontal_padding: Dp::new(10.0),
                minimum_item_width: Dp::new(36.0),
                separator_width: Dp::new(20.0),
                icon_size: Dp::new(16.0),
                radius: Dp::new(6.0),
                character_width: Dp::new(7.0),
                popup_row_height: Dp::new(34.0),
                popup_padding: Dp::new(4.0),
            },
        }
    }
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsBreadcrumbItemRenderPlan {
    pub item_index: usize,
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub separator_bounds: Option<Rect>,
    pub current: bool,
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsBreadcrumbOverflowRowRenderPlan {
    pub item_index: usize,
    pub bounds: Rect,
    pub text_bounds: Rect,
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsBreadcrumbRenderPlan {
    pub bounds: Rect,
    pub items: Vec<ZsBreadcrumbItemRenderPlan>,
    pub hidden_indices: Vec<usize>,
    pub overflow_bounds: Option<Rect>,
    pub overflow_separator_bounds: Option<Rect>,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub popup_rows: Vec<ZsBreadcrumbOverflowRowRenderPlan>,
    pub icon_size: i32,
    pub radius: i32,
    pub platform: ZsBreadcrumbPlatformStyle,
}

#[cfg(feature = "breadcrumb")]
pub fn zs_breadcrumb_render_plan(
    bounds: Rect,
    items: &[crate::ZsBreadcrumbItem],
    overflow_open: bool,
    platform: ZsBreadcrumbPlatformStyle,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsBreadcrumbRenderPlan {
    let metrics = ZsBreadcrumbMetrics::for_platform(platform);
    let padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(1);
    let minimum_width = metrics.minimum_item_width.to_px(dpi).round_i32().max(1);
    let separator_width = metrics.separator_width.to_px(dpi).round_i32().max(1);
    let character_width = metrics.character_width.to_px(dpi).round_i32().max(1);
    let icon_size = metrics.icon_size.to_px(dpi).round_i32().max(1);
    let control_height = metrics
        .control_height
        .to_px(dpi)
        .round_i32()
        .min(bounds.height.max(1))
        .max(1);
    let y = bounds
        .y
        .saturating_add((bounds.height.saturating_sub(control_height)) / 2);
    let desired_widths = items
        .iter()
        .map(|item| {
            (item.label().chars().count() as i32)
                .saturating_mul(character_width)
                .saturating_add(padding.saturating_mul(2))
                .max(minimum_width)
        })
        .collect::<Vec<_>>();
    let full_width = desired_widths
        .iter()
        .copied()
        .fold(0_i32, i32::saturating_add)
        .saturating_add(separator_width.saturating_mul(items.len().saturating_sub(1) as i32));

    let mut visible_indices = (0..items.len()).collect::<Vec<_>>();
    let mut hidden_indices = Vec::new();
    let overflow_width = control_height;
    if full_width > bounds.width.max(1) && items.len() > 1 {
        visible_indices.clear();
        let keep_root = platform == ZsBreadcrumbPlatformStyle::Macos && items.len() > 1;
        let reserved_root = if keep_root {
            desired_widths[0].saturating_add(separator_width)
        } else {
            0
        };
        let mut used = overflow_width
            .saturating_add(separator_width)
            .saturating_add(reserved_root);
        for index in (if keep_root { 1 } else { 0 }..items.len()).rev() {
            let width = desired_widths[index].saturating_add(if index + 1 < items.len() {
                separator_width
            } else {
                0
            });
            if index == items.len() - 1 || used.saturating_add(width) <= bounds.width.max(1) {
                visible_indices.push(index);
                used = used.saturating_add(width);
            } else {
                hidden_indices.push(index);
            }
        }
        visible_indices.reverse();
        hidden_indices.reverse();
        if keep_root {
            visible_indices.insert(0, 0);
        }
        hidden_indices = (0..items.len())
            .filter(|index| !visible_indices.contains(index))
            .collect();
    }

    let has_overflow = !hidden_indices.is_empty();
    let mut ordered = Vec::<Option<usize>>::new();
    if has_overflow {
        if platform == ZsBreadcrumbPlatformStyle::Macos && visible_indices.first() == Some(&0) {
            ordered.push(Some(0));
            ordered.push(None);
            ordered.extend(visible_indices.iter().copied().skip(1).map(Some));
        } else {
            ordered.push(None);
            ordered.extend(visible_indices.iter().copied().map(Some));
        }
    } else {
        ordered.extend(visible_indices.iter().copied().map(Some));
    }

    let mut x = bounds.x;
    let mut item_plans = Vec::new();
    let mut overflow_bounds = None;
    let mut overflow_separator_bounds = None;
    let right = bounds.x.saturating_add(bounds.width.max(1));
    for (position, entry) in ordered.iter().copied().enumerate() {
        let is_last = position + 1 == ordered.len();
        let width = entry.map_or(overflow_width, |index| desired_widths[index]);
        let remaining = right.saturating_sub(x).max(1);
        let future_count = ordered.len().saturating_sub(position + 1) as i32;
        let future_reserve = future_count.saturating_mul(2);
        let segment_limit = remaining.saturating_sub(future_reserve).max(1);
        let segment_width = width.min(segment_limit).max(1);
        let segment = Rect {
            x,
            y,
            width: segment_width,
            height: control_height,
        };
        let separator_remaining = right.saturating_sub(x.saturating_add(segment_width));
        let minimum_after_separator = future_count.saturating_mul(2).saturating_sub(1);
        let separator = (!is_last && separator_remaining > 0).then_some(Rect {
            x: x.saturating_add(segment_width),
            y,
            width: separator_width.min(
                separator_remaining
                    .saturating_sub(minimum_after_separator)
                    .max(1),
            ),
            height: control_height,
        });
        let text_inset = padding.min(segment.width.saturating_sub(1) / 2);
        if let Some(index) = entry {
            item_plans.push(ZsBreadcrumbItemRenderPlan {
                item_index: index,
                bounds: segment,
                text_bounds: Rect {
                    x: segment.x.saturating_add(text_inset),
                    y: segment.y,
                    width: segment
                        .width
                        .saturating_sub(text_inset.saturating_mul(2))
                        .max(1),
                    height: segment.height,
                },
                separator_bounds: separator,
                current: index + 1 == items.len(),
            });
        } else {
            overflow_bounds = Some(segment);
            overflow_separator_bounds = separator;
        }
        x = separator
            .map(|separator| separator.x.saturating_add(separator.width))
            .unwrap_or_else(|| x.saturating_add(segment_width));
    }

    let popup_padding = metrics.popup_padding.to_px(dpi).round_i32().max(0);
    let popup_row_height = metrics.popup_row_height.to_px(dpi).round_i32().max(1);
    let popup_width = hidden_indices
        .iter()
        .map(|index| desired_widths[*index])
        .max()
        .unwrap_or(minimum_width)
        .max(minimum_width.saturating_mul(2));
    let popup_height = popup_row_height
        .saturating_mul(hidden_indices.len() as i32)
        .saturating_add(popup_padding.saturating_mul(2));
    let placed_popup = (overflow_open && has_overflow).then(|| {
        place_popup(
            overflow_bounds.unwrap_or(bounds),
            popup_width,
            popup_height,
            Dp::new(4.0).to_px(dpi).round_i32().max(1),
            viewport,
        )
    });
    let popup_rows = placed_popup
        .map(|placed| {
            hidden_indices
                .iter()
                .copied()
                .enumerate()
                .map(|(row, item_index)| {
                    let bounds = Rect {
                        x: placed.bounds.x.saturating_add(popup_padding),
                        y: placed
                            .bounds
                            .y
                            .saturating_add(popup_padding)
                            .saturating_add((row as i32).saturating_mul(popup_row_height)),
                        width: placed
                            .bounds
                            .width
                            .saturating_sub(popup_padding.saturating_mul(2))
                            .max(1),
                        height: popup_row_height,
                    };
                    ZsBreadcrumbOverflowRowRenderPlan {
                        item_index,
                        bounds,
                        text_bounds: Rect {
                            x: bounds.x.saturating_add(padding),
                            y: bounds.y,
                            width: bounds
                                .width
                                .saturating_sub(padding.saturating_mul(2))
                                .max(1),
                            height: bounds.height,
                        },
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    ZsBreadcrumbRenderPlan {
        bounds,
        items: item_plans,
        hidden_indices,
        overflow_bounds,
        overflow_separator_bounds,
        popup: placed_popup.map(|placed| placed.bounds),
        popup_placement: placed_popup.map(|placed| placed.placement),
        popup_rows,
        icon_size,
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "breadcrumb")]
pub fn zs_breadcrumb_native_draw_plan(
    plan: &ZsBreadcrumbRenderPlan,
    items: &[crate::ZsBreadcrumbItem],
    focused: Option<crate::ZsBreadcrumbFocusTarget>,
) -> NativeDrawPlan {
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: plan.bounds,
        fill: NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Control,
            alpha: 0,
        },
        stroke: None,
        radius: plan.radius,
    }];
    let draw_separator = |commands: &mut Vec<NativeDrawCommand>, bounds: Rect| {
        let size = bounds.width.min(bounds.height).min(plan.icon_size).max(1);
        commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
            ZsIcon::ChevronRight,
            Rect {
                x: bounds
                    .x
                    .saturating_add((bounds.width.saturating_sub(size)) / 2),
                y: bounds
                    .y
                    .saturating_add((bounds.height.saturating_sub(size)) / 2),
                width: size,
                height: size,
            },
            NativeIconColorMode::ThemeAware,
        )));
    };
    for item in &plan.items {
        let Some(spec) = items.get(item.item_index) else {
            continue;
        };
        if focused == Some(crate::ZsBreadcrumbFocusTarget::Item(spec.id())) {
            commands.push(NativeDrawCommand::RoundRect {
                rect: item.bounds,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 12,
                },
                stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                radius: plan.radius,
            });
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            spec.label(),
            item.text_bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: if item.current {
                    ColorRole::PrimaryText
                } else {
                    ColorRole::SecondaryText
                },
                weight: if item.current {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
        if let Some(separator) = item.separator_bounds {
            draw_separator(&mut commands, separator);
        }
    }
    if let Some(bounds) = plan.overflow_bounds {
        if focused == Some(crate::ZsBreadcrumbFocusTarget::Overflow) {
            commands.push(NativeDrawCommand::RoundRect {
                rect: bounds,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 12,
                },
                stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                radius: plan.radius,
            });
        }
        let size = bounds.width.min(bounds.height).min(plan.icon_size).max(1);
        commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
            ZsIcon::More,
            Rect {
                x: bounds
                    .x
                    .saturating_add((bounds.width.saturating_sub(size)) / 2),
                y: bounds
                    .y
                    .saturating_add((bounds.height.saturating_sub(size)) / 2),
                width: size,
                height: size,
            },
            NativeIconColorMode::ThemeAware,
        )));
        if let Some(separator) = plan.overflow_separator_bounds {
            draw_separator(&mut commands, separator);
        }
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "breadcrumb")]
pub fn zs_breadcrumb_popup_native_draw_plan(
    plan: &ZsBreadcrumbRenderPlan,
    items: &[crate::ZsBreadcrumbItem],
    focused: Option<crate::ZsBreadcrumbFocusTarget>,
) -> NativeDrawPlan {
    let mut commands = Vec::new();
    if let Some(popup) = plan.popup {
        commands.push(NativeDrawCommand::RoundRect {
            rect: popup,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.radius,
        });
        for row in &plan.popup_rows {
            let Some(spec) = items.get(row.item_index) else {
                continue;
            };
            if focused == Some(crate::ZsBreadcrumbFocusTarget::Item(spec.id())) {
                commands.push(NativeDrawCommand::RoundRect {
                    rect: row.bounds,
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 28,
                    },
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    radius: plan.radius,
                });
            }
            commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                spec.label(),
                row.text_bounds,
                SemanticTextStyle::body(),
            )));
        }
    }
    NativeDrawPlan::new(commands)
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker"
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsPopupPlacement {
    Below,
    Above,
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker"
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ZsPlacedPopup {
    bounds: Rect,
    placement: ZsPopupPlacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsToggleRenderPlan {
    pub bounds: Rect,
    pub track: Rect,
    pub knob: Rect,
    pub track_radius: i32,
    pub knob_radius: i32,
    pub hovered: bool,
    pub checked: bool,
}

/// Stable geometry for the owner-drawn settings toggle.
pub fn zs_toggle_render_plan(
    bounds: Rect,
    hovered: bool,
    checked: bool,
    dpi: Dpi,
) -> ZsToggleRenderPlan {
    let row_h = bounds.height.max(scale(24, dpi));
    let row_w = bounds.width.max(scale(48, dpi));
    let track_height = ((row_h * 20) / 32).clamp(scale(20, dpi), row_h - scale(4, dpi));
    let track_width =
        ((track_height * 40) / 20).clamp(track_height + scale(12, dpi), row_w - scale(6, dpi));
    let track_x = bounds.x + (bounds.width - track_width) / 2;
    let track_y = bounds.y + (bounds.height - track_height) / 2;
    let track = Rect {
        x: track_x,
        y: track_y,
        width: track_width,
        height: track_height,
    };
    let track_radius = (track_height / 2).max(scale(6, dpi));

    let knob_size = if checked {
        ((track_height * 14) / 20).max(scale(12, dpi))
    } else {
        ((track_height * 12) / 20).max(scale(10, dpi))
    };
    let knob_y = track_y + (track_height - knob_size) / 2;
    let knob_pad = if checked {
        ((track_height - knob_size) / 2).max(scale(3, dpi))
    } else {
        ((track_height - knob_size) / 2).max(scale(4, dpi))
    };
    let knob_x = if checked {
        track_x + track_width - knob_size - knob_pad
    } else {
        track_x + knob_pad
    };

    ZsToggleRenderPlan {
        bounds,
        track,
        knob: Rect {
            x: knob_x,
            y: knob_y,
            width: knob_size,
            height: knob_size,
        },
        track_radius,
        knob_radius: scale(if checked { 7 } else { 6 }, dpi),
        hovered,
        checked,
    }
}

pub fn zs_toggle_native_draw_plan(plan: &ZsToggleRenderPlan) -> NativeDrawPlan {
    let track_fill = if plan.checked {
        NativeDrawFill::Role(ColorRole::Accent)
    } else {
        NativeDrawFill::Role(ColorRole::Control)
    };
    let off_role = if plan.hovered {
        ColorRole::PrimaryText
    } else {
        ColorRole::SecondaryText
    };
    let track_stroke = if plan.checked {
        NativeDrawFill::Role(ColorRole::Accent)
    } else {
        NativeDrawFill::Role(off_role)
    };
    let knob_fill = if plan.checked {
        NativeDrawFill::Color(Color::rgb(255, 255, 255))
    } else {
        NativeDrawFill::Role(off_role)
    };

    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.track,
            fill: track_fill,
            stroke: Some(track_stroke),
            radius: plan.track_radius,
        },
        NativeDrawCommand::RoundRect {
            rect: plan.knob,
            fill: knob_fill,
            stroke: Some(knob_fill),
            radius: plan.knob_radius,
        },
    ])
}

#[cfg(feature = "toggle-button")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsToggleButtonPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "toggle-button")]
impl ZsToggleButtonPlatformStyle {
    pub(crate) const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "toggle-button")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsToggleButtonMetrics {
    pub minimum_height: Dp,
    pub radius: Dp,
    pub horizontal_padding: Dp,
    pub selected_indicator_width: Dp,
    pub selected_indicator_height: Dp,
    pub checked_content_offset_y: Dp,
}

#[cfg(feature = "toggle-button")]
impl ZsToggleButtonMetrics {
    pub const fn for_platform(platform: ZsToggleButtonPlatformStyle) -> Self {
        match platform {
            ZsToggleButtonPlatformStyle::Windows => Self {
                minimum_height: Dp::new(32.0),
                radius: Dp::new(4.0),
                horizontal_padding: Dp::new(12.0),
                selected_indicator_width: Dp::new(16.0),
                selected_indicator_height: Dp::new(2.0),
                checked_content_offset_y: Dp::new(0.0),
            },
            ZsToggleButtonPlatformStyle::Macos => Self {
                minimum_height: Dp::new(28.0),
                radius: Dp::new(6.0),
                horizontal_padding: Dp::new(12.0),
                selected_indicator_width: Dp::new(14.0),
                selected_indicator_height: Dp::new(2.0),
                checked_content_offset_y: Dp::new(1.0),
            },
            ZsToggleButtonPlatformStyle::Gtk => Self {
                minimum_height: Dp::new(34.0),
                radius: Dp::new(5.0),
                horizontal_padding: Dp::new(14.0),
                selected_indicator_width: Dp::new(18.0),
                selected_indicator_height: Dp::new(3.0),
                checked_content_offset_y: Dp::new(1.0),
            },
        }
    }
}

#[cfg(feature = "toggle-button")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsToggleButtonRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub selected_indicator: Rect,
    pub radius: i32,
    pub checked: bool,
    pub platform: ZsToggleButtonPlatformStyle,
}

#[cfg(feature = "toggle-button")]
pub fn zs_toggle_button_render_plan(
    bounds: Rect,
    checked: bool,
    platform: ZsToggleButtonPlatformStyle,
    dpi: Dpi,
) -> ZsToggleButtonRenderPlan {
    let metrics = ZsToggleButtonMetrics::for_platform(platform);
    let padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let offset_y = if checked {
        metrics
            .checked_content_offset_y
            .to_px(dpi)
            .round_i32()
            .max(0)
    } else {
        0
    };
    let indicator_width = metrics
        .selected_indicator_width
        .to_px(dpi)
        .round_i32()
        .min(bounds.width)
        .max(1);
    let indicator_height = metrics
        .selected_indicator_height
        .to_px(dpi)
        .round_i32()
        .min(bounds.height)
        .max(1);
    let indicator_inset = Dp::new(3.0)
        .to_px(dpi)
        .round_i32()
        .max(0)
        .min(bounds.height.saturating_sub(indicator_height).max(0));
    ZsToggleButtonRenderPlan {
        bounds,
        text_bounds: Rect {
            x: bounds.x.saturating_add(padding),
            y: bounds.y.saturating_add(offset_y),
            width: bounds.width.saturating_sub(padding.saturating_mul(2)),
            height: bounds.height.saturating_sub(offset_y),
        },
        selected_indicator: Rect {
            x: bounds.x + (bounds.width - indicator_width) / 2,
            y: bounds
                .y
                .saturating_add(bounds.height)
                .saturating_sub(indicator_height)
                .saturating_sub(indicator_inset),
            width: indicator_width,
            height: indicator_height,
        },
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        checked,
        platform,
    }
}

#[cfg(feature = "toggle-button")]
pub fn zs_toggle_button_native_draw_plan(
    plan: &ZsToggleButtonRenderPlan,
    label: &str,
) -> NativeDrawPlan {
    let mut text_style = SemanticTextStyle::body();
    text_style.role = TextRole::Button;
    text_style.color = if plan.checked {
        ColorRole::AccentText
    } else {
        ColorRole::PrimaryText
    };
    let mut commands = vec![
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(if plan.checked {
                ColorRole::Accent
            } else {
                ColorRole::Control
            }),
            stroke: Some(NativeDrawFill::Role(if plan.checked {
                ColorRole::Accent
            } else {
                ColorRole::Border
            })),
            radius: plan.radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            plan.text_bounds,
            text_style,
        )),
    ];
    if plan.checked {
        commands.push(NativeDrawCommand::RoundFill {
            rect: plan.selected_indicator,
            fill: NativeDrawFill::Role(ColorRole::AccentText),
            radius: (plan.selected_indicator.height / 2).max(1),
        });
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "slider")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsSliderRenderPlan {
    pub bounds: Rect,
    pub track: Rect,
    pub filled_track: Rect,
    pub thumb: Rect,
    pub track_radius: i32,
    pub thumb_radius: i32,
}

#[cfg(feature = "slider")]
pub fn zs_slider_render_plan(bounds: Rect, fraction: f32, dpi: Dpi) -> ZsSliderRenderPlan {
    let thumb_size = scale(16, dpi).min(bounds.height.max(1)).max(1);
    let thumb_radius = (thumb_size / 2).max(1);
    let track_height = scale(4, dpi).min(bounds.height.max(1)).max(1);
    let track_x = bounds.x.saturating_add(thumb_radius);
    let track_width = bounds
        .width
        .saturating_sub(thumb_radius.saturating_mul(2))
        .max(1);
    let track_y = bounds
        .y
        .saturating_add((bounds.height.saturating_sub(track_height)) / 2);
    let track = Rect {
        x: track_x,
        y: track_y,
        width: track_width,
        height: track_height,
    };
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled_width = ((track_width as f32) * fraction).round() as i32;
    let thumb_center = track_x.saturating_add(filled_width);
    let thumb = Rect {
        x: thumb_center.saturating_sub(thumb_radius),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(thumb_size)) / 2),
        width: thumb_size,
        height: thumb_size,
    };
    ZsSliderRenderPlan {
        bounds,
        track,
        filled_track: Rect {
            x: track.x,
            y: track.y,
            width: filled_width.max(1),
            height: track.height,
        },
        thumb,
        track_radius: (track_height / 2).max(1),
        thumb_radius,
    }
}

#[cfg(feature = "slider")]
pub fn zs_slider_native_draw_plan(plan: &ZsSliderRenderPlan) -> NativeDrawPlan {
    NativeDrawPlan::new([
        NativeDrawCommand::RoundFill {
            rect: plan.track,
            fill: NativeDrawFill::Role(ColorRole::Control),
            radius: plan.track_radius,
        },
        NativeDrawCommand::RoundFill {
            rect: plan.filled_track,
            fill: NativeDrawFill::Role(ColorRole::Accent),
            radius: plan.track_radius,
        },
        NativeDrawCommand::RoundRect {
            rect: plan.thumb,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
            radius: plan.thumb_radius,
        },
    ])
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsNumberBoxPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "number-box")]
impl ZsNumberBoxPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsNumberBoxMetrics {
    pub button_width: Dp,
    pub button_gap: Dp,
    pub text_inset: Dp,
    pub radius: Dp,
    pub horizontal_buttons: bool,
}

#[cfg(feature = "number-box")]
impl ZsNumberBoxMetrics {
    pub const fn for_platform(platform: ZsNumberBoxPlatformStyle) -> Self {
        match platform {
            ZsNumberBoxPlatformStyle::Windows => Self {
                button_width: Dp::new(32.0),
                button_gap: Dp::new(0.0),
                text_inset: Dp::new(8.0),
                radius: Dp::new(4.0),
                horizontal_buttons: true,
            },
            ZsNumberBoxPlatformStyle::Macos => Self {
                button_width: Dp::new(18.0),
                button_gap: Dp::new(4.0),
                text_inset: Dp::new(7.0),
                radius: Dp::new(5.0),
                horizontal_buttons: false,
            },
            ZsNumberBoxPlatformStyle::Gtk => Self {
                button_width: Dp::new(32.0),
                button_gap: Dp::new(0.0),
                text_inset: Dp::new(8.0),
                radius: Dp::new(5.0),
                horizontal_buttons: true,
            },
        }
    }
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsNumberBoxRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub decrement_button: Rect,
    pub increment_button: Rect,
    pub radius: i32,
    pub platform: ZsNumberBoxPlatformStyle,
}

#[cfg(feature = "number-box")]
pub fn zs_number_box_render_plan(
    bounds: Rect,
    platform: ZsNumberBoxPlatformStyle,
    dpi: Dpi,
) -> ZsNumberBoxRenderPlan {
    let metrics = ZsNumberBoxMetrics::for_platform(platform);
    let button_width = metrics
        .button_width
        .to_px(dpi)
        .round_i32()
        .min(bounds.width.max(1))
        .max(1);
    let gap = metrics.button_gap.to_px(dpi).round_i32().max(0);
    let inset = metrics.text_inset.to_px(dpi).round_i32().max(0);
    let trailing_width = if metrics.horizontal_buttons {
        button_width.saturating_mul(2)
    } else {
        button_width
    };
    let buttons_x = bounds
        .x
        .saturating_add(bounds.width.saturating_sub(trailing_width));
    let (decrement_button, increment_button) = if metrics.horizontal_buttons {
        (
            Rect {
                x: buttons_x,
                y: bounds.y,
                width: button_width,
                height: bounds.height,
            },
            Rect {
                x: buttons_x.saturating_add(button_width),
                y: bounds.y,
                width: button_width,
                height: bounds.height,
            },
        )
    } else {
        let upper_height = (bounds.height / 2).max(1);
        (
            Rect {
                x: buttons_x,
                y: bounds.y.saturating_add(upper_height),
                width: button_width,
                height: bounds.height.saturating_sub(upper_height).max(1),
            },
            Rect {
                x: buttons_x,
                y: bounds.y,
                width: button_width,
                height: upper_height,
            },
        )
    };
    ZsNumberBoxRenderPlan {
        bounds,
        text_bounds: Rect {
            x: bounds.x.saturating_add(inset),
            y: bounds.y,
            width: bounds
                .width
                .saturating_sub(trailing_width)
                .saturating_sub(gap)
                .saturating_sub(inset.saturating_mul(2))
                .max(0),
            height: bounds.height,
        },
        decrement_button,
        increment_button,
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "number-box")]
pub fn zs_number_box_native_draw_plan(
    plan: &ZsNumberBoxRenderPlan,
    text: &str,
    valid: bool,
    decrement_enabled: bool,
    increment_enabled: bool,
) -> NativeDrawPlan {
    let stroke = if valid {
        ColorRole::Control
    } else {
        ColorRole::Danger
    };
    let (decrement_label, increment_label) = match plan.platform {
        ZsNumberBoxPlatformStyle::Gtk => ("−", "+"),
        ZsNumberBoxPlatformStyle::Windows | ZsNumberBoxPlatformStyle::Macos => ("▼", "▲"),
    };
    let mut decrement_style = SemanticTextStyle::body();
    decrement_style.color = if decrement_enabled {
        ColorRole::PrimaryText
    } else {
        ColorRole::SecondaryText
    };
    let mut increment_style = SemanticTextStyle::body();
    increment_style.color = if increment_enabled {
        ColorRole::PrimaryText
    } else {
        ColorRole::SecondaryText
    };
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(stroke)),
            radius: plan.radius,
        },
        NativeDrawCommand::FillRect {
            rect: plan.decrement_button,
            fill: NativeDrawFill::Role(ColorRole::Control),
        },
        NativeDrawCommand::FillRect {
            rect: plan.increment_button,
            fill: NativeDrawFill::Role(ColorRole::Control),
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            text,
            plan.text_bounds,
            SemanticTextStyle::body(),
        )),
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            decrement_label,
            plan.decrement_button,
            decrement_style,
        )),
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            increment_label,
            plan.increment_button,
            increment_style,
        )),
    ])
}

#[cfg(feature = "radio")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsRadioRenderPlan {
    pub bounds: Rect,
    pub indicator: Rect,
    pub selected_dot: Option<Rect>,
    pub indicator_radius: i32,
    pub dot_radius: i32,
    pub selected: bool,
}

#[cfg(feature = "radio")]
pub fn zs_radio_render_plan(bounds: Rect, selected: bool, dpi: Dpi) -> ZsRadioRenderPlan {
    let indicator_size = scale(20, dpi).min(bounds.height.max(1)).max(1);
    let indicator_radius = (indicator_size / 2).max(1);
    let indicator = Rect {
        x: bounds.x,
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(indicator_size)) / 2),
        width: indicator_size,
        height: indicator_size,
    };
    let dot_size = scale(8, dpi).min(indicator_size).max(1);
    let dot_inset = (indicator_size.saturating_sub(dot_size)) / 2;
    ZsRadioRenderPlan {
        bounds,
        indicator,
        selected_dot: selected.then_some(Rect {
            x: indicator.x.saturating_add(dot_inset),
            y: indicator.y.saturating_add(dot_inset),
            width: dot_size,
            height: dot_size,
        }),
        indicator_radius,
        dot_radius: (dot_size / 2).max(1),
        selected,
    }
}

#[cfg(feature = "radio")]
pub fn zs_radio_native_draw_plan(plan: &ZsRadioRenderPlan) -> NativeDrawPlan {
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: plan.indicator,
        fill: NativeDrawFill::Role(ColorRole::Surface),
        stroke: Some(NativeDrawFill::Role(if plan.selected {
            ColorRole::Accent
        } else {
            ColorRole::Border
        })),
        radius: plan.indicator_radius,
    }];
    if let Some(dot) = plan.selected_dot {
        commands.push(NativeDrawCommand::RoundFill {
            rect: dot,
            fill: NativeDrawFill::Role(ColorRole::Accent),
            radius: plan.dot_radius,
        });
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "progress")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsProgressBarRenderPlan {
    pub bounds: Rect,
    pub track: Rect,
    pub filled_track: Option<Rect>,
    pub radius: i32,
}

#[cfg(feature = "progress")]
pub fn zs_progress_bar_render_plan(
    bounds: Rect,
    fraction: f32,
    dpi: Dpi,
) -> ZsProgressBarRenderPlan {
    let track_height = scale(4, dpi).min(bounds.height.max(1)).max(1);
    let track = Rect {
        x: bounds.x,
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(track_height)) / 2),
        width: bounds.width.max(1),
        height: track_height,
    };
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled_width = ((track.width as f32) * fraction).round() as i32;
    ZsProgressBarRenderPlan {
        bounds,
        track,
        filled_track: (filled_width > 0).then_some(Rect {
            width: filled_width.min(track.width),
            ..track
        }),
        radius: (track_height / 2).max(1),
    }
}

#[cfg(feature = "progress")]
pub fn zs_progress_bar_native_draw_plan(plan: &ZsProgressBarRenderPlan) -> NativeDrawPlan {
    let mut commands = vec![NativeDrawCommand::RoundFill {
        rect: plan.track,
        fill: NativeDrawFill::Role(ColorRole::Control),
        radius: plan.radius,
    }];
    if let Some(filled_track) = plan.filled_track {
        commands.push(NativeDrawCommand::RoundFill {
            rect: filled_track,
            fill: NativeDrawFill::Role(ColorRole::Accent),
            radius: plan.radius,
        });
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTabPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "tabs")]
impl ZsTabPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }

    pub const fn arrow_selects(self) -> bool {
        matches!(self, Self::Macos)
    }

    pub const fn supports_home_end_focus(self) -> bool {
        matches!(self, Self::Gtk)
    }
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTabViewMetrics {
    pub strip_height: Dp,
    pub outer_inset: Dp,
    pub item_gap: Dp,
    pub horizontal_padding: Dp,
    pub minimum_item_width: Dp,
    pub maximum_item_width: Dp,
    pub radius: Dp,
    pub selection_indicator_height: Dp,
}

#[cfg(feature = "tabs")]
impl ZsTabViewMetrics {
    pub const fn for_platform(platform: ZsTabPlatformStyle) -> Self {
        match platform {
            ZsTabPlatformStyle::Windows => Self {
                strip_height: Dp::new(40.0),
                outer_inset: Dp::new(0.0),
                item_gap: Dp::new(2.0),
                horizontal_padding: Dp::new(16.0),
                minimum_item_width: Dp::new(120.0),
                maximum_item_width: Dp::new(240.0),
                radius: Dp::new(8.0),
                selection_indicator_height: Dp::new(2.0),
            },
            ZsTabPlatformStyle::Macos => Self {
                strip_height: Dp::new(32.0),
                outer_inset: Dp::new(12.0),
                item_gap: Dp::new(0.0),
                horizontal_padding: Dp::new(14.0),
                minimum_item_width: Dp::new(72.0),
                maximum_item_width: Dp::new(160.0),
                radius: Dp::new(6.0),
                selection_indicator_height: Dp::new(0.0),
            },
            ZsTabPlatformStyle::Gtk => Self {
                strip_height: Dp::new(38.0),
                outer_inset: Dp::new(0.0),
                item_gap: Dp::new(0.0),
                horizontal_padding: Dp::new(12.0),
                minimum_item_width: Dp::new(72.0),
                maximum_item_width: Dp::new(220.0),
                radius: Dp::new(6.0),
                selection_indicator_height: Dp::new(3.0),
            },
        }
    }
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTabHeaderRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub selected: bool,
    pub selection_indicator: Option<Rect>,
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTabViewRenderPlan {
    pub bounds: Rect,
    pub strip_bounds: Rect,
    pub content_bounds: Rect,
    pub headers: Vec<ZsTabHeaderRenderPlan>,
    pub platform: ZsTabPlatformStyle,
    pub radius: i32,
}

#[cfg(feature = "tabs")]
pub fn zs_tab_view_render_plan(
    bounds: Rect,
    labels: &[String],
    selected_index: Option<usize>,
    platform: ZsTabPlatformStyle,
    dpi: Dpi,
) -> ZsTabViewRenderPlan {
    let metrics = ZsTabViewMetrics::for_platform(platform);
    let strip_height = metrics
        .strip_height
        .to_px(dpi)
        .round_i32()
        .min(bounds.height.max(0))
        .max(0);
    let strip_bounds = Rect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(0),
        height: strip_height,
    };
    let content_bounds = Rect {
        x: bounds.x,
        y: bounds.y.saturating_add(strip_height),
        width: bounds.width.max(0),
        height: bounds.height.saturating_sub(strip_height).max(0),
    };
    let inset = metrics
        .outer_inset
        .to_px(dpi)
        .round_i32()
        .max(0)
        .min(strip_bounds.width / 2);
    let interior_width = strip_bounds
        .width
        .saturating_sub(inset.saturating_mul(2))
        .max(0);
    let gap_count = labels.len().saturating_sub(1) as i32;
    let gap = metrics
        .item_gap
        .to_px(dpi)
        .round_i32()
        .max(0)
        .min(if gap_count > 0 {
            interior_width / gap_count
        } else {
            0
        });
    let horizontal_padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let minimum_width = metrics.minimum_item_width.to_px(dpi).round_i32().max(1);
    let maximum_width = metrics
        .maximum_item_width
        .to_px(dpi)
        .round_i32()
        .max(minimum_width);
    let available_width = interior_width
        .saturating_sub(gap.saturating_mul(labels.len().saturating_sub(1) as i32))
        .max(0);
    let text_unit = Dp::new(7.5).to_px(dpi).round_i32().max(1);
    let mut widths = labels
        .iter()
        .map(|label| {
            (label.chars().count() as i32)
                .saturating_mul(text_unit)
                .saturating_add(horizontal_padding.saturating_mul(2))
                .clamp(minimum_width, maximum_width)
        })
        .collect::<Vec<_>>();
    if !widths.is_empty() {
        if platform == ZsTabPlatformStyle::Macos {
            let equal = widths.iter().copied().max().unwrap_or(minimum_width);
            widths.fill(equal);
        }
        let requested: i32 = widths.iter().copied().sum();
        if requested > available_width {
            let equal = available_width / widths.len() as i32;
            let remainder = available_width % widths.len() as i32;
            for (index, width) in widths.iter_mut().enumerate() {
                *width = equal + i32::from((index as i32) < remainder);
            }
        }
    }
    let assigned_width: i32 = widths.iter().copied().sum::<i32>()
        + gap.saturating_mul(widths.len().saturating_sub(1) as i32);
    let mut x = strip_bounds.x.saturating_add(inset);
    if platform == ZsTabPlatformStyle::Macos {
        x = x.saturating_add(available_width.saturating_sub(assigned_width) / 2);
    }
    let indicator_height = metrics
        .selection_indicator_height
        .to_px(dpi)
        .round_i32()
        .max(0);
    let headers = widths
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            let header = Rect {
                x,
                y: strip_bounds.y,
                width,
                height: strip_bounds.height,
            };
            x = x.saturating_add(width).saturating_add(gap);
            let selected = selected_index == Some(index);
            let header_padding = horizontal_padding.min(header.width / 2);
            let selection_indicator = (selected && indicator_height > 0 && header.width > 0)
                .then_some(Rect {
                    x: header.x.saturating_add(header_padding / 2),
                    y: header
                        .y
                        .saturating_add(header.height.saturating_sub(indicator_height)),
                    width: header.width.saturating_sub(header_padding).max(1),
                    height: indicator_height,
                });
            ZsTabHeaderRenderPlan {
                bounds: header,
                text_bounds: Rect {
                    x: header.x.saturating_add(header_padding),
                    y: header.y,
                    width: header
                        .width
                        .saturating_sub(header_padding.saturating_mul(2))
                        .max(0),
                    height: header.height,
                },
                selected,
                selection_indicator,
            }
        })
        .collect();
    ZsTabViewRenderPlan {
        bounds,
        strip_bounds,
        content_bounds,
        headers,
        platform,
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
    }
}

#[cfg(feature = "tabs")]
pub fn zs_tab_view_native_draw_plan(
    plan: &ZsTabViewRenderPlan,
    labels: &[String],
) -> NativeDrawPlan {
    let mut commands = vec![NativeDrawCommand::FillRect {
        rect: plan.strip_bounds,
        fill: NativeDrawFill::Role(ColorRole::Surface),
    }];
    if plan.platform == ZsTabPlatformStyle::Macos && !plan.headers.is_empty() {
        let first = plan
            .headers
            .first()
            .expect("tab headers are non-empty")
            .bounds;
        let last = plan
            .headers
            .last()
            .expect("tab headers are non-empty")
            .bounds;
        commands.push(NativeDrawCommand::RoundRect {
            rect: Rect {
                x: first.x,
                y: first.y.saturating_add(2),
                width: last.x.saturating_add(last.width).saturating_sub(first.x),
                height: first.height.saturating_sub(4).max(1),
            },
            fill: NativeDrawFill::Role(ColorRole::Control),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.radius,
        });
    }
    for (header, label) in plan.headers.iter().zip(labels) {
        let fill = match (plan.platform, header.selected) {
            (_, true) => NativeDrawFill::Role(ColorRole::SurfaceRaised),
            (ZsTabPlatformStyle::Windows, false) => NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Control,
                alpha: 112,
            },
            _ => NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Control,
                alpha: 1,
            },
        };
        let stroke = match (plan.platform, header.selected) {
            (ZsTabPlatformStyle::Windows, true) => Some(NativeDrawFill::Role(ColorRole::Border)),
            (ZsTabPlatformStyle::Macos, _) => None,
            _ => None,
        };
        commands.push(NativeDrawCommand::RoundRect {
            rect: header.bounds,
            fill,
            stroke,
            radius: plan.radius,
        });
        if let Some(indicator) = header.selection_indicator {
            commands.push(NativeDrawCommand::RoundFill {
                rect: indicator,
                fill: NativeDrawFill::Role(ColorRole::Accent),
                radius: indicator.height.max(1) / 2,
            });
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            header.text_bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: if header.selected {
                    ColorRole::PrimaryText
                } else {
                    ColorRole::SecondaryText
                },
                weight: if header.selected {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Center,
                ..SemanticTextStyle::body()
            },
        )));
    }
    commands.push(NativeDrawCommand::StrokeRect {
        rect: plan.content_bounds,
        stroke: NativeDrawFill::Role(ColorRole::Border),
        width: 1,
    });
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "auto-suggest")]
pub const ZS_AUTO_SUGGEST_MAX_VISIBLE_ITEMS: usize = 8;

#[cfg(feature = "auto-suggest")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsAutoSuggestPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "auto-suggest")]
impl ZsAutoSuggestPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }
}

#[cfg(feature = "auto-suggest")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsAutoSuggestMetrics {
    pub control_height: Dp,
    pub row_height: Dp,
    pub text_padding: Dp,
    pub icon_column_width: Dp,
    pub icon_size: Dp,
    pub popup_gap: Dp,
    pub control_radius: Dp,
    pub overlay_radius: Dp,
    pub leading_search_icon: bool,
}

#[cfg(feature = "auto-suggest")]
impl ZsAutoSuggestMetrics {
    pub const fn for_platform(platform: ZsAutoSuggestPlatformStyle) -> Self {
        match platform {
            ZsAutoSuggestPlatformStyle::Windows => Self {
                control_height: Dp::new(32.0),
                row_height: Dp::new(36.0),
                text_padding: Dp::new(12.0),
                icon_column_width: Dp::new(32.0),
                icon_size: Dp::new(16.0),
                popup_gap: Dp::new(4.0),
                control_radius: Dp::new(4.0),
                overlay_radius: Dp::new(8.0),
                leading_search_icon: false,
            },
            ZsAutoSuggestPlatformStyle::Macos => Self {
                control_height: Dp::new(28.0),
                row_height: Dp::new(28.0),
                text_padding: Dp::new(8.0),
                icon_column_width: Dp::new(24.0),
                icon_size: Dp::new(14.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(6.0),
                overlay_radius: Dp::new(10.0),
                leading_search_icon: true,
            },
            ZsAutoSuggestPlatformStyle::Gtk => Self {
                control_height: Dp::new(34.0),
                row_height: Dp::new(34.0),
                text_padding: Dp::new(10.0),
                icon_column_width: Dp::new(28.0),
                icon_size: Dp::new(16.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(8.0),
                overlay_radius: Dp::new(12.0),
                leading_search_icon: true,
            },
        }
    }
}

#[cfg(feature = "auto-suggest")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAutoSuggestRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub search_button: Option<Rect>,
    pub search_icon: Option<Rect>,
    pub clear_button: Option<Rect>,
    pub clear_icon: Option<Rect>,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub first_visible_suggestion: usize,
    pub suggestion_rows: Vec<Rect>,
    pub control_radius: i32,
    pub overlay_radius: i32,
    pub platform: ZsAutoSuggestPlatformStyle,
}

#[cfg(feature = "auto-suggest")]
#[allow(clippy::too_many_arguments)]
pub fn zs_auto_suggest_render_plan(
    bounds: Rect,
    row_count: usize,
    highlighted_index: Option<usize>,
    expanded: bool,
    query_empty: bool,
    query_icon: bool,
    platform: ZsAutoSuggestPlatformStyle,
    dpi: Dpi,
) -> ZsAutoSuggestRenderPlan {
    zs_auto_suggest_render_plan_impl(
        bounds,
        row_count,
        highlighted_index,
        expanded,
        query_empty,
        query_icon,
        platform,
        dpi,
        None,
    )
}

#[cfg(feature = "auto-suggest")]
#[allow(clippy::too_many_arguments)]
pub fn zs_auto_suggest_render_plan_in_viewport(
    bounds: Rect,
    row_count: usize,
    highlighted_index: Option<usize>,
    expanded: bool,
    query_empty: bool,
    query_icon: bool,
    platform: ZsAutoSuggestPlatformStyle,
    dpi: Dpi,
    viewport: Rect,
) -> ZsAutoSuggestRenderPlan {
    zs_auto_suggest_render_plan_impl(
        bounds,
        row_count,
        highlighted_index,
        expanded,
        query_empty,
        query_icon,
        platform,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "auto-suggest")]
#[allow(clippy::too_many_arguments)]
fn zs_auto_suggest_render_plan_impl(
    bounds: Rect,
    row_count: usize,
    highlighted_index: Option<usize>,
    expanded: bool,
    query_empty: bool,
    query_icon: bool,
    platform: ZsAutoSuggestPlatformStyle,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsAutoSuggestRenderPlan {
    let metrics = ZsAutoSuggestMetrics::for_platform(platform);
    let padding = metrics.text_padding.to_px(dpi).round_i32().max(1);
    let icon_column = metrics.icon_column_width.to_px(dpi).round_i32().max(1);
    let icon_size = metrics
        .icon_size
        .to_px(dpi)
        .round_i32()
        .clamp(1, bounds.height.max(1));
    let icon_rect = |x: i32| Rect {
        x: x.saturating_add((icon_column.saturating_sub(icon_size)) / 2),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(icon_size)) / 2),
        width: icon_size,
        height: icon_size,
    };
    let button_rect = |x: i32| Rect {
        x,
        y: bounds.y,
        width: icon_column.min(bounds.width.max(1)),
        height: bounds.height,
    };
    let search_visible = query_icon && (metrics.leading_search_icon || query_empty);
    let search_button = search_visible.then(|| {
        let x = if metrics.leading_search_icon {
            bounds.x
        } else {
            bounds
                .x
                .saturating_add(bounds.width)
                .saturating_sub(icon_column)
        };
        button_rect(x)
    });
    let clear_button = (!query_empty).then(|| {
        button_rect(
            bounds
                .x
                .saturating_add(bounds.width)
                .saturating_sub(icon_column),
        )
    });
    let text_left = bounds.x.saturating_add(padding).saturating_add(
        if search_visible && metrics.leading_search_icon {
            icon_column
        } else {
            0
        },
    );
    let trailing_column =
        if clear_button.is_some() || (search_visible && !metrics.leading_search_icon) {
            icon_column
        } else {
            0
        };
    let text_right = bounds
        .x
        .saturating_add(bounds.width)
        .saturating_sub(padding)
        .saturating_sub(trailing_column);
    let text_bounds = Rect {
        x: text_left,
        y: bounds.y,
        width: text_right.saturating_sub(text_left).max(0),
        height: bounds.height,
    };
    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let gap = metrics.popup_gap.to_px(dpi).round_i32().max(0);
    let visible_rows = auto_suggest_visible_row_count(bounds, row_count, row_height, gap, viewport);
    let maximum_first = row_count.saturating_sub(visible_rows);
    let first_visible_suggestion = highlighted_index
        .filter(|index| *index < row_count)
        .map(|index| {
            index
                .saturating_add(1)
                .saturating_sub(visible_rows)
                .min(maximum_first)
        })
        .unwrap_or_default();
    let placed_popup = (expanded && row_count > 0).then(|| {
        place_popup(
            bounds,
            bounds.width.max(1),
            row_height.saturating_mul(i32::try_from(visible_rows).unwrap_or(i32::MAX)),
            gap,
            viewport,
        )
    });
    let popup = placed_popup.map(|placed| placed.bounds);
    let suggestion_rows = popup
        .map(|popup| {
            (0..visible_rows)
                .map(|index| Rect {
                    x: popup.x,
                    y: popup.y.saturating_add(
                        row_height.saturating_mul(i32::try_from(index).unwrap_or(i32::MAX)),
                    ),
                    width: popup.width,
                    height: row_height,
                })
                .collect()
        })
        .unwrap_or_default();
    ZsAutoSuggestRenderPlan {
        bounds,
        text_bounds,
        search_button,
        search_icon: search_button.map(|button| icon_rect(button.x)),
        clear_button,
        clear_icon: clear_button.map(|button| icon_rect(button.x)),
        popup,
        popup_placement: placed_popup.map(|placed| placed.placement),
        first_visible_suggestion,
        suggestion_rows,
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(1),
        overlay_radius: metrics.overlay_radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "auto-suggest")]
fn auto_suggest_visible_row_count(
    anchor: Rect,
    row_count: usize,
    row_height: i32,
    gap: i32,
    viewport: Option<Rect>,
) -> usize {
    let capped_count = row_count.min(ZS_AUTO_SUGGEST_MAX_VISIBLE_ITEMS);
    let Some(viewport) = viewport.filter(|viewport| viewport.width > 0 && viewport.height > 0)
    else {
        return capped_count;
    };
    let viewport_bottom = viewport.y.saturating_add(viewport.height);
    let below_y = anchor.y.saturating_add(anchor.height).saturating_add(gap);
    let above_bottom = anchor.y.saturating_sub(gap);
    let available_below = viewport_bottom.saturating_sub(below_y).max(0);
    let available_above = above_bottom.saturating_sub(viewport.y).max(0);
    let available_rows = available_below.max(available_above) / row_height.max(1);
    capped_count.min(available_rows.max(1) as usize)
}

#[cfg(feature = "auto-suggest")]
pub fn zs_auto_suggest_header_native_draw_plan(
    plan: &ZsAutoSuggestRenderPlan,
    query: &str,
    placeholder: Option<&str>,
) -> NativeDrawPlan {
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: plan.bounds,
        fill: NativeDrawFill::Role(ColorRole::Surface),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.control_radius,
    }];
    let mut text_style = SemanticTextStyle::body();
    let label = if query.is_empty() {
        text_style.color = ColorRole::SecondaryText;
        placeholder.unwrap_or_default()
    } else {
        query
    };
    commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
        label,
        plan.text_bounds,
        text_style,
    )));
    if let Some(bounds) = plan.search_icon {
        commands.push(NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(ZsIcon::Search, bounds, NativeIconColorMode::ThemeAware)
                .with_color(ColorRole::SecondaryText),
        ));
    }
    if let Some(bounds) = plan.clear_icon {
        commands.push(NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(ZsIcon::Close, bounds, NativeIconColorMode::ThemeAware)
                .with_color(ColorRole::SecondaryText),
        ));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "auto-suggest")]
pub fn zs_auto_suggest_popup_native_draw_plan(
    plan: &ZsAutoSuggestRenderPlan,
    suggestions: &[ZsAutoSuggestion],
    highlighted: Option<crate::ZsAutoSuggestionId>,
    no_results_text: Option<&str>,
    dpi: Dpi,
) -> NativeDrawPlan {
    let Some(popup) = plan.popup else {
        return NativeDrawPlan::default();
    };
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: popup,
        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.overlay_radius,
    }];
    let padding = ZsAutoSuggestMetrics::for_platform(plan.platform)
        .text_padding
        .to_px(dpi)
        .round_i32()
        .max(1);
    if suggestions.is_empty() {
        if let (Some(label), Some(row)) = (no_results_text, plan.suggestion_rows.first()) {
            let mut style = SemanticTextStyle::body();
            style.color = ColorRole::SecondaryText;
            commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                label,
                inset_row_text(*row, padding),
                style,
            )));
        }
        return NativeDrawPlan::new(commands);
    }
    for (suggestion, row) in suggestions
        .iter()
        .skip(plan.first_visible_suggestion)
        .zip(&plan.suggestion_rows)
    {
        if highlighted == Some(suggestion.id()) {
            commands.push(NativeDrawCommand::RoundFill {
                rect: *row,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                radius: plan.control_radius,
            });
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            suggestion.text(),
            inset_row_text(*row, padding),
            SemanticTextStyle::body(),
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(any(feature = "auto-suggest", feature = "table"))]
fn inset_row_text(row: Rect, padding: i32) -> Rect {
    Rect {
        x: row.x.saturating_add(padding),
        y: row.y,
        width: row.width.saturating_sub(padding.saturating_mul(2)).max(0),
        height: row.height,
    }
}

#[cfg(feature = "grid-view")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsGridViewPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "grid-view")]
impl ZsGridViewPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "grid-view")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsGridViewMetrics {
    pub minimum_item_width: Dp,
    pub item_height: Dp,
    pub item_gap: Dp,
    pub item_padding: Dp,
    pub media_height: Dp,
    pub icon_size: Dp,
    pub item_radius: Dp,
    pub media_radius: Dp,
    pub text_gap: Dp,
}

#[cfg(feature = "grid-view")]
impl ZsGridViewMetrics {
    pub const fn for_platform(platform: ZsGridViewPlatformStyle) -> Self {
        match platform {
            ZsGridViewPlatformStyle::Windows => Self {
                minimum_item_width: Dp::new(132.0),
                item_height: Dp::new(112.0),
                item_gap: Dp::new(8.0),
                item_padding: Dp::new(10.0),
                media_height: Dp::new(58.0),
                icon_size: Dp::new(32.0),
                item_radius: Dp::new(4.0),
                media_radius: Dp::new(3.0),
                text_gap: Dp::new(2.0),
            },
            ZsGridViewPlatformStyle::Macos => Self {
                minimum_item_width: Dp::new(124.0),
                item_height: Dp::new(104.0),
                item_gap: Dp::new(10.0),
                item_padding: Dp::new(9.0),
                media_height: Dp::new(54.0),
                icon_size: Dp::new(28.0),
                item_radius: Dp::new(8.0),
                media_radius: Dp::new(6.0),
                text_gap: Dp::new(1.0),
            },
            ZsGridViewPlatformStyle::Gtk => Self {
                minimum_item_width: Dp::new(136.0),
                item_height: Dp::new(116.0),
                item_gap: Dp::new(8.0),
                item_padding: Dp::new(10.0),
                media_height: Dp::new(60.0),
                icon_size: Dp::new(32.0),
                item_radius: Dp::new(6.0),
                media_radius: Dp::new(5.0),
                text_gap: Dp::new(2.0),
            },
        }
    }
}

#[cfg(feature = "grid-view")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsGridViewItemRenderPlan {
    pub item_index: usize,
    pub bounds: Rect,
    pub media_bounds: Rect,
    pub icon_bounds: Option<Rect>,
    pub title_bounds: Rect,
    pub subtitle_bounds: Option<Rect>,
    pub selected: bool,
}

#[cfg(feature = "grid-view")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsGridViewRenderPlan {
    pub bounds: Rect,
    pub items: Vec<ZsGridViewItemRenderPlan>,
    pub column_count: usize,
    pub row_count: usize,
    pub content_height: i32,
    pub item_radius: i32,
    pub media_radius: i32,
    pub platform: ZsGridViewPlatformStyle,
}

#[cfg(feature = "grid-view")]
pub fn zs_grid_view_render_plan(
    bounds: Rect,
    items: &[crate::ZsGridViewItem],
    selected: Option<crate::ZsGridViewItemId>,
    platform: ZsGridViewPlatformStyle,
    dpi: Dpi,
) -> ZsGridViewRenderPlan {
    let metrics = ZsGridViewMetrics::for_platform(platform);
    let minimum_item_width = metrics.minimum_item_width.to_px(dpi).round_i32().max(1);
    let item_height = metrics.item_height.to_px(dpi).round_i32().max(1);
    let gap = metrics.item_gap.to_px(dpi).round_i32().max(0);
    let padding = metrics.item_padding.to_px(dpi).round_i32().max(0);
    let media_height = metrics
        .media_height
        .to_px(dpi)
        .round_i32()
        .min(item_height)
        .max(1);
    let icon_size = metrics.icon_size.to_px(dpi).round_i32().max(1);
    let text_gap = metrics.text_gap.to_px(dpi).round_i32().max(0);
    let unique = crate::grid_view::unique_grid_view_items(items);
    let available_width = bounds.width.max(1);
    let calculated_columns = available_width
        .saturating_add(gap)
        .checked_div(minimum_item_width.saturating_add(gap).max(1))
        .unwrap_or(1)
        .max(1) as usize;
    let column_count = calculated_columns.min(unique.len().max(1));
    let gaps_width = gap.saturating_mul(column_count.saturating_sub(1) as i32);
    let item_width = available_width
        .saturating_sub(gaps_width)
        .checked_div(column_count as i32)
        .unwrap_or(available_width)
        .max(1);
    let row_count = unique.len().div_ceil(column_count);
    let content_height = item_height
        .saturating_mul(row_count as i32)
        .saturating_add(gap.saturating_mul(row_count.saturating_sub(1) as i32));
    let item_plans = unique
        .iter()
        .enumerate()
        .map(|(unique_index, item)| {
            let source_index = items
                .iter()
                .position(|candidate| candidate.id() == item.id())
                .unwrap_or(unique_index);
            let row = unique_index / column_count;
            let column = unique_index % column_count;
            let x = bounds
                .x
                .saturating_add((item_width.saturating_add(gap)).saturating_mul(column as i32));
            let width = if column + 1 == column_count {
                bounds
                    .x
                    .saturating_add(available_width)
                    .saturating_sub(x)
                    .max(1)
            } else {
                item_width
            };
            let y = bounds
                .y
                .saturating_add((item_height.saturating_add(gap)).saturating_mul(row as i32));
            let tile = Rect {
                x,
                y,
                width,
                height: item_height,
            };
            let inner_x = tile.x.saturating_add(padding.min(tile.width / 2));
            let inner_width = tile.width.saturating_sub(padding.saturating_mul(2)).max(1);
            let media = Rect {
                x: inner_x,
                y: tile.y.saturating_add(padding),
                width: inner_width,
                height: media_height.saturating_sub(padding).max(1),
            };
            let centered_icon = item.item_icon().map(|_| {
                let size = icon_size.min(media.width).min(media.height).max(1);
                Rect {
                    x: media
                        .x
                        .saturating_add((media.width.saturating_sub(size)) / 2),
                    y: media
                        .y
                        .saturating_add((media.height.saturating_sub(size)) / 2),
                    width: size,
                    height: size,
                }
            });
            let title_y = media
                .y
                .saturating_add(media.height)
                .saturating_add(text_gap);
            let text_bottom = tile.y.saturating_add(tile.height).saturating_sub(padding);
            let has_subtitle = item.item_subtitle().is_some();
            let text_height = text_bottom.saturating_sub(title_y).max(1);
            let title_height = if has_subtitle {
                text_height
                    .saturating_sub(text_gap)
                    .checked_div(2)
                    .unwrap_or(1)
            } else {
                text_height
            }
            .max(1);
            let title = Rect {
                x: inner_x,
                y: title_y,
                width: inner_width,
                height: title_height,
            };
            ZsGridViewItemRenderPlan {
                item_index: source_index,
                bounds: tile,
                media_bounds: media,
                icon_bounds: centered_icon,
                title_bounds: title,
                subtitle_bounds: has_subtitle.then_some(Rect {
                    x: inner_x,
                    y: title
                        .y
                        .saturating_add(title.height)
                        .saturating_add(text_gap),
                    width: inner_width,
                    height: text_bottom
                        .saturating_sub(title.y.saturating_add(title.height))
                        .saturating_sub(text_gap)
                        .max(1),
                }),
                selected: selected == Some(item.id()),
            }
        })
        .collect();
    ZsGridViewRenderPlan {
        bounds,
        items: item_plans,
        column_count,
        row_count,
        content_height,
        item_radius: metrics.item_radius.to_px(dpi).round_i32().max(1),
        media_radius: metrics.media_radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "grid-view")]
pub fn zs_grid_view_native_draw_plan(
    plan: &ZsGridViewRenderPlan,
    items: &[crate::ZsGridViewItem],
) -> NativeDrawPlan {
    let mut commands = vec![
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: None,
            radius: plan.item_radius,
        },
        NativeDrawCommand::PushClip { rect: plan.bounds },
    ];
    for tile in &plan.items {
        let Some(item) = items.get(tile.item_index) else {
            continue;
        };
        let (fill, stroke) = if tile.selected {
            match plan.platform {
                ZsGridViewPlatformStyle::Windows => (
                    NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 28,
                    },
                    Some(NativeDrawFill::Role(ColorRole::Accent)),
                ),
                ZsGridViewPlatformStyle::Macos => (
                    NativeDrawFill::Role(ColorRole::Accent),
                    Some(NativeDrawFill::Role(ColorRole::Accent)),
                ),
                ZsGridViewPlatformStyle::Gtk => (
                    NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 44,
                    },
                    Some(NativeDrawFill::Role(ColorRole::Accent)),
                ),
            }
        } else {
            (
                NativeDrawFill::Role(ColorRole::Control),
                Some(NativeDrawFill::Role(ColorRole::Border)),
            )
        };
        commands.push(NativeDrawCommand::RoundRect {
            rect: tile.bounds,
            fill,
            stroke,
            radius: plan.item_radius,
        });
        commands.push(NativeDrawCommand::RoundFill {
            rect: tile.media_bounds,
            fill: NativeDrawFill::RoleWithAlpha {
                role: if tile.selected && plan.platform == ZsGridViewPlatformStyle::Macos {
                    ColorRole::AccentText
                } else {
                    ColorRole::PrimaryText
                },
                alpha: if tile.selected { 22 } else { 10 },
            },
            radius: plan.media_radius,
        });
        let foreground = if tile.selected && plan.platform == ZsGridViewPlatformStyle::Macos {
            ColorRole::AccentText
        } else {
            ColorRole::PrimaryText
        };
        if let (Some(icon), Some(bounds)) = (item.item_icon(), tile.icon_bounds) {
            commands.push(NativeDrawCommand::Icon(
                NativeDrawIconCommand::new(icon, bounds, NativeIconColorMode::ThemeAware)
                    .with_color(foreground),
            ));
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            item.title(),
            tile.title_bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color: foreground,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
        if let (Some(subtitle), Some(bounds)) = (item.item_subtitle(), tile.subtitle_bounds) {
            commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                subtitle,
                bounds,
                SemanticTextStyle {
                    role: TextRole::Caption,
                    color: if foreground == ColorRole::AccentText {
                        foreground
                    } else {
                        ColorRole::SecondaryText
                    },
                    weight: TextWeight::Regular,
                    horizontal_align: HorizontalAlign::Start,
                    vertical_align: crate::VerticalAlign::Center,
                    wrap: crate::TextWrap::NoWrap,
                    ellipsis: true,
                },
            )));
        }
    }
    commands.push(NativeDrawCommand::PopClip);
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "tree")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTreePlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "tree")]
impl ZsTreePlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }
}

#[cfg(feature = "tree")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTreeViewMetrics {
    pub row_height: Dp,
    pub depth_indent: Dp,
    pub disclosure_column: Dp,
    pub disclosure_size: Dp,
    pub icon_size: Dp,
    pub leading_padding: Dp,
    pub content_gap: Dp,
    pub row_radius: Dp,
}

#[cfg(feature = "tree")]
impl ZsTreeViewMetrics {
    pub const fn for_platform(platform: ZsTreePlatformStyle) -> Self {
        match platform {
            ZsTreePlatformStyle::Windows => Self {
                row_height: Dp::new(32.0),
                depth_indent: Dp::new(20.0),
                disclosure_column: Dp::new(24.0),
                disclosure_size: Dp::new(12.0),
                icon_size: Dp::new(16.0),
                leading_padding: Dp::new(6.0),
                content_gap: Dp::new(6.0),
                row_radius: Dp::new(4.0),
            },
            ZsTreePlatformStyle::Macos => Self {
                row_height: Dp::new(22.0),
                depth_indent: Dp::new(16.0),
                disclosure_column: Dp::new(18.0),
                disclosure_size: Dp::new(10.0),
                icon_size: Dp::new(16.0),
                leading_padding: Dp::new(4.0),
                content_gap: Dp::new(4.0),
                row_radius: Dp::new(4.0),
            },
            ZsTreePlatformStyle::Gtk => Self {
                row_height: Dp::new(34.0),
                depth_indent: Dp::new(24.0),
                disclosure_column: Dp::new(24.0),
                disclosure_size: Dp::new(12.0),
                icon_size: Dp::new(16.0),
                leading_padding: Dp::new(6.0),
                content_gap: Dp::new(6.0),
                row_radius: Dp::new(6.0),
            },
        }
    }
}

#[cfg(feature = "tree")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeRowRenderPlan {
    pub node: crate::ZsTreeNodeId,
    pub parent: Option<crate::ZsTreeNodeId>,
    pub depth: usize,
    pub label: String,
    pub icon: Option<ZsIcon>,
    pub expandable: bool,
    pub expanded: bool,
    pub selected: bool,
    pub bounds: Rect,
    pub disclosure_bounds: Option<Rect>,
    pub icon_bounds: Option<Rect>,
    pub label_bounds: Rect,
}

#[cfg(feature = "tree")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeViewRenderPlan {
    pub bounds: Rect,
    pub rows: Vec<ZsTreeRowRenderPlan>,
    pub row_radius: i32,
    pub platform: ZsTreePlatformStyle,
}

#[cfg(feature = "tree")]
pub fn zs_tree_view_render_plan(
    bounds: Rect,
    roots: &[crate::ZsTreeNode],
    expanded: &BTreeSet<crate::ZsTreeNodeId>,
    selected: Option<crate::ZsTreeNodeId>,
    platform: ZsTreePlatformStyle,
    dpi: Dpi,
) -> ZsTreeViewRenderPlan {
    let metrics = ZsTreeViewMetrics::for_platform(platform);
    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let depth_indent = metrics.depth_indent.to_px(dpi).round_i32().max(1);
    let disclosure_column = metrics.disclosure_column.to_px(dpi).round_i32().max(1);
    let disclosure_size = metrics.disclosure_size.to_px(dpi).round_i32().max(1);
    let icon_size = metrics.icon_size.to_px(dpi).round_i32().max(1);
    let leading_padding = metrics.leading_padding.to_px(dpi).round_i32().max(0);
    let content_gap = metrics.content_gap.to_px(dpi).round_i32().max(0);
    let rows = crate::tree::visible_tree_nodes(roots, expanded)
        .into_iter()
        .enumerate()
        .map(|(index, visible)| {
            let row_y = bounds.y.saturating_add(
                row_height.saturating_mul(i32::try_from(index).unwrap_or(i32::MAX)),
            );
            let row = Rect {
                x: bounds.x,
                y: row_y,
                width: bounds.width,
                height: row_height,
            };
            let depth = i32::try_from(visible.depth).unwrap_or(i32::MAX);
            let disclosure_x = row
                .x
                .saturating_add(leading_padding)
                .saturating_add(depth_indent.saturating_mul(depth));
            let disclosure_slot = Rect {
                x: disclosure_x,
                y: row.y,
                width: disclosure_column,
                height: row.height,
            };
            let center_in = |slot: Rect, size: i32| Rect {
                x: slot.x.saturating_add((slot.width.saturating_sub(size)) / 2),
                y: slot
                    .y
                    .saturating_add((slot.height.saturating_sub(size)) / 2),
                width: size,
                height: size,
            };
            let content_x = disclosure_slot
                .x
                .saturating_add(disclosure_slot.width)
                .saturating_add(content_gap);
            let icon_bounds = visible.node.node_icon().map(|_| {
                center_in(
                    Rect {
                        x: content_x,
                        y: row.y,
                        width: icon_size,
                        height: row.height,
                    },
                    icon_size,
                )
            });
            let label_x = content_x.saturating_add(if icon_bounds.is_some() {
                icon_size.saturating_add(content_gap)
            } else {
                0
            });
            ZsTreeRowRenderPlan {
                node: visible.node.id(),
                parent: visible.parent,
                depth: visible.depth,
                label: visible.node.label().to_string(),
                icon: visible.node.node_icon(),
                expandable: visible.node.is_expandable(),
                expanded: visible.expanded,
                selected: selected == Some(visible.node.id()),
                bounds: row,
                disclosure_bounds: visible
                    .node
                    .is_expandable()
                    .then(|| center_in(disclosure_slot, disclosure_size)),
                icon_bounds,
                label_bounds: Rect {
                    x: label_x,
                    y: row.y,
                    width: row
                        .x
                        .saturating_add(row.width)
                        .saturating_sub(label_x)
                        .saturating_sub(leading_padding)
                        .max(0),
                    height: row.height,
                },
            }
        })
        .collect();
    ZsTreeViewRenderPlan {
        bounds,
        rows,
        row_radius: metrics.row_radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "tree")]
pub fn zs_tree_view_native_draw_plan(plan: &ZsTreeViewRenderPlan) -> NativeDrawPlan {
    let mut commands = vec![
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: None,
            radius: plan.row_radius,
        },
        NativeDrawCommand::PushClip { rect: plan.bounds },
    ];
    for row in &plan.rows {
        if row.selected {
            let fill = match plan.platform {
                ZsTreePlatformStyle::Macos => NativeDrawFill::Role(ColorRole::Accent),
                ZsTreePlatformStyle::Windows => NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                ZsTreePlatformStyle::Gtk => NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 48,
                },
            };
            commands.push(NativeDrawCommand::RoundFill {
                rect: row.bounds,
                fill,
                radius: plan.row_radius,
            });
        }
        let foreground = if row.selected && plan.platform == ZsTreePlatformStyle::Macos {
            ColorRole::AccentText
        } else {
            ColorRole::PrimaryText
        };
        if let Some(bounds) = row.disclosure_bounds {
            commands.push(NativeDrawCommand::Icon(
                NativeDrawIconCommand::new(
                    if row.expanded {
                        ZsIcon::ChevronDown
                    } else {
                        ZsIcon::ChevronRight
                    },
                    bounds,
                    NativeIconColorMode::ThemeAware,
                )
                .with_color(foreground),
            ));
        }
        if let (Some(icon), Some(bounds)) = (row.icon, row.icon_bounds) {
            commands.push(NativeDrawCommand::Icon(
                NativeDrawIconCommand::new(icon, bounds, NativeIconColorMode::ThemeAware)
                    .with_color(foreground),
            ));
        }
        let mut style = SemanticTextStyle::body();
        style.color = foreground;
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            row.label.clone(),
            row.label_bounds,
            style,
        )));
    }
    commands.push(NativeDrawCommand::PopClip);
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTablePlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "table")]
impl ZsTablePlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTableMetrics {
    pub header_height: Dp,
    pub row_height: Dp,
    pub horizontal_padding: Dp,
    pub sort_icon_size: Dp,
    pub radius: Dp,
    pub separator_width: Dp,
}

#[cfg(feature = "table")]
impl ZsTableMetrics {
    pub const fn for_platform(platform: ZsTablePlatformStyle) -> Self {
        match platform {
            ZsTablePlatformStyle::Windows => Self {
                header_height: Dp::new(36.0),
                row_height: Dp::new(32.0),
                horizontal_padding: Dp::new(12.0),
                sort_icon_size: Dp::new(12.0),
                radius: Dp::new(4.0),
                separator_width: Dp::new(1.0),
            },
            ZsTablePlatformStyle::Macos => Self {
                header_height: Dp::new(24.0),
                row_height: Dp::new(24.0),
                horizontal_padding: Dp::new(8.0),
                sort_icon_size: Dp::new(10.0),
                radius: Dp::new(5.0),
                separator_width: Dp::new(1.0),
            },
            ZsTablePlatformStyle::Gtk => Self {
                header_height: Dp::new(36.0),
                row_height: Dp::new(34.0),
                horizontal_padding: Dp::new(12.0),
                sort_icon_size: Dp::new(12.0),
                radius: Dp::new(6.0),
                separator_width: Dp::new(1.0),
            },
        }
    }
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableColumnRenderPlan {
    pub column: crate::ZsTableColumnId,
    pub header: String,
    pub alignment: HorizontalAlign,
    pub sortable: bool,
    pub sort: Option<crate::ZsTableSortDirection>,
    pub bounds: Rect,
    pub label_bounds: Rect,
    pub sort_icon_bounds: Option<Rect>,
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableCellRenderPlan {
    pub column: crate::ZsTableColumnId,
    pub value: String,
    pub alignment: HorizontalAlign,
    pub bounds: Rect,
    pub text_bounds: Rect,
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableRowRenderPlan {
    pub row: crate::ZsTableRowId,
    pub selected: bool,
    pub bounds: Rect,
    pub cells: Vec<ZsTableCellRenderPlan>,
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTableRenderPlan {
    pub bounds: Rect,
    pub header_bounds: Rect,
    pub columns: Vec<ZsTableColumnRenderPlan>,
    pub rows: Vec<ZsTableRowRenderPlan>,
    pub radius: i32,
    pub separator_width: i32,
    pub platform: ZsTablePlatformStyle,
}

#[cfg(feature = "table")]
fn table_column_widths(
    columns: &[&crate::ZsTableColumn],
    available_width: i32,
    dpi: Dpi,
) -> Vec<i32> {
    let available_width = available_width.max(0);
    let fixed_total = columns
        .iter()
        .map(|column| match column.width() {
            crate::ZsTableColumnWidth::Fixed(width) => width.to_px(dpi).round_i32().max(0),
            crate::ZsTableColumnWidth::Fill(_) => 0,
        })
        .fold(0_i32, i32::saturating_add);
    let fill_total = columns
        .iter()
        .map(|column| match column.width() {
            crate::ZsTableColumnWidth::Fixed(_) => 0_u32,
            crate::ZsTableColumnWidth::Fill(weight) => u32::from(weight.max(1)),
        })
        .fold(0_u32, u32::saturating_add);
    let fill_available = available_width.saturating_sub(fixed_total).max(0);
    let mut desired = columns
        .iter()
        .map(|column| match column.width() {
            crate::ZsTableColumnWidth::Fixed(width) => width.to_px(dpi).round_i32().max(0),
            crate::ZsTableColumnWidth::Fill(weight) if fill_total > 0 => {
                let portion = i64::from(fill_available).saturating_mul(i64::from(weight.max(1)))
                    / i64::from(fill_total);
                i32::try_from(portion).unwrap_or(i32::MAX)
            }
            crate::ZsTableColumnWidth::Fill(_) => 0,
        })
        .collect::<Vec<_>>();
    let desired_total = desired.iter().copied().fold(0_i32, i32::saturating_add);
    if desired_total < available_width {
        if let Some(last) = desired.last_mut() {
            *last = last.saturating_add(available_width - desired_total);
        }
    }
    let mut remaining = available_width;
    for width in &mut desired {
        *width = (*width).min(remaining).max(0);
        remaining = remaining.saturating_sub(*width);
    }
    desired
}

#[cfg(feature = "table")]
pub fn zs_table_render_plan(
    bounds: Rect,
    columns: &[crate::ZsTableColumn],
    rows: &[crate::ZsTableRow],
    selected: Option<crate::ZsTableRowId>,
    sort: Option<crate::ZsTableSort>,
    platform: ZsTablePlatformStyle,
    dpi: Dpi,
) -> ZsTableRenderPlan {
    let metrics = ZsTableMetrics::for_platform(platform);
    let header_height = metrics.header_height.to_px(dpi).round_i32().max(1);
    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let padding = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let sort_icon_size = metrics.sort_icon_size.to_px(dpi).round_i32().max(1);
    let unique_columns = crate::table::unique_table_columns(columns);
    let widths = table_column_widths(&unique_columns, bounds.width, dpi);
    let header_bounds = Rect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: header_height,
    };
    let mut x = bounds.x;
    let columns = unique_columns
        .iter()
        .zip(widths.iter().copied())
        .map(|(column, width)| {
            let column_bounds = Rect {
                x,
                y: bounds.y,
                width,
                height: header_height,
            };
            x = x.saturating_add(width);
            let active_sort = sort
                .filter(|sort| sort.column == column.id())
                .map(|sort| sort.direction);
            let sort_icon_bounds = active_sort.map(|_| Rect {
                x: column_bounds
                    .x
                    .saturating_add(column_bounds.width)
                    .saturating_sub(padding)
                    .saturating_sub(sort_icon_size),
                y: column_bounds
                    .y
                    .saturating_add((column_bounds.height.saturating_sub(sort_icon_size)) / 2),
                width: sort_icon_size,
                height: sort_icon_size,
            });
            let trailing = padding.saturating_add(if sort_icon_bounds.is_some() {
                sort_icon_size.saturating_add(padding / 2)
            } else {
                0
            });
            ZsTableColumnRenderPlan {
                column: column.id(),
                header: column.header().to_string(),
                alignment: column.column_alignment(),
                sortable: column.is_sortable(),
                sort: active_sort,
                bounds: column_bounds,
                label_bounds: Rect {
                    x: column_bounds.x.saturating_add(padding),
                    y: column_bounds.y,
                    width: column_bounds
                        .width
                        .saturating_sub(padding)
                        .saturating_sub(trailing)
                        .max(0),
                    height: column_bounds.height,
                },
                sort_icon_bounds,
            }
        })
        .collect::<Vec<_>>();
    let rows = crate::table::unique_table_rows(rows)
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            let row_bounds = Rect {
                x: bounds.x,
                y: bounds.y.saturating_add(header_height).saturating_add(
                    row_height.saturating_mul(i32::try_from(index).unwrap_or(i32::MAX)),
                ),
                width: bounds.width,
                height: row_height,
            };
            let cells = columns
                .iter()
                .enumerate()
                .map(|(column_index, column)| {
                    let cell_bounds = Rect {
                        x: column.bounds.x,
                        y: row_bounds.y,
                        width: column.bounds.width,
                        height: row_bounds.height,
                    };
                    ZsTableCellRenderPlan {
                        column: column.column,
                        value: row.cell(column_index).to_string(),
                        alignment: column.alignment,
                        bounds: cell_bounds,
                        text_bounds: inset_row_text(cell_bounds, padding),
                    }
                })
                .collect();
            ZsTableRowRenderPlan {
                row: row.id(),
                selected: selected == Some(row.id()),
                bounds: row_bounds,
                cells,
            }
        })
        .collect();
    ZsTableRenderPlan {
        bounds,
        header_bounds,
        columns,
        rows,
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        separator_width: metrics.separator_width.to_px(dpi).round_i32().max(1),
        platform,
    }
}

#[cfg(feature = "table")]
pub fn zs_table_native_draw_plan(plan: &ZsTableRenderPlan) -> NativeDrawPlan {
    let mut commands = vec![
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.radius,
        },
        NativeDrawCommand::PushClip { rect: plan.bounds },
        NativeDrawCommand::FillRect {
            rect: plan.header_bounds,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        },
    ];
    for column in &plan.columns {
        let mut style = SemanticTextStyle::body();
        style.weight = TextWeight::Semibold;
        style.horizontal_align = column.alignment;
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            column.header.clone(),
            column.label_bounds,
            style,
        )));
        if let (Some(direction), Some(bounds)) = (column.sort, column.sort_icon_bounds) {
            commands.push(NativeDrawCommand::Icon(
                NativeDrawIconCommand::new(
                    match direction {
                        crate::ZsTableSortDirection::Ascending => ZsIcon::ChevronUp,
                        crate::ZsTableSortDirection::Descending => ZsIcon::ChevronDown,
                    },
                    bounds,
                    NativeIconColorMode::ThemeAware,
                )
                .with_color(ColorRole::PrimaryText),
            ));
        }
        let separator_x = column
            .bounds
            .x
            .saturating_add(column.bounds.width)
            .saturating_sub(plan.separator_width);
        commands.push(NativeDrawCommand::FillRect {
            rect: Rect {
                x: separator_x,
                y: plan.bounds.y,
                width: plan.separator_width,
                height: plan.bounds.height,
            },
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Border,
                alpha: 180,
            },
        });
    }
    commands.push(NativeDrawCommand::FillRect {
        rect: Rect {
            x: plan.header_bounds.x,
            y: plan
                .header_bounds
                .y
                .saturating_add(plan.header_bounds.height)
                .saturating_sub(plan.separator_width),
            width: plan.header_bounds.width,
            height: plan.separator_width,
        },
        fill: NativeDrawFill::Role(ColorRole::Border),
    });
    for row in &plan.rows {
        if row.selected {
            let fill = match plan.platform {
                ZsTablePlatformStyle::Macos => NativeDrawFill::Role(ColorRole::Accent),
                ZsTablePlatformStyle::Windows => NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                ZsTablePlatformStyle::Gtk => NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 48,
                },
            };
            commands.push(NativeDrawCommand::FillRect {
                rect: row.bounds,
                fill,
            });
        }
        let foreground = if row.selected && plan.platform == ZsTablePlatformStyle::Macos {
            ColorRole::AccentText
        } else {
            ColorRole::PrimaryText
        };
        for cell in &row.cells {
            let mut style = SemanticTextStyle::body();
            style.color = foreground;
            style.horizontal_align = cell.alignment;
            commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                cell.value.clone(),
                cell.text_bounds,
                style,
            )));
        }
        commands.push(NativeDrawCommand::FillRect {
            rect: Rect {
                x: row.bounds.x,
                y: row
                    .bounds
                    .y
                    .saturating_add(row.bounds.height)
                    .saturating_sub(plan.separator_width),
                width: row.bounds.width,
                height: plan.separator_width,
            },
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Border,
                alpha: 128,
            },
        });
    }
    commands.push(NativeDrawCommand::PopClip);
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "combo")]
/// Matches WinUI's default `ComboBoxPopupMaxNumberOfItems` resource.
pub const ZS_COMBO_BOX_MAX_VISIBLE_OPTIONS: usize = 15;

#[cfg(feature = "combo")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsComboBoxRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub first_visible_option: usize,
    pub option_rows: Vec<Rect>,
    pub radius: i32,
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_render_plan(
    bounds: Rect,
    option_count: usize,
    expanded: bool,
    dpi: Dpi,
) -> ZsComboBoxRenderPlan {
    zs_combo_box_render_plan_impl(bounds, option_count, None, Some(0), expanded, dpi, None)
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_render_plan_in_viewport(
    bounds: Rect,
    option_count: usize,
    expanded: bool,
    dpi: Dpi,
    viewport: Rect,
) -> ZsComboBoxRenderPlan {
    zs_combo_box_render_plan_impl(
        bounds,
        option_count,
        None,
        Some(0),
        expanded,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_render_plan_with_scroll(
    bounds: Rect,
    option_count: usize,
    selected_index: Option<usize>,
    first_visible_option: Option<usize>,
    expanded: bool,
    dpi: Dpi,
) -> ZsComboBoxRenderPlan {
    zs_combo_box_render_plan_impl(
        bounds,
        option_count,
        selected_index,
        first_visible_option,
        expanded,
        dpi,
        None,
    )
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_render_plan_in_viewport_with_scroll(
    bounds: Rect,
    option_count: usize,
    selected_index: Option<usize>,
    first_visible_option: Option<usize>,
    expanded: bool,
    dpi: Dpi,
    viewport: Rect,
) -> ZsComboBoxRenderPlan {
    zs_combo_box_render_plan_impl(
        bounds,
        option_count,
        selected_index,
        first_visible_option,
        expanded,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "combo")]
fn zs_combo_box_render_plan_impl(
    bounds: Rect,
    option_count: usize,
    selected_index: Option<usize>,
    first_visible_option: Option<usize>,
    expanded: bool,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsComboBoxRenderPlan {
    let horizontal_padding = scale(12, dpi).min(bounds.width.max(1) / 3).max(1);
    let icon_size = scale(16, dpi).min(bounds.height.max(1)).max(1);
    let icon_right = bounds
        .x
        .saturating_add(bounds.width)
        .saturating_sub(horizontal_padding);
    let icon_bounds = Rect {
        x: icon_right.saturating_sub(icon_size),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(icon_size)) / 2),
        width: icon_size,
        height: icon_size,
    };
    let text_right = icon_bounds.x.saturating_sub(scale(8, dpi));
    let text_x = bounds.x.saturating_add(horizontal_padding);
    let text_bounds = Rect {
        x: text_x,
        y: bounds.y,
        width: text_right.saturating_sub(text_x).max(0),
        height: bounds.height,
    };
    let row_height = bounds.height.max(scale(32, dpi)).max(1);
    let popup_gap = scale(4, dpi);
    let visible_option_count =
        combo_visible_option_count(bounds, option_count, row_height, popup_gap, viewport);
    let maximum_first_visible = option_count.saturating_sub(visible_option_count);
    let first_visible_option = first_visible_option.map_or_else(
        || {
            selected_index
                .filter(|index| *index < option_count)
                .map(|index| {
                    index
                        .saturating_add(1)
                        .saturating_sub(visible_option_count)
                        .min(maximum_first_visible)
                })
                .unwrap_or_default()
        },
        |index| index.min(maximum_first_visible),
    );
    let placed_popup = (expanded && option_count > 0).then(|| {
        place_popup(
            bounds,
            bounds.width.max(1),
            row_height.saturating_mul(visible_option_count.min(i32::MAX as usize) as i32),
            popup_gap,
            viewport,
        )
    });
    let popup = placed_popup.map(|placed| placed.bounds);
    let option_rows = popup
        .map(|popup| {
            (0..visible_option_count)
                .map(|index| Rect {
                    x: popup.x,
                    y: popup.y.saturating_add(
                        row_height.saturating_mul(i32::try_from(index).unwrap_or(i32::MAX)),
                    ),
                    width: popup.width,
                    height: row_height,
                })
                .collect()
        })
        .unwrap_or_default();
    ZsComboBoxRenderPlan {
        bounds,
        text_bounds,
        icon_bounds,
        popup,
        popup_placement: placed_popup.map(|placed| placed.placement),
        first_visible_option,
        option_rows,
        radius: scale(6, dpi),
    }
}

#[cfg(feature = "combo")]
fn combo_visible_option_count(
    anchor: Rect,
    option_count: usize,
    row_height: i32,
    gap: i32,
    viewport: Option<Rect>,
) -> usize {
    let capped_count = option_count.min(ZS_COMBO_BOX_MAX_VISIBLE_OPTIONS);
    let Some(viewport) = viewport.filter(|viewport| viewport.width > 0 && viewport.height > 0)
    else {
        return capped_count;
    };
    let viewport_bottom = viewport.y.saturating_add(viewport.height);
    let below_y = anchor.y.saturating_add(anchor.height).saturating_add(gap);
    let above_bottom = anchor.y.saturating_sub(gap);
    let available_below = viewport_bottom.saturating_sub(below_y).max(0);
    let available_above = above_bottom.saturating_sub(viewport.y).max(0);
    let available_rows = available_below.max(available_above) / row_height.max(1);
    capped_count.min(available_rows.max(1) as usize)
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_header_native_draw_plan(
    plan: &ZsComboBoxRenderPlan,
    selected_text: Option<&str>,
    placeholder: Option<&str>,
) -> NativeDrawPlan {
    let label = selected_text.or(placeholder).unwrap_or_default();
    let mut text_style = SemanticTextStyle::body();
    if selected_text.is_none() {
        text_style.color = ColorRole::SecondaryText;
    }
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            plan.text_bounds,
            text_style,
        )),
        NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                ZsIcon::ChevronDown,
                plan.icon_bounds,
                NativeIconColorMode::ThemeAware,
            )
            .with_color(ColorRole::SecondaryText),
        ),
    ])
}

#[cfg(feature = "combo")]
pub fn zs_combo_box_popup_native_draw_plan(
    plan: &ZsComboBoxRenderPlan,
    options: &[String],
    selected: Option<usize>,
    dpi: Dpi,
) -> NativeDrawPlan {
    let Some(popup) = plan.popup else {
        return NativeDrawPlan::default();
    };
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: popup,
        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.radius,
    }];
    let padding = scale(12, dpi);
    for ((index, label), row) in options
        .iter()
        .enumerate()
        .skip(plan.first_visible_option)
        .zip(&plan.option_rows)
    {
        if selected == Some(index) {
            commands.push(NativeDrawCommand::RoundFill {
                rect: *row,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                radius: plan.radius,
            });
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            Rect {
                x: row.x.saturating_add(padding),
                y: row.y,
                width: row.width.saturating_sub(padding.saturating_mul(2)),
                height: row.height,
            },
            SemanticTextStyle::body(),
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "date-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDatePickerDayCell {
    pub bounds: Rect,
    pub date: ZsDate,
    pub in_display_month: bool,
    pub enabled: bool,
    pub selected: bool,
    pub today: bool,
}

#[cfg(feature = "date-picker")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDatePickerRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub month_label_bounds: Option<Rect>,
    pub previous_button: Option<Rect>,
    pub next_button: Option<Rect>,
    pub weekday_cells: Vec<Rect>,
    pub day_cells: Vec<ZsDatePickerDayCell>,
    pub control_radius: i32,
    pub overlay_radius: i32,
}

/// Computes the self-drawn CalendarDatePicker geometry.
///
/// The closed-field metrics follow the WinUI 3 default template: a 32-DIP
/// control, 32-DIP glyph column, 12-DIP text inset and 4-DIP control radius.
/// The popup uses the CalendarView header/weekday/day rhythm and the 8-DIP
/// overlay radius from the same template.
#[cfg(feature = "date-picker")]
pub fn zs_date_picker_render_plan(
    bounds: Rect,
    value: ZsDate,
    visible_month: ZsDate,
    minimum: ZsDate,
    maximum: ZsDate,
    expanded: bool,
    dpi: Dpi,
) -> ZsDatePickerRenderPlan {
    zs_date_picker_render_plan_impl(
        bounds,
        value,
        visible_month,
        minimum,
        maximum,
        None,
        expanded,
        dpi,
        None,
    )
}

#[cfg(feature = "date-picker")]
#[allow(clippy::too_many_arguments)]
pub fn zs_date_picker_render_plan_with_today(
    bounds: Rect,
    value: ZsDate,
    visible_month: ZsDate,
    minimum: ZsDate,
    maximum: ZsDate,
    today: Option<ZsDate>,
    expanded: bool,
    dpi: Dpi,
) -> ZsDatePickerRenderPlan {
    zs_date_picker_render_plan_impl(
        bounds,
        value,
        visible_month,
        minimum,
        maximum,
        today,
        expanded,
        dpi,
        None,
    )
}

#[cfg(feature = "date-picker")]
#[allow(clippy::too_many_arguments)]
pub fn zs_date_picker_render_plan_in_viewport(
    bounds: Rect,
    value: ZsDate,
    visible_month: ZsDate,
    minimum: ZsDate,
    maximum: ZsDate,
    expanded: bool,
    dpi: Dpi,
    viewport: Rect,
) -> ZsDatePickerRenderPlan {
    zs_date_picker_render_plan_impl(
        bounds,
        value,
        visible_month,
        minimum,
        maximum,
        None,
        expanded,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "date-picker")]
#[allow(clippy::too_many_arguments)]
pub fn zs_date_picker_render_plan_in_viewport_with_today(
    bounds: Rect,
    value: ZsDate,
    visible_month: ZsDate,
    minimum: ZsDate,
    maximum: ZsDate,
    today: Option<ZsDate>,
    expanded: bool,
    dpi: Dpi,
    viewport: Rect,
) -> ZsDatePickerRenderPlan {
    zs_date_picker_render_plan_impl(
        bounds,
        value,
        visible_month,
        minimum,
        maximum,
        today,
        expanded,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "date-picker")]
#[allow(clippy::too_many_arguments)]
fn zs_date_picker_render_plan_impl(
    bounds: Rect,
    value: ZsDate,
    visible_month: ZsDate,
    minimum: ZsDate,
    maximum: ZsDate,
    today: Option<ZsDate>,
    expanded: bool,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsDatePickerRenderPlan {
    let (minimum, maximum) = if minimum <= maximum {
        (minimum, maximum)
    } else {
        (maximum, minimum)
    };
    let visible_month = visible_month.first_day_of_month();
    let icon_column_width = scale(32, dpi).min(bounds.width.max(1));
    let text_padding = scale(12, dpi).min(bounds.width.max(1) / 3).max(1);
    let icon_size = scale(12, dpi).min(bounds.height.max(1)).max(1);
    let icon_column_x = bounds
        .x
        .saturating_add(bounds.width)
        .saturating_sub(icon_column_width);
    let icon_bounds = Rect {
        x: icon_column_x.saturating_add((icon_column_width.saturating_sub(icon_size)) / 2),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(icon_size)) / 2),
        width: icon_size,
        height: icon_size,
    };
    let text_x = bounds.x.saturating_add(text_padding);
    let text_bounds = Rect {
        x: text_x,
        y: bounds.y,
        width: icon_column_x.saturating_sub(text_x).max(0),
        height: bounds.height,
    };

    // CalendarView's 7 columns are 40-DIP day items with 1-DIP margins;
    // TemplateSettings.MinViewWidth plus the outer border resolves to 296 DIPs.
    let popup_width = scale(296, dpi);
    let popup_gap = scale(4, dpi);
    let border_inset = scale(1, dpi);
    let header_height = scale(40, dpi);
    let weekday_height = scale(38, dpi);
    let day_height = scale(42, dpi);
    let popup_height = header_height
        .saturating_add(weekday_height)
        .saturating_add(day_height.saturating_mul(6))
        .saturating_add(border_inset.saturating_mul(2));
    let placed_popup =
        expanded.then(|| place_popup(bounds, popup_width, popup_height, popup_gap, viewport));
    let popup = placed_popup.map(|placed| placed.bounds);

    let mut month_label_bounds = None;
    let mut previous_button = None;
    let mut next_button = None;
    let mut weekday_cells = Vec::new();
    let mut day_cells = Vec::new();
    if let Some(popup) = popup {
        let content = Rect {
            x: popup.x.saturating_add(border_inset),
            y: popup.y.saturating_add(border_inset),
            width: popup.width.saturating_sub(border_inset.saturating_mul(2)),
            height: popup.height.saturating_sub(border_inset.saturating_mul(2)),
        };
        let navigation_width = content.width / 7;
        previous_button = Some(Rect {
            x: content
                .x
                .saturating_add(content.width.saturating_sub(navigation_width * 2)),
            y: content.y,
            width: navigation_width,
            height: header_height,
        });
        next_button = Some(Rect {
            x: content
                .x
                .saturating_add(content.width.saturating_sub(navigation_width)),
            y: content.y,
            width: navigation_width,
            height: header_height,
        });
        month_label_bounds = Some(Rect {
            x: content.x.saturating_add(scale(12, dpi)),
            y: content.y,
            width: content
                .width
                .saturating_sub(navigation_width * 2)
                .saturating_sub(scale(12, dpi)),
            height: header_height,
        });

        let column_left = |column: i32| content.x + content.width * column / 7;
        let column_right = |column: i32| content.x + content.width * (column + 1) / 7;
        for column in 0..7 {
            weekday_cells.push(Rect {
                x: column_left(column),
                y: content.y.saturating_add(header_height),
                width: column_right(column).saturating_sub(column_left(column)),
                height: weekday_height,
            });
        }

        let first = visible_month.add_days(-i32::from(visible_month.weekday_from_sunday()));
        for index in 0..42_i32 {
            let column = index % 7;
            let row = index / 7;
            let date = first.add_days(index);
            day_cells.push(ZsDatePickerDayCell {
                bounds: Rect {
                    x: column_left(column),
                    y: content
                        .y
                        .saturating_add(header_height)
                        .saturating_add(weekday_height)
                        .saturating_add(day_height.saturating_mul(row)),
                    width: column_right(column).saturating_sub(column_left(column)),
                    height: day_height,
                },
                date,
                in_display_month: date.year() == visible_month.year()
                    && date.month() == visible_month.month(),
                enabled: date >= minimum && date <= maximum,
                selected: date == value,
                today: today == Some(date),
            });
        }
    }

    ZsDatePickerRenderPlan {
        bounds,
        text_bounds,
        icon_bounds,
        popup,
        popup_placement: placed_popup.map(|placed| placed.placement),
        month_label_bounds,
        previous_button,
        next_button,
        weekday_cells,
        day_cells,
        control_radius: scale(4, dpi),
        overlay_radius: scale(8, dpi),
    }
}

#[cfg(feature = "date-picker")]
pub fn zs_date_picker_header_native_draw_plan(
    plan: &ZsDatePickerRenderPlan,
    value: ZsDate,
) -> NativeDrawPlan {
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill: NativeDrawFill::Role(ColorRole::Control),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.control_radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            value.iso_string(),
            plan.text_bounds,
            SemanticTextStyle::body(),
        )),
        NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                ZsIcon::Calendar,
                plan.icon_bounds,
                NativeIconColorMode::ThemeAware,
            )
            .with_color(ColorRole::PrimaryText),
        ),
    ])
}

#[cfg(feature = "date-picker")]
pub fn zs_date_picker_popup_native_draw_plan(
    plan: &ZsDatePickerRenderPlan,
    visible_month: ZsDate,
    dpi: Dpi,
) -> NativeDrawPlan {
    let Some(popup) = plan.popup else {
        return NativeDrawPlan::default();
    };
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: popup,
        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.overlay_radius,
    }];
    if let Some(bounds) = plan.month_label_bounds {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            format!("{:04} / {:02}", visible_month.year(), visible_month.month()),
            bounds,
            SemanticTextStyle {
                weight: TextWeight::Semibold,
                ..SemanticTextStyle::body()
            },
        )));
    }
    let navigation_icon_size = scale(12, dpi);
    for (bounds, icon) in [
        (plan.previous_button, ZsIcon::ChevronLeft),
        (plan.next_button, ZsIcon::ChevronRight),
    ] {
        if let Some(bounds) = bounds {
            let icon_bounds = Rect {
                x: bounds.x + (bounds.width - navigation_icon_size) / 2,
                y: bounds.y + (bounds.height - navigation_icon_size) / 2,
                width: navigation_icon_size,
                height: navigation_icon_size,
            };
            commands.push(NativeDrawCommand::Icon(
                NativeDrawIconCommand::new(icon, icon_bounds, NativeIconColorMode::ThemeAware)
                    .with_color(ColorRole::PrimaryText),
            ));
        }
    }

    let weekday_style = SemanticTextStyle {
        role: TextRole::Caption,
        weight: TextWeight::Semibold,
        horizontal_align: HorizontalAlign::Center,
        ..SemanticTextStyle::body()
    };
    for (label, bounds) in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
        .into_iter()
        .zip(&plan.weekday_cells)
    {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label,
            *bounds,
            weekday_style,
        )));
    }
    for cell in &plan.day_cells {
        let highlighted = cell.selected || (cell.today && cell.enabled);
        if highlighted {
            let diameter = scale(32, dpi)
                .min(cell.bounds.width)
                .min(cell.bounds.height);
            commands.push(NativeDrawCommand::RoundRect {
                rect: Rect {
                    x: cell.bounds.x + (cell.bounds.width - diameter) / 2,
                    y: cell.bounds.y + (cell.bounds.height - diameter) / 2,
                    width: diameter,
                    height: diameter,
                },
                fill: NativeDrawFill::Role(ColorRole::Accent),
                stroke: (cell.selected && cell.today)
                    .then_some(NativeDrawFill::Role(ColorRole::AccentText)),
                radius: diameter / 2,
            });
        }
        let color = if !cell.enabled {
            ColorRole::DisabledText
        } else if highlighted {
            ColorRole::AccentText
        } else if cell.in_display_month {
            ColorRole::PrimaryText
        } else {
            ColorRole::SecondaryText
        };
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            cell.date.day().to_string(),
            cell.bounds,
            SemanticTextStyle {
                color,
                horizontal_align: HorizontalAlign::Center,
                ..SemanticTextStyle::body()
            },
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTimePickerPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "time-picker")]
impl ZsTimePickerPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Gtk
        }
    }

    pub const fn default_clock(self) -> ZsClockFormat {
        match self {
            Self::Windows => ZsClockFormat::TwelveHour,
            Self::Macos | Self::Gtk => ZsClockFormat::TwentyFourHour,
        }
    }
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsTimePickerMetrics {
    pub popup_width: Dp,
    pub row_height: Dp,
    pub visible_rows: usize,
    pub text_padding: Dp,
    pub icon_column_width: Dp,
    pub popup_gap: Dp,
    pub control_radius: Dp,
    pub overlay_radius: Dp,
}

#[cfg(feature = "time-picker")]
impl ZsTimePickerMetrics {
    pub const fn for_platform(platform: ZsTimePickerPlatformStyle) -> Self {
        match platform {
            ZsTimePickerPlatformStyle::Windows => Self {
                popup_width: Dp::new(280.0),
                row_height: Dp::new(40.0),
                visible_rows: 5,
                text_padding: Dp::new(12.0),
                icon_column_width: Dp::new(32.0),
                popup_gap: Dp::new(4.0),
                control_radius: Dp::new(4.0),
                overlay_radius: Dp::new(8.0),
            },
            ZsTimePickerPlatformStyle::Macos => Self {
                popup_width: Dp::new(216.0),
                row_height: Dp::new(30.0),
                visible_rows: 3,
                text_padding: Dp::new(10.0),
                icon_column_width: Dp::new(26.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(6.0),
                overlay_radius: Dp::new(10.0),
            },
            ZsTimePickerPlatformStyle::Gtk => Self {
                popup_width: Dp::new(240.0),
                row_height: Dp::new(36.0),
                visible_rows: 3,
                text_padding: Dp::new(12.0),
                icon_column_width: Dp::new(34.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(6.0),
                overlay_radius: Dp::new(12.0),
            },
        }
    }
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsTimePickerSegment {
    Hour,
    Minute,
    Period,
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTimePickerChoice {
    pub bounds: Rect,
    pub value: ZsTime,
    pub segment: ZsTimePickerSegment,
    pub label: String,
    pub selected: bool,
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTimePickerRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub column_bounds: Vec<Rect>,
    pub choices: Vec<ZsTimePickerChoice>,
    pub platform: ZsTimePickerPlatformStyle,
    pub clock: ZsClockFormat,
    pub control_radius: i32,
    pub overlay_radius: i32,
}

#[cfg(feature = "time-picker")]
pub fn zs_time_picker_render_plan(
    bounds: Rect,
    value: ZsTime,
    increment: ZsMinuteIncrement,
    clock: ZsClockFormat,
    expanded: bool,
    platform: ZsTimePickerPlatformStyle,
    dpi: Dpi,
) -> ZsTimePickerRenderPlan {
    zs_time_picker_render_plan_impl(
        bounds, value, increment, clock, expanded, platform, dpi, None,
    )
}

#[cfg(feature = "time-picker")]
#[allow(clippy::too_many_arguments)]
pub fn zs_time_picker_render_plan_in_viewport(
    bounds: Rect,
    value: ZsTime,
    increment: ZsMinuteIncrement,
    clock: ZsClockFormat,
    expanded: bool,
    platform: ZsTimePickerPlatformStyle,
    dpi: Dpi,
    viewport: Rect,
) -> ZsTimePickerRenderPlan {
    zs_time_picker_render_plan_impl(
        bounds,
        value,
        increment,
        clock,
        expanded,
        platform,
        dpi,
        Some(viewport),
    )
}

#[cfg(feature = "time-picker")]
#[allow(clippy::too_many_arguments)]
fn zs_time_picker_render_plan_impl(
    bounds: Rect,
    value: ZsTime,
    increment: ZsMinuteIncrement,
    clock: ZsClockFormat,
    expanded: bool,
    platform: ZsTimePickerPlatformStyle,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsTimePickerRenderPlan {
    let value = value.snap(increment);
    let metrics = ZsTimePickerMetrics::for_platform(platform);
    let icon_column_width = metrics
        .icon_column_width
        .to_px(dpi)
        .round_i32()
        .max(1)
        .min(bounds.width.max(1));
    let text_padding = metrics
        .text_padding
        .to_px(dpi)
        .round_i32()
        .max(1)
        .min(bounds.width.max(1) / 3);
    let icon_size = scale(12, dpi).min(bounds.height.max(1)).max(1);
    let icon_column_x = bounds
        .x
        .saturating_add(bounds.width)
        .saturating_sub(icon_column_width);
    let icon_bounds = Rect {
        x: icon_column_x.saturating_add((icon_column_width.saturating_sub(icon_size)) / 2),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(icon_size)) / 2),
        width: icon_size,
        height: icon_size,
    };
    let text_x = bounds.x.saturating_add(text_padding);
    let text_bounds = Rect {
        x: text_x,
        y: bounds.y,
        width: icon_column_x.saturating_sub(text_x).max(0),
        height: bounds.height,
    };

    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let border_inset = scale(1, dpi);
    let popup_height = row_height
        .saturating_mul(metrics.visible_rows.max(1) as i32)
        .saturating_add(border_inset.saturating_mul(2));
    let placed_popup = expanded.then(|| {
        place_popup(
            bounds,
            metrics.popup_width.to_px(dpi).round_i32().max(1),
            popup_height,
            metrics.popup_gap.to_px(dpi).round_i32().max(0),
            viewport,
        )
    });
    let popup = placed_popup.map(|placed| placed.bounds);
    let mut column_bounds = Vec::new();
    let mut choices = Vec::new();
    if let Some(popup) = popup {
        let content = Rect {
            x: popup.x.saturating_add(border_inset),
            y: popup.y.saturating_add(border_inset),
            width: popup.width.saturating_sub(border_inset.saturating_mul(2)),
            height: popup.height.saturating_sub(border_inset.saturating_mul(2)),
        };
        let column_count = if clock == ZsClockFormat::TwelveHour {
            3
        } else {
            2
        };
        let column_left = |column: i32| content.x + content.width * column / column_count;
        let column_right = |column: i32| content.x + content.width * (column + 1) / column_count;
        for column in 0..column_count {
            column_bounds.push(Rect {
                x: column_left(column),
                y: content.y,
                width: column_right(column).saturating_sub(column_left(column)),
                height: content.height,
            });
        }

        let middle = metrics.visible_rows as i32 / 2;
        for row in 0..metrics.visible_rows as i32 {
            let offset = row - middle;
            let hour = match clock {
                ZsClockFormat::TwentyFourHour => {
                    (i32::from(value.hour()) + offset).rem_euclid(24) as u8
                }
                ZsClockFormat::TwelveHour => {
                    let display_hour = match value.hour() % 12 {
                        0 => 12,
                        hour => hour,
                    };
                    let candidate = (i32::from(display_hour) - 1 + offset).rem_euclid(12) as u8 + 1;
                    candidate % 12 + if value.hour() >= 12 { 12 } else { 0 }
                }
            };
            let next = value.with_hour(hour).expect("rendered hour is valid");
            choices.push(ZsTimePickerChoice {
                bounds: Rect {
                    x: column_bounds[0].x,
                    y: content.y.saturating_add(row_height.saturating_mul(row)),
                    width: column_bounds[0].width,
                    height: row_height,
                },
                value: next,
                segment: ZsTimePickerSegment::Hour,
                label: match clock {
                    ZsClockFormat::TwentyFourHour => format!("{hour:02}"),
                    ZsClockFormat::TwelveHour => match hour % 12 {
                        0 => "12".to_string(),
                        hour => hour.to_string(),
                    },
                },
                selected: hour == value.hour(),
            });

            let minute = (i32::from(value.minute())
                + offset.saturating_mul(i32::from(increment.get())))
            .rem_euclid(60) as u8;
            let next = value.with_minute(minute).expect("rendered minute is valid");
            choices.push(ZsTimePickerChoice {
                bounds: Rect {
                    x: column_bounds[1].x,
                    y: content.y.saturating_add(row_height.saturating_mul(row)),
                    width: column_bounds[1].width,
                    height: row_height,
                },
                value: next,
                segment: ZsTimePickerSegment::Minute,
                label: format!("{minute:02}"),
                selected: minute == value.minute(),
            });
        }

        if clock == ZsClockFormat::TwelveHour {
            let period_column = column_bounds[2];
            let period_height = row_height.min(period_column.height / 2).max(1);
            let start_y = period_column
                .y
                .saturating_add((period_column.height.saturating_sub(period_height * 2)) / 2);
            for (index, afternoon) in [false, true].into_iter().enumerate() {
                let hour = value.hour() % 12 + if afternoon { 12 } else { 0 };
                choices.push(ZsTimePickerChoice {
                    bounds: Rect {
                        x: period_column.x,
                        y: start_y.saturating_add(period_height.saturating_mul(index as i32)),
                        width: period_column.width,
                        height: period_height,
                    },
                    value: value
                        .with_hour(hour)
                        .expect("rendered period hour is valid"),
                    segment: ZsTimePickerSegment::Period,
                    label: if afternoon { "PM" } else { "AM" }.to_string(),
                    selected: afternoon == (value.hour() >= 12),
                });
            }
        }
    }

    ZsTimePickerRenderPlan {
        bounds,
        text_bounds,
        icon_bounds,
        popup,
        popup_placement: placed_popup.map(|placed| placed.placement),
        column_bounds,
        choices,
        platform,
        clock,
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(1),
        overlay_radius: metrics.overlay_radius.to_px(dpi).round_i32().max(1),
    }
}

#[cfg(feature = "time-picker")]
pub fn zs_time_picker_header_native_draw_plan(
    plan: &ZsTimePickerRenderPlan,
    value: ZsTime,
) -> NativeDrawPlan {
    let fill = match plan.platform {
        ZsTimePickerPlatformStyle::Macos => NativeDrawFill::Role(ColorRole::Surface),
        ZsTimePickerPlatformStyle::Windows | ZsTimePickerPlatformStyle::Gtk => {
            NativeDrawFill::Role(ColorRole::Control)
        }
    };
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill,
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.control_radius,
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            value.format(plan.clock),
            plan.text_bounds,
            SemanticTextStyle::body(),
        )),
        NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                ZsIcon::ChevronDown,
                plan.icon_bounds,
                NativeIconColorMode::ThemeAware,
            )
            .with_color(ColorRole::PrimaryText),
        ),
    ])
}

#[cfg(feature = "time-picker")]
pub fn zs_time_picker_popup_native_draw_plan(plan: &ZsTimePickerRenderPlan) -> NativeDrawPlan {
    let Some(popup) = plan.popup else {
        return NativeDrawPlan::default();
    };
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: popup,
        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.overlay_radius,
    }];
    for column in plan.column_bounds.iter().skip(1) {
        commands.push(NativeDrawCommand::FillRect {
            rect: Rect {
                x: column.x,
                y: column.y.saturating_add(4),
                width: 1,
                height: column.height.saturating_sub(8).max(1),
            },
            fill: NativeDrawFill::Role(ColorRole::Border),
        });
    }
    for choice in &plan.choices {
        if choice.selected {
            let fill = match plan.platform {
                ZsTimePickerPlatformStyle::Macos => NativeDrawFill::Role(ColorRole::Accent),
                ZsTimePickerPlatformStyle::Windows => NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 48,
                },
                ZsTimePickerPlatformStyle::Gtk => NativeDrawFill::Role(ColorRole::Control),
            };
            commands.push(NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: choice.bounds.x.saturating_add(4),
                    y: choice.bounds.y.saturating_add(3),
                    width: choice.bounds.width.saturating_sub(8).max(1),
                    height: choice.bounds.height.saturating_sub(6).max(1),
                },
                fill,
                radius: plan.control_radius,
            });
        }
        let color = if choice.selected && plan.platform == ZsTimePickerPlatformStyle::Macos {
            ColorRole::AccentText
        } else if choice.selected {
            ColorRole::PrimaryText
        } else {
            ColorRole::SecondaryText
        };
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            &choice.label,
            choice.bounds,
            SemanticTextStyle {
                role: TextRole::Body,
                color,
                weight: if choice.selected {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Center,
                ..SemanticTextStyle::body()
            },
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(feature = "color-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsColorPickerPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "color-picker")]
impl ZsColorPickerPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "color-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsColorPickerMetrics {
    pub popup_width: Dp,
    pub popup_padding: Dp,
    pub preview_size: Dp,
    pub preview_gap: Dp,
    pub spectrum_height: Dp,
    pub spectrum_gap: Dp,
    pub hue_track_height: Dp,
    pub row_height: Dp,
    pub row_gap: Dp,
    pub label_width: Dp,
    pub value_width: Dp,
    pub track_height: Dp,
    pub thumb_width: Dp,
    pub popup_gap: Dp,
    pub control_radius: Dp,
    pub overlay_radius: Dp,
}

#[cfg(feature = "color-picker")]
impl ZsColorPickerMetrics {
    pub const fn for_platform(platform: ZsColorPickerPlatformStyle) -> Self {
        match platform {
            ZsColorPickerPlatformStyle::Windows => Self {
                popup_width: Dp::new(320.0),
                popup_padding: Dp::new(16.0),
                preview_size: Dp::new(48.0),
                preview_gap: Dp::new(10.0),
                // WinUI recommends a square spectrum of at least 256 px when
                // editable precision fields are not present. The 288-DP
                // content width keeps this first-pass picker above that
                // accuracy floor without introducing a native child control.
                spectrum_height: Dp::new(256.0),
                spectrum_gap: Dp::new(8.0),
                hue_track_height: Dp::new(10.0),
                row_height: Dp::new(28.0),
                row_gap: Dp::new(4.0),
                label_width: Dp::new(24.0),
                value_width: Dp::new(42.0),
                track_height: Dp::new(8.0),
                thumb_width: Dp::new(12.0),
                popup_gap: Dp::new(4.0),
                control_radius: Dp::new(4.0),
                overlay_radius: Dp::new(8.0),
            },
            ZsColorPickerPlatformStyle::Macos => Self {
                popup_width: Dp::new(264.0),
                popup_padding: Dp::new(12.0),
                preview_size: Dp::new(40.0),
                preview_gap: Dp::new(10.0),
                spectrum_height: Dp::new(0.0),
                spectrum_gap: Dp::new(0.0),
                hue_track_height: Dp::new(0.0),
                row_height: Dp::new(32.0),
                row_gap: Dp::new(3.0),
                label_width: Dp::new(20.0),
                value_width: Dp::new(36.0),
                track_height: Dp::new(6.0),
                thumb_width: Dp::new(10.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(6.0),
                overlay_radius: Dp::new(10.0),
            },
            ZsColorPickerPlatformStyle::Gtk => Self {
                popup_width: Dp::new(288.0),
                popup_padding: Dp::new(16.0),
                preview_size: Dp::new(44.0),
                preview_gap: Dp::new(12.0),
                spectrum_height: Dp::new(96.0),
                spectrum_gap: Dp::new(8.0),
                hue_track_height: Dp::new(10.0),
                row_height: Dp::new(36.0),
                row_gap: Dp::new(4.0),
                label_width: Dp::new(24.0),
                value_width: Dp::new(40.0),
                track_height: Dp::new(8.0),
                thumb_width: Dp::new(12.0),
                popup_gap: Dp::new(6.0),
                control_radius: Dp::new(6.0),
                overlay_radius: Dp::new(12.0),
            },
        }
    }
}

#[cfg(feature = "color-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsColorPickerChannelRenderPlan {
    pub channel: ZsColorChannel,
    pub bounds: Rect,
    pub label_bounds: Rect,
    pub track: Rect,
    pub thumb: Rect,
    pub value_bounds: Rect,
    pub active: bool,
}

#[cfg(feature = "color-picker")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsColorPickerRenderPlan {
    pub bounds: Rect,
    pub swatch_bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
    pub popup_placement: Option<ZsPopupPlacement>,
    pub spectrum_bounds: Option<Rect>,
    pub spectrum_thumb: Option<Rect>,
    pub hue_track: Option<Rect>,
    pub hue_thumb: Option<Rect>,
    pub preview_bounds: Option<Rect>,
    pub hex_bounds: Option<Rect>,
    pub channels: Vec<ZsColorPickerChannelRenderPlan>,
    pub platform: ZsColorPickerPlatformStyle,
    pub control_radius: i32,
    pub overlay_radius: i32,
}

/// Computes shared paint and hit geometry for the self-drawn color well and editor.
///
/// The closed control follows the platform color-well/button idiom. The expanded
/// editor keeps one application-owned RGBA value and exposes precise channel
/// sliders without creating or driving a native child control.
#[cfg(feature = "color-picker")]
pub fn zs_color_picker_render_plan(
    bounds: Rect,
    state: ZsColorPickerState,
    platform: ZsColorPickerPlatformStyle,
    dpi: Dpi,
) -> ZsColorPickerRenderPlan {
    zs_color_picker_render_plan_impl(bounds, state, platform, dpi, None)
}

#[cfg(feature = "color-picker")]
pub fn zs_color_picker_render_plan_in_viewport(
    bounds: Rect,
    state: ZsColorPickerState,
    platform: ZsColorPickerPlatformStyle,
    dpi: Dpi,
    viewport: Rect,
) -> ZsColorPickerRenderPlan {
    zs_color_picker_render_plan_impl(bounds, state, platform, dpi, Some(viewport))
}

#[cfg(feature = "color-picker")]
fn zs_color_picker_render_plan_impl(
    bounds: Rect,
    state: ZsColorPickerState,
    platform: ZsColorPickerPlatformStyle,
    dpi: Dpi,
    viewport: Option<Rect>,
) -> ZsColorPickerRenderPlan {
    let state = state.normalized();
    let metrics = ZsColorPickerMetrics::for_platform(platform);
    let inset = metrics.popup_padding.to_px(dpi).round_i32().max(1);
    let swatch_size = scale(
        match platform {
            ZsColorPickerPlatformStyle::Windows => 22,
            ZsColorPickerPlatformStyle::Macos => 18,
            ZsColorPickerPlatformStyle::Gtk => 20,
        },
        dpi,
    )
    .min(bounds.height.saturating_sub(scale(8, dpi)).max(1));
    let header_padding = scale(8, dpi);
    let icon_column = scale(28, dpi).min(bounds.width.max(1));
    let swatch_bounds = Rect {
        x: bounds.x.saturating_add(header_padding),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(swatch_size)) / 2),
        width: swatch_size,
        height: swatch_size,
    };
    let icon_size = scale(12, dpi).min(bounds.height.max(1));
    let icon_bounds = Rect {
        x: bounds
            .x
            .saturating_add(bounds.width)
            .saturating_sub(icon_column)
            .saturating_add((icon_column.saturating_sub(icon_size)) / 2),
        y: bounds
            .y
            .saturating_add((bounds.height.saturating_sub(icon_size)) / 2),
        width: icon_size,
        height: icon_size,
    };
    let text_x = swatch_bounds
        .x
        .saturating_add(swatch_bounds.width)
        .saturating_add(scale(8, dpi));
    let text_bounds = Rect {
        x: text_x,
        y: bounds.y,
        width: icon_bounds.x.saturating_sub(text_x).max(0),
        height: bounds.height,
    };

    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let row_gap = metrics.row_gap.to_px(dpi).round_i32().max(0);
    let preview_size = metrics.preview_size.to_px(dpi).round_i32().max(1);
    let preview_gap = metrics.preview_gap.to_px(dpi).round_i32().max(0);
    let spectrum_height = metrics.spectrum_height.to_px(dpi).round_i32().max(0);
    let spectrum_gap = metrics.spectrum_gap.to_px(dpi).round_i32().max(0);
    let hue_track_height = metrics.hue_track_height.to_px(dpi).round_i32().max(0);
    let spectrum_section_height = if spectrum_height > 0 && hue_track_height > 0 {
        spectrum_height
            .saturating_add(spectrum_gap)
            .saturating_add(hue_track_height)
            .saturating_add(spectrum_gap)
    } else {
        0
    };
    let channel_count = state.channels().len() as i32;
    let popup_height = inset
        .saturating_mul(2)
        .saturating_add(spectrum_section_height)
        .saturating_add(preview_size)
        .saturating_add(preview_gap)
        .saturating_add(row_height.saturating_mul(channel_count))
        .saturating_add(row_gap.saturating_mul(channel_count.saturating_sub(1)));
    let placed_popup = state.expanded.then(|| {
        place_popup(
            bounds,
            metrics.popup_width.to_px(dpi).round_i32().max(1),
            popup_height,
            metrics.popup_gap.to_px(dpi).round_i32().max(0),
            viewport,
        )
    });
    let popup = placed_popup.map(|placed| placed.bounds);
    let mut spectrum_bounds = None;
    let mut spectrum_thumb = None;
    let mut hue_track = None;
    let mut hue_thumb = None;
    let mut preview_bounds = None;
    let mut hex_bounds = None;
    let mut channels = Vec::new();

    if let Some(popup) = popup {
        let content = Rect {
            x: popup.x.saturating_add(inset),
            y: popup.y.saturating_add(inset),
            width: popup.width.saturating_sub(inset.saturating_mul(2)).max(1),
            height: popup.height.saturating_sub(inset.saturating_mul(2)).max(1),
        };
        let hsv = ZsHsvColor::from_color(state.color);
        let mut content_y = content.y;
        if spectrum_section_height > 0 {
            let spectrum = Rect {
                x: content.x,
                y: content_y,
                width: content.width,
                height: spectrum_height,
            };
            let spectrum_thumb_size = scale(14, dpi);
            spectrum_bounds = Some(spectrum);
            spectrum_thumb = Some(Rect {
                x: spectrum
                    .x
                    .saturating_add(
                        ((spectrum.width.saturating_sub(1) as f32) * hsv.saturation).round() as i32,
                    )
                    .saturating_sub(spectrum_thumb_size / 2),
                y: spectrum
                    .y
                    .saturating_add(
                        ((spectrum.height.saturating_sub(1) as f32) * (1.0 - hsv.value)).round()
                            as i32,
                    )
                    .saturating_sub(spectrum_thumb_size / 2),
                width: spectrum_thumb_size,
                height: spectrum_thumb_size,
            });
            content_y = content_y
                .saturating_add(spectrum.height)
                .saturating_add(spectrum_gap);
            let hue = Rect {
                x: content.x,
                y: content_y,
                width: content.width,
                height: hue_track_height,
            };
            let hue_thumb_width = scale(8, dpi);
            hue_track = Some(hue);
            hue_thumb = Some(Rect {
                x: hue
                    .x
                    .saturating_add(
                        ((hue.width.saturating_sub(1) as f32) * (hsv.hue / 360.0)).round() as i32,
                    )
                    .saturating_sub(hue_thumb_width / 2),
                y: hue.y.saturating_sub(scale(3, dpi)),
                width: hue_thumb_width,
                height: hue.height.saturating_add(scale(6, dpi)),
            });
            content_y = content_y
                .saturating_add(hue.height)
                .saturating_add(spectrum_gap);
        }
        let preview = Rect {
            x: content.x,
            y: content_y,
            width: preview_size.min(content.width),
            height: preview_size.min(content.height),
        };
        preview_bounds = Some(preview);
        hex_bounds = Some(Rect {
            x: preview
                .x
                .saturating_add(preview.width)
                .saturating_add(preview_gap),
            y: preview.y,
            width: content
                .x
                .saturating_add(content.width)
                .saturating_sub(preview.x.saturating_add(preview.width))
                .saturating_sub(preview_gap)
                .max(0),
            height: preview.height,
        });

        let label_width = metrics.label_width.to_px(dpi).round_i32().max(1);
        let value_width = metrics.value_width.to_px(dpi).round_i32().max(1);
        let track_height = metrics.track_height.to_px(dpi).round_i32().max(1);
        let thumb_width = metrics.thumb_width.to_px(dpi).round_i32().max(1);
        let rows_y = preview
            .y
            .saturating_add(preview.height)
            .saturating_add(preview_gap);
        for (index, channel) in state.channels().iter().copied().enumerate() {
            let row = Rect {
                x: content.x,
                y: rows_y.saturating_add(
                    (row_height.saturating_add(row_gap)).saturating_mul(index as i32),
                ),
                width: content.width,
                height: row_height,
            };
            let label_bounds = Rect {
                x: row.x,
                y: row.y,
                width: label_width.min(row.width),
                height: row.height,
            };
            let value_bounds = Rect {
                x: row.x.saturating_add(row.width).saturating_sub(value_width),
                y: row.y,
                width: value_width.min(row.width),
                height: row.height,
            };
            let track_x = label_bounds.x.saturating_add(label_bounds.width);
            let track_right = value_bounds.x.saturating_sub(scale(8, dpi));
            let track = Rect {
                x: track_x,
                y: row
                    .y
                    .saturating_add((row.height.saturating_sub(track_height)) / 2),
                width: track_right.saturating_sub(track_x).max(1),
                height: track_height,
            };
            let fraction = f32::from(state.channel_value(channel)) / 255.0;
            let thumb_x = track
                .x
                .saturating_add(((track.width.saturating_sub(1) as f32) * fraction).round() as i32)
                .saturating_sub(thumb_width / 2);
            let thumb = Rect {
                x: thumb_x,
                y: row.y.saturating_add(scale(6, dpi)),
                width: thumb_width,
                height: row.height.saturating_sub(scale(12, dpi)).max(1),
            };
            channels.push(ZsColorPickerChannelRenderPlan {
                channel,
                bounds: row,
                label_bounds,
                track,
                thumb,
                value_bounds,
                active: channel == state.active_channel,
            });
        }
    }

    ZsColorPickerRenderPlan {
        bounds,
        swatch_bounds,
        text_bounds,
        icon_bounds,
        popup,
        popup_placement: placed_popup.map(|placed| placed.placement),
        spectrum_bounds,
        spectrum_thumb,
        hue_track,
        hue_thumb,
        preview_bounds,
        hex_bounds,
        channels,
        platform,
        control_radius: metrics.control_radius.to_px(dpi).round_i32().max(1),
        overlay_radius: metrics.overlay_radius.to_px(dpi).round_i32().max(1),
    }
}

#[cfg(feature = "color-picker")]
impl ZsColorPickerRenderPlan {
    pub fn spectrum_color_at(&self, state: ZsColorPickerState, point: Point) -> Option<Color> {
        let bounds = self.spectrum_bounds?;
        let saturation =
            point.x.saturating_sub(bounds.x) as f32 / bounds.width.saturating_sub(1).max(1) as f32;
        let value = 1.0
            - point.y.saturating_sub(bounds.y) as f32
                / bounds.height.saturating_sub(1).max(1) as f32;
        Some(
            ZsHsvColor::from_color(state.color)
                .with_saturation_value(saturation, value)
                .to_color(state.color.a),
        )
    }

    pub fn hue_color_at(&self, state: ZsColorPickerState, point: Point) -> Option<Color> {
        let bounds = self.hue_track?;
        let fraction =
            point.x.saturating_sub(bounds.x) as f32 / bounds.width.saturating_sub(1).max(1) as f32;
        Some(
            ZsHsvColor::from_color(state.color)
                .with_hue(fraction.clamp(0.0, 1.0) * 359.999)
                .to_color(state.color.a),
        )
    }
}

#[cfg(feature = "color-picker")]
pub fn zs_color_picker_header_native_draw_plan(
    plan: &ZsColorPickerRenderPlan,
    state: ZsColorPickerState,
) -> NativeDrawPlan {
    let state = state.normalized();
    let fill = match plan.platform {
        ZsColorPickerPlatformStyle::Macos => NativeDrawFill::Role(ColorRole::Surface),
        ZsColorPickerPlatformStyle::Windows | ZsColorPickerPlatformStyle::Gtk => {
            NativeDrawFill::Role(ColorRole::Control)
        }
    };
    let swatch_color = Color::rgb(state.color.r, state.color.g, state.color.b);
    NativeDrawPlan::new([
        NativeDrawCommand::RoundRect {
            rect: plan.bounds,
            fill,
            stroke: Some(NativeDrawFill::Role(if state.expanded {
                ColorRole::Accent
            } else {
                ColorRole::Border
            })),
            radius: plan.control_radius,
        },
        NativeDrawCommand::RoundRect {
            rect: plan.swatch_bounds,
            fill: NativeDrawFill::Color(swatch_color),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: (plan.control_radius / 2).max(1),
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            state.hex_label(),
            plan.text_bounds,
            SemanticTextStyle::body(),
        )),
        NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                if state.expanded {
                    ZsIcon::ChevronUp
                } else {
                    ZsIcon::ChevronDown
                },
                plan.icon_bounds,
                NativeIconColorMode::ThemeAware,
            )
            .with_color(ColorRole::PrimaryText),
        ),
    ])
}

#[cfg(feature = "color-picker")]
fn color_over_background(color: Color, background: Color) -> Color {
    let alpha = u32::from(color.a);
    let inverse = 255_u32.saturating_sub(alpha);
    let blend = |foreground: u8, behind: u8| {
        ((u32::from(foreground) * alpha + u32::from(behind) * inverse + 127) / 255) as u8
    };
    Color::rgb(
        blend(color.r, background.r),
        blend(color.g, background.g),
        blend(color.b, background.b),
    )
}

#[cfg(feature = "color-picker")]
fn color_picker_channel_color(
    state: ZsColorPickerState,
    channel: ZsColorChannel,
    value: u8,
    light_background: bool,
) -> Color {
    let candidate = channel.with_value(state.color, value);
    if channel == ZsColorChannel::Alpha {
        color_over_background(
            candidate,
            if light_background {
                Color::rgb(238, 238, 238)
            } else {
                Color::rgb(184, 184, 184)
            },
        )
    } else {
        Color::rgb(candidate.r, candidate.g, candidate.b)
    }
}

#[cfg(feature = "color-picker")]
pub fn zs_color_picker_popup_native_draw_plan(
    plan: &ZsColorPickerRenderPlan,
    state: ZsColorPickerState,
) -> NativeDrawPlan {
    let state = state.normalized();
    let Some(popup) = plan.popup else {
        return NativeDrawPlan::default();
    };
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: popup,
        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
        stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
        radius: plan.overlay_radius,
    }];

    if let Some(spectrum) = plan.spectrum_bounds {
        let hsv = ZsHsvColor::from_color(state.color);
        let columns = 64_i32.min(spectrum.width.max(1));
        let rows = 64_i32.min(spectrum.height.max(1));
        for row in 0..rows {
            for column in 0..columns {
                let x0 = spectrum.x + spectrum.width * column / columns;
                let x1 = spectrum.x + spectrum.width * (column + 1) / columns;
                let y0 = spectrum.y + spectrum.height * row / rows;
                let y1 = spectrum.y + spectrum.height * (row + 1) / rows;
                let saturation = column as f32 / columns.saturating_sub(1).max(1) as f32;
                let value = 1.0 - row as f32 / rows.saturating_sub(1).max(1) as f32;
                commands.push(NativeDrawCommand::FillRect {
                    rect: Rect {
                        x: x0,
                        y: y0,
                        width: x1.saturating_sub(x0).max(1),
                        height: y1.saturating_sub(y0).max(1),
                    },
                    fill: NativeDrawFill::Color(
                        ZsHsvColor::new(hsv.hue, saturation, value).to_color(255),
                    ),
                });
            }
        }
        commands.push(NativeDrawCommand::StrokeRect {
            rect: spectrum,
            stroke: NativeDrawFill::Role(ColorRole::Border),
            width: 1,
        });
        if let Some(thumb) = plan.spectrum_thumb {
            commands.push(NativeDrawCommand::RoundRect {
                rect: thumb,
                fill: NativeDrawFill::Color(Color::rgb(
                    state.color.r,
                    state.color.g,
                    state.color.b,
                )),
                stroke: Some(NativeDrawFill::Role(ColorRole::Surface)),
                radius: (thumb.width / 2).max(1),
            });
        }
    }
    if let Some(hue) = plan.hue_track {
        let segments = 48_i32.min(hue.width.max(1));
        for segment in 0..segments {
            let x0 = hue.x + hue.width * segment / segments;
            let x1 = hue.x + hue.width * (segment + 1) / segments;
            let hue_degrees = segment as f32 / segments.saturating_sub(1).max(1) as f32 * 359.999;
            commands.push(NativeDrawCommand::FillRect {
                rect: Rect {
                    x: x0,
                    y: hue.y,
                    width: x1.saturating_sub(x0).max(1),
                    height: hue.height,
                },
                fill: NativeDrawFill::Color(ZsHsvColor::new(hue_degrees, 1.0, 1.0).to_color(255)),
            });
        }
        if let Some(thumb) = plan.hue_thumb {
            commands.push(NativeDrawCommand::RoundRect {
                rect: thumb,
                fill: NativeDrawFill::Role(ColorRole::Surface),
                stroke: Some(NativeDrawFill::Role(ColorRole::PrimaryText)),
                radius: (thumb.width / 2).max(1),
            });
        }
    }

    if let Some(preview) = plan.preview_bounds {
        let cells = 6_i32;
        for row in 0..cells {
            for column in 0..cells {
                let x0 = preview.x + preview.width * column / cells;
                let x1 = preview.x + preview.width * (column + 1) / cells;
                let y0 = preview.y + preview.height * row / cells;
                let y1 = preview.y + preview.height * (row + 1) / cells;
                let background = if (row + column) % 2 == 0 {
                    Color::rgb(238, 238, 238)
                } else {
                    Color::rgb(184, 184, 184)
                };
                commands.push(NativeDrawCommand::FillRect {
                    rect: Rect {
                        x: x0,
                        y: y0,
                        width: x1.saturating_sub(x0).max(1),
                        height: y1.saturating_sub(y0).max(1),
                    },
                    fill: NativeDrawFill::Color(color_over_background(state.color, background)),
                });
            }
        }
        commands.push(NativeDrawCommand::StrokeRect {
            rect: preview,
            stroke: NativeDrawFill::Role(ColorRole::Border),
            width: 1,
        });
    }
    if let Some(hex_bounds) = plan.hex_bounds {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            state.hex_label(),
            hex_bounds,
            SemanticTextStyle {
                role: TextRole::BodyLarge,
                weight: TextWeight::Semibold,
                ..SemanticTextStyle::body()
            },
        )));
    }

    for row in &plan.channels {
        if row.active {
            commands.push(NativeDrawCommand::RoundFill {
                rect: row.bounds,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: match plan.platform {
                        ZsColorPickerPlatformStyle::Windows => 24,
                        ZsColorPickerPlatformStyle::Macos => 18,
                        ZsColorPickerPlatformStyle::Gtk => 20,
                    },
                },
                radius: plan.control_radius,
            });
        }
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            row.channel.label(),
            row.label_bounds,
            SemanticTextStyle {
                color: if row.active {
                    ColorRole::Accent
                } else {
                    ColorRole::SecondaryText
                },
                weight: if row.active {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Center,
                ..SemanticTextStyle::body()
            },
        )));

        let segments = 32_i32.min(row.track.width.max(1));
        for segment in 0..segments {
            let x0 = row.track.x + row.track.width * segment / segments;
            let x1 = row.track.x + row.track.width * (segment + 1) / segments;
            let value = ((segment * 255) / (segments.saturating_sub(1).max(1))) as u8;
            commands.push(NativeDrawCommand::FillRect {
                rect: Rect {
                    x: x0,
                    y: row.track.y,
                    width: x1.saturating_sub(x0).max(1),
                    height: row.track.height,
                },
                fill: NativeDrawFill::Color(color_picker_channel_color(
                    state,
                    row.channel,
                    value,
                    segment % 2 == 0,
                )),
            });
        }
        commands.push(NativeDrawCommand::RoundRect {
            rect: row.thumb,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(if row.active {
                ColorRole::Accent
            } else {
                ColorRole::Border
            })),
            radius: (row.thumb.width / 2).max(1),
        });
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            state.channel_value(row.channel).to_string(),
            row.value_bounds,
            SemanticTextStyle {
                horizontal_align: HorizontalAlign::End,
                ..SemanticTextStyle::body()
            },
        )));
    }

    NativeDrawPlan::new(commands)
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsContentDialogPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "dialog")]
impl ZsContentDialogPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsContentDialogMetrics {
    pub minimum_width: Dp,
    pub maximum_width: Dp,
    pub viewport_margin: Dp,
    pub content_padding: Dp,
    pub title_gap: Dp,
    pub action_gap: Dp,
    pub button_gap: Dp,
    pub button_height: Dp,
    pub minimum_button_width: Dp,
    pub surface_radius: Dp,
    pub button_radius: Dp,
}

#[cfg(feature = "dialog")]
impl ZsContentDialogMetrics {
    pub const fn for_platform(platform: ZsContentDialogPlatformStyle) -> Self {
        match platform {
            ZsContentDialogPlatformStyle::Windows => Self {
                minimum_width: Dp::new(320.0),
                maximum_width: Dp::new(548.0),
                viewport_margin: Dp::new(24.0),
                content_padding: Dp::new(24.0),
                title_gap: Dp::new(12.0),
                action_gap: Dp::new(24.0),
                button_gap: Dp::new(8.0),
                button_height: Dp::new(40.0),
                minimum_button_width: Dp::new(88.0),
                surface_radius: Dp::new(8.0),
                button_radius: Dp::new(4.0),
            },
            ZsContentDialogPlatformStyle::Macos => Self {
                minimum_width: Dp::new(360.0),
                maximum_width: Dp::new(480.0),
                viewport_margin: Dp::new(28.0),
                content_padding: Dp::new(20.0),
                title_gap: Dp::new(8.0),
                action_gap: Dp::new(20.0),
                button_gap: Dp::new(8.0),
                button_height: Dp::new(28.0),
                minimum_button_width: Dp::new(82.0),
                surface_radius: Dp::new(12.0),
                button_radius: Dp::new(6.0),
            },
            ZsContentDialogPlatformStyle::Gtk => Self {
                minimum_width: Dp::new(340.0),
                maximum_width: Dp::new(480.0),
                viewport_margin: Dp::new(24.0),
                content_padding: Dp::new(24.0),
                title_gap: Dp::new(8.0),
                action_gap: Dp::new(24.0),
                button_gap: Dp::new(8.0),
                button_height: Dp::new(34.0),
                minimum_button_width: Dp::new(86.0),
                surface_radius: Dp::new(12.0),
                button_radius: Dp::new(6.0),
            },
        }
    }
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsContentDialogButtonRenderPlan {
    pub button: crate::ZsContentDialogButton,
    pub label: String,
    pub bounds: Rect,
    pub focused: bool,
    pub default: bool,
    pub destructive: bool,
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsContentDialogRenderPlan {
    pub viewport: Rect,
    pub surface: Rect,
    pub title_bounds: Option<Rect>,
    pub content_bounds: Rect,
    pub buttons: Vec<ZsContentDialogButtonRenderPlan>,
    pub surface_radius: i32,
    pub button_radius: i32,
    pub platform: ZsContentDialogPlatformStyle,
}

#[cfg(feature = "dialog")]
fn content_dialog_visual_buttons(
    spec: &crate::ZsContentDialogSpec,
    platform: ZsContentDialogPlatformStyle,
) -> Vec<crate::ZsContentDialogButton> {
    use crate::ZsContentDialogButton::{Close, Primary, Secondary};
    let order = match platform {
        ZsContentDialogPlatformStyle::Windows => [Primary, Secondary, Close],
        ZsContentDialogPlatformStyle::Macos | ZsContentDialogPlatformStyle::Gtk => {
            [Close, Secondary, Primary]
        }
    };
    let mut buttons = order
        .into_iter()
        .filter(|button| spec.has_button(*button))
        .collect::<Vec<_>>();
    if platform == ZsContentDialogPlatformStyle::Macos {
        if let Some(default) = spec.default_response() {
            if let Some(index) = buttons.iter().position(|button| *button == default) {
                buttons.remove(index);
                buttons.push(default);
            }
        }
    }
    buttons
}

#[cfg(feature = "dialog")]
pub fn zs_content_dialog_render_plan(
    viewport: Rect,
    spec: &crate::ZsContentDialogSpec,
    focused_button: crate::ZsContentDialogButton,
    platform: ZsContentDialogPlatformStyle,
    dpi: Dpi,
) -> ZsContentDialogRenderPlan {
    let metrics = ZsContentDialogMetrics::for_platform(platform);
    let margin = metrics.viewport_margin.to_px(dpi).round_i32().max(0);
    let available_width = viewport
        .width
        .saturating_sub(margin.saturating_mul(2))
        .max(1);
    let minimum_width = metrics.minimum_width.to_px(dpi).round_i32().max(1);
    let maximum_width = metrics.maximum_width.to_px(dpi).round_i32().max(1);
    let surface_width = maximum_width
        .min(available_width)
        .max(minimum_width.min(available_width));
    let padding = metrics.content_padding.to_px(dpi).round_i32().max(0);
    let title_gap = metrics.title_gap.to_px(dpi).round_i32().max(0);
    let action_gap = metrics.action_gap.to_px(dpi).round_i32().max(0);
    let button_gap = metrics.button_gap.to_px(dpi).round_i32().max(0);
    let button_height = metrics.button_height.to_px(dpi).round_i32().max(1);
    let inner_width = surface_width
        .saturating_sub(padding.saturating_mul(2))
        .max(1);
    let title_height = spec
        .dialog_title()
        .map(|title| {
            let lines = ((title.chars().count() + 39) / 40).clamp(1, 2) as i32;
            lines.saturating_mul(scale(24, dpi))
        })
        .unwrap_or(0);
    let content_lines = ((spec.content().chars().count() + 55) / 56).clamp(1, 5) as i32;
    let content_height = content_lines.saturating_mul(scale(20, dpi));
    let desired_height = padding
        .saturating_mul(2)
        .saturating_add(title_height)
        .saturating_add((title_height > 0).then_some(title_gap).unwrap_or(0))
        .saturating_add(content_height)
        .saturating_add(action_gap)
        .saturating_add(button_height);
    let available_height = viewport
        .height
        .saturating_sub(margin.saturating_mul(2))
        .max(1);
    let surface_height = desired_height.min(available_height);
    let surface = Rect {
        x: viewport.x + (viewport.width - surface_width) / 2,
        y: viewport.y + (viewport.height - surface_height) / 2,
        width: surface_width,
        height: surface_height,
    };
    let content_left = surface.x.saturating_add(padding);
    let title_bounds = (title_height > 0).then_some(Rect {
        x: content_left,
        y: surface.y.saturating_add(padding),
        width: inner_width,
        height: title_height,
    });
    let content_y = title_bounds
        .map(|bounds| {
            bounds
                .y
                .saturating_add(bounds.height)
                .saturating_add(title_gap)
        })
        .unwrap_or_else(|| surface.y.saturating_add(padding));
    let buttons_y = surface
        .y
        .saturating_add(surface.height)
        .saturating_sub(padding)
        .saturating_sub(button_height);
    let content_bounds = Rect {
        x: content_left,
        y: content_y,
        width: inner_width,
        height: buttons_y
            .saturating_sub(action_gap)
            .saturating_sub(content_y)
            .max(0),
    };

    let visual_buttons = content_dialog_visual_buttons(spec, platform);
    let total_gap = button_gap.saturating_mul(visual_buttons.len().saturating_sub(1) as i32);
    let minimum_button_width = metrics.minimum_button_width.to_px(dpi).round_i32().max(1);
    let available_button_width = inner_width.saturating_sub(total_gap).max(1);
    let equal_width = available_button_width
        .checked_div(visual_buttons.len().max(1) as i32)
        .unwrap_or(available_button_width)
        .max(1);
    let mut button_layout = visual_buttons
        .into_iter()
        .filter_map(|button| {
            let label = spec.button_label(button)?.to_owned();
            let width = match platform {
                ZsContentDialogPlatformStyle::Windows => equal_width,
                ZsContentDialogPlatformStyle::Macos | ZsContentDialogPlatformStyle::Gtk => {
                    let glyph_width = if platform == ZsContentDialogPlatformStyle::Macos {
                        scale(7, dpi)
                    } else {
                        scale(8, dpi)
                    };
                    let label_width = (label.chars().count() as i32)
                        .saturating_mul(glyph_width)
                        .saturating_add(scale(28, dpi));
                    label_width.max(minimum_button_width)
                }
            };
            Some((button, label, width))
        })
        .collect::<Vec<_>>();
    let natural_width = button_layout
        .iter()
        .fold(0i32, |total, (_, _, width)| total.saturating_add(*width));
    if natural_width > available_button_width {
        for (_, _, width) in &mut button_layout {
            *width = equal_width;
        }
    }
    let buttons_width = button_layout
        .iter()
        .fold(total_gap, |total, (_, _, width)| {
            total.saturating_add(*width)
        });
    let mut button_x = match platform {
        ZsContentDialogPlatformStyle::Windows => content_left,
        ZsContentDialogPlatformStyle::Macos | ZsContentDialogPlatformStyle::Gtk => surface
            .x
            .saturating_add(surface.width)
            .saturating_sub(padding)
            .saturating_sub(buttons_width),
    };
    let buttons = button_layout
        .into_iter()
        .map(|(button, label, button_width)| {
            let bounds = Rect {
                x: button_x,
                y: buttons_y,
                width: button_width,
                height: button_height,
            };
            button_x = button_x
                .saturating_add(button_width)
                .saturating_add(button_gap);
            ZsContentDialogButtonRenderPlan {
                button,
                label,
                bounds,
                focused: focused_button == button,
                default: spec.default_response() == Some(button),
                destructive: spec.destructive_response() == Some(button),
            }
        })
        .collect();

    ZsContentDialogRenderPlan {
        viewport,
        surface,
        title_bounds,
        content_bounds,
        buttons,
        surface_radius: metrics.surface_radius.to_px(dpi).round_i32().max(0),
        button_radius: metrics.button_radius.to_px(dpi).round_i32().max(0),
        platform,
    }
}

#[cfg(feature = "dialog")]
pub fn zs_content_dialog_native_draw_plan(
    plan: &ZsContentDialogRenderPlan,
    spec: &crate::ZsContentDialogSpec,
) -> NativeDrawPlan {
    let scrim_alpha = match plan.platform {
        ZsContentDialogPlatformStyle::Windows => 88,
        ZsContentDialogPlatformStyle::Macos => 56,
        ZsContentDialogPlatformStyle::Gtk => 104,
    };
    let shadow = Rect {
        x: plan.surface.x.saturating_sub(4),
        y: plan.surface.y.saturating_add(2),
        width: plan.surface.width.saturating_add(8),
        height: plan.surface.height.saturating_add(6),
    };
    let mut commands = vec![
        NativeDrawCommand::FillRect {
            rect: plan.viewport,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: scrim_alpha,
            },
        },
        NativeDrawCommand::RoundFill {
            rect: shadow,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: 28,
            },
            radius: plan.surface_radius.saturating_add(4),
        },
        NativeDrawCommand::RoundRect {
            rect: plan.surface,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.surface_radius,
        },
    ];
    if let (Some(title), Some(bounds)) = (spec.dialog_title(), plan.title_bounds) {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            title,
            bounds,
            SemanticTextStyle {
                role: TextRole::Subtitle,
                color: ColorRole::PrimaryText,
                weight: TextWeight::Semibold,
                horizontal_align: HorizontalAlign::Start,
                vertical_align: crate::VerticalAlign::Start,
                wrap: crate::TextWrap::Word,
                ellipsis: true,
            },
        )));
    }
    commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
        spec.content(),
        plan.content_bounds,
        SemanticTextStyle {
            color: ColorRole::PrimaryText,
            vertical_align: crate::VerticalAlign::Start,
            wrap: crate::TextWrap::Word,
            ellipsis: true,
            ..SemanticTextStyle::body()
        },
    )));
    for button in &plan.buttons {
        let (fill, stroke, text_color) = if button.destructive {
            (
                NativeDrawFill::Role(ColorRole::Control),
                NativeDrawFill::Role(ColorRole::Danger),
                ColorRole::Danger,
            )
        } else if button.default {
            (
                NativeDrawFill::Role(ColorRole::Accent),
                NativeDrawFill::Role(ColorRole::Accent),
                ColorRole::AccentText,
            )
        } else {
            (
                NativeDrawFill::Role(ColorRole::Control),
                NativeDrawFill::Role(if button.focused {
                    ColorRole::Accent
                } else {
                    ColorRole::Border
                }),
                ColorRole::PrimaryText,
            )
        };
        commands.push(NativeDrawCommand::RoundRect {
            rect: button.bounds,
            fill,
            stroke: Some(stroke),
            radius: plan.button_radius,
        });
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            &button.label,
            button.bounds,
            SemanticTextStyle {
                role: TextRole::Button,
                color: text_color,
                weight: if button.default {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                horizontal_align: HorizontalAlign::Center,
                vertical_align: crate::VerticalAlign::Center,
                wrap: crate::TextWrap::NoWrap,
                ellipsis: true,
            },
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker"
))]
fn place_popup(
    anchor: Rect,
    requested_width: i32,
    requested_height: i32,
    gap: i32,
    viewport: Option<Rect>,
) -> ZsPlacedPopup {
    let requested_width = requested_width.max(1);
    let requested_height = requested_height.max(1);
    let below_y = anchor.y.saturating_add(anchor.height).saturating_add(gap);
    let Some(viewport) = viewport.filter(|viewport| viewport.width > 0 && viewport.height > 0)
    else {
        return ZsPlacedPopup {
            bounds: Rect {
                x: anchor.x,
                y: below_y,
                width: requested_width,
                height: requested_height,
            },
            placement: ZsPopupPlacement::Below,
        };
    };

    let viewport_right = viewport.x.saturating_add(viewport.width);
    let viewport_bottom = viewport.y.saturating_add(viewport.height);
    let width = requested_width.min(viewport.width).max(1);
    let minimum_x = viewport.x;
    let maximum_x = viewport_right.saturating_sub(width).max(minimum_x);
    let x = anchor.x.clamp(minimum_x, maximum_x);
    let above_bottom = anchor.y.saturating_sub(gap);
    let above_y = above_bottom.saturating_sub(requested_height);
    let available_below = viewport_bottom.saturating_sub(below_y).max(0);
    let available_above = above_bottom.saturating_sub(viewport.y).max(0);
    let fits_below = requested_height <= available_below;
    let fits_above = requested_height <= available_above;
    let placement = if fits_below || (!fits_above && available_below >= available_above) {
        ZsPopupPlacement::Below
    } else {
        ZsPopupPlacement::Above
    };
    let mut y = match placement {
        ZsPopupPlacement::Below => below_y,
        ZsPopupPlacement::Above => above_y,
    };
    if requested_height <= viewport.height {
        y = y.clamp(viewport.y, viewport_bottom.saturating_sub(requested_height));
    } else {
        y = viewport.y;
    }
    ZsPlacedPopup {
        bounds: Rect {
            x,
            y,
            width,
            height: requested_height,
        },
        placement,
    }
}

fn scale(value: i32, dpi: Dpi) -> i32 {
    Dp::new(value as f32).to_px(dpi).round_i32().max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "tree")]
    #[test]
    fn tree_render_plan_preserves_platform_rows_depth_and_disclosure_geometry() {
        let roots = [crate::ZsTreeNode::new(1, "Workspace")
            .icon(ZsIcon::Folder)
            .children([
                crate::ZsTreeNode::new(2, "src")
                    .icon(ZsIcon::Folder)
                    .children([crate::ZsTreeNode::new(3, "lib.rs").icon(ZsIcon::File)]),
                crate::ZsTreeNode::new(4, "Cargo.toml").icon(ZsIcon::File),
            ])];
        let expanded = BTreeSet::from([crate::ZsTreeNodeId::new(1)]);
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 280,
            height: 160,
        };
        let windows = zs_tree_view_render_plan(
            bounds,
            &roots,
            &expanded,
            Some(crate::ZsTreeNodeId::new(2)),
            ZsTreePlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = ZsTreeViewMetrics::for_platform(ZsTreePlatformStyle::Macos);
        let gtk = ZsTreeViewMetrics::for_platform(ZsTreePlatformStyle::Gtk);

        assert_eq!(windows.rows.len(), 3);
        assert_eq!(windows.rows[0].depth, 0);
        assert_eq!(windows.rows[1].depth, 1);
        assert!(windows.rows[0].expanded);
        assert!(windows.rows[1].selected);
        assert!(windows.rows[0].disclosure_bounds.is_some());
        assert!(windows.rows[2].disclosure_bounds.is_none());
        assert!(windows.rows[1].label_bounds.x > windows.rows[0].label_bounds.x);
        assert!(macos.row_height.0 < gtk.row_height.0);
        assert!(macos.depth_indent.0 < gtk.depth_indent.0);

        let draw = zs_tree_view_native_draw_plan(&windows);
        assert_eq!(draw.text_count(), 3);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == ZsIcon::ChevronDown
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundFill {
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                ..
            }
        )));
    }

    #[cfg(feature = "grid-view")]
    #[test]
    fn grid_view_render_plan_is_responsive_clipped_and_platform_specific() {
        let items = [
            crate::ZsGridViewItem::new(1, "Desktop")
                .subtitle("Folder")
                .icon(crate::ZsIcon::Folder),
            crate::ZsGridViewItem::new(2, "Photos")
                .subtitle("Collection")
                .icon(crate::ZsIcon::Image),
            crate::ZsGridViewItem::new(3, "README").icon(crate::ZsIcon::Text),
            crate::ZsGridViewItem::new(4, "Cargo.toml").icon(crate::ZsIcon::File),
            crate::ZsGridViewItem::new(2, "Duplicate strong ID"),
        ];
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 420,
            height: 220,
        };
        let windows = zs_grid_view_render_plan(
            bounds,
            &items,
            Some(crate::ZsGridViewItemId::new(2)),
            ZsGridViewPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_grid_view_render_plan(
            bounds,
            &items,
            Some(crate::ZsGridViewItemId::new(2)),
            ZsGridViewPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_grid_view_render_plan(
            bounds,
            &items,
            Some(crate::ZsGridViewItemId::new(2)),
            ZsGridViewPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.items.len(), 4);
        assert_eq!(windows.column_count, 3);
        assert_eq!(windows.row_count, 2);
        assert_eq!(macos.column_count, 3);
        assert_eq!(gtk.column_count, 2);
        assert!(windows.items[1].selected);
        assert_eq!(windows.items[3].bounds.y, windows.items[0].bounds.y + 120);
        assert!(windows.items[0].icon_bounds.is_some());
        assert!(windows.items[0].subtitle_bounds.is_some());
        assert!(macos.item_radius > windows.item_radius);

        let draw = zs_grid_view_native_draw_plan(&windows, &items);
        assert!(matches!(
            draw.commands.first(),
            Some(NativeDrawCommand::RoundRect { .. })
        ));
        assert!(draw.commands.iter().any(
            |command| matches!(command, NativeDrawCommand::PushClip { rect } if *rect == bounds)
        ));
        assert!(draw
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::Image)));
        assert!(matches!(
            draw.commands.last(),
            Some(NativeDrawCommand::PopClip)
        ));
    }

    #[cfg(feature = "table")]
    #[test]
    fn table_render_plan_preserves_typed_columns_platform_metrics_and_sort_visual() {
        let columns = [
            crate::ZsTableColumn::new(1, "Name")
                .fixed_width(Dp::new(160.0))
                .sortable(true),
            crate::ZsTableColumn::new(2, "Size")
                .fill_width(1)
                .alignment(HorizontalAlign::End)
                .sortable(true),
        ];
        let rows = [
            crate::ZsTableRow::new(10, ["Cargo.toml", "4 KB"]),
            crate::ZsTableRow::new(11, ["src", "—"]),
        ];
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 300,
            height: 160,
        };
        let windows = zs_table_render_plan(
            bounds,
            &columns,
            &rows,
            Some(crate::ZsTableRowId::new(11)),
            Some(crate::ZsTableSort::new(
                crate::ZsTableColumnId::new(2),
                crate::ZsTableSortDirection::Ascending,
            )),
            ZsTablePlatformStyle::Windows,
            Dpi::standard(),
        );

        assert_eq!(windows.columns.len(), 2);
        assert_eq!(windows.columns[0].bounds.width, 160);
        assert_eq!(windows.columns[1].bounds.width, 140);
        assert_eq!(windows.rows.len(), 2);
        assert!(windows.rows[1].selected);
        assert_eq!(windows.rows[0].cells[1].alignment, HorizontalAlign::End);
        assert!(
            ZsTableMetrics::for_platform(ZsTablePlatformStyle::Macos)
                .row_height
                .0
                < ZsTableMetrics::for_platform(ZsTablePlatformStyle::Gtk)
                    .row_height
                    .0
        );

        let draw = zs_table_native_draw_plan(&windows);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == ZsIcon::ChevronUp
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::FillRect {
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 36,
                },
                ..
            }
        )));
    }

    #[cfg(feature = "dialog")]
    #[test]
    fn content_dialog_render_plan_uses_platform_order_metrics_and_semantic_actions() {
        use crate::ZsContentDialogButton::{Close, Primary, Secondary};

        let spec = crate::ZsContentDialogSpec::new(
            "This file already exists. Choose how ZSUI should continue.",
            "Cancel",
        )
        .title("Replace existing file?")
        .primary_button("Replace")
        .secondary_button("Keep Both")
        .default_button(Primary)
        .destructive_button(Secondary);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let windows = zs_content_dialog_render_plan(
            viewport,
            &spec,
            Primary,
            ZsContentDialogPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_content_dialog_render_plan(
            viewport,
            &spec,
            Secondary,
            ZsContentDialogPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_content_dialog_render_plan(
            viewport,
            &spec,
            Close,
            ZsContentDialogPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(
            windows
                .buttons
                .iter()
                .map(|button| button.button)
                .collect::<Vec<_>>(),
            vec![Primary, Secondary, Close]
        );
        assert_eq!(
            macos
                .buttons
                .iter()
                .map(|button| button.button)
                .collect::<Vec<_>>(),
            vec![Close, Secondary, Primary]
        );
        assert_eq!(
            gtk.buttons
                .iter()
                .map(|button| button.button)
                .collect::<Vec<_>>(),
            vec![Close, Secondary, Primary]
        );
        assert!(windows.buttons[0].default);
        assert!(windows.buttons[1].destructive);
        assert!(windows.buttons[0].focused);
        assert!(macos.buttons[0].bounds.x > macos.surface.x);
        assert!(
            ZsContentDialogMetrics::for_platform(ZsContentDialogPlatformStyle::Windows)
                .button_height
                .0
                > ZsContentDialogMetrics::for_platform(ZsContentDialogPlatformStyle::Macos)
                    .button_height
                    .0
        );

        let draw = zs_content_dialog_native_draw_plan(&windows, &spec);
        assert!(matches!(
            draw.commands.first(),
            Some(NativeDrawCommand::FillRect {
                rect,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 88,
                },
            }) if *rect == viewport
        ));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect {
                stroke: Some(NativeDrawFill::Role(ColorRole::Danger)),
                ..
            }
        )));
    }

    #[cfg(feature = "info-bar")]
    #[test]
    fn info_bar_render_plan_preserves_inline_platform_geometry_and_semantic_icon() {
        let spec = crate::ZsInfoBarSpec::new("Reconnect to continue working.")
            .title("No Internet")
            .severity(crate::ZsInfoBarSeverity::Error)
            .action("Network Settings");
        let bounds = Rect {
            x: 20,
            y: 30,
            width: 720,
            height: 64,
        };
        let windows = zs_info_bar_render_plan(
            bounds,
            &spec,
            Some(crate::ZsInfoBarControl::Action),
            ZsInfoBarPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_info_bar_render_plan(
            bounds,
            &spec,
            Some(crate::ZsInfoBarControl::Close),
            ZsInfoBarPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_info_bar_render_plan(
            bounds,
            &spec,
            Some(crate::ZsInfoBarControl::Action),
            ZsInfoBarPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.surface, bounds);
        assert!(windows.accent_bounds.is_some());
        assert!(macos.accent_bounds.is_none());
        assert_eq!(gtk.surface_radius, 0);
        assert!(windows.action_bounds.is_some());
        assert!(windows.close_bounds.unwrap().x > windows.action_bounds.unwrap().x);
        assert!(windows.title_bounds.is_some());
        assert!(windows.message_bounds.is_some());
        assert!(
            ZsInfoBarMetrics::for_platform(ZsInfoBarPlatformStyle::Windows)
                .minimum_height
                .0
                > ZsInfoBarMetrics::for_platform(ZsInfoBarPlatformStyle::Gtk)
                    .minimum_height
                    .0
        );

        let draw = zs_info_bar_native_draw_plan(&windows, &spec);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon)
                if icon.icon == ZsIcon::Error && icon.color == ColorRole::Danger
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "No Internet"
        )));

        let narrow_bounds = Rect {
            x: 12,
            y: 8,
            width: 24,
            height: 32,
        };
        let narrow = zs_info_bar_render_plan(
            narrow_bounds,
            &spec,
            None,
            ZsInfoBarPlatformStyle::Windows,
            Dpi::standard(),
        );
        for rect in [
            Some(narrow.icon_bounds),
            narrow.action_bounds,
            narrow.close_bounds,
        ]
        .into_iter()
        .flatten()
        {
            assert!(rect.x >= narrow_bounds.x);
            assert!(
                rect.x.saturating_add(rect.width)
                    <= narrow_bounds.x.saturating_add(narrow_bounds.width)
            );
        }
    }

    #[cfg(feature = "teaching-tip")]
    #[test]
    fn teaching_tip_plan_points_to_target_and_flips_inside_viewport() {
        let spec = crate::ZsTeachingTipSpec::new(
            "Save automatically",
            "Your changes are saved as you work.",
        )
        .action("Review settings")
        .preferred_placement(crate::ZsTeachingTipPlacement::Bottom);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 420,
        };
        let target = Rect {
            x: 280,
            y: 368,
            width: 80,
            height: 32,
        };
        let windows = zs_teaching_tip_render_plan(
            viewport,
            target,
            &spec,
            crate::ZsTeachingTipControl::Action,
            ZsTeachingTipPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_teaching_tip_render_plan(
            viewport,
            target,
            &spec,
            crate::ZsTeachingTipControl::Close,
            ZsTeachingTipPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_teaching_tip_render_plan(
            viewport,
            target,
            &spec,
            crate::ZsTeachingTipControl::Action,
            ZsTeachingTipPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.placement, crate::ZsTeachingTipPlacement::Top);
        assert!(windows.surface.y + windows.surface.height < target.y);
        assert!(windows.action_bounds.is_some());
        assert!(windows.tail.iter().any(|point| point.y > windows.surface.y));
        assert!(macos.surface_radius > windows.surface_radius);
        assert!(gtk.control_radius > windows.control_radius);
        for plan in [windows, macos, gtk] {
            assert!(plan.surface.x >= viewport.x);
            assert!(plan.surface.y >= viewport.y);
            assert!(plan.surface.x + plan.surface.width <= viewport.x + viewport.width);
            assert!(plan.surface.y + plan.surface.height <= viewport.y + viewport.height);
            let draw = zs_teaching_tip_native_draw_plan(&plan, &spec);
            assert!(draw
                .commands
                .iter()
                .any(|command| matches!(command, NativeDrawCommand::FillTriangle { .. })));
        }
    }

    #[cfg(feature = "breadcrumb")]
    #[test]
    fn breadcrumb_plan_collapses_by_platform_and_keeps_popup_inside_viewport() {
        let items = [
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(1), "Home"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(2), "Projects"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(3), "ZSUI Framework"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(4), "Documentation"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(5), "BreadcrumbBar"),
        ];
        let bounds = Rect {
            x: 24,
            y: 40,
            width: 240,
            height: 34,
        };
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 220,
        };
        let windows = zs_breadcrumb_render_plan(
            bounds,
            &items,
            true,
            ZsBreadcrumbPlatformStyle::Windows,
            Dpi::standard(),
            Some(viewport),
        );
        let macos = zs_breadcrumb_render_plan(
            bounds,
            &items,
            true,
            ZsBreadcrumbPlatformStyle::Macos,
            Dpi::standard(),
            Some(viewport),
        );
        let gtk = zs_breadcrumb_render_plan(
            bounds,
            &items,
            true,
            ZsBreadcrumbPlatformStyle::Gtk,
            Dpi::standard(),
            Some(viewport),
        );

        assert!(!windows.hidden_indices.is_empty());
        assert_eq!(windows.popup_rows.len(), windows.hidden_indices.len());
        assert_eq!(windows.items.last().map(|item| item.item_index), Some(4));
        assert_eq!(macos.items.first().map(|item| item.item_index), Some(0));
        assert!(gtk.overflow_bounds.is_some());
        for plan in [&windows, &macos, &gtk] {
            let popup = plan.popup.expect("open overflow should have a popup");
            assert!(popup.x >= viewport.x);
            assert!(popup.y >= viewport.y);
            assert!(popup.x + popup.width <= viewport.x + viewport.width);
            assert!(popup.y + popup.height <= viewport.y + viewport.height);
        }

        let draw = zs_breadcrumb_native_draw_plan(
            &windows,
            &items,
            Some(crate::ZsBreadcrumbFocusTarget::Overflow),
        );
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == ZsIcon::More
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == ZsIcon::ChevronRight
        )));
        let popup_draw = zs_breadcrumb_popup_native_draw_plan(
            &windows,
            &items,
            Some(crate::ZsBreadcrumbFocusTarget::Item(
                items[windows.hidden_indices[0]].id(),
            )),
        );
        assert_eq!(popup_draw.text_count(), windows.hidden_indices.len());

        let narrow_bounds = Rect {
            x: 7,
            y: 9,
            width: 3,
            height: 12,
        };
        let narrow = zs_breadcrumb_render_plan(
            narrow_bounds,
            &items,
            false,
            ZsBreadcrumbPlatformStyle::Windows,
            Dpi::standard(),
            Some(viewport),
        );
        for rect in narrow
            .items
            .iter()
            .flat_map(|item| [Some(item.bounds), item.separator_bounds])
            .chain([narrow.overflow_bounds, narrow.overflow_separator_bounds])
            .flatten()
        {
            assert!(rect.x >= narrow_bounds.x);
            assert!(rect.x + rect.width <= narrow_bounds.x + narrow_bounds.width);
        }
    }

    #[cfg(feature = "toast")]
    #[test]
    fn toast_render_plan_is_bottom_centered_and_preserves_platform_metrics() {
        let spec = crate::ZsToastSpec::new(1, "File deleted").action("Undo");
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let windows = zs_toast_render_plan(
            viewport,
            &spec,
            crate::ZsToastControl::Action,
            ZsToastPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_toast_render_plan(
            viewport,
            &spec,
            crate::ZsToastControl::Close,
            ZsToastPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_toast_render_plan(
            viewport,
            &spec,
            crate::ZsToastControl::Action,
            ZsToastPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(
            windows.surface.x + windows.surface.width / 2,
            viewport.x + viewport.width / 2
        );
        assert!(windows.surface.y > viewport.height / 2);
        assert!(windows.action_bounds.is_some());
        assert!(windows.close_bounds.x > windows.action_bounds.unwrap().x);
        assert!(
            ZsToastMetrics::for_platform(ZsToastPlatformStyle::Windows)
                .control_height
                .0
                > ZsToastMetrics::for_platform(ZsToastPlatformStyle::Macos)
                    .control_height
                    .0
        );
        assert!(macos.surface.height < windows.surface.height);
        assert!(gtk.surface_radius > windows.surface_radius);

        let draw = zs_toast_native_draw_plan(&windows, &spec);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "File deleted"
        )));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == ZsIcon::Close
        )));
    }

    #[cfg(feature = "number-box")]
    #[test]
    fn number_box_render_plan_preserves_each_platform_stepper_shape() {
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 180,
            height: 36,
        };
        let windows =
            zs_number_box_render_plan(bounds, ZsNumberBoxPlatformStyle::Windows, Dpi::standard());
        let macos =
            zs_number_box_render_plan(bounds, ZsNumberBoxPlatformStyle::Macos, Dpi::standard());
        let gtk = zs_number_box_render_plan(bounds, ZsNumberBoxPlatformStyle::Gtk, Dpi::standard());

        assert_eq!(windows.increment_button.y, windows.decrement_button.y);
        assert!(windows.increment_button.x > windows.decrement_button.x);
        assert_eq!(macos.increment_button.width, 18);
        assert_eq!(macos.increment_button.x, macos.decrement_button.x);
        assert!(macos.increment_button.y < macos.decrement_button.y);
        assert_eq!(gtk.increment_button.y, gtk.decrement_button.y);
        assert!(gtk.increment_button.x > gtk.decrement_button.x);
        assert_eq!(windows.radius, 4);
        assert_eq!(gtk.radius, 5);
        assert_eq!(
            zs_number_box_native_draw_plan(&windows, "12.5", true, true, true).command_count(),
            6
        );
        assert!(matches!(
            zs_number_box_native_draw_plan(&windows, "-", false, false, true).commands[0],
            NativeDrawCommand::RoundRect {
                stroke: Some(NativeDrawFill::Role(ColorRole::Danger)),
                ..
            }
        ));
    }

    #[cfg(feature = "tabs")]
    #[test]
    fn tab_metrics_preserve_each_desktop_platform_character() {
        let windows = ZsTabViewMetrics::for_platform(ZsTabPlatformStyle::Windows);
        let macos = ZsTabViewMetrics::for_platform(ZsTabPlatformStyle::Macos);
        let gtk = ZsTabViewMetrics::for_platform(ZsTabPlatformStyle::Gtk);

        assert!(windows.strip_height.0 > macos.strip_height.0);
        assert!(macos.outer_inset.0 > windows.outer_inset.0);
        assert!(gtk.selection_indicator_height.0 > windows.selection_indicator_height.0);
        assert!(!ZsTabPlatformStyle::Windows.arrow_selects());
        assert!(ZsTabPlatformStyle::Macos.arrow_selects());
        assert!(!ZsTabPlatformStyle::Gtk.arrow_selects());
        assert!(!ZsTabPlatformStyle::Windows.supports_home_end_focus());
        assert!(!ZsTabPlatformStyle::Macos.supports_home_end_focus());
        assert!(ZsTabPlatformStyle::Gtk.supports_home_end_focus());
    }

    #[cfg(feature = "tabs")]
    #[test]
    fn tab_render_plan_keeps_headers_and_selected_content_inside_bounds() {
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 420,
            height: 280,
        };
        let labels = vec!["General".into(), "Advanced".into(), "About".into()];
        let windows = zs_tab_view_render_plan(
            bounds,
            &labels,
            Some(1),
            ZsTabPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_tab_view_render_plan(
            bounds,
            &labels,
            Some(1),
            ZsTabPlatformStyle::Macos,
            Dpi::standard(),
        );

        assert_eq!(windows.headers.len(), 3);
        assert!(windows.headers[1].selected);
        assert!(windows.headers[1].selection_indicator.is_some());
        assert_eq!(windows.content_bounds.y, bounds.y + 40);
        assert!(windows.headers.iter().all(|header| {
            header.bounds.x >= bounds.x
                && header.bounds.x + header.bounds.width <= bounds.x + bounds.width
        }));
        assert!(macos
            .headers
            .iter()
            .all(|header| { header.bounds.width == macos.headers[0].bounds.width }));
        assert!(macos.headers[0].bounds.x > bounds.x);

        let narrow = zs_tab_view_render_plan(
            Rect {
                x: 10,
                y: 20,
                width: 2,
                height: 20,
            },
            &labels,
            Some(1),
            ZsTabPlatformStyle::Windows,
            Dpi::standard(),
        );
        assert!(narrow.headers.iter().all(|header| {
            header.bounds.x >= 10
                && header.bounds.x + header.bounds.width <= 12
                && header.text_bounds.x >= header.bounds.x
                && header.text_bounds.x + header.text_bounds.width
                    <= header.bounds.x + header.bounds.width
        }));

        let draw = zs_tab_view_native_draw_plan(&windows, &labels);
        assert_eq!(draw.text_count(), 3);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundFill {
                fill: NativeDrawFill::Role(ColorRole::Accent),
                ..
            }
        )));
    }

    #[test]
    fn toggle_geometry_matches_standard_dpi_shape() {
        let off = zs_toggle_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 32,
            },
            false,
            false,
            Dpi::standard(),
        );
        let on = zs_toggle_render_plan(off.bounds, false, true, Dpi::standard());

        assert_eq!(off.track.width, 40);
        assert_eq!(off.track.height, 20);
        assert!(off.knob.x < on.knob.x);
        assert_eq!(zs_toggle_native_draw_plan(&on).command_count(), 2);
    }

    #[cfg(feature = "toggle-button")]
    #[test]
    fn toggle_button_render_plan_preserves_platform_metrics_and_checked_cue() {
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 144,
            height: 36,
        };
        let windows = zs_toggle_button_render_plan(
            bounds,
            false,
            ZsToggleButtonPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_toggle_button_render_plan(
            bounds,
            true,
            ZsToggleButtonPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_toggle_button_render_plan(
            bounds,
            true,
            ZsToggleButtonPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.radius, 4);
        assert_eq!(macos.radius, 6);
        assert_eq!(gtk.radius, 5);
        assert_eq!(windows.text_bounds.y, bounds.y);
        assert_eq!(macos.text_bounds.y, bounds.y + 1);
        assert_eq!(gtk.selected_indicator.height, 3);
        assert_eq!(
            zs_toggle_button_native_draw_plan(&windows, "Pin").command_count(),
            2
        );
        let checked = zs_toggle_button_native_draw_plan(&macos, "Pin");
        assert_eq!(checked.command_count(), 3);
        assert!(matches!(
            checked.commands.as_slice(),
            [
                NativeDrawCommand::RoundRect {
                    fill: NativeDrawFill::Role(ColorRole::Accent),
                    ..
                },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::RoundFill { .. }
            ]
        ));
    }

    #[cfg(feature = "slider")]
    #[test]
    fn slider_geometry_maps_fraction_to_semantic_track_and_thumb() {
        let plan = zs_slider_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            0.25,
            Dpi::standard(),
        );

        assert_eq!(plan.track.x, 8);
        assert_eq!(plan.track.width, 184);
        assert_eq!(plan.filled_track.width, 46);
        assert_eq!(plan.thumb.x, 46);
        assert!(matches!(
            zs_slider_native_draw_plan(&plan).commands.as_slice(),
            [
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::Role(ColorRole::Control),
                    ..
                },
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::Role(ColorRole::Accent),
                    ..
                },
                NativeDrawCommand::RoundRect {
                    fill: NativeDrawFill::Role(ColorRole::Surface),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    ..
                }
            ]
        ));
    }

    #[cfg(feature = "radio")]
    #[test]
    fn radio_geometry_uses_semantic_circle_and_selected_dot() {
        let plan = zs_radio_render_plan(
            Rect {
                x: 4,
                y: 8,
                width: 180,
                height: 32,
            },
            true,
            Dpi::standard(),
        );

        assert_eq!(plan.indicator.width, 20);
        assert_eq!(plan.indicator.height, 20);
        assert_eq!(plan.selected_dot.expect("selected radio dot").width, 8);
        assert_eq!(zs_radio_native_draw_plan(&plan).command_count(), 2);
        assert_eq!(
            zs_radio_native_draw_plan(&zs_radio_render_plan(plan.bounds, false, Dpi::standard()))
                .command_count(),
            1
        );
    }

    #[cfg(feature = "progress")]
    #[test]
    fn progress_geometry_clamps_fill_and_omits_zero_accent() {
        let bounds = Rect {
            x: 4,
            y: 8,
            width: 200,
            height: 32,
        };
        let plan = zs_progress_bar_render_plan(bounds, 0.625, Dpi::standard());

        assert_eq!(plan.track.width, 200);
        assert_eq!(plan.track.height, 4);
        assert_eq!(plan.filled_track.expect("determinate fill").width, 125);
        assert_eq!(zs_progress_bar_native_draw_plan(&plan).command_count(), 2);
        assert_eq!(
            zs_progress_bar_native_draw_plan(&zs_progress_bar_render_plan(
                bounds,
                0.0,
                Dpi::standard()
            ))
            .command_count(),
            1
        );
    }

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn auto_suggest_preserves_platform_search_field_metrics_and_semantic_icons() {
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 300,
            height: 32,
        };
        let windows = zs_auto_suggest_render_plan(
            bounds,
            3,
            None,
            true,
            true,
            true,
            ZsAutoSuggestPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_auto_suggest_render_plan(
            bounds,
            3,
            None,
            false,
            false,
            true,
            ZsAutoSuggestPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_auto_suggest_render_plan(
            Rect {
                height: 34,
                ..bounds
            },
            3,
            None,
            false,
            true,
            true,
            ZsAutoSuggestPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(windows.control_radius, 4);
        assert_eq!(
            windows.search_button.expect("Windows query button").width,
            32
        );
        assert_eq!(windows.search_icon.expect("Windows query icon").width, 16);
        assert_eq!(windows.popup_placement, Some(ZsPopupPlacement::Below));
        assert_eq!(windows.suggestion_rows.len(), 3);
        assert_eq!(macos.search_button.expect("macOS search icon").x, 0);
        assert_eq!(macos.clear_button.expect("macOS cancel button").x, 276);
        assert_eq!(macos.control_radius, 6);
        assert_eq!(gtk.control_radius, 8);
        assert!(matches!(
            zs_auto_suggest_header_native_draw_plan(&windows, "", Some("Search"))
                .commands
                .as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::Icon(NativeDrawIconCommand {
                    icon: ZsIcon::Search,
                    ..
                })
            ]
        ));
    }

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn auto_suggest_popup_keeps_strong_id_highlight_visible_and_flips_in_viewport() {
        let suggestions = (0..20)
            .map(|index| ZsAutoSuggestion::new(index as u64, format!("Result {index}")))
            .collect::<Vec<_>>();
        let plan = zs_auto_suggest_render_plan_in_viewport(
            Rect {
                x: 250,
                y: 250,
                width: 180,
                height: 32,
            },
            suggestions.len(),
            Some(18),
            true,
            false,
            true,
            ZsAutoSuggestPlatformStyle::Windows,
            Dpi::standard(),
            Rect {
                x: 0,
                y: 0,
                width: 360,
                height: 320,
            },
        );

        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Above));
        assert!(plan.first_visible_suggestion > 0);
        assert!(18 >= plan.first_visible_suggestion);
        assert!(18 < plan.first_visible_suggestion + plan.suggestion_rows.len());
        assert_eq!(plan.popup.expect("popup").x, 180);
        assert_eq!(
            zs_auto_suggest_popup_native_draw_plan(
                &plan,
                &suggestions,
                Some(18_u64.into()),
                None,
                Dpi::standard(),
            )
            .command_count(),
            plan.suggestion_rows.len() + 2
        );
    }

    #[cfg(feature = "combo")]
    #[test]
    fn combo_geometry_separates_header_popup_rows_and_semantic_icon() {
        let bounds = Rect {
            x: 12,
            y: 20,
            width: 220,
            height: 36,
        };
        let plan = zs_combo_box_render_plan(bounds, 3, true, Dpi::standard());
        let popup = plan
            .popup
            .expect("expanded combo should have popup geometry");

        assert_eq!(popup.y, 60);
        assert_eq!(popup.height, 108);
        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Below));
        assert_eq!(plan.option_rows.len(), 3);
        assert_eq!(plan.option_rows[1].y, 96);
        assert!(matches!(
            zs_combo_box_header_native_draw_plan(&plan, Some("Balanced"), None)
                .commands
                .as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::Icon(NativeDrawIconCommand {
                    icon: ZsIcon::ChevronDown,
                    ..
                })
            ]
        ));
        assert_eq!(
            zs_combo_box_popup_native_draw_plan(
                &plan,
                &["Balanced".into(), "Fast".into(), "Quiet".into()],
                Some(1),
                Dpi::standard(),
            )
            .command_count(),
            5
        );
        assert!(zs_combo_box_render_plan(bounds, 3, false, Dpi::standard())
            .popup
            .is_none());
    }

    #[cfg(feature = "combo")]
    #[test]
    fn combo_popup_flips_above_and_clamps_to_viewport_right_edge() {
        let plan = zs_combo_box_render_plan_in_viewport(
            Rect {
                x: 250,
                y: 180,
                width: 100,
                height: 32,
            },
            3,
            true,
            Dpi::standard(),
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 240,
            },
        );

        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Above));
        assert_eq!(
            plan.popup,
            Some(Rect {
                x: 200,
                y: 80,
                width: 100,
                height: 96,
            })
        );
        assert_eq!(plan.option_rows[2].y, 144);
    }

    #[cfg(feature = "combo")]
    #[test]
    fn combo_popup_caps_rows_to_winui_limit_and_keeps_selection_visible() {
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 300,
        };
        let plan = zs_combo_box_render_plan_in_viewport_with_scroll(
            Rect {
                x: 20,
                y: 100,
                width: 200,
                height: 32,
            },
            100,
            Some(90),
            None,
            true,
            Dpi::standard(),
            viewport,
        );
        let popup = plan.popup.expect("long combo should expose a popup");

        assert_eq!(plan.option_rows.len(), 5);
        assert_eq!(plan.first_visible_option, 86);
        assert!(plan.first_visible_option <= 90);
        assert!(90 < plan.first_visible_option + plan.option_rows.len());
        assert!(popup.y >= viewport.y);
        assert!(popup.y + popup.height <= viewport.y + viewport.height);

        let options = (0..100)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let draw = zs_combo_box_popup_native_draw_plan(&plan, &options, Some(90), Dpi::standard());
        assert!(draw.commands.iter().any(
            |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "Option 86")
        ));

        let unconstrained = zs_combo_box_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            100,
            true,
            Dpi::standard(),
        );
        assert_eq!(
            unconstrained.option_rows.len(),
            ZS_COMBO_BOX_MAX_VISIBLE_OPTIONS
        );
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn date_picker_geometry_uses_winui_metrics_and_typed_calendar_cells() {
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let plan = zs_date_picker_render_plan_with_today(
            Rect {
                x: 24,
                y: 64,
                width: 472,
                height: 32,
            },
            value,
            value.first_day_of_month(),
            ZsDate::new(2026, 6, 15).unwrap(),
            ZsDate::new(2026, 8, 20).unwrap(),
            Some(ZsDate::new(2026, 7, 14).unwrap()),
            true,
            Dpi::standard(),
        );

        assert_eq!(plan.control_radius, 4);
        assert_eq!(plan.overlay_radius, 8);
        assert_eq!(plan.icon_bounds.width, 12);
        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Below));
        assert_eq!(plan.popup.unwrap().y, 100);
        assert_eq!(plan.popup.unwrap().width, 296);
        assert_eq!(plan.popup.unwrap().height, 332);
        assert_eq!(plan.weekday_cells.len(), 7);
        assert_eq!(plan.day_cells.len(), 42);
        assert_eq!(plan.day_cells[0].date, ZsDate::new(2026, 6, 28).unwrap());
        assert_eq!(
            plan.day_cells.iter().filter(|cell| cell.selected).count(),
            1
        );
        assert_eq!(plan.day_cells.iter().filter(|cell| cell.today).count(), 1);
        assert!(plan
            .day_cells
            .iter()
            .any(|cell| cell.today && !cell.selected && cell.enabled));
        assert!(matches!(
            zs_date_picker_header_native_draw_plan(&plan, value)
                .commands
                .as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::Icon(NativeDrawIconCommand {
                    icon: ZsIcon::Calendar,
                    ..
                })
            ]
        ));
        assert_eq!(
            zs_date_picker_popup_native_draw_plan(
                &plan,
                value.first_day_of_month(),
                Dpi::standard()
            )
            .command_count(),
            55
        );
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn date_picker_popup_flips_above_and_clamps_to_viewport_at_scaled_dpi() {
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let plan = zs_date_picker_render_plan_in_viewport(
            Rect {
                x: 520,
                y: 720,
                width: 200,
                height: 64,
            },
            value,
            value.first_day_of_month(),
            ZsDate::new(1900, 1, 1).unwrap(),
            ZsDate::new(2100, 12, 31).unwrap(),
            true,
            Dpi::new(192.0),
            Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 960,
            },
        );

        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Above));
        assert_eq!(
            plan.popup,
            Some(Rect {
                x: 208,
                y: 48,
                width: 592,
                height: 664,
            })
        );
        assert_eq!(plan.day_cells.len(), 42);
        assert!(plan
            .day_cells
            .iter()
            .all(|cell| cell.bounds.x >= 208 && cell.bounds.x < 800));
    }

    #[cfg(feature = "time-picker")]
    #[test]
    fn time_picker_uses_platform_metrics_and_typed_segment_choices() {
        let value = ZsTime::new(18, 15).unwrap();
        let bounds = Rect {
            x: 24,
            y: 64,
            width: 240,
            height: 32,
        };
        let windows = zs_time_picker_render_plan(
            bounds,
            value,
            ZsMinuteIncrement::FIFTEEN,
            ZsClockFormat::TwelveHour,
            true,
            ZsTimePickerPlatformStyle::Windows,
            Dpi::standard(),
        );

        assert_eq!(windows.popup_placement, Some(ZsPopupPlacement::Below));
        assert_eq!(windows.popup.unwrap().width, 280);
        assert_eq!(windows.column_bounds.len(), 3);
        assert_eq!(windows.choices.len(), 12);
        assert_eq!(
            windows
                .choices
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            3
        );
        assert!(windows.choices.iter().any(|choice| {
            choice.segment == ZsTimePickerSegment::Minute
                && choice.label == "30"
                && choice.value == ZsTime::new(18, 30).unwrap()
        }));
        assert!(matches!(
            zs_time_picker_header_native_draw_plan(&windows, value)
                .commands
                .as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::Icon(NativeDrawIconCommand {
                    icon: ZsIcon::ChevronDown,
                    ..
                })
            ]
        ));

        let macos = zs_time_picker_render_plan(
            bounds,
            value,
            ZsMinuteIncrement::FIFTEEN,
            ZsClockFormat::TwentyFourHour,
            true,
            ZsTimePickerPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_time_picker_render_plan(
            bounds,
            value,
            ZsMinuteIncrement::FIFTEEN,
            ZsClockFormat::TwentyFourHour,
            true,
            ZsTimePickerPlatformStyle::Gtk,
            Dpi::standard(),
        );
        assert_eq!(macos.column_bounds.len(), 2);
        assert_eq!(macos.choices.len(), 6);
        assert_eq!(macos.control_radius, 6);
        assert_eq!(gtk.popup.unwrap().width, 240);
        assert_eq!(gtk.overlay_radius, 12);
    }

    #[cfg(feature = "time-picker")]
    #[test]
    fn time_picker_popup_flips_and_clamps_with_shared_viewport_placement() {
        let plan = zs_time_picker_render_plan_in_viewport(
            Rect {
                x: 250,
                y: 220,
                width: 120,
                height: 32,
            },
            ZsTime::new(9, 30).unwrap(),
            ZsMinuteIncrement::THIRTY,
            ZsClockFormat::TwentyFourHour,
            true,
            ZsTimePickerPlatformStyle::Gtk,
            Dpi::standard(),
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 280,
            },
        );

        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Above));
        assert_eq!(plan.popup.unwrap().x, 60);
        assert!(plan
            .choices
            .iter()
            .all(|choice| choice.bounds.x >= 60 && choice.bounds.x < 300));
    }

    #[cfg(feature = "color-picker")]
    #[test]
    fn color_picker_uses_platform_color_well_metrics_and_shared_channel_geometry() {
        let state = ZsColorPickerState::new(Color::rgba(12, 96, 220, 144))
            .with_expanded(true)
            .with_active_channel(ZsColorChannel::Blue);
        let bounds = Rect {
            x: 24,
            y: 20,
            width: 220,
            height: 32,
        };
        let windows = zs_color_picker_render_plan(
            bounds,
            state,
            ZsColorPickerPlatformStyle::Windows,
            Dpi::standard(),
        );

        assert_eq!(windows.popup_placement, Some(ZsPopupPlacement::Below));
        assert_eq!(windows.popup.unwrap().width, 320);
        assert!(windows.spectrum_bounds.is_some());
        assert!(windows.hue_track.is_some());
        assert_eq!(windows.channels.len(), 4);
        assert_eq!(windows.channels[2].channel, ZsColorChannel::Blue);
        assert!(windows.channels[2].active);
        assert!(windows.channels.iter().all(|row| row.track.width > 0));
        assert!(matches!(
            zs_color_picker_header_native_draw_plan(&windows, state)
                .commands
                .as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::Icon(NativeDrawIconCommand {
                    icon: ZsIcon::ChevronUp,
                    ..
                })
            ]
        ));
        let popup = zs_color_picker_popup_native_draw_plan(&windows, state);
        assert!(
            popup
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    NativeDrawCommand::FillRect {
                        fill: NativeDrawFill::Color(_),
                        ..
                    }
                ))
                .count()
                >= 64 * 64
        );
        assert!(popup.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "#0C60DC90"
        )));

        let macos = ZsColorPickerMetrics::for_platform(ZsColorPickerPlatformStyle::Macos);
        let gtk = ZsColorPickerMetrics::for_platform(ZsColorPickerPlatformStyle::Gtk);
        assert!(macos.row_height.0 < gtk.row_height.0);
        assert!(macos.popup_width.0 < gtk.popup_width.0);
        assert_eq!(macos.spectrum_height, Dp::new(0.0));
        let spectrum = windows.spectrum_bounds.unwrap();
        assert!(spectrum.width >= 256);
        assert!(spectrum.height >= 256);
        let saturated = windows
            .spectrum_color_at(
                state,
                Point {
                    x: spectrum.x + spectrum.width - 1,
                    y: spectrum.y,
                },
            )
            .unwrap();
        assert_eq!(saturated.a, state.color.a);
        assert_eq!(ZsHsvColor::from_color(saturated).saturation, 1.0);
    }

    #[cfg(feature = "color-picker")]
    #[test]
    fn color_picker_popup_flips_clamps_and_omits_alpha_when_disabled() {
        let state = ZsColorPickerState::new(Color::rgba(32, 64, 96, 40))
            .without_alpha()
            .with_expanded(true);
        let plan = zs_color_picker_render_plan_in_viewport(
            Rect {
                x: 250,
                y: 250,
                width: 120,
                height: 32,
            },
            state,
            ZsColorPickerPlatformStyle::Gtk,
            Dpi::standard(),
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 300,
            },
        );

        assert_eq!(plan.popup_placement, Some(ZsPopupPlacement::Above));
        assert_eq!(plan.popup.unwrap().x, 12);
        assert_eq!(plan.channels.len(), 3);
        assert!(plan
            .channels
            .iter()
            .all(|row| row.channel != ZsColorChannel::Alpha));
    }
}
