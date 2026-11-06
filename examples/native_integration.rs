//! Native child views, an integrated caption and a window-owned global shortcut.

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use zsui::prelude::*;
    use zsui::{Command, HotkeySpec};

    const EDIT: NativeHostId = NativeHostId::new(1001);
    const TREE: NativeHostId = NativeHostId::new(1002);
    const BROWSER: NativeHostId = NativeHostId::new(1003);
    #[derive(Clone)]
    enum Message {
        Native(NativeWindowEvent),
        Files(FilesDropped),
        Toggle,
    }
    struct State {
        reserved: f32,
        status: String,
        visible: bool,
        theme: ThemeMode,
    }

    let mut arguments = std::env::args_os().skip(1);
    let folder = std::path::PathBuf::from(
        arguments
            .next()
            .ok_or("usage: native_integration <folder> [--dark]")?,
    )
    .canonicalize()?;
    if !folder.is_dir() {
        return Err("the selected path is not a folder".into());
    }
    let theme = if arguments.any(|argument| argument == "--dark") {
        ThemeMode::Dark
    } else {
        ThemeMode::System
    };
    let providers = NativeHosts::new()
        .edit(EDIT, "", false)?
        .namespace_tree(TREE, &folder)?
        .explorer(BROWSER, &folder)?;
    window("Native integration")
        .size(960, 640)
        .min_size(600, 360)
        .native_hosts(providers)
        .integrated_title_bar(Dp::new(40.0), WindowBackdrop::Mica)?
        .global_hotkey(HotkeySpec::new(
            "Ctrl+Alt+Shift+F9",
            Command::ToggleMainWindow,
        ))
        .release_view_when_hidden()
        .stateful_with_commands(
            State {
                reserved: 138.0,
                status: "Drop files here".into(),
                visible: true,
                theme,
            },
            |state| {
                column([
                    row([
                        text("Native integration").flex(1.0),
                        spacer().width(Dp::new(state.reserved)),
                    ])
                    .height(Dp::new(40.0))
                    .window_drag_region(),
                    native_host(EDIT).height(Dp::new(34.0)),
                    row([
                        native_host(TREE).width(Dp::new(220.0)),
                        native_host(BROWSER).flex(1.0),
                    ])
                    .gap(Dp::new(12.0))
                    .flex(1.0),
                    text(&state.status),
                    text("Ctrl+Alt+Shift+F9: show or hide"),
                ])
                .gap(Dp::new(12.0))
                .padding(Dp::new(12.0))
                .theme_mode(state.theme)
                .on_native_window_event(Message::Native)
                .on_drop_files(Message::Files)
            },
            |state, message, cx| match message {
                Message::Native(NativeWindowEvent::CaptionChanged(metrics)) => {
                    state.reserved = metrics.reserved_width().0
                }
                Message::Native(NativeWindowEvent::Error { message, .. }) => state.status = message,
                Message::Native(_) => {}
                Message::Files(files) => {
                    state.status = format!("{} files received", files.files.len())
                }
                Message::Toggle => {
                    state.visible = !state.visible;
                    cx.window_command(if state.visible {
                        WindowCommand::Show
                    } else {
                        WindowCommand::Hide
                    });
                }
            },
            |command| matches!(command, Command::ToggleMainWindow).then_some(Message::Toggle),
        )
        .run()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("this example requires the Windows native host".into())
}
