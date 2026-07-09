use zsui::{
    list, text, Dpi, Rect, View, ViewEvent, ViewEventCx, ViewLayoutCx, ViewPaintCx, WidgetId,
};

const FIRST: WidgetId = WidgetId::new(1);
const SECOND: WidgetId = WidgetId::new(2);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    RowSelected(usize),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut view = list([(FIRST, "First"), (SECOND, "Second")], |(id, label)| {
        text(label).id(id)
    })
    .selected_index(Some(0))
    .on_select(Msg::RowSelected);

    view.layout(&mut ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 96,
        },
        Dpi::standard(),
    ));

    let mut events = ViewEventCx::new();
    view.event(&mut events, &ViewEvent::Click { widget: SECOND });

    let mut paint = ViewPaintCx::new(Dpi::standard());
    view.paint(&mut paint);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "messages": events.into_messages().len(),
            "draw_command_count": paint.plan().command_count(),
            "text_command_count": paint.plan().text_count()
        }))?
    );
    Ok(())
}
