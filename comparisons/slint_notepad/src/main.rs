#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, env, fs, path::PathBuf, rc::Rc, time::Duration};

use slint::{ComponentHandle, SharedString, Timer};

#[path = "../../common/memory_probe.rs"]
mod memory_probe;

slint::slint! {
    import { Button, CheckBox, TextEdit } from "std-widgets.slint";

    export component NotepadWindow inherits Window {
        title: root.window-title;
        icon: @image-url("../../../assets/notepad/notepad.png");
        preferred-width: 900px;
        preferred-height: 620px;
        min-width: 520px;
        min-height: 360px;
        background: #f3f3f3;

        in property<string> window-title;
        in-out property<string> document-text;
        in-out property<bool> word-wrap: true;
        in-out property<bool> show-status: true;
        in property<string> status-left;
        in property<string> status-right;
        in property<string> error-text;

        callback new-document();
        callback open-document();
        callback save-document();
        callback save-as-document();
        callback document-edited(string);

        VerticalLayout {
            spacing: 0px;

            Rectangle {
                height: 58px;
                background: white;
                border-width: 1px;
                border-color: #e5e5e5;

                HorizontalLayout {
                    padding-left: 12px;
                    padding-right: 12px;
                    padding-top: 10px;
                    padding-bottom: 10px;
                    spacing: 8px;

                    Button { text: "New"; clicked => { root.new-document(); } }
                    Button { text: "Open..."; clicked => { root.open-document(); } }
                    Button { text: "Save"; clicked => { root.save-document(); } }
                    Button { text: "Save as..."; clicked => { root.save-as-document(); } }
                    Rectangle { horizontal-stretch: 1; }
                    CheckBox { text: "Word wrap"; checked <=> root.word-wrap; }
                    CheckBox { text: "Status"; checked <=> root.show-status; }
                }
            }

            Rectangle {
                horizontal-stretch: 1;
                vertical-stretch: 1;
                background: white;
                border-width: 1px;
                border-color: #d8d8d8;

                TextEdit {
                    x: 10px;
                    y: 10px;
                    width: parent.width - 20px;
                    height: parent.height - 20px;
                    text <=> root.document-text;
                    wrap: root.word-wrap ? TextWrap.word-wrap : TextWrap.no-wrap;
                    font-size: 14px;
                    edited => { root.document-edited(self.text); }
                }
            }

            if root.error-text != "": Rectangle {
                height: 32px;
                background: #fde7e9;
                Text {
                    x: 12px;
                    width: parent.width - 24px;
                    height: parent.height;
                    text: root.error-text;
                    color: #a4262c;
                    vertical-alignment: center;
                    overflow: elide;
                }
            }

            if root.show-status: Rectangle {
                height: 30px;
                background: #f8f8f8;
                border-width: 1px;
                border-color: #e5e5e5;

                HorizontalLayout {
                    padding-left: 12px;
                    padding-right: 12px;
                    Text { text: root.status-left; vertical-alignment: center; }
                    Rectangle { horizontal-stretch: 1; }
                    Text { text: root.status-right; vertical-alignment: center; }
                }
            }
        }
    }
}

#[derive(Clone)]
struct LaunchOptions {
    open_path: Option<PathBuf>,
    auto_close: Option<Duration>,
    memory_report: Option<PathBuf>,
    sample_after: Duration,
}

impl LaunchOptions {
    fn from_env() -> Self {
        let arguments = env::args().skip(1).collect::<Vec<_>>();
        let auto_close = arguments
            .windows(2)
            .find(|pair| pair[0] == "--benchmark-seconds")
            .and_then(|pair| pair[1].parse::<u64>().ok())
            .map(Duration::from_secs)
            .or_else(|| {
                arguments
                    .iter()
                    .any(|argument| argument == "--smoke")
                    .then_some(Duration::from_millis(1200))
            });
        let open_path = arguments
            .windows(2)
            .find(|pair| pair[0] == "--open")
            .map(|pair| PathBuf::from(&pair[1]));
        let memory_report = arguments
            .windows(2)
            .find(|pair| pair[0] == "--memory-report")
            .map(|pair| PathBuf::from(&pair[1]));
        let sample_after = arguments
            .windows(2)
            .find(|pair| pair[0] == "--sample-after-ms")
            .and_then(|pair| pair[1].parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_secs(3));

        Self {
            open_path,
            auto_close,
            memory_report,
            sample_after,
        }
    }
}

