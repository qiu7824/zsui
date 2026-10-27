use super::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct AllocationProbe;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

fn allocated() {
    let _ = ALLOCATIONS.try_with(|counter| {
        if let Some(count) = counter.get() {
            counter.set(Some(count + 1));
        }
    });
}

// Test-only instrumentation delegates every request to the process allocator.
// Counts are opt-in and local to the measuring thread, so unrelated native
// and parallel tests cannot contaminate the assertion.
unsafe impl GlobalAlloc for AllocationProbe {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated();
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocated();
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        allocated();
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: AllocationProbe = AllocationProbe;

struct Counting;
impl Drop for Counting {
    fn drop(&mut self) {
        ALLOCATIONS.with(|counter| counter.set(None));
    }
}

#[test]
fn retained_layout_hits_do_not_allocate_text_or_font_keys() {
    let mut engine = ZsRustTextEngine::new();
    let style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
    let first = engine.layout("Retained 中文 العربية", &style, 640, 48, 1.0);
    ALLOCATIONS.with(|counter| counter.set(Some(0)));
    let guard = Counting;
    for _ in 0..100 {
        let cached = engine.layout("Retained 中文 العربية", &style, 640, 48, 1.0);
        assert!(Arc::ptr_eq(&first, &cached));
    }
    let count = ALLOCATIONS.with(|counter| counter.get().unwrap());
    drop(guard);
    assert_eq!(count, 0, "cache hits must borrow text and family names");
}

#[cfg(windows)]
#[test]
fn shared_font_bytes_keep_window_dpi_caches_and_retained_layouts_independent() {
    let _sharing = font_files::SharingScope::for_windows(3);
    fn pixels(engine: &mut ZsRustTextEngine, layout: &ZsRustTextLayout) -> Vec<u8> {
        let mut pixels = vec![255; 640 * 160 * 4];
        engine
            .composite_bgra(
                layout,
                &mut pixels,
                640,
                160,
                640 * 4,
                Color::rgb(20, 30, 40),
                ZsTextRasterProfile::grayscale(),
            )
            .unwrap();
        pixels
    }
    let mut first =
        ZsRustTextEngine::new().with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
    let mut second =
        ZsRustTextEngine::new().with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
    let mut third =
        ZsRustTextEngine::new().with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
    assert!(Arc::ptr_eq(
        first._font_files.as_ref().unwrap(),
        second._font_files.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        first._font_files.as_ref().unwrap(),
        third._font_files.as_ref().unwrap()
    ));
    let style = TextStyle::line("Segoe UI", 18.0, Color::rgb(20, 30, 40));
    let original = first.layout("Window 中文 العربية שלום 🦀", &style, 640, 160, 1.0);
    let expected = pixels(&mut first, &original);
    let second_layout = second.layout("Window 中文 العربية שלום 🦀", &style, 640, 160, 2.0);
    let third_layout = third.layout("Window 中文 العربية שלום 🦀", &style, 640, 160, 1.5);
    assert_ne!(expected, pixels(&mut second, &second_layout));
    assert_ne!(expected, pixels(&mut third, &third_layout));
    drop(second);
    drop(third);
    first.clear_layout_cache();
    first.clear_glyph_cache();
    assert_eq!(pixels(&mut first, &original), expected);
    assert!(!original.glyphs().is_empty());
}
