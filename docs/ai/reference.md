# ZSUI Full AI Reference

This is an optional deep reference for completion/readiness audits. Normal
tasks start at `docs/ai-agent.md` and load one task pack from
`docs/ai/context-packs.json`; they should not load this file by default.

## Current Completion

ZSUI is roughly 60% complete as a standalone framework product. Component-level
milestones must not be used as overall framework readiness.

- Foundation contracts: about 78% complete.
- Declaration API: about 84% complete.
- Component library: about 42% complete (20 first-pass runtime surfaces out of
  48 catalogued component families).
- Minimal native window runtime: about 86% complete.
- Feature-pruned architecture: about 51% complete.
- Rust-first API model: about 88% complete.
- Full desktop native hosts: about 66% complete.
- Android and Harmony: about 32% complete.
- Product adapter/runtime harness: about 67% complete.
- Native smoke verification: about 82% complete.

The Windows implementation is further ahead than the overall
framework: its window, draw-plan, stateful View and shell-layout foundation is
roughly 75%
ready. macOS/Linux native product hosts and real Android/Harmony runtimes keep
cross-platform product readiness substantially lower. Report these separately.

The machine-readable audit tracks 18 required native capabilities per platform:

- Windows: 2 ready, 6 first-pass runtime implementations, 10 contract-only.
- macOS: 0 ready, 2 first-pass runtime implementations, 16 contract-only.
- Linux: 0 ready, 2 first-pass runtime implementations, 16 contract-only.
- Android: 0 runtime implementations, 18 contract-only.
- Harmony: 0 runtime implementations, 18 contract-only.

Use `native_ui_platform_readiness_reports()` for current capability-level
evidence instead of inferring platform completeness from backend registration.
Use `zsui_component_catalog_summary()` for component coverage: 20 families have
a first-pass runtime surface, 8 are contract-only and 20 are not started. A
composite workbench does not make its underlying missing controls complete.

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

