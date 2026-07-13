use serde::{Deserialize, Serialize};

/// Stable application-owned identity for one auto-suggest result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsAutoSuggestionId(u64);

impl ZsAutoSuggestionId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsAutoSuggestionId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// One display value supplied by application state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAutoSuggestion {
    id: ZsAutoSuggestionId,
    text: String,
}

impl ZsAutoSuggestion {
    pub fn new(id: impl Into<ZsAutoSuggestionId>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }

    pub const fn id(&self) -> ZsAutoSuggestionId {
        self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Why the displayed query changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsAutoSuggestTextChangeReason {
    UserInput,
    SuggestionChosen,
    ProgrammaticChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAutoSuggestTextChange {
    pub text: String,
    pub reason: ZsAutoSuggestTextChangeReason,
}

impl ZsAutoSuggestTextChange {
    pub fn new(text: impl Into<String>, reason: ZsAutoSuggestTextChangeReason) -> Self {
        Self {
            text: text.into(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAutoSuggestSubmission {
    pub query: String,
    pub chosen: Option<ZsAutoSuggestionId>,
}

impl ZsAutoSuggestSubmission {
    pub fn new(query: impl Into<String>, chosen: Option<ZsAutoSuggestionId>) -> Self {
        Self {
            query: query.into(),
            chosen,
        }
    }
}

/// Read-only state snapshot used by native input routing and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsAutoSuggestState {
    pub query: String,
    pub suggestion_ids: Vec<ZsAutoSuggestionId>,
    pub highlighted: Option<ZsAutoSuggestionId>,
    pub expanded: bool,
}

impl ZsAutoSuggestState {
    pub fn next_highlight(&self, offset: isize) -> Option<ZsAutoSuggestionId> {
        if self.suggestion_ids.is_empty() {
            return None;
        }
        let last = self.suggestion_ids.len() - 1;
        let index = self.highlighted.and_then(|id| {
            self.suggestion_ids
                .iter()
                .position(|candidate| *candidate == id)
        });
        let next = match (index, offset.cmp(&0)) {
            (Some(index), std::cmp::Ordering::Less) => index.saturating_sub(offset.unsigned_abs()),
            (Some(index), std::cmp::Ordering::Greater) => {
                index.saturating_add(offset as usize).min(last)
            }
            (Some(index), std::cmp::Ordering::Equal) => index,
            (None, std::cmp::Ordering::Less) => last,
            (None, _) => 0,
        };
        self.suggestion_ids.get(next).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_ids_drive_clamped_highlight_navigation() {
        let state = ZsAutoSuggestState {
            query: "z".into(),
            suggestion_ids: vec![1_u64.into(), 2_u64.into(), 3_u64.into()],
            highlighted: Some(2_u64.into()),
            expanded: true,
        };

        assert_eq!(state.next_highlight(-1), Some(1_u64.into()));
        assert_eq!(state.next_highlight(1), Some(3_u64.into()));
        assert_eq!(state.next_highlight(99), Some(3_u64.into()));
    }
}
