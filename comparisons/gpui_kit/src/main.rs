#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[path = "../../../examples/support/device_demo.rs"]
mod model;
use gpui_kit::component::{
    Theme, ThemeMode,
    button::{Button, ButtonVariants},
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
    _ticker: Option<Task<()>>,
}
impl Demo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("搜索名称或类型"));
        let load = cx.new(|_| SliderState::new().min(0.0).max(100.0).default_value(42.0));
        let input = cx.subscribe_in(&query, window, |this, input, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.state
                    .update(model::Message::Query(input.read(cx).value().to_string()));
                cx.notify();
            }
        });
        let slider = cx.subscribe_in(&load, window, |this, _, event, _, cx| {
            if let SliderEvent::Change(value) = event {
                this.state.update(model::Message::Load(value.start()));
                cx.notify();
            }
        });
        let ticker = if model::has_flag("--benchmark-repaint") {
            let started = std::time::Instant::now();
            Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(16))
                        .await;
                    if this
                        .update_in(cx, |this, window, cx| {
                            this.state.load = model::benchmark_load(started);
                            let value = this.state.load;
                            this.load
                                .update(cx, |slider, cx| slider.set_value(value, window, cx));
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }))
        } else {
            None
        };
        Self {
            state: model::State::default(),
            query,
            load,
            _subscriptions: vec![input, slider],
            empty: model::has_flag("--benchmark-empty"),
            _ticker: ticker,
        }
    }
}
fn caption(value: impl Into<String>) -> Div {
    div()
        .text_size(px(12.0))
        .text_color(rgb(0x616161))
        .child(value.into())
}
fn strong(value: impl Into<String>) -> Div {
    div().font_weight(FontWeight::SEMIBOLD).child(value.into())
}
fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .p(px(18.0))
        .rounded_xl()
        .bg(rgb(0xffffff))
}
fn stat(title: &str, value: String, unit: &str, accent: bool) -> Div {
    card()
        .p(px(16.0))
        .flex_1()
        .min_w(px(0.0))
        .h(px(96.0))
        .gap(px(5.0))
        .child(caption(title))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(strong(value).text_size(px(28.0)).text_color(rgb(if accent {
                    0x0067c0
                } else {
                    0x1b1b1b
                })))
                .child(caption(unit)),
        )
}
impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .size_full()
            .bg(rgb(0xf3f3f3))
            .text_color(rgb(0x1b1b1b))
            .text_size(px(14.0));
        if self.empty {
            return base;
        }
        let selected = model::DEVICES[self.state.selected];
        let name = selected.name.split(" / ").next().unwrap();
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(60.0))
            .flex_shrink_0()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .child(strong("设备控制台").text_size(px(28.0)))
                    .child(caption("查看连接状态，调整模拟负载")),
            )
            .child(caption("DEVICE CONSOLE  /  本地演示"));
        let stats = div()
            .flex()
            .gap(px(16.0))
            .h(px(96.0))
            .flex_shrink_0()
            .child(stat("在线设备", "04".into(), "/ 6 台", true))
            .child(stat("平均响应", "20".into(), "ms", false))
            .child(stat(
                "模拟负载",
                format!("{:.0}", self.state.load),
                "%",
                false,
            ));
        let mut list = card()
            .flex_1()
            .min_w(px(0.0))
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.0))
                    .child(strong("设备列表"))
                    .child(caption(format!("{} 台设备", self.state.visible().len()))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(34.0))
                    .child(div().flex_1().child(Input::new(&self.query)))
                    .child(
                        Checkbox::new("online-only")
                            .label("仅在线")
                            .checked(self.state.online_only)
                            .on_change(cx.listener(|this, value, _, cx| {
                                this.state.update(model::Message::OnlineOnly(*value));
                                cx.notify();
                            })),
                    ),
            );
        let mut rows = div().flex().flex_col().gap(px(5.0));
        for (index, device) in self.state.visible() {
            let chosen = index == self.state.selected;
            rows =
                rows.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .h(px(48.0))
                        .flex_shrink_0()
                        .p(px(5.0))
                        .rounded_lg()
                        .bg(rgb(if chosen { 0xf3f3f3 } else { 0xffffff }))
                        .child(
                            div()
                                .w(px(3.0))
                                .h(px(28.0))
                                .bg(rgb(if chosen { 0x0067c0 } else { 0xffffff }))
                                .rounded_sm(),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .flex_1()
                                .child(
                                    strong(device.name.split(" / ").next().unwrap())
                                        .text_color(rgb(if chosen { 0x0067c0 } else { 0x1b1b1b })),
                                )
                                .child(caption(device.kind)),
                        )
                        .child(
                            caption(if device.online {
                                "● 在线"
                            } else {
                                "○ 离线"
                            })
                            .w(px(66.0))
                            .text_color(rgb(if device.online { 0x0f7b0f } else { 0x616161 })),
                        )
                        .child(
                            Button::new(("select", index))
                                .label("›")
                                .w(px(28.0))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.state.update(model::Message::Select(index));
                                    cx.notify();
                                })),
                        ),
                );
        }
        list = list.child(rows);
        let details = card()
            .w(px(260.0))
            .flex_shrink_0()
            .gap(px(10.0))
            .child(caption(format!("DEVICE {:02}", self.state.selected + 1)))
            .child(strong(name).text_size(px(20.0)))
            .child(caption(selected.kind))
            .child(div().h(px(8.0)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .h(px(24.0))
                    .child(caption("连接状态"))
                    .child(
                        strong(if selected.online { "在线" } else { "离线" })
                            .text_color(rgb(if selected.online { 0x0f7b0f } else { 0x616161 })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .h(px(24.0))
                    .child(caption("响应时间"))
                    .child(strong(if selected.online {
                        format!("{} ms", selected.latency)
                    } else {
                        "—".into()
                    })),
            )
            .child(div().h(px(1.0)).bg(rgb(0xd1d1d1)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .h(px(28.0))
                    .child(strong("负载模拟"))
                    .child(strong(format!("{:.0}%", self.state.load)).text_color(rgb(0x0067c0))),
            )
            .child(
                div()
                    .h(px(28.0))
                    .w_full()
                    .child(Slider::new(&self.load).bg(rgb(0x0067c0))),
            )
            .child(
                div()
                    .h(px(4.0))
                    .w_full()
                    .bg(rgb(0xd1d1d1))
                    .rounded_sm()
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.state.load / 100.0))
                            .bg(rgb(0x0067c0))
                            .rounded_sm(),
                    ),
            )
            .child(caption("拖动滑块调整演示数据"))
            .child(div().flex_1())
            .child(
                Button::new("reset")
                    .primary()
                    .label("恢复默认")
                    .w_full()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.state.update(model::Message::Reset);
                        this.query
                            .update(cx, |input, cx| input.set_value("", window, cx));
                        this.load
                            .update(cx, |slider, cx| slider.set_value(42.0, window, cx));
                        cx.notify();
                    })),
            );
        base.flex()
            .flex_col()
            .p(px(24.0))
            .gap(px(16.0))
            .child(header)
            .child(stats)
            .child(
                div()
                    .flex()
                    .gap(px(16.0))
                    .h(px(440.0))
                    .flex_shrink_0()
                    .child(list)
                    .child(details),
            )
    }
}
fn main() {
    application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);
            Theme::update(cx, |theme| {
                theme.colors.primary = rgb(0x0067c0).into();
                theme.colors.primary_foreground = rgb(0xffffff).into();
                theme.colors.primary_hover = rgb(0x005fae).into();
                theme.colors.primary_active = rgb(0x00559f).into();
                theme.colors.ring = rgb(0x0067c0).into();
            });
            cx.on_window_closed(|cx, _| {
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
                        title: Some("设备控制台 · GPUI Kit".into()),
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
