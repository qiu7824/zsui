#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use zsui::{
    button, column, native_window, row, text, textbox, AppCx, Command, Dp,
    NativeProofProcessMemoryEvidence, NativeWindowSmokeRunOptions, UiInvalidationHandle, ViewNode,
    ZsWindowCommand,
};

#[derive(Default)]
struct State {
    count: u32,
    commands: Vec<&'static str>,
}
#[derive(Clone)]
enum Msg {
    Increment,
    Hide,
    Show,
}
#[derive(Serialize)]
struct Sample {
    phase: &'static str,
    memory: NativeProofProcessMemoryEvidence,
}

fn view(state: &State, large_text: bool) -> ViewNode<Msg> {
    let mut children = vec![
        text(format!(
            "Native memory lifecycle / 原生内存：{}",
            state.count
        )),
        row([
            button("Increment").on_click(Msg::Increment),
            button("Hide").on_click(Msg::Hide),
        ])
        .gap(Dp::new(12.0)),
    ];
    if large_text {
        children.push(
            text("Rust 中文 العربية שלום — bounded native text rendering\n".repeat(16))
                .height(Dp::new(600.0))
                .width(Dp::new(1000.0)),
        );
    } else {
        let fields = |offset| {
            column((0..8).map(|index| {
                row([
                    text(format!("Field {} / 字段", offset + index)).width(Dp::new(180.0)),
                    textbox::<Msg>("Sample / 示例文本").width(Dp::new(260.0)),
                ])
                .gap(Dp::new(8.0))
            }))
            .gap(Dp::new(12.0))
        };
        children.push(row([fields(1), fields(9)]).gap(Dp::new(24.0)));
    }
    column(children).gap(Dp::new(16.0)).padding(Dp::new(20.0))
}

fn sample(phase: &'static str, samples: &mut Vec<Sample>) {
    for _ in 0..7 {
        samples.push(Sample {
            phase,
            memory: zsui::native_smoke::capture_native_process_memory(phase)
                .expect("the selected native host must expose process memory"),
        });
        thread::sleep(Duration::from_millis(100));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    let output = args
        .iter()
        .position(|arg| arg == "--output")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .ok_or("--output is required")?;
    if !output.is_absolute() {
        return Err("--output must be absolute".into());
    }
    let large_text = args.iter().any(|arg| arg == "--large-text");
    fs::create_dir_all(&output)?;
    let before_window = zsui::native_smoke::capture_native_process_memory("before_window");
    let state = Arc::new(Mutex::new(State::default()));
    let view_state = state.clone();
    let update_state = state.clone();
    let wake = UiInvalidationHandle::new();
    let worker_wake = wake.clone();
    let worker = thread::spawn(move || {
        let mut samples = Vec::with_capacity(21);
        thread::sleep(Duration::from_secs(3));
        worker_wake
            .request_app_command(Command::custom("increment"))
            .unwrap();
        thread::sleep(Duration::from_millis(400));
        sample("visible", &mut samples);
        worker_wake
            .request_app_command(Command::custom("hide"))
            .unwrap();
        thread::sleep(Duration::from_secs(2));
        sample("hidden", &mut samples);
        worker_wake
            .request_app_command(Command::custom("show"))
            .unwrap();
        thread::sleep(Duration::from_secs(2));
        sample("restored", &mut samples);
        samples
    });
    let report = native_window("ZSUI memory lifecycle")
        .size(1080, 760)
        .release_view_when_hidden()
        .invalidation_handle(wake)
        .stateful_view_with_app_commands(
            (),
            move |_| view(&view_state.lock().unwrap(), large_text),
            move |_, message, cx: &mut AppCx| {
                let mut state = update_state.lock().unwrap();
                match message {
                    Msg::Increment => {
                        state.count += 1;
                        state.commands.push("increment");
                    }
                    Msg::Hide => {
                        state.commands.push("hide");
                        cx.window_command(ZsWindowCommand::Hide);
                    }
                    Msg::Show => {
                        state.commands.push("show");
                        cx.window_command(ZsWindowCommand::Show);
                    }
                }
            },
            |command| match command {
                Command::Custom { id, .. } => match id.as_str() {
                    "increment" => Some(Msg::Increment),
                    "hide" => Some(Msg::Hide),
                    "show" => Some(Msg::Show),
                    _ => None,
                },
                _ => None,
            },
        )
        .run_smoke(
            NativeWindowSmokeRunOptions::new(12500)
                .screenshot_file(output.join("window.png").to_string_lossy())
                .require_screenshot(true),
        )?;
    let samples = worker.join().map_err(|_| "memory sampler failed")?;
    let after_close = zsui::native_smoke::capture_native_process_memory("after_close");
    let state = state.lock().unwrap();
    assert_eq!(state.count, 1);
    assert_eq!(state.commands, ["increment", "hide", "show"]);
    assert!(report.visible_window_was_created() && report.screenshot_captured);
    fs::write(
        output.join("memory.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "zsui.memory-lifecycle/v1", "scene": if large_text {"large-text"} else {"panel"},
            "width": 1080, "height": 760, "before_window": before_window, "samples": samples,
            "after_close": after_close, "state_count": state.count, "commands": state.commands, "runtime": report,
        }))?,
    )?;
    Ok(())
}
