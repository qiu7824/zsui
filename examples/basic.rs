use zsui::{app, Command, MemoryHost, TraySpec, Window};

fn main() -> zsui::ZsuiResult<()> {
    // For a real native OS window, this is enough:
    // zsui::native_window("Example").size(900, 620).run()

    let mut host = MemoryHost::new();

    let runtime = app("Example")
        .window(Window::new("Example").size(900, 620))
        .tray(
            TraySpec::new()
                .tooltip("Example")
                .item("Open", Command::ShowMainWindow)
                .separator()
                .item("Quit", Command::Quit),
        )
        .global_hotkey("Alt+V", Command::OpenQuickPanel)
        .run_with_host(&mut host)?;

    println!(
        "registered {} window(s), tray={}",
        runtime.windows.len(),
        runtime.tray.is_some()
    );

    Ok(())
}
