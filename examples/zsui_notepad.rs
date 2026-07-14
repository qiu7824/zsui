use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use zsui::{
    button, column, native_window, row, spacer, text, text_editor, AppCx, Command, Dp,
    FileDialogService, FileDialogSpec, MenuItemSpec, MenuSpec, NativeFileDialogService,
    NativeViewKey, NativeWindowSmokeRunOptions, Point, SaveFileDialogSpec, TextWrap,
    ThemeColorToken, ViewNode, WidgetId, ZsAccelerator, ZsDocumentShellCommand, ZsTextCursorStatus,
    ZsTextDocument, ZsTextEditCommand, ZsTextSelection, ZsuiError, ZsuiResult,
};

const DOCUMENT_EDITOR: WidgetId = WidgetId::new(1);
const UNDO_BUTTON: WidgetId = WidgetId::new(2);
const WRAP_BUTTON: WidgetId = WidgetId::new(3);
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
    const fn label(self) -> &'static str {
        match self {
            Self::New => "creating a new document",
            Self::Open => "opening another document",
            Self::Close => "closing the application",
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
            notice: "Ready".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    DocumentChanged(String),
    SelectionChanged(ZsTextSelection),
    Command(ZsDocumentShellCommand),
    SavePending,
    DiscardPending,
    CancelPending,
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
    let title = format!("{}{}", state.document.display_name(), dirty_mark);

    let document_header = row(vec![
        text(title).flex(1.0),
        text("Self-drawn editor · native desktop host"),
    ])
    .height(Dp::new(36.0))
    .gap(Dp::new(12.0));

    let command_bar = row(vec![
        command_button("New", ZsDocumentShellCommand::New),
        command_button("Open", ZsDocumentShellCommand::Open),
        command_button("Save", ZsDocumentShellCommand::Save),
        command_button("Save as", ZsDocumentShellCommand::SaveAs),
        command_button("Undo", ZsDocumentShellCommand::Undo).id(UNDO_BUTTON),
        spacer(),
        command_button("Status", ZsDocumentShellCommand::ToggleStatus),
        command_button("Wrap", ZsDocumentShellCommand::ToggleWrap).id(WRAP_BUTTON),
        command_button("About", ZsDocumentShellCommand::About),
    ])
    .height(Dp::new(40.0))
    .gap(Dp::new(8.0))
    .bg(ThemeColorToken::Surface);

    let mut content = vec![document_header, command_bar];
    if let Some(pending) = state.pending {
        content.push(
            column(vec![
                text(format!("Save changes before {}?", pending.label())),
                row(vec![
                    button("Save").on_click(Msg::SavePending),
                    button("Discard").on_click(Msg::DiscardPending),
                    button("Cancel").on_click(Msg::CancelPending),
                    spacer(),
                ])
                .height(Dp::new(36.0))
                .gap(Dp::new(8.0)),
            ])
            .gap(Dp::new(6.0))
            .padding(Dp::new(10.0))
            .radius(Dp::new(8.0))
            .bg(ThemeColorToken::SurfaceRaised),
        );
    }

    content.push(
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
    );

    if state.show_status {
        let line_count = state.document.text().lines().count().max(1);
        let cursor =
            ZsTextCursorStatus::from_character_caret(state.document.text(), state.selection.caret);
        content.push(
            row(vec![
                text(state.notice).flex(1.0),
                text(format!("Ln {}, Col {}", cursor.line, cursor.column)),
                text(format!("Lines {line_count}")),
                text(format!("Characters {}", cursor.character_count)),
                text(state.document.encoding().label()),
                text(if state.word_wrap {
                    "Wrap on"
                } else {
                    "Wrap off"
                }),
            ])
            .height(Dp::new(30.0))
            .gap(Dp::new(16.0))
            .bg(ThemeColorToken::Surface),
        );
    }

    column(content)
        .gap(Dp::new(10.0))
        .padding(Dp::new(12.0))
        .bg(ThemeColorToken::Surface)
}

fn command_button(label: &str, command: ZsDocumentShellCommand) -> ViewNode<Msg> {
    button(label)
        .width(Dp::new(88.0))
        .on_click(Msg::Command(command))
}

