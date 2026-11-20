# HTML Design to Native Rust

Convert an HTML/CSS design into Rust layout, drawing and typed interaction code.
The source defines the visual target; it does not determine the shipped runtime.
Use the `html-to-rust` context pack and the [capability map](design-capabilities.md).

## Runtime boundary

- Ship native Rust UI code and required assets. Do not ship a WebView, DOM
  renderer, HTML/CSS interpreter or JavaScript UI runtime for the converted UI.
  Blitz and browser engines may be research or reference tools, not application
  dependencies for this route. Embedding HTML in a Rust string is not conversion.
- Keep reference extraction, screenshots and comparison tools outside the
  application's runtime dependency graph and distribution. Reference HTML/CSS
  may remain in a development fixture; it must not drive the released interface.
- Use the existing State/Msg/update loop, typed units, native services and
  buffered paint path. Do not introduce a second event loop or reactive runtime.
- Components and platform presets are optional. Select a control because its
  behavior and customizable structure fit the target, not because its catalog
  entry exists. Native drawing may own bespoke layout, decoration and artwork.
- Reconstruct editable text and interactions as behavior, not pixels. A whole
  page screenshot with click rectangles is not a native conversion. Rasterized
  decorative artwork is acceptable only when its scale, purpose and fidelity
  are explicit; text and controls must retain their required semantics.

## Reference contract

Before implementation, identify the source HTML/CSS, assets, visible content,
fonts, target platform and reference viewport(s). Keep reference identity and
content hashes with the working evidence so comparisons use the same design.

| Evidence | Record |
| --- | --- |
| Environment | Client area in CSS logical units, screenshot pixels, device scale, browser zoom, target DPI, color scheme and font files/weights |
| Structure | Stable source node IDs, visible hierarchy, reading order, control roles and relationships |
| Layout intent | Flex/Grid rules, fixed/fill/content sizing, min/max bounds, gaps, padding, alignment, wrapping, positioning, clipping and overflow ownership |
| Computed geometry | Actual element bounds, text line boxes/baselines, scroll extents, stacking order and transform origins |
| Appearance | Resolved colors, borders, radii, shadows, gradients, image fit/crop and font fallback |
| States | Normal, hover, pressed, focus-visible, selected, disabled, invalid, loading and empty states that the design uses |
| Behavior | Actions, values, tab order, keyboard activation, text selection/IME, scroll and accessibility requirements |

Use computed styles and rendered geometry together with source constraints.
Computed rectangles alone lose responsiveness; stylesheet text alone misses
inheritance, default styles and font fallback. Capture narrow, target and wide
layouts when resizing is part of the brief. For animation, fix a timeline point.

Reference acquisition does not authorize computer control. Use supplied evidence
or file/offscreen analysis that does not control the user's UI. Browser/window
control, simulated input and native UI automation require explicit authorization.
If necessary evidence cannot be obtained, continue independent work and record
the affected acceptance case as pending; do not label it verified.

## Conversion record

Maintain a compact development-only record per meaningful region:

```text
source_id -> rust_symbol / WidgetId
role and actions -> typed messages and state
layout rules -> native constraints and measurement
appearance and states -> theme/style values or drawing operations
assets and fonts -> application resource identities
capability -> available | bounded | gap
acceptance -> reference case and measured result
```

This record is evidence for the agent, not a new runtime schema or an embedded
interpreter. Group repeated design values into small Rust structs/constants and
ordinary functions. Keep provenance, diagnostics and editing notes out of UI copy.

## Implementation sequence

1. Reduce the design to its required content and actions; preserve its visual
   hierarchy and decorative structure. Do not force it into a settings shell,
   minimal service panel or platform skin.
2. Map each region using the capability table. Locate the exact public symbol
   and feature before writing a call. Classify unsupported effects as gaps;
   do not silently discard CSS or invent a ZSUI API.
3. Choose one compatible authoring family for each connected tree. The stable
   `Element<Msg>` and compatibility `ViewNode<Msg>` APIs are not interchangeable.
   Use a pinned compatibility surface where necessary, or implement a small
   stable facade as separately scoped framework work.
4. Implement structure and text measurement first. Keep intrinsic sizing, line
   wrapping and constraints in Rust; do not freeze every node to one screenshot's
   coordinates. Custom drawing computes local geometry from the allocated size.
5. Add appearance and assets, then required interaction states. Keep paint,
   hit-test, clipping and accessibility geometry consistent. A layout container
   or decoration may add nodes without adding a business feature.
