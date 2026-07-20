use crate::{
    backend_profile::BackendProfile,
    platform_identity::{NativeUiBackendDescriptor, NativeUiBackendStatus, NativeUiPlatform},
    platform_style::ZsPlatformStyle,
};

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
    backend_status: NativeUiBackendStatus,
    adapter_boundary: &'static str,
}

impl PlatformExperience {
    pub(crate) const fn for_platform(platform: NativeUiPlatform) -> Self {
        match platform {
            NativeUiPlatform::Windows => Self {
                platform,
                design_language: PlatformDesignLanguage::Fluent,
                backend: BackendProfile::windows(),
                backend_status: NativeUiBackendStatus::NativeHostFirstPass,
                adapter_boundary: "WindowsWin32GdiNativeWindowBoundary",
            },
            NativeUiPlatform::Macos => Self {
                platform,
                design_language: PlatformDesignLanguage::AppKit,
                backend: BackendProfile::macos(),
                backend_status: NativeUiBackendStatus::NativeHostFirstPass,
                adapter_boundary: "MacosAppKitWindowService",
            },
            NativeUiPlatform::Linux => Self {
                platform,
                design_language: PlatformDesignLanguage::Gtk,
                backend: BackendProfile::linux(),
                backend_status: NativeUiBackendStatus::NativeHostFirstPass,
                adapter_boundary: "LinuxDirectWindowHost",
            },
            NativeUiPlatform::Android => Self {
                platform,
                design_language: PlatformDesignLanguage::Material,
                backend: BackendProfile::android(),
                backend_status: NativeUiBackendStatus::AdapterBoundaryScaffold,
                adapter_boundary: "AndroidActivityAdapterBoundary",
            },
            NativeUiPlatform::Harmony => Self {
                platform,
                design_language: PlatformDesignLanguage::Harmony,
                backend: BackendProfile::harmony(),
                backend_status: NativeUiBackendStatus::AdapterBoundaryScaffold,
                adapter_boundary: "HarmonyAbilityAdapterBoundary",
            },
        }
    }

    pub(crate) const fn current() -> Option<Self> {
        let platform = match NativeUiPlatform::current_target() {
            Some(platform) => platform,
            None => return None,
        };
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            #[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
            {
                return Some(Self {
                    backend: BackendProfile::linux_lite(),
                    ..Self::for_platform(platform)
                });
            }
        }
        Some(Self::for_platform(platform))
    }

    pub(crate) const fn platform(self) -> NativeUiPlatform {
        self.platform
    }

    pub(crate) const fn backend(self) -> BackendProfile {
        self.backend
    }

    pub(crate) const fn backend_descriptor(self) -> NativeUiBackendDescriptor {
        NativeUiBackendDescriptor {
            platform: self.platform,
            toolkit: self.backend.toolkit(),
            status: self.backend_status,
            adapter_boundary: self.adapter_boundary,
            module_path: self.backend.host().module_path(),
        }
    }

    pub(crate) const fn is_desktop(self) -> bool {
        matches!(
            self.platform,
            NativeUiPlatform::Windows | NativeUiPlatform::Macos | NativeUiPlatform::Linux
        )
    }

    /// Maps a registered platform experience to the shared component profile.
    ///
    /// Desktop backends may reuse one of these profiles without adding target
    /// branches to component modules. Mobile registrations deliberately return
    /// `None` until their own component profiles and runtime hosts exist.
    pub(crate) const fn shared_component_style(self) -> Option<ZsPlatformStyle> {
        match self.design_language {
            PlatformDesignLanguage::Fluent => Some(ZsPlatformStyle::Windows),
            PlatformDesignLanguage::AppKit => Some(ZsPlatformStyle::Macos),
            PlatformDesignLanguage::Gtk => Some(ZsPlatformStyle::Gtk),
            PlatformDesignLanguage::Material | PlatformDesignLanguage::Harmony => None,
        }
    }
}

