# ZSUI Framework Goals

This is the long-range target for the standalone ZSUI framework. The
machine-readable version lives in `src/framework_goals.rs` and is exposed by
`zsui_rust_first_goals()`.

ZSUI should feel like Rust, not like a C++ or C# control hierarchy ported into
Rust. The public API should be safe, typed, explicit and easy to trim at build
time. Native platform details belong behind host traits and backend modules.

The revised target is: extract ZSClip's reusable native UI foundation into a
standalone Rust-first framework, keep product behavior outside the crate, and
make the ordinary app path as small as:

```rust,no_run
zsui::native_window("Example").size(900, 620).run()?;
# Ok::<(), zsui::ZsuiError>(())
```

That one-line entry is the desktop target for Windows, macOS and Linux. Android
and Harmony remain first-class platform targets, but they need real
Activity/Ability runtime hosts and device smoke proof before this same public
shape can create mobile native surfaces.

ZSClip's latest no-flicker self-draw work is the Windows rendering baseline:
avoid background erase, paint into an owned buffer when possible, then present
once to the target surface. Wider platform APIs such as `windows-rs` should be
introduced only when a concrete backend surface needs them; core declarations
must stay independent of raw platform handles and broad native dependencies.

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
        checkbox("Dark mode", state.dark_mode).on_toggle(Msg::ToggleDark),
    ))
}
```

Do not hide product state inside controls or global registries. State changes
should flow through typed messages and explicit contexts such as `AppCx`,
`ViewEventCx`, `ViewLayoutCx` and `ViewPaintCx`.

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

As the API matures, required app/window states should move toward typestate
builders where that improves compile-time safety without making the simple
path noisy.

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
- `zsui-widgets-extra`: Dialog, Toast, Chart, WebView and advanced widgets.

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

## Completion Definition

The framework is not complete just because declarations compile. A surface is
complete only when the Rust API is product-neutral, feature-gated where needed,
covered by tests or examples, backed by a real host implementation when claimed
as native, and has target smoke artifacts for the OS or device. Until then,
report code-level readiness, target-smoke readiness and system-complete status
separately.
