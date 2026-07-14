use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::WidgetId;

/// Stable application identity for one in-app toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsToastId(pub u64);

impl ZsToastId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// How long a toast remains visible if the user does not respond.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsToastDuration {
    /// Five seconds, suitable for ordinary confirmation and undo feedback.
    #[default]
    Short,
    /// Ten seconds, suitable for messages that need more reading time.
    Long,
    /// Remain visible until the application replaces it or the user closes it.
    Persistent,
}

impl ZsToastDuration {
    pub const fn timeout_ms(self) -> Option<u64> {
        match self {
            Self::Short => Some(5_000),
            Self::Long => Some(10_000),
            Self::Persistent => None,
        }
    }
}

/// Immutable application-owned content for one transient in-app toast.
///
/// Toasts intentionally expose at most one action plus a close control. This
/// keeps the same application contract compatible with WinUI-like transient
/// tips, understated macOS foreground feedback and GTK/libadwaita toasts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsToastSpec {
    id: ZsToastId,
    message: String,
    action_label: Option<String>,
    duration: ZsToastDuration,
}

impl ZsToastSpec {
    pub fn new(id: impl Into<ZsToastId>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            action_label: None,
            duration: ZsToastDuration::Short,
        }
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.action_label = (!label.trim().is_empty()).then_some(label);
        self
    }

    pub const fn duration(mut self, duration: ZsToastDuration) -> Self {
        self.duration = duration;
        self
    }

    pub const fn id(&self) -> ZsToastId {
        self.id
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn action_label(&self) -> Option<&str> {
        self.action_label.as_deref()
    }

    pub const fn toast_duration(&self) -> ZsToastDuration {
        self.duration
    }

    pub fn is_empty(&self) -> bool {
        self.message.trim().is_empty()
    }

    pub(crate) const fn initial_control(&self) -> ZsToastControl {
        if self.action_label.is_some() {
            ZsToastControl::Action
        } else {
            ZsToastControl::Close
        }
    }

    pub(crate) const fn has_control(&self, control: ZsToastControl) -> bool {
        match control {
            ZsToastControl::Action => self.action_label.is_some(),
            ZsToastControl::Close => true,
        }
    }

    pub(crate) const fn relative_control(
        &self,
        current: ZsToastControl,
        offset: isize,
    ) -> ZsToastControl {
        if self.action_label.is_none() || offset == 0 {
            return ZsToastControl::Close;
        }
        match current {
            ZsToastControl::Action => ZsToastControl::Close,
            ZsToastControl::Close => ZsToastControl::Action,
        }
    }
}

impl From<u64> for ZsToastId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsToastControl {
    Action,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsToastDismissReason {
    CloseButton,
    EscapeKey,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsToastResponse {
    Action,
    Dismissed(ZsToastDismissReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsToastResult {
    pub id: ZsToastId,
    pub response: ZsToastResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsToastState {
    pub toast: Option<ZsToastId>,
    pub focused_control: ZsToastControl,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ZsToastRuntime {
    active: Option<(WidgetId, ZsToastId)>,
    hide_at: Option<Instant>,
}

impl ZsToastRuntime {
    pub(crate) fn sync(&mut self, active: Option<(WidgetId, &ZsToastSpec)>, now: Instant) -> bool {
        let next = active.map(|(widget, spec)| (widget, spec.id()));
        if self.active == next {
            return false;
        }
        self.active = next;
        self.hide_at = active.and_then(|(_, spec)| {
            spec.toast_duration()
                .timeout_ms()
                .map(|milliseconds| now + Duration::from_millis(milliseconds))
        });
        true
    }

    pub(crate) fn poll_interval_ms(&self, now: Instant) -> Option<u64> {
        self.hide_at.map(|deadline| {
            deadline
                .saturating_duration_since(now)
                .as_millis()
                .clamp(1, u64::MAX as u128) as u64
        })
    }

    pub(crate) fn take_expired(&mut self, now: Instant) -> Option<(WidgetId, ZsToastId)> {
        if !self.hide_at.is_some_and(|deadline| now >= deadline) {
            return None;
        }
        self.hide_at = None;
        self.active.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_spec_keeps_one_typed_action_and_platform_neutral_duration() {
        let spec = ZsToastSpec::new(7, "File deleted")
            .action("Undo")
            .duration(ZsToastDuration::Long);

        assert_eq!(spec.id(), ZsToastId::new(7));
        assert_eq!(spec.action_label(), Some("Undo"));
        assert_eq!(spec.toast_duration().timeout_ms(), Some(10_000));
        assert_eq!(spec.initial_control(), ZsToastControl::Action);
        assert_eq!(
            spec.relative_control(ZsToastControl::Action, 1),
            ZsToastControl::Close
        );
    }

    #[test]
    fn runtime_restarts_only_when_the_stable_toast_identity_changes() {
        let start = Instant::now();
        let widget = WidgetId::new(2);
        let first = ZsToastSpec::new(11, "Saved");
        let replacement = ZsToastSpec::new(12, "Moved");
        let mut runtime = ZsToastRuntime::default();

        assert!(runtime.sync(Some((widget, &first)), start));
        assert!(!runtime.sync(Some((widget, &first)), start + Duration::from_secs(2)));
        assert_eq!(
            runtime.take_expired(start + Duration::from_millis(4_999)),
            None
        );
        assert_eq!(
            runtime.take_expired(start + Duration::from_secs(5)),
            Some((widget, first.id()))
        );
        assert!(runtime.sync(Some((widget, &replacement)), start));
    }
}
