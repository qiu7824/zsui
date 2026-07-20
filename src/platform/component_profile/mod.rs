#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "tabs",
    feature = "grid-view",
    feature = "tree",
    feature = "table",
    feature = "time-picker",
    feature = "color-picker"
))]
use crate::ColorRole;
#[cfg(any(feature = "label", feature = "button", feature = "tabs"))]
use crate::TextRole;
#[cfg(feature = "auto-suggest")]
use crate::ZsAutoSuggestMetrics;
#[cfg(feature = "breadcrumb")]
use crate::ZsBreadcrumbMetrics;
#[cfg(feature = "color-picker")]
use crate::ZsColorPickerMetrics;
#[cfg(feature = "command-palette")]
use crate::ZsCommandPaletteMetrics;
#[cfg(feature = "grid-view")]
use crate::ZsGridViewMetrics;
#[cfg(feature = "info-bar")]
use crate::ZsInfoBarMetrics;
#[cfg(feature = "button")]
use crate::ZsNavigationItemMetrics;
#[cfg(feature = "number-box")]
use crate::ZsNumberBoxMetrics;
#[cfg(feature = "tabs")]
use crate::ZsTabViewMetrics;
#[cfg(feature = "table")]
use crate::ZsTableMetrics;
#[cfg(feature = "teaching-tip")]
use crate::ZsTeachingTipMetrics;
#[cfg(feature = "time-picker")]
use crate::ZsTimePickerMetrics;
#[cfg(feature = "toast")]
use crate::ZsToastMetrics;
#[cfg(feature = "toggle-button")]
use crate::ZsToggleButtonMetrics;
#[cfg(feature = "tree")]
use crate::ZsTreeViewMetrics;
#[cfg(feature = "label")]
use crate::ZsuiSpacingTokens;
use crate::{Dp, ZsBaseControlMetrics, ZsPlatformStyle};
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
    pub base_control: PlatformBaseControlProfile,
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
    #[cfg(feature = "info-bar")]
    pub info_bar: PlatformInfoBarProfile,
    #[cfg(feature = "teaching-tip")]
    pub teaching_tip: PlatformTeachingTipProfile,
    #[cfg(feature = "toast")]
    pub toast: PlatformToastProfile,
    #[cfg(feature = "breadcrumb")]
    pub breadcrumb: PlatformBreadcrumbProfile,
    #[cfg(feature = "toggle-button")]
    pub toggle_button: PlatformToggleButtonProfile,
    #[cfg(feature = "number-box")]
    pub number_box: PlatformNumberBoxProfile,
    #[cfg(feature = "auto-suggest")]
    pub auto_suggest: PlatformAutoSuggestProfile,
    #[cfg(feature = "grid-view")]
    pub grid_view: PlatformGridViewProfile,
    #[cfg(feature = "tree")]
    pub tree_view: PlatformTreeViewProfile,
    #[cfg(feature = "table")]
    pub table: PlatformTableProfile,
    #[cfg(feature = "time-picker")]
    pub time_picker: PlatformTimePickerProfile,
    #[cfg(feature = "color-picker")]
    pub color_picker: PlatformColorPickerProfile,
    #[cfg(feature = "command-palette")]
    pub command_palette: PlatformCommandPaletteProfile,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformBaseControlProfile {
    pub metrics: ZsBaseControlMetrics,
}

impl PlatformBaseControlProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).base_control
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
pub(crate) enum PlatformNavigationItemComposition {
    FluentPaneRow,
    AppKitSourceListRow,
    GtkSidebarRow,
}

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformNavigationItemProfile {
    pub composition: PlatformNavigationItemComposition,
    pub metrics: ZsNavigationItemMetrics,
}

#[cfg(feature = "button")]
impl PlatformNavigationItemProfile {
    pub(crate) const fn draws_selection_indicator(self) -> bool {
        matches!(
            self.composition,
            PlatformNavigationItemComposition::FluentPaneRow
        )
    }

    pub(crate) const fn selected_fill(self) -> (ColorRole, Option<u8>) {
        match self.composition {
            PlatformNavigationItemComposition::FluentPaneRow => (ColorRole::Control, None),
            PlatformNavigationItemComposition::AppKitSourceListRow => (ColorRole::Accent, Some(30)),
            PlatformNavigationItemComposition::GtkSidebarRow => (ColorRole::PrimaryText, Some(26)),
        }
    }

    pub(crate) const fn selected_content_color(self) -> ColorRole {
        match self.composition {
            PlatformNavigationItemComposition::AppKitSourceListRow => ColorRole::Accent,
            PlatformNavigationItemComposition::FluentPaneRow
            | PlatformNavigationItemComposition::GtkSidebarRow => ColorRole::PrimaryText,
        }
    }
}

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformButtonProfile {
    pub fill: ColorRole,
    pub stroke: Option<ColorRole>,
    pub navigation_item: PlatformNavigationItemProfile,
}

