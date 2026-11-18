#[cfg(all(
    feature = "ui-document-runtime",
    feature = "button",
    feature = "textbox"
))]
pub fn name_form() -> zsui::ViewNode<&'static str> {
    use zsui::Dp;
    zsui::column([
        zsui::textbox("Ada"),
        zsui::button("Save").id(zsui::WidgetId(2)).on_click("save"),
    ])
    .max_width(Dp::new(640.0))
    .max_height(Dp::new(240.0))
}

#[cfg(feature = "canvas")]
pub fn custom_panel() -> zsui::ViewNode<()> {
    use zsui::Dp;
    use zsui::{
        canvas_with, ColorRole, NativeDrawFill, ZsCanvasPrimitive, ZsCanvasRect, ZsCanvasScene,
    };
    canvas_with::<()>(|cx| {
        let rect = ZsCanvasRect::new(
            Dp::new(0.0),
            Dp::new(0.0),
            Dp::new(cx.size().width.0 / 2.0),
            Dp::new(8.0),
        );
        ZsCanvasScene::new().with(ZsCanvasPrimitive::fill_rect(
            rect,
            NativeDrawFill::role(ColorRole::Accent),
        ))
    })
    .max_width(Dp::new(320.0))
}

#[cfg(all(feature = "virtual-list", feature = "label"))]
pub fn records() -> zsui::ViewNode<usize> {
    use zsui::{Dp, WidgetId};
    zsui::items_repeater::<_, usize>(
        100_000,
        (10..14).map(|i| (i, format!("Row {i}"))),
        |i, value| zsui::text(value).id(WidgetId(1000 + i as u64)),
    )
    .item_height(Dp::new(20.0))
    .scroll_y(Dp::new(200.0))
    .on_items_repeater_select_with(|i| i)
}
