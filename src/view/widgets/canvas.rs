/// Creates a retained, backend-neutral custom drawing surface.
///
/// The scene uses local [`Dp`](crate::Dp) coordinates and semantic colors.
/// Set an explicit size through the ordinary View style methods. Assign a
/// [`WidgetId`] and `on_click(...)` when the surface is interactive.
pub fn canvas<Msg>(scene: crate::ZsCanvasScene) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Canvas {
        scene,
        on_click: None,
    })
}
