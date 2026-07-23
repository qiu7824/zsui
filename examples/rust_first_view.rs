#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    button, column, native_window, row, styled_text, textbox, toggle, AppCx, Command, CommandId,
    Dp, NativeWindowRuntimeDriver, NativeWindowSmokeRunOptions, Point, SemanticTextStyle,
    UiCommand, ViewNode, WidgetId,
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

struct AppState {
    name: String,
    dark_mode: bool,
    save_count: u32,
}

fn view(state: &AppState) -> ViewNode<Msg> {
    column([
        styled_text(format!("Hello, {}", state.name), SemanticTextStyle::body()),
        textbox(&state.name).id(NAME).on_change(Msg::NameChanged),
        row([
            styled_text("Dark mode", SemanticTextStyle::body()),
            toggle(state.dark_mode)
                .id(DARK_MODE)
                .on_toggle(Msg::DarkModeChanged),
        ]),
        button("Save").id(SAVE).on_click(Msg::SaveClicked),
        styled_text(
            format!("Saved {} time(s)", state.save_count),
            SemanticTextStyle::body(),
        ),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(20.0))
}

fn update(state: &mut AppState, msg: Msg, cx: &mut AppCx) {
    match msg {
        Msg::SaveClicked => {
            state.save_count += 1;
            cx.command(Command::custom("settings.save"));
            cx.ui_command(UiCommand::app(CommandId("settings.persist")));
        }
        Msg::NameChanged(name) => state.name = name,
        Msg::DarkModeChanged(dark_mode) => state.dark_mode = dark_mode,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let builder = native_window("ZSUI Stateful View")
        .size(520, 360)
        .stateful_view(
            AppState {
                name: "ZSUI".to_string(),
                dark_mode: false,
                save_count: 0,
            },
            view,
            update,
        )
        .app_command_executor(NativeWindowRuntimeDriver::new())
        .ui_command_executor(NativeWindowRuntimeDriver::new());

    if args.iter().any(|arg| arg == "--smoke") {
        let artifact_dir = "target/zsui-stateful-view";
        fs::create_dir_all(artifact_dir)?;
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1200)
                .screenshot_file(format!("{artifact_dir}/window.png"))
                .require_screenshot(true)
                .native_view_click(Point { x: 260, y: 68 })
                .native_view_text_input(" Native")
                .native_view_click(Point { x: 476, y: 112 })
                .native_view_click(Point { x: 80, y: 156 }),
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--manifest") {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "draw_command_count": builder.native_draw_plan().map(|plan| plan.command_count()),
                "hit_target_count": builder.native_view_interaction_plan().map(|plan| plan.hit_target_count()),
                "live_runtime": builder.native_live_view_runtime().is_some(),
                "app_command_executor": builder.native_app_command_executor().is_some(),
                "ui_command_executor": builder.native_ui_command_executor().is_some(),
                "revision": builder.native_live_view_runtime().map(|runtime| runtime.revision()),
            }))?
        );
        return Ok(());
    }

    builder.run()?;
    Ok(())
}
