# ZSUI Durable Project Memory

This file records durable product and engineering decisions. It is not a
progress log. Current code, the component catalog, target artifacts and Git
history remain authoritative for implementation status.

## Product boundary

- ZSUI is an independent, publishable, Rust-first native UI framework.
- Work for this project changes only the ZSUI repository. Sibling product
  repositories are read-only implementation references.
- Reuse proven generic platform techniques where useful, but keep product
  data, persistence, synchronization and business workflows outside ZSUI.
- Public documentation describes ZSUI on its own terms and does not carry
  source-product or migration history.

## Public application experience

- Preserve one shared Rust application shape across Win32, AppKit and Linux:
  `native_window(...).stateful_view(...).run()`.
- Application code must not expose platform `cfg`, raw handles, Objective-C or
  GTK objects, drawing handles, or native event loops.
- A concise native-window entry is important, but it is only the bootstrap
  contract. The real objective is a complete native application loop.
- Controls and advanced capabilities should remain Cargo-feature selectable so
  unused surfaces and heavy dependencies can be omitted.
- v0.2 additionally requires a versioned semantic UI document, typed binding
  validation and a prebuilt native Viewer that can reload visual-only changes
  without invoking Cargo. Existing Rust builders remain supported and may mix
  with document-backed subtrees; the document format must not introduce
  reflection, a global string event bus or platform types into application UI.
- The reloadable authoring path must emit deterministic structured data for AI
  editing and support release embedding without file watchers, preview
  transport, another mandatory process or other development-only dependencies.
  Stable IDs preserve compatible focus, selection, scrolling and control state.
  UiDocument `scroll` owns exactly one child, nonnegative `content_height`, an
  optional controlled `offset_y` number property and a typed `scroll` number
  action. Viewer updates the explicit offset binding before rebuilding, and
  layout clamps restored offsets to the current content range. Native scroll
  smoke must travel through the host input route rather than mutating the
  shared View directly.
  Viewer smoke reports use the versioned `zsui.ui-viewer-proof/v1` schema and
  include target capture identity, logical/pixel window metrics and a
  deterministic preorder node/layout snapshot. Fixed AppKit and Linux proof
  must execute the same document and typed scroll input, retain the final
  platform-surface PNG, and reject missing messages, memory or handled-scroll
  evidence. Native UI Proof run `29883039068` on commit
  `348808b6f5b862d90c19d8687a15f991e8790344` is the accepted first target
  evidence: both reports contain the same 15-node document snapshot, one
  handled scroll, one typed Viewer message and a final target PNG. It measured
  about 61.83 MiB on AppKit and 27.26 MiB on Linux Direct; these are
  run-specific resident-memory observations, not universal benchmarks.
- Authoring contracts live behind the optional `ui-document` feature.
  `src/ui_document.rs` owns schema version 1, typed layout/theme/localization/
  accessibility fields, `UiBindingManifest<State, Msg>` and deterministic
  validation. `zsui-uic check` consumes an application-exported binding schema;
  component availability remains tied to Cargo features and the framework
  component catalog.
- Typography is a framework contract, not a Demo detail. Windows resolves the
  Win32 `SPI_GETNONCLIENTMETRICS.message_font` at runtime, matching the ZSClip
  system message font (currently `Microsoft YaHei UI` on the proof host); the
  Segoe UI profile is only a fallback when the system family cannot be loaded.
  AppKit and GTK similarly resolve their host system families. `text` and
  `styled_text` keep Demo code on semantic `TextRole` values, while the shared
  `resolve_semantic_text_style` path supplies the same family, metrics,
  weight and layout flags to every native renderer.
- UiDocument PasswordBox values never use the ordinary JSON property/action
  channel. Documents bind `password_box.value` to a name registered with
  `UiBindingManifest::register_secret_property`; changes use
  `register_secret_action` and `UiDocumentSecretAction`. Runtime and Viewer
  state keep `ZsPassword` in non-serializable `UiSecretValues` storage.
  Literal, localized and `values.json` password values are validation errors;
  AI handoff exposes only the binding name and a `sensitive` contract marker.
- The prebuilt native development host lives behind the separate optional
  `ui-viewer` feature. `zsui-viewer` polls document and binding files inside the
  existing native live-View refresh loop, keeps the last valid document after
  invalid edits, retains ordinary `UiViewerState`, and derives stable WidgetIds
  from author IDs in a reserved namespace. Accepted reloads expose a
  deterministic preserved/added/reset report. The native input runtime keeps
  focus, text selection and text-editor viewport for compatible stable IDs and
  clears stale focus, selection, drag and IME state after removal or control
  class changes. Text, toggle, slider and scroll actions capture
  node/action/property identities in per-control `ViewMessageMapper` callbacks,
  carry typed JSON payloads and update explicit property bindings across View
  rebuilds. Document-ready NumberBox uses one `nullable_number` value/action
  contract and validates minimum, maximum, step, large step, fraction digits
  and wrapping before compilation. Document-ready ComboBox uses `string_array`,
  `nullable_integer`, `integer` and boolean contracts; selected index and
  expanded state update explicit bindings through owned callbacks so both
  survive View rebuilds. Document-ready DatePicker represents serialized
  calendar values with a canonical ISO `date` type. `register_date_property`
  and `register_date_action` keep application bindings strongly typed as
  `ZsDate`; selected date, first-of-month navigation state and expanded state
  use independent controlled property/action loops. Authoring validation and
  release compilation reject invalid dates, inverted or violated ranges and
  noncanonical visible months. Viewer native smoke accepts repeatable fixed
  click sequences and requires typed message evidence before accepting them.
  Document-ready TimePicker serializes wall-clock state as canonical `HH:MM`
  `time` values while retaining platform-owned display formatting.
  `register_time_property` and `register_time_action` keep Rust bindings typed
  as `ZsTime`; value and expanded state use independent controlled loops.
  Minute increments must be nonzero divisors of 60 and selected minutes must
  align with the increment in both schema validation and release compilation.
  Document-ready ColorPicker uses canonical uppercase `#RRGGBBAA` values and
  typed `Color` manifest helpers. Value, expanded state and active channel use
  independent controlled loops. The semantic channel values are `red`,
  `green`, `blue` and `alpha`; disabling alpha rejects nonopaque colors and an
  active alpha channel instead of silently normalizing document state. Windows
  Viewer proof changes the active channel and canonical RGBA value through one
  real pointer click and captures the final Win32 surface.
  Document-ready AutoSuggestBox stores suggestions as a typed array of stable
  semantic string IDs and display text. Query, nullable highlighted ID and
  expanded state use independent controlled bindings; choose emits the stable
  ID and submit emits both query and optional chosen ID. The release runtime
  derives private numeric `ZsAutoSuggestionId` values from the owning node and
  semantic item ID, rejects collisions or missing highlights, and never uses
  declaration order as identity. Windows Viewer proof submits one suggestion
  through a real pointer click and retains the final Win32 surface.
  Document-ready CommandPalette stores command metadata in a typed array of
  stable semantic string IDs, titles, optional subtitles, search keywords,
  shortcut labels, semantic icons and enablement. Query, nullable highlighted
  ID and open state use independent controlled bindings; highlight and invoke
  emit the stable ID. The release runtime derives private numeric
  `ZsCommandPaletteItemId` values from the owning node and semantic item ID,
  rejects collisions plus unavailable highlights, and never executes product
  commands. Windows Viewer proof invokes one command through a real pointer
  click, closes the controlled overlay and retains the final Win32 surface.
  Document-ready TreeView stores one recursive typed node array whose stable
  semantic string IDs are unique across the whole hierarchy. A deduplicated
  ID array owns the complete expanded set, while a nullable ID owns selection;
  selection and invocation emit one semantic ID and expansion emits the full
  next set. The release runtime derives private `ZsTreeNodeId` values from the
  owning document node plus semantic node ID, rejects collisions, unknown
  selections and non-expandable entries, and preserves hidden selection when
  an ancestor collapses. Windows Viewer proof selects and invokes one real row
  through a pointer click and retains the final Win32 surface.
  Document-ready GridView stores one typed tile array with unique stable
  semantic string IDs, non-empty titles, optional subtitles and semantic
  icons. A nullable item ID owns explicit single selection; selection and
  invocation each emit one semantic ID. The release runtime derives private
  `ZsGridViewItemId` values from the owning document node plus semantic item
  ID, rejects collisions and unknown selections, and never uses responsive
  column position as identity. Windows Viewer proof selects and invokes one
  real tile through a pointer click and retains the final Win32 surface.
  Document-ready Tabs treats each direct child as one
  typed content slot: the child's stable `UiNodeId` derives the internal
  `ZsTabId`, keys its required label and optional semantic icon, and is the
  string value emitted by the controlled selection action. Document-ready Grid
  stores fixed-DP/fraction tracks in semantic `grid_track_array` values and
  keys every `grid_placement_map` entry by a direct child's stable `UiNodeId`.
  The map must cover exactly those children; positive spans and row/column
  bounds are checked both for inline documents and again after resolving bound
  release values. Child declaration order therefore does not determine cell
  identity. Document-ready List also uses each direct child's stable
  `UiNodeId` as the controlled string selection value and emits that ID through
  its typed select callback, so item reordering does not corrupt selection. The
  shared List builder floors direct rows at the platform selection height and
  applies the platform spacing inset to row content; typography scaling raises
  that hard floor instead of compressing mixed-script line boxes.
  Document-ready ContentDialog owns exactly one page child, explicit open
  state, semantic title/content/button labels and optional default/destructive
  button roles. Validation rejects empty required copy, unavailable roles and
  conflicting default/destructive roles. Its result action emits `primary`,
  `secondary` or `close` through an owned typed callback; bound `open` state
  requires a Boolean `open_change` action and receives `false` after a response,
  so Viewer rebuilds cannot reopen a dismissed dialog. Release compilation does
  not add a global event registry.
  Document-ready InfoBar is an inline semantic surface with required non-empty
  message, optional title/action label, four validated severities and a
  default-true closable flag. Its typed event binding emits only `action` or
  `close`; application state decides whether the next View removes it. Runtime
  compilation owns `ZsInfoBarSpec`, while each platform profile retains height,
  icon, action-button, corner and spacing metrics. The callback is an owned
  node-local mapper, so document reload keeps binding identity without a global
  control registry.
  Document-ready Toast wraps exactly one page child and keeps `open`,
  non-empty `message`, optional action label and short/long/persistent duration
  explicit. Its result binding emits `action`, `close`, `escape` or `timeout`;
  a separate typed `open_change` binding emits `false` for every response so
  Viewer state cannot resurrect a dismissed or timed-out toast. Runtime owns
  `ZsToastSpec` and the closure-capable result/open callbacks, while platform
  profiles retain placement, metrics and timer scheduling.
  Document-ready Tooltip wraps exactly one child and carries non-empty text,
  platform-neutral placement and an optional nonnegative preview delay.
  Runtime attaches `ZsTooltipSpec` directly to the compiled child and preserves
  that child's stable `WidgetId`; the document wrapper does not add a hit
  target, backend child, registry entry or second event path. Platform profiles
  and hosts retain timing, metrics, typography and final overlay placement.
  Document-ready TeachingTip wraps exactly one page subtree and targets one of
  its stable descendant node IDs. Title/subtitle content, optional action and
  placement remain semantic; result emits `action`, `close` or `escape`, then
  a typed `open_change` emits `false` so controlled Viewer state cannot reopen
  a dismissed tip. Runtime revalidates bound target identity and owns only the
  `ZsTeachingTipSpec` plus node-local callbacks; platform profiles retain tail,
  metrics, typography, ordering and final placement.
  Document-ready ProgressRing uses one optional `nullable_number` value for
  determinate/indeterminate mode, validates its numeric range twice and maps
  small/medium/large to platform-owned metrics. Ordinary
  function-pointer handlers remain
  allocation-free; shared owned closures are
  allocated only through explicit `*_with` builders. `zsui-uic handoff`
  canonicalizes the validated document, binding schema, optional value snapshot
  and optional native PNG into a deterministic directory. Its stable manifest
  records content-change fingerprints, required features, node indexes and
  component contracts without timestamps, absolute paths or random IDs; the
  FNV fingerprint is not a cryptographic integrity hash. This does not add a
  reactive runtime, browser shell or global widget registry.
