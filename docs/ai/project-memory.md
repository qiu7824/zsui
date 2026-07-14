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

## Architecture preferences

- Prefer composition, traits, typed messages, explicit state, strong IDs,
  typed `Dp`/`Px`/`Dpi`, RAII, `Result` and safe public APIs.
- Keep raw platform APIs and `unsafe` inside backend modules.
- Do not introduce a control inheritance hierarchy, string event bus, global
  mutable widget registry or an unrelated reactive runtime.
- Demos validate framework capability; they must not define the architecture.

## Native platform bar

- Desktop backends are real Win32, AppKit and GTK4 paths. Winit may remain an
  explicit fallback but is not evidence of AppKit or GTK4 completion.
- Built-in controls follow ZSUI's self-drawn rendering path and adapt their
  visual metrics and behavior to the target platform. On Windows, WinUI 3 and
  Fluent resources are the design reference; classic `comctl32` visuals must
  not be presented as modern Windows styling.
- Platform-native style does not imply embedding a second widget tree. Shared
  Rust code owns typed state, messages and layout, while the render backend
  maps platform style tokens into the existing buffered paint path.
- Popup controls use one shared, DPI-aware viewport placement result for both
  painting and hit testing. Window-edge flipping and horizontal clamping stay
  in the framework instead of being reimplemented by individual backends.
- Expanded popup state is dismissed through the shared typed View event path
  on outside pointer input, focus traversal and window focus loss. Backends
  must not keep a separate popup-open flag or bypass application messages.
- Preserve the buffered, background-erase-suppressed Windows paint path.
  Flicker is a release blocker for self-drawn Windows surfaces.
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
- High contrast is an accessibility appearance, not a dark-theme alias.
  System mode must override an application's light/dark preference when the OS
  requests high contrast. Win32 uses `SPI_GETHIGHCONTRAST` plus user-selected
  `GetSysColor` pairs; AppKit and GTK4 resolve their semantic appearance/theme
  colors. The deterministic shared palette is only a backend fallback.
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
  and Space activate the focused action. Accessibility dialog semantics, prior
  focus restoration, arbitrary ViewNode content, validation/deferrals and
  AppKit/GTK target interaction smoke remain readiness gaps.
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
- ToggleButton is an independent `toggle-button` Cargo feature and reuses the
  shared explicit Boolean `Toggled` message path rather than inheriting Button
  behavior or storing state in a backend. It remains self-drawn: Windows,
  macOS and GTK select internal metric profiles, while hover/pressed state is
  transient runtime decoration and checked state has both fill and a shape cue.
  Indeterminate mode, accessibility and non-Windows target proof remain gaps.
- Large collections use virtualization, pagination, background prefetch and a
  small bounded cache; product storage remains outside the framework.

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
