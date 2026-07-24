#[cfg(feature = "virtual-list")]
pub fn virtual_list_viewport(
    total_count: usize,
    row_height: Dp,
    offset_y: Dp,
    viewport_height: Dp,
    overscan_rows: usize,
    direction: VirtualListScrollDirection,
) -> VirtualListViewport {
    items_repeater_viewport_with_metrics(
        total_count,
        row_height,
        &[],
        offset_y,
        viewport_height,
        overscan_rows,
        direction,
    )
}

/// Computes a controlled ItemsRepeater viewport from one estimated height and
/// a sparse set of known variable-height item metrics.
#[cfg(feature = "virtual-list")]
pub fn items_repeater_viewport_with_metrics(
    total_count: usize,
    row_height: Dp,
    item_metrics: &[ZsItemsRepeaterItemMetric],
    offset_y: Dp,
    viewport_height: Dp,
    overscan_rows: usize,
    direction: VirtualListScrollDirection,
) -> VirtualListViewport {
    let row_height = normalized_items_repeater_estimated_height(row_height);
    let item_metrics = normalized_items_repeater_metrics(total_count, item_metrics);
    let viewport_height = if viewport_height.0.is_finite() {
        viewport_height.0.max(0.0)
    } else {
        0.0
    };
    let requested_offset = if offset_y.0.is_finite() {
        offset_y.0.max(0.0)
    } else {
        0.0
    };
    let content_height = items_repeater_content_height_normalized(
        total_count,
        row_height,
        &item_metrics,
    );
    let max_offset = (content_height - viewport_height as f64).max(0.0) as f32;
    let offset_y = requested_offset.min(max_offset);
    if total_count == 0 || viewport_height <= 0.0 {
        return VirtualListViewport {
            offset_y: Dp::new(offset_y),
            row_height: Dp::new(row_height),
            visible_range: VirtualListRange::new(0, 0),
            materialized_range: VirtualListRange::new(0, 0),
            direction,
        };
    }

    let offset = f64::from(offset_y);
    let viewport_end = offset + f64::from(viewport_height);
    let mut start_low = 0usize;
    let mut start_high = total_count;
    while start_low < start_high {
        let middle = start_low + (start_high - start_low) / 2;
        let bottom = items_repeater_height_before_normalized(
            middle.saturating_add(1),
            row_height,
            &item_metrics,
        );
        if bottom <= offset {
            start_low = middle.saturating_add(1);
        } else {
            start_high = middle;
        }
    }
    let start = start_low.min(total_count);
    let mut end_low = start.saturating_add(1).min(total_count);
    let mut end_high = total_count;
    while end_low < end_high {
        let middle = end_low + (end_high - end_low) / 2;
        let top = items_repeater_height_before_normalized(middle, row_height, &item_metrics);
        if top < viewport_end {
            end_low = middle.saturating_add(1);
        } else {
            end_high = middle;
        }
    }
    let end = end_low.max(start.saturating_add(1)).min(total_count);
    let visible_range = VirtualListRange::new(start, end);
    let materialized_range = VirtualListRange::new(
        start.saturating_sub(overscan_rows),
        end.saturating_add(overscan_rows).min(total_count),
    );
    VirtualListViewport {
        offset_y: Dp::new(offset_y),
        row_height: Dp::new(row_height),
        visible_range,
        materialized_range,
        direction,
    }
}

#[cfg(feature = "virtual-list")]
fn normalized_items_repeater_estimated_height(row_height: Dp) -> f32 {
    if row_height.0.is_finite() {
        row_height.0.max(1.0)
    } else {
        1.0
    }
}

