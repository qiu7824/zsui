use serde::{Deserialize, Serialize};

use crate::{
    render_protocol::{Color, TextRole, ZsTypographyPlatformStyle},
    Dp, TextWeight,
};

pub const ZSUI_FLUENT_GRID_UNIT: i32 = 4;
pub const ZSUI_FLUENT_CONTROL_RADIUS: i32 = 4;
pub const ZSUI_FLUENT_CARD_RADIUS: i32 = 8;
pub const ZSUI_FLUENT_COMPACT_CONTROL_HEIGHT: i32 = 28;
pub const ZSUI_FLUENT_STANDARD_CONTROL_HEIGHT: i32 = 32;
pub const ZSUI_FLUENT_TOUCH_TARGET: i32 = 40;
pub const ZSUI_FLUENT_NAVIGATION_ROW_HEIGHT: i32 = 40;
pub const ZSUI_FLUENT_SMALL_ICON_SIZE: i32 = 16;
pub const ZSUI_FLUENT_STANDARD_ICON_SIZE: i32 = 20;
/// Recommended minimum width for a short-label Windows command button.
pub const ZSUI_WINUI_BUTTON_MIN_WIDTH: i32 = 120;
/// WinUI `ButtonPadding` from the default Button theme resources.
pub const ZSUI_WINUI_BUTTON_PADDING_LEFT: i32 = 11;
pub const ZSUI_WINUI_BUTTON_PADDING_TOP: i32 = 5;
pub const ZSUI_WINUI_BUTTON_PADDING_RIGHT: i32 = 11;
pub const ZSUI_WINUI_BUTTON_PADDING_BOTTOM: i32 = 6;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsuiThemeMode {
    #[default]
    System,
    Light,
    Dark,
    HighContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeColorToken {
    Surface,
    SurfaceRaised,
    TextPrimary,
    TextSecondary,
    Accent,
    Control,
    Border,
    AccentText,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypographyToken {
    Caption,
    Body,
    BodyStrong,
    BodyLarge,
    Subtitle,
    Title,
    TitleLarge,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMetricToken {
    CompactHeight,
    StandardHeight,
    TouchTarget,
    NavigationRowHeight,
    SmallIcon,
    StandardIcon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadiusToken {
    None,
    Small,
    Medium,
    Large,
    Pill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpacingToken {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    ContentGap,
    ContentPadding,
    PagePadding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsuiColorTokens {
    pub surface: Color,
    pub surface_raised: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub control: Color,
    pub border: Color,
    pub accent_text: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsuiTypographyStyle {
    pub size: Dp,
    pub line_height: Dp,
    pub weight: TextWeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZsuiTypographyTokens {
    pub caption: ZsuiTypographyStyle,
    pub body: ZsuiTypographyStyle,
    pub body_strong: ZsuiTypographyStyle,
    pub body_large: ZsuiTypographyStyle,
    pub subtitle: ZsuiTypographyStyle,
    pub title: ZsuiTypographyStyle,
    pub title_large: ZsuiTypographyStyle,
    pub display: ZsuiTypographyStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsuiControlMetrics {
    pub compact_height: Dp,
    pub standard_height: Dp,
    pub touch_target: Dp,
    pub navigation_row_height: Dp,
    pub small_icon: Dp,
    pub standard_icon: Dp,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsuiRadiusTokens {
    pub small: Dp,
    pub medium: Dp,
    pub large: Dp,
    pub pill: Dp,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsuiSpacingTokens {
    pub xs: Dp,
    pub sm: Dp,
    pub md: Dp,
    pub lg: Dp,
    pub xl: Dp,
    pub content_gap: Dp,
    pub content_padding: Dp,
    pub page_padding: Dp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsuiTheme {
    pub colors: ZsuiColorTokens,
    pub radius: ZsuiRadiusTokens,
    pub spacing: ZsuiSpacingTokens,
    pub typography: ZsuiTypographyTokens,
    pub controls: ZsuiControlMetrics,
}

impl ZsuiTheme {
    pub fn light() -> Self {
        Self {
            colors: ZsuiColorTokens {
                surface: Color::rgb(243, 243, 243),
                surface_raised: Color::rgb(255, 255, 255),
                text_primary: Color::rgb(27, 27, 27),
                text_secondary: Color::rgb(97, 97, 97),
                accent: Color::rgb(0, 103, 192),
                control: Color::rgb(255, 255, 255),
                border: Color::rgb(209, 209, 209),
                accent_text: Color::rgb(255, 255, 255),
                success: Color::rgb(15, 123, 15),
                warning: Color::rgb(157, 93, 0),
                danger: Color::rgb(196, 43, 28),
            },
            radius: ZsuiRadiusTokens::default(),
            spacing: ZsuiSpacingTokens::default(),
            typography: ZsuiTypographyTokens::default(),
            controls: ZsuiControlMetrics::default(),
        }
    }

    pub fn dark() -> Self {
        Self {
            colors: ZsuiColorTokens {
                surface: Color::rgb(30, 32, 36),
                surface_raised: Color::rgb(39, 42, 48),
                text_primary: Color::rgb(241, 243, 245),
                text_secondary: Color::rgb(177, 184, 194),
                accent: Color::rgb(84, 150, 255),
                control: Color::rgb(54, 58, 66),
                border: Color::rgb(77, 77, 77),
                accent_text: Color::rgb(0, 0, 0),
                success: Color::rgb(108, 203, 95),
                warning: Color::rgb(255, 200, 61),
                danger: Color::rgb(255, 98, 116),
            },
            radius: ZsuiRadiusTokens::default(),
            spacing: ZsuiSpacingTokens::default(),
            typography: ZsuiTypographyTokens::default(),
            controls: ZsuiControlMetrics::default(),
        }
    }

    /// Returns the deterministic fallback used when a backend cannot resolve
    /// the operating system's user-selected high-contrast colors.
    ///
    /// Native desktop renderers should prefer their platform semantic colors
    /// while the system high-contrast appearance is active.
    pub fn high_contrast(dark: bool) -> Self {
        let (surface, text_primary, accent, accent_text) = if dark {
            (
                Color::rgb(0, 0, 0),
                Color::rgb(255, 255, 255),
                Color::rgb(255, 255, 0),
                Color::rgb(0, 0, 0),
            )
        } else {
            (
                Color::rgb(255, 255, 255),
                Color::rgb(0, 0, 0),
                Color::rgb(0, 0, 128),
                Color::rgb(255, 255, 255),
            )
        };
        Self {
            colors: ZsuiColorTokens {
                surface,
                surface_raised: surface,
                text_primary,
                text_secondary: text_primary,
                accent,
                control: surface,
                border: text_primary,
                accent_text,
                success: text_primary,
                warning: text_primary,
                danger: text_primary,
            },
            radius: ZsuiRadiusTokens::default(),
            spacing: ZsuiSpacingTokens::default(),
            typography: ZsuiTypographyTokens::default(),
            controls: ZsuiControlMetrics::default(),
        }
    }

    pub fn color(&self, token: ThemeColorToken) -> Color {
        match token {
            ThemeColorToken::Surface => self.colors.surface,
            ThemeColorToken::SurfaceRaised => self.colors.surface_raised,
            ThemeColorToken::TextPrimary => self.colors.text_primary,
            ThemeColorToken::TextSecondary => self.colors.text_secondary,
            ThemeColorToken::Accent => self.colors.accent,
            ThemeColorToken::Control => self.colors.control,
            ThemeColorToken::Border => self.colors.border,
            ThemeColorToken::AccentText => self.colors.accent_text,
            ThemeColorToken::Success => self.colors.success,
            ThemeColorToken::Warning => self.colors.warning,
            ThemeColorToken::Danger => self.colors.danger,
        }
    }

    pub fn typography(&self, token: TypographyToken) -> ZsuiTypographyStyle {
        match token {
            TypographyToken::Caption => self.typography.caption,
            TypographyToken::Body => self.typography.body,
            TypographyToken::BodyStrong => self.typography.body_strong,
            TypographyToken::BodyLarge => self.typography.body_large,
            TypographyToken::Subtitle => self.typography.subtitle,
            TypographyToken::Title => self.typography.title,
            TypographyToken::TitleLarge => self.typography.title_large,
            TypographyToken::Display => self.typography.display,
        }
    }

    pub fn control_metric(&self, token: ControlMetricToken) -> Dp {
        match token {
            ControlMetricToken::CompactHeight => self.controls.compact_height,
            ControlMetricToken::StandardHeight => self.controls.standard_height,
            ControlMetricToken::TouchTarget => self.controls.touch_target,
            ControlMetricToken::NavigationRowHeight => self.controls.navigation_row_height,
            ControlMetricToken::SmallIcon => self.controls.small_icon,
            ControlMetricToken::StandardIcon => self.controls.standard_icon,
        }
    }

    pub fn radius(&self, token: RadiusToken) -> Dp {
        match token {
            RadiusToken::None => Dp::new(0.0),
            RadiusToken::Small => self.radius.small,
            RadiusToken::Medium => self.radius.medium,
            RadiusToken::Large => self.radius.large,
            RadiusToken::Pill => self.radius.pill,
        }
    }

    pub fn spacing(&self, token: SpacingToken) -> Dp {
        match token {
            SpacingToken::Xs => self.spacing.xs,
            SpacingToken::Sm => self.spacing.sm,
            SpacingToken::Md => self.spacing.md,
            SpacingToken::Lg => self.spacing.lg,
            SpacingToken::Xl => self.spacing.xl,
            SpacingToken::ContentGap => self.spacing.content_gap,
            SpacingToken::ContentPadding => self.spacing.content_padding,
            SpacingToken::PagePadding => self.spacing.page_padding,
        }
    }
}

impl Default for ZsuiTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl Default for ZsuiRadiusTokens {
    fn default() -> Self {
        Self::for_platform(crate::ZsBaseControlPlatformStyle::current())
    }
}

impl ZsuiRadiusTokens {
    pub(crate) const fn for_platform(platform: crate::ZsBaseControlPlatformStyle) -> Self {
        crate::platform_component_profile::PlatformStyleTokenProfile::for_platform(platform).radius
    }
}

impl Default for ZsuiSpacingTokens {
    fn default() -> Self {
        Self::for_platform(crate::ZsBaseControlPlatformStyle::current())
    }
}

impl ZsuiSpacingTokens {
    /// Returns the native spacing scale and semantic content insets for one
    /// desktop family. Normal applications use [`Default`] and never select a
    /// platform; the explicit form exists for framework proofs.
    pub(crate) const fn for_platform(platform: crate::ZsBaseControlPlatformStyle) -> Self {
        crate::platform_component_profile::PlatformStyleTokenProfile::for_platform(platform).spacing
    }
}

impl Default for ZsuiTypographyTokens {
    fn default() -> Self {
        Self::for_platform(ZsTypographyPlatformStyle::current())
    }
}

impl ZsuiTypographyTokens {
    pub fn for_platform(platform: ZsTypographyPlatformStyle) -> Self {
        fn role_style(role: TextRole, platform: ZsTypographyPlatformStyle) -> ZsuiTypographyStyle {
            let metrics = role.metrics_for(platform);
            ZsuiTypographyStyle::new(metrics.size, metrics.line_height, metrics.default_weight)
        }

        let body = TextRole::Body.metrics_for(platform);
        Self {
            caption: role_style(TextRole::Caption, platform),
            body: role_style(TextRole::Body, platform),
            body_strong: ZsuiTypographyStyle::new(
                body.size,
                body.line_height,
                TextWeight::Semibold,
            ),
            body_large: role_style(TextRole::BodyLarge, platform),
            subtitle: role_style(TextRole::Subtitle, platform),
            title: role_style(TextRole::Title, platform),
            title_large: role_style(TextRole::TitleLarge, platform),
            display: role_style(TextRole::Display, platform),
        }
    }
}

impl ZsuiTypographyStyle {
    pub const fn new(size: f32, line_height: f32, weight: TextWeight) -> Self {
        Self {
            size: Dp::new(size),
            line_height: Dp::new(line_height),
            weight,
        }
    }
}

impl Default for ZsuiControlMetrics {
    fn default() -> Self {
        Self::for_platform(crate::ZsBaseControlPlatformStyle::current())
    }
}

impl ZsuiControlMetrics {
    pub(crate) const fn for_platform(platform: crate::ZsBaseControlPlatformStyle) -> Self {
        crate::platform_component_profile::PlatformStyleTokenProfile::for_platform(platform)
            .controls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_resolves_tokens_without_scattered_literals() {
        let theme = ZsuiTheme::light();

        assert_eq!(
            theme.spacing(SpacingToken::Md),
            ZsuiSpacingTokens::default().md
        );
        assert_eq!(
            theme.radius(RadiusToken::Medium),
            ZsuiRadiusTokens::default().medium
        );
        assert_eq!(theme.color(ThemeColorToken::Accent).a, 255);
        assert_eq!(
            theme.typography(TypographyToken::BodyStrong).weight,
            TextWeight::Semibold
        );
        assert_eq!(
            theme.typography(TypographyToken::TitleLarge),
            ZsuiTypographyTokens::for_platform(ZsTypographyPlatformStyle::current()).title_large
        );
        assert_eq!(
            theme.typography(TypographyToken::Display),
            ZsuiTypographyTokens::for_platform(ZsTypographyPlatformStyle::current()).display
        );
        assert_eq!(
            theme.control_metric(ControlMetricToken::StandardHeight),
            ZsuiControlMetrics::default().standard_height
        );
    }

    #[test]
    fn typography_tokens_are_platform_profiles_not_fluent_globals() {
        let windows = ZsuiTypographyTokens::for_platform(ZsTypographyPlatformStyle::Windows);
        let macos = ZsuiTypographyTokens::for_platform(ZsTypographyPlatformStyle::Macos);
        let gtk = ZsuiTypographyTokens::for_platform(ZsTypographyPlatformStyle::Gtk);

        assert_eq!(
            windows.body,
            ZsuiTypographyStyle::new(14.0, 20.0, TextWeight::Regular)
        );
        assert_eq!(
            macos.body,
            ZsuiTypographyStyle::new(13.0, 16.0, TextWeight::Regular)
        );
        assert_eq!(
            macos.title,
            ZsuiTypographyStyle::new(22.0, 26.0, TextWeight::Regular)
        );
        assert_eq!(
            gtk.caption,
            ZsuiTypographyStyle::new(11.5, 16.0, TextWeight::Regular)
        );
    }

    #[test]
    fn spacing_tokens_keep_platform_density_out_of_application_branches() {
        let windows = ZsuiSpacingTokens::for_platform(crate::ZsBaseControlPlatformStyle::Windows);
        let macos = ZsuiSpacingTokens::for_platform(crate::ZsBaseControlPlatformStyle::Macos);
        let gtk = ZsuiSpacingTokens::for_platform(crate::ZsBaseControlPlatformStyle::Gtk);

        assert_eq!(windows.content_gap, Dp::new(10.0));
        assert_eq!(macos.content_gap, Dp::new(8.0));
        assert_eq!(gtk.content_gap, Dp::new(12.0));
        assert_eq!(windows.content_padding, Dp::new(12.0));
        assert_eq!(macos.content_padding, Dp::new(12.0));
        assert_eq!(gtk.content_padding, Dp::new(16.0));
        assert_eq!(windows.page_padding, Dp::new(24.0));
        assert_eq!(macos.page_padding, Dp::new(20.0));
        assert_eq!(gtk.page_padding, Dp::new(24.0));
    }

    #[test]
    fn radius_and_control_tokens_follow_the_same_platform_profiles_as_widgets() {
        let windows = ZsuiControlMetrics::for_platform(crate::ZsBaseControlPlatformStyle::Windows);
        let macos = ZsuiControlMetrics::for_platform(crate::ZsBaseControlPlatformStyle::Macos);
        let gtk = ZsuiControlMetrics::for_platform(crate::ZsBaseControlPlatformStyle::Gtk);

        assert_eq!(windows.standard_height, Dp::new(32.0));
        assert_eq!(macos.standard_height, Dp::new(28.0));
        assert_eq!(gtk.standard_height, Dp::new(34.0));
        assert_eq!(windows.navigation_row_height, Dp::new(36.0));
        assert_eq!(macos.navigation_row_height, Dp::new(28.0));
        assert_eq!(gtk.navigation_row_height, Dp::new(34.0));
        assert_eq!(
            ZsuiRadiusTokens::for_platform(crate::ZsBaseControlPlatformStyle::Windows).medium,
            Dp::new(8.0)
        );
        assert_eq!(
            ZsuiRadiusTokens::for_platform(crate::ZsBaseControlPlatformStyle::Macos).medium,
            Dp::new(6.0)
        );
        assert_eq!(
            ZsuiRadiusTokens::for_platform(crate::ZsBaseControlPlatformStyle::Gtk).medium,
            Dp::new(12.0)
        );
    }

    #[test]
    fn high_contrast_fallback_keeps_text_borders_and_selection_unambiguous() {
        let dark = ZsuiTheme::high_contrast(true);
        assert_eq!(dark.colors.surface, Color::rgb(0, 0, 0));
        assert_eq!(dark.colors.text_primary, Color::rgb(255, 255, 255));
        assert_eq!(dark.colors.border, dark.colors.text_primary);
        assert_eq!(dark.colors.accent, Color::rgb(255, 255, 0));
        assert_eq!(dark.colors.accent_text, dark.colors.surface);

        let light = ZsuiTheme::high_contrast(false);
        assert_eq!(light.colors.surface, Color::rgb(255, 255, 255));
        assert_eq!(light.colors.text_primary, Color::rgb(0, 0, 0));
        assert_eq!(light.colors.border, light.colors.text_primary);
        assert_ne!(light.colors.accent, light.colors.surface);
        assert_ne!(light.colors.accent_text, light.colors.accent);
    }
}