#[cfg(feature = "button")]
impl PlatformButtonProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).button
    }
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

#[cfg(any(feature = "info-bar", feature = "teaching-tip", feature = "toast"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformFeedbackActionTreatment {
    NeutralControl,
    #[cfg(any(feature = "info-bar", feature = "toast"))]
    TransparentAccent,
    #[cfg(feature = "teaching-tip")]
    AccentFilled,
}

#[cfg(feature = "info-bar")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformInfoBarComposition {
    FluentStatus,
    AppKitStatus,
    GtkBanner,
}

#[cfg(feature = "info-bar")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformInfoBarProfile {
    pub composition: PlatformInfoBarComposition,
    pub metrics: ZsInfoBarMetrics,
    pub surface_alpha: u8,
}

#[cfg(feature = "info-bar")]
impl PlatformInfoBarProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).info_bar
    }

    pub(crate) const fn action_treatment(self) -> PlatformFeedbackActionTreatment {
        match self.composition {
            PlatformInfoBarComposition::FluentStatus => {
                PlatformFeedbackActionTreatment::NeutralControl
            }
            PlatformInfoBarComposition::AppKitStatus | PlatformInfoBarComposition::GtkBanner => {
                PlatformFeedbackActionTreatment::TransparentAccent
            }
        }
    }
}

#[cfg(feature = "teaching-tip")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformTeachingTipComposition {
    FluentTeachingTip,
    AppKitPopover,
    GtkPopover,
}

#[cfg(feature = "teaching-tip")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformTeachingTipProfile {
    pub composition: PlatformTeachingTipComposition,
    pub metrics: ZsTeachingTipMetrics,
    pub shadow_alpha: u8,
}

#[cfg(feature = "teaching-tip")]
impl PlatformTeachingTipProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).teaching_tip
    }

    pub(crate) const fn action_treatment(self) -> PlatformFeedbackActionTreatment {
        match self.composition {
            PlatformTeachingTipComposition::FluentTeachingTip => {
                PlatformFeedbackActionTreatment::NeutralControl
            }
            PlatformTeachingTipComposition::AppKitPopover
            | PlatformTeachingTipComposition::GtkPopover => {
                PlatformFeedbackActionTreatment::AccentFilled
            }
        }
    }
}

#[cfg(feature = "toast")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformToastComposition {
    FluentForeground,
    AppKitForeground,
    GtkToast,
}

#[cfg(feature = "toast")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformToastProfile {
    pub composition: PlatformToastComposition,
    pub metrics: ZsToastMetrics,
    pub shadow_alpha: u8,
}

#[cfg(feature = "toast")]
impl PlatformToastProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).toast
    }

    pub(crate) const fn action_treatment(self) -> PlatformFeedbackActionTreatment {
        match self.composition {
            PlatformToastComposition::FluentForeground => {
                PlatformFeedbackActionTreatment::NeutralControl
            }
            PlatformToastComposition::AppKitForeground | PlatformToastComposition::GtkToast => {
                PlatformFeedbackActionTreatment::TransparentAccent
            }
        }
    }

    pub(crate) const fn emphasizes_message(self) -> bool {
        matches!(self.composition, PlatformToastComposition::GtkToast)
    }
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformBreadcrumbCollapseBehavior {
    CollapseLeadingAncestors,
    PreserveRootWhenPossible,
}

#[cfg(feature = "breadcrumb")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformBreadcrumbProfile {
    pub metrics: ZsBreadcrumbMetrics,
    collapse_behavior: PlatformBreadcrumbCollapseBehavior,
}

#[cfg(feature = "breadcrumb")]
impl PlatformBreadcrumbProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).breadcrumb
    }

    pub(crate) const fn preserves_root(self) -> bool {
        matches!(
            self.collapse_behavior,
            PlatformBreadcrumbCollapseBehavior::PreserveRootWhenPossible
        )
    }
}

#[cfg(feature = "toggle-button")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformToggleButtonProfile {
    pub metrics: ZsToggleButtonMetrics,
}

#[cfg(feature = "toggle-button")]
impl PlatformToggleButtonProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).toggle_button
    }
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformNumberBoxStepperPresentation {
    ChevronIcons,
    TextSigns,
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformNumberBoxProfile {
    pub metrics: ZsNumberBoxMetrics,
    pub button_icon_size: Dp,
    pub stepper_presentation: PlatformNumberBoxStepperPresentation,
}

#[cfg(feature = "number-box")]
impl PlatformNumberBoxProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).number_box
    }
}

