# Text rendering

ZSUI owns the text contract and its retained resources; a platform backend owns
the final native experience. Font discovery, fallback, shaping, line breaking,
layout geometry, glyph rasterization and platform presentation are separate
stages, but measure, paint, caret, selection, IME and accessibility must consume
the same retained layout result. A renderer is not considered correct because
one font or one Latin sentence looks acceptable.

## Production text mechanism

`ZsRustTextEngine` is the optional ZSUI-owned portable text context. It is not a
wrapper around a single font and it is not a proof runner embedded in an app.
One application/window renderer owns the context explicitly; there is no
process-global mutable font or widget registry.

```text
TextStyle + text + constraints + DPI
                 |
                 v
      family/weight/fallback resolution
                 |
                 v
       HarfRust shaping and line layout
                 |
                 v
       Arc<ZsRustTextLayout> geometry
          |                     |
          v                     v
 measure/caret/IME       Swash glyph raster
                                |
                                v
                  native buffered compositor
```

The production layout contains intrinsic size, overflow flags, line geometry
and compact glyph IDs, UTF-8 cluster ranges, direction, origins and advances.
Visual lines receive unique indices even when several wrapped lines originate
from one source paragraph. Exact shaper positions are retained for hit testing,
caret/selection and vector geometry; separately quantized raster-cache origins
are private implementation data and never replace the layout geometry.
The Windows metric policy derives GPOS pair corrections from the selected
OpenType face instead of font-name exceptions, preserves one baseline phase for
all visual lines of a source paragraph, and retains wrap-separating ASCII spaces
as zero-ink glyphs so caret and selection geometry still cover every source
byte. When a semibold-or-heavier request has neither a suitable family face nor
a `wght` axis, visible glyphs retain a bounded synthetic-bold strength in both
their advances and raster-cache key; whitespace and zero-advance marks are not
artificially widened.
Resolved font-family/PostScript strings, JSON schemas, SVG paths and
pixel-difference buffers are absent from retained production layouts unless the
development-only proof feature is enabled. The requested family remains part of
the layout-cache key. Paint color is deliberately excluded from that key, so
theme changes reuse geometry and only change composition.

The context retains two independently bounded LRU caches:

| Cache | Default entry limit | Default byte limit | Ownership |
| --- | ---: | ---: | --- |
| Layout | 256 | 2 MiB | `Arc<ZsRustTextLayout>` shared by measure and paint |
| Glyph image | 2,048 | 4 MiB | `Arc<Image>`; cache hits do not clone pixel buffers |

Both entry limits and retained-payload byte budgets are enforceable at runtime;
the byte counters are cache accounting, not a promise about allocator or font
database RSS. Oversized entries are not retained, eviction is
least-recently-used, clearing glyph images does not invalidate an owned layout,
and `ZsTextCacheStats` exposes hit/miss/eviction counters for framework
diagnostics. The Win32 adapter initializes this context on first measure or
draw, and `WindowsGdiRenderer::text_layout()` returns a facade backed by the
same per-window resource cache. Linux Direct Lite also owns one shared context
for menu measurement, final-surface drawing, typography metrics and text-input
caret/selection geometry; it no longer constructs a second `FontSystem`,
`Buffer` or `SwashCache` path beside the ZSUI engine.

## Optional Windows paths

| Cargo feature | Layout and raster source | Presentation |
| --- | --- | --- |
| `windows-gdi` | GDI | Existing buffered Win32 DIB |
| `windows-directwrite` | System DirectWrite | Existing buffered Win32 DIB |
| `windows-rust-text` | ZSUI Rust pipeline using HarfRust and Swash | Existing buffered Win32 DIB |
| `linux-direct-lite` | The same retained ZSUI Rust pipeline | Final tiny-skia/Softbuffer surface |
| `rust-text-proof` | Development-only proof data/functions on top of `rust-text` | No application presentation |
| `windows-text-proof` | DirectWrite reference plus `rust-text-proof` | PNG, JSON and SVG evidence |

The Rust path is optional and does not add a WebView, Direct2D application
runtime or global control registry. Its Windows DIB grows only to the largest
text box retained by that renderer. GDI remains the fallback for unsupported
overflowing ellipsis behavior. The platform adapter selects the Windows
line-metric policy; other platforms must not inherit Windows typography
conventions merely because they reuse the portable engine.

Backends may composite a retained layout directly into a larger BGRA software
surface with a physical origin and clip. This path reuses the glyph cache and
does not allocate a temporary text bitmap for every draw command. Windows may
select subpixel RGB; Linux Lite deliberately selects grayscale while retaining
its GTK typography metrics and native window/input/service behavior.

