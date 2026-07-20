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
        "macos_appkit"
    }

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()> {
        let _shell_runtimes = request.shell_runtimes;
        if !request.trays.is_empty() {
            return Err(ZsuiError::unsupported(
                "native_window_status_item",
                "the AppKit NSStatusItem runtime is not connected to the unified event loop",
            ));
        }
        crate::macos_appkit_services::run_macos_appkit_native_window_event_loop(
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
        let run = crate::macos_appkit_services::run_macos_appkit_native_window_event_loop(
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
        let accessibility_evidence_event = run.accessibility_evidence_event.clone();
        let mut report = complete_native_smoke(
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
                accessibility_backend: run.accessibility_backend,
                accessibility_node_count: run.accessibility_node_count,
                accessibility_action_count: 0,
            },
            DesktopNativeSmokeMetadata {
                proof_backend: "appkit",
                screenshot_backend: "appkit_nsview_bitmap_cache",
                missing_capture_error:
                    "the AppKit event loop exited before the final NSView capture",
            },
        )?;
        if let Some(event) = accessibility_evidence_event {
            report.events.push(event);
        }
        Ok(report)
    }

    fn scaffold_capabilities(&self) -> HostCapabilities {
        HostCapabilities::macos_scaffold()
    }

    fn native_host_capabilities(&self) -> HostCapabilities {
        HostCapabilities::macos_native_window_host()
    }

    fn desktop_capabilities(&self) -> DesktopCapabilities {
        DesktopCapabilities::macos_appkit_current()
    }

    fn native_proof_backend_name(&self) -> &'static str {
        "appkit"
    }

    fn native_proof_typography(&self, typography_scale: f32) -> crate::NativeTypographyProfile {
        crate::NativeTypographyProfile::fallback(
            crate::ZsTypographyPlatformStyle::Macos,
            typography_scale,
        )
    }

    fn capture_process_memory(
        &self,
        sample_point: &'static str,
    ) -> Option<crate::NativeProofProcessMemoryEvidence> {
        super::process_memory::capture_macos(sample_point)
    }

    #[cfg(feature = "clipboard")]
    fn read_clipboard(&mut self) -> ZsuiResult<Option<crate::ClipboardData>> {
        let mut clipboard = crate::macos_appkit_services::MacosAppKitClipboardService;
        crate::ClipboardService::read_clipboard(&mut clipboard)
    }

    #[cfg(feature = "clipboard")]
    fn write_clipboard(&mut self, data: &crate::ClipboardData) -> ZsuiResult<()> {
        let mut clipboard = crate::macos_appkit_services::MacosAppKitClipboardService;
        crate::ClipboardService::write_clipboard(&mut clipboard, data)
    }

    fn open_file_dialog(
        &mut self,
        spec: &FileDialogSpec,
    ) -> Option<ZsuiResult<Option<Vec<std::path::PathBuf>>>> {
        Some(crate::macos_appkit_services::macos_appkit_open_file_dialog(
            spec,
        ))
    }

    fn save_file_dialog(
        &mut self,
        spec: &SaveFileDialogSpec,
    ) -> ZsuiResult<Option<std::path::PathBuf>> {
        crate::macos_appkit_services::macos_appkit_save_file_dialog(spec)
    }
}
