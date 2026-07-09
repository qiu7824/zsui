# ZSUI AI Agent Guide

This document is the standalone ZSUI entry point for AI coding agents. It
describes what is reusable framework code, what is still only a contract, and
where agents should edit first.

## Current Completion

ZSUI is about 77% complete as a standalone UI framework.

- Foundation contracts: about 74% complete.
- Declaration API: about 74% complete.
- Minimal native window runtime: about 75% complete.
- Feature-pruned architecture: about 38% complete.
- Rust-first API model: about 68% complete.
- Full desktop native hosts: about 55% complete.
- Android and Harmony: about 16% complete.
- Product adapter/runtime harness: about 62% complete.
- Native smoke verification: about 70% complete.

The crate can already describe and audit windows, tray/status menus, commands,
hotkeys, settings pages, host capabilities, shared geometry,
command/event/layout/render protocols, declarative component trees and native
host contracts. It can also create a minimal real desktop window through
`zsui::native_window("Title").run()`.
Use `AppBuilder::declaration_report()`,
`AppBuilder::declaration_report_for(capabilities)` or
`ZsuiApp::declaration_report_for(capabilities)` to get a structured
`ZsuiAppDeclarationReport` before binding an app declaration to a host.
The current machine-readable handoff is `zsui::zsui_agent_context()`; tools can
also call `zsui::zsui_agent_context_json()` to read the same platform, gate and
completion data as JSON.

