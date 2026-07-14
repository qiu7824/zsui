use crate::native_text_edit::{char_count, NativeTextSelection};
use crate::{
    ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Point, Rect,
    ViewHitTarget, ViewHitTargetKind, ViewInteractionPlan, WidgetId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeTextVisualGeometry {
    pub caret: Rect,
    pub selections: Vec<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeTextVisualDirection {
    Up,
    Down,
}

pub(crate) fn native_text_visual_target(
    target: ViewHitTarget,
    interaction: &ViewInteractionPlan,
) -> ViewHitTarget {
    #[allow(unused_mut)]
    let mut target = target;
    #[cfg(feature = "password-box")]
    {
        if target.kind == ViewHitTargetKind::PasswordBox {
            if let Some(reveal) = interaction.hit_targets.iter().find(|candidate| {
                candidate.widget == target.widget
                    && candidate.kind == ViewHitTargetKind::PasswordBoxReveal
            }) {
                target.bounds.width = reveal.bounds.x.saturating_sub(target.bounds.x).max(0);
            }
        }
    }
    #[cfg(feature = "auto-suggest")]
    {
        if target.kind == ViewHitTargetKind::AutoSuggestBox {
            for accessory in interaction.hit_targets.iter().filter(|candidate| {
                candidate.widget == target.widget
                    && matches!(
                        candidate.kind,
                        ViewHitTargetKind::AutoSuggestSearch | ViewHitTargetKind::AutoSuggestClear
                    )
            }) {
                if accessory.bounds.x <= target.bounds.x {
                    let offset = accessory.bounds.width.min(target.bounds.width.max(0));
                    target.bounds.x = target.bounds.x.saturating_add(offset);
                    target.bounds.width = target.bounds.width.saturating_sub(offset);
                } else {
                    let right = target.bounds.x.saturating_add(target.bounds.width);
                    target.bounds.width = accessory
                        .bounds
                        .x
                        .saturating_sub(target.bounds.x)
                        .min(right.saturating_sub(target.bounds.x))
                        .max(0);
                }
            }
        }
    }
    #[cfg(not(any(feature = "password-box", feature = "auto-suggest")))]
    let _ = interaction;
    target
}

#[cfg(test)]
pub(crate) fn native_text_visual_geometry(
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    native_text_visual_geometry_in_viewport(target, value, selection, 0, wrap, dpi)
}

pub(crate) fn native_text_visual_geometry_in_viewport(
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics(target, dpi);
    let text_bounds = metrics.text_bounds;
    let character_width = metrics.character_width;
    let line_height = metrics.line_height;
    let max_columns = text_bounds
        .width
        .checked_div(character_width)
        .unwrap_or(1)
        .max(1) as usize;
    let lines = text_lines(value, multiline, wrap, max_columns);
    let first_visible_row = first_visible_row.min(lines.len().saturating_sub(1));
    let selection = selection.clamp(value);
    let (caret_row, caret_column) = text_position(selection.caret, &lines);
    let caret_x = text_bounds
        .x
        .saturating_add((caret_column as i32).saturating_mul(character_width))
        .min(
            text_bounds
                .x
                .saturating_add(text_bounds.width)
                .saturating_sub(1),
        );
    let caret_y = visual_row_y(text_bounds.y, caret_row, first_visible_row, line_height);
    let caret = Rect {
        x: caret_x,
        y: caret_y,
        width: Dp::new(1.0).to_px(dpi).round_i32().max(1),
        height: line_height
            .min(
                text_bounds
                    .y
                    .saturating_add(text_bounds.height)
                    .saturating_sub(caret_y),
            )
            .max(1),
    };

    let (start, end) = selection.ordered();
    let mut selections = Vec::new();
    if start != end {
        for (row, line) in lines.into_iter().enumerate().skip(first_visible_row) {
            let overlap_start = start.max(line.start);
            let overlap_end = end.min(line.end);
            if overlap_start >= overlap_end
                && !(!line.soft_wrap_after && end > line.end && start <= line.end)
            {
                continue;
            }
            let start_column = overlap_start.saturating_sub(line.start);
            let end_column = overlap_end.saturating_sub(line.start);
            let x = text_bounds
                .x
                .saturating_add((start_column as i32).saturating_mul(character_width));
            let selected_columns = end_column.saturating_sub(start_column).max(1) as i32;
            let width = selected_columns
                .saturating_mul(character_width)
                .min(
                    text_bounds
                        .x
                        .saturating_add(text_bounds.width)
                        .saturating_sub(x),
                )
                .max(1);
            let y = visual_row_y(text_bounds.y, row, first_visible_row, line_height);
            if y >= text_bounds.y.saturating_add(text_bounds.height) {
                break;
            }
            selections.push(Rect {
                x,
                y,
                width,
                height: line_height
                    .min(
                        text_bounds
                            .y
                            .saturating_add(text_bounds.height)
                            .saturating_sub(y),
                    )
                    .max(1),
            });
        }
    }
    NativeTextVisualGeometry { caret, selections }
}

#[cfg(test)]
pub(crate) fn native_text_index_for_point(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    native_text_index_for_point_in_viewport(target, value, point, 0, wrap, dpi)
}

pub(crate) fn native_text_index_for_point_in_viewport(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics(target, dpi);
    let max_columns = metrics
        .text_bounds
        .width
        .checked_div(metrics.character_width)
        .unwrap_or(1)
        .max(1) as usize;
    let lines = text_lines(value, multiline, wrap, max_columns);
    let row = first_visible_row
        .saturating_add(if multiline {
            point
                .y
                .saturating_sub(metrics.text_bounds.y)
                .max(0)
                .checked_div(metrics.line_height)
                .unwrap_or(0) as usize
        } else {
            0
        })
        .min(lines.len().saturating_sub(1));
    let line = lines[row];
    let relative_x = point.x.saturating_sub(metrics.text_bounds.x);
    let column = if relative_x <= 0 {
        0
    } else {
        relative_x
            .saturating_add(metrics.character_width / 2)
            .checked_div(metrics.character_width)
            .unwrap_or(0) as usize
    }
    .min(line.end.saturating_sub(line.start));
    line.start.saturating_add(column)
}

pub(crate) fn native_text_index_for_vertical_move(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_column: Option<usize>,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> (usize, usize) {
    let metrics = native_text_visual_metrics(target, dpi);
    let max_columns = metrics
        .text_bounds
        .width
        .checked_div(metrics.character_width)
        .unwrap_or(1)
        .max(1) as usize;
    let lines = text_lines(
        value,
        target.kind == ViewHitTargetKind::TextEditor,
        wrap,
        max_columns,
    );
    let (row, current_column) = text_position(caret.min(char_count(value)), &lines);
    let preferred_column = preferred_column.unwrap_or(current_column);
    let target_row = match direction {
        NativeTextVisualDirection::Up => row.saturating_sub(1),
        NativeTextVisualDirection::Down => row.saturating_add(1).min(lines.len() - 1),
    };
    let line = lines[target_row];
    let line_columns = line.end.saturating_sub(line.start);
    let column = if line.soft_wrap_after {
        preferred_column.min(line_columns.saturating_sub(1))
    } else {
        preferred_column.min(line_columns)
    };
    (line.start.saturating_add(column), preferred_column)
}

pub(crate) fn native_text_first_visible_row_for_caret(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor {
        return 0;
    }
    let (lines, visible_rows) = native_text_viewport_lines(target, value, wrap, dpi);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    let first_visible_row = first_visible_row.min(maximum_first);
    let (caret_row, _) = text_position(caret.min(char_count(value)), &lines);
    if caret_row < first_visible_row {
        caret_row
    } else if caret_row >= first_visible_row.saturating_add(visible_rows) {
        caret_row
            .saturating_add(1)
            .saturating_sub(visible_rows)
            .min(maximum_first)
    } else {
        first_visible_row
    }
}

pub(crate) fn native_text_scroll_visual_rows(
    target: ViewHitTarget,
    value: &str,
    first_visible_row: usize,
    row_delta: isize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor {
        return 0;
    }
    let (lines, visible_rows) = native_text_viewport_lines(target, value, wrap, dpi);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    if row_delta > 0 {
        first_visible_row
            .saturating_add(row_delta as usize)
            .min(maximum_first)
    } else if row_delta < 0 {
        first_visible_row.saturating_sub(row_delta.unsigned_abs())
    } else {
        first_visible_row.min(maximum_first)
    }
}

pub(crate) fn native_text_wheel_row_delta(delta_y: Dp) -> isize {
    if !delta_y.0.is_finite() || delta_y.0 == 0.0 {
        return 0;
    }
    let rows = (delta_y.0.abs() / 18.0).round().max(1.0) as isize;
    if delta_y.0 > 0.0 {
        rows
    } else {
        -rows
    }
}

#[cfg(test)]
pub(crate) fn decorate_native_text_edit_visuals(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    decorate_native_text_edit_visuals_in_viewport(plan, target, value, selection, 0, wrap, dpi)
}

pub(crate) fn decorate_native_text_edit_visuals_in_viewport(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    let geometry = native_text_visual_geometry_in_viewport(
        target,
        value,
        selection,
        first_visible_row,
        wrap,
        dpi,
    );
    if target.kind == ViewHitTargetKind::TextEditor {
        decorate_native_text_editor_viewport(
            plan,
            target,
            value,
            first_visible_row,
            wrap,
            dpi,
            &geometry,
        );
        return geometry;
    }
    if !geometry.selections.is_empty() {
        let text_index = plan.commands.iter().position(|command| match command {
            NativeDrawCommand::Text(text) => {
                (text.text == value || target.kind == ViewHitTargetKind::TextEditor)
                    && rect_contains(target.bounds, text.bounds)
            }
            #[cfg(feature = "password-box")]
            NativeDrawCommand::SecureText(text) => rect_contains(target.bounds, text.bounds),
            _ => false,
        });
        let insertion_index = text_index.unwrap_or(plan.commands.len());
        for (offset, rect) in geometry.selections.iter().copied().enumerate() {
            plan.commands.insert(
                insertion_index + offset,
                NativeDrawCommand::FillRect {
                    rect,
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 64,
                    },
                },
            );
        }
    }
    plan.push(NativeDrawCommand::FillRect {
        rect: geometry.caret,
        fill: NativeDrawFill::Role(ColorRole::Accent),
    });
    geometry
}

fn decorate_native_text_editor_viewport(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    geometry: &NativeTextVisualGeometry,
) {
    let text_index = plan.commands.iter().position(|command| {
        matches!(command, NativeDrawCommand::Text(text) if rect_contains(target.bounds, text.bounds))
    });
    let Some(text_index) = text_index else {
        return;
    };
    let Some(mut style) = plan.commands.iter().find_map(|command| match command {
        NativeDrawCommand::Text(text) if rect_contains(target.bounds, text.bounds) => {
            Some(text.style)
        }
        _ => None,
    }) else {
        return;
    };
    plan.commands.retain(|command| {
        !matches!(command, NativeDrawCommand::Text(text) if rect_contains(target.bounds, text.bounds))
    });

    let metrics = native_text_visual_metrics(target, dpi);
    let (lines, visible_rows) = native_text_viewport_lines(target, value, wrap, dpi);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    let first_visible_row = first_visible_row.min(maximum_first);
    style.wrap = crate::TextWrap::NoWrap;
    style.vertical_align = crate::VerticalAlign::Start;
    style.ellipsis = false;

    let mut commands = Vec::new();
    commands.push(NativeDrawCommand::PushClip {
        rect: metrics.text_bounds,
    });
    commands.extend(
        geometry
            .selections
            .iter()
            .copied()
            .map(|rect| NativeDrawCommand::FillRect {
                rect,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 64,
                },
            }),
    );
    for (row, line) in lines
        .iter()
        .copied()
        .enumerate()
        .skip(first_visible_row)
        .take(visible_rows)
    {
        commands.push(NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
            char_slice(value, line.start, line.end),
            Rect {
                x: metrics.text_bounds.x,
                y: visual_row_y(
                    metrics.text_bounds.y,
                    row,
                    first_visible_row,
                    metrics.line_height,
                ),
                width: metrics.text_bounds.width,
                height: metrics.line_height,
            },
            style,
        )));
    }
    commands.push(NativeDrawCommand::FillRect {
        rect: geometry.caret,
        fill: NativeDrawFill::Role(ColorRole::Accent),
    });
    commands.push(NativeDrawCommand::PopClip);
    plan.commands.splice(text_index..text_index, commands);
}