#[cfg(feature = "auto-suggest")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformAutoSuggestProfile {
    pub metrics: ZsAutoSuggestMetrics,
}

#[cfg(feature = "auto-suggest")]
impl PlatformAutoSuggestProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).auto_suggest
    }
}

#[cfg(any(
    feature = "grid-view",
    feature = "tree",
    feature = "table",
    feature = "time-picker"
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformCollectionSelectionProfile {
    fill_role: ColorRole,
    fill_alpha: Option<u8>,
    foreground: ColorRole,
}

#[cfg(any(
    feature = "grid-view",
    feature = "tree",
    feature = "table",
    feature = "time-picker"
))]
impl PlatformCollectionSelectionProfile {
    pub(crate) const fn fill(self) -> (ColorRole, Option<u8>) {
        (self.fill_role, self.fill_alpha)
    }

    pub(crate) const fn foreground(self) -> ColorRole {
        self.foreground
    }
}

#[cfg(feature = "grid-view")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformGridViewProfile {
    pub metrics: ZsGridViewMetrics,
    pub selection: PlatformCollectionSelectionProfile,
}

#[cfg(feature = "grid-view")]
impl PlatformGridViewProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).grid_view
    }
}

#[cfg(feature = "tree")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformTreeViewProfile {
    pub metrics: ZsTreeViewMetrics,
    pub selection: PlatformCollectionSelectionProfile,
}

#[cfg(feature = "tree")]
impl PlatformTreeViewProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).tree_view
    }
}

#[cfg(feature = "table")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformTableProfile {
    pub metrics: ZsTableMetrics,
    pub selection: PlatformCollectionSelectionProfile,
}

#[cfg(feature = "table")]
impl PlatformTableProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).table
    }
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformTimePickerProfile {
    pub metrics: ZsTimePickerMetrics,
    pub header_fill: ColorRole,
    pub selection: PlatformCollectionSelectionProfile,
}

#[cfg(feature = "time-picker")]
impl PlatformTimePickerProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).time_picker
    }
}

#[cfg(feature = "color-picker")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformColorPickerProfile {
    pub metrics: ZsColorPickerMetrics,
    pub swatch_size: Dp,
    pub header_fill: ColorRole,
    pub active_channel_alpha: u8,
}

#[cfg(feature = "color-picker")]
impl PlatformColorPickerProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).color_picker
    }
}

#[cfg(feature = "command-palette")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformCommandPaletteProfile {
    pub metrics: ZsCommandPaletteMetrics,
    pub scrim_alpha: u8,
}

