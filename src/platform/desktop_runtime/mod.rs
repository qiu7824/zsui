use std::path::PathBuf;

use crate::{
    native::NativeViewInputRuntime, ClipboardData, DesktopCapabilities, DialogResponse,
    FileDialogSpec, HostCapabilities, NativeDialogSpec, NativeDrawPlan,
    NativeProofProcessMemoryEvidence, NativeTypographyProfile, NativeWindowSmokeRunOptions,
    NativeWindowSmokeRunReport, SaveFileDialogSpec, TraySpec, WindowSpec, ZsShellRuntime,
    ZsuiError, ZsuiResult,
};

mod process_memory;

#[cfg_attr(all(windows, feature = "windows-win32"), path = "windows.rs")]
#[cfg_attr(all(target_os = "macos", feature = "macos-appkit"), path = "macos.rs")]
#[cfg_attr(
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        feature = "linux-direct-host"
    ),
    path = "linux_direct.rs"
)]
#[cfg_attr(
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        feature = "linux-gtk",
        not(feature = "linux-direct-host")
    ),
    path = "linux_gtk.rs"
)]
#[cfg_attr(
    any(
        all(
            target_os = "macos",
            feature = "desktop-winit",
            not(feature = "macos-appkit")
        ),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            feature = "desktop-winit",
            not(feature = "linux-direct-host"),
            not(feature = "linux-gtk")
        )
    ),
    path = "winit.rs"
)]
#[cfg_attr(
    not(any(
        all(windows, feature = "windows-win32"),
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            feature = "linux-direct-host"
        ),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            feature = "linux-gtk",
            not(feature = "linux-direct-host")
        ),
        all(
            target_os = "macos",
            feature = "desktop-winit",
            not(feature = "macos-appkit")
        ),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            feature = "desktop-winit",
            not(feature = "linux-direct-host"),
            not(feature = "linux-gtk")
        )
    )),
    path = "unsupported.rs"
)]
mod selected;

use selected::Backend as SelectedDesktopRuntimeBackend;

pub(super) struct DesktopRuntimeRequest {
    pub(super) windows: Vec<WindowSpec>,
    pub(super) trays: Vec<TraySpec>,
    pub(super) draw_plans: Vec<Option<NativeDrawPlan>>,
    pub(super) view_runtimes: Vec<NativeViewInputRuntime>,
    pub(super) shell_runtimes: Vec<Option<ZsShellRuntime>>,
}

pub(super) struct DesktopSmokeRequest {
    pub(super) windows: Vec<WindowSpec>,
    pub(super) draw_plans: Vec<Option<NativeDrawPlan>>,
    pub(super) view_runtime: NativeViewInputRuntime,
    pub(super) shell_runtime: Option<ZsShellRuntime>,
    pub(super) options: NativeWindowSmokeRunOptions,
}

#[allow(dead_code)]
pub(super) struct DesktopNativeSmokeOutcome {
    pub(super) created_window_count: usize,
    pub(super) proof_input_reports: Vec<crate::native::NativeViewInputDispatchReport>,
    pub(super) native_view_capture: Option<Result<crate::NativeViewCaptureEvidence, String>>,
    pub(super) menu_command_routed: bool,
    pub(super) menu_surface_created: bool,
    pub(super) menu_surface_height: u32,
    pub(super) menu_surface_open_at_capture: bool,
    pub(super) status_item_created: bool,
    pub(super) status_menu_native_command_count: usize,
    pub(super) status_menu_command_routed: bool,
    pub(super) status_menu_popup_created: bool,
    pub(super) status_menu_popup_destroyed: bool,
    pub(super) process_memory: Option<crate::NativeProofProcessMemoryEvidence>,
    pub(super) accessibility_backend: Option<&'static str>,
    pub(super) accessibility_node_count: usize,
    pub(super) accessibility_action_count: usize,
}

#[allow(dead_code)]
pub(super) struct DesktopNativeSmokeMetadata {
    pub(super) proof_backend: &'static str,
    pub(super) screenshot_backend: &'static str,
    pub(super) missing_capture_error: &'static str,
}

