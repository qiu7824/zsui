#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf, time::Duration};

#[cfg(feature = "perf-viewer")]
use std::fs;

use slint::{ComponentHandle, Timer, TimerMode};

#[cfg(all(not(feature = "perf-full"), not(feature = "perf-viewer")))]
mod profile_ui {
    slint::slint! {
        import { Button } from "std-widgets.slint";

        export component PerformanceWindow inherits Window {
            title: "UI 性能矩阵 · Minimal · Slint";
            preferred-width: 1000px;
            preferred-height: 700px;
            min-width: 820px;
            min-height: 560px;
            in property<bool> empty;
            in property<int> revision;
            if !root.empty: VerticalLayout {
                alignment: center;
                spacing: 16px;
                Text { text: "发票助手 / Invoice Assistant"; font-size: 28px; font-weight: 700; horizontal-alignment: center; }
                Text { text: "Window + Text + Button"; horizontal-alignment: center; }
                HorizontalLayout {
                    alignment: center;
                    Button { text: "选择发票 / Choose invoice"; }
                }
            }
        }
    }
}

#[cfg(all(feature = "perf-full", not(feature = "perf-viewer")))]
mod profile_ui {
    slint::slint! {
        import { Button, CheckBox, ComboBox, LineEdit, ProgressIndicator, Slider, SpinBox, Switch } from "std-widgets.slint";

        component Panel inherits Rectangle {
            background: white;
            border-width: 1px;
            border-color: #dce1e8;
            border-radius: 9px;
        }

        component FileRow inherits Rectangle {
            in property<string> label;
            height: 44px;
            background: #f9fafb;
            border-radius: 6px;
            HorizontalLayout {
                padding: 8px;
                spacing: 10px;
                Text { text: root.label; vertical-alignment: center; }
                Rectangle { horizontal-stretch: 1; }
                Button { text: "移除"; }
            }
        }

        export component PerformanceWindow inherits Window {
            title: "UI 性能矩阵 · Full Native App · Slint";
            preferred-width: 1000px;
            preferred-height: 700px;
            min-width: 820px;
            min-height: 560px;
            background: #f5f6f8;
            in property<bool> empty;
            in property<int> revision;
            if !root.empty: HorizontalLayout {
                spacing: 0px;
                Rectangle {
                    width: 210px;
                    background: white;
                    border-width: 1px;
                    border-color: #e2e5ea;
                    VerticalLayout {
                        padding: 18px;
                        spacing: 8px;
                        Text { text: "票据工坊"; font-size: 22px; font-weight: 700; }
                        Text { text: "FULL NATIVE APP"; font-size: 10px; color: #7a818c; }
                        Rectangle { height: 12px; }
                        Button { text: "仪表盘"; }
                        Button { text: "发票"; }
                        Button { text: "规则"; }
                        Button { text: "客户"; }
                        Button { text: "设置"; }
                        Rectangle { vertical-stretch: 1; }
                        Text { text: "本地处理 · 文件不上传"; font-size: 11px; color: #7a818c; }
                    }
                }
                VerticalLayout {
                    padding: 18px;
                    spacing: 12px;
                    HorizontalLayout {
                        Text { text: "发票处理中心"; font-size: 26px; font-weight: 700; vertical-alignment: center; }
                        Rectangle { horizontal-stretch: 1; }
                        Button { text: "新建"; }
                        Button { text: "导入"; }
                        Button { text: "导出"; }
                    }
                    HorizontalLayout {
                        spacing: 12px;
                        Panel {
                            horizontal-stretch: 1;
                            VerticalLayout {
                                padding: 14px;
                                spacing: 9px;
                                Text { text: "表单 / Form"; font-size: 17px; font-weight: 700; }
                                LineEdit { text: "示例销售方_28.30.pdf"; placeholder-text: "发票名称"; }
                                CheckBox { text: "自动识别"; checked: true; }
                                Switch { text: "保留原文件"; checked: true; }
                                CheckBox { text: "标准规则"; checked: true; }
                                CheckBox { text: "自定义规则"; }
                                Slider { minimum: 0; maximum: 100; value: 68; }
                                ComboBox { model: ["销售方_税额", "日期_号码", "客户_金额"]; current-index: 0; }
                                SpinBox { minimum: 1; maximum: 99; value: 2; }
                                ProgressIndicator { progress: 0.68; }
                            }
                        }
                        Panel {
                            horizontal-stretch: 1;
                            VerticalLayout {
                                padding: 14px;
                                spacing: 8px;
                                Text { text: "集合与状态 / Collections"; font-size: 17px; font-weight: 700; }
                                FileRow { label: "1. 示例销售方_28.30.pdf"; }
                                FileRow { label: "2. 示例发票_16.80.pdf"; }
                                FileRow { label: "3. 差旅报销_230.00.pdf"; }
                                FileRow { label: "4. 办公用品_86.40.pdf"; }
                                HorizontalLayout { Text { text: "销售方"; } Rectangle { horizontal-stretch: 1; } Text { text: "金额"; } Text { text: "状态"; } }
                                HorizontalLayout { Text { text: "示例公司"; } Rectangle { horizontal-stretch: 1; } Text { text: "28.30"; } Text { text: "完成"; } }
                                HorizontalLayout { Text { text: "办公商店"; } Rectangle { horizontal-stretch: 1; } Text { text: "86.40"; } Text { text: "待核对"; } }
                            }
                        }
                    }
                    HorizontalLayout {
                        Text { text: "✓ 24 个常用控件实例已加载"; color: #177245; vertical-alignment: center; }
                        Rectangle { horizontal-stretch: 1; }
                        Button { text: "取消"; }
                        Button { text: "开始重命名"; }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "perf-viewer")]
mod profile_ui {
    slint::slint! {
        import { Button, CheckBox, ComboBox, LineEdit, ProgressIndicator, Slider, Switch } from "std-widgets.slint";

        export component PerformanceWindow inherits Window {
            title: "UI 性能矩阵 · Viewer · Slint";
            preferred-width: 1000px;
            preferred-height: 700px;
            min-width: 820px;
            min-height: 560px;
            background: #f5f6f8;
            in property<bool> empty;
            in property<int> revision;
            if !root.empty: VerticalLayout {
                padding: 20px;
                spacing: 10px;
                Text { text: "UiDocument 全组件性能页 / All-component performance page"; font-size: 24px; font-weight: 700; }
                Text { text: "26 种文档组件 · 250 ms 热重载 · 修订 " + root.revision; color: #5f6670; }
                Rectangle {
                    background: white;
                    border-width: 1px;
                    border-color: #dce1e8;
                    border-radius: 9px;
                    VerticalLayout {
                        padding: 14px;
                        spacing: 8px;
                        Text { text: "固定加载当前 schema 的全部文档组件；Viewer 额外保留文件轮询、解析、验证与状态映射。"; }
                        Button { text: "选择发票 / Choose invoice"; }
                        LineEdit { text: "示例销售方_28.30.pdf"; }
                        CheckBox { text: "自动识别 / Auto detect"; checked: true; }
                        Switch { text: "固定规则 / Pin rule"; checked: true; }
                        Slider { minimum: 0; maximum: 100; value: 68; }
                        ProgressIndicator { progress: 0.68; }
                        ComboBox { model: ["销售方_税额", "日期_号码", "客户_金额"]; current-index: 0; }
                        HorizontalLayout { Button { text: "重新加载 / Reload"; } Button { text: "验证文档 / Validate"; } }
                    }
                }
                Text { text: "文档组件 / Document components"; font-size: 17px; font-weight: 700; }
                HorizontalLayout {
                    spacing: 32px;
                    VerticalLayout {
                        spacing: 3px;
                        Text { text: "Stack"; } Text { text: "Border"; } Text { text: "Scroll"; }
                        Text { text: "Tabs"; } Text { text: "List"; } Text { text: "Grid"; }
                        Text { text: "Text"; } Text { text: "Button"; } Text { text: "ToggleButton"; }
                        Text { text: "CheckBox"; } Text { text: "Toggle"; } Text { text: "TextBox"; }
                        Text { text: "PasswordBox"; }
                    }
                    VerticalLayout {
                        spacing: 3px;
                        Text { text: "RadioButton"; } Text { text: "Slider"; } Text { text: "NumberBox"; }
                        Text { text: "ComboBox"; } Text { text: "AutoSuggestBox"; } Text { text: "CommandPalette"; }
                        Text { text: "TreeView"; } Text { text: "GridView"; } Text { text: "DatePicker"; }
                        Text { text: "TimePicker"; } Text { text: "ColorPicker"; } Text { text: "ProgressBar"; }
                        Text { text: "ProgressRing"; }
                    }
                }
            }
        }
    }
}

use profile_ui::PerformanceWindow;

fn main() -> Result<(), slint::PlatformError> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let auto_close = arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs);
    let document = arguments
        .windows(2)
        .find(|pair| pair[0] == "--document")
        .map(|pair| PathBuf::from(&pair[1]));
    let ui = PerformanceWindow::new()?;
    ui.set_empty(
        arguments
            .iter()
            .any(|argument| argument == "--benchmark-empty"),
    );

    let repaint_timer = Timer::default();
    if arguments
        .iter()
        .any(|argument| argument == "--benchmark-repaint")
    {
        let weak = ui.as_weak();
        repaint_timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
            if let Some(ui) = weak.upgrade() {
                ui.window().request_redraw();
            }
        });
    }

    let poll_timer = Timer::default();
    #[cfg(feature = "perf-viewer")]
    if let Some(document) = document {
        let weak = ui.as_weak();
        let mut last_modified = None;
        poll_timer.start(TimerMode::Repeated, Duration::from_millis(250), move || {
            let modified = fs::metadata(&document)
                .and_then(|metadata| metadata.modified())
                .ok();
            if modified.is_some() && modified != last_modified {
                last_modified = modified;
                if let Some(ui) = weak.upgrade() {
                    ui.set_revision(ui.get_revision().saturating_add(1));
                }
            }
        });
    }
    #[cfg(not(feature = "perf-viewer"))]
    let _ = (document, &poll_timer);

    if let Some(duration) = auto_close {
        let weak = ui.as_weak();
        Timer::single_shot(duration, move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        });
    }
    let _keep_repaint_timer_alive = repaint_timer;
    ui.run()
}