- Release embedding remains separately feature-pruned: `zsui-uic embed`
  validates and emits a deterministic versioned `.zsui` artifact containing
  only canonical document and binding data. `UiEmbeddedDocument::decode`
  rechecks the header, lengths, payload fingerprint, schema, application
  binding schema and compiled features. The `ui-document-runtime` feature
  depends only on `ui-document`; applications explicitly enable only the
  component features used by the artifact and compile it to typed
  `ViewNode<Msg>` through `ui_document_view`. It does not link `ui-viewer`,
  file polling, preview transport, native smoke code or another process.
  Full component coverage and advanced-control state retention remain
  unfinished. Fixed AppKit/Linux Viewer proof has passed; Windows Viewer has
  local real-host evidence but still needs a fixed Runner baseline.
- A browser/WASM projection is an optional approximate design tool, never
  native platform evidence. A full drag-and-drop designer is outside the v0.2
  completion gate. This added authoring goal does not remove any existing
  native service, IME, accessibility, interaction, trimming or proof gate.
- WebView is outside the ZSUI product boundary. Do not add WebView2, WKWebView,
  WebKitGTK, Wry, Tauri or another browser-shell dependency. Keep the isolated
  Tauri benchmark under `comparisons/` out of the root package graph. Enforce
  this boundary with `scripts/check-native-boundary.ps1` in CI.
- User-facing Windows release executables use the Windows GUI PE subsystem and
  must not open a console window. The final binary crate owns Rust's crate-level
  `windows_subsystem` attribute, so ZSUI examples declare it explicitly and CI
  enforces the source rule with `scripts/check-windows-gui-subsystem.ps1`.
  Debug builds and explicitly marked command-line/smoke drivers retain the
  console for diagnostics; release packaging must also verify PE subsystem 2.

## Architecture preferences

- Prefer composition, traits, typed messages, explicit state, strong IDs,
  typed `Dp`/`Px`/`Dpi`, RAII, `Result` and safe public APIs.
- Keep raw platform APIs and `unsafe` inside backend modules.
- Custom Canvas drawing retains backend-neutral primitives in local `Dp`
  coordinates and uses semantic color/text/icon roles. It must emit a balanced
  clip through the shared draw protocol and must not expose renderer or native
  handles to application code. Interaction returns through typed View messages.
  `ZsCanvasPointerEvent` reports press/move/release/cancel phases, primary,
  secondary, middle or auxiliary buttons, keyboard modifiers, local-DP
  positions and an explicit inside flag. Pointer capture keeps outside drag
  positions unbounded, cancellation follows capture/focus loss, and the
  existing primary `on_click` activation remains source-compatible.
- The public `crate::view` module is physically organized under `src/view/`:
  node, layout, event, focus, paint, overlay and widget-family source units
  share the existing module namespace so public paths and privacy stay stable.
- The public `crate::windows_win32_host` backend is physically organized under
  `src/platform/windows/`: application/window procedure, input, services, text,
  popup, timer and DPI source units share one backend namespace. Raw Win32
  handles and `unsafe` remain confined to that backend.
- Do not introduce a control inheritance hierarchy, string event bus, global
  mutable widget registry or an unrelated reactive runtime.
- Localization is an opt-in application-owned service. `ZsLocalizer` uses
  stable Fluent message ids, Unicode locale identifiers, parent/fallback
  lookup, named values, plural/select rules and direction-isolated formatting.
  Locale changes remain explicit application state and rebuild the View so
  intrinsic text metrics are recomputed; do not add a process-global mutable
  translation catalog or use source copy as message identity.
- Demos validate framework capability; they must not define the architecture.
- The optional document-shell boundary owns reusable `ZsTextDocument` file
  decoding, explicit dirty state and transactional UTF-8 save/save-as. Native
  file pickers and close-confirmation policy remain host/application concerns.
- Applications request native open/save panels through the safe
  `NativeFileDialogService` facade and owned `PathBuf` specs. Target selection
  stays inside ZSUI; missing backend features return `ZsuiError::Unsupported`.
  Dialogs bind to the active native window when available: Win32 sets
  `hwndOwner`, AppKit presents an `NSOpenPanel`/`NSSavePanel` sheet, and the
  default lightweight Linux host uses the XDG desktop portal. The optional
  `linux-gtk` compatibility backend retains `FileChooserNative`.
- Native message dialogs flow from one `NativeDialogSpec` through
  `NativeDesktopDialogService` or `NativeWindowHost` into the selected private
  desktop-runtime adapter. Win32 owns owner-bound `MessageBoxW`, AppKit prefers
  an active-window `NSAlert` sheet, GTK uses `GtkAlertDialog`, and linux-direct
  uses the desktop-provided Zenity surface while reporting `Unsupported` when
  that provider is absent. Applications receive only `DialogResponse` and do
  not choose platform action order or import native dialog types. Capability
  status remains partial until target interaction and non-Windows localization
  proof exist.
- Menu accelerators use the strong `ZsAccelerator` / `ZsAcceleratorKey`
  contract rather than application-parsed strings. `Primary` means Control on
  Windows and Linux and Command on macOS; Win32 `HACCEL`, AppKit key-equivalent
  and Linux accelerator details stay inside their native adapters.
  `src/platform/menu_accelerator.rs` owns AppKit/GTK string projection;
  `src/menu.rs` remains free of target `cfg` and toolkit encodings.
- Applications that need native window-menu actions in their typed update loop
  use `stateful_view_with_app_commands(...)`. Its `Command -> Option<Msg>`
  mapping stays platform-neutral. Win32 and AppKit connect their native menu
  objects. `linux-direct` renders an owned Linux desktop menu bar and popup in
  the application window and routes pointer, F10/arrow/Enter navigation and
  accelerators through the same command mapping. It does not claim a
  compositor-owned global menu.
- Applications register title-bar close policy with
  `on_close_requested(Command)`. Win32 `WM_CLOSE`, AppKit
  `windowShouldClose:` and the Linux native close event route that command
  through the same typed application update. An unmapped request keeps normal OS close
  behavior; a mapped request is vetoed unless the update calls `AppCx::quit()`.
  Test-only auto-close may bypass the policy, but application close buttons and
  menus must use the same command so dirty-document policy has one path.
- Application command executors run outside live-view locks. After a successful
  external effect, every desktop host refreshes the shared view, interaction
  plan and draw plan before repaint so modal dialogs and file I/O cannot leave
  stale application state on screen.
- The notepad acceptance application uses one platform-neutral source file,
  the shared self-drawn `text_editor`, typed document commands, native menus
  and target-dispatched file dialogs. `WindowsWin32OwnedTextEditor` remains an
  optional Windows service for native EDIT integration; it is not the demo
  architecture and is not evidence for AppKit or GTK editor completion.
- The calculator acceptance application also uses one platform-neutral
  `State/Msg/view/update` source and the normal `native_window(...).stateful_view(...)`
  entry. Its reusable `calculator_view` owns adaptive composition and stable
  action IDs; examples must not restore a target-specific event loop.
