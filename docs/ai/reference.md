# ZSUI Full AI Reference

This is an optional deep reference for completion/readiness audits. Normal
tasks start at `docs/ai-agent.md` and load one task pack from
`docs/ai/context-packs.json`; they should not load this file by default.

## Current Completion

ZSUI is roughly 64% complete as a standalone framework product, including the
still-scaffolded Android target. The previously measured desktop-only native
application areas are roughly 75% complete, but v0.2 now also contains a new
reloadable UI authoring gate with its schema, validator and native Viewer first
pass implemented. Do not use the previous 75% as a recalibrated total for the
expanded milestone, and do not use component-level milestones as overall
framework readiness.

- Foundation contracts: about 78% complete.
- Declaration API: about 85% complete.
- Component library: 100% first-pass runtime coverage (48 runtime surfaces out
  of 48 catalogued component families); readiness gaps remain per component.
- Minimal native window runtime: about 89% complete.
- Feature-pruned architecture: about 55% complete.
- Rust-first API model: about 90% complete.
- Reloadable UI authoring: about 97% complete; schema version 1, the typed
  `State`/`Msg` binding manifest, `zsui-uic check` and the prebuilt native
  auto-reload Viewer have a first pass. Accepted reloads now report stable-ID
  compatibility, preserve native focus/text selection/editor viewport for
  compatible controls, and clear incompatible focus/text/drag/IME state. Text,
  toggle and slider value actions use owned typed control callbacks and update
  explicit bound Viewer state across rebuilds. `zsui-uic handoff` now emits a
  deterministic canonical package with document, binding/value snapshots,
  optional final native PNG metadata, stable node indexes, feature requirements
  and component contracts. `zsui-uic embed` and the feature-pruned
  `ui-document-runtime` now provide a deterministic, versioned embedded
  artifact plus reusable `UiDocument`-to-`ViewNode<Msg>` compilation without
  linking Viewer, watcher or preview code. Controlled scroll offset now
  survives View rebuilds through explicit number bindings, and Win32 Viewer
  smoke routes a fixed native scroll before final capture. Viewer proof now
  has a versioned target identity, logical/pixel window metrics and
  deterministic node/layout snapshot. Native UI Proof run `29883039068` passes
  the same controlled-scroll document on fixed AppKit and Linux jobs, with one
  handled scroll, one typed Viewer message, final platform-surface PNGs and
  runtime memory evidence. All 48 catalog components are document-ready,
  including Toast, InfoBar, ContentDialog, Image, ItemsRepeater, SettingsCard
  and the typed Workbench composition; NumberBox
  adds a nullable numeric contract, while ComboBox adds homogeneous string
  options plus controlled nullable selection and expanded state. Tabs maps each
  direct child's stable ID to a typed content slot, semantic header and
  controlled string selection. Grid compiles typed fixed/fraction tracks and
  a complete stable-child-ID placement map, retaining cell identity across
  sibling reordering and rejecting invalid spans or bounds before native
  layout. TimePicker uses canonical `HH:MM` state, typed `ZsTime` manifest
  helpers, validated minute increments and separate controlled value/expanded
  loops while keeping target display formatting platform-owned. ColorPicker
  adds canonical uppercase `#RRGGBBAA`, typed `Color` manifest helpers and
  separately controlled color, expanded and active-channel state while keeping
  WinUI, AppKit and GTK rendering profiles platform-owned. Local Win32 Viewer
  proof changes both channel and RGBA state through one native click with no
  unhandled input and retains the final platform-surface PNG. A catalog audit
  prevents future components from silently omitting a document schema.
  Accepted reloads compare property-binding type and secure/ordinary storage
  class, preserve compatible values and filter removed or incompatible values
  before the new View is compiled. Native UI Proof run `30069326871`, job
  `89406615062`, passes the fixed `windows-2025` Win32 Viewer reload scene:
  revision 2 preserves all four Workbench nodes and `timeline_offset`, resets
  removed `composer_draft` state explicitly, captures the final 960x640
  `WM_PRINTCLIENT` surface, uses the Win32 system UI font and records process
  memory. Broader AppKit/Linux reload interaction evidence remains.
- Full desktop native host implementation: about 94% complete; product
  readiness remains lower until broader AppKit and Linux IME, accessibility and
  per-control target evidence exists.
