use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ZsIcon;

/// Stable application-owned identity for one command-palette entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsCommandPaletteItemId(u64);

impl ZsCommandPaletteItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsCommandPaletteItemId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Application-owned display metadata for one command.
///
/// ZSUI renders and routes the entry, but never executes it. Applications keep
/// command behavior, enablement and persistence outside the framework.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCommandPaletteItem {
    id: ZsCommandPaletteItemId,
    title: String,
    subtitle: Option<String>,
    keywords: Vec<String>,
    shortcut: Option<String>,
    icon: Option<ZsIcon>,
    enabled: bool,
}

impl ZsCommandPaletteItem {
    pub fn new(id: impl Into<ZsCommandPaletteItemId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            keywords: Vec::new(),
            shortcut: None,
            icon: None,
            enabled: true,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        let subtitle = subtitle.into();
        self.subtitle = (!subtitle.is_empty()).then_some(subtitle);
        self
    }

    pub fn keywords<T>(mut self, keywords: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        self.keywords = keywords
            .into_iter()
            .map(Into::into)
            .filter(|keyword| !keyword.is_empty())
            .collect();
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        let shortcut = shortcut.into();
        self.shortcut = (!shortcut.is_empty()).then_some(shortcut);
        self
    }

    pub fn icon(mut self, icon: ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn id(&self) -> ZsCommandPaletteItemId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn item_subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn item_keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn shortcut_label(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }

    pub const fn item_icon(&self) -> Option<ZsIcon> {
        self.icon
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn matches_query(&self, terms: &[String]) -> bool {
        if terms.is_empty() {
            return true;
        }
        let mut searchable = self.title.to_lowercase();
        if let Some(subtitle) = &self.subtitle {
            searchable.push(' ');
            searchable.push_str(&subtitle.to_lowercase());
        }
        for keyword in &self.keywords {
            searchable.push(' ');
            searchable.push_str(&keyword.to_lowercase());
        }
        terms.iter().all(|term| searchable.contains(term))
    }
}

/// Read-only state snapshot used by desktop input routing and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCommandPaletteState {
    pub open: bool,
    pub query: String,
    pub visible_items: Vec<ZsCommandPaletteItemId>,
    pub enabled_items: Vec<ZsCommandPaletteItemId>,
    pub highlighted: Option<ZsCommandPaletteItemId>,
}

impl ZsCommandPaletteState {
    pub fn first_enabled(&self) -> Option<ZsCommandPaletteItemId> {
        self.enabled_items.first().copied()
    }

    pub fn last_enabled(&self) -> Option<ZsCommandPaletteItemId> {
        self.enabled_items.last().copied()
    }

    pub fn relative_highlight(&self, offset: isize) -> Option<ZsCommandPaletteItemId> {
        if self.enabled_items.is_empty() {
            return None;
        }
        let last = self.enabled_items.len().saturating_sub(1);
        let current = self.highlighted.and_then(|highlighted| {
            self.enabled_items
                .iter()
                .position(|candidate| *candidate == highlighted)
        });
        let index = match (current, offset.cmp(&0)) {
            (Some(index), std::cmp::Ordering::Less) => index.saturating_sub(offset.unsigned_abs()),
            (Some(index), std::cmp::Ordering::Greater) => {
                index.saturating_add(offset as usize).min(last)
            }
            (Some(index), std::cmp::Ordering::Equal) => index,
            (None, std::cmp::Ordering::Less) => last,
            (None, _) => 0,
        };
        self.enabled_items.get(index).copied()
    }
}

pub(crate) fn filtered_command_palette_items<'a>(
    items: &'a [ZsCommandPaletteItem],
    query: &str,
) -> Vec<&'a ZsCommandPaletteItem> {
    let terms = query
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    items
        .iter()
        .filter(|item| seen.insert(item.id()))
        .filter(|item| item.matches_query(&terms))
        .collect()
}

pub(crate) fn command_palette_state(
    open: bool,
    query: &str,
    items: &[ZsCommandPaletteItem],
    highlighted: Option<ZsCommandPaletteItemId>,
) -> ZsCommandPaletteState {
    let filtered = filtered_command_palette_items(items, query);
    let visible_items = filtered.iter().map(|item| item.id()).collect::<Vec<_>>();
    let enabled_items = filtered
        .iter()
        .filter(|item| item.is_enabled())
        .map(|item| item.id())
        .collect::<Vec<_>>();
    let highlighted = highlighted.filter(|id| enabled_items.contains(id));
    ZsCommandPaletteState {
        open,
        query: query.to_owned(),
        visible_items,
        enabled_items,
        highlighted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_is_stable_deduplicated_and_checks_all_terms() {
        let items = [
            ZsCommandPaletteItem::new(1_u64, "Open Settings")
                .keywords(["preferences", "configuration"]),
            ZsCommandPaletteItem::new(1_u64, "Duplicate"),
            ZsCommandPaletteItem::new(2_u64, "Open File").subtitle("From disk"),
        ];

        let filtered = filtered_command_palette_items(&items, "open pref");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id(), 1_u64.into());
    }

    #[test]
    fn disabled_entries_remain_visible_but_are_skipped_by_keyboard_navigation() {
        let items = [
            ZsCommandPaletteItem::new(1_u64, "First").enabled(false),
            ZsCommandPaletteItem::new(2_u64, "Second"),
            ZsCommandPaletteItem::new(3_u64, "Third"),
        ];
        let state = command_palette_state(true, "", &items, None);

        assert_eq!(state.visible_items.len(), 3);
        assert_eq!(state.first_enabled(), Some(2_u64.into()));
        assert_eq!(state.relative_highlight(-1), Some(3_u64.into()));
    }
}
