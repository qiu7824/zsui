use crate::native_text_edit::{
    char_count, grapheme_boundaries, move_selection, move_selection_to, snap_grapheme_index,
    NativeTextEditResult, NativeTextMovement, NativeTextSelection,
};
#[cfg(test)]
use crate::native_text_edit::{grapheme_count_in_range, grapheme_index_for_column};
use crate::{
    ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Point, Rect,
    ViewHitTarget, ViewHitTargetKind, ViewInteractionPlan, WidgetId,
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

const NATIVE_TEXT_SHAPING_CACHE_CAPACITY: usize = 256;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub(crate) struct NativeTextShapingCache {
    entries: Arc<Mutex<VecDeque<(String, NativeShapedTextLine)>>>,
}

#[allow(dead_code)]
impl NativeTextShapingCache {
    fn release_idle_memory(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
            entries.shrink_to_fit();
        }
    }

    fn shape(
        &self,
        text: &str,
        shape: impl FnOnce() -> Option<NativeShapedTextLine>,
    ) -> Option<NativeShapedTextLine> {
        if let Ok(mut entries) = self.entries.lock() {
            if let Some(position) = entries.iter().position(|(cached, _)| cached == text) {
                let entry = entries.remove(position)?;
                let shaped = entry.1.clone();
                entries.push_back(entry);
                return Some(shaped);
            }
        }
        let shaped = shape()?;
        if let Ok(mut entries) = self.entries.lock() {
            if entries.len() == NATIVE_TEXT_SHAPING_CACHE_CAPACITY {
                entries.pop_front();
            }
            entries.push_back((text.to_string(), shaped.clone()));
        }
        Some(shaped)
    }
}
pub(crate) trait NativeTextShaper {
    fn debug_name(&self) -> &'static str;

    fn typography_scale(&self) -> f32 {
        1.0
    }

    fn shape_line(&self, text: &str) -> Option<NativeShapedTextLine>;
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub(crate) enum NativeTextShapingBackend {
    #[default]
    LogicalCells,
    Platform(
        Arc<dyn crate::platform_text_shaper::NativePlatformTextShaper>,
        NativeTextShapingCache,
    ),
    #[cfg(test)]
    Test(fn(&str, i32) -> Option<NativeShapedTextLine>),
}

impl std::fmt::Debug for NativeTextShapingBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LogicalCells => formatter.write_str("LogicalCells"),
            Self::Platform(shaper, _) => formatter.write_str(shaper.debug_name()),
            #[cfg(test)]
            Self::Test(_) => formatter.write_str("Test"),
        }
    }
}

impl NativeTextShapingBackend {
    #[allow(dead_code)]
    pub(crate) fn platform(
        shaper: impl crate::platform_text_shaper::NativePlatformTextShaper + 'static,
    ) -> Self {
        Self::Platform(Arc::new(shaper), NativeTextShapingCache::default())
    }

    fn typography_scale(&self) -> f32 {
        match self {
            Self::Platform(shaper, _) => shaper.typography_scale(),
            Self::LogicalCells => 1.0,
            #[cfg(test)]
            Self::Test(_) => 1.0,
        }
    }

    pub(crate) fn release_idle_memory(&self) {
        if let Self::Platform(_, cache) = self {
            cache.release_idle_memory();
        }
    }

