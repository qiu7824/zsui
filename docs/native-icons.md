# Platform-native icons

ZSUI components use `ZsIcon` semantic values. Components do not contain font
code points, platform object pointers or file paths. The selected desktop host
resolves each value through `IconService` and keeps platform-specific lookup in
the backend.

## Resolution order

| Platform | Primary source | Secondary source | Portable fallback |
| --- | --- | --- | --- |
| Windows 11 | Segoe Fluent Icons installed by Windows | Segoe MDL2 Assets | MIT Fluent System Icons SVG |
| Windows 10 | Segoe Fluent Icons when installed | Segoe MDL2 Assets installed by Windows | MIT Fluent System Icons SVG |
| macOS | SF Symbols through AppKit | - | MIT Fluent System Icons SVG |
| Linux | Current freedesktop icon theme symbolic icon | - | MIT Fluent System Icons SVG |

No Microsoft or Apple icon font is distributed with ZSUI. Windows checks the
font selected by GDI instead of assuming that a requested family exists. The
live Windows renderer uses Segoe Fluent Icons first and Segoe MDL2 Assets when
the Fluent family is unavailable.

On a macOS target, `macos-appkit` includes SF Symbol names plus the portable
fallback catalog. On a Linux target, `linux-direct` resolves freedesktop theme
names and includes the same fallback catalog; `linux-gtk` retains GTK theme
lookup for the compatibility backend. The `fluent-icons` feature can enable the SVG
catalog explicitly on any target. This target-aware gating avoids putting the
fallback assets into an ordinary Windows build.

```toml
[dependencies]
zsui = { git = "https://github.com/qiu7824/zsui", default-features = false, features = [
    "window",
    "button",
    "fluent-icons",
] }
```

Framework and backend code can inspect ordered candidates or use an
availability callback:

```rust
use zsui::{native_icon_candidates, resolve_native_icon, PlatformName, ZsIcon};

let candidates = native_icon_candidates(&PlatformName::Macos, ZsIcon::Save);
assert_eq!(candidates[0].identifier, "square.and.arrow.down");

let source = resolve_native_icon(&PlatformName::Linux, ZsIcon::Copy, &|source| {
    source.identifier == "edit-copy-symbolic"
})?;
# Ok::<(), zsui::ZsuiError>(())
```

Built-in controls also use semantic values. For example, ComboBox requests
`ZsIcon::ChevronDown`, which resolves to Segoe Fluent/MDL2, `chevron.down`, or
`pan-down-symbolic` before using the MIT SVG fallback.

InfoBar uses the semantic `Info`, `Success`, `Warning` and `Error` values rather
than embedding status glyphs in the component. They resolve to the documented
Segoe Fluent Icons code points on Windows, `info.circle`, `checkmark.circle`,
`exclamationmark.triangle` and `exclamationmark.circle` SF Symbols on macOS,
the corresponding GTK `dialog-*`/`emblem-ok-symbolic` names on Linux, or four
selected MIT Fluent System Icons SVG fallbacks.

## Runtime status

- Windows: font detection and semantic glyph drawing are connected to the GDI
  renderer.
- macOS: SF Symbol names and safe resolver contracts are complete; the AppKit
  `NSImage` lookup remains part of the unfinished AppKit host.
- Linux: freedesktop symbolic names and runtime theme-file lookup are connected
  in `linux-direct`; target visual proof remains required.

The capability report therefore marks Windows native icons as supported and
macOS/Linux native icons as partial. A name catalog is not runtime proof.

## Licensing

The portable SVG files are a selected subset of Microsoft Fluent UI System
Icons and are licensed under MIT. See
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) and the preserved
upstream license files under `third_party/fluentui-system-icons/`.

SF Symbols and GTK theme icons are requested from the operating system at
runtime and are not redistributed by this repository. SF Symbols must remain
on Apple platforms.
