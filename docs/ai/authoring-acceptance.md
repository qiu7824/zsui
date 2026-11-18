# Native UI authoring acceptance

Use each brief independently with the same toolchain and model configuration.
Record the initial compile result, repair attempts, unsupported API names, input
tokens and behavioral assertions passed. Supply the function signatures from
[the reference submissions](../../tests/authoring/cases.rs) and the briefs
below. Replace the bodies and [form document](../../tests/authoring/name-form.json)
with the submission in an evaluation checkout, leaving the
[assertions](../../tests/authoring_contracts.rs) fixed. The Rust Save button has
widget ID 2 and emits `"save"`. Record rows use widget ID `1000 + index`.

| Brief | Observable result | Required features |
| --- | --- | --- |
| Build a name form with a Save action. Limit its outer size to 640 × 240 DP. Use the available space on a smaller surface. Supply equivalent Rust and UiDocument layouts. | At 320 × 160, 640 × 240 and 960 × 480 DP, both trees agree on child rectangles at 100%, 150% and 200% DPI. Save emits one action. | `ui-document-runtime,button,textbox,label` |
| Draw a custom panel with an 8 DP high accent bar occupying half the panel width. Cap the panel at 320 DP. | At 160 and 480 DP available width the bar draws at 80 and 160 DP respectively. | `canvas` |
| Present records 10–13 from a 100,000-record collection and return the selected global row number. | Exactly four rows are retained. Activating the third returns 12. | `virtual-list,label` |

Visual grouping, typography and composition follow the brief. The custom panel
uses native drawing. The form and list retain semantic controls and typed events.

Run the reference cases with:

```sh
cargo test --locked --no-default-features --features ui-document-runtime,button,textbox,label,canvas,virtual-list --test authoring_contracts
```
