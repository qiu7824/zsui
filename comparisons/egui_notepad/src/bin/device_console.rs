#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use eframe::egui::{self, Color32, RichText};
use model::{Message, State};

const BLUE: Color32 = Color32::from_rgb(0, 103, 192);
const MUTED: Color32 = Color32::from_rgb(97, 97, 97);
struct Demo {
    state: State,
    empty: bool,
    repaint: bool,
    started: std::time::Instant,
}
fn caption(text: impl Into<String>) -> RichText {
    RichText::new(text).size(12.0).color(MUTED)
}
fn panel(ui: &mut egui::Ui, width: f32, height: f32, draw: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            egui::Frame::new()
                .fill(Color32::WHITE)
                .corner_radius(12.0)
                .inner_margin(18.0)
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(width - 36.0, height - 36.0));
                    ui.set_max_size(egui::vec2(width - 36.0, height - 36.0));
                    draw(ui);
                });
        },
    );
}
impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if self.repaint {
            self.state.load = model::benchmark_load(self.started);
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_gray(243));
        if self.empty {
            return;
        }
        egui::Frame::new().inner_margin(24.0).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 16.0);
            let width = ui.available_width();
            ui.allocate_ui_with_layout(
                egui::vec2(width, 60.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 5.0;
                        ui.label(RichText::new("设备控制台").size(28.0).strong());
                        ui.label(caption("查看连接状态，调整模拟负载"));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(caption("DEVICE CONSOLE  /  本地演示"));
                    });
                },
            );
            ui.horizontal(|ui| {
                for (title, value, unit, accent) in [
                    ("在线设备", "04".to_string(), "/ 6 台", true),
                    ("平均响应", "20".into(), "ms", false),
                    ("模拟负载", format!("{:.0}", self.state.load), "%", false),
                ] {
                    panel(ui, (width - 32.0) / 3.0, 96.0, |ui| {
                        ui.spacing_mut().item_spacing.y = 5.0;
                        ui.label(caption(title));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(value).size(28.0).strong().color(if accent {
                                BLUE
                            } else {
                                Color32::from_gray(27)
                            }));
                            ui.label(caption(unit));
                        });
                    });
                }
            });
            ui.horizontal_top(|ui| {
                panel(ui, width - 276.0, 440.0, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                    ui.horizontal(|ui| {
                        ui.strong("设备列表");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(caption(format!("{} 台设备", self.state.visible().len())));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [ui.available_width() - 90.0, 34.0],
                            egui::TextEdit::singleline(&mut self.state.query)
                                .hint_text("搜索名称或类型"),
                        );
                        ui.checkbox(&mut self.state.online_only, "仅在线");
                    });
                    ui.spacing_mut().item_spacing.y = 5.0;
                    for (index, device) in self.state.visible() {
                        let selected = index == self.state.selected;
                        let rect = egui::Rect::from_min_size(
                            ui.cursor().min,
                            egui::vec2(ui.available_width(), 48.0),
                        );
                        if selected {
                            ui.painter().rect_filled(rect, 8.0, Color32::from_gray(243));
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    rect.min + egui::vec2(7.0, 10.0),
                                    egui::vec2(3.0, 28.0),
                                ),
                                2.0,
                                BLUE,
                            );
                        }
                        ui.allocate_ui_with_layout(
                            rect.size(),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_space(20.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(rect.width() - 134.0, 42.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 2.0;
                                        ui.label(
                                            RichText::new(device.name.split(" / ").next().unwrap())
                                                .size(14.0)
                                                .strong()
                                                .color(if selected {
                                                    BLUE
                                                } else {
                                                    Color32::from_gray(27)
                                                }),
                                        );
                                        ui.label(caption(device.kind));
                                    },
                                );
                                ui.label(
                                    RichText::new(if device.online {
                                        "● 在线"
                                    } else {
                                        "○ 离线"
                                    })
                                    .size(12.0)
                                    .color(if device.online {
                                        Color32::from_rgb(15, 123, 15)
                                    } else {
                                        MUTED
                                    }),
                                );
                                if ui.add_sized([28.0, 28.0], egui::Button::new("›")).clicked() {
                                    self.state.update(Message::Select(index));
                                }
                            },
                        );
                    }
                });
                panel(ui, 260.0, 440.0, |ui| {
                    let device = model::DEVICES[self.state.selected];
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.label(caption(format!("DEVICE {:02}", self.state.selected + 1)));
                    ui.label(
                        RichText::new(device.name.split(" / ").next().unwrap())
                            .size(20.0)
                            .strong(),
                    );
                    ui.label(caption(device.kind));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(caption("连接状态"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(if device.online { "在线" } else { "离线" })
                                    .strong()
                                    .color(if device.online {
                                        Color32::from_rgb(15, 123, 15)
                                    } else {
                                        MUTED
                                    }),
                            );
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(caption("响应时间"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.strong(if device.online {
                                format!("{} ms", device.latency)
                            } else {
                                "—".into()
                            });
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.strong("负载模拟");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{:.0}%", self.state.load)).color(BLUE));
                        });
                    });
                    ui.spacing_mut().slider_width = 224.0;
                    ui.add(egui::Slider::new(&mut self.state.load, 0.0..=100.0).show_value(false));
                    ui.add(
                        egui::ProgressBar::new(self.state.load / 100.0)
                            .desired_width(224.0)
                            .desired_height(4.0),
                    );
                    ui.label(caption("拖动滑块调整演示数据"));
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [224.0, 34.0],
                                egui::Button::new(RichText::new("恢复默认").color(Color32::WHITE))
                                    .fill(BLUE),
                            )
                            .clicked()
                        {
                            self.state.update(Message::Reset);
                        }
                    });
                });
            });
        });
    }
}
fn main() -> eframe::Result {
    let empty = model::has_flag("--benchmark-empty");
    let repaint = model::has_flag("--benchmark-repaint");
    eframe::run_native(
        "设备控制台 · egui",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([model::WIDTH as f32, model::HEIGHT as f32]),
            ..Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            if let Some(dir) = std::env::var_os("WINDIR") {
                if let Ok(bytes) =
                    std::fs::read(std::path::PathBuf::from(dir).join("Fonts/msyh.ttc"))
                {
                    let mut fonts = egui::FontDefinitions::default();
                    fonts.font_data.insert(
                        "cjk".into(),
                        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .push("cjk".into());
                    cc.egui_ctx.set_fonts(fonts);
                }
            }
            Ok(Box::new(Demo {
                state: State::default(),
                empty,
                repaint,
                started: std::time::Instant::now(),
            }))
        }),
    )
}
