use serde::{Deserialize, Serialize};

/// The three semantic response slots exposed by a content dialog.
///
/// The renderer chooses the visual order for the target platform; application
/// code responds to the semantic slot and never needs platform conditionals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsContentDialogButton {
    Primary,
    Secondary,
    Close,
}

/// Strongly typed result emitted after a dialog response is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsContentDialogResult {
    Primary,
    Secondary,
    Close,
}

impl From<ZsContentDialogButton> for ZsContentDialogResult {
    fn from(button: ZsContentDialogButton) -> Self {
        match button {
            ZsContentDialogButton::Primary => Self::Primary,
            ZsContentDialogButton::Secondary => Self::Secondary,
            ZsContentDialogButton::Close => Self::Close,
        }
    }
}

/// Application-owned immutable content and command metadata for one dialog.
///
/// A safe close response is mandatory. Primary and secondary responses are
/// optional, matching the three built-in ContentDialog response slots without
/// exposing a platform button object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsContentDialogSpec {
    title: Option<String>,
    content: String,
    primary_button: Option<String>,
    secondary_button: Option<String>,
    close_button: String,
    default_button: Option<ZsContentDialogButton>,
    destructive_button: Option<ZsContentDialogButton>,
}

impl ZsContentDialogSpec {
    pub fn new(content: impl Into<String>, close_button: impl Into<String>) -> Self {
        Self {
            title: None,
            content: content.into(),
            primary_button: None,
            secondary_button: None,
            close_button: close_button.into(),
            default_button: None,
            destructive_button: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        let title = title.into();
        self.title = (!title.is_empty()).then_some(title);
        self
    }

    pub fn primary_button(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.primary_button = (!label.is_empty()).then_some(label);
        self
    }

    pub fn secondary_button(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.secondary_button = (!label.is_empty()).then_some(label);
        self
    }

    pub fn default_button(mut self, button: ZsContentDialogButton) -> Self {
        self.default_button = Some(button);
        if self.destructive_button == Some(button) {
            self.destructive_button = None;
        }
        self
    }

    pub fn destructive_button(mut self, button: ZsContentDialogButton) -> Self {
        self.destructive_button = Some(button);
        if self.destructive_button == self.default_button {
            self.default_button = None;
        }
        self
    }

    pub fn dialog_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn button_label(&self, button: ZsContentDialogButton) -> Option<&str> {
        match button {
            ZsContentDialogButton::Primary => self
                .primary_button
                .as_deref()
                .filter(|label| !label.is_empty()),
            ZsContentDialogButton::Secondary => self
                .secondary_button
                .as_deref()
                .filter(|label| !label.is_empty()),
            ZsContentDialogButton::Close => Some(self.close_button.as_str()),
        }
    }

    pub fn has_button(&self, button: ZsContentDialogButton) -> bool {
        self.button_label(button).is_some()
    }

    pub fn default_response(&self) -> Option<ZsContentDialogButton> {
        self.default_button
            .filter(|button| self.has_button(*button))
    }

    pub fn destructive_response(&self) -> Option<ZsContentDialogButton> {
        self.destructive_button
            .filter(|button| self.has_button(*button))
    }

    pub fn initial_focus(&self) -> ZsContentDialogButton {
        self.default_response()
            .or_else(|| {
                [
                    ZsContentDialogButton::Primary,
                    ZsContentDialogButton::Secondary,
                    ZsContentDialogButton::Close,
                ]
                .into_iter()
                .find(|button| self.has_button(*button))
            })
            .unwrap_or(ZsContentDialogButton::Close)
    }

    pub fn relative_button(
        &self,
        current: ZsContentDialogButton,
        offset: isize,
    ) -> ZsContentDialogButton {
        let buttons = [
            ZsContentDialogButton::Primary,
            ZsContentDialogButton::Secondary,
            ZsContentDialogButton::Close,
        ]
        .into_iter()
        .filter(|button| self.has_button(*button))
        .collect::<Vec<_>>();
        if buttons.is_empty() {
            return ZsContentDialogButton::Close;
        }
        let current = buttons
            .iter()
            .position(|button| *button == current)
            .unwrap_or(0);
        let next = (current as isize + offset).rem_euclid(buttons.len() as isize) as usize;
        buttons[next]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsContentDialogState {
    pub open: bool,
    pub focused_button: ZsContentDialogButton,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_dialog_keeps_safe_close_and_validates_response_roles() {
        let spec = ZsContentDialogSpec::new("Save changes?", "Cancel")
            .default_button(ZsContentDialogButton::Primary)
            .primary_button("Save")
            .destructive_button(ZsContentDialogButton::Secondary);

        assert_eq!(
            spec.button_label(ZsContentDialogButton::Close),
            Some("Cancel")
        );
        assert_eq!(
            spec.default_response(),
            Some(ZsContentDialogButton::Primary)
        );
        assert_eq!(spec.destructive_response(), None);

        let last_role_wins = ZsContentDialogSpec::new("Delete?", "Cancel")
            .primary_button("Delete")
            .destructive_button(ZsContentDialogButton::Primary)
            .default_button(ZsContentDialogButton::Primary);
        assert_eq!(
            last_role_wins.default_response(),
            Some(ZsContentDialogButton::Primary)
        );
        assert_eq!(last_role_wins.destructive_response(), None);
    }

    #[test]
    fn content_dialog_button_navigation_is_typed_and_cyclic() {
        let spec = ZsContentDialogSpec::new("Replace the file?", "Cancel")
            .primary_button("Replace")
            .secondary_button("Keep Both");

        assert_eq!(
            spec.relative_button(ZsContentDialogButton::Primary, 1),
            ZsContentDialogButton::Secondary
        );
        assert_eq!(
            spec.relative_button(ZsContentDialogButton::Close, 1),
            ZsContentDialogButton::Primary
        );
    }
}