pub(crate) fn decorate_native_focus_ring(
    plan: &mut NativeDrawPlan,
    interaction_plan: &ViewInteractionPlan,
    focused_widget: Option<WidgetId>,
    dpi: Dpi,
) -> Option<Rect> {
    #[allow(unused_mut)]
    let mut target = interaction_plan.hit_target_for_widget(focused_widget?)?;
    #[cfg(feature = "grid-view")]
    if target.kind == ViewHitTargetKind::GridView {
        target = interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|candidate| {
                candidate.widget == target.widget
                    && matches!(candidate.kind, ViewHitTargetKind::GridViewItem { .. })
            })
            .unwrap_or(target);
    }
    #[cfg(feature = "dialog")]
    if matches!(
        target.kind,
        ViewHitTargetKind::ContentDialog
            | ViewHitTargetKind::ContentDialogScrim
            | ViewHitTargetKind::ContentDialogButton { .. }
    ) {
        return None;
    }
    #[cfg(feature = "toast")]
    if matches!(
        target.kind,
        ViewHitTargetKind::Toast | ViewHitTargetKind::ToastAction | ViewHitTargetKind::ToastClose
    ) {
        return None;
    }
    #[cfg(feature = "info-bar")]
    if matches!(
        target.kind,
        ViewHitTargetKind::InfoBar
            | ViewHitTargetKind::InfoBarAction
            | ViewHitTargetKind::InfoBarClose
    ) {
        return None;
    }
    #[cfg(feature = "breadcrumb")]
    if matches!(
        target.kind,
        ViewHitTargetKind::BreadcrumbOverflow
            | ViewHitTargetKind::BreadcrumbItem { .. }
            | ViewHitTargetKind::BreadcrumbOverflowItem { .. }
    ) {
        return None;
    }
    #[cfg(feature = "teaching-tip")]
    if matches!(
        target.kind,
        ViewHitTargetKind::TeachingTip
            | ViewHitTargetKind::TeachingTipAction
            | ViewHitTargetKind::TeachingTipClose
    ) {
        return None;
    }
    #[cfg(feature = "auto-suggest")]
    if target.kind == ViewHitTargetKind::AutoSuggestBox
        && crate::ZsAutoSuggestPlatformStyle::current()
            == crate::ZsAutoSuggestPlatformStyle::Windows
    {
        let height = Dp::new(2.0).to_px(dpi).round_i32().max(1);
        let indicator = Rect {
            x: target.bounds.x,
            y: target
                .bounds
                .y
                .saturating_add(target.bounds.height)
                .saturating_sub(height),
            width: target.bounds.width,
            height,
        };
        plan.push(NativeDrawCommand::FillRect {
            rect: indicator,
            fill: NativeDrawFill::Role(ColorRole::Accent),
        });
        return Some(indicator);
    }
    let requested_inset = Dp::new(1.0).to_px(dpi).round_i32().max(1);
    let maximum_inset = (target.bounds.width.min(target.bounds.height).max(1) - 1) / 2;
    let inset = requested_inset.min(maximum_inset.max(0));
    let ring = Rect {
        x: target.bounds.x.saturating_add(inset),
        y: target.bounds.y.saturating_add(inset),
        width: target.bounds.width.saturating_sub(inset.saturating_mul(2)),
        height: target.bounds.height.saturating_sub(inset.saturating_mul(2)),
    };
    let width = Dp::new(2.0).to_px(dpi).round_i32().max(1);
    plan.push(NativeDrawCommand::StrokeRect {
        rect: ring,
        stroke: NativeDrawFill::Role(ColorRole::Accent),
        width,
    });
    Some(ring)
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "command-palette",
    feature = "date-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "password-box",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast",
    feature = "toggle-button",
    feature = "table",
    feature = "tree"
))]
pub(crate) type NativePointerVisualKey = (WidgetId, ViewHitTargetKind);

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "command-palette",
    feature = "date-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "password-box",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast",
    feature = "toggle-button",
    feature = "table",
    feature = "tree"
))]
pub(crate) fn native_pointer_visual_key(target: ViewHitTarget) -> Option<NativePointerVisualKey> {
    let supported = false;
    #[cfg(feature = "auto-suggest")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::AutoSuggestSearch
                | ViewHitTargetKind::AutoSuggestClear
                | ViewHitTargetKind::AutoSuggestSuggestion { .. }
        );
    #[cfg(feature = "breadcrumb")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::BreadcrumbOverflow
                | ViewHitTargetKind::BreadcrumbItem { .. }
                | ViewHitTargetKind::BreadcrumbOverflowItem { .. }
        );
    #[cfg(feature = "color-picker")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::ColorPicker | ViewHitTargetKind::ColorPickerChannel { .. }
        );
    #[cfg(feature = "command-palette")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::CommandPaletteClear | ViewHitTargetKind::CommandPaletteItem { .. }
        );
    #[cfg(feature = "grid-view")]
    let supported = supported || matches!(target.kind, ViewHitTargetKind::GridViewItem { .. });
    #[cfg(feature = "toggle-button")]
    let supported = supported || target.kind == ViewHitTargetKind::ToggleButton;
    #[cfg(feature = "tree")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::TreeNode { .. } | ViewHitTargetKind::TreeNodeExpander { .. }
        );
    #[cfg(feature = "table")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::TableHeader { .. } | ViewHitTargetKind::TableRow { .. }
        );
    #[cfg(feature = "dialog")]
    let supported =
        supported || matches!(target.kind, ViewHitTargetKind::ContentDialogButton { .. });
    #[cfg(feature = "toast")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::ToastAction | ViewHitTargetKind::ToastClose
        );
    #[cfg(feature = "info-bar")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::InfoBarAction | ViewHitTargetKind::InfoBarClose
        );
    #[cfg(feature = "teaching-tip")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::TeachingTipAction | ViewHitTargetKind::TeachingTipClose
        );
    #[cfg(feature = "password-box")]
    let supported = supported || target.kind == ViewHitTargetKind::PasswordBoxReveal;
    #[cfg(feature = "date-picker")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::DatePicker
                | ViewHitTargetKind::DatePickerDay { .. }
                | ViewHitTargetKind::DatePickerPreviousMonth
                | ViewHitTargetKind::DatePickerNextMonth
        );
    #[cfg(feature = "tabs")]
    let supported = supported || matches!(target.kind, ViewHitTargetKind::Tab { .. });
    #[cfg(feature = "time-picker")]
    let supported = supported
        || matches!(
            target.kind,
            ViewHitTargetKind::TimePicker | ViewHitTargetKind::TimePickerChoice { .. }
        );
    supported.then_some((target.widget, target.kind))
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "command-palette",
    feature = "date-picker",
    feature = "dialog",
    feature = "grid-view",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "password-box",
    feature = "tabs",
    feature = "time-picker",
    feature = "toast",
    feature = "toggle-button",
    feature = "table",
    feature = "tree"
))]
pub(crate) fn decorate_native_pointer_visuals(
    plan: &mut NativeDrawPlan,
    interaction_plan: &ViewInteractionPlan,
    hovered: Option<NativePointerVisualKey>,
    pressed: Option<NativePointerVisualKey>,
    dpi: Dpi,
) -> usize {
    let active = match pressed {
        Some(pressed) if hovered == Some(pressed) => Some((pressed, true)),
        Some(_) => None,
        None => hovered.map(|hovered| (hovered, false)),
    };
    let Some(((widget, kind), is_pressed)) = active else {
        return 0;
    };
    let Some(target) = interaction_plan
        .hit_targets
        .iter()
        .copied()
        .find(|target| target.widget == widget && target.kind == kind)
    else {
        return 0;
    };

    #[cfg(feature = "toggle-button")]
    if kind == ViewHitTargetKind::ToggleButton {
        if let Some(NativeDrawCommand::RoundRect { fill, stroke, .. }) =
            plan.commands.iter_mut().rfind(|command| {
                matches!(command, NativeDrawCommand::RoundRect { rect, .. }
                    if rect_contains(*rect, target.bounds))
            })
        {
            let checked = *fill == NativeDrawFill::Role(ColorRole::Accent);
            *stroke = Some(NativeDrawFill::Role(if checked {
                ColorRole::AccentText
            } else if is_pressed {
                ColorRole::Accent
            } else {
                ColorRole::PrimaryText
            }));
            return 1;
        }
    }

    #[cfg(feature = "date-picker")]
    if matches!(kind, ViewHitTargetKind::DatePickerDay { .. }) {
        if let Some(command) = plan.commands.iter_mut().find(|command| {
            matches!(command, NativeDrawCommand::RoundRect { rect, fill: NativeDrawFill::Role(ColorRole::Accent), .. }
                if rect_contains(target.bounds, *rect))
        }) {
            let NativeDrawCommand::RoundRect { stroke, .. } = command else {
                unreachable!("matched date highlight must remain a round rectangle")
            };
            *stroke = Some(NativeDrawFill::Role(if is_pressed {
                ColorRole::PrimaryText
            } else {
                ColorRole::AccentText
            }));
            return 1;
        }
    }

    let backdrop_index = plan.commands.iter().rposition(|command| {
        matches!(command, NativeDrawCommand::RoundRect { rect, .. }
            if rect_contains(*rect, target.bounds))
    });
    let Some(backdrop_index) = backdrop_index else {
        return 0;
    };
    #[cfg(feature = "date-picker")]
    let rect = match kind {
        ViewHitTargetKind::DatePickerDay { .. } => {
            let diameter = Dp::new(32.0)
                .to_px(dpi)
                .round_i32()
                .min(target.bounds.width)
                .min(target.bounds.height)
                .max(1);
            Rect {
                x: target.bounds.x + (target.bounds.width - diameter) / 2,
                y: target.bounds.y + (target.bounds.height - diameter) / 2,
                width: diameter,
                height: diameter,
            }
        }
        _ => target.bounds,
    };
    #[cfg(not(feature = "date-picker"))]
    let rect = target.bounds;
    #[cfg(feature = "date-picker")]
    let is_date_cell = matches!(kind, ViewHitTargetKind::DatePickerDay { .. });
    #[cfg(not(feature = "date-picker"))]
    let is_date_cell = false;
    let radius = if is_date_cell {
        rect.width.min(rect.height) / 2
    } else {
        Dp::new(4.0).to_px(dpi).round_i32().max(1)
    };
    plan.commands.insert(
        backdrop_index + 1,
        NativeDrawCommand::RoundFill {
            rect,
            fill: NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: if is_pressed { 28 } else { 14 },
            },
            radius,
        },
    );
    1
}

