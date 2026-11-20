# ZSUI Application Authoring Contract

Use this contract when building an application with ZSUI. Framework internals,
component galleries and acceptance applications are not application templates.

## Authoring route

Start with visual intent, required information and actions. Choose the route
before choosing controls. Native Rust execution, platform-styled appearance
and component composition are separate decisions.

| Route | Select when | Implementation direction |
| --- | --- | --- |
| Design-led native drawing | A visual reference, brand treatment or custom layout is supplied; otherwise the default authoring route | Reconstruct or design the layout and visuals with native primitives, typography and explicit state; use matching controls where helpful |
| Platform-controls composition | The brief requests system-style UI, platform conventions or standard controls | Use the platform preset and standard components, with deliberate themes and state styles |

The word "native" alone establishes the execution requirement. Infer a system
skin only when the surrounding brief makes that appearance explicit. Components
remain reusable implementation choices on either route; neither a catalog nor
the availability of a control dictates the design. This route distinction does
not rename existing Rust APIs or remove controls.

For an HTML/CSS design reference, select the `html-to-rust` pack and follow
[HTML to Rust conversion](html-to-rust.md). HTML is a design input, never the
shipped UI runtime. Consult the [capability map](design-capabilities.md) before
using custom drawing or assuming CSS-equivalent functionality.

Do not invent business destinations, actions or data. Decorative containers,
spacing, layers and visual grouping needed by a design are permitted; they are
not additional business features. Do not copy navigation, cards or command bars
from a framework example merely because it exposes them.

Before writing code, reduce the brief to this internal decision record:

```text
surface: compact_utility | form | navigation | document | custom
route: design_led | platform_controls
visual_target: reference | original_design | platform_preset
required_regions: [...]       # business regions and reference design sections
required_actions: [...]       # one visual action for each product action
required_data: [...]          # only data needed to make a decision
forbidden_regions: [...]      # unrelated business regions, not decorative nodes
initial_size: width x height  # brief/reference or content, never a demo default
features: [...]               # only capabilities actually used
```

Do not serialize this record into the product. It is a selection guard for the
implementation task.

## Surface selection gates

These are optional compositions for matching workflows, not mandatory layouts.

| Surface | Select only when | Available composition | Do not infer from |
| --- | --- | --- | --- |
| Compact utility or control panel | A small number of values and actions fit in one view | `column`, `row`, `text`, `button`, `toggle` | “manager”, “console”, “service”, “task”, “status”, “log” |
| Form or settings page | Several editable settings need labels, validation or grouped sections | Basic View controls; `settings_card` only for real groups | The presence of one toggle |
| Navigation application | The product has at least two durable top-level destinations | `navigation_view` with one content subtree | A list of actions or status categories |
| Document application | Editing a document is the primary workflow | `ZsDocumentShellSpec` | A read-only text field or log excerpt |

`navigation-shell`, `document-shell`, Gallery and Viewer are
opt-in compositions. Never use one as a shortcut for arranging ordinary
controls.

## Requirement preservation

- Give each requested action one obvious control. Do not repeat Start in a
  sidebar, toolbar and footer.
- Keep the native title-bar close affordance; do not add an in-content Exit
  action unless the workflow explicitly requires it.
- Use semantic `ZsIcon` values only when an icon materially improves the
  action. An icon-only action requires an accessible label and an unambiguous
  platform convention.
- Do not copy placeholder copy, sample records, code blocks, English status
  words or demonstration navigation from an example.
- All visible product copy belongs to application state or its localizer.
- Derive initial and minimum window sizes from the brief/reference, content
  constraints and interaction needs. A demonstration's `1280 x 800` window is
  not a default. Preserve reference geometry when converting a design.
- Platform profiles provide initial fonts, spacing, geometry and icons. Visual
  design may choose a different palette, typography, density, tab treatment or
  focus treatment through `AppTheme`, `Appearance` and `FontSpec`; application
  code does not select WinUI, AppKit or GTK backend enums.

## Visual freedom within functional scope

Functionality and appearance have separate boundaries. Do not invent workflows,
data or actions; freely arrange the requested content and interactions into a
considered visual design. The native preset is one supported starting point.
Standard buttons, inputs and tabs retain their interaction,
keyboard and accessibility behavior when customized.

Use an application/subtree `AppTheme` for inherited tokens and typography, and
an element `Appearance` for instance and state overrides. Use accurate semantic
icons or typed `IconAsset` resources with attribution. Use controls when their
structure and styling preserve the design. Custom native drawing is a first-class
choice for bespoke visuals, with explicit hit testing, focus, keyboard and
accessibility obligations. A painted textbox is not an editable textbox, and a
Canvas hover region is not a separate accessible control. Reuse input services
and behavior without forcing their default appearance onto the design.
Keep source-content, layout/ink metrics, text fallback and raster differences
distinct when diagnosing a visual problem.

