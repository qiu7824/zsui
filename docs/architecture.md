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

The reusable product boundary lives in `src/product_adapter.rs`.
Applications implement `ProductAdapterHost` to project product state into ZSUI
UI declarations, execute UI commands, expose settings, bridge async events and
publish AI capability descriptors. `ZsuiReusableRuntimeHarness` wires that
adapter to a `NativeRuntimeDriver` without moving product behavior into a
platform host. `ProductAdapterRuntimeSmokeRequest` exercises that reusable
handoff path and produces a JSON-serializable smoke report before a product is
bound to a real native driver.

## Public Entry Point

Application authors start with:

```rust
use zsui::{app, Command, TraySpec, Window};
```

The public API is plain Rust data with `serde` support where practical, so
tools can inspect or generate UI declarations without loading a native backend.
`AppBuilder::declaration_report()` and
`AppBuilder::declaration_report_for(capabilities)` return a structured
`ZsuiAppDeclarationReport` that validates app/window/content/menu/tray/hotkey
and settings shapes, and records host capability degradation before any native
event loop is started.

For a minimal real native window, use:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

That convenience builder uses `NativeWindowHost` for the desktop event loop and
keeps full product behavior outside the framework. Android and Harmony are
represented in the platform/capability model as scaffolds; they need dedicated
Activity/Ability runtime hosts before `native_window(...).run()` can create
mobile surfaces. Their current scaffold manifests live in `src/mobile_host.rs`,
`src/android_activity_host.rs` and `src/harmony_ability_host.rs`, and can be
printed with `examples/mobile_scaffold_manifest.rs`.

The reusable desktop bridge for product adapters is `NativeWindowRuntimeDriver`.
It maps `ProductUiProjection` startup requests into ZSUI window, status
item/menu and settings declarations and can be used by
`ZsuiReusableRuntimeHarness`. Status item and settings startup declarations now
pass through `NativeStatusItemHost` and `NativeSettingsPageModelHost`, giving
native backends a concrete operation surface instead of only a stored startup
snapshot. Target proof still requires the platform smoke artifacts in
`docs/native-host-smoke.md`.

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
Native backend completion also requires the smoke artifact contract in
`docs/native-host-smoke.md`; code-level host contracts alone are not proof that
the target OS integration is complete.
