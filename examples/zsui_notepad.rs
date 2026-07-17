#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use zsui::{
    column, command_bar, content_dialog, native_window, row, tab_view, text, text_editor,
    toolbar_button, AppCx, Command, Dp, FileDialogService, FileDialogSpec, MenuItemSpec, MenuSpec,
    NativeFileDialogService, NativeViewKey, NativeWindowSmokeRunOptions, Point, SaveFileDialogSpec,
    TextWrap, ThemeColorToken, ViewNode, WidgetId, ZsAccelerator, ZsBaseControlMetrics,
    ZsCommandBarSpec, ZsContentDialogButton, ZsContentDialogResult, ZsContentDialogSpec,
    ZsDocumentShellCommand, ZsIcon, ZsTabId, ZsTabItem, ZsTextCursorStatus, ZsTextDocument,
    ZsTextEditCommand, ZsTextSelection, ZsuiError, ZsuiResult, ZsuiSpacingTokens,
};

const DOCUMENT_EDITOR: WidgetId = WidgetId::new(1);
const UNDO_BUTTON: WidgetId = WidgetId::new(2);
const WRAP_BUTTON: WidgetId = WidgetId::new(3);
const PENDING_DIALOG: WidgetId = WidgetId::new(4);
const DOCUMENT_TAB: ZsTabId = ZsTabId::new(1);
const EFFECT_OPEN: &str = "notepad.effect.open";
const EFFECT_SAVE: &str = "notepad.effect.save";
const EFFECT_SAVE_AS: &str = "notepad.effect.save-as";
const EFFECT_SAVE_PENDING: &str = "notepad.effect.save-pending";

type SharedState = Arc<Mutex<NotepadState>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    New,
    Open,
    Close,
}

