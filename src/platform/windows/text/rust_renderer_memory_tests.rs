use super::*;
use crate::windows_gdi_renderer::{
    WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette, WindowsGdiRenderer,
    WindowsGdiResourceCache,
};
use crate::{
    NativeDrawCommand, NativeDrawPlan, NativeDrawTextCommand, Renderer, SemanticTextStyle,
};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, LPtoDP, SelectClipRgn, RGN_OR};

fn assert_same_pixels(actual: &[u8], expected: &[u8]) {
    assert_eq!(actual.len(), expected.len());
    let differences: Vec<_> = actual
        .iter()
        .zip(expected)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(8)
        .map(|(i, (a, b))| (i, *a, *b))
        .collect();
    assert!(
        differences.is_empty(),
        "first different BGRA bytes: {differences:?}"
    );
}

fn scene_style() -> TextStyle {
    TextStyle {
        font_family: "Segoe UI".into(),
        size: 17.0,
        line_height: 23.0,
        semantic_role: Some(crate::TextRole::Body),
        weight: crate::TextWeight::Regular,
        color: crate::Color::rgb(23, 47, 91),
        horizontal_align: crate::HorizontalAlign::Start,
        vertical_align: crate::VerticalAlign::Start,
        wrap: crate::TextWrap::Word,
        ellipsis: false,
    }
}

#[test]
fn scratch_tiles_match_contiguous_rust_pixels_across_clip_boundaries() {
    let width = 800;
    let height = 260;
    let mut target = WindowsRustTextBitmap::new(std::ptr::null_mut(), width, height).unwrap();
    for pixel in target.pixels_mut().unwrap().chunks_exact_mut(4) {
        pixel.copy_from_slice(&[231, 239, 247, 0]);
    }
    let mut expected = target.pixels_mut().unwrap().to_vec();
    let clip = ZsTextPixelRect::new(17, 13, 742, 210);
    let region = unsafe { CreateRectRgn(17, 13, 759, 223) };
    assert!(!region.is_null());
    unsafe {
        SelectClipRgn(target.dc, region);
        DeleteObject(region as HGDIOBJ);
    }
    let run = TextRun {
        text: "Rust 中文 العربية שלום 🦀 — clipping and retained glyphs\n".repeat(16),
        bounds: Rect {
            x: -31,
            y: -45,
            width: 900,
            height: 370,
        },
    };
    let style = scene_style();
    let mut actual = WindowsRustTextState::default();
    let mut reference =
        ZsRustTextEngine::new().with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
    let layout = reference.layout(&run.text, &style, run.bounds.width, run.bounds.height, 1.25);
    reference
        .composite_bgra_clipped(
            &layout,
            &mut expected,
            width,
            height,
            width as usize * 4,
            run.bounds.x,
            run.bounds.y,
            clip,
            style.color,
            actual.profile,
        )
        .unwrap();
    assert!(actual.draw_text(target.dc, &run, &style, 1.25));
    unsafe {
        GdiFlush();
    }
    assert_same_pixels(target.pixels_mut().unwrap(), &expected);
    assert!(
        expected
            .chunks_exact(4)
            .filter(|p| p[..3] != [231, 239, 247])
            .count()
            > 100
    );
    assert!(actual.stats().bitmap_bytes > 0);
    assert!(actual.stats().max_bitmap_bytes <= 128 * 1024);
}