#[derive(Debug, Clone, Copy)]
struct NativeTextLine {
    start: usize,
    end: usize,
    soft_wrap_after: bool,
}

#[derive(Debug, Clone, Copy)]
struct NativeTextVisualMetrics {
    text_bounds: Rect,
    character_width: i32,
    line_height: i32,
}

fn native_text_visual_metrics(target: ViewHitTarget, dpi: Dpi) -> NativeTextVisualMetrics {
    let inset = Dp::new(8.0).to_px(dpi).round_i32().max(1);
    let bounds = target.bounds;
    #[cfg(feature = "number-box")]
    let bounds = if target.kind == ViewHitTargetKind::NumberBox {
        crate::zs_number_box_render_plan(
            target.bounds,
            crate::ZsNumberBoxPlatformStyle::current(),
            dpi,
        )
        .text_bounds
    } else {
        bounds
    };
    NativeTextVisualMetrics {
        text_bounds: Rect {
            x: bounds.x.saturating_add(inset),
            y: bounds.y.saturating_add(inset),
            width: bounds.width.saturating_sub(inset.saturating_mul(2)).max(1),
            height: bounds.height.saturating_sub(inset.saturating_mul(2)).max(1),
        },
        character_width: Dp::new(8.0).to_px(dpi).round_i32().max(1),
        line_height: Dp::new(18.0).to_px(dpi).round_i32().max(1),
    }
}

