#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use iced::widget::{
    button, checkbox, column, container, progress_bar, row, slider, space, text, text_input,
};
use iced::{Color, Element, Fill, Size, Task, Theme};
use model::{Message, State};
const BLUE: Color = Color::from_rgb(0.0, 103.0 / 255.0, 192.0 / 255.0);
const MUTED: Color = Color::from_rgb(97.0 / 255.0, 97.0 / 255.0, 97.0 / 255.0);
fn bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }
}
fn caption(value: impl Into<String>) -> iced::widget::Text<'static> {
    text(value.into()).size(12).color(MUTED)
}
fn white_panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::WHITE.into()),
        border: iced::Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn selected_panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(243, 243, 243).into()),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.background = Some(BLUE.into());
    style.text_color = Color::WHITE;
    style
}
fn stat(title: &str, value: String, unit: &str, accent: bool) -> Element<'static, Message> {
    container(
        column![
            caption(title),
            row![
                text(value)
                    .size(28)
                    .font(bold())
                    .color(if accent { BLUE } else { Color::BLACK }),
                caption(unit)
            ]
            .spacing(6)
            .align_y(iced::Center)
        ]
        .spacing(5),
    )
    .padding(16)
    .width(Fill)
    .height(96)
    .style(white_panel)
    .into()
}
struct Demo {
    state: State,
    empty: bool,
    repaint: bool,
    started: std::time::Instant,
}
fn repaint_task() -> Task<Message> {
    Task::perform(
        async { std::thread::sleep(std::time::Duration::from_millis(16)) },
        |_| Message::Repaint,
    )
}
impl Demo {
    fn theme(&self) -> Theme {
        Theme::Light
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        let again = matches!(message, Message::Repaint) && self.repaint;
        self.state.update(message);
        if again {
            self.state.load = model::benchmark_load(self.started);
        }
        if again { repaint_task() } else { Task::none() }
    }
    fn view(&self) -> Element<'_, Message> {
        if self.empty {
            return container(text("")).width(Fill).height(Fill).into();
        }
        let selected = model::DEVICES[self.state.selected];
        let header = row![
            column![
                text("设备控制台").size(28).font(bold()),
                caption("查看连接状态，调整模拟负载")
            ]
            .spacing(5),
            space::horizontal(),
            caption("DEVICE CONSOLE  /  本地演示")
        ]
        .height(60)
        .align_y(iced::Center);
        let stats = row![
            stat("在线设备", "04".into(), "/ 6 台", true),
            stat("平均响应", "20".into(), "ms", false),
            stat("模拟负载", format!("{:.0}", self.state.load), "%", false)
        ]
        .spacing(16);
        let mut rows = column![].spacing(5);
        for (index, device) in self.state.visible() {
            let chosen = index == self.state.selected;
            let marker =
                container(space::horizontal())
                    .width(3)
                    .height(28)
                    .style(move |_: &Theme| container::Style {
                        background: Some((if chosen { BLUE } else { Color::WHITE }).into()),
                        ..Default::default()
                    });
            let content = row![
                marker,
                column![
                    text(device.name.split(" / ").next().unwrap())
                        .font(bold())
                        .color(if chosen { BLUE } else { Color::BLACK }),
                    caption(device.kind)
                ]
                .spacing(2)
                .width(Fill),
                text(if device.online {
                    "● 在线"
                } else {
                    "○ 离线"
                })
                .size(12)
                .color(if device.online {
                    Color::from_rgb8(15, 123, 15)
                } else {
                    MUTED
                })
                .width(66),
                button("›")
                    .on_press(Message::Select(index))
                    .width(28)
                    .height(28)
                    .padding(0)
            ]
            .spacing(12)
            .align_y(iced::Center);
            let item = container(content).padding([5, 7]).height(48).width(Fill);
            rows = rows.push(if chosen {
                item.style(selected_panel)
            } else {
                item
            });
        }
        let list = container(
            column![
                row![
                    text("设备列表").font(bold()),
                    space::horizontal(),
                    caption(format!("{} 台设备", self.state.visible().len()))
                ]
                .height(24),
                row![
                    text_input("搜索名称或类型", &self.state.query)
                        .on_input(Message::Query)
                        .width(Fill),
                    checkbox(self.state.online_only)
                        .label("仅在线")
                        .on_toggle(Message::OnlineOnly)
                ]
                .spacing(12)
                .height(34)
                .align_y(iced::Center),
                rows,
            ]
            .spacing(12),
        )
        .padding(18)
        .width(Fill)
        .height(440)
        .style(white_panel);
        let details = container(
            column![
                caption(format!("DEVICE {:02}", self.state.selected + 1)),
                text(selected.name.split(" / ").next().unwrap())
                    .size(20)
                    .font(bold()),
                caption(selected.kind),
                space::vertical().height(8),
                row![
                    caption("连接状态"),
                    space::horizontal(),
                    text(if selected.online { "在线" } else { "离线" })
                        .font(bold())
                        .color(if selected.online {
                            Color::from_rgb8(15, 123, 15)
                        } else {
                            MUTED
                        })
                ]
                .height(24),
                row![
                    caption("响应时间"),
                    space::horizontal(),
                    text(if selected.online {
                        format!("{} ms", selected.latency)
                    } else {
                        "—".into()
                    })
                    .font(bold())
                ]
                .height(24),
                container(space::horizontal())
                    .height(1)
                    .width(Fill)
                    .style(|_: &Theme| container::Style {
                        background: Some(Color::from_rgb8(209, 209, 209).into()),
                        ..Default::default()
                    }),
                row![
                    text("负载模拟").font(bold()),
                    space::horizontal(),
                    text(format!("{:.0}%", self.state.load)).color(BLUE)
                ]
                .height(28),
                slider(0.0..=100.0, self.state.load, Message::Load),
                progress_bar(0.0..=100.0, self.state.load).girth(4),
                caption("拖动滑块调整演示数据"),
                space::vertical(),
                button("恢复默认")
                    .on_press(Message::Reset)
                    .width(Fill)
                    .height(34)
                    .style(primary),
            ]
            .spacing(10)
            .height(Fill),
        )
        .padding(18)
        .width(260)
        .height(440)
        .style(white_panel);
        container(column![header, stats, row![list, details].spacing(16)].spacing(16))
            .padding(24)
            .width(Fill)
            .height(Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgb8(243, 243, 243).into()),
                ..Default::default()
            })
            .into()
    }
}
fn main() -> iced::Result {
    iced::application(
        || {
            let repaint = model::has_flag("--benchmark-repaint");
            (
                Demo {
                    state: State::default(),
                    empty: model::has_flag("--benchmark-empty"),
                    repaint,
                    started: std::time::Instant::now(),
                },
                if repaint {
                    repaint_task()
                } else {
                    Task::none()
                },
            )
        },
        Demo::update,
        Demo::view,
    )
    .title("设备控制台 · Iced")
    .theme(Demo::theme)
    .window(iced::window::Settings {
        size: Size::new(model::WIDTH as f32, model::HEIGHT as f32),
        ..Default::default()
    })
    .run()
}
