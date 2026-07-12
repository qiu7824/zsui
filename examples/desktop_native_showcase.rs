use std::fs;

use zsui::{
    button, column, list, native_window, row, scroll, spacer, text, text_editor, textbox, toggle,
    AppCx, AppEvent, ClipboardData, ClipboardService, Command, Dp, FileDialogService,
    FileDialogSpec, MenuSpec, NativeViewKey, NativeWindowSmokeRunOptions, Point,
    SaveFileDialogSpec, ThemeColorToken, ViewNode, WidgetId, ZsuiError, ZsuiResult, ZsuiThemeMode,
};

const TITLE_INPUT: WidgetId = WidgetId::new(100);
const DOCUMENT_EDITOR: WidgetId = WidgetId::new(101);
const THEME_TOGGLE: WidgetId = WidgetId::new(102);
const RECENT_SCROLL: WidgetId = WidgetId::new(103);
const NAV_EDITOR: WidgetId = WidgetId::new(10);
const NAV_RECENT: WidgetId = WidgetId::new(11);
const NAV_ABOUT: WidgetId = WidgetId::new(12);
const OPEN_BUTTON: WidgetId = WidgetId::new(20);
const SAVE_BUTTON: WidgetId = WidgetId::new(21);
const COPY_BUTTON: WidgetId = WidgetId::new(22);
const PASTE_BUTTON: WidgetId = WidgetId::new(23);

#[derive(Debug, Clone, PartialEq)]
struct AppState {
    active_navigation: usize,
    title: String,
    document: String,
    dark_theme: bool,
    selected_recent: Option<usize>,
    recent_scroll: Dp,
    recent_files: Vec<String>,
    status: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_navigation: 0,
            title: "Untitled".to_string(),
            document: "ZSUI uses one State, Msg, view and update path on every desktop target."
                .to_string(),
            dark_theme: false,
            selected_recent: None,
            recent_scroll: Dp::new(0.0),
            recent_files: vec![
                "Desktop host contract.md".to_string(),
                "Native menu notes.txt".to_string(),
                "IME verification.txt".to_string(),
                "DPI test matrix.md".to_string(),
                "Wayland smoke report.txt".to_string(),
            ],
            status: "Ready".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Navigate(usize),
    TitleChanged(String),
    DocumentChanged(String),
    ThemeChanged(bool),
    OpenFile,
    SaveFile,
    Copy,
    Paste,
    RecentSelected(usize),
    RecentScrolled(Dp),
}

fn view(state: &AppState) -> ViewNode<Msg> {
    let navigation = column(vec![
        text("ZSUI Desktop").height(Dp::new(48.0)),
        button("Editor")
            .id(NAV_EDITOR)
            .height(Dp::new(40.0))
            .on_click(Msg::Navigate(0)),
        button("Recent")
            .id(NAV_RECENT)
            .height(Dp::new(40.0))
            .on_click(Msg::Navigate(1)),
        button("About")
            .id(NAV_ABOUT)
            .height(Dp::new(40.0))
            .on_click(Msg::Navigate(2)),
        spacer(),
        text(format!("Section {}", state.active_navigation + 1)).height(Dp::new(32.0)),
    ])
    .width(Dp::new(220.0))
    .gap(Dp::new(8.0))
    .padding(Dp::new(12.0))
    .bg(ThemeColorToken::SurfaceRaised);

    let command_bar = row(vec![
        button("Open")
            .id(OPEN_BUTTON)
            .width(Dp::new(88.0))
            .on_click(Msg::OpenFile),
        button("Save")
            .id(SAVE_BUTTON)
            .width(Dp::new(88.0))
            .on_click(Msg::SaveFile),
        button("Copy")
            .id(COPY_BUTTON)
            .width(Dp::new(88.0))
            .on_click(Msg::Copy),
        button("Paste")
            .id(PASTE_BUTTON)
            .width(Dp::new(88.0))
            .on_click(Msg::Paste),
        spacer(),
    ])
    .height(Dp::new(40.0))
    .gap(Dp::new(8.0));

    let theme_row = row(vec![
        text(if state.dark_theme {
            "Dark theme"
        } else {
            "Light theme"
        }),
        spacer(),
        toggle(state.dark_theme)
            .id(THEME_TOGGLE)
            .width(Dp::new(48.0))
            .on_toggle(Msg::ThemeChanged),
    ])
    .height(Dp::new(36.0))
    .gap(Dp::new(8.0));

    let recent = scroll(
        list(state.recent_files.iter().enumerate(), |(index, name)| {
            text(name).id(WidgetId::new(200 + index as u64))
        })
        .selected_index(state.selected_recent)
        .on_select(Msg::RecentSelected)
        .gap(Dp::new(4.0)),
    )
    .id(RECENT_SCROLL)
    .height(Dp::new(136.0))
    .content_height(Dp::new(240.0))
    .scroll_y(state.recent_scroll)
    .on_scroll(Msg::RecentScrolled);

    let content = column(vec![
        command_bar,
        theme_row,
        textbox(&state.title)
            .id(TITLE_INPUT)
            .height(Dp::new(36.0))
            .on_change(Msg::TitleChanged),
        text_editor(&state.document)
            .id(DOCUMENT_EDITOR)
            .flex(1.0)
            .on_change(Msg::DocumentChanged),
        recent,
        text(&state.status).height(Dp::new(28.0)),
    ])
    .flex(1.0)
    .gap(Dp::new(10.0))
    .padding(Dp::new(16.0));

    row(vec![navigation, content])
        .gap(Dp::new(12.0))
        .bg(ThemeColorToken::Surface)
        .theme_mode(if state.dark_theme {
            ZsuiThemeMode::Dark
        } else {
            ZsuiThemeMode::Light
        })
}

