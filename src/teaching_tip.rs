use serde::{Deserialize, Serialize};

/// Preferred side of a targeted teaching tip.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTeachingTipPlacement {
    #[default]
    Auto,
    Top,
    Bottom,
    Left,
    Right,
}

/// Application-owned content for one targeted, transient teaching tip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTeachingTipSpec {
    title: String,
    subtitle: String,
    action_label: Option<String>,
    preferred_placement: ZsTeachingTipPlacement,
}

impl ZsTeachingTipSpec {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            action_label: None,
            preferred_placement: ZsTeachingTipPlacement::Auto,
        }
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.action_label = (!label.trim().is_empty()).then_some(label);
        self
    }

    pub const fn preferred_placement(mut self, placement: ZsTeachingTipPlacement) -> Self {
        self.preferred_placement = placement;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle(&self) -> &str {
        &self.subtitle
    }

    pub fn action_label(&self) -> Option<&str> {
        self.action_label.as_deref()
    }

    pub const fn placement(&self) -> ZsTeachingTipPlacement {
        self.preferred_placement
    }

    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty() && self.subtitle.trim().is_empty()
    }

    pub const fn initial_control(&self) -> ZsTeachingTipControl {
        if self.action_label.is_some() {
            ZsTeachingTipControl::Action
        } else {
            ZsTeachingTipControl::Close
        }
    }

    pub const fn has_control(&self, control: ZsTeachingTipControl) -> bool {
        match control {
            ZsTeachingTipControl::Action => self.action_label.is_some(),
            ZsTeachingTipControl::Close => true,
        }
    }

    pub const fn relative_control(
        &self,
        current: ZsTeachingTipControl,
        offset: isize,
    ) -> ZsTeachingTipControl {
        if offset == 0 || self.action_label.is_none() {
            return current;
        }
        match current {
            ZsTeachingTipControl::Action => ZsTeachingTipControl::Close,
            ZsTeachingTipControl::Close => ZsTeachingTipControl::Action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTeachingTipControl {
    Action,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTeachingTipDismissReason {
    CloseButton,
    EscapeKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTeachingTipResponse {
    Action,
    Dismissed(ZsTeachingTipDismissReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsTeachingTipResult {
    pub response: ZsTeachingTipResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTeachingTipState {
    pub open: bool,
    pub target: crate::WidgetId,
    pub focused_control: ZsTeachingTipControl,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teaching_tip_spec_keeps_targeted_guidance_contract_typed() {
        let spec =
            ZsTeachingTipSpec::new("Save automatically", "Your changes are saved as you work.")
                .action("Review settings")
                .preferred_placement(ZsTeachingTipPlacement::Top);

        assert_eq!(spec.title(), "Save automatically");
        assert_eq!(spec.action_label(), Some("Review settings"));
        assert_eq!(spec.placement(), ZsTeachingTipPlacement::Top);
        assert_eq!(spec.initial_control(), ZsTeachingTipControl::Action);
        assert_eq!(
            spec.relative_control(ZsTeachingTipControl::Action, 1),
            ZsTeachingTipControl::Close
        );
    }

    #[test]
    fn teaching_tip_without_action_keeps_mandatory_close_control() {
        let spec = ZsTeachingTipSpec::new("Keyboard shortcut", "Press Ctrl+S to save.");

        assert_eq!(spec.initial_control(), ZsTeachingTipControl::Close);
        assert!(spec.has_control(ZsTeachingTipControl::Close));
        assert!(!spec.has_control(ZsTeachingTipControl::Action));
    }
}
