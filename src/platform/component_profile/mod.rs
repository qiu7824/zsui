#[cfg(feature = "tabs")]
use crate::ZsTabViewMetrics;
#[cfg(feature = "label")]
use crate::ZsuiSpacingTokens;
#[cfg(any(feature = "label", feature = "button", feature = "tabs"))]
use crate::{ColorRole, TextRole};
use crate::{Dp, ZsPlatformStyle};
#[cfg(feature = "dialog")]
use crate::{ZsContentDialogButton, ZsContentDialogMetrics, ZsContentDialogSpec};

mod gtk;
mod macos;
mod windows;

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
    #[cfg(feature = "tabs")]
    pub tabs: PlatformTabProfile,
    #[cfg(feature = "dialog")]
    pub dialog: PlatformDialogProfile,
    pub shell: PlatformShellProfile,
}

impl PlatformComponentProfile {
    pub(crate) const fn for_style(style: ZsPlatformStyle) -> Self {
        match style {
            ZsPlatformStyle::Windows => windows::profile(),
            ZsPlatformStyle::Macos => macos::profile(),
            ZsPlatformStyle::Gtk => gtk::profile(),
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

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformTabComposition {
    FluentUnderline,
    AppKitSegmented,
    GtkTabBar,
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformTabCycleShortcut {
    None,
    ControlTab,
    ControlPage,
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformTabProfile {
    pub composition: PlatformTabComposition,
    pub metrics: ZsTabViewMetrics,
    pub label_role: TextRole,
    pub arrow_selects: bool,
    pub supports_home_end_focus: bool,
    pub cycle_shortcut: PlatformTabCycleShortcut,
}

#[cfg(feature = "tabs")]
impl PlatformTabProfile {
    pub(crate) const fn equal_item_widths(self) -> bool {
        matches!(self.composition, PlatformTabComposition::AppKitSegmented)
    }

    pub(crate) const fn center_items(self) -> bool {
        matches!(self.composition, PlatformTabComposition::AppKitSegmented)
    }

    pub(crate) const fn draw_strip_border(self) -> bool {
        matches!(
            self.composition,
            PlatformTabComposition::FluentUnderline | PlatformTabComposition::GtkTabBar
        )
    }

    pub(crate) const fn draw_group_outline(self) -> bool {
        matches!(self.composition, PlatformTabComposition::AppKitSegmented)
    }

    pub(crate) const fn draw_header_separators(self) -> bool {
        matches!(self.composition, PlatformTabComposition::FluentUnderline)
    }

    pub(crate) const fn selected_fill(self) -> ColorRole {
        match self.composition {
            PlatformTabComposition::GtkTabBar => ColorRole::Control,
            PlatformTabComposition::FluentUnderline | PlatformTabComposition::AppKitSegmented => {
                ColorRole::SurfaceRaised
            }
        }
    }

    pub(crate) const fn selected_stroke(self) -> Option<ColorRole> {
        match self.composition {
            PlatformTabComposition::FluentUnderline => Some(ColorRole::Border),
            PlatformTabComposition::AppKitSegmented | PlatformTabComposition::GtkTabBar => None,
        }
    }

    pub(crate) const fn leading_label(self, has_icon: bool) -> bool {
        matches!(self.composition, PlatformTabComposition::FluentUnderline) || has_icon
    }
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformDialogComposition {
    FluentEqualActions,
    AppKitTrailingActions,
    GtkTrailingActions,
}

#[cfg(feature = "dialog")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformDialogProfile {
    pub composition: PlatformDialogComposition,
    pub metrics: ZsContentDialogMetrics,
    pub scrim_alpha: u8,
    pub estimated_glyph_width: Dp,
    pub estimated_label_padding: Dp,
}

#[cfg(feature = "dialog")]
impl PlatformDialogProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).dialog
    }

    pub(crate) const fn equal_action_widths(self) -> bool {
        matches!(
            self.composition,
            PlatformDialogComposition::FluentEqualActions
        )
    }

    pub(crate) const fn trailing_actions(self) -> bool {
        matches!(
            self.composition,
            PlatformDialogComposition::AppKitTrailingActions
                | PlatformDialogComposition::GtkTrailingActions
        )
    }

    pub(crate) fn visual_buttons(self, spec: &ZsContentDialogSpec) -> Vec<ZsContentDialogButton> {
        use ZsContentDialogButton::{Close, Primary, Secondary};

        let order = match self.composition {
            PlatformDialogComposition::FluentEqualActions => [Primary, Secondary, Close],
            PlatformDialogComposition::AppKitTrailingActions
            | PlatformDialogComposition::GtkTrailingActions => [Close, Secondary, Primary],
        };
        let mut buttons = order
            .into_iter()
            .filter(|button| spec.has_button(*button))
            .collect::<Vec<_>>();
        if matches!(
            self.composition,
            PlatformDialogComposition::AppKitTrailingActions
        ) {
            if let Some(default) = spec.default_response() {
                if let Some(index) = buttons.iter().position(|button| *button == default) {
                    buttons.remove(index);
                    buttons.push(default);
                }
            }
        }
        buttons
    }

    pub(crate) fn relative_button(
        self,
        spec: &ZsContentDialogSpec,
        current: ZsContentDialogButton,
        offset: isize,
    ) -> ZsContentDialogButton {
        let buttons = self.visual_buttons(spec);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformShellNavigationComposition {
    FluentPane,
    AppKitSourceList,
    GtkSidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformShellSectionComposition {
    FluentCards,
    AppKitForms,
    GtkBoxedLists,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformShellProfile {
    pub style: ZsPlatformStyle,
    pub navigation: PlatformShellNavigationComposition,
    pub sections: PlatformShellSectionComposition,
    pub preferred_navigation_width: Dp,
    pub top_height: Dp,
    pub navigation_start: Dp,
    pub content_gap: Dp,
    pub content_top_gap: Dp,
    pub viewport_mask_height: Dp,
    pub scrollbar_width: Dp,
    pub active_scrollbar_width: Dp,
    pub scrollbar_margin: Dp,
    pub section_header_height: Dp,
    pub section_row_height: Dp,
    pub section_row_gap: Dp,
    pub section_gap: Dp,
    pub section_horizontal_padding: Dp,
    pub section_height_extra: Dp,
    pub section_body_bottom_inset: Dp,
    pub navigation_item_height: Dp,
    pub navigation_item_stride: Dp,
    pub navigation_item_inset: Dp,
    pub navigation_item_radius: Dp,
    pub section_radius: Dp,
    pub title_x: Dp,
    pub title_y: Dp,
    pub title_width: Dp,
    pub title_height: Dp,
    pub menu_icon_x: Dp,
    pub menu_icon_y: Dp,
    pub menu_icon_size: Dp,
    pub show_menu_icon: bool,
    pub app_title_x: Dp,
    pub app_title_y: Dp,
    pub app_title_width: Dp,
    pub app_title_height: Dp,
    pub action_margin: Dp,
    pub action_height: Dp,
    pub primary_action_width: Dp,
    pub secondary_action_width: Dp,
    pub action_gap: Dp,
    pub draw_row_separators: bool,
}

impl PlatformShellProfile {
    pub(crate) fn navigation_width(self, logical_window_width: f32) -> Dp {
        match self.navigation {
            PlatformShellNavigationComposition::GtkSidebar => Dp::new(
                (logical_window_width * 0.25).clamp(180.0, self.preferred_navigation_width.0),
            ),
            PlatformShellNavigationComposition::FluentPane
            | PlatformShellNavigationComposition::AppKitSourceList => {
                self.preferred_navigation_width
            }
        }
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

        #[cfg(feature = "tabs")]
        {
            assert_eq!(
                windows.tabs.composition,
                PlatformTabComposition::FluentUnderline
            );
            assert_eq!(
                macos.tabs.composition,
                PlatformTabComposition::AppKitSegmented
            );
            assert_eq!(gtk.tabs.composition, PlatformTabComposition::GtkTabBar);
            assert_eq!(windows.tabs.label_role, TextRole::Body);
            assert!(macos.tabs.arrow_selects);
            assert!(gtk.tabs.supports_home_end_focus);
        }

        #[cfg(feature = "dialog")]
        {
            use ZsContentDialogButton::{Close, Primary, Secondary};

            assert_eq!(
                windows.dialog.composition,
                PlatformDialogComposition::FluentEqualActions
            );
            assert_eq!(
                macos.dialog.composition,
                PlatformDialogComposition::AppKitTrailingActions
            );
            assert_eq!(
                gtk.dialog.composition,
                PlatformDialogComposition::GtkTrailingActions
            );
            assert!(windows.dialog.equal_action_widths());
            assert!(macos.dialog.trailing_actions());
            assert!(gtk.dialog.trailing_actions());

            let dialog = ZsContentDialogSpec::new("Body", "Close")
                .primary_button("Primary")
                .secondary_button("Secondary")
                .default_button(Secondary);
            assert_eq!(
                windows.dialog.visual_buttons(&dialog),
                vec![Primary, Secondary, Close]
            );
            assert_eq!(
                macos.dialog.visual_buttons(&dialog),
                vec![Close, Primary, Secondary]
            );
            assert_eq!(
                gtk.dialog.visual_buttons(&dialog),
                vec![Close, Secondary, Primary]
            );
            assert_eq!(macos.dialog.relative_button(&dialog, Close, 1), Primary);
            assert_eq!(gtk.dialog.relative_button(&dialog, Close, 1), Secondary);
        }

        assert_eq!(
            windows.shell.navigation,
            PlatformShellNavigationComposition::FluentPane
        );
        assert_eq!(
            macos.shell.sections,
            PlatformShellSectionComposition::AppKitForms
        );
        assert_eq!(
            gtk.shell.sections,
            PlatformShellSectionComposition::GtkBoxedLists
        );
        assert_eq!(gtk.shell.navigation_width(800.0), Dp::new(200.0));
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
