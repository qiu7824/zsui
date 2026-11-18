# Application appearance and retained updates

ZSUI's native profile is the default appearance. Application code can change
palette, typography, density, tab treatment and state styles without replacing
standard control behavior with custom drawing.

## Appearance priority

`AppTheme::new(Appearance)` applies to a subtree. Nested themes inherit missing
properties from their parent. `Element::appearance` / `ViewNode::appearance`
override one element. Explicit instance layout dimensions and padding continue
to determine layout; theme defaults fill missing layout values.

`StateStyles` resolves normal, selected, hover, pressed, keyboard-visible focus,
invalid and disabled rules in that order. Disabled controls do not acquire
native pointer/focus state. Application state can supply `visual_state` and
`invalid`; these modify visuals, while enablement, actions and semantic state
remain owned by the existing control APIs.

Semantic palette overrides retain their role alongside the application color.
The native high-contrast palette takes precedence. Text, icon and vector tint
overrides follow the same high-contrast rule.

```rust,no_run
use zsui::prelude::*;
let theme = AppTheme::new(
    Appearance::default()
        .color(ColorRole::Accent, Color::rgb(120, 65, 185))
        .font(FontSpec::family("Microsoft YaHei UI")
            .fallback("Noto Sans CJK SC")
            .fallback("Segoe UI Emoji")
            .size(Dp::new(14.0))
            .line_height(Dp::new(22.0)))
        .tabs(TabAppearance::Pill)
        .focus(FocusAppearance::Outline)
);
```

`TabAppearance` supports the platform preset, underline, filled, pill and
segmented treatments. `FocusAppearance` supports the platform preset, outline,
underline or no focus chrome; it does not change focus routing. Applications
that remove focus chrome should provide an equivalent visible focus state.
`ControlDensity` changes default logical control extents. Explicit View dimensions
retain precedence.

## Typography

`FontSpec` declares an ordered family list, logical size, line height and weight.
An empty family list retains semantic `TextRole` family selection. Explicit
families retain the host's system fallback after the declared list. The Rust
shaper selects a declared face that supports each grapheme, keeps adjacent spans
together for shaping and includes the entire family list in layout cache keys.
Pango and AppKit use native family/cascade descriptors.

The same font declaration reaches native measurement, retained painting and the
text input shaper. Text measurement cache identity includes the font declaration.
Editable line geometry uses the resolved size and line height for caret,
selection, viewport and IME positioning. Placeholder text inherits the same
typography but remains outside the value and undo journal.
Line height is clamped to the resolved font size in layout, paint and input
geometry. Compact density reduces surrounding chrome without shrinking the
resolved text line.

No-wrap ellipsis is performed by the Rust text engine. It shapes the ellipsis
with the same family, weight, scale and raster policy, selects grapheme-safe
prefixes and maps visible clusters back to the original text. Overflow alone
does not switch the Windows paint path to another text renderer. Explicit native
failure fallbacks remain available.

## Icons

The semantic catalog includes display, window, camera, microphone, volume,
playback, recording, message and account symbols. Applications should select the
matching semantic identity instead of substituting an unrelated icon.

The optional `custom-icons` feature provides `IconAsset`, `IconAssetId` and
`IconSource`. `from_svg`, `from_svg_file` and `from_path` return typed errors.
Resources must include a license or ownership declaration. SVG input is limited
to self-contained vector geometry; text, embedded images, scripting, animation,
filters and external references are rejected. Parsing occurs once per resource.
Raster caches are local to the resource and bounded to 16 entries and 1 MiB.

Use `icon_source` on a standard icon or button. `MenuSpec::command_icon` associates
resources with typed commands in framework-rendered menu flyouts. Native system
menu surfaces retain their platform menu contract. `glyph_size` changes the
logical em/viewbox extent without changing the hit target. Compound controls
constrain artwork to its content slot; icon buttons and leading button icons
reserve label space using the resolved artwork size. Optical ink bounds
are measured separately: equal allocation sizes do not imply identical visible
contours, internal whitespace or stroke weight.

## Retained subtrees

`memoized(id, revision, build)` makes subtree construction conditional on a strong
identity and an explicit revision. All captured data and callback dependencies
must participate in that revision. `UpdateContext::invalidate_subtree` forces a
rebuild independently of the revision.

Unchanged subtrees retain their existing controls and layout output. Bounds, DPI,
typography, style and relevant native text measurements invalidate cached layout.
Measurements for unrelated text do not invalidate a subtree. Normal View
construction remains the unconditional fallback. `LiveViewUpdate::work` reports
rebuilding and layout work; it is distinct from native frame time and process
memory.

Live updates retain node bounds directly. Explicit layout queries return a
flattened geometry snapshot. Memoized subtrees reuse their geometry and scroll
state across ordinary rebuilds and check the retained scene when native text
measurements change.

