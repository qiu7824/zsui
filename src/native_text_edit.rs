use std::ops::Range;

use crate::WidgetId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeTextSelection {
    pub anchor: usize,
    pub caret: usize,
}

impl NativeTextSelection {
    pub(crate) const fn collapsed(caret: usize) -> Self {
        Self {
            anchor: caret,
            caret,
        }
    }

    pub(crate) const fn ordered(self) -> (usize, usize) {
        if self.anchor <= self.caret {
            (self.anchor, self.caret)
        } else {
            (self.caret, self.anchor)
        }
    }

    pub(crate) const fn is_collapsed(self) -> bool {
        self.anchor == self.caret
    }

    pub(crate) fn clamp(self, value: &str) -> Self {
        let len = char_count(value);
        Self {
            anchor: self.anchor.min(len),
            caret: self.caret.min(len),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeTextEditState {
    pub widget: WidgetId,
    pub selection: NativeTextSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeTextDragState {
    pub widget: WidgetId,
    pub anchor: usize,
}

impl NativeTextEditState {
    pub(crate) fn at_end(widget: WidgetId, value: &str) -> Self {
        Self {
            widget,
            selection: NativeTextSelection::collapsed(char_count(value)),
        }
    }

    pub(crate) fn clamp(&mut self, value: &str) {
        self.selection = self.selection.clamp(value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeTextMovement {
    Left,
    Right,
    Home,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NativeTextEditResult {
    pub handled: bool,
    pub text_changed: bool,
    pub selection_changed: bool,
}

pub(crate) fn apply_text_input(
    value: &mut String,
    selection: &mut NativeTextSelection,
    text: &str,
    multiline: bool,
) -> NativeTextEditResult {
    let mut result = NativeTextEditResult::default();
    let mut previous_was_carriage_return = false;
    for ch in text.chars() {
        let edit = match ch {
            '\u{8}' => delete_backward(value, selection),
            '\u{7f}' => delete_forward(value, selection),
            '\r' if multiline => {
                previous_was_carriage_return = true;
                insert_text(value, selection, "\n")
            }
            '\n' if multiline && !previous_was_carriage_return => {
                insert_text(value, selection, "\n")
            }
            ch if !ch.is_control() => {
                let mut buffer = [0_u8; 4];
                insert_text(value, selection, ch.encode_utf8(&mut buffer))
            }
            _ => NativeTextEditResult::default(),
        };
        result.handled |= edit.handled;
        result.text_changed |= edit.text_changed;
        result.selection_changed |= edit.selection_changed;
        if ch != '\r' {
            previous_was_carriage_return = false;
        }
    }
    result
}

pub(crate) fn insert_text(
    value: &mut String,
    selection: &mut NativeTextSelection,
    text: &str,
) -> NativeTextEditResult {
    if text.is_empty() {
        return NativeTextEditResult::default();
    }
    replace_selection(value, selection, text)
}

pub(crate) fn delete_backward(
    value: &mut String,
    selection: &mut NativeTextSelection,
) -> NativeTextEditResult {
    *selection = selection.clamp(value);
    let (start, end) = selection.ordered();
    if start != end {
        return replace_char_range(value, selection, start..end, "");
    }
    if start == 0 {
        return NativeTextEditResult {
            handled: true,
            ..NativeTextEditResult::default()
        };
    }
    replace_char_range(value, selection, start - 1..start, "")
}

pub(crate) fn delete_forward(
    value: &mut String,
    selection: &mut NativeTextSelection,
) -> NativeTextEditResult {
    *selection = selection.clamp(value);
    let (start, end) = selection.ordered();
    if start != end {
        return replace_char_range(value, selection, start..end, "");
    }
    let len = char_count(value);
    if end >= len {
        return NativeTextEditResult {
            handled: true,
            ..NativeTextEditResult::default()
        };
    }
    replace_char_range(value, selection, end..end + 1, "")
}

pub(crate) fn move_selection(
    value: &str,
    selection: &mut NativeTextSelection,
    movement: NativeTextMovement,
    extend: bool,
    multiline: bool,
) -> NativeTextEditResult {
    let before = selection.clamp(value);
    *selection = before;
    let len = char_count(value);
    let (start, end) = before.ordered();
    let target = match movement {
        NativeTextMovement::Left if !extend && !before.is_collapsed() => start,
        NativeTextMovement::Right if !extend && !before.is_collapsed() => end,
        NativeTextMovement::Left => before.caret.saturating_sub(1),
        NativeTextMovement::Right => before.caret.saturating_add(1).min(len),
        NativeTextMovement::Home if multiline => line_start(value, before.caret),
        NativeTextMovement::End if multiline => line_end(value, before.caret),
        NativeTextMovement::Home => 0,
        NativeTextMovement::End => len,
    };
    if extend {
        selection.caret = target;
    } else {
        *selection = NativeTextSelection::collapsed(target);
    }
    NativeTextEditResult {
        handled: true,
        selection_changed: *selection != before,
        ..NativeTextEditResult::default()
    }
}

pub(crate) fn set_pointer_selection(
    value: &str,
    selection: &mut NativeTextSelection,
    anchor: usize,
    caret: usize,
) -> NativeTextEditResult {
    let before = selection.clamp(value);
    *selection = NativeTextSelection { anchor, caret }.clamp(value);
    NativeTextEditResult {
        handled: true,
        selection_changed: *selection != before,
        ..NativeTextEditResult::default()
    }
}

pub(crate) fn char_count(value: &str) -> usize {
    value.chars().count()
}

pub(crate) fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

fn replace_selection(
    value: &mut String,
    selection: &mut NativeTextSelection,
    replacement: &str,
) -> NativeTextEditResult {
    *selection = selection.clamp(value);
    let (start, end) = selection.ordered();
    replace_char_range(value, selection, start..end, replacement)
}

fn replace_char_range(
    value: &mut String,
    selection: &mut NativeTextSelection,
    range: Range<usize>,
    replacement: &str,
) -> NativeTextEditResult {
    let before_selection = *selection;
    let start_byte = char_to_byte_index(value, range.start);
    let end_byte = char_to_byte_index(value, range.end);
    let text_changed = value.get(start_byte..end_byte) != Some(replacement);
    if text_changed {
        value.replace_range(start_byte..end_byte, replacement);
    }
    let caret = range.start.saturating_add(char_count(replacement));
    *selection = NativeTextSelection::collapsed(caret);
    NativeTextEditResult {
        handled: true,
        text_changed,
        selection_changed: *selection != before_selection,
    }
}

fn line_start(value: &str, caret: usize) -> usize {
    value
        .chars()
        .take(caret)
        .enumerate()
        .filter_map(|(index, character)| (character == '\n').then_some(index + 1))
        .last()
        .unwrap_or(0)
}

fn line_end(value: &str, caret: usize) -> usize {
    value
        .chars()
        .enumerate()
        .skip(caret)
        .find_map(|(index, character)| (character == '\n').then_some(index))
        .unwrap_or_else(|| char_count(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_insert_replaces_selection_and_keeps_character_indices() {
        let mut value = "A中文Z".to_string();
        let mut selection = NativeTextSelection {
            anchor: 1,
            caret: 3,
        };

        let result = insert_text(&mut value, &mut selection, "🙂");

        assert!(result.text_changed);
        assert_eq!(value, "A🙂Z");
        assert_eq!(selection, NativeTextSelection::collapsed(2));
    }

    #[test]
    fn backward_and_forward_delete_remove_complete_unicode_scalars() {
        let mut value = "A🙂中Z".to_string();
        let mut selection = NativeTextSelection::collapsed(2);

        let backward = delete_backward(&mut value, &mut selection);
        let forward = delete_forward(&mut value, &mut selection);

        assert!(backward.text_changed);
        assert!(forward.text_changed);
        assert_eq!(value, "AZ");
        assert_eq!(selection, NativeTextSelection::collapsed(1));
    }

    #[test]
    fn navigation_extends_and_collapses_selection_with_multiline_home_end() {
        let value = "ab\n中文\nz";
        let mut selection = NativeTextSelection::collapsed(5);

        move_selection(value, &mut selection, NativeTextMovement::Home, true, true);
        assert_eq!(
            selection,
            NativeTextSelection {
                anchor: 5,
                caret: 3
            }
        );

        move_selection(
            value,
            &mut selection,
            NativeTextMovement::Right,
            false,
            true,
        );
        assert_eq!(selection, NativeTextSelection::collapsed(5));

        move_selection(value, &mut selection, NativeTextMovement::End, false, true);
        assert_eq!(selection, NativeTextSelection::collapsed(5));
    }

    #[test]
    fn pointer_selection_preserves_anchor_and_clamps_outside_text() {
        let mut selection = NativeTextSelection::collapsed(1);

        let extended = set_pointer_selection("A中文", &mut selection, 1, 99);

        assert!(extended.handled);
        assert!(extended.selection_changed);
        assert_eq!(
            selection,
            NativeTextSelection {
                anchor: 1,
                caret: 3
            }
        );
    }
}
