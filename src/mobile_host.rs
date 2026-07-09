use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    android_activity_host::android_activity_host_scaffold,
    harmony_ability_host::harmony_ability_host_scaffold,
};
use crate::{
    NativeUiAdapterCapability, NativeUiBackendStatus, NativeUiPlatform, NativeUiToolkit, ZsuiError,
    ZsuiResult,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MobileRuntimeBridgeCallbackKind {
    Bootstrap,
    Lifecycle,
    Surface,
    Input,
    Command,
    EventPoll,
    Shutdown,
}

impl MobileRuntimeBridgeCallbackKind {
    pub const fn kind_name(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::Lifecycle => "lifecycle",
            Self::Surface => "surface",
            Self::Input => "input",
            Self::Command => "command",
            Self::EventPoll => "event_poll",
            Self::Shutdown => "shutdown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeCallback {
    pub callback_name: &'static str,
    pub symbol_name: &'static str,
    pub kind: MobileRuntimeBridgeCallbackKind,
    pub kind_name: &'static str,
    pub payload_contract: &'static str,
    pub required_for_runtime: bool,
}

impl MobileRuntimeBridgeCallback {
    pub const fn new(
        callback_name: &'static str,
        symbol_name: &'static str,
        kind: MobileRuntimeBridgeCallbackKind,
        payload_contract: &'static str,
        required_for_runtime: bool,
    ) -> Self {
        Self {
            callback_name,
            symbol_name,
            kind,
            kind_name: kind.kind_name(),
            payload_contract,
            required_for_runtime,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeDeviceSmokeArtifact {
    pub artifact_name: &'static str,
    pub file_name: &'static str,
    pub required_for_device_smoke: bool,
    pub description: &'static str,
}

impl MobileRuntimeDeviceSmokeArtifact {
    pub const fn new(
        artifact_name: &'static str,
        file_name: &'static str,
        required_for_device_smoke: bool,
        description: &'static str,
    ) -> Self {
        Self {
            artifact_name,
            file_name,
            required_for_device_smoke,
            description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContract {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit: NativeUiToolkit,
    pub toolkit_name: &'static str,
    pub module_path: &'static str,
    pub native_library_name: &'static str,
    pub rust_entry_point: &'static str,
    pub foreign_language: &'static str,
    pub foreign_entry_file: &'static str,
    pub callbacks: Vec<MobileRuntimeBridgeCallback>,
    pub device_smoke_artifacts: Vec<MobileRuntimeDeviceSmokeArtifact>,
    pub safety_rules: Vec<&'static str>,
}

impl MobileRuntimeBridgeContract {
    pub fn required_callback_symbol_names(&self) -> Vec<&'static str> {
        self.callbacks
            .iter()
            .filter(|callback| callback.required_for_runtime)
            .map(|callback| callback.symbol_name)
            .collect()
    }

    pub fn required_device_smoke_file_names(&self) -> Vec<&'static str> {
        self.device_smoke_artifacts
            .iter()
            .filter(|artifact| artifact.required_for_device_smoke)
            .map(|artifact| artifact.file_name)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeDeviceSmokePlan {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub backend_status_name: &'static str,
    pub artifact_dir: String,
    pub manifest_file: String,
    pub bridge_contract_file: String,
    pub manifest_command: String,
    pub bridge_contract_command: String,
    pub review_command: String,
    pub runtime_implemented: bool,
    pub device_smoke_ready: bool,
    pub blocking_reason: Option<String>,
    pub artifact_requirements: Vec<MobileRuntimeDeviceSmokeArtifact>,
}

impl MobileRuntimeDeviceSmokePlan {
    pub fn required_artifact_file_names(&self) -> Vec<&'static str> {
        self.artifact_requirements
            .iter()
            .filter(|artifact| artifact.required_for_device_smoke)
            .map(|artifact| artifact.file_name)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeDeviceSmokeArtifactStatus {
    pub artifact_name: &'static str,
    pub file_name: &'static str,
    pub required_for_device_smoke: bool,
    pub path: String,
    pub exists: bool,
    pub byte_len: Option<u64>,
    pub non_empty: bool,
    pub json_valid: Option<bool>,
    pub png_valid: Option<bool>,
    pub validation_error: Option<String>,
}

impl MobileRuntimeDeviceSmokeArtifactStatus {
    pub fn device_smoke_satisfied(&self) -> bool {
        self.exists
            && self.non_empty
            && self
                .json_valid
                .map(|json_valid| json_valid && self.validation_error.is_none())
                .unwrap_or_else(|| self.validation_error.is_none())
            && self
                .png_valid
                .map(|png_valid| png_valid && self.validation_error.is_none())
                .unwrap_or_else(|| self.validation_error.is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeDeviceSmokeReviewReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub artifact_dir: String,
    pub reviewed_at_unix_ms: u128,
    pub artifact_statuses: Vec<MobileRuntimeDeviceSmokeArtifactStatus>,
    pub required_artifact_count: usize,
    pub present_required_artifact_count: usize,
    pub valid_required_artifact_count: usize,
    pub missing_required_artifacts: Vec<String>,
    pub invalid_required_artifacts: Vec<String>,
    pub device_smoke_complete: bool,
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
    pub bridge_contract: MobileRuntimeBridgeContract,
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

pub fn mobile_runtime_bridge_contract(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeBridgeContract> {
    mobile_runtime_host_scaffold(platform).map(|scaffold| scaffold.bridge_contract)
}

pub fn mobile_runtime_bridge_contracts() -> Vec<MobileRuntimeBridgeContract> {
    mobile_runtime_host_scaffolds()
        .into_iter()
        .map(|scaffold| scaffold.bridge_contract)
        .collect()
}

pub fn mobile_runtime_bridge_contract_module_paths() -> Vec<&'static str> {
    mobile_runtime_bridge_contracts()
        .iter()
        .map(|contract| contract.module_path)
        .collect()
}

pub fn mobile_runtime_bridge_callback_symbol_names() -> Vec<&'static str> {
    mobile_runtime_bridge_contracts()
        .iter()
        .flat_map(|contract| {
            contract
                .callbacks
                .iter()
                .map(|callback| callback.symbol_name)
        })
        .collect()
}

pub fn mobile_runtime_device_smoke_artifact_names() -> Vec<&'static str> {
    mobile_runtime_bridge_contracts()
        .iter()
        .flat_map(|contract| {
            contract
                .device_smoke_artifacts
                .iter()
                .map(|artifact| artifact.file_name)
        })
        .collect()
}

pub fn mobile_runtime_device_smoke_command_names() -> Vec<&'static str> {
    vec![
        "mobile_scaffold_manifest",
        "mobile_scaffold_manifest --bridge",
        "mobile_scaffold_manifest --smoke",
        "mobile_scaffold_manifest --review",
    ]
}

pub fn mobile_runtime_bridge_contract_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contract(platform))
}

pub fn mobile_runtime_bridge_contracts_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contracts())
}

pub fn mobile_runtime_device_smoke_plan(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeDeviceSmokePlan> {
    mobile_runtime_device_smoke_plan_with_artifact_root(platform, "target/mobile-device-smoke")
}

pub fn mobile_runtime_device_smoke_plan_with_artifact_root(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> Option<MobileRuntimeDeviceSmokePlan> {
    let scaffold = mobile_runtime_host_scaffold(platform)?;
    let contract = scaffold.bridge_contract.clone();
    let platform_name = scaffold.platform_name;
    let artifact_dir = artifact_root.as_ref().join(platform_name);
    let runtime_implemented = scaffold.status != NativeUiBackendStatus::AdapterBoundaryScaffold
        && scaffold
            .capability_bindings
            .iter()
            .any(|binding| binding.implemented);

    Some(MobileRuntimeDeviceSmokePlan {
        platform,
        platform_name,
        toolkit_name: scaffold.toolkit_name,
        backend_status_name: scaffold.status_name,
        manifest_file: path_to_mobile_manifest_string(artifact_dir.join("manifest.json")),
        bridge_contract_file: path_to_mobile_manifest_string(
            artifact_dir.join("bridge-contract.json"),
        ),
        manifest_command: format!(
            "cargo run --example mobile_scaffold_manifest -- --smoke {platform_name}"
        ),
        bridge_contract_command: format!(
            "cargo run --example mobile_scaffold_manifest -- --bridge {platform_name}"
        ),
        review_command: format!(
            "cargo run --example mobile_scaffold_manifest -- --review {platform_name}"
        ),
        device_smoke_ready: runtime_implemented,
        blocking_reason: if runtime_implemented {
            None
        } else {
            Some(format!(
                "{platform_name} has a bridge contract but still needs real Activity/Ability FFI implementation and device artifacts"
            ))
        },
        artifact_dir: path_to_mobile_manifest_string(artifact_dir),
        artifact_requirements: contract.device_smoke_artifacts,
        runtime_implemented,
    })
}

pub fn mobile_runtime_device_smoke_plans() -> Vec<MobileRuntimeDeviceSmokePlan> {
    vec![NativeUiPlatform::Android, NativeUiPlatform::Harmony]
        .into_iter()
        .filter_map(mobile_runtime_device_smoke_plan)
        .collect()
}

pub fn mobile_runtime_device_smoke_plan_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_device_smoke_plan(platform))
}

pub fn mobile_runtime_device_smoke_plans_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_device_smoke_plans())
}

pub fn review_mobile_runtime_device_smoke_artifacts(
    platform: NativeUiPlatform,
) -> ZsuiResult<MobileRuntimeDeviceSmokeReviewReport> {
    review_mobile_runtime_device_smoke_artifacts_at(platform, "target/mobile-device-smoke")
}

pub fn review_mobile_runtime_device_smoke_artifacts_at(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<MobileRuntimeDeviceSmokeReviewReport> {
    let plan = mobile_runtime_device_smoke_plan_with_artifact_root(platform, artifact_root)
        .ok_or_else(|| {
            ZsuiError::unsupported(
                "mobile_device_smoke_review",
                format!(
                    "no mobile device smoke plan exists for `{}`",
                    platform.platform_name()
                ),
            )
        })?;
    let artifact_dir = PathBuf::from(&plan.artifact_dir);
    let artifact_statuses: Vec<_> = plan
        .artifact_requirements
        .iter()
        .map(|requirement| review_mobile_smoke_artifact(&artifact_dir, requirement))
        .collect();
    let required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_device_smoke)
        .count();
    let present_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_device_smoke && artifact.exists)
        .count();
    let valid_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_device_smoke && artifact.device_smoke_satisfied())
        .count();
    let missing_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_device_smoke && !artifact.exists)
        .map(|artifact| artifact.file_name.to_string())
        .collect();
    let invalid_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| {
            artifact.required_for_device_smoke
                && artifact.exists
                && !artifact.device_smoke_satisfied()
        })
        .map(|artifact| artifact.file_name.to_string())
        .collect();

    Ok(MobileRuntimeDeviceSmokeReviewReport {
        platform,
        platform_name: plan.platform_name,
        artifact_dir: plan.artifact_dir,
        reviewed_at_unix_ms: unix_ms_now(),
        device_smoke_complete: valid_required_artifact_count == required_artifact_count,
        artifact_statuses,
        required_artifact_count,
        present_required_artifact_count,
        valid_required_artifact_count,
        missing_required_artifacts,
        invalid_required_artifacts,
    })
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