#[cfg(feature = "virtual-list")]
fn normalized_items_repeater_metrics(
    total_count: usize,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> Vec<ZsItemsRepeaterItemMetric> {
    let mut metrics = item_metrics
        .iter()
        .copied()
        .filter(|metric| metric.index < total_count)
        .map(|metric| ZsItemsRepeaterItemMetric::new(metric.index, metric.height))
        .collect::<Vec<_>>();
    metrics.sort_by_key(|metric| metric.index);
    let mut normalized = Vec::with_capacity(metrics.len());
    for metric in metrics {
        if normalized
            .last()
            .is_some_and(|current: &ZsItemsRepeaterItemMetric| current.index == metric.index)
        {
            *normalized.last_mut().expect("metric entry must exist") = metric;
        } else {
            normalized.push(metric);
        }
    }
    normalized
}

#[cfg(feature = "virtual-list")]
fn items_repeater_height_before_normalized(
    index: usize,
    estimated_height: f32,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> f64 {
    let mut height = index as f64 * f64::from(estimated_height);
    for metric in item_metrics.iter().take_while(|metric| metric.index < index) {
        height += f64::from(metric.height.0 - estimated_height);
    }
    height.max(0.0)
}

#[cfg(feature = "virtual-list")]
fn items_repeater_content_height_normalized(
    total_count: usize,
    estimated_height: f32,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> f64 {
    items_repeater_height_before_normalized(total_count, estimated_height, item_metrics)
}

#[cfg(feature = "virtual-list")]
pub(crate) fn items_repeater_content_height(
    total_count: usize,
    row_height: Dp,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> Dp {
    let row_height = normalized_items_repeater_estimated_height(row_height);
    let metrics = normalized_items_repeater_metrics(total_count, item_metrics);
    Dp::new(
        items_repeater_content_height_normalized(total_count, row_height, &metrics)
            .min(f64::from(f32::MAX)) as f32,
    )
}

#[cfg(feature = "virtual-list")]
pub(crate) fn items_repeater_item_height(
    index: usize,
    row_height: Dp,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> Dp {
    item_metrics
        .binary_search_by_key(&index, |metric| metric.index)
        .ok()
        .and_then(|position| item_metrics.get(position))
        .map(|metric| metric.height)
        .unwrap_or_else(|| Dp::new(normalized_items_repeater_estimated_height(row_height)))
}

#[cfg(feature = "virtual-list")]
pub(crate) fn items_repeater_height_before(
    index: usize,
    row_height: Dp,
    item_metrics: &[ZsItemsRepeaterItemMetric],
) -> Dp {
    let row_height = normalized_items_repeater_estimated_height(row_height);
    Dp::new(
        items_repeater_height_before_normalized(index, row_height, item_metrics)
            .min(f64::from(f32::MAX)) as f32,
    )
}

#[cfg(feature = "virtual-list")]
fn virtual_list_row_bounds(
    bounds: Rect,
    index: usize,
    row_height: Dp,
    item_metrics: &[ZsItemsRepeaterItemMetric],
    offset_y: Dp,
    dpi: Dpi,
) -> Rect {
    let row_height_px = items_repeater_item_height(index, row_height, item_metrics)
        .to_px(dpi)
        .round_i32()
        .max(1);
    let offset_px = offset_y.to_px(dpi).round_i32().max(0);
    let row_top = i64::from(
        items_repeater_height_before(index, row_height, item_metrics)
            .to_px(dpi)
            .round_i32(),
    )
        .saturating_sub(offset_px as i64)
        .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Rect {
        x: bounds.x,
        y: bounds.y.saturating_add(row_top),
        width: bounds.width,
        height: row_height_px,
    }
}

#[cfg(feature = "virtual-list")]
fn items_repeater_scrollbar_layout<Msg>(
    node: &ViewNode<Msg>,
) -> Option<ZsItemsRepeaterScrollbarLayout> {
    let bounds = node.bounds?;
    let ViewNodeKind::VirtualList {
        total_count,
        row_height,
        item_metrics,
        offset_y,
        ..
    } = &node.kind
    else {
        return None;
    };
    let content_bounds = inset_bounds(bounds, node.style.padding, node.layout_dpi);
    let content_height = items_repeater_content_height(*total_count, *row_height, item_metrics);
    let viewport_height = Dp::new(
        content_bounds.height.max(0) as f32
            / node.layout_dpi.scale_factor().max(f32::EPSILON),
    );
    let maximum_offset = Dp::new((content_height.0 - viewport_height.0).max(0.0));
    if maximum_offset.0 <= 0.0 || content_bounds.height <= 0 || content_bounds.width <= 0 {
        return None;
    }

    let profile = crate::platform_component_profile::PlatformComponentProfile::for_style(
        node.resolved_platform_style(),
    )
    .shell;
    let bar_width = profile
        .scrollbar_width
        .to_px(node.layout_dpi)
        .round_i32()
        .max(1);
    let margin = profile
        .scrollbar_margin
        .to_px(node.layout_dpi)
        .round_i32()
        .max(0);
    let layout = crate::shell_layout::ZsShellScrollLayout::new(
        content_bounds.y,
        content_bounds.y.saturating_add(content_bounds.height),
        content_height.to_px(node.layout_dpi).round_i32().max(0),
        content_bounds.height,
        content_bounds.x.saturating_add(content_bounds.width),
        margin,
        bar_width,
        node.layout_dpi,
    );
    let offset_px = offset_y.to_px(node.layout_dpi).round_i32().max(0);
    let track = layout.track_rect()?;
    let thumb = layout.thumb_rect(offset_px)?;
    let thumb_hit = layout.thumb_hit_rect(
        offset_px,
        Dp::new(4.0).to_px(node.layout_dpi).round_i32().max(0),
    )?;
    let to_rect = |rect: crate::UiRect| Rect {
        x: rect.left,
        y: rect.top,
        width: (rect.right - rect.left).max(0),
        height: (rect.bottom - rect.top).max(0),
    };
    Some(ZsItemsRepeaterScrollbarLayout {
        track: to_rect(track),
        thumb: to_rect(thumb),
        thumb_hit: to_rect(thumb_hit),
        maximum_offset,
    })
}

fn split_child_bounds<Msg>(
    bounds: Rect,
    kind: &ViewNodeKind<Msg>,
    children: &[ViewNode<Msg>],
    gap: Option<Dp>,
    dpi: Dpi,
    typography_scale: f32,
) -> Vec<Rect> {
    let child_count = children.len();
    if child_count == 0 {
        return Vec::new();
    }
    let gap = gap
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);

    match kind {
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => {
            let widths = allocate_axis_lengths(
                bounds.width,
                gap,
                children,
                |style| style.width,
                |style| style.min_width,
                dpi,
                false,
                typography_scale,
                bounds.height,
            );
            let mut x = bounds.x;
            widths
                .into_iter()
                .zip(children)
                .map(|(width, child)| {
                    let height = cross_axis_length(
                        bounds.height,
                        child,
                        dpi,
                        true,
                        typography_scale,
                        width,
                    );
                    let rect = Rect {
                        x,
                        y: bounds
                            .y
                            .saturating_add(bounds.height.saturating_sub(height) / 2),
                        width,
                        height,
                    };
                    x += width + gap;
                    rect
                })
                .collect()
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => split_column_child_bounds(bounds, children, gap, dpi, typography_scale),
        #[cfg(feature = "list")]
        ViewNodeKind::List { .. } => {
            split_column_child_bounds(bounds, children, gap, dpi, typography_scale)
        }
        #[cfg(feature = "scroll")]
        ViewNodeKind::Scroll {
            offset_y,
            content_height,
            ..
        } => {
            let offset_y = offset_y.to_px(dpi).round_i32().max(0);
            let height = content_height
                .map(|height| height.to_px(dpi).round_i32())
                .unwrap_or(bounds.height)
                .max(bounds.height);
            vec![
                Rect {
                    x: bounds.x,
                    y: bounds.y - offset_y,
                    width: bounds.width,
                    height,
                };
                child_count
            ]
        }
        #[cfg(feature = "grid")]
        ViewNodeKind::Grid {
            columns,
            rows,
            placements,
            column_gap,
            row_gap,
        } => split_grid_child_bounds(
            bounds,
            columns,
            rows,
            placements,
            children,
            column_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            row_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            dpi,
            typography_scale,
        ),
        _ => vec![bounds; child_count],
    }
}

#[cfg(feature = "grid")]
fn split_grid_child_bounds<Msg>(
    bounds: Rect,
    columns: &[ZsGridTrack],
    rows: &[ZsGridTrack],
    placements: &[ZsGridPlacement],
    children: &[ViewNode<Msg>],
    column_gap: i32,
    row_gap: i32,
    dpi: Dpi,
    typography_scale: f32,
) -> Vec<Rect> {
    let column_minimums = grid_track_minimums(
        columns.len(),
        placements,
        children,
        false,
        dpi,
        typography_scale,
    );
    let row_minimums = grid_track_minimums(
        rows.len(),
        placements,
        children,
        true,
        dpi,
        typography_scale,
    );
    let column_lengths =
        allocate_grid_track_lengths(bounds.width, column_gap, columns, &column_minimums, dpi);
    let row_lengths =
        allocate_grid_track_lengths(bounds.height, row_gap, rows, &row_minimums, dpi);
    (0..children.len())
        .map(|index| {
            placements
                .get(index)
                .and_then(|placement| {
                    grid_placement_bounds(
                        bounds,
                        &column_lengths,
                        &row_lengths,
                        column_gap,
                        row_gap,
                        *placement,
                    )
                })
                .map(|cell| constrained_child_bounds(cell, &children[index], dpi, typography_scale))
                .unwrap_or(Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: 0,
                    height: 0,
                })
        })
        .collect()
}

/// Computes controlled ItemsRepeater viewport ranges without constructing a
/// View tree.
#[cfg(feature = "virtual-list")]
pub fn items_repeater_viewport(
    total_count: usize,
    row_height: Dp,
    offset_y: Dp,
    viewport_height: Dp,
    overscan_rows: usize,
    direction: ZsItemsRepeaterScrollDirection,
) -> ZsItemsRepeaterViewport {
    virtual_list_viewport(
        total_count,
        row_height,
        offset_y,
        viewport_height,
        overscan_rows,
        direction,
    )
}

#[cfg(feature = "grid")]
fn grid_track_minimums<Msg>(
    track_count: usize,
    placements: &[ZsGridPlacement],
    children: &[ViewNode<Msg>],
    vertical: bool,
    dpi: Dpi,
    typography_scale: f32,
) -> Vec<i32> {
    let mut minimums = vec![0; track_count];
    for (placement, child) in placements.iter().zip(children) {
        let (track, span) = if vertical {
            (placement.row, placement.row_span.get())
        } else {
            (placement.column, placement.column_span.get())
        };
        if span != 1 || track >= track_count {
            continue;
        }
        let (fixed, minimum) = if vertical {
            (child.style.height, child.style.min_height)
        } else {
            (child.style.width, child.style.min_width)
        };
        let fixed = fixed
            .map(|value| {
                typography_aware_length_px(
                    value,
                    dpi,
                    vertical && child.typography_scaled_height,
                    typography_scale,
                )
            })
            .unwrap_or(0);
        let minimum = minimum
            .map(|value| {
                typography_aware_length_px(
                    value,
                    dpi,
                    vertical && child.typography_scaled_height,
                    typography_scale,
                )
            })
            .unwrap_or(0);
        let intrinsic = if vertical {
            intrinsic_min_height_px(child, 0, dpi, typography_scale)
        } else {
            intrinsic_min_width_px(child, dpi, typography_scale)
        };
        minimums[track] = minimums[track].max(fixed.max(minimum).max(intrinsic));
    }
    minimums
}

#[cfg(feature = "grid")]
fn allocate_grid_track_lengths(
    total: i32,
    gap: i32,
    tracks: &[ZsGridTrack],
    minimums: &[i32],
    dpi: Dpi,
) -> Vec<i32> {
    if tracks.is_empty() {
        return Vec::new();
    }
    let total = total.max(0);
    let total_gap = gap
        .max(0)
        .saturating_mul(tracks.len().saturating_sub(1) as i32)
        .min(total);
    let available = total.saturating_sub(total_gap);
    let requested = tracks
        .iter()
        .map(|track| match track {
            ZsGridTrack::Fixed(size) => Some(size.to_px(dpi).round_i32().max(0)),
            ZsGridTrack::Fraction(_) => None,
        })
        .collect::<Vec<_>>();
    let mut lengths = requested
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .unwrap_or(0)
                .max(minimums.get(index).copied().unwrap_or(0))
        })
        .collect::<Vec<_>>();
    let base_total = lengths
        .iter()
        .copied()
        .fold(0i32, i32::saturating_add);

    if base_total >= available {
        // Grid uses the same hard-size contract as Stack: fixed tracks and
        // native control/text minimums never collapse to fit a smaller slot.
        // The parent viewport clips or scrolls the overflow instead of
        // compressing a line box or control below its platform metric.
        return lengths;
    }

    let fractional_indices = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| match track {
            ZsGridTrack::Fraction(weight) => Some((index, weight.get())),
            ZsGridTrack::Fixed(_) => None,
        })
        .collect::<Vec<_>>();
    if fractional_indices.is_empty() {
        return lengths;
    }
    let fixed_total = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| matches!(track, ZsGridTrack::Fixed(_)).then_some(lengths[index]))
        .fold(0i32, i32::saturating_add);
    let mut unresolved = fractional_indices;
    let mut distributable = available.saturating_sub(fixed_total).max(0);
    loop {
        let total_weight = unresolved.iter().fold(0u64, |total, (_, weight)| {
            total.saturating_add(u64::from(*weight))
        });
        let constrained = unresolved
            .iter()
            .filter_map(|(index, weight)| {
                let weighted_share = ((distributable as u128 * u128::from(*weight))
                    / u128::from(total_weight))
                .min(i32::MAX as u128) as i32;
                (weighted_share < lengths[*index]).then_some(*index)
            })
            .collect::<Vec<_>>();
        if constrained.is_empty() {
            let mut assigned = 0;
            for (position, (index, weight)) in unresolved.iter().enumerate() {
                let length = if position + 1 == unresolved.len() {
                    distributable.saturating_sub(assigned)
                } else {
                    ((distributable as u128 * u128::from(*weight))
                        / u128::from(total_weight))
                    .min(i32::MAX as u128) as i32
                };
                lengths[*index] = length.max(lengths[*index]);
                assigned = assigned.saturating_add(length);
            }
            break;
        }
        unresolved.retain(|(index, _)| {
            if constrained.contains(index) {
                distributable = distributable.saturating_sub(lengths[*index]);
                false
            } else {
                true
            }
        });
        if unresolved.is_empty() {
            break;
        }
    }
    lengths
}

