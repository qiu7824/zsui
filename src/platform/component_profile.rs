#[cfg(any(feature = "label", feature = "button"))]
use crate::ColorRole;
#[cfg(feature = "label")]
use crate::ZsuiSpacingTokens;
use crate::{Dp, TextRole, ZsPlatformStyle};

/// Framework-owned component composition and metric defaults for one design
/// profile.
///
/// This is deliberately separate from the native backend profile. A new
/// backend can reuse an existing component profile while supplying its own
/// Host/Text/Raster/Presenter/Services implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformComponentProfile {
    pub style: ZsPlatformStyle,
    #[cfg(feature = "label")]
    pub section: PlatformSectionProfile,
    #[cfg(feature = "label")]
    pub navigation: PlatformNavigationProfile,
    #[cfg(feature = "button")]
    pub button: PlatformButtonProfile,
    #[cfg(feature = "button")]
    pub command_bar: PlatformCommandBarProfile,
}

impl PlatformComponentProfile {
    pub(crate) const fn for_style(style: ZsPlatformStyle) -> Self {
        match style {
            ZsPlatformStyle::Windows => Self {
                style,
                #[cfg(feature = "label")]
                section: PlatformSectionProfile {
                    composition: PlatformSectionComposition::FluentCard,
                    heading_color: ColorRole::PrimaryText,
                },
                #[cfg(feature = "label")]
                navigation: PlatformNavigationProfile {
                    composition: PlatformNavigationComposition::FluentPane,
                    behavior: PlatformNavigationBehavior::FluentAdaptive,
                    title_role: TextRole::Subtitle,
                    preferred_pane_width: Dp::new(320.0),
                    horizontal_inset: Dp::new(32.0),
                    pane_color: ColorRole::SurfaceRaised,
                    scrim_alpha: 42,
                    toggle_icon_size: Dp::new(20.0),
                },
                #[cfg(feature = "button")]
                button: PlatformButtonProfile {
                    fill: ColorRole::Control,
                    stroke: Some(ColorRole::Border),
                },
                #[cfg(feature = "button")]
                command_bar: PlatformCommandBarProfile {
                    bar_height: Dp::new(48.0),
                    button_height: Dp::new(48.0),
                    icon_size: Dp::new(20.0),
                    content_gap: Dp::new(8.0),
                    item_gap: Dp::new(8.0),
                    label_role: TextRole::Caption,
                },
            },
            ZsPlatformStyle::Macos => Self {
                style,
                #[cfg(feature = "label")]
                section: PlatformSectionProfile {
                    composition: PlatformSectionComposition::AppKitForm,
                    heading_color: ColorRole::SecondaryText,
                },
                #[cfg(feature = "label")]
                navigation: PlatformNavigationProfile {
                    composition: PlatformNavigationComposition::AppKitSourceList,
                    behavior: PlatformNavigationBehavior::AppKitSplitView,
                    title_role: TextRole::Body,
                    preferred_pane_width: Dp::new(240.0),
                    horizontal_inset: Dp::new(24.0),
                    pane_color: ColorRole::SurfaceRaised,
                    scrim_alpha: 32,
                    toggle_icon_size: Dp::new(16.0),
                },
                #[cfg(feature = "button")]
                button: PlatformButtonProfile {
                    fill: ColorRole::Control,
                    stroke: None,
                },
                #[cfg(feature = "button")]
                command_bar: PlatformCommandBarProfile {
                    bar_height: Dp::new(28.0),
                    button_height: Dp::new(28.0),
                    icon_size: Dp::new(16.0),
                    content_gap: Dp::new(6.0),
                    item_gap: Dp::new(6.0),
                    label_role: TextRole::Button,
                },
            },
            ZsPlatformStyle::Gtk => Self {
                style,
                #[cfg(feature = "label")]
                section: PlatformSectionProfile {
                    composition: PlatformSectionComposition::GtkBoxedList,
                    heading_color: ColorRole::PrimaryText,
                },
                #[cfg(feature = "label")]
                navigation: PlatformNavigationProfile {
                    composition: PlatformNavigationComposition::GtkBoxedSidebar,
                    behavior: PlatformNavigationBehavior::GtkAdaptive,
                    title_role: TextRole::Body,
                    preferred_pane_width: Dp::new(280.0),
                    horizontal_inset: Dp::new(32.0),
                    pane_color: ColorRole::Surface,
                    scrim_alpha: 48,
                    toggle_icon_size: Dp::new(16.0),
                },
                #[cfg(feature = "button")]
                button: PlatformButtonProfile {
                    fill: ColorRole::SurfaceRaised,
                    stroke: Some(ColorRole::Border),
                },
                #[cfg(feature = "button")]
                command_bar: PlatformCommandBarProfile {
                    bar_height: Dp::new(34.0),
                    button_height: Dp::new(34.0),
                    icon_size: Dp::new(16.0),
                    content_gap: Dp::new(6.0),
                    item_gap: Dp::new(6.0),
                    label_role: TextRole::Button,
                },
            },
        }
    }

