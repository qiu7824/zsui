use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiPlatform {
    Windows,
    Macos,
    Linux,
    Android,
    Harmony,
}

impl NativeUiPlatform {
    pub const fn platform_name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Harmony => "harmony",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiToolkit {
    WinitDesktop,
    Win32Gdi,
    AppKitSwiftUI,
    Gtk4Libadwaita,
    AndroidActivity,
    HarmonyAbility,
}

impl NativeUiToolkit {
    pub const fn toolkit_name(self) -> &'static str {
        match self {
            Self::WinitDesktop => "winit_desktop",
            Self::Win32Gdi => "win32_gdi",
            Self::AppKitSwiftUI => "appkit_swiftui",
            Self::Gtk4Libadwaita => "gtk4_libadwaita",
            Self::AndroidActivity => "android_activity",
            Self::HarmonyAbility => "harmony_ability",
        }
    }
}

pub const SUPPORTED_NATIVE_UI_PLATFORMS: [NativeUiPlatform; 5] = [
    NativeUiPlatform::Windows,
    NativeUiPlatform::Macos,
    NativeUiPlatform::Linux,
    NativeUiPlatform::Android,
    NativeUiPlatform::Harmony,
];

pub const SUPPORTED_NATIVE_UI_TOOLKITS: [NativeUiToolkit; 3] = [
    NativeUiToolkit::WinitDesktop,
    NativeUiToolkit::AndroidActivity,
    NativeUiToolkit::HarmonyAbility,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiBackendStatus {
    NativeHostIntegrated,
    NativeHostFirstPass,
    AdapterBoundaryScaffold,
}

impl NativeUiBackendStatus {
    pub const fn status_name(self) -> &'static str {
        match self {
            Self::NativeHostIntegrated => "native_host_integrated",
            Self::NativeHostFirstPass => "native_host_first_pass",
            Self::AdapterBoundaryScaffold => "adapter_boundary_scaffold",
        }
    }

    pub const fn is_native_runtime_ready(self) -> bool {
        matches!(self, Self::NativeHostIntegrated)
    }

    pub const fn is_scaffold(self) -> bool {
        matches!(self, Self::AdapterBoundaryScaffold)
    }

    pub const fn is_first_pass_native_host(self) -> bool {
        matches!(self, Self::NativeHostFirstPass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NativeUiBackendDescriptor {
    pub platform: NativeUiPlatform,
    pub toolkit: NativeUiToolkit,
    pub status: NativeUiBackendStatus,
    pub adapter_boundary: &'static str,
    pub module_path: &'static str,
}

impl NativeUiBackendDescriptor {
    pub const fn platform_name(&self) -> &'static str {
        self.platform.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.toolkit.toolkit_name()
    }

    pub const fn status_name(&self) -> &'static str {
        self.status.status_name()
    }
}

pub const SUPPORTED_NATIVE_UI_BACKENDS: [NativeUiBackendDescriptor; 5] = [
    NativeUiBackendDescriptor {
        platform: NativeUiPlatform::Windows,
        toolkit: NativeUiToolkit::WinitDesktop,
        status: NativeUiBackendStatus::NativeHostFirstPass,
        adapter_boundary: "DesktopWinitNativeWindowBoundary",
        module_path: "src/native.rs",
    },
    NativeUiBackendDescriptor {
        platform: NativeUiPlatform::Macos,
        toolkit: NativeUiToolkit::WinitDesktop,
        status: NativeUiBackendStatus::NativeHostFirstPass,
        adapter_boundary: "DesktopWinitNativeWindowBoundary",
        module_path: "src/native.rs",
    },
    NativeUiBackendDescriptor {
        platform: NativeUiPlatform::Linux,
        toolkit: NativeUiToolkit::WinitDesktop,
        status: NativeUiBackendStatus::NativeHostFirstPass,
        adapter_boundary: "DesktopWinitNativeWindowBoundary",
        module_path: "src/native.rs",
    },
    NativeUiBackendDescriptor {
        platform: NativeUiPlatform::Android,
        toolkit: NativeUiToolkit::AndroidActivity,
        status: NativeUiBackendStatus::AdapterBoundaryScaffold,
        adapter_boundary: "AndroidActivityAdapterBoundary",
        module_path: "src/android_activity_host.rs",
    },
    NativeUiBackendDescriptor {
        platform: NativeUiPlatform::Harmony,
        toolkit: NativeUiToolkit::HarmonyAbility,
        status: NativeUiBackendStatus::AdapterBoundaryScaffold,
        adapter_boundary: "HarmonyAbilityAdapterBoundary",
        module_path: "src/harmony_ability_host.rs",
    },
];

pub fn native_ui_backend_for_platform(
    platform: NativeUiPlatform,
) -> Option<&'static NativeUiBackendDescriptor> {
    SUPPORTED_NATIVE_UI_BACKENDS
        .iter()
        .find(|backend| backend.platform == platform)
}