#[cfg(any(feature = "grid", feature = "tabs"))]
fn constrained_child_bounds<Msg>(
    cell: Rect,
    child: &ViewNode<Msg>,
    dpi: Dpi,
    typography_scale: f32,
) -> Rect {
    let width = constrained_child_axis_length(
        cell.width,
        child,
        dpi,
        false,
        typography_scale,
        cell.height,
    );
    let height = constrained_child_axis_length(
        cell.height,
        child,
        dpi,
        true,
        typography_scale,
        width,
    );
    Rect {
        x: cell.x,
        y: cell.y,
        width,
        height,
    }
}

#[cfg(any(feature = "grid", feature = "tabs"))]
fn constrained_child_axis_length<Msg>(
    available: i32,
    child: &ViewNode<Msg>,
    dpi: Dpi,
    vertical: bool,
    typography_scale: f32,
    measurement_cross: i32,
) -> i32 {
    let fixed = if vertical {
        child.style.height
    } else {
        child.style.width
    };
    let minimum = if vertical {
        child.style.min_height
    } else {
        child.style.min_width
    }
        .map(|value| {
            typography_aware_length_px(
                value,
                dpi,
                vertical && child.typography_scaled_height,
                typography_scale,
            )
        })
        .unwrap_or(0);
    let minimum = minimum.max(if vertical {
        intrinsic_min_height_px(child, measurement_cross, dpi, typography_scale)
    } else {
        intrinsic_min_width_px(child, dpi, typography_scale)
    });
    fixed
        .map(|value| {
            typography_aware_length_px(
                value,
                dpi,
                vertical && child.typography_scaled_height,
                typography_scale,
            )
            .max(minimum)
        })
        .unwrap_or_else(|| available.max(minimum))
}

