# ZSUI AI Agent Guide

This document is the standalone ZSUI entry point for AI coding agents. It
describes what is reusable framework code, what is still only a contract, and
where agents should edit first.

## Current Completion

ZSUI is about 45% complete as a standalone UI framework.

- Foundation contracts: about 70% complete.
- Declaration API: about 72% complete.
- Minimal native window runtime: about 45% complete.
- Full desktop native hosts: about 15% complete.
- Android and Harmony: about 10% complete.
- Product adapter/runtime harness: about 57% complete.
- Native smoke verification: about 55% complete.

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
The current desktop backend metadata points to the actual `src/native.rs`
`winit_desktop` runtime. Direct Win32/AppKit/GTK files are not represented as
implemented ZSUI modules until they are really extracted.
`ProductUiProjection` now carries the main window, status item/tray menu and
settings pages into `NativeRuntimeStartupRequest`; `NativeWindowRuntimeDriver`
routes those declarations through `NativeStatusItemHost` and
`NativeSettingsPageModelHost`, then reports operation names, status menu counts
and settings page counts.
Windows first-pass target smoke has a local artifact path:
`cargo run --example native_smoke_run -- windows` captures `window.png`, and
`cargo run --example native_smoke_review -- windows` reports
`target_smoke_complete=true` when all six required artifacts are present.
macOS, Linux, Android and Harmony still require target/device proof.

## Agent Entry Points

- Skill folder for another AI: `docs/skills/zsui-native-ui/`
- Public API: `src/lib.rs`
- App declarations and declaration audit: `src/app.rs`, `src/window.rs`,
  `src/tray.rs`, `src/menu.rs`
- Component tree declarations: `src/components.rs`
- Capability model: `src/capability.rs`
- AI/agent context: `src/agent_context.rs`
- Minimal real native window: `src/native.rs`
- Host contracts: `src/host.rs`, `src/host_protocol.rs`, `src/native_hosts.rs`
- Adapter discovery: `src/native_adapter_manifest.rs`
- Launch planning: `src/native_host_launch.rs`
- Mobile host scaffolds: `src/mobile_host.rs`,
  `src/android_activity_host.rs`, `src/harmony_ability_host.rs`,
  `examples/mobile_scaffold_manifest.rs`
- Shared protocols: `src/geometry.rs`, `src/command_protocol.rs`,
  `src/event_protocol.rs`, `src/component_protocol.rs`,
  `src/control_protocol.rs`, `src/render_protocol.rs`,
  `src/ui_surface_protocol.rs`, `src/timer_protocol.rs`
- Product adapter/runtime harness: `src/product_adapter.rs`
- Product adapter examples: `examples/product_adapter.rs`,
  `examples/product_adapter_smoke.rs`,
  `examples/product_adapter_native_driver.rs`
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

1. Make `NativeRuntimeDriver` and launch plans drive real host event loops.
2. Add product-neutral Win32/AppKit/GTK host implementations around window,
   status item, menu, dialog and clipboard contracts.
3. Turn the Android Activity and Harmony Ability scaffold manifests into real
   FFI/runtime bridges.
4. Expand the non-clipboard product adapter example into a target native smoke
   harness.
5. Expand host capability reporting so agents can choose supported APIs without
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
- `AppBuilder::declaration_report()` and
  `AppBuilder::declaration_report_for(capabilities)`: structural declaration
  audit with errors, warnings and host degradation details.
- `zsui_declaration_audit_surface_names()`: machine-readable list of
  declaration surfaces currently covered by the audit.
- `mobile_runtime_host_scaffold(platform)`: Activity/Ability bridge,
  lifecycle, capability and device-smoke scaffold for Android or Harmony.
- `product_adapter_reuse_checklist()`: surfaces, tasks and AI executor
  boundaries a product must provide.
- `zsui_reusable_runtime_harness_stage_names()`: reusable startup, command,
  event, AI and shutdown pipeline stages.
- `ProductAdapterRuntimeSmokeRequest`: exercises the reusable runtime harness
  and returns a `ProductAdapterRuntimeSmokeReport` for JSON evidence.
- `NativeWindowRuntimeDriver`: current desktop native-window driver bridge for
  running a product adapter through `ZsuiReusableRuntimeHarness`; it records
  projected main window, status item/menu and settings declarations through
  native host contract operations.
- `native_host_smoke_plan(platform)`: target artifact manifest for proving a
  native backend beyond code-level readiness.
- `write_native_host_smoke_artifacts(platform)`: writes contract-level smoke
  artifacts and reports which required target artifacts, such as `window.png`,
  are still missing.
- `native_window(...).run_smoke(...)`: opens a real first-pass native window,
  auto-closes it and can capture `window.png` on Windows when
  `NativeWindowSmokeRunOptions::screenshot_file(...)` is set.
- `review_native_host_smoke_artifacts(platform)`: checks the artifact directory,
  validates JSON files and reports whether target smoke proof is complete.

## Completion Semantics

Treat a module as complete only when all of the following are true:

- It is public or intentionally internal through `src/lib.rs`.
- It has unit tests or examples covering the public behavior.
- It does not depend on ZSClip modules.
- It can be used from a standalone crate with `zsui` as the only UI dependency.
- It reports unsupported or partial platform behavior honestly instead of
  silently pretending to work.
