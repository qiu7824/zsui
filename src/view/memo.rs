/// Work performed while preparing a retained View update.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewWorkStats {
    pub rebuilt_nodes: usize,
    pub reused_subtrees: usize,
    pub layout_nodes: usize,
    pub reused_layout_nodes: usize,
}

#[derive(Debug, Clone, Default)]
struct LayoutWork {
    visited: Arc<std::sync::atomic::AtomicUsize>,
    reused: Arc<std::sync::atomic::AtomicUsize>,
}
impl PartialEq for LayoutWork {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}
impl LayoutWork {
    fn snapshot(&self) -> (usize, usize) {
        use std::sync::atomic::Ordering;
        (
            self.visited.load(Ordering::Relaxed),
            self.reused.load(Ordering::Relaxed),
        )
    }
    fn visit(&self) {
        self.visited
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn reuse(&self, count: usize) {
        self.reused
            .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct MemoView<Msg> {
    revision: u64,
    build: Arc<dyn Fn() -> ViewNode<Msg> + Send + Sync>,
    materialized: bool,
    layout: Option<MemoLayout>,
    restore: Option<Box<ViewNode<Msg>>>,
}
impl<Msg> fmt::Debug for MemoView<Msg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoView")
            .field("revision", &self.revision)
            .field("materialized", &self.materialized)
            .finish()
    }
}
#[derive(Debug, Clone)]
struct MemoLayout {
    bounds: Rect,
    dpi: Dpi,
    scale: u16,
    style: ViewStyle,
    appearance: Option<Arc<crate::Appearance>>,
    measurements: Arc<ViewTextMeasurements>,
    text: Vec<crate::NativeDrawTextCommand>,
    output: LayoutOutput,
}

/// Lazily constructs a stable subtree only when its revision changes.
///
/// Every state dependency and captured callback dependency must participate in
/// `revision`. Changing bounds, DPI, typography or relevant measurements still
/// invalidates layout. A normal View remains the unconditional rebuild fallback.
pub fn memoized<Msg>(
    id: WidgetId,
    revision: u64,
    build: impl Fn() -> ViewNode<Msg> + Send + Sync + 'static,
) -> ViewNode<Msg> {
    let mut node = column([]).id(id);
    node.memo = Some(Box::new(MemoView {
        revision,
        build: Arc::new(build),
        materialized: false,
        layout: None,
        restore: None,
    }));
    node
}

impl<Msg: Clone> ViewNode<Msg> {
    fn node_count(&self) -> usize {
        1 + self.children.iter().map(Self::node_count).sum::<usize>()
    }
    fn collect_memos(&mut self, into: &mut std::collections::BTreeMap<WidgetId, ViewNode<Msg>>) {
        if self.memo.is_some() {
            if let Some(id) = self.id {
                let old = std::mem::replace(self, ViewNode::new(ViewNodeKind::Spacer));
                into.entry(id).or_insert(old);
                return;
            }
        }
        for child in &mut self.children {
            child.collect_memos(into);
        }
    }
    fn resolve_memos(
        &mut self,
        previous: &mut std::collections::BTreeMap<WidgetId, ViewNode<Msg>>,
        invalidated: &BTreeSet<WidgetId>,
        stats: &mut ViewWorkStats,
    ) {
        if let Some(memo) = self.memo.as_mut() {
            if !memo.materialized {
                let mut old = self.id.and_then(|id| previous.remove(&id));
                let reuse = old.as_ref().is_some_and(|old| {
                    old.memo
                        .as_ref()
                        .is_some_and(|old| old.revision == memo.revision)
                        && !old.contains_invalidated(invalidated)
                });
                if reuse {
                    let mut old = old.take().unwrap();
                    self.children = std::mem::take(&mut old.children);
                    memo.layout = old.memo.as_mut().and_then(|old| old.layout.take());
                    stats.reused_subtrees += 1;
                    memo.materialized = true;
                    return;
                } else {
                    if let Some(old) = old.as_mut() {
                        for child in &mut old.children {
                            child.collect_memos(previous);
                        }
                    }
                    self.children = vec![(memo.build)()];
                    memo.restore = old.map(Box::new);
                    stats.rebuilt_nodes += 1;
                }
                memo.materialized = true;
            }
        } else {
            stats.rebuilt_nodes += 1;
        }
        for child in &mut self.children {
            child.resolve_memos(previous, invalidated, stats);
        }
    }
    fn prepare_lazy(&mut self) {
        self.resolve_memos(
            &mut std::collections::BTreeMap::new(),
            &BTreeSet::new(),
            &mut ViewWorkStats::default(),
        );
    }
    fn restore_memo_state(&mut self) {
        #[allow(unused_variables)]
        if let Some(previous) = self.memo.as_mut().and_then(|memo| memo.restore.take()) {
            #[cfg(feature = "scroll")]
            self.restore_adaptive_scroll_state(&previous);
            #[cfg(feature = "label")]
            self.restore_navigation_state(&previous);
            #[cfg(feature = "context-menu")]
            {
                let mut state = Vec::new();
                previous.collect_context_menu_states(&mut state);
                self.restore_context_menu_states(&state);
            }
        }
        for child in &mut self.children {
            child.restore_memo_state();
        }
    }
    fn subtree_targets_event(&self, event: &ViewEvent) -> bool {
        self.event_targets_self(event)
            || self
                .children
                .iter()
                .any(|child| child.subtree_targets_event(event))
    }
    fn invalidate_memo_for_event(&mut self, event: &ViewEvent) {
        #[allow(unused_variables)]
        let global = false;
        #[cfg(any(
            feature = "flyout",
            feature = "menu-flyout",
            feature = "combo",
            feature = "date-picker",
            feature = "time-picker",
            feature = "color-picker",
            feature = "auto-suggest",
            feature = "breadcrumb"
        ))]
        let global = matches!(event, ViewEvent::DismissPopupOverlays { .. });
        if self.memo.is_some() && (global || self.subtree_targets_event(event)) {
            self.memo.as_mut().unwrap().layout = None;
        }
    }
    fn contains_invalidated(&self, ids: &BTreeSet<WidgetId>) -> bool {
        self.id.is_some_and(|id| ids.contains(&id))
            || self
                .children
                .iter()
                .any(|child| child.contains_invalidated(ids))
    }

    fn cached_memo_layout(&self, cx: &ViewLayoutCx) -> Option<LayoutOutput> {
        let cached = self.memo.as_ref()?.layout.as_ref()?;
        if cached.bounds != cx.bounds
            || cached.dpi != cx.dpi
            || cached.scale != cx.typography_scale_per_mille
            || cached.style != self.style
            || cached.appearance.as_ref() != self.effective_appearance()
        {
            return None;
        }
        if !Arc::ptr_eq(&cached.measurements, &cx.text_measurements) {
            for text in &cached.text {
                for width in [0, text.bounds.width] {
                    if cached
                        .measurements
                        .with_font(text.font.clone())
                        .measure(&text.text, text.style, width)
                        != cx
                            .text_measurements
                            .with_font(text.font.clone())
                            .measure(&text.text, text.style, width)
                    {
                        return None;
                    }
                }
            }
        }
        Some(cached.output.clone())
    }
    fn store_memo_layout(&mut self, cx: &ViewLayoutCx, output: &LayoutOutput) {
        if self.memo.is_none() {
            return;
        }
        let mut paint = ViewPaintCx::new(cx.dpi);
        self.paint(&mut paint);
        let text = paint
            .into_plan()
            .commands
            .into_iter()
            .filter_map(|command| match command {
                NativeDrawCommand::Text(text) => Some(text),
                _ => None,
            })
            .collect();
        let appearance = self.effective_appearance().cloned();
        self.memo.as_mut().unwrap().layout = Some(MemoLayout {
            bounds: cx.bounds,
            dpi: cx.dpi,
            scale: cx.typography_scale_per_mille,
            style: self.style,
            appearance,
            measurements: cx.text_measurements.clone(),
            text,
            output: output.clone(),
        });
    }
}