#[cfg(feature = "grid")]
fn grid_placement_bounds(
    bounds: Rect,
    column_lengths: &[i32],
    row_lengths: &[i32],
    column_gap: i32,
    row_gap: i32,
    placement: ZsGridPlacement,
) -> Option<Rect> {
    let (x, width) = grid_axis_span(
        bounds.x,
        column_lengths,
        column_gap,
        placement.column,
        placement.column_span,
    )?;
    let (y, height) = grid_axis_span(
        bounds.y,
        row_lengths,
        row_gap,
        placement.row,
        placement.row_span,
    )?;
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

#[cfg(feature = "grid")]
fn grid_axis_span(
    origin: i32,
    lengths: &[i32],
    gap: i32,
    start: usize,
    span: ZsGridSpan,
) -> Option<(i32, i32)> {
    if start >= lengths.len() {
        return None;
    }
    let gap = gap.max(0);
    let before = lengths[..start]
        .iter()
        .fold(0i32, |total, length| total.saturating_add(*length))
        .saturating_add(gap.saturating_mul(start as i32));
    let end = start
        .saturating_add(usize::from(span.get()))
        .min(lengths.len());
    let covered_tracks = end.saturating_sub(start);
    let length = lengths[start..end]
        .iter()
        .fold(0i32, |total, length| total.saturating_add(*length))
        .saturating_add(gap.saturating_mul(covered_tracks.saturating_sub(1) as i32));
    Some((origin.saturating_add(before), length.max(0)))
}

fn split_column_child_bounds<Msg>(
    bounds: Rect,
    children: &[ViewNode<Msg>],
    gap: i32,
    dpi: Dpi,
    typography_scale: f32,
) -> Vec<Rect> {
    let heights = allocate_axis_lengths(
        bounds.height,
        gap,
        children,
        |style| style.height,
        |style| style.min_height,
        dpi,
        true,
        typography_scale,
        bounds.width,
    );
    let mut y = bounds.y;
    heights
        .into_iter()
        .zip(children)
        .map(|(height, child)| {
            let width = cross_axis_length(
                bounds.width,
                child,
                dpi,
                false,
                typography_scale,
                height,
            );
            let rect = Rect {
                x: bounds.x,
                y,
                width,
                height,
            };
            y += height + gap;
            rect
        })
        .collect()
}

#[cfg(feature = "label")]
fn navigation_intrinsic_height_px<Msg>(
    node: &ViewNode<Msg>,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    let explicit = node
        .style
        .height
        .or(node.style.min_height)
        .map(|height| {
            typography_aware_length_px(
                height,
                dpi,
                node.typography_scaled_height,
                typography_scale,
            )
        });
    if let Some(explicit) = explicit {
        return explicit.max(1);
    }
    let gap = node
        .style
        .gap
        .map(|gap| gap.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    let padding = node
        .style
        .padding
        .map(|padding| padding.to_px(dpi).round_i32().max(0))
        .unwrap_or(0)
        .saturating_mul(2);
    let content = match &node.kind {
        ViewNodeKind::Text { style, .. } => Dp::new(
            style.role
                .metrics_for(crate::ZsTypographyPlatformStyle::current())
                .line_height
                * typography_scale,
        )
        .to_px(dpi)
        .round_i32()
        .max(1),
        #[cfg(feature = "icon")]
        ViewNodeKind::Icon { size, .. } => standalone_icon_size_px(node, *size, dpi),
        #[cfg(feature = "badge")]
        ViewNodeKind::Badge { content, .. } => {
            standalone_badge_size_px(node, *content, dpi, typography_scale).1
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => node
            .children
            .iter()
            .map(|child| navigation_intrinsic_height_px(child, dpi, typography_scale))
            .max()
            .unwrap_or(0),
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => node
            .children
            .iter()
            .map(|child| navigation_intrinsic_height_px(child, dpi, typography_scale))
            .fold(0i32, i32::saturating_add)
            .saturating_add(gap.saturating_mul(node.children.len().saturating_sub(1) as i32)),
        ViewNodeKind::Spacer => 0,
        _ => crate::ZsBaseControlMetrics::for_platform(
            crate::ZsBaseControlPlatformStyle::current(),
        )
        .button_height
        .to_px(dpi)
        .round_i32()
        .max(1),
    };
    content.saturating_add(padding).max(1)
}

fn allocate_axis_lengths<Msg>(
    total: i32,
    gap: i32,
    children: &[ViewNode<Msg>],
    fixed: impl Fn(&ViewStyle) -> Option<Dp>,
    minimum: impl Fn(&ViewStyle) -> Option<Dp>,
    dpi: Dpi,
    vertical: bool,
    typography_scale: f32,
    cross_available: i32,
) -> Vec<i32> {
    let total = total.max(0);
    let total_gap = gap
        .saturating_mul(children.len().saturating_sub(1) as i32)
        .min(total);
    let available = total - total_gap;
    let requested = children
        .iter()
        .map(|child| {
            fixed(&child.style).map(|value| {
                typography_aware_length_px(
                    value,
                    dpi,
                    vertical && child.typography_scaled_height,
                    typography_scale,
                )
            })
        })
        .collect::<Vec<_>>();
    let minimums = children
        .iter()
        .map(|child| {
            let declared = minimum(&child.style)
                .map(|value| {
                    typography_aware_length_px(
                        value,
                        dpi,
                        vertical && child.typography_scaled_height,
                        typography_scale,
                    )
                })
                .unwrap_or(0);
            let intrinsic = if vertical {
                intrinsic_min_height_px(child, cross_available, dpi, typography_scale)
            } else {
                intrinsic_min_width_px(child, dpi, typography_scale)
            };
            declared.max(intrinsic)
        })
        .collect::<Vec<_>>();
    let mut lengths = requested
        .iter()
        .zip(&minimums)
        .map(|(fixed, minimum)| fixed.unwrap_or(*minimum).max(*minimum))
        .collect::<Vec<_>>();
    let base_total: i32 = lengths.iter().copied().sum();

    if base_total >= available {
        // Explicit and minimum sizes are hard layout contracts. Scaling them
        // down made buttons and line boxes narrower/shorter than their native
        // text metrics, which produced accidental glyph clipping. An
        // over-constrained stack now overflows its viewport; callers can use
        // Scroll or an adaptive/overflow composition without corrupting the
        // controls themselves.
        return lengths;
    }

    let remaining = (available - base_total).max(0);
    let flexible = requested
        .iter()
        .enumerate()
        .filter(|(index, value)| {
            value.is_none() && children[*index].style.flex > f32::EPSILON
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if flexible.is_empty() {
        return lengths;
    }

    let flex_total: f32 = flexible
        .iter()
        .map(|index| children[*index].style.flex.max(0.0))
        .sum();
    let denominator = flex_total;
    let mut assigned = 0;
    for (position, index) in flexible.iter().enumerate() {
        let length = if position + 1 == flexible.len() {
            remaining - assigned
        } else {
            let weight = children[*index].style.flex.max(0.0);
            ((remaining as f32) * weight / denominator).floor() as i32
        }
        .max(0);
        lengths[*index] = lengths[*index].saturating_add(length);
        assigned += length;
    }
    lengths
}

fn cross_axis_length<Msg>(
    available: i32,
    child: &ViewNode<Msg>,
    dpi: Dpi,
    vertical: bool,
    typography_scale: f32,
    measurement_cross: i32,
) -> i32 {
    let available = available.max(0);
    let fixed = if vertical {
        child.style.height
    } else {
        child.style.width
    };
    let declared_minimum = if vertical {
        child.style.min_height
    } else {
        child.style.min_width
    };
    let declared_minimum = declared_minimum
        .map(|value| {
            typography_aware_length_px(
                value,
                dpi,
                vertical && child.typography_scaled_height,
                typography_scale,
            )
        })
        .unwrap_or(0);
    let intrinsic = if vertical {
        intrinsic_min_height_px(child, measurement_cross, dpi, typography_scale)
    } else {
        intrinsic_min_width_px(child, dpi, typography_scale)
    };
    let minimum = declared_minimum.max(intrinsic);
    // A stack owns its cross-axis cell, and text fills the available width so
    // wrapping is measured against the real line box. `flex` only distributes
    // the parent's main axis; it must not collapse either case cross-axis.
    #[cfg(feature = "label")]
    let text_stretches_width =
        !vertical && matches!(&child.kind, ViewNodeKind::Text { .. });
    #[cfg(not(feature = "label"))]
    let text_stretches_width = false;
    let stretches_cross_axis =
        matches!(&child.kind, ViewNodeKind::Stack { .. }) || text_stretches_width;
    fixed
        .map(|value| {
            typography_aware_length_px(
                value,
                dpi,
                vertical && child.typography_scaled_height,
                typography_scale,
            )
                .max(minimum)
        })
        .or_else(|| {
            (child.style.flex <= f32::EPSILON && minimum > 0 && !stretches_cross_axis)
                .then_some(minimum)
        })
        .unwrap_or_else(|| available.max(minimum))
}

fn intrinsic_min_width_px<Msg>(
    node: &ViewNode<Msg>,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    let declared = node
        .style
        .width
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0)
        .max(
            node.style
                .min_width
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(0),
        );
    let padding = node
        .style
        .padding
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    let gap = node
        .style
        .gap
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);

    #[cfg(feature = "scroll")]
    if matches!(node.kind, ViewNodeKind::Scroll { .. }) {
        return declared;
    }

    let content = match &node.kind {
        #[cfg(feature = "label")]
        ViewNodeKind::Text { text, style } => {
            estimated_text_min_width_px(text, *style, dpi, typography_scale)
        }
        #[cfg(feature = "icon")]
        ViewNodeKind::Icon { size, .. } => standalone_icon_size_px(node, *size, dpi),
        #[cfg(feature = "badge")]
        ViewNodeKind::Badge { content, .. } => {
            standalone_badge_size_px(node, *content, dpi, typography_scale).0
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => node
            .children
            .iter()
            .map(|child| intrinsic_min_width_px(child, dpi, typography_scale))
            .fold(0i32, i32::saturating_add)
            .saturating_add(gap.saturating_mul(node.children.len().saturating_sub(1) as i32)),
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => node
            .children
            .iter()
            .map(|child| intrinsic_min_width_px(child, dpi, typography_scale))
            .max()
            .unwrap_or(0),
        #[cfg(feature = "list")]
        ViewNodeKind::List { .. } => node
            .children
            .iter()
            .map(|child| intrinsic_min_width_px(child, dpi, typography_scale))
            .max()
            .unwrap_or(0),
        ViewNodeKind::Spacer => 0,
        _ => node
            .children
            .iter()
            .map(|child| intrinsic_min_width_px(child, dpi, typography_scale))
            .max()
            .unwrap_or(0),
    };
    declared.max(content.saturating_add(padding.saturating_mul(2)))
}

fn intrinsic_min_height_px<Msg>(
    node: &ViewNode<Msg>,
    available_width: i32,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    let declared_height = |value| {
        typography_aware_length_px(
            value,
            dpi,
            node.typography_scaled_height,
            typography_scale,
        )
    };
    let declared = node
        .style
        .height
        .map(declared_height)
        .unwrap_or(0)
        .max(
            node.style
                .min_height
                .map(declared_height)
                .unwrap_or(0),
        );
    let padding = node
        .style
        .padding
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    let gap = node
        .style
        .gap
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    let content_width = available_width.saturating_sub(padding.saturating_mul(2));

    #[cfg(feature = "scroll")]
    if matches!(node.kind, ViewNodeKind::Scroll { .. }) {
        return declared;
    }

    let content = match &node.kind {
        #[cfg(feature = "label")]
        ViewNodeKind::Text { text, style } => {
            estimated_text_min_height_px(text, *style, content_width, dpi, typography_scale)
        }
        #[cfg(feature = "icon")]
        ViewNodeKind::Icon { size, .. } => standalone_icon_size_px(node, *size, dpi),
        #[cfg(feature = "badge")]
        ViewNodeKind::Badge { content, .. } => {
            standalone_badge_size_px(node, *content, dpi, typography_scale).1
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => {
            // A row's height depends on the widths that its children actually
            // receive. Measuring wrapped text at its shortest unbreakable
            // segment would invent many extra lines and vertically displace
            // compact controls beside it.
            let widths = allocate_axis_lengths(
                content_width,
                gap,
                &node.children,
                |style| style.width,
                |style| style.min_width,
                dpi,
                false,
                typography_scale,
                0,
            );
            node.children
                .iter()
                .zip(widths)
                .map(|(child, width)| {
                    intrinsic_min_height_px(child, width, dpi, typography_scale)
                })
                .max()
                .unwrap_or(0)
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => node
            .children
            .iter()
            .map(|child| {
                intrinsic_min_height_px(child, content_width, dpi, typography_scale)
            })
            .fold(0i32, i32::saturating_add)
            .saturating_add(gap.saturating_mul(node.children.len().saturating_sub(1) as i32)),
        #[cfg(feature = "list")]
        ViewNodeKind::List { .. } => node
            .children
            .iter()
            .map(|child| {
                intrinsic_min_height_px(child, content_width, dpi, typography_scale)
            })
            .fold(0i32, i32::saturating_add)
            .saturating_add(gap.saturating_mul(node.children.len().saturating_sub(1) as i32)),
        ViewNodeKind::Spacer => 0,
        _ => node
            .children
            .iter()
            .map(|child| {
                intrinsic_min_height_px(child, content_width, dpi, typography_scale)
            })
            .max()
            .unwrap_or(0),
    };
    declared.max(content.saturating_add(padding.saturating_mul(2)))
}

#[cfg(feature = "icon")]
fn standalone_icon_size_px<Msg>(
    node: &ViewNode<Msg>,
    size: crate::ZsIconSize,
    dpi: Dpi,
) -> i32 {
    crate::platform_component_profile::PlatformIconProfile::for_platform(
        node.resolved_platform_style(),
    )
    .size(size)
    .to_px(dpi)
    .round_i32()
    .max(1)
}

#[cfg(feature = "badge")]
fn standalone_badge_size_px<Msg>(
    node: &ViewNode<Msg>,
    content: crate::ZsBadgeContent,
    dpi: Dpi,
    typography_scale: f32,
) -> (i32, i32) {
    let (width, height) =
        crate::platform_component_profile::PlatformBadgeProfile::for_platform(
            node.resolved_platform_style(),
        )
        .size(content, typography_scale);
    (
        width.to_px(dpi).round_i32().max(1),
        height.to_px(dpi).round_i32().max(1),
    )
}

#[cfg(feature = "label")]
fn estimated_text_min_width_px(
    text: &str,
    style: SemanticTextStyle,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    let units = if style.wrap == crate::TextWrap::Word {
        text.lines()
            .map(estimated_wrapping_segment_units)
            .max()
            .unwrap_or(0)
    } else {
        crate::widget_render::zs_estimated_text_width_units(text)
    };
    estimated_text_units_px(units, style, dpi, typography_scale)
}

#[cfg(feature = "label")]
fn estimated_text_min_height_px(
    text: &str,
    style: SemanticTextStyle,
    available_width: i32,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    let line_height = Dp::new(
        style
            .role
            .metrics_for(crate::ZsTypographyPlatformStyle::current())
            .line_height
            * typography_scale.max(0.0),
    )
    .to_px(dpi)
    .round_i32()
    .max(1);
    let rows = text
        .lines()
        .map(|line| {
            if style.wrap != crate::TextWrap::Word || available_width <= 0 {
                return 1;
            }
            let units = crate::widget_render::zs_estimated_text_width_units(line);
            let width = estimated_text_units_px(units, style, dpi, typography_scale);
            width
                .saturating_add(available_width.saturating_sub(1))
                .checked_div(available_width)
                .unwrap_or(1)
                .max(1)
        })
        .fold(0i32, i32::saturating_add)
        .max(1);
    line_height.saturating_mul(rows)
}

#[cfg(feature = "label")]
fn estimated_text_units_px(
    units: i32,
    style: SemanticTextStyle,
    dpi: Dpi,
    typography_scale: f32,
) -> i32 {
    if units <= 0 {
        return 0;
    }
    let platform = crate::ZsBaseControlPlatformStyle::current();
    let base = crate::ZsBaseControlMetrics::for_platform(platform);
    let body_size = crate::TextRole::Body.metrics_for(platform).size.max(1.0);
    let role_size = style.role.metrics_for(platform).size.max(1.0);
    let weight_reserve = match style.weight {
        crate::TextWeight::Semibold | crate::TextWeight::Bold => 1.08,
        crate::TextWeight::Medium => 1.04,
        crate::TextWeight::Automatic | crate::TextWeight::Regular => 1.0,
    };
    let width = (units.saturating_add(2) as f32)
        * base.average_character_width.0
        * (role_size / body_size)
        * typography_scale.max(0.0)
        * weight_reserve;
    Dp::new(width).to_px(dpi).ceil_i32().max(1)
}

#[cfg(feature = "label")]
fn estimated_wrapping_segment_units(line: &str) -> i32 {
    let mut longest = 0i32;
    let mut current_ascii_word = 0i32;
    for character in line.chars() {
        let width = crate::widget_render::zs_character_width_units(character);
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '/' | '.') {
            current_ascii_word = current_ascii_word.saturating_add(width);
        } else {
            longest = longest.max(current_ascii_word).max(width);
            current_ascii_word = 0;
        }
    }
    longest.max(current_ascii_word)
}

fn typography_aware_length_px(
    value: Dp,
    dpi: Dpi,
    typography_scaled: bool,
    typography_scale: f32,
) -> i32 {
    let scale = if typography_scaled {
        typography_scale.max(0.0)
    } else {
        1.0
    };
    Dp::new(value.0 * scale).to_px(dpi).round_i32().max(0)
}

#[cfg(feature = "date-picker")]
fn clamp_visible_month(month: ZsDate, minimum: ZsDate, maximum: ZsDate) -> ZsDate {
    let (minimum, maximum) = if minimum <= maximum {
        (minimum, maximum)
    } else {
        (maximum, minimum)
    };
    month
        .first_day_of_month()
        .max(minimum.first_day_of_month())
        .min(maximum.first_day_of_month())
}

fn inset_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    let padding = padding
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    Rect {
        x: bounds.x + padding,
        y: bounds.y + padding,
        width: (bounds.width - padding * 2).max(0),
        height: (bounds.height - padding * 2).max(0),
    }
}

#[cfg(feature = "list")]
fn horizontal_inset_bounds(bounds: Rect, inset: Option<Dp>, dpi: Dpi) -> Rect {
    let inset = inset
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    Rect {
        x: bounds.x.saturating_add(inset),
        y: bounds.y,
        width: bounds.width.saturating_sub(inset.saturating_mul(2)).max(0),
        height: bounds.height,
    }
}

#[cfg(feature = "tabs")]
fn tab_layout_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    inset_bounds(bounds, padding, dpi)
}

#[cfg(any(feature = "label", feature = "button", feature = "textbox"))]
fn padded_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    inset_bounds(bounds, padding, dpi)
}

#[cfg(feature = "button")]
fn button_content_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    if padding.is_some() {
        return padded_bounds(bounds, padding, dpi);
    }
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    side_inset_bounds(
        bounds,
        metrics.button_padding_left,
        metrics.button_padding_top,
        metrics.button_padding_right,
        metrics.button_padding_bottom,
        dpi,
    )
}

#[cfg(feature = "textbox")]
fn text_input_content_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    if padding.is_some() {
        return padded_bounds(bounds, padding, dpi);
    }
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    side_inset_bounds(
        bounds,
        metrics.text_input_padding_left,
        metrics.text_input_padding_top,
        metrics.text_input_padding_right,
        metrics.text_input_padding_bottom,
        dpi,
    )
}

