---
name: zsui-native-ui
description: Work on ZSUI's standalone Rust UI contracts and native Windows Win32, macOS AppKit/SwiftUI, lightweight Linux Wayland/X11 plus optional GTK/libadwaita compatibility, and Android Activity hosts. Use this when modifying or verifying native UI framework surfaces such as windows, tray/status menus, settings pages, clipboard, dialogs, file pickers, host capabilities, launch plans, target smoke tests, or when an AI agent needs to understand how to build on ZSUI without copying product behavior.
---

# ZSUI Native UI

Use this skill to work on the standalone ZSUI framework layer without turning a
platform host into a copy of a product application.

## Quick Start

1. Read `docs/ai-agent.md` only.
2. Select one task pack with `scripts/ai-context.ps1 -Pack <id>`.
3. Read only the pack's required paths and use `rg` inside them.
4. Load optional paths only when a concrete question remains unanswered.
5. Use `completion-audit` for framework-wide progress and
   `windows-renderer`/`desktop-hosts`/`mobile-hosts` for platform work.
6. Read `docs/native-host-smoke.md` only before a target-smoke or
   system-complete claim.
7. Do not bulk-load `references/native-ui-entrypoints.md`,
   `docs/ai/reference.md` or `src/agent_context.rs` for ordinary feature work.

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
  split crates/modules for heavy widget and backend families, Android
  host boundaries, buffered no-flicker rendering and no global widget
  registration.
- Do not report a platform feature as complete just because a declaration or
  scaffold compiles. Use code-level, target-smoke and system-complete
  separately.

## Common Workflow

1. Identify the feature surface: app declaration audit, Cargo feature gate,
   window, tray/status menu, menu, hotkey, clipboard, settings, generic
   navigation/card shell layout, conversation/task workbench, document-editor
   shell, calculator engine/shell, component catalog, dialog, shell-open, file
   picker, runtime launch, adapter metadata or mobile host.
2. Check the shared contract in `src/` before editing platform code.
   Use `AppBuilder::declaration_report_for(...)` when changing app, window,
   menu, tray, hotkey or settings declaration shapes.
   For live application UI, preserve the
   `stateful_view(State, view, update)` path through `SharedLiveViewRuntime`;
   native hosts should deliver typed events and request repaint, not own
   product state. Route `AppCx::command(...)` through an explicitly attached
   `SharedAppCommandExecutor`; never discard commands after recording counts.
   Route `AppCx::ui_command(...)` and command-backed View output through
   `SharedUiCommandExecutor`; use `ProductAdapterUiCommandExecutor` when the
   product implements `ProductAdapterHost`.
   For switch-style input, reuse `zs_toggle_render_plan(...)` from
   `src/widget_render.rs`; its geometry must stay shared with shell accessories.
   For desktop conversation/task applications, compose
   `ZsWorkbenchSpec` instead of creating product-specific navigation, message,
   tool-output, composer and inspector layout code. Keep product commands and
   persistence outside the workbench runtime.
   Built-in visuals must consume the shared Fluent tokens and semantic
   `ZsIcon` catalog. Do not add private PUA glyph strings, local palettes or
   duplicate control metrics to component modules.
   Reuse `ZsDocumentShellSpec` for text-oriented application chrome. Treat
   `examples/zsui_notepad.rs` as a shared self-drawn acceptance application,
   not as the source of reusable editor, file-dialog, accelerator or
   document-lifecycle architecture. The Windows native editor service remains
   optional and must not replace the shared application path.
   Reuse `ZsCalculatorEngine` and `ZsCalculatorShellSpec` for standard decimal
   calculator behavior and presentation. Keep scientific/conversion modes and
   product-specific commands outside that shell until their contracts exist.
3. For Android, inspect `mobile_runtime_host_scaffold(platform)` and
   `mobile_runtime_bridge_contract(platform)` before editing Activity
   bridge code. Use `mobile_runtime_bridge_parity_report(platform)` to check
   required callback route coverage and pending FFI symbols. Use
   `mobile_runtime_bridge_dispatch_report(platform)` to check how required
   callback symbols map to lifecycle, surface, typed input and runtime driver
   operations. Use `mobile_runtime_bridge_contract_smoke_report(platform)` for
   local contract dispatch smoke before claiming device proof. Use
   `write_mobile_runtime_bridge_contract_artifacts(platform)` to record local
   contract artifacts, device-smoke plan and agent context without fabricating
   device evidence. Use
   `review_mobile_runtime_bridge_contract_artifacts(platform)` to validate
   local contract artifacts and expected JSON schemas separately from device
   proof. Use the `*_for_all` variants or CLI `all` target when updating
   the configured mobile target. Use
   `mobile_runtime_device_smoke_plan(platform)` and
   `review_mobile_runtime_device_smoke_artifacts(platform)` when changing
   mobile device proof requirements; device trace JSON must satisfy the
   device-sourced lifecycle, surface and input schemas. Use
   `mobile_runtime_device_smoke_trace_templates(platform)` or
   `mobile_scaffold_manifest --trace-template` to inspect the exact trace shape
   expected from a future Activity bridge.
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
`native_host_smoke_plan()` and `docs/ai/reference.md` as the detailed ZSUI
source of truth for progress. If the
current machine is Windows, say that macOS, Linux and Android runtime
proof still requires target artifacts.

## Verification

Local checks:

```powershell
cargo fmt --check
cargo run --quiet --example basic
.\scripts\ai-context.ps1 -Validate
cargo check
.\scripts\check-feature-matrix.ps1 -Locked
cargo test --features full
cargo test --no-default-features
cargo test --example zsui_notepad --no-default-features --features notepad-demo
cargo test --lib --no-default-features --features calculator calculator
cargo run --example zsui_calculator --no-default-features --features calculator-demo -- --smoke
```

On Windows, run `scripts/measure-notepad-comparison.ps1` only when changing the
notepad benchmark or making a size, memory or implementation-effort claim.
Run `scripts/measure-calculator-comparison.ps1` under the same conditions for
the calculator benchmark.

Target smoke checks are platform-specific and should store inspectable artifacts
under `target/native-host-smoke/<platform>/` before a platform is called
verified. Use `cargo run --example native_smoke_review -- <platform>` to check
for missing, empty or invalid target proof files.
