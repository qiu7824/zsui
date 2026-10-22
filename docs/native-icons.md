# Platform-native icons

ZSUI components use `ZsIcon` semantic values. Components do not contain font
code points, platform object pointers or file paths. The selected desktop host
resolves each value through `IconService` and keeps platform-specific lookup in
the backend.

## Resolution order

| Platform | Primary source | Secondary source | Portable fallback |
| --- | --- | --- | --- |
| Windows 11 | Segoe Fluent Icons installed by Windows | Segoe MDL2 Assets | Built-in symbolic vectors; optional MIT SVG catalog |
| Windows 10 | Segoe Fluent Icons when installed | Segoe MDL2 Assets installed by Windows | Built-in symbolic vectors; optional MIT SVG catalog |
| macOS | SF Symbols through AppKit | - | Built-in symbolic vectors; optional MIT SVG catalog |
| Linux `linux-direct` | Built-in Cairo symbolic vectors | Optional freedesktop theme through `linux-system-icons` | Built-in symbolic vectors; optional MIT SVG catalog |
| Linux `linux-gtk` | Current GTK icon theme symbolic icon | - | Built-in symbolic vectors; optional MIT SVG catalog |

No Microsoft or Apple icon font is distributed with ZSUI. Windows checks both the selected GDI font and the requested glyph instead of
assuming that a font family contains every semantic symbol. The
live Windows renderer uses Segoe Fluent Icons first and Segoe MDL2 Assets when
the Fluent family or glyph is unavailable; complete symbolic vectors cover
missing native glyphs without a replacement-character box. The default Win32 Rust text compositor
rasterizes semantic icon glyphs with grayscale coverage, independently from
subpixel UI text, so 16–20 epx icons do not inherit colored ClearType fringes.

On a macOS target, `macos-appkit` includes SF Symbol names plus the portable
fallback catalog. On Linux, the lightweight `linux-direct` profile draws a
complete 16-unit symbolic vector set directly with Cairo. Applications that
require the exact active desktop icon theme opt into `linux-system-icons`,
which adds freedesktop lookup and GdkPixbuf decoding. `linux-gtk` retains GTK
theme lookup for the compatibility backend. The `fluent-icons` feature can
enable the SVG catalog explicitly on any target. This gating keeps SVG loaders
and their dynamic libraries out of an ordinary lightweight Linux process.

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
Component profiles keep the interactive slot separate from the visible glyph:
Windows TreeView uses an 8-DP disclosure glyph, while ComboBox and Accordion
use 12-DP trailing chevrons. Accordion uses down/up states in a dedicated
expander header instead of inheriting command-bar icon placement.

Navigation shells use the same typed contract, so application code does not
select a platform glyph or maintain a parallel string icon catalog:

```rust
use zsui::{ZsIcon, ZsNavItemSpec};

let rename = ZsNavItemSpec::new("rename", "Rename")
    .semantic_icon(ZsIcon::Edit);
```

InfoBar uses the semantic `Info`, `Success`, `Warning` and `Error` values rather
than embedding status glyphs in the component. They resolve to the documented
Segoe Fluent Icons code points on Windows, `info.circle`, `checkmark.circle`,
`exclamationmark.triangle` and `exclamationmark.circle` SF Symbols on macOS,
the corresponding GTK `dialog-*`/`emblem-ok-symbolic` names on Linux, or four
selected MIT Fluent System Icons SVG fallbacks.

## Metrics and rendering

`ZsIconRole` selects small/standard/large, navigation, toolbar, status, disclosure
and tree metrics from the current platform profile. `NativeDrawIconCommand::in_slot`
centers the glyph in its independent layout slot. Aspect fitting never stretches
an icon to fill a non-square slot, and role changes do not move its hit target.
View builders support `icon_role`; UI documents accept `icon_role` or `size`,
with mutually exclusive validation. The stable facade exposes `Icon`, `IconRole`,
`IconSize`, `icon`, `icon_button` and `toolbar_button` without native font details.

Windows composites original RGBA icons with premultiplied alpha. AppKit requests
SF Symbols at the resolved point size and applies semantic tint in an isolated
compositing layer. GTK checks theme availability, loads at physical DPI and
applies symbolic alpha tint. The Linux Direct cache includes the resolved RGBA
color, invalidates on theme changes and is limited to 4 MiB and 512-pixel rasters.
All native renderers have the complete built-in vector fallback.

## Runtime evidence

The Windows fallback paint test checks all 45 semantic icons for nonempty GDI+
pixels; the native layout example exercises light, dark and high-contrast
rendering, including differently tinted instances of one icon. Native target
captures cover all 45 icons in light, dark and high-contrast scenes on
[Win32, AppKit, Linux Direct, Direct Lite and GTK](https://github.com/qiu7824/zsui/actions/runs/39750838016)
at `45f2c6b0d34bcff2d362119a5a26fb4666a11ac5`. AppKit and GTK paint the
framework background behind semantic glyphs, and every scene rejects blank RGB
captures, including transparent glyph-only images. These fixed scenes establish
target rendering evidence; they do not replace manual optical review at every
OS font setting or display scale.

## Licensing

The portable SVG files are a selected subset of Microsoft Fluent UI System
Icons and are licensed under MIT. See
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) and the preserved
upstream license files under `third_party/fluentui-system-icons/`.

SF Symbols and GTK theme icons are requested from the operating system at
runtime and are not redistributed by this repository. SF Symbols must remain
on Apple platforms.
