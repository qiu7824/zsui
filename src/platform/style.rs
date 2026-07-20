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
        match crate::platform_experience::PlatformExperience::current() {
            Some(experience) => match experience.shared_component_style() {
                Some(style) => style,
                None => Self::Windows,
            },
            None => Self::Windows,
        }
    }

    pub const fn typography(self) -> Self {
        self
    }

    #[cfg(feature = "tabs")]
    pub const fn arrow_selects(self) -> bool {
        crate::platform_component_profile::PlatformComponentProfile::for_style(self)
            .tabs
            .arrow_selects
    }

    #[cfg(feature = "tabs")]
    pub const fn supports_home_end_focus(self) -> bool {
        crate::platform_component_profile::PlatformComponentProfile::for_style(self)
            .tabs
            .supports_home_end_focus
    }

    #[cfg(feature = "time-picker")]
    pub const fn default_clock(self) -> crate::ZsClockFormat {
        crate::platform_component_profile::PlatformTimePickerProfile::for_platform(self)
            .default_clock
    }

    #[cfg(feature = "password-box")]
    pub const fn default_password_reveal_mode(self) -> crate::ZsPasswordRevealMode {
        crate::platform_component_profile::PlatformPasswordBoxProfile::for_platform(self)
            .default_reveal_mode
    }
}

#[cfg(test)]
mod tests {
    use super::ZsPlatformStyle;

    #[test]
    fn shared_profile_preserves_typography_identity() {
        assert_eq!(
            ZsPlatformStyle::Windows.typography(),
            ZsPlatformStyle::Windows
        );
    }

    #[cfg(feature = "tabs")]
    #[test]
    fn shared_profile_delegates_tab_interaction_conventions() {
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

    #[cfg(feature = "password-box")]
    #[test]
    fn shared_profile_preserves_password_reveal_defaults() {
        assert_eq!(
            ZsPlatformStyle::Windows.default_password_reveal_mode(),
            crate::ZsPasswordRevealMode::Peek
        );
        assert_eq!(
            ZsPlatformStyle::Macos.default_password_reveal_mode(),
            crate::ZsPasswordRevealMode::Hidden
        );
        assert_eq!(
            ZsPlatformStyle::Gtk.default_password_reveal_mode(),
            crate::ZsPasswordRevealMode::Hidden
        );
    }
}
