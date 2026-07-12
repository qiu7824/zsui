use std::{thread, time::Duration};

use zsui::{
    button, column, native_window, paged_list, row, text, AppCx, Dp, NativeWindowSmokeRunOptions,
    Page, PageRequest, PagedItem, PagedListConfig, PagedListState, Point, ThemeColorToken,
    ViewNode, VirtualListViewport, WidgetId, ZsuiError, ZsuiResult,
};

const TOTAL_ROWS: usize = 100_000;
const PAGE_SIZE: usize = 50;
const LIST: WidgetId = WidgetId::new(1);
const RELOAD: WidgetId = WidgetId::new(2);
const RETRY: WidgetId = WidgetId::new(3);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    title: String,
    detail: String,
}

struct AppState {
    records: PagedListState<u64, Record>,
    selected: Option<usize>,
}

impl AppState {
    fn new() -> ZsuiResult<Self> {
        let records = PagedListState::new(
            PagedListConfig::default()
                .page_size(PAGE_SIZE)
                .cache_pages(5)
                .prefetch_pages(1)
                .initial_viewport_rows(16)
                .total_count_hint(TOTAL_ROWS),
            |request: PageRequest| {
                // This closure runs on the dedicated page worker, never on the UI thread.
                thread::sleep(Duration::from_millis(35));
                let end = request
                    .offset()
                    .saturating_add(request.page_size)
                    .min(TOTAL_ROWS);
                Ok(Page::new((request.offset()..end).map(|index| {
                    PagedItem::new(
                        index as u64,
                        Record {
                            title: format!("Record {:06}", index + 1),
                            detail: format!("Page {} / cached data row", request.page.0 + 1),
                        },
                    )
                }))
                .total_count(TOTAL_ROWS)
                .has_more(end < TOTAL_ROWS))
            },
        )?;
        Ok(Self {
            records,
            selected: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    ViewportChanged(VirtualListViewport),
    Selected(usize),
    Reload,
    Retry,
}

fn view(state: &AppState) -> ViewNode<Msg> {
    let snapshot = state.records.snapshot();
    let status = if let Some(error) = snapshot.last_error {
        format!("Load failed: {}", error.error)
    } else if snapshot.loading {
        format!(
            "Loading in background / {} cached pages",
            snapshot.cached_pages
        )
    } else {
        format!(
            "{} rows / {} cached pages",
            TOTAL_ROWS, snapshot.cached_pages
        )
    };
    let toolbar = row([
        text(status).flex(1.0),
        button("Reload").id(RELOAD).on_click(Msg::Reload),
        button("Retry").id(RETRY).on_click(Msg::Retry),
    ])
    .height(Dp::new(40.0))
    .gap(Dp::new(8.0));
    let records = paged_list(&state.records, |index, item| {
        row([
            text(item.value.title.clone()).flex(1.0),
            text(item.value.detail.clone()).width(Dp::new(190.0)),
        ])
        .id(WidgetId::new(10_000 + index as u64))
        .padding(Dp::new(10.0))
        .gap(Dp::new(12.0))
    })
    .id(LIST)
    .item_height(Dp::new(52.0))
    .overscan_rows(8)
    .selected_index(state.selected)
    .on_select(Msg::Selected)
    .on_viewport_changed(Msg::ViewportChanged)
    .bg(ThemeColorToken::Surface);

    column([toolbar, records])
        .padding(Dp::new(12.0))
        .gap(Dp::new(8.0))
        .bg(ThemeColorToken::Surface)
}

fn update(state: &mut AppState, message: Msg, _cx: &mut AppCx) {
    match message {
        Msg::ViewportChanged(viewport) => state.records.update_viewport(viewport),
        Msg::Selected(index) => state.selected = Some(index),
        Msg::Reload => {
            state.selected = None;
            state.records.reset(Some(TOTAL_ROWS));
        }
        Msg::Retry => {
            state.records.retry_failed();
        }
    }
}

fn main() -> ZsuiResult<()> {
    let builder = native_window("ZSUI Paged Virtual List")
        .size(820, 620)
        .min_size(560, 380)
        .stateful_view(AppState::new()?, view, update);
    if std::env::args().any(|argument| argument == "--smoke") {
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(900).native_view_scroll(Point { x: 400, y: 360 }, 520),
        )?;
        if !report.visible_window_was_created() || report.native_view_scroll_count == 0 {
            return Err(ZsuiError::host(
                "paged_virtual_list_smoke",
                "the native list window or typed scroll path was not observed",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}