- Android: about 32% complete.
- Product adapter/runtime harness: about 67% complete.
- Native smoke verification: about 88% complete.

The Windows implementation is further ahead than the overall
framework: its window, draw-plan, stateful View and shell-layout foundation is
roughly 76%
ready. macOS/Linux native product hosts and the real Android runtime keep
cross-platform product readiness substantially lower. Report these separately.

The machine-readable audit tracks 18 required native capabilities per platform:

- Windows: 2 ready, 8 first-pass runtime implementations, 8 contract-only.
- macOS: 0 ready, 8 first-pass runtime implementations, 10 contract-only.
- Linux: 0 ready, 8 first-pass runtime implementations, 10 contract-only.
- Android: 0 runtime implementations, 18 contract-only.

Use `native_ui_platform_readiness_reports()` for current capability-level
evidence instead of inferring platform completeness from backend registration.
Use `zsui_component_catalog_summary()` for component coverage: all 48 families
have a first-pass runtime surface; none are contract-only or not started. A
composite workbench does not make its underlying contract-only controls
complete. WebView is intentionally outside the v0.2 product boundary.

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

It is not yet a complete application UI runtime. AppKit and Linux now have
first-pass native hosts, renderers, typed input, clipboard, file-dialog and
menu paths plus final-surface target proof, but their complete input, IME,
accessibility and per-control interaction matrices remain incomplete. The
current Windows backend metadata points to the
`win32_gdi` runtime. Win32 main/quick window style, transient-window host,
create-params, message-loop and `NativeMainWindowHost` implementation live in
`src/platform/windows/mod.rs` and are wired into the default
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
`src/platform/windows/mod.rs` can attach a `NativeDrawPlan` to an `HWND`, then
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
macOS, Linux and Android still require target/device proof.
Android now has an explicit mobile bridge contract in
`src/mobile_host.rs` and `src/android_activity_host.rs`: callback symbols,
lifecycle/surface/input/
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
`--write-contract all <root>` and `--review-contract all <root>` cover the
configured mobile target in one command. It has
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
`clipboard`, `image`, `desktop-winit`, `windows-gdi`, `macos-appkit`,
`linux-gtk` and the internal `text-input-core` are optional-dependency
features. Unicode segmentation is enabled only by text-capable controls, while
non-text `widgets-input` slices remain pruned; advanced widgets stay opt-in.
This is feature/crate based
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
IDs. The expanded v0.2 target also requires versioned semantic UI documents,
typed binding validation, a prebuilt native auto-reload Viewer, deterministic
AI handoff and release embedding without development-only dependencies. It
explicitly defers a full drag-and-drop designer and does not accept browser
projection as native proof. It also records the larger framework target: keep
`zsui::native_window(...).run()?` as the normal native-window entry, make
buffered no-flicker self-draw the Windows baseline, treat Android as a real
Activity host target, and add `windows-rs` or
other broader platform bindings only for specific backend work. The source
target records the preferred and avoided API shapes, such as `enum Msg` over
string events and feature/crate based trimming over global widget registration.
The first concrete Rust-first API pass now exists in `src/view/mod.rs`,
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
selection state. `ZsWorkbenchShellSpec` assembles explicit
`ZsMessageTimelineSpec`, `ZsComposerSpec` and `ZsInspectorPanelSpec` child
contracts; `workbench_shell(...)` builds the View and
`NativeWindowBuilder::workbench(...)` accepts the structured shell or the
flattened compatibility spec. The retained route exposes timeline scrolling,
editable composer text and typed toolbar/message/sidebar/inspector actions;
three-target composer IME, selection, accessibility and interaction-state proof
remain explicit gaps.
Its built-in visuals use separate WinUI, AppKit and GTK component metrics plus
semantic `ZsIcon` commands from `src/icon.rs`. Agents must not add PUA glyph
strings, private component palettes or arbitrary control dimensions to this
module. Windows detects Segoe Fluent Icons and falls back to Segoe MDL2 Assets
in the live GDI renderer. The shared resolver orders SF Symbols on macOS and
GTK symbolic theme names on Linux before the optional MIT Fluent SVG fallback.
AppKit `NSImage` and GTK `GtkIconTheme` runtime lookup remain incomplete, as do
dark/high contrast smoke and complete hover/pressed/focus-visible coverage.
`src/component_catalog.rs` tracks 48 component families, all with a first-pass
runtime surface. The optional
Canvas surface retains backend-neutral primitives in local `Dp` coordinates,
uses semantic color and text roles, emits a balanced clipped native draw plan
and maps pointer or keyboard activation into a typed application message. The
optional Grid surface uses typed fixed/fractional tracks, nonzero spans,
independent row/column gaps, explicit typed cell placement and one DPI-aware
layout result for paint and hit testing on Win32/AppKit/GTK4. Windows has a
real layout/click screenshot artifact; content-sized tracks, baseline
alignment, accessibility grouping and non-Windows target proof remain open.
The feature-gated self-drawn Tabs surface uses strong tab IDs, one active page,
WinUI focus-only arrow navigation on Windows, AppKit selection-style arrows and
GTK4 focus/selection shortcuts. Local Win32 plus GitHub-hosted AppKit, X11 and
real Weston Wayland target proofs click a typed tab header and exercise the
platform Right-arrow rule before final-surface capture. Accessibility and
document-tab close/reorder/overflow remain open.
The independent `time-picker` feature adds validated `ZsTime` and
`ZsMinuteIncrement` values, explicit 12/24-hour formatting, shared popup
placement, and internal WinUI/AppKit/GTK metric profiles on the self-drawn
path. Windows pointer and keyboard interaction has a real smoke artifact;
system-locale formatting, accessibility and non-Windows target runs remain
open.
The independent `number-box` feature adds finite `ZsNumberRange` bounds,
small/large steps, empty values, a draft/commit edit model, internal
WinUI/AppKit/GTK metric profiles, pointer steppers and keyboard stepping.
Windows has a real edit/step/commit screenshot artifact; locale-aware number
formatting, expressions, accessibility, autorepeat/wheel behavior and
non-Windows target runs remain open.
The independent `toggle-button` feature adds an explicit Boolean-state button,
typed pointer/Space activation, transient hover/pressed decoration and internal
WinUI/AppKit/GTK metric profiles. Windows has a real checked-state screenshot
and interaction artifact; indeterminate mode, accessibility and non-Windows
target runs remain open.
The independent `password-box` feature keeps owned values in redacted,
zeroizing `ZsPassword` state and uses secure draw commands that omit secrets
from serialization. Unicode editing, masked IME reports, platform reveal
policies and Windows press-and-hold Peek run through the shared self-drawn tree;
Alt+F8, locked memory, complete accessibility and non-Windows target proof are
still open.
The independent `tooltip` feature attaches concise help text to any stable-ID
`ViewNode` without adding another hit target or native child control. It uses
internal WinUI/AppKit/GTK metric profiles, shared placement and clamping,
delayed pointer hover, immediate keyboard-focus display and timed dismissal.
Win32 reads the system mouse-hover and message-duration settings and has a real
buffered screenshot plus deterministic hover-route coverage. Accessibility
relationships, top-level overflow beyond the current viewport and AppKit/GTK
target-machine evidence remain open.
The independent `flyout` feature wraps one ordinary page and one arbitrary
application View subtree while keeping open state, stable presenter/target IDs
and content state in application code. Shared placement flips and clamps the
platform-specific Fluent, AppKit-popover or GTK-popover profile; a modal focus
scope absorbs the first outside click and emits typed action, Escape or
light-dismiss results. Native UI Proof run `29762236980` captures the final
AppKit, X11 and real Weston Wayland surfaces after the same Flyout action and
verifies the typed message and focused widget. Nested overlays, accessibility
announcements/Flyout-specific AT-SPI actions and complete target Escape,
light-dismiss and resize matrices remain open.
The independent `progress-ring` feature adds active/inactive and validated
determinate/indeterminate state without enabling ProgressBar. Its shared
`StrokeArc` draw command maps to antialiased GDI+, NSBezierPath and Cairo;
Win32, AppKit and GTK4 event-loop timers advance the same self-drawn animation
without application messages or hit targets. Windows has a real buffered
indeterminate/determinate screenshot and repeated background-refresh evidence.
Reduced-motion policy, accessibility and AppKit/GTK target animation artifacts
remain open.
`src/document_shell.rs` is the reusable boundary used by the Windows notepad
benchmark. It provides a document tab, command bar, editor frame, status
layout, semantic draw plan and hit regions plus `ZsTextDocument` UTF-8/UTF-16
loading, explicit dirty state and transactional UTF-8 save/save-as without raw
platform handles. Its file selection now uses the shared
`NativeFileDialogService`, whose owned specs dispatch to Win32, AppKit or GTK4
without application `cfg`. Typed shortcut declarations, Win32 accelerator
resource ownership and Win32 multiline editor hosting now live in the
framework. Native file dialogs bind to the active target window; only
dirty-close policy still lives in the example's platform module. Use
`docs/notepad-demo.md` and `scripts/measure-notepad-comparison.ps1` to compare
ZSUI with the isolated egui, Iced, Slint and Tauri 2 baselines. The script
measures complete process trees, including Tauri's WebView2 descendants. Read
an individual `comparisons/*_notepad` directory only when that baseline is
needed; it is optional AI context, not bootstrap context. Do not interpret the
comparison as completing ZSUI's native text-editor or file-dialog target proof.
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
touch/inertial pointer routing and macOS/Linux target input evidence are still
pending. Left/Right now traverses platform-shaped primary caret positions in
visual x order, including Shift selection and soft-wrap boundaries. Shared text-capable
controls keep navigation, deletion, pointer hits, visual wrapping and IME marked
selections on Unicode extended-grapheme boundaries; Uniscribe, Core Text and
Pango provide the proportional advances, visual cluster boxes and primary/secondary
caret offsets consumed by paint, selection, hit testing, wrap, horizontal reveal
and candidate-window anchoring.
The optional `accessibility` feature adds a native focused-text semantic bridge
without embedding a platform editor or browser surface. Win32 answers UI
Automation root requests from `WM_GETOBJECT` with an Edit provider and a
read/write ValuePattern plus TextPattern for ordinary text. The text provider
exposes document, selection and visible ranges; range cloning/comparison,
movement, search, point hit testing, native shaped bounding rectangles and
typed selection/ScrollIntoView routing stay on the existing self-drawn input
route. Protected text is masked and advertises neither pattern. AppKit exposes focused text role,
value, selection, UTF-16 ranges, frame and protected-content selectors on the
custom `NSView`. GTK4 keeps its TextBox semantic surface hidden until a text
target is focused and updates native value/multiline/read-only properties. UIA
rich attributes/embedded-object ranges and real AppKit/GTK screen-reader target
artifacts are still pending.
`scripts/check-windows-text-accessibility.ps1` is the real Windows gate: it
launches the hidden native notepad HWND, focuses the self-drawn editor through
Win32 messages, resolves the provider through UI Automation and verifies the
ZSUI Edit identity plus readable ValuePattern/TextPattern, single selection,
range movement, ScrollIntoView routing and native shaped bounding rectangles. AppKit/GTK target
assistive-technology proof is still pending.
The feature-gated `combo_box(...)` owns explicit selected and expanded state,
emits typed selection/expansion messages, and paints its popup in a final
overlay pass so later layout siblings cannot cover it. Overlay option hit
targets win normal hit testing without becoming duplicate Tab stops. Win32 and
the shared AppKit/GTK4 input runtime route pointer selection plus
Enter/Space/Up/Down/Home/End/Escape keyboard behavior; the dedicated
`native_smoke_run --combo-view` path records real Windows selection, keyboard
selection, expansion, command execution and screenshot evidence. Viewport
flipping, outside-click/focus-loss dismissal, long-option scrolling,
accessibility providers and AppKit/GTK4 target-machine evidence remain open.
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
- Rust-first view API: `src/view/mod.rs`
- Reusable widget geometry: `src/widget_render.rs`
- AppCx/UI command executor boundaries: `src/app_command.rs`,
  `src/command_protocol.rs`