impl PendingAction {
    const fn save_prompt(self) -> &'static str {
        match self {
            Self::New => "新建文档前是否保存更改？ / Save changes before creating a new document?",
            Self::Open => {
                "打开其他文档前是否保存更改？ / Save changes before opening another document?"
            }
            Self::Close => {
                "关闭应用前是否保存更改？ / Save changes before closing the application?"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NotepadState {
    document: ZsTextDocument,
    selection: ZsTextSelection,
    show_status: bool,
    word_wrap: bool,
    pending: Option<PendingAction>,
    notice: String,
}

impl Default for NotepadState {
    fn default() -> Self {
        Self {
            document: ZsTextDocument::untitled(
                "ZSUI Notepad\n\nThis editor, its layout and its state/update loop are shared by Win32, AppKit and GTK4.\n",
            ),
            selection: ZsTextSelection::default(),
            show_status: true,
            word_wrap: true,
            pending: None,
            notice: "就绪 / Ready".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    DocumentChanged(String),
    SelectionChanged(ZsTextSelection),
    Command(ZsDocumentShellCommand),
    PendingResult(ZsContentDialogResult),
}

fn lock_state(state: &SharedState) -> ZsuiResult<MutexGuard<'_, NotepadState>> {
    state
        .lock()
        .map_err(|_| ZsuiError::host("notepad.state", "shared state lock was poisoned"))
}

fn view(shared: &SharedState) -> ViewNode<Msg> {
    let state = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let dirty_mark = if state.document.is_dirty() {
        " •"
    } else {
        ""
    };
    let display_name = state.document.display_name();
    let display_name = if display_name == "Untitled" {
        "未命名 / Untitled".to_string()
    } else {
        display_name
    };
    let title = format!("{display_name}{dirty_mark}");

    let command_button = |label: &str, icon: ZsIcon, command: ZsDocumentShellCommand| {
        toolbar_button(label, icon).on_click(Msg::Command(command))
    };
    let command_bar = command_bar(
        ZsCommandBarSpec::new()
            .leading([
                command_button("新建 / New", ZsIcon::Add, ZsDocumentShellCommand::New),
                command_button("打开 / Open", ZsIcon::Folder, ZsDocumentShellCommand::Open),
                command_button("保存 / Save", ZsIcon::Save, ZsDocumentShellCommand::Save),
            ])
            .trailing([
                command_button("撤销 / Undo", ZsIcon::Undo, ZsDocumentShellCommand::Undo)
                    .id(UNDO_BUTTON),
                command_button(
                    "换行 / Wrap",
                    ZsIcon::Text,
                    ZsDocumentShellCommand::ToggleWrap,
                )
                .id(WRAP_BUTTON),
            ]),
    );

    let document_tab = tab_view(
        [ZsTabItem::new(
            DOCUMENT_TAB,
            title,
            text_editor(state.document.text())
                .id(DOCUMENT_EDITOR)
                .text_wrap(if state.word_wrap {
                    TextWrap::Word
                } else {
                    TextWrap::NoWrap
                })
                .flex(1.0)
                .on_change(Msg::DocumentChanged)
                .on_text_selection_change(Msg::SelectionChanged),
        )
        .icon(ZsIcon::File)],
        Some(DOCUMENT_TAB),
    );
    let mut content = vec![command_bar, document_tab.flex(1.0)];

    if state.show_status {
        let line_count = state.document.text().lines().count().max(1);
        let cursor =
            ZsTextCursorStatus::from_character_caret(state.document.text(), state.selection.caret);
        let status_metrics = ZsBaseControlMetrics::current();
        let status_field = |label: String| {
            let width = status_metrics.estimated_text_width_with_shaping_reserve(&label);
            text(label).width(width).flex(0.0)
        };
        content.push(
            row(vec![
                text(state.notice).flex(1.0),
                status_field(format!("{}:{} / Ln:Col", cursor.line, cursor.column)),
                status_field(format!("{line_count} 行 / lines")),
                status_field(format!("{} 字符 / chars", cursor.character_count)),
                status_field(state.document.encoding().label().to_string()),
                status_field(if state.word_wrap {
                    "换行 / Wrap".to_string()
                } else {
                    "不换行 / No wrap".to_string()
                }),
            ])
            .height(Dp::new(30.0))
            .gap(Dp::new(16.0))
            .bg(ThemeColorToken::Surface),
        );
    }

    let spacing = ZsuiSpacingTokens::default();
    let page = column(content)
        .gap(spacing.content_gap)
        .padding(spacing.content_padding)
        .bg(ThemeColorToken::Surface);

    let Some(pending) = state.pending else {
        return page;
    };
    content_dialog(
        PENDING_DIALOG,
        true,
        ZsContentDialogSpec::new(pending.save_prompt(), "取消 / Cancel")
            .title("未保存的更改 / Unsaved changes")
            .primary_button("保存 / Save")
            .secondary_button("放弃 / Discard")
            .default_button(ZsContentDialogButton::Primary),
        page,
    )
    .on_dialog_result(Msg::PendingResult)
}

fn update(shared: &mut SharedState, message: Msg, cx: &mut AppCx) {
    let mut state = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match message {
        Msg::DocumentChanged(value) => {
            state.document.replace_text(value);
            state.notice = "已修改 / Modified".to_string();
        }
        Msg::SelectionChanged(selection) => state.selection = selection,
        Msg::Command(command) => dispatch_document_command(&mut state, command, cx),
        Msg::PendingResult(result) => match result {
            ZsContentDialogResult::Primary => cx.command(Command::custom(EFFECT_SAVE_PENDING)),
            ZsContentDialogResult::Secondary => {
                if let Some(pending) = state.pending.take() {
                    continue_pending_action(&mut state, pending, cx);
                }
            }
            ZsContentDialogResult::Close => {
                state.pending = None;
                state.notice = "操作已取消 / Action cancelled".to_string();
            }
        },
    }
}

fn dispatch_document_command(
    state: &mut NotepadState,
    command: ZsDocumentShellCommand,
    cx: &mut AppCx,
) {
    match command {
        ZsDocumentShellCommand::New => request_pending_action(state, PendingAction::New, cx),
        ZsDocumentShellCommand::Open => request_pending_action(state, PendingAction::Open, cx),
        ZsDocumentShellCommand::Close => request_pending_action(state, PendingAction::Close, cx),
        ZsDocumentShellCommand::Save => {
            cx.command(Command::custom(if state.document.path().is_some() {
                EFFECT_SAVE
            } else {
                EFFECT_SAVE_AS
            }))
        }
        ZsDocumentShellCommand::SaveAs => cx.command(Command::custom(EFFECT_SAVE_AS)),
        ZsDocumentShellCommand::ToggleStatus => state.show_status = !state.show_status,
        ZsDocumentShellCommand::About => {
            state.notice = "ZSUI 记事本使用统一 Rust view/update 路径，不含 WebView / One Rust view/update path, no WebView".to_string();
        }
        ZsDocumentShellCommand::Undo => {
            cx.text_edit_command_for(DOCUMENT_EDITOR, ZsTextEditCommand::Undo)
        }
        ZsDocumentShellCommand::Cut => {
            cx.text_edit_command_for(DOCUMENT_EDITOR, ZsTextEditCommand::Cut)
        }
        ZsDocumentShellCommand::Copy => {
            cx.text_edit_command_for(DOCUMENT_EDITOR, ZsTextEditCommand::Copy)
        }
        ZsDocumentShellCommand::Paste => {
            cx.text_edit_command_for(DOCUMENT_EDITOR, ZsTextEditCommand::Paste)
        }
        ZsDocumentShellCommand::SelectAll => {
            cx.text_edit_command_for(DOCUMENT_EDITOR, ZsTextEditCommand::SelectAll)
        }
        ZsDocumentShellCommand::ToggleWrap => {
            state.word_wrap = !state.word_wrap;
            state.notice = if state.word_wrap {
                "已启用自动换行 / Word wrap enabled"
            } else {
                "已关闭自动换行 / Word wrap disabled"
            }
            .to_string();
        }
    }
}

fn request_pending_action(state: &mut NotepadState, action: PendingAction, cx: &mut AppCx) {
    if state.document.is_dirty() {
        state.pending = Some(action);
        state.notice = "未保存 / Unsaved".to_string();
    } else {
        continue_pending_action(state, action, cx);
    }
}

fn continue_pending_action(state: &mut NotepadState, action: PendingAction, cx: &mut AppCx) {
    match action {
        PendingAction::New => {
            state.document = ZsTextDocument::default();
            state.selection = ZsTextSelection::default();
            state.notice = "新建文档 / New document".to_string();
        }
        PendingAction::Open => cx.command(Command::custom(EFFECT_OPEN)),
        PendingAction::Close => cx.quit(),
    }
}

fn message_for_app_command(command: &Command) -> Option<Msg> {
    ZsDocumentShellCommand::from_command(command).map(Msg::Command)
}

fn menu_item(
    label: &str,
    command: ZsDocumentShellCommand,
    accelerator: Option<ZsAccelerator>,
) -> MenuItemSpec {
    let item = MenuItemSpec::command(label, command.to_command());
    match accelerator {
        Some(accelerator) => item.accelerator(accelerator),
        None => item,
    }
}

fn notepad_menu() -> MenuSpec {
    let mut file = MenuSpec::new();
    file.items.push(menu_item(
        "新建 / New",
        ZsDocumentShellCommand::New,
        Some(ZsAccelerator::primary_character('N')),
    ));
    file.items.push(menu_item(
        "打开… / Open…",
        ZsDocumentShellCommand::Open,
        Some(ZsAccelerator::primary_character('O')),
    ));
    file.items.push(menu_item(
        "保存 / Save",
        ZsDocumentShellCommand::Save,
        Some(ZsAccelerator::primary_character('S')),
    ));
    file.items.push(menu_item(
        "另存为… / Save as…",
        ZsDocumentShellCommand::SaveAs,
        Some(ZsAccelerator::primary_character('S').shifted()),
    ));
    file.items.push(MenuItemSpec::Separator);
    file.items.push(menu_item(
        "关闭 / Close",
        ZsDocumentShellCommand::Close,
        Some(ZsAccelerator::primary_character('W')),
    ));

    let mut edit = MenuSpec::new();
    edit.items.push(menu_item(
        "撤销 / Undo",
        ZsDocumentShellCommand::Undo,
        Some(ZsAccelerator::primary_character('Z')),
    ));
    edit.items.push(MenuItemSpec::Separator);
    edit.items.push(menu_item(
        "剪切 / Cut",
        ZsDocumentShellCommand::Cut,
        Some(ZsAccelerator::primary_character('X')),
    ));
    edit.items.push(menu_item(
        "复制 / Copy",
        ZsDocumentShellCommand::Copy,
        Some(ZsAccelerator::primary_character('C')),
    ));
    edit.items.push(menu_item(
        "粘贴 / Paste",
        ZsDocumentShellCommand::Paste,
        Some(ZsAccelerator::primary_character('V')),
    ));
    edit.items.push(MenuItemSpec::Separator);
    edit.items.push(menu_item(
        "全选 / Select all",
        ZsDocumentShellCommand::SelectAll,
        Some(ZsAccelerator::primary_character('A')),
    ));

    let mut view_menu = MenuSpec::new();
    view_menu.items.push(menu_item(
        "自动换行 / Word wrap",
        ZsDocumentShellCommand::ToggleWrap,
        None,
    ));
    view_menu.items.push(menu_item(
        "状态栏 / Status bar",
        ZsDocumentShellCommand::ToggleStatus,
        None,
    ));

    let mut help = MenuSpec::new();
    help.items.push(menu_item(
        "关于 ZSUI 记事本 / About ZSUI Notepad",
        ZsDocumentShellCommand::About,
        None,
    ));

    MenuSpec::new()
        .title("ZSUI Notepad")
        .submenu("文件 / File", file)
        .submenu("编辑 / Edit", edit)
        .submenu("视图 / View", view_menu)
        .submenu("帮助 / Help", help)
}

fn execute_effect(
    shared: &SharedState,
    command: &Command,
    dialogs: &mut impl FileDialogService,
) -> ZsuiResult<()> {
    let Command::Custom { id, payload: None } = command else {
        return Err(ZsuiError::invalid_spec(
            "notepad.effect",
            "expected a payload-free custom command",
        ));
    };
    match id.as_str() {
        EFFECT_OPEN => {
            open_document(shared, dialogs)?;
        }
        EFFECT_SAVE => {
            save_document(shared, dialogs, false)?;
        }
        EFFECT_SAVE_AS => {
            save_document(shared, dialogs, true)?;
        }
        EFFECT_SAVE_PENDING => save_pending_document(shared, dialogs)?,
        _ => {
            return Err(ZsuiError::invalid_spec(
                "notepad.effect",
                format!("unknown effect command `{id}`"),
            ));
        }
    }
    Ok(())
}

fn open_document(shared: &SharedState, dialogs: &mut impl FileDialogService) -> ZsuiResult<bool> {
    let current_directory = {
        let state = lock_state(shared)?;
        state
            .document
            .path()
            .and_then(|path| path.parent())
            .map(PathBuf::from)
    };
    let mut spec = FileDialogSpec::new("打开文本文档 / Open text document")
        .filter("文本文档 / Text documents", ["*.txt", "*.md", "*.rs"])
        .filter("所有文件 / All files", ["*.*"]);
    if let Some(directory) = current_directory {
        spec = spec.current_path(directory);
    }
    let Some(path) = dialogs
        .open_file_dialog(&spec)?
        .and_then(|paths| paths.into_iter().next())
    else {
        lock_state(shared)?.notice = "已取消打开 / Open cancelled".to_string();
        return Ok(false);
    };

    let document = ZsTextDocument::open(path)?;
    let name = document.display_name();
    let mut state = lock_state(shared)?;
    state.document = document;
    state.selection = ZsTextSelection::default();
    state.pending = None;
    state.notice = format!("已打开 {name} / Opened {name}");
    Ok(true)
}

fn save_document(
    shared: &SharedState,
    dialogs: &mut impl FileDialogService,
    force_save_as: bool,
) -> ZsuiResult<bool> {
    let document = lock_state(shared)?.document.clone();
    if document.path().is_some() && !force_save_as {
        let mut saved = document;
        saved.save()?;
        let name = saved.display_name();
        let mut state = lock_state(shared)?;
        state.document = saved;
        state.notice = format!("已保存 {name} / Saved {name}");
        return Ok(true);
    }

    let current_directory = document
        .path()
        .and_then(|path| path.parent())
        .map(PathBuf::from);
    let mut spec = SaveFileDialogSpec::new("保存文本文档 / Save text document")
        .suggested_name(document.display_name())
        .filter("文本文档 / Text documents", ["*.txt", "*.md"])
        .filter("所有文件 / All files", ["*.*"]);
    if let Some(directory) = current_directory {
        spec = spec.current_path(directory);
    }
    let Some(path) = dialogs.save_file_dialog(&spec)? else {
        lock_state(shared)?.notice = "已取消保存 / Save cancelled".to_string();
        return Ok(false);
    };

    let mut saved = document;
    saved.save_as(path)?;
    let name = saved.display_name();
    let mut state = lock_state(shared)?;
    state.document = saved;
    state.notice = format!("已保存 {name} / Saved {name}");
    Ok(true)
}

fn save_pending_document(
    shared: &SharedState,
    dialogs: &mut impl FileDialogService,
) -> ZsuiResult<()> {
    if lock_state(shared)?.pending.is_none() {
        save_document(shared, dialogs, false)?;
        return Ok(());
    }
    if !save_document(shared, dialogs, false)? {
        return Ok(());
    }

    let pending = lock_state(shared)?.pending.take();
    match pending {
        Some(PendingAction::New) => {
            let mut state = lock_state(shared)?;
            state.document = ZsTextDocument::default();
            state.selection = ZsTextSelection::default();
            state.notice = "已保存并新建文档 / Saved; new document created".to_string();
        }
        Some(PendingAction::Open) => {
            open_document(shared, dialogs)?;
        }
        Some(PendingAction::Close) => {
            lock_state(shared)?.notice =
                "已保存，请再次关闭以安全退出 / Saved; choose Close again to exit safely"
                    .to_string();
        }
        None => {}
    }
    Ok(())
}

fn main() -> ZsuiResult<()> {
    let shared = Arc::new(Mutex::new(NotepadState::default()));
    let executor_state = shared.clone();
    let builder = native_window("ZSUI Notepad")
        .size(960, 680)
        .min_size(640, 440)
        .menu(notepad_menu())
        .on_close_requested(ZsDocumentShellCommand::Close.to_command())
        .stateful_view_with_app_commands(shared.clone(), view, update, message_for_app_command)
        .app_command_executor(move |command| {
            let mut dialogs = NativeFileDialogService::new();
            let result = execute_effect(&executor_state, &command, &mut dialogs);
            if let Err(error) = &result {
                if let Ok(mut state) = executor_state.lock() {
                    state.notice = format!("操作失败 / Operation failed: {error}");
                }
            }
            result.map(|_| Vec::new())
        });

    let args = std::env::args().collect::<Vec<_>>();
    let native_proof = args.iter().any(|argument| argument == "--native-proof");
    if native_proof || args.iter().any(|argument| argument == "--smoke") {
        let interaction_plan = builder
            .native_view_interaction_plan()
            .ok_or_else(|| ZsuiError::host("notepad_smoke", "interaction plan is missing"))?;
        let undo_bounds = interaction_plan
            .hit_target_for_widget(UNDO_BUTTON)
            .map(|target| target.bounds)
            .ok_or_else(|| {
                ZsuiError::host("notepad_smoke", "Undo button has no interaction bounds")
            })?;
        let undo_point = Point {
            x: undo_bounds.x + undo_bounds.width / 2,
            y: undo_bounds.y + undo_bounds.height / 2,
        };
        let wrap_bounds = interaction_plan
            .hit_target_for_widget(WRAP_BUTTON)
            .map(|target| target.bounds)
            .ok_or_else(|| {
                ZsuiError::host("notepad_smoke", "Wrap button has no interaction bounds")
            })?;
        let wrap_point = Point {
            x: wrap_bounds.x + wrap_bounds.width / 2,
            y: wrap_bounds.y + wrap_bounds.height / 2,
        };
        let editor_bounds = interaction_plan
            .hit_target_for_widget(DOCUMENT_EDITOR)
            .map(|target| target.bounds)
            .ok_or_else(|| {
                ZsuiError::host("notepad_smoke", "Text editor has no interaction bounds")
            })?;
        let editor_point = Point {
            x: editor_bounds.x + editor_bounds.width / 3,
            y: editor_bounds.y + editor_bounds.height / 3,
        };
        let editor_top_edge = Point {
            x: editor_point.x,
            y: editor_bounds.y + 1,
        };
        let output = args
            .windows(2)
            .find(|pair| pair[0] == "--output")
            .map(|pair| PathBuf::from(&pair[1]));
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone())
            .or_else(|| {
                native_proof.then(|| {
                    output
                        .clone()
                        .unwrap_or_else(|| "target/native-proof".into())
                        .join("notepad-interaction.png")
                        .to_string_lossy()
                        .into_owned()
                })
            });
        let report_path = args
            .windows(2)
            .find(|pair| pair[0] == "--report")
            .map(|pair| pair[1].clone())
            .or_else(|| {
                native_proof.then(|| {
                    output
                        .unwrap_or_else(|| "target/native-proof".into())
                        .join("notepad-interaction.json")
                        .to_string_lossy()
                        .into_owned()
                })
            });
        if let Some(parent) = screenshot
            .as_deref()
            .map(std::path::Path::new)
            .and_then(std::path::Path::parent)
        {
            fs::create_dir_all(parent)
                .map_err(|error| ZsuiError::host("create_notepad_proof_dir", error.to_string()))?;
        }
        let smoke_text = (1..=36)
            .map(|line| format!("第{line:02}行"))
            .collect::<Vec<_>>()
            .join("\n");
        let horizontal_smoke_text = format!(
            "WiWi-עברית-中文-horizontal-start-{}-HORIZONTAL-END",
            "viewport-fill-".repeat(8)
        );
        let grapheme_smoke_text = "G-\u{65}\u{301}👩🏽‍💻";
        let mut options = NativeWindowSmokeRunOptions::new(2_000)
            .native_view_click(editor_point)
            .native_view_text_input(smoke_text)
            .native_view_drag(editor_point, editor_top_edge)
            .native_view_click(undo_point)
            .native_view_click(editor_point)
            .native_view_key_down(NativeViewKey::Up)
            .native_view_key_down(NativeViewKey::PageDown)
            .native_view_scroll(editor_point, -48)
            .native_view_click(wrap_point)
            .native_view_click(editor_point)
            .native_view_text_input(grapheme_smoke_text)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_text_input("\u{8}")
            .native_view_key_down(NativeViewKey::End)
            .native_view_text_input(horizontal_smoke_text)
            .native_view_key_down(NativeViewKey::End)
            .native_view_text_input("\nabאב")
            .native_view_key_down(NativeViewKey::Home)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Right)
            .native_window_close_request();
        if let Some(path) = screenshot {
            options = options.screenshot_file(path).require_screenshot(true);
        }
        let widgets = interaction_plan.hit_targets.clone();
        let report = builder.run_smoke(options)?;
        if let Some(path) = report_path {
            let document = if native_proof {
                serde_json::json!({
                    "schema": "zsui.native-proof/v1",
                    "platform": std::env::consts::OS,
                    "architecture": std::env::consts::ARCH,
                    "application": "zsui_notepad",
                    "scenario": "notepad-interaction",
                    "theme": "system",
                    "window": { "width": 960, "height": 680 },
                    "widgets": widgets,
                    "runtime": &report,
                })
            } else {
                serde_json::to_value(&report).map_err(|error| {
                    ZsuiError::host("serialize_notepad_report", error.to_string())
                })?
            };
            let bytes = serde_json::to_vec_pretty(&document)
                .map_err(|error| ZsuiError::host("serialize_notepad_report", error.to_string()))?;
            fs::write(path, bytes)
                .map_err(|error| ZsuiError::host("write_notepad_report", error.to_string()))?;
        }
        if !report.visible_window_was_created()
            || report.native_view_text_input_count == 0
            || report.native_view_text_undo_count == 0
            || report.native_view_text_navigation_count < 9
            || report.native_view_text_selection_change_count == 0
            || report.native_view_text_drag_count == 0
            || report.native_view_text_drag_scroll_count == 0
            || report.native_view_scroll_count == 0
            || report.native_view_unhandled_scroll_count != 0
            || report.native_view_unhandled_key_count != 0
            || report.native_view_window_close_request_count == 0
            || report.native_view_window_close_veto_count == 0
            || !report.window_menu_command_routed
        {
            return Err(ZsuiError::host(
                "notepad_smoke",
                "native window, menu routing, text input/page navigation/edge-drag scrolling, typed undo or close veto was not verified",
            ));
        }
        if lock_state(&shared)?.word_wrap {
            return Err(ZsuiError::host(
                "notepad_smoke",
                "runtime word-wrap toggle was not applied to shared state",
            ));
        }
        let grapheme_probe = lock_state(&shared)?.document.text().to_string();
        if !grapheme_probe.contains("G-👩🏽‍💻") || grapheme_probe.contains("\u{65}\u{301}")
        {
            return Err(ZsuiError::host(
                "notepad_smoke",
                "extended grapheme navigation or deletion split a committed text cluster",
            ));
        }
        if lock_state(&shared)?.pending != Some(PendingAction::Close) {
            return Err(ZsuiError::host(
                "notepad_smoke",
                "native title-bar close did not enter the shared unsaved-confirmation path",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(1);

    #[derive(Default)]
    struct TestDialogs {
        open: Option<Vec<PathBuf>>,
        save: Option<PathBuf>,
    }

    impl FileDialogService for TestDialogs {
        fn open_file_dialog(&mut self, _spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
            Ok(self.open.clone())
        }

        fn save_file_dialog(&mut self, _spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
            Ok(self.save.clone())
        }
    }

    #[test]
    fn dirty_document_requires_an_explicit_pending_decision() {
        let shared = Arc::new(Mutex::new(NotepadState::default()));
        let mut cx = AppCx::new();
        update(
            &mut shared.clone(),
            Msg::DocumentChanged("changed".to_string()),
            &mut cx,
        );
        update(
            &mut shared.clone(),
            Msg::Command(ZsDocumentShellCommand::Open),
            &mut cx,
        );

        let state = shared.lock().unwrap();
        assert!(state.document.is_dirty());
        assert_eq!(state.pending, Some(PendingAction::Open));
        assert!(cx.commands().is_empty());
    }

    #[test]
    fn dirty_document_vetoes_close_until_the_user_decides() {
        let shared = Arc::new(Mutex::new(NotepadState::default()));
        let mut cx = AppCx::new();
        update(
            &mut shared.clone(),
            Msg::DocumentChanged("changed".to_string()),
            &mut cx,
        );
        update(
            &mut shared.clone(),
            Msg::Command(ZsDocumentShellCommand::Close),
            &mut cx,
        );

        let state = shared.lock().unwrap();
        assert_eq!(state.pending, Some(PendingAction::Close));
        assert!(!cx.quit_requested());
    }

    #[test]
    fn file_effects_mutate_the_same_shared_cross_platform_state() {
        let suffix = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let source = std::env::temp_dir().join(format!(
            "zsui-notepad-open-{}-{suffix}.txt",
            std::process::id(),
        ));
        let target = source.with_file_name(format!(
            "zsui-notepad-save-{}-{suffix}.txt",
            std::process::id(),
        ));
        fs::write(&source, "opened").unwrap();
        let shared = Arc::new(Mutex::new(NotepadState::default()));
        let mut dialogs = TestDialogs {
            open: Some(vec![source.clone()]),
            save: Some(target.clone()),
        };

        execute_effect(&shared, &Command::custom(EFFECT_OPEN), &mut dialogs).unwrap();
        assert_eq!(shared.lock().unwrap().document.text(), "opened");
        shared
            .lock()
            .unwrap()
            .document
            .replace_text("saved as utf-8");
        execute_effect(&shared, &Command::custom(EFFECT_SAVE_AS), &mut dialogs).unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "saved as utf-8");
        assert!(!shared.lock().unwrap().document.is_dirty());
        let _ = fs::remove_file(source);
        let _ = fs::remove_file(target);
    }

    #[test]
    fn native_menu_exposes_supported_shared_editor_commands() {
        let menu = notepad_menu();
        let commands = menu
            .items
            .iter()
            .filter_map(|item| match item {
                MenuItemSpec::Submenu { menu, .. } => Some(menu),
                _ => None,
            })
            .flat_map(|menu| menu.items.iter())
            .filter_map(|item| match item {
                MenuItemSpec::Command { command, .. } => {
                    ZsDocumentShellCommand::from_command(command)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(commands.contains(&ZsDocumentShellCommand::Open));
        assert!(commands.contains(&ZsDocumentShellCommand::Undo));
        assert!(commands.contains(&ZsDocumentShellCommand::Cut));
        assert!(commands.contains(&ZsDocumentShellCommand::Copy));
        assert!(commands.contains(&ZsDocumentShellCommand::Paste));
        assert!(commands.contains(&ZsDocumentShellCommand::SelectAll));
        assert!(commands.contains(&ZsDocumentShellCommand::ToggleWrap));
    }

    fn button_count(node: &ViewNode<Msg>) -> usize {
        usize::from(matches!(node.kind, zsui::ViewNodeKind::Button { .. }))
            + node.children.iter().map(button_count).sum::<usize>()
    }

    fn document_tabs(node: &ViewNode<Msg>) -> Option<&[zsui::ZsTabSpec]> {
        if let zsui::ViewNodeKind::Tabs { tabs, .. } = &node.kind {
            return Some(tabs);
        }
        node.children.iter().find_map(document_tabs)
    }

    #[test]
    fn notepad_view_uses_the_shared_command_bar_and_document_tab_contracts() {
        let shared = Arc::new(Mutex::new(NotepadState::default()));
        let page = view(&shared);

        assert!(button_count(&page) >= 5);
        let tabs = document_tabs(&page).expect("document must be hosted by TabView");
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].icon, Some(ZsIcon::File));
        assert!(tabs[0].label.contains("Untitled"));
        assert_eq!(page.widget_text_wrap(DOCUMENT_EDITOR), Some(TextWrap::Word));
    }

    #[test]
    fn wrap_command_updates_the_shared_text_editor_configuration() {
        let shared = Arc::new(Mutex::new(NotepadState::default()));
        let mut cx = AppCx::new();
        assert_eq!(
            view(&shared).widget_text_wrap(DOCUMENT_EDITOR),
            Some(TextWrap::Word)
        );

        update(
            &mut shared.clone(),
            Msg::Command(ZsDocumentShellCommand::ToggleWrap),
            &mut cx,
        );

        assert!(!shared.lock().unwrap().word_wrap);
        assert_eq!(
            view(&shared).widget_text_wrap(DOCUMENT_EDITOR),
            Some(TextWrap::NoWrap)
        );
    }
}
