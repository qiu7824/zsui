#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    native_window, NativeWindowSmokeRunOptions, Point, ZsActionAreaSpec, ZsActionButtonSpec,
    ZsGroupCardSpec, ZsNavItemSpec, ZsRowAccessory, ZsShellContentRowSpec, ZsShellLayoutSpec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let shell = invoice_shell();
    let audit = shell.audit();
    assert!(audit.valid, "{:?}", audit.issues);

    let builder = native_window("发票工作台 · ZSUI")
        .size(1100, 740)
        .min_size(900, 620)
        .shell_layout(shell);

    if args.iter().any(|arg| arg == "--smoke") {
        let artifact_dir = "target/invoice-ui-comparison";
        fs::create_dir_all(artifact_dir)?;
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone())
            .unwrap_or_else(|| format!("{artifact_dir}/zsui.png"));
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1_800)
                .screenshot_file(screenshot)
                .require_screenshot(cfg!(windows))
                .native_view_click(Point { x: 120, y: 184 })
                .native_view_scroll(Point { x: 820, y: 430 }, 48),
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    builder.run()?;
    Ok(())
}

fn invoice_shell() -> ZsShellLayoutSpec {
    ZsShellLayoutSpec::new("invoice-workbench", "发票重命名")
        .app_title("票据工坊")
        .selected_nav("rename")
        .nav_item(ZsNavItemSpec::new("merge", "发票合并打印").icon("general"))
        .nav_item(ZsNavItemSpec::new("extract", "发票信息提取").icon("plugin"))
        .nav_item(ZsNavItemSpec::new("rename", "发票重命名").icon("keyboard"))
        .nav_item(ZsNavItemSpec::new("folders", "发票划分文件夹").icon("folder"))
        .card(
            ZsGroupCardSpec::new("rule", "自定义重命名规则")
                .description("使用发票字段生成清晰、稳定的文件名")
                .row(
                    ZsShellContentRowSpec::new("rule-template", "销售方名称_税额")
                        .description("示例：永新行业协会_28.30.pdf")
                        .accessory(ZsRowAccessory::value("已启用")),
                )
                .row(
                    ZsShellContentRowSpec::new("conflict", "重名处理")
                        .description("保留原文件，并为重名文件追加序号")
                        .accessory(ZsRowAccessory::dropdown(
                            "追加序号",
                            ["追加序号".to_string(), "覆盖".to_string()],
                        )),
                ),
        )
        .card(
            ZsGroupCardSpec::new("files", "待处理发票 · 2")
                .row(
                    ZsShellContentRowSpec::new("invoice-1", "永新行业协会_28.30.pdf")
                        .description("原文件：20260714_001.pdf · 电子发票")
                        .accessory(ZsRowAccessory::button("移除", "invoice.remove.1")),
                )
                .row(
                    ZsShellContentRowSpec::new("invoice-2", "示例发票_16.80.pdf")
                        .description("原文件：扫描件_0714.pdf · 已识别销售方和税额")
                        .accessory(ZsRowAccessory::button("移除", "invoice.remove.2")),
                ),
        )
        .card(
            ZsGroupCardSpec::new("output", "输出设置")
                .row(
                    ZsShellContentRowSpec::new("output-folder", "输出文件夹")
                        .description("处理后的文件保存在原文件旁的“已重命名”目录")
                        .accessory(ZsRowAccessory::accent_button("选择文件夹", "output.choose")),
                )
                .row(
                    ZsShellContentRowSpec::new("keep-source", "保留原始文件")
                        .description("完成后仍可从原始名称追溯发票")
                        .accessory(ZsRowAccessory::toggle(true)),
                ),
        )
        .action_area(
            ZsActionAreaSpec::new()
                .secondary(ZsActionButtonSpec::secondary("invoice.add", "添加发票"))
                .primary(ZsActionButtonSpec::primary("invoice.rename", "开始重命名")),
        )
}