pub(super) trait DesktopRuntimeBackend: Default {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str;

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()>;

    fn run_smoke_event_loop(
        self,
        request: DesktopSmokeRequest,
    ) -> ZsuiResult<NativeWindowSmokeRunReport>;

    fn scaffold_capabilities(&self) -> HostCapabilities;

    fn native_host_capabilities(&self) -> HostCapabilities;

    fn desktop_capabilities(&self) -> DesktopCapabilities;

    fn native_proof_backend_name(&self) -> &'static str;

    fn native_proof_typography(&self, typography_scale: f32) -> NativeTypographyProfile;

    fn capture_process_memory(
        &self,
        sample_point: &'static str,
    ) -> Option<NativeProofProcessMemoryEvidence>;

    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        Err(ZsuiError::unsupported(
            "read_clipboard",
            "enable the clipboard feature and target-native desktop backend",
        ))
    }

    fn write_clipboard(&mut self, _data: &ClipboardData) -> ZsuiResult<()> {
        Err(ZsuiError::unsupported(
            "write_clipboard",
            "enable the clipboard feature and target-native desktop backend",
        ))
    }

    fn open_file_dialog(
        &mut self,
        _spec: &FileDialogSpec,
    ) -> Option<ZsuiResult<Option<Vec<PathBuf>>>> {
        None
    }

    fn save_file_dialog(&mut self, _spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        Err(ZsuiError::unsupported(
            "save_file_dialog",
            "the selected desktop backend does not implement a native save dialog",
        ))
    }

    fn show_native_dialog(&mut self, _spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        Err(ZsuiError::unsupported(
            "native_dialogs",
            "the selected desktop backend does not implement native message dialogs",
        ))
    }
}

pub(crate) fn run_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtimes: Vec<NativeViewInputRuntime>,
    shell_runtimes: Vec<Option<ZsShellRuntime>>,
) -> ZsuiResult<()> {
    SelectedDesktopRuntimeBackend::default().run_event_loop(DesktopRuntimeRequest {
        windows,
        trays,
        draw_plans,
        view_runtimes,
        shell_runtimes,
    })
}

pub(crate) fn scaffold_capabilities() -> HostCapabilities {
    SelectedDesktopRuntimeBackend::default().scaffold_capabilities()
}

pub(crate) fn native_host_capabilities() -> HostCapabilities {
    SelectedDesktopRuntimeBackend::default().native_host_capabilities()
}

pub(crate) fn desktop_capabilities() -> DesktopCapabilities {
    SelectedDesktopRuntimeBackend::default().desktop_capabilities()
}

pub(crate) fn native_proof_backend_name() -> &'static str {
    SelectedDesktopRuntimeBackend::default().native_proof_backend_name()
}

pub(crate) fn native_proof_typography(typography_scale: f32) -> NativeTypographyProfile {
    SelectedDesktopRuntimeBackend::default().native_proof_typography(typography_scale)
}

pub(crate) fn capture_process_memory(
    sample_point: &'static str,
) -> Option<NativeProofProcessMemoryEvidence> {
    SelectedDesktopRuntimeBackend::default().capture_process_memory(sample_point)
}

pub(crate) fn open_file_dialog(spec: &FileDialogSpec) -> Option<ZsuiResult<Option<Vec<PathBuf>>>> {
    SelectedDesktopRuntimeBackend::default().open_file_dialog(spec)
}

pub(crate) fn open_file_dialog_required(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    open_file_dialog(spec).unwrap_or_else(|| {
        Err(ZsuiError::unsupported(
            "open_file_dialog",
            "enable the target-native desktop backend feature",
        ))
    })
}

pub(crate) fn read_clipboard() -> ZsuiResult<Option<ClipboardData>> {
    SelectedDesktopRuntimeBackend::default().read_clipboard()
}

pub(crate) fn write_clipboard(data: &ClipboardData) -> ZsuiResult<()> {
    SelectedDesktopRuntimeBackend::default().write_clipboard(data)
}

