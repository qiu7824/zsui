#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use zsui::{
    button, column, native_window, row, styled_text, text, textbox, AppCx, DirectoryDialogSpec, Dp,
    NativeViewKey, NativeWindowSmokeRunOptions, SemanticTextStyle, Size, WidgetId,
    ZsTextEditCommand, ZsTextSelection, ZsuiResult,
};
const FIELD: WidgetId = WidgetId::new(951);
const ACTION: WidgetId = WidgetId::new(952);
const SELECT: WidgetId = WidgetId::new(953);
const PICK: WidgetId = WidgetId::new(954);
#[derive(Clone)]
enum Msg {
    Changed(String),
    Selection(ZsTextSelection),
    Action,
    Select,
    Pick,
    Picked(ZsuiResult<Option<PathBuf>>),
}
struct State {
    path: String,
    actions: usize,
    selections: usize,
    selected_all: bool,
    picked: Option<ZsuiResult<Option<PathBuf>>>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let arg = |name: &str| {
        args.windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].clone())
    };
    let directory = arg("--scene").as_deref() == Some("directory");
    let initial = format!("C:/{}document.txt", "工程资料/长路径🙂/".repeat(36));
    let state = Arc::new(Mutex::new(State {
        path: initial.clone(),
        actions: 0,
        selections: 0,
        selected_all: false,
        picked: None,
    }));
    let initial_folder = arg("--initial-directory").map(PathBuf::from);
    let builder = native_window("ZSUI Desktop Reliability")
        .size(960, 320)
        .stateful_view(
            state.clone(),
            move |state| {
                let state = state.lock().unwrap();
                if directory {
                    column([
                        text("Directory"),
                        button("Choose directory").id(PICK).on_click(Msg::Pick),
                        text(format!("{:?}", state.picked)),
                    ])
                    .padding(Dp::new(20.0))
                } else {
                    column([
                        row([
                            styled_text(
                                state.path.clone(),
                                SemanticTextStyle {
                                    ellipsis: true,
                                    ..SemanticTextStyle::body()
                                },
                            )
                            .flex(1.0),
                            button("Action")
                                .id(ACTION)
                                .width(Dp::new(96.0))
                                .on_click(Msg::Action),
                        ])
                        .gap(Dp::new(12.0)),
                        textbox(state.path.clone())
                            .id(FIELD)
                            .width(Dp::new(560.0))
                            .on_change(Msg::Changed)
                            .on_text_selection_change(Msg::Selection),
                        button("Select all").id(SELECT).on_click(Msg::Select),
                        text(format!("Actions: {}", state.actions)),
                    ])
                    .padding(Dp::new(20.0))
                    .gap(Dp::new(16.0))
                }
            },
            move |state, message, cx: &mut AppCx| {
                let mut state = state.lock().unwrap();
                match message {
                    Msg::Changed(value) => state.path = value,
                    Msg::Selection(selection) => {
                        state.selected_all |=
                            selection.ordered() == (0, state.path.chars().count());
                        state.selections += 1;
                    }
                    Msg::Action => state.actions += 1,
                    Msg::Select => cx.text_edit_command_for(FIELD, ZsTextEditCommand::SelectAll),
                    Msg::Pick => {
                        let mut spec = DirectoryDialogSpec::new("Choose directory");
                        if let Some(path) = &initial_folder {
                            spec = spec.current_path(path);
                        }
                        if let Err(error) = cx.try_pick_directory(spec, Msg::Picked) {
                            state.picked = Some(Err(zsui::ZsuiError::host(
                                "directory_queue",
                                error.to_string(),
                            )));
                        }
                    }
                    Msg::Picked(result) => state.picked = Some(result),
                }
            },
        );
    if !args.iter().any(|arg| arg == "--smoke") {
        builder.run()?;
        return Ok(());
    }
    let output = PathBuf::from(arg("--output").ok_or("--smoke requires --output")?);
    fs::create_dir_all(&output)?;
    let duration = arg("--duration-ms")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(if directory { 15000 } else { 1800 });
    let mut options = NativeWindowSmokeRunOptions::new(duration)
        .screenshot_file(output.join("window.png").to_string_lossy())
        .require_screenshot(true);
    if directory {
        options = options.native_view_click_widget(PICK);
    } else {
        options = options
            .native_window_resize(Size {
                width: 640,
                height: 320,
            })
            .require_native_window_resize(true)
            .native_view_click_widget(ACTION)
            .native_view_click_widget(FIELD)
            .native_view_key_down(NativeViewKey::Home)
            .native_view_key_down(NativeViewKey::End)
            .native_view_ime_preedit("中文🙂", Some((3, 3)))
            .native_view_ime_commit("中文🙂")
            .native_view_click_widget(SELECT)
            .native_view_click_widget(FIELD)
            .native_view_key_down(NativeViewKey::End)
            .native_view_text_input("tail");
    }
    let report = builder.run_smoke(options)?;
    let state = state.lock().unwrap();
    fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"path":state.path,"actions":state.actions,"selection_changes":state.selections,"directory_result":state.picked,"runtime":report}),
        )?,
    )?;
    assert!(report.visible_window_was_created() && report.screenshot_captured);
    if directory {
        assert!(
            state.picked.as_ref().is_some_and(|result| result.is_ok()),
            "directory service did not complete successfully"
        );
    } else {
        assert_eq!(state.path, format!("{initial}中文🙂tail"));
        assert_eq!(state.actions, 1);
        assert!(state.selections > 0);
        assert!(state.selected_all);
        assert!(report.native_view_ime_commit_count > 0);
    }
    Ok(())
}
