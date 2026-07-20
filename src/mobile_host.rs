use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::android_activity_host::android_activity_host_scaffold;
use crate::native_hosts::NativeRuntimeDriverOperation;
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

const REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS: [MobileRuntimeBridgeCallbackKind; 7] = [
    MobileRuntimeBridgeCallbackKind::Bootstrap,
    MobileRuntimeBridgeCallbackKind::Lifecycle,
    MobileRuntimeBridgeCallbackKind::Surface,
    MobileRuntimeBridgeCallbackKind::Input,
    MobileRuntimeBridgeCallbackKind::Command,
    MobileRuntimeBridgeCallbackKind::EventPoll,
    MobileRuntimeBridgeCallbackKind::Shutdown,
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MobileRuntimeDeviceSmokeTraceKind {
    Lifecycle,
    Surface,
    Input,
    Clipboard,
}

impl MobileRuntimeDeviceSmokeTraceKind {
    pub const fn trace_kind_name(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::Surface => "surface",
            Self::Input => "input",
            Self::Clipboard => "clipboard",
        }
    }

    pub const fn artifact_name(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle_trace",
            Self::Surface => "surface_trace",
            Self::Input => "input_trace",
            Self::Clipboard => "clipboard_trace",
        }
    }

    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle.json",
            Self::Surface => "surface.json",
            Self::Input => "input.json",
            Self::Clipboard => "clipboard.json",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeDeviceSmokeTrace {
    pub artifact_source: &'static str,
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub trace_kind: MobileRuntimeDeviceSmokeTraceKind,
    pub trace_kind_name: &'static str,
    pub artifact_name: &'static str,
    pub file_name: &'static str,
    pub required_for_device_smoke: bool,
    pub events: Vec<&'static str>,
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
    pub schema_valid: Option<bool>,
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
                .schema_valid
                .map(|schema_valid| schema_valid && self.validation_error.is_none())
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
pub struct MobileRuntimeBridgeParityReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub status_name: &'static str,
    pub scaffold_module_path: &'static str,
    pub contract_module_path: &'static str,
    pub native_library_name: &'static str,
    pub rust_entry_point: &'static str,
    pub foreign_entry_file: &'static str,
    pub entry_point_symbol_names: Vec<&'static str>,
    pub contract_callback_kind_names: Vec<&'static str>,
    pub required_callback_kind_names: Vec<&'static str>,
    pub missing_required_callback_kind_names: Vec<&'static str>,
    pub required_callback_symbol_names: Vec<&'static str>,
    pub pending_ffi_callback_symbol_names: Vec<&'static str>,
    pub required_device_smoke_file_names: Vec<&'static str>,
    pub lifecycle_binding_count: usize,
    pub required_lifecycle_binding_count: usize,
    pub capability_binding_count: usize,
    pub implemented_capability_count: usize,
    pub pending_capability_count: usize,
    pub required_device_smoke_artifact_count: usize,
    pub scaffold_matches_contract: bool,
    pub contract_covers_required_runtime_routes: bool,
    pub runtime_ffi_implemented: bool,
    pub ready_for_device_smoke: bool,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeDispatchStep {
    pub callback_name: &'static str,
    pub symbol_name: &'static str,
    pub kind: MobileRuntimeBridgeCallbackKind,
    pub kind_name: &'static str,
    pub dispatch_operation_name: &'static str,
    pub payload_contract: &'static str,
    pub required_for_runtime: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeDispatchReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub status_name: &'static str,
    pub dispatch_steps: Vec<MobileRuntimeBridgeDispatchStep>,
    pub dispatch_operation_names: Vec<&'static str>,
    pub native_runtime_driver_operation_names: Vec<&'static str>,
    pub required_callback_kind_names: Vec<&'static str>,
    pub covered_required_callback_kind_names: Vec<&'static str>,
    pub missing_required_callback_kind_names: Vec<&'static str>,
    pub required_dispatch_step_count: usize,
    pub dispatch_contract_ready: bool,
    pub runtime_ffi_implemented: bool,
    pub device_smoke_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractSmokeStep {
    pub callback_name: &'static str,
    pub symbol_name: &'static str,
    pub kind_name: &'static str,
    pub dispatch_operation_name: &'static str,
    pub payload_contract: &'static str,
    pub accepted_by_contract: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractSmokeReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub status_name: &'static str,
    pub smoke_kind_name: &'static str,
    pub steps: Vec<MobileRuntimeBridgeContractSmokeStep>,
    pub observed_dispatch_operation_names: Vec<&'static str>,
    pub required_dispatch_operation_names: Vec<&'static str>,
    pub missing_dispatch_operation_names: Vec<&'static str>,
    pub native_runtime_driver_operation_names: Vec<&'static str>,
    pub lifecycle_event_count: usize,
    pub surface_event_count: usize,
    pub input_event_count: usize,
    pub command_event_count: usize,
    pub event_poll_count: usize,
    pub shutdown_count: usize,
    pub accepted_step_count: usize,
    pub required_step_count: usize,
    pub contract_smoke_complete: bool,
    pub runtime_ffi_implemented: bool,
    pub device_smoke_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractArtifactWriteReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub artifact_dir: String,
    pub written_files: Vec<String>,
    pub contract_artifacts_complete: bool,
    pub device_smoke_complete: bool,
    pub missing_required_device_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractArtifactRequirement {
    pub artifact_name: &'static str,
    pub file_name: &'static str,
    pub required_for_contract_smoke: bool,
    pub description: &'static str,
}

impl MobileRuntimeBridgeContractArtifactRequirement {
    pub const fn new(
        artifact_name: &'static str,
        file_name: &'static str,
        required_for_contract_smoke: bool,
        description: &'static str,
    ) -> Self {
        Self {
            artifact_name,
            file_name,
            required_for_contract_smoke,
            description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractArtifactStatus {
    pub artifact_name: &'static str,
    pub file_name: &'static str,
    pub required_for_contract_smoke: bool,
    pub path: String,
    pub exists: bool,
    pub byte_len: Option<u64>,
    pub non_empty: bool,
    pub json_valid: Option<bool>,
    pub schema_valid: Option<bool>,
    pub validation_error: Option<String>,
}

impl MobileRuntimeBridgeContractArtifactStatus {
    pub fn contract_smoke_satisfied(&self) -> bool {
        self.exists
            && self.non_empty
            && self
                .json_valid
                .map(|json_valid| json_valid && self.validation_error.is_none())
                .unwrap_or_else(|| self.validation_error.is_none())
            && self
                .schema_valid
                .map(|schema_valid| schema_valid && self.validation_error.is_none())
                .unwrap_or_else(|| self.validation_error.is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRuntimeBridgeContractArtifactReviewReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub artifact_dir: String,
    pub reviewed_at_unix_ms: u128,
    pub artifact_statuses: Vec<MobileRuntimeBridgeContractArtifactStatus>,
    pub required_artifact_count: usize,
    pub present_required_artifact_count: usize,
    pub valid_required_artifact_count: usize,
    pub missing_required_artifacts: Vec<String>,
    pub invalid_required_artifacts: Vec<String>,
    pub contract_artifacts_complete: bool,
    pub device_smoke_proof_claimed: bool,
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
        "mobile_scaffold_manifest --parity",
        "mobile_scaffold_manifest --dispatch",
        "mobile_scaffold_manifest --dispatch-smoke",
        "mobile_scaffold_manifest --write-contract",
        "mobile_scaffold_manifest --review-contract",
        "mobile_scaffold_manifest --smoke",
        "mobile_scaffold_manifest --trace-template",
        "mobile_scaffold_manifest --review",
    ]
}

pub fn mobile_runtime_required_bridge_callback_kind_names() -> Vec<&'static str> {
    REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS
        .iter()
        .map(|kind| kind.kind_name())
        .collect()
}

pub fn mobile_runtime_bridge_contract_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contract(platform))
}

pub fn mobile_runtime_bridge_contracts_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contracts())
}

pub fn mobile_runtime_bridge_parity_report(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeBridgeParityReport> {
    let scaffold = mobile_runtime_host_scaffold(platform)?;
    let contract = scaffold.bridge_contract.clone();
    let required_callback_symbol_names = contract.required_callback_symbol_names();
    let required_callback_kind_names = mobile_runtime_required_bridge_callback_kind_names();
    let contract_callback_kind_names = contract_callback_kind_names(&contract.callbacks);
    let missing_required_callback_kind_names =
        missing_required_callback_kind_names(&contract.callbacks);
    let entry_point_symbol_names = scaffold
        .bridge_entry_points
        .iter()
        .map(|entry| entry.symbol_name)
        .collect::<Vec<_>>();
    let runtime_ffi_implemented = scaffold.status != NativeUiBackendStatus::AdapterBoundaryScaffold
        && scaffold
            .capability_bindings
            .iter()
            .any(|binding| binding.implemented);
    let pending_ffi_callback_symbol_names = if runtime_ffi_implemented {
        Vec::new()
    } else {
        required_callback_symbol_names.clone()
    };
    let required_device_smoke_file_names = contract.required_device_smoke_file_names();
    let ready_for_device_smoke = mobile_runtime_device_smoke_plan(platform)
        .map(|plan| plan.device_smoke_ready)
        .unwrap_or(false);
    let scaffold_matches_contract = scaffold.platform == contract.platform
        && scaffold.toolkit == contract.toolkit
        && scaffold.module_path == contract.module_path
        && scaffold.native_library_name == contract.native_library_name
        && scaffold.rust_entry_point == contract.rust_entry_point;
    let contract_covers_required_runtime_routes = missing_required_callback_kind_names.is_empty();
    let implemented_capability_count = scaffold
        .capability_bindings
        .iter()
        .filter(|binding| binding.implemented)
        .count();
    let pending_capability_count = scaffold
        .capability_bindings
        .iter()
        .filter(|binding| !binding.implemented)
        .count();
    let mut blocking_reasons = Vec::new();

    if !scaffold_matches_contract {
        blocking_reasons.push(format!(
            "{} scaffold and bridge contract metadata differ",
            scaffold.platform_name
        ));
    }
    if !contract_covers_required_runtime_routes {
        blocking_reasons.push(format!(
            "{} bridge contract is missing required callback route kinds: {}",
            scaffold.platform_name,
            missing_required_callback_kind_names.join(", ")
        ));
    }
    if !runtime_ffi_implemented {
        blocking_reasons.push(format!(
            "{} FFI callback symbols are declared but still pending implementation",
            scaffold.platform_name
        ));
    }
    if !ready_for_device_smoke {
        blocking_reasons.push(format!(
            "{} device smoke is blocked until the Activity runtime exists",
            scaffold.platform_name
        ));
    }

    Some(MobileRuntimeBridgeParityReport {
        platform,
        platform_name: scaffold.platform_name,
        toolkit_name: scaffold.toolkit_name,
        status_name: scaffold.status_name,
        scaffold_module_path: scaffold.module_path,
        contract_module_path: contract.module_path,
        native_library_name: scaffold.native_library_name,
        rust_entry_point: scaffold.rust_entry_point,
        foreign_entry_file: contract.foreign_entry_file,
        entry_point_symbol_names,
        contract_callback_kind_names,
        required_callback_kind_names,
        missing_required_callback_kind_names,
        required_callback_symbol_names,
        pending_ffi_callback_symbol_names,
        required_device_smoke_file_names,
        lifecycle_binding_count: scaffold.lifecycle_bindings.len(),
        required_lifecycle_binding_count: scaffold
            .lifecycle_bindings
            .iter()
            .filter(|binding| binding.required_for_runtime)
            .count(),
        capability_binding_count: scaffold.capability_bindings.len(),
        implemented_capability_count,
        pending_capability_count,
        required_device_smoke_artifact_count: contract
            .device_smoke_artifacts
            .iter()
            .filter(|artifact| artifact.required_for_device_smoke)
            .count(),
        scaffold_matches_contract,
        contract_covers_required_runtime_routes,
        runtime_ffi_implemented,
        ready_for_device_smoke,
        blocking_reasons,
    })
}

pub fn mobile_runtime_bridge_parity_reports() -> Vec<MobileRuntimeBridgeParityReport> {
    vec![NativeUiPlatform::Android]
        .into_iter()
        .filter_map(mobile_runtime_bridge_parity_report)
        .collect()
}

pub fn mobile_runtime_bridge_parity_report_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_parity_report(platform))
}

pub fn mobile_runtime_bridge_parity_reports_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_parity_reports())
}