It is not yet a complete application UI runtime. The full Win32, AppKit and GTK
product hosts are still being split out of ZSClip because those implementations
currently mix reusable host behavior with clipboard-product behavior.
The current Windows backend metadata points to the extracted `win32_gdi`
runtime. ZSClip's reusable Win32 main/quick window style, transient-window host,
create-params, message-loop and `NativeMainWindowHost` implementation are now
extracted in `src/windows_win32_host.rs` and wired into the default
`native_window(...).run()` path on Windows. AppKit/GTK files are not represented
as implemented ZSUI modules until they are really extracted.
`src/native_host_actions.rs` is directly split from the reusable ZSClip native
host action/status/settings command contracts and adapted to standalone ZSUI
types. `ProductUiProjection` now carries the main window, status item/tray menu
and settings pages into `NativeRuntimeStartupRequest`; `NativeWindowRuntimeDriver`
routes those declarations through `NativeStatusItemHost` and
`NativeSettingsPageModelHost`, dispatches status menu commands through
`NativeStatusMenuCommandHost`, updates bound settings item values through
`NativeSettingsItemUpdateHost`, then reports operation names, status menu counts
and settings page counts.
ZSClip's self-drawn window code also used a reusable command-plan shape
(`FillRect`, `RoundRect`, `RoundFill`, text commands and icon commands). ZSUI
now exposes the product-neutral version in `src/render_protocol.rs` as
`NativeDrawPlan`, `NativeDrawCommand` and `NativeDrawCommandSink`, and
`src/windows_gdi_renderer.rs` contains the extracted Windows GDI renderer/text
layout/draw sink and the ZSClip no-flicker buffered paint foundation.
`src/windows_win32_host.rs` can attach a `NativeDrawPlan` to an `HWND`, then
paint it through the buffered Win32/GDI path. GDI brushes, pens, fonts,
selected-object restoration, buffered-paint handles, window HDC acquisition,
compatible memory DCs, smoke-screenshot HBITMAP ownership and owned Win32 main/
quick HWND cleanup now use internal RAII wrappers. Owned HICON wrappers and an
owned app-icon resource model also exist, including file loading through
`LoadImageW`, shared small/big icon handling, retention from owned window
handles and declarative `WindowSpec::icon_path(...)` validation. Win32 tray
icons now have a `Shell_NotifyIconW` backed RAII owner and a
`WindowsWin32StatusItemHost`; the direct Windows `NativeWindowHost` path can
create declared status items, and `native_smoke_run --tray` can request a real
status item during smoke runs. Win32 status menus now have a native command-id
table, reusable status-menu command dispatch, RAII-owned popup menu creation,
`TrackPopupMenu` selection routing and explicit popup cleanup evidence; `--tray`
records the non-blocking pieces in `interaction.json`. A target artifact that
exercises real user popup selection is still pending. Higher-level APIs should
keep avoiding raw HWND exposure. Wider `windows-rs` APIs should be added only
when a concrete backend needs them.
Windows first-pass target smoke has a local artifact path:
`cargo run --example native_smoke_run -- windows` captures `window.png`, and
`cargo run --example native_smoke_review -- windows` reports
`target_smoke_complete=true` when all six required artifacts are present.
macOS, Linux, Android and Harmony still require target/device proof.
Android and Harmony now have explicit mobile bridge contracts in
`src/mobile_host.rs`, `src/android_activity_host.rs` and
`src/harmony_ability_host.rs`: callback symbols, lifecycle/surface/input/
command routes, FFI safety rules and required device-smoke artifact names are
serialized through `mobile_runtime_bridge_contract(platform)` and
`examples/mobile_scaffold_manifest.rs --bridge <platform>`. The same module now
has device-smoke plans and read-only artifact review through
`mobile_runtime_device_smoke_plan(platform)`,
`review_mobile_runtime_device_smoke_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --review <platform>`. These are still
contracts and verifiers, not native FFI implementations or device proof.
The Cargo feature boundary is now explicit in `Cargo.toml` and
`src/feature_manifest.rs`: defaults are `window`, `button` and `label`;
`clipboard`, `image`, `desktop-winit` and `windows-gdi` are optional dependency
features; advanced widgets remain opt-in. This is feature/crate based
trimming, not automatic unused-widget pruning inside an enabled crate. Cargo
features are unified across the dependency graph, so the long-range shape is a
small facade plus feature-gated crates/modules such as `zsui-core`,
`zsui-shell`, `zsui-render`, `zsui-style`, `zsui-widgets-base`,
`zsui-widgets-input`, `zsui-widgets-list` and `zsui-widgets-extra`.
The framework target is also now explicit in `src/framework_goals.rs` and
`docs/framework-goals.md`: ZSUI should use composition plus traits, typed
messages, RAII native resources, typed units, compile-time builder constraints
where useful, explicit contexts, isolated unsafe, explicit app state, theme
tokens, declarative Rust builders, `Result<T, ZsuiError>`, capability traits,
feature-gated platform backends, split-crate/module trimming and strong typed
IDs. It also records the larger extraction target: keep
`zsui::native_window(...).run()?` as the normal native-window entry, make
ZSClip's reusable no-flicker self-draw path the Windows baseline, treat Android
and Harmony as real Activity/Ability host targets, and add `windows-rs` or
other broader platform bindings only for specific backend work. The source
target records the preferred and avoided API shapes, such as `enum Msg` over
string events and feature/crate based trimming over global widget registration.
The first concrete Rust-first API pass now exists in `src/view.rs`,
`src/style.rs` and `src/geometry.rs`: `View<Msg>`, typed event messages,
`WidgetId`, `AppCx`, `ViewEventCx`, `ViewPaintCx`, `ViewInteractionPlan`,
typed list selection, `Px`, `Dp`, `Dpi`, `UiLength`, `ZsuiTheme` and theme
tokens.
`ProductViewAdapterHost` and `ZsuiReusableRuntimeHarness::run_view_smoke(...)`
now prove that typed view messages can flow through `AppCx` into product events
and reusable `UiCommand` dispatch without a string event bus.
`NativeWindowBuilder::view(...)` now lays out and paints a `ViewNode<Msg>` into
a product-neutral `NativeDrawPlan`, and the direct Win32 smoke path attaches
that plan to the native `HWND` for no-flicker GDI painting. The smoke runner can
exercise this with `cargo run --example native_smoke_run -- windows --view`;
`NativeWindowBuilder::ui_command_view(...)` additionally keeps a command-backed
view tree for native input routing. On Windows the direct Win32 host now handles
`WM_LBUTTONUP`, hit-tests through `ViewInteractionPlan`, dispatches into
`ViewEventCx<UiCommand>`, and handles focused `WM_CHAR` text input for textbox
views. It also handles `WM_KEYDOWN` keyboard activation for focused button and
checkbox targets. Native smoke records emitted command ids, focus counts, text
character counts, selection counts, keydown counts and keyboard activation
counts. Checkbox clicks and Space-key activation route to typed `Toggled`
events and reusable `UiCommand`s when the checkbox feature is enabled. The
feature-gated `list` builder now supports typed row selection through child
IDs, and `native_smoke_run --view` can dispatch those selection messages into
reusable command IDs. Win32 Up/Down key routing can move selection between list
rows and records keyboard list selection in native smoke. Broader pointer
routing, IME/composition input and macOS/Linux input dispatch are still
pending.

## Agent Entry Points