It is not yet a complete application UI runtime. Complete AppKit and GTK hosts
are not implemented. The current Windows backend metadata points to the
`win32_gdi` runtime. Win32 main/quick window style, transient-window host,
create-params, message-loop and `NativeMainWindowHost` implementation live in
`src/windows_win32_host.rs` and are wired into the default
`native_window(...).run()` path on Windows.
`src/native_host_actions.rs` defines the native host action/status/settings
command contracts. `ProductUiProjection` now carries the main window, status item/tray menu
and settings pages into `NativeRuntimeStartupRequest`; `NativeWindowRuntimeDriver`
routes those declarations through `NativeStatusItemHost` and
`NativeSettingsPageModelHost`, dispatches status menu commands through
`NativeStatusMenuCommandHost`, updates bound settings item values through
`NativeSettingsItemUpdateHost`, then reports operation names, status menu counts
and settings page counts.
The self-drawn runtime uses a reusable command-plan shape (`FillRect`,
`RoundRect`, `RoundFill`, text commands and icon commands). ZSUI exposes it in
`src/render_protocol.rs` as
`NativeDrawPlan`, `NativeDrawCommand` and `NativeDrawCommandSink`, and
`src/windows_gdi_renderer.rs` contains the Windows GDI renderer, text-layout
sink and buffered no-flicker paint pipeline.
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
has bridge parity reports through `mobile_runtime_bridge_parity_report(platform)`
and `examples/mobile_scaffold_manifest.rs --parity <platform>` so agents can
check scaffold/contract metadata, required callback route kinds and pending FFI
symbols without treating the mobile host as implemented. It also has bridge
dispatch reports through `mobile_runtime_bridge_dispatch_report(platform)` and
`examples/mobile_scaffold_manifest.rs --dispatch <platform>` mapping each
required callback symbol to the runtime operation it must call, including
`start_runtime`, lifecycle/surface handlers, typed UI input,
`dispatch_ui_command`, `poll_application_event` and `request_shutdown`.
Contract-level dispatch smoke through
`mobile_runtime_bridge_contract_smoke_report(platform)` and
`examples/mobile_scaffold_manifest.rs --dispatch-smoke <platform>` replays the
declared callback sequence and verifies required dispatch-operation coverage
without pretending a device or FFI runtime exists.
`write_mobile_runtime_bridge_contract_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --write-contract <platform>` write the
local contract artifacts (`manifest.json`, bridge contract, parity, dispatch,
dispatch-smoke, `device-smoke-plan.json` and `agent-context.json`) while
deliberately leaving real device artifacts such as launch logs, screenshots
and lifecycle traces missing.
`review_mobile_runtime_bridge_contract_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --review-contract <platform>` validate
those local contract artifacts and their expected JSON schema separately from
real device smoke. Both
`--write-contract all <root>` and `--review-contract all <root>` cover Android
and Harmony in one command. It has
device-smoke plans and read-only artifact review through
`mobile_runtime_device_smoke_plan(platform)`,
`mobile_runtime_device_smoke_trace_templates(platform)`,
`review_mobile_runtime_device_smoke_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --trace-template <platform>` /
`--review <platform>`. Device review now requires expected JSON schemas for
manifest, lifecycle, surface and input traces so contract-only JSON cannot pass
as real device proof, and the trace-template command gives mobile bridge code a
machine-readable shape to write. These are still contracts and verifiers, not
native FFI implementations or device proof.
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
IDs. It also records the larger framework target: keep
`zsui::native_window(...).run()?` as the normal native-window entry, make
buffered no-flicker self-draw the Windows baseline, treat Android
and Harmony as real Activity/Ability host targets, and add `windows-rs` or
other broader platform bindings only for specific backend work. The source
target records the preferred and avoided API shapes, such as `enum Msg` over
string events and feature/crate based trimming over global widget registration.
The first concrete Rust-first API pass now exists in `src/view.rs`,
`src/style.rs` and `src/geometry.rs`: `View<Msg>`, typed event messages,
`WidgetId`, `AppCx`, `ViewEventCx`, `ViewPaintCx`, `ViewInteractionPlan`,
typed list selection, a feature-gated `scroll` container with typed scroll
events, clipped hit targets and `PushClip`/`PopClip` drawing, `Px`, `Dp`,
`Dpi`, `UiLength`, `ZsuiTheme` and theme tokens.
The generic WinUI-style self-drawn layout contract now lives in
`src/shell_layout.rs`. It owns the shared nav width, content offsets, card
spacing, viewport mask, scrollbar metrics and form-row geometry. It is
not tied to settings storage: agents can declare a left navigation pane, right
content header, grouped cards, content rows, description text, row accessories
such as values/toggles/buttons/dropdowns and an action-button area through
`ZsShellLayoutSpec` or `ZsNavigationScaffoldSpec`. The module audits the
layout, computes stable regions and projects the result into a product-neutral
`NativeDrawPlan` for the same no-flicker native painting path.
The optional `workbench` feature in `src/workbench.rs` adds a reusable
conversation/task workspace. It covers collapsible navigation, grouped history,
user/assistant/system/tool message roles, paragraph/code/tool/notice blocks,
message actions, composer controls and an optional inspector. The first pass is
DPI-aware and exposes draw plans, hit regions, bounded scrolling and local
selection state. `NativeWindowBuilder::workbench(...)` renders it on the native
window path; full Win32 event-loop routing and editable composer input remain
explicit gaps.
Its built-in visuals use the Fluent token definitions in `src/style.rs` and
semantic `ZsIcon` commands from `src/icon.rs`. Agents must not add PUA glyph
strings, private component palettes or arbitrary control dimensions to this
module. Windows resolves semantic icons through Segoe Fluent Icons; GTK theme
names and SF Symbol names are catalogued for future native bindings. Dark/high
contrast smoke and complete hover/pressed/focus-visible state coverage are not
yet complete.
`src/component_catalog.rs` tracks 48 WinUI-style component families: 20 have a
first-pass runtime surface, 8 are contract-only and 20 are not started.
`src/document_shell.rs` is the reusable visual boundary used by the Windows
notepad benchmark. It provides a document tab, command bar, editor frame,
status layout, semantic draw plan and hit regions without owning product state
or raw platform handles. The native editor, file dialog, accelerator and
lifecycle code still lives in the example's platform module. Use
`docs/notepad-demo.md` and
`scripts/measure-notepad-comparison.ps1` to judge output size, memory and AI
implementation effort without upgrading the native text-editor or file-dialog
services to complete.
The optional `calculator` feature in `src/calculator.rs` is a second runnable
application slice. `ZsCalculatorEngine` provides decimal arithmetic, typed
actions, memory and history; `ZsCalculatorShellSpec` provides DPI-aware Fluent
layout, semantic draw commands and hit regions. The Windows example proves
mouse, keyboard, icon, DPI and buffered-paint behavior. It does not make the
scientific, programmer, graphing, conversion, localization or accessibility
surfaces complete. Use `docs/calculator-demo.md` and
`scripts/measure-calculator-comparison.ps1` for its measured local comparison.
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
views. It also handles `WM_KEYDOWN` keyboard activation for focused button,
checkbox and toggle targets, and Tab focus traversal through ordered
`ViewInteractionPlan` targets. Native smoke records emitted command ids, focus
counts, keyboard focus traversal counts, text character counts, selection
counts, keydown counts and keyboard activation counts. Checkbox clicks and
Space-key activation route to typed `Toggled` events and reusable `UiCommand`s
when the checkbox feature is enabled. The feature-gated `list` builder now
supports typed row selection through child IDs, and `native_smoke_run --view`
can dispatch those selection messages into reusable command IDs. Win32 Up/Down
key routing can move selection between list rows and records keyboard list
selection in native smoke. Win32 `WM_MOUSEWHEEL` can route into typed
`ScrollBy` events for `scroll` containers and reusable command IDs. Broader
pointer routing, touch/inertial scroll, IME/composition input and macOS/Linux
input dispatch are still pending.
The feature-gated `toggle(...)` widget reuses `ZsToggleRenderPlan`. The same track/knob/DPI
geometry drives Shell accessories and normal View painting, while Win32 click
and Space activation emit typed `Toggled` messages. The stateful native smoke
captures the checked rendering after a real click.
`NativeWindowBuilder::stateful_view(...)` now owns the first real typed
application loop. It stores user state behind a safe shared runtime, turns
native input into `Msg`, calls the user `update(&mut State, Msg, &mut AppCx)`,
rebuilds the View and replaces the Win32 buffered draw plan. Native smoke now
records live revisions and application-command results;
`SharedAppCommandExecutor` now hands `AppCx::command(...)` to an explicitly
composed executor, and `NativeWindowRuntimeDriver` implements that contract.
`SharedUiCommandExecutor` does the same for both static command Views and
`AppCx::ui_command(...)`; `ProductAdapterUiCommandExecutor` is the standard
product bridge. `examples/rust_first_view.rs` proves one app command and one UI
command execute successfully with emitted events and zero unhandled commands in
a real interactive Win32 run.
`typed_native_window(...)` adds the first compile-time builder constraint:
content attachment changes `NativeWindowContentMissing` into
`NativeWindowContentReady`, and a compile-fail doctest proves that the missing
state cannot call `run`. The original one-line builder remains unchanged.

