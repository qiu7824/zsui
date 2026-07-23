#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, time::Duration};

use slint::{ComponentHandle, SharedString, Timer, TimerMode};

slint::slint! {
    import { Button } from "std-widgets.slint";

    component NavItem inherits Rectangle {
        in property<string> label;
        in property<bool> selected;
        callback activated();
        height: 44px;
        border-radius: 7px;
        background: selected ? #eaf1ff : transparent;
        Text {
            x: 14px;
            width: parent.width - 28px;
            height: parent.height;
            text: (root.selected ? "●   " : "○   ") + root.label;
            color: root.selected ? #245dc9 : #4b525c;
            font-weight: root.selected ? 700 : 400;
            vertical-alignment: center;
        }
        TouchArea { clicked => { root.activated(); } }
    }

    component InfoCard inherits Rectangle {
        in property<string> title;
        in property<string> description;
        in property<string> trailing;
        height: 92px;
        background: white;
        border-width: 1px;
        border-color: #dce1e8;
        border-radius: 9px;
        HorizontalLayout {
            padding: 16px;
            spacing: 12px;
            VerticalLayout {
                spacing: 6px;
                Text { text: root.title; font-size: 16px; font-weight: 700; color: #245dc9; }
                Text { text: root.description; font-size: 12px; color: #747b86; }
            }
            Rectangle { horizontal-stretch: 1; }
            Text { text: root.trailing; color: #245dc9; font-weight: 700; vertical-alignment: center; }
        }
    }

    component FileCard inherits Rectangle {
        in property<string> file-name;
        in property<string> source-name;
        callback remove();
        height: 74px;
        background: white;
        border-width: 1px;
        border-color: #dce1e8;
        border-radius: 9px;
        HorizontalLayout {
            padding: 14px;
            spacing: 14px;
            Text { text: "PDF"; color: #d83e4a; font-weight: 700; vertical-alignment: center; }
            VerticalLayout {
                spacing: 5px;
                Text { text: root.file-name; font-size: 15px; color: #245dc9; font-weight: 700; }
                Text { text: root.source-name; font-size: 12px; color: #747b86; }
            }
            Rectangle { horizontal-stretch: 1; }
            Button { text: "移除"; clicked => { root.remove(); } }
        }
    }

    export component InvoiceWindow inherits Window {
        title: "发票工作台 · Slint";
        preferred-width: 1000px;
        preferred-height: 700px;
        min-width: 820px;
        min-height: 560px;
        background: #f7f8fa;

        in-out property<int> selected: 2;
        in-out property<int> file-count: 2;
        in-out property<string> status: "字段识别完成，可以开始重命名";
        in property<bool> empty;
        callback select(int);
        callback add-file();
        callback remove-file();
        callback rename-files();

        if !root.empty: HorizontalLayout {
            spacing: 0px;
            Rectangle {
                width: 230px;
                background: white;
                border-width: 1px;
                border-color: #e2e5ea;
                VerticalLayout {
                    padding: 20px;
                    spacing: 8px;
                    Text { text: "票据工坊"; font-size: 22px; font-weight: 700; color: #20242b; }
                    Text { text: "INVOICE WORKBENCH"; font-size: 10px; color: #8a919c; }
                    Rectangle { height: 18px; }
                    NavItem { label: "发票合并打印"; selected: root.selected == 0; activated => { root.select(0); } }
                    NavItem { label: "发票信息提取"; selected: root.selected == 1; activated => { root.select(1); } }
                    NavItem { label: "发票重命名"; selected: root.selected == 2; activated => { root.select(2); } }
                    NavItem { label: "发票划分文件夹"; selected: root.selected == 3; activated => { root.select(3); } }
                    Rectangle { vertical-stretch: 1; }
                    Text { text: "本地处理 · 文件不上传"; font-size: 11px; color: #8a919c; }
                }
            }

            Rectangle {
                horizontal-stretch: 1;
                background: #f7f8fa;
                VerticalLayout {
                    padding: 22px;
                    spacing: 14px;
                    HorizontalLayout {
                        VerticalLayout {
                            spacing: 5px;
                            Text { text: "发票重命名"; font-size: 28px; font-weight: 700; color: #20242b; }
                            Text { text: "按发票字段批量生成清晰文件名"; font-size: 13px; color: #747b86; }
                        }
                        Rectangle { horizontal-stretch: 1; }
                        Button { text: "＋ 添加发票"; clicked => { root.add-file(); } }
                    }

                    InfoCard {
                        title: "销售方名称_税额";
                        description: "自定义重命名规则 · 示例：示例销售方_28.30.pdf";
                        trailing: "✓ 已启用";
                    }

                    HorizontalLayout {
                        Text { text: "待处理发票 · " + root.file-count; font-size: 16px; font-weight: 700; color: #20242b; }
                        Rectangle { horizontal-stretch: 1; }
                        Text { text: "识别状态：完成"; font-size: 13px; color: #2f8a51; }
                    }
                    if root.file-count > 0: FileCard {
                        file-name: "示例销售方_28.30.pdf";
                        source-name: "原文件：20260714_001.pdf · 电子发票";
                        remove => { root.remove-file(); }
                    }
                    if root.file-count > 1: FileCard {
                        file-name: "示例发票_16.80.pdf";
                        source-name: "原文件：扫描件_0714.pdf · 已识别销售方和税额";
                        remove => { root.remove-file(); }
                    }

                    InfoCard {
                        title: "输出设置";
                        description: "原文件旁的“已重命名”目录 · 保留原始文件";
                        trailing: "选择文件夹";
                    }
                    InfoCard {
                        title: "输出确认";
                        description: "将重命名 2 张发票并保留原始文件。";
                        trailing: "待确认";
                    }
                    Rectangle { vertical-stretch: 1; }
                    HorizontalLayout {
                        Text { text: root.status; font-size: 13px; color: #747b86; vertical-alignment: center; }
                        Rectangle { horizontal-stretch: 1; }
                        Button { text: "开始重命名"; clicked => { root.rename-files(); } }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let auto_close = args
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs);
    let ui = InvoiceWindow::new()?;
    ui.set_empty(args.iter().any(|argument| argument == "--benchmark-empty"));

    let repaint_timer = Timer::default();
    if args
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

    ui.on_select({
        let weak = ui.as_weak();
        move |index| {
            if let Some(ui) = weak.upgrade() {
                ui.set_selected(index);
            }
        }
    });
    ui.on_add_file({
        let weak = ui.as_weak();
        move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_file_count(ui.get_file_count() + 1);
                ui.set_status(SharedString::from("已添加一张待处理发票"));
            }
        }
    });
    ui.on_remove_file({
        let weak = ui.as_weak();
        move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_file_count((ui.get_file_count() - 1).max(0));
                ui.set_status(SharedString::from("已移除一张发票"));
            }
        }
    });
    ui.on_rename_files({
        let weak = ui.as_weak();
        move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_status(SharedString::from(format!(
                    "已完成 {} 张发票重命名",
                    ui.get_file_count()
                )));
            }
        }
    });

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
