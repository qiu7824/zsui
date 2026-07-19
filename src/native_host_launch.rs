use serde::Serialize;

use crate::{
    native_ui_backend_for_platform, platform_experience::PlatformExperience, NativeUiPlatform,
    NativeUiToolkit,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NativeHostLaunchMode {
    RealNativeHost,
    DesktopTransportFallback,
    ContractScaffoldFallback,
}

impl NativeHostLaunchMode {
    pub const fn mode_name(self) -> &'static str {
        match self {
            Self::RealNativeHost => "real_native_host",
            Self::DesktopTransportFallback => "desktop_transport_fallback",
            Self::ContractScaffoldFallback => "contract_scaffold_fallback",
        }
    }

    pub const fn enters_real_event_loop(self) -> bool {
        matches!(self, Self::RealNativeHost)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NativeHostLaunchPlan {
    pub platform: NativeUiPlatform,
    pub toolkit: NativeUiToolkit,
    pub entry_point: &'static str,
    pub native_application_type: &'static str,
    pub native_window_type: &'static str,
    pub real_host_module_path: &'static str,
    pub text_backend: &'static str,
    pub raster_backend: &'static str,
    pub presenter_backend: &'static str,
    pub services_backend: &'static str,
    pub fallback_module_path: &'static str,
    pub mode: NativeHostLaunchMode,
    pub target_os_verification_required: bool,
}

impl NativeHostLaunchPlan {
    pub const fn platform_name(&self) -> &'static str {
        self.platform.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.toolkit.toolkit_name()
    }

    pub const fn mode_name(&self) -> &'static str {
        self.mode.mode_name()
    }

    pub const fn enters_real_event_loop(&self) -> bool {
        self.mode.enters_real_event_loop()
    }

    pub const fn needs_target_os_verification(&self) -> bool {
        self.target_os_verification_required
    }
}

pub fn native_host_launch_plan_for_platform(
    platform: NativeUiPlatform,
) -> Option<NativeHostLaunchPlan> {
    let backend = native_ui_backend_for_platform(platform)?;
    let experience = PlatformExperience::for_platform(platform);
    let profile = experience.backend();
    debug_assert_eq!(profile.toolkit(), backend.toolkit);
    debug_assert_eq!(profile.host().module_path(), backend.module_path);

    Some(NativeHostLaunchPlan {
        platform,
        toolkit: profile.toolkit(),
        entry_point: if experience.is_desktop() {
            "zsui::native_window(\"Title\").run()"
        } else {
            "mobile runtime host scaffold"
        },
        native_application_type: profile.host().native_application_type(),
        native_window_type: profile.host().native_window_type(),
        real_host_module_path: profile.host().module_path(),
        text_backend: profile.text().name(),
        raster_backend: profile.raster().name(),
        presenter_backend: profile.presenter().name(),
        services_backend: profile.services().name(),
        fallback_module_path: "src/host.rs",
        mode: if profile.has_real_runtime() {
            NativeHostLaunchMode::RealNativeHost
        } else {
            NativeHostLaunchMode::ContractScaffoldFallback
        },
        target_os_verification_required: true,
    })
}

pub fn native_host_launch_plan_for_current_target() -> Option<NativeHostLaunchPlan> {
    native_host_launch_plan_for_platform(PlatformExperience::current()?.platform())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_plan_reports_platform_toolkit_and_real_loop_mode() {
        let plan = native_host_launch_plan_for_platform(NativeUiPlatform::Windows)
            .expect("windows launch plan should exist");

        assert_eq!(plan.platform_name(), "windows");
        assert_eq!(plan.toolkit_name(), "win32_gdi");
        assert_eq!(plan.mode_name(), "real_native_host");
        assert_eq!(plan.real_host_module_path, "src/platform/windows/mod.rs");
        assert_eq!(plan.text_backend, "uniscribe");
        assert_eq!(plan.raster_backend, "gdi_plus");
        assert_eq!(plan.presenter_backend, "buffered_dib");
        assert_eq!(plan.services_backend, "win32");
        assert!(plan.enters_real_event_loop());
        assert!(plan.needs_target_os_verification());
    }

    #[test]
    fn mobile_launch_plans_are_explicit_scaffold_fallbacks() {
        let android = native_host_launch_plan_for_platform(NativeUiPlatform::Android)
            .expect("android launch plan should exist");
        let harmony = native_host_launch_plan_for_platform(NativeUiPlatform::Harmony)
            .expect("harmony launch plan should exist");

        assert_eq!(android.toolkit, NativeUiToolkit::AndroidActivity);
        assert_eq!(android.mode, NativeHostLaunchMode::ContractScaffoldFallback);
        assert!(!android.enters_real_event_loop());
        assert_eq!(harmony.toolkit, NativeUiToolkit::HarmonyAbility);
        assert_eq!(harmony.mode_name(), "contract_scaffold_fallback");
    }

    #[test]
    fn appkit_and_linux_direct_launch_plans_enter_real_native_event_loops() {
        let macos = native_host_launch_plan_for_platform(NativeUiPlatform::Macos)
            .expect("macOS launch plan should exist");
        let linux = native_host_launch_plan_for_platform(NativeUiPlatform::Linux)
            .expect("Linux launch plan should exist");

        assert_eq!(macos.toolkit, NativeUiToolkit::AppKit);
        assert_eq!(linux.toolkit, NativeUiToolkit::LinuxDirect);
        assert_eq!(macos.mode, NativeHostLaunchMode::RealNativeHost);
        assert_eq!(linux.mode_name(), "real_native_host");
        assert!(macos.enters_real_event_loop());
        assert!(linux.enters_real_event_loop());
    }
}
