# Host Porting Contract

A ZSUI host translates framework declarations into native platform behavior.
It should not embed product business logic.
Current platform names include Windows, macOS, Linux, Android and Harmony.
Android and Harmony are capability scaffolds until dedicated mobile runtime
hosts are implemented. Use `mobile_runtime_host_scaffold(platform)` or
`examples/mobile_scaffold_manifest.rs` to inspect the current Activity/Ability
bridge entry points, lifecycle bindings, capability bindings and target smoke
requirements. Use `mobile_runtime_bridge_contract(platform)` or
`examples/mobile_scaffold_manifest.rs --bridge <platform>` for the stricter
FFI contract: exported callback symbols, lifecycle/surface/input/command
routes, FFI safety rules and required device-smoke artifact files. Use
`examples/mobile_scaffold_manifest.rs --smoke <platform>` for the device
artifact plan and `--review <platform>` for read-only validation of captured
mobile artifacts.
Backend crates or modules should stay behind Cargo features. The current
feature graph is mirrored by `zsui_feature_manifest()`: `desktop-winit`,
`windows-gdi`, `windows-win32`, `android` and `harmony` are platform/backend
gates, while `clipboard` and `image` own their optional dependencies.
The Rust-first target list is exposed by `zsui_rust_first_goals()` and expanded
in `docs/framework-goals.md`. Backend work should specifically preserve safe
public APIs, RAII ownership for native handles, `Result<T, ZsuiError>` error
reporting, explicit context/state flow and typed capability traits.
It should also preserve the one-line `zsui::native_window(...).run()?` entry
shape for ordinary apps, keep reusable ZSClip no-flicker self-draw behavior as
the Windows baseline, and add wider bindings such as `windows-rs` only when a
specific backend surface needs them.
The first-pass typed view layer is `src/view.rs`: hosts should treat
`View<Msg>`, `WidgetId`, `ViewEventCx` and `ViewPaintCx` as the direction for
future event and paint routing instead of introducing string event buses or
global widget registries.
`NativeWindowBuilder::view(...)` now converts a typed `ViewNode<Msg>` into a
`NativeDrawPlan` for the desktop native-window path. Backends should consume
that draw plan through their renderer/text layout sink, and should add input
routing back into `ViewEventCx` as a distinct gate instead of coupling it to
product state.
For product integration, use `ProductViewAdapterHost` and
`ZsuiReusableRuntimeHarness::run_view_smoke(...)` to verify typed view messages
before wiring a native backend to real product state.

Implement host surfaces in this order:

1. `ZsuiHost::capabilities`.
2. Window creation and visibility.
3. Tray/status menu creation.
4. Menu and hotkey command routing.
5. Clipboard text, then images and files.
6. File picker and native dialogs.
7. Settings page presentation.
8. Renderer/text layout binding.
9. Event polling and event-loop ownership.

For product-adapter startup, desktop/mobile runtime drivers should map
`NativeRuntimeStartupRequest.status_item` through `NativeStatusItemHost` and
`NativeRuntimeStartupRequest.settings_pages` through
`NativeSettingsPageModelHost`. This keeps status menus and settings models as
native host responsibilities while command execution and product state remain
behind `ProductAdapterHost`.

For self-drawn surfaces, translate `NativeDrawPlan` / `NativeDrawCommand` into
the target drawing API through `NativeDrawCommandSink`. Windows already has the
ZSClip-extracted `WindowsGdiRenderer`, `WindowsGdiTextLayout` and
`WindowsGdiDrawSink`; other backends should keep the same command contract and
only swap the native drawing implementation.

For direct desktop window hosts, keep the product-neutral shape from
`WindowsWin32MainWindowHost`: map `NativeMainWindowRequest` and
`NativeWindowOptions` to platform styles, preserve create-params for the window
procedure, expose a small message-loop wrapper, and implement
`NativeMainWindowHost` without product callbacks. Transient window hosts should
follow the extracted `WindowsWin32TransientWindowHost` shape: topmost,
tool-window and no-activate presentation with product behavior outside the
host. The extracted Windows version lives in `src/windows_win32_host.rs`.
When a backend has a self-drawn surface, attach or store `NativeDrawPlan`
content beside the native window handle and render it through the backend sink;
the Win32 host now does this with `set_windows_win32_window_draw_plan(...)` and
the no-flicker buffered GDI paint path. Follow the existing Win32/GDI RAII
pattern for native drawing resources: buffered paint, window HDC acquisition,
compatible memory DCs, smoke-screenshot HBITMAPs, owned main/quick HWND
cleanup, owned HICON app-icon resources, brushes, pens, fonts and
selected-object restoration are owned internally. Window icon paths should load
through owned HICON resources. Win32 tray/status items should use the
`WindowsWin32StatusItemHost` and its `Shell_NotifyIconW` backed RAII owner. The
direct Windows host can already create declared status items and
`native_smoke_run --tray` can request one; status menu command-id routing is
also available through the Win32 command table, and RAII popup-menu creation
plus cleanup is smoke-recorded. The host also exposes `TrackPopupMenu`
selection routing. Still add required target smoke artifacts for real user
popup menu selection before claiming system completion.

Each backend should report real support through `HostCapabilities`.
Use `CapabilityStatus::Partial` when a declaration can be accepted but native
behavior is incomplete, session-dependent or not yet smoke-tested.

Platform handles, native widget objects and message/event-loop details belong
inside the host implementation. Product behavior belongs behind the
application's product adapter.
For reusable applications, implement `ProductAdapterHost` and connect it through
`ZsuiReusableRuntimeHarness` before adding product-specific callbacks to a
platform host. Run `examples/product_adapter_smoke.rs` or call
`ProductAdapterRuntimeSmokeRequest` to prove the product boundary before wiring
the adapter to a target native runtime.
On desktop, `NativeWindowRuntimeDriver` is the current reusable driver bridge
between `ZsuiReusableRuntimeHarness` and the minimal native-window runtime.
Before claiming target-smoke readiness, generate a platform manifest with
`native_host_smoke_plan(platform)` or `examples/native_smoke_manifest.rs` and
store the required artifacts described in `docs/native-host-smoke.md`.