Tooltip and explicit accessibility metadata are allocated only when used.
Memory comparisons include strings, child vectors, memo state and optional
metadata, rather than reporting only the inline node size.

## Damage regions

Damage calculation compares ordered visible commands and their effective clips.
Insertions, deletions and changed middle spans invalidate their previous and next
footprints. The native buffered renderer replays intersecting content in scene
order, preserving overlap and alpha composition. Theme/typography changes,
native surface changes, malformed clips and exceptionally large command streams
retain a full repaint fallback. Identical plans take an allocation-free fast path.

## Undo and redo

Editable plain text uses a grapheme-span delta journal, bounded to 100 transactions
and 1 MiB of journal data. One current-value mirror detects external controlled
replacements exactly; its size is proportional to the current document and is
reported separately from journal growth. Oversized replacements clear older
history instead of retaining an unbounded snapshot.

Each native input transaction, including an IME commit or paste, is one undo
unit; unrelated transactions are not time-merged. Undo and redo restore both
content and selection. A new edit clears redo, and an external replacement or
widget change clears incompatible history. Primary+Y and Primary+Shift+Z request
redo. Sensitive inputs retain no undo/redo history and do not expose copy/cut.

## Examples and verification

`appearance_styles` uses standard stable controls for three appearances sharing
one update function. It is an interactive example intended for a user to launch.

`appearance_offscreen_proof <absolute-output-directory>` uses memory DCs and
creates no UI window. It writes control images and icon grids at 16, 18, 20, 24
and 32 DP and 100%, 125%, 150% and 200% scale. Reports identify the actual icon
source, allocation, ink extent and centroid. The existing text geometry proof
compares fixed fonts, sizes, weights and scales against DirectWrite. Pixel
differences and geometric/font-selection differences remain separate evidence.

Model and offscreen tests cover theme priority, control states, focus semantics,
font-aware measurement, memo invalidation, delta history and damage bounds.
Native interactive verification remains a separate target evidence layer.

### Allocation and work measurements

The Windows x64 measurement uses Rust 1.99.0, `full`, and the test profile with
debug information disabled. The comparison keeps identical payload types,
strings and child-vector capacities; the baseline retains tooltip and
accessibility fields inline. Ordinary nodes occupy 736 inline bytes versus
872 in that baseline representation.

| Scene | Nodes | Baseline owned bytes | Current owned bytes |
| --- | ---: | ---: | ---: |
| Labels | 1,001 | 894,000 | 758,000 |
| Buttons | 1,001 | 886,000 | 750,000 |
| Mixed form with sparse tooltip/accessibility metadata | 801 | 878,220 | 746,220 |

These allocation measurements include indirect storage. Baseline conversion
allocation counts are recorded separately from current construction counts.
The offscreen executable reports process memory independently.

The nested memo regression rebuilds four of six nodes and reuses two layout
nodes when one nested subtree is invalidated; the unaffected sibling keeps its
geometry. Revision changes preserve scroll offsets. The generated work report
records model update/layout time separately from native frame time.

### Windows release update workloads

Twelve alternating process pairs ran 32 warm-up updates followed by 256 timed
updates per process on Windows 11 x64. The list workload dispatches a typed
event and lays out a changing counter beside 1,000 retained labels with
explicit IDs. Common regenerates the invoice shell's draw plan. The large-text
case refreshes the multilingual page and regenerates its draw plan. The two
page cases produced byte-identical 96 DPI offscreen RGBA images across versions.

| Workload | Private Bytes (KiB), baseline → candidate | Allocations or growths per update | Median update (µs) | p95 update (µs) |
| --- | ---: | ---: | ---: | ---: |
| Memoized list | 1,130 → 992 | 356 → 18 | 945.15 → 795.65 | 1,061.55 → 888.85 |
| Common | 1,750 → 1,720 | 152 → 152 | 23.90 → 24.50 | 26.25 → 27.35 |
| Large text | 1,862 → 1,728 | 44 → 40 | 35.75 → 35.20 | 43.15 → 42.05 |

The list's live Rust allocations fell from 381,898 to 268,104 bytes. Its paired
95% interval for the median update was −170.7 to −139.8 µs. The interval for
p95 included zero, as did the interval for Common private memory. Neither page
reached the 512 KiB savings target. Native presentation, first frame and
compositor latency require a separate windowed measurement. The
[raw pairs, build identities and confidence intervals](quality/authoring-update-performance-windows.json)
record the workload-specific measurement boundaries.

The icon proof covers 4,380 cases across 73 semantic icons, five logical sizes,
four scales and three sources. The measured Windows run has no blank or
out-of-allocation ink. Font geometry remains a separate matrix: 61 of 66 cases
meet the strict geometry threshold, with the five remaining cases described in
[Text rendering](text-rendering.md#acceptance-order).
