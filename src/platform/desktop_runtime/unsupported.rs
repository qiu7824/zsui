use super::{DesktopRuntimeBackend, DesktopRuntimeRequest};
use crate::{ZsuiError, ZsuiResult};

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
}
