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