pub const SUPPORTED_NATIVE_UI_BACKENDS: [NativeUiBackendDescriptor; 5] = [
    PlatformExperience::for_platform(NativeUiPlatform::Windows).backend_descriptor(),
    PlatformExperience::for_platform(NativeUiPlatform::Macos).backend_descriptor(),
    PlatformExperience::for_platform(NativeUiPlatform::Linux).backend_descriptor(),
    PlatformExperience::for_platform(NativeUiPlatform::Android).backend_descriptor(),
    PlatformExperience::for_platform(NativeUiPlatform::Harmony).backend_descriptor(),
];

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
    fn current_experience_consumes_the_canonical_target_selector() {
        let current = PlatformExperience::current();
        assert_eq!(
            current.map(PlatformExperience::platform),
            NativeUiPlatform::current_target()
        );
        assert_eq!(
            NativeUiPlatform::current_target(),
            native_ui_platform_for_current_target()
        );
        assert_eq!(
            NativeUiPlatform::current_target()
                .map(crate::PlatformName::from)
                .unwrap_or(crate::PlatformName::Unknown),
            crate::PlatformName::current()
        );
    }

    #[test]
    fn public_backend_inventory_is_derived_from_platform_experience() {
        for platform in crate::SUPPORTED_NATIVE_UI_PLATFORMS {
            let expected = PlatformExperience::for_platform(platform).backend_descriptor();
            let actual = native_ui_backend_for_platform(platform)
                .expect("supported platform should have a backend descriptor");
            assert_eq!(*actual, expected);
        }

        let manifest = include_str!("../native_adapter_manifest.rs");
        let manifest_core = manifest
            .split_once("#[cfg(test)]")
            .map_or(manifest, |(core, _)| core);
        assert!(manifest_core
            .contains("pub use crate::platform_experience::SUPPORTED_NATIVE_UI_BACKENDS"));
        assert!(!manifest_core.contains("adapter_boundary: \""));
        assert!(!manifest_core.contains("module_path: \"src/"));

        let experience = include_str!("experience.rs");
        let experience_core = experience
            .split_once("#[cfg(test)]")
            .map_or(experience, |(core, _)| core);
        assert!(experience_core.contains("pub const SUPPORTED_NATIVE_UI_BACKENDS"));
        for boundary in [
            "WindowsWin32GdiNativeWindowBoundary",
            "MacosAppKitWindowService",
            "LinuxDirectWindowHost",
            "AndroidActivityAdapterBoundary",
            "HarmonyAbilityAdapterBoundary",
        ] {
            assert_eq!(
                experience_core.matches(boundary).count(),
                1,
                "backend identity should have one registration for {boundary}"
            );
        }

        let launch = include_str!("../native_host_launch.rs");
        let launch_core = launch
            .split_once("#[cfg(test)]")
            .map_or(launch, |(core, _)| core);
        assert!(launch_core.contains("experience.backend_descriptor()"));
        assert!(!launch_core.contains("native_ui_backend_for_platform"));

        let capability = include_str!("../capability.rs");
        let capability_core = capability
            .split_once("#[cfg(test)]")
            .map_or(capability, |(core, _)| core);
        assert!(capability_core.contains("NativeUiPlatform::current_target()"));
        assert!(!capability_core.contains("cfg!(target_os"));
        assert!(!capability_core.contains("#[cfg(target_os"));
    }

    #[test]
    fn shared_component_style_is_registered_once_and_mobile_remains_explicit() {
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Windows).shared_component_style(),
            Some(ZsPlatformStyle::Windows)
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Macos).shared_component_style(),
            Some(ZsPlatformStyle::Macos)
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Linux).shared_component_style(),
            Some(ZsPlatformStyle::Gtk)
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Android).shared_component_style(),
            None
        );
        assert_eq!(
            PlatformExperience::for_platform(NativeUiPlatform::Harmony).shared_component_style(),
            None
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
            (
                "zsui_calculator",
                include_str!("../../examples/zsui_calculator.rs"),
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
    fn ordinary_desktop_examples_do_not_select_targets_for_smoke_or_rendering() {
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
            (
                "zsui_calculator",
                include_str!("../../examples/zsui_calculator.rs"),
            ),
            ("codex_zsui", include_str!("../../examples/codex_zsui.rs")),
            (
                "invoice_workbench",
                include_str!("../../examples/invoice_workbench.rs"),
            ),
            (
                "navigation_shell_layout",
                include_str!("../../examples/navigation_shell_layout.rs"),
            ),
            (
                "rust_first_view",
                include_str!("../../examples/rust_first_view.rs"),
            ),
            (
                "workbench_shell",
                include_str!("../../examples/workbench_shell.rs"),
            ),
        ];

        for (name, source) in applications {
            let production = source
                .split_once("#[cfg(test)]")
                .map_or(source, |(production, _)| production);
            for forbidden in [
                "cfg!(",
                "#[cfg(",
                "target_os",
                "NativeUiPlatform",
                "windows_sys",
                "objc2",
                "GtkWidget",
                "HWND",
                "NSView",
            ] {
                assert!(
                    !production.contains(forbidden),
                    "{name} must leave target selection to the framework: {forbidden}"
                );
            }
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
            include_str!("../view/widgets/calculator.rs"),
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
    fn shared_menu_and_live_view_contracts_do_not_encode_target_backends() {
        let menu = include_str!("../menu.rs");
        for forbidden in [
            "target_os",
            "windows-win32",
            "macos-appkit",
            "linux-gtk",
            "gtk_accelerator",
            "appkit_key_equivalent",
            "Page_Up",
            "\\u{f700}",
        ] {
            assert!(
                !menu.contains(forbidden),
                "shared menu model contains target encoding: {forbidden}"
            );
        }

        let live_view = include_str!("../view/focus.rs");
        for forbidden in ["#[cfg(all(windows", "windows-win32", "target_os"] {
            assert!(
                !live_view.contains(forbidden),
                "shared live View runtime contains a target gate: {forbidden}"
            );
        }
        assert!(live_view.contains("fn surface(&self) -> (Rect, Dpi);"));
        assert!(live_view.contains("pub(crate) fn surface(&self) -> (Rect, Dpi)"));

        let accelerator_adapter = include_str!("menu_accelerator.rs");
        assert!(accelerator_adapter.contains("pub(crate) fn gtk_accelerator("));
        assert!(accelerator_adapter.contains("pub(crate) fn appkit_key_equivalent("));
        assert!(include_str!("../macos_appkit_menu.rs")
            .contains("platform_menu_accelerator::appkit_key_equivalent"));
        assert!(include_str!("../linux_gtk_menu.rs")
            .contains("platform_menu_accelerator::gtk_accelerator"));
    }

    #[test]
    fn built_in_render_contracts_share_one_platform_style_type() {
        let component_sources = [
            include_str!("../password_box.rs"),
            include_str!("../progress.rs"),
            include_str!("../render_protocol.rs"),
            include_str!("../tooltip.rs"),
            include_str!("../widget_render.rs"),
        ]
        .join("\n");
        let alias_count = component_sources
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("pub type Zs")
                    && line.contains("PlatformStyle = crate::ZsPlatformStyle")
            })
            .count();
        let duplicated_enum_count = component_sources
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("pub enum Zs") && line.contains("PlatformStyle")
            })
            .count();
        let shared_style_source = include_str!("style.rs");

        assert_eq!(alias_count, 20, "update the shared-style alias inventory");
        assert_eq!(
            duplicated_enum_count, 0,
            "component render contracts must not declare separate platform enums"
        );
        assert_eq!(
            shared_style_source
                .matches("pub enum ZsPlatformStyle")
                .count(),
            1
        );
        assert_eq!(
            shared_style_source
                .matches("PlatformExperience::current()")
                .count(),
            1
        );
        assert_eq!(
            shared_style_source
                .matches("shared_component_style()")
                .count(),
            1
        );
        assert!(!component_sources.contains("PlatformExperience::"));
        assert!(!component_sources.contains("cfg!(target_os"));
        assert!(!component_sources.contains("cfg!(all(target_os"));
        assert!(!shared_style_source.contains("cfg!(target_os"));

        let experience_source = include_str!("experience.rs");
        let experience_core = experience_source
            .split_once("#[cfg(test)]")
            .map_or(experience_source, |(core, _)| core);
        assert_eq!(
            experience_core
                .matches("pub(crate) const fn shared_component_style")
                .count(),
            1
        );
        assert!(!experience_core.contains("select_desktop<"));
    }

    #[test]
    fn semantic_view_composition_resolves_through_one_internal_component_profile() {
        let profile = include_str!("component_profile/mod.rs");
        let profile_core = profile
            .split_once("#[cfg(test)]")
            .map_or(profile, |(core, _)| core);
        assert_eq!(
            profile_core
                .matches("pub(crate) const fn for_style")
                .count(),
            1
        );
        for contract in [
            "PlatformStyleTokenProfile",
            "PlatformTypographyProfile",
            "PlatformFocusVisualProfile",
            "PlatformBaseControlProfile",
            "PlatformButtonProfile",
            "PlatformNavigationItemProfile",
            "PlatformSectionComposition",
            "PlatformNavigationComposition",
            "PlatformCommandBarProfile",
            "PlatformTabProfile",
            "PlatformDialogProfile",
            "PlatformInfoBarProfile",
            "PlatformTeachingTipProfile",
            "PlatformToastProfile",
            "PlatformBreadcrumbProfile",
            "PlatformToggleButtonProfile",
            "PlatformNumberBoxProfile",
            "PlatformPasswordBoxProfile",
            "PlatformTooltipProfile",
            "PlatformProgressRingProfile",
            "PlatformAutoSuggestProfile",
            "PlatformGridViewProfile",
            "PlatformTreeViewProfile",
            "PlatformTableProfile",
            "PlatformTimePickerProfile",
            "PlatformColorPickerProfile",
            "PlatformCommandPaletteProfile",
            "PlatformShellProfile",
        ] {
            assert!(
                profile_core.contains(contract),
                "component profile is missing {contract}"
            );
        }
        assert!(profile_core.contains("Host/Text/Raster/Presenter/Services"));
        assert!(!profile_core.contains("BackendProfile"));
        assert!(!profile_core.contains("NativeUiPlatform"));

        let data = include_str!("../view/widgets/data.rs");
        let data_core = data
            .split_once("mod data_tests")
            .map_or(data, |(core, _)| core);
        let button = include_str!("../view/widgets/button.rs");
        let paint = include_str!("../view/paint.rs");
        let view_impl = paint
            .find("impl<Msg: Clone> View<Msg> for ViewNode")
            .expect("View paint implementation should exist");
        let navigation_start = paint[view_impl..]
            .find("ViewNodeKind::NavigationView {")
            .map(|offset| view_impl + offset)
            .expect("NavigationView paint arm should exist");
        let navigation_end = paint[navigation_start..]
            .find("ViewNodeKind::Text { text, style }")
            .map(|offset| navigation_start + offset)
            .expect("Text paint arm should follow NavigationView");
        let navigation_paint = &paint[navigation_start..navigation_end];

        for (name, source) in [
            ("data", data_core),
            ("button", button),
            ("navigation paint", navigation_paint),
        ] {
            assert!(
                source.contains("PlatformComponentProfile::"),
                "{name} should resolve component defaults through PlatformComponentProfile"
            );
            for forbidden in [
                "ZsBaseControlPlatformStyle::Windows",
                "ZsBaseControlPlatformStyle::Macos",
                "ZsBaseControlPlatformStyle::Gtk",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{name} contains a platform composition branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn component_defaults_are_owned_by_separate_platform_modules() {
        let resolver = include_str!("component_profile/mod.rs");
        for (module, style, forbidden_styles) in [
            (
                include_str!("component_profile/windows.rs"),
                "ZsPlatformStyle::Windows",
                ["ZsPlatformStyle::Macos", "ZsPlatformStyle::Gtk"],
            ),
            (
                include_str!("component_profile/macos.rs"),
                "ZsPlatformStyle::Macos",
                ["ZsPlatformStyle::Windows", "ZsPlatformStyle::Gtk"],
            ),
            (
                include_str!("component_profile/gtk.rs"),
                "ZsPlatformStyle::Gtk",
                ["ZsPlatformStyle::Windows", "ZsPlatformStyle::Macos"],
            ),
        ] {
            assert_eq!(module.matches("const fn profile()").count(), 1);
            assert!(module.contains(style));
            assert!(!module.contains("BackendProfile"));
            assert!(!module.contains("NativeUiPlatform"));
            assert!(!module.contains("target_os"));
            for forbidden in forbidden_styles {
                assert!(
                    !module.contains(forbidden),
                    "{style} component module contains another platform profile: {forbidden}"
                );
            }
        }
        for delegation in [
            "ZsPlatformStyle::Windows => windows::profile()",
            "ZsPlatformStyle::Macos => macos::profile()",
            "ZsPlatformStyle::Gtk => gtk::profile()",
        ] {
            assert!(resolver.contains(delegation));
        }
    }

    #[test]
    fn tab_layout_paint_and_keyboard_behavior_consume_one_internal_profile() {
        let render = include_str!("../widget_render.rs");
        let tab_start = render
            .find("pub type ZsTabPlatformStyle")
            .expect("tab render section should exist");
        let tab_end = render[tab_start..]
            .find("pub const ZS_AUTO_SUGGEST_MAX_VISIBLE_ITEMS")
            .map(|offset| tab_start + offset)
            .expect("auto-suggest section should follow tabs");
        let tab_render = &render[tab_start..tab_end];
        assert!(tab_render.contains("PlatformComponentProfile::for_style"));

        let native = include_str!("../native.rs");
        let cycle_start = native
            .find("fn native_tab_cycle_offset")
            .expect("tab cycle helper should exist");
        let cycle_end = native[cycle_start..]
            .find("#[cfg(feature = \"combo\")]")
            .map(|offset| cycle_start + offset)
            .expect("combo section should follow tab cycle helper");
        let keyboard_start = native
            .find("if matches!(target.kind, crate::ViewHitTargetKind::Tab")
            .expect("tab keyboard route should exist");
        let keyboard_end = native[keyboard_start..]
            .find("#[cfg(feature = \"date-picker\")]")
            .map(|offset| keyboard_start + offset)
            .expect("date picker route should follow tabs");
        for (name, source) in [
            ("tab render", tab_render),
            ("tab cycle", &native[cycle_start..cycle_end]),
            ("tab keyboard", &native[keyboard_start..keyboard_end]),
        ] {
            assert!(
                source.contains("PlatformComponentProfile::"),
                "{name} should consume PlatformComponentProfile"
            );
            for forbidden in [
                "ZsTabPlatformStyle::Windows",
                "ZsTabPlatformStyle::Macos",
                "ZsTabPlatformStyle::Gtk",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[cfg(feature = "dialog")]
    #[test]
    fn content_dialog_layout_paint_and_keyboard_behavior_consume_one_internal_profile() {
        let render = include_str!("../widget_render.rs");
        let dialog_start = render
            .find("pub type ZsContentDialogPlatformStyle")
            .expect("content dialog render section should exist");
        let dialog_end = render[dialog_start..]
            .find("fn place_popup(")
            .map(|offset| dialog_start + offset)
            .expect("popup placement helper should follow content dialog");
        let dialog_render = &render[dialog_start..dialog_end];
        assert!(dialog_render.contains("PlatformDialogProfile::for_platform"));

        let native = include_str!("../native.rs");
        let keyboard_start = native
            .find("if let Some(dialog_target) = interaction_plan")
            .expect("content dialog keyboard route should exist");
        let keyboard_end = native[keyboard_start..]
            .find("if key == NativeViewKey::Tab && !control")
            .map(|offset| keyboard_start + offset)
            .expect("ordinary focus traversal should follow content dialog");
        let dialog_keyboard = &native[keyboard_start..keyboard_end];
        assert!(dialog_keyboard.contains("PlatformComponentProfile::current()"));

        let model = include_str!("../content_dialog.rs");
        for (name, source) in [
            ("dialog render", dialog_render),
            ("dialog keyboard", dialog_keyboard),
            ("dialog public model", model),
        ] {
            for forbidden in [
                "ZsContentDialogPlatformStyle::Windows",
                "ZsContentDialogPlatformStyle::Macos",
                "ZsContentDialogPlatformStyle::Gtk",
                "NativeUiPlatform",
                "target_os",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn feedback_layout_and_paint_consume_internal_component_profiles() {
        let render = include_str!("../widget_render.rs");
        for (name, start_marker, end_marker, profile, style_alias) in [
            (
                "info bar",
                "pub type ZsInfoBarPlatformStyle",
                "pub type ZsTeachingTipPlatformStyle",
                "PlatformInfoBarProfile::for_platform",
                "ZsInfoBarPlatformStyle",
            ),
            (
                "teaching tip",
                "pub type ZsTeachingTipPlatformStyle",
                "pub type ZsToastPlatformStyle",
                "PlatformTeachingTipProfile::for_platform",
                "ZsTeachingTipPlatformStyle",
            ),
            (
                "toast",
                "pub type ZsToastPlatformStyle",
                "pub type ZsBreadcrumbPlatformStyle",
                "PlatformToastProfile::for_platform",
                "ZsToastPlatformStyle",
            ),
        ] {
            let start = render
                .find(start_marker)
                .unwrap_or_else(|| panic!("{name} render section should exist"));
            let end = render[start..]
                .find(end_marker)
                .map(|offset| start + offset)
                .unwrap_or_else(|| panic!("{end_marker} should follow {name}"));
            let section = &render[start..end];
            assert!(
                section.contains(profile),
                "{name} should resolve through {profile}"
            );
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("{style_alias}::{platform}");
                assert!(
                    !section.contains(&forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn foundational_controls_consume_internal_component_profiles() {
        let render = include_str!("../widget_render.rs");
        for (name, start_marker, end_marker, profile, style_alias) in [
            (
                "base controls",
                "pub type ZsBaseControlPlatformStyle",
                "pub type ZsInfoBarPlatformStyle",
                "PlatformBaseControlProfile::for_platform",
                "ZsBaseControlPlatformStyle",
            ),
            (
                "breadcrumb",
                "pub type ZsBreadcrumbPlatformStyle",
                "pub type ZsToggleButtonPlatformStyle",
                "PlatformBreadcrumbProfile::for_platform",
                "ZsBreadcrumbPlatformStyle",
            ),
            (
                "toggle button",
                "pub type ZsToggleButtonPlatformStyle",
                "pub type ZsNumberBoxPlatformStyle",
                "PlatformToggleButtonProfile::for_platform",
                "ZsToggleButtonPlatformStyle",
            ),
            (
                "number box",
                "pub type ZsNumberBoxPlatformStyle",
                "pub type ZsTabPlatformStyle",
                "PlatformNumberBoxProfile::for_platform",
                "ZsNumberBoxPlatformStyle",
            ),
        ] {
            let start = render
                .find(start_marker)
                .unwrap_or_else(|| panic!("{name} render section should exist"));
            let end = render[start..]
                .find(end_marker)
                .map(|offset| start + offset)
                .unwrap_or_else(|| panic!("{end_marker} should follow {name}"));
            let section = &render[start..end];
            assert!(
                section.contains(profile),
                "{name} should resolve through {profile}"
            );
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("{style_alias}::{platform}");
                assert!(
                    !section.contains(&forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn shared_tokens_typography_and_focus_visuals_consume_platform_profiles() {
        let style = include_str!("../style.rs");
        let style_core = style
            .split_once("#[cfg(test)]")
            .map_or(style, |(core, _)| core);
        assert!(style_core.contains("PlatformStyleTokenProfile::for_platform"));

        let render = include_str!("../render_protocol.rs");
        let render_core = render
            .split_once("#[cfg(test)]")
            .map_or(render, |(core, _)| core);
        assert!(render_core.contains("PlatformTypographyProfile::for_platform"));

        let input = include_str!("../native_input_visuals.rs");
        let input_core = input
            .split_once("mod tests {")
            .map_or(input, |(core, _)| core);
        assert!(input_core.contains("PlatformFocusVisualProfile::for_platform"));
        assert!(!input_core.contains("#[cfg(windows)]"));
        assert!(!input_core.contains("#[cfg(not(windows))]"));
        assert!(include_str!("text_shaper_boundary.rs").contains("#[cfg(windows)]"));

        let shared_platform_style = include_str!("style.rs");
        let shared_platform_style_core = shared_platform_style
            .split_once("#[cfg(test)]")
            .map_or(shared_platform_style, |(core, _)| core);
        assert!(shared_platform_style_core.contains("PlatformTimePickerProfile::for_platform"));
        assert!(shared_platform_style_core.contains("PlatformPasswordBoxProfile::for_platform"));

        for (name, source) in [
            ("style tokens", style_core),
            ("typography", render_core),
            ("input focus", input_core),
            ("shared platform behavior", shared_platform_style_core),
        ] {
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("PlatformStyle::{platform}");
                assert!(
                    !source.contains(&forbidden),
                    "{name} contains a desktop platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn password_tooltip_and_progress_ring_consume_internal_component_profiles() {
        for (name, source, profile, style_alias) in [
            (
                "password box",
                include_str!("../password_box.rs"),
                "PlatformPasswordBoxProfile::for_platform",
                "ZsPasswordBoxPlatformStyle",
            ),
            (
                "tooltip",
                include_str!("../tooltip.rs"),
                "PlatformTooltipProfile::for_platform",
                "ZsTooltipPlatformStyle",
            ),
            (
                "progress ring",
                include_str!("../progress.rs"),
                "PlatformProgressRingProfile::for_platform",
                "ZsProgressRingPlatformStyle",
            ),
        ] {
            let production = source
                .split_once("#[cfg(test)]")
                .map_or(source, |(production, _)| production);
            assert!(
                production.contains(profile),
                "{name} should resolve through {profile}"
            );
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("{style_alias}::{platform}");
                assert!(
                    !production.contains(&forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }

        let platform_style = include_str!("style.rs");
        let reveal_start = platform_style
            .find("pub const fn default_password_reveal_mode")
            .expect("password reveal default should exist");
        let reveal = &platform_style[reveal_start..];
        assert!(reveal.contains("PlatformPasswordBoxProfile::for_platform"));
    }

    #[test]
    fn search_and_collection_controls_consume_internal_component_profiles() {
        let render = include_str!("../widget_render.rs");
        for (name, start_marker, end_marker, profile, style_alias) in [
            (
                "auto suggest",
                "pub type ZsAutoSuggestPlatformStyle",
                "pub type ZsGridViewPlatformStyle",
                "PlatformAutoSuggestProfile::for_platform",
                "ZsAutoSuggestPlatformStyle",
            ),
            (
                "grid view",
                "pub type ZsGridViewPlatformStyle",
                "pub type ZsTreePlatformStyle",
                "PlatformGridViewProfile::for_platform",
                "ZsGridViewPlatformStyle",
            ),
            (
                "tree view",
                "pub type ZsTreePlatformStyle",
                "pub type ZsTablePlatformStyle",
                "PlatformTreeViewProfile::for_platform",
                "ZsTreePlatformStyle",
            ),
            (
                "data grid",
                "pub type ZsTablePlatformStyle",
                "pub type ZsTimePickerPlatformStyle",
                "PlatformTableProfile::for_platform",
                "ZsTablePlatformStyle",
            ),
        ] {
            let start = render
                .find(start_marker)
                .unwrap_or_else(|| panic!("{name} render section should exist"));
            let end = render[start..]
                .find(end_marker)
                .map(|offset| start + offset)
                .unwrap_or_else(|| panic!("{end_marker} should follow {name}"));
            let section = &render[start..end];
            assert!(
                section.contains(profile),
                "{name} should resolve through {profile}"
            );
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("{style_alias}::{platform}");
                assert!(
                    !section.contains(&forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn picker_and_palette_controls_consume_internal_component_profiles() {
        let render = include_str!("../widget_render.rs");
        for (name, start_marker, end_marker, profile, style_alias) in [
            (
                "time picker",
                "pub type ZsTimePickerPlatformStyle",
                "pub type ZsColorPickerPlatformStyle",
                "PlatformTimePickerProfile::for_platform",
                "ZsTimePickerPlatformStyle",
            ),
            (
                "color picker",
                "pub type ZsColorPickerPlatformStyle",
                "pub type ZsCommandPalettePlatformStyle",
                "PlatformColorPickerProfile::for_platform",
                "ZsColorPickerPlatformStyle",
            ),
            (
                "command palette",
                "pub type ZsCommandPalettePlatformStyle",
                "pub type ZsContentDialogPlatformStyle",
                "PlatformCommandPaletteProfile::for_platform",
                "ZsCommandPalettePlatformStyle",
            ),
        ] {
            let start = render
                .find(start_marker)
                .unwrap_or_else(|| panic!("{name} render section should exist"));
            let end = render[start..]
                .find(end_marker)
                .map(|offset| start + offset)
                .unwrap_or_else(|| panic!("{end_marker} should follow {name}"));
            let section = &render[start..end];
            assert!(
                section.contains(profile),
                "{name} should resolve through {profile}"
            );
            for platform in ["Windows", "Macos", "Gtk"] {
                let forbidden = format!("{style_alias}::{platform}");
                assert!(
                    !section.contains(&forbidden),
                    "{name} contains a platform branch outside the profile: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn shared_widget_renderer_has_no_desktop_platform_variant_branches() {
        let render = include_str!("../widget_render.rs");
        let production = render
            .split_once("#[cfg(test)]")
            .map_or(render, |(production, _)| production);
        for platform in ["Windows", "Macos", "Gtk"] {
            let forbidden = format!("PlatformStyle::{platform}");
            assert!(
                !production.contains(&forbidden),
                "shared widget renderer contains a desktop branch outside profiles: {forbidden}"
            );
        }
    }

    #[test]
    fn workbench_consumes_platform_tokens_without_fluent_constants() {
        let workbench = include_str!("../workbench.rs");
        let production = workbench
            .split_once("#[cfg(test)]")
            .map_or(workbench, |(production, _)| production);

        assert!(production.contains("PlatformComponentProfile::current().style_tokens"));
        assert!(!production.contains("ZSUI_FLUENT_"));
        assert!(!production.contains("cfg!(target_os"));
        assert!(!production.contains("#[cfg(target_os"));
    }

    #[test]
    fn document_shell_consumes_the_internal_platform_profile() {
        let shell = include_str!("../document_shell.rs");
        let production = shell
            .split_once("#[cfg(test)]")
            .map_or(shell, |(production, _)| production);

        assert!(production.contains("PlatformComponentProfile::current()"));
        assert!(production.contains("PlatformDocumentShellProfile"));
        for forbidden in [
            "TAB_STRIP_HEIGHT_DP",
            "COMMAND_BAR_HEIGHT_DP",
            "ZSUI_FLUENT_",
            "cfg!(target_os",
            "#[cfg(target_os",
        ] {
            assert!(
                !production.contains(forbidden),
                "document shell bypasses the platform profile: {forbidden}"
            );
        }
    }

    #[test]
    fn calculator_shell_consumes_the_internal_platform_profile() {
        let calculator = include_str!("../calculator.rs");
        let production = calculator
            .split_once("#[cfg(test)]")
            .map_or(calculator, |(production, _)| production);

        assert!(production.contains("PlatformComponentProfile::current().calculator_shell"));
        assert!(production.contains("PlatformCalculatorShellProfile"));
        for forbidden in [
            "HEADER_HEIGHT_DP",
            "DISPLAY_HEIGHT_DP",
            "ZSUI_FLUENT_",
            "cfg!(target_os",
            "#[cfg(target_os",
        ] {
            assert!(
                !production.contains(forbidden),
                "calculator shell bypasses the platform profile: {forbidden}"
            );
        }
    }

    #[test]
    fn legacy_shell_authoring_is_platform_neutral_and_profile_resolved() {
        let shell = include_str!("../shell_layout.rs");
        let shell_core = shell
            .find("mod tests {")
            .map_or(shell, |offset| &shell[..offset]);

        assert!(shell_core.contains("current_shell_profile()"));
        assert!(shell_core.contains("PlatformShellProfile"));
        for forbidden in [
            "PlatformExperience::",
            "NativeUiPlatform",
            "cfg!(target_os",
            "#[cfg(target_os",
        ] {
            assert!(
                !shell_core.contains(forbidden),
                "legacy shell contains a public platform branch: {forbidden}"
            );
        }
        for line in shell_core.lines().map(str::trim_start) {
            if line.starts_with("pub fn ") {
                assert!(
                    !line.contains("PlatformStyle"),
                    "legacy shell exposes platform selection in its public API: {line}"
                );
            }
        }
    }
}