pub fn native_ui_backend_for_toolkit(
    toolkit: NativeUiToolkit,
) -> Option<&'static NativeUiBackendDescriptor> {
    SUPPORTED_NATIVE_UI_BACKENDS
        .iter()
        .find(|backend| backend.toolkit == toolkit)
}

pub fn native_ui_platform_for_current_target() -> Option<NativeUiPlatform> {
    #[cfg(target_env = "ohos")]
    {
        return Some(NativeUiPlatform::Harmony);
    }
    #[cfg(target_os = "windows")]
    {
        return Some(NativeUiPlatform::Windows);
    }
    #[cfg(target_os = "macos")]
    {
        return Some(NativeUiPlatform::Macos);
    }
    #[cfg(target_os = "android")]
    {
        return Some(NativeUiPlatform::Android);
    }
    #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
    {
        return Some(NativeUiPlatform::Linux);
    }
    #[allow(unreachable_code)]
    None
}

pub fn native_ui_backend_for_current_target() -> Option<&'static NativeUiBackendDescriptor> {
    native_ui_backend_for_platform(native_ui_platform_for_current_target()?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiAdapterCapability {
    MainWindow,
    SettingsWindow,
    SettingsDropdown,
    InputDialog,
    EditDialog,
    Clipboard,
    PopupMenu,
    StatusItem,
    Renderer,
    TextLayout,
    MainSearchControl,
    TransientWindow,
    Ime,
    ShellOpen,
    FileDialog,
    PasteTarget,
    WindowIdentity,
    MainExecutionPlanBridge,
}

impl NativeUiAdapterCapability {
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::MainWindow => "main_window",
            Self::SettingsWindow => "settings_window",
            Self::SettingsDropdown => "settings_dropdown",
            Self::InputDialog => "input_dialog",
            Self::EditDialog => "edit_dialog",
            Self::Clipboard => "clipboard",
            Self::PopupMenu => "popup_menu",
            Self::StatusItem => "status_item",
            Self::Renderer => "renderer",
            Self::TextLayout => "text_layout",
            Self::MainSearchControl => "main_search_control",
            Self::TransientWindow => "transient_window",
            Self::Ime => "ime",
            Self::ShellOpen => "shell_open",
            Self::FileDialog => "file_dialog",
            Self::PasteTarget => "paste_target",
            Self::WindowIdentity => "window_identity",
            Self::MainExecutionPlanBridge => "main_execution_plan_bridge",
        }
    }
}

pub const REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES: [NativeUiAdapterCapability; 18] = [
    NativeUiAdapterCapability::MainWindow,
    NativeUiAdapterCapability::SettingsWindow,
    NativeUiAdapterCapability::SettingsDropdown,
    NativeUiAdapterCapability::InputDialog,
    NativeUiAdapterCapability::EditDialog,
    NativeUiAdapterCapability::Clipboard,
    NativeUiAdapterCapability::PopupMenu,
    NativeUiAdapterCapability::StatusItem,
    NativeUiAdapterCapability::Renderer,
    NativeUiAdapterCapability::TextLayout,
    NativeUiAdapterCapability::MainSearchControl,
    NativeUiAdapterCapability::TransientWindow,
    NativeUiAdapterCapability::Ime,
    NativeUiAdapterCapability::ShellOpen,
    NativeUiAdapterCapability::FileDialog,
    NativeUiAdapterCapability::PasteTarget,
    NativeUiAdapterCapability::WindowIdentity,
    NativeUiAdapterCapability::MainExecutionPlanBridge,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiBackendCapabilityMatrix {
    pub backend: NativeUiBackendDescriptor,
    pub required_capabilities: Vec<NativeUiAdapterCapability>,
}

impl NativeUiBackendCapabilityMatrix {
    pub fn native_runtime_ready(&self) -> bool {
        self.backend.status.is_native_runtime_ready()
    }

    pub fn scaffolded(&self) -> bool {
        self.backend.status.is_scaffold()
    }

    pub fn required_capability_names(&self) -> Vec<&'static str> {
        self.required_capabilities
            .iter()
            .map(|capability| capability.capability_name())
            .collect()
    }
}

pub fn native_ui_backend_capability_matrix() -> Vec<NativeUiBackendCapabilityMatrix> {
    SUPPORTED_NATIVE_UI_BACKENDS
        .iter()
        .map(|backend| NativeUiBackendCapabilityMatrix {
            backend: *backend,
            required_capabilities: REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES.to_vec(),
        })
        .collect()
}

