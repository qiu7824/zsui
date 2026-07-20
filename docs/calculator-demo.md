# ZSUI Calculator Demo

`examples/zsui_calculator.rs` is a single-source desktop acceptance application
for Win32, AppKit and Linux. It uses the same public application shape as other
ZSUI applications:

```rust
native_window("ZSUI Calculator")
    .size(420, 680)
    .stateful_view(CalculatorState::default(), view, update)
    .run()?;
```

The application source contains no target `cfg`, raw window handle, native
drawing handle or platform event loop. `window` selects the target backend;
`calculator_view` resolves platform typography, spacing, button geometry,
primary-action presentation and semantic icons inside the framework.

## Included Behavior

- Decimal arithmetic backed by `rust_decimal`, including exact `0.1 + 0.2`.
- Add, subtract, multiply, divide, repeated equals and contextual percent.
- Reciprocal, square, square root, sign and backspace operations.
- Clear, clear-entry, five memory actions and a recent-history panel.
- Explicit `CalculatorState`, typed `Msg::Action` and exhaustive `update`.
- Stable, namespaced widget IDs for pointer, focus and proof automation.
- Platform-adaptive standard, primary and semantic-icon buttons.
- Target-native window, text shaping, painting, DPI and final-surface capture.

Direct digit/operator shortcuts are not yet part of the shared application-key
contract. All actions are pointer accessible and participate in the ordinary
button focus/activation path; raw Win32 key handling is deliberately not kept
in the example.

Run the application:

```powershell
cargo run --example zsui_calculator --no-default-features --features calculator-demo
```

Run the deterministic smoke path:

```powershell
cargo run --example zsui_calculator --no-default-features --features calculator-demo -- --smoke
```

The smoke opens a real target window, invokes `1 + 2 =` through four pointer
actions, verifies four typed messages and live View revisions, checks that the
semantic Display/Title text is `3`, captures the final surface and writes:

```text
target/zsui-calculator/
├── window.png
└── report.json
```

The reusable API remains optional:

```toml
zsui = { version = "0.2.0-preview.6", default-features = false, features = [
    "window",
    "calculator",
] }
```

`calculator` enables only its decimal engine plus the button, label, Grid and
style slices used by `calculator_view`. It does not enable the complete widget
catalog. `calculator-demo` additionally enables the target window and smoke
artifact writer.

## Ownership

- `ZsCalculatorEngine` owns values, pending operations, memory and history.
- `ZsCalculatorShellSpec` is an immutable projection of calculator state.
- `calculator_view` builds the platform-adaptive View tree and action routing.
- `ZsCalculatorViewIds` gives each instance a stable 64-ID namespace.
- Win32, AppKit and Linux backends own native lifecycle, presentation, input
  translation, typography and final-surface capture.

The calculator remains a standard-mode framework acceptance surface. It does
not claim scientific, programmer, graphing, conversion, localization or full
accessibility parity with operating-system calculators.

## Target Evidence

The Windows smoke records a real `WM_PRINTCLIENT` final-surface PNG, Win32
system typography, pointer routing, typed updates and process-memory evidence.
The same unchanged source cross-compiles for `aarch64-apple-darwin` and
`x86_64-unknown-linux-gnu`; calculator-specific AppKit and Linux screenshots
remain target-proof work and are not inferred from cross-compilation.

## Reproducible Windows Comparison

The comparison script builds the current release binary, starts the ZSUI and
Windows calculators, samples process counters, captures both windows and writes
JSON plus Markdown reports under `target/calculator-comparison`:

```powershell
.\scripts\measure-calculator-comparison.ps1
```

It counts only the single shared application source file. Process-group and
component measurements remain separate when `ApplicationFrameHost` owns the
visible Windows Calculator window. Binary size, working set, private working
set and private bytes are reported as different quantities and must not be
treated as interchangeable. Measurements are machine-specific observations,
not framework guarantees.
