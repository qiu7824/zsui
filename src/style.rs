use serde::{Deserialize, Serialize};

use crate::{render_protocol::Color, Dp, TextWeight};

pub const ZSUI_FLUENT_GRID_UNIT: i32 = 4;
pub const ZSUI_FLUENT_CONTROL_RADIUS: i32 = 4;
pub const ZSUI_FLUENT_CARD_RADIUS: i32 = 8;
pub const ZSUI_FLUENT_COMPACT_CONTROL_HEIGHT: i32 = 28;
pub const ZSUI_FLUENT_STANDARD_CONTROL_HEIGHT: i32 = 32;
pub const ZSUI_FLUENT_TOUCH_TARGET: i32 = 40;
pub const ZSUI_FLUENT_NAVIGATION_ROW_HEIGHT: i32 = 40;
pub const ZSUI_FLUENT_SMALL_ICON_SIZE: i32 = 16;
pub const ZSUI_FLUENT_STANDARD_ICON_SIZE: i32 = 20;

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
pub struct ZsuiTypographyTokens {
    pub caption: ZsuiTypographyStyle,
    pub body: ZsuiTypographyStyle,
    pub body_strong: ZsuiTypographyStyle,
    pub body_large: ZsuiTypographyStyle,
    pub subtitle: ZsuiTypographyStyle,
    pub title: ZsuiTypographyStyle,
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
        Self {
            small: Dp::new(ZSUI_FLUENT_CONTROL_RADIUS as f32),
            medium: Dp::new(ZSUI_FLUENT_CARD_RADIUS as f32),
            large: Dp::new(12.0),
            pill: Dp::new(999.0),
        }
    }
}

impl Default for ZsuiSpacingTokens {
    fn default() -> Self {
        Self {
            xs: Dp::new(ZSUI_FLUENT_GRID_UNIT as f32),
            sm: Dp::new((ZSUI_FLUENT_GRID_UNIT * 2) as f32),
            md: Dp::new((ZSUI_FLUENT_GRID_UNIT * 3) as f32),
            lg: Dp::new((ZSUI_FLUENT_GRID_UNIT * 4) as f32),
            xl: Dp::new((ZSUI_FLUENT_GRID_UNIT * 6) as f32),
        }
    }
}

impl Default for ZsuiTypographyTokens {
    fn default() -> Self {
        Self {
            caption: ZsuiTypographyStyle::new(12.0, 16.0, TextWeight::Regular),
            body: ZsuiTypographyStyle::new(14.0, 20.0, TextWeight::Regular),
            body_strong: ZsuiTypographyStyle::new(14.0, 20.0, TextWeight::Semibold),
            body_large: ZsuiTypographyStyle::new(18.0, 24.0, TextWeight::Regular),
            subtitle: ZsuiTypographyStyle::new(20.0, 28.0, TextWeight::Semibold),
            title: ZsuiTypographyStyle::new(28.0, 36.0, TextWeight::Semibold),
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
        Self {
            compact_height: Dp::new(ZSUI_FLUENT_COMPACT_CONTROL_HEIGHT as f32),
            standard_height: Dp::new(ZSUI_FLUENT_STANDARD_CONTROL_HEIGHT as f32),
            touch_target: Dp::new(ZSUI_FLUENT_TOUCH_TARGET as f32),
            navigation_row_height: Dp::new(ZSUI_FLUENT_NAVIGATION_ROW_HEIGHT as f32),
            small_icon: Dp::new(ZSUI_FLUENT_SMALL_ICON_SIZE as f32),
            standard_icon: Dp::new(ZSUI_FLUENT_STANDARD_ICON_SIZE as f32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_resolves_tokens_without_scattered_literals() {
        let theme = ZsuiTheme::light();

        assert_eq!(theme.spacing(SpacingToken::Md), Dp::new(12.0));
        assert_eq!(theme.radius(RadiusToken::Medium), Dp::new(8.0));
        assert_eq!(theme.color(ThemeColorToken::Accent).a, 255);
        assert_eq!(
            theme.typography(TypographyToken::BodyStrong).weight,
            TextWeight::Semibold
        );
        assert_eq!(
            theme.control_metric(ControlMetricToken::StandardHeight),
            Dp::new(32.0)
        );
    }
}
