use zsui::{
    app, Command, HostCapabilities, SettingsItemSpec, SettingsPageSpec, TraySpec, UiNode, Window,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report =
        app("Audit Example")
            .window(
                Window::new("Audit Example").size(900, 620).content(
                    UiNode::column("root")
                        .gap(8)
                        .child(UiNode::text("title", "Audit Example"))
                        .child(UiNode::button("open", "Open", Command::ShowMainWindow)),
                ),
            )
            .tray(
                TraySpec::new()
                    .tooltip("Audit Example")
                    .item("Open", Command::ShowMainWindow)
                    .item("Quit", Command::Quit),
            )
            .global_hotkey("Alt+Space", Command::OpenQuickPanel)
            .settings_page(SettingsPageSpec::new("general", "General").item(
                SettingsItemSpec::toggle("launch_on_startup", "Launch on startup", false),
            ))
            .declaration_report_for(&HostCapabilities::linux_scaffold());

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
