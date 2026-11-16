use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn nested_invalidation_rebuilds_the_owner_and_reuses_the_untouched_sibling() {
    fn tree(counts: Arc<[AtomicUsize; 3]>) -> ViewNode<()> {
        memoized(WidgetId(10), 0, move || {
            counts[0].fetch_add(1, Ordering::SeqCst);
            let left = counts.clone();
            let right = counts.clone();
            column([
                memoized(WidgetId(11), 0, move || {
                    left[1].fetch_add(1, Ordering::SeqCst);
                    crate::text("left").id(WidgetId(201))
                }),
                memoized(WidgetId(12), 0, move || {
                    right[2].fetch_add(1, Ordering::SeqCst);
                    crate::text("right").id(WidgetId(202))
                }),
            ])
        })
    }
    let counts = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
    let bounds = Rect {
        x: 0,
        y: 0,
        width: 300,
        height: 160,
    };
    let mut before = tree(counts.clone());
    let mut previous = std::collections::BTreeMap::new();
    let mut initial = ViewWorkStats::default();
    before.resolve_memos(&mut previous, &BTreeSet::new(), &mut initial);
    let mut cx = ViewLayoutCx::new(bounds, Dpi::standard());
    before.layout(&mut cx);
    let right_bounds = before.children[0].children[1].children[0].bounds();
    before.collect_memos(&mut previous);

    let mut after = tree(counts.clone());
    let mut update = ViewWorkStats::default();
    let started = std::time::Instant::now();
    after.resolve_memos(&mut previous, &BTreeSet::from([WidgetId(11)]), &mut update);
    let mut cx = ViewLayoutCx::new(bounds, Dpi::standard());
    after.layout(&mut cx);
    let elapsed = started.elapsed();
    assert_eq!(
        counts.each_ref().map(|count| count.load(Ordering::SeqCst)),
        [2, 2, 1]
    );
    assert_eq!(update.reused_subtrees, 1);
    assert!(update.rebuilt_nodes < initial.rebuilt_nodes);
    assert_eq!(
        after.children[0].children[1].children[0].bounds(),
        right_bounds
    );
    assert!(cx.work.snapshot().1 > 0);
    if let Some(directory) = std::env::var_os("ZSUI_LAYOUT_PROOF_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(directory.is_absolute());
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("memo-work.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"zsui.memo-work/v1", "initial_rebuilt_nodes":initial.rebuilt_nodes,
            "update_rebuilt_nodes":update.rebuilt_nodes,"reused_subtrees":update.reused_subtrees,
            "layout_nodes":cx.work.snapshot().0,"reused_layout_nodes":cx.work.snapshot().1,
            "model_update_and_layout_microseconds":elapsed.as_micros(),"native_frame_time_measured":false
        })).unwrap()).unwrap();
    }
}

#[test]
#[cfg(feature = "scroll")]
fn revision_changes_preserve_scroll_state_after_layout() {
    let tree = |revision| {
        memoized(WidgetId(30), revision, || {
            column(
                (0..20)
                    .map(|index| crate::text::<()>(format!("Row {index}")).height(Dp::new(30.0))),
            )
            .auto_scroll_y()
            .id(WidgetId(31))
        })
    };
    let bounds = Rect {
        x: 0,
        y: 0,
        width: 300,
        height: 120,
    };
    let mut before = tree(0);
    before.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
    before.event(
        &mut ViewEventCx::default(),
        &ViewEvent::ScrollBy {
            widget: WidgetId(31),
            delta_y: Dp::new(60.0),
        },
    );
    let offset = before.children[0].adaptive_scroll_offset_y;
    assert!(offset.0 > 0.0);
    let mut previous = std::collections::BTreeMap::new();
    before.collect_memos(&mut previous);
    let mut after = tree(1);
    after.resolve_memos(
        &mut previous,
        &BTreeSet::new(),
        &mut ViewWorkStats::default(),
    );
    after.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
    assert_eq!(after.children[0].adaptive_scroll_offset_y, offset);
}

#[test]
#[cfg(feature = "label")]
fn memoized_revision_reuses_siblings_and_explicit_invalidation_rebuilds() {
    let builds = Arc::new(AtomicUsize::new(0));
    let left = builds.clone();
    let mut first = column([
        memoized(WidgetId(1), 0, move || {
            left.fetch_add(1, Ordering::SeqCst);
            crate::text::<()>("retained")
        }),
        crate::text("initial"),
    ]);
    let mut cx = ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 300,
            height: 200,
        },
        Dpi::standard(),
    );
    first.layout(&mut cx);
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    let mut previous = std::collections::BTreeMap::new();
    first.collect_memos(&mut previous);
    let left = builds.clone();
    let mut next = column([
        memoized(WidgetId(1), 0, move || {
            left.fetch_add(1, Ordering::SeqCst);
            crate::text("retained")
        }),
        crate::text("changed"),
    ]);
    let mut stats = ViewWorkStats::default();
    next.resolve_memos(&mut previous, &BTreeSet::new(), &mut stats);
    next.layout(&mut cx);
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    assert_eq!(stats.reused_subtrees, 1);
    assert!(cx.work.snapshot().1 > 0);
    next.collect_memos(&mut previous);
    let left = builds.clone();
    let mut forced = memoized(WidgetId(1), 0, move || {
        left.fetch_add(1, Ordering::SeqCst);
        crate::text("updated")
    });
    forced.resolve_memos(&mut previous, &BTreeSet::from([WidgetId(1)]), &mut stats);
    assert_eq!(builds.load(Ordering::SeqCst), 2);
}

#[test]
#[cfg(feature = "label")]
fn layout_cache_rechecks_relevant_text_metrics_and_dpi() {
    let mut node = memoized(WidgetId(4), 0, || crate::text::<()>("measured"));
    let mut cx = ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 60,
        },
        Dpi::standard(),
    );
    node.layout(&mut cx);
    let mut measurements = ViewTextMeasurements::default();
    measurements.insert(
        "unrelated",
        crate::SemanticTextStyle::body(),
        0,
        crate::Size {
            width: 70,
            height: 20,
        },
    );
    cx.text_measurements = Arc::new(measurements.clone());
    assert!(node.cached_memo_layout(&cx).is_some());
    measurements.insert(
        "measured",
        crate::SemanticTextStyle::body(),
        0,
        crate::Size {
            width: 80,
            height: 24,
        },
    );
    cx.text_measurements = Arc::new(measurements);
    assert!(node.cached_memo_layout(&cx).is_none());
    node.layout(&mut cx);
    cx.dpi = Dpi::new(144.0);
    assert!(node.cached_memo_layout(&cx).is_none());
}