`rust-text` is a production capability. `rust-text-proof` and
`windows-text-proof` are framework-development features used by examples, tools
and CI only. Applications and release feature bundles must not enable proof,
and proof output must never be presented as an alternative runtime renderer or
serialized UI format.

## Multi-font proof matrix

This section describes framework development and CI, not an application build
mode.

Run the Windows oracle suite with:

```powershell
cargo run --example text_geometry_proof `
  --no-default-features `
  --features windows-text-proof `
  -- target/text-proof
```

The suite currently combines 11 corpora with six size, line-height, weight and
DPI variants, producing 66 cases. It covers:

- Segoe UI Latin, kerning, ligatures, punctuation and numerals;
- Microsoft YaHei UI simplified Chinese;
- Yu Gothic UI Japanese and Malgun Gothic Korean;
- Segoe UI Arabic and Hebrew bidirectional runs;
- Nirmala UI Devanagari and Leelawadee UI Thai shaping;
- Segoe UI Emoji color/ZWJ and variation-selector sequences;
- Consolas code and punctuation;
- mixed fallback, bidirectional text and constrained wrapping.

The six variants exercise caption, body, title and display sizes; regular,
semibold and bold requests; 100%, 125%, 150% and 200% scale factors; and fixed
line heights. Every case fixes text, box dimensions, wrapping, weight and DPI.

Each case emits:

- `directwrite.json` and `rust.json` for lines, resolved font faces, glyph IDs,
  clusters, origins and advances;
- `geometry-diff.json` and `geometry-overlay.svg` for structural comparison;
- `directwrite.png`, `rust-subpixel.png` and `pixel-difference.png` for final
  raster comparison;
- `rust-outline.svg` for outline inspection independent of antialiasing.

`summary.json` aggregates the entire matrix. The reference PNG is rendered by
DirectWrite into a real Windows DIB; it is not produced from the Rust draw
plan.

## Acceptance order

Text changes are evaluated in this order:

1. requested family resolves to the intended physical face for every run;
2. fallback face, glyph ID, cluster range, direction and line break agree;
3. baselines, glyph origins and advances satisfy the scale-aware tolerance;
4. critical glyph regions satisfy calibrated pixel thresholds;
5. the complete line image satisfies a looser antialiasing tolerance.

Pixel similarity cannot excuse a different font, glyph sequence or line break.
Conversely, a subpixel edge difference does not invalidate otherwise identical
geometry. Animated content, carets and time-dependent text must be frozen or
masked before comparison.

Geometry comparison pairs glyphs by visual line, UTF-8 cluster, glyph ID and
physical face before measuring origins. One backend omitting a trailing wrap
space therefore remains a local count/line-break failure instead of shifting
every later pair. A geometry pass also requires exact glyph and face identity;
the tolerance applies only to placement of already matched glyphs.

The Rust candidate is experimental until the complete matrix is stable. A
single passing UI font, script, size or DPI is only path evidence and must not
be reported as renderer parity.

The current 66-case Windows run passes geometry tolerance in 61 cases and has
exact resolved faces and glyph IDs in 61 cases. All Latin, Chinese, Japanese,
Korean, Arabic, Hebrew, Devanagari, Thai and Consolas cases pass. The retained
rules now cover mixed-script Kana GPOS pairs, DirectWrite-compatible RTL pair
placement, visible-glyph synthetic bold, wrap-separating spaces and paragraph
baseline phase. The remaining five failures are weighted emoji and mixed
semibold cases where DirectWrite and fontdb choose different CJK/fallback
physical faces; matched advances differ by at most 0.001 px. Maximum pixel
difference is 9.17%, and the largest matched-origin delta is 5.342 px after the
different fallback face changes earlier advances. These numbers are development
evidence, not production telemetry or a completion claim, and the Rust backend
must not replace DirectWrite by default yet.

## Font matching rule

Font weight is resolved inside the requested family before shaping. This is
required for families without an exact semibold face: a 600 request for
Consolas or Microsoft YaHei UI must choose the closest face in that family,
not silently switch the Latin portion to an unrelated family. The resolved
PostScript name and face index remain in every glyph proof so fallback changes
are visible.

Variable-font axes are honored before synthesis. Visible-glyph synthetic bold
is retained when no semibold face or weight axis exists; optical sizing and
platform fallback ordering remain explicit compatibility work. They must be
added to the proof schema and matrix rather than hidden behind appearance-only
tuning.
