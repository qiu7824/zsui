#[cfg(feature = "combo")]
pub fn combo_box<T, Msg>(
    options: impl IntoIterator<Item = T>,
    selected_index: Option<usize>,
) -> ViewNode<Msg>
where
    T: Into<String>,
{
    let options = options.into_iter().map(Into::into).collect::<Vec<_>>();
    let selected_index = selected_index.filter(|index| *index < options.len());
    ViewNode::new(ViewNodeKind::ComboBox {
        options,
        selected_index,
        expanded: false,
        placeholder: None,
        on_select: None,
        on_expanded_change: None,
    })
}

#[cfg(feature = "tabs")]
pub fn tab_view<Msg>(
    items: impl IntoIterator<Item = ZsTabItem<Msg>>,
    selected: Option<ZsTabId>,
) -> ViewNode<Msg> {
    let items = items.into_iter().collect::<Vec<_>>();
    let tabs = items
        .iter()
        .map(|item| item.spec.clone())
        .collect::<Vec<_>>();
    let selected = selected
        .filter(|selected| tabs.iter().any(|tab| tab.id == *selected))
        .or_else(|| tabs.first().map(|tab| tab.id));
    ViewNode::new(ViewNodeKind::Tabs {
        tabs,
        selected,
        on_select: None,
    })
    .children(items.into_iter().map(|item| item.content))
}
#[cfg(feature = "list")]
pub fn list<T, Msg>(
    items: impl IntoIterator<Item = T>,
    render: impl FnMut(T) -> ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::List {
        selected_index: None,
        on_select: None,
    })
    .children(items.into_iter().map(render))
}

/// Creates a responsive self-drawn collection of selectable gallery tiles.
#[cfg(feature = "grid-view")]
pub fn grid_view<Msg>(items: impl IntoIterator<Item = crate::ZsGridViewItem>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::GridView {
        items: items.into_iter().collect(),
        selected: None,
        on_select: None,
        on_invoke: None,
    })
}

#[cfg(feature = "tree")]
pub fn tree_view<Msg>(roots: impl IntoIterator<Item = crate::ZsTreeNode>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::TreeView {
        roots: roots.into_iter().collect(),
        expanded: BTreeSet::new(),
        selected: None,
        on_select: None,
        on_expansion_change: None,
        on_invoke: None,
    })
}

#[cfg(feature = "table")]
pub fn data_grid<Msg>(
    columns: impl IntoIterator<Item = crate::ZsTableColumn>,
    rows: impl IntoIterator<Item = crate::ZsTableRow>,
) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::DataGrid {
        columns: columns.into_iter().collect(),
        rows: rows.into_iter().collect(),
        selected: None,
        sort: None,
        on_select: None,
        on_sort: None,
        on_invoke: None,
    })
}

