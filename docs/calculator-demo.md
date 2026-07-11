# ZSUI Calculator Demo

`examples/zsui_calculator.rs` is a runnable modern Windows calculator used to
verify ZSUI's self-drawn application shell, typed calculator state and native
input loop against the Windows system Calculator. It covers the standard mode;
it is not a claim of product parity with every Calculator mode.

## Included Behavior

- Decimal arithmetic backed by `rust_decimal`, including exact `0.1 + 0.2`.
- Add, subtract, multiply, divide, repeated equals and contextual percent.
- Reciprocal, square, square root, sign and backspace operations.
- Clear, clear-entry, five memory actions and a recent-history panel.
- Mouse hover/press/capture and keyboard input for digits and operators.
- DPI-aware Fluent spacing, typography, semantic colors and rounded controls.
- Semantic calculator, history and backspace icons instead of text glyphs.
- Buffered Win32/GDI painting with background erase suppressed.
- A real application icon, minimum window size and per-monitor DPI handling.

Run the application:

```powershell
cargo run --example zsui_calculator --no-default-features --features calculator-demo
```

Run its auto-closing smoke path:

```powershell
cargo run --example zsui_calculator --no-default-features --features calculator-demo -- --smoke
```

The reusable API is feature gated:

```toml
zsui = { version = "0.1", default-features = false, features = [
    "calculator",
    "windows-gdi",
] }
```

`ZsCalculatorEngine` owns typed decimal state and operations.
`ZsCalculatorShellSpec` owns the DPI-aware visual layout, draw plan and hit
regions. The example's platform module currently owns the Win32 event loop.

## Reproducible Comparison

The comparison script builds the ZSUI release binary, starts both calculators,
waits five seconds, samples process counters six times, captures both windows
and writes JSON plus Markdown reports under `target/calculator-comparison`.

```powershell
.\scripts\measure-calculator-comparison.ps1
```

A reference run on Windows 11 build 26200 with Rust 1.94 and Windows Calculator
11.2605.9.0 produced:

| Implementation | Processes | App files | App lines | Packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI Calculator | 1 | 2 | 442 | 28 | 0.28 MiB | 1.24 MiB | 10.66 MiB | 1.92 MiB |
| Windows Calculator process group | 2 | system app | system app | n/a | not comparable | 47.48 MiB | 176.65 MiB | 127.17 MiB |

In this run, `CalculatorApp` itself used 26.96 MiB by the Task Manager
private-working-set metric. Its visible window was owned by a separate
`ApplicationFrameHost`, which used 20.52 MiB. The process-group row sums both.
On launches where Calculator owns its own visible window, the script records
one process instead. A shared frame host is not an isolated framework
allocation, so both the grouped and component values must remain visible.

The packaged `CalculatorApp.exe` file is only one part of the installed app and
does not include package assets or a separate frame host. Its file size is not
a useful comparison with the standalone ZSUI executable.

Memory values are steady-state observations from one machine, not universal
constants. "Task Manager memory" means private working set. Total working set
includes resident shared pages, while private bytes measures committed private
virtual memory; these counters are different quantities.

## What The Result Means

ZSUI has a concrete size and idle-overhead advantage for this standard-mode
calculator. It also gives applications exact decimal input arithmetic, a
framework-owned visual shell and full control over product behavior without a
large UI runtime.

Windows Calculator remains much broader. It includes scientific, graphing,
programmer and date modes, unit and currency conversion, mature localization,
accessibility, touch behavior and deeper operating-system integration. ZSUI's
small process does not establish feature parity.

The application-specific part is two Rust files and 442 lines. That is compact
for a raw native self-drawn program, but it is not yet as concise as a mature
declarative Rust GUI toolkit because the example still contains Win32 window,
input and lifecycle plumbing. The next API reduction target is a reusable
calculator runtime attached through `NativeWindowBuilder`, leaving an
application to provide only its title, theme and optional action hooks.

The reusable engine and shell currently occupy one 1,270-line framework
module and have focused tests for decimal precision, repeated equals, error
recovery, memory, contextual percent, pending unary operands, clear-entry
semantics, history layout and semantic icon drawing.
