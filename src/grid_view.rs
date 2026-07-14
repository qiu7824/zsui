use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ZsIcon;

/// Stable application-owned identity for one GridView item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsGridViewItemId(u64);

impl ZsGridViewItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsGridViewItemId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Application-owned display data for one first-pass GridView tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsGridViewItem {
    id: ZsGridViewItemId,
    title: String,
    subtitle: Option<String>,
    icon: Option<ZsIcon>,
}

impl ZsGridViewItem {
    pub fn new(id: impl Into<ZsGridViewItemId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            icon: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn icon(mut self, icon: ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub const fn id(&self) -> ZsGridViewItemId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn item_subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub const fn item_icon(&self) -> Option<ZsIcon> {
        self.icon
    }
}

/// Explicit selection and responsive-column snapshot used by native input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsGridViewState {
    pub selected: Option<ZsGridViewItemId>,
    pub items: Vec<ZsGridViewItemId>,
    pub column_count: usize,
}

impl ZsGridViewState {
    pub fn contains(&self, item: ZsGridViewItemId) -> bool {
        self.items.contains(&item)
    }

    pub fn first(&self) -> Option<ZsGridViewItemId> {
        self.items.first().copied()
    }

    pub fn last(&self) -> Option<ZsGridViewItemId> {
        self.items.last().copied()
    }

    pub fn relative_horizontal(&self, offset: isize) -> Option<ZsGridViewItemId> {
        let columns = self.column_count.max(1);
        let current = self
            .selected
            .and_then(|selected| self.items.iter().position(|item| *item == selected));
        let index = match current {
            Some(index) => {
                let row_start = (index / columns) * columns;
                let row_end = row_start
                    .saturating_add(columns)
                    .min(self.items.len())
                    .saturating_sub(1);
                (index as isize + offset).clamp(row_start as isize, row_end as isize) as usize
            }
            None if offset < 0 => self.items.len().checked_sub(1)?,
            None => 0,
        };
        self.items.get(index).copied()
    }

    pub fn relative_vertical(&self, offset_rows: isize) -> Option<ZsGridViewItemId> {
        let columns = self.column_count.max(1);
        let current = self
            .selected
            .and_then(|selected| self.items.iter().position(|item| *item == selected));
        let index = match current {
            Some(index) => {
                let last = self.items.len().saturating_sub(1);
                (index as isize + offset_rows.saturating_mul(columns as isize))
                    .clamp(0, last as isize) as usize
            }
            None if offset_rows < 0 => self.items.len().checked_sub(1)?,
            None => 0,
        };
        self.items.get(index).copied()
    }
}

pub(crate) fn unique_grid_view_items(items: &[ZsGridViewItem]) -> Vec<&ZsGridViewItem> {
    let mut seen = BTreeSet::new();
    items.iter().filter(|item| seen.insert(item.id())).collect()
}

pub(crate) fn grid_view_state(
    items: &[ZsGridViewItem],
    selected: Option<ZsGridViewItemId>,
    column_count: usize,
) -> ZsGridViewState {
    ZsGridViewState {
        selected,
        items: unique_grid_view_items(items)
            .into_iter()
            .map(ZsGridViewItem::id)
            .collect(),
        column_count: column_count.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_view_state_deduplicates_strong_ids_and_keeps_row_navigation_bounded() {
        let state = grid_view_state(
            &[
                ZsGridViewItem::new(1, "One"),
                ZsGridViewItem::new(2, "Two"),
                ZsGridViewItem::new(3, "Three"),
                ZsGridViewItem::new(4, "Four"),
                ZsGridViewItem::new(2, "Duplicate"),
            ],
            Some(ZsGridViewItemId::new(2)),
            3,
        );

        assert_eq!(
            state.items,
            vec![1_u64.into(), 2_u64.into(), 3_u64.into(), 4_u64.into()]
        );
        assert_eq!(state.relative_horizontal(1), Some(ZsGridViewItemId::new(3)));
        assert_eq!(state.relative_vertical(1), Some(ZsGridViewItemId::new(4)));
        assert_eq!(
            state.relative_horizontal(-5),
            Some(ZsGridViewItemId::new(1))
        );
    }
}