## Agent Entry Points

- Minimal AI bootstrap: `docs/ai-agent.md`
- Task context routing: `docs/ai/context-packs.json`,
  `scripts/ai-context.ps1`
- Skill folder for another AI: `docs/skills/zsui-native-ui/`
- Demo and comparison gallery: `docs/gallery.md`
- Public API: `src/lib.rs`
- Rust-first goals: `src/framework_goals.rs`
- Rust-first goal narrative: `docs/framework-goals.md`
- Rust-first view API: `src/view.rs`
- Reusable widget geometry: `src/widget_render.rs`
- AppCx/UI command executor boundaries: `src/app_command.rs`,
  `src/command_protocol.rs`
- WinUI-style navigation/card shell layout API: `src/shell_layout.rs`
- Conversation/task workbench API: `src/workbench.rs`
- Document editor shell API: `src/document_shell.rs`
- Component readiness catalog: `src/component_catalog.rs`
- Notepad integration benchmark: `docs/notepad-demo.md`,
  `examples/zsui_notepad.rs`, `scripts/measure-notepad-comparison.ps1`
- Calculator engine, shell and benchmark: `src/calculator.rs`,
  `docs/calculator-demo.md`, `examples/zsui_calculator.rs`,
  `scripts/measure-calculator-comparison.ps1`
- Theme tokens and typed units: `src/style.rs`, `src/geometry.rs`
- Cargo features: `Cargo.toml`, `src/feature_manifest.rs`
- Feature matrix gate: `scripts/check-feature-matrix.ps1`,
  `.github/workflows/ci.yml`
- App declarations and declaration audit: `src/app.rs`, `src/window.rs`,
  `src/tray.rs`, `src/menu.rs`
- Component tree declarations: `src/components.rs`
- Capability model: `src/capability.rs`
- AI/agent context: `src/agent_context.rs`
- Minimal real native window: `src/native.rs`
- Windows self-draw sink: `src/windows_gdi_renderer.rs`
- Windows Win32 main/transient window host:
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
- Navigation/card shell layout example: `examples/navigation_shell_layout.rs`
- Native smoke manifests: `src/native_smoke.rs`,
  `examples/native_smoke_manifest.rs`, `examples/native_smoke_record.rs`,
  `examples/native_smoke_run.rs`, `examples/native_smoke_review.rs`,
  `docs/native-host-smoke.md`