pub(crate) fn run_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtime: NativeViewInputRuntime,
    shell_runtime: Option<ZsShellRuntime>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    SelectedDesktopRuntimeBackend::default().run_smoke_event_loop(DesktopSmokeRequest {
        windows,
        draw_plans,
        view_runtime,
        shell_runtime,
        options,
    })
}

#[allow(dead_code)]
pub(super) fn complete_native_smoke(
    request: DesktopSmokeRequest,
    outcome: DesktopNativeSmokeOutcome,
    metadata: DesktopNativeSmokeMetadata,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    let DesktopSmokeRequest {
        windows,
        draw_plans,
        view_runtime,
        shell_runtime,
        options,
    } = request;
    let expected_proof_input_reports =
        crate::native::native_view_smoke_input_dispatch_count(&options.native_view_inputs);
    if outcome.proof_input_reports.len() != expected_proof_input_reports {
        return Err(ZsuiError::host(
            "native_window_smoke_proof_inputs",
            format!(
                "backend `{}` returned {} scripted input reports; expected {}",
                metadata.proof_backend,
                outcome.proof_input_reports.len(),
                expected_proof_input_reports
            ),
        ));
    }
    let _backend_owned_state = (view_runtime, shell_runtime);
    let mut report = NativeWindowSmokeRunReport {
        requested_window_count: windows.len(),
        window_menu_requested_count: windows
            .iter()
            .filter(|window| window.menu.is_some())
            .count(),
        window_menu_native_command_count: windows
            .iter()
            .filter_map(|window| window.menu.as_ref())
            .map(crate::native::menu_command_count)
            .sum(),
        auto_close_after_ms: options.auto_close_after_ms,
        ..NativeWindowSmokeRunReport::empty(options.clone())
    };
    crate::native::record_draw_plan_smoke(&mut report, &draw_plans);
    report.created_window_count = outcome.created_window_count;
    report.window_menu_attached_count = report
        .window_menu_requested_count
        .min(outcome.created_window_count);
    report.close_requested_count = outcome.created_window_count;
    report.exited_by_auto_close = true;
    report.events.extend(
        windows
            .iter()
            .take(outcome.created_window_count)
            .map(|spec| format!("window_created:{}", spec.title)),
    );
    report.events.push("auto_close_elapsed".to_string());
    crate::native::record_native_view_input_reports(
        &mut report,
        &options.native_view_inputs,
        &outcome.proof_input_reports,
        metadata.proof_backend,
    );
    report.window_menu_command_routed = outcome.menu_command_routed;
    report.window_menu_surface_created = outcome.menu_surface_created;
    report.window_menu_surface_height = outcome.menu_surface_height;
    report.window_menu_surface_open_at_capture = outcome.menu_surface_open_at_capture;
    report.status_item_created = outcome.status_item_created;
    report.status_menu_native_command_count = outcome.status_menu_native_command_count;
    report.status_menu_command_routed = outcome.status_menu_command_routed;
    report.status_menu_popup_created = outcome.status_menu_popup_created;
    report.status_menu_popup_destroyed = outcome.status_menu_popup_destroyed;
    report.process_memory_during_runtime = outcome.process_memory;
    report.native_accessibility_backend = outcome.accessibility_backend;
    report.native_accessibility_node_count = outcome.accessibility_node_count;
    report.native_accessibility_action_count = outcome.accessibility_action_count;
    if let Some(backend) = outcome.accessibility_backend {
        report.events.push(format!(
            "native_accessibility_bridge:{backend}:nodes={}",
            outcome.accessibility_node_count
        ));
    }
    if outcome.menu_command_routed {
        report.events.push("window_menu_command_routed".to_string());
    }
    match outcome.native_view_capture {
        Some(Ok(capture)) => {
            report.screenshot_captured = true;
            report.native_view_capture = Some(capture);
            if let Some(path) = &report.screenshot_file {
                report.events.push(format!("screenshot_captured:{path}"));
            }
            report.events.push(format!(
                "screenshot_backend:{}",
                metadata.screenshot_backend
            ));
        }
        Some(Err(error)) => {
            report.screenshot_error = Some(error);
            report.events.push("screenshot_error".to_string());
        }
        None if options.screenshot_file.is_some() => {
            report.screenshot_error = Some(metadata.missing_capture_error.to_string());
            report.events.push("screenshot_error".to_string());
        }
        None => {}
    }
    if options.status_item.is_some() && !outcome.status_item_created {
        report.status_item_error = Some(
            "status-item smoke is not connected to the selected native event loop".to_string(),
        );
        report.events.push("status_item_unsupported".to_string());
    } else if outcome.status_item_created {
        report.events.push("status_item_created".to_string());
        report.events.push(format!(
            "status_menu_native_commands:{}",
            outcome.status_menu_native_command_count
        ));
        if outcome.status_menu_command_routed {
            report.events.push("status_menu_command_routed".to_string());
        }
        if outcome.status_menu_popup_created {
            report.events.push("status_menu_popup_created".to_string());
        }
        if outcome.status_menu_popup_destroyed {
            report
                .events
                .push("status_menu_popup_destroyed".to_string());
        }
    }
    if options.require_visible_window && !report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot && !report.screenshot_captured {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_status_item && !report.status_item_created {
        return Err(ZsuiError::unsupported(
            "native_window_smoke_status_item",
            report
                .status_item_error
                .clone()
                .unwrap_or_else(|| "status item was not created".to_string()),
        ));
    }
    Ok(report)
}

pub(crate) fn save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    SelectedDesktopRuntimeBackend::default().save_file_dialog(spec)
}

