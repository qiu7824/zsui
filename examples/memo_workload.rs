//! Headless update and layout benchmark for a static list beside a live counter.
#[path = "support/memory_allocation_probe.rs"]
mod allocations;

use std::time::Instant;
use zsui::{
    button, column, live_view_runtime, memoized, row, text, Dpi, Rect, ViewEvent, WidgetId,
};

#[global_allocator]
static ALLOCATOR: allocations::AllocationProbe = allocations::AllocationProbe;

fn main() {
    let runtime = live_view_runtime(
        0usize,
        |count| {
            row([
                memoized(WidgetId(1), 1, || {
                    column((0..1000).map(|i| text(format!("Item {i}")).id(WidgetId(100 + i))))
                }),
                button(format!("Counter {count}"))
                    .id(WidgetId(2))
                    .on_click(()),
            ])
        },
        |count, (), _| *count += 1,
        Rect {
            x: 0,
            y: 0,
            width: 1000,
            height: 720,
        },
        Dpi::standard(),
    );
    let update = || {
        let result = runtime.dispatch_event(&ViewEvent::Click {
            widget: WidgetId(2),
        });
        assert_eq!(result.message_count, 1);
        assert_eq!(result.work.reused_subtrees, 1);
    };
    for _ in 0..32 {
        update();
    }
    let mut times = [0u64; 256];
    let before = allocations::snapshot();
    for time in &mut times {
        let start = Instant::now();
        update();
        *time = start.elapsed().as_nanos() as u64;
    }
    let after = allocations::snapshot();
    times.sort_unstable();
    println!(
        "{}",
        serde_json::json!({
            "updates": times.len(), "p50_ns": times[127], "p95_ns": times[243],
            "before": before, "after": after,
            "process": zsui::native_smoke::capture_native_process_memory("warm"),
        })
    );
}