#[cfg(any(feature = "button", feature = "textbox"))]
fn side_inset_bounds(
    bounds: Rect,
    left: Dp,
    top: Dp,
    right: Dp,
    bottom: Dp,
    dpi: Dpi,
) -> Rect {
    let left = left.to_px(dpi).round_i32().max(0);
    let top = top.to_px(dpi).round_i32().max(0);
    let right = right.to_px(dpi).round_i32().max(0);
    let bottom = bottom.to_px(dpi).round_i32().max(0);
    Rect {
        x: bounds.x.saturating_add(left),
        y: bounds.y.saturating_add(top),
        width: bounds.width.saturating_sub(left.saturating_add(right)).max(0),
        height: bounds
            .height
            .saturating_sub(top.saturating_add(bottom))
            .max(0),
    }
}

fn radius_px(radius: Option<Dp>, dpi: Dpi) -> i32 {
    radius
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0)
}

#[cfg(feature = "scroll")]
fn scroll_max_offset_y(bounds: Option<Rect>, content_height: Option<Dp>, dpi: Dpi) -> Dp {
    let viewport_px = bounds
        .map(|bounds| bounds.height.max(0) as f32)
        .unwrap_or(0.0);
    let content_px = content_height
        .map(|height| height.to_px(dpi).0.max(0.0))
        .unwrap_or(viewport_px);
    let scale = (dpi.0 / Dpi::standard().0).max(f32::EPSILON);
    Dp::new(((content_px - viewport_px) / scale).max(0.0))
}

fn color_role_for_token(token: ThemeColorToken) -> ColorRole {
    match token {
        ThemeColorToken::Surface => ColorRole::Surface,
        ThemeColorToken::SurfaceRaised => ColorRole::SurfaceRaised,
        ThemeColorToken::TextPrimary => ColorRole::PrimaryText,
        ThemeColorToken::TextSecondary => ColorRole::SecondaryText,
        ThemeColorToken::Accent => ColorRole::Accent,
        ThemeColorToken::AccentText => ColorRole::AccentText,
        ThemeColorToken::Control => ColorRole::Control,
        ThemeColorToken::Border => ColorRole::Border,
        ThemeColorToken::Success => ColorRole::Success,
        ThemeColorToken::Warning => ColorRole::Warning,
        ThemeColorToken::Danger => ColorRole::Danger,
    }
}

fn clipped_rect(rect: Rect, clip: Option<Rect>) -> Option<Rect> {
    let Some(clip) = clip else {
        return Some(rect);
    };
    let left = rect.x.max(clip.x);
    let top = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    let width = right - left;
    let height = bottom - top;
    (width > 0 && height > 0).then_some(Rect {
        x: left,
        y: top,
        width,
        height,
    })
}
