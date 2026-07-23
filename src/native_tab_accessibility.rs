use std::collections::BTreeMap;

use crate::{
    Dpi, NativeDrawCommand, NativeDrawPlan, Rect, ViewHitTarget, ViewHitTargetKind,
    ViewInteractionPlan, WidgetId, ZsTabId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeTabAccessibilityItem {
    pub(crate) target: ViewHitTarget,
    pub(crate) label: String,
    pub(crate) selected: bool,
    pub(crate) focused: bool,
    pub(crate) position: usize,
    pub(crate) count: usize,
}

impl NativeTabAccessibilityItem {
    pub(crate) const fn tab(&self) -> ZsTabId {
        match self.target.kind {
            ViewHitTargetKind::Tab { tab, .. } => tab,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeTabAccessibilitySnapshot {
    pub(crate) tab_view: WidgetId,
    pub(crate) list_bounds: Rect,
    pub(crate) panel_bounds: Rect,
    pub(crate) items: Vec<NativeTabAccessibilityItem>,
}

impl NativeTabAccessibilitySnapshot {
    pub(crate) fn selected_item(&self) -> Option<&NativeTabAccessibilityItem> {
        self.items.iter().find(|item| item.selected)
    }

    pub(crate) fn focused_item(&self) -> Option<&NativeTabAccessibilityItem> {
        self.items.iter().find(|item| item.focused)
    }

    #[cfg(any(windows, target_os = "macos"))]
    pub(crate) fn item(&self, tab: ZsTabId) -> Option<&NativeTabAccessibilityItem> {
        self.items.iter().find(|item| item.tab() == tab)
    }
}

pub(crate) fn native_tab_accessibility_snapshots(
    plan: &NativeDrawPlan,
    interaction: &ViewInteractionPlan,
    focused_widget: Option<WidgetId>,
    dpi: Dpi,
    mut tab_view_bounds: impl FnMut(WidgetId) -> Option<Rect>,
    mut tab_selected: impl FnMut(WidgetId) -> Option<bool>,
) -> Vec<NativeTabAccessibilitySnapshot> {
    let mut groups = BTreeMap::<u64, Vec<ViewHitTarget>>::new();
    for target in interaction.hit_targets.iter().copied() {
        let ViewHitTargetKind::Tab { tab_view, .. } = target.kind else {
            continue;
        };
        groups.entry(tab_view.0).or_default().push(target);
    }

    groups
        .into_iter()
        .filter_map(|(_, mut targets)| {
            targets.sort_by_key(|target| match target.kind {
                ViewHitTargetKind::Tab { index, .. } => index,
                _ => usize::MAX,
            });
            let tab_view = match targets.first()?.kind {
                ViewHitTargetKind::Tab { tab_view, .. } => tab_view,
                _ => return None,
            };
            let list_bounds = targets.iter().fold(None, |bounds, target| {
                Some(union_rect(bounds, target.bounds))
            })?;
            let view_bounds = tab_view_bounds(tab_view).unwrap_or(list_bounds);
            let strip_height =
                crate::ZsTabViewMetrics::for_platform(crate::ZsTabPlatformStyle::current())
                    .strip_height
                    .to_px(dpi)
                    .round_i32()
                    .clamp(0, view_bounds.height.max(0));
            let panel_bounds = Rect {
                x: view_bounds.x,
                y: view_bounds.y.saturating_add(strip_height),
                width: view_bounds.width,
                height: view_bounds.height.saturating_sub(strip_height),
            };
            let count = targets.len();
            let items = targets
                .into_iter()
                .enumerate()
                .map(|(position, target)| NativeTabAccessibilityItem {
                    target,
                    label: native_accessible_label(plan, target),
                    selected: tab_selected(target.widget).unwrap_or(false),
                    focused: focused_widget == Some(target.widget),
                    position: position.saturating_add(1),
                    count,
                })
                .collect::<Vec<_>>();
            Some(NativeTabAccessibilitySnapshot {
                tab_view,
                list_bounds,
                panel_bounds,
                items,
            })
        })
        .collect()
}

fn union_rect(current: Option<Rect>, next: Rect) -> Rect {
    let Some(current) = current else {
        return next;
    };
    let left = current.x.min(next.x);
    let top = current.y.min(next.y);
    let right = current
        .x
        .saturating_add(current.width.max(0))
        .max(next.x.saturating_add(next.width.max(0)));
    let bottom = current
        .y
        .saturating_add(current.height.max(0))
        .max(next.y.saturating_add(next.height.max(0)));
    Rect {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    }
}

fn native_accessible_label(plan: &NativeDrawPlan, target: ViewHitTarget) -> String {
    let mut best: Option<(i64, &str)> = None;
    for command in &plan.commands {
        let NativeDrawCommand::Text(text) = command else {
            continue;
        };
        let value = text.text.trim();
        if value.is_empty() {
            continue;
        }
        let overlap = overlap_area(target.bounds, text.bounds);
        if overlap > 0 && best.is_none_or(|(score, _)| overlap > score) {
            best = Some((overlap, value));
        }
    }
    best.map(|(_, value)| value.to_string())
        .unwrap_or_else(|| format!("Tab {}", target.widget.0))
}

fn overlap_area(left: Rect, right: Rect) -> i64 {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = left
        .x
        .saturating_add(left.width.max(0))
        .min(right.x.saturating_add(right.width.max(0)));
    let y1 = left
        .y
        .saturating_add(left.height.max(0))
        .min(right.y.saturating_add(right.height.max(0)));
    i64::from((x1 - x0).max(0)) * i64::from((y1 - y0).max(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NativeDrawCommand, NativeDrawTextCommand, SemanticTextStyle};

    #[test]
    fn snapshot_groups_tabs_and_exposes_selected_panel_relationship() {
        let tab_view = WidgetId(10);
        let general = ZsTabId::new(11);
        let advanced = ZsTabId::new(12);
        let general_target = ViewHitTarget::with_kind(
            WidgetId(general.0),
            Rect {
                x: 8,
                y: 8,
                width: 100,
                height: 32,
            },
            ViewHitTargetKind::Tab {
                tab_view,
                tab: general,
                index: 0,
            },
        );
        let advanced_target = ViewHitTarget::with_kind(
            WidgetId(advanced.0),
            Rect {
                x: 112,
                ..general_target.bounds
            },
            ViewHitTargetKind::Tab {
                tab_view,
                tab: advanced,
                index: 1,
            },
        );
        let plan = NativeDrawPlan::new([
            NativeDrawCommand::Text(NativeDrawTextCommand::new(
                "常规 / General",
                general_target.bounds,
                SemanticTextStyle::body(),
            )),
            NativeDrawCommand::Text(NativeDrawTextCommand::new(
                "高级 / Advanced",
                advanced_target.bounds,
                SemanticTextStyle::body(),
            )),
        ]);

        let snapshots = native_tab_accessibility_snapshots(
            &plan,
            &ViewInteractionPlan::new([general_target, advanced_target]),
            Some(WidgetId(advanced.0)),
            Dpi::standard(),
            |widget| {
                (widget == tab_view).then_some(Rect {
                    x: 0,
                    y: 0,
                    width: 320,
                    height: 240,
                })
            },
            |widget| Some(widget == WidgetId(advanced.0)),
        );

        assert_eq!(snapshots.len(), 1);
        let snapshot = &snapshots[0];
        assert_eq!(snapshot.tab_view, tab_view);
        assert_eq!(snapshot.items.len(), 2);
        assert_eq!(snapshot.items[0].label, "常规 / General");
        assert_eq!(snapshot.items[1].label, "高级 / Advanced");
        assert_eq!(snapshot.items[1].position, 2);
        assert_eq!(snapshot.items[1].count, 2);
        assert_eq!(
            snapshot.selected_item().map(|item| item.tab()),
            Some(advanced)
        );
        assert_eq!(
            snapshot.focused_item().map(|item| item.tab()),
            Some(advanced)
        );
        assert_eq!(snapshot.panel_bounds.x, 0);
        assert_eq!(snapshot.panel_bounds.width, 320);
        assert!(snapshot.panel_bounds.y > snapshot.list_bounds.y);
    }
}