- Shared TextBox/TextEditor selection uses `ZsTextSelection` with Unicode-scalar
  anchor/caret indices and `on_text_selection_change(...)`. Edits, keyboard
  movement and pointer drag selection route through the same typed View update
  path on Win32, AppKit and Linux; backends do not own application cursor state.
  Scalar indices remain the public interchange format, but the shared input
  runtime normalizes endpoints to Unicode extended-grapheme boundaries. Left/
  Right, Backspace/Delete, pointer hits, wrapping and IME marked selections
  must not split combining sequences or joined emoji. Text geometry is shaped
  by Uniscribe on Win32, Core Text on AppKit and Pango on Linux; caret, selection,
  pointer hit testing, wrapping and IME candidate anchoring consume the same
  per-grapheme advances and primary/secondary bidirectional insertion positions.
  Left/Right traversal sorts each shaped row by the platform primary-caret x
  position, skips duplicate scalar stops at soft-wrap boundaries and keeps
  Shift selection on the same typed scalar-index path. Target macOS/Linux
  CJK/bidirectional interaction evidence remains separate work.
  Shaped rows use a bounded 256-entry cache owned by the per-window shaping
  backend; do not replace it with a global font/layout registry.
  Win32 assembles `WM_CHAR` UTF-16 surrogate pairs in per-window transient
  input state before dispatching one scalar to this shared model.
- Shared edit actions use `ZsTextEditCommand` queued through `AppCx` for the
  focused editor or an explicit strong `WidgetId`. The per-window input runtime
  may retain bounded undo snapshots as transient interaction state, but every
  resulting value and selection returns through typed View messages so the
  application remains authoritative. Cut/copy/paste require the optional
  `clipboard` feature and use target-native clipboard services; examples do not
  call platform clipboard APIs directly.
- Shared multiline editors default to `TextWrap::Word` and accept runtime
  `ViewNode::text_wrap(...)` configuration; single-line TextBox remains
  `NoWrap`. Rendering, caret placement, selection rectangles and pointer hit
  testing must consume the same wrap state on Win32, AppKit and Linux. Up/Down
  navigation follows these visual rows, while PageUp/PageDown moves by the
  current visible-row count and scrolls the transient viewport by the same page.
  Both preserve the desired shaped x position across shorter hard or soft lines;
  Shift extends the application-owned selection. Horizontal input, edits and
  pointer selection reset that transient column. This is shared self-drawn
  behavior and must not introduce a native child editor or a WebView.
- Multiline editor viewport position is transient per-window interaction state,
  not application document state. The same visual-row model must drive clipped
  text paint, selection/caret geometry, pointer hit testing, wheel scrolling and
  caret reveal after edits or keyboard movement. `TextWrap::NoWrap` also keeps a
  transient horizontal pixel offset shared by paint, selection/caret and
  pointer hit testing; caret movement reveals it horizontally, while wrapped
  modes reset the offset. During captured selection drags, each pointer update
  beyond a text edge advances that transient row or horizontal viewport by one
  visual step and hit-tests the newly visible edge instead of jumping to the
  document boundary. Editor viewport scrolling stays available with the
  `textbox` slice and must not pull in the general `scroll` container feature,
  a platform child editor or a WebView.

## Native platform bar

- ZSUI 0.3.0 is the Native Proof CI milestone. Its first blocking target is
  real AppKit execution on GitHub's fixed `macos-15` standard ARM64 runner;
  `macos-latest` is not a visual-baseline target. Proof must launch
  `NSApplication`/`NSWindow`/the ZSUI `NSView`, execute deterministic Gallery
  and Notepad scenarios, capture the final `NSView` through AppKit bitmap
  caching APIs, and emit versioned layout/focus/event/lifecycle JSON. Shared
  `DrawPlan` PNGs are not platform proof. CI baselines are read-only and change
  only through explicit reviewed commits. Win32 and Linux must adopt the same
  proof schema before the final 0.3.0 release; real Mac IME candidate-window
  and VoiceOver experience remain separate release-time manual gates.
- The first `macos-15` Native Proof workflow is operational: GitHub-hosted
  AppKit launches the real Gallery and Notepad windows, replays typed input,
  captures the final `NSView` bitmap and uploads the PNG plus versioned JSON.
  This is runtime evidence, not yet the complete baseline/diff gate or the full
  fixed-scene suite required for the final 0.3.0 release.
- Linux proof launches a real X11 or Wayland window, presents the Cairo/Pango
  frame through the native surface and captures the final presented frame.
  A shared `DrawPlan` image or cross-compilation is not Linux target evidence;
  the fixed Ubuntu proof job must upload the final surface PNG and matching
  runtime JSON.
- Native UI Proof run `29660600122` on commit `00951e5` passed AppKit,
  lightweight Linux and real Weston Wayland scenes. The Wayland artifact
  records `display_server=wayland`, final presented PNGs, AccessKit/AT-SPI
  gallery action evidence, Linux menu-surface geometry and command routing.
  CI run `29660600124` also passed the Ubuntu `linux-direct` target checks,
  macOS target checks, Windows tests, core tests and the locked feature matrix.
- Native UI Proof run `29762236980` on commit `64e553b` passed AppKit, X11 and
  real Weston Wayland Flyout interaction. Each target retained the initially
  open Flyout through first-window negotiation, invoked `FlyoutAction`, focused
  widget 407 and captured the final platform surface. The Linux host treats an
  initial unfocused notification and pre-presentation surface negotiation as
  startup state, while a later true-to-false focus transition or resize still
  performs transient-overlay dismissal. The remaining Flyout target gates are
  Escape/light-dismiss/resize matrices, nested overlay stacking and explicit
  platform accessibility announcement/AT-SPI action evidence.
- Canvas target proof uses `NativeViewSmokeInput::PointerDrag` rather than a
  backend-only test hook. Win32, AppKit, Linux Direct and optional GTK lower
  the same typed button/modifier sequence into the shared runtime; reports
  count Canvas events and completed drag sequences separately. The local
  Win32 Gallery catalog proof produced one primary activation plus five typed
  Canvas events from primary click and secondary-button drag, with balanced
  two-down/one-move/two-up input counts and no unhandled click. Native UI Proof
  run `29771247450` passed the same enforced catalog assertions on AppKit and
  Linux Direct for commit `a1d74a1`; Canvas no longer carries a target
  interaction-smoke gap.
- `menu-flyout` is an independent `widgets-base` feature rather than an alias
  for `flyout`. Applications provide one `MenuSpec` through the shared
  `menu_flyout` View builder and receive typed `Command` plus open-state
  messages; platform profiles own WinUI, AppKit and GTK menu density,
  placement, corner, accelerator and submenu geometry. The shared runtime
  owns pointer invocation, keyboard traversal, an eight-level bounded submenu
  path/stack, same-direction edge-aware cascading, modal focus restoration,
  Escape/light-dismiss/resize closure and report counters. Pointer traversal
  opens or collapses submenu branches only after the platform timing expires;
  Win32 reads `SPI_GETMENUSHOWDELAY`, while AppKit and GTK use their component
  profile timing. The Gallery feedback proof descends through two submenu
  branches, invokes the third-level command, reopens the menu and leaves the
  full three-surface stack visible for the final native capture.
  Native UI Proof run `29776254335` on commit `e75295e` passed the final
  MenuFlyout screenshot, command invocation, reopen, focus and role assertions
  on AppKit, lightweight X11 and real Weston Wayland/AT-SPI. Remaining 0.2
  gaps are complete cross-platform accessibility providers.
  Follow-up Native UI Proof run `29779123448` on commit `71a82bd` read the
  self-drawn menu through real Wayland AT-SPI and verified `Auto save` as a
  checked `check menu item` plus `More` as a `menu`. AccessKit 0.24 retains
  ZSUI's expanded metadata but its AT-SPI translator does not currently map it
  to `STATE_EXPANDED`; do not treat polling or a cached false state as platform
  evidence. Linux therefore exposes visible MenuFlyout rows as a real recursive
  parent/child accessibility tree with canonical path-based author IDs. Fixed
  Native UI Proof run `29790997519` verified `More -> Export -> PDF document`
  through Weston AT-SPI while the structured runtime proof independently
  required all four submenu state transitions. Windows exposes the same
  recursive surface as a real UIA Fragment tree with checked TogglePattern and
  nested ExpandCollapsePattern providers; the external UI Automation probe
  runs in the full Windows CI job. AppKit constructs real
  `NSAccessibilityElement` menu-item children from the same semantic snapshot
  and accepts its backend evidence only after reading back the recursive node
  count, checked value and expanded states. The AppKit job in fixed macOS 15
  Native UI Proof run `29788824442` passed with 8/8 native nodes, 1/1 checked
  value and 2/2 expanded submenu states. MenuFlyout paths canonicalize every
  truncated ancestor so nonzero sibling indices remain attached to the correct
  recursive provider; no ZSUI-owned catalog accessibility gap remains.
- AppKit status items are a real target service behind the shared `TraySpec`
  and native host API. The backend creates and retains `NSStatusItem` objects
  through `NSStatusBar`, attaches detached native `NSMenu` trees, treats loaded
  images as template icons, routes commands into the same typed application
  path and keeps tray applications alive after their last window closes.
  Built-in show/hide/toggle/quit commands also retain native host behavior.
  Required status-item smoke must report native creation, recursive command
  count, menu attachment/cleanup and routed command evidence. Fixed macOS 15
  Native UI Proof run `29793379808` passed those gates on commit `d835b7a`;
  cross-compilation alone is not status-item evidence, and the automated
  selector invocation does not replace a release-time manual menu-bar click.
