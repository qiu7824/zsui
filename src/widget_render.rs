use serde::{Deserialize, Serialize};

#[cfg(any(
    feature = "date-picker",
    feature = "tabs",
    feature = "time-picker",
    feature = "toggle-button"
))]
use crate::TextRole;
#[cfg(feature = "date-picker")]
use crate::ZsDate;
use crate::{Color, ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect};
#[cfg(any(feature = "date-picker", feature = "tabs", feature = "time-picker"))]
use crate::{HorizontalAlign, TextWeight};
#[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
use crate::{NativeDrawIconCommand, NativeIconColorMode, ZsIcon};
#[cfg(any(
    feature = "combo",
    feature = "date-picker",
    feature = "number-box",
    feature = "tabs",
    feature = "time-picker",
    feature = "toggle-button"
))]
use crate::{NativeDrawTextCommand, SemanticTextStyle};
#[cfg(feature = "time-picker")]
use crate::{ZsClockFormat, ZsMinuteIncrement, ZsTime};

#[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsPopupPlacement {
    Below,
    Above,
}

#[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
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

#[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
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
}