struct DocumentState {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    error: Option<String>,
}

impl DocumentState {
    fn new(open_path: Option<PathBuf>) -> Self {
        match open_path {
            Some(path) => match read_text(&path) {
                Ok(text) => Self {
                    path: Some(path),
                    text,
                    dirty: false,
                    error: None,
                },
                Err(error) => Self {
                    path: None,
                    text: String::new(),
                    dirty: false,
                    error: Some(error),
                },
            },
            None => Self {
                path: None,
                text: "Slint Notepad baseline / 记事本基线\n\nA declarative UI text editing benchmark. / 文本编辑内存基线。\n"
                    .to_string(),
                dirty: false,
                error: None,
            },
        }
    }

    fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let LaunchOptions {
        open_path,
        auto_close,
        memory_report,
        sample_after,
    } = LaunchOptions::from_env();
    let ui = NotepadWindow::new()?;
    let state = Rc::new(RefCell::new(DocumentState::new(open_path)));
    sync_ui(&ui, &state.borrow());

    ui.on_document_edited({
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        move |text: SharedString| {
            let mut state = state.borrow_mut();
            state.text = text.to_string();
            state.dirty = true;
            state.error = None;
            if let Some(ui) = weak.upgrade() {
                sync_ui(&ui, &state);
            }
        }
    });

    ui.on_new_document({
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        move || {
            let mut state = state.borrow_mut();
            state.path = None;
            state.text.clear();
            state.dirty = false;
            state.error = None;
            if let Some(ui) = weak.upgrade() {
                sync_ui(&ui, &state);
            }
        }
    });

    ui.on_open_document({
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Text", &["txt", "md", "log"])
                .pick_file()
            else {
                return;
            };
            let result = read_text(&path);
            let mut state = state.borrow_mut();
            match result {
                Ok(text) => {
                    state.path = Some(path);
                    state.text = text;
                    state.dirty = false;
                    state.error = None;
                }
                Err(error) => state.error = Some(error),
            }
            if let Some(ui) = weak.upgrade() {
                sync_ui(&ui, &state);
            }
        }
    });

    ui.on_save_document({
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        move || save_document(&weak, &state, false)
    });
    ui.on_save_as_document({
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        move || save_document(&weak, &state, true)
    });

    if let Some(path) = memory_report {
        Timer::single_shot(sample_after, move || {
            if let Err(error) =
                memory_probe::write_report(&path, "slint", "notepad", "first_frame_idle")
            {
                eprintln!("memory report failed: {error}");
            }
        });
    }

    if let Some(duration) = auto_close {
        let weak = ui.as_weak();
        Timer::single_shot(duration, move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        });
    }

    ui.run()
}

fn save_document(
    weak: &slint::Weak<NotepadWindow>,
    state: &Rc<RefCell<DocumentState>>,
    force_picker: bool,
) {
    let (current_path, display_name, text) = {
        let state = state.borrow();
        (state.path.clone(), state.display_name(), state.text.clone())
    };
    let path = if force_picker || current_path.is_none() {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md", "log"])
            .set_file_name(display_name)
            .save_file()
        else {
            return;
        };
        path
    } else {
        current_path.expect("path checked above")
    };

    let result = fs::write(&path, text.as_bytes());
    let mut state = state.borrow_mut();
    match result {
        Ok(()) => {
            state.path = Some(path);
            state.dirty = false;
            state.error = None;
        }
        Err(error) => state.error = Some(error.to_string()),
    }
    if let Some(ui) = weak.upgrade() {
        sync_ui(&ui, &state);
    }
}

fn sync_ui(ui: &NotepadWindow, state: &DocumentState) {
    ui.set_document_text(state.text.clone().into());
    ui.set_window_title(
        format!(
            "{}{} - Slint Notepad baseline",
            if state.dirty { "*" } else { "" },
            state.display_name()
        )
        .into(),
    );
    ui.set_status_left(format!("{} chars", state.text.chars().count()).into());
    ui.set_status_right("UTF-8".into());
    ui.set_error_text(state.error.clone().unwrap_or_default().into());
}

fn read_text(path: &PathBuf) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(rest.to_vec()).map_err(|error| error.to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        if rest.len() % 2 != 0 {
            return Err("UTF-16 file has an odd byte length".to_string());
        }
        let units = rest
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units).map_err(|error| error.to_string());
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}