pub fn mobile_runtime_bridge_dispatch_steps(
    platform: NativeUiPlatform,
) -> Option<Vec<MobileRuntimeBridgeDispatchStep>> {
    let contract = mobile_runtime_bridge_contract(platform)?;
    Some(
        contract
            .callbacks
            .into_iter()
            .map(|callback| MobileRuntimeBridgeDispatchStep {
                callback_name: callback.callback_name,
                symbol_name: callback.symbol_name,
                kind: callback.kind,
                kind_name: callback.kind_name,
                dispatch_operation_name: mobile_runtime_bridge_dispatch_operation_name(
                    callback.kind,
                ),
                payload_contract: callback.payload_contract,
                required_for_runtime: callback.required_for_runtime,
            })
            .collect(),
    )
}

pub fn mobile_runtime_bridge_dispatch_report(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeBridgeDispatchReport> {
    let scaffold = mobile_runtime_host_scaffold(platform)?;
    let dispatch_steps = mobile_runtime_bridge_dispatch_steps(platform)?;
    let covered_required_callback_kind_names =
        covered_required_callback_kind_names_from_steps(&dispatch_steps);
    let missing_required_callback_kind_names =
        missing_required_callback_kind_names_from_steps(&dispatch_steps);
    let dispatch_operation_names = dispatch_operation_names_from_steps(&dispatch_steps);
    let native_runtime_driver_operation_names =
        mobile_runtime_bridge_native_driver_operation_names_from_steps(&dispatch_steps);
    let runtime_ffi_implemented = scaffold.status != NativeUiBackendStatus::AdapterBoundaryScaffold
        && scaffold
            .capability_bindings
            .iter()
            .any(|binding| binding.implemented);
    let device_smoke_ready = mobile_runtime_device_smoke_plan(platform)
        .map(|plan| plan.device_smoke_ready)
        .unwrap_or(false);
    let required_dispatch_step_count = dispatch_steps
        .iter()
        .filter(|step| step.required_for_runtime)
        .count();
    let dispatch_contract_ready = missing_required_callback_kind_names.is_empty()
        && native_runtime_driver_operation_names
            .contains(&NativeRuntimeDriverOperation::StartRuntime.operation_name())
        && native_runtime_driver_operation_names
            .contains(&NativeRuntimeDriverOperation::DispatchUiCommand.operation_name())
        && native_runtime_driver_operation_names
            .contains(&NativeRuntimeDriverOperation::PollApplicationEvent.operation_name())
        && native_runtime_driver_operation_names
            .contains(&NativeRuntimeDriverOperation::RequestShutdown.operation_name())
        && dispatch_operation_names.contains(&"apply_lifecycle_event")
        && dispatch_operation_names.contains(&"bind_or_resize_surface")
        && dispatch_operation_names.contains(&"dispatch_ui_event");

    Some(MobileRuntimeBridgeDispatchReport {
        platform,
        platform_name: scaffold.platform_name,
        toolkit_name: scaffold.toolkit_name,
        status_name: scaffold.status_name,
        dispatch_steps,
        dispatch_operation_names,
        native_runtime_driver_operation_names,
        required_callback_kind_names: mobile_runtime_required_bridge_callback_kind_names(),
        covered_required_callback_kind_names,
        missing_required_callback_kind_names,
        required_dispatch_step_count,
        dispatch_contract_ready,
        runtime_ffi_implemented,
        device_smoke_ready,
    })
}

pub fn mobile_runtime_bridge_dispatch_reports() -> Vec<MobileRuntimeBridgeDispatchReport> {
    vec![NativeUiPlatform::Android]
        .into_iter()
        .filter_map(mobile_runtime_bridge_dispatch_report)
        .collect()
}

pub fn mobile_runtime_bridge_dispatch_report_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_dispatch_report(platform))
}

pub fn mobile_runtime_bridge_dispatch_reports_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_dispatch_reports())
}

