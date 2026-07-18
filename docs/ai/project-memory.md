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

- Preserve one shared Rust application shape across Win32, AppKit and GTK4:
  `native_window(...).stateful_view(...).run()`.
- Application code must not expose platform `cfg`, raw handles, Objective-C or
  GTK objects, drawing handles, or native event loops.
- A concise native-window entry is important, but it is only the bootstrap
  contract. The real objective is a complete native application loop.
- Controls and advanced capabilities should remain Cargo-feature selectable so
  unused surfaces and heavy dependencies can be omitted.
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
  `hwndOwner`, AppKit presents an `NSOpenPanel`/`NSSavePanel` sheet, and GTK4
  sets `transient-for`; targets fall back to application-modal presentation
  only when no active owner exists.
- Menu accelerators use the strong `ZsAccelerator` / `ZsAcceleratorKey`
  contract rather than application-parsed strings. `Primary` means Control on
  Windows and Linux and Command on macOS; Win32 `HACCEL`, AppKit key-equivalent
  and GTK action-accelerator details stay inside their native adapters.
- Applications that need native window-menu actions in their typed update loop
  use `stateful_view_with_app_commands(...)`. Its `Command -> Option<Msg>`
  mapping stays platform-neutral; Win32, AppKit and GTK4 dispatch through the
  owned live-view host, rebuild the shared draw plan and request native repaint
  without exposing a raw menu id, handle or event loop.
- Applications register title-bar close policy with
  `on_close_requested(Command)`. Win32 `WM_CLOSE`, AppKit
  `windowShouldClose:` and GTK4 `close-request` route that command through the
  same typed application update. An unmapped request keeps normal OS close
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
- Shared TextBox/TextEditor selection uses `ZsTextSelection` with Unicode-scalar
  anchor/caret indices and `on_text_selection_change(...)`. Edits, keyboard
  movement and pointer drag selection route through the same typed View update
  path on Win32, AppKit and GTK4; backends do not own application cursor state.
  Scalar indices remain the public interchange format, but the shared input
  runtime normalizes endpoints to Unicode extended-grapheme boundaries. Left/
  Right, Backspace/Delete, pointer hits, wrapping and IME marked selections
  must not split combining sequences or joined emoji. Text geometry is shaped
  by Uniscribe on Win32, Core Text on AppKit and Pango on GTK4; caret, selection,
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
  testing must consume the same wrap state on Win32, AppKit and GTK4. Up/Down
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
  only through explicit reviewed commits. Win32 and GTK4 must adopt the same
  proof schema before the final 0.3.0 release; real Mac IME candidate-window
  and VoiceOver experience remain separate release-time manual gates.
- The first `macos-15` Native Proof workflow is operational: GitHub-hosted
  AppKit launches the real Gallery and Notepad windows, replays typed input,
  captures the final `NSView` bitmap and uploads the PNG plus versioned JSON.
  This is runtime evidence, not yet the complete baseline/diff gate or the full
  fixed-scene suite required for the final 0.3.0 release.
- GTK4 proof captures the realized ZSUI `DrawingArea` through
  `GtkWidgetPaintable`, a GTK snapshot and the native GSK renderer texture.
  A shared `DrawPlan` image or cross-compilation is not Linux target evidence;
  the fixed Ubuntu/X11 proof job must upload the final texture PNG and matching
  runtime JSON.
- Native proof JSON uses the framework-owned `NativeProofDocument` envelope.
  Acceptance applications supply scenario metadata and typed message names;
  the framework projects backend identity, runner metadata, logical/pixel
  geometry, scale, focus, widget roles, unhandled commands and runtime errors.
  Examples must not maintain separate per-platform proof schemas.
- Native typography is a backend-resolved `NativeTypographyProfile`, not a
  demo-owned type ramp. Native layout, paint and proof must share the resolved
  families, role metrics, accessibility scale and rasterization identity.
  Native proof also records process resident and peak resident memory from the
  target OS; executable size or Runner-wide memory is not runtime evidence.
- Desktop backends are real Win32, AppKit and GTK4 paths. Winit may remain an
  explicit fallback but is not evidence of AppKit or GTK4 completion.
