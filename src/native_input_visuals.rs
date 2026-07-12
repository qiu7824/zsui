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

pub(crate) fn native_text_visual_geometry(
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics(target, dpi);
    let text_bounds = metrics.text_bounds;
    let character_width = metrics.character_width;
    let line_height = metrics.line_height;
    let selection = selection.clamp(value);
    let (caret_row, caret_column) = text_position(value, selection.caret, multiline);
    let caret_x = text_bounds
        .x
        .saturating_add((caret_column as i32).saturating_mul(character_width))
        .min(
            text_bounds
                .x
                .saturating_add(text_bounds.width)
                .saturating_sub(1),
        );
    let caret_y = text_bounds
        .y
        .saturating_add((caret_row as i32).saturating_mul(line_height))
        .min(
            text_bounds
                .y
                .saturating_add(text_bounds.height)
                .saturating_sub(1),
        );
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
        for (row, line) in text_lines(value, multiline).into_iter().enumerate() {
            let overlap_start = start.max(line.start);
            let overlap_end = end.min(line.end);
            if overlap_start >= overlap_end && !(end > line.end && start <= line.end) {
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
            let y = text_bounds
                .y
                .saturating_add((row as i32).saturating_mul(line_height));
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

pub(crate) fn native_text_index_for_point(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    dpi: Dpi,
) -> usize {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics(target, dpi);
    let lines = text_lines(value, multiline);
    let row = if multiline {
        point
            .y
            .saturating_sub(metrics.text_bounds.y)
            .max(0)
            .checked_div(metrics.line_height)
            .unwrap_or(0) as usize
    } else {
        0
    }
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

pub(crate) fn decorate_native_text_edit_visuals(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    let geometry = native_text_visual_geometry(target, value, selection, dpi);
    if !geometry.selections.is_empty() {
        let text_index = plan.commands.iter().rposition(|command| {
            matches!(command, NativeDrawCommand::Text(text)
                if text.text == value && rect_contains(target.bounds, text.bounds))
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

pub(crate) fn decorate_native_focus_ring(
    plan: &mut NativeDrawPlan,
    interaction_plan: &ViewInteractionPlan,
    focused_widget: Option<WidgetId>,
    dpi: Dpi,
) -> Option<Rect> {
    let target = interaction_plan.hit_target_for_widget(focused_widget?)?;
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

#[derive(Debug, Clone, Copy)]
struct NativeTextLine {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
struct NativeTextVisualMetrics {
    text_bounds: Rect,
    character_width: i32,
    line_height: i32,
}

fn native_text_visual_metrics(target: ViewHitTarget, dpi: Dpi) -> NativeTextVisualMetrics {
    let inset = Dp::new(8.0).to_px(dpi).round_i32().max(1);
    NativeTextVisualMetrics {
        text_bounds: Rect {
            x: target.bounds.x.saturating_add(inset),
            y: target.bounds.y.saturating_add(inset),
            width: target
                .bounds
                .width
                .saturating_sub(inset.saturating_mul(2))
                .max(1),
            height: target
                .bounds
                .height
                .saturating_sub(inset.saturating_mul(2))
                .max(1),
        },
        character_width: Dp::new(8.0).to_px(dpi).round_i32().max(1),
        line_height: Dp::new(18.0).to_px(dpi).round_i32().max(1),
    }
}

fn text_lines(value: &str, multiline: bool) -> Vec<NativeTextLine> {
    if !multiline {
        return vec![NativeTextLine {
            start: 0,
            end: char_count(value),
        }];
    }
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, character) in value.chars().enumerate() {
        if character == '\n' {
            lines.push(NativeTextLine { start, end: index });
            start = index + 1;
        }
    }
    lines.push(NativeTextLine {
        start,
        end: char_count(value),
    });
    lines
}

fn text_position(value: &str, index: usize, multiline: bool) -> (usize, usize) {
    if !multiline {
        return (0, index.min(char_count(value)));
    }
    let mut row = 0;
    let mut column = 0;
    for character in value.chars().take(index) {
        if character == '\n' {
            row += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    (row, column)
}

fn rect_contains(outer: Rect, inner: Rect) -> bool {
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
                Dpi::standard()
            ),
            2
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: 16, y: 30 },
                Dpi::standard()
            ),
            4
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: 500, y: 500 },
                Dpi::standard()
            ),
            5
        );
        assert_eq!(
            native_text_index_for_point(
                target,
                "A中\n🙂Z",
                Point { x: -20, y: -20 },
                Dpi::standard()
            ),
            0
        );
    }
}
