#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use iced::widget::{
    button, checkbox, column, container, progress_bar, row, slider, text, text_input,
};
use iced::{Element, Fill, Size, Task, Theme};
use model::{Message, State};

struct Demo {
    state: State,
    empty: bool,
    repaint: bool,
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
        let repaint = matches!(message, Message::Repaint) && self.repaint;
        self.state.update(message);
        if repaint {
            repaint_task()
        } else {
            Task::none()
        }
    }
    fn view(&self) -> Element<'_, Message> {
        if self.empty {
            return container(text("")).width(Fill).height(Fill).into();
        }
        let mut content = column![
            text(model::TITLE).size(26),
            text("6 台设备 / 6 devices     在线 4 / Online 4     平均延迟 / Average latency 20 ms"),
            row![
                text_input("筛选设备 / Filter devices", &self.state.query)
                    .on_input(Message::Query)
                    .width(Fill),
                checkbox(self.state.online_only)
                    .label("仅在线 / Online only")
                    .on_toggle(Message::OnlineOnly),
                button("重置 / Reset").on_press(Message::Reset),
            ]
            .spacing(14)
            .align_y(iced::Center),
            row![
                text("设备 / Device").width(Fill),
                text("类型 / Type").width(90),
                text("状态 / Status").width(180),
                text("").width(120)
            ]
            .spacing(12)
            .height(24),
        ]
        .spacing(12);
        for (index, device) in self.state.visible() {
            content = content.push(
                row![
                    text(device.name).width(Fill),
                    text(device.kind).width(90),
                    text(model::status(device)).width(180),
                    button("选择 / Select")
                        .on_press(Message::Select(index))
                        .width(120),
                ]
                .height(42)
                .spacing(12)
                .align_y(iced::Center),
            );
        }
        content = content
            .push(text(format!(
                "模拟负载 / Simulated load: {:.0}%",
                self.state.load
            )))
            .push(slider(0.0..=100.0, self.state.load, Message::Load))
            .push(progress_bar(0.0..=100.0, self.state.load).girth(12))
            .push(text(self.state.selection()));
        container(content)
            .padding(24)
            .width(Fill)
            .height(Fill)
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
    .title("设备控制台 / Device Console · Iced")
    .theme(Demo::theme)
    .window(iced::window::Settings {
        size: Size::new(model::WIDTH as f32, model::HEIGHT as f32),
        ..Default::default()
    })
    .run()
}