- Architecture docs: `docs/architecture.md`
- Porting docs: `docs/porting.md`

## Agent Handoff

The default handoff is `docs/ai-agent.md` plus one selected context pack:

```powershell
.\scripts\ai-context.ps1 -Pack <id> -Format Paths
```

The detailed native-host workflow remains available at:

- `docs/skills/zsui-native-ui/SKILL.md`
- `docs/skills/zsui-native-ui/references/native-ui-entrypoints.md`
- `docs/skills/zsui-native-ui/agents/openai.yaml`

## Agent Rules

Keep ZSUI product-neutral. Do not add clipboard history storage, sync logic,
AI provider clients, prompt templates for a product, database schemas or
application window procedures to this crate.

Prefer adding reusable contracts and host adapters in ZSUI, then let products
bind their own data and behavior through adapters. Platform handles and native
objects may exist inside host implementations, but must not leak into
declaration models or shared protocols.

When adding a feature, update tests in the same crate. ZSUI should be verifiable
with:

```powershell
.\scripts\ai-context.ps1 -Validate
.\scripts\check-feature-matrix.ps1 -Locked
cargo test --features full
cargo test --no-default-features
cargo test --example zsui_notepad --no-default-features --features notepad-demo
cargo test --lib --no-default-features --features calculator calculator
cargo run --example zsui_calculator --no-default-features --features calculator-demo -- --smoke
```

## Runtime Roadmap

1. Connect the first-pass Rust-first user API layer (`View<Msg>`, typed
   messages, explicit contexts, typed units, theme tokens and strong IDs) to
   native host input/paint routing.
2. Preserve the one-line native-window path while proving Windows, macOS and
   Linux target smoke for that entry point.
3. Keep the default facade small and split heavier widget/backend families into
   feature-gated crates or modules as their contracts stabilize.
4. Connect `ZsWorkbenchRuntime` hit regions, scrolling and composer input to
   real host event loops, then generalize the reusable composite-control route.
5. Complete Win32 main-window/GDI/tray integration, then implement AppKit/GTK
   status item, menu, dialog, clipboard, rendering and input capabilities.
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
- `native_ui_platform_readiness_reports()`: capability-level runtime evidence
  and contract-only gaps for all five target platforms.
- `zsui_component_catalog()` / `zsui_component_catalog_summary()`:
  component-level runtime, contract-only and not-started counts.
- `zsui_agent_context_json()`: JSON form of the same context.
- `zsui_reuse_readiness_report()`: compact platform/toolkit readiness summary.
- `zsui_reuse_bootstrap_plan(platform)`: one platform's adapter boundary,
  binding names and next runtime gate.
- `zsui_completion_areas()`: current standalone completion estimate by area.
- `zsui_rust_first_goals()`: the revised Rust-first design target list.
- `zsui_rust_first_goal_names()`: compact names for the Rust-first target list.
- `View<Msg>`, `ViewNode`, `WidgetId`, `AppCx`, `ViewEventCx`,
  `ViewInteractionPlan` and `ViewPaintCx`: first-pass typed view/message/
  hit-target/context API, including feature-gated scroll containers and list
  row selection.
- `SharedLiveViewRuntime`, `LiveViewUpdate` and
  `NativeWindowBuilder::stateful_view(...)`: typed application state/update/
  repaint loop for the direct Win32 host.
- `Px`, `Dp`, `Dpi`, `UiLength` and `ZsuiTheme`: first-pass typed unit and
  theme-token API.
- `ZsShellLayoutSpec` / `ZsNavigationScaffoldSpec`: product-neutral
  WinUI-style left-nav/right-content layout with grouped cards, content rows,
  description text, row
  accessories, action buttons, audit output, stable layout regions, viewport
  masks, scrollbar plans and `NativeDrawPlan` projection.
- `ZsWorkbenchSpec`, `ZsWorkbenchRuntime` and
  `NativeWindowBuilder::workbench(...)`: reusable navigation, message timeline,
  composer and inspector composite surface with DPI-aware layout and hit regions.
- `ZsDocumentShellSpec` and `ZsDocumentShellLayout`: reusable document tab,
  command bar, native-editor inset, status surface, semantic draw plan and
  command hit regions.
