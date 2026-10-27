#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[path = "support/device_demo.rs"]
mod model;

use model::{Message, State};
use zsui::{button, native_window, text, textbox, AppCx};
use zsui::{
    checkbox, column, progress_bar, row, slider, styled_text, Dp, ProgressRange, SemanticTextStyle,
    SliderRange, TextRole, ViewNode,
};

fn view(state: &State) -> ViewNode<Message> {
    let mut children: Vec<ViewNode<Message>> = vec![
        styled_text(model::TITLE, SemanticTextStyle::for_role(TextRole::Title)),
        text("6 台设备 / 6 devices     在线 4 / Online 4     平均延迟 / Average latency 20 ms"),
        row([
            textbox(state.query.clone())
                .placeholder("筛选设备 / Filter devices")
                .on_change(Message::Query)
                .flex(1.0),
            checkbox("仅在线 / Online only", state.online_only).on_toggle(Message::OnlineOnly),
            button("重置 / Reset")
                .on_click(Message::Reset)
                .width(Dp::new(120.0)),
        ])
        .gap(Dp::new(14.0))
        .height(Dp::new(36.0))
        .flex(0.0),
        text(
            "设备 / Device                                      类型 / Type             状态 / Status",
        ),
    ];
    for (index, device) in state.visible() {
        children.push(
            row([
                text(device.name).flex(1.0),
                text(device.kind).width(Dp::new(90.0)),
                text(model::status(device)).width(Dp::new(180.0)),
                button("选择 / Select")
                    .on_click(Message::Select(index))
                    .width(Dp::new(120.0)),
            ])
            .gap(Dp::new(12.0))
            .height(Dp::new(42.0))
            .flex(0.0),
        );
    }
    children.push(text(format!(
        "模拟负载 / Simulated load: {:.0}%",
        state.load
    )));
    children.push(
        slider(state.load, SliderRange::new(0.0, 100.0))
            .on_slide(Message::Load)
            .height(Dp::new(30.0))
            .flex(0.0),
    );
    children.push(
        progress_bar(state.load, ProgressRange::new(0.0, 100.0))
            .height(Dp::new(12.0))
            .flex(0.0),
    );
    children.push(text(state.selection()));
    column(children).gap(Dp::new(12.0)).padding(Dp::new(24.0))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder =
        native_window(format!("{} · ZSUI", model::TITLE)).size(model::WIDTH, model::HEIGHT);
    if model::has_flag("--benchmark-empty") {
        builder.run()?;
    } else {
        builder
            .stateful_view(State::default(), view, |state, message, _: &mut AppCx| {
                state.update(message);
            })
            .run()?;
    }
    Ok(())
}
