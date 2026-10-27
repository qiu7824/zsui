#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[path = "support/device_demo.rs"]
mod model;

use model::{Message, State};
use zsui::{
    checkbox, column, grid, icon_button, native_window, primary_button, progress_bar, row, slider,
    spacer, styled_text, textbox, AppCx, ColorRole, Dp, ProgressRange, SemanticTextStyle,
    SliderRange, TextRole, TextWeight, ThemeColorToken as Bg, ViewAlign, ViewNode, ZsGridCell,
    ZsGridTrack, ZsIcon, ZsuiThemeMode,
};

fn label(value: impl Into<String>, role: TextRole, color: ColorRole) -> ViewNode<Message> {
    styled_text(
        value,
        SemanticTextStyle {
            color,
            ..SemanticTextStyle::for_role(role)
        },
    )
}
fn muted(value: impl Into<String>) -> ViewNode<Message> {
    label(value, TextRole::Caption, ColorRole::SecondaryText)
}
fn strong(value: impl Into<String>, color: ColorRole) -> ViewNode<Message> {
    styled_text(
        value,
        SemanticTextStyle {
            color,
            weight: TextWeight::Semibold,
            ..SemanticTextStyle::body()
        },
    )
}
fn card(content: ViewNode<Message>) -> ViewNode<Message> {
    column([content])
        .padding(Dp::new(18.0))
        .radius(Dp::new(12.0))
        .bg(Bg::SurfaceRaised)
}
fn stat(title: &str, value: String, unit: &str, accent: bool) -> ViewNode<Message> {
    card(
        column([
            muted(title),
            row([
                label(
                    value,
                    TextRole::Title,
                    if accent {
                        ColorRole::Accent
                    } else {
                        ColorRole::PrimaryText
                    },
                ),
                muted(unit).padding(Dp::new(4.0)),
            ])
            .gap(Dp::new(6.0))
            .align(ViewAlign::Center),
        ])
        .gap(Dp::new(5.0)),
    )
    .padding(Dp::new(16.0))
    .height(Dp::new(96.0))
    .flex(1.0)
}

