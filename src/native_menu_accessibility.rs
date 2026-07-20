use crate::{NativeDrawCommand, NativeDrawPlan, Rect, ViewHitTarget, ViewHitTargetKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeMenuFlyoutAccessibilityItem {
    pub(crate) target: ViewHitTarget,
    pub(crate) label: String,
    pub(crate) enabled: bool,
}

impl NativeMenuFlyoutAccessibilityItem {
    pub(crate) const fn path(&self) -> crate::ZsMenuFlyoutPath {
        match self.target.kind {
            ViewHitTargetKind::MenuFlyoutItem { path, .. } => path,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn expanded(&self) -> Option<bool> {
        match self.target.kind {
            ViewHitTargetKind::MenuFlyoutItem {
                row_kind: crate::ZsMenuFlyoutRowKind::Submenu,
                expanded,
                ..
            } => Some(expanded),
            _ => None,
        }
    }

    pub(crate) const fn checked(&self) -> Option<bool> {
        match self.target.kind {
            ViewHitTargetKind::MenuFlyoutItem {
                row_kind: crate::ZsMenuFlyoutRowKind::Command { checked },
                ..
            } => Some(checked),
            _ => None,
        }
    }

    pub(crate) const fn highlighted(&self) -> bool {
        matches!(
            self.target.kind,
            ViewHitTargetKind::MenuFlyoutItem {
                highlighted: true,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeMenuFlyoutAccessibilitySnapshot {
    pub(crate) widget: crate::WidgetId,
    pub(crate) bounds: Rect,
    pub(crate) items: Vec<NativeMenuFlyoutAccessibilityItem>,
}

impl NativeMenuFlyoutAccessibilitySnapshot {
    #[cfg(any(windows, target_os = "macos"))]
    pub(crate) fn item(
        &self,
        path: crate::ZsMenuFlyoutPath,
    ) -> Option<&NativeMenuFlyoutAccessibilityItem> {
        self.items.iter().find(|item| item.path() == path)
    }

    pub(crate) fn highlighted_item(&self) -> Option<&NativeMenuFlyoutAccessibilityItem> {
        self.items.iter().find(|item| item.highlighted())
    }
}

pub(crate) fn native_menu_flyout_accessibility_snapshot(
    plan: &NativeDrawPlan,
    interaction: &crate::ViewInteractionPlan,
    menu: Option<&crate::MenuSpec>,
) -> Option<NativeMenuFlyoutAccessibilitySnapshot> {
    let mut widget = None;
    let mut bounds = None;
    let mut items = Vec::new();
    for target in interaction.hit_targets.iter().copied() {
        let ViewHitTargetKind::MenuFlyoutItem { row_kind, .. } = target.kind else {
            continue;
        };
        if matches!(row_kind, crate::ZsMenuFlyoutRowKind::Separator) {
            continue;
        }
        widget.get_or_insert(target.widget);
        bounds = Some(union_rect(bounds, target.bounds));
        let label = match target.kind {
            ViewHitTargetKind::MenuFlyoutItem { path, .. } => menu
                .and_then(|menu| crate::menu_flyout::menu_flyout_item(menu, path))
                .and_then(menu_item_label)
                .map(str::to_string)
                .unwrap_or_else(|| native_accessible_label(plan, target)),
            _ => native_accessible_label(plan, target),
        };
        let enabled = match target.kind {
            ViewHitTargetKind::MenuFlyoutItem { path, .. } => menu
                .and_then(|menu| crate::menu_flyout::menu_flyout_item(menu, path))
                .is_none_or(menu_item_enabled),
            _ => true,
        };
        items.push(NativeMenuFlyoutAccessibilityItem {
            target,
            label,
            enabled,
        });
    }
    Some(NativeMenuFlyoutAccessibilitySnapshot {
        widget: widget?,
        bounds: bounds?,
        items,
    })
}

fn menu_item_label(item: &crate::MenuItemSpec) -> Option<&str> {
    match item {
        crate::MenuItemSpec::Command { label, .. } | crate::MenuItemSpec::Submenu { label, .. } => {
            Some(label)
        }
        crate::MenuItemSpec::Separator => None,
    }
}

fn menu_item_enabled(item: &crate::MenuItemSpec) -> bool {
    match item {
        crate::MenuItemSpec::Command { enabled, .. }
        | crate::MenuItemSpec::Submenu { enabled, .. } => *enabled,
        crate::MenuItemSpec::Separator => false,
    }
}

pub(crate) fn native_accessible_label(plan: &NativeDrawPlan, target: ViewHitTarget) -> String {
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
        .unwrap_or_else(|| format!("Menu item {}", target.widget.0))
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
    use crate::{NativeDrawTextCommand, SemanticTextStyle, ZsuiThemeMode};

    #[test]
    fn menu_snapshot_keeps_native_labels_and_recursive_state() {
        let widget = crate::WidgetId(9);
        let checked = ViewHitTarget::with_kind(
            widget,
            Rect {
                x: 10,
                y: 20,
                width: 160,
                height: 28,
            },
            ViewHitTargetKind::MenuFlyoutItem {
                path: crate::ZsMenuFlyoutPath::root(0),
                row_kind: crate::ZsMenuFlyoutRowKind::Command { checked: true },
                expanded: false,
                highlighted: false,
            },
        );
        let submenu = ViewHitTarget::with_kind(
            widget,
            Rect {
                y: 48,
                ..checked.bounds
            },
            ViewHitTargetKind::MenuFlyoutItem {
                path: crate::ZsMenuFlyoutPath::root(1),
                row_kind: crate::ZsMenuFlyoutRowKind::Submenu,
                expanded: true,
                highlighted: true,
            },
        );
        let plan = NativeDrawPlan::new([
            NativeDrawCommand::Text(NativeDrawTextCommand {
                text: "自动保存 / Auto save".to_string(),
                bounds: checked.bounds,
                style: SemanticTextStyle::body(),
            }),
            NativeDrawCommand::Text(NativeDrawTextCommand {
                text: "更多 / More".to_string(),
                bounds: submenu.bounds,
                style: SemanticTextStyle::body(),
            }),
        ])
        .theme_mode(ZsuiThemeMode::Light);
        let snapshot = native_menu_flyout_accessibility_snapshot(
            &plan,
            &crate::ViewInteractionPlan::new([checked, submenu]),
            Some(
                &crate::MenuSpec::new()
                    .item("自动保存 / Auto save", crate::Command::custom("auto-save"))
                    .submenu(
                        "更多 / More",
                        crate::MenuSpec::new().item("复制 / Copy", crate::Command::custom("copy")),
                    ),
            ),
        )
        .expect("open MenuFlyout should produce an accessibility snapshot");

        assert_eq!(snapshot.widget, widget);
        assert_eq!(snapshot.bounds.height, 56);
        assert_eq!(snapshot.items[0].label, "自动保存 / Auto save");
        assert!(snapshot.items[0].enabled);
        assert_eq!(snapshot.items[0].checked(), Some(true));
        assert_eq!(snapshot.items[0].expanded(), None);
        assert_eq!(snapshot.items[1].label, "更多 / More");
        assert_eq!(snapshot.items[1].expanded(), Some(true));
        assert_eq!(snapshot.highlighted_item(), Some(&snapshot.items[1]));
    }
}