pub fn mobile_runtime_required_bridge_dispatch_operation_names() -> Vec<&'static str> {
    vec![
        NativeRuntimeDriverOperation::StartRuntime.operation_name(),
        "apply_lifecycle_event",
        "bind_or_resize_surface",
        "dispatch_ui_event",
        NativeRuntimeDriverOperation::DispatchUiCommand.operation_name(),
        NativeRuntimeDriverOperation::PollApplicationEvent.operation_name(),
        NativeRuntimeDriverOperation::RequestShutdown.operation_name(),
    ]
}

pub fn mobile_runtime_bridge_contract_smoke_report(
    platform: NativeUiPlatform,
) -> Option<MobileRuntimeBridgeContractSmokeReport> {
    let dispatch = mobile_runtime_bridge_dispatch_report(platform)?;
    let required_dispatch_operation_names =
        mobile_runtime_required_bridge_dispatch_operation_names();
    let observed_dispatch_operation_names =
        dispatch_operation_names_from_steps(&dispatch.dispatch_steps);
    let missing_dispatch_operation_names = required_dispatch_operation_names
        .iter()
        .filter(|operation| !observed_dispatch_operation_names.contains(operation))
        .copied()
        .collect::<Vec<_>>();
    let steps = dispatch
        .dispatch_steps
        .iter()
        .map(|step| MobileRuntimeBridgeContractSmokeStep {
            callback_name: step.callback_name,
            symbol_name: step.symbol_name,
            kind_name: step.kind_name,
            dispatch_operation_name: step.dispatch_operation_name,
            payload_contract: step.payload_contract,
            accepted_by_contract: step.required_for_runtime
                && required_dispatch_operation_names.contains(&step.dispatch_operation_name),
        })
        .collect::<Vec<_>>();
    let accepted_step_count = steps
        .iter()
        .filter(|step| step.accepted_by_contract)
        .count();
    let contract_smoke_complete = missing_dispatch_operation_names.is_empty()
        && accepted_step_count == dispatch.required_dispatch_step_count
        && dispatch.dispatch_contract_ready;

    Some(MobileRuntimeBridgeContractSmokeReport {
        platform,
        platform_name: dispatch.platform_name,
        toolkit_name: dispatch.toolkit_name,
        status_name: dispatch.status_name,
        smoke_kind_name: "contract_dispatch_smoke",
        lifecycle_event_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::Lifecycle,
        ),
        surface_event_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::Surface,
        ),
        input_event_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::Input,
        ),
        command_event_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::Command,
        ),
        event_poll_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::EventPoll,
        ),
        shutdown_count: mobile_runtime_bridge_kind_count(
            &dispatch.dispatch_steps,
            MobileRuntimeBridgeCallbackKind::Shutdown,
        ),
        native_runtime_driver_operation_names: dispatch.native_runtime_driver_operation_names,
        required_step_count: dispatch.required_dispatch_step_count,
        runtime_ffi_implemented: dispatch.runtime_ffi_implemented,
        device_smoke_ready: dispatch.device_smoke_ready,
        steps,
        observed_dispatch_operation_names,
        required_dispatch_operation_names,
        missing_dispatch_operation_names,
        accepted_step_count,
        contract_smoke_complete,
    })
}

pub fn mobile_runtime_bridge_contract_smoke_reports() -> Vec<MobileRuntimeBridgeContractSmokeReport>
{
    vec![NativeUiPlatform::Android]
        .into_iter()
        .filter_map(mobile_runtime_bridge_contract_smoke_report)
        .collect()
}

pub fn mobile_runtime_bridge_contract_smoke_report_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contract_smoke_report(platform))
}

pub fn mobile_runtime_bridge_contract_smoke_reports_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_bridge_contract_smoke_reports())
}

pub fn mobile_runtime_bridge_contract_artifact_requirements(
) -> Vec<MobileRuntimeBridgeContractArtifactRequirement> {
    vec![
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "mobile_manifest",
            "manifest.json",
            true,
            "serialized mobile host scaffold manifest",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "bridge_contract",
            "bridge-contract.json",
            true,
            "serialized mobile bridge callback contract",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "bridge_parity",
            "bridge-parity.json",
            true,
            "scaffold and bridge contract parity report",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "bridge_dispatch",
            "bridge-dispatch.json",
            true,
            "callback to runtime-operation dispatch report",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "dispatch_smoke",
            "dispatch-smoke.json",
            true,
            "local contract dispatch smoke report",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "device_smoke_plan",
            "device-smoke-plan.json",
            true,
            "device-smoke artifact plan for the mobile bridge",
        ),
        MobileRuntimeBridgeContractArtifactRequirement::new(
            "agent_context",
            "agent-context.json",
            true,
            "serialized ZSUI agent context captured with the mobile contract bundle",
        ),
    ]
}

pub fn mobile_runtime_bridge_contract_artifact_file_names() -> Vec<&'static str> {
    mobile_runtime_bridge_contract_artifact_requirements()
        .iter()
        .map(|requirement| requirement.file_name)
        .collect()
}

pub fn write_mobile_runtime_bridge_contract_artifacts(
    platform: NativeUiPlatform,
) -> ZsuiResult<MobileRuntimeBridgeContractArtifactWriteReport> {
    write_mobile_runtime_bridge_contract_artifacts_to(platform, "target/mobile-device-smoke")
}

pub fn write_mobile_runtime_bridge_contract_artifacts_for_all(
) -> ZsuiResult<Vec<MobileRuntimeBridgeContractArtifactWriteReport>> {
    write_mobile_runtime_bridge_contract_artifacts_for_all_to("target/mobile-device-smoke")
}

