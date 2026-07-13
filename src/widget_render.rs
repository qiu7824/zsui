use serde::{Deserialize, Serialize};

use crate::{Color, ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect};
#[cfg(feature = "combo")]
use crate::{
    NativeDrawIconCommand, NativeDrawTextCommand, NativeIconColorMode, SemanticTextStyle, ZsIcon,
};

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

#[cfg(feature = "combo")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsComboBoxRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
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
    let popup = (expanded && option_count > 0).then_some(Rect {
        x: bounds.x,
        y: bounds
            .y
            .saturating_add(bounds.height)
            .saturating_add(popup_gap),
        width: bounds.width.max(1),
        height: row_height.saturating_mul(option_count.min(i32::MAX as usize) as i32),
    });
    let option_rows = popup
        .map(|popup| {
            (0..option_count)
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
        option_rows,
        radius: scale(6, dpi),
    }
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
    for (index, (label, row)) in options.iter().zip(&plan.option_rows).enumerate() {
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

fn scale(value: i32, dpi: Dpi) -> i32 {
    Dp::new(value as f32).to_px(dpi).round_i32().max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
