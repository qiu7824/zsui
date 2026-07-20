/// Wraps a page with a target-anchored menu surface.
///
/// Applications declare one [`MenuSpec`](crate::MenuSpec) and receive its
/// typed [`Command`](crate::Command) values through `on_menu_flyout_command`.
/// ZSUI owns placement, keyboard navigation, light dismissal and the distinct
/// Windows, macOS and Linux menu metrics.
pub fn menu_flyout<Msg>(
    widget: WidgetId,
    open: bool,
    target: WidgetId,
    menu: crate::MenuSpec,
    page: ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::MenuFlyout {
        menu,
        open,
        target,
        highlighted: None,
        open_submenu: None,
        on_command: None,
        on_open_change: None,
    })
    .id(widget)
    .child(page)
}
