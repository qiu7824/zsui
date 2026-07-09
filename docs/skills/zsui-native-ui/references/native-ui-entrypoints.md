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
- `docs/framework-goals.md`
- `docs/porting.md`
- `docs/native-host-smoke.md`
- `Cargo.toml`
- `src/lib.rs`
- `src/agent_context.rs`
- `src/feature_manifest.rs`
- `src/framework_goals.rs`
- `src/style.rs`
- `src/view.rs`
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
- `examples/rust_first_view.rs`
- `examples/list_selection.rs`
- `examples/scroll_view.rs`
- `examples/native_smoke_manifest.rs`
- `examples/native_smoke_record.rs`
- `examples/native_smoke_run.rs`
- `examples/native_smoke_review.rs`
- `examples/mobile_scaffold_manifest.rs`
- `examples/product_adapter.rs`
- `examples/product_adapter_view.rs`
- `examples/product_adapter_smoke.rs`
- `examples/product_adapter_native_driver.rs`

## Main Source Entrypoints

| Need | Start here |
| --- | --- |
| Public API and exports | `src/lib.rs` |
| Cargo feature graph | `Cargo.toml`, `src/feature_manifest.rs` |
| Rust-first framework goals | `src/framework_goals.rs`, `docs/framework-goals.md` |
| Rust-first view API | `src/view.rs` |
| Theme tokens and typed units | `src/style.rs`, `src/geometry.rs` |
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
| Mobile host scaffolds and bridge contracts | `src/mobile_host.rs`, `src/android_activity_host.rs`, `src/harmony_ability_host.rs` |
| Target smoke artifacts | `src/native_smoke.rs`, `docs/native-host-smoke.md` |
| Product adapter and runtime harness | `src/product_adapter.rs` |
| Declarations and audit | `src/app.rs`, `src/window.rs`, `src/tray.rs`, `src/menu.rs`, `src/settings.rs`, `examples/declaration_audit.rs` |
| Rust-first view example | `examples/rust_first_view.rs` |
| Typed list selection example | `examples/list_selection.rs` |
| Scroll container example | `examples/scroll_view.rs` |
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
- Window icon path declarations through `WindowSpec::icon_path(...)`, including
  declaration audit and Win32 owned HICON loading.
- Structured declaration audit through `AppBuilder::declaration_report()`,
  `AppBuilder::declaration_report_for(...)`, `ZsuiAppDeclarationReport` and
  `examples/declaration_audit.rs`.
- Declarative `UiNode` component trees for text, buttons, inputs, checkboxes,
  stacks and spacers.
- `MemoryHost` for deterministic tests.
- `PlatformHost` scaffold and minimal `NativeWindowHost`.
- One-line desktop entry:
  `zsui::native_window("Example").size(900, 620).run()?`.
- Feature-gated build shape: default features are `window`, `button` and
  `label`; optional dependency features include `clipboard`, `image`,
  `desktop-winit` and `windows-gdi`; advanced widgets are opt-in. Cargo
  features are unified across the dependency graph, so larger widget/backend
  families should move toward split crates or modules as they stabilize.
- Revised Rust-first target manifest through `zsui_rust_first_goals()`:
  one-line native window entry points, composition and traits, typed messages,
  RAII native resources, reusable ZSClip no-flicker rendering foundations,
  typed units, compile-time builder constraints, explicit contexts, safe public
  APIs, explicit state, theme tokens, declarative Rust builders, `Result`
  errors, capability traits, Android/Harmony mobile host boundaries,
  feature-gated platform backends, split-crate/module trimming, platform API
  use on demand and strong typed IDs. The manifest records preferred and
  avoided API shapes;
  `docs/framework-goals.md` expands the target with examples and the
  feature/crate trimming policy.
- First-pass typed view API through `View<Msg>`, `ViewNode`, `WidgetId`,
  `ViewEventCx`, `ViewInteractionPlan`, `ViewPaintCx`, `AppCx`, typed view
  events, feature-gated scroll containers with typed `ScrollBy`/`on_scroll`
  routing, typed list selection, `Px`, `Dp`, `Dpi`, `UiLength` and
  `ZsuiTheme`.
- `NativeWindowBuilder::view(...)` projection from typed `ViewNode<Msg>` into
  `NativeDrawPlan`, with Windows smoke paint through the extracted no-flicker
  Win32/GDI path.
- `NativeWindowBuilder::ui_command_view(...)` routing from Win32
  `WM_LBUTTONUP` and focused `WM_CHAR` textbox input through
  `ViewEventCx<UiCommand>` into stable command ids, including checkbox
  `Toggled` events, Tab focus traversal, list row selection, Up/Down keyboard
  list navigation and focused `WM_KEYDOWN` Enter/Space activation when the
  relevant widget features are enabled.
- Product adapter view smoke through `ProductViewAdapterHost`,
  `ProductViewRuntimeSmokeRequest` and `examples/product_adapter_view.rs`.
- Shared geometry, command, event, lifecycle, layout, component, render,
  self-draw command, host surface and native control protocols.
- Product-neutral native host contracts for dialogs, file pickers, shell-open,
  clipboard, popup/transient windows, IME, text caret, main/settings windows,
  search controls and runtime startup.
- Native backend metadata for Windows through the extracted `win32_gdi`
  runtime, macOS/Linux through the current `winit_desktop` runtime, plus
  Android and Harmony adapter scaffolds.
- Android Activity and Harmony Ability scaffold manifests with bridge entry
  points, lifecycle bindings, permissions and capability mappings.
- Android/Harmony bridge contracts through `mobile_runtime_bridge_contract()`,
  including FFI callback symbols, lifecycle/surface/input/command routes,
  safety rules and required device-smoke artifact names.
