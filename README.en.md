<div align="center">

# ZSUI

**A native desktop UI framework for Rust**

Build Windows, macOS and Linux applications from one Rust state, message and view model.

[![crates.io](https://img.shields.io/crates/v/zsui?include_prereleases)](https://crates.io/crates/zsui)
[![Docs](https://docs.rs/zsui/badge.svg)](https://docs.rs/zsui)
[![CI](https://github.com/qiu7824/zsui/actions/workflows/ci.yml/badge.svg)](https://github.com/qiu7824/zsui/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/qiu7824/zsui)](https://github.com/qiu7824/zsui/blob/main/LICENSE)

[简体中文](https://github.com/qiu7824/zsui/blob/main/README.md) · [English](https://github.com/qiu7824/zsui/blob/main/README.en.md)

</div>

ZSUI uses real system windows and platform services, with framework-owned controls drawn
inside each window. Applications update through `State → view → Msg → update`; backends
handle window events, text, rendering, IME and system integration.
The current version, **`0.3.0-alpha.2`**, is a prerelease.

[Quick start](#quick-start) · [Cargo features](#cargo-features) · [Platforms](#platforms-and-scope) · [Examples](#examples) · [Documentation](#documentation)

## Overview

- **One application codebase**: shared state, messages, layout and component declarations across Win32, AppKit and Linux hosts.
- **Cargo feature selection**: opt into controls, services and backends. Defaults are `window`, `button` and `label`.
- **Explicit state and identity**: typed messages, strong IDs, controlled input and stable component identity.
- **Platform appearance**: semantic themes, system fonts, icons, DPI and platform component profiles shape the final UI.
- **Two authoring paths**: Rust Views or validated, reloadable and embeddable UI documents.
- **Native runtime evidence**: CI runs target applications and retains final-window captures plus structured layout, input and focus reports.

The catalog contains **47 component families with runtime implementations**.
Interaction, accessibility and platform coverage vary by component; consult the
[component catalog](https://github.com/qiu7824/zsui/blob/main/src/component_catalog.rs) for details.

## Screenshots

Gallery windows captured on three desktop platforms:

<table>
  <tr><th>Windows · Win32</th><th>macOS · AppKit</th><th>Linux · X11</th></tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/component-gallery-preview7-windows.png" alt="Windows Win32 component gallery" width="320"></td>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/platform-proof/macos/gallery-inputs-light.png" alt="macOS AppKit component gallery" width="320"></td>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/platform-proof/linux/gallery-inputs-light.png" alt="Linux X11 component gallery" width="320"></td>
  </tr>
</table>

These captures show platform layout and appearance at different window sizes.
Pixel comparisons use fixed-scene proof reports. See the
[gallery](https://github.com/qiu7824/zsui/blob/main/docs/gallery.md)
for Notepad, Calculator, components and framework comparisons.

## Quick start

Add ZSUI to a Rust binary project's `Cargo.toml`:

```toml
[dependencies]
zsui = "0.3.0-alpha.2"
```

Save this as `src/main.rs`:

```rust,no_run
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use zsui::prelude::{button, column, text, window, Dp, Element, UpdateContext};

#[derive(Clone)]
enum Message {
    Increment,
}

struct State {
    count: u32,
}

fn view(state: &State) -> Element<Message> {
    column([
        text(format!("Count: {}", state.count)),
        button("Increment").on_click(Message::Increment),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(20.0))
}

fn update(state: &mut State, message: Message, _cx: &mut UpdateContext<'_>) {
    match message {
        Message::Increment => state.count += 1,
    }
}

fn main() -> Result<(), zsui::stable::Error> {
    window("ZSUI")
        .size(480, 320)
        .stateful(State { count: 0 }, view, update)
        .run()
}
```

```sh
cargo run
```

A button emits a `Message`, `update` changes application state, and the runtime rebuilds
and repaints the view. Application code imports from `zsui::prelude` or `zsui::stable`.
See the [API compatibility policy](https://github.com/qiu7824/zsui/blob/main/docs/api-stability.md)
for the supported compatibility boundary.

### Build prerequisites

Use a current stable Rust toolchain and the native development tools for your target:

| Platform | Requirements |
| --- | --- |
| Windows | MSVC toolchain, Visual Studio C++ Build Tools and Windows SDK |
| macOS | Xcode Command Line Tools |
| Default Linux backend | C/C++ build tools, pkg-config, D-Bus, Cairo and Pango development packages |
| Linux GTK4 backend | GTK4 4.10 or newer development packages in the corresponding build environment |

For the default backend on Ubuntu 24.04:

```sh
sudo apt-get install build-essential pkg-config libdbus-1-dev libcairo2-dev libpango1.0-dev
```

Add `libgtk-4-dev` when selecting `linux-gtk`. Install equivalent packages on other
distributions. Backend selection and porting constraints are covered in
[porting](https://github.com/qiu7824/zsui/blob/main/docs/porting.md).

## Cargo features

The default configuration provides windows, buttons and text. Enable additional
features for the controls and services your application uses:

```toml
[dependencies]
zsui = { version = "0.3.0-alpha.2", default-features = false, features = [
    "window", "button", "label", "textbox", "clipboard", "dialog",
] }
```

| Purpose | Main features |
| --- | --- |
| Window hosts and backends | `window`, `windows-win32`, `macos-appkit`, `linux-direct`, `linux-direct-lite`, `linux-gtk` |
| Basic UI and layout | `button`, `label`, `icon`, `grid`, `scroll`, `split-view`, `resize` |
| Text and numeric input | `textbox`, `password-box`, `number-box`, `auto-suggest`, `command-palette` |
| Selection and dates | `checkbox`, `toggle`, `toggle-button`, `radio`, `slider`, `combo`, `date-picker`, `time-picker`, `color-picker` |
| Collections | `list`, `virtual-list`, `paged-list`, `grid-view`, `tree`, `table` |
| Overlays and feedback | `dialog`, `flyout`, `menu-flyout`, `context-menu`, `tooltip`, `toast`, `info-bar`, `teaching-tip`, `progress`, `progress-ring` |
| Compositions | `shell`, `document-shell`, `tabs`, `accordion`, `breadcrumb`, `badge`, `calculator` |
| Media | `image`, `image-preview`, `video` |
| System services and themes | `clipboard`, `tray`, `hotkey`, `settings`, `style`, `dark-mode`, `localization` |
| Accessibility | `accessibility`, `linux-direct-accessibility` |
| UI documents and tools | `ui-document`, `ui-document-runtime`, `ui-viewer` |
| Aggregate configurations | `all-widgets`, `full` |

`window` selects the default desktop host through target-specific dependencies.
Linux defaults to `linux-direct`. To choose GTK4 or `linux-direct-lite`, disable
default features and select the backend explicitly, so the default Linux renderer
is not also enabled. `linux-direct-lite` uses tiny-skia and ZSUI's Rust text engine.

`accessibility` supplies shared semantics and Win32, AppKit and GTK4 bridges.
Linux Direct's AccessKit/AT-SPI bridge additionally requires `linux-direct-accessibility`.
Cargo features are additive across the dependency graph. `full` enables a broad set of
controls, backends and proof capabilities for framework validation; applications should
select the features they need. ZSUI currently ships as one crate.

## Platforms and scope

| Platform | Native host and rendering | System integration |
| --- | --- | --- |
| Windows | Win32 windows, buffered GDI/GDI+ rendering, Rust text by default | IME, UI Automation, clipboard, file panels, menus, tray |
| macOS | NSApplication/NSWindow, AppKit/Core Graphics, Core Text | NSTextInputClient, NSAccessibility, clipboard, file panels, menus, NSStatusItem |
| Linux | Wayland/X11 + Softbuffer + Cairo/Pango by default; optional tiny-skia or GTK4 host | IME, clipboard, XDG portal, menus; AT-SPI through the selected accessibility bridge |
| Android | Host, lifecycle and FFI contracts | Activity runtime implementation and device verification remain pending |

ZSUI draws its controls inside native windows. Hosts provide system windows, event loops
and desktop services, while platform implementations own component layout and visual
profiles. Service availability depends on the backend and desktop environment; consult
capability reports and operation results. The Linux Direct tray host is not yet connected
to the unified event loop.

Applications use semantic icons. Windows resolves system icon fonts and macOS uses
SF Symbols. Linux uses built-in symbolic graphics by default; enable `linux-system-icons`
for exact desktop icon-theme decoding. See
[native icons](https://github.com/qiu7824/zsui/blob/main/docs/native-icons.md)
for SVG fallback and licensing.

IME protocols, caret geometry and accessibility bridges have automated checks.
Real candidate-window placement and Narrator/NVDA, VoiceOver and Orca speech/focus
experience also require manual checks on the target environment. Touch and inertial
scrolling, complex rich text, editing/virtualization in some collection controls and the
Android runtime remain areas of development. Runtime implementation, target proof and
complete user experience are tracked separately.

## State, background work and resources

Applications own business state and update the UI through typed messages. Ordinary
controls can use automatically assigned structural identities. Reordered collections,
cross-component references and externally targeted actions should use explicit stable IDs.

Background work requests a rebuild through the window's `InvalidationHandle`.
Requests coalesce and the native UI thread rebuilds the stateful view. File operations,
asynchronous tasks and native services are documented in
[application effects and commands](https://github.com/qiu7824/zsui/blob/main/docs/app-effects.md).

The text engine bounds layout and glyph caches by entry count and byte size, and
periodically ages shaped runs. These limits cover the corresponding framework caches;
application data, system fonts and platform resources have their own lifetimes.
`release_view_when_hidden()` can release hidden-window UI resources. Hosts such as
Wayland restrict programmatic visibility, restoration and focus requests. See
[text rendering](https://github.com/qiu7824/zsui/blob/main/docs/text-rendering.md) and
[desktop runtime constraints](https://github.com/qiu7824/zsui/blob/main/docs/desktop-hardening-plan.md).

## UI documents and native Viewer

`ui-document` defines versioned semantic component trees and typed binding manifests.
Rust owns state and messages; documents describe components, layout, themes,
localization keys, accessibility and stable node IDs.

- `zsui-uic check` validates schemas, features, properties, node identity and binding types.
- `zsui-viewer` reloads valid documents in a native window and preserves compatible focus, selection, scrolling and control state.
- `zsui-uic embed` and `ui-document-runtime` embed validated documents in release applications, pruned to the components they use.

Run the validator or Viewer from the repository root:

```sh
cargo run --locked --bin zsui-uic --no-default-features --features ui-document,button,label -- check examples/ui-documents/basic.json --bindings examples/ui-documents/basic.bindings.json
cargo run --locked --bin zsui-viewer --no-default-features --features ui-viewer -- examples/ui-documents/basic.json --bindings examples/ui-documents/basic.bindings.json --values examples/ui-documents/basic.values.json
```

Once the Viewer is compiled, saving a compatible document edit updates the same window
without invoking Cargo again. Release applications use `ui-document-runtime`; polling,
development diagnostics and the Viewer-wide component configuration belong to the
separate tool. See
[UI Document](https://github.com/qiu7824/zsui/blob/main/docs/ui-document.md)
for the format, bindings, secure password channel and release embedding.

## Examples

Clone the repository and run from its root:

```sh
cargo run --locked --example stable_api
cargo run --locked --example component_gallery --no-default-features --features component-gallery-demo
cargo run --locked --example zsui_notepad --no-default-features --features notepad-demo
cargo run --locked --example zsui_calculator --no-default-features --features calculator-demo
```

| Example | Coverage | Guide |
| --- | --- | --- |
| `component_gallery` | Controls, themes and platform layout | [Gallery](https://github.com/qiu7824/zsui/blob/main/docs/gallery.md) |
| `zsui_notepad` | Shared three-platform Notepad, edit commands and file panels | [Notepad](https://github.com/qiu7824/zsui/blob/main/docs/notepad-demo.md) |
| `zsui_calculator` | Decimal arithmetic, memory, history and adaptive keypad | [Calculator](https://github.com/qiu7824/zsui/blob/main/docs/calculator-demo.md) |
| `resizable_layout` | Controlled NavigationView, SplitView and DataGrid extents | [Resizable layouts](https://github.com/qiu7824/zsui/blob/main/docs/resizable-layout.md) |
| `paged_virtual_list` | Background paging, prefetch, stable scroll anchors and LRU page cache | [Paged virtual list](https://github.com/qiu7824/zsui/blob/main/docs/paged-virtual-list.md) |
| `image_preview` | Background PNG decoding, coalesced requests and complete-frame swaps | [Image preview](https://github.com/qiu7824/zsui/blob/main/docs/image-preview.md) |

`video` accepts the latest application-supplied pixel frame and handles layout and
presentation. Capture, codecs, audio, seeking and subtitles belong to media adapters.
See [video and frame sources](https://github.com/qiu7824/zsui/blob/main/docs/video.md)
for the interface boundary.

## Documentation

| Topic | Guide |
| --- | --- |
| Architecture and ownership | [Architecture](https://github.com/qiu7824/zsui/blob/main/docs/architecture.md) · [Framework goals](https://github.com/qiu7824/zsui/blob/main/docs/framework-goals.md) |
| API and compatibility | [Rustdoc](https://docs.rs/zsui) · [API stability](https://github.com/qiu7824/zsui/blob/main/docs/api-stability.md) |
| Layout, themes and platform appearance | [Platform design contracts](https://github.com/qiu7824/zsui/blob/main/docs/platform-design-contracts.md) · [Resizable layouts](https://github.com/qiu7824/zsui/blob/main/docs/resizable-layout.md) |
| Localization | [Fluent and locale switching](https://github.com/qiu7824/zsui/blob/main/docs/localization.md) |
| Input and accessibility | [Input experience](https://github.com/qiu7824/zsui/blob/main/docs/v0.3-input-experience.md) · [Manual checklist](https://github.com/qiu7824/zsui/blob/main/docs/quality/manual-assistive-technology-checklist.md) |
| Native proof | [Native Proof CI](https://github.com/qiu7824/zsui/blob/main/docs/v0.3-native-proof-ci.md) · [Target smoke](https://github.com/qiu7824/zsui/blob/main/docs/native-host-smoke.md) |
| Performance | [Four-workload matrix](https://github.com/qiu7824/zsui/blob/main/docs/ui-performance-matrix.md) · [Application comparison](https://github.com/qiu7824/zsui/blob/main/docs/invoice-tool-comparison.md) |
| Distribution and updates | [Installers, signing and update manifests](https://github.com/qiu7824/zsui/blob/main/docs/distribution.md) |
| Release notes | [0.3.0-alpha.2](https://github.com/qiu7824/zsui/blob/main/docs/releases/v0.3.0-alpha.2.md) |

Performance reports identify the platform, build features, window scene and workload.
Minimal, Common, Full Native App and Viewer are measured separately; comparisons
require matching conditions.

## Development and verification

```powershell
cargo fmt --all -- --check
cargo test --locked --no-default-features
cargo test --locked --features full
cargo check --locked --manifest-path tools/native-proof/Cargo.toml
.\scripts\check-native-boundary.ps1 -Locked
.\scripts\check-feature-matrix.ps1 -Locked
```

CI also runs desktop target compilation, native interaction, final-surface captures and
ARM64 verification. Contributors should start with the
[development entry point](https://github.com/qiu7824/zsui/blob/main/docs/ai-agent.md)
and load the context and checks for the relevant module.

## Support and license

ZSUI is licensed under [MPL-2.0](https://github.com/qiu7824/zsui/blob/main/LICENSE).
Bundled Fluent System Icons SVG fallback assets use the MIT license; see
[third-party notices](https://github.com/qiu7824/zsui/blob/main/THIRD_PARTY_NOTICES.md).

Support the project with a [GitHub Star](https://github.com/qiu7824/zsui) or a donation.

<details>
<summary>Donation</summary>

![Donation](https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/donate.png)

</details>

[![GitHub star history](https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/star-history.svg)](https://github.com/qiu7824/zsui/stargazers)
