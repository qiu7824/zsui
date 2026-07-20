use super::{DesktopRuntimeBackend, DesktopRuntimeRequest, DesktopSmokeRequest};
use crate::{
    DesktopCapabilities, HostCapabilities, NativeWindowSmokeRunReport, PlatformName, ZsuiError,
    ZsuiResult,
};

#[derive(Default)]
pub(super) struct Backend;

impl DesktopRuntimeBackend for Backend {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str {
        "unsupported"
    }

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()> {
        let _backend_owned_state = (
            request.windows,
            request.trays,
            request.draw_plans,
            request.view_runtimes,
            request.shell_runtimes,
        );
        let detail = if cfg!(windows) {
            "enable the windows-win32 feature to compile the direct Win32 native window host"
        } else {
            "desktop native windows are implemented for Windows, macOS and Linux; Android needs a mobile runtime host"
        };
        Err(ZsuiError::unsupported("native_window", detail))
    }

    fn run_smoke_event_loop(
        self,
        request: DesktopSmokeRequest,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        let _backend_owned_state = (
            request.windows,
            request.draw_plans,
            request.view_runtime,
            request.shell_runtime,
            request.options,
        );
        let detail = if cfg!(windows) {
            "enable the windows-win32 feature to compile the direct Win32 native smoke host"
        } else {
            "desktop native smoke windows are implemented for Windows, macOS and Linux; Android needs a mobile runtime host"
        };
        Err(ZsuiError::unsupported("native_window_smoke", detail))
    }

    fn scaffold_capabilities(&self) -> HostCapabilities {
        match PlatformName::current() {
            PlatformName::Windows => HostCapabilities::windows_scaffold(),
            PlatformName::Macos => HostCapabilities::macos_scaffold(),
            PlatformName::Linux => HostCapabilities::linux_scaffold(),
            PlatformName::Android => HostCapabilities::android_scaffold(),
            platform => HostCapabilities::all_unsupported(platform),
        }
    }

    fn native_host_capabilities(&self) -> HostCapabilities {
        match PlatformName::current() {
            PlatformName::Android => HostCapabilities::android_native_window_host(),
            platform => HostCapabilities::all_unsupported(platform),
        }
    }

    fn desktop_capabilities(&self) -> DesktopCapabilities {
        DesktopCapabilities::all_unsupported(PlatformName::current())
    }

    fn native_proof_backend_name(&self) -> &'static str {
        "unavailable"
    }

    fn native_proof_typography(&self, typography_scale: f32) -> crate::NativeTypographyProfile {
        #[cfg(all(windows, feature = "windows-gdi"))]
        {
            return crate::windows_gdi_renderer::windows_native_typography_profile()
                .with_typography_scale(typography_scale);
        }
        #[allow(unreachable_code)]
        crate::NativeTypographyProfile::fallback(
            crate::ZsTypographyPlatformStyle::current(),
            typography_scale,
        )
    }

    fn capture_process_memory(
        &self,
        sample_point: &'static str,
    ) -> Option<crate::NativeProofProcessMemoryEvidence> {
        #[cfg(all(windows, feature = "windows-gdi"))]
        {
            return super::process_memory::capture_windows(sample_point);
        }
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            return super::process_memory::capture_linux(sample_point);
        }
        #[allow(unreachable_code)]
        {
            let _ = sample_point;
            None
        }
    }
}
