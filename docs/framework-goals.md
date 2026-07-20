# ZSUI Framework Goals

This is the long-range target for the standalone ZSUI framework. The
machine-readable version lives in `src/framework_goals.rs` and is exposed by
`zsui_rust_first_goals()`.

ZSUI should feel like Rust, not like a C++ or C# control hierarchy ported into
Rust. The public API should be safe, typed, explicit and easy to trim at build
time. Native platform details belong behind host traits and backend modules.

The revised target is: build a standalone Rust-first native UI framework, keep
product behavior outside the crate, and
make the ordinary app path as small as:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

That one-line entry is the desktop target for Windows, macOS and Linux. Android
and Harmony remain first-class platform targets, but they need real
Activity/Ability runtime hosts and device smoke proof before this same public
shape can create mobile native surfaces.

## Highest Priority: Fully Unified Application Authoring

Windows, macOS and Linux applications must use one ordinary Rust authoring
model. The same source owns `State`, `Msg`, `view`, `update`, semantic component
declarations, theme overrides and the `native_window(...)` entry. Application
view code must not select a platform, renderer or host and must not contain
target `cfg`, `PlatformStyle` matches, raw handles, Objective-C/GTK objects or
duplicated per-platform view trees. Platform packaging metadata is generated or
configured outside the application view source.

"Unified" applies to the public authoring model, not to the backend
implementation or to visual output. ZSUI must preserve each platform's own
composition and interaction conventions:

- `PlatformExperience` resolves semantic navigation, toolbar/header, tabs,
  forms, dialogs, popups, metrics, typography and icon sources inside the
  framework. It may produce different component trees for the same semantic
  declaration. `src/platform/identity.rs` owns the canonical target/toolkit
  types, and `NativeUiPlatform::current_target` selects the build target;
  backend inventory and launch metadata are derived from the experience
  registration rather than maintained as parallel platform tables.
- Low-level render and proof APIs share one `ZsPlatformStyle` profile selected
  by `PlatformExperience`. Existing component-specific style names remain
  compatibility aliases, not separate selectors or application-level choices.
  `PlatformExperience::shared_component_style` is the only target-experience
  to component-profile mapping; component modules consume the resolved shared
  profile and never select a target experience themselves.
  `src/platform/component_profile.rs` groups the framework-owned composition
  and metrics for semantic sections, adaptive navigation, base buttons and
  command bars while remaining independent from native backend selection.
- A compile-time backend profile owns `Host`, `Text`, `Raster`, `Presenter` and
  `Services`. Windows, AppKit and Linux implementations remain independent and
  can use different event loops, text stacks, rasterizers and system services.
- Every text backend must expose one canonical layout result used by measure,
  paint, hit testing, caret/selection geometry and accessibility. A demo-local
  font or clipping correction is not an acceptable fix.
- Applications may customize semantic specs and public theme/token values once;
  ZSUI resolves those values against the current platform without requiring an
  application branch.
- Cargo features remain the build boundary. A unified API must not force every
  widget, renderer or platform integration into the final binary.

The acceptance gate is one unchanged Gallery, Notepad or utility application
source compiled for all three desktop targets. Each target must launch its real
host, execute the same semantic scenario and emit final-surface screenshots and
structured evidence. Example-only platform adjustments do not satisfy this
goal; reusable corrections belong in framework contracts and backends first.

## v0.3 Core Milestone: Native Proof CI

ZSUI 0.3.0 prioritizes repeatable native runtime evidence over adding more
component declarations. Every UI-affecting change must eventually launch the
real Win32, AppKit and Linux application paths on target runners, execute fixed
interaction scenarios, capture the final platform view, emit structured
layout/focus/event evidence and compare it against reviewed baselines.

The original first blocking target—AppKit on the fixed GitHub-hosted
`macos-15` ARM64 runner—is now operational. Shared `DrawPlan` images are not
target evidence; the AppKit gate captures the final `NSView`. The current
blocking target is one unchanged application-authoring path with aligned
Win32, AppKit and Linux evidence and reviewed regression baselines. Baselines
are read-only in CI and can change only through an explicit reviewed commit.
The complete milestone, phased delivery order and release gates are defined in
[`v0.3-native-proof-ci.md`](v0.3-native-proof-ci.md).