- `ZsCalculatorEngine` and `ZsCalculatorShellSpec`: feature-gated decimal
  calculator state, typed actions, memory/history, Fluent draw plan and hit
  regions without product or raw-window ownership.
- `ProductViewAdapterHost` and `ProductViewRuntimeSmokeRequest`: smoke path for
  typed view messages through a product adapter and reusable runtime harness.
- `zsui_feature_manifest()`: Cargo feature graph for default, widget, service,
  platform and backend gates.
- `zsui_default_feature_names()`: current default feature list.
- `zsui_optional_dependency_feature_names()`: feature gates that pull optional
  dependencies into the build.
- `required_native_draw_command_operation_names()`: stable self-draw command
  sink operation names used by native renderers.
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
- `mobile_runtime_bridge_parity_report(platform)`: Android/Harmony scaffold vs
  contract report covering required callback route kinds, pending FFI callback
  symbols and whether device smoke is still blocked.
- `mobile_runtime_bridge_parity_reports_json()`: JSON form of both mobile
  parity reports for AI/tool handoff.
- `mobile_runtime_bridge_dispatch_report(platform)`: Android/Harmony callback
  dispatch report mapping required bridge symbols to lifecycle, surface, input
  and `NativeRuntimeDriver` operations without claiming FFI implementation.
- `mobile_runtime_bridge_dispatch_reports_json()`: JSON form of both mobile
  dispatch reports for AI/tool handoff.
- `mobile_runtime_bridge_contract_smoke_report(platform)`: local Android/
  Harmony contract smoke that replays required bridge dispatch steps and reports
  dispatch-operation coverage without claiming device proof.
- `mobile_runtime_bridge_contract_smoke_reports_json()`: JSON form of both
  mobile contract dispatch smoke reports for AI/tool handoff.
- `write_mobile_runtime_bridge_contract_artifacts(platform)`: writes local
  Android/Harmony bridge contract artifacts, the device-smoke plan and the
  current ZSUI agent context without generating device launch, screenshot,
  lifecycle, surface or input proof.
- `review_mobile_runtime_bridge_contract_artifacts(platform)`: validates local
  Android/Harmony bridge contract artifacts and expected JSON schema without
  claiming device proof.
- `write_mobile_runtime_bridge_contract_artifacts_for_all()` and
  `review_mobile_runtime_bridge_contract_artifacts_for_all()`: write/review
  both Android and Harmony local contract artifacts in one call.
- `mobile_runtime_device_smoke_plan(platform)`: required Android/Harmony device
  artifact plan without faking target proof.
- `mobile_runtime_device_smoke_trace_templates(platform)`: lifecycle, surface,
  input and optional clipboard/pasteboard trace JSON templates expected by
  mobile device-smoke review.
- `review_mobile_runtime_device_smoke_artifacts(platform)`: read-only verifier
  for mobile `manifest.json`, launch log, screenshot, lifecycle, surface and
  input artifacts, including schema checks that require device-sourced trace
  JSON.
- `product_adapter_reuse_checklist()`: surfaces, tasks and AI executor
  boundaries a product must provide.
- `zsui_reusable_runtime_harness_stage_names()`: reusable startup, command,
  event, AI and shutdown pipeline stages.
- `ProductAdapterRuntimeSmokeRequest`: exercises the reusable runtime harness
  and returns a `ProductAdapterRuntimeSmokeReport` for JSON evidence.
- `NativeWindowRuntimeDriver`: current desktop native-window driver bridge for
  running a product adapter through `ZsuiReusableRuntimeHarness`; it records
  projected main window, status item/menu and settings declarations through
  native host contract operations, including status-menu command and
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
  counts plus Win32 click/text/toggle/list-selection/keyboard, Tab focus
  traversal and keyboard-list selection to `UiCommand` routing in the smoke
  report. `NativeWindowSmokeRunOptions::native_view_scroll(...)` and the Win32
  input route can also record scroll counters when a smoke path supplies a
  scroll target; `native_smoke_run --scroll-view` exercises that path.
- `review_native_host_smoke_artifacts(platform)`: checks the artifact directory,
  validates JSON files and reports whether target smoke proof is complete.

## Completion Semantics

Treat a module as complete only when all of the following are true:

- It is public or intentionally internal through `src/lib.rs`.
- It has unit tests or examples covering the public behavior.
- It does not depend on application modules.
- It can be used from a standalone crate with `zsui` as the only UI dependency.
- Optional dependencies and advanced widgets are behind explicit Cargo features.
- It reports unsupported or partial platform behavior honestly instead of
  silently pretending to work.
