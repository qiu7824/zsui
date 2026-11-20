# Native Design Capability Map

Use this map to choose real ZSUI APIs for design-led Rust authoring and
[HTML design conversion](html-to-rust.md). Inspect the linked symbols before
implementation; the table describes source contracts, not blanket cross-platform
visual certification. Update the affected row when its implementation changes.

## API families

`zsui::prelude` re-exports `zsui::stable`, which builds `Element<Msg>`.
Crate-root drawing/layout builders generally build `ViewNode<Msg>` and use
`native_window(...).stateful_view(...)`. They remain compatibility APIs; see
[API stability](../api-stability.md). `Element::from_raw/into_raw` are private.
Do not invent stable wrappers or mix the two tree types. Choose the smallest
verified API surface, pin compatibility usage, and keep platform handles private.

Status meanings: **available** = an existing callable contract; **bounded** =
only the stated subset; **gap** = no corresponding general authoring contract
in the linked implementation. Drawing primitives are independent of choosing a
platform-controls appearance.

## Mapping

| Design intent | Existing Rust surface and feature | Status and conversion boundary |
| --- | --- | --- |
| Row/column, fill, gap, padding, alignment | `stable::{row,column,spacer}`, element `flex/gap/padding_sides/justify/align`; [stable.rs](../../src/stable.rs) | Available; flex growth is not a complete CSS Flexbox implementation |
| Width/height and constraints | Element and View dimension builders, [node.rs](../../src/view/node.rs) | Fixed, minimum and maximum outer dimensions in DP. Percent sizing, aspect ratios and a general constraint builder remain gaps. |
| Grid with spans | Stable `grid` and `GridTrack` support explicitly placed cells and fixed/fraction tracks. Crate-root `grid` also supports spans; `grid`; [stable design](../../src/stable_design.rs), [data.rs](../../src/view/widgets/data.rs) | Auto, minmax and subgrid remain gaps. |
| Responsive wrapping and layered children | Row/column/Grid; size-aware `canvas_with` for custom geometry | Gap for general Wrap, layout-constraint callbacks and layer/positioned child composition; a popup is not a general layer container |
| Custom native drawing | Stable `canvas_with`, `CanvasScene::fill_rect`, `CanvasColor`; `canvas`; [stable design](../../src/stable_design.rs) | Stable rectangles use local DP geometry and semantic fills. Crate-root [Canvas builders](../../src/view/widgets/canvas.rs) also expose text measurement, hit regions and other primitives. |
| Palette, density and visual states | `AppTheme`, `Appearance`, `StateStyles`, `ControlStyle`; [appearance.rs](../../src/appearance.rs) | Available for the defined fields; no general CSS cascade, arbitrary control-part templates or effect graph |
| Font family, size, line height, weight, wrapping | `FontSpec`, `text_wrap`, semantic text; `label`/`textbox`; [appearance.rs](../../src/appearance.rs), [stable.rs](../../src/stable.rs) | Bounded; FontSpec has no general italic/letter-spacing/rich-span authoring fields; validate actual fallback, line breaks and ink |
| Borders and rounded fills | Element `border/radius`, ControlStyle border width, Canvas primitives; [node.rs](../../src/view/node.rs), [render protocol](../../src/render_protocol.rs) | Bounded; element border is a one-physical-pixel outline; a rounded fill does not round-clip descendants; independent corner radii and arbitrary paths are not general scene APIs |
| Gradients, shadows, transforms, group opacity, shaped clipping | Shared `NativeDrawCommand`; [render_protocol.rs](../../src/render_protocol.rs) | Gap for these general scene operations; per-fill alpha and rectangle clipping are not substitutes for group composition |
| Raster images and SVG icons | Stable `image`, `ImageFrame::from_png`, `ImageFit`; `image-preview`; [image inspector](../../examples/image_inspector.rs). Stable `IconAsset`/`IconSource` use `custom-icons`. | Retained raster frames and validated vector icons. |
| Optional standard interactions | Stable buttons, textboxes, selection controls and themes; [stable_controls.rs](../../src/stable_controls.rs) | Available where appearance/structure fits; Button label/presentation is bounded, without a general arbitrary-content slot |
| Custom interaction | `on_canvas_pointer`, `on_canvas_hover`, `on_click`, stable IDs; [node.rs](../../src/view/node.rs), [canvas.rs](../../src/canvas.rs) | Bounded: pointer/hover regions do not automatically become independent keyboard-focus or accessible nodes; custom text editing needs the native input services |
| Scrolling | Crate-root `scroll`, retained offset, `auto_scroll_y`; `scroll`; [data.rs](../../src/view/widgets/data.rs), [node.rs](../../src/view/node.rs) | Bounded: general Scroll is vertical; text-editor horizontal scrolling is not a general two-axis CSS overflow API |
| Retained work and transitions | `memoized`, invalidation and draw-plan damage; [memo.rs](../../src/view/memo.rs), [render_protocol.rs](../../src/render_protocol.rs) | Available reuse/invalidation; built-in progress animation does not provide general implicit style/layout transitions |
| Reference extraction and visual comparison | Native offscreen rendering and example metrics; [appearance_offscreen_proof.rs](../../examples/appearance_offscreen_proof.rs) | Bounded: no end-to-end HTML computed-style extraction, source-to-Rust mapping and visual acceptance runner supplied by this example |
| Optional semantic UI documents | `UiDocument`, runtime and Viewer; [ui_document.rs](../../src/ui_document.rs), [ui_document_runtime.rs](../../src/ui_document_runtime.rs) | Includes maximum width and height through shared Rust layout. Appearance/FontSpec/state-style coverage is incomplete. |

Check the [Cargo feature definitions](../../Cargo.toml) and the actual export in
[lib.rs](../../src/lib.rs) before choosing imports. Similar names in source do
not establish stable API coverage. Native host services and input are retained
regardless of whether a design uses standard controls.

## Extension boundaries

| Priority | Reusable gap | Required proof before marking available |
| --- | --- | --- |
| P0 | Stable Grid spans and additional Canvas primitives for concrete design cases | Minimal-feature examples compile with stable imports and preserve the geometry and behavior of the corresponding native builders. |
| P0 | Development-only extraction, mapping and comparison workflow | Same inputs/viewport/fonts/states identify the generated Rust regions and produce per-case geometry/pixel results; no shipped HTML runtime |
| P1 | Constraints, wrapping, adaptive and layer layout | Resize and DPI cases preserve intrinsic text size, hit geometry and reading order |
| P1 | Custom content and reusable interaction semantics | Custom visuals retain focus-visible, keyboard activation, disabled behavior, IME where applicable and an accessible subtree |
| P1 | Missing scene operations | Shared command semantics, clipping/composition and damage bounds agree across affected native renderers |
| P2 | Rich typography and general transitions | Measurement/paint/editing agree; transitions respect reduced motion and stop scheduling frames when idle |

These are implementation gaps, not callable APIs or a request to implement every
item at once. Use a concrete reference case to scope each addition. Keep reusable
operations feature-selectable and application-specific composition in ordinary
Rust functions. A component library remains an optional platform-controls route.
