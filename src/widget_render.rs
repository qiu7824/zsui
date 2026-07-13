use serde::{Deserialize, Serialize};

use crate::{Color, ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect};
#[cfg(feature = "date-picker")]
use crate::{HorizontalAlign, TextRole, TextWeight, ZsDate};
#[cfg(any(feature = "combo", feature = "date-picker"))]
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

#[cfg(feature = "date-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDatePickerDayCell {
    pub bounds: Rect,
    pub date: ZsDate,
    pub in_display_month: bool,
    pub enabled: bool,
    pub selected: bool,
}

#[cfg(feature = "date-picker")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDatePickerRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub icon_bounds: Rect,
    pub popup: Option<Rect>,
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
    let popup = expanded.then_some(Rect {
        x: bounds.x,
        y: bounds
            .y
            .saturating_add(bounds.height)
            .saturating_add(popup_gap),
        width: popup_width,
        height: popup_height,
    });

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
            });
        }
    }

    ZsDatePickerRenderPlan {
        bounds,
        text_bounds,
        icon_bounds,
        popup,
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
        if cell.selected {
            let diameter = scale(32, dpi)
                .min(cell.bounds.width)
                .min(cell.bounds.height);
            commands.push(NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: cell.bounds.x + (cell.bounds.width - diameter) / 2,
                    y: cell.bounds.y + (cell.bounds.height - diameter) / 2,
                    width: diameter,
                    height: diameter,
                },
                fill: NativeDrawFill::Role(ColorRole::Accent),
                radius: diameter / 2,
            });
        }
        let color = if !cell.enabled {
            ColorRole::DisabledText
        } else if cell.selected {
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

    #[cfg(feature = "date-picker")]
    #[test]
    fn date_picker_geometry_uses_winui_metrics_and_typed_calendar_cells() {
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let plan = zs_date_picker_render_plan(
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
            true,
            Dpi::standard(),
        );

        assert_eq!(plan.control_radius, 4);
        assert_eq!(plan.overlay_radius, 8);
        assert_eq!(plan.icon_bounds.width, 12);
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
            54
        );
    }
}