fn update(shared: &mut SharedState, message: Msg, cx: &mut AppCx) {
    let mut state = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match message {
        Msg::DocumentChanged(value) => {
            state.document.replace_text(value);
            state.notice = "Modified".to_string();
        }
        Msg::SelectionChanged(selection) => state.selection = selection,
        Msg::Command(command) => dispatch_document_command(&mut state, command, cx),
        Msg::SavePending => cx.command(Command::custom(EFFECT_SAVE_PENDING)),
        Msg::DiscardPending => {
            if let Some(pending) = state.pending.take() {
                continue_pending_action(&mut state, pending, cx);
            }
        }
        Msg::CancelPending => {
            state.pending = None;
            state.notice = "Action cancelled".to_string();
        }
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
            state.notice =
                "ZSUI Notepad uses one Rust view/update path and no WebView.".to_string();
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
                "Word wrap enabled"
            } else {
                "Word wrap disabled"
            }
            .to_string();
        }
    }
}

fn request_pending_action(state: &mut NotepadState, action: PendingAction, cx: &mut AppCx) {
    if state.document.is_dirty() {
        state.pending = Some(action);
        state.notice = "Unsaved changes".to_string();
    } else {
        continue_pending_action(state, action, cx);
    }
}

fn continue_pending_action(state: &mut NotepadState, action: PendingAction, cx: &mut AppCx) {
    match action {
        PendingAction::New => {
            state.document = ZsTextDocument::default();
            state.selection = ZsTextSelection::default();
            state.notice = "New document".to_string();
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
        "New",
        ZsDocumentShellCommand::New,
        Some(ZsAccelerator::primary_character('N')),
    ));
    file.items.push(menu_item(
        "Open…",
        ZsDocumentShellCommand::Open,
        Some(ZsAccelerator::primary_character('O')),
    ));
    file.items.push(menu_item(
        "Save",
        ZsDocumentShellCommand::Save,
        Some(ZsAccelerator::primary_character('S')),
    ));
    file.items.push(menu_item(
        "Save as…",
        ZsDocumentShellCommand::SaveAs,
        Some(ZsAccelerator::primary_character('S').shifted()),
    ));
    file.items.push(MenuItemSpec::Separator);
    file.items.push(menu_item(
        "Close",
        ZsDocumentShellCommand::Close,
        Some(ZsAccelerator::primary_character('W')),
    ));

    let mut edit = MenuSpec::new();
    edit.items.push(menu_item(
        "Undo",
        ZsDocumentShellCommand::Undo,
        Some(ZsAccelerator::primary_character('Z')),
    ));
    edit.items.push(MenuItemSpec::Separator);
    edit.items.push(menu_item(
        "Cut",
        ZsDocumentShellCommand::Cut,
        Some(ZsAccelerator::primary_character('X')),
    ));
    edit.items.push(menu_item(
        "Copy",
        ZsDocumentShellCommand::Copy,
        Some(ZsAccelerator::primary_character('C')),
    ));
    edit.items.push(menu_item(
        "Paste",
        ZsDocumentShellCommand::Paste,
        Some(ZsAccelerator::primary_character('V')),
    ));
    edit.items.push(MenuItemSpec::Separator);
    edit.items.push(menu_item(
        "Select all",
        ZsDocumentShellCommand::SelectAll,
        Some(ZsAccelerator::primary_character('A')),
    ));

    let mut view_menu = MenuSpec::new();
    view_menu.items.push(menu_item(
        "Word wrap",
        ZsDocumentShellCommand::ToggleWrap,
        None,
    ));
    view_menu.items.push(menu_item(
        "Status bar",
        ZsDocumentShellCommand::ToggleStatus,
        None,
    ));

    let mut help = MenuSpec::new();
    help.items.push(menu_item(
        "About ZSUI Notepad",
        ZsDocumentShellCommand::About,
        None,
    ));

    MenuSpec::new()
        .title("ZSUI Notepad")
        .submenu("File", file)
        .submenu("Edit", edit)
        .submenu("View", view_menu)
        .submenu("Help", help)
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
    let mut spec = FileDialogSpec::new("Open text document")
        .filter("Text documents", ["*.txt", "*.md", "*.rs"])
        .filter("All files", ["*.*"]);
    if let Some(directory) = current_directory {
        spec = spec.current_path(directory);
    }
    let Some(path) = dialogs
        .open_file_dialog(&spec)?
        .and_then(|paths| paths.into_iter().next())
    else {
        lock_state(shared)?.notice = "Open cancelled".to_string();
        return Ok(false);
    };

    let document = ZsTextDocument::open(path)?;
    let name = document.display_name();
    let mut state = lock_state(shared)?;
    state.document = document;
    state.selection = ZsTextSelection::default();
    state.pending = None;
    state.notice = format!("Opened {name}");
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
        state.notice = format!("Saved {name}");
        return Ok(true);
    }

    let current_directory = document
        .path()
        .and_then(|path| path.parent())
        .map(PathBuf::from);
    let mut spec = SaveFileDialogSpec::new("Save text document")
        .suggested_name(document.display_name())
        .filter("Text documents", ["*.txt", "*.md"])
        .filter("All files", ["*.*"]);
    if let Some(directory) = current_directory {
        spec = spec.current_path(directory);
    }
    let Some(path) = dialogs.save_file_dialog(&spec)? else {
        lock_state(shared)?.notice = "Save cancelled".to_string();
        return Ok(false);
    };

    let mut saved = document;
    saved.save_as(path)?;
    let name = saved.display_name();
    let mut state = lock_state(shared)?;
    state.document = saved;
    state.notice = format!("Saved {name}");
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
            state.notice = "Saved; new document created".to_string();
        }
        Some(PendingAction::Open) => {
            open_document(shared, dialogs)?;
        }
        Some(PendingAction::Close) => {
            lock_state(shared)?.notice = "Saved. Choose Close again to exit safely.".to_string();
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
                    state.notice = format!("Operation failed: {error}");
                }
            }
            result.map(|_| Vec::new())
        });

    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|argument| argument == "--smoke") {
        let undo_bounds = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(UNDO_BUTTON))
            .map(|target| target.bounds)
            .ok_or_else(|| {
                ZsuiError::host("notepad_smoke", "Undo button has no interaction bounds")
            })?;
        let undo_point = Point {
            x: undo_bounds.x + undo_bounds.width / 2,
            y: undo_bounds.y + undo_bounds.height / 2,
        };
        let wrap_bounds = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(WRAP_BUTTON))
            .map(|target| target.bounds)
            .ok_or_else(|| {
                ZsuiError::host("notepad_smoke", "Wrap button has no interaction bounds")
            })?;
        let wrap_point = Point {
            x: wrap_bounds.x + wrap_bounds.width / 2,
            y: wrap_bounds.y + wrap_bounds.height / 2,
        };
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone());
        let report_path = args
            .windows(2)
            .find(|pair| pair[0] == "--report")
            .map(|pair| pair[1].clone());
        let smoke_text = (1..=36)
            .map(|line| format!("第{line:02}行"))
            .collect::<Vec<_>>()
            .join("\n");
        let horizontal_smoke_text = format!(
            "horizontal-start-{}-HORIZONTAL-END",
            "viewport-fill-".repeat(16)
        );
        let mut options = NativeWindowSmokeRunOptions::new(1_200)
            .native_view_click(Point { x: 360, y: 220 })
            .native_view_text_input(smoke_text)
            .native_view_drag(Point { x: 360, y: 220 }, Point { x: 360, y: 100 })
            .native_view_click(undo_point)
            .native_view_click(Point { x: 360, y: 220 })
            .native_view_key_down(NativeViewKey::Up)
            .native_view_key_down(NativeViewKey::PageDown)
            .native_view_scroll(Point { x: 360, y: 220 }, -48)
            .native_view_click(wrap_point)
            .native_view_click(Point { x: 360, y: 220 })
            .native_view_text_input(horizontal_smoke_text)
            .native_view_key_down(NativeViewKey::End)
            .native_window_close_request();
        if let Some(path) = screenshot {
            options = options.screenshot_file(path).require_screenshot(true);
        }
        let report = builder.run_smoke(options)?;
        if let Some(path) = report_path {
            let bytes = serde_json::to_vec_pretty(&report)
                .map_err(|error| ZsuiError::host("serialize_notepad_report", error.to_string()))?;
            fs::write(path, bytes)
                .map_err(|error| ZsuiError::host("write_notepad_report", error.to_string()))?;
        }
        if !report.visible_window_was_created()
            || report.native_view_text_input_count == 0
            || report.native_view_text_undo_count == 0
            || report.native_view_text_navigation_count < 3
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
