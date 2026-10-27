#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../../examples/support/device_demo.rs"]
mod model;
use gpui_kit::component::{
    Theme, ThemeMode,
    button::Button,
    checkbox::Checkbox,
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState},
};
use gpui_kit::*;

struct Demo {
    state: model::State,
    query: Entity<InputState>,
    load: Entity<SliderState>,
    _subscriptions: Vec<Subscription>,
    empty: bool,
    repaint: bool,
}

impl Demo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query =
            cx.new(|cx| InputState::new(window, cx).placeholder("筛选设备 / Filter devices"));
        let load = cx.new(|_| SliderState::new().min(0.0).max(100.0).default_value(42.0));
        let input_changed = cx.subscribe_in(&query, window, |this, input, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.state
                    .update(model::Message::Query(input.read(cx).value().to_string()));
                cx.notify();
            }
        });
        let load_changed = cx.subscribe_in(&load, window, |this, _, event, _, cx| {
            if let SliderEvent::Change(value) = event {
                this.state.update(model::Message::Load(value.start()));
                cx.notify();
            }
        });
        Self {
            state: model::State::default(),
            query,
            load,
            _subscriptions: vec![input_changed, load_changed],
            empty: model::has_flag("--benchmark-empty"),
            repaint: model::has_flag("--benchmark-repaint"),
        }
    }
}

impl Render for Demo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.repaint {
            window.request_animation_frame();
        }
        let base = div()
            .size_full()
            .bg(rgb(0xf7f8fa))
            .text_color(rgb(0x20242b))
            .text_size(px(14.0));
        if self.empty {
            return base;
        }
        let filters = div()
            .flex()
            .items_center()
            .gap(px(14.0))
            .h(px(36.0))
            .child(div().flex_1().child(Input::new(&self.query)))
            .child(
                Checkbox::new("online-only")
                    .label("仅在线 / Online only")
                    .checked(self.state.online_only)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.state.update(model::Message::OnlineOnly(*value));
                        cx.notify();
                    })),
            )
            .child(
                Button::new("reset")
                    .label("重置 / Reset")
                    .w(px(120.0))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.state.update(model::Message::Reset);
                        this.query
                            .update(cx, |input, cx| input.set_value("", window, cx));
                        this.load
                            .update(cx, |slider, cx| slider.set_value(42.0, window, cx));
                        cx.notify();
                    })),
            );
        let mut content = base.flex().flex_col().p(px(24.0)).gap(px(12.0))
            .child(div().h(px(36.0)).text_size(px(26.0)).child(model::TITLE))
            .child(div().h(px(24.0)).child("6 台设备 / 6 devices     在线 4 / Online 4     平均延迟 / Average latency 20 ms"))
            .child(filters)
            .child(div().h(px(24.0)).child("设备 / Device                                      类型 / Type             状态 / Status"));
        for (index, device) in self.state.visible() {
            content = content.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(42.0))
                    .flex_shrink_0()
                    .child(div().flex_1().child(device.name))
                    .child(div().w(px(90.0)).child(device.kind))
                    .child(div().w(px(180.0)).child(model::status(device)))
                    .child(
                        Button::new(("select", index))
                            .label("选择 / Select")
                            .w(px(120.0))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.state.update(model::Message::Select(index));
                                cx.notify();
                            })),
                    ),
            );
        }
        content
            .child(div().h(px(24.0)).child(format!(
                "模拟负载 / Simulated load: {:.0}%",
                self.state.load
            )))
            .child(div().h(px(30.0)).child(Slider::new(&self.load)))
            .child(
                div()
                    .h(px(12.0))
                    .w_full()
                    .bg(rgb(0xdce1e8))
                    .rounded_sm()
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.state.load / 100.0))
                            .bg(rgb(0x0067c0))
                            .rounded_sm(),
                    ),
            )
            .child(div().h(px(24.0)).child(self.state.selection()))
    }
}

fn main() {
    application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(
                None,
                size(px(model::WIDTH as f32), px(model::HEIGHT as f32)),
                cx,
            );
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(format!("{} · GPUI Kit", model::TITLE).into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| Demo::new(window, cx)),
            )
            .expect("GPUI Kit device console");
            cx.activate(true);
        });
}
