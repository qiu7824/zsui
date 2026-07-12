# Paged virtual lists

ZSUI separates long-list rendering from data access:

- `virtual-list` computes the visible and overscan ranges, lays out only those
  rows and clips painting and hit testing to the viewport.
- `paged-list` adds a dedicated loader thread, page request deduplication,
  generation-based stale-result rejection and a bounded LRU page cache.
- Applications own their database or network query. ZSUI never requires a
  particular async runtime or persistence library.

Enable only the required surface:

```toml
zsui = { git = "https://github.com/qiu7824/zsui", default-features = false, features = [
    "window", "label", "paged-list"
] }
```

## Application shape

Create one `PagedListState` with a synchronous page source. The source is
executed by the framework worker and must return stable keys:

```rust
let records = PagedListState::new(
    PagedListConfig::default()
        .page_size(50)
        .cache_pages(5)
        .prefetch_pages(1)
        .total_count_hint(100_000),
    |request: PageRequest| database_page(request),
)?;
```

The view contains no pagination, thread or cache bookkeeping:

```rust
paged_list(&state.records, |index, item| record_row(index, &item.value))
    .id(RECORDS)
    .item_height(Dp::new(52.0))
    .overscan_rows(8)
    .on_viewport_changed(Msg::RecordsViewport)
```

Route the typed viewport message back to the state:

```rust
Msg::RecordsViewport(viewport) => state.records.update_viewport(viewport),
```

## Runtime behavior

1. Layout derives global visible and overscan ranges directly from scroll
   offset, viewport height and fixed row height. It does not iterate through
   the total item count.
2. Missing pages are queued once. The worker continues through queued
   prefetch pages without blocking the UI thread.
3. Missing visible rows paint stable placeholders until their page arrives.
4. Completed pages update a shared snapshot. Stateful Win32 views poll only
   while work is pending, rebuild the draw plan and stop polling when idle.
5. Pages outside the protected viewport are evicted by least-recently-used
   order. Visible and in-flight pages are never evicted.
6. Resetting a query increments its generation. Results from older searches or
   filters are discarded even when they finish later.
7. The first loaded visible key is retained as a scroll anchor, keeping
   selection and position independent from cache eviction.

The initial implementation uses fixed row heights. Variable-height metrics and
scrollbar thumb dragging remain separate follow-up capabilities; neither is
required for database history, file lists and other uniform-row collections.

## Verification

```powershell
cargo test --no-default-features --features paged-list,label
cargo check --example paged_virtual_list --no-default-features --features window,button,label,paged-list
cargo run --example paged_virtual_list --no-default-features --features window,button,label,paged-list -- --smoke
```

The tests cover range math at 100,000 rows, visible-only layout and painting,
global selection indices, background prefetch, request deduplication, LRU
eviction, stale generation rejection, retry and stable anchors.
