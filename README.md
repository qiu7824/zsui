# ZSUI

ZSUI is a Rust-first native system UI framework contract extracted from ZSClip.
It is intentionally declaration-first: application code describes windows,
tray/status menus, commands, hotkeys, settings pages and host capabilities in
Rust, while each platform host translates those declarations to Win32, AppKit,
GTK/libadwaita or mobile hosts.

ZSUI is not a browser shell and not a self-drawn widget kit. Product behavior
stays in the product crate; ZSUI owns portable UI specs, command/event ids,
capability reporting and host traits.
Today the concrete runtime in this crate is the minimal `NativeWindowHost`.
On Windows it now enters the extracted Win32/GDI path from
`src/windows_win32_host.rs` and uses the ZSClip no-flicker paint foundation:
`WM_ERASEBKGND` is suppressed and paint goes through a buffered top-down DIB
when available. macOS and Linux still use the `winit_desktop` first-pass
runtime. AppKit/GTK product hosts are still being split out of ZSClip because
they currently mix reusable host code with clipboard-product behavior.

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

When a native smoke path needs direct UI command routing, use the command-view
variant. It keeps widget input typed while emitting reusable `UiCommand`s:

```rust,no_run
use zsui::{button, column, native_window, text, CommandId, UiCommand, WidgetId};

native_window("Example")
    .size(900, 620)
    .ui_command_view(column(vec![
        text::<UiCommand>("Settings"),
        button("Save")
            .id(WidgetId::new(1))
            .on_click(UiCommand::app(CommandId("app.save"))),
    ]))
    .run()?;
# Ok::<(), zsui::ZsuiError>(())
```

Use a small feature set when embedding ZSUI into another Rust app:

```toml
[dependencies]
zsui = { version = "0.1", default-features = false, features = [
    "window",
    "button",
    "list",
    "scroll",
    "dark-mode",
] }
```

The intended shape is Rust-style compile-on-demand: default features stay small
(`window`, `button`, `label`), heavy backend dependencies are optional, and
advanced widgets are behind explicit feature gates. This is feature/crate based
trimming, not a promise that Cargo magically removes every unused symbol inside
an enabled crate. Cargo features are additive across the dependency graph, so
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
Activity/Ability hosts, uses reusable ZSClip no-flicker native rendering work as
the Windows baseline, and only adds wider platform API crates such as
`windows-rs` when a concrete backend surface needs them. The modular target is
a small facade with feature-gated crates/modules, not a monolithic always-on
control registry.
The machine-readable target list is exposed through
`zsui_rust_first_goals()` and `zsui_rust_first_goal_names()`; the longer
target narrative is in `docs/framework-goals.md`.
The first concrete layer is now in `src/view.rs`, `src/style.rs` and
`src/geometry.rs`: `View<Msg>`, `WidgetId`, `AppCx`, `ViewEventCx`,
`ViewPaintCx`, `ViewInteractionPlan`, `Px`, `Dp`, `Dpi`, `ZsuiTheme` and
tokenized color/radius/spacing primitives.

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
- Android/Harmony device-smoke plans and read-only artifact review through
  `mobile_runtime_device_smoke_plan()` and
  `review_mobile_runtime_device_smoke_artifacts()`
- shared geometry, command, event, lifecycle, layout, component, render, host
  surface and native control protocols
- Rust-first typed view builders and contexts through `View<Msg>`, `WidgetId`,
  `AppCx`, `ViewEventCx`, `ViewPaintCx`, `column`, `row`, `text`, `button`,
  `textbox`, `checkbox`, feature-gated `scroll` containers and `list`
  selection
- `NativeWindowBuilder::view(...)` projection from typed `ViewNode<Msg>` into
  `NativeDrawPlan` content used by the native smoke path
