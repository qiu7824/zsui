# ZSUI Notepad Demo

`examples/zsui_notepad.rs` is a runnable Windows text editor used to compare
ZSUI with four representative UI approaches and the Windows system Notepad.
It covers a complete basic notepad workflow; it is not a claim of product
feature parity with Windows Notepad.

## ZSUI Behavior

- Reusable `WindowsWin32OwnedTextEditor` hosting a native multiline editor with
  Windows IME and accessibility behavior, owned font/DPI updates, word-wrap,
  selection and edit commands.
- Reusable `document-shell` visuals for the document tab, command bar, rounded
  editor frame and status surface.
- Reusable `ZsTextDocument` loading, encoding metadata, dirty state and
  transactional UTF-8 save/save-as.
- Target-dispatched `NativeFileDialogService` open/save panels with owned path
  and filter specs instead of example-local common-dialog FFI.
- Buffered parent painting, Fluent semantic icons and shared design tokens.
- New, open, save, save as and dirty-document confirmation.
- Undo, cut, copy, paste and select all.
- UTF-8 save plus UTF-8 and UTF-16 input decoding.
- Word-wrap and status-bar toggles.
- Line, column, character count and encoding status.
- Shared `ZsTextCursorStatus` line/column calculation from native UTF-16 caret
  offsets, including non-BMP Unicode text.
- Typed `ZsAccelerator` bindings backed by a framework-owned RAII accelerator
  table, plus DPI awareness and an application icon.

Run ZSUI directly:

```powershell
cargo run --example zsui_notepad --features notepad-demo
```

Run its auto-closing smoke path:

```powershell
cargo run --example zsui_notepad --features notepad-demo -- --smoke
```

## Comparison Sources

The repository keeps the application-owned source, configuration and lock file
for each standalone baseline:

| Baseline | UI shape | Location |
| --- | --- | --- |
| eframe/egui 0.35 | Immediate-mode Rust UI | `comparisons/egui_notepad` |
| Iced 0.14 | Typed state, message, update and view | `comparisons/iced_notepad` |
| Slint 1.17 | Declarative Slint markup with safe Rust callbacks | `comparisons/slint_notepad` |
| Tauri 2.11 | HTML/CSS/JavaScript frontend with Rust commands | `comparisons/tauri_notepad` |

Run any baseline with its isolated manifest, for example:

```powershell
cargo run --release --locked --manifest-path comparisons\iced_notepad\Cargo.toml
cargo run --release --locked --manifest-path comparisons\slint_notepad\Cargo.toml
cargo run --release --locked --manifest-path comparisons\tauri_notepad\Cargo.toml
```

Downloaded crates, `target` directories, Tauri-generated schemas and benchmark
reports are deliberately excluded from Git. This keeps the demos reproducible
without committing their support libraries or build output.

## Reproducible Measurement

The script builds all five release applications into a support directory
outside the repository, samples each complete process tree, captures the real
windows and writes JSON and Markdown reports:

```powershell
.\scripts\measure-notepad-comparison.ps1
```

The default support directory is `..\zsui-ui-benchmark-support`. It can be
changed without placing generated files in the repository:

```powershell
.\scripts\measure-notepad-comparison.ps1 `
    -SupportRoot E:\rust\zsui-ui-benchmarks\matrix `
    -PublishGallery
```

A reference run on Windows 11 with Rust 1.94, a five-second warmup and six
steady-state samples produced the runtime figures below. Source-file and
nonblank-line counts are recomputed from the current checked-in applications:

| Implementation | Processes | App files | Nonblank app lines | Resolved packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI Notepad | 1 | 2 | 732 | 31 | 0.27 MiB | 1.84 MiB | 15.80 MiB | 2.50 MiB |
| eframe/egui baseline | 1 | 2 | 344 | 295 | 5.67 MiB | 43.47 MiB | 73.03 MiB | 66.85 MiB |
| Iced baseline | 1 | 2 | 259 | 347 | 4.07 MiB | 5.50 MiB | 19.44 MiB | 7.17 MiB |
| Slint baseline | 1 | 2 | 328 | 579 | 9.66 MiB | 5.04 MiB | 22.43 MiB | 5.90 MiB |
| Tauri 2 baseline | 7 | 8 | 411 | 427 | 2.65 MiB* | 80.57 MiB | 356.31 MiB | 164.64 MiB |
| Windows Notepad | 1 | system app | system app | n/a | 3.15 MiB* | 37.65 MiB | 111.36 MiB | 98.12 MiB |

`*` The Tauri executable excludes the system WebView2 runtime. The packaged
Windows Notepad executable excludes the rest of its package. Those binary-size
numbers are not directly comparable to standalone executables.

"Task Manager memory" is the summed private working set of the root process and
all recursive child processes. Tauri therefore includes its six WebView2 child
processes. Working set includes resident shared pages; private bytes is
committed private virtual memory. App line counts include nonblank demo-owned
source and configuration lines; generated files are excluded. Cargo package
counts are resolved dependency graph nodes and can include target-specific
packages. These are repeatable
single-machine observations, not universal constants.

## Functional Scope

The baselines intentionally share the editing core, but are not exact product
clones. This matrix prevents source-line comparisons from implying false
feature parity:

| Capability | ZSUI | egui | Iced | Slint | Tauri 2 |
| --- | :---: | :---: | :---: | :---: | :---: |
| Multiline edit, new/open/save/save as | yes | yes | yes | yes | yes |
| Word wrap and status information | yes | yes | yes | yes | yes |
| UTF-8/UTF-16 file input | yes | yes | yes | yes | yes |
| Application icon and automatic benchmark close | yes | yes | yes | yes | yes |
| Primary UI layer | Rust shell + Win32 service | Rust immediate mode | Rust typed update/view | Slint + Rust callbacks | HTML/CSS/JS + Rust commands |
| Dirty-close confirmation | yes | yes | no | no | no |
| Native Windows text service | yes | no | no | no | WebView2 |
| Cross-platform application path | not yet | yes | yes | yes | yes |

## Interpretation

ZSUI has the smallest executable and lowest measured idle memory in this
native-service sample, but currently needs the most application code. Text-file
decoding, dirty state and transactional save now live in reusable
`ZsTextDocument`, while open/save selection uses the shared
`NativeFileDialogService` and keyboard shortcuts use typed framework
accelerators with framework-owned native resources. The Win32 multiline editor,
font, DPI, bounds, text, selection, word-wrap and edit commands now use one
framework-owned RAII service. Parent-window modality glue and the application
dirty-close policy still live in the example.

Iced is the shortest baseline and remains close to native-process memory. Slint
also has low measured memory, with the largest binary and resolved dependency
graph in this configuration. egui is concise but uses more idle memory here.
Tauri has a modest application executable, while its complete WebView2 process
tree has the highest runtime footprint. Its frontend source can still be the
fastest route for teams already productive with web UI.

Windows Notepad remains ahead in editor maturity, including tabs, session
restore, search and replace, print, spell checking, richer encoding choices and
system integration. ZSUI's useful current advantage is a small Rust-controlled
native stack, not feature superiority over the Windows product.

The next code-reduction target is parent-window dialog binding and a shared
AppKit/GTK document-editor path. Dirty-close confirmation remains application
policy. This should further reduce application and AI-generated code without
giving up the measured native-host characteristics.
