use serde::{Deserialize, Serialize};

/// Stable application-owned identity for one breadcrumb item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsBreadcrumbId(u64);

impl ZsBreadcrumbId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// One labelled location in a root-to-current breadcrumb path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsBreadcrumbItem {
    id: ZsBreadcrumbId,
    label: String,
}

impl ZsBreadcrumbItem {
    pub fn new(id: ZsBreadcrumbId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    pub const fn id(&self) -> ZsBreadcrumbId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Semantic focus inside one breadcrumb bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsBreadcrumbFocusTarget {
    Overflow,
    Item(ZsBreadcrumbId),
}

/// Public state exposed to native input adapters without platform handles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsBreadcrumbState {
    pub items: Vec<ZsBreadcrumbItem>,
    pub overflow_open: bool,
    pub focused: Option<ZsBreadcrumbFocusTarget>,
}

impl ZsBreadcrumbState {
    pub fn current(&self) -> Option<ZsBreadcrumbId> {
        self.items.last().map(ZsBreadcrumbItem::id)
    }

    pub fn item_index(&self, id: ZsBreadcrumbId) -> Option<usize> {
        self.items.iter().position(|item| item.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumb_items_keep_strong_identity_separate_from_labels() {
        let first = ZsBreadcrumbItem::new(ZsBreadcrumbId::new(1), "Projects");
        let second = ZsBreadcrumbItem::new(ZsBreadcrumbId::new(2), "Projects");
        let state = ZsBreadcrumbState {
            items: vec![first, second],
            overflow_open: false,
            focused: None,
        };

        assert_eq!(state.current(), Some(ZsBreadcrumbId::new(2)));
        assert_eq!(state.item_index(ZsBreadcrumbId::new(1)), Some(0));
    }
}
