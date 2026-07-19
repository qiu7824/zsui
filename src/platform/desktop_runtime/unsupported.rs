use super::{DesktopRuntimeBackend, DesktopRuntimeRequest, DesktopSmokeRequest};
use crate::{DesktopCapabilities, NativeWindowSmokeRunReport, PlatformName, ZsuiError, ZsuiResult};

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
            "desktop native windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts"
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
            "desktop native smoke windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts"
        };
        Err(ZsuiError::unsupported("native_window_smoke", detail))
    }

    fn capabilities(&self) -> DesktopCapabilities {
        DesktopCapabilities::all_unsupported(PlatformName::current())
    }
}
