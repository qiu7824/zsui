#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use zsui::prelude::{button, column, text, window, Dp, Element, UpdateContext};

#[derive(Clone)]
enum Message {
    Increment,
}

struct State {
    count: u32,
}

fn view(state: &State) -> Element<Message> {
    column([
        text(format!("Count: {}", state.count)),
        button("Increment").on_click(Message::Increment),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(20.0))
}

fn update(state: &mut State, message: Message, _context: &mut UpdateContext<'_>) {
    match message {
        Message::Increment => state.count += 1,
    }
}

fn main() -> Result<(), zsui::stable::Error> {
    window("ZSUI Stable API")
        .size(480, 320)
        .stateful(State { count: 0 }, view, update)
        .run()
}
