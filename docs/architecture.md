# ZSUI Architecture

ZSUI is a reusable Rust UI foundation for native desktop tools.

The framework layers are:

- Core contracts: stable ids, commands, events, errors and host traits.
- Declaration models: windows, menus, tray/status items, hotkeys, clipboard,
  settings specs and product-neutral navigation/card shell layouts.
- Shared protocols: geometry, layout, command queues, lifecycle/events, components, renderer/text layout traits and self-draw command plans.
- Capability model: honest support, partial support and degradation reporting per host.
- Native host boundary: platform code creates windows, controls, menus, dialogs and clipboard bridges.
- Product adapter boundary: each application owns domain data, persistence, side effects and AI/tool integrations.

Reusable framework code must not depend on a product database, product settings
schema, sync transport, AI provider, platform handle or native message loop.
Native hosts may depend on platform APIs. Product crates may depend on ZSUI.
ZSUI itself should remain the shared contract between them.

The reusable product boundary lives in `src/product_adapter.rs`.
Applications implement `ProductAdapterHost` to project product state into ZSUI
UI declarations, execute UI commands, expose settings, bridge async events and
publish AI capability descriptors. `ZsuiReusableRuntimeHarness` wires that
adapter to a `NativeRuntimeDriver` without moving product behavior into a
platform host. `ProductAdapterRuntimeSmokeRequest` exercises that reusable
handoff path and produces a JSON-serializable smoke report before a product is
bound to a real native driver.

## Public Entry Point

Application authors start with:

```rust
use zsui::{app, Command, TraySpec, Window};
```

The public API is plain Rust data with `serde` support where practical, so
tools can inspect or generate UI declarations without loading a native backend.
`AppBuilder::declaration_report()` and
`AppBuilder::declaration_report_for(capabilities)` return a structured
`ZsuiAppDeclarationReport` that validates app/window/content/menu/tray/hotkey
and settings shapes, and records host capability degradation before any native
event loop is started.

For a minimal real native window, use:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

That convenience builder uses `NativeWindowHost` for the desktop event loop and
keeps full product behavior outside the framework. Android and Harmony are
represented in the platform/capability model as scaffolds; they need dedicated
Activity/Ability runtime hosts before `native_window(...).run()` can create
mobile surfaces. Their current scaffold manifests and bridge contracts live in
`src/mobile_host.rs`, `src/android_activity_host.rs` and
`src/harmony_ability_host.rs`, and can be printed with
`examples/mobile_scaffold_manifest.rs` or
`examples/mobile_scaffold_manifest.rs --bridge <platform>`. The bridge
contracts name the FFI symbols, lifecycle/surface/input callbacks, safety
rules and device-smoke artifact files that the real mobile hosts must satisfy.
`mobile_runtime_bridge_parity_report(platform)` and
`examples/mobile_scaffold_manifest.rs --parity <platform>` compare the
scaffold and contract metadata, confirm required callback route coverage and
list pending FFI callback symbols without claiming runtime readiness.
`mobile_runtime_bridge_dispatch_report(platform)` and
`examples/mobile_scaffold_manifest.rs --dispatch <platform>` map those callback
symbols to lifecycle, surface, typed input and `NativeRuntimeDriver`
operations before real Activity/Ability FFI glue exists.
`mobile_runtime_bridge_contract_smoke_report(platform)` and
`examples/mobile_scaffold_manifest.rs --dispatch-smoke <platform>` locally
replay that dispatch sequence as contract smoke while still reporting that FFI
and device proof are pending.
`write_mobile_runtime_bridge_contract_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --write-contract <platform>` write the
local contract JSON artifacts, including `device-smoke-plan.json` and
`agent-context.json`, without fabricating launch logs, screenshots, lifecycle
traces, surface traces or input traces.
`review_mobile_runtime_bridge_contract_artifacts(platform)` and
`examples/mobile_scaffold_manifest.rs --review-contract <platform>` validate
those local contract artifacts and expected JSON schemas without treating them
as device proof. The `*_for_all` helpers and CLI `all` target can write/review
Android and Harmony contract artifacts together.
`mobile_runtime_device_smoke_plan(platform)` and
`review_mobile_runtime_device_smoke_artifacts(platform)` provide the current
read-only verification contract for those required device artifacts, including
device-sourced JSON schema checks for lifecycle, surface and input traces.
`mobile_runtime_device_smoke_trace_templates(platform)` and
`examples/mobile_scaffold_manifest.rs --trace-template <platform>` expose the
same trace shapes for the future Activity/Ability bridge implementation.
The same desktop builder can now accept a typed view with
`native_window("Example").view(view).run()?`. That first lays out and paints
`ViewNode<Msg>` into `NativeDrawPlan`; on the direct Windows host the plan is
attached to the created `HWND` and rendered through the buffered no-flicker
GDI path. `ui_command_view(...)` keeps a command-backed view tree for native
input. On Windows, `WM_LBUTTONUP` is routed through `ViewInteractionPlan`,
  dispatched into `ViewEventCx<UiCommand>`, handed to an optional
  `SharedUiCommandExecutor` outside the Win32 route registry lock and recorded
  with success/failure/event evidence in native smoke. Focused `WM_CHAR` input is also routed into textbox