- Linux memory comparison run `29669817180` measured the default X11 Notepad
  at 34.44 MiB median RSS, 21.24 MiB private RSS and 25.03 MiB PSS over five
  runs. Its smaps diagnosis attributed 4.60 MiB RSS to `librsvg` and 5.34 MiB
  to system font data. Exact freedesktop/GdkPixbuf icon-theme decoding is now
  the optional `linux-system-icons` capability; default `linux-direct` uses a
  complete Cairo symbolic vector set so lightweight applications do not load
  the SVG decoder. Ubuntu Sans plus CJK fallback remains intentional and must
  not be removed merely to lower RSS.
- Linux memory comparison run `29674253855` measured the optimized default X11
  Notepad at 25.39 MiB median RSS and peak RSS, 15.20 MiB private RSS and
  18.23 MiB PSS. Relative to run `29669817180`, removing the default SVG icon
  stack and rendering Cairo directly into the Softbuffer presentation buffer
  reduced median RSS by 9.05 MiB (26.3%), peak RSS by 11.08 MiB (30.4%),
  private RSS by 6.04 MiB (28.4%) and PSS by 6.80 MiB (27.2%). Native UI Proof
  run `29674253848` passed AppKit, X11 and real Weston Wayland/AT-SPI/menu
  scenes after the direct-buffer change.
- Linux framework memory comparisons must render a bilingual CJK workload and
  capture smaps categories for ZSUI, Slint and Iced; an English-only comparator
  understates its font residency. Run `29674669518` measured ZSUI at 25.31 MiB
  RSS, 15.18 MiB private RSS and 18.15 MiB PSS, Slint at 23.04/17.47/18.92 MiB
  and Iced at 16.73/11.51/12.87 MiB respectively. ZSUI's higher aggregate RSS
  versus Slint comes from shared Cairo/Pango/GLib and native Ubuntu/CJK font
  mappings; its private RSS and PSS were lower than Slint in that fixed run.
  Do not replace the native Cairo/Pango typography path or remove Ubuntu Sans
  and CJK fallback merely to optimize aggregate RSS.
- Linux memory comparison run `29677560838` on commit `d4b5de6` measured the
  optional pure-Rust `linux-direct-lite` X11 Notepad over five bilingual runs at
  15.77 MiB median RSS, 10.74 MiB private RSS and 12.09 MiB PSS, with a 5.14 MiB
  binary. The same run measured default ZSUI at 25.32/15.15/18.18 MiB, Slint at
  22.89/17.32/18.77 MiB and Iced at 16.71/11.51/12.90 MiB respectively. Lite
  reduced default ZSUI median RSS by 9.55 MiB (37.7%) while retaining Ubuntu
  Sans 11 plus CJK fallback. Native UI Proof run `29677560805` launched its
  real X11 window, captured the final cosmic-text/tiny-skia/Softbuffer surface,
  and reported no runtime errors or unhandled commands. The same proof run also
  passed AppKit, default X11 and default Weston Wayland/AT-SPI/menu jobs; it is
  not evidence that the lite renderer itself has passed Wayland or AT-SPI.
  CI run `29677560820` passed core, full Windows, AppKit/Linux target checks and
  the locked feature matrix for the same commit.
- Native proof JSON uses the framework-owned `NativeProofDocument` envelope.
  Acceptance applications supply scenario metadata and typed message names;
  the framework projects backend identity, runner metadata, logical/pixel
  geometry, scale, focus, widget roles, unhandled commands and runtime errors.
  Examples must not maintain separate per-platform proof schemas.
- Scripted native text proof stores only scalar/script traits for committed
  input, never the original payload. Navigation evidence records the backend,
  semantic key, handled state, scalar caret and typed selection after each
  shaped movement. The shared Notepad acceptance route must reject a backend
  unless four Right keys over `abאב` produce relative visual-order carets
  `1, 4, 3, 2`; this proves the target-injected shaper path without claiming a
  real IME candidate-window session. Visual insertion positions come from the
  directed edges of shaped grapheme clusters, not from a platform API's strong
  caret alone; AppKit resolves ambiguous Core Text edges with Unicode bidi
  levels before the shared navigation code consumes them. Backend proof reports
  must correspond exactly to scripted inputs; native menu acceptance is tracked
  separately and must never be prepended to that positional report stream.
  Native UI Proof run `29801544191` on commit `c2c775c` passed the enforced
  CJK/RTL script traits and relative `1, 4, 3, 2` caret/selection trace on real
  AppKit, X11 and Weston Wayland hosts. CI run `29801544136` and UI Memory
  Comparison run `29801544149` also passed for the same commit.
- Native resize evidence must call the real top-level window API, observe a
  platform resize callback and capture the final platform surface after shared
  relayout. Resizing only the shared View surface is not native proof. The
  common `NativeWindowSmokeRunOptions` contract exposes an optional resize
  target plus an explicit required flag, while `NativeWindowResizeEvidence`
  records the backend, requested size, observed initial/final sizes, native
  event count and exact-application result. Win32 proves `SetWindowPos` plus
  `WM_SIZE`; AppKit proves `setContentSize:` plus `windowDidResize:`;
  `linux-direct` proves Winit `request_inner_size` plus
  `WindowEvent::Resized`. The fixed Gallery scenarios end at `800x520` and
  `1180x640`, export post-resize widget bounds and reject messages/errors.
  Native UI Proof run `29809658203` on commit `ee49c40` passed the AppKit, X11
  and real Weston Wayland resize gates; local Win32 proof recorded one surface
  change and the exact `960x560 -> 1180x640` result. CI run `29809658198`
  passed the locked feature matrix and all desktop target checks for the same
  commit.
- Native typography is a backend-resolved `NativeTypographyProfile`, not a
  demo-owned type ramp. Native layout, paint and proof must share the resolved
  families, role metrics, accessibility scale and rasterization identity.
  Native proof also records process resident and peak resident memory from the
  target OS; executable size or Runner-wide memory is not runtime evidence.
- Ubuntu Native Proof must configure the Ubuntu 24.04 desktop font
  (`Ubuntu Sans 11`). A generic headless `Sans` fallback is not acceptable
  evidence of Ubuntu-native typography.
- The default Linux backend is `linux-direct`: a real Wayland/X11 window,
  direct software presentation, Cairo/Pango text and geometry, built-in Cairo
  symbolic icons, native IME events, system clipboard, XDG portal dialogs and
  an owned desktop menu surface. Exact freedesktop theme lookup and GdkPixbuf
  decoding are isolated behind `linux-system-icons`. With the optional
  `accessibility` feature, the
  shared hit-target tree is projected through AccessKit to AT-SPI with stable
  author IDs, roles, bounds, labels, focus and supported actions. Accessibility
  remains optional and does not increase the default lightweight build.
  Its controls are ZSUI self-drawn controls adapted to the Linux platform
  profile; they are not GTK widget instances. This is native-window/system
  integration, not a claim that the control tree is toolkit-native GTK.
  `linux-gtk` remains an explicit compatibility backend and is not pulled into
  the default application. Winit is not evidence of AppKit completion.
- `linux-direct-lite` is the opt-in pure-Rust renderer experiment over the same
  `linux-direct-host` lifecycle, Wayland/X11, IME, menu, portal and AccessKit
  paths. It uses cosmic-text/swash plus tiny-skia and binds directly to the
  Softbuffer frame. It must be built without `linux-direct`; if both Cargo
  features are enabled, the established Cairo/Pango renderer wins. Do not make
  lite the default or call it complete until target CI proves CJK, bidi, IME,
  accessibility and both display servers and records a repeatable RSS/PSS win.
- Built-in controls follow ZSUI's self-drawn rendering path and adapt their
  visual metrics and behavior to the target platform. On Windows, WinUI 3 and
  Fluent resources are the design reference; classic `comctl32` visuals must
  not be presented as modern Windows styling.
- Platform-native style does not imply embedding a second widget tree. Shared
  Rust code owns typed state, messages and layout, while the render backend
  maps platform style tokens into the existing buffered paint path.
- AppKit and Linux Cairo render semantic alpha with source-over paths;
  preblending `RoleWithAlpha` against the page surface is only an opaque
  renderer fallback and cannot represent modal composition. AppKit NSString
  drawing always uses line-fragment origins because shared text rectangles are
  top-left line boxes, not baseline origins.
- Popup controls use one shared, DPI-aware viewport placement result for both
  painting and hit testing. Window-edge flipping and horizontal clamping stay
  in the framework instead of being reimplemented by individual backends.
- Expanded popup state is dismissed through the shared typed View event path
  on outside pointer input, focus traversal and window focus loss. Backends
  must not keep a separate popup-open flag or bypass application messages.
- Preserve the buffered, background-erase-suppressed Windows paint path.
  Flicker is a release blocker for self-drawn Windows surfaces.
- Native desktop windows stay hidden until the initial draw plan, typed input
  route, appearance and icon are attached. Backends with native menu surfaces
  also attach them before showing the window. Win32, AppKit and Linux must
  not expose an empty host surface and repaint it as the first visible frame.
- Treat antialiasing, DPI, IME, scrolling, margins and window services as
  reusable framework capabilities rather than example-local fixes.
- Never promote a platform from declarations or cross-compilation alone.
  Completion requires target screenshots and interaction artifacts.

## UI and performance direction

- Built-in UI uses shared theme tokens, semantic icons and platform-native icon
  sources. Licensed vector assets are fallbacks, not private font code points.
- Reusable settings composition means navigation, grouped cards, setting rows,
  explanatory text and action regions—not a product-specific settings page.
- Follow modern Fluent/WinUI proportions on Windows while allowing AppKit and
  the Linux platform profile to present native platform character.
- Platform-native character includes composition, not only control sizes:
  Windows may use Fluent navigation and card groups, macOS uses a source-list
  sidebar with aligned AppKit-style form stacks, and GTK uses sidebar
  navigation with headings outside Adwaita-style boxed groups. Shared state and
  typed messages do not require all three platforms to reuse a WinUI page tree.