## Delivery Order

ZSUI is delivered through runnable vertical slices rather than contract count.
The current priority order is:

1. Make fully unified application authoring the enforced desktop boundary:
   remove remaining application platform branches and renderer selection, move
   platform composition into framework-owned experience contracts, and prove
   one unchanged source on Win32, AppKit and Linux.
2. Harden the operational AppKit Native Proof CI gate on `macos-15`: retain
   real `NSApplication`/`NSWindow`/`NSView` execution and deterministic Gallery
   and Notepad scenarios, then add reviewed visual baselines.
3. Move the existing Win32 proof and the Linux target runtime onto the same
   versioned proof schema and regression tool so all three desktop paths block
   regressions before the 0.3.0 release.
4. Complete the reusable conversation/task workbench loop: navigation,
   timeline scrolling, composer input, tool/message actions and inspector state.
5. Stabilize the Rust-first application loop so typed messages update explicit
   state and repaint the live window, then cover IME, accessibility, focus,
   menus and dialogs required by native utility applications.
6. Tighten Cargo feature boundaries and split crates only after the public
   widget/runtime boundaries are proven by real applications.
7. Replace Android Activity and Harmony Ability scaffolds with real FFI hosts
   and device smoke artifacts.

Protocol manifests, AI handoff metadata and mobile bridge contracts support
these slices, but they do not advance product readiness by themselves. New
contract-only work should be deferred when a runnable slice still has an
unclosed input, state, paint or target-verification gap.

Buffered no-flicker self-draw is the Windows rendering baseline:
avoid background erase, paint into an owned buffer when possible, then present
once to the target surface. Wider platform APIs such as `windows-rs` should be
introduced only when a concrete backend surface needs them; core declarations
must stay independent of raw platform handles and broad native dependencies.

## Component Coverage

`zsui_component_catalog()` is the component-level source of truth. The current
catalog covers 48 desktop component families: 46 have a first-pass runtime
surface, 2 have contracts only and none are not started. Canvas and Flyout
remain contract-only. Composite shells do not change those statuses.
Embedded browser controls are intentionally outside the v0.2 product boundary.

`workbench` is the first reusable application-shell feature. It provides
navigation history, a message timeline, paragraph/code/tool/notice blocks,
composer controls and an optional inspector while leaving persistence, model
execution and product commands outside the framework.

`document-shell` provides the same product-neutral boundary for text-oriented
utilities: a document tab, command bar, rounded editor frame, status surface,
semantic icons and stable hit regions. Its `ZsTextDocument` adds UTF-8 and
BOM-tagged UTF-16 loading, explicit dirty state and transactional UTF-8 save or
save-as without platform handles. The Windows notepad benchmark proves the
hybrid route with a native text service and the target-dispatched
`NativeFileDialogService`. Parent-window modality, multi-tab state, target
interaction proof and a reusable rich-text engine remain incomplete.

`calculator` provides a complete standard-mode vertical slice at the framework
level: typed decimal operations, memory, history, a platform-adaptive View
keypad, semantic icons and stable namespaced actions. One unchanged acceptance
source now enters Win32, AppKit or Linux through the normal stateful View
builder. Windows proves its real input-state-paint loop; calculator-specific
AppKit/Linux target screenshots remain separate evidence gates. This composite
does not increase the component-catalog count or imply scientific, programmer,
graphing, conversion, localization or accessibility parity.

First-pass component status is not Fluent conformance. A built-in component
may advance toward ready only when it uses the shared typography, spacing,
radius, control-metric and semantic-color tokens; emits semantic icons instead
of private glyph text; and has hover, pressed, disabled, focus-visible, dark
theme and high-contrast evidence. The Windows workbench now satisfies the
token and semantic-icon parts. Its complete state matrix and non-Windows native
bindings remain open gates.

## Rust-First API Shape

Use composition and traits instead of inheritance. The direction is:

```rust
trait View<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx);
    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: ViewEvent);
    fn paint(&self, cx: &mut ViewPaintCx);
}
```

Application UI should compose values instead of subclassing controls:

```rust
button("Save")
    .padding(Dp::new(12.0))
    .radius(RadiusToken::Medium)
    .on_click(Msg::SaveClicked)
```