`TextChanged` events when the textbox feature is enabled, and checkbox clicks
route to typed `Toggled` events when the checkbox feature is enabled. The Win32
host also routes `WM_KEYDOWN` Enter/Space activation for focused button,
checkbox and toggle targets, and Tab traverses ordered focus targets from
`ViewInteractionPlan`. Feature-gated list row selection uses child IDs and can
flow through the same command-backed view tree; Win32 Up/Down keys can move
focused selection between list rows. Feature-gated scroll containers can
consume typed `ScrollBy` events, and Win32 `WM_MOUSEWHEEL` can route to the
nearest scroll container; `native_smoke_run --scroll-view` exercises that
command-backed route. Broader pointer dispatch, touch/inertial scroll,
IME/composition input and non-Windows input routing are still separate runtime
gates.
The standalone `toggle(...)` View widget and Shell toggle accessories both use
`ZsToggleRenderPlan` from `src/widget_render.rs`. Its track/knob sizing and DPI
math are shared by both surfaces, so the framework does not maintain competing
implementations.
`NativeWindowBuilder::stateful_view(state, view, update)` now provides the
normal application loop: Win32 input emits typed messages, `update` mutates the
application-owned state through `AppCx`, the framework rebuilds and lays out the
View, replaces the native draw plan and invalidates the HWND. Resize and DPI
surface changes relayout the existing tree without invoking the application
view function again. All three desktop hosts create the native window hidden,
attach the initial draw plan, typed input route, appearance and menu, and only
then make a requested-visible window visible. `AppCx::quit()` closes the native
window. `AppCx::command(...)` values are handed to the explicitly composed
`SharedAppCommandExecutor`, with success, failure and emitted-event evidence in
native smoke reports. `NativeWindowRuntimeDriver` implements this executor
contract. Product-owned `UiCommand` values use the parallel
`SharedUiCommandExecutor`; `ProductAdapterUiCommandExecutor` delegates directly
to `ProductAdapterHost` rather than hiding product behavior inside View state.