pub(crate) fn show_native_dialog(spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
    SelectedDesktopRuntimeBackend::default().show_native_dialog(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_smoke_report_accepts_native_status_item_evidence() {
        let options = NativeWindowSmokeRunOptions::quick()
            .status_item(TraySpec::new().item("Quit", crate::Command::Quit))
            .require_status_item(true);
        let report = complete_native_smoke(
            DesktopSmokeRequest {
                windows: vec![WindowSpec::new("Status item proof")],
                draw_plans: vec![None],
                view_runtime: NativeViewInputRuntime::default(),
                shell_runtime: None,
                options,
            },
            DesktopNativeSmokeOutcome {
                created_window_count: 1,
                proof_input_reports: Vec::new(),
                native_view_capture: None,
                menu_command_routed: false,
                menu_surface_created: false,
                menu_surface_height: 0,
                menu_surface_open_at_capture: false,
                status_item_created: true,
                status_menu_native_command_count: 1,
                status_menu_command_routed: true,
                status_menu_popup_created: true,
                status_menu_popup_destroyed: true,
                process_memory: None,
                accessibility_backend: None,
                accessibility_node_count: 0,
                accessibility_action_count: 0,
            },
            DesktopNativeSmokeMetadata {
                proof_backend: "test",
                screenshot_backend: "test",
                missing_capture_error: "not requested",
            },
        )
        .expect("native status item evidence should satisfy the required smoke gate");

        assert!(report.status_item_created);
        assert_eq!(report.status_item_error, None);
        assert_eq!(report.status_menu_native_command_count, 1);
        assert!(report.status_menu_command_routed);
        assert!(report.status_menu_popup_created);
        assert!(report.status_menu_popup_destroyed);
    }

    #[test]
    fn shared_smoke_report_rejects_unaligned_scripted_input_reports() {
        let options =
            NativeWindowSmokeRunOptions::quick().native_view_key_down(crate::NativeViewKey::Right);
        let error = complete_native_smoke(
            DesktopSmokeRequest {
                windows: vec![WindowSpec::new("Input proof")],
                draw_plans: vec![None],
                view_runtime: NativeViewInputRuntime::default(),
                shell_runtime: None,
                options,
            },
            DesktopNativeSmokeOutcome {
                created_window_count: 1,
                proof_input_reports: Vec::new(),
                native_view_capture: None,
                menu_command_routed: false,
                menu_surface_created: false,
                menu_surface_height: 0,
                menu_surface_open_at_capture: false,
                status_item_created: false,
                status_menu_native_command_count: 0,
                status_menu_command_routed: false,
                status_menu_popup_created: false,
                status_menu_popup_destroyed: false,
                process_memory: None,
                accessibility_backend: None,
                accessibility_node_count: 0,
                accessibility_action_count: 0,
            },
            DesktopNativeSmokeMetadata {
                proof_backend: "misaligned-test",
                screenshot_backend: "test",
                missing_capture_error: "not requested",
            },
        )
        .expect_err("missing scripted input reports must fail the native proof");

        assert!(error.to_string().contains("expected 1"));
        assert!(error.to_string().contains("misaligned-test"));
    }

    #[test]
    fn selected_backend_has_a_stable_identity() {
        let backend = SelectedDesktopRuntimeBackend::default();
        assert!(!backend.backend_name().is_empty());
        assert_eq!(
            backend.desktop_capabilities().platform,
            crate::PlatformName::current()
        );
        assert_eq!(
            crate::DesktopCapabilities::current_native_backend(),
            backend.desktop_capabilities()
        );
        assert_eq!(
            crate::HostCapabilities::current_platform_scaffold(),
            backend.scaffold_capabilities()
        );
        assert_eq!(
            crate::HostCapabilities::current_native_window_host(),
            backend.native_host_capabilities()
        );
    }

    #[test]
    fn appkit_host_capabilities_match_the_native_status_item_backend() {
        let capabilities = HostCapabilities::macos_native_window_host();
        if cfg!(feature = "macos-appkit") {
            assert_eq!(
                capabilities.tray_or_status_menu.status,
                crate::CapabilityStatus::Partial
            );
            assert!(capabilities
                .tray_or_status_menu
                .detail
                .contains("NSStatusItem"));
            assert!(capabilities
                .tray_or_status_menu
                .detail
                .contains("macOS 15 runtime smoke passed"));
        } else {
            assert_eq!(
                capabilities.tray_or_status_menu.status,
                crate::CapabilityStatus::Unsupported
            );
        }
    }

    #[test]
    fn native_core_delegates_production_runtime_and_dialog_selection() {
        let source = include_str!("../../native.rs");
        let desktop_services = include_str!("../../desktop_services.rs");
        let desktop_services_core = desktop_services
            .split_once("#[cfg(test)]")
            .map_or(desktop_services, |(core, _)| core);

        assert!(source.contains("crate::desktop_runtime::run_event_loop("));
        assert!(source.contains("crate::desktop_runtime::open_file_dialog(spec)"));
        assert!(source.contains("crate::desktop_runtime::save_file_dialog(spec)"));
        assert!(source.contains("crate::desktop_runtime::show_native_dialog(spec)"));
        assert!(source.contains("crate::desktop_runtime::run_smoke_event_loop("));
        assert!(desktop_services_core.contains("crate::desktop_runtime::desktop_capabilities()"));
        let capability_source = include_str!("../../capability.rs");
        let capability_core = capability_source
            .split_once("#[cfg(test)]")
            .map_or(capability_source, |(core, _)| core);
        assert!(capability_core.contains("crate::desktop_runtime::scaffold_capabilities()"));
        assert!(capability_core.contains("crate::desktop_runtime::native_host_capabilities()"));
        assert!(!capability_core.contains("PlatformName::current()"));
        assert!(!source.contains("fn run_native_window_event_loop("));
        assert!(!source.contains("fn run_native_window_smoke_event_loop("));
        assert!(!source.contains("capture_win32_hwnd_png"));
        assert!(!source.contains("post_windows_native_view_input"));
        assert!(!source.contains("WindowsWin32MessageLoop"));
        assert!(!source.contains("WindowsWin32ViewInputRoute"));
        assert!(!source.contains("windows_win32_route"));
        assert!(!source.contains("windows_win32_host"));
        assert!(!source.contains("run_macos_appkit_native_window_event_loop"));
        assert!(!source.contains("run_linux_direct_native_window_event_loop"));
        assert!(!source.contains("run_linux_gtk_native_window_event_loop"));
        assert!(!source.contains("windows_win32_open_file_dialog"));
        assert!(!source.contains("macos_appkit_open_file_dialog"));
        assert!(!source.contains("linux_direct_open_file_dialog"));
        assert!(!source.contains("linux_gtk_open_file_dialog"));
        assert!(desktop_services_core.contains("crate::desktop_runtime::read_clipboard()"));
        assert!(desktop_services_core.contains("crate::desktop_runtime::write_clipboard(data)"));
        assert!(desktop_services_core
            .contains("crate::desktop_runtime::open_file_dialog_required(spec)"));
        assert!(desktop_services_core.contains("crate::desktop_runtime::save_file_dialog(spec)"));
        assert!(desktop_services_core.contains("crate::desktop_runtime::show_native_dialog(spec)"));
        for forbidden in [
            "#[cfg(",
            "cfg!(target_os",
            "PlatformName::current()",
            "windows_win32_host",
            "macos_appkit_services",
            "linux_direct::",
            "linux_gtk_services",
        ] {
            assert!(
                !desktop_services_core.contains(forbidden),
                "desktop service facade contains platform dispatch: {forbidden}"
            );
        }
    }

    #[test]
    fn native_proof_delegates_target_identity_typography_and_memory_sampling() {
        let source = include_str!("../../native_proof.rs");
        let core = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(core, _)| core);

        assert!(core.contains("crate::desktop_runtime::native_proof_backend_name()"));
        assert!(core.contains("crate::desktop_runtime::native_proof_typography("));
        assert!(core.contains("crate::desktop_runtime::capture_process_memory("));
        for forbidden in [
            "target_os",
            "windows_sys",
            "libc::",
            "/proc/self",
            "windows_gdi_renderer",
            "cfg!(feature",
        ] {
            assert!(
                !core.contains(forbidden),
                "native proof shared layer contains target dispatch: {forbidden}"
            );
        }

        let process_memory = include_str!("process_memory.rs");
        assert!(process_memory.contains("GetProcessMemoryInfo"));
        assert!(process_memory.contains("mach_task_basic_info"));
        assert!(process_memory.contains("/proc/self/smaps_rollup"));
        for adapter in [
            include_str!("windows.rs"),
            include_str!("macos.rs"),
            include_str!("linux_direct.rs"),
            include_str!("linux_gtk.rs"),
            include_str!("winit.rs"),
            include_str!("unsupported.rs"),
        ] {
            assert!(adapter.contains("fn native_proof_backend_name("));
            assert!(adapter.contains("fn native_proof_typography("));
            assert!(adapter.contains("fn capture_process_memory("));
        }

        let windows_smoke = include_str!("windows_smoke.rs");
        assert!(windows_smoke.contains("report.native_view_capture = Some(capture)"));
        assert!(windows_smoke.contains("report.process_memory_during_runtime = process_memory"));
        assert!(windows_smoke.contains("Result<crate::NativeViewCaptureEvidence, String>"));
    }

    #[test]
    fn win32_input_route_owns_shared_semantics_without_a_second_state_machine() {
        let route = include_str!("../windows/input/mod.rs");
        let pointer = include_str!("../windows/input/pointer.rs");
        let runtime = include_str!("../windows/input/runtime.rs");
        let route_state = route
            .split_once("pub(crate) fn windows_win32_view_input_route")
            .map_or(route, |(state, _)| state);

        assert!(route_state.contains("shared_runtime: crate::native::NativeViewInputRuntime"));
        assert!(pointer.contains("self.shared_runtime.dispatch_pointer_down"));
        assert!(runtime.contains("self.shared_runtime.dispatch_app_command"));
        for duplicate in [
            "interaction_plan: ViewInteractionPlan",
            "ui_command_view: Option<ViewNode<UiCommand>>",
            "live_view: Option<SharedLiveViewRuntime>",
            "text_edit: Option<NativeTextEditState>",
            "combo_type_ahead: NativeComboTypeAheadState",
            "slider_drag: Option<crate::WidgetId>",
            "color_picker_drag: Option<",
            "pending_app_commands: Vec<Command>",
            "pending_ui_commands: Vec<UiCommand>",
        ] {
            assert!(
                !route_state.contains(duplicate),
                "Win32 input route duplicated shared semantic state: {duplicate}"
            );
        }
    }
}
