# ZSUI Application Authoring Contract

Use this contract when building an application with ZSUI. Framework internals,
component galleries and acceptance applications are not application templates.

## Default decision

Start with the smallest `zsui::prelude` / `zsui::stable` tree that contains the
regions and actions explicitly requested by the product brief. The default
application shape is:

```text
window
└── column / row
    ├── semantic text
    └── explicitly requested controls
```

An absent requirement is not permission to invent another region. Do not add
navigation, cards, tabs, inspectors, command bars,
status banners or duplicate actions merely because a framework example exposes
them.

Before writing code, reduce the brief to this internal decision record:

```text
surface: compact_utility | form | navigation | document | custom
required_regions: [...]       # only regions named or required by the workflow
required_actions: [...]       # one visual action for each product action
required_data: [...]          # only data needed to make a decision
forbidden_regions: [...]      # everything unrelated to the selected surface
initial_size: width x height  # derived from content, never copied from a demo
features: [...]               # smallest Cargo feature set
```

Do not serialize this record into the product. It is a selection guard for the
implementation task.

## Surface selection gates

| Surface | Select only when | Normal composition | Do not infer from |
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
- Derive initial and minimum window sizes from intrinsic content and platform
  control metrics. A demonstration's `1280 x 800` window is not a default.
- Let ZSUI's platform experience profiles resolve native fonts, spacing,
  control geometry and icons. Application code does not select WinUI, AppKit
  or GTK styling enums.

## Feature selection

Start with `default-features = false` and enable only the controls used by the
view. Do not enable `full`, `all-widgets`, `settings`,
`document-shell`, `ui-viewer` or a Gallery feature unless the selected surface
requires it.

A compact service control surface normally needs only:

```toml
zsui = { version = "0.2", default-features = false, features = [
    "window", "button", "label", "toggle"
] }
```

Use [`examples/compact_service_panel.rs`](../../examples/compact_service_panel.rs)
as the application-shape reference. It intentionally contains one status
region and one action region, without a shell.

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
- Is any region absent from the product brief?
- Is one action duplicated in multiple regions?
- Was a demo window size, placeholder or sample record copied?
- Is `full` enabled for a small application?
- Is visible framework-owned English mixed into localized application copy?
- Do background or streaming results use a zero-size Video, animation, timer
  widget or fixed-rate polling loop instead of `InvalidationHandle`?
- Could the same workflow use fewer components without losing information or
  accessibility?