- A platform profile is not valid if it only changes colors, radii, spacing or
  font metrics while retaining the WinUI component tree. Navigation selection,
  row grouping, toolbar/header-bar placement, tab treatment, dialog action
  order and popup composition are platform contracts. Framework primitives such
  as `section`, `navigation_view(ZsNavigationViewSpec)` and
  `command_bar(ZsCommandBarSpec)` own these composition choices; demos consume
  those public semantic contracts and must not recreate platform branches as
  example-local architecture.
- Normal application-facing View construction never accepts a platform enum.
  Deterministic `*_for_style` constructors are crate-private proof hooks.
  Platform-native spacing, radius and control-density defaults resolve inside
  the framework; applications can override public semantic spec fields or use
  resolved spacing tokens with ordinary View modifiers without adding `cfg` or
  matching a platform.
- Fully unified desktop authoring is the highest-priority architecture target.
  One application source owns `State`, `Msg`, `view`, `update`, semantic specs,
  theme overrides and `native_window(...)` on Windows, macOS and Linux. Normal
  application View code must not select a host, renderer or platform and must
  not duplicate per-platform trees behind `cfg`.
- Unified authoring does not mean a unified backend or a shared Windows skin.
  Framework-owned `PlatformExperience` composition resolves navigation,
  toolbar/header placement, tabs, forms, dialogs and popups; a statically
  selected Host/Text/Raster/Presenter/Services profile owns target execution.
  Application parameters remain editable once through public semantic specs and
  tokens, and optional Cargo features remain independently trimmable.
- `src/platform/identity.rs` owns the canonical target/toolkit types and
  `NativeUiPlatform::current_target` is the compile-target identity selector.
  `src/platform/experience.rs` consumes it for framework experience
  defaults and owns the matching backend status and adapter identity; the
  public backend inventory and launch plan derive from that registration.
  `src/platform/style.rs` owns one low-level `ZsPlatformStyle` selected by that
  experience. `PlatformExperience::shared_component_style` is the sole
  platform-experience to component-profile mapper; component modules consume
  `ZsPlatformStyle` defaults and do not call `PlatformExperience` themselves.
  `src/platform/component_profile/` defines the framework component-profile
  contracts and sole style resolver and keeps Windows, macOS and GTK defaults
  in separate internal modules. Those profiles own semantic sections,
  adaptive navigation, foundational controls and navigation rows, command
  bars, tabs, content-dialog action order/sizing/alignment/scrim/focus
  traversal, feature-gated InfoBar/TeachingTip/Toast/BreadcrumbBar/
  ToggleButton/NumberBox/PasswordBox/ToolTip/ProgressRing/AutoSuggestBox/
  GridView/TreeView/DataGrid/TimePicker/ColorPicker/CommandPalette metrics and
  interaction treatments, global radius/spacing/control-density tokens,
  semantic typography defaults and shared focus visuals, and the legacy
  navigation/card shell. Feature-gated Document Shell and Calculator Shell
  direct-draw compatibility layouts also resolve through dedicated profiles.
  View, Shell, typography and shared keyboard/input routing consume the
  resolved profile instead of repeating platform matches.
  Target backends continue to own installed-font discovery, shaping,
  rasterization and native resources. Production shared component, token,
  typography and focus code contains no direct Windows, macOS or GTK variant
  branch.
  One `ZsShellLayoutSpec` therefore resolves to a Fluent pane/card composition,
  AppKit source-list/forms composition or GTK sidebar/boxed-list composition
  without exposing a platform selector in the application API.
  Built-in component-specific `Zs*PlatformStyle` names are compatibility
  aliases of the shared type, not separate selectors.
  `src/platform/backend_profile.rs` keeps Host, Text, Raster, Presenter and
  Services choices separate. The platform modules remain internal:
  ordinary View constructors and acceptance-application authoring must not take
  a platform style, platform enum or raw native handle.
- Production desktop event-loop, runtime-smoke, final-surface capture, scaffold
  and native-host `HostCapabilities`, active `DesktopCapabilities`, clipboard
  and native file-panel selection live behind the private
  `src/platform/desktop_runtime/` adapter contract. `native.rs`, `capability.rs`
  and the public desktop-service facades pass platform-neutral windows, draw
  plans, input runtimes, clipboard data and service specs into that boundary;
  target modules own Win32, AppKit, Linux-direct, GTK or Winit calls and
  resources. Target
  smoke results are normalized into the shared proof report inside that
  boundary. A new desktop backend adds one adapter implementation instead of
  adding target branches to the shared host, desktop services or application
  API. Explicit per-platform capability constructors are inspection contracts;
  the selected adapter alone chooses all active capability profiles. A new
  backend implements those profile methods alongside its event loop instead of
  adding another current-platform match to shared capability code. Capability
  details must name only their own platform implementation and remain
  unsupported or partial until the corresponding feature and target evidence
  exist.
- The shared native-proof document does not select an operating system or call
  target APIs. The selected `desktop_runtime` adapter supplies its proof
  backend identity, fallback typography and process-memory sampler; Win32
  counters, Mach task information and Linux procfs parsing stay in the backend
  namespace. A new target extends that adapter contract without adding `cfg`
  or native dependencies to `native_proof.rs`.
- A successful desktop final-surface capture always returns
  `NativeViewCaptureEvidence`. Win32 uses `WM_PRINTCLIENT` plus a GDI DIB and
  records client pixels, `GetDpiForWindow` scale, logical geometry, draw-plan
  typography scale and detected system typography; AppKit and Linux fill the
  same platform-neutral contract from their final surfaces. Runtime memory is
  sampled before native-window teardown, not reconstructed by a demo.
- A selected desktop backend owns a cloned `NativeViewInputRuntime` containing
  the static typed View or shared live View, semantic pointer/keyboard/text/IME
  state, resource policy, close command and command executors. Raw backend
  routes translate native events and execute host effects outside runtime
  locks; they must not duplicate focus, popup, selection, drag or edit state.
  `native.rs` does not construct or return a Win32/AppKit/Linux route type, and
  adding another backend does not add a target method or platform branch to the
  shared input runtime. Its surface snapshot accessor is part of the common
  live-View contract and must not be compiled only for one target backend.
- Public `ViewNodeKind` and `ZsButtonPresentation` payloads remain semantic and
  must not store a platform selector. Toolbar and adaptive-navigation layout,
  construction, paint and hit testing resolve the framework experience
  internally. A private `ViewNode` style override exists only for deterministic
  framework proof; the ordinary public builders leave it unset.
- Root View layout assigns collision-checked `WidgetId` values to interactive
  nodes that omit `.id(...)`, using a deterministic tree-path namespace so a
  same-shape stateful rebuild preserves focus and event routing without a
  global registry. Explicit IDs always win and remain required for cross-widget
  references or identity that must survive insertion and reordering.
- Shared text-input geometry depends on the platform-neutral `NativeTextShaper`
  contract and a bounded per-window cache. Win32 GDI/Uniscribe, AppKit/Core
  Text, Linux Direct Pango, Linux Direct Lite Cosmic Text and GTK Pango inject
  backend-owned shapers into the input runtime; their native contexts and
  platform variants must not return to `native_input_visuals.rs` or the public
  View API. Target execution constraints belong in
  `platform/text_shaper_boundary.rs`. A new platform supplies another shaper
  implementation without extending a shared platform enum.
- Typography, clipping or composition corrections found in Gallery, Notepad or
  another acceptance application must be implemented as reusable framework
  rules before the example consumes them. Measure, paint, hit testing,
  caret/selection geometry and accessibility must share one backend text-layout
  result; demo-local fixes do not advance the unified-platform goal.
- GTK/Adwaita boxed sections use padded rows with one-pixel separators, and
  the row's outer minimum height must include its interior padding. GTK sidebar
  selection stays neutral (not accent-filled), matching the
  `navigation-sidebar` style contract; accent remains available for actionable
  controls.
- Command bars use `command_bar(ZsCommandBarSpec)` and `toolbar_button`.
  Every declared bar action remains visible; overflow/menu projection is a
  separate capability and must not be simulated by silently dropping actions.
  Toolbar buttons carry semantic icons and typed messages through the shared
  Button event path while the framework owns platform metrics and chrome.
  Windows primary command icons use the WinUI 20-DIP metric, and a visible
  label to the icon's right uses the AppBarButton 12-DIP label role.
- The Notepad acceptance surface uses the real framework TabView for its
  document header. The semantic file icon and title occupy the same tab row and
  the editor is the selected tab content. This remains a one-static-tab proof;
  add, close, reorder and cross-window document-tab behavior are not implied.
- Acceptance examples follow the same rule: Notepad declares one five-action
  command bar on all targets. Save As, Status and About remain native-menu
  commands and are not passed to the bar. Platform differences come from the
  framework's toolbar metrics, icon source and rendering path, not example
  branches.
- Linux native proof must exercise the default `linux-direct` Winit input route
  before capture and provide Ubuntu Sans, a real CJK system fallback font,
  and the complete built-in symbolic vector set. Exact Adwaita theme and SVG
  loader proof belongs to the optional `linux-system-icons` matrix. Missing-
  glyph boxes, generic square icons, clipped bilingual labels or a screenshot
  produced without the scripted interaction are proof failures rather than
  acceptable headless runner differences. The Ubuntu 24.04 X11/Xvfb proof first passed on commit
  `cbe7b24`. Wayland proof must run with `DISPLAY` unset against a real Weston
  socket, record `display_server=wayland` from Winit's raw display handle, and
  use an external `pyatspi` client to enumerate and invoke the exported tree.
  An internal tree dump alone is not AT-SPI target evidence.
