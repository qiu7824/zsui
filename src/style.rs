use serde::{Deserialize, Serialize};

use crate::{render_protocol::Color, Dp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeColorToken {
    Surface,
    SurfaceRaised,
    TextPrimary,
    TextSecondary,
    Accent,
    Control,
    Danger,
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
    pub danger: Color,
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
}

impl ZsuiTheme {
    pub fn light() -> Self {
        Self {
            colors: ZsuiColorTokens {
                surface: Color::rgb(248, 249, 250),
                surface_raised: Color::rgb(255, 255, 255),
                text_primary: Color::rgb(24, 27, 31),
                text_secondary: Color::rgb(91, 99, 110),
                accent: Color::rgb(28, 98, 220),
                control: Color::rgb(235, 238, 243),
                danger: Color::rgb(196, 39, 53),
            },
            radius: ZsuiRadiusTokens::default(),
            spacing: ZsuiSpacingTokens::default(),
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
                danger: Color::rgb(255, 98, 116),
            },
            radius: ZsuiRadiusTokens::default(),
            spacing: ZsuiSpacingTokens::default(),
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
            ThemeColorToken::Danger => self.colors.danger,
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
            small: Dp::new(4.0),
            medium: Dp::new(8.0),
            large: Dp::new(12.0),
            pill: Dp::new(999.0),
        }
    }
}

impl Default for ZsuiSpacingTokens {
    fn default() -> Self {
        Self {
            xs: Dp::new(4.0),
            sm: Dp::new(8.0),
            md: Dp::new(12.0),
            lg: Dp::new(16.0),
            xl: Dp::new(24.0),
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
    }
}
