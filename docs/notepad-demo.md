# ZSUI Notepad Demo

`examples/zsui_notepad.rs` is a runnable Windows text editor used to test ZSUI
against a common Rust UI stack and the Windows system Notepad. It is a complete
basic notepad workflow, not a claim of feature parity with the system app.

## Included Behavior

- Native multiline text editing with IME and platform accessibility behavior
  inherited from the Windows edit service.
- Reusable `document-shell` visuals for a document tab, command bar, rounded
  editor frame and status surface.
- Buffered no-flicker parent painting with semantic Fluent icons and shared
  typography, color, spacing and radius tokens.
- New, open, save, save as and dirty-document confirmation.
- Undo, cut, copy, paste and select all.
- UTF-8 save plus UTF-8 and UTF-16 input decoding.
- Word-wrap and status-bar toggles.
- Line, column, character count and encoding status.
- Keyboard accelerators, DPI awareness and an application icon.
- ZSUI light-theme text and surface tokens.

Run the application:

```powershell
cargo run --example zsui_notepad --features notepad-demo
```

Run its auto-closing smoke path:

```powershell
cargo run --example zsui_notepad --features notepad-demo -- --smoke
```

## Reproducible Comparison

The isolated `comparisons/egui_notepad` crate implements the same basic
workflow with `eframe`, `egui` and `rfd`. The measurement script builds release
binaries, samples process memory, captures all three windows and writes JSON
and Markdown reports under `target/notepad-comparison`.

```powershell
.\scripts\measure-notepad-comparison.ps1
```

A reference run on Windows 11 with Rust 1.94 produced:

| Implementation | App files | App lines | Packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI Notepad | 3 | 937 | 31 | 0.26 MiB | 1.84 MiB | 15.85 MiB | 2.51 MiB |
| eframe/egui baseline | 2 | 344 | 295 | 5.67 MiB | 43.61 MiB | 73.24 MiB | 68.36 MiB |
| Windows Notepad | system app | system app | n/a | 3.15 MiB* | 37.89 MiB | 111.67 MiB | 98.38 MiB |

`*` The packaged system executable does not represent the complete Windows
Notepad package, so that binary-size number is not directly comparable.
Memory values are steady-state samples from one machine and must be treated as
measurements, not universal constants. The script waits five seconds before
sampling. "Task Manager memory" is the private working set used by the
Processes-page memory column. Total working set includes resident shared pages,
while private bytes measures committed private virtual memory; these counters
must not be compared as if they were the same quantity.

## What The Result Means

ZSUI already has a concrete advantage in dependency count, binary size and
idle overhead for this native-service application. The modern self-drawn shell
uses about 1.84 MiB by the Task Manager private-working-set metric and keeps the
total working set near 16 MiB while replacing the classic menu and inset editor
presentation. It also keeps the document model in safe Rust and isolates
platform calls in one module.

The current API is not yet the fastest choice for AI-generated generic desktop
utilities. The ZSUI example contains 937 app-level lines because native editor,
file-dialog, accelerator and dirty-document lifecycle code still lives in the
example. The equivalent egui implementation is 344 lines and is cross-platform.
Today, egui is easier for an AI to produce quickly; ZSUI is more attractive when
native behavior, small output and framework-controlled product composites are
the priority.

Windows Notepad remains ahead in editing maturity: tabs, session restore,
search and replace, print, spell checking, rich encoding and line-ending
choices, accessibility validation and operating-system integration. ZSUI has
no general-purpose feature advantage over it yet. The useful advantage is that
an application can own a much smaller, Rust-controlled UI stack and customize
it without adopting the system Notepad product model.

`ZsDocumentShellSpec` is now the reusable visual boundary. It owns DPI-aware
layout, semantic draw commands, compact command sizing, interaction regions and
selected/hovered/pressed states. It does not own document persistence or raw
Windows handles. Keyboard accelerators remain available, but custom command-bar
focus navigation, UI Automation exposure, dark/high-contrast proof and
non-Windows bindings remain explicit gaps. The next reduction target is a reusable text-editor service,
file-dialog service, accelerator binding and document lifecycle controller.
Those APIs should move the remaining platform plumbing out of application
examples while preserving the measured native host characteristics.
