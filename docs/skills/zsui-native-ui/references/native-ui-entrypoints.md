# Native UI Entrypoints

This reference is the short map an AI agent should use before changing or
judging standalone ZSUI native UI work.

## What To Send Another AI

If the AI has repository access, send this skill folder:

- `docs/skills/zsui-native-ui/`

If the AI cannot load a skill folder, send these files together:

- `docs/skills/zsui-native-ui/SKILL.md`
- `docs/skills/zsui-native-ui/references/native-ui-entrypoints.md`
- `docs/ai-agent.md`
- `docs/architecture.md`
- `docs/porting.md`
- `docs/native-host-smoke.md`
- `src/lib.rs`
- `src/agent_context.rs`
- `src/components.rs`
- `src/host.rs`
- `src/host_protocol.rs`
- `src/native.rs`
- `src/native_hosts.rs`
- `src/native_adapter_manifest.rs`
- `src/native_host_launch.rs`
- `src/mobile_host.rs`
- `src/android_activity_host.rs`
- `src/harmony_ability_host.rs`
- `src/native_smoke.rs`
- `src/product_adapter.rs`
- `examples/declaration_audit.rs`
- `examples/native_smoke_manifest.rs`
- `examples/native_smoke_record.rs`
- `examples/native_smoke_run.rs`
- `examples/native_smoke_review.rs`
- `examples/mobile_scaffold_manifest.rs`
- `examples/product_adapter.rs`
- `examples/product_adapter_smoke.rs`
- `examples/product_adapter_native_driver.rs`

## Main Source Entrypoints

| Need | Start here |
| --- | --- |
| Public API and exports | `src/lib.rs` |
| AI/agent context | `src/agent_context.rs` |
| Current standalone completion | `docs/ai-agent.md` |
| Architecture boundary | `docs/architecture.md` |
| Host porting contract | `docs/porting.md` |
| Component tree declarations | `src/components.rs` |
| Minimal native window builder | `src/native.rs` |
| Generic host trait and test host | `src/host.rs` |
| Clipboard, dialog, shell-open, file-picker and service traits | `src/host_protocol.rs` |
| Runtime/window/settings/search host contracts | `src/native_hosts.rs` |
| Platform/toolkit capability metadata | `src/native_adapter_manifest.rs` |
| Target launch planning | `src/native_host_launch.rs` |
| Mobile host scaffolds | `src/mobile_host.rs`, `src/android_activity_host.rs`, `src/harmony_ability_host.rs` |
| Target smoke artifacts | `src/native_smoke.rs`, `docs/native-host-smoke.md` |
| Product adapter and runtime harness | `src/product_adapter.rs` |
| Declarations and audit | `src/app.rs`, `src/window.rs`, `src/tray.rs`, `src/menu.rs`, `src/settings.rs`, `examples/declaration_audit.rs` |
| Shared UI protocols | `src/geometry.rs`, `src/command_protocol.rs`, `src/event_protocol.rs`, `src/render_protocol.rs`, `src/control_protocol.rs`, `src/ui_surface_protocol.rs` |

## Feature Status Vocabulary

Use these states in reports:

- Code-level ready: Rust contracts, adapters and host callbacks exist; local
  tests pass.
- Target smoke verified: target OS or device run produced logs, screenshots or
  interaction artifacts.
- System complete: real OS behavior is proven, including permissions, focus,
  native lifecycle and desktop/mobile integration.

Do not collapse these states. For example, Android and Harmony can appear in
the capability model while still needing real Activity/Ability runtime hosts
before they are target-smoke verified.

## Current Standalone Shape

Already reusable at code level:

- Declaration-first app, window, tray, menu, hotkey, clipboard and settings
  specs.
- Structured declaration audit through `AppBuilder::declaration_report()`,
  `AppBuilder::declaration_report_for(...)`, `ZsuiAppDeclarationReport` and
  `examples/declaration_audit.rs`.
- Declarative `UiNode` component trees for text, buttons, inputs, checkboxes,
  stacks and spacers.
- `MemoryHost` for deterministic tests.
- `PlatformHost` scaffold and minimal `NativeWindowHost`.
- One-line desktop entry:
  `zsui::native_window("Example").size(900, 620).run()?`.
- Shared geometry, command, event, lifecycle, layout, component, render, host
  surface and native control protocols.
- Product-neutral native host contracts for dialogs, file pickers, shell-open,
  clipboard, popup/transient windows, IME, text caret, main/settings windows,
  search controls and runtime startup.
- Native backend metadata for Windows, macOS and Linux through the current
  `winit_desktop` runtime, plus Android and Harmony adapter scaffolds.
- Android Activity and Harmony Ability scaffold manifests with bridge entry
  points, lifecycle bindings, permissions and capability mappings.
- Machine-readable AI context through `zsui_agent_context()` and
  `zsui_agent_context_json()`.
- Product adapter and reusable runtime harness contracts through
  `ProductAdapterHost` and `ZsuiReusableRuntimeHarness`.
- Product adapter runtime harness smoke reports through
  `ProductAdapterRuntimeSmokeRequest` and `examples/product_adapter_smoke.rs`.
- Product adapter to native runtime bridge through `NativeWindowRuntimeDriver`
  and `examples/product_adapter_native_driver.rs`, including projected status
  item/menu and settings startup declarations routed through native host
  operations.
- Native host smoke manifest planning through `native_host_smoke_plan()` and
  `examples/native_smoke_manifest.rs`.
- Contract-level smoke artifact writing through
  `write_native_host_smoke_artifacts()` and `examples/native_smoke_record.rs`.
- Auto-closing first-pass native smoke windows through
  `native_window(...).run_smoke(...)` and `examples/native_smoke_run.rs`.
- Windows native smoke screenshot capture through
  `NativeWindowSmokeRunOptions::screenshot_file(...)`.
- Windows first-pass target smoke proof through
  `cargo run --example native_smoke_run -- windows` followed by
  `cargo run --example native_smoke_review -- windows`.
- Read-only target artifact review through
  `review_native_host_smoke_artifacts()` and
  `examples/native_smoke_review.rs`.

Still requiring extraction or target proof before system-complete claims:

- Full reusable Win32, AppKit and GTK host implementations split from ZSClip
  product behavior.
- Target screenshots and interaction artifacts under
  `target/native-host-smoke/<platform>/`.
- Target artifact review on each OS/device through `native_smoke_review`.
- Real tray/status, menu, dialog, file-picker, shell-open and clipboard monitor
  proof on each desktop target.
- Android Activity and Harmony Ability runtime host implementations.
- Android/Harmony FFI bridge code beyond the current scaffold manifests.
- Target smoke proof that a real native host can run through
  `ZsuiReusableRuntimeHarness`.
- Real product adapter smoke through a target native driver.
- Target artifacts proving the product adapter native-driver bridge on each OS.

## Editing Rules

- Add reusable behavior to ZSUI contracts and protocols first.
- Add only platform presentation and OS API handoff to native hosts.
- Prefer existing host traits, capability reports and adapter metadata before
  introducing a new abstraction.
- Add or update tests for shared routing and source guards when a new native
  host surface is introduced.
- Update `docs/ai-agent.md` when standalone completion or AI handoff guidance
  changes.
