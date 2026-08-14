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
navigation, cards, tabs, inspectors, timelines, composers, command bars,
status banners or duplicate actions merely because a framework example exposes
them.

Before writing code, reduce the brief to this internal decision record:

```text
surface: compact_utility | form | navigation | workbench | document | custom
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
| Workbench | A message timeline and composer are explicit product requirements | `ZsWorkbenchShellSpec`; inspector only when explicitly required | “task”, “tool”, “assistant”, “management”, process output |
| Document application | Editing a document is the primary workflow | `ZsDocumentShellSpec` | A read-only text field or log excerpt |

`navigation-shell`, `workbench`, `document-shell`, Gallery and Viewer are
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
view. Do not enable `full`, `all-widgets`, `workbench`, `settings`,
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
