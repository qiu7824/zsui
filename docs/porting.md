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
`mobile_runtime_bridge_parity_report(platform)` or
`examples/mobile_scaffold_manifest.rs --parity <platform>` to verify scaffold
and contract metadata, required callback route coverage and pending FFI symbols
without claiming runtime readiness. Use
`mobile_runtime_bridge_dispatch_report(platform)` or
`examples/mobile_scaffold_manifest.rs --dispatch <platform>` to map required
callback symbols to lifecycle, surface, typed input and `NativeRuntimeDriver`
operations before adding real FFI glue. Use
`mobile_runtime_bridge_contract_smoke_report(platform)` or
`examples/mobile_scaffold_manifest.rs --dispatch-smoke <platform>` for a local
contract smoke that replays the declared callback sequence. Use
`write_mobile_runtime_bridge_contract_artifacts(platform)` or
`examples/mobile_scaffold_manifest.rs --write-contract <platform>` to write
local contract artifacts, device-smoke plan and agent context without
generating device proof. Use
`review_mobile_runtime_bridge_contract_artifacts(platform)` or
`examples/mobile_scaffold_manifest.rs --review-contract <platform>` to validate
those local contract artifacts and expected JSON schemas separately from
device-smoke proof. The write/review contract APIs also have `*_for_all`
variants, and the CLI accepts `all` for both Android and Harmony. Use
`examples/mobile_scaffold_manifest.rs --smoke <platform>` for the device
artifact plan, `--trace-template <platform>` for lifecycle/surface/input trace
templates and `--review <platform>` for read-only validation of captured mobile
artifacts. Device review requires device-sourced JSON schemas for lifecycle,
surface and input traces, so local contract JSON is not enough for a
device-smoke pass.
Backend crates or modules should stay behind Cargo features. The current
feature graph is mirrored by `zsui_feature_manifest()`: `desktop-winit`,
`windows-gdi`, `windows-win32`, `macos-appkit`, `linux-gtk`, `android` and
`harmony` are platform/backend gates, while `clipboard` and `image` own their
optional dependencies. The
default `window` umbrella must keep the one-line desktop entry working and rely
on target-specific dependencies to compile only the active platform backend.

The AppKit and GTK4 backend features provide target-native desktop service
slices through safe Rust contracts. Both now map `WindowSpec` through
`WindowService` to owned `NSWindow` or `ApplicationWindow` instances with
strong `WindowId` routing for title, visibility, redraw and close operations.
macOS maps open/save requests to
`NSOpenPanel`/`NSSavePanel` and lowers `MenuSpec` into owned
`NSMenu`/`NSMenuItem` objects; UTF-8 clipboard text uses `NSPasteboard`. Linux
maps dialogs to GTK4 `FileChooserNative`, menus to `GMenu`/`SimpleAction`, and
UTF-8 clipboard text to `GdkClipboard`. Both menu paths preserve nested,
disabled, checked and accelerator state and return typed `Command` values as
`DesktopEvent::MenuCommand`; native toolkit objects remain private. Clipboard
images and files remain explicitly unsupported until their native formats are
implemented and tested.

The unified native-window path also attaches backend-neutral `NativeDrawPlan`
content to both platforms. AppKit uses a flipped custom `NSView`,
`NSBezierPath`, semantic `NSString` attributes and SF Symbols. GTK4 uses a
`DrawingArea`, Cairo, Pango and the current icon theme with the bundled Fluent
SVG fallback. Both sinks implement fill, stroke, rounded geometry, text, icon
and balanced clip commands. AppKit `mouseUp:`/`scrollWheel:` and GTK4
`GestureClick`/`EventControllerScroll` also convert local coordinates into the
shared `ViewInteractionPlan`, dispatch typed static/live view messages, hand
emitted commands to shared executors and replace the draw plan after stateful
updates. The content views are focusable and also route Tab/Shift+Tab, Enter/Space,
list Up/Down, direct UTF-8 character input, multiline return and deletion.
AppKit now implements `NSTextInputClient`; GTK4 owns a focused
`GtkIMMulticontext`. Both keep marked text provisional in the shared input
runtime, render it without mutating application state, commit UTF-8 through
the normal typed `TextChanged` path and anchor the native candidate window to
the focused editor. Precise cursor/selection editing, target CJK interaction
artifacts, resize-driven relayout and accessibility remain separate gates.

