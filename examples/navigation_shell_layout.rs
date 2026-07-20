#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    native_window, Dpi, NativeWindowSmokeRunOptions, Point, Rect, ZsActionAreaSpec,
    ZsActionButtonSpec, ZsGroupCardSpec, ZsNavItemSpec, ZsRowAccessory, ZsShellContentRowSpec,
    ZsShellLayoutSpec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let shell = gallery_shell();
    let audit = shell.audit();
    assert!(audit.valid, "{:?}", audit.issues);

    let bounds = Rect {
        x: 0,
        y: 0,
        width: 1100,
        height: 740,
    };
    let layout = shell.layout_plan(bounds, Dpi::standard());
    let builder = native_window("ZSUI Control Gallery")
        .size(bounds.width as u32, bounds.height as u32)
        .min_size(900, 620)
        .shell_layout(shell.clone());

    if args.iter().any(|arg| arg == "--smoke") {
        let artifact_dir = "target/zsui-shell-gallery";
        fs::create_dir_all(artifact_dir)?;
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1200)
                .screenshot_file(format!("{artifact_dir}/window.png"))
                .require_screenshot(true)
                .native_view_click(Point { x: 40, y: 140 })
                .native_view_scroll(Point { x: 800, y: 360 }, 96),
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if !args.iter().any(|arg| arg == "--manifest") {
        builder.run()?;
        return Ok(());
    }

    let draw_plan = builder
        .native_draw_plan()
        .expect("shell builder should carry a native draw plan");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "draw_command_count": draw_plan.command_count(),
            "interactive_shell_runtime": builder.native_shell_runtime().is_some(),
            "region_count": layout.regions.len(),
            "run_command": "cargo run --example navigation_shell_layout --features full",
            "smoke_command": "cargo run --example navigation_shell_layout --features full -- --smoke",
            "text_command_count": draw_plan.text_count(),
            "shell_title": shell.title,
        }))?
    );

    Ok(())
}

fn gallery_shell() -> ZsShellLayoutSpec {
    ZsShellLayoutSpec::new("control-gallery", "Controls")
        .app_title("ZSUI Gallery")
        .selected_nav("general")
        .nav_item(ZsNavItemSpec::new("general", "General").icon("settings"))
        .nav_item(ZsNavItemSpec::new("controls", "Controls").icon("extension"))
        .nav_item(ZsNavItemSpec::new("shortcuts", "Shortcuts").icon("keyboard"))
        .nav_item(ZsNavItemSpec::new("sync", "Sync").icon("cloud"))
        .nav_item(
            ZsNavItemSpec::new("about", "About")
                .icon("info")
                .badge(true),
        )
        .card(
            ZsGroupCardSpec::new("appearance", "Appearance")
                .row(
                    ZsShellContentRowSpec::new("dark-mode", "Dark mode")
                        .description("Switch the reusable self-drawn surface theme")
                        .accessory(ZsRowAccessory::toggle(false)),
                )
                .row(
                    ZsShellContentRowSpec::new("language", "Language")
                        .description("Dropdown state stays explicit in the shell declaration")
                        .accessory(ZsRowAccessory::dropdown(
                            "English",
                            ["English".to_string(), "Chinese".to_string()],
                        )),
                ),
        )
        .card(
            ZsGroupCardSpec::new("behavior", "Behavior")
                .row(
                    ZsShellContentRowSpec::new("history", "Clipboard history")
                        .description("Toggle rows use the shared control geometry")
                        .accessory(ZsRowAccessory::toggle(true)),
                )
                .row(
                    ZsShellContentRowSpec::new("ignored", "Ignored applications")
                        .description("Action buttons emit product-neutral action identifiers")
                        .accessory(ZsRowAccessory::button("Manage", "ignored.manage")),
                )
                .row(
                    ZsShellContentRowSpec::new("retention", "Retention")
                        .description("Read-only values use the same row alignment contract")
                        .accessory(ZsRowAccessory::value("30 days")),
                ),
        )
        .card(
            ZsGroupCardSpec::new("native", "Native runtime")
                .row(
                    ZsShellContentRowSpec::new("backend", "Rendering backend")
                        .description("Windows uses the buffered Win32/GDI paint path")
                        .accessory(ZsRowAccessory::value("Win32 / GDI+")),
                )
                .row(
                    ZsShellContentRowSpec::new("diagnostics", "Diagnostics")
                        .description("Open a product-owned diagnostics surface")
                        .accessory(ZsRowAccessory::accent_button("Open", "diagnostics.open")),
                ),
        )
        .card(
            ZsGroupCardSpec::new("scroll-proof", "Scroll verification")
                .extra_px(260)
                .row(
                    ZsShellContentRowSpec::new("scroll-state", "Scroll state")
                        .description("Wheel, track click and thumb drag share one scroll model")
                        .accessory(ZsRowAccessory::value("Interactive")),
                ),
        )
        .action_area(
            ZsActionAreaSpec::new()
                .secondary(ZsActionButtonSpec::secondary("cancel", "Cancel"))
                .primary(ZsActionButtonSpec::primary("apply", "Apply")),
        )
}
