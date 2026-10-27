#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[path = "support/memory_allocation_probe.rs"]
mod allocation_probe;
#[global_allocator]
static ALLOCATION_PROBE: allocation_probe::AllocationProbe = allocation_probe::AllocationProbe;

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
    NativeProofProcessMemoryEvidence, NativeWindowHost, NativeWindowSmokeRunOptions,
    UiInvalidationHandle, ViewNode, ZsWindowCommand, ZsuiHost,
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
    timestamp_millis: u128,
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

fn sample(phase: &'static str, samples: &mut Vec<Sample>, count: usize) {
    for _ in 0..count {
        samples.push(Sample {
            phase,
            timestamp_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            memory: zsui::native_smoke::capture_native_process_memory(phase)
                .expect("the selected native host must expose process memory"),
        });
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(windows)]
mod windows_probe_backend {
    pub(super) fn pump_teardown_messages() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
        };
        // This proof re-enters the host after closing every window. Keep the UI
        // thread servicing native teardown timers during its controlled idle phase.
        unsafe {
            let mut message: MSG = std::mem::zeroed();
            for _ in 0..128 {
                if PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                    break;
                }
                if message.message != WM_QUIT {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
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
    if let Some(index) = args.iter().position(|arg| arg == "--external-windows") {
        let count: usize = args
            .get(index + 1)
            .ok_or("--external-windows needs a count")?
            .parse()?;
        if !(1..=3).contains(&count) {
            return Err("external window count must be in 1..=3".into());
        }
        let rounds = args
            .windows(2)
            .find(|pair| pair[0] == "--external-rounds")
            .map(|pair| pair[1].parse::<usize>())
            .transpose()?
            .unwrap_or(1);
        if !(1..=200).contains(&rounds) {
            return Err("external rounds must be in 1..=200".into());
        }
        let before = zsui::native_smoke::capture_native_process_memory("before_windows");
        for round in 1..=rounds {
            let mut host = NativeWindowHost::new();
            for index in 0..count {
                let builder = native_window(format!(
                    "ZSUI memory lifecycle {} / round {round}",
                    index + 1
                ))
                .size(1080, 760)
                .view(view(&State::default(), large_text));
                host.set_next_window_draw_plan(builder.native_draw_plan().cloned());
                host.create_main_window(builder.window_spec())?;
            }
            host.run_event_loop()?;
            drop(host);
            if rounds > 1 {
                fs::write(
                    output.join("completed-round.json"),
                    serde_json::to_vec(&serde_json::json!({
                        "round": round, "after": zsui::native_smoke::capture_native_process_memory("after_round"),
                        "rust_heap": allocation_probe::snapshot(),
                    }))?,
                )?;
                let deadline = std::time::Instant::now() + Duration::from_secs(20);
                let expected = round.to_string();
                while fs::read_to_string(output.join("continue-round.txt"))
                    .ok()
                    .as_deref()
                    != Some(expected.as_str())
                {
                    #[cfg(windows)]
                    windows_probe_backend::pump_teardown_messages();
                    if std::time::Instant::now() >= deadline {
                        return Err("external round controller did not acknowledge teardown".into());
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        fs::write(
            output.join("external-exit.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": "zsui.external-memory-lifecycle/v1", "windows": count, "rounds": rounds,
                "before": before,
                "after": zsui::native_smoke::capture_native_process_memory("after_windows"),
                "rust_heap": allocation_probe::snapshot(),
            }))?,
        )?;
        return Ok(());
    }
    let sample_count = args
        .windows(2)
        .find(|pair| pair[0] == "--sample-count")
        .map(|pair| pair[1].parse::<usize>())
        .transpose()?
        .unwrap_or(7);
    if !(1..=100).contains(&sample_count) {
        return Err("sample count must be in 1..=100".into());
    }
    let before_window = zsui::native_smoke::capture_native_process_memory("before_window");
    let state = Arc::new(Mutex::new(State::default()));
    let view_state = state.clone();
    let update_state = state.clone();
    let wake = UiInvalidationHandle::new();
    let worker_wake = wake.clone();
    let worker = thread::spawn(move || {
        let mut samples = Vec::with_capacity(sample_count * 3);
        thread::sleep(Duration::from_secs(3));
        worker_wake
            .request_app_command(Command::custom("increment"))
            .unwrap();
        thread::sleep(Duration::from_millis(400));
        sample("visible", &mut samples, sample_count);
        worker_wake
            .request_app_command(Command::custom("hide"))
            .unwrap();
        thread::sleep(Duration::from_secs(2));
        sample("hidden", &mut samples, sample_count);
        worker_wake
            .request_app_command(Command::custom("show"))
            .unwrap();
        thread::sleep(Duration::from_secs(2));
        sample("restored", &mut samples, sample_count);
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
            NativeWindowSmokeRunOptions::new(10400 + sample_count as u64 * 300)
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
