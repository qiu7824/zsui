use crate::{
    ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect,
    ViewInteractionPlan, WidgetId,
};

pub(crate) fn decorate_native_focus_ring(
    plan: &mut NativeDrawPlan,
    interaction_plan: &ViewInteractionPlan,
    focused_widget: Option<WidgetId>,
    dpi: Dpi,
) -> Option<Rect> {
    let target = interaction_plan.hit_target_for_widget(focused_widget?)?;
    let requested_inset = Dp::new(1.0).to_px(dpi).round_i32().max(1);
    let maximum_inset = (target.bounds.width.min(target.bounds.height).max(1) - 1) / 2;
    let inset = requested_inset.min(maximum_inset.max(0));
    let ring = Rect {
        x: target.bounds.x.saturating_add(inset),
        y: target.bounds.y.saturating_add(inset),
        width: target.bounds.width.saturating_sub(inset.saturating_mul(2)),
        height: target.bounds.height.saturating_sub(inset.saturating_mul(2)),
    };
    let width = Dp::new(2.0).to_px(dpi).round_i32().max(1);
    plan.push(NativeDrawCommand::StrokeRect {
        rect: ring,
        stroke: NativeDrawFill::Role(ColorRole::Accent),
        width,
    });
    Some(ring)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ViewHitTarget, ViewHitTargetKind};

    #[test]
    fn focus_ring_uses_semantic_accent_and_insets_target_bounds() {
        let widget = WidgetId::new(91);
        let interaction_plan = ViewInteractionPlan::new([ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 10,
                y: 20,
                width: 120,
                height: 32,
            },
            ViewHitTargetKind::Button,
        )]);
        let mut plan = NativeDrawPlan::default();

        let ring =
            decorate_native_focus_ring(&mut plan, &interaction_plan, Some(widget), Dpi::standard())
                .expect("focused target should produce a ring");

        assert_eq!(ring.x, 11);
        assert_eq!(ring.y, 21);
        assert_eq!(ring.width, 118);
        assert_eq!(ring.height, 30);
        assert!(matches!(
            plan.commands.as_slice(),
            [NativeDrawCommand::StrokeRect {
                rect,
                stroke: NativeDrawFill::Role(ColorRole::Accent),
                width: 2,
            }] if *rect == ring
        ));
    }
}