- Text labels carry semantic roles through the View and renderer boundary.
  Windows follows the Microsoft type ramp (12/16 caption, 14/20 body, 18/24
  body large, 20/28 subtitle, 28/36 content title, 40/52 title large and 68/92
  display), plus a compact framework `WindowTitle` role at 24/32. It uses
  regular 400 or semibold 600, resolves all UI text roles to the live
  `SPI_GETNONCLIENTMETRICS` message font with a Segoe UI failure fallback, and
  scales `HFONT` height from the active window DPI. This is a backend-owned
  framework rule shared by Gallery, Notepad, Viewer and normal applications;
  demos must not declare a replacement family. Do not restore raw per-widget
  title sizes or use ClearType color filtering for icon-font glyphs.
- Semantic text roles resolve through a framework-owned desktop typography
  profile before layout, shaping and paint. AppKit follows Apple's macOS text
  styles (10/13 caption, 13/16 body, 15/20 title 3, 17/22 title 2, 22/26
  title 1 and 26/32 large title), uses `NSFont` system faces and keeps Core
  Text shaping, NSString measurement, label intrinsic height, editor visual
  rows, selection and caret geometry on the same metrics. GTK selects the
  configured `GtkSettings:gtk-font-name` family and size and maps libadwaita's
  relative caption/body/title classes. The AppKit preferred body font and GTK
  configured UI font produce one deterministic runtime scale stored in the
  draw plan; semantic line/control heights, final text shaping, editor visual
  rows, selections and carets consume that same value. Examples must not patch
  font sizes or line boxes.
- GTK/Pango no-wrap text must not receive a finite layout width unless it is
  explicitly ellipsized. `single_paragraph_mode` alone does not disable width-
  driven wrapping, which can move the tail of one editor row onto the next row
  and overpaint its text. The renderer measures unconstrained no-wrap rows and
  applies horizontal alignment at the Cairo origin, then clips ink to the draw
  command bounds; wrapping and ellipsized text remain width-constrained.
- Stack and Grid intrinsic measurement is recursive. Descendant line boxes,
  control minimums, gaps and both sides of container padding are hard layout
  floors; an overconstrained parent may overflow or scroll but must not squash
  glyphs or overlap the following sibling. A row measures wrapped text height
  from the width actually allocated after fixed controls and gaps, not from the
  text's shortest unbreakable segment.
- `ViewStyle::flex` distributes only a Stack's main axis. Text fills the column
  cross-axis width even when wrapped text is content-height (`flex(0)`), while
  a wrapping label beside a fixed action uses explicit main-axis flex when it
  must receive the remaining row width.
- UiDocument page/content spacing uses `UiSpacingToken` rather than copied
  Windows constants. Text documents expose semantic role, wrap, ellipsis,
  weight and alignment; enum bindings are validated after value resolution.
- Windows Button defaults come from current WinUI resources and guidance:
  32 epx standard control height, 120 epx minimum width for short labels,
  `11,5,11,6` content padding, 4 epx control radius, centered content and a
  semantic control border. A Button is content-sized in a row/column instead
  of silently consuming an equal flex share; explicit width/height/flex still
  override the defaults.
- Platform minimum widths are floors, not fixed label widths. Button,
  ToggleButton, CheckBox and RadioButton constructors estimate an intrinsic
  label width from the active desktop profile so longer labels remain visible;
  Breadcrumb item widths include a renderer-measured guard so short and long
  segments do not ellipsize while the bar still has space. Applications can
  still request a larger explicit width. WinUI InfoBar keeps
  its 48 epx minimum but grows to fit title plus message content instead of
  clipping the second line.
- The complete component acceptance surface is the optional
  `component-gallery-demo` profile. It intentionally enables `all-widgets` and
  native smoke support, while the default build remains `window`, `button` and
  `label`. Its five pages and target screenshots validate the framework; they
  do not change component readiness or make contract-only families complete.
- A stateful View is built once before the first frame. Client-size or DPI-only
  changes relayout the existing tree; only state updates and explicit refresh
  rebuild it. Application storage reconciliation, initial page I/O and retry
  loops stay outside the first-frame path and return through typed completion
  messages or existing bounded background workers.
- Resident monitoring applications may opt into
  `NativeWindowResourcePolicy::ReleaseViewWhenHidden`. A hidden or minimized
  native window drops its stateful View tree, draw/hit plans, shaped-text cache
  and transient input state through ordinary Rust ownership, while application
  state, command routing and app-owned monitors remain alive. Showing the
  window rebuilds the View from retained state; the default remains retain-view
  for backward compatibility.
- High contrast is an accessibility appearance, not a dark-theme alias.
  System mode must override an application's light/dark preference when the OS
  requests high contrast. Win32 uses `SPI_GETHIGHCONTRAST` plus user-selected
  `GetSysColor` pairs; AppKit and Linux resolve their semantic appearance/theme
  colors. The deterministic shared palette is only a backend fallback.
- Win32 translucent semantic fills use GDI+ source-over composition inside the
  existing buffered paint path, so modal scrims, selections, hover fills and
  shadows preserve already-painted content. Preblending against the surface is
  only a fallback when the alpha-capable GDI+ operation is unavailable.
- Win32 must call `GdiFlush` before every transition from queued GDI text/icon
  output to a GDI+ graphics context. This preserves draw-plan order when a
  later antialiased selection, hover fill, arc or image follows text in the
  same buffered frame; otherwise final `WM_PRINTCLIENT` captures may omit
  already-issued glyphs even though the shared draw plan is complete.
- DatePicker resolves its today marker from the operating system's local time
  zone when the optional control is constructed, while exposing an explicit
  typed override for deterministic applications and tests.
- DatePicker hover and pressed visuals are transient runtime state keyed by
  typed hit targets, not application state or backend-local widget flags.
  Win32, AppKit and Linux route pointer motion, release/cancel and leave through
  the shared semantic-token draw decoration; target proof is still required on
  each non-Windows desktop.
- Direct RadioButton children of the same row or column form a local group
  without a global registry. The selected option, or otherwise the first
  option, is the group's single Tab stop. Ordinary arrow navigation follows
  WinUI logical order, does not wrap at group boundaries, and moves focus and
  selection together; Ctrl+arrow moves focus only. Selection still routes
  through the application's typed message so explicit state remains
  authoritative.
- ComboBox type-ahead consumes each backend's committed text input through one
  shared one-second, case-insensitive prefix buffer. Repeated single characters
  cycle from the current selection, and matches route through the existing
  typed selection message; this behavior must not invent backend-specific
  visual metrics.
- Long ComboBox popups follow WinUI's 15-item default maximum, reduce the
  visible row window further when the host viewport is smaller, and initially
  keep the selected option visible. Pointer-wheel scrolling moves that bounded
  internal window through typed `ComboBoxScrolled` events on Win32, AppKit and
  GTK4 without exposing backend state or requiring the general `scroll`
  feature.
- Tabs use `ZsTabId` rather than label text or positional indices as the public
  identity. Exactly one valid tab page is active; only that page participates
  in layout, paint, hit testing and event dispatch, while selection changes
  return through the application's typed `on_tab_select` message.
- Composite controls derive private child `WidgetId` values from both the
  parent widget and typed local ID in a reserved synthetic namespace. They must
  not reinterpret a local `ZsTabId` or similar typed child ID as an application
  `WidgetId`; application IDs, automatic tree-path IDs and synthetic child IDs
  occupy disjoint namespaces.
- Tabs remain self-drawn and use internal platform metric profiles rather than
  native child controls. Windows follows the WinUI interaction split: Left and
  Right move header focus without wrapping, Enter or Space selects, and
  Ctrl+Tab/Ctrl+Shift+Tab select cyclically. AppKit arrow keys select the
  adjacent page. GTK4 arrow keys plus Home/End move header focus, Space selects,
  and Ctrl+PageUp/Ctrl+PageDown changes page. Application code and the public
  API contain no platform `cfg`.
- Tab content composes the caller's explicit View padding with a platform-owned
  12-DP content inset; selected text and compact controls retain their native
  intrinsic line/control height at the content area's top-left instead of
  stretching through the page. Header measurement may overflow the clipped
  strip, but must never divide labels below the platform minimum width merely
  to fit a narrow viewport. A future overflow affordance may expose hidden
  headers without changing this no-compression contract.
- Windows TabView headers follow the WinUI row composition: an optional
  16-DIP semantic icon and the header label share one 32-DIP item row below an
  8-DIP strip inset. Header labels use Body text; Caption is reserved for
  secondary metadata. GTK TabView uses an AdwTabBar-style raised strip and a
  neutral inset rounded selected tab, never the Windows accent underline.
  Static tabs keep their identity and selection semantics but do not imply
  document-tab close, add, reorder or overflow behavior.
- Tabs target proof must exercise the public stateful View path rather than a
  backend-only hook. The Gallery navigation scene clicks Advanced and sends
  Right with the header focused. Local Win32, X11 and real Weston Wayland keep
  Advanced selected while focusing About; AppKit selects About. Reports require
  exact typed selection/keyboard counters, exactly one semantically focused tab
  whose ID matches the runtime focus ID, zero unhandled click/key input, the
  platform key backend and a final platform-surface PNG.
  Native UI Proof run `29812803034` on commit `06c249f` passed AppKit, X11 and
  Wayland. Remaining Tabs gates are accessibility providers, header-state
  polish and document-tab close/reorder/overflow behavior.
