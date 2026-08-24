# Windows Runtime and Capability Baseline

Baseline date: 2026-08-09. The reference source is
`kusutori/DataViewer@d6af795331ff5012e6273ca17461e9696fa51461`.
Both applications were built in Release mode and observed on the same Windows
x64 host.

## Runtime observation

Each row is the median of three runs. Startup is measured from process creation
until the first nonzero top-level `MainWindowHandle`. Runtime counters are
sampled three seconds later and summed over the process tree. CPU is cumulative
processor time at the sample point, not steady-state CPU utilization.

| State | Visible handle | Processes | Working set | Private bytes | CPU time | Threads | Handles |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Reference, empty | 454.2 ms | 1 | 132.77 MiB | 88.84 MiB | 703.1 ms | 54 | 1,038 |
| ZSUI, empty | 48.1 ms | 1 | 20.29 MiB | 3.69 MiB | 93.8 ms | 10 | 256 |
| ZSUI, CSV loaded | 78.0 ms | 1 | 29.05 MiB | 5.52 MiB | 234.4 ms | 10 | 256 |

At the matched empty state, the observed ZSUI window handle appeared 9.4×
sooner. Its process used 6.5× less working-set memory, 24.1× less private
memory, 5.4× fewer threads and 4.1× fewer handles. Handle visibility is not a
substitute for first-content-frame or interaction-ready instrumentation.

## Distribution size

| Package | Files | Size | Runtime prerequisite |
| --- | ---: | ---: | --- |
| ZSUI Release | 2 | 36.45 MiB | Windows system components |
| Reference framework-dependent publish | 331 | 177.38 MiB | .NET 10 Desktop Runtime |
| Reference fully self-contained publish | 518 | 253.85 MiB | Windows system components |

The ZSUI package consists of the stripped executable and the official DuckDB
native library. The reference project sets `WindowsAppSDKSelfContained=true`,
which bundles Windows App SDK components; a fully self-contained .NET publish
still requires `dotnet publish --self-contained true`.

## Build and dependency observations

| Dimension | Reference | ZSUI |
| --- | ---: | ---: |
| Direct application dependencies | 4 NuGet packages | 4 Cargo dependencies |
| Resolved Windows runtime packages | 24 NuGet libraries | 112 Cargo packages |
| Warm Release build/publish median | 2,014.7 ms | 438.9 ms |
| Executable tests in application source | 0 | 9 |

Dependency counts are ecosystem-specific and are not a size or quality score:
Rust statically links much of its dependency graph, while .NET carries a larger
runtime and app-local framework directory.

## Capability comparison

| Capability | Reference | ZSUI implementation |
| --- | --- | --- |
| Data formats | CSV, TSV, Parquet, JSON/JSONL/NDJSON | Same |
| Query engine | DuckDB, relation alias `dataset`, 200-row preview | Same |
| File loading/query execution | Synchronous after the picker returns | Background worker with generation-safe result application |
| SQL editor | WebView2 + CodeMirror highlighting, completion, search and lint | Native multiline editor with Unicode/IME, selection, undo and scrolling |
| Offline editor | CodeMirror modules fetched from `esm.sh` at runtime | No web runtime or network dependency |
| Results table | Sort, per-column filter, resize, reorder, extended selection and export UI | Sort, global filter, selection, copy, CSV export and vertical scrolling |
| Theme | System/light/dark plus editor-specific theme/font settings | System/light/dark |
| Desktop targets | Windows 10/11 WinUI 3 | Win32, macOS AppKit and Linux Direct from one state/message/view path |
| Accessibility evidence | WinUI platform defaults; no repository proof artifact | Win32 UIA report with 9 nodes and 8 actions |
| Automated target matrix | No repository workflow | Windows 2025, macOS 15 and Ubuntu 24.04 build/test/window-capture jobs |

The reference currently leads in SQL editing and advanced table manipulation.
ZSUI leads in startup footprint, package size, offline operation, asynchronous
data work, target coverage and executable verification. Source-size results are
recorded separately in [`CODE_SIZE.md`](CODE_SIZE.md).

## Visual evidence status

- ZSUI loaded-state Win32 image and native report are available under
  `artifacts/windows-local`.
- Reference target capture and aligned pixel composites remain pending.