    pub(crate) const fn current() -> Self {
        Self::for_style(ZsPlatformStyle::current())
    }
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformSectionComposition {
    FluentCard,
    AppKitForm,
    GtkBoxedList,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSectionProfile {
    pub composition: PlatformSectionComposition,
    pub heading_color: ColorRole,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformNavigationComposition {
    FluentPane,
    AppKitSourceList,
    GtkBoxedSidebar,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlatformNavigationBehavior {
    FluentAdaptive,
    AppKitSplitView,
    GtkAdaptive,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformNavigationLayoutMode {
    Expanded,
    Compact,
    Collapsed,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformNavigationProfile {
    pub composition: PlatformNavigationComposition,
    behavior: PlatformNavigationBehavior,
    pub title_role: TextRole,
    pub preferred_pane_width: Dp,
    pub horizontal_inset: Dp,
    pub pane_color: ColorRole,
    pub scrim_alpha: u8,
    pub toggle_icon_size: Dp,
}

#[cfg(feature = "label")]
impl PlatformNavigationProfile {
    pub(crate) fn open_pane_width(self, logical_width: f32, override_width: Option<Dp>) -> f32 {
        override_width.map_or_else(
            || match self.behavior {
                PlatformNavigationBehavior::GtkAdaptive => {
                    (logical_width * 0.25).clamp(180.0, self.preferred_pane_width.0)
                }
                PlatformNavigationBehavior::FluentAdaptive
                | PlatformNavigationBehavior::AppKitSplitView => self.preferred_pane_width.0,
            },
            |width| width.0.max(0.0),
        )
    }

    pub(crate) fn layout_mode(
        self,
        logical_width: f32,
        open_pane_width: f32,
        minimum_content_width: f32,
    ) -> PlatformNavigationLayoutMode {
        match self.behavior {
            PlatformNavigationBehavior::FluentAdaptive if logical_width >= 1008.0 => {
                PlatformNavigationLayoutMode::Expanded
            }
            PlatformNavigationBehavior::FluentAdaptive if logical_width > 640.0 => {
                PlatformNavigationLayoutMode::Compact
            }
            PlatformNavigationBehavior::FluentAdaptive => PlatformNavigationLayoutMode::Collapsed,
            PlatformNavigationBehavior::AppKitSplitView
                if minimum_content_width > 0.0
                    && logical_width < open_pane_width + minimum_content_width =>
            {
                PlatformNavigationLayoutMode::Collapsed
            }
            PlatformNavigationBehavior::AppKitSplitView => PlatformNavigationLayoutMode::Expanded,
            PlatformNavigationBehavior::GtkAdaptive => {
                let constraint_breakpoint = if minimum_content_width > 0.0 {
                    180.0 + minimum_content_width
                } else {
                    0.0
                };
                if logical_width <= 400.0_f32.max(constraint_breakpoint) {
                    PlatformNavigationLayoutMode::Collapsed
                } else {
                    PlatformNavigationLayoutMode::Expanded
                }
            }
        }
    }

    pub(crate) const fn compact_width(self) -> Dp {
        Dp::new(48.0)
    }

    pub(crate) const fn pane_padding(self, spacing: ZsuiSpacingTokens) -> Dp {
        match self.composition {
            PlatformNavigationComposition::GtkBoxedSidebar => spacing.md,
            PlatformNavigationComposition::FluentPane
            | PlatformNavigationComposition::AppKitSourceList => spacing.lg,
        }
    }

    pub(crate) const fn collapsed_header_height(self, button_height: Dp, small_spacing: Dp) -> Dp {
        match self.behavior {
            PlatformNavigationBehavior::FluentAdaptive => Dp::new(52.0),
            PlatformNavigationBehavior::AppKitSplitView
            | PlatformNavigationBehavior::GtkAdaptive => {
                Dp::new(button_height.0 + small_spacing.0 * 2.0)
            }
        }
    }
}

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformButtonProfile {
    pub fill: ColorRole,
    pub stroke: Option<ColorRole>,
}

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformCommandBarProfile {
    pub bar_height: Dp,
    pub button_height: Dp,
    pub icon_size: Dp,
    pub content_gap: Dp,
    pub item_gap: Dp,
    pub label_role: TextRole,
}

#[cfg(feature = "button")]
impl PlatformCommandBarProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).command_bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_keep_platform_composition_and_backend_selection_independent() {
        let windows = PlatformComponentProfile::for_style(ZsPlatformStyle::Windows);
        let macos = PlatformComponentProfile::for_style(ZsPlatformStyle::Macos);
        let gtk = PlatformComponentProfile::for_style(ZsPlatformStyle::Gtk);

        assert_eq!(windows.style, ZsPlatformStyle::Windows);
        assert_eq!(macos.style, ZsPlatformStyle::Macos);
        assert_eq!(gtk.style, ZsPlatformStyle::Gtk);

        #[cfg(feature = "label")]
        {
            assert_eq!(
                windows.section.composition,
                PlatformSectionComposition::FluentCard
            );
            assert_eq!(
                macos.section.composition,
                PlatformSectionComposition::AppKitForm
            );
            assert_eq!(
                gtk.navigation.composition,
                PlatformNavigationComposition::GtkBoxedSidebar
            );
        }

        #[cfg(feature = "button")]
        {
            assert_eq!(windows.button.fill, ColorRole::Control);
            assert_eq!(windows.button.stroke, Some(ColorRole::Border));
            assert_eq!(macos.button.stroke, None);
            assert_eq!(gtk.button.fill, ColorRole::SurfaceRaised);
            assert_eq!(windows.command_bar.icon_size, Dp::new(20.0));
            assert_eq!(macos.command_bar.bar_height, Dp::new(28.0));
            assert_eq!(gtk.command_bar.item_gap, Dp::new(6.0));
        }
    }

    #[cfg(feature = "label")]
    #[test]
    fn navigation_adaptation_is_owned_by_the_profile() {
        let windows = PlatformComponentProfile::for_style(ZsPlatformStyle::Windows).navigation;
        let macos = PlatformComponentProfile::for_style(ZsPlatformStyle::Macos).navigation;
        let gtk = PlatformComponentProfile::for_style(ZsPlatformStyle::Gtk).navigation;

        assert_eq!(
            windows.layout_mode(1100.0, 320.0, 500.0),
            PlatformNavigationLayoutMode::Expanded
        );
        assert_eq!(
            windows.layout_mode(800.0, 320.0, 500.0),
            PlatformNavigationLayoutMode::Compact
        );
        assert_eq!(
            macos.layout_mode(700.0, 240.0, 500.0),
            PlatformNavigationLayoutMode::Collapsed
        );
        assert_eq!(
            gtk.layout_mode(420.0, 180.0, 300.0),
            PlatformNavigationLayoutMode::Collapsed
        );
        assert_eq!(gtk.open_pane_width(800.0, None), 200.0);
    }
}
