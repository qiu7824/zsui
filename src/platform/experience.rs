use crate::{backend_profile::BackendProfile, NativeUiPlatform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformDesignLanguage {
    Fluent,
    AppKit,
    Gtk,
    Material,
    Harmony,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformExperience {
    platform: NativeUiPlatform,
    design_language: PlatformDesignLanguage,
    backend: BackendProfile,
}

impl PlatformExperience {
    pub(crate) const fn for_platform(platform: NativeUiPlatform) -> Self {
        match platform {
            NativeUiPlatform::Windows => Self {
                platform,
                design_language: PlatformDesignLanguage::Fluent,
                backend: BackendProfile::windows(),
            },
            NativeUiPlatform::Macos => Self {
                platform,
                design_language: PlatformDesignLanguage::AppKit,
                backend: BackendProfile::macos(),
            },
            NativeUiPlatform::Linux => Self {
                platform,
                design_language: PlatformDesignLanguage::Gtk,
                backend: BackendProfile::linux(),
            },
            NativeUiPlatform::Android => Self {
                platform,
                design_language: PlatformDesignLanguage::Material,
                backend: BackendProfile::android(),
            },
            NativeUiPlatform::Harmony => Self {
                platform,
                design_language: PlatformDesignLanguage::Harmony,
                backend: BackendProfile::harmony(),
            },
        }
    }

    pub(crate) const fn current() -> Option<Self> {
        #[cfg(target_env = "ohos")]
        {
            return Some(Self::for_platform(NativeUiPlatform::Harmony));
        }
        #[cfg(target_os = "windows")]
        {
            return Some(Self::for_platform(NativeUiPlatform::Windows));
        }
        #[cfg(target_os = "macos")]
        {
            return Some(Self::for_platform(NativeUiPlatform::Macos));
        }
        #[cfg(target_os = "android")]
        {
            return Some(Self::for_platform(NativeUiPlatform::Android));
        }
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            #[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
            {
                return Some(Self {
                    backend: BackendProfile::linux_lite(),
                    ..Self::for_platform(NativeUiPlatform::Linux)
                });
            }
            #[cfg(not(all(feature = "linux-direct-lite", not(feature = "linux-direct"))))]
            {
                return Some(Self::for_platform(NativeUiPlatform::Linux));
            }
        }
        #[allow(unreachable_code)]
        None
    }

    pub(crate) const fn current_or_desktop_fallback() -> Self {
        match Self::current() {
            Some(experience) => experience,
            None => Self::for_platform(NativeUiPlatform::Windows),
        }
    }

    pub(crate) const fn platform(self) -> NativeUiPlatform {
        self.platform
    }

    pub(crate) const fn backend(self) -> BackendProfile {
        self.backend
    }

    pub(crate) const fn is_desktop(self) -> bool {
        matches!(
            self.platform,
            NativeUiPlatform::Windows | NativeUiPlatform::Macos | NativeUiPlatform::Linux
        )
    }

    pub(crate) const fn select_desktop<T: Copy>(
        self,
        windows: T,
        macos: T,
        linux: T,
        fallback: T,
    ) -> T {
        match self.design_language {
            PlatformDesignLanguage::Fluent => windows,
            PlatformDesignLanguage::AppKit => macos,
            PlatformDesignLanguage::Gtk => linux,
            PlatformDesignLanguage::Material | PlatformDesignLanguage::Harmony => fallback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend_profile::{
            BackendProfile, NativePresenterProfile, NativeRasterProfile, NativeServicesProfile,
            NativeTextProfile,
        },
        native_ui_backend_for_platform, native_ui_platform_for_current_target,
    };

    #[test]
    fn every_platform_has_one_experience_and_backend_profile() {
        for platform in crate::SUPPORTED_NATIVE_UI_PLATFORMS {
            let experience = PlatformExperience::for_platform(platform);
            let backend = native_ui_backend_for_platform(platform)
                .expect("supported platform should have backend metadata");

            assert_eq!(experience.platform(), platform);
            assert_eq!(experience.backend().toolkit(), backend.toolkit);
            assert_eq!(
                experience.backend().host().module_path(),
                backend.module_path
            );
        }
    }

    #[test]
    fn current_experience_is_the_only_target_selector() {
        let current = PlatformExperience::current();
        assert_eq!(
            current.map(PlatformExperience::platform),
            native_ui_platform_for_current_target()
        );
        assert_eq!(
            current
                .map(PlatformExperience::platform)
                .map(crate::PlatformName::from)
                .unwrap_or(crate::PlatformName::Unknown),
            crate::PlatformName::current()
        );
    }

    #[test]
    fn desktop_style_selection_is_semantic_and_mobile_fallback_is_explicit() {
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Windows)
                .select_desktop("fluent", "appkit", "gtk", "fallback"),
            "fluent"
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Macos)
                .select_desktop("fluent", "appkit", "gtk", "fallback"),
            "appkit"
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Linux)
                .select_desktop("fluent", "appkit", "gtk", "fallback"),
            "gtk"
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Android)
                .select_desktop("fluent", "appkit", "gtk", "fallback"),
            "fallback"
        );
    }

    #[test]
    fn backend_profiles_keep_host_text_raster_presenter_and_services_separate() {
        let windows = PlatformExperience::for_platform(NativeUiPlatform::Windows).backend();
        assert_eq!(windows.text(), NativeTextProfile::Uniscribe);
        assert_eq!(windows.raster(), NativeRasterProfile::GdiPlus);
        assert_eq!(windows.presenter(), NativePresenterProfile::BufferedDib);
        assert_eq!(windows.services(), NativeServicesProfile::Win32);
        assert!(windows.has_real_runtime());

        let android = PlatformExperience::for_platform(NativeUiPlatform::Android).backend();
        assert!(!android.has_real_runtime());

        let lite = BackendProfile::linux_lite();
        assert_eq!(lite.text(), NativeTextProfile::CosmicText);
        assert_eq!(lite.raster(), NativeRasterProfile::TinySkia);
        assert_eq!(lite.presenter(), NativePresenterProfile::Softbuffer);
        assert_eq!(lite.services(), NativeServicesProfile::XdgDesktop);
    }

    #[test]
    fn acceptance_application_view_sources_do_not_branch_on_platforms() {
        let applications = [
            (
                "desktop_native_showcase",
                include_str!("../../examples/desktop_native_showcase.rs"),
            ),
            (
                "component_gallery",
                include_str!("../../examples/component_gallery.rs"),
            ),
            (
                "zsui_notepad",
                include_str!("../../examples/zsui_notepad.rs"),
            ),
        ];
        let forbidden = [
            "cfg!(",
            "#[cfg(",
            "target_os",
            "PlatformStyle",
            "NativeUiPlatform",
            "windows_sys",
            "objc2",
            "GtkWidget",
            "HWND",
            "NSView",
        ];

        for (name, source) in applications {
            let view_start = source
                .find("fn view(")
                .unwrap_or_else(|| panic!("{name} should define a shared view function"));
            let main_start = source
                .find("fn main()")
                .unwrap_or_else(|| panic!("{name} should define one shared main function"));
            let authoring_source = &source[view_start..main_start];

            for token in forbidden {
                assert!(
                    !authoring_source.contains(token),
                    "{name} application authoring must not contain platform token {token}"
                );
            }
            assert!(source[main_start..].contains("native_window("));
            assert!(source[main_start..].contains(".stateful_view"));
        }
    }

    #[test]
    fn public_view_builders_do_not_accept_platform_style_parameters() {
        let view_sources = [
            include_str!("../view/widgets/button.rs"),
            include_str!("../view/widgets/input.rs"),
            include_str!("../view/widgets/selection.rs"),
            include_str!("../view/widgets/navigation.rs"),
            include_str!("../view/widgets/data.rs"),
        ];

        for source in view_sources {
            for line in source.lines() {
                let line = line.trim_start();
                if line.starts_with("pub fn ") {
                    assert!(
                        !line.contains("PlatformStyle"),
                        "public View builder exposes a platform style: {line}"
                    );
                    assert!(
                        !line.contains("_for_style"),
                        "style-specific proof hook must remain crate-private: {line}"
                    );
                }
            }
        }
    }

    #[test]
    fn public_view_ast_payloads_do_not_store_platform_selection() {
        let source = include_str!("../view/node.rs");
        let start = source
            .find("pub enum ZsButtonPresentation")
            .expect("button presentation should remain part of the View AST");
        let end = source[start..]
            .find("pub struct ViewStyle")
            .map(|offset| start + offset)
            .expect("ViewStyle should follow the public View payload enums");
        let public_payloads = &source[start..end];

        assert!(!public_payloads.contains("PlatformStyle"));
        assert!(!public_payloads.contains("NativeUiPlatform"));
        assert!(!public_payloads.contains("platform:"));
    }

    #[test]
    fn built_in_style_defaults_use_the_shared_experience_selector() {
        let style_sources = [
            include_str!("../password_box.rs"),
            include_str!("../progress.rs"),
            include_str!("../render_protocol.rs"),
            include_str!("../tooltip.rs"),
            include_str!("../widget_render.rs"),
        ]
        .join("\n");
        let style_count = style_sources
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("pub enum Zs") && line.contains("PlatformStyle")
            })
            .count();
        let selector_count = style_sources
            .matches("PlatformExperience::current_or_desktop_fallback")
            .count();

        assert_eq!(style_count, 20, "update the centralized-style inventory");
        assert!(selector_count >= style_count);
        assert!(!style_sources.contains("cfg!(target_os"));
        assert!(!style_sources.contains("cfg!(all(target_os"));
    }
}
