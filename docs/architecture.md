# ZSUI Architecture

ZSUI is a reusable Rust UI foundation for native desktop tools.

The framework layers are:

- Core contracts: stable ids, commands, events, errors and host traits.
- Declaration models: windows, menus, tray/status items, hotkeys, clipboard and settings specs.
- Shared protocols: geometry, layout, command queues, lifecycle/events, components, renderer/text layout traits and self-draw command plans.
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
mobile surfaces. Their current scaffold manifests and bridge contracts live in
`src/mobile_host.rs`, `src/android_activity_host.rs` and
`src/harmony_ability_host.rs`, and can be printed with
`examples/mobile_scaffold_manifest.rs` or
`examples/mobile_scaffold_manifest.rs --bridge <platform>`. The bridge
contracts name the FFI symbols, lifecycle/surface/input callbacks, safety
rules and device-smoke artifact files that the real mobile hosts must satisfy.
`mobile_runtime_device_smoke_plan(platform)` and
`review_mobile_runtime_device_smoke_artifacts(platform)` provide the current
read-only verification contract for those required device artifacts.
The same desktop builder can now accept a typed view with
`native_window("Example").view(view).run()?`. That first lays out and paints
`ViewNode<Msg>` into `NativeDrawPlan`; on the direct Windows host the plan is
attached to the created `HWND` and rendered through the extracted no-flicker
GDI path. `ui_command_view(...)` keeps a command-backed view tree for native
input. On Windows, `WM_LBUTTONUP` is routed through `ViewInteractionPlan`,
dispatched into `ViewEventCx<UiCommand>` and recorded as stable command ids in
native smoke. Focused `WM_CHAR` input is also routed into textbox
`TextChanged` events when the textbox feature is enabled, and checkbox clicks
route to typed `Toggled` events when the checkbox feature is enabled. Broader
pointer/keyboard dispatch, IME/composition input and non-Windows input routing
are still separate runtime gates.

The reusable desktop bridge for product adapters is `NativeWindowRuntimeDriver`.
It maps `ProductUiProjection` startup requests into ZSUI window, status
item/menu and settings declarations and can be used by
`ZsuiReusableRuntimeHarness`. Status item and settings startup declarations now
pass through `NativeStatusItemHost` and `NativeSettingsPageModelHost`, and the
reusable ZSClip status/settings action contracts now live in
`src/native_host_actions.rs`. `NativeWindowRuntimeDriver` also implements
status-menu command dispatch and bound settings-item updates so native backends
have a concrete operation surface instead of only a stored startup snapshot.
The reusable self-draw command shape from ZSClip's owner-drawn windows is now
represented in `src/render_protocol.rs` as `NativeDrawPlan` and
`NativeDrawCommandSink`. The Windows GDI implementation extracted from ZSClip
now lives in `src/windows_gdi_renderer.rs`; future platform renderers should
translate the same commands to Direct2D, AppKit, GTK snapshot APIs, Android
Canvas or Harmony Canvas only when that backend needs it, without leaking
product state into the drawing layer.
ZSClip's reusable Win32 main/quick window style mapping, transient-window host,
create-params, message-loop wrapper and `NativeMainWindowHost` implementation
now live in `src/windows_win32_host.rs`. That direct host is available as
framework code and the one-line `native_window(...).run()` convenience path uses
it on Windows. The Win32 paint path treats ZSClip's latest anti-flicker approach
as a baseline: suppress background erase and render through buffered paint
before presenting to the target HDC. It can now also attach a product-neutral
`NativeDrawPlan` to an `HWND`, so the extracted GDI sink paints real framework
draw commands instead of only a background fill. The extracted GDI path now
uses internal RAII wrappers for buffered paint, window HDC acquisition,
compatible memory DCs, smoke-screenshot HBITMAPs, owned main/quick HWND cleanup,
owned HICON app-icon resources, brushes, pens, fonts and selected-object
restoration. Window icon paths are now declaration-audited and loaded into
owned HICON app-icon resources. Win32 tray icons now have a
`Shell_NotifyIconW` backed RAII owner and a `WindowsWin32StatusItemHost`; the
direct Windows `NativeWindowHost` path can now create declared status items,
and native smoke can request one with `native_smoke_run --tray`. Status menus
now have native command-id table routing, RAII popup-menu creation/cleanup and
`TrackPopupMenu` selection routing, but a target-smoke artifact that exercises
real user popup selection is still required before the native backend can be
called complete.
Target proof still requires the platform smoke artifacts in
`docs/native-host-smoke.md`.

