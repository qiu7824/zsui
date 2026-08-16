/// Creates a controlled Accordion from titled sections and arbitrary content.
///
/// `on_change` receives the complete next expanded-ID set, so application
/// state can apply single or multiple expansion without duplicating toggle
/// rules. Accordion content may contain another Accordion.
pub fn accordion<Msg: Clone>(
    widget: WidgetId,
    items: impl IntoIterator<Item = crate::ZsAccordionItem<Msg>>,
    expanded: impl IntoIterator<Item = crate::ZsAccordionItemId>,
    mode: crate::ZsAccordionMode,
    on_change: impl Fn(crate::ZsAccordionChange) -> Msg,
) -> ViewNode<Msg> {
    let items = items.into_iter().collect::<Vec<_>>();
    let item_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
    let expanded = crate::accordion::accordion_expanded_ids(&item_ids, expanded, mode);
    let spacing = crate::ZsuiSpacingTokens::default();
    let mut sections = Vec::with_capacity(items.len().saturating_mul(2));

    for item in items {
        let is_expanded = expanded.contains(&item.id);
        let change = crate::accordion::accordion_change(&expanded, item.id, mode);
        let changed = change.next_expanded != expanded;
        let mut trigger = toolbar_button(
            item.title,
            if is_expanded {
                crate::ZsIcon::ChevronDown
            } else {
                crate::ZsIcon::ChevronRight
            },
        )
        .id(WidgetId::synthetic_child(widget, item.id.0))
        .enabled(item.enabled);
        if item.enabled && changed {
            trigger = trigger.on_click(on_change(change));
        }
        sections.push(trigger);
        if is_expanded {
            sections.push(
                column([item.content])
                    .padding(spacing.md)
                    .flex(0.0),
            );
        }
    }

    column(sections).id(widget).gap(spacing.xs)
}
