<div align="center">

# ZSUI

**A lightweight, Rust-first native UI framework**

Compose with traits, route typed messages, and compile only the controls,
services, and platform backends an application enables.

[![CI](https://github.com/qiu7824/zsui/actions/workflows/ci.yml/badge.svg)](https://github.com/qiu7824/zsui/actions/workflows/ci.yml)
![Version](https://img.shields.io/badge/version-0.1.0-2f6fdf)
[![License](https://img.shields.io/github/license/qiu7824/zsui)](LICENSE)
![Core](https://img.shields.io/badge/core-Rust-dea584)
![Windows](https://img.shields.io/badge/Windows-Win32%20%2F%20GDI%2B-0078d4)
![Build](https://img.shields.io/badge/build-feature--gated-0f7b0f)

[简体中文](README.md) | **English**

</div>

ZSUI is a Rust-first native system UI framework.
It is intentionally declaration-first: application code describes windows,
tray/status menus, commands, hotkeys, settings pages and host capabilities in
Rust, while each platform host translates those declarations to Win32, AppKit,
GTK/libadwaita or mobile hosts.

ZSUI is not a browser shell and not yet a full self-drawn widget kit. Product
behavior stays in the product crate; ZSUI owns portable UI specs,
WinUI-like navigation/card layout contracts, command/event ids, capability
reporting and host traits.
Today the concrete runtime in this crate is the minimal `NativeWindowHost`.
On Windows it enters the Win32/GDI path from
`src/windows_win32_host.rs` and uses a buffered no-flicker paint pipeline:
`WM_ERASEBKGND` is suppressed and paint goes through a buffered top-down DIB
when available. macOS and Linux still use the `winit_desktop` first-pass
runtime. Complete AppKit and GTK host implementations are still pending.

<p align="center">
  <img src="docs/images/workbench.png" alt="ZSUI workbench" width="100%">
</p>

<table>
  <tr>
    <td width="68%"><img src="docs/images/notepad.png" alt="ZSUI Notepad"></td>
    <td width="32%"><img src="docs/images/calculator.png" alt="ZSUI Calculator"></td>
  </tr>
  <tr>
    <td align="center">Modern document shell with a native text service</td>
    <td align="center">Modern standard calculator</td>
  </tr>
</table>

<p align="center"><a href="docs/gallery.md"><b>Open the full demo and comparison gallery</b></a></p>

<details>
<summary><b>Show ZSUI / egui / Windows comparisons</b></summary>

<h4>Notepad</h4>
<table>
  <tr><th>ZSUI</th><th>eframe / egui</th><th>Windows Notepad</th></tr>
  <tr>
    <td><img src="docs/images/notepad.png" alt="ZSUI Notepad"></td>
    <td><img src="docs/images/notepad-egui.png" alt="egui Notepad"></td>
    <td><img src="docs/images/notepad-windows.png" alt="Windows Notepad"></td>
  </tr>
</table>

<h4>Calculator</h4>
<table>
  <tr><th>ZSUI</th><th>Windows Calculator</th></tr>
  <tr>
    <td><img src="docs/images/calculator.png" alt="ZSUI Calculator"></td>
    <td><img src="docs/images/calculator-windows.png" alt="Windows Calculator"></td>
  </tr>
</table>

</details>

## Quick Start

```rust
use zsui::{app, Command, MemoryHost, TraySpec, Window};

let mut host = MemoryHost::new();
let runtime = app("Example")
    .window(Window::new("Example").size(900, 620))
    .tray(
        TraySpec::new()
            .tooltip("Example")
            .item("Open", Command::ShowMainWindow)
            .separator()
            .item("Quit", Command::Quit),
    )
    .global_hotkey("Alt+V", Command::OpenQuickPanel)
    .run_with_host(&mut host)?;
# Ok::<(), zsui::ZsuiError>(())
```

Create a real native OS window with one line:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

Attach a typed Rust view to the same native window path:

```rust,no_run
use zsui::{button, column, native_window, text, WidgetId};

#[derive(Clone)]
enum Msg {
    Save,
}

native_window("Example")
    .size(900, 620)
    .view(column(vec![
        text::<Msg>("Settings"),
        button("Save").id(WidgetId::new(1)).on_click(Msg::Save),
    ]))
    .run()?;
# Ok::<(), zsui::ZsuiError>(())
```

For an application-owned state loop, use `stateful_view`. Every native input is
converted to `Msg`, passed through `update`, then the view is rebuilt and the
window is repainted:

```rust,no_run
use zsui::{
    button, column, native_window, text, AppCx, Command, NativeWindowRuntimeDriver,
    ViewNode, WidgetId,
};

struct State { count: u32 }
#[derive(Clone)]
enum Msg { Increment }

fn view(state: &State) -> ViewNode<Msg> {
    column([
        text(format!("Count: {}", state.count)),
        button("Increment").id(WidgetId::new(1)).on_click(Msg::Increment),
    ])
}

fn update(state: &mut State, msg: Msg, cx: &mut AppCx) {
    match msg {
        Msg::Increment => {
            state.count += 1;
            cx.command(Command::custom("counter.incremented"));
        }
    }
}

native_window("Counter")
    .stateful_view(State { count: 0 }, view, update)
    .app_command_executor(NativeWindowRuntimeDriver::new())
    .run()?;
# Ok::<(), zsui::ZsuiError>(())
```

For codebases that want compile-time content enforcement, use the opt-in
typestate entry point. `build`, `run` and `run_smoke` do not exist until one of
the content methods changes the builder to `NativeWindowContentReady`:

```rust,no_run
use zsui::{text, typed_native_window};

typed_native_window("Strict Window")
    .size(640, 420)
    .view(text::<()>("Ready"))
    .run()?;
# Ok::<(), zsui::ZsuiError>(())
```

When a native smoke path needs direct UI command routing, use the command-view
variant. It keeps widget input typed while emitting reusable `UiCommand`s:

```rust,no_run
use zsui::{
    button, column, native_window, text, CommandId, NativeWindowRuntimeDriver, UiCommand,
    WidgetId,
};

native_window("Example")
    .size(900, 620)
    .ui_command_view(column(vec![
        text::<UiCommand>("Settings"),
        button("Save")
            .id(WidgetId::new(1))
            .on_click(UiCommand::app(CommandId("app.save"))),
    ]))
    .ui_command_executor(NativeWindowRuntimeDriver::new())
    .run()?;
# Ok::<(), zsui::ZsuiError>(())
```

Use a small feature set when embedding ZSUI into another Rust app:

```toml
[dependencies]
zsui = { version = "0.1", default-features = false, features = [
    "window",
    "button",
    "toggle",
    "list",
    "scroll",
    "dark-mode",
] }
```

The intended shape is Rust-style compile-on-demand: default features stay small
(`window`, `button`, `label`), heavy backend dependencies are optional, and
advanced widgets are behind explicit feature gates. This is feature/crate based
trimming, not a promise that Cargo magically removes every unused symbol inside
an enabled crate. The `window` feature selects Win32 on Windows and Winit on
macOS/Linux through target-specific dependencies, so the one-line window entry
does not require an extra backend feature on supported desktop targets. Cargo
features are additive across the dependency graph, so
large widgets and heavy native backends should move toward split crates or
feature modules such as `zsui-core`, `zsui-shell`, `zsui-render`,
`zsui-style`, `zsui-widgets-base`, `zsui-widgets-input`,
`zsui-widgets-list` and `zsui-widgets-extra`.

## Rust-First Target

ZSUI's long-term API target is not a C++/C# style inheritance tree. The
framework should be built around trait surfaces, typed messages, explicit state,
RAII-owned native resources, safe public APIs, `Result<T, ZsuiError>` failures,
capability traits, feature-gated backends, theme tokens, typed units and strong
IDs. It also preserves the simple `zsui::native_window(...).run()?` entry point
for native desktop windows, treats Android and Harmony as explicit future
Activity/Ability hosts, uses buffered no-flicker native rendering as the
Windows baseline, and only adds wider platform API crates such as
`windows-rs` when a concrete backend surface needs them. The modular target is
a small facade with feature-gated crates/modules, not a monolithic always-on
control registry.
The machine-readable target list is exposed through
`zsui_rust_first_goals()` and `zsui_rust_first_goal_names()`; the longer
target narrative is in `docs/framework-goals.md`.
The first concrete layer is now in `src/view.rs`, `src/app_command.rs`,
`src/style.rs` and `src/geometry.rs`: `View<Msg>`, `WidgetId`, `AppCx`,
`SharedAppCommandExecutor`, `ViewEventCx`,
`ViewPaintCx`, `ViewInteractionPlan`, `Px`, `Dp`, `Dpi`, `ZsuiTheme` and
tokenized color/radius/spacing primitives.
For the WinUI-style self-drawn surface, `src/shell_layout.rs` now provides a
product-neutral `ZsShellLayoutSpec`/`ZsNavigationScaffoldSpec` contract for
left navigation, right content, grouped cards, content rows with description
text, row accessories and action buttons. Its dimensions, card spacing,
viewport mask and scrollbar math are maintained as shared framework behavior.
The shell can also be attached as a live runtime with
`NativeWindowBuilder::shell_layout(...)`. On Windows, navigation hover and
selection, row accessories, wheel scrolling, scrollbar track clicks and thumb
dragging update the buffered draw plan in the normal event loop. Run the
standalone gallery with:

```text
cargo run --example navigation_shell_layout --features full
```

Use `--smoke` for an auto-closing native screenshot check or `--manifest` for
the non-window JSON summary.

The optional `workbench` feature provides a reusable desktop conversation and
task workspace: collapsible navigation, grouped conversation history, a
message timeline, paragraph/code/tool/notice blocks, message actions, a
multiline composer surface, mode/model actions and an optional inspector pane.
Applications own the data and commands; ZSUI owns stable DPI-aware layout,
draw plans, hit regions and local selection/scroll state.

```rust
native_window("Workbench")
    .size(1280, 800)
    .workbench(spec)
    .run()?;
```

Run the standalone workbench gallery with
`cargo run --example workbench_shell --features full`; add `--smoke` for a
real Win32 screenshot or `--manifest` for its structural report. Use
`zsui_component_catalog_summary()` to inspect the current WinUI-style component
coverage without treating declaration-only components as implemented.

The Windows notepad demo combines the reusable `document-shell` feature with a
native multiline text service. Its self-drawn document tab, command bar,
rounded editor frame and status surface reuse Fluent tokens, semantic icons and
the buffered no-flicker Windows renderer; file dialogs, keyboard accelerators,
document lifecycle and the application icon remain real native integrations:

```text
cargo run --example zsui_notepad --features notepad-demo
```

`docs/notepad-demo.md` records the reproducible ZSUI, egui and Windows Notepad
comparison. The result is intentionally candid: ZSUI is much smaller for this
native-service sample and keeps that advantage with the modern shell, while
egui currently needs less application code.

The optional `calculator` feature adds a typed decimal engine and reusable
standard-calculator shell with a Fluent keypad, memory row, history panel,
semantic icons and stable hit regions. The interactive Windows example uses
the same buffered no-flicker renderer and includes mouse, keyboard, DPI and app
icon handling:

```text
cargo run --example zsui_calculator --no-default-features --features calculator-demo
```

`docs/calculator-demo.md` records a reproducible comparison with the local
Windows Calculator, including separate process-group and component memory
counters when `ApplicationFrameHost` owns the visible system window.

Built-in workbench visuals consume the shared Fluent token layer rather than
embedding product colors or icon code points. The Windows renderer uses Segoe
UI Variable Text, the Windows 11 12/16 and 14/20 type ramp, semantic surface and
border colors, 4 epx control corners, 8 epx card corners and semantic `ZsIcon`
commands resolved through Segoe Fluent Icons. GTK icon-theme names and macOS SF
Symbol names are exposed by the same icon catalog for their native backends.
System accent/high-contrast discovery and native GTK/macOS icon binding remain
explicit completion gates.

Audit a declaration before attaching it to a host:

```rust
use zsui::{app, HostCapabilities, Window};

let report = app("Example")
    .window(Window::new("Example"))
    .declaration_report_for(&HostCapabilities::windows_scaffold());

assert!(report.is_valid());
# Ok::<(), zsui::ZsuiError>(())
```

## Current Scope

- `WindowSpec` / `Window`
- `WindowSpec::icon_path(...)` declaration validation and Win32 owned HICON
  loading for window app icons
- `TraySpec`
- `MenuSpec` / `MenuItemSpec`
- `HotkeySpec`
- `ClipboardData`
- `SettingsPageSpec` / `SettingsItemSpec`
- `ZsShellLayoutSpec` / `ZsNavigationScaffoldSpec` for product-neutral
  WinUI-style navigation/card layouts with grouped cards, content rows,
  description text, row accessories and action-button areas
- `ZsWorkbenchSpec` / `ZsWorkbenchRuntime` for reusable conversation and task
  workspaces with navigation, message blocks, composer and inspector regions
- `zsui_component_catalog()` / `zsui_component_catalog_summary()` for
  machine-readable component readiness and missing-control inventory
- `Command` / `AppEvent`
- `UiNode` / `UiNodeKind` declarative component trees
- `HostCapabilities`
- `ZsuiAppDeclarationReport` for structural declaration audits and host
  degradation warnings
- `ZsuiHost`, `MemoryHost` and `PlatformHost`
- `NativeWindowHost` for a minimal real Windows/macOS/Linux window event loop
- `NativeWindowRuntimeDriver` for wiring product adapters into the current
  desktop native-window runtime boundary, including projected status menu and
  settings declarations, status-menu command dispatch and settings-item updates
  through native host operations
- Android and Harmony capability scaffolds for future mobile runtime hosts
- Android Activity and Harmony Ability scaffold manifests plus FFI/lifecycle/
  surface/input bridge contracts through `mobile_runtime_host_scaffold()` and
  `mobile_runtime_bridge_contract()`
- Android/Harmony bridge parity reports through
  `mobile_runtime_bridge_parity_report()` for checking scaffold/contract
  metadata, required callback route kinds and pending FFI symbols without
  claiming device runtime readiness
- Android/Harmony bridge dispatch reports through
  `mobile_runtime_bridge_dispatch_report()` for mapping required callback
  symbols to lifecycle, surface, typed input and `NativeRuntimeDriver`
  operations before real FFI code is added
- Android/Harmony contract dispatch smoke through
  `mobile_runtime_bridge_contract_smoke_report()` for locally replaying the
  required bridge sequence without faking device proof
- Android/Harmony contract artifact writing through
  `write_mobile_runtime_bridge_contract_artifacts()` without generating device
  launch, screenshot, lifecycle, surface or input proof; the local bundle also
  includes `device-smoke-plan.json` and `agent-context.json` for AI handoff
- Android/Harmony contract artifact review through
  `review_mobile_runtime_bridge_contract_artifacts()` so local bridge artifacts
  and expected JSON schemas can be validated separately from device smoke; the
  `for_all` variants and CLI `all` target cover both Android and Harmony in one
  run
- Android/Harmony device-smoke plans and read-only artifact review through
  `mobile_runtime_device_smoke_plan()` and
  `mobile_runtime_device_smoke_trace_templates()` plus
  `review_mobile_runtime_device_smoke_artifacts()` with schema checks for
  device-sourced lifecycle, surface and input traces
- shared geometry, command, event, lifecycle, layout, component, render, host
  surface and native control protocols
- Rust-first typed view builders and contexts through `View<Msg>`, `WidgetId`,
  `AppCx`, `ViewEventCx`, `ViewPaintCx`, `column`, `row`, `text`, `button`,
  `textbox`, `checkbox`, the owner-drawn `toggle`, feature-gated `scroll`
  containers and `list`
  selection
- `NativeWindowBuilder::view(...)` projection from typed `ViewNode<Msg>` into
  `NativeDrawPlan` content used by the native smoke path
- opt-in `typed_native_window(...)` typestate construction, where content is a
  compile-time requirement before `build`, `run` or `run_smoke`; the ordinary
  `native_window(...)` path remains available for empty native surfaces
- `NativeWindowBuilder::stateful_view(...)` and `SharedLiveViewRuntime` for a
  real `State -> Msg -> update -> rebuild/layout/paint` loop on Win32, including
  resize/DPI surface refresh, native repaint, `AppCx::quit()` window closure
  and `AppCx::command(...)` handoff through `SharedAppCommandExecutor`; attach a
  handler with `NativeWindowBuilder::app_command_executor(...)`
- `NativeWindowBuilder::ui_command_view(...)` and `ViewInteractionPlan` routing
  for Win32 `WM_LBUTTONUP` clicks and focused `WM_CHAR` textbox input into
  `ViewEventCx<UiCommand>` and reusable command ids; attach
  `SharedUiCommandExecutor` through `ui_command_executor(...)` to execute them
  through `NativeWindowRuntimeDriver`, a closure or
  `ProductAdapterUiCommandExecutor`; checkbox clicks and
  focused `WM_KEYDOWN` keyboard activation route to typed events when the
  relevant widget features are enabled, Tab can traverse native focus targets,
  list row selection can dispatch through the same command-backed view tree,
  and `WM_MOUSEWHEEL` can route into typed scroll events for scroll containers
- reusable `ZsToggleRenderPlan` geometry for the owner-drawn settings toggle;
  the same plan drives Shell accessories and the
  standalone feature-gated `toggle(...)` View widget
- product adapter typed view smoke through `ProductViewAdapterHost`,
  `ProductViewRuntimeSmokeRequest` and `examples/product_adapter_view.rs`
- typed units and theme token primitives through `Px`, `Dp`, `Dpi`,
  `UiLength`, `ZsuiTheme`, `ThemeColorToken`, `RadiusToken` and `SpacingToken`
- product-neutral self-draw command plans (`NativeDrawPlan`,
  `NativeDrawCommand`, `NativeDrawCommandSink`) and the Windows GDI
  renderer/text layout sink in `src/windows_gdi_renderer.rs`
- product-neutral WinUI-style shell layout plans in `src/shell_layout.rs`,
  including left navigation, right content headers, grouped cards, rows,
  descriptions, inline controls, action areas, viewport masks and scrollbars
  projected to `NativeDrawPlan`
- workbench composite layout plans in `src/workbench.rs`, including grouped
  conversation navigation, user/assistant/tool surfaces, code and notice
  blocks, composer actions, inspector tabs, DPI scaling, hit testing and local
  interaction state
- document editor shell plans in `src/document_shell.rs`, including a document
  tab, compact command bar, rounded native-editor inset, status surface,
  semantic icons and stable pointer hit regions
- internal RAII wrappers for Win32/GDI buffered paint, window HDCs, compatible
  memory DCs, smoke-screenshot HBITMAPs, owned main/quick HWND cleanup,
  owned HICON app-icon resources loaded from icon paths, brushes, pens, fonts
  and selected-object restoration
- Win32 owned tray icon resources and `WindowsWin32StatusItemHost` backed by
  `Shell_NotifyIconW`, wired into the direct Windows `NativeWindowHost` path
  and optional `native_smoke_run --tray` status-item smoke, with native
  command-id table routing, RAII popup-menu creation/cleanup and
  `TrackPopupMenu` selection routing for status menus; target smoke for real
  user popup selection is still pending
- Win32 main/quick/transient window style, create-params,
  message-loop and `NativeMainWindowHost`/`NativeTransientWindowHost`
  implementations in `src/windows_win32_host.rs`
- Win32 no-flicker paint can now consume product-neutral `NativeDrawPlan`
  content through `create_windows_win32_for_specs_with_draw_plans(...)` and
  `set_windows_win32_window_draw_plan(...)`
- native host action/status/settings command contracts in
  `src/native_host_actions.rs`
- native adapter manifest, timer routing and reusable platform service host
  contracts
- machine-readable AI/agent context through `zsui_agent_context()` and
  `zsui_agent_context_json()`
- native target-smoke manifest planning through `native_host_smoke_plan()` and
  `examples/native_smoke_manifest.rs`
- target-smoke artifact writing through `write_native_host_smoke_artifacts()`
  and `examples/native_smoke_record.rs`
- first-pass auto-closing native smoke windows through
  `NativeWindowSmokeRunOptions` and `examples/native_smoke_run.rs`
- Windows `window.png` capture for native smoke artifacts through the direct
  Win32 `HWND`
- target-smoke artifact review through
  `review_native_host_smoke_artifacts()` and `examples/native_smoke_review.rs`
- product adapter and reusable runtime harness contracts for keeping product
  state, settings, async events and AI/tool execution outside native hosts
- product adapter runtime smoke reports through
  `ProductAdapterRuntimeSmokeRequest` and `examples/product_adapter_smoke.rs`
- Cargo feature manifest helpers through `zsui_feature_manifest()`,
  `zsui_default_feature_names()` and `zsui_optional_dependency_feature_names()`
- Rust-first framework goal helpers through `zsui_rust_first_goals()` and
  `zsui_rust_first_goal_names()`

`MemoryHost` is the deterministic test backend. `PlatformHost` is a small
scaffold for the current target that records declarations and bridges text
clipboard access when the `clipboard` feature is enabled. Without that feature
it falls back to in-memory clipboard storage.

## AI Context On Demand

An AI agent should not read the repository recursively before every task.
ZSUI provides a small bootstrap document and task-specific context packs:

1. Read only [`docs/ai-agent.md`](docs/ai-agent.md) first.
2. List the available packs:

   ```powershell
   .\scripts\ai-context.ps1 -List
   ```

3. Select one pack and read only its required files:

   ```powershell
   .\scripts\ai-context.ps1 -Pack calculator
   .\scripts\ai-context.ps1 -Pack windows-renderer -IncludeOptional
   ```

The machine-readable routing table is
[`docs/ai/context-packs.json`](docs/ai/context-packs.json). Full progress and
platform references remain available as optional material instead of entering
the default prompt. This follows the same shape as feature-gated controls:
start with a small core, then compose only the context needed for the task.

## Repository Shape

- `src/`: public framework API and host contracts.
- `examples/basic.rs`: minimal declaration and memory-host run.
- `examples/declaration_audit.rs`: JSON declaration audit report for host
  readiness and AI/tooling checks.
- `examples/rust_first_view.rs`: runnable typed `State`/`View<Msg>`/`AppCx`
  update-and-repaint example with `--smoke` and `--manifest` modes.
- `examples/list_selection.rs`: feature-gated typed list row selection example.
- `examples/scroll_view.rs`: feature-gated scroll container layout, typed
  scroll event, clipping and draw-plan example.
- `examples/navigation_shell_layout.rs`: product-neutral WinUI-style
  navigation/card shell layout projected to a native draw plan.
- `examples/workbench_shell.rs`: reusable desktop conversation/task workbench
  with real Win32 screenshot and machine-readable manifest modes.
- `examples/zsui_notepad.rs`: hybrid Fluent document shell and native Windows
  text service with a release-size and runtime-memory comparison script.
- `examples/zsui_calculator.rs`: modern standard calculator using the reusable
  decimal engine/shell and a measured local Windows Calculator comparison.
- `examples/native_smoke_manifest.rs`: JSON manifest for target native host
  smoke artifacts.
- `examples/native_smoke_record.rs`: writes contract-level target smoke
  artifacts without faking screenshots.
- `examples/native_smoke_run.rs`: opens a real native smoke window, auto-closes
  it, and records interaction artifacts.
- `examples/native_smoke_review.rs`: reviews target smoke artifacts and reports
  missing or invalid required proof files.
- `examples/mobile_scaffold_manifest.rs`: JSON manifest for Android Activity
  and Harmony Ability host scaffolds, bridge contracts with `--bridge`, parity
  reports with `--parity`, dispatch reports with `--dispatch`, contract
  dispatch smoke with `--dispatch-smoke`, local contract artifact writing with
  `--write-contract` including device-smoke plans and agent context, local
  contract artifact review with `--review-contract`, device smoke plans with
  `--smoke`, device trace templates with `--trace-template` and artifact
  review with `--review`; the write/review contract commands also accept
  `all`.
- `examples/product_adapter.rs`: product adapter plus reusable runtime harness
  wiring without application-specific product code.
- `examples/product_adapter_smoke.rs`: machine-readable runtime harness smoke
  report covering startup, command dispatch, event polling, AI routing and
  shutdown.
- `examples/product_adapter_native_driver.rs`: product adapter smoke using
  `NativeWindowRuntimeDriver` as the reusable native driver bridge.
- `examples/product_adapter_view.rs`: product adapter smoke for typed
  `View<Msg>` messages flowing through `AppCx` into reusable UI commands.
- `docs/architecture.md`: framework boundary and layering rules.
- `docs/framework-goals.md`: long-range Rust-first API and trimming target.
- `docs/porting.md`: host implementation contract for new platform backends.
- `docs/native-host-smoke.md`: target artifact contract before platform
  completion claims.
- `docs/ai-agent.md`: compact first-read router for AI agents.
- `docs/ai/context-packs.json`: task-specific required and optional file sets.
- `scripts/ai-context.ps1`: prints the smallest context set for one task.
- `docs/skills/zsui-native-ui/`: skill-style AI handoff package for standalone
  ZSUI development.

ZSUI is designed so another Rust application can provide its own product
adapter and choose a native host without placing storage, sync or business
logic inside the framework.

Verify every public single feature and the supported widget/backend
combinations with:

```powershell
.\scripts\ai-context.ps1 -Validate
.\scripts\check-feature-matrix.ps1 -Locked
```

The same matrix runs in `.github/workflows/ci.yml`, together with default,
no-default, full Windows and Linux/macOS desktop checks.

On Windows, the current first-pass native smoke run can produce the full target
artifact set:

```powershell
cargo run --example native_smoke_run -- windows
cargo run --example native_smoke_review -- windows
```

To additionally request a real Win32 status item during the smoke run:

```powershell
cargo run --example native_smoke_run -- windows --tray
```

To attach a typed Rust view draw plan and route Win32 input into `UiCommand`s
during the smoke run:

```powershell
cargo run --example native_smoke_run -- windows --view
```

To run the dedicated typed scroll smoke path:

```powershell
cargo run --features "scroll,label" --example native_smoke_run -- windows --scroll-view
```

When the `textbox` feature is enabled, the same example also routes focused
`WM_CHAR` text input through `ViewEventCx<UiCommand>`.
When the `checkbox` feature is enabled, it also routes checkbox toggles through
typed `Toggled` events.
When the `toggle` feature is enabled, the standalone owner-drawn switch uses the
same geometry as the settings Shell and routes click/Space activation
through typed `Toggled` events.
When the `list` feature is enabled, it records typed list row selection through
the same command route, including Up/Down keyboard selection between rows. It
also records Tab focus traversal and focused `WM_KEYDOWN` keyboard activation
for the typed view.
The `--scroll-view` path records `WM_MOUSEWHEEL` to typed `ScrollBy` event
routing and a reusable scroll `UiCommand`.

## Support

If this project helps you, support is welcome and helps fund continued work on
Rust-native UI infrastructure.

![Support](docs/images/donate.png)

## License

This project is licensed under [GPL-3.0-only](LICENSE).
