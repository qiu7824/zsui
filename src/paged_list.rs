use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    view::{
        virtual_list, virtual_list_viewport, ViewNode, VirtualListRange,
        VirtualListScrollDirection, VirtualListViewport,
    },
    Dp, ZsuiError, ZsuiResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    pub generation: u64,
    pub page: PageIndex,
    pub page_size: usize,
}

impl PageRequest {
    pub const fn offset(self) -> usize {
        self.page.0.saturating_mul(self.page_size)
    }

    pub const fn range(self) -> VirtualListRange {
        VirtualListRange::new(self.offset(), self.offset().saturating_add(self.page_size))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagedItem<K, T> {
    pub key: K,
    pub value: T,
}

impl<K, T> PagedItem<K, T> {
    pub const fn new(key: K, value: T) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<K, T> {
    pub items: Vec<PagedItem<K, T>>,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

impl<K, T> Page<K, T> {
    pub fn new(items: impl IntoIterator<Item = PagedItem<K, T>>) -> Self {
        Self {
            items: items.into_iter().collect(),
            total_count: None,
            has_more: true,
        }
    }

    pub const fn total_count(mut self, total_count: usize) -> Self {
        self.total_count = Some(total_count);
        self
    }

    pub const fn has_more(mut self, has_more: bool) -> Self {
        self.has_more = has_more;
        self
    }
}

pub trait PagedDataSource<K, T>: Send + Sync + 'static {
    fn load_page(&self, request: PageRequest) -> ZsuiResult<Page<K, T>>;
}

impl<K, T, F> PagedDataSource<K, T> for F
where
    F: Fn(PageRequest) -> ZsuiResult<Page<K, T>> + Send + Sync + 'static,
{
    fn load_page(&self, request: PageRequest) -> ZsuiResult<Page<K, T>> {
        self(request)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PagedListConfig {
    pub page_size: usize,
    pub cache_pages: usize,
    pub prefetch_pages: usize,
    pub initial_viewport_rows: usize,
    pub total_count_hint: Option<usize>,
}

impl Default for PagedListConfig {
    fn default() -> Self {
        Self {
            page_size: 50,
            cache_pages: 5,
            prefetch_pages: 1,
            initial_viewport_rows: 12,
            total_count_hint: None,
        }
    }
}

impl PagedListConfig {
    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.max(1);
        self
    }

    pub fn cache_pages(mut self, cache_pages: usize) -> Self {
        self.cache_pages = cache_pages.max(1);
        self
    }

    pub const fn prefetch_pages(mut self, prefetch_pages: usize) -> Self {
        self.prefetch_pages = prefetch_pages;
        self
    }

    pub fn initial_viewport_rows(mut self, rows: usize) -> Self {
        self.initial_viewport_rows = rows.max(1);
        self
    }

    pub const fn total_count_hint(mut self, total_count: usize) -> Self {
        self.total_count_hint = Some(total_count);
        self
    }

    fn normalized(self) -> Self {
        Self {
            page_size: self.page_size.max(1),
            cache_pages: self.cache_pages.max(1),
            initial_viewport_rows: self.initial_viewport_rows.max(1),
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagedListAnchor<K> {
    pub key: K,
    pub index: usize,
    pub offset_within_row: Dp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PagedListSyncReconcile {
    pub previous_generation: u64,
    pub generation: u64,
    pub previous_anchor_index: Option<usize>,
    pub anchor_index: Option<usize>,
    pub offset_y: Dp,
}

impl PagedListSyncReconcile {
    pub const fn anchor_preserved(self) -> bool {
        self.previous_anchor_index.is_some() && self.anchor_index.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLoadError {
    pub request: PageRequest,
    pub error: ZsuiError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagedListSnapshot<K, T> {
    pub generation: u64,
    pub revision: u64,
    pub total_count: usize,
    pub rows: Vec<(usize, PagedItem<K, T>)>,
    pub visible_range: VirtualListRange,
    pub materialized_range: VirtualListRange,
    pub offset_y: Dp,
    pub anchor: Option<PagedListAnchor<K>>,
    pub loading: bool,
    pub cached_pages: usize,
    pub last_error: Option<PageLoadError>,
}

struct CachedPage<K, T> {
    items: Vec<PagedItem<K, T>>,
    touched_at: u64,
}

struct PagedListInner<K, T> {
    config: PagedListConfig,
    generation: u64,
    revision: u64,
    touch_clock: u64,
    total_count: Option<usize>,
    has_more: bool,
    pages: BTreeMap<PageIndex, CachedPage<K, T>>,
    in_flight: BTreeSet<PageIndex>,
    visible_range: VirtualListRange,
    materialized_range: VirtualListRange,
    offset_y: Dp,
    row_height: Dp,
    anchor: Option<PagedListAnchor<K>>,
    last_error: Option<PageLoadError>,
}

impl<K, T> PagedListInner<K, T> {
    fn effective_total_count(&self) -> usize {
        if let Some(total_count) = self.total_count {
            return total_count;
        }
        let loaded_end = self
            .pages
            .iter()
            .map(|(page, cached)| {
                page.0
                    .saturating_mul(self.config.page_size)
                    .saturating_add(cached.items.len())
            })
            .max()
            .unwrap_or(0);
        if self.has_more {
            loaded_end
                .saturating_add(self.config.page_size)
                .max(self.config.page_size)
        } else {
            loaded_end
        }
    }

    fn page_for_index(&self, index: usize) -> PageIndex {
        PageIndex(index / self.config.page_size)
    }

    fn protected_pages(&self) -> BTreeSet<PageIndex> {
        let mut protected = self.in_flight.clone();
        if !self.materialized_range.is_empty() {
            let first = self.page_for_index(self.materialized_range.start);
            let last = self.page_for_index(self.materialized_range.end.saturating_sub(1));
            protected.extend((first.0..=last.0).map(PageIndex));
        }
        protected
    }

    fn evict_lru_pages(&mut self) {
        let protected = self.protected_pages();
        while self.pages.len() > self.config.cache_pages {
            let candidate = self
                .pages
                .iter()
                .filter(|(page, _)| !protected.contains(page))
                .min_by_key(|(_, cached)| cached.touched_at)
                .map(|(page, _)| *page);
            let Some(candidate) = candidate else {
                break;
            };
            self.pages.remove(&candidate);
        }
    }

    fn item_at(&self, index: usize) -> Option<&PagedItem<K, T>> {
        let page = self.page_for_index(index);
        let offset = index % self.config.page_size;
        self.pages.get(&page)?.items.get(offset)
    }
}

enum WorkerCommand {
    Load(PageRequest),
    Shutdown,
}

pub struct PagedListState<K, T> {
    inner: Arc<Mutex<PagedListInner<K, T>>>,
    sender: Sender<WorkerCommand>,
    cancelled: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

pub fn paged_list<K, T, Msg>(
    state: &PagedListState<K, T>,
    render: impl FnMut(usize, &PagedItem<K, T>) -> ViewNode<Msg>,
) -> ViewNode<Msg>
where
    K: Clone + Send + 'static,
    T: Clone + Send + 'static,
    Msg: Clone,
{
    let snapshot = state.snapshot();
    virtual_list(
        snapshot.total_count,
        snapshot.rows.iter().map(|(index, item)| (*index, item)),
        render,
    )
    .scroll_y(snapshot.offset_y)
    .loading(snapshot.loading)
}

impl<K, T> fmt::Debug for PagedListState<K, T>
where
    K: Send + 'static,
    T: Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.lock();
        f.debug_struct("PagedListState")
            .field("generation", &inner.generation)
            .field("revision", &inner.revision)
            .field("total_count", &inner.effective_total_count())
            .field("cached_pages", &inner.pages.len())
            .field("in_flight", &inner.in_flight)
            .finish_non_exhaustive()
    }
}

impl<K, T> PagedListState<K, T>
where
    K: Send + 'static,
    T: Send + 'static,
{
    pub fn new(config: PagedListConfig, source: impl PagedDataSource<K, T>) -> ZsuiResult<Self> {
        let config = config.normalized();
        let inner = Arc::new(Mutex::new(PagedListInner {
            config,
            generation: 0,
            revision: 0,
            touch_clock: 0,
            total_count: config.total_count_hint,
            has_more: config.total_count_hint != Some(0),
            pages: BTreeMap::new(),
            in_flight: BTreeSet::new(),
            visible_range: VirtualListRange::new(0, config.initial_viewport_rows),
            materialized_range: VirtualListRange::new(0, config.initial_viewport_rows),
            offset_y: Dp::new(0.0),
            row_height: Dp::new(40.0),
            anchor: None,
            last_error: None,
        }));
        let (sender, receiver) = mpsc::channel();
        let worker_sender = sender.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_inner = Arc::clone(&inner);
        let worker_cancelled = Arc::clone(&cancelled);
        let worker = thread::Builder::new()
            .name("zsui-paged-list".to_string())
            .spawn(move || {
                while let Ok(command) = receiver.recv() {
                    if worker_cancelled.load(Ordering::Acquire) {
                        break;
                    }
                    let WorkerCommand::Load(request) = command else {
                        break;
                    };
                    if !page_request_is_relevant(&worker_inner, request) {
                        continue;
                    }
                    let result = source.load_page(request);
                    for request in complete_page_request(&worker_inner, request, result) {
                        if worker_sender.send(WorkerCommand::Load(request)).is_err() {
                            break;
                        }
                    }
                }
            })
            .map_err(|error| ZsuiError::host("spawn_paged_list_worker", error.to_string()))?;

        let state = Self {
            inner,
            sender,
            cancelled,
            worker: Some(worker),
        };
        state.enqueue_range(
            VirtualListRange::new(0, config.initial_viewport_rows),
            VirtualListScrollDirection::Forward,
        );
        Ok(state)
    }

    pub fn generation(&self) -> u64 {
        self.lock().generation
    }

    pub fn revision(&self) -> u64 {
        self.lock().revision
    }

    pub fn is_loading(&self) -> bool {
        !self.lock().in_flight.is_empty()
    }

    pub fn cached_page_count(&self) -> usize {
        self.lock().pages.len()
    }

    pub fn update_viewport(&mut self, viewport: VirtualListViewport)
    where
        K: Clone,
    {
        {
            let mut inner = self.lock();
            inner.visible_range = viewport.visible_range;
            inner.materialized_range = viewport.materialized_range;
            inner.offset_y = viewport.offset_y;
            inner.row_height = viewport.row_height;
            inner.anchor = inner
                .item_at(viewport.visible_range.start)
                .map(|item| PagedListAnchor {
                    key: item.key.clone(),
                    index: viewport.visible_range.start,
                    offset_within_row: Dp::new(
                        viewport.offset_y.0
                            - viewport.visible_range.start as f32 * viewport.row_height.0,
                    ),
                })
                .or_else(|| {
                    inner
                        .anchor
                        .clone()
                        .filter(|anchor| viewport.visible_range.contains(anchor.index))
                });
            inner.evict_lru_pages();
        }
        self.enqueue_range(viewport.materialized_range, viewport.direction);
    }

    pub fn reset(&mut self, total_count_hint: Option<usize>) {
        let initial_rows = {
            let mut inner = self.lock();
            inner.generation = inner.generation.saturating_add(1);
            inner.revision = inner.revision.saturating_add(1);
            inner.total_count = total_count_hint;
            inner.has_more = total_count_hint != Some(0);
            inner.pages.clear();
            inner.in_flight.clear();
            inner.visible_range = VirtualListRange::new(0, inner.config.initial_viewport_rows);
            inner.materialized_range = inner.visible_range;
            inner.offset_y = Dp::new(0.0);
            inner.row_height = Dp::new(40.0);
            inner.anchor = None;
            inner.last_error = None;
            inner.config.initial_viewport_rows
        };
        self.enqueue_range(
            VirtualListRange::new(0, initial_rows),
            VirtualListScrollDirection::Forward,
        );
    }

    pub fn reconcile_synced(
        &mut self,
        total_count: usize,
        locate_anchor: impl FnOnce(&K) -> Option<usize>,
    ) -> PagedListSyncReconcile
    where
        K: Clone,
    {
        let previous_anchor = self.lock().anchor.clone();
        let next_anchor_index = previous_anchor
            .as_ref()
            .and_then(|anchor| locate_anchor(&anchor.key))
            .filter(|index| *index < total_count);
        let (report, materialized_range) = {
            let mut inner = self.lock();
            let previous_generation = inner.generation;
            let previous_anchor_index = previous_anchor.as_ref().map(|anchor| anchor.index);
            let visible_rows = inner.visible_range.len().max(1);
            let overscan_rows = inner
                .visible_range
                .start
                .saturating_sub(inner.materialized_range.start)
                .max(
                    inner
                        .materialized_range
                        .end
                        .saturating_sub(inner.visible_range.end),
                );
            let row_height = if inner.row_height.0.is_finite() {
                Dp::new(inner.row_height.0.max(1.0))
            } else {
                Dp::new(40.0)
            };
            let anchor_offset = previous_anchor
                .as_ref()
                .map(|anchor| anchor.offset_within_row.0)
                .unwrap_or(0.0);
            let viewport_offset_within_row = (inner.offset_y.0
                - inner.visible_range.start as f32 * row_height.0)
                .clamp(0.0, row_height.0);
            let viewport_height =
                (row_height.0 * visible_rows as f32 - viewport_offset_within_row).max(0.0);
            let requested_offset = next_anchor_index
                .map(|index| index as f32 * row_height.0 + anchor_offset)
                .unwrap_or(inner.offset_y.0);
            let viewport = virtual_list_viewport(
                total_count,
                row_height,
                Dp::new(requested_offset),
                Dp::new(viewport_height),
                overscan_rows,
                VirtualListScrollDirection::Stationary,
            );

            inner.generation = inner.generation.saturating_add(1);
            inner.revision = inner.revision.saturating_add(1);
            inner.total_count = Some(total_count);
            inner.has_more = false;
            inner.pages.clear();
            inner.in_flight.clear();
            inner.visible_range = viewport.visible_range;
            inner.materialized_range = viewport.materialized_range;
            inner.offset_y = viewport.offset_y;
            inner.row_height = viewport.row_height;
            inner.anchor = previous_anchor.as_ref().and_then(|anchor| {
                next_anchor_index.map(|index| PagedListAnchor {
                    key: anchor.key.clone(),
                    index,
                    offset_within_row: anchor.offset_within_row,
                })
            });
            inner.last_error = None;
            (
                PagedListSyncReconcile {
                    previous_generation,
                    generation: inner.generation,
                    previous_anchor_index,
                    anchor_index: next_anchor_index,
                    offset_y: inner.offset_y,
                },
                inner.materialized_range,
            )
        };
        self.enqueue_range(materialized_range, VirtualListScrollDirection::Stationary);
        report
    }

    pub fn retry_failed(&mut self) -> bool {
        let request = {
            let mut inner = self.lock();
            let Some(error) = inner.last_error.take() else {
                return false;
            };
            if error.request.generation != inner.generation
                || inner.in_flight.contains(&error.request.page)
            {
                return false;
            }
            inner.in_flight.insert(error.request.page);
            error.request
        };
        self.send_request(request);
        true
    }

    pub fn wait_for_idle(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if !self.is_loading() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn enqueue_range(&self, range: VirtualListRange, direction: VirtualListScrollDirection) {
        let requests = collect_page_requests(&mut self.lock(), range, direction);
        for request in requests {
            self.send_request(request);
        }
    }

    fn send_request(&self, request: PageRequest) {
        if self.sender.send(WorkerCommand::Load(request)).is_ok() {
            return;
        }
        let mut inner = self.lock();
        if request.generation == inner.generation {
            inner.in_flight.remove(&request.page);
            inner.last_error = Some(PageLoadError {
                request,
                error: ZsuiError::host(
                    "queue_paged_list_request",
                    "the background page worker is no longer available",
                ),
            });
            inner.revision = inner.revision.saturating_add(1);
        }
    }

    fn lock(&self) -> MutexGuard<'_, PagedListInner<K, T>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl<K, T> PagedListState<K, T>
where
    K: Clone + Send + 'static,
    T: Clone + Send + 'static,
{
    pub fn snapshot(&self) -> PagedListSnapshot<K, T> {
        let mut inner = self.lock();
        let materialized = inner.materialized_range;
        let first_page = inner.page_for_index(materialized.start);
        let last_page = inner.page_for_index(materialized.end.saturating_sub(1));
        inner.touch_clock = inner.touch_clock.saturating_add(1);
        let touched_at = inner.touch_clock;
        let page_size = inner.config.page_size;
        let mut rows = Vec::new();
        for (page_index, cached) in &mut inner.pages {
            if materialized.is_empty() || page_index.0 < first_page.0 || page_index.0 > last_page.0
            {
                continue;
            }
            cached.touched_at = touched_at;
            let start = page_index.0.saturating_mul(page_size);
            rows.extend(
                cached
                    .items
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(offset, item)| (start.saturating_add(offset), item)),
            );
        }
        PagedListSnapshot {
            generation: inner.generation,
            revision: inner.revision,
            total_count: inner.effective_total_count(),
            rows,
            visible_range: inner.visible_range,
            materialized_range: inner.materialized_range,
            offset_y: inner.offset_y,
            anchor: inner.anchor.clone(),
            loading: !inner.in_flight.is_empty(),
            cached_pages: inner.pages.len(),
            last_error: inner.last_error.clone(),
        }
    }
}

impl<K, T> Drop for PagedListState<K, T> {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        let _ = self.sender.send(WorkerCommand::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn collect_page_requests<K, T>(
    inner: &mut PagedListInner<K, T>,
    range: VirtualListRange,
    direction: VirtualListScrollDirection,
) -> Vec<PageRequest> {
    let total_count = inner.effective_total_count();
    let range = range.clamp(total_count);
    if range.is_empty() || total_count == 0 {
        return Vec::new();
    }
    let page_size = inner.config.page_size;
    let first_page = range.start / page_size;
    let last_page = range.end.saturating_sub(1) / page_size;
    let prefetch = inner.config.prefetch_pages;
    let max_page = total_count.saturating_sub(1) / page_size;
    let before = first_page.saturating_sub(prefetch);
    let after = last_page.saturating_add(prefetch).min(max_page);
    let mut order = Vec::new();
    match direction {
        VirtualListScrollDirection::Backward => {
            order.extend((before..=last_page).rev());
            order.extend(last_page.saturating_add(1)..=after);
        }
        VirtualListScrollDirection::Forward => {
            order.extend(first_page..=after);
            order.extend((before..first_page).rev());
        }
        VirtualListScrollDirection::Stationary => {
            order.extend(first_page..=last_page);
            order.extend(last_page.saturating_add(1)..=after);
            order.extend((before..first_page).rev());
        }
    }
    let generation = inner.generation;
    let failed_page = inner
        .last_error
        .as_ref()
        .and_then(|error| (error.request.generation == generation).then_some(error.request.page));
    let mut requests = Vec::new();
    for page in order.into_iter().map(PageIndex) {
        if Some(page) == failed_page
            || inner.pages.contains_key(&page)
            || !inner.in_flight.insert(page)
        {
            continue;
        }
        requests.push(PageRequest {
            generation,
            page,
            page_size,
        });
    }
    requests
}

fn page_request_is_relevant<K, T>(
    inner: &Arc<Mutex<PagedListInner<K, T>>>,
    request: PageRequest,
) -> bool {
    let mut inner = inner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if request.generation != inner.generation {
        return false;
    }
    if !inner.in_flight.contains(&request.page) {
        return false;
    }
    if inner.pages.contains_key(&request.page) {
        inner.in_flight.remove(&request.page);
        return false;
    }

    let total_count = inner.effective_total_count();
    let range = inner.materialized_range.clamp(total_count);
    if range.is_empty() || total_count == 0 {
        inner.in_flight.remove(&request.page);
        return false;
    }
    let first_page = range.start / inner.config.page_size;
    let last_page = range.end.saturating_sub(1) / inner.config.page_size;
    let max_page = total_count.saturating_sub(1) / inner.config.page_size;
    let first_wanted = first_page.saturating_sub(inner.config.prefetch_pages);
    let last_wanted = last_page
        .saturating_add(inner.config.prefetch_pages)
        .min(max_page);
    let relevant = request.page.0 >= first_wanted && request.page.0 <= last_wanted;
    if !relevant {
        inner.in_flight.remove(&request.page);
    }
    relevant
}

fn complete_page_request<K, T>(
    inner: &Arc<Mutex<PagedListInner<K, T>>>,
    request: PageRequest,
    result: ZsuiResult<Page<K, T>>,
) -> Vec<PageRequest> {
    let mut inner = inner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if request.generation != inner.generation {
        return Vec::new();
    }
    inner.in_flight.remove(&request.page);
    inner.revision = inner.revision.saturating_add(1);
    match result {
        Ok(page)
            if page.items.len() <= request.page_size
                && (!page.has_more || page.items.len() == request.page_size) =>
        {
            let loaded_count = page.items.len();
            inner.touch_clock = inner.touch_clock.saturating_add(1);
            let touched_at = inner.touch_clock;
            if let Some(total_count) = page.total_count {
                inner.total_count = Some(total_count);
            } else if !page.has_more {
                inner.total_count = Some(request.offset().saturating_add(loaded_count));
            }
            inner.has_more = page.has_more;
            inner.pages.insert(
                request.page,
                CachedPage {
                    items: page.items,
                    touched_at,
                },
            );
            inner.last_error = None;
            inner.evict_lru_pages();
            let range = inner.materialized_range;
            collect_page_requests(&mut inner, range, VirtualListScrollDirection::Stationary)
        }
        Ok(page) => {
            inner.last_error = Some(PageLoadError {
                request,
                error: ZsuiError::invalid_spec(
                    "paged_list.page.items",
                    format!(
                        "page {} returned {} items for page size {} with has_more={}",
                        request.page.0,
                        page.items.len(),
                        request.page_size,
                        page.has_more
                    ),
                ),
            });
            Vec::new()
        }
        Err(error) => {
            inner.last_error = Some(PageLoadError { request, error });
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn viewport(
        start: usize,
        end: usize,
        direction: VirtualListScrollDirection,
    ) -> VirtualListViewport {
        VirtualListViewport {
            offset_y: Dp::new(start as f32 * 20.0),
            row_height: Dp::new(20.0),
            visible_range: VirtualListRange::new(start, end),
            materialized_range: VirtualListRange::new(start, end),
            direction,
        }
    }

    #[test]
    fn background_worker_prefetches_and_deduplicates_pages() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let source_calls = Arc::clone(&calls);
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .cache_pages(4)
                .prefetch_pages(1)
                .initial_viewport_rows(5)
                .total_count_hint(100),
            move |request: PageRequest| {
                source_calls.lock().unwrap().push(request.page);
                Ok(Page::new((0..request.page_size).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(index, format!("Row {index}"))
                }))
                .total_count(100))
            },
        )
        .unwrap();
        state.update_viewport(viewport(0, 5, VirtualListScrollDirection::Forward));
        state.update_viewport(viewport(0, 5, VirtualListScrollDirection::Forward));

        assert!(state.wait_for_idle(Duration::from_secs(1)));
        let calls = calls.lock().unwrap().clone();
        assert_eq!(calls, vec![PageIndex(0), PageIndex(1)]);
        let snapshot = state.snapshot();
        assert_eq!(snapshot.cached_pages, 2);
        assert_eq!(snapshot.rows.len(), 10);
    }

    #[test]
    fn unknown_totals_extend_one_prefetch_page_without_loading_the_entire_source() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let source_calls = Arc::clone(&calls);
        let state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .cache_pages(3)
                .prefetch_pages(1)
                .initial_viewport_rows(5),
            move |request: PageRequest| {
                source_calls.lock().unwrap().push(request.page);
                Ok(Page::new((0..10).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(index, index)
                })))
            },
        )
        .unwrap();

        assert!(state.wait_for_idle(Duration::from_secs(1)));
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &[PageIndex(0), PageIndex(1)]
        );
        assert_eq!(state.cached_page_count(), 2);
    }

    #[test]
    fn returning_to_a_cached_page_does_not_query_the_source_again() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let source_calls = Arc::clone(&calls);
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .cache_pages(3)
                .prefetch_pages(0)
                .initial_viewport_rows(5)
                .total_count_hint(30),
            move |request: PageRequest| {
                source_calls.lock().unwrap().push(request.page);
                Ok(Page::new((0..10).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(index, index)
                }))
                .total_count(30))
            },
        )
        .unwrap();
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(viewport(10, 15, VirtualListScrollDirection::Forward));
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(viewport(0, 5, VirtualListScrollDirection::Backward));
        assert!(state.wait_for_idle(Duration::from_secs(1)));

        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &[PageIndex(0), PageIndex(1)]
        );
    }

    #[test]
    fn lru_cache_keeps_visible_pages_and_evicts_old_pages() {
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .cache_pages(2)
                .prefetch_pages(0)
                .initial_viewport_rows(5)
                .total_count_hint(100),
            |request: PageRequest| {
                Ok(Page::new((0..request.page_size).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(index, index)
                }))
                .total_count(100))
            },
        )
        .unwrap();
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(viewport(10, 15, VirtualListScrollDirection::Forward));
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(viewport(20, 25, VirtualListScrollDirection::Forward));
        assert!(state.wait_for_idle(Duration::from_secs(1)));

        let inner = state.lock();
        assert_eq!(inner.pages.len(), 2);
        assert!(!inner.pages.contains_key(&PageIndex(0)));
        assert!(inner.pages.contains_key(&PageIndex(1)));
        assert!(inner.pages.contains_key(&PageIndex(2)));
    }

    #[test]
    fn reset_discards_results_from_an_old_generation() {
        let started = Arc::new(AtomicUsize::new(0));
        let source_started = Arc::clone(&started);
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(4)
                .prefetch_pages(0)
                .initial_viewport_rows(4)
                .total_count_hint(4),
            move |request: PageRequest| {
                source_started.fetch_add(1, Ordering::SeqCst);
                if request.generation == 0 {
                    thread::sleep(Duration::from_millis(30));
                }
                Ok(
                    Page::new((0..4).map(|offset| PagedItem::new(offset, request.generation)))
                        .total_count(4),
                )
            },
        )
        .unwrap();
        while started.load(Ordering::SeqCst) == 0 {
            thread::yield_now();
        }
        state.reset(Some(4));

        assert!(state.wait_for_idle(Duration::from_secs(1)));
        let snapshot = state.snapshot();
        assert_eq!(snapshot.generation, 1);
        assert_eq!(snapshot.rows.len(), 4);
        assert!(snapshot.rows.iter().all(|(_, item)| item.value == 1));
    }

    #[test]
    fn loaded_visible_item_becomes_a_stable_scroll_anchor() {
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .prefetch_pages(0)
                .initial_viewport_rows(5)
                .total_count_hint(30),
            |request: PageRequest| {
                Ok(Page::new((0..request.page_size).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(format!("id-{index}"), index)
                }))
                .total_count(30))
            },
        )
        .unwrap();
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(VirtualListViewport {
            offset_y: Dp::new(44.0),
            row_height: Dp::new(20.0),
            visible_range: VirtualListRange::new(2, 7),
            materialized_range: VirtualListRange::new(1, 8),
            direction: VirtualListScrollDirection::Forward,
        });

        let anchor = state.snapshot().anchor.unwrap();
        assert_eq!(anchor.key, "id-2");
        assert_eq!(anchor.index, 2);
        assert_eq!(anchor.offset_within_row, Dp::new(4.0));
    }

    #[test]
    fn synced_reconcile_keeps_the_visible_key_at_the_same_pixel_position() {
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .prefetch_pages(0)
                .initial_viewport_rows(5)
                .total_count_hint(30),
            |request: PageRequest| {
                Ok(Page::new((0..request.page_size).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(format!("id-{index}"), index)
                }))
                .total_count(40))
            },
        )
        .unwrap();
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        state.update_viewport(VirtualListViewport {
            offset_y: Dp::new(44.0),
            row_height: Dp::new(20.0),
            visible_range: VirtualListRange::new(2, 7),
            materialized_range: VirtualListRange::new(1, 8),
            direction: VirtualListScrollDirection::Forward,
        });

        let report = state.reconcile_synced(40, |key| (key == "id-2").then_some(5));

        assert!(report.anchor_preserved());
        assert_eq!(report.previous_anchor_index, Some(2));
        assert_eq!(report.anchor_index, Some(5));
        assert_eq!(report.offset_y, Dp::new(104.0));
        let snapshot = state.snapshot();
        assert_eq!(snapshot.offset_y, Dp::new(104.0));
        assert_eq!(snapshot.visible_range, VirtualListRange::new(5, 10));
        assert_eq!(snapshot.anchor.unwrap().key, "id-2");
    }

    #[test]
    fn reversing_far_away_skips_obsolete_queued_prefetch_pages() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let source_calls = Arc::clone(&calls);
        let started = Arc::new(AtomicBool::new(false));
        let source_started = Arc::clone(&started);
        let release = Arc::new(AtomicBool::new(false));
        let source_release = Arc::clone(&release);
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(10)
                .cache_pages(8)
                .prefetch_pages(2)
                .initial_viewport_rows(5)
                .total_count_hint(100),
            move |request: PageRequest| {
                source_calls.lock().unwrap().push(request.page);
                if request.page == PageIndex(0) {
                    source_started.store(true, Ordering::Release);
                    while !source_release.load(Ordering::Acquire) {
                        thread::yield_now();
                    }
                }
                Ok(Page::new((0..request.page_size).map(|offset| {
                    let index = request.offset() + offset;
                    PagedItem::new(index, index)
                }))
                .total_count(100))
            },
        )
        .unwrap();
        while !started.load(Ordering::Acquire) {
            thread::yield_now();
        }
        state.update_viewport(viewport(80, 85, VirtualListScrollDirection::Forward));
        release.store(true, Ordering::Release);

        assert!(state.wait_for_idle(Duration::from_secs(1)));
        let calls = calls.lock().unwrap();
        assert!(calls.contains(&PageIndex(0)));
        assert!(calls.contains(&PageIndex(8)));
        assert!(!calls.contains(&PageIndex(1)));
        assert!(!calls.contains(&PageIndex(2)));
    }

    #[test]
    fn failed_page_can_be_retried_without_duplicate_in_flight_requests() {
        let calls = Arc::new(AtomicUsize::new(0));
        let source_calls = Arc::clone(&calls);
        let mut state = PagedListState::new(
            PagedListConfig::default()
                .page_size(5)
                .prefetch_pages(0)
                .initial_viewport_rows(5)
                .total_count_hint(5),
            move |_request: PageRequest| {
                if source_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err(ZsuiError::host("load_page", "temporary failure"))
                } else {
                    Ok(Page::new((0..5).map(|index| PagedItem::new(index, index))).total_count(5))
                }
            },
        )
        .unwrap();
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        assert!(state.snapshot().last_error.is_some());
        assert!(state.retry_failed());
        assert!(!state.retry_failed());
        assert!(state.wait_for_idle(Duration::from_secs(1)));
        assert_eq!(state.snapshot().rows.len(), 5);
    }
}
