use serde::{Deserialize, Serialize};

/// The semantic importance of an inline status message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsInfoBarSeverity {
    #[default]
    Informational,
    Success,
    Warning,
    Error,
}

impl ZsInfoBarSeverity {
    pub const fn icon(self) -> crate::ZsIcon {
        match self {
            Self::Informational => crate::ZsIcon::Info,
            Self::Success => crate::ZsIcon::Success,
            Self::Warning => crate::ZsIcon::Warning,
            Self::Error => crate::ZsIcon::Error,
        }
    }
}

/// Application-owned content for one persistent, inline information bar.
///
/// An InfoBar participates in ordinary layout. It never floats over the page
/// and never owns a timeout. Applications decide when to insert or remove it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsInfoBarSpec {
    title: Option<String>,
    message: String,
    severity: ZsInfoBarSeverity,
    action_label: Option<String>,
    closable: bool,
}

impl ZsInfoBarSpec {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            title: None,
            message: message.into(),
            severity: ZsInfoBarSeverity::Informational,
            action_label: None,
            closable: true,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        let title = title.into();
        self.title = (!title.trim().is_empty()).then_some(title);
        self
    }

    pub const fn severity(mut self, severity: ZsInfoBarSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.action_label = (!label.trim().is_empty()).then_some(label);
        self
    }

    pub const fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn title_text(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn info_bar_severity(&self) -> ZsInfoBarSeverity {
        self.severity
    }

    pub fn action_label(&self) -> Option<&str> {
        self.action_label.as_deref()
    }

    pub const fn is_closable(&self) -> bool {
        self.closable
    }

    pub fn is_empty(&self) -> bool {
        self.title
            .as_deref()
            .is_none_or(|title| title.trim().is_empty())
            && self.message.trim().is_empty()
    }

    pub const fn initial_control(&self) -> Option<ZsInfoBarControl> {
        if self.action_label.is_some() {
            Some(ZsInfoBarControl::Action)
        } else if self.closable {
            Some(ZsInfoBarControl::Close)
        } else {
            None
        }
    }

    pub const fn has_control(&self, control: ZsInfoBarControl) -> bool {
        match control {
            ZsInfoBarControl::Action => self.action_label.is_some(),
            ZsInfoBarControl::Close => self.closable,
        }
    }

    pub const fn relative_control(
        &self,
        current: ZsInfoBarControl,
        offset: isize,
    ) -> ZsInfoBarControl {
        if offset == 0 || !(self.action_label.is_some() && self.closable) {
            return current;
        }
        match current {
            ZsInfoBarControl::Action => ZsInfoBarControl::Close,
            ZsInfoBarControl::Close => ZsInfoBarControl::Action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsInfoBarControl {
    Action,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsInfoBarEvent {
    Action,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsInfoBarState {
    pub focused_control: Option<ZsInfoBarControl>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_bar_spec_keeps_semantic_severity_and_one_optional_action() {
        let spec = ZsInfoBarSpec::new("Reconnect to continue working.")
            .title("No Internet")
            .severity(ZsInfoBarSeverity::Error)
            .action("Network Settings");

        assert_eq!(spec.title_text(), Some("No Internet"));
        assert_eq!(spec.info_bar_severity(), ZsInfoBarSeverity::Error);
        assert_eq!(spec.info_bar_severity().icon(), crate::ZsIcon::Error);
        assert_eq!(spec.action_label(), Some("Network Settings"));
        assert!(spec.is_closable());
        assert_eq!(spec.initial_control(), Some(ZsInfoBarControl::Action));
        assert_eq!(
            spec.relative_control(ZsInfoBarControl::Action, 1),
            ZsInfoBarControl::Close
        );
    }

    #[test]
    fn persistent_required_info_bar_can_have_no_interactive_control() {
        let spec = ZsInfoBarSpec::new("Offline mode is required.").closable(false);

        assert_eq!(spec.initial_control(), None);
        assert!(!spec.has_control(ZsInfoBarControl::Action));
        assert!(!spec.has_control(ZsInfoBarControl::Close));
    }
}