#[cfg(feature = "command-palette")]
impl PlatformCommandPaletteProfile {
    pub(crate) const fn for_platform(platform: ZsPlatformStyle) -> Self {
        PlatformComponentProfile::for_style(platform).command_palette
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
        assert_eq!(windows.base_control.metrics.button_height, Dp::new(32.0));
        assert_eq!(macos.base_control.metrics.button_height, Dp::new(28.0));
        assert_eq!(gtk.base_control.metrics.button_height, Dp::new(34.0));

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
            assert!(windows.button.navigation_item.draws_selection_indicator());
            assert!(!macos.button.navigation_item.draws_selection_indicator());
            assert_eq!(
                gtk.button.navigation_item.selected_fill(),
                (ColorRole::PrimaryText, Some(26))
            );
            assert_eq!(
                macos.button.navigation_item.selected_content_color(),
                ColorRole::Accent
            );
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

        #[cfg(feature = "info-bar")]
        {
            assert_eq!(
                windows.info_bar.composition,
                PlatformInfoBarComposition::FluentStatus
            );
            assert_eq!(
                macos.info_bar.composition,
                PlatformInfoBarComposition::AppKitStatus
            );
            assert_eq!(
                gtk.info_bar.composition,
                PlatformInfoBarComposition::GtkBanner
            );
            assert_eq!(
                windows.info_bar.action_treatment(),
                PlatformFeedbackActionTreatment::NeutralControl
            );
            assert_eq!(
                gtk.info_bar.action_treatment(),
                PlatformFeedbackActionTreatment::TransparentAccent
            );
            assert_eq!(macos.info_bar.surface_alpha, 18);
        }

        #[cfg(feature = "teaching-tip")]
        {
            assert_eq!(
                windows.teaching_tip.composition,
                PlatformTeachingTipComposition::FluentTeachingTip
            );
            assert_eq!(
                macos.teaching_tip.composition,
                PlatformTeachingTipComposition::AppKitPopover
            );
            assert_eq!(
                gtk.teaching_tip.composition,
                PlatformTeachingTipComposition::GtkPopover
            );
            assert_eq!(
                windows.teaching_tip.action_treatment(),
                PlatformFeedbackActionTreatment::NeutralControl
            );
            assert_eq!(
                macos.teaching_tip.action_treatment(),
                PlatformFeedbackActionTreatment::AccentFilled
            );
            assert_eq!(gtk.teaching_tip.shadow_alpha, 34);
        }

        #[cfg(feature = "toast")]
        {
            assert_eq!(
                windows.toast.composition,
                PlatformToastComposition::FluentForeground
            );
            assert_eq!(
                macos.toast.composition,
                PlatformToastComposition::AppKitForeground
            );
            assert_eq!(gtk.toast.composition, PlatformToastComposition::GtkToast);
            assert_eq!(
                windows.toast.action_treatment(),
                PlatformFeedbackActionTreatment::NeutralControl
            );
            assert_eq!(
                macos.toast.action_treatment(),
                PlatformFeedbackActionTreatment::TransparentAccent
            );
            assert!(!windows.toast.emphasizes_message());
            assert!(gtk.toast.emphasizes_message());
        }

        #[cfg(feature = "breadcrumb")]
        {
            assert!(!windows.breadcrumb.preserves_root());
            assert!(macos.breadcrumb.preserves_root());
            assert!(!gtk.breadcrumb.preserves_root());
            assert_eq!(macos.breadcrumb.metrics.control_height, Dp::new(24.0));
        }

        #[cfg(feature = "toggle-button")]
        {
            assert_eq!(windows.toggle_button.metrics.minimum_height, Dp::new(32.0));
            assert_eq!(macos.toggle_button.metrics.minimum_height, Dp::new(28.0));
            assert_eq!(gtk.toggle_button.metrics.minimum_height, Dp::new(34.0));
        }

        #[cfg(feature = "number-box")]
        {
            assert!(windows.number_box.metrics.horizontal_buttons);
            assert!(!macos.number_box.metrics.horizontal_buttons);
            assert_eq!(
                gtk.number_box.stepper_presentation,
                PlatformNumberBoxStepperPresentation::TextSigns
            );
            assert_eq!(macos.number_box.button_icon_size, Dp::new(10.0));
        }

        #[cfg(feature = "auto-suggest")]
        {
            assert!(!windows.auto_suggest.metrics.leading_search_icon);
            assert!(macos.auto_suggest.metrics.leading_search_icon);
            assert_eq!(gtk.auto_suggest.metrics.control_height, Dp::new(34.0));
        }

        #[cfg(feature = "grid-view")]
        {
            assert_eq!(
                windows.grid_view.selection.fill(),
                (ColorRole::Accent, Some(28))
            );
            assert_eq!(
                macos.grid_view.selection.foreground(),
                ColorRole::AccentText
            );
            assert_eq!(gtk.grid_view.metrics.item_height, Dp::new(116.0));
        }

        #[cfg(feature = "tree")]
        {
            assert_eq!(windows.tree_view.metrics.row_height, Dp::new(32.0));
            assert_eq!(macos.tree_view.metrics.row_height, Dp::new(22.0));
            assert_eq!(
                gtk.tree_view.selection.fill(),
                (ColorRole::Accent, Some(48))
            );
        }

        #[cfg(feature = "table")]
        {
            assert_eq!(windows.table.metrics.row_height, Dp::new(32.0));
            assert_eq!(macos.table.selection.foreground(), ColorRole::AccentText);
            assert_eq!(gtk.table.selection.fill(), (ColorRole::Accent, Some(48)));
        }

        #[cfg(feature = "time-picker")]
        {
            assert_eq!(windows.time_picker.header_fill, ColorRole::Control);
            assert_eq!(macos.time_picker.header_fill, ColorRole::Surface);
            assert_eq!(
                macos.time_picker.selection.foreground(),
                ColorRole::AccentText
            );
            assert_eq!(gtk.time_picker.selection.fill(), (ColorRole::Control, None));
        }

        #[cfg(feature = "color-picker")]
        {
            assert_eq!(windows.color_picker.metrics.spectrum_height, Dp::new(256.0));
            assert_eq!(macos.color_picker.swatch_size, Dp::new(18.0));
            assert_eq!(gtk.color_picker.active_channel_alpha, 20);
        }

        #[cfg(feature = "command-palette")]
        {
            assert_eq!(
                windows.command_palette.metrics.preferred_width,
                Dp::new(640.0)
            );
            assert_eq!(macos.command_palette.scrim_alpha, 44);
            assert_eq!(gtk.command_palette.scrim_alpha, 72);
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
