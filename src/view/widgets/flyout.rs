#[cfg(feature = "flyout")]
/// Wraps a page with a target-anchored, arbitrary-content flyout.
///
/// `page` remains the ordinary layout root. `content` is laid out only while
/// the flyout is open, inside the platform profile's popover surface.
pub fn flyout<Msg>(
    widget: WidgetId,
    open: bool,
    target: WidgetId,
    spec: crate::ZsFlyoutSpec,
    content: ViewNode<Msg>,
    page: ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Flyout {
        spec,
        open,
        target,
        on_dismiss: None,
        on_open_change: None,
    })
    .id(widget)
    .child(page)
    .child(content)
}
