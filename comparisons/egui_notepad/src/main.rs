#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::egui;

fn main() -> eframe::Result {
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
    let icon = eframe::icon_data::from_png_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/notepad/notepad.png"
    )))
    .ok();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([520.0, 360.0])
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };
    eframe::run_native(
        "egui Notepad baseline",
        options,
        Box::new(move |context| {
            context.egui_ctx.set_theme(egui::Theme::Light);
            context.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(NotepadApp::new(open_path, auto_close)))
        }),
    )
}

struct NotepadApp {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    word_wrap: bool,
    show_status: bool,
    line: usize,
    column: usize,
    started: Instant,
    auto_close: Option<Duration>,
    confirm_close: bool,
    error: Option<String>,
}

impl NotepadApp {
    fn new(path: Option<PathBuf>, auto_close: Option<Duration>) -> Self {
        let (path, text, error) = match path {
            Some(path) => match read_text(&path) {
                Ok(text) => (Some(path), text, None),
                Err(error) => (None, String::new(), Some(error)),
            },
            None => (
                None,
                "egui Notepad baseline\n\nA complete immediate-mode text editing benchmark.\n"
                    .to_string(),
                None,
            ),
        };
        Self {
            path,
            text,
            dirty: false,
            word_wrap: true,
            show_status: true,
            line: 1,
            column: 1,
            started: Instant::now(),
            auto_close,
            confirm_close: false,
            error,
        }
    }

    fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn title(&self) -> String {
        format!(
            "{}{} - egui Notepad baseline",
            if self.dirty { "*" } else { "" },
            self.display_name()
        )
    }

    fn new_document(&mut self) {
        if self.dirty {
            self.confirm_close = true;
            return;
        }
        self.path = None;
        self.text.clear();
        self.dirty = false;
    }

    fn open_document(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md", "log"])
            .pick_file()
        else {
            return;
        };
        match read_text(&path) {
            Ok(text) => {
                self.path = Some(path);
                self.text = text;
                self.dirty = false;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn save(&mut self, force_picker: bool) -> bool {
        let path = if force_picker || self.path.is_none() {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Text", &["txt", "md", "log"])
                .set_file_name(self.display_name())
                .save_file()
            else {
                return false;
            };
            self.path = Some(path.clone());
            path
        } else {
            self.path.clone().expect("path checked above")
        };
        match fs::write(path, self.text.as_bytes()) {
            Ok(()) => {
                self.dirty = false;
                true
            }
            Err(error) => {
                self.error = Some(error.to_string());
                false
            }
        }
    }
}

impl eframe::App for NotepadApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        *ui.visuals_mut() = egui::Visuals::light();
        let background = ui.visuals().panel_fill;
        ui.painter().rect_filled(ui.max_rect(), 0.0, background);

        let context = ui.ctx().clone();
        if self
            .auto_close
            .is_some_and(|duration| self.started.elapsed() >= duration)
        {
            self.dirty = false;
            context.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if context.input(|input| input.viewport().close_requested()) && self.dirty {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        context.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));

        self.handle_shortcuts(&context);
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New    Ctrl+N").clicked() {
                    self.new_document();
                    ui.close();
                }
                if ui.button("Open...    Ctrl+O").clicked() {
                    self.open_document();
                    ui.close();
                }
                if ui.button("Save    Ctrl+S").clicked() {
                    self.save(false);
                    ui.close();
                }
                if ui.button("Save As...    Ctrl+Shift+S").clicked() {
                    self.save(true);
                    ui.close();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    if self.dirty {
                        self.confirm_close = true;
                    } else {
                        context.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                ui.label("Undo, cut, copy, paste and select-all use standard shortcuts.");
            });
            ui.menu_button("Format", |ui| {
                ui.checkbox(&mut self.word_wrap, "Word wrap");
            });
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_status, "Status bar");
            });
            ui.menu_button("Help", |ui| {
                ui.label("egui comparison implementation");
            });
        });
        ui.separator();
        let status_height = if self.show_status { 30.0 } else { 0.0 };
        let editor_size = egui::vec2(
            ui.available_width(),
            (ui.available_height() - status_height).max(80.0),
        );
        ui.allocate_ui_with_layout(
            editor_size,
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let mut edit = egui::TextEdit::multiline(&mut self.text)
                    .desired_rows(30)
                    .desired_width(ui.available_width())
                    .lock_focus(true);
                if !self.word_wrap {
                    edit = edit.desired_width(f32::INFINITY);
                }
                let output = edit.show(ui);
                if output.response.changed() {
                    self.dirty = true;
                }
                if let Some(range) = output.cursor_range {
                    let caret: usize = range.primary.index.into();
                    let caret = caret.min(self.text.chars().count());
                    let prefix = self.text.chars().take(caret).collect::<String>();
                    self.line = prefix
                        .chars()
                        .filter(|character| *character == '\n')
                        .count()
                        + 1;
                    self.column = prefix
                        .rsplit_once('\n')
                        .map(|(_, tail)| tail.chars().count() + 1)
                        .unwrap_or_else(|| prefix.chars().count() + 1);
                }
            },
        );

        if self.show_status {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Ln {}, Col {}", self.line, self.column));
                ui.separator();
                ui.label(format!("{} chars", self.text.chars().count()));
                ui.separator();
                ui.label("UTF-8");
                ui.separator();
                ui.label(if self.word_wrap { "Wrap" } else { "No wrap" });
            });
        }

        self.show_dialogs(&context);
    }
}

impl NotepadApp {
    fn handle_shortcuts(&mut self, context: &egui::Context) {
        let command = egui::Modifiers::COMMAND;
        if context.input_mut(|input| input.consume_key(command, egui::Key::N)) {
            self.new_document();
        }
        if context.input_mut(|input| input.consume_key(command, egui::Key::O)) {
            self.open_document();
        }
        if context.input_mut(|input| input.consume_key(command, egui::Key::S)) {
            self.save(false);
        }
    }

    fn show_dialogs(&mut self, context: &egui::Context) {
        if self.confirm_close {
            egui::Window::new("Unsaved changes")
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.label(format!("Save changes to {}?", self.display_name()));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() && self.save(false) {
                            self.confirm_close = false;
                            context.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Don't save").clicked() {
                            self.dirty = false;
                            self.confirm_close = false;
                            context.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_close = false;
                        }
                    });
                });
        }
        if let Some(error) = self.error.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        self.error = None;
                    }
                });
        }
    }
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