6. Connect typed state and actions. Preserve stable identities, focus, selection,
   scrolling and input composition across rebuilds. Reuse native input and
   accessibility services even when appearance is custom.
7. Render, measure and repair differences in the order below. Change one class
   of discrepancy at a time and rerun the affected cases.

If fidelity requires a missing capability, isolate the reusable primitive or
layout operation and its feature before extending the framework. Do not replace
the design with a preset or add a full rendering engine to hide the gap. If the
task does not authorize that framework work, identify the precise limitation
and continue the supported regions.

## Visual acceptance

Compare reference and native output at the same content, client size, physical
scale, font resources, theme, scroll offset, state and animation time. Crop window
chrome consistently. Native styles on different platforms need separate target
evidence; one host's screenshot is not proof for every host.

| Order | Check | Default acceptance rule |
| --- | --- | --- |
| 1 | Content and structure | No missing/extra text, actions or assets; hierarchy and reading order preserved |
| 2 | Layout | No unintended overlap, clipping or overflow; deterministic edges/baselines within one physical pixel of the reference at each fixed case |
| 3 | Typography | Required face/weight and fallback identified; line breaks and truncation match; investigate geometry separately from antialiasing |
| 4 | Appearance | Compare per-region colors, borders, radii, gradients, shadows and image crop; solid interior sRGB channels within one 8-bit level under a matched color pipeline |
| 5 | Interaction | Required states, hit regions, keyboard/focus, selection, IME and accessible actions behave correctly; disabled actions cannot fire |
| 6 | Adaptation | Narrow/target/wide and 100/150/200 percent scale cases as applicable; no bitmap enlargement replacing DPI-aware rendering |

These are acceptance targets, not measured claims. A task may specify different
tolerances before comparison. Record any accepted deviation by region and cause;
do not widen thresholds after a failure simply to produce a pass. Do not silently
substitute a missing font or drop an unsupported effect.

Use side-by-side, overlay and difference images with geometry and text metrics.
Whole-image similarity alone cannot pass a case: large blank areas can hide an
incorrect control, and antialiasing can exaggerate otherwise matching text.
Never report a universal restoration percentage from unrelated web benchmarks.

Use [offscreen appearance proof](../../examples/appearance_offscreen_proof.rs)
as a renderer entry example, not a complete HTML comparison harness. It opens no
UI windows. Offscreen proof cannot establish OS focus, real IME or accessibility
integration; those cases need authorized target testing or remain explicitly
unverified. Comparison reports use pass, fail, not-run or blocked per case.

## Lightweight delivery and verification

- Enable only the application features used by the reconstructed design. Keep
  `full`, galleries, the Viewer and extraction tools out of ordinary release
  bundles. Pure Rust does not by itself establish low memory or small binaries.
  When adding framework capabilities, record the dependency/features delta and
  compare build size, idle memory and relevant frame work against the same
  application's baseline; do not add an abstraction solely to shorten one call.
- Inspect the resolved application runtime dependencies for the actual target
  and feature set, and inspect the packaged file allowlist. No HTML/CSS/DOM
  runtime may enter through a transitive dependency or embedded resource loader.
  `check-native-boundary.ps1 -Locked` checks the existing WebView boundary; it
  does not by itself prove the absence of every HTML/CSS engine.
- Compile the converted application with its actual locked dependency and
  minimal feature set, then test its required state/actions. Run focused checks
  for the Rust code changed. Shared/public/renderer changes
  retain formatting, no-default tests, full tests and the locked feature matrix.
  Respect the operating workspace's build-cache and disk-space rules.
- Documentation-only changes require `ai-context.ps1 -Validate`, local-link
  validation and `git diff --check`, without rebuilding unrelated code.
- Retain the generated Rust source, necessary licensed resources, build feature
  selection and concise acceptance results. Keep temporary reference captures,
  comparison output and scripts in the configured operations directory, not
  in the product's UI or release package.

## Framework gap priorities

The [capability map](design-capabilities.md) is the source-linked inventory.
Implement only gaps exercised by the chosen design, in this order:

1. Small stable facades for existing native drawing/layout/image capabilities,
   and a development-only reference-to-Rust mapping/comparison workflow.
2. Reusable constraints, wrapping/adaptive/layer layouts and custom control
   content with preserved input and accessibility behavior.
3. Shared drawing operations for gradients, shadows, transforms, group opacity
   and shaped clipping, verified across the affected native renderers.
4. Rich typography and general transitions when a concrete design requires them.

Prefer a few composable typed operations over another catalog of fixed skins.
Do not turn this workflow into a mandatory schema compiler, CSS runtime,
dependency-injection framework or global widget registry.