- Built-in controls follow ZSUI's self-drawn rendering path and adapt their
  visual metrics and behavior to the target platform. On Windows, WinUI 3 and
  Fluent resources are the design reference; classic `comctl32` visuals must
  not be presented as modern Windows styling.
- Platform-native style does not imply embedding a second widget tree. Shared
  Rust code owns typed state, messages and layout, while the render backend
  maps platform style tokens into the existing buffered paint path.
- AppKit and GTK render semantic alpha with their native source-over paths;
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
  route, appearance, icon and menu are attached. Win32, AppKit and GTK4 must
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
  GTK4 to present native platform character.
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
- Linux native proof must exercise the GTK runtime input route before capture
  and provide a real CJK system fallback font. Missing-glyph boxes, clipped
  bilingual labels or a screenshot produced without the scripted interaction
  are proof failures rather than acceptable headless-runner differences.
- Text labels carry semantic roles through the View and renderer boundary.
  Windows follows the Microsoft type ramp (12/16 caption, 14/20 body, 18/24
  body large, 20/28 subtitle, 28/36 title, 40/52 title large and 68/92
  display), uses regular 400 or semibold 600, selects Segoe UI Variable
  Small/Text/Display with a Segoe UI fallback, and scales `HFONT` height from
  the active window DPI. Do not restore raw per-widget title sizes or use
  ClearType color filtering for icon-font glyphs.
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
  `GetSysColor` pairs; AppKit and GTK4 resolve their semantic appearance/theme
  colors. The deterministic shared palette is only a backend fallback.
- Win32 translucent semantic fills use GDI+ source-over composition inside the
  existing buffered paint path, so modal scrims, selections, hover fills and
  shadows preserve already-painted content. Preblending against the surface is
  only a fallback when the alpha-capable GDI+ operation is unavailable.
- DatePicker resolves its today marker from the operating system's local time
  zone when the optional control is constructed, while exposing an explicit
  typed override for deterministic applications and tests.
- DatePicker hover and pressed visuals are transient runtime state keyed by
  typed hit targets, not application state or backend-local widget flags.
  Win32, AppKit and GTK4 route pointer motion, release/cancel and leave through
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
- Tabs remain self-drawn and use internal platform metric profiles rather than
  native child controls. Windows follows the WinUI interaction split: Left and
  Right move header focus without wrapping, Enter or Space selects, and
  Ctrl+Tab/Ctrl+Shift+Tab select cyclically. AppKit arrow keys select the
  adjacent page. GTK4 arrow keys plus Home/End move header focus, Space selects,
  and Ctrl+PageUp/Ctrl+PageDown changes page. Application code and the public
  API contain no platform `cfg`.
- Windows TabView headers follow the WinUI row composition: an optional
  16-DIP semantic icon and the header label share one 32-DIP item row below an
  8-DIP strip inset. Header labels use Body text; Caption is reserved for
  secondary metadata. GTK TabView uses an AdwTabBar-style raised strip and a
  neutral inset rounded selected tab, never the Windows accent underline.
  Static tabs keep their identity and selection semantics but do not imply
  document-tab close, add, reorder or overflow behavior.
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
  state and changes remain explicit typed application messages. System-locale
  clock selection and non-Windows target proof remain readiness gaps.
- Grid is an independent `grid` Cargo feature. Its public contract uses typed
  fixed/fractional tracks and nonzero spans; the shared layout pass owns row/
  column gaps, explicit cell/span geometry, DPI conversion, paint bounds and
  hit-test bounds for all three desktop renderers. Every canonical child is a
  typed `ZsGridCell`; overlapping cells retain declaration order. Content-sized
  tracks, baseline alignment and non-Windows target proof remain readiness gaps.
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
  Close slots; application code never chooses platform button order or imports
  a native dialog object. The modal surface, scrim, focus scope, hit testing and
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

## Acceptance applications

- The notepad and calculator are acceptance applications, not product goals.
- The final notepad comparison should evaluate application code volume, memory,
  native visual quality and cross-platform consistency against egui and other
  relevant stacks, including Tauri 2.
- Demo source may be versioned; generated binaries, dependency build output and
  measurement scratch data should not clutter the repository.
