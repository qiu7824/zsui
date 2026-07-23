use serde::{Deserialize, Serialize};

/// Semantic content displayed by a noninteractive information badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ZsBadgeContent {
    /// A compact attention dot with no text or icon.
    Dot,
    /// A nonnegative notification count. The capsule grows for more digits.
    Number(u32),
    /// A platform-resolved semantic icon.
    Icon(crate::ZsIcon),
}

impl ZsBadgeContent {
    pub const fn number(value: u32) -> Self {
        Self::Number(value)
    }

    pub const fn icon(icon: crate::ZsIcon) -> Self {
        Self::Icon(icon)
    }
}

/// Theme-aware badge emphasis without platform colors in application code.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsBadgeTone {
    Neutral,
    #[default]
    Accent,
    Success,
    Warning,
    Danger,
}
