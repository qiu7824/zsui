# ZSUI

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
- `docs/ai-agent.md`: standalone guide for AI agents working on ZSUI.
- `docs/skills/zsui-native-ui/`: skill-style AI handoff package for standalone
  ZSUI development.

ZSUI is designed so another Rust application can provide its own product
adapter and choose a native host without placing storage, sync or business
logic inside the framework.

Verify every public single feature and the supported widget/backend
combinations with:

```powershell
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
