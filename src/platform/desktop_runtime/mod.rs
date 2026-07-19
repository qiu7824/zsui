use std::path::PathBuf;

use crate::{
    native::NativeViewInputRuntime, FileDialogSpec, NativeDrawPlan, SaveFileDialogSpec, TraySpec,
    WindowSpec, ZsShellRuntime, ZsuiError, ZsuiResult,
};

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

pub(super) trait DesktopRuntimeBackend: Default {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str;

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()>;

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

pub(crate) fn open_file_dialog(spec: &FileDialogSpec) -> Option<ZsuiResult<Option<Vec<PathBuf>>>> {
    SelectedDesktopRuntimeBackend::default().open_file_dialog(spec)
}

pub(crate) fn save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    SelectedDesktopRuntimeBackend::default().save_file_dialog(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_backend_has_a_stable_identity() {
        assert!(!SelectedDesktopRuntimeBackend::default()
            .backend_name()
            .is_empty());
    }

    #[test]
    fn native_core_delegates_production_runtime_and_dialog_selection() {
        let source = include_str!("../../native.rs");

        assert!(source.contains("crate::desktop_runtime::run_event_loop("));
        assert!(source.contains("crate::desktop_runtime::open_file_dialog(spec)"));
        assert!(source.contains("crate::desktop_runtime::save_file_dialog(spec)"));
        assert!(!source.contains("fn run_native_window_event_loop("));
        assert!(!source.contains("windows_win32_open_file_dialog"));
        assert!(!source.contains("macos_appkit_open_file_dialog"));
        assert!(!source.contains("linux_direct_open_file_dialog"));
        assert!(!source.contains("linux_gtk_open_file_dialog"));
    }
}