Messages should be typed. Prefer `enum Msg` over string event names:

```rust
enum Msg {
    SaveClicked,
    NameChanged(String),
    WindowClosed,
}

button("Save").on_click(Msg::SaveClicked)
```

This keeps event handling exhaustive, refactorable and visible to
rust-analyzer. Do not introduce string event APIs such as
`button.on("click", callback)` for framework-level controls.

## Ownership And State

Native resources should be RAII-owned. Users should create windows, icons,
tray items, fonts and bitmaps as Rust values, and cleanup should happen through
`Drop`. Unsafe platform calls belong inside the backend:

```rust
let icon = Icon::from_file("app.ico")?;
let tray = TrayIcon::new(icon)?;

let window = Window::new()
    .title("ZSUI")
    .size(900, 600)
    .build()?;
```

Public APIs must not require users to call `DestroyWindow`, `DeleteObject`,
`Release` or equivalent platform cleanup functions.

Application state should be explicit:

```rust
struct AppState {
    input: String,
    dark_mode: bool,
    selected_index: Option<usize>,
}

fn view(state: &AppState) -> impl View<Msg> {
    column((
        textbox(&state.input).on_change(Msg::NameChanged),
        row([text("Dark mode"), toggle(state.dark_mode).on_toggle(Msg::ToggleDark)]),
    ))
}
```

Do not hide product state inside controls or global registries. State changes
should flow through typed messages and explicit contexts such as `AppCx`,
`ViewEventCx`, `ViewLayoutCx` and `ViewPaintCx`.
`AppCx::command(...)` and `AppCx::ui_command(...)` must leave the View runtime
through explicit shared executors. Native hosts execute them after releasing
internal route locks, and product `UiCommand` values delegate through
`ProductAdapterUiCommandExecutor` rather than a global event bus.

## Typed Data

Avoid raw `i32` and `f32` for UI units when the meaning matters. Use typed
units at API boundaries:

```rust
let padding = Dp::new(12.0);
let real_px = padding.to_px(dpi);
```

Geometry and layout should move toward `Px`, `Dp`, `Dpi` and `UiLength` rather
than loose width/height/DPI numbers.

Use strong IDs instead of strings:

```rust
let input_id = WidgetId::new();
let settings_id = WindowId::new();
```

`typed_native_window(...)` now provides an opt-in content typestate:
`NativeWindowContentMissing` can be configured but cannot build or run, while
attaching a View, live View, draw plan or shell layout produces
`NativeWindowContentReady`. Keep `native_window(...)` as the concise path for
legitimate empty native surfaces; add more typestate only when it prevents a
real invalid state.

## Styling

Theme values should be tokens, not scattered literals:

```rust
theme.color.surface;
theme.color.text_primary;
theme.radius.medium;
theme.spacing.md;
```

This keeps Windows 11 styling, dark mode, high contrast and brand themes
replaceable without rewriting widgets.

Reusable self-drawn layout patterns should stay product-neutral and preserve
their verified interaction and rendering invariants. A WinUI-style left-nav/
right-content shell should be expressed as
layout data and typed draw commands, not as a product settings screen. Grouped
cards, row titles, description text, row accessories, viewport masks,
scrollbars and action-button areas belong in reusable contracts such as
`ZsShellLayoutSpec`; the product crate owns the actual data and command
behavior.

## Error And Platform Boundaries

Fallible host operations return `Result<T, ZsuiError>`. Backend creation,
DPI discovery, icon loading, font creation and native service setup should not
panic or show framework-owned error message boxes for recoverable failures.

Platform capabilities must be trait-based and honest:

```rust
trait TrayBackend {
    fn create_tray(&self, desc: TrayDesc) -> Result<TrayId, ZsuiError>;
}

trait HotkeyBackend {
    fn register_hotkey(&self, hotkey: Hotkey) -> Result<HotkeyId, ZsuiError>;
}
```

Expose a coherent ZSUI API, but do not pretend Windows, macOS, Linux, Android
and Harmony support identical native services. Unsupported or partial behavior
must be visible through capability reports or `ZsuiError::Unsupported`.

