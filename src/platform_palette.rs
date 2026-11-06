//! Read-only snapshots of the palette actually resolved by a native host.
use crate::{Color, ColorRole, ZsuiThemeMode};
use serde::{Deserialize, Serialize};

/// A resolved native appearance snapshot with the same roles used by View paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformPalette {
    /// Effective mode, including system high-contrast overrides.
    pub mode: ZsuiThemeMode,
    pub(crate) colors: [Color; 13],
}

impl PlatformPalette {
    /// Reads the current host palette. Unsupported hosts return an explicit error.
    pub fn current(mode: ZsuiThemeMode) -> crate::ZsuiResult<Self> {
        crate::desktop_runtime::platform_palette(mode)
    }
    /// Resolves one semantic color exactly as the native renderer does.
    pub const fn resolve(self, role: ColorRole) -> Color {
        self.colors[match role {
            ColorRole::PrimaryText => 0,
            ColorRole::SecondaryText => 1,
            ColorRole::DisabledText => 2,
            ColorRole::Accent => 3,
            ColorRole::AccentText => 4,
            ColorRole::Surface => 5,
            ColorRole::SurfaceRaised => 6,
            ColorRole::Control => 7,
            ColorRole::Border => 8,
            ColorRole::StrongStroke => 9,
            ColorRole::Success => 10,
            ColorRole::Warning => 11,
            ColorRole::Danger => 12,
        }]
    }
}