- A Gallery sidebar item is not a centered outlined Button. Navigation rows use
  semantic icons, left-aligned labels, a 36-DIP Windows row and a 3-by-16-DIP
  accent selection indicator. The expanded Windows Gallery pane follows the
  WinUI `OpenPaneLength` default of 320 DIPs. `navigation_view` can own the
  content pane through one platform-neutral `.content(WidgetId, ViewNode)`
  declaration: Windows Auto mode uses expanded/compact/minimal composition at
  1008/640 effective-pixel boundaries with a 48-DIP compact rail and 52-DIP
  minimal header; AppKit collapses its source-list sidebar when the declared
  content constraint cannot fit; GTK uses a 25% sidebar clamped to 180–280
  logical pixels and a `NavigationSplitView`-style breakpoint. The compact
  toggle, overlay scrim, focus order, paint order and dismissal stay inside the
  framework. Top navigation, accessibility semantics, and feeding AppKit's
  runtime standard sidebar min/max thickness back into shared layout remain
  explicit readiness gaps.
- TimePicker uses validated `ZsTime` and `ZsMinuteIncrement` values. It is an
  independent `time-picker` Cargo feature, stays on the shared self-drawn path,
  and selects Windows, macOS or GTK metric profiles internally. The picker
  popup shares viewport flipping and clamping with other overlays; its expanded
  state and changes remain explicit typed application messages. UiDocument
  represents application state with canonical `HH:MM` values and retains the
  platform display clock separately. Windows Viewer proof selects `10:45`
  through two real pointer clicks and captures the final Win32 surface.
  System-locale clock selection and non-Windows target proof remain readiness
  gaps.
- Grid is an independent `grid` Cargo feature. Its public contract uses typed
  fixed/fractional tracks and nonzero spans; the shared layout pass owns row/
  column gaps, explicit cell/span geometry, DPI conversion, paint bounds and
  hit-test bounds for all three desktop renderers. Every canonical child is a
  typed `ZsGridCell`; overlapping cells retain declaration order. Fixed tracks
  and one-track child minimums are hard constraints: an over-constrained Grid
  overflows its viewport instead of scaling platform control metrics or text
  line boxes. Compact children retain their intrinsic size at the cell's
  top-left while unconstrained children stretch. General content-sized tracks,
  baseline alignment and non-Windows target proof remain readiness gaps.
- NumberBox is an independent `number-box` Cargo feature and stays on the shared
  self-drawn path. `ZsNumberRange` validates finite bounds and small/large steps;
  the editable draft is kept separate from committed `Option<f64>` application
  state so partial input is not reformatted mid-edit. Enter, focus loss, pointer
  steppers and Up/Down/PageUp/PageDown route typed events. Windows, macOS and GTK
  select internal metric profiles modeled on NumberBox/NSStepper/SpinButton;
  locale formatting, expression input, accessibility and non-Windows target
  proof remain explicit readiness gaps.
- PasswordBox is an independent `password-box` Cargo feature and stays on the
  shared self-drawn tree. `ZsPassword` zeroizes its owned allocation on drop,
  always redacts `Debug`, and deliberately does not implement serialization;
  password events and secure draw commands omit their value when serialized.
  Hidden plans, IME surrounding/preedit reports and text geometry contain only
  one mask glyph per Unicode scalar. Windows defaults to press-and-hold Peek;
  macOS and GTK default to Hidden, and all platforms select internal metric and
  semantic-icon profiles without native child controls. Renderer APIs are the
  explicit clear-text boundary. OS text stacks may still make transient copies;
  locked memory, Alt+F8, full accessibility/caps-lock signaling and non-Windows
  target smoke remain readiness gaps.
- ToolTip is an independent `tooltip` Cargo feature and is an attached modifier
  on a stable-ID `ViewNode`, not a native child widget or a duplicate hit target.
  It stays self-drawn and noninteractive, with internal Windows, macOS and GTK
  metric profiles. Pointer hover opens after the host delay, keyboard focus
  opens immediately, and leave/click/key/blur or the display timeout dismisses
  it. Win32 reads the system hover/message timing; AppKit and GTK use owned
  one-shot main-loop timers. Auto placement prefers centered above the pointer,
  flips and clamps inside the current viewport. A top-level overflow popup,
  accessibility relationship and non-Windows target proof remain readiness gaps.
- ContentDialog is an independent `dialog` Cargo feature over `widgets-base`.
  It composes around one application page, keeps `open` in application state and
  emits `ZsContentDialogResult` from semantic Primary, Secondary or mandatory
  Close slots. Document-backed bindings pair `open` with a Boolean
  `open_change` action so a response is retained as `false` across Viewer
  rebuilds; application code never chooses platform button order or imports a
  native dialog object. The modal surface, scrim, focus scope, hit testing and
  hover/pressed feedback stay on the shared self-drawn tree, preserving the
  buffered Win32 path and avoiding child HWND/NSView/GtkWidget registries.
  Windows uses WinUI-like equal action widths, macOS puts intrinsic actions at
  the trailing edge with the default last, and GTK uses trailing AlertDialog-like
  actions. Escape activates Close; Tab and arrows cycle semantic actions; Enter
  and Space activate the focused action. Opening a dialog immediately moves the
  native input route into its modal focus scope, suppresses underlying text/IME
  visuals and restores the prior valid focus target after close. Accessibility
  dialog semantics, arbitrary ViewNode content, validation/deferrals and
  AppKit/GTK target interaction smoke remain readiness gaps.
- Flyout is an independent `flyout` Cargo feature over `widgets-base` and wraps
  one ordinary page plus one arbitrary application View subtree. Applications
  own `open`, the stable presenter and target IDs, and the content state; the
  framework owns target lookup, viewport flipping/clamping, the modal focus
  scope and typed `LightDismiss`/`EscapeKey` results. Pointer input inside the
  surface stays with the content; the first outside click is absorbed and
  closes the Flyout. Window focus loss and surface resize also dismiss it.
  Windows uses a Fluent flyout without a tail, macOS uses an AppKit-popover
  profile and Linux uses a GTK-popover profile. Application code does not branch
  on platforms or import native handles. AppKit, Linux X11 and real Weston
  Wayland final-surface proofs invoke the same typed action and retain focus in
  the overlay. Accessibility announcements and Flyout-specific AT-SPI actions,
  nested overlay validation, and target Escape/light-dismiss/resize matrices
  remain readiness gaps.
- Toast is an independent `toast` Cargo feature over `widgets-base`. It is a
  nonmodal in-window feedback layer, not an imitation of Windows or macOS
  system-notification chrome. Applications own an optional `ZsToastSpec` with a
  stable `ZsToastId`; the framework owns bottom-centered placement, one optional
  action, the mandatory close affordance, keyboard routing and the active
  timeout, then emits `ZsToastResult`. Windows follows the non-targeted WinUI
  TeachingTip placement/surface model, macOS uses restrained foreground
  feedback, and GTK uses AdwToast-like one-action/close geometry through the
  same self-drawn renderer protocol. Accessibility live-region announcement,
  hover/focus timeout pause, queues/priority replacement and AppKit/GTK target
  interaction smoke remain readiness gaps.
- InfoBar is an independent `info-bar` Cargo feature over `widgets-base`. It is
  an ordinary inline status surface, never an overlay and never a timer-owned
  notification. Applications own presence/removal through the View tree and
  receive typed `ZsInfoBarEvent::Action` or `Close`; `ZsInfoBarSpec` keeps the
  informational/success/warning/error severity, title, message, one optional
  action and default-on close affordance explicit. Windows uses Fluent InfoBar
  geometry with a severity edge, macOS uses restrained system status colors
  without imitating modal `NSAlert`, and GTK uses AdwBanner-like compact
  geometry. Severity must remain represented by text and semantic icon, not
  color alone. Accessibility live-region announcement, close deferrals,
  arbitrary View content, bidirectional layout and AppKit/GTK target smoke are
  readiness gaps.
- TeachingTip is an independent `teaching-tip` Cargo feature over
  `widgets-base`. It is a targeted, nonmodal in-window overlay: applications
  own `open`, a stable presenter ID and a stable target `WidgetId`, while the
  framework owns viewport-aware auto placement, tail geometry, one optional
  action, the mandatory close affordance and typed action/dismiss results. The
  target remains in ordinary layout and the page remains interactive. Windows
  uses Fluent TeachingTip metrics, macOS uses restrained NSPopover-like metrics
  and GTK uses GtkPopover-like metrics through one self-drawn protocol; the
  triangle tail is a shared draw command consumed by all three renderers.
  Light-dismiss, close cancellation/deferrals, arbitrary View/hero/icon
  content, complete RTL/placement coverage, accessibility focus handoff and
  AppKit/GTK target interaction smoke remain readiness gaps.
- BreadcrumbBar is an independent `breadcrumb` Cargo feature over
  `widgets-base`; it must not pull in Tabs, TreeView or a native child control.
  Applications own the root-to-current `ZsBreadcrumbItem` sequence, stable
  `ZsBreadcrumbId` values and explicit overflow-open state. The framework owns
  width-aware collapse, transient semantic focus, one Tab stop, internal
  arrow/Home/End navigation, overflow Up/Down navigation and typed expanded/
  selection messages. Windows and GTK collapse leftmost ancestors behind a
  leading ellipsis; macOS preserves the root before the ellipsis when space
  permits. Windows follows Fluent BreadcrumbBar, macOS uses compact Path
  Control-like metrics, and GTK uses a ZSUI self-drawn profile informed by
  GNOME navigation/Adwaita because GTK has no public breadcrumb widget.
  Accessibility relationships, editable/file paths, semantic item icons,
  drag-and-drop, complete RTL and AppKit/GTK target interaction smoke remain
  readiness gaps.
  UiDocument exposes this same component as `breadcrumb`: a typed
  `breadcrumb_item_array` carries unique stable semantic IDs and non-empty
  labels, `breadcrumb_item_id` selection returns the author ID, and Boolean
  expanded/open changes retain state through Viewer rebuilds. The release
  runtime derives private `ZsBreadcrumbId` values from the owning document
  node plus the semantic item ID; declaration order is never identity.
