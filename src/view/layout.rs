#[cfg(feature = "virtual-list")]
pub fn virtual_list_viewport(
    total_count: usize,
    row_height: Dp,
    offset_y: Dp,
    viewport_height: Dp,
    overscan_rows: usize,
    direction: VirtualListScrollDirection,
) -> VirtualListViewport {
    let row_height = if row_height.0.is_finite() {
        row_height.0.max(1.0)
    } else {
        1.0
    };
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
    let content_height = total_count as f64 * row_height as f64;
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

    let start = ((offset_y / row_height).floor() as usize).min(total_count);
    let end = (((offset_y + viewport_height) / row_height).ceil() as usize)
        .max(start.saturating_add(1))
        .min(total_count);
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
fn virtual_list_row_bounds(
    bounds: Rect,
    index: usize,
    row_height: Dp,
    offset_y: Dp,
    dpi: Dpi,
) -> Rect {
    let row_height_px = row_height.to_px(dpi).round_i32().max(1);
    let offset_px = offset_y.to_px(dpi).round_i32().max(0);
    let row_top = (index as i64)
        .saturating_mul(row_height_px as i64)
        .saturating_sub(offset_px as i64)
        .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Rect {
        x: bounds.x,
        y: bounds.y.saturating_add(row_top),
        width: bounds.width,
        height: row_height_px,
    }
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
            );
            let mut x = bounds.x;
            widths
                .into_iter()
                .zip(children)
                .map(|(width, child)| {
                    let height = cross_axis_length(
                        bounds.height,
                        child.style.height,
                        child.style.min_height,
                        child.style.flex,
                        dpi,
                        child.typography_scaled_height,
                        typography_scale,
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
            children.len(),
            column_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            row_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            dpi,
        ),
        _ => vec![bounds; child_count],
    }
}

#[cfg(feature = "grid")]
fn split_grid_child_bounds(
    bounds: Rect,
    columns: &[ZsGridTrack],
    rows: &[ZsGridTrack],
    placements: &[ZsGridPlacement],
    child_count: usize,
    column_gap: i32,
    row_gap: i32,
    dpi: Dpi,
) -> Vec<Rect> {
    let column_lengths = allocate_grid_track_lengths(bounds.width, column_gap, columns, dpi);
    let row_lengths = allocate_grid_track_lengths(bounds.height, row_gap, rows, dpi);
    (0..child_count)
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
                .unwrap_or(Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: 0,
                    height: 0,
                })
        })
        .collect()
}

#[cfg(feature = "grid")]
fn allocate_grid_track_lengths(total: i32, gap: i32, tracks: &[ZsGridTrack], dpi: Dpi) -> Vec<i32> {
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
    let fixed_total = requested
        .iter()
        .flatten()
        .fold(0i32, |total, value| total.saturating_add(*value));
    let mut lengths = vec![0; tracks.len()];

    if fixed_total >= available && fixed_total > 0 {
        let fixed_indices = requested
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.map(|value| (index, value)))
            .collect::<Vec<_>>();
        let mut assigned = 0;
        for (position, (index, value)) in fixed_indices.iter().enumerate() {
            let length = if position + 1 == fixed_indices.len() {
                available.saturating_sub(assigned)
            } else {
                ((i64::from(*value) * i64::from(available)) / i64::from(fixed_total))
                    .clamp(0, i64::from(i32::MAX)) as i32
            };
            lengths[*index] = length;
            assigned = assigned.saturating_add(length);
        }
        return lengths;
    }

    for (index, value) in requested.iter().enumerate() {
        if let Some(value) = value {
            lengths[index] = *value;
        }
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
    let remaining = available.saturating_sub(fixed_total).max(0);
    let total_weight = fractional_indices.iter().fold(0u64, |total, (_, weight)| {
        total.saturating_add(u64::from(*weight))
    });
    let mut assigned = 0;
    for (position, (index, weight)) in fractional_indices.iter().enumerate() {
        let length = if position + 1 == fractional_indices.len() {
            remaining.saturating_sub(assigned)
        } else {
            ((remaining as u128 * u128::from(*weight)) / u128::from(total_weight))
                .min(i32::MAX as u128) as i32
        };
        lengths[*index] = length;
        assigned = assigned.saturating_add(length);
    }
    lengths
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
    );
    let mut y = bounds.y;
    heights
        .into_iter()
        .zip(children)
        .map(|(height, child)| {
            let width = cross_axis_length(
                bounds.width,
                child.style.width,
                child.style.min_width,
                child.style.flex,
                dpi,
                false,
                typography_scale,
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
            minimum(&child.style)
                .map(|value| {
                    typography_aware_length_px(
                        value,
                        dpi,
                        vertical && child.typography_scaled_height,
                        typography_scale,
                    )
                })
                .unwrap_or(0)
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

fn cross_axis_length(
    available: i32,
    fixed: Option<Dp>,
    minimum: Option<Dp>,
    flex: f32,
    dpi: Dpi,
    typography_scaled: bool,
    typography_scale: f32,
) -> i32 {
    let available = available.max(0);
    let minimum = minimum
        .map(|value| typography_aware_length_px(value, dpi, typography_scaled, typography_scale))
        .unwrap_or(0)
        .min(available);
    fixed
        .map(|value| {
            typography_aware_length_px(value, dpi, typography_scaled, typography_scale)
                .max(minimum)
        })
        .or_else(|| (flex <= f32::EPSILON && minimum > 0).then_some(minimum))
        .unwrap_or(available)
        .min(available)
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