    fn shape_line(&self, text: &str, fallback_width: i32) -> NativeShapedTextLine {
        let shaped = match self {
            Self::LogicalCells => None,
            Self::Platform(shaper, cache) => cache.shape(text, || shaper.shape_line(text)),
            #[cfg(test)]
            Self::Test(shape) => shape(text, fallback_width),
        };
        shaped.unwrap_or_else(|| NativeShapedTextLine::logical_cells(text, fallback_width))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeShapedTextCluster {
    pub start: usize,
    pub end: usize,
    pub start_x: i32,
    pub end_x: i32,
}

impl NativeShapedTextCluster {
    fn left(self) -> i32 {
        self.start_x.min(self.end_x)
    }

    fn right(self) -> i32 {
        self.start_x.max(self.end_x)
    }

    fn width(self) -> i32 {
        self.right().saturating_sub(self.left())
    }

    fn left_index(self) -> usize {
        if self.start_x <= self.end_x {
            self.start
        } else {
            self.end
        }
    }

    fn right_index(self) -> usize {
        if self.start_x <= self.end_x {
            self.end
        } else {
            self.start
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeShapedTextCaret {
    pub index: usize,
    pub primary_x: i32,
    pub secondary_x: i32,
}

impl NativeShapedTextCaret {
    #[allow(dead_code)]
    pub(crate) fn closest_cluster_edges(self, next: Self) -> (i32, i32) {
        [
            (self.primary_x, next.primary_x),
            (self.primary_x, next.secondary_x),
            (self.secondary_x, next.primary_x),
            (self.secondary_x, next.secondary_x),
        ]
        .into_iter()
        .min_by_key(|(start, end)| start.abs_diff(*end))
        .unwrap_or((self.primary_x, next.primary_x))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeShapedTextLine {
    pub width: i32,
    pub clusters: Vec<NativeShapedTextCluster>,
    pub carets: Vec<NativeShapedTextCaret>,
}

impl NativeShapedTextLine {
    #[allow(dead_code)]
    pub(crate) fn new(
        width: i32,
        clusters: Vec<NativeShapedTextCluster>,
        carets: Vec<NativeShapedTextCaret>,
    ) -> Option<Self> {
        let scalar_len = clusters.last().map(|cluster| cluster.end).unwrap_or(0);
        let valid_clusters = clusters.first().map(|cluster| cluster.start) == Some(0)
            && clusters.windows(2).all(|pair| pair[0].end == pair[1].start)
            && clusters.iter().all(|cluster| cluster.start < cluster.end);
        let valid_carets = carets.first().map(|caret| caret.index) == Some(0)
            && carets.last().map(|caret| caret.index) == Some(scalar_len)
            && carets.windows(2).all(|pair| pair[0].index < pair[1].index);
        (valid_clusters && valid_carets).then_some(Self {
            width: width.max(0),
            clusters,
            carets,
        })
    }

    fn logical_cells(text: &str, fallback_width: i32) -> Self {
        let fallback_width = fallback_width.max(1);
        let boundaries = grapheme_boundaries(text);
        let clusters = boundaries
            .windows(2)
            .enumerate()
            .map(|(column, pair)| NativeShapedTextCluster {
                start: pair[0],
                end: pair[1],
                start_x: i32::try_from(column)
                    .unwrap_or(i32::MAX)
                    .saturating_mul(fallback_width),
                end_x: i32::try_from(column.saturating_add(1))
                    .unwrap_or(i32::MAX)
                    .saturating_mul(fallback_width),
            })
            .collect::<Vec<_>>();
        let carets = boundaries
            .into_iter()
            .enumerate()
            .map(|(column, index)| {
                let x = i32::try_from(column)
                    .unwrap_or(i32::MAX)
                    .saturating_mul(fallback_width);
                NativeShapedTextCaret {
                    index,
                    primary_x: x,
                    secondary_x: x,
                }
            })
            .collect::<Vec<_>>();
        Self {
            width: clusters.last().map(|cluster| cluster.right()).unwrap_or(0),
            clusters,
            carets,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeTextVisualHorizontalDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeTextDragViewport {
    pub point: Point,
    pub first_visible_row: usize,
    pub horizontal_scroll_px: usize,
    pub scrolled: bool,
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
    native_text_visual_geometry_in_viewport(target, value, selection, 0, 0, wrap, dpi)
}

#[cfg(test)]
pub(crate) fn native_text_visual_geometry_in_viewport(
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    first_visible_column: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    native_text_visual_geometry_in_viewport_with_backend(
        target,
        value,
        selection,
        first_visible_row,
        first_visible_column,
        wrap,
        dpi,
        &NativeTextShapingBackend::LogicalCells,
    )
}

pub(crate) fn native_text_visual_geometry_in_viewport_with_backend(
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> NativeTextVisualGeometry {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let text_bounds = metrics.text_bounds;
    let line_height = metrics.line_height;
    let lines = text_lines_with_backend(
        value,
        multiline,
        wrap,
        text_bounds.width,
        metrics.character_width,
        backend,
    );
    let first_visible_row = first_visible_row.min(lines.len().saturating_sub(1));
    let horizontal_scroll_px = if multiline && wrap == crate::TextWrap::NoWrap {
        i32::try_from(horizontal_scroll_px).unwrap_or(i32::MAX)
    } else {
        0
    };
    let selection = selection.clamp(value);
    let (caret_row, caret_line_x) = text_position_x(value, selection.caret, &lines);
    let caret_x = text_bounds
        .x
        .saturating_add(caret_line_x)
        .saturating_sub(horizontal_scroll_px);
    let caret_x = caret_x.clamp(
        text_bounds.x,
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
        for (row, line) in lines.iter().enumerate().skip(first_visible_row) {
            let y = visual_row_y(text_bounds.y, row, first_visible_row, line_height);
            if y >= text_bounds.y.saturating_add(text_bounds.height) {
                break;
            }
            let mut selected_clusters = line
                .clusters
                .iter()
                .copied()
                .filter(|cluster| cluster.start < end && cluster.end > start)
                .collect::<Vec<_>>();
            selected_clusters.sort_by_key(|cluster| (cluster.left(), cluster.right()));
            let height = line_height
                .min(
                    text_bounds
                        .y
                        .saturating_add(text_bounds.height)
                        .saturating_sub(y),
                )
                .max(1);
            let mut current: Option<(i32, i32)> = None;
            for cluster in selected_clusters {
                let left = text_bounds
                    .x
                    .saturating_add(cluster.left())
                    .saturating_sub(horizontal_scroll_px);
                let right = text_bounds
                    .x
                    .saturating_add(cluster.right())
                    .saturating_sub(horizontal_scroll_px);
                current = match current {
                    Some((start_x, end_x)) if left <= end_x.saturating_add(1) => {
                        Some((start_x, end_x.max(right)))
                    }
                    Some((start_x, end_x)) => {
                        push_clipped_selection(
                            &mut selections,
                            text_bounds,
                            start_x,
                            end_x,
                            y,
                            height,
                        );
                        Some((left, right))
                    }
                    None => Some((left, right)),
                };
            }
            if let Some((start_x, end_x)) = current {
                push_clipped_selection(&mut selections, text_bounds, start_x, end_x, y, height);
            } else if !line.soft_wrap_after && end > line.end && start <= line.end {
                let x = text_bounds
                    .x
                    .saturating_add(line.caret_x(line.end))
                    .saturating_sub(horizontal_scroll_px);
                push_clipped_selection(
                    &mut selections,
                    text_bounds,
                    x,
                    x.saturating_add(1),
                    y,
                    height,
                );
            }
        }
    }
    NativeTextVisualGeometry { caret, selections }
}

fn push_clipped_selection(
    selections: &mut Vec<Rect>,
    bounds: Rect,
    left: i32,
    right: i32,
    y: i32,
    height: i32,
) {
    let left = left.max(bounds.x);
    let right = right.min(bounds.x.saturating_add(bounds.width));
    if right > left {
        selections.push(Rect {
            x: left,
            y,
            width: right.saturating_sub(left).max(1),
            height,
        });
    }
}

#[cfg(test)]
pub(crate) fn native_text_index_for_point(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    native_text_index_for_point_in_viewport(target, value, point, 0, 0, wrap, dpi)
}

#[cfg(test)]
pub(crate) fn native_text_index_for_point_in_viewport(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    first_visible_row: usize,
    first_visible_column: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> usize {
    native_text_index_for_point_in_viewport_with_backend(
        target,
        value,
        point,
        first_visible_row,
        first_visible_column,
        wrap,
        dpi,
        &NativeTextShapingBackend::LogicalCells,
    )
}

pub(crate) fn native_text_index_for_point_in_viewport_with_backend(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    first_visible_row: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    let multiline = target.kind == ViewHitTargetKind::TextEditor;
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        multiline,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
    );
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
    let line = &lines[row];
    let relative_x = point
        .x
        .saturating_sub(metrics.text_bounds.x)
        .saturating_add(if multiline && wrap == crate::TextWrap::NoWrap {
            i32::try_from(horizontal_scroll_px).unwrap_or(i32::MAX)
        } else {
            0
        });
    line.index_for_x(relative_x)
}

#[cfg(test)]
pub(crate) fn native_text_index_for_vertical_move(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_column: Option<usize>,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> (usize, usize) {
    native_text_index_for_vertical_row_delta(
        target,
        value,
        caret,
        direction,
        preferred_column,
        1,
        wrap,
        dpi,
    )
}

#[cfg(test)]
pub(crate) fn native_text_index_for_vertical_page_move(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_column: Option<usize>,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> (usize, usize, usize) {
    let (_, visible_rows) = native_text_viewport_lines(target, value, wrap, dpi);
    let (target_index, preferred_column) = native_text_index_for_vertical_row_delta(
        target,
        value,
        caret,
        direction,
        preferred_column,
        visible_rows,
        wrap,
        dpi,
    );
    let row_delta = isize::try_from(visible_rows).unwrap_or(isize::MAX);
    let row_delta = match direction {
        NativeTextVisualDirection::Up => row_delta.saturating_neg(),
        NativeTextVisualDirection::Down => row_delta,
    };
    let first_visible_row =
        native_text_scroll_visual_rows(target, value, first_visible_row, row_delta, wrap, dpi);
    let first_visible_row = native_text_first_visible_row_for_caret(
        target,
        value,
        target_index,
        first_visible_row,
        wrap,
        dpi,
    );
    (target_index, preferred_column, first_visible_row)
}

#[cfg(test)]
fn native_text_index_for_vertical_row_delta(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_column: Option<usize>,
    row_delta: usize,
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
    let caret = snap_grapheme_index(value, caret);
    let (row, current_column) = text_position(value, caret, &lines);
    let preferred_column = preferred_column.unwrap_or(current_column);
    let target_row = match direction {
        NativeTextVisualDirection::Up => row.saturating_sub(row_delta),
        NativeTextVisualDirection::Down => row.saturating_add(row_delta).min(lines.len() - 1),
    };
    let line = &lines[target_row];
    let line_columns = line.columns;
    let column = if line.soft_wrap_after {
        preferred_column.min(line_columns.saturating_sub(1))
    } else {
        preferred_column.min(line_columns)
    };
    (
        grapheme_index_for_column(value, line.start, line.end, column),
        preferred_column,
    )
}

pub(crate) fn native_text_index_for_vertical_move_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_x: Option<i32>,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> (usize, i32) {
    native_text_index_for_vertical_row_delta_with_backend(
        target,
        value,
        caret,
        direction,
        preferred_x,
        1,
        wrap,
        dpi,
        backend,
    )
}

pub(crate) fn native_text_index_for_horizontal_move_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualHorizontalDirection,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        target.kind == ViewHitTargetKind::TextEditor,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
    );
    let caret = snap_grapheme_index(value, caret);
    let (row, _) = text_position_x(value, caret, &lines);
    let Some(line) = lines.get(row) else {
        return caret;
    };
    let stops = line.visual_caret_stops();
    let Some(position) = stops.iter().position(|stop| stop.index == caret) else {
        return caret;
    };

    let candidate = match direction {
        NativeTextVisualHorizontalDirection::Left => stops[..position]
            .iter()
            .rev()
            .find(|stop| stop.x < stops[position].x)
            .copied()
            .or_else(|| {
                lines[..row].iter().rev().find_map(|line| {
                    line.visual_caret_stops()
                        .into_iter()
                        .rev()
                        .find(|stop| stop.index != caret)
                })
            }),
        NativeTextVisualHorizontalDirection::Right => stops[position.saturating_add(1)..]
            .iter()
            .find(|stop| stop.x > stops[position].x)
            .copied()
            .or_else(|| {
                lines[row.saturating_add(1)..].iter().find_map(|line| {
                    line.visual_caret_stops()
                        .into_iter()
                        .find(|stop| stop.index != caret)
                })
            }),
    };
    candidate.map(|stop| stop.index).unwrap_or(caret)
}

pub(crate) fn move_native_text_selection_horizontally_with_backend(
    target: ViewHitTarget,
    value: &str,
    selection: &mut NativeTextSelection,
    direction: NativeTextVisualHorizontalDirection,
    extend: bool,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> NativeTextEditResult {
    if !extend && !selection.is_collapsed() {
        return move_selection(
            value,
            selection,
            match direction {
                NativeTextVisualHorizontalDirection::Left => NativeTextMovement::Left,
                NativeTextVisualHorizontalDirection::Right => NativeTextMovement::Right,
            },
            false,
            target.kind == ViewHitTargetKind::TextEditor,
        );
    }
    let target_index = native_text_index_for_horizontal_move_with_backend(
        target,
        value,
        selection.caret,
        direction,
        wrap,
        dpi,
        backend,
    );
    move_selection_to(value, selection, target_index, extend)
}

pub(crate) fn native_text_index_for_vertical_page_move_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_x: Option<i32>,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> (usize, i32, usize) {
    let (_, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
    let (target_index, preferred_x) = native_text_index_for_vertical_row_delta_with_backend(
        target,
        value,
        caret,
        direction,
        preferred_x,
        visible_rows,
        wrap,
        dpi,
        backend,
    );
    let row_delta = isize::try_from(visible_rows).unwrap_or(isize::MAX);
    let row_delta = match direction {
        NativeTextVisualDirection::Up => row_delta.saturating_neg(),
        NativeTextVisualDirection::Down => row_delta,
    };
    let first_visible_row = native_text_scroll_visual_rows_with_backend(
        target,
        value,
        first_visible_row,
        row_delta,
        wrap,
        dpi,
        backend,
    );
    let first_visible_row = native_text_first_visible_row_for_caret_with_backend(
        target,
        value,
        target_index,
        first_visible_row,
        wrap,
        dpi,
        backend,
    );
    (target_index, preferred_x, first_visible_row)
}

fn native_text_index_for_vertical_row_delta_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    direction: NativeTextVisualDirection,
    preferred_x: Option<i32>,
    row_delta: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> (usize, i32) {
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        target.kind == ViewHitTargetKind::TextEditor,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
    );
    let caret = snap_grapheme_index(value, caret);
    let (row, current_x) = text_position_x(value, caret, &lines);
    let preferred_x = preferred_x.unwrap_or(current_x);
    let target_row = match direction {
        NativeTextVisualDirection::Up => row.saturating_sub(row_delta),
        NativeTextVisualDirection::Down => row.saturating_add(row_delta).min(lines.len() - 1),
    };
    (lines[target_row].index_for_x(preferred_x), preferred_x)
}

#[cfg(test)]
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
    let (caret_row, _) = text_position(value, snap_grapheme_index(value, caret), &lines);
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

pub(crate) fn native_text_first_visible_row_for_caret_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor {
        return 0;
    }
    let (lines, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    let first_visible_row = first_visible_row.min(maximum_first);
    let (caret_row, _) = text_position_x(value, snap_grapheme_index(value, caret), &lines);
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

pub(crate) fn native_text_horizontal_scroll_for_caret_with_backend(
    target: ViewHitTarget,
    value: &str,
    caret: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor || wrap != crate::TextWrap::NoWrap {
        return 0;
    }
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        true,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
    );
    let (_, caret_x) = text_position_x(value, snap_grapheme_index(value, caret), &lines);
    let maximum_scroll = lines
        .iter()
        .map(|line| line.width)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .saturating_sub(metrics.text_bounds.width)
        .max(0);
    let mut scroll = i32::try_from(horizontal_scroll_px)
        .unwrap_or(i32::MAX)
        .min(maximum_scroll);
    if caret_x < scroll {
        scroll = caret_x.max(0);
    } else if caret_x >= scroll.saturating_add(metrics.text_bounds.width) {
        scroll = caret_x
            .saturating_add(1)
            .saturating_sub(metrics.text_bounds.width)
            .min(maximum_scroll)
            .max(0);
    }
    usize::try_from(scroll).unwrap_or(usize::MAX)
}

#[cfg(test)]
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

pub(crate) fn native_text_scroll_visual_rows_with_backend(
    target: ViewHitTarget,
    value: &str,
    first_visible_row: usize,
    row_delta: isize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor {
        return 0;
    }
    let (lines, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
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

pub(crate) fn native_text_drag_viewport_for_point_with_backend(
    target: ViewHitTarget,
    value: &str,
    point: Point,
    first_visible_row: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> NativeTextDragViewport {
    if target.kind != ViewHitTargetKind::TextEditor {
        return NativeTextDragViewport {
            point,
            first_visible_row: 0,
            horizontal_scroll_px: 0,
            scrolled: false,
        };
    }
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let mut adjusted = point;
    let mut row = first_visible_row;
    let mut scroll = if wrap == crate::TextWrap::NoWrap {
        horizontal_scroll_px
    } else {
        0
    };
    let bottom = metrics
        .text_bounds
        .y
        .saturating_add(metrics.text_bounds.height);
    if point.y < metrics.text_bounds.y {
        row =
            native_text_scroll_visual_rows_with_backend(target, value, row, -1, wrap, dpi, backend);
        adjusted.y = metrics.text_bounds.y;
    } else if point.y >= bottom {
        row =
            native_text_scroll_visual_rows_with_backend(target, value, row, 1, wrap, dpi, backend);
        let visible_rows = metrics
            .text_bounds
            .height
            .saturating_add(metrics.line_height.saturating_sub(1))
            .checked_div(metrics.line_height)
            .unwrap_or(1)
            .max(1);
        adjusted.y = metrics.text_bounds.y.saturating_add(
            visible_rows
                .saturating_sub(1)
                .saturating_mul(metrics.line_height),
        );
    }

    if wrap == crate::TextWrap::NoWrap {
        let right = metrics
            .text_bounds
            .x
            .saturating_add(metrics.text_bounds.width);
        if point.x < metrics.text_bounds.x {
            scroll = native_text_scroll_horizontal_pixels(
                target,
                value,
                scroll,
                -metrics.character_width,
                wrap,
                dpi,
                backend,
            );
            adjusted.x = metrics.text_bounds.x;
        } else if point.x >= right {
            scroll = native_text_scroll_horizontal_pixels(
                target,
                value,
                scroll,
                metrics.character_width,
                wrap,
                dpi,
                backend,
            );
            adjusted.x = right.saturating_sub(1);
        }
    }

    NativeTextDragViewport {
        point: adjusted,
        first_visible_row: row,
        horizontal_scroll_px: scroll,
        scrolled: row != first_visible_row || scroll != horizontal_scroll_px,
    }
}

fn native_text_scroll_horizontal_pixels(
    target: ViewHitTarget,
    value: &str,
    horizontal_scroll_px: usize,
    delta_px: i32,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    if target.kind != ViewHitTargetKind::TextEditor || wrap != crate::TextWrap::NoWrap {
        return 0;
    }
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        true,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
    );
    let maximum = lines
        .iter()
        .map(|line| line.width)
        .max()
        .unwrap_or(0)
        .saturating_sub(metrics.text_bounds.width)
        .max(0);
    let current = i32::try_from(horizontal_scroll_px)
        .unwrap_or(i32::MAX)
        .min(maximum);
    usize::try_from(current.saturating_add(delta_px).clamp(0, maximum)).unwrap_or(usize::MAX)
}

pub(crate) fn native_text_wheel_row_delta(delta_y: Dp) -> isize {
    if !delta_y.0.is_finite() || delta_y.0 == 0.0 {
        return 0;
    }
    let line_height = crate::TextRole::Body
        .metrics_for(crate::ZsTypographyPlatformStyle::current())
        .line_height
        .max(1.0);
    let rows = (delta_y.0.abs() / line_height).round().max(1.0) as isize;
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
    decorate_native_text_edit_visuals_in_viewport(plan, target, value, selection, 0, 0, wrap, dpi)
}

#[cfg(test)]
pub(crate) fn decorate_native_text_edit_visuals_in_viewport(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    first_visible_column: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> NativeTextVisualGeometry {
    decorate_native_text_edit_visuals_in_viewport_with_backend(
        plan,
        target,
        value,
        selection,
        first_visible_row,
        first_visible_column,
        wrap,
        dpi,
        &NativeTextShapingBackend::LogicalCells,
    )
}

pub(crate) fn decorate_native_text_edit_visuals_in_viewport_with_backend(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    selection: NativeTextSelection,
    first_visible_row: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> NativeTextVisualGeometry {
    let geometry = native_text_visual_geometry_in_viewport_with_backend(
        target,
        value,
        selection,
        first_visible_row,
        horizontal_scroll_px,
        wrap,
        dpi,
        backend,
    );
    if target.kind == ViewHitTargetKind::TextEditor {
        decorate_native_text_editor_viewport_with_backend(
            plan,
            target,
            value,
            first_visible_row,
            horizontal_scroll_px,
            wrap,
            dpi,
            &geometry,
            backend,
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

fn decorate_native_text_editor_viewport_with_backend(
    plan: &mut NativeDrawPlan,
    target: ViewHitTarget,
    value: &str,
    first_visible_row: usize,
    horizontal_scroll_px: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    geometry: &NativeTextVisualGeometry,
    backend: &NativeTextShapingBackend,
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

    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let (lines, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    let first_visible_row = first_visible_row.min(maximum_first);
    let horizontal_scroll_px = if wrap == crate::TextWrap::NoWrap {
        i32::try_from(horizontal_scroll_px).unwrap_or(i32::MAX)
    } else {
        0
    };
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
        .enumerate()
        .skip(first_visible_row)
        .take(visible_rows)
    {
        commands.push(NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
            char_slice(value, line.start, line.end),
            Rect {
                x: metrics.text_bounds.x.saturating_sub(horizontal_scroll_px),
                y: visual_row_y(
                    metrics.text_bounds.y,
                    row,
                    first_visible_row,
                    metrics.line_height,
                ),
                width: line.width.max(metrics.text_bounds.width),
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
    let mut target = interaction_plan.focus_target_for_widget(focused_widget?)?;
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
    #[cfg(feature = "menu-flyout")]
    if matches!(
        target.kind,
        ViewHitTargetKind::MenuFlyout
            | ViewHitTargetKind::MenuFlyoutScrim
            | ViewHitTargetKind::MenuFlyoutItem { .. }
    ) {
        return None;
    }
    let focus_profile = crate::platform_component_profile::PlatformFocusVisualProfile::for_platform(
        crate::ZsPlatformStyle::current(),
    );
    #[allow(unused_mut)]
    let mut uses_text_input_indicator = matches!(
        target.kind,
        ViewHitTargetKind::Textbox | ViewHitTargetKind::TextEditor
    );
    #[cfg(feature = "auto-suggest")]
    {
        uses_text_input_indicator |= target.kind == ViewHitTargetKind::AutoSuggestBox;
    }
    #[cfg(feature = "password-box")]
    {
        uses_text_input_indicator |= target.kind == ViewHitTargetKind::PasswordBox;
    }
    #[cfg(feature = "number-box")]
    {
        uses_text_input_indicator |= target.kind == ViewHitTargetKind::NumberBox;
    }
    if uses_text_input_indicator {
        if let Some(height) = focus_profile.text_input_indicator_height {
            let height = height.to_px(dpi).round_i32().max(1);
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
    }
    let requested_inset = focus_profile.outline_inset.to_px(dpi).round_i32().max(1);
    let maximum_inset = (target.bounds.width.min(target.bounds.height).max(1) - 1) / 2;
    let inset = requested_inset.min(maximum_inset.max(0));
    let ring = Rect {
        x: target.bounds.x.saturating_add(inset),
        y: target.bounds.y.saturating_add(inset),
        width: target.bounds.width.saturating_sub(inset.saturating_mul(2)),
        height: target.bounds.height.saturating_sub(inset.saturating_mul(2)),
    };
    let width = focus_profile.outline_width.to_px(dpi).round_i32().max(1);
    plan.push(NativeDrawCommand::StrokeRect {
        rect: ring,
        stroke: NativeDrawFill::Role(ColorRole::Accent),
        width,
    });
    Some(ring)
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "button",
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
    feature = "button",
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
    #[cfg(feature = "label")]
    let supported = supported || target.kind == ViewHitTargetKind::NavigationViewToggle;
    #[cfg(feature = "button")]
    let supported = supported || target.kind == ViewHitTargetKind::Button;
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
    feature = "button",
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

#[derive(Debug, Clone)]
struct NativeTextLine {
    start: usize,
    end: usize,
    #[cfg(test)]
    columns: usize,
    width: i32,
    clusters: Vec<NativeShapedTextCluster>,
    carets: Vec<NativeShapedTextCaret>,
    soft_wrap_after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeTextVisualCaretStop {
    index: usize,
    x: i32,
}

impl NativeTextLine {
    fn caret_x(&self, index: usize) -> i32 {
        self.carets
            .binary_search_by_key(&index, |caret| caret.index)
            .ok()
            .and_then(|position| self.carets.get(position))
            .map(|caret| caret.primary_x)
            .unwrap_or_else(|| {
                self.carets
                    .iter()
                    .min_by_key(|caret| caret.index.abs_diff(index))
                    .map(|caret| caret.primary_x)
                    .unwrap_or(0)
            })
    }

    fn index_for_x(&self, x: i32) -> usize {
        let mut clusters = self.clusters.iter().copied().collect::<Vec<_>>();
        clusters.sort_by_key(|cluster| (cluster.left(), cluster.right()));
        let Some(first) = clusters.first().copied() else {
            return self.start;
        };
        if x <= first.left() {
            return first.left_index();
        }
        for cluster in clusters.iter().copied() {
            let midpoint = cluster
                .left()
                .saturating_add(cluster.width().saturating_add(1) / 2);
            if x < midpoint {
                return cluster.left_index();
            }
            if x <= cluster.right() {
                return cluster.right_index();
            }
        }
        clusters
            .last()
            .copied()
            .map(NativeShapedTextCluster::right_index)
            .unwrap_or(self.end)
    }

    fn visual_caret_stops(&self) -> Vec<NativeTextVisualCaretStop> {
        let mut stops = self
            .carets
            .iter()
            .map(|caret| NativeTextVisualCaretStop {
                index: caret.index,
                x: caret.primary_x,
            })
            .collect::<Vec<_>>();
        stops.sort_by_key(|stop| (stop.x, stop.index));
        stops
    }
}

#[derive(Debug, Clone, Copy)]
struct NativeTextVisualMetrics {
    text_bounds: Rect,
    character_width: i32,
    line_height: i32,
}

#[cfg(test)]
fn native_text_visual_metrics(target: ViewHitTarget, dpi: Dpi) -> NativeTextVisualMetrics {
    native_text_visual_metrics_with_scale(target, dpi, 1.0)
}

fn native_text_visual_metrics_with_scale(
    target: ViewHitTarget,
    dpi: Dpi,
    typography_scale: f32,
) -> NativeTextVisualMetrics {
    let inset = Dp::new(8.0).to_px(dpi).round_i32().max(1);
    let body_typography =
        crate::TextRole::Body.metrics_for(crate::ZsTypographyPlatformStyle::current());
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
        character_width: Dp::new(8.0 * typography_scale)
            .to_px(dpi)
            .round_i32()
            .max(1),
        line_height: Dp::new(body_typography.line_height * typography_scale)
            .to_px(dpi)
            .round_i32()
            .max(1),
    }
}

#[cfg(test)]
fn native_text_viewport_lines(
    target: ViewHitTarget,
    value: &str,
    wrap: crate::TextWrap,
    dpi: Dpi,
) -> (Vec<NativeTextLine>, usize) {
    native_text_viewport_lines_with_backend(
        target,
        value,
        wrap,
        dpi,
        &NativeTextShapingBackend::LogicalCells,
    )
}

fn native_text_viewport_lines_with_backend(
    target: ViewHitTarget,
    value: &str,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> (Vec<NativeTextLine>, usize) {
    let metrics = native_text_visual_metrics_with_scale(target, dpi, backend.typography_scale());
    let lines = text_lines_with_backend(
        value,
        target.kind == ViewHitTargetKind::TextEditor,
        wrap,
        metrics.text_bounds.width,
        metrics.character_width,
        backend,
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

#[allow(dead_code)]
pub(crate) fn native_text_visible_range_with_backend(
    target: ViewHitTarget,
    value: &str,
    first_visible_row: usize,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> std::ops::Range<usize> {
    let (lines, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
    let first_visible_row = first_visible_row.min(lines.len().saturating_sub(1));
    let last_visible_row = first_visible_row
        .saturating_add(visible_rows.saturating_sub(1))
        .min(lines.len().saturating_sub(1));
    let start = lines
        .get(first_visible_row)
        .map(|line| line.start)
        .unwrap_or(0);
    let end = lines
        .get(last_visible_row.saturating_add(1))
        .map(|line| line.start)
        .or_else(|| lines.get(last_visible_row).map(|line| line.end))
        .unwrap_or(start);
    start..end.max(start)
}

#[allow(dead_code)]
pub(crate) fn native_text_first_visible_row_for_index_alignment_with_backend(
    target: ViewHitTarget,
    value: &str,
    index: usize,
    align_to_top: bool,
    wrap: crate::TextWrap,
    dpi: Dpi,
    backend: &NativeTextShapingBackend,
) -> usize {
    let (lines, visible_rows) =
        native_text_viewport_lines_with_backend(target, value, wrap, dpi, backend);
    let maximum_first = lines.len().saturating_sub(visible_rows);
    let row = text_position_x(value, index, &lines).0;
    if align_to_top {
        row.min(maximum_first)
    } else {
        row.saturating_add(1)
            .saturating_sub(visible_rows)
            .min(maximum_first)
    }
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

#[cfg(test)]
fn text_lines(
    value: &str,
    multiline: bool,
    wrap: crate::TextWrap,
    max_columns: usize,
) -> Vec<NativeTextLine> {
    text_lines_with_backend(
        value,
        multiline,
        wrap,
        i32::try_from(max_columns.max(1)).unwrap_or(i32::MAX),
        1,
        &NativeTextShapingBackend::LogicalCells,
    )
}

fn text_lines_with_backend(
    value: &str,
    multiline: bool,
    wrap: crate::TextWrap,
    max_width: i32,
    fallback_width: i32,
    backend: &NativeTextShapingBackend,
) -> Vec<NativeTextLine> {
    let end = char_count(value);
    if !multiline {
        return vec![shape_text_line(
            value,
            0,
            end,
            fallback_width,
            backend,
            false,
        )];
    }
    let characters = value.chars().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, character) in characters.iter().copied().enumerate() {
        if character == '\n' {
            push_shaped_text_lines(
                &mut lines,
                value,
                start,
                index,
                wrap,
                max_width,
                fallback_width,
                backend,
            );
            start = index + 1;
        }
    }
    push_shaped_text_lines(
        &mut lines,
        value,
        start,
        characters.len(),
        wrap,
        max_width,
        fallback_width,
        backend,
    );
    lines
}

fn push_shaped_text_lines(
    lines: &mut Vec<NativeTextLine>,
    value: &str,
    start: usize,
    end: usize,
    wrap: crate::TextWrap,
    max_width: i32,
    fallback_width: i32,
    backend: &NativeTextShapingBackend,
) {
    let max_width = max_width.max(1);
    let whole = shape_text_line(value, start, end, fallback_width, backend, false);
    if wrap == crate::TextWrap::NoWrap || whole.width <= max_width || whole.clusters.is_empty() {
        lines.push(whole);
        return;
    }

    let mut logical_clusters = whole.clusters;
    logical_clusters.sort_by_key(|cluster| cluster.start);
    let mut first = 0_usize;
    while first < logical_clusters.len() {
        let mut after_last = first;
        let mut width = 0_i32;
        let mut whitespace_break = None;
        while after_last < logical_clusters.len() {
            let cluster = logical_clusters[after_last];
            let next_width = width.saturating_add(cluster.width().max(1));
            if after_last > first && next_width > max_width {
                break;
            }
            width = next_width;
            after_last += 1;
            if char_slice(value, cluster.start, cluster.end)
                .chars()
                .all(char::is_whitespace)
            {
                whitespace_break = Some(after_last);
            }
        }
        let break_after = if after_last == logical_clusters.len() {
            after_last
        } else {
            whitespace_break
                .filter(|position| *position > first)
                .unwrap_or(after_last)
        };
        let line_start = logical_clusters[first].start;
        let line_end = logical_clusters[break_after.saturating_sub(1)].end;
        let soft_wrap_after = break_after < logical_clusters.len();
        lines.push(shape_text_line(
            value,
            line_start,
            line_end,
            fallback_width,
            backend,
            soft_wrap_after,
        ));
        first = break_after;
    }
}

fn shape_text_line(
    value: &str,
    start: usize,
    end: usize,
    fallback_width: i32,
    backend: &NativeTextShapingBackend,
    soft_wrap_after: bool,
) -> NativeTextLine {
    let shaped = backend.shape_line(&char_slice(value, start, end), fallback_width);
    NativeTextLine {
        start,
        end,
        #[cfg(test)]
        columns: shaped.clusters.len(),
        width: shaped.width,
        clusters: shaped
            .clusters
            .into_iter()
            .map(|cluster| NativeShapedTextCluster {
                start: start.saturating_add(cluster.start),
                end: start.saturating_add(cluster.end),
                start_x: cluster.start_x,
                end_x: cluster.end_x,
            })
            .collect(),
        carets: shaped
            .carets
            .into_iter()
            .map(|caret| NativeShapedTextCaret {
                index: start.saturating_add(caret.index),
                primary_x: caret.primary_x,
                secondary_x: caret.secondary_x,
            })
            .collect(),
        soft_wrap_after,
    }
}

#[cfg(test)]
fn text_position(value: &str, index: usize, lines: &[NativeTextLine]) -> (usize, usize) {
    let index = snap_grapheme_index(value, index);
    for (row, line) in lines.iter().enumerate() {
        if index < line.end || (index == line.end && !line.soft_wrap_after) {
            let column = if index == line.end {
                line.columns
            } else {
                grapheme_count_in_range(value, line.start, index)
            };
            return (row, column);
        }
    }
    lines
        .last()
        .map(|line| {
            (
                lines.len().saturating_sub(1),
                if index >= line.end {
                    line.columns
                } else {
                    grapheme_count_in_range(value, line.start, index)
                },
            )
        })
        .unwrap_or((0, 0))
}

fn text_position_x(value: &str, index: usize, lines: &[NativeTextLine]) -> (usize, i32) {
    let index = snap_grapheme_index(value, index);
    for (row, line) in lines.iter().enumerate() {
        if index < line.end || (index == line.end && !line.soft_wrap_after) {
            return (row, line.caret_x(index));
        }
    }
    lines
        .last()
        .map(|line| {
            (
                lines.len().saturating_sub(1),
                line.caret_x(index.min(line.end)),
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
    fn shared_text_shaping_contract_has_no_platform_variants() {
        let source = include_str!("native_input_visuals.rs");
        let contract_start = source
            .find("pub(crate) trait NativeTextShaper")
            .expect("text shaper contract should exist");
        let contract_end = source
            .find("pub(crate) struct NativeShapedTextCluster")
            .expect("shared shaped cluster should follow the backend contract");
        let contract = &source[contract_start..contract_end];
        assert!(contract.contains("platform_text_shaper::NativePlatformTextShaper"));
        for forbidden in [
            "#[cfg(windows)]",
            "#[cfg(not(windows))]",
            "WindowsGdi(",
            "AppKit(",
            "LinuxDirect(",
            "LinuxDirectLite(",
            "Gtk(",
            "pango::",
            "gtk4::",
            "windows_gdi_renderer",
            "macos_appkit_renderer",
            "linux_direct::",
            "linux_gtk_renderer",
        ] {
            assert!(
                !contract.contains(forbidden),
                "shared shaping contract contains platform implementation: {forbidden}"
            );
        }

        let native = include_str!("native.rs");
        assert!(native.contains("fn set_text_shaping_backend("));
        for forbidden in [
            "use_appkit_text_shaping",
            "use_linux_direct_text_shaping",
            "use_linux_direct_lite_text_shaping",
            "use_gtk_text_shaping",
        ] {
            assert!(!native.contains(forbidden));
        }
    }

    fn body_line_height() -> i32 {
        Dp::new(
            crate::TextRole::Body
                .metrics_for(crate::ZsTypographyPlatformStyle::current())
                .line_height,
        )
        .to_px(Dpi::standard())
        .round_i32()
        .max(1)
    }

    fn editor_height_for_visible_rows(rows: i32) -> i32 {
        16_i32.saturating_add(body_line_height().saturating_mul(rows.max(1)))
    }

    fn proportional_test_shape(text: &str, _fallback_width: i32) -> Option<NativeShapedTextLine> {
        if text.is_empty() {
            return None;
        }
        let boundaries = grapheme_boundaries(text);
        let mut x = 0_i32;
        let mut clusters = Vec::new();
        let mut carets = vec![NativeShapedTextCaret {
            index: 0,
            primary_x: 0,
            secondary_x: 0,
        }];
        for pair in boundaries.windows(2) {
            let width = if char_slice(text, pair[0], pair[1]) == "W" {
                16
            } else {
                4
            };
            let next_x = x.saturating_add(width);
            clusters.push(NativeShapedTextCluster {
                start: pair[0],
                end: pair[1],
                start_x: x,
                end_x: next_x,
            });
            carets.push(NativeShapedTextCaret {
                index: pair[1],
                primary_x: next_x,
                secondary_x: next_x,
            });
            x = next_x;
        }
        NativeShapedTextLine::new(x, clusters, carets)
    }

    fn mixed_direction_test_shape(
        text: &str,
        _fallback_width: i32,
    ) -> Option<NativeShapedTextLine> {
        if text != "abאב" {
            return None;
        }
        NativeShapedTextLine::new(
            40,
            vec![
                NativeShapedTextCluster {
                    start: 0,
                    end: 1,
                    start_x: 0,
                    end_x: 10,
                },
                NativeShapedTextCluster {
                    start: 1,
                    end: 2,
                    start_x: 10,
                    end_x: 20,
                },
                NativeShapedTextCluster {
                    start: 2,
                    end: 3,
                    start_x: 40,
                    end_x: 30,
                },
                NativeShapedTextCluster {
                    start: 3,
                    end: 4,
                    start_x: 30,
                    end_x: 20,
                },
            ],
            vec![
                NativeShapedTextCaret {
                    index: 0,
                    primary_x: 0,
                    secondary_x: 0,
                },
                NativeShapedTextCaret {
                    index: 1,
                    primary_x: 10,
                    secondary_x: 10,
                },
                NativeShapedTextCaret {
                    index: 2,
                    primary_x: 40,
                    secondary_x: 20,
                },
                NativeShapedTextCaret {
                    index: 3,
                    primary_x: 30,
                    secondary_x: 30,
                },
                NativeShapedTextCaret {
                    index: 4,
                    primary_x: 20,
                    secondary_x: 20,
                },
            ],
        )
    }

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
    #[cfg(feature = "menu-flyout")]
    fn menu_flyout_uses_row_highlight_without_an_outer_focus_rectangle() {
        let widget = WidgetId::new(913);
        let interaction_plan = ViewInteractionPlan::new([ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 80,
                y: 60,
                width: 240,
                height: 160,
            },
            ViewHitTargetKind::MenuFlyout,
        )]);
        let mut plan = NativeDrawPlan::default();

        assert_eq!(
            decorate_native_focus_ring(&mut plan, &interaction_plan, Some(widget), Dpi::standard(),),
            None
        );
        assert!(plan.commands.is_empty());
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
    fn shaped_mixed_direction_geometry_uses_visual_clusters_and_dual_carets() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(921),
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 40,
            },
            ViewHitTargetKind::Textbox,
        );
        let backend = NativeTextShapingBackend::Test(mixed_direction_test_shape);
        let geometry = native_text_visual_geometry_in_viewport_with_backend(
            target,
            "abאב",
            NativeTextSelection {
                anchor: 1,
                caret: 3,
            },
            0,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
            &backend,
        );

        assert_eq!(geometry.caret.x, 38);
        assert_eq!(
            geometry.selections,
            vec![
                Rect {
                    x: 18,
                    y: 8,
                    width: 10,
                    height: body_line_height(),
                },
                Rect {
                    x: 38,
                    y: 8,
                    width: 10,
                    height: body_line_height(),
                },
            ]
        );
        assert_eq!(
            native_text_index_for_point_in_viewport_with_backend(
                target,
                "abאב",
                Point { x: 30, y: 10 },
                0,
                0,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            4
        );
        assert_eq!(
            native_text_index_for_point_in_viewport_with_backend(
                target,
                "abאב",
                Point { x: 45, y: 10 },
                0,
                0,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            2
        );

        let shaped = mixed_direction_test_shape("abאב", 8).expect("test line should shape");
        assert_eq!(shaped.carets[2].primary_x, 40);
        assert_eq!(shaped.carets[2].secondary_x, 20);
        assert_eq!(
            NativeShapedTextCaret {
                index: 6,
                primary_x: 50,
                secondary_x: 50,
            }
            .closest_cluster_edges(NativeShapedTextCaret {
                index: 7,
                primary_x: 70,
                secondary_x: 40,
            }),
            (50, 40)
        );
    }

    #[test]
    fn shaped_horizontal_navigation_follows_visual_caret_order() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(923),
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 40,
            },
            ViewHitTargetKind::Textbox,
        );
        let backend = NativeTextShapingBackend::Test(mixed_direction_test_shape);
        let mut caret = 0;
        for expected in [1, 4, 3, 2] {
            caret = native_text_index_for_horizontal_move_with_backend(
                target,
                "abאב",
                caret,
                NativeTextVisualHorizontalDirection::Right,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            );
            assert_eq!(caret, expected);
        }
        assert_eq!(
            native_text_index_for_horizontal_move_with_backend(
                target,
                "abאב",
                caret,
                NativeTextVisualHorizontalDirection::Right,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            2
        );
        for expected in [3, 4, 1, 0] {
            caret = native_text_index_for_horizontal_move_with_backend(
                target,
                "abאב",
                caret,
                NativeTextVisualHorizontalDirection::Left,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            );
            assert_eq!(caret, expected);
        }

        let mut selection = NativeTextSelection::collapsed(0);
        for expected in [1, 4] {
            let moved = move_native_text_selection_horizontally_with_backend(
                target,
                "abאב",
                &mut selection,
                NativeTextVisualHorizontalDirection::Right,
                true,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            );
            assert!(moved.selection_changed);
            assert_eq!(selection.anchor, 0);
            assert_eq!(selection.caret, expected);
        }
    }

    #[test]
    fn visual_left_skips_the_duplicate_index_at_a_soft_wrap_boundary() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(924),
            Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 80,
            },
            ViewHitTargetKind::TextEditor,
        );
        assert_eq!(
            native_text_index_for_horizontal_move_with_backend(
                target,
                "abcd",
                2,
                NativeTextVisualHorizontalDirection::Left,
                crate::TextWrap::Word,
                Dpi::standard(),
                &NativeTextShapingBackend::LogicalCells,
            ),
            1
        );
    }

    #[test]
    fn shaped_line_cache_reuses_unchanged_visual_rows() {
        let cache = NativeTextShapingCache::default();
        let calls = std::cell::Cell::new(0_usize);
        let first = cache
            .shape("Wi", || {
                calls.set(calls.get() + 1);
                Some(NativeShapedTextLine::logical_cells("Wi", 8))
            })
            .expect("the first line should shape");
        let second = cache
            .shape("Wi", || {
                calls.set(calls.get() + 1);
                Some(NativeShapedTextLine::logical_cells("Wi", 8))
            })
            .expect("the cached line should remain available");

        assert_eq!(calls.get(), 1);
        assert_eq!(first, second);
    }

    #[test]
    fn shaped_proportional_advances_drive_wrap_scroll_and_vertical_navigation() {
        let backend = NativeTextShapingBackend::Test(proportional_test_shape);
        let lines = text_lines_with_backend("WWii", true, crate::TextWrap::Word, 20, 8, &backend);
        assert_eq!(
            lines
                .iter()
                .map(|line| (line.start, line.end, line.width))
                .collect::<Vec<_>>(),
            vec![(0, 1, 16), (1, 3, 20), (3, 4, 4)]
        );

        let target = ViewHitTarget::with_kind(
            WidgetId::new(922),
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 70,
            },
            ViewHitTargetKind::TextEditor,
        );
        assert_eq!(
            native_text_horizontal_scroll_for_caret_with_backend(
                target,
                "WWii",
                4,
                0,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            17
        );
        assert_eq!(
            native_text_index_for_vertical_move_with_backend(
                target,
                "Wi\nWW",
                1,
                NativeTextVisualDirection::Down,
                None,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            (4, 16)
        );
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

        assert_eq!(
            (wrapped.caret.x, wrapped.caret.y),
            (8, 8 + body_line_height())
        );
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
                height: editor_height_for_visible_rows(2),
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
                0,
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
        let expected_rows = (48.0 / body_line_height() as f32).round().max(1.0) as isize;
        assert_eq!(native_text_wheel_row_delta(Dp::new(48.0)), expected_rows);
        assert_eq!(
            native_text_visible_range_with_backend(
                target,
                value,
                first_visible,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &NativeTextShapingBackend::LogicalCells,
            ),
            10..19
        );
        assert_eq!(
            native_text_first_visible_row_for_index_alignment_with_backend(
                target,
                value,
                10,
                true,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &NativeTextShapingBackend::LogicalCells,
            ),
            2
        );
        assert_eq!(
            native_text_first_visible_row_for_index_alignment_with_backend(
                target,
                value,
                10,
                false,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &NativeTextShapingBackend::LogicalCells,
            ),
            1
        );

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
            0,
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
        assert_eq!(geometry.caret.y, 8 + body_line_height());
        assert!(matches!(
            plan.commands.first(),
            Some(NativeDrawCommand::PushClip { .. })
        ));
        assert!(matches!(
            plan.commands.last(),
            Some(NativeDrawCommand::PopClip)
        ));
    }

    #[test]
    fn no_wrap_editor_viewport_reveals_columns_and_offsets_pointer_hits() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(97),
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 52,
            },
            ViewHitTargetKind::TextEditor,
        );
        let value = "0123456789\nabc";
        let backend = NativeTextShapingBackend::LogicalCells;
        let horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            target,
            value,
            10,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
            &backend,
        );

        assert_eq!(horizontal_scroll_px, 49);
        assert_eq!(
            native_text_index_for_point_in_viewport_with_backend(
                target,
                value,
                Point { x: 8, y: 10 },
                0,
                horizontal_scroll_px,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            6
        );
        assert_eq!(
            native_text_horizontal_scroll_for_caret_with_backend(
                target,
                value,
                10,
                horizontal_scroll_px,
                crate::TextWrap::Word,
                Dpi::standard(),
                &backend,
            ),
            0
        );

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
        let geometry = decorate_native_text_edit_visuals_in_viewport_with_backend(
            &mut plan,
            target,
            value,
            NativeTextSelection::collapsed(10),
            0,
            horizontal_scroll_px,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
            &backend,
        );
        let visible_text = plan
            .commands
            .iter()
            .filter_map(|command| match command {
                NativeDrawCommand::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(visible_text.first().copied(), Some("0123456789"));
        assert_eq!(geometry.caret.x, 39);
    }

    #[test]
    fn editor_drag_edges_scroll_one_visual_step_before_hit_testing() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(100),
            Rect {
                x: 0,
                y: 0,
                width: 160,
                height: editor_height_for_visible_rows(3),
            },
            ViewHitTargetKind::TextEditor,
        );
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let backend = NativeTextShapingBackend::LogicalCells;
        let dragged = native_text_drag_viewport_for_point_with_backend(
            target,
            value,
            Point { x: 16, y: 500 },
            0,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
            &backend,
        );

        assert!(dragged.scrolled);
        assert_eq!(
            (dragged.first_visible_row, dragged.point.y),
            (1, 8 + body_line_height() * 2)
        );
        assert_eq!(
            native_text_index_for_point_in_viewport_with_backend(
                target,
                value,
                dragged.point,
                dragged.first_visible_row,
                dragged.horizontal_scroll_px,
                crate::TextWrap::NoWrap,
                Dpi::standard(),
                &backend,
            ),
            10
        );

        let long_line = native_text_drag_viewport_for_point_with_backend(
            ViewHitTarget::with_kind(
                WidgetId::new(101),
                Rect {
                    x: 0,
                    y: 0,
                    width: 48,
                    height: 52,
                },
                ViewHitTargetKind::TextEditor,
            ),
            "0123456789",
            Point { x: 500, y: 10 },
            0,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
            &backend,
        );
        assert_eq!(
            (
                long_line.horizontal_scroll_px,
                long_line.point.x,
                long_line.scrolled
            ),
            (8, 39, true)
        );
    }

    #[cfg(feature = "text-input-core")]
    #[test]
    fn text_geometry_hit_testing_and_wrap_use_extended_grapheme_columns() {
        let textbox = ViewHitTarget::with_kind(
            WidgetId::new(102),
            Rect {
                x: 0,
                y: 0,
                width: 160,
                height: 44,
            },
            ViewHitTargetKind::Textbox,
        );
        let value = "A\u{65}\u{301}👩🏽‍💻Z";
        let geometry = native_text_visual_geometry(
            textbox,
            value,
            NativeTextSelection::collapsed(3),
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );
        assert_eq!(geometry.caret.x, 24);
        assert_eq!(
            native_text_index_for_point(
                textbox,
                value,
                Point { x: 24, y: 10 },
                crate::TextWrap::NoWrap,
                Dpi::standard(),
            ),
            3
        );
        assert_eq!(
            native_text_index_for_point(
                textbox,
                value,
                Point { x: 32, y: 10 },
                crate::TextWrap::NoWrap,
                Dpi::standard(),
            ),
            7
        );
        let selected = native_text_visual_geometry(
            textbox,
            value,
            NativeTextSelection {
                anchor: 1,
                caret: 7,
            },
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );
        assert_eq!(selected.selections[0].width, 16);

        let wrapped = text_lines("abc👩🏽‍💻d", true, crate::TextWrap::Word, 4);
        assert_eq!((wrapped[0].start, wrapped[0].end), (0, 7));
        assert!(wrapped[0].soft_wrap_after);
        assert_eq!((wrapped[1].start, wrapped[1].end), (7, 8));
    }

    #[test]
    fn editor_page_move_preserves_column_and_scrolls_one_visual_page() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(98),
            Rect {
                x: 0,
                y: 0,
                width: 160,
                height: editor_height_for_visible_rows(3),
            },
            ViewHitTargetKind::TextEditor,
        );
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";

        let (down, preferred, first_visible) = native_text_index_for_vertical_page_move(
            target,
            value,
            1,
            NativeTextVisualDirection::Down,
            None,
            0,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );
        assert_eq!((down, preferred, first_visible), (10, 1, 3));

        let (up, preferred, first_visible) = native_text_index_for_vertical_page_move(
            target,
            value,
            down,
            NativeTextVisualDirection::Up,
            Some(preferred),
            first_visible,
            crate::TextWrap::NoWrap,
            Dpi::standard(),
        );
        assert_eq!((up, preferred, first_visible), (1, 1, 0));
    }

    #[test]
    fn editor_page_move_counts_soft_wrapped_visual_rows() {
        let target = ViewHitTarget::with_kind(
            WidgetId::new(99),
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: editor_height_for_visible_rows(2),
            },
            ViewHitTargetKind::TextEditor,
        );

        let moved = native_text_index_for_vertical_page_move(
            target,
            "abcdefghijkl",
            1,
            NativeTextVisualDirection::Down,
            None,
            0,
            crate::TextWrap::Word,
            Dpi::standard(),
        );

        assert_eq!(moved, (9, 1, 1));
    }
}