pub fn native_ui_backend_capability_matrix_for_platform(
    platform: NativeUiPlatform,
) -> Option<NativeUiBackendCapabilityMatrix> {
    native_ui_backend_for_platform(platform).map(|backend| NativeUiBackendCapabilityMatrix {
        backend: *backend,
        required_capabilities: REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES.to_vec(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiAdapterBindingPlan {
    pub platform: NativeUiPlatform,
    pub toolkit: NativeUiToolkit,
    pub status: NativeUiBackendStatus,
    pub adapter_boundary: &'static str,
    pub binding_names: Vec<&'static str>,
}

impl NativeUiAdapterBindingPlan {
    pub fn new(
        platform: NativeUiPlatform,
        toolkit: NativeUiToolkit,
        status: NativeUiBackendStatus,
        adapter_boundary: &'static str,
        binding_names: Vec<&'static str>,
    ) -> Self {
        Self {
            platform,
            toolkit,
            status,
            adapter_boundary,
            binding_names,
        }
    }

    pub const fn platform_name(&self) -> &'static str {
        self.platform.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.toolkit.toolkit_name()
    }

    pub const fn status_name(&self) -> &'static str {
        self.status.status_name()
    }

    pub fn has_binding_name(&self, binding_name: &str) -> bool {
        self.binding_names.contains(&binding_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiAdapterReusePackage<TBootstrap> {
    pub manifest: NativeUiAdapterManifest,
    pub bootstrap: TBootstrap,
    pub binding_plan: NativeUiAdapterBindingPlan,
}

impl<TBootstrap> NativeUiAdapterReusePackage<TBootstrap> {
    pub const fn new(
        manifest: NativeUiAdapterManifest,
        bootstrap: TBootstrap,
        binding_plan: NativeUiAdapterBindingPlan,
    ) -> Self {
        Self {
            manifest,
            bootstrap,
            binding_plan,
        }
    }

    pub const fn platform_name(&self) -> &'static str {
        self.manifest.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.manifest.toolkit_name()
    }

    pub const fn status_name(&self) -> &'static str {
        self.manifest.status_name()
    }

    pub fn binding_count_matches_manifest(&self) -> bool {
        self.binding_plan.binding_names.len() == self.manifest.binding_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeUiAdapterParityReport {
    pub platform_names: Vec<&'static str>,
    pub toolkit_names: Vec<&'static str>,
    pub status_names: Vec<&'static str>,
    pub adapter_boundaries: Vec<&'static str>,
    pub binding_counts: Vec<usize>,
    pub main_execution_plan_counts: Vec<usize>,
    pub shared_non_host_protocol_counts: Vec<usize>,
    pub native_runtime_ready_platforms: Vec<&'static str>,
    pub first_pass_native_host_platforms: Vec<&'static str>,
    pub scaffold_platforms: Vec<&'static str>,
    pub all_binding_counts_match_manifest: bool,
    pub all_main_execution_plan_counts_match: bool,
    pub all_shared_non_host_protocol_counts_match: bool,
}

pub fn native_ui_adapter_parity_report<TBootstrap>(
    packages: &[NativeUiAdapterReusePackage<TBootstrap>],
) -> NativeUiAdapterParityReport {
    let main_execution_plan_counts: Vec<_> = packages
        .iter()
        .map(|package| package.manifest.main_execution_plans)
        .collect();
    let shared_non_host_protocol_counts: Vec<_> = packages
        .iter()
        .map(|package| package.manifest.shared_non_host_protocols)
        .collect();

    NativeUiAdapterParityReport {
        platform_names: packages
            .iter()
            .map(|package| package.platform_name())
            .collect(),
        toolkit_names: packages
            .iter()
            .map(|package| package.toolkit_name())
            .collect(),
        status_names: packages
            .iter()
            .map(|package| package.status_name())
            .collect(),
        adapter_boundaries: packages
            .iter()
            .map(|package| package.binding_plan.adapter_boundary)
            .collect(),
        binding_counts: packages
            .iter()
            .map(|package| package.manifest.binding_count)
            .collect(),
        main_execution_plan_counts: main_execution_plan_counts.clone(),
        shared_non_host_protocol_counts: shared_non_host_protocol_counts.clone(),
        native_runtime_ready_platforms: packages
            .iter()
            .filter(|package| package.manifest.status.is_native_runtime_ready())
            .map(|package| package.platform_name())
            .collect(),
        first_pass_native_host_platforms: packages
            .iter()
            .filter(|package| package.manifest.status.is_first_pass_native_host())
            .map(|package| package.platform_name())
            .collect(),
        scaffold_platforms: packages
            .iter()
            .filter(|package| package.manifest.status.is_scaffold())
            .map(|package| package.platform_name())
            .collect(),
        all_binding_counts_match_manifest: packages
            .iter()
            .all(NativeUiAdapterReusePackage::binding_count_matches_manifest),
        all_main_execution_plan_counts_match: all_counts_match(&main_execution_plan_counts),
        all_shared_non_host_protocol_counts_match: all_counts_match(
            &shared_non_host_protocol_counts,
        ),
    }
}

fn all_counts_match(counts: &[usize]) -> bool {
    match counts.first() {
        Some(expected) => counts.iter().all(|count| count == expected),
        None => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeUiAdapterManifest {
    pub platform: NativeUiPlatform,
    pub toolkit: NativeUiToolkit,
    pub status: NativeUiBackendStatus,
    pub binding_count: usize,
    pub main_execution_plans: usize,
    pub shared_non_host_protocols: usize,
}

impl NativeUiAdapterManifest {
    pub const fn new(
        platform: NativeUiPlatform,
        toolkit: NativeUiToolkit,
        status: NativeUiBackendStatus,
        binding_count: usize,
        main_execution_plans: usize,
        shared_non_host_protocols: usize,
    ) -> Self {
        Self {
            platform,
            toolkit,
            status,
            binding_count,
            main_execution_plans,
            shared_non_host_protocols,
        }
    }

    pub const fn platform_name(&self) -> &'static str {
        self.platform.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.toolkit.toolkit_name()
    }

    pub const fn status_name(&self) -> &'static str {
        self.status.status_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_manifest_resolves_desktop_targets() {
        let windows = native_ui_backend_for_platform(NativeUiPlatform::Windows)
            .expect("windows backend should be declared");

        assert_eq!(windows.toolkit, NativeUiToolkit::WinitDesktop);
        assert_eq!(windows.status, NativeUiBackendStatus::NativeHostFirstPass);
        assert_eq!(windows.platform_name(), "windows");
        assert_eq!(windows.toolkit_name(), "winit_desktop");
        let android = native_ui_backend_for_platform(NativeUiPlatform::Android)
            .expect("android backend should be declared");
        assert_eq!(android.toolkit, NativeUiToolkit::AndroidActivity);
        assert_eq!(
            android.status,
            NativeUiBackendStatus::AdapterBoundaryScaffold
        );

        let harmony = native_ui_backend_for_toolkit(NativeUiToolkit::HarmonyAbility)
            .expect("harmony backend should be declared");
        assert_eq!(harmony.platform, NativeUiPlatform::Harmony);
        assert_eq!(
            harmony.status,
            NativeUiBackendStatus::AdapterBoundaryScaffold
        );

        assert_eq!(SUPPORTED_NATIVE_UI_PLATFORMS.len(), 5);
        assert_eq!(SUPPORTED_NATIVE_UI_TOOLKITS.len(), 3);
        assert_eq!(REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES.len(), 18);
    }

    #[test]
    fn adapter_parity_report_tracks_runtime_statuses() {
        let packages = [
            NativeUiAdapterReusePackage::new(
                NativeUiAdapterManifest::new(
                    NativeUiPlatform::Windows,
                    NativeUiToolkit::Win32Gdi,
                    NativeUiBackendStatus::NativeHostIntegrated,
                    2,
                    1,
                    3,
                ),
                (),
                NativeUiAdapterBindingPlan::new(
                    NativeUiPlatform::Windows,
                    NativeUiToolkit::Win32Gdi,
                    NativeUiBackendStatus::NativeHostIntegrated,
                    "WindowsWin32AdapterBoundary",
                    vec!["main_window", "renderer"],
                ),
            ),
            NativeUiAdapterReusePackage::new(
                NativeUiAdapterManifest::new(
                    NativeUiPlatform::Linux,
                    NativeUiToolkit::Gtk4Libadwaita,
                    NativeUiBackendStatus::NativeHostFirstPass,
                    2,
                    1,
                    3,
                ),
                (),
                NativeUiAdapterBindingPlan::new(
                    NativeUiPlatform::Linux,
                    NativeUiToolkit::Gtk4Libadwaita,
                    NativeUiBackendStatus::NativeHostFirstPass,
                    "LinuxGtkAdapterBoundary",
                    vec!["main_window", "renderer"],
                ),
            ),
        ];

        let report = native_ui_adapter_parity_report(&packages);

        assert_eq!(report.native_runtime_ready_platforms, vec!["windows"]);
        assert_eq!(report.first_pass_native_host_platforms, vec!["linux"]);
        assert!(report.all_binding_counts_match_manifest);
        assert!(report.all_main_execution_plan_counts_match);
        assert!(report.all_shared_non_host_protocol_counts_match);
    }
}