## Feature selection

Start with `default-features = false` and enable only the controls used by the
view. Do not enable `full`, `all-widgets`, `settings`,
`document-shell`, `ui-viewer` or a Gallery feature unless the selected surface
requires it.

A compact service control surface using the platform-controls route can use:

```toml
zsui = { version = "0.2", default-features = false, features = [
    "window", "button", "label", "toggle"
] }
```

Read [`examples/compact_service_panel.rs`](../../examples/compact_service_panel.rs)
only for that route and workflow; it is not the default shape for custom designs.

## Agent API discipline

Search the selected capability's public symbols and one relevant example. Prefer
`zsui::stable` / `zsui::prelude` where the needed operation exists. The current
stable surface does not wrap every crate-root drawing/layout API; do not invent
`stable::canvas` or mix `Element<Msg>` with `ViewNode<Msg>`. A compatibility API
requires an explicit import and a pinned framework revision, or a small stable
facade added as separate framework work. See the capability map for exact paths.

Use `typed::{button,textbox,toggle,text}` to configure control events before
converting to `Element<Msg>` with `.into()`. The compiler checks the event
against the control type at that boundary. The [image inspector](../../examples/image_inspector.rs)
composes typed buttons, a Grid toolbar and retained PNG pixels through the
prelude. The [authoring acceptance briefs](authoring-acceptance.md) cover
document geometry, custom drawing and global selection in a virtual list.

Use small application-local Rust functions, typed messages and reusable style
values. Reduce repeated code and dependencies, not necessary visual structure.
Do not enable `full`, add an interpreter, or build another reactive layer to
make a single UI easier to generate. Add framework APIs only for a demonstrated
reusable gap, with the existing feature and compatibility gates.

## Text, spacing and retained scrolling

Use `.text_wrap(TextWrap::Word)` on a label for width-aware wrapping. The same
modifier configures a multiline text editor; single-line inputs remain single
line. A wrapping label beside an action needs `.flex(1.0)` to receive the row's
remaining width. In a column, `.align(ViewAlign::Stretch)` allocates the full
cross-axis width to its children. Native control and text minimums remain hard
floors: choose a suitable minimum window width or a vertical composition when
the controls cannot fit side by side.

`.padding_xy(horizontal, vertical)` and
`.padding_sides(left, top, right, bottom)` provide logical-unit inner spacing.
The most recent padding modifier wins, including `.padding(uniform)`.
Measurement, layout, drawing and input bounds use the same insets.
`.border(ThemeColorToken::Border)` adds a one-physical-pixel themed outline
following the node's radius, without adding children or changing its layout.
Reserve inner space explicitly with padding when the content should be separated
from the outline. The stable Element API exposes these spacing and outline
modifiers as well as `text_wrap`.

`scroll(content)` retains its offset across stateful rebuilds for the same
widget identity. Give independently scrollable pages distinct stable IDs.
Use `.scroll_y(state.offset).on_scroll(Msg::Scrolled)` for an application-owned
offset; an explicit `.scroll_y(Dp::new(0.0))` resets it on each rebuild.
Both modes clamp after content shrinkage or viewport resize. `auto_scroll_y()`
remains available for a container that should scroll only when it overflows.

Background work calls `UiInvalidationHandle::request_rebuild()` after updating
application-owned state. Requests made before window binding remain pending;
no startup delay or synthetic pointer event is required. With `window-chrome`,
call `AppCx::try_window_command` with `ZsWindowCommand::Hide` or
`ZsWindowCommand::Show`; hosts execute the request after releasing UI state
locks. Worker and tray
callbacks send an application command through `UiInvalidationHandle` and map
it to the same update path. See [application effects](../app-effects.md).

## Composite component rule

Read a composite component pack only after the product brief passes its
selection gate. When selected, populate only optional regions that are real
requirements. A component catalog proves availability; it does not define an
application's information architecture.

## Review checklist

Reject the implementation before visual polish if any answer is “yes”:

- Did a keyword select a composite shell without its required workflow?
- Is any business region absent from the product brief or required workflow?
- Did a component preset replace an explicit design without justification?
- Does an HTML conversion ship an HTML/CSS interpreter, DOM engine or WebView?
- Does a custom interaction lack keyboard, focus, input or accessibility behavior?
- Does the code call an unverified API or mix incompatible authoring types?
- Is one action duplicated in multiple regions?
- Was a demo window size, placeholder or sample record copied?
- Is `full` enabled for a small application?
- Is visible framework-owned English mixed into localized application copy?
- Do background or streaming results use a zero-size Video, animation, timer
  widget or fixed-rate polling loop instead of `InvalidationHandle`?
- Does duplicated implementation add dependencies or behavior without serving
  the requested design or workflow?
