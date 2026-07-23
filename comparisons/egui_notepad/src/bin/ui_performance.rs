#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use eframe::egui::{self, Color32, RichText};

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

#[derive(Clone, Copy, PartialEq, Eq)]
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

fn main() -> eframe::Result {
    let launch = LaunchOptions::from_env();
    let title = match PROFILE {
        Profile::Minimal => "UI 性能矩阵 · Minimal · egui",
        Profile::Full => "UI 性能矩阵 · Full Native App · egui",
        Profile::Viewer => "UI 性能矩阵 · Viewer · egui",
    };
    eframe::run_native(
        title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 700.0])
                .with_min_inner_size([820.0, 560.0]),
            ..Default::default()
        },
        Box::new(move |context| {
            context.egui_ctx.set_theme(egui::Theme::Light);
            context.egui_ctx.set_visuals(egui::Visuals::light());
            install_windows_cjk_font(&context.egui_ctx);
            Ok(Box::new(PerformanceApp::new(launch)))
        }),
    )
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
    started: Instant,
    last_poll: Instant,
    last_modified: Option<SystemTime>,
    revision: u64,
    text: String,
    enabled: bool,
    choice: usize,
    progress: f32,
}

impl PerformanceApp {
    fn new(launch: LaunchOptions) -> Self {
        Self {
            launch,
            started: Instant::now(),
            last_poll: Instant::now(),
            last_modified: None,
            revision: 1,
            text: "示例销售方_28.30.pdf".to_owned(),
            enabled: true,
            choice: 0,
            progress: 68.0,
        }
    }

    fn poll_document(&mut self, ui: &egui::Ui) {
        if PROFILE != Profile::Viewer || self.last_poll.elapsed() < Duration::from_millis(250) {
            return;
        }
        self.last_poll = Instant::now();
        if let Some(path) = &self.launch.document {
            let modified = fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .ok();
            if modified.is_some() && modified != self.last_modified {
                self.last_modified = modified;
                self.revision = self.revision.saturating_add(1);
            }
        }
        ui.ctx().request_repaint_after(Duration::from_millis(250));
    }

    fn minimal(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(220.0);
            ui.heading("发票助手 / Invoice Assistant");
            ui.label("Window + Text + Button");
            ui.add_space(16.0);
            let _ = ui.button("选择发票 / Choose invoice");
        });
    }

    fn full(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(210.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.heading("票据工坊");
                    ui.small("FULL NATIVE APP");
                    ui.separator();
                    for label in ["仪表盘", "发票", "规则", "客户", "设置"] {
                        let _ = ui.selectable_label(label == "发票", label);
                    }
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("发票处理中心");
                        ui.separator();
                        let _ = ui.button("新建");
                        let _ = ui.button("导入");
                        let _ = ui.button("导出");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.columns(2, |columns| {
                            columns[0].group(|ui| {
                                ui.strong("表单 / Form");
                                ui.text_edit_singleline(&mut self.text);
                                ui.checkbox(&mut self.enabled, "自动识别");
                                ui.toggle_value(&mut self.enabled, "保留原文件");
                                ui.radio_value(&mut self.choice, 0, "标准规则");
                                ui.radio_value(&mut self.choice, 1, "自定义规则");
                                ui.add(
                                    egui::Slider::new(&mut self.progress, 0.0..=100.0)
                                        .text("置信度"),
                                );
                                egui::ComboBox::from_label("命名模板")
                                    .selected_text("销售方_税额")
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.choice, 0, "销售方_税额");
                                        ui.selectable_value(&mut self.choice, 1, "日期_号码");
                                    });
                                ui.add(
                                    egui::ProgressBar::new(self.progress / 100.0).show_percentage(),
                                );
                            });
                            columns[1].group(|ui| {
                                ui.strong("集合与状态 / Collections");
                                for (index, name) in [
                                    "示例销售方_28.30.pdf",
                                    "示例发票_16.80.pdf",
                                    "差旅报销_230.00.pdf",
                                    "办公用品_86.40.pdf",
                                ]
                                .into_iter()
                                .enumerate()
                                {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", index + 1));
                                        let _ = ui.selectable_label(index == 0, name);
                                        let _ = ui.small_button("移除");
                                    });
                                }
                                ui.separator();
                                egui::Grid::new("invoice-grid")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for row in [
                                            ["销售方", "金额", "状态"],
                                            ["示例公司", "28.30", "完成"],
                                            ["办公商店", "86.40", "待核对"],
                                        ] {
                                            for value in row {
                                                ui.label(value);
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                Color32::from_rgb(0, 120, 70),
                                "✓ 24 个常用控件实例已加载",
                            );
                            let _ = ui.button("取消");
                            let _ = ui.button(RichText::new("开始重命名").strong());
                        });
                    });
                },
            );
        });
    }

    fn viewer(&mut self, ui: &mut egui::Ui) {
        self.poll_document(ui);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("UiDocument 全组件性能页 / All-component performance page");
            ui.label(format!(
                "26 种文档组件 · 250 ms 热重载 · 修订 {}",
                self.revision
            ));
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label("固定加载当前 schema 的全部文档组件；Viewer 额外保留文件轮询、解析、验证与状态映射。");
            });
            ui.add_space(10.0);
            let _ = ui.button("选择发票 / Choose invoice");
            ui.toggle_value(&mut self.enabled, "固定规则 / Pin rule");
            ui.checkbox(&mut self.enabled, "自动识别 / Auto detect");
            ui.text_edit_singleline(&mut self.text);
            ui.add(egui::Slider::new(&mut self.progress, 0.0..=100.0));
            ui.horizontal(|ui| {
                let _ = ui.button("重新加载 / Reload");
                let _ = ui.button("验证文档 / Validate");
            });
            ui.separator();
            ui.strong("文档组件 / Document components");
            ui.columns(2, |columns| {
                for (index, component) in DOCUMENT_COMPONENTS.into_iter().enumerate() {
                    columns[index % 2].label(component);
                }
            });
        });
    }
}

impl eframe::App for PerformanceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(250, 250, 250));
        if self.launch.repaint {
            ui.ctx().request_repaint();
        }
        if self
            .launch
            .auto_close
            .is_some_and(|duration| self.started.elapsed() >= duration)
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if self.launch.empty {
            return;
        }
        match PROFILE {
            Profile::Minimal => self.minimal(ui),
            Profile::Full => self.full(ui),
            Profile::Viewer => self.viewer(ui),
        }
    }
}

fn install_windows_cjk_font(context: &egui::Context) {
    let Some(windows_dir) = env::var_os("WINDIR") else {
        return;
    };
    let font_dir = PathBuf::from(windows_dir).join("Fonts");
    let Some(bytes) = ["msyh.ttc", "msyh.ttf", "simhei.ttf"]
        .into_iter()
        .find_map(|name| fs::read(font_dir.join(name)).ok())
    else {
        return;
    };
    let mut definitions = egui::FontDefinitions::default();
    definitions.font_data.insert(
        "windows-cjk".to_owned(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        definitions
            .families
            .entry(family)
            .or_default()
            .insert(0, "windows-cjk".to_owned());
    }
    context.set_fonts(definitions);
}