## Feature-Gated Build

ZSUI should stay usable as a small dependency. The default Cargo feature set is
`window`, `button` and `label`; heavier services and backends are optional.
`src/feature_manifest.rs` mirrors the `Cargo.toml` feature graph for tools and
AI agents through `zsui_feature_manifest()`.

Applications can opt into only the pieces they need:

```toml
zsui = { version = "0.1", default-features = false, features = [
    "window",
    "button",
    "list",
    "scroll",
    "dark-mode",
] }
```

Optional dependencies must stay behind explicit feature gates: `clipboard`
enables `arboard`, `image` enables `png`, `desktop-winit` enables `winit`, and
`windows-gdi` enables `windows-sys`. Advanced controls should be gated by
widget features or moved into separate crates as they become real
implementations. Avoid global widget registries that instantiate every control
type at startup; public examples should import and build only the controls they
use. Cargo features are unified across the dependency graph, so the long-term
shape should prefer a small default `zsui` facade plus split crates or modules:
`zsui-core`, `zsui-shell`, `zsui-render`, `zsui-style`,
`zsui-widgets-base`, `zsui-widgets-input`, `zsui-widgets-list` and
`zsui-widgets-extra`.

## Rust-First API Target

The revised long-term target is a Rust-native UI framework, not an inheritance
based control hierarchy. The canonical machine-readable list lives in
`src/framework_goals.rs` and is exposed as `zsui_rust_first_goals()`. The
longer narrative is `docs/framework-goals.md`.
The target also captures the product direction from the extraction work: keep
the one-line `zsui::native_window(...).run()?` path as the normal native-window
entry point, use reusable ZSClip host/rendering code as the baseline, add
Android/Harmony as explicit Activity/Ability hosts, and introduce wider
platform API bindings only when a concrete backend needs them.
The first implementation layer lives in `src/view.rs`, `src/style.rs` and
`src/geometry.rs`: typed `View<Msg>` trees, `WidgetId`, explicit app/event/paint
contexts, `ViewInteractionPlan`, `Px`/`Dp`/`Dpi`, `UiLength` and theme tokens.
`ProductViewAdapterHost` connects that typed view layer to product adapters, and
`ZsuiReusableRuntimeHarness::run_view_smoke(...)` verifies the flow from native
view events to typed messages, `AppCx`, product events and reusable
`UiCommand` dispatch.

- Use composition and traits for views/components instead of base classes.
- Preserve one-line native window creation for ordinary desktop apps.
- Use typed messages such as `enum Msg` instead of string event names.
- Own windows, fonts, bitmaps, icons and tray handles with RAII wrappers.
- Use ZSClip's reusable no-flicker self-draw path as the Windows baseline.
- Use typed units such as `Px`, `Dp` and `Dpi` instead of loose numeric sizes.
- Move invalid builder states toward compile-time constraints where practical.
- Avoid global mutable registries; pass explicit app/event/layout/paint contexts.
- Keep public APIs safe and isolate `unsafe` inside backend modules.
- Keep control state explicit in application state and derive UI from state.
- Use theme tokens for colors, radius, spacing and typography.
- Keep the user-facing UI API declarative Rust, without XML or reflection.
- Return `Result<T, ZsuiError>` for backend failures instead of panicking.
- Model platform differences with traits and capability reports.
- Treat Android and Harmony as explicit mobile native hosts, not desktop clones.
- Use Cargo features for widgets, services, platform backends and heavy deps.
- Split large widget/backend families into smaller crates or feature modules.
- Add windows-rs or other wider platform bindings only for concrete backend work.
- Use strong typed IDs for windows, widgets, commands and resources.

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