pub fn write_mobile_runtime_bridge_contract_artifacts_for_all_to(
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<Vec<MobileRuntimeBridgeContractArtifactWriteReport>> {
    let artifact_root = artifact_root.as_ref();
    [NativeUiPlatform::Android]
        .into_iter()
        .map(|platform| write_mobile_runtime_bridge_contract_artifacts_to(platform, artifact_root))
        .collect()
}

pub fn write_mobile_runtime_bridge_contract_artifacts_to(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<MobileRuntimeBridgeContractArtifactWriteReport> {
    let plan = mobile_runtime_device_smoke_plan_with_artifact_root(platform, artifact_root)
        .ok_or_else(|| {
            ZsuiError::unsupported(
                "mobile_bridge_contract_artifacts",
                format!(
                    "no mobile bridge contract artifact plan exists for `{}`",
                    platform.platform_name()
                ),
            )
        })?;
    let scaffold = mobile_runtime_host_scaffold(platform).ok_or_else(|| {
        ZsuiError::unsupported(
            "mobile_bridge_contract_artifacts",
            format!(
                "no mobile host scaffold exists for `{}`",
                platform.platform_name()
            ),
        )
    })?;
    let bridge_contract = mobile_runtime_bridge_contract(platform).ok_or_else(|| {
        ZsuiError::unsupported(
            "mobile_bridge_contract_artifacts",
            format!(
                "no mobile bridge contract exists for `{}`",
                platform.platform_name()
            ),
        )
    })?;
    let parity = mobile_runtime_bridge_parity_report(platform).ok_or_else(|| {
        ZsuiError::unsupported(
            "mobile_bridge_contract_artifacts",
            format!(
                "no mobile bridge parity report exists for `{}`",
                platform.platform_name()
            ),
        )
    })?;
    let dispatch = mobile_runtime_bridge_dispatch_report(platform).ok_or_else(|| {
        ZsuiError::unsupported(
            "mobile_bridge_contract_artifacts",
            format!(
                "no mobile bridge dispatch report exists for `{}`",
                platform.platform_name()
            ),
        )
    })?;
    let dispatch_smoke =
        mobile_runtime_bridge_contract_smoke_report(platform).ok_or_else(|| {
            ZsuiError::unsupported(
                "mobile_bridge_contract_artifacts",
                format!(
                    "no mobile bridge contract smoke report exists for `{}`",
                    platform.platform_name()
                ),
            )
        })?;
    let agent_context = crate::agent_context::zsui_agent_context();

    let artifact_dir = PathBuf::from(&plan.artifact_dir);
    fs::create_dir_all(&artifact_dir)
        .map_err(|err| mobile_smoke_io_error("create_artifact_dir", &artifact_dir, err))?;

    let mut written_files = Vec::new();
    write_mobile_json_artifact(
        &artifact_dir,
        "manifest.json",
        &scaffold,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "bridge-contract.json",
        &bridge_contract,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "bridge-parity.json",
        &parity,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "bridge-dispatch.json",
        &dispatch,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "dispatch-smoke.json",
        &dispatch_smoke,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "device-smoke-plan.json",
        &plan,
        &mut written_files,
    )?;
    write_mobile_json_artifact(
        &artifact_dir,
        "agent-context.json",
        &agent_context,
        &mut written_files,
    )?;

    let missing_required_device_artifacts: Vec<String> = plan
        .artifact_requirements
        .iter()
        .filter(|artifact| artifact.required_for_device_smoke)
        .filter(|artifact| !artifact_dir.join(artifact.file_name).exists())
        .map(|artifact| artifact.file_name.to_string())
        .collect();

    Ok(MobileRuntimeBridgeContractArtifactWriteReport {
        platform,
        platform_name: plan.platform_name,
        artifact_dir: plan.artifact_dir,
        contract_artifacts_complete: written_files.len()
            == mobile_runtime_bridge_contract_artifact_requirements().len(),
        device_smoke_complete: missing_required_device_artifacts.is_empty(),
        written_files,
        missing_required_device_artifacts,
    })
}

pub fn review_mobile_runtime_bridge_contract_artifacts(
    platform: NativeUiPlatform,
) -> ZsuiResult<MobileRuntimeBridgeContractArtifactReviewReport> {
    review_mobile_runtime_bridge_contract_artifacts_at(platform, "target/mobile-device-smoke")
}

pub fn review_mobile_runtime_bridge_contract_artifacts_for_all(
) -> ZsuiResult<Vec<MobileRuntimeBridgeContractArtifactReviewReport>> {
    review_mobile_runtime_bridge_contract_artifacts_for_all_at("target/mobile-device-smoke")
}

pub fn review_mobile_runtime_bridge_contract_artifacts_for_all_at(
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<Vec<MobileRuntimeBridgeContractArtifactReviewReport>> {
    let artifact_root = artifact_root.as_ref();
    [NativeUiPlatform::Android]
        .into_iter()
        .map(|platform| review_mobile_runtime_bridge_contract_artifacts_at(platform, artifact_root))
        .collect()
}

pub fn review_mobile_runtime_bridge_contract_artifacts_at(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<MobileRuntimeBridgeContractArtifactReviewReport> {
    let plan = mobile_runtime_device_smoke_plan_with_artifact_root(platform, artifact_root)
        .ok_or_else(|| {
            ZsuiError::unsupported(
                "mobile_bridge_contract_artifact_review",
                format!(
                    "no mobile bridge contract artifact review exists for `{}`",
                    platform.platform_name()
                ),
            )
        })?;
    let artifact_dir = PathBuf::from(&plan.artifact_dir);
    let artifact_statuses: Vec<_> = mobile_runtime_bridge_contract_artifact_requirements()
        .iter()
        .map(|requirement| {
            review_mobile_contract_artifact(&artifact_dir, requirement, plan.platform_name)
        })
        .collect();
    let required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_contract_smoke)
        .count();
    let present_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_contract_smoke && artifact.exists)
        .count();
    let valid_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| {
            artifact.required_for_contract_smoke && artifact.contract_smoke_satisfied()
        })
        .count();
    let missing_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_contract_smoke && !artifact.exists)
        .map(|artifact| artifact.file_name.to_string())
        .collect();
    let invalid_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| {
            artifact.required_for_contract_smoke
                && artifact.exists
                && !artifact.contract_smoke_satisfied()
        })
        .map(|artifact| artifact.file_name.to_string())
        .collect();

    Ok(MobileRuntimeBridgeContractArtifactReviewReport {
        platform,
        platform_name: plan.platform_name,
        artifact_dir: plan.artifact_dir,
        reviewed_at_unix_ms: unix_ms_now(),
        contract_artifacts_complete: valid_required_artifact_count == required_artifact_count,
        device_smoke_proof_claimed: false,
        artifact_statuses,
        required_artifact_count,
        present_required_artifact_count,
        valid_required_artifact_count,
        missing_required_artifacts,
        invalid_required_artifacts,
    })
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
                "{platform_name} has a bridge contract but still needs a real Activity FFI implementation and device artifacts"
            ))
        },
        artifact_dir: path_to_mobile_manifest_string(artifact_dir),
        artifact_requirements: contract.device_smoke_artifacts,
        runtime_implemented,
    })
}

pub fn mobile_runtime_device_smoke_plans() -> Vec<MobileRuntimeDeviceSmokePlan> {
    vec![NativeUiPlatform::Android]
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

pub fn mobile_runtime_device_smoke_trace_template(
    platform: NativeUiPlatform,
    trace_kind: MobileRuntimeDeviceSmokeTraceKind,
) -> Option<MobileRuntimeDeviceSmokeTrace> {
    let plan = mobile_runtime_device_smoke_plan(platform)?;
    plan.artifact_requirements
        .into_iter()
        .find(|artifact| artifact.artifact_name == trace_kind.artifact_name())
        .map(|artifact| MobileRuntimeDeviceSmokeTrace {
            artifact_source: "device_smoke",
            platform,
            platform_name: plan.platform_name,
            trace_kind,
            trace_kind_name: trace_kind.trace_kind_name(),
            artifact_name: artifact.artifact_name,
            file_name: artifact.file_name,
            required_for_device_smoke: artifact.required_for_device_smoke,
            events: mobile_runtime_device_smoke_trace_example_events(platform, trace_kind),
        })
}

pub fn mobile_runtime_device_smoke_trace_templates(
    platform: NativeUiPlatform,
) -> Option<Vec<MobileRuntimeDeviceSmokeTrace>> {
    let plan = mobile_runtime_device_smoke_plan(platform)?;
    Some(
        plan.artifact_requirements
            .into_iter()
            .filter_map(|artifact| {
                let trace_kind = mobile_runtime_device_smoke_trace_kind_for_artifact_name(
                    artifact.artifact_name,
                )?;
                Some(MobileRuntimeDeviceSmokeTrace {
                    artifact_source: "device_smoke",
                    platform,
                    platform_name: plan.platform_name,
                    trace_kind,
                    trace_kind_name: trace_kind.trace_kind_name(),
                    artifact_name: artifact.artifact_name,
                    file_name: artifact.file_name,
                    required_for_device_smoke: artifact.required_for_device_smoke,
                    events: mobile_runtime_device_smoke_trace_example_events(platform, trace_kind),
                })
            })
            .collect(),
    )
}

pub fn mobile_runtime_device_smoke_trace_templates_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_device_smoke_trace_templates(platform))
}

pub fn mobile_runtime_device_smoke_trace_template_json(
    platform: NativeUiPlatform,
    trace_kind: MobileRuntimeDeviceSmokeTraceKind,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&mobile_runtime_device_smoke_trace_template(
        platform, trace_kind,
    ))
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
        .map(|requirement| {
            review_mobile_smoke_artifact(&artifact_dir, requirement, plan.platform_name)
        })
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
        NativeUiPlatform::Windows | NativeUiPlatform::Macos | NativeUiPlatform::Linux => None,
    }
}

pub fn mobile_runtime_host_scaffolds() -> Vec<MobileRuntimeHostScaffold> {
    vec![android_activity_host_scaffold()]
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

fn mobile_runtime_bridge_dispatch_operation_name(
    kind: MobileRuntimeBridgeCallbackKind,
) -> &'static str {
    match kind {
        MobileRuntimeBridgeCallbackKind::Bootstrap => {
            NativeRuntimeDriverOperation::StartRuntime.operation_name()
        }
        MobileRuntimeBridgeCallbackKind::Lifecycle => "apply_lifecycle_event",
        MobileRuntimeBridgeCallbackKind::Surface => "bind_or_resize_surface",
        MobileRuntimeBridgeCallbackKind::Input => "dispatch_ui_event",
        MobileRuntimeBridgeCallbackKind::Command => {
            NativeRuntimeDriverOperation::DispatchUiCommand.operation_name()
        }
        MobileRuntimeBridgeCallbackKind::EventPoll => {
            NativeRuntimeDriverOperation::PollApplicationEvent.operation_name()
        }
        MobileRuntimeBridgeCallbackKind::Shutdown => {
            NativeRuntimeDriverOperation::RequestShutdown.operation_name()
        }
    }
}