Row and column layout distinguish fixed sizes, minimum sizes and flexible
growth. `ViewNode::min_width(...)` and `min_height(...)` reserve typed `Dp`
space without forcing the node to consume the remaining axis. On Windows the
basic Button defaults follow the official
[WinUI Button resources](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/Button_themeresources.xaml)
and [button guidance](https://learn.microsoft.com/windows/apps/develop/ui/controls/buttons):
32 epx control height, 120 epx short-label minimum, `11,5,11,6` content padding
and 4 epx `ControlCornerRadius`; explicit layout and style methods remain
authoritative.

The optional `dialog` feature adds `content_dialog(id, open, spec, page)` as a
compositional modal layer, not a native child-control driver. Applications own
the open flag and receive a typed `ZsContentDialogResult`; the framework owns
the scrim, platform-specific action order and metrics, one trapped focus scope,
pointer feedback, Escape/Tab/arrow/Enter/Space routing and overlay paint order.
The same draw and interaction plans feed buffered Win32, AppKit and Linux hosts,
so no HWND, Objective-C object or GtkWidget enters the public view API.

The optional `toast` feature adds `toast_presenter(id, toast, page)` for
nonmodal foreground feedback. The application owns `Option<ZsToastSpec>` and a
stable `ZsToastId`; the shared runtime owns the active timeout and emits a typed
action, close, Escape or timeout result. The renderer exposes one semantic
action plus an always-available close control and chooses Windows, macOS or GTK
metrics internally. This is deliberately an in-window feedback surface rather
than a copy of Notification Center or Windows app-notification chrome. Windows
uses the non-targeted TeachingTip placement model, macOS stays understated for
foreground delivery, and GTK follows the AdwToast one-action/close structure.
All three remain in the existing self-drawn tree and buffered paint path.

The optional `info-bar` feature adds `info_bar(id, spec)` as an ordinary inline
status surface rather than an overlay or native child control. The application
owns presence/removal and receives typed action or close events. Severity,
title, message, one optional action and the default-on close affordance remain
explicit in `ZsInfoBarSpec`; the shared runtime owns control focus, arrow-key
movement and Enter/Space/Escape routing. Windows uses Fluent InfoBar geometry,
macOS uses a restrained rounded status surface rather than modal `NSAlert`
chrome, and GTK uses AdwBanner-like compact metrics. Severity is always carried
by both semantic icon and text, and all hosts consume the same draw plan.

The optional `teaching-tip` feature adds
`teaching_tip(id, open, target_id, spec, page)` as a targeted, nonmodal overlay.
The application owns visibility and stable presenter/target IDs; the shared
layout resolves the target's final bounds, flips and clamps the bubble within
the current viewport, and keeps action/close results typed. The page remains in
ordinary layout and interactive. Windows, macOS and GTK select TeachingTip,
NSPopover-like and GtkPopover-like metric profiles while consuming the same
self-drawn surface and `FillTriangle` tail command; no child native widget or
global target registry is introduced.

The optional `breadcrumb` feature adds `breadcrumb_bar(items)` as a compact
root-to-current navigation path. Applications own immutable
`ZsBreadcrumbItem` values, stable `ZsBreadcrumbId` identities and the explicit
overflow-open flag; the framework owns transient semantic focus, width-aware
collapse, popup placement and typed selection/expanded events. The bar is one
Tab stop with internal arrow/Home/End navigation. Windows and GTK place the
ellipsis before the surviving trailing path, while the macOS profile preserves
the root before the ellipsis when space permits. All three profiles use the
same self-drawn View, hit plan and popup overlay. GTK has no public breadcrumb
widget to wrap, so its metrics are a ZSUI profile informed by GNOME navigation
and Adwaita conventions rather than a false native-control claim.

The optional `grid-view` feature adds `grid_view(items)` as a responsive,
single-select gallery control. Applications own immutable item data, stable
`ZsGridViewItemId` identities and selected state; the framework derives
equal-width columns from the final bounds and emits separate typed selection
and invocation events. One shared plan owns tile paint and hit geometry, while
one root Tab stop routes two-axis arrows, Home/End, Space and Enter. Windows,
macOS and GTK select Fluent GridView-, NSCollectionView- and GtkGridView-like
metric profiles internally, but no backend creates a child collection widget
or stores an application collection model. Scrolling and virtualization remain
outside this bounded first pass and are explicit readiness work.

The optional `color-picker` feature adds `color_picker(state)` as a compact
color well with an expanded color editor. Applications own the selected
`Color`, expanded flag, active `ZsColorChannel` and alpha policy in
`ZsColorPickerState`; the framework owns viewport-aware popup placement,
shared paint/hit geometry, pointer dragging and one root Tab stop with typed
channel, value and expanded events. Windows uses a Fluent ColorPicker-like
square HSV spectrum, hue track, preview and RGBA sliders; its first-pass
spectrum remains at least 256 DIPs high while editable precision fields are
absent. macOS uses NSColorWell/custom-panel-like compact slider metrics, and
GTK uses a ColorDialogButton-like entry and HSV editor surface. All remain
self-drawn through the existing render protocol; no backend creates or drives
a child picker.
This bounded first pass follows [Microsoft ColorPicker](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/color-picker),
[Apple color wells](https://developer.apple.com/design/human-interface-guidelines/color-wells),
[AppKit NSColorWell](https://developer.apple.com/documentation/appkit/nscolorwell)
and [GTK4 ColorDialogButton](https://docs.gtk.org/gtk4/class.ColorDialogButton.html).
Editable RGB/HSV/hex fields, swatches, eyedropper, accessibility range
providers, HDR/color-space management and AppKit/GTK target evidence remain
explicit readiness work. This is a platform-informed first pass, not a claim
of control-for-control WinUI 3 parity.

The optional `command-palette` feature adds
`command_palette(widget, open, query, items, page)` as a keyboard-first modal
overlay. Applications own immutable command metadata, stable
`ZsCommandPaletteItemId` identities, query, highlight and open state, then
execute the typed invocation message themselves. ZSUI owns stable
case-insensitive all-term substring filtering, a bounded eight-row render/hit
plan, disabled-item skipping and the modal search focus scope; it never owns a
global accelerator, persistence or product command behavior. Windows uses a
Fluent/PowerToys launcher-like profile, macOS an NSSearchField/Spotlight-like
profile and GTK a SearchEntry/list-popover-like profile. These are three
platform-informed self-drawn metric sets, not wrapped child controls or one
platform skin reused everywhere. The design follows
[PowerToys Command Palette](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/overview),
[WinUI commanding](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/commanding),
[AppKit NSSearchField](https://developer.apple.com/documentation/appkit/nssearchfield),
[GTK4 SearchEntry](https://docs.gtk.org/gtk4/class.SearchEntry.html) and
[GTK4 Popover](https://docs.gtk.org/gtk4/class.Popover.html). Fuzzy and
multilingual ranking, recent-command storage, result virtualization,
search-dialog/result-list accessibility semantics and AppKit/GTK target proof
remain explicit readiness work.

For the reusable WinUI-style layout pattern, `src/shell_layout.rs` adds
`ZsShellLayoutSpec` / `ZsNavigationScaffoldSpec`. This is a generic self-drawn
surface contract, not a settings-storage model: it describes a left navigation
pane, right content header, grouped cards, rows, description text, row
accessories and action buttons, then emits stable layout regions and a
product-neutral `NativeDrawPlan` for host renderers. The layout math is
centralized here, including the fixed card spacing, viewport mask and scrollbar
formulas.
`NativeWindowBuilder::shell_layout(...)` now keeps that shell as live runtime
state on the direct Win32 path. The normal event loop routes
navigation hover/selection and scrollbar pointer math, plus product-neutral row
accessory/action events, then replaces the buffered draw plan and invalidates
the window. This closes the first visible input-state-paint loop while keeping
application settings outside the framework.

`src/workbench.rs` defines the higher-level desktop workbench family. Its
declaration separates navigation history, message content blocks, composer
actions and inspector content from application commands and persistence. The
layout produces stable DPI-aware regions and a product-neutral
`NativeDrawPlan`; `ZsWorkbenchRuntime` adds hit testing, bounded timeline
scrolling, conversation selection, sidebar collapse and inspector-tab state.
`NativeWindowBuilder::workbench(...)` is the compact static native-window entry
while direct native event-loop routing remains a separate completion gate.

`src/document_shell.rs` defines a smaller document-editor composite. It owns
the DPI-aware tab strip, command bar, editor-frame inset, status surface,
semantic icon commands and pointer hit regions. A host can place a native text
service inside `ZsDocumentShellLayout::editor_content`, preserving IME and
accessibility while the surrounding surface uses the buffered self-draw path.
The shell does not own files, encodings, accelerator dispatch or platform
handles; those remain host/application services.

`src/calculator.rs` defines a feature-gated standard-calculator slice.
`ZsCalculatorEngine` owns decimal values, pending operations, repeated equals,
memory and history through typed actions. `ZsCalculatorShellSpec` owns the
DPI-aware header, display, memory row, keypad, history panel, semantic draw
plan and hit regions. The reusable layer has no raw HWND ownership; the Windows
example currently supplies the mouse, keyboard and lifecycle loop. Scientific,
programmer, graphing and conversion modes remain separate future surfaces.

The workbench does not own private glyph strings or an independent visual
palette. `src/style.rs` defines the shared Fluent grid, control/card radii,
control metrics, type ramp and semantic colors. `src/icon.rs` defines semantic
`ZsIcon` values and platform symbol names. A workbench draw plan emits
`NativeDrawCommand::Icon`; the Windows GDI sink resolves that command with
Segoe Fluent Icons and uses a raster asset only when `Original` color mode is
requested. This mirrors the IconElement/IconSource boundary without exposing a
Windows font code point in component code.

Text nodes carry `SemanticTextStyle` rather than raw widget-local sizes. The
shared Fluent type ramp exposes caption 12/16, body 14/20, body-large 18/24,
subtitle 20/28, title 28/36, title-large 40/52 and display 68/92 roles. Heading
roles default to semibold 600. On Windows, the GDI sink selects the installed
Segoe UI Variable Small, Text and Display optical families, falls back to
Segoe UI when the variable family is unavailable, and converts the DIP font
size with the current window DPI before creating an `HFONT`. AppKit keeps the
system `NSFont` family and GTK4 keeps its Pango system family. Semantic icon
fonts use grayscale antialiasing on Windows so small chevrons do not acquire
ClearType color fringes.

`src/component_catalog.rs` is the authoritative component readiness inventory.
It distinguishes first-pass runtime surfaces from contract-only and not-started
components, so a WinUI analogue name is never sufficient evidence by itself.

The reusable desktop bridge for product adapters is `NativeWindowRuntimeDriver`.
It maps `ProductUiProjection` startup requests into ZSUI window, status
item/menu and settings declarations and can be used by
`ZsuiReusableRuntimeHarness`. Status item and settings startup declarations now
pass through `NativeStatusItemHost` and `NativeSettingsPageModelHost`, and the
status/settings action contracts live in `src/native_host_actions.rs`.
`NativeWindowRuntimeDriver` also implements
status-menu command dispatch and bound settings-item updates so native backends
have a concrete operation surface instead of only a stored startup snapshot.
The reusable self-draw command shape is represented in
`src/render_protocol.rs` as `NativeDrawPlan` and `NativeDrawCommandSink`. The
protocol includes a filled triangle primitive for targeted overlay tails;
Win32 GDI, AppKit `NSBezierPath` and GTK Cairo translate it locally. The
Windows GDI implementation lives in `src/windows_gdi_renderer.rs`; future platform renderers should
translate the same commands to Direct2D, AppKit, GTK snapshot APIs, Android
Canvas or Harmony Canvas only when that backend needs it, without leaking
product state into the drawing layer.
Win32 main/quick window style mapping, transient-window host, create-params,
message-loop wrapper and `NativeMainWindowHost` implementation live in
`src/platform/windows/mod.rs`. That direct host is available as
framework code and the one-line `native_window(...).run()` convenience path uses
it on Windows. The Win32 paint path suppresses background erase and renders through buffered paint
before presenting to the target HDC. It can now also attach a product-neutral
`NativeDrawPlan` to an `HWND`, so the GDI sink paints real framework draw
commands instead of only a background fill. The GDI path
uses internal RAII wrappers for buffered paint, window HDC acquisition,
compatible memory DCs, smoke-screenshot HBITMAPs, owned main/quick HWND cleanup,
owned HICON app-icon resources, brushes, pens, fonts and selected-object
restoration. Window icon paths are now declaration-audited and loaded into
owned HICON app-icon resources. Win32 tray icons now have a
`Shell_NotifyIconW` backed RAII owner and a `WindowsWin32StatusItemHost`; the
direct Windows `NativeWindowHost` path can now create declared status items,
and native smoke can request one with `native_smoke_run --tray`. Status menus
now have native command-id table routing, RAII popup-menu creation/cleanup and
`TrackPopupMenu` selection routing, but a target-smoke artifact that exercises
real user popup selection is still required before the native backend can be
called complete.
Target proof still requires the platform smoke artifacts in
`docs/native-host-smoke.md`.

## Feature-Gated Build

ZSUI should stay usable as a small dependency. The default Cargo feature set is
`window`, `button` and `label`; heavier services and backends are optional.
`src/feature_manifest.rs` mirrors the `Cargo.toml` feature graph for tools and
AI agents through `zsui_feature_manifest()`.

Applications can opt into only the pieces they need:

```toml
zsui = { version = "0.1", default-features = false, features = [
    "window",
    "button",
    "list",
    "scroll",
    "dark-mode",
] }
```

Optional dependencies must stay behind explicit feature gates: `clipboard`
enables `arboard`, `image` enables `png`, `calculator` enables `rust_decimal`,
`desktop-winit` enables `winit`, `windows-gdi` enables `windows-sys`,
`macos-appkit` enables optional `objc2` AppKit bindings, `linux-direct` enables
the lightweight Wayland/X11, Cairo/Pango, built-in symbolic-vector and portal
stack, while `linux-direct-lite` selects the same host with the optional
pure-Rust cosmic-text/swash and tiny-skia renderer. The two renderer features
are additive in Cargo, so `linux-direct` remains authoritative when both are
enabled; a lite-only build must omit `linux-direct`. `linux-system-icons`
optionally adds freedesktop theme lookup plus
GdkPixbuf decoding, and `linux-gtk` enables optional GTK4 compatibility
bindings. Platform-native window, clipboard, file-dialog
and menu adapters therefore do not enter builds that omit their backend
feature. Window adapters own `NSWindow` or Wayland/X11 window instances behind
strong `WindowId` values; clipboard adapters map `ClipboardData::Text`/`Empty`
to the target system clipboard. Native toolkit objects and callback targets
stay out of the public application API. The `window` umbrella selects Win32,
AppKit or `linux-direct` by target; `desktop-winit` remains an explicit blank
fallback and is not completion evidence for AppKit.
Advanced controls should be gated by
widget features or moved into separate crates as they become real
implementations. Avoid global widget registries that instantiate every control
type at startup; public examples should import and build only the controls they
use. Composite conversation/task surfaces remain behind the `workbench`
feature, document-editor chrome remains behind `document-shell`, and the
decimal calculator slice remains behind `calculator`. The
default `window` umbrella enables both desktop backend features while
target-specific dependency sections ensure only the current platform library is
compiled. Cargo features are unified across the dependency graph, so the long-term
shape should prefer a small default `zsui` facade plus split crates or modules:
`zsui-core`, `zsui-shell`, `zsui-render`, `zsui-style`,
`zsui-widgets-base`, `zsui-widgets-input`, `zsui-widgets-list` and
`zsui-widgets-extra`.

## AI Context-Gated Reading

AI documentation follows the same small-core principle as Cargo features.
`docs/ai-agent.md` is the only bootstrap document. It contains stable boundary
rules and a task router, not the full implementation/readiness narrative.

`docs/ai/context-packs.json` maps each task family to:

- a small required file set;
- optional files that are loaded only after a concrete blocker;
- focused verification commands.

`scripts/ai-context.ps1` validates the manifest and prints one selected pack.
`AGENTS.md` makes this sequence the default repository workflow. Detailed
completion material stays in `docs/ai/reference.md`, while
`src/agent_context.rs` remains available for tools that explicitly need the
full machine-readable readiness model.

Context packs are routing metadata, not ownership shortcuts. An agent still
uses `rg` inside required files and can add another pack when a task genuinely
crosses modules. Pack definitions should remain focused, use relative paths,
avoid generated output and be validated in CI. New implementation history must
not accumulate in the bootstrap document.

## Rust-First API Target

The revised long-term target is a Rust-native UI framework, not an inheritance
based control hierarchy. The canonical machine-readable list lives in
`src/framework_goals.rs` and is exposed as `zsui_rust_first_goals()`. The
longer narrative is `docs/framework-goals.md`.
The target also captures the product direction: keep
the one-line `zsui::native_window(...).run()?` path as the normal native-window
entry point, use stable host/rendering behavior as the baseline, add
Android/Harmony as explicit Activity/Ability hosts, and introduce wider
platform API bindings only when a concrete backend needs them.
The first implementation layer lives in `src/view/mod.rs`, `src/style.rs` and
`src/geometry.rs`: typed `View<Msg>` trees, `WidgetId`, explicit app/event/paint
contexts, `ViewInteractionPlan`, feature-gated scroll containers, typed list
selection, `Px`/`Dp`/`Dpi`, `UiLength` and theme tokens.
`src/shell_layout.rs` adds the generic navigation/card shell layout contract on
top of those typed units and draw commands with one shared spacing model.
`ProductViewAdapterHost` connects that typed view layer to product adapters, and
`ZsuiReusableRuntimeHarness::run_view_smoke(...)` verifies the flow from native
view events to typed messages, `AppCx`, product events and reusable
`UiCommand` dispatch.
`typed_native_window(...)` is the opt-in compile-time builder surface. Its
`NativeWindowContentMissing` state exposes configuration and content transition
methods but not `build`/`run`; attaching a View, live View, draw plan or shell
layout returns `NativeWindowContentReady`. The unconstrained
`native_window(...)` entry point remains for legitimate empty native surfaces.

- Use composition and traits for views/components instead of base classes.
- Preserve one-line native window creation for ordinary desktop apps.
- Use typed messages such as `enum Msg` instead of string event names.
- Own windows, fonts, bitmaps, icons and tray handles with RAII wrappers.
- Use buffered no-flicker self-draw as the Windows rendering baseline.
- Use typed units such as `Px`, `Dp` and `Dpi` instead of loose numeric sizes.
- Use the opt-in native-window content typestate when compile-time constraints
  are more valuable than an intentionally empty native surface.
- Avoid global mutable registries; pass explicit app/event/layout/paint contexts.
- Keep public APIs safe and isolate `unsafe` inside backend modules.
- Keep control state explicit in application state and derive UI from state.
- Use theme tokens for colors, radius, spacing and typography.
- Keep the user-facing UI API declarative Rust, without XML or reflection.
- Return `Result<T, ZsuiError>` for backend failures instead of panicking.
- Model platform differences with traits and capability reports.
- Treat Android and Harmony as explicit mobile native hosts, not desktop clones.
- Use Cargo features for widgets, services, platform backends and heavy deps.
- Split large widget/backend families into smaller crates or feature modules.
- Add windows-rs or other wider platform bindings only for concrete backend work.
- Use strong typed IDs for windows, widgets, commands and resources.

## Fully Unified Desktop Authoring Boundary

Ordinary Windows, macOS and Linux applications have one source-level path:

```text
State + Msg + view + update + semantic specs/tokens
                         |
                         v
              framework PlatformExperience
                         |
                         v
       compile-time Host/Text/Raster/Presenter/Services profile
                         |
             +-----------+-----------+
             |           |           |
           Win32       AppKit      Linux desktop
```

The upper boundary is completely platform-neutral. Application View code does
not receive a platform enum, select a renderer, import raw host objects or
duplicate a component tree behind target `cfg`. Semantic parameter changes are
made once through public specs, typed units and theme tokens.

The lower boundary is deliberately not uniform. The framework's platform
experience layer may map the same semantic navigation, toolbar, tab, form,
dialog or popup declaration to different composition and interaction rules.
The statically selected backend profile then supplies the real event loop,
text layout, rasterization, presentation and operating-system services. This
keeps application authoring unified without applying a Windows component tree
or one renderer to every target.

The boundary has three concrete internal layers. `src/platform/experience.rs`
owns the single compile-target selection for semantic component defaults and
maps them to Fluent, AppKit or GTK behavior. `src/platform/backend_profile.rs`
describes Host, Text, Raster, Presenter and Services choices independently.
`src/platform/desktop_runtime/` is the production adapter contract: its single
compile-time selector delegates the event loop, runtime smoke, final-surface
capture, clipboard and native file panels to a target-owned Win32, AppKit,
Linux-direct, GTK compatibility or Winit-fallback module. `native.rs` and the
public desktop-service facades consume that contract
and contain no production or smoke backend selection for those operations.
Win32 GDI capture ownership and Winit smoke lifecycle details stay in dedicated
backend modules; AppKit, Linux-direct and GTK convert target results into one
platform-neutral proof report at the adapter boundary. Adding another desktop
backend therefore adds one adapter implementation without changing application
authoring, desktop-service dispatch or the shared host loop. Public View
builders do not accept the internal experience or the low-level render-proof
`PlatformStyle` enums.
Regression tests scan the
Gallery, Notepad and desktop showcase authoring slices so target `cfg`, platform
enums and raw native handles cannot silently return to ordinary `view`/`update`
code.

The retained public View payload is semantic as well: toolbar buttons and
adaptive navigation nodes store icons, labels, item counts and state, but no
platform selector. Framework construction plus adaptive layout, paint and hit
testing resolve the framework-owned experience. Deterministic cross-platform
proof may attach a crate-private style override to `ViewNode`; that override is
not an application parameter and is absent from normal public builder output.

Layout, paint, hit testing, caret/selection geometry and accessibility must
consume the same backend text-layout result. Typography, clipping and density
corrections therefore belong in reusable framework text/style contracts rather
than in Gallery or Notepad. Cargo features remain orthogonal: one API shape does
not imply one always-enabled binary surface.

## Host Boundary

Applications call `ZsuiHost` operations:

- create and show/hide windows
- create tray/status menus
- register global hotkeys
- read/write clipboard data
- open file pickers
- show native dialogs
- poll events and run the host event loop

Unsupported features return `ZsuiError::Unsupported` or appear in
`HostCapabilities` degradation reports. A host may accept a window declaration
and still downgrade unsupported traits such as transparency or always-on-top.
Native backend completion also requires the smoke artifact contract in
`docs/native-host-smoke.md`; code-level host contracts alone are not proof that
the target OS integration is complete.
