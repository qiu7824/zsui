use serde::{Deserialize, Serialize};

use crate::ViewNode;

/// Stable identity of one item in an Accordion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsAccordionItemId(pub u64);

impl ZsAccordionItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<u64> for ZsAccordionItemId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Expansion policy for an Accordion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsAccordionMode {
    /// At most one item is expanded. `collapsible` controls whether clicking
    /// the expanded item may close the final open section.
    Single { collapsible: bool },
    /// Any number of items may be expanded independently.
    Multiple,
}

impl ZsAccordionMode {
    pub const fn single(collapsible: bool) -> Self {
        Self::Single { collapsible }
    }

    pub const fn multiple() -> Self {
        Self::Multiple
    }
}

impl Default for ZsAccordionMode {
    fn default() -> Self {
        Self::Single { collapsible: true }
    }
}

/// Typed state transition produced by an Accordion header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAccordionChange {
    /// Item whose header was activated.
    pub item: ZsAccordionItemId,
    /// Whether that item is expanded in `next_expanded`.
    pub expanded: bool,
    /// Complete controlled expansion state after the activation.
    pub next_expanded: Vec<ZsAccordionItemId>,
}

/// One titled Accordion section with arbitrary retained View content.
#[derive(Debug, Clone)]
pub struct ZsAccordionItem<Msg> {
    pub(crate) id: ZsAccordionItemId,
    pub(crate) title: String,
    pub(crate) content: ViewNode<Msg>,
    pub(crate) enabled: bool,
}

impl<Msg> ZsAccordionItem<Msg> {
    pub fn new(
        id: impl Into<ZsAccordionItemId>,
        title: impl Into<String>,
        content: ViewNode<Msg>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content,
            enabled: true,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn id(&self) -> ZsAccordionItemId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

pub(crate) fn accordion_expanded_ids(
    item_ids: &[ZsAccordionItemId],
    expanded: impl IntoIterator<Item = ZsAccordionItemId>,
    mode: ZsAccordionMode,
) -> Vec<ZsAccordionItemId> {
    let mut normalized = Vec::new();
    for id in expanded {
        if item_ids.contains(&id) && !normalized.contains(&id) {
            normalized.push(id);
            if matches!(mode, ZsAccordionMode::Single { .. }) {
                break;
            }
        }
    }
    normalized
}

pub(crate) fn accordion_change(
    current: &[ZsAccordionItemId],
    item: ZsAccordionItemId,
    mode: ZsAccordionMode,
) -> ZsAccordionChange {
    let was_expanded = current.contains(&item);
    let mut next_expanded = current.to_vec();
    match mode {
        ZsAccordionMode::Single { collapsible } => {
            if was_expanded {
                if collapsible {
                    next_expanded.clear();
                }
            } else {
                next_expanded.clear();
                next_expanded.push(item);
            }
        }
        ZsAccordionMode::Multiple => {
            if was_expanded {
                next_expanded.retain(|candidate| *candidate != item);
            } else {
                next_expanded.push(item);
            }
        }
    }
    ZsAccordionChange {
        item,
        expanded: next_expanded.contains(&item),
        next_expanded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_and_multiple_modes_produce_complete_next_state() {
        let one = ZsAccordionItemId::new(1);
        let two = ZsAccordionItemId::new(2);

        let single = accordion_change(&[one], two, ZsAccordionMode::single(false));
        assert_eq!(single.next_expanded, vec![two]);
        let retained = accordion_change(&[two], two, ZsAccordionMode::single(false));
        assert_eq!(retained.next_expanded, vec![two]);
        let collapsed = accordion_change(&[two], two, ZsAccordionMode::single(true));
        assert!(collapsed.next_expanded.is_empty());

        let multiple = accordion_change(&[one], two, ZsAccordionMode::multiple());
        assert_eq!(multiple.next_expanded, vec![one, two]);
        let toggled = accordion_change(&multiple.next_expanded, one, ZsAccordionMode::multiple());
        assert_eq!(toggled.next_expanded, vec![two]);
    }

    #[test]
    fn normalization_rejects_unknown_duplicates_and_extra_single_values() {
        let one = ZsAccordionItemId::new(1);
        let two = ZsAccordionItemId::new(2);
        let unknown = ZsAccordionItemId::new(99);
        assert_eq!(
            accordion_expanded_ids(
                &[one, two],
                [unknown, two, two, one],
                ZsAccordionMode::multiple()
            ),
            vec![two, one]
        );
        assert_eq!(
            accordion_expanded_ids(&[one, two], [two, one], ZsAccordionMode::single(true)),
            vec![two]
        );
    }
}