fn covered_required_callback_kind_names_from_steps(
    dispatch_steps: &[MobileRuntimeBridgeDispatchStep],
) -> Vec<&'static str> {
    REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS
        .iter()
        .filter(|kind| {
            dispatch_steps
                .iter()
                .any(|step| step.required_for_runtime && step.kind == **kind)
        })
        .map(|kind| kind.kind_name())
        .collect()
}

fn missing_required_callback_kind_names_from_steps(
    dispatch_steps: &[MobileRuntimeBridgeDispatchStep],
) -> Vec<&'static str> {
    REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS
        .iter()
        .filter(|kind| {
            !dispatch_steps
                .iter()
                .any(|step| step.required_for_runtime && step.kind == **kind)
        })
        .map(|kind| kind.kind_name())
        .collect()
}

fn dispatch_operation_names_from_steps(
    dispatch_steps: &[MobileRuntimeBridgeDispatchStep],
) -> Vec<&'static str> {
    let mut names = Vec::new();
    for step in dispatch_steps {
        if step.required_for_runtime && !names.contains(&step.dispatch_operation_name) {
            names.push(step.dispatch_operation_name);
        }
    }
    names
}

fn mobile_runtime_bridge_native_driver_operation_names_from_steps(
    dispatch_steps: &[MobileRuntimeBridgeDispatchStep],
) -> Vec<&'static str> {
    let native_names = [
        NativeRuntimeDriverOperation::StartRuntime.operation_name(),
        NativeRuntimeDriverOperation::DispatchUiCommand.operation_name(),
        NativeRuntimeDriverOperation::PollApplicationEvent.operation_name(),
        NativeRuntimeDriverOperation::RequestShutdown.operation_name(),
    ];
    dispatch_operation_names_from_steps(dispatch_steps)
        .into_iter()
        .filter(|name| native_names.contains(name))
        .collect()
}

fn mobile_runtime_bridge_kind_count(
    dispatch_steps: &[MobileRuntimeBridgeDispatchStep],
    kind: MobileRuntimeBridgeCallbackKind,
) -> usize {
    dispatch_steps
        .iter()
        .filter(|step| step.required_for_runtime && step.kind == kind)
        .count()
}

fn write_mobile_json_artifact<T: Serialize>(
    artifact_dir: &Path,
    file_name: &str,
    value: &T,
    written_files: &mut Vec<String>,
) -> ZsuiResult<()> {
    let path = artifact_dir.join(file_name);
    let json = serde_json::to_string_pretty(value).map_err(|err| {
        ZsuiError::host(
            "write_mobile_json_artifact",
            format!("serialize `{file_name}` failed: {err}"),
        )
    })?;
    fs::write(&path, format!("{json}\n"))
        .map_err(|err| mobile_smoke_io_error("write_artifact", &path, err))?;
    written_files.push(path_to_mobile_manifest_string(path));
    Ok(())
}

fn review_mobile_contract_artifact(
    artifact_dir: &Path,
    requirement: &MobileRuntimeBridgeContractArtifactRequirement,
    expected_platform_name: &str,
) -> MobileRuntimeBridgeContractArtifactStatus {
    let path = artifact_dir.join(requirement.file_name);
    let path_string = path_to_mobile_manifest_string(&path);
    let metadata = fs::metadata(&path);
    let exists = metadata.is_ok();
    let byte_len = metadata.ok().map(|metadata| metadata.len());
    let non_empty = byte_len.map(|len| len > 0).unwrap_or(false);
    let mut json_valid = None;
    let mut schema_valid = None;
    let mut validation_error = None;

    if exists && !non_empty {
        validation_error = Some("artifact is empty".to_string());
    }

    if exists && requirement.file_name.ends_with(".json") {
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                Ok(value) => {
                    json_valid = Some(true);
                    match validate_mobile_contract_artifact_schema(
                        requirement,
                        expected_platform_name,
                        &value,
                    ) {
                        Ok(()) => schema_valid = Some(true),
                        Err(err) => {
                            schema_valid = Some(false);
                            validation_error = Some(format!("schema mismatch: {err}"));
                        }
                    }
                }
                Err(err) => {
                    json_valid = Some(false);
                    schema_valid = Some(false);
                    validation_error = Some(format!("invalid json: {err}"));
                }
            },
            Err(err) => {
                json_valid = Some(false);
                schema_valid = Some(false);
                validation_error = Some(format!("read failed: {err}"));
            }
        }
    }

    MobileRuntimeBridgeContractArtifactStatus {
        artifact_name: requirement.artifact_name,
        file_name: requirement.file_name,
        required_for_contract_smoke: requirement.required_for_contract_smoke,
        path: path_string,
        exists,
        byte_len,
        non_empty,
        json_valid,
        schema_valid,
        validation_error,
    }
}

fn validate_mobile_contract_artifact_schema(
    requirement: &MobileRuntimeBridgeContractArtifactRequirement,
    expected_platform_name: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    match requirement.artifact_name {
        "mobile_manifest" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_object(value, "bridge_contract")?;
            require_json_array_non_empty(value, "bridge_entry_points")?;
        }
        "bridge_contract" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_array_non_empty(value, "callbacks")?;
            require_json_array_contains_all_kind_names(
                value,
                "callbacks",
                &mobile_runtime_required_bridge_callback_kind_names(),
            )?;
        }
        "bridge_parity" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_bool(value, "scaffold_matches_contract", true)?;
            require_json_bool(value, "contract_covers_required_runtime_routes", true)?;
            require_json_array_non_empty(value, "required_callback_symbol_names")?;
            require_json_array(value, "pending_ffi_callback_symbol_names")?;
        }
        "bridge_dispatch" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_bool(value, "dispatch_contract_ready", true)?;
            require_json_array_contains_all_strings(
                value,
                "dispatch_operation_names",
                &mobile_runtime_required_bridge_dispatch_operation_names(),
            )?;
        }
        "dispatch_smoke" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_string(value, "smoke_kind_name", "contract_dispatch_smoke")?;
            require_json_bool(value, "contract_smoke_complete", true)?;
            require_json_array_contains_all_strings(
                value,
                "observed_dispatch_operation_names",
                &mobile_runtime_required_bridge_dispatch_operation_names(),
            )?;
        }
        "device_smoke_plan" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_bool_field(value, "device_smoke_ready")?;
            require_json_array_non_empty(value, "artifact_requirements")?;
            require_json_array_contains_file_name(
                value,
                "artifact_requirements",
                "device-window.png",
            )?;
        }
        "agent_context" => {
            require_json_string(
                value,
                "framework_name",
                crate::agent_context::ZSUI_FRAMEWORK_NAME,
            )?;
            let readiness = value
                .get("readiness")
                .ok_or_else(|| "missing `readiness` object".to_string())?;
            require_json_array_contains_all_strings(
                readiness,
                "mobile_runtime_bridge_contract_artifact_file_names",
                &["device-smoke-plan.json", "agent-context.json"],
            )?;
        }
        artifact_name => {
            return Err(format!("unknown contract artifact `{artifact_name}`"));
        }
    }
    Ok(())
}

fn require_json_string(
    value: &serde_json::Value,
    field_name: &str,
    expected: &str,
) -> Result<(), String> {
    match value.get(field_name).and_then(serde_json::Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "`{field_name}` is `{actual}`, expected `{expected}`"
        )),
        None => Err(format!("missing string field `{field_name}`")),
    }
}

fn require_json_bool(
    value: &serde_json::Value,
    field_name: &str,
    expected: bool,
) -> Result<(), String> {
    match value.get(field_name).and_then(serde_json::Value::as_bool) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "`{field_name}` is `{actual}`, expected `{expected}`"
        )),
        None => Err(format!("missing bool field `{field_name}`")),
    }
}

fn require_json_bool_field(value: &serde_json::Value, field_name: &str) -> Result<(), String> {
    value
        .get(field_name)
        .and_then(serde_json::Value::as_bool)
        .map(|_| ())
        .ok_or_else(|| format!("missing bool field `{field_name}`"))
}

