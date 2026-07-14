use std::ops::Range;

use crate::native_text_edit::NativeTextSelection;
use crate::{Rect, ViewHitTarget, ViewHitTargetKind, WidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum NativeTextAccessibilityKind {
    SingleLine,
    MultiLine,
    Protected,
}

#[allow(dead_code)]
impl NativeTextAccessibilityKind {
    fn from_target_kind(kind: ViewHitTargetKind) -> Option<Self> {
        match kind {
            ViewHitTargetKind::Textbox => Some(Self::SingleLine),
            ViewHitTargetKind::TextEditor => Some(Self::MultiLine),
            #[cfg(feature = "password-box")]
            ViewHitTargetKind::PasswordBox => Some(Self::Protected),
            #[cfg(feature = "number-box")]
            ViewHitTargetKind::NumberBox => Some(Self::SingleLine),
            #[cfg(feature = "auto-suggest")]
            ViewHitTargetKind::AutoSuggestBox => Some(Self::SingleLine),
            #[cfg(feature = "command-palette")]
            ViewHitTargetKind::CommandPalette => Some(Self::SingleLine),
            _ => None,
        }
    }

    pub(crate) const fn is_multiline(self) -> bool {
        matches!(self, Self::MultiLine)
    }

    pub(crate) const fn is_protected(self) -> bool {
        matches!(self, Self::Protected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeTextAccessibilitySnapshot {
    widget: WidgetId,
    kind: NativeTextAccessibilityKind,
    exposed_text: String,
    selection: NativeTextSelection,
    bounds: Rect,
    caret: Rect,
}

#[allow(dead_code)]
impl NativeTextAccessibilitySnapshot {
    pub(crate) fn new(
        target: ViewHitTarget,
        exposed_text: String,
        selection: NativeTextSelection,
        caret: Rect,
    ) -> Option<Self> {
        let kind = NativeTextAccessibilityKind::from_target_kind(target.kind)?;
        let selection = selection.clamp(&exposed_text);
        Some(Self {
            widget: target.widget,
            kind,
            exposed_text,
            selection,
            bounds: target.bounds,
            caret,
        })
    }

    pub(crate) const fn widget(&self) -> WidgetId {
        self.widget
    }

    pub(crate) const fn kind(&self) -> NativeTextAccessibilityKind {
        self.kind
    }

    pub(crate) fn exposed_text(&self) -> &str {
        &self.exposed_text
    }

    pub(crate) fn character_count(&self) -> usize {
        self.exposed_text.chars().count()
    }

    pub(crate) const fn selection(&self) -> NativeTextSelection {
        self.selection
    }

    pub(crate) fn ordered_selection(&self) -> Range<usize> {
        let (start, end) = self.selection.ordered();
        start..end
    }

    pub(crate) fn selected_text(&self) -> String {
        self.text_in_range(self.ordered_selection())
            .unwrap_or_default()
    }

    pub(crate) fn text_in_range(&self, range: Range<usize>) -> Option<String> {
        if range.start > range.end || range.end > self.character_count() {
            return None;
        }
        Some(
            self.exposed_text
                .chars()
                .skip(range.start)
                .take(range.end - range.start)
                .collect(),
        )
    }

    pub(crate) fn utf16_offset(&self, scalar_index: usize) -> Option<usize> {
        if scalar_index > self.character_count() {
            return None;
        }
        Some(
            self.exposed_text
                .chars()
                .take(scalar_index)
                .map(char::len_utf16)
                .sum(),
        )
    }

    pub(crate) fn utf16_selection(&self) -> Range<usize> {
        let selection = self.ordered_selection();
        let start = self.utf16_offset(selection.start).unwrap_or(0);
        let end = self.utf16_offset(selection.end).unwrap_or(start);
        start..end
    }

    pub(crate) const fn bounds(&self) -> Rect {
        self.bounds
    }

    pub(crate) const fn caret(&self) -> Rect {
        self.caret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(kind: ViewHitTargetKind) -> ViewHitTarget {
        ViewHitTarget::with_kind(
            WidgetId(7),
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 80,
            },
            kind,
        )
    }

    fn zero_rect() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    #[test]
    fn snapshot_keeps_scalar_ranges_and_reports_utf16_offsets() {
        let snapshot = NativeTextAccessibilitySnapshot::new(
            target(ViewHitTargetKind::TextEditor),
            "A😀中".to_string(),
            NativeTextSelection {
                anchor: 3,
                caret: 1,
            },
            Rect {
                x: 20,
                y: 30,
                width: 1,
                height: 18,
            },
        )
        .expect("text editor should create an accessibility snapshot");

        assert_eq!(snapshot.character_count(), 3);
        assert_eq!(snapshot.ordered_selection(), 1..3);
        assert_eq!(snapshot.selected_text(), "😀中");
        assert_eq!(snapshot.utf16_offset(0), Some(0));
        assert_eq!(snapshot.utf16_offset(1), Some(1));
        assert_eq!(snapshot.utf16_offset(2), Some(3));
        assert_eq!(snapshot.utf16_offset(3), Some(4));
        assert_eq!(snapshot.utf16_selection(), 1..4);
        assert!(snapshot.kind().is_multiline());
        assert!(!snapshot.kind().is_protected());
    }

    #[test]
    fn snapshot_rejects_non_text_targets_and_invalid_ranges() {
        assert!(NativeTextAccessibilitySnapshot::new(
            target(ViewHitTargetKind::Button),
            "button".to_string(),
            NativeTextSelection::collapsed(0),
            zero_rect(),
        )
        .is_none());

        let snapshot = NativeTextAccessibilitySnapshot::new(
            target(ViewHitTargetKind::Textbox),
            "text".to_string(),
            NativeTextSelection::collapsed(99),
            zero_rect(),
        )
        .expect("textbox should create an accessibility snapshot");
        assert_eq!(snapshot.selection(), NativeTextSelection::collapsed(4));
        assert_eq!(snapshot.text_in_range(3..2), None);
        assert_eq!(snapshot.text_in_range(0..5), None);
    }

    #[cfg(feature = "password-box")]
    #[test]
    fn protected_snapshot_exposes_only_the_supplied_mask() {
        let snapshot = NativeTextAccessibilitySnapshot::new(
            target(ViewHitTargetKind::PasswordBox),
            "••••••".to_string(),
            NativeTextSelection {
                anchor: 1,
                caret: 4,
            },
            zero_rect(),
        )
        .expect("password box should create an accessibility snapshot");

        assert!(snapshot.kind().is_protected());
        assert_eq!(snapshot.exposed_text(), "••••••");
        assert_eq!(snapshot.selected_text(), "•••");
    }
}
