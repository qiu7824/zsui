#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use eframe::egui::{self, Color32, RichText};
use model::{Message, State};

struct Demo {
    state: State,
    empty: bool,
    repaint: bool,
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.repaint {
            ui.ctx().request_repaint();
        }
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(247, 248, 250));
        if self.empty {
            return;
        }
        egui::Frame::new().inner_margin(24.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(model::TITLE).size(26.0).strong());
                ui.add_space(12.0);
                ui.label("6 台设备 / 6 devices     在线 4 / Online 4     平均延迟 / Average latency 20 ms");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_sized([490.0, 32.0], egui::TextEdit::singleline(&mut self.state.query).hint_text("筛选设备 / Filter devices"));
                    ui.checkbox(&mut self.state.online_only, "仅在线 / Online only");
                    if ui.button("重置 / Reset").clicked() { self.state.update(Message::Reset); }
                });
                ui.add_space(12.0);
                egui::Grid::new("devices").num_columns(4).spacing([18.0, 12.0]).min_row_height(30.0).show(ui, |ui| {
                    ui.strong("设备 / Device"); ui.strong("类型 / Type"); ui.strong("状态 / Status"); ui.label(""); ui.end_row();
                    for (index, device) in self.state.visible() {
                        ui.add_sized([330.0, 30.0], egui::Label::new(device.name));
                        ui.add_sized([90.0, 30.0], egui::Label::new(device.kind));
                        ui.add_sized([180.0, 30.0], egui::Label::new(model::status(device)));
                        if ui.button("选择 / Select").clicked() { self.state.update(Message::Select(index)); }
                        ui.end_row();
                    }
                });
                ui.add_space(16.0);
                ui.label(format!("模拟负载 / Simulated load: {:.0}%", self.state.load));
                ui.style_mut().spacing.slider_width = 880.0;
                ui.add(egui::Slider::new(&mut self.state.load, 0.0..=100.0).show_value(false));
                ui.add(egui::ProgressBar::new(self.state.load / 100.0).desired_width(900.0).desired_height(12.0));
                ui.add_space(12.0);
                ui.label(self.state.selection());
            });
        });
    }
}

fn main() -> eframe::Result {
    let empty = model::has_flag("--benchmark-empty");
    let repaint = model::has_flag("--benchmark-repaint");
    eframe::run_native(
        &format!("{} · egui", model::TITLE),
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([model::WIDTH as f32, model::HEIGHT as f32]),
            ..Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            if let Some(dir) = std::env::var_os("WINDIR") {
                let path = std::path::PathBuf::from(dir).join("Fonts/msyh.ttc");
                if let Ok(bytes) = std::fs::read(path) {
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
            }))
        }),
    )
}
