# Resizable layout

`NavigationView`, `SplitView` and `DataGrid` share captured pointer routing,
logical DP constraints and typed resize messages. Applications own the resulting
sizes in their state and use those values when rebuilding the View.

## Panes

Enable a divider with `on_pane_resize` and set `pane_resize_limits` to a validated
`ZsResizeLimits`. `SplitView::axis` selects horizontal or vertical splitting;
`pane_extent` is the width or height along that axis. `minimum_content_extent`
reserves space for the other region. Leading and trailing panes share the same
positive extent convention. Navigation uses `pane_width` and
`minimum_content_width`; `pane_open` and `on_navigation_open_change` can control
its compact overlay state.

```rust
use zsui::{split_view, text, Dp, WidgetId, ZsResizeLimits, ZsSplitViewSpec};
#[derive(Clone)]
enum Msg { Width(Dp) }
let pane = split_view(
    WidgetId::new(10),
    ZsSplitViewSpec::new(true).pane_extent(Dp::new(240.0)),
    text("Panel"), text("Content"),
).pane_resize_limits(ZsResizeLimits::new(Dp::new(120.0), Dp::new(480.0))?)
 .on_pane_resize(Msg::Width);
# Ok::<(), zsui::ZsuiError>(())
```

The stable facade provides `PanelSpec`, `split_panel`, `NavigationSpec`,
`navigation_view`, `ResizeLimits` and `Element::on_resize` with the same contract.

## Table columns

Every column is resizable by default when `on_table_column_resize` is attached.
`ZsTableColumn::resizable(false)` disables an individual handle; `resize_limits`
defines its range. The callback carries the stable column ID and next DP width.
Apply it to the matching application-owned column using `resize_to`. A dragged
fill column becomes fixed; a fixed last column can shrink without automatically
absorbing the viewport's unused width. Stable applications use `TableColumn`,
`ColumnResize` and `data_grid`.

## Input and accessibility

The divider has an independent 8-DP pointer hit region. Capture preserves the
initial pointer offset and clamps against both declared limits and the available
layout. Escape restores the extent at capture start; target removal, suspension
and DPI changes cancel capture. Focused dividers accept the matching arrow keys
in 8-DP steps, Shift in 32-DP steps, and Home/End for their limits. They expose an
adjustable splitter through UIA, AppKit and AccessKit, including the same bounds
and numeric constraints used for pointer input.

## UI documents

Navigation and split panes bind `pane_extent` (or the legacy `pane_width`) to a
number property and `resize` to a number action. A document cannot specify both
extent names. Optional `minimum_pane_extent` and `maximum_pane_extent` define the
bounds; SplitView accepts `axis: "horizontal"` or `"vertical"`.

Tables bind `columns` and the `columns_change` action to `table_column_array`.
Each action contains the complete next column array, retaining author IDs,
metadata and preceding resize changes. Column properties include `resizable`,
`minimum_width` and `maximum_width`. Interactive fixed widths must be inside the
declared limits. Navigation may additionally bind `open` and `open_change`.

`examples/resizable_layout.rs` exercises sidebars, horizontal/trailing and
vertical panels, every table column and all semantic icons. Its smoke mode
routes actual host pointer input and verifies the resulting application state.

Native proof covers the navigation pane, horizontal/trailing and vertical
panels, and all three table columns in light, dark and high-contrast themes.
[All five backends](https://github.com/qiu7824/zsui/actions/runs/39750838016)
pass on `45f2c6b0d34bcff2d362119a5a26fb4666a11ac5`. Each scene checks the retained
application extents and rejects accidental table sorting during a resize.
Every captured scene must contain visible RGB content. Win32 additionally
verifies native UIA numeric actions on all six dividers.
