#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf, time::Duration};

use iced::widget::{button, column, container, row, space, text, text_editor, toggler};
use iced::window;
use iced::{Center, Element, Fill, Size, Task, Theme};

#[path = "../../common/memory_probe.rs"]
mod memory_probe;

fn main() -> iced::Result {
    let launch = LaunchOptions::from_env();
    let icon =
        window::icon::from_file_data(include_bytes!("../../../assets/notepad/notepad.png"), None)
            .ok();

    iced::application(
        move || Notepad::new(launch.clone()),
        Notepad::update,
        Notepad::view,
    )
    .title(Notepad::title)
    .theme(Notepad::theme)
    .window(window::Settings {
        size: Size::new(900.0, 620.0),
        min_size: Some(Size::new(520.0, 360.0)),
        icon,
        ..window::Settings::default()
    })
    .run()
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

struct Notepad {
    path: Option<PathBuf>,
    content: text_editor::Content,
    dirty: bool,
    word_wrap: bool,
    show_status: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    Save,
    SaveAs,
    ToggleWrap(bool),
    ToggleStatus(bool),
    SampleMemory(PathBuf),
    Close,
}

impl Notepad {
    fn new(launch: LaunchOptions) -> (Self, Task<Message>) {
        let (path, text, error) = match launch.open_path {
            Some(path) => match read_text(&path) {
                Ok(text) => (Some(path), text, None),
                Err(error) => (None, String::new(), Some(error)),
            },
            None => (
                None,
                "Iced Notepad baseline\n\nA typed Elm-style text editing benchmark.\n".to_string(),
                None,
            ),
        };
        let mut tasks = Vec::new();
        if let Some(duration) = launch.auto_close {
            tasks.push(Task::perform(
                async move { std::thread::sleep(duration) },
                |_| Message::Close,
            ));
        }
        if let Some(path) = launch.memory_report {
            tasks.push(Task::perform(
                async move { std::thread::sleep(launch.sample_after) },
                move |_| Message::SampleMemory(path.clone()),
            ));
        }

        (
            Self {
                path,
                content: text_editor::Content::with_text(&text),
                dirty: false,
                word_wrap: true,
                show_status: true,
                error,
            },
            Task::batch(tasks),
        )
    }

    fn title(&self) -> String {
        format!(
            "{}{} - Iced Notepad baseline",
            if self.dirty { "*" } else { "" },
            self.display_name()
        )
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.dirty |= action.is_edit();
                self.content.perform(action);
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                self.dirty = false;
                self.error = None;
            }
            Message::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Text", &["txt", "md", "log"])
                    .pick_file()
                {
                    match read_text(&path) {
                        Ok(text) => {
                            self.path = Some(path);
                            self.content = text_editor::Content::with_text(&text);
                            self.dirty = false;
                            self.error = None;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
            }
            Message::Save => self.save(false),
            Message::SaveAs => self.save(true),
            Message::ToggleWrap(value) => self.word_wrap = value,
            Message::ToggleStatus(value) => self.show_status = value,
            Message::SampleMemory(path) => {
                if let Err(error) =
                    memory_probe::write_report(&path, "iced", "notepad", "first_frame_idle")
                {
                    self.error = Some(format!("memory report failed: {error}"));
                }
            }
            Message::Close => return iced::exit(),
        }

        Task::none()
    }

    fn save(&mut self, force_picker: bool) {
        let path = if force_picker || self.path.is_none() {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Text", &["txt", "md", "log"])
                .set_file_name(self.display_name())
                .save_file()
            else {
                return;
            };
            self.path = Some(path.clone());
            path
        } else {
            self.path.clone().expect("path checked above")
        };

        match fs::write(path, self.content.text().as_bytes()) {
            Ok(()) => {
                self.dirty = false;
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            button("New").on_press(Message::New),
            button("Open...").on_press(Message::Open),
            button("Save").on_press(Message::Save),
            button("Save as...").on_press(Message::SaveAs),
            space::horizontal(),
            toggler(self.word_wrap)
                .label("Word wrap")
                .on_toggle(Message::ToggleWrap),
            toggler(self.show_status)
                .label("Status")
                .on_toggle(Message::ToggleStatus),
        ]
        .spacing(8)
        .align_y(Center);

        let editor = text_editor(&self.content)
            .height(Fill)
            .on_action(Message::Edit)
            .wrapping(if self.word_wrap {
                text::Wrapping::Word
            } else {
                text::Wrapping::None
            });

        let mut body = column![toolbar, container(editor).height(Fill)]
            .spacing(10)
            .padding(12);

        if let Some(error) = &self.error {
            body = body.push(text(format!("Error: {error}")));
        }
        if self.show_status {
            let cursor = self.content.cursor();
            body = body.push(
                row![
                    text(format!(
                        "Ln {}, Col {}",
                        cursor.position.line + 1,
                        cursor.position.column + 1
                    )),
                    space::horizontal(),
                    text(format!("{} chars", self.content.text().chars().count())),
                    text("UTF-8"),
                ]
                .spacing(16)
                .align_y(Center),
            );
        }

        container(body).width(Fill).height(Fill).into()
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