- Skill folder for another AI: `docs/skills/zsui-native-ui/`
- Public API: `src/lib.rs`
- Rust-first goals: `src/framework_goals.rs`
- Rust-first goal narrative: `docs/framework-goals.md`
- Rust-first view API: `src/view.rs`
- Theme tokens and typed units: `src/style.rs`, `src/geometry.rs`
- Cargo features: `Cargo.toml`, `src/feature_manifest.rs`
- App declarations and declaration audit: `src/app.rs`, `src/window.rs`,
  `src/tray.rs`, `src/menu.rs`
- Component tree declarations: `src/components.rs`
- Capability model: `src/capability.rs`
- AI/agent context: `src/agent_context.rs`
- Minimal real native window: `src/native.rs`
- Extracted Windows self-draw sink: `src/windows_gdi_renderer.rs`
- Extracted Windows Win32 main/transient window host:
  `src/windows_win32_host.rs`
- Host contracts: `src/host.rs`, `src/host_protocol.rs`, `src/native_hosts.rs`,
  `src/native_host_actions.rs`
- Adapter discovery: `src/native_adapter_manifest.rs`
- Launch planning: `src/native_host_launch.rs`
- Mobile host scaffolds and bridge contracts: `src/mobile_host.rs`,
  `src/android_activity_host.rs`, `src/harmony_ability_host.rs`,
  `examples/mobile_scaffold_manifest.rs`
- Shared protocols: `src/geometry.rs`, `src/command_protocol.rs`,
  `src/event_protocol.rs`, `src/component_protocol.rs`,
  `src/control_protocol.rs`, `src/render_protocol.rs`,
  `src/ui_surface_protocol.rs`, `src/timer_protocol.rs`
- Product adapter/runtime harness: `src/product_adapter.rs`
- Product adapter examples: `examples/product_adapter.rs`,
  `examples/product_adapter_smoke.rs`,
  `examples/product_adapter_native_driver.rs`,
  `examples/product_adapter_view.rs`
- Rust-first API example: `examples/rust_first_view.rs`
- Typed list selection example: `examples/list_selection.rs`
- Native smoke manifests: `src/native_smoke.rs`,
  `examples/native_smoke_manifest.rs`, `examples/native_smoke_record.rs`,
  `examples/native_smoke_run.rs`, `examples/native_smoke_review.rs`,
  `docs/native-host-smoke.md`
- Architecture docs: `docs/architecture.md`
- Porting docs: `docs/porting.md`

## Source Material From ZSClip

The original AI handoff material in ZSClip lives under
`docs/skills/zsclip-native-ui/`. The standalone ZSUI copy keeps the same intent
but removes ZSClip product behavior from the instructions:

- `docs/skills/zsui-native-ui/SKILL.md`
- `docs/skills/zsui-native-ui/references/native-ui-entrypoints.md`
- `docs/skills/zsui-native-ui/agents/openai.yaml`

## Agent Rules

Keep ZSUI product-neutral. Do not add clipboard history storage, sync logic,
AI provider clients, prompt templates for a product, database schemas or ZSClip
window procedures to this crate.

Prefer adding reusable contracts and host adapters in ZSUI, then let products
bind their own data and behavior through adapters. Platform handles and native
objects may exist inside host implementations, but must not leak into
declaration models or shared protocols.

When adding a feature, update tests in the same crate. ZSUI should be verifiable
with:

```powershell
cargo test
```

## Runtime Roadmap

1. Connect the first-pass Rust-first user API layer (`View<Msg>`, typed
   messages, explicit contexts, typed units, theme tokens and strong IDs) to
   native host input/paint routing.
2. Preserve the one-line native-window path while proving Windows, macOS and
   Linux target smoke for that entry point.
3. Keep the default facade small and split heavier widget/backend families into
   feature-gated crates or modules as their contracts stabilize.
4. Make `NativeRuntimeDriver` and launch plans drive real host event loops.
5. Connect the extracted Win32 main-window/GDI/tray pieces to the default
   runtime, then continue AppKit/GTK extraction around status item, menu,
   dialog and clipboard contracts.
6. Turn the Android Activity and Harmony Ability bridge contracts into real
   FFI/runtime implementations with device smoke artifacts.
7. Expand the non-clipboard product adapter example into a target native smoke
   harness.
8. Expand host capability reporting so agents can choose supported APIs without
   reading platform code.

## Machine-Readable Progress

Use these public functions when another AI, tool or product adapter needs a
stable context without reading prose:

