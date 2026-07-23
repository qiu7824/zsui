#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, radio, row, scrollable, slider,
    space, text, text_input, toggler,
};
use iced::window;
use iced::{Center, Element, Fill, Size, Task, Theme};

const DOCUMENT_COMPONENTS: [&str; 26] = [
    "Stack",
    "Border",
    "Scroll",
    "Tabs",
    "List",
    "Grid",
    "Text",
    "Button",
    "ToggleButton",
    "CheckBox",
    "Toggle",
    "TextBox",
    "PasswordBox",
    "RadioButton",
    "Slider",
    "NumberBox",
    "ComboBox",
    "AutoSuggestBox",
    "CommandPalette",
    "TreeView",
    "GridView",
    "DatePicker",
    "TimePicker",
    "ColorPicker",
    "ProgressBar",
    "ProgressRing",
];
const TEMPLATES: [&str; 3] = ["销售方_税额", "日期_号码", "客户_金额"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Profile {
    Minimal,
    Full,
    Viewer,
}

#[cfg(feature = "perf-viewer")]
const PROFILE: Profile = Profile::Viewer;
#[cfg(all(not(feature = "perf-viewer"), feature = "perf-full"))]
const PROFILE: Profile = Profile::Full;
#[cfg(all(not(feature = "perf-viewer"), not(feature = "perf-full")))]
const PROFILE: Profile = Profile::Minimal;

fn main() -> iced::Result {
    let launch = LaunchOptions::from_env();
    let title = match PROFILE {
        Profile::Minimal => "UI 性能矩阵 · Minimal · Iced",
        Profile::Full => "UI 性能矩阵 · Full Native App · Iced",
        Profile::Viewer => "UI 性能矩阵 · Viewer · Iced",
    };
    iced::application(
        move || PerformanceApp::new(launch.clone()),
        PerformanceApp::update,
        PerformanceApp::view,
    )
    .title(title)
    .theme(PerformanceApp::theme)
    .window(window::Settings {
        size: Size::new(1000.0, 700.0),
        min_size: Some(Size::new(820.0, 560.0)),
        ..window::Settings::default()
    })
    .run()
}

#[derive(Clone)]
struct LaunchOptions {
    empty: bool,
    repaint: bool,
    auto_close: Option<Duration>,
    document: Option<PathBuf>,
}

impl LaunchOptions {
    fn from_env() -> Self {
        let arguments = env::args().skip(1).collect::<Vec<_>>();
        Self {
            empty: arguments
                .iter()
                .any(|argument| argument == "--benchmark-empty"),
            repaint: arguments
                .iter()
                .any(|argument| argument == "--benchmark-repaint"),
            auto_close: arguments
                .windows(2)
                .find(|pair| pair[0] == "--benchmark-seconds")
                .and_then(|pair| pair[1].parse::<u64>().ok())
                .map(Duration::from_secs),
            document: arguments
                .windows(2)
                .find(|pair| pair[0] == "--document")
                .map(|pair| PathBuf::from(&pair[1])),
        }
    }
}

struct PerformanceApp {
    launch: LaunchOptions,
    text: String,
    enabled: bool,
    progress: f32,
    template: Option<&'static str>,
    document_modified: Option<SystemTime>,
    revision: u64,
}

#[derive(Debug, Clone)]
enum Message {
    ChooseInvoice,
    TextChanged(String),
    Enabled(bool),
    Progress(f32),
    Template(&'static str),
    Poll,
    Repaint,
    Close,
}

impl PerformanceApp {
    fn new(launch: LaunchOptions) -> (Self, Task<Message>) {
        let close = launch.auto_close.map_or_else(Task::none, |duration| {
            Task::perform(async move { std::thread::sleep(duration) }, |_| {
                Message::Close
            })
        });
        let poll = if PROFILE == Profile::Viewer {
            delayed_poll()
        } else {
            Task::none()
        };
        let repaint = if launch.repaint {
            delayed_repaint()
        } else {
            Task::none()
        };
        (
            Self {
                launch,
                text: "示例销售方_28.30.pdf".to_owned(),
                enabled: true,
                progress: 68.0,
                template: Some(TEMPLATES[0]),
                document_modified: None,
                revision: 1,
            },
            Task::batch([close, poll, repaint]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChooseInvoice => self.text = "新发票_128.00.pdf".to_owned(),
            Message::TextChanged(value) => self.text = value,
            Message::Enabled(value) => self.enabled = value,
            Message::Progress(value) => self.progress = value,
            Message::Template(value) => self.template = Some(value),
            Message::Poll => {
                if let Some(path) = &self.launch.document {
                    let modified = fs::metadata(path)
                        .and_then(|metadata| metadata.modified())
                        .ok();
                    if modified.is_some() && modified != self.document_modified {
                        self.document_modified = modified;
                        self.revision = self.revision.saturating_add(1);
                    }
                }
                return delayed_poll();
            }
            Message::Repaint => return delayed_repaint(),
            Message::Close => return iced::exit(),
        }
        Task::none()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn view(&self) -> Element<'_, Message> {
        if self.launch.empty {
            return container(space::vertical()).width(Fill).height(Fill).into();
        }
        match PROFILE {
            Profile::Minimal => self.minimal(),
            Profile::Full => self.full(),
            Profile::Viewer => self.viewer(),
        }
    }

    fn minimal(&self) -> Element<'_, Message> {
        container(
            column![
                text("发票助手 / Invoice Assistant").size(28),
                text("Window + Text + Button"),
                button("选择发票 / Choose invoice").on_press(Message::ChooseInvoice),
            ]
            .spacing(16)
            .align_x(Center),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    fn full(&self) -> Element<'_, Message> {
        let navigation = container(
            column![
                text("票据工坊").size(22),
                text("FULL NATIVE APP").size(11),
                separator(),
                button("仪表盘").width(Fill),
                button("发票").width(Fill),
                button("规则").width(Fill),
                button("客户").width(Fill),
                button("设置").width(Fill),
            ]
            .spacing(8),
        )
        .width(210)
        .height(Fill)
        .padding(18)
        .style(container::rounded_box);

        let form = panel(
            column![
                text("表单 / Form").size(17),
                text_input("发票名称", &self.text).on_input(Message::TextChanged),
                checkbox(self.enabled)
                    .label("自动识别")
                    .on_toggle(Message::Enabled),
                toggler(self.enabled)
                    .label("保留原文件")
                    .on_toggle(Message::Enabled),
                radio("标准规则", 0, Some(0), |_| Message::ChooseInvoice),
                radio("自定义规则", 1, Some(0), |_| Message::ChooseInvoice),
                slider(0.0..=100.0, self.progress, Message::Progress),
                pick_list(TEMPLATES, self.template, Message::Template),
                progress_bar(0.0..=100.0, self.progress),
            ]
            .spacing(10),
        );

        let collection = panel(
            column![
                text("集合与状态 / Collections").size(17),
                list_row("1", "示例销售方_28.30.pdf"),
                list_row("2", "示例发票_16.80.pdf"),
                list_row("3", "差旅报销_230.00.pdf"),
                list_row("4", "办公用品_86.40.pdf"),
                separator(),
                row![
                    text("销售方"),
                    space::horizontal(),
                    text("金额"),
                    text("状态")
                ]
                .spacing(12),
                row![
                    text("示例公司"),
                    space::horizontal(),
                    text("28.30"),
                    text("完成")
                ]
                .spacing(12),
                row![
                    text("办公商店"),
                    space::horizontal(),
                    text("86.40"),
                    text("待核对")
                ]
                .spacing(12),
            ]
            .spacing(9),
        );

        let toolbar = row![
            text("发票处理中心").size(26),
            space::horizontal(),
            button("新建"),
            button("导入"),
            button("导出"),
        ]
        .spacing(8)
        .align_y(Center);
        let body = scrollable(
            column![
                toolbar,
                row![form, collection].spacing(14),
                row![
                    text("✓ 24 个常用控件实例已加载"),
                    space::horizontal(),
                    button("取消"),
                    button("开始重命名").on_press(Message::ChooseInvoice),
                ]
                .align_y(Center),
            ]
            .spacing(14)
            .padding(18),
        );
        row![navigation, container(body).width(Fill).height(Fill)].into()
    }

    fn viewer(&self) -> Element<'_, Message> {
        let mut components = column![text("文档组件 / Document components").size(17)].spacing(5);
        for component in DOCUMENT_COMPONENTS {
            components = components.push(text(component));
        }
        scrollable(column![
            row![
                text("UiDocument 全组件性能页 / All-component performance page").size(24),
                space::horizontal(),
                text(format!("热重载：250 ms · 修订 {}", self.revision)),
            ]
            .align_y(Center),
            panel(
                column![
                    text("固定加载当前 schema 的全部文档组件；Viewer 额外保留文件轮询、解析、验证与状态映射。"),
                    button("选择发票 / Choose invoice"),
                    text_input("发票名称", &self.text).on_input(Message::TextChanged),
                    checkbox(self.enabled)
                        .label("自动识别 / Auto detect")
                        .on_toggle(Message::Enabled),
                    toggler(self.enabled)
                        .label("固定规则 / Pin rule")
                        .on_toggle(Message::Enabled),
                    slider(0.0..=100.0, self.progress, Message::Progress),
                    progress_bar(0.0..=100.0, self.progress),
                    pick_list(TEMPLATES, self.template, Message::Template),
                    row![button("重新加载 / Reload"), button("验证文档 / Validate")].spacing(8),
                ]
                .spacing(10),
            ),
            components,
        ].spacing(12).padding(20))
        .into()
    }
}

fn delayed_poll() -> Task<Message> {
    Task::perform(
        async { std::thread::sleep(Duration::from_millis(250)) },
        |_| Message::Poll,
    )
}

fn delayed_repaint() -> Task<Message> {
    Task::perform(
        async { std::thread::sleep(Duration::from_millis(16)) },
        |_| Message::Repaint,
    )
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Fill)
        .padding(14)
        .style(container::rounded_box)
        .into()
}

fn separator<'a>() -> Element<'a, Message> {
    container(text(" ")).width(Fill).height(1).into()
}

fn list_row<'a>(index: &'a str, label: &'a str) -> Element<'a, Message> {
    row![
        text(index),
        text(label),
        space::horizontal(),
        button("移除"),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}
