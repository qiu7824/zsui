use super::{DesktopRuntimeBackend, DesktopRuntimeRequest};
use crate::{FileDialogSpec, SaveFileDialogSpec, ZsuiResult};

#[derive(Default)]
pub(super) struct Backend;

impl DesktopRuntimeBackend for Backend {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str {
        "windows_win32"
    }

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()> {
        let input_routes = request
            .view_runtimes
            .iter()
            .map(crate::native::NativeViewInputRuntime::windows_win32_route)
            .collect::<Vec<_>>();
        let shell_routes = request
            .shell_runtimes
            .into_iter()
            .map(|runtime| runtime.map(crate::windows_win32_host::WindowsWin32ShellInputRoute::new))
            .collect::<Vec<_>>();
        crate::windows_win32_host::run_windows_win32_native_window_event_loop_with_routes_and_status_items(
            &request.windows,
            &request.draw_plans,
            &input_routes,
            &shell_routes,
            &request.trays,
        )
    }

    fn open_file_dialog(
        &mut self,
        spec: &FileDialogSpec,
    ) -> Option<ZsuiResult<Option<Vec<std::path::PathBuf>>>> {
        Some(crate::windows_win32_host::windows_win32_open_file_dialog(
            spec,
        ))
    }

    fn save_file_dialog(
        &mut self,
        spec: &SaveFileDialogSpec,
    ) -> ZsuiResult<Option<std::path::PathBuf>> {
        crate::windows_win32_host::windows_win32_save_file_dialog(spec)
    }
}