fn update(state: &mut AppState, message: Msg, cx: &mut AppCx) {
    match message {
        Msg::Navigate(index) => {
            state.active_navigation = index;
            state.status = format!("Navigation changed to section {}", index + 1);
        }
        Msg::TitleChanged(title) => state.title = title,
        Msg::DocumentChanged(document) => state.document = document,
        Msg::ThemeChanged(dark) => {
            state.dark_theme = dark;
            state.status = if dark {
                "Dark theme requested"
            } else {
                "Light theme requested"
            }
            .to_string();
            cx.command(Command::custom(if dark {
                "desktop.theme.dark"
            } else {
                "desktop.theme.light"
            }));
        }
        Msg::OpenFile => {
            state.status = "Open-file dialog requested".to_string();
            cx.command(Command::custom("desktop.file.open"));
        }
        Msg::SaveFile => {
            state.status = "Save-file dialog requested".to_string();
            cx.command(Command::custom("desktop.file.save"));
        }
        Msg::Copy => {
            state.status = "Copy requested".to_string();
            cx.command(Command::CopySelection);
        }
        Msg::Paste => {
            state.status = "Paste requested".to_string();
            cx.command(Command::PasteSelection);
        }
        Msg::RecentSelected(index) => {
            state.selected_recent = Some(index);
            state.status = format!("Selected {}", state.recent_files[index]);
        }
        Msg::RecentScrolled(offset) => state.recent_scroll = offset,
    }
}

fn showcase_menu() -> MenuSpec {
    MenuSpec::new()
        .submenu(
            "File",
            MenuSpec::new()
                .item("Open", Command::custom("desktop.file.open"))
                .item("Save", Command::custom("desktop.file.save"))
                .separator()
                .item("Quit", Command::Quit),
        )
        .submenu(
            "Edit",
            MenuSpec::new()
                .item("Copy", Command::CopySelection)
                .item("Paste", Command::PasteSelection),
        )
}

pub fn open_document(
    services: &mut impl FileDialogService,
) -> ZsuiResult<Option<(String, String)>> {
    let selection = services.open_file_dialog(
        &FileDialogSpec::new("Open document").filter("Text", ["*.txt", "*.md"]),
    )?;
    let Some(path) = selection.and_then(|paths| paths.into_iter().next()) else {
        return Ok(None);
    };
    let document = fs::read_to_string(&path)
        .map_err(|error| ZsuiError::host("read_showcase_document", error.to_string()))?;
    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string();
    Ok(Some((title, document)))
}