These services do not complete either native host. The unified
`native_window(...).run()` path now enters `NSApplication` on macOS and
`GtkApplication` on Linux, while the explicit `desktop-winit` feature remains a
fallback transport. Shared View rendering, click/scroll, keyboard focus,
activation, direct text editing and first-pass IME composition now reach all
three native window surfaces, but target screenshot capture, CJK interaction
evidence and precise caret/selection behavior remain required; entering or
painting a native event loop alone is not system-complete evidence.
The Rust-first target list is exposed by `zsui_rust_first_goals()` and expanded
in `docs/framework-goals.md`. Backend work should specifically preserve safe
public APIs, RAII ownership for native handles, `Result<T, ZsuiError>` error
reporting, explicit context/state flow and typed capability traits.
Before claiming component parity, compare the backend against
`zsui_component_catalog()`. A contract-only component or a composite
`workbench` draw plan does not prove native input, accessibility or target
interaction support for its underlying WinUI analogue.
It should also preserve the one-line `zsui::native_window(...).run()?` entry
shape for ordinary apps, keep buffered no-flicker self-draw behavior as
the Windows baseline, and add wider bindings such as `windows-rs` only when a
specific backend surface needs them.
The first-pass typed view layer is `src/view.rs`: hosts should treat
`View<Msg>`, `WidgetId`, `ViewEventCx`, `ViewInteractionPlan` and
`ViewPaintCx` as the direction for future event and paint routing instead of
introducing string event buses or global widget registries.
The feature-gated `scroll` container offsets its child content, clips hit
targets to the viewport and emits `PushClip`/`PopClip` draw commands; backend
renderers should preserve that clipping boundary before adding wheel/touch
scroll input. It now also accepts typed `ScrollBy` events and emits an optional
typed `on_scroll(Dp)` message after clamping the offset to the declared content
height.
`NativeWindowBuilder::view(...)` now converts a typed `ViewNode<Msg>` into a
`NativeDrawPlan` for the desktop native-window path. Backends should consume
that draw plan through their renderer/text layout sink.
`NativeWindowBuilder::ui_command_view(...)` keeps a command-backed tree for
native input. The Win32 host already maps `WM_LBUTTONUP` through
`ViewInteractionPlan`, dispatches into `ViewEventCx<UiCommand>` and records
command ids during native smoke. Backends must hand those commands to
`SharedUiCommandExecutor` after releasing internal route locks; use
`ProductAdapterUiCommandExecutor` for the standard product boundary. It also
routes focused `WM_CHAR` input into
textbox `TextChanged` events when the textbox feature is enabled and checkbox
clicks into typed `Toggled` events when the checkbox feature is enabled.
`WM_KEYDOWN` Enter/Space activation is also routed for focused button and
checkbox/toggle targets, and Tab traverses the ordered `ViewInteractionPlan` focus
targets. Feature-gated list row selection uses child IDs and dispatches through
the same `ViewEventCx` path; Win32 Up/Down keys can move focused list selection
and emit the same typed message. Win32 `WM_MOUSEWHEEL` can target the nearest
scroll container and emit a typed scroll event. Other backends should add their
OS pointer, wheel/touch scroll, keyboard focus, keyboard activation and IME
routing back into `ViewEventCx` as distinct gates instead of coupling it to
product state.
Render `ViewHitTargetKind::Toggle` from the shared `ZsToggleRenderPlan`; do not
replace its track/knob geometry with a backend-specific approximation.
Use `native_smoke_run --scroll-view` on Windows to exercise the command-backed
scroll route before claiming parity in another backend.
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
the target drawing API through `NativeDrawCommandSink`. Windows has the
`WindowsGdiRenderer`, `WindowsGdiTextLayout` and
`WindowsGdiDrawSink`; AppKit and GTK4 now keep the same command contract in
`macos_appkit_renderer.rs` and `linux_gtk_renderer.rs` while swapping only the
native drawing and text-layout implementation.

For direct desktop window hosts, keep the product-neutral shape from
`WindowsWin32MainWindowHost`: map `NativeMainWindowRequest` and
`NativeWindowOptions` to platform styles, preserve create-params for the window
procedure, expose a small message-loop wrapper, and implement
`NativeMainWindowHost` without product callbacks. Transient window hosts should
follow the `WindowsWin32TransientWindowHost` shape: topmost,
tool-window and no-activate presentation with product behavior outside the
host. The Windows version lives in `src/windows_win32_host.rs`.
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

Window menus retain their `HMENU` and `HACCEL` resources through RAII. The
message loop calls `TranslateAcceleratorW` before normal dispatch, so a shared
`MenuItemSpec::accelerator` routes the same typed `Command` as clicking the
native menu item. AppKit maps the same accelerator to key equivalents and GTK4
maps it to application action accelerators.

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
