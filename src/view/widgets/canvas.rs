/// Creates a retained, backend-neutral custom drawing surface.
///
/// The scene uses local [`Dp`](crate::Dp) coordinates and semantic colors.
/// Set an explicit size through the ordinary View style methods. Assign a
/// [`WidgetId`] and `on_canvas_pointer(...)` when the surface is interactive.
/// `on_click(...)` remains available for primary-button activation.
pub fn canvas<Msg>(scene: crate::ZsCanvasScene) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Canvas {
        scene,
        builder: None,
        on_click: None,
        on_pointer: None,
    })
}

/// Creates a size-aware Canvas whose scene is rebuilt from its final layout.
///
/// `build` receives the Canvas' local size and a text measurer backed by the
/// same native measurements as labels. It runs during layout; without an
/// explicit height the node uses [`crate::ZsCanvasScene::with_extent_height`]
/// from a build at the available width as its natural height, which lets a
/// Scroll host variable-length custom content.
pub fn canvas_with<Msg>(
    build: impl Fn(&crate::ZsCanvasLayoutContext<'_>) -> crate::ZsCanvasScene + Send + Sync + 'static,
) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Canvas {
        scene: crate::ZsCanvasScene::new(),
        builder: Some(crate::ZsCanvasBuilder::new(build)),
        on_click: None,
        on_pointer: None,
    })
}