pub fn save_document(
    services: &mut impl FileDialogService,
    suggested_name: &str,
    document: &str,
) -> ZsuiResult<Option<String>> {
    let selection = services.save_file_dialog(
        &SaveFileDialogSpec::new("Save document")
            .suggested_name(suggested_name)
            .filter("Text", ["*.txt", "*.md"]),
    )?;
    let Some(path) = selection else {
        return Ok(None);
    };
    fs::write(&path, document)
        .map_err(|error| ZsuiError::host("write_showcase_document", error.to_string()))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

pub fn copy_document(services: &mut impl ClipboardService, document: &str) -> ZsuiResult<()> {
    services.write_clipboard(&ClipboardData::text(document))
}

pub fn paste_document(services: &mut impl ClipboardService) -> ZsuiResult<Option<String>> {
    Ok(match services.read_clipboard()? {
        Some(ClipboardData::Text(text)) => Some(text),
        _ => None,
    })
}

fn main() -> ZsuiResult<()> {
    let builder = native_window("ZSUI Native Showcase")
        .size(960, 640)
        .min_size(760, 520)
        .icon_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/notepad/notepad.ico"
        ))
        .menu(showcase_menu())
        .stateful_view(AppState::default(), view, update)
        .app_command_executor(|command| Ok(vec![AppEvent::MenuCommand { command }]));

    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|argument| argument == "--smoke") {
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone());
        let report_path = args
            .windows(2)
            .find(|pair| pair[0] == "--report")
            .map(|pair| pair[1].clone());
        let scenario = args
            .windows(2)
            .find(|pair| pair[0] == "--scenario")
            .map(|pair| pair[1].as_str())
            .unwrap_or("interaction");
        let mut options = NativeWindowSmokeRunOptions::new(1_200);
        if matches!(scenario, "interaction" | "text-input") {
            options = options
                .native_view_click(Point { x: 270, y: 20 })
                .native_view_click(Point { x: 300, y: 126 })
                .native_view_text_input("Native")
                .native_view_key_down(NativeViewKey::Tab)
                .native_view_scroll(Point { x: 300, y: 480 }, 48);
            if scenario == "interaction" {
                options = options.native_view_click(Point { x: 920, y: 82 });
            }
        } else if scenario == "dark-theme" {
            options = options.native_view_click(Point { x: 920, y: 82 });
        } else if scenario != "startup" {
            return Err(ZsuiError::invalid_spec(
                "desktop_native_showcase.scenario",
                format!("unknown smoke scenario `{scenario}`"),
            ));
        }
        if let Some(path) = screenshot {
            options = options.screenshot_file(path).require_screenshot(true);
        }
        let report = builder.run_smoke(options)?;
        if let Some(path) = report_path {
            let bytes = serde_json::to_vec_pretty(&report).map_err(|error| {
                ZsuiError::host("serialize_desktop_showcase_report", error.to_string())
            })?;
            fs::write(path, bytes).map_err(|error| {
                ZsuiError::host("write_desktop_showcase_report", error.to_string())
            })?;
        }
        if !report.visible_window_was_created() {
            return Err(ZsuiError::host(
                "desktop_native_showcase_smoke",
                "the native showcase window was not created",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    struct Services {
        open: Option<Vec<PathBuf>>,
        save: Option<PathBuf>,
        clipboard: Option<ClipboardData>,
    }

    impl FileDialogService for Services {
        fn open_file_dialog(&mut self, _spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
            Ok(self.open.clone())
        }

        fn save_file_dialog(&mut self, _spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
            Ok(self.save.clone())
        }
    }

    impl ClipboardService for Services {
        fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
            Ok(self.clipboard.clone())
        }

        fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
            self.clipboard = Some(data.clone());
            Ok(())
        }
    }

    #[test]
    fn typed_update_changes_state_and_emits_desktop_commands() {
        let mut state = AppState::default();
        let mut cx = AppCx::new();

        update(&mut state, Msg::ThemeChanged(true), &mut cx);
        update(&mut state, Msg::OpenFile, &mut cx);

        assert!(state.dark_theme);
        assert_eq!(cx.commands().len(), 2);
        assert_eq!(cx.commands()[1], Command::custom("desktop.file.open"));
    }

    #[test]
    fn platform_independent_file_and_clipboard_logic_uses_services() {
        let path =
            std::env::temp_dir().join(format!("zsui-desktop-showcase-{}.txt", std::process::id()));
        fs::write(&path, "native desktop").unwrap();
        let mut services = Services {
            open: Some(vec![path.clone()]),
            save: Some(path.clone()),
            clipboard: None,
        };

        let opened = open_document(&mut services).unwrap().unwrap();
        assert_eq!(opened.1, "native desktop");
        assert!(save_document(&mut services, "notes.txt", "saved")
            .unwrap()
            .is_some());
        copy_document(&mut services, "copied").unwrap();
        assert_eq!(
            paste_document(&mut services).unwrap().as_deref(),
            Some("copied")
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "saved");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn showcase_declares_focusable_editor_scroll_and_native_menu() {
        let mut view = view(&AppState::default());
        let mut layout = zsui::ViewLayoutCx::new(
            zsui::Rect {
                x: 0,
                y: 0,
                width: 960,
                height: 640,
            },
            zsui::Dpi::standard(),
        );
        zsui::View::layout(&mut view, &mut layout);
        let interactions = view.interaction_plan();
        let menu = showcase_menu();

        assert_eq!(
            interactions
                .hit_target_for_widget(DOCUMENT_EDITOR)
                .map(|target| target.kind),
            Some(zsui::ViewHitTargetKind::TextEditor)
        );
        assert!(interactions.hit_target_for_widget(RECENT_SCROLL).is_some());
        assert_eq!(menu.items.len(), 2);
    }
}