- `NativeWindowBuilder::ui_command_view(...)` and `ViewInteractionPlan` routing
  for Win32 `WM_LBUTTONUP` clicks and focused `WM_CHAR` textbox input into
  `ViewEventCx<UiCommand>` and reusable command ids; checkbox clicks and
  focused `WM_KEYDOWN` keyboard activation route to typed events when the
  relevant widget features are enabled, Tab can traverse native focus targets,
  list row selection can dispatch through the same command-backed view tree,
  and `WM_MOUSEWHEEL` can route into typed scroll events for scroll containers
- product adapter typed view smoke through `ProductViewAdapterHost`,
  `ProductViewRuntimeSmokeRequest` and `examples/product_adapter_view.rs`
- typed units and theme token primitives through `Px`, `Dp`, `Dpi`,
  `UiLength`, `ZsuiTheme`, `ThemeColorToken`, `RadiusToken` and `SpacingToken`
- product-neutral self-draw command plans (`NativeDrawPlan`,
  `NativeDrawCommand`, `NativeDrawCommandSink`) and the extracted ZSClip
  Windows GDI renderer/text layout sink in `src/windows_gdi_renderer.rs`
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
- extracted ZSClip Win32 main/quick/transient window style, create-params,
  message-loop and `NativeMainWindowHost`/`NativeTransientWindowHost`
  implementations in `src/windows_win32_host.rs`
- Win32 no-flicker paint can now consume product-neutral `NativeDrawPlan`
  content through `create_windows_win32_for_specs_with_draw_plans(...)` and
  `set_windows_win32_window_draw_plan(...)`
- native host action/status/settings command contracts extracted from reusable
  ZSClip app-core code into `src/native_host_actions.rs`
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
- `examples/rust_first_view.rs`: typed `View<Msg>`/`WidgetId`/`AppCx`
  example without string events or global state.
- `examples/list_selection.rs`: feature-gated typed list row selection example.
- `examples/scroll_view.rs`: feature-gated scroll container layout, typed
  scroll event, clipping and draw-plan example.
- `examples/native_smoke_manifest.rs`: JSON manifest for target native host
  smoke artifacts.
- `examples/native_smoke_record.rs`: writes contract-level target smoke
  artifacts without faking screenshots.
- `examples/native_smoke_run.rs`: opens a real native smoke window, auto-closes
  it, and records interaction artifacts.
- `examples/native_smoke_review.rs`: reviews target smoke artifacts and reports
  missing or invalid required proof files.
- `examples/mobile_scaffold_manifest.rs`: JSON manifest for Android Activity
  and Harmony Ability host scaffolds, bridge contracts with `--bridge`, device
  smoke plans with `--smoke` and artifact review with `--review`.
- `examples/product_adapter.rs`: product adapter plus reusable runtime harness
  wiring without ZSClip product code.
- `examples/product_adapter_smoke.rs`: machine-readable runtime harness smoke
  report covering startup, command dispatch, event polling, AI routing and
  shutdown.
- `examples/product_adapter_native_driver.rs`: product adapter smoke using
  `NativeWindowRuntimeDriver` as the reusable native driver bridge.
- `examples/product_adapter_view.rs`: product adapter smoke for typed
  `View<Msg>` messages flowing through `AppCx` into reusable UI commands.
- `docs/architecture.md`: extraction boundary and layering rules.
- `docs/framework-goals.md`: long-range Rust-first API and trimming target.
- `docs/porting.md`: host implementation contract for new platform backends.
- `docs/native-host-smoke.md`: target artifact contract before platform
  completion claims.
- `docs/ai-agent.md`: standalone guide for AI agents working on ZSUI.
- `docs/skills/zsui-native-ui/`: skill-style AI handoff package migrated from
  the original ZSClip native UI handoff docs and adapted for standalone ZSUI.

ZSUI is designed so another Rust application can provide its own product
adapter and choose a native host without copying ZSClip storage, sync or
business logic.

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
When the `list` feature is enabled, it records typed list row selection through
the same command route, including Up/Down keyboard selection between rows. It
also records Tab focus traversal and focused `WM_KEYDOWN` keyboard activation
for the typed view.
The `--scroll-view` path records `WM_MOUSEWHEEL` to typed `ScrollBy` event
routing and a reusable scroll `UiCommand`.
