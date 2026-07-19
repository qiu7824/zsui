use serde::{Deserialize, Serialize};

use crate::{PlatformName, ZsIcon, ZsuiError, ZsuiResult};

pub const WINDOWS_FLUENT_ICON_FONT_FAMILY: &str = "Segoe Fluent Icons";
pub const WINDOWS_MDL2_ICON_FONT_FAMILY: &str = "Segoe MDL2 Assets";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeIconSourceKind {
    WindowsSystemFont,
    MacOsSystemSymbol,
    LinuxIconTheme,
    BundledFluentSvg,
}

impl NativeIconSourceKind {
    pub const fn source_name(self) -> &'static str {
        match self {
            Self::WindowsSystemFont => "windows_system_font",
            Self::MacOsSystemSymbol => "macos_system_symbol",
            Self::LinuxIconTheme => "linux_icon_theme",
            Self::BundledFluentSvg => "bundled_fluent_svg",
        }
    }

    pub const fn is_platform_native(self) -> bool {
        !matches!(self, Self::BundledFluentSvg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeIconSource {
    pub icon: ZsIcon,
    pub kind: NativeIconSourceKind,
    pub identifier: String,
    pub glyph: Option<String>,
}

impl NativeIconSource {
    pub fn windows_font(icon: ZsIcon, family: &'static str, glyph: &'static str) -> Self {
        Self {
            icon,
            kind: NativeIconSourceKind::WindowsSystemFont,
            identifier: family.to_string(),
            glyph: Some(glyph.to_string()),
        }
    }

    pub fn macos_symbol(icon: ZsIcon) -> Self {
        Self {
            icon,
            kind: NativeIconSourceKind::MacOsSystemSymbol,
            identifier: icon.sf_symbol_name().to_string(),
            glyph: None,
        }
    }

    pub fn linux_theme(icon: ZsIcon) -> Self {
        Self {
            icon,
            kind: NativeIconSourceKind::LinuxIconTheme,
            identifier: icon.gtk_symbolic_name().to_string(),
            glyph: None,
        }
    }

    #[cfg(any(
        feature = "fluent-icons",
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            any(feature = "linux-direct-host", feature = "linux-gtk")
        )
    ))]
    pub fn bundled_fluent_svg(icon: ZsIcon) -> Self {
        Self {
            icon,
            kind: NativeIconSourceKind::BundledFluentSvg,
            identifier: icon.fluent_svg_asset_name().to_string(),
            glyph: None,
        }
    }
}

pub trait NativeIconLookup {
    fn is_icon_source_available(&self, source: &NativeIconSource) -> bool;
}

impl<F> NativeIconLookup for F
where
    F: Fn(&NativeIconSource) -> bool,
{
    fn is_icon_source_available(&self, source: &NativeIconSource) -> bool {
        self(source)
    }
}

pub fn native_icon_candidates(platform: &PlatformName, icon: ZsIcon) -> Vec<NativeIconSource> {
    let candidates = match platform {
        PlatformName::Windows => vec![
            NativeIconSource::windows_font(
                icon,
                WINDOWS_FLUENT_ICON_FONT_FAMILY,
                icon.windows_fluent_glyph(),
            ),
            NativeIconSource::windows_font(
                icon,
                WINDOWS_MDL2_ICON_FONT_FAMILY,
                icon.windows_mdl2_glyph(),
            ),
        ],
        PlatformName::Macos => vec![NativeIconSource::macos_symbol(icon)],
        PlatformName::Linux => vec![NativeIconSource::linux_theme(icon)],
        PlatformName::Android
        | PlatformName::Harmony
        | PlatformName::Unknown
        | PlatformName::Other(_) => Vec::new(),
    };
    #[cfg(any(
        feature = "fluent-icons",
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            any(feature = "linux-direct-host", feature = "linux-gtk")
        )
    ))]
    {
        candidates
            .into_iter()
            .chain(std::iter::once(NativeIconSource::bundled_fluent_svg(icon)))
            .collect()
    }
    #[cfg(not(any(
        feature = "fluent-icons",
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            any(feature = "linux-direct-host", feature = "linux-gtk")
        )
    )))]
    {
        candidates
    }
}

