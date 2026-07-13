# Native Host Smoke Artifacts

Target smoke is the evidence layer between code-level contracts and a complete
native backend. A platform is not system-complete until a real target run stores
inspectable artifacts under:

```text
target/native-host-smoke/<platform>/
```

Generate the smoke manifest with:

```powershell
cargo run --example native_smoke_manifest -- windows
```

Use `macos`, `linux`, `android`, `harmony`, `current` or `all` for other
manifest scopes.

Record the contract-level artifact files with:

```powershell
cargo run --example native_smoke_record -- windows
```

This writes `manifest.json`, `launch.log`, `interaction.json`,
`capabilities.json` and `agent-context.json`. It intentionally does not fake
`window.png`; run the interactive native smoke command to capture or provide a
real target screenshot before target-smoke is complete.

Run the first-pass native smoke window with:

```powershell
cargo run --example native_smoke_run -- windows
```

On Windows this opens the Win32/GDI native window path, closes it
automatically, then rewrites `interaction.json` and `launch.log` with the
observed window lifecycle. It also captures `window.png` into the artifact
directory through the direct Win32 `HWND`. macOS now enters `NSApplication`
with owned `NSWindow` objects, and Linux enters `GtkApplication` with owned
`ApplicationWindow` objects. Both direct native smoke paths auto-close and
record lifecycle evidence, but still need target screenshot capture before
target-smoke proof is complete.

Windows can also request a real status item during the same smoke run:

```powershell
cargo run --example native_smoke_run -- windows --tray
```

That path uses the `Shell_NotifyIconW` backed `WindowsWin32StatusItemHost` and
records status-item fields in `interaction.json`. It also exercises the
native status-menu command table and records `status_menu_command_routed`.
It creates and destroys a native popup menu and records
`status_menu_popup_destroyed`. Real user popup menu clicks are still separate
proof before the tray surface is system-complete; the Win32 host exposes the
`TrackPopupMenu` selection route, but the auto-closing smoke runner does not
block waiting for manual selection.

The native window-menu smoke path is:

```powershell
cargo run --example native_smoke_run -- windows --menu
```

On Windows this installs an owned `HMENU` plus `HACCEL`, preserves nested and
disabled item state, and records typed window-menu command routing in
`interaction.json`. The same `MenuSpec` uses `Primary+O`/`Primary+S`, which the
AppKit and GTK4 menu services lower to their platform-native accelerator forms.
Target interaction proof for those services still requires real macOS/Linux
hosts.

All three direct desktop hosts attach a typed Rust view draw plan to their
native content surface. Win32 paints through its buffered GDI sink, AppKit
through a custom `NSView`, and GTK4 through `DrawingArea`/Cairo/Pango. Windows
posts native pointer messages during the smoke run. AppKit mouse down/drag/up and
GTK4 gesture/motion controllers are connected to the same typed
hit-test/message/executor path,
while AppKit `scrollWheel:` and GTK4 `EventControllerScroll` emit the same
typed `ScrollBy` path. Their focusable content views also route Tab/Shift+Tab,
keyboard activation and direct UTF-8 edits. AppKit `NSTextInputClient` and GTK4
`GtkIMMulticontext` now route provisional preedit, committed UTF-8 and candidate
window anchors through the same shared runtime. Each renderer also feeds its
actual content bounds back into shared layout before painting, so resize updates
draw commands, hit targets and text-input geometry rather than stretching a
startup snapshot. Pointer/Tab focus appends the same semantic accent focus ring
on all three draw sinks, while native focus loss rebuilds the clean plan. The
Windows interaction artifact records this as `native_view_focus_visual_count`.
Both still require target-machine interaction artifacts:

```powershell
cargo run --example native_smoke_run -- windows --view
```

The dedicated typed scroll smoke path is:

```powershell
cargo run --features "scroll,label" --example native_smoke_run -- windows --scroll-view
```

The dedicated typed slider smoke path is:

