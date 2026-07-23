#[cfg(feature = "split-view")]
/// Composes one pane and one content subtree from the same Rust declaration on
/// Win32, AppKit and Linux.
///
/// The pane is the first child and content is the second. Applications own the
/// open state; light dismiss and Escape emit `false` through
/// [`ViewNode::on_split_view_open_change_with`].
pub fn split_view<Msg>(
    widget: WidgetId,
    spec: crate::ZsSplitViewSpec,
    pane: ViewNode<Msg>,
    content: ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::SplitView {
        spec,
        on_open_change: None,
    })
    .id(widget)
    .child(pane)
    .child(content)
}
