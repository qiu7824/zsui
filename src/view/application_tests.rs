use super::*;

#[test]
fn application_feedback_asymmetric_padding_matches_layout_and_measurement() {
    for dpi in [Dpi::standard(), Dpi::new(144.0), Dpi::new(192.0)] {
        let mut view: ViewNode<()> = column([spacer().flex(1.0)])
            .padding_sides(Dp::new(4.0), Dp::new(6.0), Dp::new(10.0), Dp::new(12.0))
            .align(ViewAlign::Stretch);
        let bounds = Rect {
            x: 20,
            y: 30,
            width: 200,
            height: 100,
        };
        view.layout(&mut ViewLayoutCx::new(bounds, dpi));
        let px = |value: f32| Dp::new(value).to_px(dpi).round_i32();
        assert_eq!(
            view.children[0].bounds(),
            Some(Rect {
                x: 20 + px(4.0),
                y: 30 + px(6.0),
                width: 200 - px(4.0) - px(10.0),
                height: 100 - px(6.0) - px(12.0),
            })
        );
        let measured: ViewNode<()> = column([spacer().width(Dp::new(20.0)).height(Dp::new(30.0))])
            .padding_xy(Dp::new(7.0), Dp::new(9.0));
        let measurements = ViewTextMeasurements::default();
        assert_eq!(
            intrinsic_width_px(
                &measured,
                dpi,
                1.0,
                &measurements,
                WidthMeasurement::Minimum
            ),
            px(20.0) + 2 * px(7.0)
        );
        assert_eq!(
            intrinsic_min_height_px(&measured, 200, dpi, 1.0, &measurements),
            px(30.0) + 2 * px(9.0)
        );
    }
}

#[test]
fn application_feedback_padding_last_modifier_wins() {
    let node: ViewNode<()> = spacer()
        .padding_xy(Dp::new(8.0), Dp::new(2.0))
        .padding(Dp::new(3.0));
    assert_eq!(node.style.resolved_padding(), Some([Dp::new(3.0); 4]));
    let node = node.padding_xy(Dp::new(5.0), Dp::new(1.0));
    assert_eq!(
        node.style.resolved_padding(),
        Some([Dp::new(5.0), Dp::new(1.0), Dp::new(5.0), Dp::new(1.0)])
    );
}

#[test]
fn application_feedback_border_preserves_content_and_uses_theme_color() {
    let mut node: ViewNode<()> = column([spacer().bg(ThemeColorToken::Accent).flex(1.0)])
        .radius(Dp::new(8.0))
        .border(ThemeColorToken::Border);
    let bounds = Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 60,
    };
    node.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
    assert_eq!(node.children[0].bounds(), Some(bounds));
    let mut cx = ViewPaintCx::new(Dpi::standard());
    node.paint(&mut cx);
    let commands = &cx.plan().commands;
    assert!(matches!(
        commands[0],
        NativeDrawCommand::FillRect {
            fill: NativeDrawFill::Role(ColorRole::Accent),
            ..
        }
    ));
    assert!(commands.iter().any(|command| matches!(command,
        NativeDrawCommand::RoundRect {
            rect, fill: NativeDrawFill::RoleWithAlpha { alpha: 0, .. },
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)), radius: 8,
        } if *rect == bounds)));
}

#[cfg(feature = "label")]
#[test]
fn application_feedback_label_wrap_changes_measurement_and_paint() {
    let content = "A long label that wraps across several lines in a narrow column";
    let make = |wrap| {
        column([
            text::<()>(content).text_wrap(wrap),
            spacer().height(Dp::new(10.0)),
        ])
        .align(ViewAlign::Stretch)
    };
    let bounds = Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 400,
    };
    let mut wrapped = make(crate::TextWrap::Word);
    let mut single = make(crate::TextWrap::NoWrap);
    wrapped.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
    single.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
    assert!(wrapped.children[1].bounds().unwrap().y > single.children[1].bounds().unwrap().y);
    let mut cx = ViewPaintCx::new(Dpi::standard());
    wrapped.paint(&mut cx);
    assert!(cx.plan().commands.iter().any(|command| matches!(command,
        NativeDrawCommand::Text(text) if text.text == content && text.style.wrap == crate::TextWrap::Word)));
}

#[cfg(feature = "textbox")]
#[test]
fn application_feedback_wrap_keeps_single_line_input_contract() {
    let input: ViewNode<()> = textbox("A line").text_wrap(crate::TextWrap::Word);
    assert!(matches!(
        input.kind,
        ViewNodeKind::Textbox {
            multiline: false,
            wrap: crate::TextWrap::NoWrap,
            ..
        }
    ));
    let editor: ViewNode<()> = text_editor("Two lines").text_wrap(crate::TextWrap::NoWrap);
    assert!(matches!(
        editor.kind,
        ViewNodeKind::Textbox {
            multiline: true,
            wrap: crate::TextWrap::NoWrap,
            ..
        }
    ));
}

#[cfg(all(feature = "scroll", feature = "label"))]
#[test]
fn application_feedback_scroll_survives_rebuild_and_respects_controlled_reset() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mode = Arc::new(AtomicUsize::new(0));
    let state = mode.clone();
    let id = WidgetId::new(71001);
    let runtime = live_view_runtime(
        (),
        move |_| {
            let mode = state.load(Ordering::SeqCst);
            let mut node =
                scroll(text("Top").height(Dp::new(if mode == 2 { 20.0 } else { 400.0 }))).id(id);
            if mode == 1 {
                node = node.scroll_y(Dp::new(0.0));
            }
            node
        },
        |_, _: (), _| {},
        Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 80,
        },
        Dpi::standard(),
    );
    let text_y = || {
        runtime
            .draw_plan()
            .commands
            .into_iter()
            .find_map(|command| match command {
                NativeDrawCommand::Text(text) if text.text == "Top" => Some(text.bounds.y),
                _ => None,
            })
            .unwrap()
    };
    runtime.dispatch_event(&ViewEvent::ScrollBy {
        widget: id,
        delta_y: Dp::new(44.0),
    });
    assert_eq!(text_y(), -44);
    runtime.refresh();
    assert_eq!(text_y(), -44);
    mode.store(1, Ordering::SeqCst);
    runtime.refresh();
    assert_eq!(text_y(), 0);
    mode.store(0, Ordering::SeqCst);
    runtime.dispatch_event(&ViewEvent::ScrollBy {
        widget: id,
        delta_y: Dp::new(44.0),
    });
    runtime.refresh();
    assert_eq!(text_y(), -44);
    mode.store(2, Ordering::SeqCst);
    runtime.refresh();
    assert_eq!(text_y(), 0);
}

#[cfg(feature = "scroll")]
#[test]
fn application_feedback_scroll_does_not_transfer_between_different_ids() {
    let mut previous: ViewNode<()> = scroll(spacer())
        .id(WidgetId::new(71002))
        .scroll_y(Dp::new(40.0));
    let mut next: ViewNode<()> = scroll(spacer()).id(WidgetId::new(71003));
    next.restore_adaptive_scroll_state(&previous);
    assert!(matches!(next.kind, ViewNodeKind::Scroll { offset_y, .. } if offset_y == Dp::new(0.0)));
    previous.id = next.id;
    next.restore_adaptive_scroll_state(&previous);
    assert!(
        matches!(next.kind, ViewNodeKind::Scroll { offset_y, .. } if offset_y == Dp::new(40.0))
    );
}