```powershell
cargo run --features "window,label,slider,windows-win32" --example native_smoke_run -- windows --slider-view
```

It presses the shared slider track, drags the thumb, releases pointer capture
and sends a Left key step through the same strongly typed `SliderChanged`
route used by AppKit and GTK4. The smoke runner attaches the framework runtime
executor, so each emitted `UiCommand` must be executed without an unhandled
command. The Windows interaction artifact records value
changes, keyboard changes and completed drags as
`native_view_slider_value_change_count`,
`native_view_slider_keyboard_change_count` and
`native_view_slider_drag_count`. AppKit and GTK4 use their native mouse/gesture
and keyboard callbacks with the shared runtime, but still require target-machine
interaction artifacts before their slider path is considered proven.

The dedicated ToggleButton smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,toggle-button,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-toggle-button --toggle-button-view
```

It clicks the self-drawn button, activates it with Space, then clicks again so
the screenshot finishes in the checked state. The application owns the
explicit Boolean state and receives the same typed callback for pointer and
keyboard activation. The runtime also records transient hover/pressed redraws
without introducing a backend-local control tree. The checked background and
bottom state cue follow the official [Windows App SDK ToggleButton](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.primitives.togglebutton?view=windows-app-sdk-1.8),
[Apple toggle-button guidance](https://developer.apple.com/design/human-interface-guidelines/toggles),
and [GTK4 ToggleButton](https://docs.gtk.org/gtk4/class.ToggleButton.html)
contracts. The Windows artifact must report three toggle events, one keyboard
activation, pointer visual changes, successful `UiCommand` execution and a
captured `window.png`. AppKit and GTK4 use the shared state/input path and
platform metrics but still require target-machine interaction evidence.

The dedicated editable NumberBox smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,number-box,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-number-box --number-box-view
```

