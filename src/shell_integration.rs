//! Typed Shell transfer and menu requests. Product file operations stay in the application.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A completed drag-and-drop operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropEffect {
    /// The target accepted a copy.
    Copy,
    /// The target accepted a move.
    Move,
    /// The target accepted a link.
    Link,
}

/// Synchronous acknowledgement of a native drop operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropResponse {
    /// The application completed the negotiated operation.
    Accepted(DropEffect),
    /// The application declined the drop.
    Rejected,
}

pub(crate) fn accepted_drop_effect(
    candidate: DropEffect,
    delivered: bool,
    response: Option<DropResponse>,
) -> Option<DropEffect> {
    if !delivered {
        return None;
    }
    match response {
        Some(DropResponse::Accepted(effect)) if effect == candidate => Some(effect),
        None if candidate == DropEffect::Copy => Some(DropEffect::Copy),
        _ => None,
    }
}

/// Explicitly permitted transfer operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DragEffects(u8);
impl DragEffects {
    /// Copy without modifying the source.
    pub const COPY: Self = Self(1);
    /// Move; the accepting application owns the actual move operation.
    pub const MOVE: Self = Self(2);
    /// Create a link.
    pub const LINK: Self = Self(4);
    /// Allows both copy and move.
    pub const COPY_MOVE: Self = Self(3);
    /// Allows all standard Shell operations.
    pub const ALL: Self = Self(7);
    /// Returns whether this set permits an operation.
    pub const fn allows(self, effect: DropEffect) -> bool {
        self.0
            & match effect {
                DropEffect::Copy => 1,
                DropEffect::Move => 2,
                DropEffect::Link => 4,
            }
            != 0
    }
    pub(crate) const fn bits(self) -> u32 {
        (self.0 & 7) as u32
    }
}

/// Shell items supplied by absolute filesystem paths or Shell parsing names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragPayload {
    /// Paths interpreted by the operating system's Shell namespace.
    ShellItems(Vec<PathBuf>),
}

/// A drag completed or was explicitly cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragOutcome {
    /// Escape, a rejected target, or release outside a target cancelled the drag.
    Cancelled,
    /// The target accepted the indicated operation.
    Performed(DropEffect),
}

/// Files delivered to a declared View drop target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesDropped {
    /// Absolute filesystem paths supplied by the Shell data object.
    pub files: Vec<PathBuf>,
    /// Negotiated operation. The application performs the corresponding work.
    pub effect: DropEffect,
}

/// A native context-menu request using client-area physical coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellMenuSpec {
    /// Items whose native Shell verbs are presented.
    pub items: Vec<PathBuf>,
    /// Application commands appended to the native menu.
    pub application: crate::MenuSpec,
    /// Menu origin in the requesting window's client coordinates.
    pub position: crate::Point,
}

/// Selection from a merged Shell and application menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellMenuOutcome {
    /// The menu was dismissed without a selection.
    Cancelled,
    /// A Shell extension's verb was invoked by the native host.
    ShellInvoked,
    /// An application-owned typed command was selected.
    Application(crate::Command),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileDropPlacement {
    pub widget: crate::WidgetId,
    pub bounds: crate::Rect,
    pub effects: DragEffects,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moves_require_an_explicit_synchronous_acknowledgement() {
        assert_eq!(accepted_drop_effect(DropEffect::Move, true, None), None);
        assert_eq!(
            accepted_drop_effect(
                DropEffect::Move,
                true,
                Some(DropResponse::Accepted(DropEffect::Move))
            ),
            Some(DropEffect::Move)
        );
        assert_eq!(
            accepted_drop_effect(
                DropEffect::Move,
                true,
                Some(DropResponse::Accepted(DropEffect::Copy))
            ),
            None
        );
        assert_eq!(
            accepted_drop_effect(DropEffect::Copy, true, None),
            Some(DropEffect::Copy)
        );
        assert_eq!(
            accepted_drop_effect(DropEffect::Copy, true, Some(DropResponse::Rejected)),
            None
        );
        assert_eq!(accepted_drop_effect(DropEffect::Copy, false, None), None);
    }
}