- WinUI-style navigation/card shell layout API: `src/shell_layout.rs`
- Conversation/task workbench API: `src/workbench.rs`
- Document editor shell API: `src/document_shell.rs`
- Component readiness catalog: `src/component_catalog.rs`
- Notepad integration and five-framework benchmark: `docs/notepad-demo.md`,
  `examples/zsui_notepad.rs`, `scripts/measure-notepad-comparison.ps1`;
  isolated baselines live under `comparisons/*_notepad`
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
  `src/platform/windows/mod.rs`
- Host contracts: `src/host.rs`, `src/host_protocol.rs`, `src/native_hosts.rs`,
  `src/native_host_actions.rs`
- Adapter discovery: `src/native_adapter_manifest.rs`
- Launch planning: `src/native_host_launch.rs`
- Mobile host scaffolds and bridge contracts: `src/mobile_host.rs`,
  `src/android_activity_host.rs`,
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
6. Turn the Android Activity bridge contract into a real FFI/runtime
   implementation with device smoke artifacts.
7. Expand the non-clipboard product adapter example into a target native smoke
   harness.
8. Expand host capability reporting so agents can choose supported APIs without
   reading platform code.

## Machine-Readable Progress

Use these public functions when another AI, tool or product adapter needs a
stable context without reading prose:

