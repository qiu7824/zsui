#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use zsui::prelude::{button, column, row, text, toggle, window, Dp, Element, UpdateContext};

#[derive(Clone)]
enum Message {
    RunningChanged(bool),
    Restart,
    Stop,
}

struct State {
    running: bool,
    restart_count: u32,
}

fn view(state: &State) -> Element<Message> {
    let status = if state.running {
        "运行中"
    } else {
        "已停止"
    };
    column([
        text("示例服务"),
        text(format!("状态：{status}")),
        row([
            text("服务开关"),
            toggle(state.running).on_toggle(Message::RunningChanged),
        ])
        .gap(Dp::new(12.0)),
        row([
            button("重启")
                .enabled(state.running)
                .on_click(Message::Restart),
            button("停止")
                .enabled(state.running)
                .on_click(Message::Stop),
        ])
        .gap(Dp::new(8.0)),
        text(format!("重启次数：{}", state.restart_count)),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(20.0))
}

fn update(state: &mut State, message: Message, _cx: &mut UpdateContext<'_>) {
    match message {
        Message::RunningChanged(running) => state.running = running,
        Message::Restart if state.running => {
            state.restart_count = state.restart_count.saturating_add(1);
        }
        Message::Restart => {}
        Message::Stop => state.running = false,
    }
}

fn main() -> Result<(), zsui::stable::Error> {
    window("服务控制")
        .size(420, 240)
        .min_size(360, 200)
        .stateful(
            State {
                running: true,
                restart_count: 0,
            },
            view,
            update,
        )
        .run()
}
