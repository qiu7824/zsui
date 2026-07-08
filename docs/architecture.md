# ZSUI Architecture

ZSUI is a reusable Rust UI foundation for native desktop tools.

The framework layers are:

- Core contracts: stable ids, commands, events, errors and host traits.
- Declaration models: windows, menus, tray/status items, hotkeys, clipboard and settings specs.
- Shared protocols: geometry, layout, command queues, lifecycle/events, components and renderer/text layout traits.
- Capability model: honest support, partial support and degradation reporting per host.
- Native host boundary: platform code creates windows, controls, menus, dialogs and clipboard bridges.
- Product adapter boundary: each application owns domain data, persistence, side effects and AI/tool integrations.

Reusable framework code must not depend on a product database, product settings
schema, sync transport, AI provider, platform handle or native message loop.
Native hosts may depend on platform APIs. Product crates may depend on ZSUI.
ZSUI itself should remain the shared contract between them.

## Public Entry Point

Application authors start with:

```rust
use zsui::{app, Command, TraySpec, Window};
```

The public API is plain Rust data with `serde` support where practical, so
tools can inspect or generate UI declarations without loading a native backend.

For a minimal real native window, use:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

That convenience builder uses `NativeWindowHost` for the desktop event loop and
keeps full product behavior outside the framework. Android and Harmony are
represented in the platform/capability model as scaffolds; they need dedicated
Activity/Ability runtime hosts before `native_window(...).run()` can create
mobile surfaces.

## Host Boundary

Applications call `ZsuiHost` operations:

- create and show/hide windows
- create tray/status menus
- register global hotkeys
- read/write clipboard data
- open file pickers
- show native dialogs
- poll events and run the host event loop

Unsupported features return `ZsuiError::Unsupported` or appear in
`HostCapabilities` degradation reports. A host may accept a window declaration
and still downgrade unsupported traits such as transparency or always-on-top.
