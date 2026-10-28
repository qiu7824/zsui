# Device Console

Six independent native applications implement the same deterministic workload:
ZSUI, eframe/egui, Iced, Slint, Tauri/WebView2 and GPUI Kit. They share the device
records and Rust interaction model in `examples/support/device_demo.rs`. Tauri
receives those same records through its initialization script.

The 960 × 680 logical client area uses a light desktop theme, three white summary
cards, a compact searchable device list and a selected-device detail panel.
Blue marks selection and the primary reset action. One text filter, one
online-only checkbox, six selectable device rows, a 0–100 load slider and a
progress indicator retain the same interactions across all six frameworks.
The initial load is 42%, device zero is selected, and all six rows are visible.
Filtering is case-insensitive and preserves selection. Reset restores the
initial state. These are synthetic records; the applications perform no network
requests or device management. There is no idle polling.

## Run

```sh
cargo run --release --example device_console --no-default-features --features window,button,label,textbox,checkbox,slider,progress,grid
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

`Device Demo Builds` compiles the six release applications and validates the
shared model on Windows Server 2025. The hosted adapter rejected egui/glow's
OpenGL requirement, so that job publishes binaries and build identities rather
than claiming native graphics measurements. Run all binaries on one supported
graphics desktop with:

```powershell
./scripts/measure-ui-performance-matrix.ps1 -DeviceDemo -SkipBuild -RepeatRuns 5 -MemorySamples 10 -StartupRuns 5 -IncludeGpuMemory -SupportRoot <outside-workspace-directory>
./scripts/check-device-demo-results.ps1 -Report <report.json>
```

The measurement records binary
size and hash, cold-like launch and warm first-frame proxies, empty/page/hidden
working set, private working set, private bytes, peak working set and idle/repaint
CPU. Tauri includes the WebView2 process tree. Windows PSS is unavailable and
must remain null. Optional GPU memory comes from Windows process counters;
shared allocations can appear in more than one process and are not a physical
memory accounting equivalent to PSS.

All builds use `opt-level=z`, thin LTO, one codegen unit, stripped symbols and
panic abort. Five independent rounds rotate framework order. Each round starts
five page processes for startup, takes ten memory samples after a three-second
warmup, and uses separate empty-window and redraw processes. Per-process memory
medians precede the cross-round median. Raw rounds and screenshots remain in the
artifact. The first-frame counter is a visible-client/redraw/DwmFlush proxy,
not an optical presentation measurement. The file cache is never purged.

An interrupted run can continue with the same arguments plus `-Resume`.
Completed observations remain in per-round checkpoints; resume requires the
same sampling settings, executable manifest and measurement-script hash.
Visible-window selection excludes zero-client helper windows and retains the
chosen window identity. WebView2 descendants are accepted as part of their
owning Tauri application. Occluded observations are discarded and retried.
GPU queries with no process instances report zero attributed counters; this
does not include compositor-owned memory or prove a zero-cost display path.

`--benchmark-empty` hides the demo content while retaining the same application
and renderer setup. `--benchmark-repaint` requests load, slider, numeric and
progress updates every 16 ms using each toolkit's application scheduler. ZSUI
uses its invalidation handle, with no polling in ordinary interactive mode.
Actual presentation rate is not implied by this requested frequency. These
flags do not affect the ordinary interactive demo.

These new demo measurements form their own comparison table. They must not be
mixed with the earlier invoice app, Notepad, proxy-client or four-profile
performance results. Visual review and working filter/selection/reset behavior
are required before accepting the six applications as comparable.