fn require_json_object(value: &serde_json::Value, field_name: &str) -> Result<(), String> {
    value
        .get(field_name)
        .and_then(serde_json::Value::as_object)
        .map(|_| ())
        .ok_or_else(|| format!("missing object field `{field_name}`"))
}

fn require_json_array(value: &serde_json::Value, field_name: &str) -> Result<(), String> {
    value
        .get(field_name)
        .and_then(serde_json::Value::as_array)
        .map(|_| ())
        .ok_or_else(|| format!("missing array field `{field_name}`"))
}

fn require_json_array_non_empty(value: &serde_json::Value, field_name: &str) -> Result<(), String> {
    let array = value
        .get(field_name)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("missing array field `{field_name}`"))?;
    if array.is_empty() {
        return Err(format!("array field `{field_name}` is empty"));
    }
    Ok(())
}

fn require_json_array_contains_all_strings(
    value: &serde_json::Value,
    field_name: &str,
    expected_values: &[&str],
) -> Result<(), String> {
    let array = value
        .get(field_name)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("missing array field `{field_name}`"))?;
    let actual_values = array
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let missing_values = expected_values
        .iter()
        .filter(|expected| !actual_values.contains(expected))
        .copied()
        .collect::<Vec<_>>();
    if missing_values.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`{field_name}` is missing values: {}",
            missing_values.join(", ")
        ))
    }
}

fn require_json_array_contains_all_kind_names(
    value: &serde_json::Value,
    field_name: &str,
    expected_kind_names: &[&str],
) -> Result<(), String> {
    let array = value
        .get(field_name)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("missing array field `{field_name}`"))?;
    let actual_kind_names = array
        .iter()
        .filter_map(|entry| entry.get("kind_name"))
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let missing_kind_names = expected_kind_names
        .iter()
        .filter(|expected| !actual_kind_names.contains(expected))
        .copied()
        .collect::<Vec<_>>();
    if missing_kind_names.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`{field_name}` is missing kind names: {}",
            missing_kind_names.join(", ")
        ))
    }
}

fn require_json_array_contains_file_name(
    value: &serde_json::Value,
    field_name: &str,
    expected_file_name: &str,
) -> Result<(), String> {
    let array = value
        .get(field_name)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("missing array field `{field_name}`"))?;
    let contains_file_name = array.iter().any(|entry| {
        entry.get("file_name").and_then(serde_json::Value::as_str) == Some(expected_file_name)
    });
    if contains_file_name {
        Ok(())
    } else {
        Err(format!(
            "`{field_name}` is missing file `{expected_file_name}`"
        ))
    }
}

fn mobile_smoke_io_error(operation: &str, path: &Path, err: std::io::Error) -> ZsuiError {
    ZsuiError::host(
        operation,
        format!("{}: {err}", path_to_mobile_manifest_string(path)),
    )
}

fn contract_callback_kind_names(callbacks: &[MobileRuntimeBridgeCallback]) -> Vec<&'static str> {
    REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS
        .iter()
        .filter(|kind| {
            callbacks
                .iter()
                .any(|callback| callback.required_for_runtime && callback.kind == **kind)
        })
        .map(|kind| kind.kind_name())
        .collect()
}

fn missing_required_callback_kind_names(
    callbacks: &[MobileRuntimeBridgeCallback],
) -> Vec<&'static str> {
    REQUIRED_MOBILE_RUNTIME_BRIDGE_CALLBACK_KINDS
        .iter()
        .filter(|kind| {
            !callbacks
                .iter()
                .any(|callback| callback.required_for_runtime && callback.kind == **kind)
        })
        .map(|kind| kind.kind_name())
        .collect()
}

fn mobile_runtime_device_smoke_trace_kind_for_artifact_name(
    artifact_name: &str,
) -> Option<MobileRuntimeDeviceSmokeTraceKind> {
    match artifact_name {
        "lifecycle_trace" => Some(MobileRuntimeDeviceSmokeTraceKind::Lifecycle),
        "surface_trace" => Some(MobileRuntimeDeviceSmokeTraceKind::Surface),
        "input_trace" => Some(MobileRuntimeDeviceSmokeTraceKind::Input),
        "clipboard_trace" => Some(MobileRuntimeDeviceSmokeTraceKind::Clipboard),
        _ => None,
    }
}

fn mobile_runtime_device_smoke_trace_example_events(
    platform: NativeUiPlatform,
    trace_kind: MobileRuntimeDeviceSmokeTraceKind,
) -> Vec<&'static str> {
    match (platform, trace_kind) {
        (NativeUiPlatform::Android, MobileRuntimeDeviceSmokeTraceKind::Lifecycle) => {
            vec![
                "onCreate",
                "onStart",
                "onResume",
                "onPause",
                "onStop",
                "onDestroy",
            ]
        }
        (NativeUiPlatform::Android, MobileRuntimeDeviceSmokeTraceKind::Surface) => {
            vec!["surfaceCreated", "surfaceChanged", "surfaceDestroyed"]
        }
        (_, MobileRuntimeDeviceSmokeTraceKind::Input) => {
            vec!["focus", "pointer_down", "pointer_up", "key_input"]
        }
        (NativeUiPlatform::Android, MobileRuntimeDeviceSmokeTraceKind::Clipboard) => {
            vec!["set_primary_clip", "read_primary_clip"]
        }
        _ => Vec::new(),
    }
}

fn validate_mobile_device_smoke_artifact_schema(
    requirement: &MobileRuntimeDeviceSmokeArtifact,
    expected_platform_name: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    match requirement.artifact_name {
        "mobile_manifest" => {
            require_json_string(value, "platform_name", expected_platform_name)?;
            require_json_object(value, "bridge_contract")?;
            require_json_array_non_empty(value, "bridge_entry_points")?;
        }
        "lifecycle_trace" => {
            require_mobile_device_trace_schema(value, expected_platform_name, "lifecycle")?;
        }
        "surface_trace" => {
            require_mobile_device_trace_schema(value, expected_platform_name, "surface")?;
        }
        "input_trace" => {
            require_mobile_device_trace_schema(value, expected_platform_name, "input")?;
        }
        "clipboard_trace" => {
            require_mobile_device_trace_schema(value, expected_platform_name, "clipboard")?;
        }
        artifact_name => {
            return Err(format!("unknown device smoke artifact `{artifact_name}`"));
        }
    }
    Ok(())
}

fn require_mobile_device_trace_schema(
    value: &serde_json::Value,
    expected_platform_name: &str,
    trace_kind: &str,
) -> Result<(), String> {
    require_json_string(value, "artifact_source", "device_smoke")?;
    require_json_string(value, "platform_name", expected_platform_name)?;
    require_json_string(value, "trace_kind", trace_kind)?;
    require_json_array_non_empty(value, "events")
}

