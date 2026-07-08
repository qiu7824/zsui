# ZSUI

ZSUI is a Rust-first native system UI framework contract extracted from ZSClip.
It is intentionally declaration-first: application code describes windows,
tray/status menus, commands, hotkeys, settings pages and host capabilities in
Rust, while each platform host translates those declarations to Win32, AppKit,
GTK/libadwaita or mobile hosts.

ZSUI is not a browser shell and not a self-drawn widget kit. Product behavior
stays in the product crate; ZSUI owns portable UI specs, command/event ids,
capability reporting and host traits.

```rust
use zsui::{app, Command, MemoryHost, TraySpec, Window};

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
# Ok::<(), zsui::ZsuiError>(())
```

Create a real native OS window with one line:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

## Current Scope

- `WindowSpec` / `Window`
- `TraySpec`
- `MenuSpec` / `MenuItemSpec`
- `HotkeySpec`
- `ClipboardData`
- `SettingsPageSpec` / `SettingsItemSpec`
- `Command` / `AppEvent`
- `HostCapabilities`
- `ZsuiHost`, `MemoryHost` and `PlatformHost`
- `NativeWindowHost` for a minimal real Windows/macOS/Linux window event loop
- Android and Harmony capability scaffolds for future mobile runtime hosts
- shared geometry, command, event, lifecycle, layout, component, render, host
  surface and native control protocols

`MemoryHost` is the deterministic test backend. `PlatformHost` is a small
scaffold for the current target that records declarations and bridges text
clipboard access where available.

## Repository Shape

- `src/`: public framework API and host contracts.
- `examples/basic.rs`: minimal declaration and memory-host run.
- `docs/architecture.md`: extraction boundary and layering rules.
- `docs/porting.md`: host implementation contract for new platform backends.

ZSUI is designed so another Rust application can provide its own product
adapter and choose a native host without copying ZSClip storage, sync or
business logic.
