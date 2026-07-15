#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use eframe::egui::{self, Color32, RichText, Stroke};

fn main() -> eframe::Result {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let auto_close = args
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "发票工作台 · egui",
        options,
        Box::new(move |context| {
            context.egui_ctx.set_theme(egui::Theme::Light);
            context.egui_ctx.set_visuals(egui::Visuals::light());
            install_windows_cjk_font(&context.egui_ctx);
            Ok(Box::new(InvoiceApp::new(auto_close)))
        }),
    )
}

fn install_windows_cjk_font(context: &egui::Context) {
    let Some(windows_dir) = env::var_os("WINDIR") else {
        return;
    };
    let font_dir = PathBuf::from(windows_dir).join("Fonts");
    let Some(bytes) = ["msyh.ttc", "msyh.ttf", "simhei.ttf"]
        .into_iter()
        .find_map(|name| std::fs::read(font_dir.join(name)).ok())
    else {
        return;
    };
    let mut definitions = egui::FontDefinitions::default();
    definitions.font_data.insert(
        "windows-cjk".to_string(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        definitions
            .families
            .entry(family)
            .or_default()
            .insert(0, "windows-cjk".to_string());
    }
    context.set_fonts(definitions);
}

struct InvoiceApp {
    selected: usize,
    file_count: usize,
    status: String,
    started: Instant,
    auto_close: Option<Duration>,
}

impl InvoiceApp {
    fn new(auto_close: Option<Duration>) -> Self {
        Self {
            selected: 2,
            file_count: 2,
            status: "字段识别完成，可以开始重命名".to_string(),
            started: Instant::now(),
            auto_close,
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.label(RichText::new("票据工坊").size(19.0).strong());
        ui.label(
            RichText::new("INVOICE WORKBENCH")
                .size(10.0)
                .color(Color32::GRAY),
        );
        ui.add_space(28.0);
        for (index, label) in [
            "发票合并打印",
            "发票信息提取",
            "发票重命名",
            "发票划分文件夹",
        ]
        .into_iter()
        .enumerate()
        {
            let selected = self.selected == index;
            let text = if selected {
                RichText::new(format!("  ●  {label}"))
                    .color(Color32::from_rgb(32, 88, 190))
                    .strong()
            } else {
                RichText::new(format!("  ○  {label}")).color(Color32::from_rgb(75, 82, 92))
            };
            if ui
                .add_sized([204.0, 44.0], egui::Button::new(text).selected(selected))
                .clicked()
            {
                self.selected = index;
            }
            ui.add_space(6.0);
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.label(
                RichText::new("本地处理 · 文件不上传")
                    .size(11.0)
                    .color(Color32::GRAY),
            );
        });
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("发票重命名").size(26.0).strong());
                ui.label(RichText::new("按发票字段批量生成清晰文件名").color(Color32::GRAY));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("＋ 添加发票").clicked() {
                    self.file_count += 1;
                    self.status = "已添加一张待处理发票".to_string();
                }
            });
        });
        ui.add_space(18.0);

        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("自定义重命名规则").size(15.0).strong());
                    ui.label(
                        RichText::new("销售方名称_税额")
                            .size(18.0)
                            .color(Color32::from_rgb(36, 93, 201))
                            .strong(),
                    );
                    ui.label(RichText::new("示例：永新行业协会_28.30.pdf").color(Color32::GRAY));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("✓ 已启用")
                            .color(Color32::from_rgb(36, 93, 201))
                            .strong(),
                    );
                });
            });
        });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("待处理发票 · {}", self.file_count))
                    .size(15.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("识别状态：完成").color(Color32::from_rgb(45, 130, 78)));
            });
        });
        ui.add_space(8.0);

        if self.file_count > 0 {
            file_card(
                ui,
                "永新行业协会_28.30.pdf",
                "原文件：20260714_001.pdf",
                || {
                    self.file_count = self.file_count.saturating_sub(1);
                },
            );
        }
        if self.file_count > 1 {
            ui.add_space(8.0);
            file_card(
                ui,
                "示例发票_16.80.pdf",
                "原文件：扫描件_0714.pdf",
                || {
                    self.file_count = self.file_count.saturating_sub(1);
                },
            );
        }

        ui.add_space(14.0);
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("输出设置").strong());
                    ui.label(
                        RichText::new("原文件旁的“已重命名”目录 · 保留原始文件")
                            .color(Color32::GRAY),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = ui.button("选择文件夹");
                });
            });
        });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.status).color(Color32::GRAY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized([150.0, 38.0], egui::Button::new("开始重命名"))
                    .clicked()
                {
                    self.status = format!("已完成 {} 张发票重命名", self.file_count);
                }
            });
        });
    }
}

impl eframe::App for InvoiceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self
            .auto_close
            .is_some_and(|duration| self.started.elapsed() >= duration)
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(247, 248, 250));
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(228.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.painter().rect_filled(ui.max_rect(), 0.0, Color32::WHITE);
                    self.navigation(ui);
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.content(ui),
            );
        });
    }
}

fn card(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 224, 230)))
        .corner_radius(8.0)
        .inner_margin(14.0)
        .show(ui, content);
}

fn file_card(ui: &mut egui::Ui, name: &str, source: &str, remove: impl FnOnce()) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PDF")
                    .color(Color32::from_rgb(218, 62, 74))
                    .strong(),
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(name)
                        .color(Color32::from_rgb(36, 93, 201))
                        .strong(),
                );
                ui.label(RichText::new(source).color(Color32::GRAY));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("移除").clicked() {
                    remove();
                }
            });
        });
    });
}
