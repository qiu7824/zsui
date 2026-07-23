#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, time::Duration};

use iced::widget::{button, column, container, row, space, text};
use iced::window;
use iced::{Center, Element, Fill, Size, Task, Theme};

fn main() -> iced::Result {
    let launch = LaunchOptions::from_env();
    iced::application(
        move || InvoiceTool::new(launch.clone()),
        InvoiceTool::update,
        InvoiceTool::view,
    )
    .title("发票工作台 · Iced")
    .theme(InvoiceTool::theme)
    .window(window::Settings {
        size: Size::new(1000.0, 700.0),
        min_size: Some(Size::new(820.0, 560.0)),
        ..window::Settings::default()
    })
    .run()
}

#[derive(Clone)]
struct LaunchOptions {
    auto_close: Option<Duration>,
    empty: bool,
    repaint: bool,
}

impl LaunchOptions {
    fn from_env() -> Self {
        let args = env::args().skip(1).collect::<Vec<_>>();
        let auto_close = args
            .windows(2)
            .find(|pair| pair[0] == "--benchmark-seconds")
            .and_then(|pair| pair[1].parse::<u64>().ok())
            .map(Duration::from_secs);
        let empty = args.iter().any(|argument| argument == "--benchmark-empty");
        let repaint = args
            .iter()
            .any(|argument| argument == "--benchmark-repaint");
        Self {
            auto_close,
            empty,
            repaint,
        }
    }
}

struct InvoiceTool {
    selected: usize,
    file_count: usize,
    status: String,
    empty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Select(usize),
    Add,
    Remove,
    Rename,
    Repaint,
    Close,
}

impl InvoiceTool {
    fn new(launch: LaunchOptions) -> (Self, Task<Message>) {
        let close = launch.auto_close.map_or_else(Task::none, |duration| {
            Task::perform(async move { std::thread::sleep(duration) }, |_| {
                Message::Close
            })
        });
        let repaint = if launch.repaint {
            delayed_repaint()
        } else {
            Task::none()
        };
        (
            Self {
                selected: 2,
                file_count: 2,
                status: "字段识别完成，可以开始重命名".to_string(),
                empty: launch.empty,
            },
            Task::batch([close, repaint]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Select(index) => self.selected = index,
            Message::Add => {
                self.file_count += 1;
                self.status = "已添加一张待处理发票".to_string();
            }
            Message::Remove => {
                self.file_count = self.file_count.saturating_sub(1);
                self.status = "已移除一张发票".to_string();
            }
            Message::Rename => {
                self.status = format!("已完成 {} 张发票重命名", self.file_count);
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
        if self.empty {
            return container(space::vertical()).width(Fill).height(Fill).into();
        }
        let labels = [
            "发票合并打印",
            "发票信息提取",
            "发票重命名",
            "发票划分文件夹",
        ];
        let mut navigation = column![
            text("票据工坊").size(22),
            text("INVOICE WORKBENCH").size(11),
            space::vertical().height(20),
        ]
        .spacing(8);
        for (index, label) in labels.into_iter().enumerate() {
            let marker = if self.selected == index { "●" } else { "○" };
            navigation = navigation.push(
                button(text(format!("{marker}   {label}")))
                    .width(Fill)
                    .height(44)
                    .on_press(Message::Select(index)),
            );
        }
        navigation = navigation
            .push(space::vertical())
            .push(text("本地处理 · 文件不上传").size(11));
        let navigation = container(navigation)
            .width(230)
            .height(Fill)
            .padding(20)
            .style(container::rounded_box);

        let heading = row![
            column![
                text("发票重命名").size(28),
                text("按发票字段批量生成清晰文件名").size(13),
            ],
            space::horizontal(),
            button("＋ 添加发票").on_press(Message::Add),
        ]
        .align_y(Center);

        let rule = panel(
            row![
                column![
                    text("自定义重命名规则").size(15),
                    text("销售方名称_税额").size(19),
                    text("示例：示例销售方_28.30.pdf").size(12),
                ]
                .spacing(5),
                space::horizontal(),
                text("✓ 已启用").size(14),
            ]
            .align_y(Center),
        );

        let mut files = column![
            row![
                text(format!("待处理发票 · {}", self.file_count)).size(16),
                space::horizontal(),
                text("识别状态：完成").size(13),
            ]
            .align_y(Center)
        ]
        .spacing(8);
        if self.file_count > 0 {
            files = files.push(file_panel(
                "示例销售方_28.30.pdf",
                "原文件：20260714_001.pdf · 电子发票",
            ));
        }
        if self.file_count > 1 {
            files = files.push(file_panel(
                "示例发票_16.80.pdf",
                "原文件：扫描件_0714.pdf · 已识别销售方和税额",
            ));
        }

        let output = panel(
            row![
                column![
                    text("输出设置").size(15),
                    text("原文件旁的“已重命名”目录 · 保留原始文件").size(12),
                ]
                .spacing(5),
                space::horizontal(),
                button("选择文件夹"),
            ]
            .align_y(Center),
        );

        let confirmation = panel(
            column![
                text("输出确认").size(15),
                text("将重命名 2 张发票并保留原始文件。").size(12),
            ]
            .spacing(5),
        );

        let footer = row![
            text(&self.status).size(13),
            space::horizontal(),
            button("开始重命名")
                .on_press(Message::Rename)
                .padding([10, 24]),
        ]
        .align_y(Center);

        let content = column![
            heading,
            rule,
            files,
            output,
            confirmation,
            space::vertical(),
            footer
        ]
        .spacing(16)
        .padding(22)
        .height(Fill);

        container(row![
            navigation,
            container(content).width(Fill).height(Fill)
        ])
        .width(Fill)
        .height(Fill)
        .into()
    }
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
        .padding(16)
        .style(container::rounded_box)
        .into()
}

fn file_panel(name: &str, source: &str) -> Element<'static, Message> {
    panel(
        row![
            text("PDF").size(13),
            column![
                text(name.to_string()).size(15),
                text(source.to_string()).size(12)
            ]
            .spacing(4),
            space::horizontal(),
            button("移除").on_press(Message::Remove),
        ]
        .spacing(14)
        .align_y(Center),
    )
}