- `zsui_agent_context()`: full framework, platform, completion and gate context.
- `native_ui_platform_readiness_reports()`: capability-level runtime evidence
  and contract-only gaps for all four target platforms.
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
- `ZsWorkbenchShellSpec`, `ZsMessageTimelineSpec`, `ZsComposerSpec`,
  `ZsInspectorPanelSpec`, `workbench_shell(...)`, `ZsWorkbenchRuntime` and
  `NativeWindowBuilder::workbench(...)`: reusable navigation, message timeline,
  composer and inspector contracts with target-owned geometry, DPI-aware layout
  and hit regions.
- `items_repeater(...)`, `image(...)` and `settings_card(...)`: named public
  Rust constructors shared by ordinary View code and the UiDocument compiler;
  the older `virtual_list`, `image_preview` and `section` entry points remain.
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
- `mobile_runtime_host_scaffold(platform)`: Activity bridge, lifecycle,
  capability and device-smoke scaffold for Android.
- `mobile_runtime_bridge_contract(platform)`: Android FFI callback,
  lifecycle, surface, input, command and device-smoke artifact contract.
- `mobile_runtime_bridge_contracts_json()`: JSON form of the mobile bridge
  contract for AI/tool handoff.
- `mobile_runtime_bridge_parity_report(platform)`: Android scaffold vs
  contract report covering required callback route kinds, pending FFI callback
  symbols and whether device smoke is still blocked.