- ProgressRing is an independent `progress-ring` Cargo feature; it must not pull
  in ProgressBar. `ZsProgressRingSpec` keeps active, determinate/indeterminate
  mode and semantic size explicit, while inactive rings reserve layout space and
  remain absent from paint and hit testing. One shared `StrokeArc` command feeds
  antialiased GDI+, NSBezierPath and Cairo drawing. Active indeterminate rings
  use the framework background interval through Win32 timers, owned `NSTimer`
  and cancellable GLib sources rather than application messages or backend
  widget state. Windows follows the documented 20-DP minimum and accent ring;
  macOS/GTK select internal spinner metrics. Reduced-motion handling,
  accessibility and non-Windows target animation proof remain readiness gaps.
- AutoSuggestBox is an independent `auto-suggest` feature over `widgets-input`;
  it must not pull in TextBox or ComboBox. Applications own suggestion data and
  stable `ZsAutoSuggestionId` values. The view keeps query, highlighted ID and
  expanded state explicit and emits typed user-input, suggestion-chosen and
  query-submitted messages. Windows uses a WinUI-like trailing query/clear
  column, macOS follows NSSearchField leading-search/trailing-cancel geometry,
  and GTK follows SearchEntry geometry; all remain self-drawn through the shared
  renderer protocol. Long-list wheel paging, accessibility providers and
  AppKit/GTK target interaction smoke remain readiness gaps.
- TreeView is an independent `tree` feature over `widgets-list`; it must not
  pull in ListView, ScrollView or a platform child control. Applications own
  immutable node trees and globally unique `ZsTreeNodeId` values, plus explicit
  expanded and selected state. Expansion, selection and invocation are separate
  typed messages. Visible rows are derived deterministically without a global
  mutable registry; an unrealized-child marker permits application-driven lazy
  loading. Windows uses WinUI-like rows and disclosures, macOS uses compact
  disclosure-triangle metrics and accent selection, and GTK uses TreeExpander-
  style indentation and disclosures through the same draw protocol. Selection
  is preserved when its row is collapsed. Accessibility tree metadata,
  multi-selection/drag-and-drop, large-tree virtualization and AppKit/GTK target
  interaction smoke remain readiness gaps.
- GridView is an independent `grid-view` feature over `widgets-list`; it must
  not pull in ListView, ScrollView, TreeView or a platform child control.
  Applications own immutable `ZsGridViewItem` values, globally unique
  `ZsGridViewItemId` values and explicit single-selection state. Selection and
  invocation are separate typed messages. The framework derives responsive
  equal-width columns from the final bounds, keeps item paint and hit geometry
  in one plan, and exposes one Tab stop with Left/Right/Up/Down, Home/End,
  Space and Enter routing. Windows follows Fluent GridView left-to-right row
  filling, macOS uses compact NSCollectionView-like metrics, and GTK uses
  GtkGridView-like metrics through the same self-drawn protocol; backends must
  not keep a collection model. The first pass is a bounded gallery surface.
  Owned scrolling/virtualization, multi-selection, rubber-band selection,
  drag-and-drop, sections, arbitrary item templates, accessibility grid
  providers and AppKit/GTK target smoke remain readiness gaps.
- ColorPicker is an independent `color-picker` feature over `widgets-input`;
  it must not pull in TextBox, ComboBox or a platform child picker. Applications
  own `ZsColorPickerState`, including the selected RGBA `Color`, expanded flag,
  active channel and alpha policy. The shared renderer and hit plan own the HSV
  spectrum, hue track, preview, RGBA sliders and viewport-aware overlay, while
  pointer and keyboard routes emit typed value/channel/expanded messages.
  Windows follows the documented Fluent ColorPicker structure and keeps a
  square spectrum at least 256 DIPs high while editable precision fields are
  absent. macOS deliberately uses an NSColorWell/custom-panel-like compact
  slider mode, and GTK uses a ColorDialogButton-like entry and HSV editor; the
  three skins remain separate self-drawn metric profiles. Editable RGB/HSV/hex
  fields, swatches, eyedropper, accessibility, HDR/color-space management and
  AppKit/GTK target interaction smoke remain readiness gaps. Do not describe
  this bounded first pass as control-for-control WinUI 3 parity.
- CommandPalette is an independent `command-palette` feature over
  `widgets-input`; it must not pull in AutoSuggestBox, ListView or a native
  child search control. Applications own immutable `ZsCommandPaletteItem`
  metadata, globally unique `ZsCommandPaletteItemId` values, the query,
  highlighted command and open state. ZSUI performs stable, case-insensitive
  all-term substring filtering, draws at most eight visible rows and emits
  typed query/highlight/invoke/open messages; it never executes a command or
  owns a global shortcut. Disabled commands remain visible but are skipped by
  keyboard navigation. Windows follows a Fluent/PowerToys launcher-like
  profile, macOS a Spotlight/NSSearchField-like profile and GTK a
  SearchEntry/list-popover-like profile; all three are separate self-drawn
  metrics over the same paint/hit plan. Up/Down/Home/End move the highlight,
  Enter invokes and closes, Escape or scrim click dismisses, and Tab remains
  trapped in the modal search scope. Fuzzy/pinyin ranking, recent-command
  persistence, result virtualization, search-dialog/result-list accessibility
  semantics and AppKit/GTK target interaction smoke remain readiness gaps.
- DataGrid is an independent `table` feature over `widgets-list`; it must not
  pull in ListView, ScrollView or a platform child control. Applications own
  immutable `ZsTableColumn` and
  `ZsTableRow` values with globally unique strong IDs, explicit selected-row and
  sort state, and the actual data ordering. The framework emits separate typed
  row-selection, row-invocation and column-sort messages; it never sorts a
  product data source or keeps a backend table model. Fixed `Dp` and weighted
  fill columns share one layout/hit plan. Windows uses Fluent-like header and
  selection geometry, macOS uses compact NSTableView-like metrics and accent
  selection, and GTK uses ColumnView-like headers and separators through the
  self-drawn renderer protocol. The first pass is read-only and row-selecting;
  cell focus/editing/validation, multi-selection, column resize/reorder,
  accessibility tree-grid providers, large-table virtualization and AppKit/GTK
  target smoke remain readiness gaps.
- Every new component remains opt-in through its own Cargo feature. Default
  features stay `window`, `button` and `label`; `all-widgets`/`full` are explicit
  profile choices and must not become implicit dependencies of component APIs.
  The internal `text-input-core` feature owns Unicode segmentation for text-
  capable controls. Non-text `widgets-input` slices such as CheckBox must not
  pull that dependency into their build.
- Native text accessibility is an independent `accessibility` feature. It may
  activate only target-specific native bindings: Win32 UI Automation exposes a
  focused Edit/Value/Text provider through `WM_GETOBJECT`; TextPattern owns
  document/selection/visible ranges, grapheme-safe unit movement, range search, point hit
  testing, native shaped bounding rectangles, typed selection routing and
  top/bottom aligned ScrollIntoView through the self-drawn text viewport.
  AppKit exposes focused text-field/text-area selectors on the custom `NSView`,
  and GTK4 exposes a hidden-until-focused TextBox/value semantic surface.
  Password snapshots stay masked; Win32 advertises neither ValuePattern nor
  TextPattern for protected text, and AppKit marks protected content with the
  secure-text subrole. This feature must not introduce a native child editor,
  WebView or global widget registry. UIA rich attributes/embedded-object ranges
  plus real AppKit/GTK assistive-technology target proof remain readiness work.
- ToggleButton is an independent `toggle-button` Cargo feature and reuses the
  shared explicit Boolean `Toggled` message path rather than inheriting Button
  behavior or storing state in a backend. It remains self-drawn: Windows,
  macOS and GTK select internal metric profiles, while hover/pressed state is
  transient runtime decoration and checked state has both fill and a shape cue.
  Indeterminate mode, accessibility and non-Windows target proof remain gaps.
- Large collections use virtualization, pagination, background prefetch and a
  small bounded cache; product storage remains outside the framework.
- Image previews use application-owned `ZsImagePreviewState`: decoding is
  coalesced on one owned worker, stale generations are rejected and the last
  complete immutable `Arc` frame stays visible until an atomic replacement is
  ready. Win32 raster presentation remains inside the buffered paint path.
- Paged collections receiving external synchronization revisions reconcile by
  stable item key and sub-row pixel offset. Index-based pages are invalidated,
  stale generations are rejected, and queued page work is rechecked against the
  newest viewport before product I/O begins.

## Verification and delivery

- A control counts only after layout, state, events, themed paint and tests are
  connected. Platform completion additionally needs target evidence.
- For each vertical slice, run focused checks, the required full gates, real
  smoke where available, update truthful documentation, then commit and push.
- Keep AI context economical: bootstrap from `docs/ai-agent.md`, select one
  context pack, use `rg`, and read optional material only for a concrete gap.
- Use the current repository and generated evidence for progress numbers; do
  not copy stale counts or completion claims into this memory.
- Performance claims use four independent release workloads: Minimal, Common,
  Full Native App and Viewer. Compare frameworks only within one fixed visual
  contract, window size, data and animation state. Measure recursive process
  trees, reject PID-reuse contamination, and keep formal application rows
  separate from Viewer rows.
- `UiDocument` remains a bounded semantic declaration format. The release
  runtime is feature-pruned; file watching, diagnostics, screenshots and full
  document-component coverage belong to the separate Viewer artifact. Do not
  evolve it into a dynamic Web-like platform or let a preview tool define
  reusable framework components.

## Acceptance applications

- The notepad and calculator are acceptance applications, not product goals.
- The final notepad comparison should evaluate application code volume, memory,
  native visual quality and cross-platform consistency against egui and other
  relevant stacks, including Tauri 2.
- Demo source may be versioned; generated binaries, dependency build output and
  measurement scratch data should not clutter the repository.
