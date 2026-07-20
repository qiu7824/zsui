#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    native_window, Dpi, NativeWindowSmokeRunOptions, Rect, ZsWorkbenchActionSpec,
    ZsWorkbenchComposerSpec, ZsWorkbenchContentBlock, ZsWorkbenchConversationGroupSpec,
    ZsWorkbenchConversationSpec, ZsWorkbenchIcon, ZsWorkbenchInspectorSpec, ZsWorkbenchMessageRole,
    ZsWorkbenchMessageSpec, ZsWorkbenchNoticeLevel, ZsWorkbenchSidebarSpec, ZsWorkbenchSpec,
    ZsWorkbenchToolStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let workbench = sample_workbench();
    let surface = Rect {
        x: 0,
        y: 0,
        width: 1280,
        height: 800,
    };
    let layout = workbench.layout(surface, Dpi::standard());
    let builder = native_window("ZSUI Workbench")
        .size(surface.width as u32, surface.height as u32)
        .min_size(760, 600)
        .workbench(workbench.clone());

    if args.iter().any(|arg| arg == "--smoke") {
        let artifact_dir = "target/zsui-workbench";
        fs::create_dir_all(artifact_dir)?;
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1400)
                .screenshot_file(format!("{artifact_dir}/window.png"))
                .require_screenshot(true),
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--manifest") {
        let draw_plan = builder
            .native_draw_plan()
            .expect("workbench builder should carry a draw plan");
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "component": "workbench_shell",
                "draw_command_count": draw_plan.command_count(),
                "inspector_visible": layout.metrics.inspector.is_some(),
                "message_count": layout.messages.len(),
                "region_count": layout.regions.len(),
                "text_command_count": draw_plan.text_count(),
                "title": workbench.title,
            }))?
        );
        return Ok(());
    }

    builder.run()?;
    Ok(())
}

fn sample_workbench() -> ZsWorkbenchSpec {
    let sidebar = ZsWorkbenchSidebarSpec::new("ZSUI")
        .primary_action(ZsWorkbenchActionSpec::new(
            "new-task",
            "New task",
            ZsWorkbenchIcon::Add,
        ))
        .primary_action(ZsWorkbenchActionSpec::new(
            "search",
            "Search",
            ZsWorkbenchIcon::Search,
        ))
        .group(
            ZsWorkbenchConversationGroupSpec::new("today", "Today")
                .conversation(
                    ZsWorkbenchConversationSpec::new("native-ui", "Native UI framework")
                        .subtitle("Workbench components")
                        .selected(true)
                        .pinned(true),
                )
                .conversation(
                    ZsWorkbenchConversationSpec::new("platforms", "Platform readiness")
                        .subtitle("Windows, macOS and Linux"),
                ),
        )
        .group(
            ZsWorkbenchConversationGroupSpec::new("earlier", "Earlier").conversation(
                ZsWorkbenchConversationSpec::new("release", "Release checklist")
                    .subtitle("Tests and artifacts")
                    .unread(true),
            ),
        )
        .footer_action(ZsWorkbenchActionSpec::new(
            "settings",
            "Settings",
            ZsWorkbenchIcon::Settings,
        ));

    let composer = ZsWorkbenchComposerSpec::new("Describe a task or ask a question")
        .draft("Add a reusable workbench shell to the application.")
        .mode("Build")
        .model("Local runtime")
        .action(ZsWorkbenchActionSpec::new(
            "attach",
            "",
            ZsWorkbenchIcon::Attach,
        ))
        .action(ZsWorkbenchActionSpec::new("mode", "Build", ZsWorkbenchIcon::Tool).selected(true));

    ZsWorkbenchSpec::new("Native UI framework", sidebar, composer)
        .subtitle("Reusable workbench shell")
        .toolbar_action(ZsWorkbenchActionSpec::new(
            "inspector",
            "Inspector",
            ZsWorkbenchIcon::Inspector,
        ))
        .toolbar_action(ZsWorkbenchActionSpec::new(
            "more",
            "More",
            ZsWorkbenchIcon::More,
        ))
        .message(
            ZsWorkbenchMessageSpec::new("message-user", ZsWorkbenchMessageRole::User).block(
                ZsWorkbenchContentBlock::paragraph(
                    "Build a reusable navigation, message timeline, composer and inspector layout.",
                ),
            ),
        )
        .message(
            ZsWorkbenchMessageSpec::new(
                "message-assistant",
                ZsWorkbenchMessageRole::Assistant,
            )
            .block(ZsWorkbenchContentBlock::paragraph(
                "The workbench is product-neutral. Applications provide conversation data, commands and tool output while ZSUI owns layout, paint and hit regions.",
            ))
            .block(ZsWorkbenchContentBlock::tool(
                "Update framework",
                "Added the shared workbench component family",
                ZsWorkbenchToolStatus::Succeeded,
            ))
            .block(ZsWorkbenchContentBlock::code(
                "rust",
                "native_window(\"Workbench\")\n    .size(1280, 800)\n    .workbench(spec)\n    .run()?;",
            ))
            .block(ZsWorkbenchContentBlock::notice(
                "Platform-specific rendering remains behind native backend boundaries.",
                ZsWorkbenchNoticeLevel::Info,
            ))
            .action(ZsWorkbenchActionSpec::new(
                "copy",
                "Copy",
                ZsWorkbenchIcon::Copy,
            ))
            .action(ZsWorkbenchActionSpec::new(
                "retry",
                "Retry",
                ZsWorkbenchIcon::Retry,
            )),
        )
        .inspector(
            ZsWorkbenchInspectorSpec::new("Inspector")
                .selected_tab("changes")
                .tab(ZsWorkbenchActionSpec::new(
                    "changes",
                    "Changes",
                    ZsWorkbenchIcon::Code,
                ))
                .tab(ZsWorkbenchActionSpec::new(
                    "output",
                    "Output",
                    ZsWorkbenchIcon::Tool,
                ))
                .body(
                    "Modified files\n\nworkbench.rs\ncomponent_catalog.rs\nworkbench_shell.rs\n\nStatus\nLayout and paint ready",
                ),
        )
}