- `mobile_runtime_bridge_parity_reports_json()`: JSON form of the mobile
  parity report for AI/tool handoff.
- `mobile_runtime_bridge_dispatch_report(platform)`: Android callback
  dispatch report mapping required bridge symbols to lifecycle, surface, input
  and `NativeRuntimeDriver` operations without claiming FFI implementation.
- `mobile_runtime_bridge_dispatch_reports_json()`: JSON form of the mobile
  dispatch report for AI/tool handoff.
- `mobile_runtime_bridge_contract_smoke_report(platform)`: local Android
  contract smoke that replays required bridge dispatch steps and reports
  dispatch-operation coverage without claiming device proof.
- `mobile_runtime_bridge_contract_smoke_reports_json()`: JSON form of the
  mobile contract dispatch smoke report for AI/tool handoff.
- `write_mobile_runtime_bridge_contract_artifacts(platform)`: writes local
  Android bridge contract artifacts, the device-smoke plan and the
  current ZSUI agent context without generating device launch, screenshot,
  lifecycle, surface or input proof.
- `review_mobile_runtime_bridge_contract_artifacts(platform)`: validates local
  Android bridge contract artifacts and expected JSON schema without
  claiming device proof.
- `write_mobile_runtime_bridge_contract_artifacts_for_all()` and
  `review_mobile_runtime_bridge_contract_artifacts_for_all()`: write/review
  the configured mobile contract artifacts in one call.
- `mobile_runtime_device_smoke_plan(platform)`: required Android device
  artifact plan without faking target proof.
- `mobile_runtime_device_smoke_trace_templates(platform)`: lifecycle, surface,
  input and optional clipboard trace JSON templates expected by
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
  counts plus Win32 click/text/toggle/list/combo selection/keyboard, Tab focus
  traversal and keyboard-list selection to `UiCommand` routing in the smoke
  report. `NativeWindowSmokeRunOptions::native_view_scroll(...)` and the Win32
  input route can also record scroll counters when a smoke path supplies a
  scroll target; `native_smoke_run --scroll-view` exercises that path.
  `native_smoke_run --combo-view` additionally proves popup overlay painting,
  pointer selection and keyboard selection/expansion counters.
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
