
pub fn row<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    let children = children.into_iter().collect::<Vec<_>>();
    let intrinsic_height = children
        .iter()
        .filter_map(|child| child.style.height.or(child.style.min_height))
        .map(|height| height.0)
        .fold(0.0_f32, f32::max);
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Row,
    })
    .children(children)
    .min_height(Dp::new(intrinsic_height))
    .flex(0.0)
}

pub fn column<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Column,
    })
    .children(children)
}

#[cfg(feature = "grid")]
/// Creates a two-dimensional Grid using shared DPI-aware layout geometry.
///
/// Every [`ZsGridCell`] carries an explicit zero-based placement, matching the
/// row/column attachment model shared by WinUI Grid, AppKit Grid View and
/// GTK4 Grid. Explicit overlaps retain declaration order for painting and hit
/// testing.
pub fn grid<Msg>(
    columns: impl IntoIterator<Item = ZsGridTrack>,
    rows: impl IntoIterator<Item = ZsGridTrack>,
    items: impl IntoIterator<Item = ZsGridCell<Msg>>,
) -> ViewNode<Msg> {
    let items = items.into_iter().collect::<Vec<_>>();
    let placements = items.iter().map(|item| item.placement).collect();
    ViewNode::<Msg>::new(ViewNodeKind::Grid {
        columns: columns.into_iter().collect(),
        rows: rows.into_iter().collect(),
        placements,
        column_gap: None,
        row_gap: None,
    })
    .children(items.into_iter().map(|item| item.content))
}

#[cfg(feature = "virtual-list")]
pub fn virtual_list<T, Msg>(
    total_count: usize,
    rows: impl IntoIterator<Item = (usize, T)>,
    mut render: impl FnMut(usize, T) -> ViewNode<Msg>,
) -> ViewNode<Msg> {
    let mut rows = rows
        .into_iter()
        .filter(|(index, _)| *index < total_count)
        .collect::<Vec<_>>();
    rows.sort_by_key(|(index, _)| *index);
    rows.dedup_by_key(|(index, _)| *index);
    let mut row_indices = Vec::with_capacity(rows.len());
    let mut children = Vec::with_capacity(rows.len());
    for (index, item) in rows {
        row_indices.push(index);
        children.push(render(index, item));
    }
    ViewNode::<Msg>::new(ViewNodeKind::VirtualList {
        total_count,
        row_height: Dp::new(40.0),
        overscan_rows: 4,
        row_indices,
        selected_index: None,
        offset_y: Dp::new(0.0),
        visible_range: VirtualListRange::new(0, 0),
        materialized_range: VirtualListRange::new(0, 0),
        on_select: None,
        on_viewport_changed: None,
        loading: false,
        show_placeholders: true,
    })
    .children(children)
}

#[cfg(feature = "scroll")]
pub fn scroll<Msg>(child: ViewNode<Msg>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Scroll {
        offset_y: Dp::new(0.0),
        content_height: None,
        on_scroll: None,
    })
    .child(child)
}

pub fn spacer<Msg>() -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Spacer)
}