- `zsui_agent_context()`: full framework, platform, completion and gate context.
- `zsui_agent_context_json()`: JSON form of the same context.
- `zsui_reuse_readiness_report()`: compact platform/toolkit readiness summary.
- `zsui_reuse_bootstrap_plan(platform)`: one platform's adapter boundary,
  binding names and next runtime gate.
- `zsui_completion_areas()`: current standalone completion estimate by area.
- `zsui_rust_first_goals()`: the revised Rust-first design target list.
- `zsui_rust_first_goal_names()`: compact names for the Rust-first target list.
- `View<Msg>`, `ViewNode`, `WidgetId`, `AppCx`, `ViewEventCx`,
  `ViewInteractionPlan` and `ViewPaintCx`: first-pass typed view/message/
  hit-target/context API, including feature-gated list row selection.
- `Px`, `Dp`, `Dpi`, `UiLength` and `ZsuiTheme`: first-pass typed unit and
  theme-token API.
- `ProductViewAdapterHost` and `ProductViewRuntimeSmokeRequest`: smoke path for
  typed view messages through a product adapter and reusable runtime harness.
- `zsui_feature_manifest()`: Cargo feature graph for default, widget, service,
  platform and backend gates.
- `zsui_default_feature_names()`: current default feature list.
- `zsui_optional_dependency_feature_names()`: feature gates that pull optional
  dependencies into the build.
- `required_native_draw_command_operation_names()`: stable self-draw command
  sink operation names extracted from ZSClip's native painting plan shape.
- `AppBuilder::declaration_report()` and
  `AppBuilder::declaration_report_for(capabilities)`: structural declaration
  audit with errors, warnings and host degradation details.
- `zsui_declaration_audit_surface_names()`: machine-readable list of
  declaration surfaces currently covered by the audit.
- `mobile_runtime_host_scaffold(platform)`: Activity/Ability bridge,
  lifecycle, capability and device-smoke scaffold for Android or Harmony.
- `mobile_runtime_bridge_contract(platform)`: Android/Harmony FFI callback,
  lifecycle, surface, input, command and device-smoke artifact contract.
- `mobile_runtime_bridge_contracts_json()`: JSON form of both mobile bridge
  contracts for AI/tool handoff.
- `mobile_runtime_device_smoke_plan(platform)`: required Android/Harmony device
  artifact plan without faking target proof.
- `review_mobile_runtime_device_smoke_artifacts(platform)`: read-only verifier
  for mobile `manifest.json`, launch log, screenshot, lifecycle, surface and
  input artifacts.
- `product_adapter_reuse_checklist()`: surfaces, tasks and AI executor
  boundaries a product must provide.
- `zsui_reusable_runtime_harness_stage_names()`: reusable startup, command,
  event, AI and shutdown pipeline stages.
- `ProductAdapterRuntimeSmokeRequest`: exercises the reusable runtime harness
  and returns a `ProductAdapterRuntimeSmokeReport` for JSON evidence.
- `NativeWindowRuntimeDriver`: current desktop native-window driver bridge for
  running a product adapter through `ZsuiReusableRuntimeHarness`; it records
  projected main window, status item/menu and settings declarations through
  native host contract operations, including extracted status-menu command and
  settings-item update contracts.
- `native_host_smoke_plan(platform)`: target artifact manifest for proving a
  native backend beyond code-level readiness.
- `write_native_host_smoke_artifacts(platform)`: writes contract-level smoke
  artifacts and reports which required target artifacts, such as `window.png`,
  are still missing.
- `native_window(...).run_smoke(...)`: opens a real first-pass native window,
  auto-closes it and can capture `window.png` on Windows when
  `NativeWindowSmokeRunOptions::screenshot_file(...)` is set. The
  `native_smoke_run --view` example also records typed-view draw-plan command
  counts plus Win32 click/text/toggle/list-selection/keyboard and keyboard-list
  selection to `UiCommand` routing in the smoke report.
- `review_native_host_smoke_artifacts(platform)`: checks the artifact directory,
  validates JSON files and reports whether target smoke proof is complete.

## Completion Semantics

Treat a module as complete only when all of the following are true:

- It is public or intentionally internal through `src/lib.rs`.
- It has unit tests or examples covering the public behavior.
- It does not depend on ZSClip modules.
- It can be used from a standalone crate with `zsui` as the only UI dependency.
- Optional dependencies and advanced widgets are behind explicit Cargo features.
- It reports unsupported or partial platform behavior honestly instead of
  silently pretending to work.