fn view(state: &State) -> ViewNode<Message> {
    let selected = model::DEVICES[state.selected];
    let selected_name = selected.name.split(" / ").next().unwrap_or(selected.name);
    let visible = state.visible();
    let header = row([
        column([
            label("设备控制台", TextRole::Title, ColorRole::PrimaryText),
            muted("查看连接状态，调整模拟负载"),
        ])
        .gap(Dp::new(5.0))
        .flex(1.0),
        muted("DEVICE CONSOLE  /  本地演示").width(Dp::new(216.0)),
    ])
    .align(ViewAlign::Center)
    .height(Dp::new(60.0))
    .flex(0.0);
    let stats = grid(
        [ZsGridTrack::FLEX; 3],
        [ZsGridTrack::fixed(Dp::new(96.0))],
        [
            ZsGridCell::new(0, 0, stat("在线设备", "04".into(), "/ 6 台", true)),
            ZsGridCell::new(0, 1, stat("平均响应", "20".into(), "ms", false)),
            ZsGridCell::new(
                0,
                2,
                stat("模拟负载", format!("{:.0}", state.load), "%", false),
            ),
        ],
    )
    .gap(Dp::new(16.0))
    .height(Dp::new(96.0))
    .flex(0.0);

    let mut device_rows = Vec::new();
    for (index, device) in &visible {
        let is_selected = *index == state.selected;
        let name = device.name.split(" / ").next().unwrap_or(device.name);
        let status_color = if device.online {
            ColorRole::Success
        } else {
            ColorRole::SecondaryText
        };
        let item = row([
            spacer()
                .width(Dp::new(3.0))
                .height(Dp::new(28.0))
                .bg(if is_selected {
                    Bg::Accent
                } else {
                    Bg::SurfaceRaised
                })
                .radius(Dp::new(2.0)),
            column([
                strong(
                    name,
                    if is_selected {
                        ColorRole::Accent
                    } else {
                        ColorRole::PrimaryText
                    },
                ),
                muted(device.kind),
            ])
            .gap(Dp::new(2.0))
            .flex(1.0),
            label(
                if device.online {
                    "● 在线"
                } else {
                    "○ 离线"
                },
                TextRole::Caption,
                status_color,
            )
            .width(Dp::new(66.0)),
            icon_button("查看设备", ZsIcon::ChevronRight).on_click(Message::Select(*index)),
        ])
        .gap(Dp::new(12.0))
        .align(ViewAlign::Center)
        .padding(Dp::new(5.0))
        .radius(Dp::new(8.0))
        .height(Dp::new(48.0))
        .flex(0.0)
        .bg(if is_selected {
            Bg::Surface
        } else {
            Bg::SurfaceRaised
        });
        device_rows.push(item);
    }
    if device_rows.is_empty() {
        device_rows.push(muted("没有匹配的设备").height(Dp::new(48.0)));
    }
    let list = card(
        column([
            row([
                strong("设备列表", ColorRole::PrimaryText).flex(1.0),
                muted(format!("{} 台设备", visible.len())),
            ])
            .height(Dp::new(24.0))
            .flex(0.0),
            row([
                textbox(state.query.clone())
                    .placeholder("搜索名称或类型")
                    .on_change(Message::Query)
                    .flex(1.0),
                checkbox("仅在线", state.online_only).on_toggle(Message::OnlineOnly),
            ])
            .gap(Dp::new(12.0))
            .height(Dp::new(34.0))
            .flex(0.0),
            column(device_rows).gap(Dp::new(5.0)).flex(1.0),
        ])
        .gap(Dp::new(12.0)),
    )
    .flex(1.0);

    let details = card(
        column([
            muted(format!("DEVICE {:02}", state.selected + 1)),
            label(selected_name, TextRole::Subtitle, ColorRole::PrimaryText),
            muted(selected.kind),
            spacer().height(Dp::new(8.0)),
            row([
                muted("连接状态").flex(1.0),
                strong(
                    if selected.online { "在线" } else { "离线" },
                    if selected.online {
                        ColorRole::Success
                    } else {
                        ColorRole::SecondaryText
                    },
                ),
            ])
            .height(Dp::new(24.0)),
            row([
                muted("响应时间").flex(1.0),
                strong(
                    if selected.online {
                        format!("{} ms", selected.latency)
                    } else {
                        "—".into()
                    },
                    ColorRole::PrimaryText,
                ),
            ])
            .height(Dp::new(24.0)),
            spacer().height(Dp::new(1.0)).bg(Bg::Border),
            row([
                strong("负载模拟", ColorRole::PrimaryText).flex(1.0),
                strong(format!("{:.0}%", state.load), ColorRole::Accent),
            ])
            .height(Dp::new(28.0)),
            row([slider(state.load, SliderRange::new(0.0, 100.0))
                .on_slide(Message::Load)
                .width(Dp::new(224.0))
                .flex(1.0)])
            .height(Dp::new(28.0))
            .flex(0.0),
            progress_bar(state.load, ProgressRange::new(0.0, 100.0)).height(Dp::new(8.0)),
            muted("拖动滑块调整演示数据"),
            spacer().flex(1.0),
            row([primary_button("恢复默认")
                .on_click(Message::Reset)
                .flex(1.0)])
            .height(Dp::new(36.0)),
        ])
        .gap(Dp::new(10.0)),
    )
    .width(Dp::new(260.0))
    .flex(0.0);

    column([
        header,
        stats,
        row([list, details])
            .gap(Dp::new(16.0))
            .height(Dp::new(440.0))
            .flex(0.0),
    ])
    .padding(Dp::new(24.0))
    .gap(Dp::new(16.0))
    .bg(Bg::Surface)
    .theme_mode(ZsuiThemeMode::Light)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    let repaint = model::has_flag("--benchmark-repaint");
    let invalidation = zsui::UiInvalidationHandle::new();
    let stop = Arc::new(AtomicBool::new(false));
    let worker = if repaint {
        let wake = invalidation.clone();
        let stop = stop.clone();
        Some(std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(16));
                wake.request_rebuild();
            }
        }))
    } else {
        None
    };
    let started = std::time::Instant::now();
    let builder = native_window("设备控制台 · ZSUI").size(model::WIDTH, model::HEIGHT);
    if model::has_flag("--benchmark-empty") {
        let result = builder.run();
        stop.store(true, Ordering::Release);
        if let Some(worker) = worker {
            let _ = worker.join();
        }
        result?;
    } else {
        let result = builder
            .invalidation_handle(invalidation)
            .stateful_view(
                State::default(),
                move |state| {
                    let mut visible = state.clone();
                    if repaint {
                        visible.load = model::benchmark_load(started);
                    }
                    view(&visible)
                },
                |state, message, _: &mut AppCx| state.update(message),
            )
            .run();
        stop.store(true, Ordering::Release);
        if let Some(worker) = worker {
            let _ = worker.join();
        }
        result?;
    }
    Ok(())
}
