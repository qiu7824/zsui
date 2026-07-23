#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    button, column, native_window, row, spacer, styled_text, Dp, NativeWindowSmokeRunOptions,
    SemanticTextStyle, TextRole, ViewNode,
};

#[derive(Debug, Clone, Copy)]
enum Msg {
    ChooseInvoice,
}

fn view() -> ViewNode<Msg> {
    let content = column([
        styled_text(
            "发票助手 / Invoice Assistant",
            SemanticTextStyle::for_role(TextRole::Title),
        ),
        styled_text("Window + Text + Button", SemanticTextStyle::body()),
        button("选择发票 / Choose invoice").on_click(Msg::ChooseInvoice),
    ])
    .gap(Dp::new(16.0))
    .width(Dp::new(360.0))
    .flex(0.0);
    column([
        spacer().flex(1.0),
        row([spacer().flex(1.0), content, spacer().flex(1.0)])
            .height(Dp::new(180.0))
            .flex(0.0),
        spacer().flex(1.0),
    ])
    .padding(Dp::new(32.0))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let empty = arguments
        .iter()
        .any(|argument| argument == "--benchmark-empty");
    let duration_ms = arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1_000).max(250));

    let builder = if empty {
        native_window("UI 性能矩阵 · Minimal · ZSUI")
            .size(1000, 700)
            .release_view_when_hidden()
    } else {
        native_window("UI 性能矩阵 · Minimal · ZSUI")
            .size(1000, 700)
            .release_view_when_hidden()
            .view(view())
    };

    if let Some(duration_ms) = duration_ms {
        let output = env::temp_dir().join("zsui-ui-performance-minimal");
        fs::create_dir_all(&output)?;
        builder.run_smoke(NativeWindowSmokeRunOptions::new(duration_ms))?;
    } else {
        builder.run()?;
    }
    Ok(())
}