fn review_mobile_smoke_artifact(
    artifact_dir: &Path,
    requirement: &MobileRuntimeDeviceSmokeArtifact,
) -> MobileRuntimeDeviceSmokeArtifactStatus {
    let path = artifact_dir.join(requirement.file_name);
    let path_string = path_to_mobile_manifest_string(&path);
    let metadata = fs::metadata(&path);
    let exists = metadata.is_ok();
    let byte_len = metadata.ok().map(|metadata| metadata.len());
    let non_empty = byte_len.map(|len| len > 0).unwrap_or(false);
    let mut json_valid = None;
    let mut png_valid = None;
    let mut validation_error = None;

    if exists && !non_empty {
        validation_error = Some("artifact is empty".to_string());
    }

    if exists && requirement.file_name.ends_with(".json") {
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                Ok(_) => json_valid = Some(true),
                Err(err) => {
                    json_valid = Some(false);
                    validation_error = Some(format!("invalid json: {err}"));
                }
            },
            Err(err) => {
                json_valid = Some(false);
                validation_error = Some(format!("read failed: {err}"));
            }
        }
    }

    if exists && requirement.file_name.ends_with(".png") {
        match fs::read(&path) {
            Ok(contents) => {
                let valid_header = contents.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]);
                png_valid = Some(valid_header);
                if !valid_header {
                    validation_error = Some("invalid png header".to_string());
                }
            }
            Err(err) => {
                png_valid = Some(false);
                validation_error = Some(format!("read failed: {err}"));
            }
        }
    }

    MobileRuntimeDeviceSmokeArtifactStatus {
        artifact_name: requirement.artifact_name,
        file_name: requirement.file_name,
        required_for_device_smoke: requirement.required_for_device_smoke,
        path: path_string,
        exists,
        byte_len,
        non_empty,
        json_valid,
        png_valid,
        validation_error,
    }
}

