use super::{
    complete_native_smoke, DesktopNativeSmokeMetadata, DesktopNativeSmokeOutcome,
    DesktopRuntimeBackend, DesktopRuntimeRequest, DesktopSmokeRequest,
};
use crate::{
    DesktopCapabilities, FileDialogSpec, HostCapabilities, NativeWindowSmokeRunReport,
    SaveFileDialogSpec, ZsuiError, ZsuiResult,
};

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

    fn run_smoke_event_loop(
        self,
        request: DesktopSmokeRequest,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        if request.windows.is_empty() {
            return Ok(NativeWindowSmokeRunReport::empty(request.options));
        }
        let run = crate::linux_gtk_services::run_linux_gtk_native_window_event_loop(
            &request.windows,
            &request.draw_plans,
            std::slice::from_ref(&request.view_runtime),
            Some(request.options.auto_close_after_ms),
            request
                .options
                .screenshot_file
                .as_deref()
                .map(std::path::Path::new),
            &request.options.native_view_inputs,
        )?;
        complete_native_smoke(
            request,
            DesktopNativeSmokeOutcome {
                created_window_count: run.created_window_count,
                proof_input_reports: run.proof_input_reports,
                native_view_capture: run.native_view_capture,
                menu_command_routed: run.menu_command_routed,
                menu_surface_created: false,
                menu_surface_height: 0,
                menu_surface_open_at_capture: false,
                process_memory: None,
                accessibility_backend: None,
                accessibility_node_count: 0,
                accessibility_action_count: 0,
            },
            DesktopNativeSmokeMetadata {
                proof_backend: "gtk",
                screenshot_backend: "gtk_widget_paintable_gsk_texture",
                missing_capture_error:
                    "the GTK event loop exited before the final DrawingArea capture",
            },
        )
    }

    fn scaffold_capabilities(&self) -> HostCapabilities {
        HostCapabilities::linux_scaffold()
    }

    fn native_host_capabilities(&self) -> HostCapabilities {
        HostCapabilities::linux_native_window_host()
    }

    fn desktop_capabilities(&self) -> DesktopCapabilities {
        DesktopCapabilities::linux_gtk_current()
    }

    #[cfg(feature = "clipboard")]
    fn read_clipboard(&mut self) -> ZsuiResult<Option<crate::ClipboardData>> {
        let mut clipboard = crate::linux_gtk_services::LinuxGtkClipboardService;
        crate::ClipboardService::read_clipboard(&mut clipboard)
    }

    #[cfg(feature = "clipboard")]
    fn write_clipboard(&mut self, data: &crate::ClipboardData) -> ZsuiResult<()> {
        let mut clipboard = crate::linux_gtk_services::LinuxGtkClipboardService;
        crate::ClipboardService::write_clipboard(&mut clipboard, data)
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
