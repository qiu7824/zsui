use zsui::{
    button, checkbox, column, text, textbox, AppCx, Command, Dp, Dpi, Rect, View, ViewEvent,
    ViewEventCx, ViewLayoutCx, ViewNode, ViewPaintCx, WidgetId,
};

const SAVE: WidgetId = WidgetId::new(1);
const NAME: WidgetId = WidgetId::new(2);
const DARK_MODE: WidgetId = WidgetId::new(3);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    SaveClicked,
    NameChanged(String),
    DarkModeChanged(bool),
}

fn settings_view(name: &str, dark_mode: bool) -> ViewNode<Msg> {
    column(vec![
        text("ZSUI settings"),
        textbox(name).id(NAME).on_change(Msg::NameChanged),
        checkbox("Dark mode", dark_mode)
            .id(DARK_MODE)
            .on_toggle(Msg::DarkModeChanged),
        button("Save")
            .id(SAVE)
            .padding(Dp::new(12.0))
            .radius(Dp::new(8.0))
            .on_click(Msg::SaveClicked),
    ])
}

fn update(msg: Msg, cx: &mut AppCx) {
    match msg {
        Msg::SaveClicked => cx.command(Command::custom("settings.save")),
        Msg::NameChanged(_) | Msg::DarkModeChanged(_) => {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut view = settings_view("ZSUI", false);
    let mut layout = ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 360,
            height: 180,
        },
        Dpi::standard(),
    );
    view.layout(&mut layout);

    let mut events = ViewEventCx::new();
    view.event(
        &mut events,
        &ViewEvent::TextChanged {
            widget: NAME,
            value: "ZSUI Native".to_string(),
        },
    );
    view.event(
        &mut events,
        &ViewEvent::Toggled {
            widget: DARK_MODE,
            checked: true,
        },
    );
    view.event(&mut events, &ViewEvent::Click { widget: SAVE });

    let mut app_cx = AppCx::new();
    let messages = events.into_messages();
    for msg in messages.iter().cloned() {
        update(msg, &mut app_cx);
    }

    let mut paint = ViewPaintCx::new(Dpi::standard());
    view.paint(&mut paint);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "message_count": messages.len(),
            "command_count": app_cx.commands().len(),
            "draw_command_count": paint.plan().command_count(),
            "text_command_count": paint.plan().text_count()
        }))?
    );
    Ok(())
}