fn review_mobile_smoke_artifact(
    artifact_dir: &Path,
    requirement: &MobileRuntimeDeviceSmokeArtifact,
    expected_platform_name: &str,
) -> MobileRuntimeDeviceSmokeArtifactStatus {
    let path = artifact_dir.join(requirement.file_name);
    let path_string = path_to_mobile_manifest_string(&path);
    let metadata = fs::metadata(&path);
    let exists = metadata.is_ok();
    let byte_len = metadata.ok().map(|metadata| metadata.len());
    let non_empty = byte_len.map(|len| len > 0).unwrap_or(false);
    let mut json_valid = None;
    let mut schema_valid = None;
    let mut png_valid = None;
    let mut validation_error = None;

    if exists && !non_empty {
        validation_error = Some("artifact is empty".to_string());
    }

    if exists && requirement.file_name.ends_with(".json") {
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                Ok(value) => {
                    json_valid = Some(true);
                    match validate_mobile_device_smoke_artifact_schema(
                        requirement,
                        expected_platform_name,
                        &value,
                    ) {
                        Ok(()) => schema_valid = Some(true),
                        Err(err) => {
                            schema_valid = Some(false);
                            validation_error = Some(format!("schema mismatch: {err}"));
                        }
                    }
                }
                Err(err) => {
                    json_valid = Some(false);
                    schema_valid = Some(false);
                    validation_error = Some(format!("invalid json: {err}"));
                }
            },
            Err(err) => {
                json_valid = Some(false);
                schema_valid = Some(false);
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
        schema_valid,
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
    fn mobile_scaffold_exists_for_android() {
        let scaffolds = mobile_runtime_host_scaffolds();

        assert_eq!(scaffolds.len(), 1);
        assert_eq!(scaffolds[0].platform_name, "android");
        assert!(mobile_runtime_host_scaffold(NativeUiPlatform::Windows).is_none());
        assert!(
            mobile_runtime_host_scaffold_module_paths().contains(&"src/android_activity_host.rs")
        );
    }

    #[test]
    fn mobile_scaffolds_name_pending_platform_bindings() {
        let android = mobile_runtime_host_scaffold(NativeUiPlatform::Android)
            .expect("android scaffold should exist");

        assert!(android.pending_capability_names().contains(&"main_window"));
        assert!(android
            .capability_binding_for(NativeUiAdapterCapability::MainWindow)
            .expect("android main window binding should exist")
            .platform_binding_name
            .contains("Activity"));
        assert!(android.implemented_capability_names().is_empty());
    }

    #[test]
    fn mobile_bridge_contracts_name_required_runtime_callbacks_and_device_artifacts() {
        let android = mobile_runtime_bridge_contract(NativeUiPlatform::Android)
            .expect("android bridge contract should exist");

        assert!(android
            .required_callback_symbol_names()
            .contains(&"zsui_android_activity_lifecycle"));
        assert!(android
            .required_callback_symbol_names()
            .contains(&"zsui_android_activity_surface_created"));
        assert!(android
            .required_device_smoke_file_names()
            .contains(&"lifecycle.json"));
        assert!(mobile_runtime_bridge_callback_symbol_names()
            .contains(&"zsui_android_activity_dispatch_ui_event"));
        assert!(mobile_runtime_device_smoke_artifact_names().contains(&"device-window.png"));
    }

    #[test]
    fn mobile_bridge_parity_reports_contract_coverage_without_claiming_runtime() {
        let reports = mobile_runtime_bridge_parity_reports();
        let android = mobile_runtime_bridge_parity_report(NativeUiPlatform::Android)
            .expect("android parity report should exist");

        assert_eq!(reports.len(), 1);
        assert!(android.scaffold_matches_contract);
        assert!(android.contract_covers_required_runtime_routes);
        assert!(android.missing_required_callback_kind_names.is_empty());
        assert!(!android.runtime_ffi_implemented);
        assert!(!android.ready_for_device_smoke);
        assert!(android.contract_callback_kind_names.contains(&"surface"));
        assert!(android
            .pending_ffi_callback_symbol_names
            .contains(&"zsui_android_activity_dispatch_ui_event"));
        assert!(android
            .required_device_smoke_file_names
            .contains(&"device-window.png"));
        assert!(mobile_runtime_required_bridge_callback_kind_names().contains(&"event_poll"));
        assert!(mobile_runtime_device_smoke_command_names()
            .contains(&"mobile_scaffold_manifest --parity"));
    }

    #[test]
    fn mobile_bridge_dispatch_report_maps_callbacks_to_runtime_operations() {
        let android = mobile_runtime_bridge_dispatch_report(NativeUiPlatform::Android)
            .expect("android dispatch report should exist");

        assert!(android.dispatch_contract_ready);
        assert!(!android.runtime_ffi_implemented);
        assert!(!android.device_smoke_ready);
        assert_eq!(
            android.missing_required_callback_kind_names,
            Vec::<&str>::new()
        );
        assert_eq!(android.covered_required_callback_kind_names.len(), 7);
        assert!(android
            .dispatch_operation_names
            .contains(&"apply_lifecycle_event"));
        assert!(android
            .dispatch_operation_names
            .contains(&"bind_or_resize_surface"));
        assert!(android
            .dispatch_operation_names
            .contains(&"dispatch_ui_event"));
        assert!(android
            .native_runtime_driver_operation_names
            .contains(&"start_runtime"));
        assert!(android
            .native_runtime_driver_operation_names
            .contains(&"dispatch_ui_command"));
        assert!(android
            .native_runtime_driver_operation_names
            .contains(&"poll_application_event"));
        assert!(android
            .native_runtime_driver_operation_names
            .contains(&"request_shutdown"));
        assert!(android.dispatch_steps.iter().any(|step| step.symbol_name
            == "zsui_android_activity_dispatch_ui_event"
            && step.dispatch_operation_name == "dispatch_ui_event"));
        assert!(mobile_runtime_bridge_dispatch_reports_json()
            .expect("dispatch reports should serialize")
            .contains("dispatch_contract_ready"));
        assert!(mobile_runtime_device_smoke_command_names()
            .contains(&"mobile_scaffold_manifest --dispatch"));
    }

    #[test]
    fn mobile_bridge_contract_smoke_replays_dispatch_steps_without_device_claims() {
        let android = mobile_runtime_bridge_contract_smoke_report(NativeUiPlatform::Android)
            .expect("android contract smoke report should exist");

        assert_eq!(android.smoke_kind_name, "contract_dispatch_smoke");
        assert!(android.contract_smoke_complete);
        assert!(!android.runtime_ffi_implemented);
        assert!(!android.device_smoke_ready);
        assert_eq!(android.accepted_step_count, android.required_step_count);
        assert!(android.missing_dispatch_operation_names.is_empty());
        assert_eq!(android.lifecycle_event_count, 1);
        assert_eq!(android.surface_event_count, 3);
        assert_eq!(android.input_event_count, 1);
        assert_eq!(android.command_event_count, 1);
        assert_eq!(android.event_poll_count, 1);
        assert_eq!(android.shutdown_count, 1);
        assert!(android
            .required_dispatch_operation_names
            .contains(&"dispatch_ui_event"));
        assert!(android
            .native_runtime_driver_operation_names
            .contains(&"start_runtime"));
        assert!(mobile_runtime_bridge_contract_smoke_reports_json()
            .expect("contract smoke reports should serialize")
            .contains("contract_dispatch_smoke"));
        assert!(mobile_runtime_device_smoke_command_names()
            .contains(&"mobile_scaffold_manifest --dispatch-smoke"));
    }

    #[test]
    fn mobile_bridge_contract_artifact_writer_does_not_fake_device_smoke() {
        let root = unique_mobile_test_root("contract-writer");
        let report =
            write_mobile_runtime_bridge_contract_artifacts_to(NativeUiPlatform::Android, &root)
                .expect("contract artifacts should write");

        assert!(report.contract_artifacts_complete);
        assert!(!report.device_smoke_complete);
        assert_eq!(report.written_files.len(), 7);
        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("manifest.json")));
        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("dispatch-smoke.json")));
        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("device-smoke-plan.json")));
        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("agent-context.json")));
        assert!(!report
            .missing_required_device_artifacts
            .contains(&"manifest.json".to_string()));
        assert!(report
            .missing_required_device_artifacts
            .contains(&"device-window.png".to_string()));

        let contract_review =
            review_mobile_runtime_bridge_contract_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("contract review should inspect contract artifacts");
        assert!(contract_review.contract_artifacts_complete);
        assert!(!contract_review.device_smoke_proof_claimed);
        assert_eq!(
            contract_review.valid_required_artifact_count,
            contract_review.required_artifact_count
        );
        assert!(contract_review.artifact_statuses.iter().all(|artifact| {
            artifact.json_valid == Some(true) && artifact.schema_valid == Some(true)
        }));
        assert!(contract_review.missing_required_artifacts.is_empty());
        assert!(contract_review.invalid_required_artifacts.is_empty());

        let review =
            review_mobile_runtime_device_smoke_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("review should inspect contract artifacts");
        assert!(!review.device_smoke_complete);
        assert_eq!(review.present_required_artifact_count, 1);
        assert!(review
            .missing_required_artifacts
            .contains(&"device-window.png".to_string()));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_bridge_contract_artifact_review_rejects_schema_mismatch() {
        let root = unique_mobile_test_root("contract-schema-mismatch");
        write_mobile_runtime_bridge_contract_artifacts_to(NativeUiPlatform::Android, &root)
            .expect("contract artifacts should write");

        let agent_context_path = root.join("android").join("agent-context.json");
        fs::write(&agent_context_path, "{\"framework_name\":\"wrong\"}\n")
            .expect("test should corrupt agent context artifact");

        let report =
            review_mobile_runtime_bridge_contract_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("contract review should inspect corrupted artifact");

        assert!(!report.contract_artifacts_complete);
        assert_eq!(
            report.invalid_required_artifacts,
            vec!["agent-context.json"]
        );
        let agent_context_status = report
            .artifact_statuses
            .iter()
            .find(|artifact| artifact.file_name == "agent-context.json")
            .expect("agent context artifact should be reviewed");
        assert_eq!(agent_context_status.json_valid, Some(true));
        assert_eq!(agent_context_status.schema_valid, Some(false));
        assert!(agent_context_status
            .validation_error
            .as_deref()
            .unwrap_or_default()
            .contains("schema mismatch"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_bridge_contract_artifact_writer_and_review_support_all_platforms() {
        let root = unique_mobile_test_root("contract-all");
        let write_reports = write_mobile_runtime_bridge_contract_artifacts_for_all_to(&root)
            .expect("all mobile contract artifacts should write");

        assert_eq!(write_reports.len(), 1);
        assert!(write_reports
            .iter()
            .all(|report| report.contract_artifacts_complete));
        assert!(write_reports
            .iter()
            .all(|report| !report.device_smoke_complete));
        assert!(write_reports
            .iter()
            .any(|report| report.platform == NativeUiPlatform::Android));

        let review_reports = review_mobile_runtime_bridge_contract_artifacts_for_all_at(&root)
            .expect("all mobile contract artifacts should review");
        assert_eq!(review_reports.len(), 1);
        assert!(review_reports
            .iter()
            .all(|report| report.contract_artifacts_complete));
        assert!(review_reports
            .iter()
            .all(|report| !report.device_smoke_proof_claimed));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_bridge_contract_artifact_review_reports_missing_contract_files() {
        let root = unique_mobile_test_root("contract-review-missing");
        let report =
            review_mobile_runtime_bridge_contract_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("contract review should report missing files");

        assert!(!report.contract_artifacts_complete);
        assert!(!report.device_smoke_proof_claimed);
        assert_eq!(report.present_required_artifact_count, 0);
        assert!(report
            .missing_required_artifacts
            .contains(&"bridge-contract.json".to_string()));
        assert!(
            mobile_runtime_bridge_contract_artifact_file_names().contains(&"dispatch-smoke.json")
        );
        assert!(
            mobile_runtime_bridge_contract_artifact_file_names().contains(&"agent-context.json")
        );
        assert!(mobile_runtime_device_smoke_command_names()
            .contains(&"mobile_scaffold_manifest --review-contract"));

        let _ = fs::remove_dir_all(root);
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
    fn mobile_device_smoke_trace_templates_match_review_schema() {
        let android_templates =
            mobile_runtime_device_smoke_trace_templates(NativeUiPlatform::Android)
                .expect("android trace templates should exist");

        assert!(android_templates.iter().any(|template| {
            template.file_name == "lifecycle.json"
                && template.artifact_source == "device_smoke"
                && template.required_for_device_smoke
                && template.events.contains(&"onCreate")
        }));
        assert!(android_templates.iter().any(|template| {
            template.file_name == "clipboard.json" && !template.required_for_device_smoke
        }));
        assert!(mobile_runtime_device_smoke_trace_template_json(
            NativeUiPlatform::Android,
            MobileRuntimeDeviceSmokeTraceKind::Input,
        )
        .expect("trace template should serialize")
        .contains("input.json"));
        assert!(mobile_runtime_device_smoke_command_names()
            .contains(&"mobile_scaffold_manifest --trace-template"));
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
        let dir = root.join("android");
        fs::create_dir_all(&dir).expect("test artifact dir should be creatable");
        let manifest = serde_json::to_string_pretty(
            &mobile_runtime_host_scaffold(NativeUiPlatform::Android)
                .expect("android scaffold should exist"),
        )
        .expect("android scaffold should serialize");
        write_text(&dir.join("manifest.json"), &manifest);
        write_text(&dir.join("device-launch.log"), "launched");
        write_png_header(&dir.join("device-window.png"));
        write_device_trace(
            &dir.join("lifecycle.json"),
            "android",
            "lifecycle",
            "onCreate",
        );
        write_device_trace(
            &dir.join("surface.json"),
            "android",
            "surface",
            "surfaceCreated",
        );
        write_device_trace(&dir.join("input.json"), "android", "input", "touch");

        let report =
            review_mobile_runtime_device_smoke_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("mobile device smoke review should inspect artifacts");

        assert!(report.device_smoke_complete);
        assert_eq!(
            report.valid_required_artifact_count,
            report.required_artifact_count
        );
        assert!(report.artifact_statuses.iter().all(|artifact| {
            !artifact.exists
                || !artifact.file_name.ends_with(".json")
                || (artifact.json_valid == Some(true) && artifact.schema_valid == Some(true))
        }));
        assert!(report.missing_required_artifacts.is_empty());
        assert!(report.invalid_required_artifacts.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_device_smoke_review_rejects_contract_only_json_as_device_proof() {
        let root = unique_mobile_test_root("device-schema-mismatch");
        let dir = root.join("android");
        fs::create_dir_all(&dir).expect("test artifact dir should be creatable");
        let manifest = serde_json::to_string_pretty(
            &mobile_runtime_host_scaffold(NativeUiPlatform::Android)
                .expect("android scaffold should exist"),
        )
        .expect("android scaffold should serialize");
        write_text(&dir.join("manifest.json"), &manifest);
        write_text(&dir.join("device-launch.log"), "launched");
        write_png_header(&dir.join("device-window.png"));
        write_text(&dir.join("lifecycle.json"), "{\"events\":[\"onCreate\"]}");
        write_device_trace(
            &dir.join("surface.json"),
            "android",
            "surface",
            "surfaceCreated",
        );
        write_device_trace(&dir.join("input.json"), "android", "input", "touch");

        let report =
            review_mobile_runtime_device_smoke_artifacts_at(NativeUiPlatform::Android, &root)
                .expect("mobile device smoke review should inspect artifacts");

        assert!(!report.device_smoke_complete);
        assert_eq!(report.invalid_required_artifacts, vec!["lifecycle.json"]);
        let lifecycle = report
            .artifact_statuses
            .iter()
            .find(|artifact| artifact.file_name == "lifecycle.json")
            .expect("lifecycle artifact should be reviewed");
        assert_eq!(lifecycle.json_valid, Some(true));
        assert_eq!(lifecycle.schema_valid, Some(false));
        assert!(lifecycle
            .validation_error
            .as_deref()
            .unwrap_or_default()
            .contains("artifact_source"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mobile_scaffold_json_serializes_for_ai_context() {
        let json = mobile_runtime_host_scaffolds_json().expect("scaffolds should serialize");

        assert!(json.contains("android_activity"));
        assert!(json.contains("device smoke artifacts"));
        assert!(json.contains("zsui_android_activity_surface_created"));

        let parity =
            mobile_runtime_bridge_parity_reports_json().expect("parity reports should serialize");
        assert!(parity.contains("pending_ffi_callback_symbol_names"));
        assert!(parity.contains("contract_covers_required_runtime_routes"));

        let smoke = mobile_runtime_bridge_contract_smoke_reports_json()
            .expect("contract smoke reports should serialize");
        assert!(smoke.contains("contract_smoke_complete"));
        assert!(smoke.contains("dispatch_ui_command"));
    }

    fn unique_mobile_test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zsui-mobile-smoke-{name}-{}", unix_ms_now()))
    }

    fn write_text(path: &Path, contents: &str) {
        fs::write(path, contents).expect("test text artifact should write");
    }

    fn write_device_trace(path: &Path, platform_name: &str, trace_kind: &str, event_name: &str) {
        let json = serde_json::json!({
            "artifact_source": "device_smoke",
            "platform_name": platform_name,
            "trace_kind": trace_kind,
            "events": [event_name],
        });
        write_text(
            path,
            &serde_json::to_string_pretty(&json).expect("device trace should serialize"),
        );
    }

    fn write_png_header(path: &Path) {
        let mut file = fs::File::create(path).expect("test png artifact should write");
        file.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])
            .expect("test png header should write");
    }
}
