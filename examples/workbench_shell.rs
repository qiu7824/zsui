#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use zsui::{
    composer, inspector_panel, message_timeline, native_window, Dpi, NativeWindowSmokeRunOptions,
    Rect, ZsWorkbenchActionSpec, ZsWorkbenchContentBlock, ZsWorkbenchConversationGroupSpec,
    ZsWorkbenchConversationSpec, ZsWorkbenchIcon, ZsWorkbenchMessageRole, ZsWorkbenchMessageSpec,
    ZsWorkbenchNoticeLevel, ZsWorkbenchShellSpec, ZsWorkbenchSidebarSpec, ZsWorkbenchToolStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let workbench = sample_workbench().into_workbench();
    let surface = Rect {
        x: 0,
        y: 0,
        width: 1280,
        height: 800,
    };
    let manifest_layout = args
        .iter()
        .any(|arg| arg == "--manifest")
        .then(|| workbench.layout(surface, Dpi::standard()));
    let manifest_title = manifest_layout.as_ref().map(|_| workbench.title.clone());
    let builder = native_window("ZSUI Workbench")
        .size(surface.width as u32, surface.height as u32)
        .min_size(760, 600)
        .workbench(workbench);

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
        let layout = manifest_layout.expect("manifest layout should be prepared");
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
                "message_scroll_max": layout.message_scroll_max,
                "message_scrollbar_visible": layout.message_scrollbar.is_some(),
                "region_count": layout.regions.len(),
                "text_command_count": draw_plan.text_count(),
                "title": manifest_title.expect("manifest title should be prepared"),
            }))?
        );
        return Ok(());
    }

    builder.run()?;
    Ok(())
}

fn sample_workbench() -> ZsWorkbenchShellSpec {
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

    let composer = composer("Describe a task or ask a question")
        .draft("Add a reusable workbench shell to the application.")
        .mode("Build")
        .model("Local runtime")
        .action(ZsWorkbenchActionSpec::new(
            "attach",
            "",
            ZsWorkbenchIcon::Attach,
        ))
        .action(ZsWorkbenchActionSpec::new("mode", "Build", ZsWorkbenchIcon::Tool).selected(true));

    let timeline = message_timeline()
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
            .block(ZsWorkbenchContentBlock::tool_with_status_label(
                "Update framework",
                "Added the shared workbench component family",
                ZsWorkbenchToolStatus::Succeeded,
                "Completed",
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
        .message(
            ZsWorkbenchMessageSpec::new(
                "message-native-runtime",
                ZsWorkbenchMessageRole::Assistant,
            )
            .block(ZsWorkbenchContentBlock::paragraph(
                "The retained View runtime updates only the affected native window, keeps the buffered paint path, and materializes just the visible timeline range plus a small overscan window.",
            ))
            .block(ZsWorkbenchContentBlock::tool_with_status_label(
                "Verify native runtime",
                "Semantic icons, overlay scrollbars and retained layout are active",
                ZsWorkbenchToolStatus::Succeeded,
                "Ready",
            ))
            .block(ZsWorkbenchContentBlock::code(
                "rust",
                "native_window(\"Workbench\")\n    .workbench(spec)\n    .invalidation_handle(handle)\n    .run()?;",
            ))
            .action(ZsWorkbenchActionSpec::new(
                "copy-runtime",
                "Copy",
                ZsWorkbenchIcon::Copy,
            )),
        );
    let inspector = inspector_panel("Inspector")
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
        );

    ZsWorkbenchShellSpec::new("Native UI framework", sidebar, composer)
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
        .timeline(timeline)
        .inspector(inspector)
}