Android and Harmony should use explicit mobile host contracts instead of
desktop metaphors. Android maps to Activity, lifecycle, Intent, ClipboardManager,
Storage Access Framework, IME and notification surfaces. Harmony maps to
Ability, Want, pasteboard, document picker, input method and notification or
card surfaces. Both targets require FFI/lifecycle bindings and target-device
smoke artifacts before completion claims. The current bridge contracts must
remain explicit about callback symbols, lifecycle/surface/input routes, safety
rules and device-smoke artifact names until real native FFI implementations
replace the scaffold state. Artifact reviewers should validate captured device
proof without generating fake screenshots or lifecycle logs.

## Build Trimming

ZSUI should be designed for explicit feature and crate boundaries:

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

The goal is feature/crate based trimming, not a claim that Cargo automatically
removes every unused symbol from an enabled crate. Keep optional dependencies
behind feature gates, keep defaults small, and move large widget families or
backend integrations into modules or crates that users opt into.
Every public feature and the supported family combinations must compile through
`scripts/check-feature-matrix.ps1`; `.github/workflows/ci.yml` keeps this gate on
Windows and also checks the default/full surfaces on Linux and macOS.

Cargo features are additive across a dependency graph. If another crate enables
`zsui/textbox`, the final build of the shared `zsui` dependency includes that
feature. The framework target is therefore to make the default set small, keep
heavy dependencies optional, and split larger surfaces when feature unification
would otherwise pull too much code into unrelated applications.

The crate split target is:

- `zsui-core`: ids, errors, events, layout/state traits and basic protocols.
- `zsui-shell`: windows, tray/status items, DPI, hotkeys and shell services.
- `zsui-render`: Direct2D, Skia, WGPU, GDI or other renderer backends.
- `zsui-style`: theme tokens, colors, radius, spacing and typography.
- `zsui-widgets-base`: Button, Label, Icon and Panel.
- `zsui-widgets-input`: TextBox, CheckBox, Slider and IME-facing controls.
- `zsui-widgets-list`: List, Tree and Table families.
- `zsui-widgets-extra`: Dialog, Toast, Chart and advanced widgets.

Per-widget crates such as `zsui-button`, `zsui-textbox`, `zsui-list` or
`zsui-dialog` are acceptable later if a widget family becomes large enough to
justify that granularity.

Avoid eager global widget registration:

```rust
register_widget(Button::new());
register_widget(TextBox::new());
register_widget(Table::new());
register_widget(TreeView::new());
```

Prefer imports and builders that only reference the controls the user enabled
and used:

```rust
use zsui::prelude::*;
use zsui::widgets::{Button, List};

fn app() -> impl View<Msg> {
    column((
        Button::new("Save"),
        List::new(items),
    ))
}
```

Release builds can add size-focused profiles such as `opt-level = "z"`, thin
LTO, `codegen-units = 1`, symbol stripping and `panic = "abort"`, but those are
secondary to clean Cargo feature boundaries.

## AI Context Trimming

AI context should be composable in the same way as Cargo features. The default
prompt starts with `docs/ai-agent.md`, then selects one task pack from
`docs/ai/context-packs.json`. A pack names a bounded required set, optional
follow-up files and focused verification commands.

Avoid making every agent load the complete readiness report, every platform
backend, all examples and generated artifacts before a local change. Use `rg`
inside the selected files, read focused ranges and add another pack only when
the task crosses a real ownership boundary. `scripts/ai-context.ps1 -Validate`
must keep pack paths valid; CI should reject stale routing metadata.

The long-range target is measurable: keep the bootstrap small, keep normal
required packs within one module family, and move detailed status/history into
optional references. This reduces repeated prompt tokens without hiding
architecture rules or weakening verification.

## Completion Definition

The framework is not complete just because declarations compile. A surface is
complete only when the Rust API is product-neutral, feature-gated where needed,
covered by tests or examples, backed by a real host implementation when claimed
as native, and has target smoke artifacts for the OS or device. Until then,
report code-level readiness, target-smoke readiness and system-complete status
separately.

Progress must be reported on three separate scales:

- implementation readiness: framework code has been implemented and tested;
- runnable-platform readiness: a real host can create, interact with and
  repaint the surface on the target OS;
- framework product readiness: an external application can build and ship the
  workflow without depending on application internals.

An overall percentage must refer to framework product readiness. Internal
implementation milestones must not be presented as overall framework completion.
