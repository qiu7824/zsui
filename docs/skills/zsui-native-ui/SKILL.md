---
name: zsui-native-ui
description: Work on ZSUI's standalone Rust UI contracts and native Windows Win32, macOS AppKit/SwiftUI, Linux GTK/libadwaita, Android Activity, and Harmony Ability hosts. Use this when modifying or verifying native UI framework surfaces such as windows, tray/status menus, settings pages, clipboard, dialogs, file pickers, host capabilities, launch plans, target smoke tests, or when an AI agent needs to understand how to build on ZSUI without copying product behavior.
---

# ZSUI Native UI

Use this skill to work on the standalone ZSUI framework layer without turning a
platform host into a copy of a product application.

## Quick Start

1. Read `references/native-ui-entrypoints.md` first for the file map and
   completion vocabulary.
2. Read `docs/ai-agent.md` for the current standalone completion estimate.
3. Read `docs/architecture.md` for the framework boundary.
4. Inspect `src/framework_goals.rs`, `docs/framework-goals.md`, `src/view.rs`
   and `src/style.rs` before changing user-facing API shape.
5. Read `docs/porting.md` before adding or changing host surfaces.
6. Read `docs/native-host-smoke.md` before claiming target-smoke or
   system-complete status.
7. Inspect the relevant Rust entry points instead of guessing from UI labels.

## Layer Rules

- Keep reusable declarations, host traits, action plans, protocols and adapter
  metadata in `src/`.
- Keep product behavior, storage, sync, prompt templates, AI provider clients
  and product-specific settings in the application crate.
- Keep platform hosts thin: create native windows/widgets, wire callbacks, call
  shared contracts and report real capabilities.
- Do not create product-specific APIs for behavior that belongs behind a
  product adapter.
- Keep the user-facing API aligned with `zsui_rust_first_goals()`: composition
  and traits, typed messages, RAII resources, typed units, explicit state and
  safe public APIs. Use `docs/framework-goals.md` for the fuller guidance on
  one-line native window entry points, typed messages, feature/crate trimming,
  split crates/modules for heavy widget and backend families, Android/Harmony
  host boundaries, ZSClip no-flicker rendering reuse and no global widget
  registration.
- Do not report a platform feature as complete just because a declaration or
  scaffold compiles. Use code-level, target-smoke and system-complete
  separately.

## Common Workflow

1. Identify the feature surface: app declaration audit, Cargo feature gate,
   window, tray/status menu, menu, hotkey, clipboard, settings, dialog,
   shell-open, file picker, runtime launch, adapter metadata or mobile host.
2. Check the shared contract in `src/` before editing platform code.
   Use `AppBuilder::declaration_report_for(...)` when changing app, window,
   menu, tray, hotkey or settings declaration shapes.
3. For Android or Harmony, inspect `mobile_runtime_host_scaffold(platform)` and
   `mobile_runtime_bridge_contract(platform)` before editing Activity/Ability
   bridge code. Use `mobile_runtime_bridge_parity_report(platform)` to check
   required callback route coverage and pending FFI symbols. Use
   `mobile_runtime_bridge_dispatch_report(platform)` to check how required
   callback symbols map to lifecycle, surface, typed input and runtime driver
   operations. Use `mobile_runtime_bridge_contract_smoke_report(platform)` for
   local contract dispatch smoke before claiming device proof. Use
   `write_mobile_runtime_bridge_contract_artifacts(platform)` to record local
   contract artifacts without fabricating device evidence. Use
   `review_mobile_runtime_bridge_contract_artifacts(platform)` to validate
   local contract artifacts separately from device proof. Use the `*_for_all`
   variants or CLI `all` target when updating Android and Harmony together. Use
   `mobile_runtime_device_smoke_plan(platform)` and
   `review_mobile_runtime_device_smoke_artifacts(platform)` when changing
   mobile device proof requirements.
4. Edit platform code only for native presentation or OS service calls.
5. Route behavior through public contracts such as `ZsuiHost`,
   `NativeRuntimeDriver`, `NativeMainWindowHost`, `NativeDialogHost`,
   `NativeFileDialogHost`, `ClipboardHost` and `HostCapabilities`.
6. For product adapter changes, run or update `examples/product_adapter_smoke.rs`
   so startup, command, event, AI and shutdown routing remain proven.
7. When product adapter work touches native startup, also run or update
   `examples/product_adapter_native_driver.rs`.
8. Update docs and source guards when a new host surface, smoke log or platform
   proof expectation is added.
9. Run local Rust checks, then require target OS smoke artifacts before marking
   a backend runtime complete.

## Completion Reporting

When answering progress questions, separate:

- `code-level`: framework contract, adapter metadata or host route exists and
  local tests pass.
- `target-smoke`: the real target process produced logs, screenshots or
  interaction artifacts on the target OS/device.
- `system-complete`: the OS integration is proven, including permissions, focus
  handoff, tray/status behavior, dialogs, file pickers or mobile lifecycle where
  relevant.

Use `native_ui_backend_capability_matrix()`,
`native_ui_adapter_parity_report()`,
`mobile_runtime_bridge_parity_report()`,
`mobile_runtime_bridge_dispatch_report()`,
`mobile_runtime_bridge_contract_smoke_report()`,
`write_mobile_runtime_bridge_contract_artifacts()`,
`review_mobile_runtime_bridge_contract_artifacts()`,
`native_host_smoke_plan()` and `docs/ai-agent.md` as the current ZSUI source of
truth for progress. If the
current machine is Windows, say that macOS, Linux, Android and Harmony runtime
proof still requires target artifacts.

## Verification

Local checks:

```powershell
cargo fmt --check
cargo check
cargo check --no-default-features --features "button,label"
cargo test
```

Target smoke checks are platform-specific and should store inspectable artifacts
under `target/native-host-smoke/<platform>/` before a platform is called
verified. Use `cargo run --example native_smoke_review -- <platform>` to check
for missing, empty or invalid target proof files.
