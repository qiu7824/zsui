use super::{DesktopRuntimeBackend, DesktopRuntimeRequest};
use crate::{FileDialogSpec, SaveFileDialogSpec, ZsuiError, ZsuiResult};

#[derive(Default)]
pub(super) struct Backend;

impl DesktopRuntimeBackend for Backend {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str {
        "linux_gtk"
    }

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()> {
        let _shell_runtimes = request.shell_runtimes;
        if !request.trays.is_empty() {
            return Err(ZsuiError::unsupported(
                "native_window_status_item",
                "the GTK4 status-item runtime is not connected to the unified event loop",
            ));
        }
        crate::linux_gtk_services::run_linux_gtk_native_window_event_loop(
            &request.windows,
            &request.draw_plans,
            &request.view_runtimes,
            None,
            None,
            &[],
        )
        .map(|_| ())
    }

    fn open_file_dialog(
        &mut self,
        spec: &FileDialogSpec,
    ) -> Option<ZsuiResult<Option<Vec<std::path::PathBuf>>>> {
        Some(crate::linux_gtk_services::linux_gtk_open_file_dialog(spec))
    }

    fn save_file_dialog(
        &mut self,
        spec: &SaveFileDialogSpec,
    ) -> ZsuiResult<Option<std::path::PathBuf>> {
        crate::linux_gtk_services::linux_gtk_save_file_dialog(spec)
    }
}