It clicks the trailing increment segment, applies small and large keyboard
steps, clears and replaces the editable draft, then commits `42.5` with Enter.
The self-drawn header chooses Windows inline down/up buttons, an AppKit-style
compact vertical two-segment stepper or GTK horizontal decrement/increment
buttons internally; application code has no platform branch. The behavior and
shape profiles follow the official [Windows NumberBox](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/number-box),
[Apple stepper](https://developer.apple.com/design/human-interface-guidelines/steppers),
and [GTK SpinButton](https://docs.gtk.org/gtk4/class.SpinButton.html) contracts.
The Windows artifact must capture
`window.png`, expose three hit targets, keep every pointer/key input handled,
execute each emitted `UiCommand` without failure or an unhandled command, and
finish with a nonzero live-view revision. AppKit and GTK4 share the typed
draft/commit path but still require target-machine interaction evidence.

The dedicated secure PasswordBox smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,password-box,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-password-box --password-box-view
```

It focuses the self-drawn field, inserts Unicode-safe committed text through
the real Win32 route, then presses and releases the trailing reveal target.
Windows follows the official [PasswordBox](https://learn.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.passwordbox?view=winrt-26100)
and [PasswordRevealMode](https://learn.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.passwordbox.passwordrevealmode?view=winrt-26100)
press-and-hold Peek model. macOS defaults to a hidden field following
[NSSecureTextField](https://developer.apple.com/documentation/appkit/nssecuretextfield),
while GTK follows [GtkPasswordEntry](https://docs.gtk.org/gtk4/class.PasswordEntry.html)
and keeps its optional peek affordance disabled by default. The shared draw
plan, event JSON, IME report and smoke artifacts must not contain the secret;
only the renderer receives it at the final platform text call. The Windows
artifact must expose two hit targets, capture `window.png`, handle four text
inputs and both pointer pairs, execute all four typed `UiCommand` values, and
finish with no command errors. Alt+F8, caps-lock/accessibility signaling,
locked memory and target-machine AppKit/GTK evidence remain explicit gaps.

The dedicated attached ToolTip smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,button,label,tooltip,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-tooltip --tooltip-view
```

It moves focus to a normal self-drawn owner with Tab and captures the concise,
noninteractive help overlay centered above it. A deterministic Win32 route test
also advances the pointer-hover deadline and verifies that the tooltip is added
to the buffered draw plan without adding a second hit target. Runtime behavior
follows the official [Windows ToolTips](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/tooltips),
[AppKit `NSView.toolTip`](https://developer.apple.com/documentation/appkit/nsview/tooltip)
and [GTK `query-tooltip`](https://docs.gtk.org/gtk4/signal.Widget.query-tooltip.html)
contracts. Win32 reads `SPI_GETMOUSEHOVERTIME` and
`SPI_GETMESSAGEDURATION`; AppKit and GTK schedule owned one-shot callbacks.
Top-level overflow outside the current window, accessibility relationships and
target-machine AppKit/GTK artifacts remain explicit gaps.

The dedicated typed RadioButton smoke path is:

```powershell
cargo run --no-default-features --features "window,label,radio,windows-win32" --example native_smoke_run -- windows --radio-view
```

It starts with one selected option, clicks a sibling option, rebuilds the
stateful view so the selection remains mutually exclusive, activates the
focused option with Space, then presses Up to move focus and selection back to
the previous logical option without wrapping. A final Tab stays on that
selected option because it is the group's only Tab stop. The artifact records
the common selection route in `native_view_radio_selection_count`, the
directional keyboard route in `native_view_radio_keyboard_selection_count` and
the Tab route in `native_view_focus_traversal_count`; all emitted
`UiCommand` values must execute without failures or unhandled commands. AppKit
and GTK4 consume the same `RadioSelected` event, single-group Tab stop and
group navigation through their native pointer and key callbacks. Ctrl+arrow
focus-only navigation does not emit a selection message and is reported
separately by `native_view_radio_keyboard_focus_only_count` when exercised;
AppKit and GTK4 target-machine interaction evidence remains pending.

The dedicated determinate ProgressBar smoke path is:

```powershell
cargo run --no-default-features --features "window,label,progress,windows-win32" --example native_smoke_run -- windows --progress-view
```

It attaches a 65% progress value through `ProgressRange`, paints the shared
semantic track/fill plan through the buffered Win32 renderer, captures the
window and keeps the feedback-only control out of the hit-test plan. AppKit and
GTK4 consume the same draw commands; target screenshots for those hosts remain
pending.

The independently selectable ProgressRing smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,progress-ring,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-progress-ring --progress-ring-view
```

It places an active indeterminate ring beside a 65% determinate ring, captures
the buffered Win32 window and records repeated live-view background refreshes
while keeping the feedback controls out of the hit-test plan. The shared arc
command is rendered with GDI+, NSBezierPath or Cairo, and the host loop uses a
Win32 timer, owned `NSTimer` or cancellable GLib timeout respectively. The
behavior follows the official [WinUI progress-control guidance](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls),
[AppKit `NSProgressIndicator`](https://developer.apple.com/documentation/appkit/nsprogressindicator)
and [GTK4 `GtkSpinner`](https://docs.gtk.org/gtk4/class.Spinner.html). macOS and
Linux target-machine animation screenshots remain required.

The dedicated typed ComboBox smoke path is:

```powershell
cargo run --no-default-features --features "window,label,combo,windows-win32" --example native_smoke_run -- windows --combo-view
```

It begins expanded, selects an overlay option with the pointer, reopens with
Space, selects another option with Down, types `B` to select `Balanced` through
the one-second type-ahead buffer, reopens, and scrolls the long popup with the
pointer wheel. The popup follows WinUI's default 15-item cap, shrinks further
to fit the available viewport, initially keeps the selected option visible,
and is painted after ordinary siblings. Its visible option hit targets retain
global indices and overlay priority without becoming extra Tab stops. The
interaction artifact records
`native_view_combo_expanded_change_count`,
`native_view_combo_selection_count`,
`native_view_combo_keyboard_selection_count`, and
`native_view_combo_type_ahead_match_count`, and
`native_view_combo_scroll_count`; all emitted `UiCommand` values must
execute without failures or unhandled commands. AppKit and GTK4 feed committed
text and pointer scroll into the same shared typed runtime, while their
target-machine evidence remains pending.

The dedicated strongly typed Tabs smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,tabs,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-tabs --tabs-view
```

It clicks the second `ZsTabId`, rebuilds the stateful view with only that page
laid out and painted, then exercises Windows header focus with Left/Right and
selection with Space/Enter. The artifact must record nonzero
`native_view_tab_selection_count`,
`native_view_tab_keyboard_selection_count`, and
`native_view_tab_keyboard_focus_only_count`, plus zero failed or unhandled UI
commands. Ctrl+Tab/Ctrl+Shift+Tab cycling is covered by the native route tests.
AppKit and GTK4 consume the same typed selection path with their platform arrow
selection behavior, but still require target-machine screenshots and
interaction artifacts.

The dedicated strongly typed TimePicker smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,label,time-picker,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-time-picker --time-picker-view
```

It starts with the self-drawn picker open, chooses a 15-minute value through a
typed popup hit target, closes with Escape, adjusts minutes and hours from the
keyboard, then reopens the popup. The Windows artifact must capture
`window.png`, keep all pointer/key inputs handled, execute all emitted
`UiCommand` values, and retain a nonzero live-view revision. AppKit and GTK4 use
their own metric profiles through the same `ZsTime` event path, while actual
target-machine screenshots remain a separate gate.

The dedicated typed Grid layout smoke path is:

```powershell
cargo run --locked --no-default-features --features "window,button,label,grid,native-smoke" --example native_smoke_run -- windows target/native-host-smoke-grid --grid-view
```

It lays out fixed and weighted fractional tracks, independent row/column gaps,
an explicit three-column header span, a two-column content span and a typed
button hit target from the same DPI-aware geometry. The Windows artifact must
capture `window.png`, route the `grid_apply` command without an unhandled click
and keep all six target-smoke files valid. AppKit and GTK4 consume the same
layout, paint and hit bounds, while their target screenshots remain separate
gates.

The default `--view` and `--scroll-view` paths exercise
`NativeWindowBuilder::ui_command_view(...)`, record
draw-plan command counts in `interaction.json`, post `WM_LBUTTONUP`, hit-test
through `ViewInteractionPlan`, dispatches into `ViewEventCx<UiCommand>` and
records the emitted command ids. When an executor is attached it also records
executed, failed, unhandled and emitted-event counts instead of treating command
generation as execution proof. It also paints the resulting `NativeDrawPlan`
through the buffered no-flicker Win32/GDI renderer. When built with the
`textbox` feature, the same path focuses a textbox and routes `WM_CHAR` text
input into `TextChanged`/`UiCommand` output. When built with the `checkbox`
feature, it routes checkbox clicks into `Toggled`/`UiCommand` output. It also
records typed row selection when built with the `list` feature, including
Up/Down keyboard selection between focused rows. It also posts `WM_KEYDOWN`
Tab to prove ordered focus traversal and Enter to prove focused keyboard
activation into the same `UiCommand` path; the resulting focus-ring repaint is
counted independently from logical focus changes. The textbox smoke also posts a
down/move/up drag sequence, verifies Unicode range replacement and records
`native_view_pointer_*`, `native_view_text_drag_count` and
`native_view_text_selection_change_count`. Shaped-glyph/grapheme/bidirectional
hit testing, non-Windows target input evidence and resize screenshot artifacts
remain later runtime gates. The `--date-picker-view` path also posts real
pointer down/up input through the Win32 host and records
`native_view_pointer_visual_change_count`; a nonzero count proves that the
semantic hover/pressed decoration reached the buffered native draw plan without
claiming the still-pending AppKit/GTK4 target runs.
Pass `--date-picker-high-contrast` to render the same typed DatePicker path with
`ZsuiThemeMode::HighContrast`. The smoke report must record
`high_contrast_draw_plan_window_count=1`, retain nonzero pointer-visual changes,
capture `window.png`, and finish without failed or unhandled UI commands. This
proves the explicit Windows high-contrast renderer path; toggling the operating
system accessibility setting and AppKit/GTK4 target runs remain separate gates.
When a smoke path supplies `NativeWindowSmokeRunOptions::native_view_scroll(...)`
and a command-backed scroll target, Win32 also records mouse-wheel scroll
counters and the emitted scroll `UiCommand`. The default `--view` example does
not yet include a scroll target because it keeps the existing button/textbox/
checkbox/list geometry stable; `--scroll-view` supplies that target.

Review the artifact directory with:

```powershell
cargo run --example native_smoke_review -- windows
```

The review is read-only. It checks required files, rejects empty artifacts,
validates JSON artifacts, validates the `window.png` PNG header and reports
`target_smoke_complete=false` until every required target artifact is present
and valid.

Required target-smoke artifacts:

- `manifest.json`: serialized `NativeHostSmokePlan`.
- `launch.log`: native runtime launch output and exit status.
- `window.png`: screenshot proving the native window was visible.
- `interaction.json`: structured interaction record.
- `capabilities.json`: observed host capability report.
- `agent-context.json`: matching `zsui_agent_context_json()` output.

Windows uses the `win32_gdi` runtime, macOS uses AppKit, and Linux uses GTK4.
All three enter their target-native event loop and paint supplied draw plans;
only Windows currently captures the required screenshot automatically.
Android and Harmony are still scaffold/bridge-contract plans until real
Activity/Ability runtime hosts exist. Their current device-smoke contract can
be inspected with:

```powershell
cargo run --example mobile_scaffold_manifest -- --bridge android
cargo run --example mobile_scaffold_manifest -- --bridge harmony
cargo run --example mobile_scaffold_manifest -- --parity android
cargo run --example mobile_scaffold_manifest -- --dispatch android
cargo run --example mobile_scaffold_manifest -- --dispatch-smoke android
cargo run --example mobile_scaffold_manifest -- --write-contract android
cargo run --example mobile_scaffold_manifest -- --review-contract android
cargo run --example mobile_scaffold_manifest -- --write-contract all target/mobile-contract-smoke
cargo run --example mobile_scaffold_manifest -- --review-contract all target/mobile-contract-smoke
cargo run --example mobile_scaffold_manifest -- --smoke android
cargo run --example mobile_scaffold_manifest -- --trace-template android
cargo run --example mobile_scaffold_manifest -- --review android
```

The mobile contracts require device-side artifacts such as
`device-launch.log`, `device-window.png`, `lifecycle.json`, `surface.json` and
`input.json` before a mobile backend can move beyond scaffold status. The
parity command reports required callback route coverage and pending FFI symbols.
The dispatch command maps the required callback symbols to lifecycle, surface,
typed input and `NativeRuntimeDriver` operations. The dispatch-smoke command
locally replays the required bridge sequence as a contract smoke only. The
write-contract command writes local contract JSON artifacts, including
`device-smoke-plan.json` and `agent-context.json`, but intentionally does not
create device launch logs, screenshots, lifecycle, surface or input traces. The
review-contract command validates only those local contract JSON artifacts and
their expected schemas. Both contract artifact commands accept `all` to cover
Android and Harmony. The review command validates device-smoke artifact
presence, JSON files, PNG headers and device-sourced trace schemas. The
trace-template command prints the lifecycle/surface/input JSON shape the
device-side bridge must write. None of these commands generates or fakes device
proof.

Current Windows proof command sequence:

```powershell
cargo run --example native_smoke_run -- windows
cargo run --example native_smoke_review -- windows
```

The Windows review should report `target_smoke_complete=true` after the run
because all six required artifacts, including `window.png`, are generated and
validated.