fn path_to_mobile_manifest_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}

fn unix_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
    fn mobile_bridge_contracts_name_required_runtime_callbacks_and_device_artifacts() {
        let android = mobile_runtime_bridge_contract(NativeUiPlatform::Android)
            .expect("android bridge contract should exist");
        let harmony = mobile_runtime_bridge_contract(NativeUiPlatform::Harmony)
            .expect("harmony bridge contract should exist");

        assert!(android
            .required_callback_symbol_names()
            .contains(&"zsui_android_activity_lifecycle"));
        assert!(android
            .required_callback_symbol_names()
            .contains(&"zsui_android_activity_surface_created"));
        assert!(harmony
            .required_callback_symbol_names()
            .contains(&"zsui_harmony_ability_lifecycle"));
        assert!(harmony
            .required_callback_symbol_names()
            .contains(&"zsui_harmony_ability_surface_created"));
        assert!(android
            .required_device_smoke_file_names()
            .contains(&"lifecycle.json"));
        assert!(harmony
            .required_device_smoke_file_names()
            .contains(&"surface.json"));
        assert!(mobile_runtime_bridge_callback_symbol_names()
            .contains(&"zsui_android_activity_dispatch_ui_event"));
        assert!(mobile_runtime_device_smoke_artifact_names().contains(&"device-window.png"));
    }

    #[test]
    fn mobile_device_smoke_plan_names_review_gate_without_claiming_runtime_ready() {
        let android = mobile_runtime_device_smoke_plan(NativeUiPlatform::Android)
            .expect("android device smoke plan should exist");

        assert_eq!(android.platform_name, "android");
        assert_eq!(android.toolkit_name, "android_activity");
        assert!(!android.runtime_implemented);
        assert!(!android.device_smoke_ready);
        assert!(android.blocking_reason.is_some());
        assert!(android
            .required_artifact_file_names()
            .contains(&"device-window.png"));
        assert!(android
            .review_command
            .contains("mobile_scaffold_manifest -- --review android"));
    }

    #[test]
    fn mobile_device_smoke_review_reports_missing_device_artifacts() {
        let root = unique_mobile_test_root("missing");
        let report =
            review_mobile_runtime_device_smoke_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("mobile device smoke review should report missing artifacts");

        assert!(!report.device_smoke_complete);
        assert_eq!(report.present_required_artifact_count, 0);
        assert!(report
            .missing_required_artifacts
            .contains(&"device-window.png".to_string()));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_device_smoke_review_validates_json_and_png_artifacts() {
        let root = unique_mobile_test_root("valid");
        let dir = root.join("harmony");
        fs::create_dir_all(&dir).expect("test artifact dir should be creatable");
        write_text(&dir.join("manifest.json"), "{}");
        write_text(&dir.join("device-launch.log"), "launched");
        write_png_header(&dir.join("device-window.png"));
        write_text(&dir.join("lifecycle.json"), "{\"events\":[\"onCreate\"]}");
        write_text(&dir.join("surface.json"), "{\"surface\":\"created\"}");
        write_text(&dir.join("input.json"), "{\"input\":\"touch\"}");

        let report =
            review_mobile_runtime_device_smoke_artifacts_at(NativeUiPlatform::Harmony, &root)
                .expect("mobile device smoke review should inspect artifacts");

        assert!(report.device_smoke_complete);
        assert_eq!(
            report.valid_required_artifact_count,
            report.required_artifact_count
        );
        assert!(report.missing_required_artifacts.is_empty());
        assert!(report.invalid_required_artifacts.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_scaffold_json_serializes_for_ai_context() {
        let json = mobile_runtime_host_scaffolds_json().expect("scaffolds should serialize");

        assert!(json.contains("android_activity"));
        assert!(json.contains("harmony_ability"));
        assert!(json.contains("device smoke artifacts"));
        assert!(json.contains("zsui_android_activity_surface_created"));
    }

    fn unique_mobile_test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zsui-mobile-smoke-{name}-{}", unix_ms_now()))
    }

    fn write_text(path: &Path, contents: &str) {
        fs::write(path, contents).expect("test text artifact should write");
    }

    fn write_png_header(path: &Path) {
        let mut file = fs::File::create(path).expect("test png artifact should write");
        file.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])
            .expect("test png header should write");
    }
}