fn native_text_viewport_lines(
    target: ViewHitTarget,
    value: &str,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> (Vec<NativeTextLine>, usize) {
    let metrics = native_text_visual_metrics(target, dpi);
    let max_columns = metrics
        .text_bounds
        .width
        .checked_div(metrics.character_width)
        .unwrap_or(1)
        .max(1) as usize;
    let lines = text_lines(
        value,
        target.kind == ViewHitTargetKind::TextEditor,
        wrap,
        max_columns,
    );
    let visible_rows = metrics
        .text_bounds
        .height
        .saturating_add(metrics.line_height.saturating_sub(1))
        .checked_div(metrics.line_height)
        .unwrap_or(1)
        .max(1) as usize;
    (lines, visible_rows)
}

fn visual_row_y(origin: i32, row: usize, first_visible_row: usize, line_height: i32) -> i32 {
    if row >= first_visible_row {
        origin.saturating_add(
            i32::try_from(row.saturating_sub(first_visible_row))
                .unwrap_or(i32::MAX)
                .saturating_mul(line_height),
        )
    } else {
        origin.saturating_sub(
            i32::try_from(first_visible_row.saturating_sub(row))
                .unwrap_or(i32::MAX)
                .saturating_mul(line_height),
        )
    }
}

fn char_slice(value: &str, start: usize, end: usize) -> String {
    value
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn text_lines(
    value: &str,
    multiline: bool,
    wrap: crate::TextWrap,
    max_columns: usize,
) -> Vec<NativeTextLine> {
    if !multiline {
        return vec![NativeTextLine {
            start: 0,
            end: char_count(value),
            soft_wrap_after: false,
        }];
    }
    let characters = value.chars().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, character) in characters.iter().copied().enumerate() {
        if character == '\n' {
            push_text_lines(&mut lines, &characters, start, index, wrap, max_columns);
            start = index + 1;
        }
    }
    push_text_lines(
        &mut lines,
        &characters,
        start,
        characters.len(),
        wrap,
        max_columns,
    );
    lines
}

fn push_text_lines(
    lines: &mut Vec<NativeTextLine>,
    characters: &[char],
    start: usize,
    end: usize,
    wrap: crate::TextWrap,
    max_columns: usize,
) {
    let max_columns = max_columns.max(1);
    if wrap == crate::TextWrap::NoWrap || end.saturating_sub(start) <= max_columns {
        lines.push(NativeTextLine {
            start,
            end,
            soft_wrap_after: false,
        });
        return;
    }

    let mut cursor = start;
    while end.saturating_sub(cursor) > max_columns {
        let limit = cursor.saturating_add(max_columns).min(end);
        let break_at = characters[cursor..limit]
            .iter()
            .rposition(|character| character.is_whitespace())
            .map(|offset| cursor.saturating_add(offset).saturating_add(1))
            .filter(|candidate| *candidate > cursor)
            .unwrap_or(limit);
        lines.push(NativeTextLine {
            start: cursor,
            end: break_at,
            soft_wrap_after: true,
        });
        cursor = break_at;
    }
    lines.push(NativeTextLine {
        start: cursor,
        end,
        soft_wrap_after: false,
    });
}

fn text_position(index: usize, lines: &[NativeTextLine]) -> (usize, usize) {
    for (row, line) in lines.iter().copied().enumerate() {
        if index < line.end || (index == line.end && !line.soft_wrap_after) {
            return (row, index.saturating_sub(line.start));
        }
    }
    lines
        .last()
        .map(|line| {
            (
                lines.len().saturating_sub(1),
                index.min(line.end).saturating_sub(line.start),
            )
        })
        .unwrap_or((0, 0))
}

pub(crate) fn rect_contains(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x.saturating_add(inner.width) <= outer.x.saturating_add(outer.width)
        && inner.y.saturating_add(inner.height) <= outer.y.saturating_add(outer.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ViewHitTarget, ViewHitTargetKind};

    #[test]
    fn focus_ring_uses_semantic_accent_and_insets_target_bounds() {
        let widget = WidgetId::new(91);
        let interaction_plan = ViewInteractionPlan::new([ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 10,
                y: 20,
                width: 120,
                height: 32,
            },
            ViewHitTargetKind::Button,
        )]);
        let mut plan = NativeDrawPlan::default();

        let ring =
            decorate_native_focus_ring(&mut plan, &interaction_plan, Some(widget), Dpi::standard())
                .expect("focused target should produce a ring");

        assert_eq!(ring.x, 11);
        assert_eq!(ring.y, 21);
        assert_eq!(ring.width, 118);
        assert_eq!(ring.height, 30);
        assert!(matches!(
            plan.commands.as_slice(),
            [NativeDrawCommand::StrokeRect {
                rect,
                stroke: NativeDrawFill::Role(ColorRole::Accent),
                width: 2,
            }] if *rect == ring
        ));
    }

    #[test]
    #[cfg(feature = "grid-view")]
    fn grid_view_focus_ring_follows_the_selected_item_target_after_the_root() {
        let widget = WidgetId::new(912);
        let root = Rect {
            x: 10,
            y: 20,
            width: 420,
            height: 240,
        };
        let selected = Rect {
            x: 154,
            y: 20,
            width: 132,
            height: 112,
        };
        let interaction_plan = ViewInteractionPlan::new([
            ViewHitTarget::with_kind(widget, root, ViewHitTargetKind::GridView),
            ViewHitTarget::with_kind(
                widget,
                selected,
                ViewHitTargetKind::GridViewItem {
                    item: crate::ZsGridViewItemId::new(2),
                },
            ),
        ]);
        let mut plan = NativeDrawPlan::default();

        let ring =
            decorate_native_focus_ring(&mut plan, &interaction_plan, Some(widget), Dpi::standard())
                .expect("focused grid view should outline its selected tile");

        assert_eq!(
            ring,
            Rect {
                x: 155,
                y: 21,
                width: 130,
                height: 110,
            }
        );
    }

    #[test]
    #[cfg(feature = "dialog")]
    fn content_dialog_keeps_internal_focus_and_button_pointer_visuals_in_overlay() {
        let widget = WidgetId::new(911);
        let surface = Rect {
            x: 80,
            y: 60,
            width: 320,
            height: 180,
        };
        let button = Rect {
            x: 280,
            y: 190,
            width: 96,
            height: 36,
        };
        let button_kind = ViewHitTargetKind::ContentDialogButton {
            button: crate::ZsContentDialogButton::Primary,
        };
        let interaction_plan = ViewInteractionPlan::new([
            ViewHitTarget::with_kind(widget, surface, ViewHitTargetKind::ContentDialog),
            ViewHitTarget::with_kind(widget, button, button_kind),
        ]);
        let mut focus_plan = NativeDrawPlan::default();
        assert_eq!(
            decorate_native_focus_ring(
                &mut focus_plan,
                &interaction_plan,
                Some(widget),
                Dpi::standard(),
            ),
            None
        );
        assert!(focus_plan.commands.is_empty());

        let mut pointer_plan = NativeDrawPlan::new([
            NativeDrawCommand::RoundRect {
                rect: button,
                fill: NativeDrawFill::Role(ColorRole::Control),
                stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                radius: 4,
            },
            NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                "Continue",
                button,
                crate::SemanticTextStyle::body(),
            )),
        ]);
        let key = (widget, button_kind);
        assert_eq!(
            decorate_native_pointer_visuals(
                &mut pointer_plan,
                &interaction_plan,
                Some(key),
                Some(key),
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            pointer_plan.commands.as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::PrimaryText,
                        alpha: 28,
                    },
                    ..
                },
                NativeDrawCommand::Text(_),
            ]
        ));
    }

    #[test]
    #[cfg(all(feature = "auto-suggest", target_os = "windows"))]
    fn windows_auto_suggest_focus_uses_winui_bottom_accent_indicator() {
        let widget = WidgetId::new(92);
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 180,
            height: 32,
        };
        let interaction_plan = ViewInteractionPlan::new([ViewHitTarget::with_kind(
            widget,
            bounds,
            ViewHitTargetKind::AutoSuggestBox,
        )]);
        let mut plan = NativeDrawPlan::default();

        let indicator =
            decorate_native_focus_ring(&mut plan, &interaction_plan, Some(widget), Dpi::standard())
                .expect("focused auto-suggest should produce an indicator");

        assert_eq!(
            indicator,
            Rect {
                y: 50,
                height: 2,
                ..bounds
            }
        );
        assert!(matches!(
            plan.commands.as_slice(),
            [NativeDrawCommand::FillRect {
                rect,
                fill: NativeDrawFill::Role(ColorRole::Accent),
            }] if *rect == indicator
        ));
    }

    #[test]
    #[cfg(feature = "auto-suggest")]
    fn auto_suggest_pointer_visual_marks_the_active_suggestion_row() {
        let widget = WidgetId::new(94);
        let popup = Rect {
            x: 10,
            y: 20,
            width: 220,
            height: 120,
        };
        let row = Rect {
            x: 14,
            y: 56,
            width: 212,
            height: 36,
        };
        let kind = ViewHitTargetKind::AutoSuggestSuggestion {
            suggestion: crate::ZsAutoSuggestionId::new(7),
        };
        let interaction_plan =
            ViewInteractionPlan::new([ViewHitTarget::with_kind(widget, row, kind)]);
        let mut plan = NativeDrawPlan::new([NativeDrawCommand::RoundRect {
            rect: popup,
            fill: NativeDrawFill::Role(ColorRole::Surface),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: 4,
        }]);

        let changed = decorate_native_pointer_visuals(
            &mut plan,
            &interaction_plan,
            Some((widget, kind)),
            None,
            Dpi::standard(),
        );

        assert_eq!(changed, 1);
        assert!(matches!(
            plan.commands.get(1),
            Some(NativeDrawCommand::RoundFill {
                rect,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 14,
                },
                radius: 4,
            }) if *rect == row
        ));
    }

    #[test]
    #[cfg(feature = "tree")]
    fn tree_pointer_visual_uses_the_platform_row_geometry() {
        let widget = WidgetId::new(96);
        let node = crate::ZsTreeNodeId::new(2);
        let roots =
            [crate::ZsTreeNode::new(1, "Root").children([crate::ZsTreeNode::new(node, "Child")])];
        let expanded = std::collections::BTreeSet::from([crate::ZsTreeNodeId::new(1)]);
        let render = crate::zs_tree_view_render_plan(
            Rect {
                x: 10,
                y: 20,
                width: 220,
                height: 96,
            },
            &roots,
            &expanded,
            Some(node),
            crate::ZsTreePlatformStyle::Windows,
            Dpi::standard(),
        );
        let row = render.rows[1].bounds;
        let kind = ViewHitTargetKind::TreeNode { node };
        let interaction = ViewInteractionPlan::new([ViewHitTarget::with_kind(widget, row, kind)]);
        let mut plan = crate::zs_tree_view_native_draw_plan(&render);

        assert_eq!(
            decorate_native_pointer_visuals(
                &mut plan,
                &interaction,
                Some((widget, kind)),
                None,
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            plan.commands.get(1),
            Some(NativeDrawCommand::RoundFill {
                rect,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 14,
                },
                ..
            }) if *rect == row
        ));
    }

    #[test]
    #[cfg(feature = "table")]
    fn table_pointer_visual_uses_the_platform_row_geometry() {
        let widget = WidgetId::new(97);
        let row_id = crate::ZsTableRowId::new(2);
        let render = crate::zs_table_render_plan(
            Rect {
                x: 10,
                y: 20,
                width: 240,
                height: 120,
            },
            &[crate::ZsTableColumn::new(1, "Name")],
            &[
                crate::ZsTableRow::new(1, ["Cargo.toml"]),
                crate::ZsTableRow::new(row_id, ["src"]),
            ],
            Some(row_id),
            None,
            crate::ZsTablePlatformStyle::Windows,
            Dpi::standard(),
        );
        let row = render.rows[1].bounds;
        let kind = ViewHitTargetKind::TableRow { row: row_id };
        let interaction = ViewInteractionPlan::new([ViewHitTarget::with_kind(widget, row, kind)]);
        let mut plan = crate::zs_table_native_draw_plan(&render);

        assert_eq!(
            decorate_native_pointer_visuals(
                &mut plan,
                &interaction,
                Some((widget, kind)),
                None,
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            plan.commands.get(1),
            Some(NativeDrawCommand::RoundFill {
                rect,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: 14,
                },
                ..
            }) if *rect == row
        ));
    }

    #[test]
    #[cfg(feature = "toggle-button")]
    fn toggle_button_pointer_visual_preserves_checked_accent_fill() {
        let widget = WidgetId::new(93);
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 120,
            height: 36,
        };
        let interaction_plan = ViewInteractionPlan::new([ViewHitTarget::with_kind(
            widget,
            bounds,
            ViewHitTargetKind::ToggleButton,
        )]);
        let render = crate::zs_toggle_button_render_plan(
            bounds,
            true,
            crate::ZsToggleButtonPlatformStyle::Windows,
            Dpi::standard(),
        );
        let mut plan = crate::zs_toggle_button_native_draw_plan(&render, "Pin");

        let changed = decorate_native_pointer_visuals(
            &mut plan,
            &interaction_plan,
            Some((widget, ViewHitTargetKind::ToggleButton)),
            Some((widget, ViewHitTargetKind::ToggleButton)),
            Dpi::standard(),
        );

        assert_eq!(changed, 1);
        assert_eq!(plan.command_count(), 3);
        assert!(matches!(
            plan.commands.first(),
            Some(NativeDrawCommand::RoundRect {
                fill: NativeDrawFill::Role(ColorRole::Accent),
                stroke: Some(NativeDrawFill::Role(ColorRole::AccentText)),
                ..
            })
        ));
    }

    #[test]
    #[cfg(feature = "date-picker")]
    fn date_picker_pointer_visuals_stay_below_text_and_preserve_selected_fill() {
        let widget = WidgetId::new(95);
        let date = crate::ZsDate::new(2026, 7, 13).expect("date should be valid");
        let popup = Rect {
            x: 0,
            y: 0,
            width: 296,
            height: 332,
        };
        let day = Rect {
            x: 40,
            y: 78,
            width: 42,
            height: 42,
        };
        let target =
            ViewHitTarget::with_kind(widget, day, ViewHitTargetKind::DatePickerDay { date });
        let interaction = ViewInteractionPlan::new([target]);
        let mut rest = NativeDrawPlan::new([
            NativeDrawCommand::RoundRect {
                rect: popup,
                fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                radius: 8,
            },
            NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                "13",
                day,
                crate::SemanticTextStyle::body(),
            )),
        ]);

        assert_eq!(
            decorate_native_pointer_visuals(
                &mut rest,
                &interaction,
                Some((widget, target.kind)),
                None,
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            rest.commands.as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::PrimaryText,
                        alpha: 14,
                    },
                    ..
                },
                NativeDrawCommand::Text(_),
            ]
        ));

        let mut selected = NativeDrawPlan::new([
            NativeDrawCommand::RoundRect {
                rect: popup,
                fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                radius: 8,
            },
            NativeDrawCommand::RoundRect {
                rect: Rect {
                    x: 45,
                    y: 83,
                    width: 32,
                    height: 32,
                },
                fill: NativeDrawFill::Role(ColorRole::Accent),
                stroke: None,
                radius: 16,
            },
        ]);
        assert_eq!(
            decorate_native_pointer_visuals(
                &mut selected,
                &interaction,
                Some((widget, target.kind)),
                Some((widget, target.kind)),
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            selected.commands.get(1),
            Some(NativeDrawCommand::RoundRect {
                fill: NativeDrawFill::Role(ColorRole::Accent),
                stroke: Some(NativeDrawFill::Role(ColorRole::PrimaryText)),
                ..
            })
        ));
    }

    #[test]
    #[cfg(feature = "time-picker")]
    fn time_picker_choice_pointer_visual_stays_below_popup_text() {
        let widget = WidgetId::new(96);
        let choice = Rect {
            x: 80,
            y: 40,
            width: 80,
            height: 40,
        };
        let target = ViewHitTarget::with_kind(
            widget,
            choice,
            ViewHitTargetKind::TimePickerChoice {
                value: crate::ZsTime::new(9, 45).unwrap(),
            },
        );
        let interaction = ViewInteractionPlan::new([target]);
        let mut plan = NativeDrawPlan::new([
            NativeDrawCommand::RoundRect {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 240,
                    height: 122,
                },
                fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                radius: 8,
            },
            NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                "45",
                choice,
                crate::SemanticTextStyle::body(),
            )),
        ]);

        assert_eq!(
            decorate_native_pointer_visuals(
                &mut plan,
                &interaction,
                Some((widget, target.kind)),
                None,
                Dpi::standard(),
            ),
            1
        );
        assert!(matches!(
            plan.commands.as_slice(),
            [
                NativeDrawCommand::RoundRect { .. },
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::PrimaryText,
                        alpha: 14,
                    },
                    ..
                },
                NativeDrawCommand::Text(_),
            ]
        ));
    }

    #[test]
    fn text_edit_visuals_place_selection_behind_text_and_caret_at_active_end() {
        let widget = WidgetId::new(92);
        let target = ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 40,
            },
            ViewHitTargetKind::Textbox,
        );
        let mut plan =
            NativeDrawPlan::new([NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                "A中文Z",
                Rect {
                    x: 8,
                    y: 8,
                    width: 184,
                    height: 24,
                },
                crate::SemanticTextStyle::body(),
            ))]);

        let geometry = decorate_native_text_edit_visuals(
            &mut plan,
            target,
            "A中文Z",
            NativeTextSelection {
                anchor: 1,
                caret: 3,
            },
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );

        assert_eq!(geometry.selections.len(), 1);
        assert_eq!(geometry.caret.x, 32);
        assert!(matches!(
            plan.commands.as_slice(),
            [
                NativeDrawCommand::FillRect {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 64,
                    },
                    ..
                },
                NativeDrawCommand::Text(_),
                NativeDrawCommand::FillRect {
                    fill: NativeDrawFill::Role(ColorRole::Accent),
                    ..
                }
            ]
        ));
    }

    #[test]
    #[cfg(feature = "password-box")]
    fn password_text_geometry_reserves_space_only_when_reveal_target_exists() {
        let widget = WidgetId::new(920);
        let target = ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 10,
                y: 0,
                width: 200,
                height: 36,
            },
            ViewHitTargetKind::PasswordBox,
        );
        let hidden = ViewInteractionPlan::new([target]);
        assert_eq!(native_text_visual_target(target, &hidden).bounds.width, 200);

        let peek = ViewInteractionPlan::new([
            target,
            ViewHitTarget::with_kind(
                widget,
                Rect {
                    x: 178,
                    y: 0,
                    width: 32,
                    height: 36,
                },
                ViewHitTargetKind::PasswordBoxReveal,
            ),
        ]);
        assert_eq!(native_text_visual_target(target, &peek).bounds.width, 168);
    }

    #[test]
    fn text_pointer_hit_testing_uses_unicode_indices_and_clamps_multiline_rows() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(93),
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 80,
            },
            ViewHitTargetKind::TextEditor,
        );

        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: 20, y: 10 },
                crate::TextWrap::NoWrap,
                Dpi::standard()
            ),
            2
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: 16, y: 30 },
                crate::TextWrap::NoWrap,
                Dpi::standard()
            ),
            4
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: 500, y: 500 },
                crate::TextWrap::NoWrap,
                Dpi::standard()
            ),
            5
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: -20, y: -20 },
                crate::TextWrap::NoWrap,
                Dpi::standard()
            ),
            0
        );
    }

    #[test]
    fn text_editor_word_wrap_aligns_caret_and_pointer_with_soft_lines() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(94),
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 80,
            },
            ViewHitTargetKind::TextEditor,
        );
        let selection = NativeTextSelection::collapsed(4);

        let wrapped = native_text_visual_geometry(
            target,
            "one two",
            selection,
            crate::TextWrap::Word,
            Dpi::standard(),
        );
        let unwrapped = native_text_visual_geometry(
            target,
            "one two",
            selection,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );

        assert_eq!((wrapped.caret.x, wrapped.caret.y), (8, 26));
        assert_eq!((unwrapped.caret.x, unwrapped.caret.y), (39, 8));
        assert_eq!(
            native_text_index_for_point(
                target,
                "one two",
                Point { x: 16, y: 30 },
                crate::TextWrap::Word,
                Dpi::standard(),
            ),
            5
        );
    }

    #[test]
    fn vertical_text_navigation_uses_visual_rows_and_preserves_the_column() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(95),
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 120,
            },
            ViewHitTargetKind::TextEditor,
        );
        let value = "abcdef\nx\nuvwxyz";

        let (second_visual_row, preferred) = native_text_index_for_vertical_move(
            target,
            value,
            2,
            NativeTextVisualDirection::Down,
            None,
            crate::TextWrap::Word,
            Dpi::standard(),
        );
        let (short_hard_line, preferred) = native_text_index_for_vertical_move(
            target,
            value,
            second_visual_row,
            NativeTextVisualDirection::Down,
            Some(preferred),
            crate::TextWrap::Word,
            Dpi::standard(),
        );
        let (next_wrapped_line, _) = native_text_index_for_vertical_move(
            target,
            value,
            short_hard_line,
            NativeTextVisualDirection::Down,
            Some(preferred),
            crate::TextWrap::Word,
            Dpi::standard(),
        );

        assert_eq!(second_visual_row, 6);
        assert_eq!(short_hard_line, 8);
        assert_eq!(next_wrapped_line, 11);
        assert_eq!(preferred, 2);
    }

    #[test]
    fn text_editor_viewport_scrolls_visual_rows_and_keeps_the_caret_visible() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(96),
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 52,
            },
            ViewHitTargetKind::TextEditor,
        );
        let value = "row0\nrow1\nrow2\nrow3";
        let caret = char_count(value);
        let first_visible = native_text_first_visible_row_for_caret(
            target,
            value,
            caret,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );

        assert_eq!(first_visible, 2);
        assert_eq!(
            native_text_index_for_point_in_viewport(
                target,
                value,
                Point { x: 8, y: 10 },
                first_visible,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
            ),
            10
        );
        assert_eq!(
            native_text_scroll_visual_rows(
                target,
                value,
                first_visible,
                -3,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
            ),
            0
        );
        assert_eq!(native_text_wheel_row_delta(Dp::new(48.0)), 3);

        let mut plan =
            NativeDrawPlan::new([NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                value,
                Rect {
                    x: 8,
                    y: 8,
                    width: 32,
                    height: 36,
                },
                crate::SemanticTextStyle::body(),
            ))]);
        let geometry = decorate_native_text_edit_visuals_in_viewport(
            &mut plan,
            target,
            value,
            NativeTextSelection::collapsed(caret),
            first_visible,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );
        let visible_text = plan
            .commands
            .iter()
            .filter_map(|command| match command {
                NativeDrawCommand::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(visible_text, vec!["row2", "row3"]);
        assert_eq!(geometry.caret.y, 26);
        assert!(matches!(
            plan.commands.first(),
            Some(NativeDrawCommand::PushClip { .. })
        ));
        assert!(matches!(
            plan.commands.last(),
            Some(NativeDrawCommand::PopClip)
        ));
    }
}