- Android/Harmony bridge parity reports through
  `mobile_runtime_bridge_parity_report()` and
  `mobile_scaffold_manifest --parity` to check scaffold/contract metadata,
  required callback route coverage and pending FFI symbols without claiming
  runtime readiness.
- Android/Harmony bridge dispatch reports through
  `mobile_runtime_bridge_dispatch_report()` and
  `mobile_scaffold_manifest --dispatch` to map required callback symbols to
  lifecycle, surface, typed input and `NativeRuntimeDriver` operations.
- Android/Harmony contract dispatch smoke through
  `mobile_runtime_bridge_contract_smoke_report()` and
  `mobile_scaffold_manifest --dispatch-smoke` to replay the declared bridge
  sequence without faking device proof.
- Android/Harmony local contract artifact writing through
  `write_mobile_runtime_bridge_contract_artifacts()` and
  `mobile_scaffold_manifest --write-contract` to capture local bridge reports,
  the device-smoke plan and agent context without generating launch,
  screenshot, lifecycle, surface or input proof.
- Android/Harmony local contract artifact review through
  `review_mobile_runtime_bridge_contract_artifacts()` and
  `mobile_scaffold_manifest --review-contract`, including expected JSON schema
  checks and separate from device-smoke proof review. The write/review contract
  paths support an `all` target for Android and Harmony together.
- Android/Harmony device-smoke plans and read-only review through
  `mobile_runtime_device_smoke_plan()` and
  `review_mobile_runtime_device_smoke_artifacts()`, including device-sourced
  schema checks for lifecycle, surface and input traces.
- Android/Harmony device trace templates through
  `mobile_runtime_device_smoke_trace_templates()` and
  `mobile_scaffold_manifest --trace-template`, so Activity/Ability bridge code
  can write reviewable lifecycle, surface and input artifacts.
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
- ZSClip self-draw paint-plan shape extracted as product-neutral
  `NativeDrawPlan`, `NativeDrawCommand` and `NativeDrawCommandSink` contracts.
- ZSClip Windows GDI renderer/text layout/draw sink extracted in
  `src/windows_gdi_renderer.rs`.
- Win32/GDI buffered paint, window HDC, compatible memory DC,
  smoke-screenshot HBITMAP, owned main/quick HWND, owned HICON app-icon
  resources loaded from icon paths, brush, pen, font and selected-object cleanup
  now use internal RAII wrappers.
- Win32 tray/status item RAII surface through `WindowsWin32OwnedTrayIcon` and
  `WindowsWin32StatusItemHost`, backed by `Shell_NotifyIconW`, now wired into
  the direct Windows `NativeWindowHost` path and optional `native_smoke_run
  --tray` status-item smoke with native command-id table routing plus RAII
  popup-menu creation/cleanup evidence and `TrackPopupMenu` selection routing.
- ZSClip Win32 main/quick/transient window host style mapping, create-params,
  message-loop wrapper and `NativeMainWindowHost`/`NativeTransientWindowHost`
  implementations extracted in `src/windows_win32_host.rs`.
- Win32 native paint can attach `NativeDrawPlan` content to an `HWND` and render
  it through the ZSClip no-flicker buffered GDI path.
- Native host smoke manifest planning through `native_host_smoke_plan()` and
  `examples/native_smoke_manifest.rs`.
- Contract-level smoke artifact writing through
  `write_native_host_smoke_artifacts()` and `examples/native_smoke_record.rs`.
- Auto-closing first-pass native smoke windows through
  `native_window(...).run_smoke(...)` and `examples/native_smoke_run.rs`.
- Dedicated Win32 typed scroll smoke through
  `cargo run --features "scroll,label" --example native_smoke_run -- windows --scroll-view`.
- Windows native smoke screenshot capture through
  `NativeWindowSmokeRunOptions::screenshot_file(...)`.
- Windows first-pass target smoke proof through
  `cargo run --example native_smoke_run -- windows` followed by
  `cargo run --example native_smoke_review -- windows`.
- Read-only target artifact review through
  `review_native_host_smoke_artifacts()` and
  `examples/native_smoke_review.rs`.

Still requiring extraction or target proof before system-complete claims:

- Connecting richer input/menu events to the extracted Win32 main-window host.
- Full reusable AppKit and GTK host implementations split from ZSClip product
  behavior.
- Moving heavier widgets into separate crates or fully gated modules and adding
  feature-matrix CI.
- Connecting the first-pass `View<Msg>` layer to native host input/paint
  routing, keeping raw HWNDs out of higher-level APIs, completing Px/Dp/Dpi
  migration and adding typestate builders while preserving the one-line native
  window entry point. Native paint routing has a first pass; Win32
  `WM_LBUTTONUP` and focused `WM_CHAR` textbox routing into
  `ViewEventCx<UiCommand>` exist, plus checkbox toggle routing and focused
  `WM_KEYDOWN` keyboard activation; Tab focus traversal, feature-gated list
  selection and Up/Down keyboard list navigation can dispatch through the same
  route. Full pointer/IME coverage and non-Windows input dispatch are still
  pending.
- Scroll containers now offset child layout, clip hit targets to the viewport
  and emit `PushClip`/`PopClip` draw commands. Win32 mouse-wheel input can
  route to a scroll target at code level and is covered by the dedicated
  `--scroll-view` smoke path; touch/inertial scroll is still pending.
- Target screenshots, tray/menu proof and interaction artifacts under
  `target/native-host-smoke/<platform>/`.
- Target artifact review on each OS/device through `native_smoke_review`.
- Real tray/status, menu, dialog, file-picker, shell-open and clipboard monitor
  proof on each desktop target.
- Android Activity and Harmony Ability runtime host implementations.
- Android/Harmony FFI bridge implementations and real device artifacts beyond
  the current bridge contracts/reviewers.
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