pub fn resolve_native_icon(
    platform: &PlatformName,
    icon: ZsIcon,
    lookup: &impl NativeIconLookup,
) -> ZsuiResult<NativeIconSource> {
    native_icon_candidates(platform, icon)
        .into_iter()
        .find(|source| lookup.is_icon_source_available(source))
        .ok_or_else(|| {
            ZsuiError::unsupported(
                "native_icon",
                format!(
                    "no available icon source for {icon:?} on {}",
                    platform.as_str()
                ),
            )
        })
}

#[cfg(any(
    feature = "fluent-icons",
    all(target_os = "macos", feature = "macos-appkit"),
    all(
        target_os = "linux",
        any(feature = "linux-direct-host", feature = "linux-gtk")
    )
))]
pub fn bundled_fluent_icon_svg(icon: ZsIcon) -> &'static [u8] {
    icon.fluent_svg_bytes()
}

#[cfg(any(
    feature = "fluent-icons",
    all(target_os = "macos", feature = "macos-appkit"),
    all(
        target_os = "linux",
        any(feature = "linux-direct-host", feature = "linux-gtk")
    )
))]
pub const FLUENT_SYSTEM_ICONS_LICENSE: &str =
    include_str!("../third_party/fluentui-system-icons/LICENSE");

#[cfg(any(
    feature = "fluent-icons",
    all(target_os = "macos", feature = "macos-appkit"),
    all(
        target_os = "linux",
        any(feature = "linux-direct-host", feature = "linux-gtk")
    )
))]
pub const FLUENT_SYSTEM_ICONS_NOTICE: &str =
    include_str!("../third_party/fluentui-system-icons/NOTICE");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_candidates_prefer_each_system_icon_source() {
        let windows = native_icon_candidates(&PlatformName::Windows, ZsIcon::Save);
        assert_eq!(windows[0].identifier, WINDOWS_FLUENT_ICON_FONT_FAMILY);
        assert_eq!(windows[1].identifier, WINDOWS_MDL2_ICON_FONT_FAMILY);
        assert!(windows[0].kind.is_platform_native());

        let macos = native_icon_candidates(&PlatformName::Macos, ZsIcon::Save);
        assert_eq!(macos[0].identifier, "square.and.arrow.down");
        let linux = native_icon_candidates(&PlatformName::Linux, ZsIcon::Save);
        assert_eq!(linux[0].identifier, "document-save-symbolic");
    }

    #[test]
    fn resolver_uses_the_first_available_platform_candidate() {
        let resolved = resolve_native_icon(
            &PlatformName::Windows,
            ZsIcon::Copy,
            &|source: &NativeIconSource| source.identifier == WINDOWS_MDL2_ICON_FONT_FAMILY,
        )
        .unwrap();

        assert_eq!(resolved.identifier, WINDOWS_MDL2_ICON_FONT_FAMILY);
        assert_eq!(
            resolved.glyph.as_deref(),
            Some(ZsIcon::Copy.windows_mdl2_glyph())
        );
    }

    #[test]
    #[cfg(any(
        feature = "fluent-icons",
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            any(feature = "linux-direct-host", feature = "linux-gtk")
        )
    ))]
    fn bundled_mit_svg_is_the_last_candidate_and_has_notices() {
        let candidates = native_icon_candidates(&PlatformName::Macos, ZsIcon::Settings);
        let fallback = candidates.last().unwrap();
        assert_eq!(fallback.kind, NativeIconSourceKind::BundledFluentSvg);
        assert!(bundled_fluent_icon_svg(ZsIcon::Settings).starts_with(b"<svg"));
        assert!(FLUENT_SYSTEM_ICONS_LICENSE.contains("MIT License"));
        assert!(FLUENT_SYSTEM_ICONS_NOTICE.contains("Third Party OSS"));
    }
}