fn render_native_scene(buffered: bool, complex_clip: bool) -> (Vec<u8>, WindowsRustTextStats) {
    let mut target = WindowsRustTextBitmap::new(std::ptr::null_mut(), 800, 400).unwrap();
    for pixel in target.pixels_mut().unwrap().chunks_exact_mut(4) {
        pixel.copy_from_slice(&[193, 211, 229, 0]);
    }
    let paint_rect = RECT {
        left: 16,
        top: 8,
        right: 784,
        bottom: 392,
    };
    let resources = WindowsGdiResourceCache::default();
    let mut paint =
        buffered.then(|| unsafe { WindowsBufferedPaint::begin(target.dc, &paint_rect).unwrap() });
    let dc = paint.as_ref().map_or(target.dc, WindowsBufferedPaint::hdc);
    let mut renderer =
        WindowsGdiRenderer::with_dpi_and_resources(dc, crate::Dpi::standard(), resources.clone());
    renderer.fill_rect(
        Rect {
            x: 16,
            y: 8,
            width: 768,
            height: 384,
        },
        crate::Color::rgb(245, 241, 235),
    );
    drop(renderer);
    if complex_clip {
        unsafe {
            // SelectClipRgn consumes device coordinates, unlike PushClip's
            // logical rectangles. Buffered DCs have a translated viewport.
            let mut corners = [
                POINT { x: 32, y: 20 },
                POINT { x: 360, y: 370 },
                POINT { x: 400, y: 20 },
                POINT { x: 760, y: 370 },
            ];
            assert_ne!(LPtoDP(dc, corners.as_mut_ptr(), corners.len() as i32), 0);
            let first = CreateRectRgn(corners[0].x, corners[0].y, corners[1].x, corners[1].y);
            let second = CreateRectRgn(corners[2].x, corners[2].y, corners[3].x, corners[3].y);
            assert!(!first.is_null() && !second.is_null());
            assert_ne!(CombineRgn(first, first, second, RGN_OR), 0);
            SelectClipRgn(dc, first);
            DeleteObject(first as HGDIOBJ);
            DeleteObject(second as HGDIOBJ);
        }
    }
    let mut semantic = SemanticTextStyle::body();
    semantic.wrap = crate::TextWrap::Word;
    semantic.ellipsis = false;
    let plan = NativeDrawPlan::new([
        NativeDrawCommand::PushClip {
            rect: Rect {
                x: 25,
                y: 17,
                width: 742,
                height: 350,
            },
        },
        NativeDrawCommand::Text(NativeDrawTextCommand::new(
            "Native pixels 中文 العربية שלום 🦀\n".repeat(14),
            Rect {
                x: -12,
                y: -9,
                width: 780,
                height: 400,
            },
            semantic,
        )),
        NativeDrawCommand::PopClip,
    ]);
    let mut sink = WindowsGdiDrawSink::with_palette_contrast_dpi_and_resources(
        dc,
        WindowsGdiPalette::default(),
        false,
        crate::Dpi::standard(),
        resources.clone(),
    );
    if let Some(paint) = paint.as_mut() {
        sink.draw_native_plan_buffered(&plan, paint);
    } else {
        sink.draw_native_plan(&plan);
    }
    assert_eq!(sink.operation_log().len(), plan.commands.len());
    drop(sink);
    drop(paint);
    unsafe {
        GdiFlush();
    }
    let pixels = target.pixels_mut().unwrap().to_vec();
    assert!(
        (20..370)
            .flat_map(|y| (32..760).map(move |x| (y * 800 + x) * 4))
            .filter(|i| pixels[*i..*i + 3] != [235, 241, 245])
            .count()
            > 100
    );
    (pixels, resources.rust_text_snapshot())
}

#[test]
fn borrowed_final_surface_matches_native_pixels_without_scratch_allocation() {
    let (expected, _) = render_native_scene(false, false);
    let (actual, stats) = render_native_scene(true, false);
    assert_same_pixels(&actual, &expected);
    assert_eq!(stats.direct_draws, 1);
    assert_eq!(stats.bitmap_bytes, 0);
}

#[test]
fn complex_native_clip_uses_bounded_fallback_without_overpainting_holes() {
    let (expected, _) = render_native_scene(false, true);
    let (actual, stats) = render_native_scene(true, true);
    assert_same_pixels(&actual, &expected);
    assert_eq!(stats.direct_draws, 0);
    assert!(stats.max_bitmap_bytes <= 128 * 1024);
}

#[test]
fn released_text_resources_reinitialize_lazily_without_affecting_other_windows() {
    let style = scene_style();
    let mut first = WindowsRustTextState::default();
    let mut second = WindowsRustTextState::default();
    first.measure("Window one 中文", &style, 300, 1.0).unwrap();
    second.measure("Window two 中文", &style, 300, 1.0).unwrap();
    assert!(first.has_retained_resources() && second.has_retained_resources());
    first.release_idle_memory();
    assert!(!first.has_retained_resources());
    assert!(second.has_retained_resources());
    assert_eq!(first.stats().bitmap_bytes, 0);
    first.measure("Window one 中文", &style, 300, 1.0).unwrap();
    assert_eq!(first.stats().engine_initializations, 2);
    assert_eq!(second.stats().engine_initializations, 1);
}
