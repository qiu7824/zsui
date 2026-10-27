# Device Console

Six independent native applications implement the same deterministic workload:
ZSUI, eframe/egui, Iced, Slint, Tauri/WebView2 and GPUI Kit. They share the device
records and Rust interaction model in `examples/support/device_demo.rs`. Tauri
receives those same records through its initialization script.

The 960 × 680 logical client area contains a bilingual heading and summary,
one text filter, one online-only checkbox, a reset button, six selectable
device rows, a 0–100 load slider, a progress indicator and a selection label.
The initial load is 42%, device zero is selected, and all six rows are visible.
Filtering is case-insensitive and preserves selection. Reset restores the
initial state. These are synthetic records; the applications perform no network
requests or device management. There is no idle polling.

## Run

```sh
cargo run --release --example device_console --no-default-features --features window,button,label,textbox,checkbox,slider,progress
cargo run --release --manifest-path comparisons/egui_notepad/Cargo.toml --bin egui-device-console
cargo run --release --manifest-path comparisons/iced_notepad/Cargo.toml --bin iced-device-console
cargo run --release --manifest-path comparisons/slint_notepad/Cargo.toml --bin slint-device-console
cargo run --release --manifest-path comparisons/tauri_notepad/Cargo.toml --bin tauri-device-console --no-default-features --features device-demo
cargo run --release --manifest-path comparisons/gpui_kit/Cargo.toml --bin gpui-device-console
```

GPUI Kit is pinned to 0.8.1, using its default styled components and assets with
the matched upstream GPUI snapshot. The optional Fast backend, WebView, speech,
syntax highlighting and JavaScript extension host are not enabled. See the
[official installation guide](https://gpui-kit.com/docs/installation/).

## Measurement

`Device Demo Comparison` builds release applications on one Windows Server 2025
runner. `scripts/measure-ui-performance-matrix.ps1 -DeviceDemo` measures binary
size and hash, cold-like launch and warm first-frame proxies, empty/page/hidden
working set, private working set, private bytes, peak working set and idle/repaint
CPU. Tauri includes the WebView2 process tree. Windows PSS is unavailable and
must remain null; GPU memory is not inferred from process RAM.

All builds use `opt-level=z`, thin LTO, one codegen unit, stripped symbols and
panic abort. Five independent rounds rotate framework order. Each round starts
five page processes for startup, takes ten memory samples after a three-second
warmup, and uses separate empty-window and redraw processes. Per-process memory
medians precede the cross-round median. Raw rounds and screenshots remain in the
artifact. The first-frame counter is a visible-client/redraw/DwmFlush proxy,
not an optical presentation measurement. The file cache is never purged.

`--benchmark-empty` hides the demo content while retaining the same application
and renderer setup. `--benchmark-repaint` requests repeated redraws; ZSUI's
native host receives Win32 redraw requests from the measurement program. These
flags do not affect the ordinary interactive demo.

These new demo measurements form their own comparison table. They must not be
mixed with the earlier invoice app, Notepad, proxy-client or four-profile
performance results. Visual review and working filter/selection/reset behavior
are required before accepting the six applications as comparable.
