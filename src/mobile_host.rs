use serde::Serialize;

use crate::{
    android_activity_host::android_activity_host_scaffold,
    harmony_ability_host::harmony_ability_host_scaffold,
};
use crate::{NativeUiAdapterCapability, NativeUiBackendStatus, NativeUiPlatform, NativeUiToolkit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeEntryPoint {
    pub symbol_name: &'static str,
    pub language: &'static str,
    pub file_path: &'static str,
    pub purpose: &'static str,
}

impl MobileRuntimeBridgeEntryPoint {
    pub const fn new(
        symbol_name: &'static str,
        language: &'static str,
        file_path: &'static str,
        purpose: &'static str,
    ) -> Self {
        Self {
            symbol_name,
            language,
            file_path,
            purpose,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeLifecycleBinding {
    pub native_callback: &'static str,
    pub zsui_stage_name: &'static str,
    pub required_for_runtime: bool,
}

impl MobileRuntimeLifecycleBinding {
    pub const fn new(
        native_callback: &'static str,
        zsui_stage_name: &'static str,
        required_for_runtime: bool,
    ) -> Self {
        Self {
            native_callback,
            zsui_stage_name,
            required_for_runtime,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeCapabilityBinding {
    pub capability: NativeUiAdapterCapability,
    pub capability_name: &'static str,
    pub platform_binding_name: &'static str,
    pub host_trait_name: &'static str,
    pub implemented: bool,
}

impl MobileRuntimeCapabilityBinding {
    pub const fn new(
        capability: NativeUiAdapterCapability,
        platform_binding_name: &'static str,
        host_trait_name: &'static str,
        implemented: bool,
    ) -> Self {
        Self {
            capability,
            capability_name: capability.capability_name(),
            platform_binding_name,
            host_trait_name,
            implemented,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimePermission {
    pub permission_name: &'static str,
    pub required_for: &'static str,
    pub required_by_default: bool,
}

impl MobileRuntimePermission {
    pub const fn new(
        permission_name: &'static str,
        required_for: &'static str,
        required_by_default: bool,
    ) -> Self {
        Self {
            permission_name,
            required_for,
            required_by_default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeHostScaffold {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit: NativeUiToolkit,
    pub toolkit_name: &'static str,
    pub status: NativeUiBackendStatus,
    pub status_name: &'static str,
    pub module_path: &'static str,
    pub native_library_name: &'static str,
    pub application_manifest_file: &'static str,
    pub native_window_type: &'static str,
    pub rust_entry_point: &'static str,
    pub bridge_entry_points: Vec<MobileRuntimeBridgeEntryPoint>,
    pub lifecycle_bindings: Vec<MobileRuntimeLifecycleBinding>,
    pub capability_bindings: Vec<MobileRuntimeCapabilityBinding>,
    pub required_permissions: Vec<MobileRuntimePermission>,
    pub target_smoke_requirements: Vec<&'static str>,
    pub next_implementation_steps: Vec<&'static str>,
}

impl MobileRuntimeHostScaffold {
    pub fn implemented_capability_names(&self) -> Vec<&'static str> {
        self.capability_bindings
            .iter()
            .filter(|binding| binding.implemented)
            .map(|binding| binding.capability_name)
            .collect()
    }

    pub fn pending_capability_names(&self) -> Vec<&'static str> {
        self.capability_bindings
            .iter()
            .filter(|binding| !binding.implemented)
            .map(|binding| binding.capability_name)
            .collect()
    }

    pub fn capability_binding_for(
        &self,
        capability: NativeUiAdapterCapability,
    ) -> Option<&MobileRuntimeCapabilityBinding> {
        self.capability_bindings
            .iter()
            .find(|binding| binding.capability == capability)
    }
}

pub fn mobile_runtime_host_scaffold(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeHostScaffold> {
    match platform {
        NativeUiPlatform::Android => Some(android_activity_host_scaffold()),
        NativeUiPlatform::Harmony => Some(harmony_ability_host_scaffold()),
        NativeUiPlatform::Windows | NativeUiPlatform::Macos | NativeUiPlatform::Linux => None,
    }
}

pub fn mobile_runtime_host_scaffolds() -> Vec<MobileRuntimeHostScaffold> {
    vec![
        android_activity_host_scaffold(),
        harmony_ability_host_scaffold(),
    ]
}

pub fn mobile_runtime_host_scaffold_module_paths() -> Vec<&'static str> {
    mobile_runtime_host_scaffolds()
        .iter()
        .map(|scaffold| scaffold.module_path)
        .collect()
}

pub fn mobile_runtime_host_scaffold_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_host_scaffold(platform))
}

pub fn mobile_runtime_host_scaffolds_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_host_scaffolds())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_scaffolds_exist_for_android_and_harmony_only() {
        let scaffolds = mobile_runtime_host_scaffolds();

        assert_eq!(scaffolds.len(), 2);
        assert_eq!(scaffolds[0].platform_name, "android");
        assert_eq!(scaffolds[1].platform_name, "harmony");
        assert!(mobile_runtime_host_scaffold(NativeUiPlatform::Windows).is_none());
        assert!(
            mobile_runtime_host_scaffold_module_paths().contains(&"src/android_activity_host.rs")
        );
        assert!(
            mobile_runtime_host_scaffold_module_paths().contains(&"src/harmony_ability_host.rs")
        );
    }

    #[test]
    fn mobile_scaffolds_name_pending_platform_bindings() {
        let android = mobile_runtime_host_scaffold(NativeUiPlatform::Android)
            .expect("android scaffold should exist");
        let harmony = mobile_runtime_host_scaffold(NativeUiPlatform::Harmony)
            .expect("harmony scaffold should exist");

        assert!(android.pending_capability_names().contains(&"main_window"));
        assert!(android
            .capability_binding_for(NativeUiAdapterCapability::MainWindow)
            .expect("android main window binding should exist")
            .platform_binding_name
            .contains("Activity"));
        assert!(harmony
            .capability_binding_for(NativeUiAdapterCapability::MainWindow)
            .expect("harmony main window binding should exist")
            .platform_binding_name
            .contains("Ability"));
        assert!(android.implemented_capability_names().is_empty());
        assert!(harmony.implemented_capability_names().is_empty());
    }

    #[test]
    fn mobile_scaffold_json_serializes_for_ai_context() {
        let json = mobile_runtime_host_scaffolds_json().expect("scaffolds should serialize");

        assert!(json.contains("android_activity"));
        assert!(json.contains("harmony_ability"));
        assert!(json.contains("device smoke artifacts"));
    }
}
