use crate::{
    Color, ColorRole, NativeDrawFill, NativeStyleResolver, SemanticTextStyle, TextStyle, ZsuiTheme,
    ZsuiThemeMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeDrawPalette {
    pub primary_text: Color,
    pub secondary_text: Color,
    pub disabled_text: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub control: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub high_contrast: bool,
}

impl NativeDrawPalette {
    pub(crate) fn for_mode(mode: ZsuiThemeMode, system_prefers_dark: bool) -> Self {
        match mode {
            ZsuiThemeMode::HighContrast => Self::high_contrast(system_prefers_dark),
            ZsuiThemeMode::Dark => Self::from_theme(&ZsuiTheme::dark()),
            ZsuiThemeMode::Light => Self::from_theme(&ZsuiTheme::light()),
            ZsuiThemeMode::System if system_prefers_dark => Self::from_theme(&ZsuiTheme::dark()),
            ZsuiThemeMode::System => Self::from_theme(&ZsuiTheme::light()),
        }
    }

    #[cfg(any(
        test,
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
    ))]
    pub(crate) fn for_system_appearance(
        mode: ZsuiThemeMode,
        system_prefers_dark: bool,
        system_high_contrast: bool,
        native_standard: Option<Self>,
        native_high_contrast: Option<Self>,
    ) -> Self {
        if system_high_contrast {
            native_high_contrast.unwrap_or_else(|| Self::high_contrast(system_prefers_dark))
        } else if matches!(mode, ZsuiThemeMode::System)
            || matches!(mode, ZsuiThemeMode::Dark) && system_prefers_dark
            || matches!(mode, ZsuiThemeMode::Light) && !system_prefers_dark
        {
            native_standard.unwrap_or_else(|| Self::for_mode(mode, system_prefers_dark))
        } else {
            Self::for_mode(mode, system_prefers_dark)
        }
    }

    pub(crate) fn high_contrast(dark: bool) -> Self {
        let theme = ZsuiTheme::high_contrast(dark);
        Self {
            primary_text: theme.colors.text_primary,
            secondary_text: theme.colors.text_primary,
            disabled_text: theme.colors.text_primary,
            accent: theme.colors.accent,
            accent_text: theme.colors.accent_text,
            surface: theme.colors.surface,
            surface_raised: theme.colors.surface_raised,
            control: theme.colors.control,
            border: theme.colors.border,
            success: theme.colors.success,
            warning: theme.colors.warning,
            danger: theme.colors.danger,
            high_contrast: true,
        }
    }

    pub(crate) fn from_theme(theme: &ZsuiTheme) -> Self {
        Self {
            primary_text: theme.colors.text_primary,
            secondary_text: theme.colors.text_secondary,
            disabled_text: blend_color(theme.colors.text_secondary, theme.colors.surface, 96),
            accent: theme.colors.accent,
            accent_text: theme.colors.accent_text,
            surface: theme.colors.surface,
            surface_raised: theme.colors.surface_raised,
            control: theme.colors.control,
            border: theme.colors.border,
            success: theme.colors.success,
            warning: theme.colors.warning,
            danger: theme.colors.danger,
            high_contrast: false,
        }
    }

    pub(crate) const fn resolve(self, role: ColorRole) -> Color {
        match role {
            ColorRole::PrimaryText => self.primary_text,
            ColorRole::SecondaryText => self.secondary_text,
            ColorRole::DisabledText => self.disabled_text,
            ColorRole::Accent => self.accent,
            ColorRole::AccentText => self.accent_text,
            ColorRole::Surface => self.surface,
            ColorRole::SurfaceRaised => self.surface_raised,
            ColorRole::Control => self.control,
            ColorRole::Border => self.border,
            ColorRole::Success => self.success,
            ColorRole::Warning => self.warning,
            ColorRole::Danger => self.danger,
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn resolve_fill(self, fill: NativeDrawFill) -> Color {
        match fill {
            NativeDrawFill::Color(color) => color,
            NativeDrawFill::Role(role) => self.resolve(role),
            NativeDrawFill::RoleWithAlpha { role, alpha } => {
                let alpha = if self.high_contrast {
                    high_contrast_alpha(alpha)
                } else {
                    alpha
                };
                blend_color(self.resolve(role), self.surface, alpha)
            }
        }
    }

    /// Resolves a draw source for backends that support source-over alpha.
    ///
    /// `resolve_fill` remains the opaque fallback for renderers that cannot
    /// preserve semantic alpha. AppKit and Cairo must receive the unflattened
    /// source color so overlays compose over the pixels already in the view.
    pub(crate) const fn resolve_source_fill(self, fill: NativeDrawFill) -> Color {
        match fill {
            NativeDrawFill::Color(color) => color,
            NativeDrawFill::Role(role) => self.resolve(role),
            NativeDrawFill::RoleWithAlpha { role, alpha } => {
                let alpha = if self.high_contrast {
                    high_contrast_alpha(alpha)
                } else {
                    alpha
                };
                color_with_multiplied_alpha(self.resolve(role), alpha)
            }
        }
    }
}

const fn high_contrast_alpha(alpha: u8) -> u8 {
    match alpha {
        0 => 0,
        1..=20 => 64,
        21..=63 => 112,
        alpha => alpha,
    }
}

const fn blend_color(foreground: Color, background: Color, alpha: u8) -> Color {
    const fn channel(foreground: u8, background: u8, alpha: u8) -> u8 {
        let alpha = alpha as u32;
        (((foreground as u32 * alpha) + (background as u32 * (255 - alpha)) + 127) / 255) as u8
    }

    Color {
        r: channel(foreground.r, background.r, alpha),
        g: channel(foreground.g, background.g, alpha),
        b: channel(foreground.b, background.b, alpha),
        a: 255,
    }
}

const fn color_with_multiplied_alpha(color: Color, alpha: u8) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: ((color.a as u32 * alpha as u32 + 127) / 255) as u8,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NativeDrawTextStyleResolver {
    typography: crate::NativeTypographyProfile,
    palette: NativeDrawPalette,
}

impl NativeDrawTextStyleResolver {
    #[cfg(test)]
    pub(crate) fn new(
        font_family: impl Into<String>,
        monospace_font_family: impl Into<String>,
        icon_font_family: impl Into<String>,
        typography_platform: crate::ZsTypographyPlatformStyle,
        palette: NativeDrawPalette,
    ) -> Self {
        Self {
            typography: crate::NativeTypographyProfile::new(
                typography_platform,
                "native_draw_text_style_resolver",
                font_family,
                monospace_font_family,
                icon_font_family,
                f32::from(crate::render_protocol::default_typography_scale_per_mille()) / 1_000.0,
                "backend_native_text",
            ),
            palette,
        }
    }

    #[cfg(any(
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    ))]
    pub(crate) fn from_profile(
        typography: crate::NativeTypographyProfile,
        palette: NativeDrawPalette,
    ) -> Self {
        Self {
            typography,
            palette,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_typography_scale(mut self, scale: f32) -> Self {
        self.typography = self.typography.with_typography_scale(scale);
        self
    }
}

impl NativeStyleResolver for NativeDrawTextStyleResolver {
    fn resolve_text_style(&self, style: SemanticTextStyle) -> TextStyle {
        crate::render_protocol::resolve_semantic_text_style(
            self.typography.metrics_for(style.role),
            self.typography.font_family_for(style.role),
            self.palette.resolve(style.color),
            style,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TextRole;
    use crate::{HorizontalAlign, TextWeight, TextWrap, VerticalAlign};

    #[test]
    fn palette_resolves_theme_roles_and_alpha_against_surface() {
        let palette = NativeDrawPalette::for_mode(ZsuiThemeMode::Dark, false);
        assert_eq!(palette.surface, ZsuiTheme::dark().colors.surface);
        assert_eq!(
            palette.resolve_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 0,
            }),
            palette.surface
        );
        assert_eq!(
            palette.resolve_source_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 64,
            }),
            Color {
                a: 64,
                ..palette.accent
            }
        );
    }

    #[test]
    fn high_contrast_mode_and_system_override_do_not_fall_back_to_normal_dark_theme() {
        let explicit = NativeDrawPalette::for_mode(ZsuiThemeMode::HighContrast, true);
        assert_eq!(explicit.surface, Color::rgb(0, 0, 0));
        assert_eq!(explicit.primary_text, Color::rgb(255, 255, 255));
        assert_eq!(explicit.disabled_text, explicit.primary_text);

        let native = NativeDrawPalette {
            accent: Color::rgb(1, 2, 3),
            ..explicit
        };
        let system = NativeDrawPalette::for_system_appearance(
            ZsuiThemeMode::Light,
            false,
            true,
            None,
            Some(native),
        );
        assert_eq!(system, native);
        assert_eq!(
            explicit.resolve_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: 14,
            }),
            Color::rgb(64, 64, 64)
        );
        assert_eq!(
            explicit.resolve_source_fill(NativeDrawFill::RoleWithAlpha {
                role: ColorRole::PrimaryText,
                alpha: 14,
            }),
            Color::rgba(255, 255, 255, 64)
        );
    }

    #[test]
    fn matching_native_appearance_uses_platform_semantic_palette() {
        let native = NativeDrawPalette {
            surface: Color::rgb(11, 12, 13),
            ..NativeDrawPalette::for_mode(ZsuiThemeMode::Light, false)
        };
        assert_eq!(
            NativeDrawPalette::for_system_appearance(
                ZsuiThemeMode::Light,
                false,
                false,
                Some(native),
                None,
            ),
            native
        );
        assert_eq!(
            NativeDrawPalette::for_system_appearance(
                ZsuiThemeMode::Dark,
                false,
                false,
                Some(native),
                None,
            ),
            NativeDrawPalette::for_mode(ZsuiThemeMode::Dark, false)
        );
    }

    #[test]
    fn text_style_resolver_preserves_semantic_layout_options() {
        let resolver = NativeDrawTextStyleResolver::new(
            "system",
            "monospace",
            "icons",
            crate::ZsTypographyPlatformStyle::Windows,
            NativeDrawPalette::for_mode(ZsuiThemeMode::Light, false),
        );
        let style = resolver.resolve_text_style(SemanticTextStyle {
            role: TextRole::Title,
            color: ColorRole::Accent,
            weight: TextWeight::Bold,
            horizontal_align: HorizontalAlign::End,
            vertical_align: VerticalAlign::Start,
            wrap: TextWrap::Word,
            ellipsis: false,
        });
        assert_eq!(style.font_family, "system");
        assert_eq!(style.size, 28.0);
        assert_eq!(style.weight, TextWeight::Bold);
        assert_eq!(style.semantic_role, Some(TextRole::Title));
        assert_eq!(style.horizontal_align, HorizontalAlign::End);
        assert_eq!(style.wrap, TextWrap::Word);
    }

    #[test]
    fn text_style_resolver_uses_native_platform_type_ramps() {
        let palette = NativeDrawPalette::for_mode(ZsuiThemeMode::Light, false);
        let macos = NativeDrawTextStyleResolver::new(
            ".AppleSystemUIFont",
            "Menlo",
            ".AppleSystemUIFont",
            crate::ZsTypographyPlatformStyle::Macos,
            palette,
        );
        let gtk = NativeDrawTextStyleResolver::new(
            "Adwaita Sans",
            "Adwaita Mono",
            "Adwaita Sans",
            crate::ZsTypographyPlatformStyle::Gtk,
            palette,
        );

        let macos_body = macos.resolve_text_style(SemanticTextStyle::body());
        assert_eq!(
            (macos_body.size, macos_body.line_height, macos_body.weight),
            (13.0, 16.0, TextWeight::Regular)
        );
        let macos_title = macos.resolve_text_style(SemanticTextStyle::for_role(TextRole::Title));
        assert_eq!(
            (
                macos_title.size,
                macos_title.line_height,
                macos_title.weight
            ),
            (22.0, 26.0, TextWeight::Regular)
        );
        let mut emphasized_title = SemanticTextStyle::for_role(TextRole::Title);
        emphasized_title.weight = TextWeight::Semibold;
        assert_eq!(
            macos.resolve_text_style(emphasized_title).weight,
            TextWeight::Semibold
        );
        let gtk_caption = gtk.resolve_text_style(SemanticTextStyle::for_role(TextRole::Caption));
        assert_eq!((gtk_caption.size, gtk_caption.line_height), (11.5, 16.0));
    }

    #[test]
    fn text_style_resolver_scales_size_and_line_box_together() {
        let style = NativeDrawTextStyleResolver::new(
            "system",
            "monospace",
            "icons",
            crate::ZsTypographyPlatformStyle::Gtk,
            NativeDrawPalette::for_mode(ZsuiThemeMode::Light, false),
        )
        .with_typography_scale(1.25)
        .resolve_text_style(SemanticTextStyle::body());

        assert_eq!((style.size, style.line_height), (17.5, 25.0));
    }
}
