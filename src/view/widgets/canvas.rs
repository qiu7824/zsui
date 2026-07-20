/// Creates a retained, backend-neutral custom drawing surface.
///
/// The scene uses local [`Dp`](crate::Dp) coordinates and semantic colors.
/// Set an explicit size through the ordinary View style methods. Assign a
/// [`WidgetId`] and `on_canvas_pointer(...)` when the surface is interactive.
/// `on_click(...)` remains available for primary-button activation.
pub fn canvas<Msg>(scene: crate::ZsCanvasScene) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Canvas {
        scene,
        on_click: None,
        on_pointer: None,
    })
}
