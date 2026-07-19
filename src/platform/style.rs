use serde::{Deserialize, Serialize};

/// Deterministic desktop design profile used by low-level render contracts.
///
/// Ordinary applications declare semantic components and theme tokens instead
/// of selecting this profile. The framework resolves the current value through
/// its private platform experience, while render/proof code can select a
/// profile explicitly to produce deterministic platform evidence.
///
/// A future backend may reuse an existing design profile or add its mapping in
/// the platform experience without introducing another component-specific
/// platform enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

impl ZsPlatformStyle {
    /// Resolves the design profile registered for the current build target.
    pub const fn current() -> Self {
        crate::platform_experience::PlatformExperience::current_or_desktop_fallback()
            .select_desktop(Self::Windows, Self::Macos, Self::Gtk, Self::Windows)
    }

    pub const fn typography(self) -> Self {
        self
    }

    pub const fn arrow_selects(self) -> bool {
        matches!(self, Self::Macos)
    }

    pub const fn supports_home_end_focus(self) -> bool {
        matches!(self, Self::Gtk)
    }

    #[cfg(feature = "time-picker")]
    pub const fn default_clock(self) -> crate::ZsClockFormat {
        match self {
            Self::Windows => crate::ZsClockFormat::TwelveHour,
            Self::Macos | Self::Gtk => crate::ZsClockFormat::TwentyFourHour,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ZsPlatformStyle;

    #[test]
    fn shared_profile_preserves_platform_interaction_conventions() {
        assert_eq!(
            ZsPlatformStyle::Windows.typography(),
            ZsPlatformStyle::Windows
        );
        assert!(!ZsPlatformStyle::Windows.arrow_selects());
        assert!(ZsPlatformStyle::Macos.arrow_selects());
        assert!(!ZsPlatformStyle::Gtk.arrow_selects());
        assert!(!ZsPlatformStyle::Windows.supports_home_end_focus());
        assert!(!ZsPlatformStyle::Macos.supports_home_end_focus());
        assert!(ZsPlatformStyle::Gtk.supports_home_end_focus());
    }

    #[cfg(feature = "time-picker")]
    #[test]
    fn shared_profile_preserves_clock_defaults() {
        assert_eq!(
            ZsPlatformStyle::Windows.default_clock(),
            crate::ZsClockFormat::TwelveHour
        );
        assert_eq!(
            ZsPlatformStyle::Macos.default_clock(),
            crate::ZsClockFormat::TwentyFourHour
        );
        assert_eq!(
            ZsPlatformStyle::Gtk.default_clock(),
            crate::ZsClockFormat::TwentyFourHour
        );
    }
}
