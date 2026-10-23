#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use zsui::{
    button, column, native_window, row, text, AppCx, Command, DirectoryDialogSpec, Dp,
    FileDialogService, FileDialogSpec, NativeFileDialogService, NativeWindowSmokeRunOptions,
    SaveFileDialogSpec, UiInvalidationHandle, ViewNode, WidgetId, ZsWindowCommand, ZsWindowState,
};

#[derive(Default)]
struct State {
    actions: Vec<String>,
    window_states: Vec<bool>,
    close_requests: usize,
    worker_result: Option<Result<String, String>>,
}
#[derive(Clone)]
enum Msg {
    Action(ZsWindowCommand),
    WindowState(ZsWindowState),
    CloseRequested,
}
fn view(state: &State) -> ViewNode<Msg> {
    column([
        text("Desktop window lifecycle").window_drag_region(),
        row([
            button("Minimize")
                .id(WidgetId::new(991))
                .on_click(Msg::Action(ZsWindowCommand::Minimize)),
            button("Maximize / restore")
                .id(WidgetId::new(992))
                .on_click(Msg::Action(ZsWindowCommand::ToggleMaximize)),
            button("Hide")
                .id(WidgetId::new(993))
                .on_click(Msg::Action(ZsWindowCommand::Hide)),
        ])
        .gap(Dp::new(8.0)),
        text(format!("Actions: {}", state.actions.len())),
        text(format!("Close requests: {}", state.close_requests)),
        text(format!("Panel response: {:?}", state.worker_result)),
    ])
    .padding(Dp::new(20.0))
    .gap(Dp::new(12.0))
    .on_window_state_change(Msg::WindowState)
}
fn update(state: &mut State, msg: Msg, cx: &mut AppCx) {
    match msg {
        Msg::Action(command) => {
            state.actions.push(format!("{command:?}"));
            cx.window_command(command);
        }
        Msg::WindowState(value) => state.window_states.push(value.maximized),
        Msg::CloseRequested => state.close_requests += 1,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    let argument = |name: &str| {
        args.iter()
            .position(|s| s == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let scene = argument("--scene").unwrap_or_else(|| "commands".into());
    let output = PathBuf::from(argument("--output").ok_or("--output is required")?);
    if !output.is_absolute() {
        return Err("--output must be absolute".into());
    }
    fs::create_dir_all(&output)?;
    if scene == "provider-failure" || scene == "provider-unavailable" || scene == "worker-no-host" {
        let result = thread::spawn(|| {
            NativeFileDialogService::new()
                .pick_directory_dialog(&DirectoryDialogSpec::new("Provider response"))
        })
        .join()
        .unwrap();
        fs::write(
            output.join("proof.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"scene":scene,"result":result}))?,
        )?;
        if scene == "provider-unavailable" {
            assert!(matches!(result, Err(zsui::ZsuiError::Unsupported { .. })));
        } else {
            assert!(matches!(result, Err(zsui::ZsuiError::Host { .. })));
        }
        return Ok(());
    }
    let state = Arc::new(Mutex::new(State::default()));
    let view_state = state.clone();
    let update_state = state.clone();
    let wake = UiInvalidationHandle::new();
    let worker_wake = wake.clone();
    let worker_state = state.clone();
    let worker_scene = scene.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        if worker_scene == "commands" {
            for id in [
                "maximize", "maximize", "minimize", "show", "hide", "show", "close",
            ] {
                worker_wake
                    .request_app_command(Command::custom(id))
                    .unwrap();
                thread::sleep(Duration::from_millis(450));
                if id == "hide" {
                    worker_wake.request_rebuild();
                    thread::sleep(Duration::from_millis(200));
                }
            }
        } else {
            let mut service = NativeFileDialogService::new();
            let result = match worker_scene.as_str() {
                "worker-open" => service
                    .open_file_dialog(&FileDialogSpec::new("Open a file"))
                    .map(|v| v.is_none()),
                "worker-save" => service
                    .save_file_dialog(&SaveFileDialogSpec::new("Save a file"))
                    .map(|v| v.is_none()),
                "worker-directory" => service
                    .pick_directory_dialog(&DirectoryDialogSpec::new("Choose a directory"))
                    .map(|v| v.is_none()),
                _ => panic!("unknown lifecycle scene"),
            }
            .map(|cancelled| {
                if cancelled {
                    "cancelled".to_owned()
                } else {
                    "selected".to_owned()
                }
            })
            .map_err(|e| e.to_string());
            worker_state.lock().unwrap().worker_result = Some(result);
            worker_wake.request_rebuild();
        }
    });
    let builder = native_window("ZSUI Desktop Lifecycle")
        .size(640, 300)
        .custom_title_bar(true)
        .release_view_when_hidden()
        .invalidation_handle(wake)
        .on_close_requested(Command::custom("close-requested"))
        .stateful_view_with_app_commands(
            (),
            move |_| view(&view_state.lock().unwrap()),
            move |_, msg, cx| update(&mut update_state.lock().unwrap(), msg, cx),
            |command| {
                if let Command::Custom { id, .. } = command {
                    match id.as_str() {
                        "maximize" => Some(Msg::Action(ZsWindowCommand::ToggleMaximize)),
                        "minimize" => Some(Msg::Action(ZsWindowCommand::Minimize)),
                        "show" => Some(Msg::Action(ZsWindowCommand::Show)),
                        "hide" => Some(Msg::Action(ZsWindowCommand::Hide)),
                        "close" => Some(Msg::Action(ZsWindowCommand::Close)),
                        "close-requested" => Some(Msg::CloseRequested),
                        _ => None,
                    }
                } else {
                    None
                }
            },
        );
    let report = builder.run_smoke(
        NativeWindowSmokeRunOptions::new(5500)
            .screenshot_file(output.join("window.png").to_string_lossy())
            .require_screenshot(true),
    )?;
    worker.join().unwrap();
    let state = state.lock().unwrap();
    fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"scene":scene,"actions":state.actions,"native_maximized_states":state.window_states,"close_requests":state.close_requests,"worker_result":state.worker_result,"runtime":report}),
        )?,
    )?;
    if scene == "commands" {
        assert_eq!(state.actions.len(), 7);
        assert_eq!(state.close_requests, 1);
        assert!(state.window_states.contains(&true));
        assert_eq!(state.window_states.last(), Some(&false));
    } else {
        assert_eq!(state.worker_result, Some(Ok("cancelled".into())));
    }
    assert!(report.visible_window_was_created() && report.screenshot_captured);
    Ok(())
}
