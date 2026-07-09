use zsui::{
    column, scroll, text, Dp, Dpi, NativeDrawCommand, Point, Rect, View, ViewEvent, ViewEventCx,
    ViewLayoutCx, ViewNode, ViewPaintCx, WidgetId,
};

const TOP_ROW: WidgetId = WidgetId::new(100);
const BOTTOM_ROW: WidgetId = WidgetId::new(101);
const SCROLL_VIEW: WidgetId = WidgetId::new(102);

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Scrolled(Dp),
}

fn main() {
    let mut view: ViewNode<Msg> = scroll(column([
        text("Scrolled out").id(TOP_ROW),
        text("Visible row").id(BOTTOM_ROW),
    ]))
    .id(SCROLL_VIEW)
    .content_height(Dp::new(120.0))
    .scroll_y(Dp::new(60.0))
    .on_scroll(Msg::Scrolled);
    let mut layout = ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 260,
            height: 60,
        },
        Dpi::standard(),
    );

    view.layout(&mut layout);
    let interaction = view.interaction_plan();
    let mut events = ViewEventCx::new();
    let mut paint = ViewPaintCx::new(Dpi::standard());

    view.event(
        &mut events,
        &ViewEvent::ScrollBy {
            widget: SCROLL_VIEW,
            delta_y: Dp::new(-20.0),
        },
    );
    view.paint(&mut paint);

    assert_eq!(events.into_messages(), vec![Msg::Scrolled(Dp::new(40.0))]);
    assert_eq!(
        interaction.target_at(Point { x: 24, y: 24 }),
        Some(BOTTOM_ROW)
    );
    assert!(interaction.hit_target_for_widget(TOP_ROW).is_none());
    assert!(paint
        .plan()
        .commands
        .iter()
        .any(|command| matches!(command, NativeDrawCommand::PushClip { .. })));
    assert!(paint
        .plan()
        .commands
        .iter()
        .any(|command| matches!(command, NativeDrawCommand::PopClip)));
}
