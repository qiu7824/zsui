use serde::Serialize;

use crate::app::zsui_declaration_audit_surface_names;
use crate::geometry::SHARED_NON_HOST_UI_PROTOCOLS;
use crate::mobile_host::mobile_runtime_host_scaffold_module_paths;
use crate::native_adapter_manifest::{
    native_ui_backend_capability_matrix, native_ui_backend_capability_matrix_for_platform,
    NativeUiBackendStatus, NativeUiPlatform, REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES,
    SUPPORTED_NATIVE_UI_PLATFORMS, SUPPORTED_NATIVE_UI_TOOLKITS,
};
use crate::native_hosts::{
    required_native_runtime_driver_operation_names,
    required_native_settings_page_model_host_operation_names,
    required_native_status_item_host_operation_names,
};
use crate::native_smoke::{native_host_smoke_artifact_names, native_host_smoke_command_names};
use crate::product_adapter::{
    product_adapter_reuse_checklist, product_adapter_runtime_smoke_example_names,
    zsui_reusable_runtime_harness_stage_names,
};
use crate::ui_surface_protocol::REQUIRED_UI_HOST_SURFACES;

pub const ZSUI_FRAMEWORK_NAME: &str = "zsui";
pub const ZSUI_AGENT_CONTEXT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ZsuiFrameworkLayer {
    CoreContracts,
    DeclarationApi,
    SharedProtocols,
    AdapterBoundary,
    NativeHost,
    ProductAdapterBoundary,
}

impl ZsuiFrameworkLayer {
    pub const fn layer_name(self) -> &'static str {
        match self {
            Self::CoreContracts => "core_contracts",
            Self::DeclarationApi => "declaration_api",
            Self::SharedProtocols => "shared_protocols",
            Self::AdapterBoundary => "adapter_boundary",
            Self::NativeHost => "native_host",
            Self::ProductAdapterBoundary => "product_adapter_boundary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiFrameworkBoundaryRule {
    pub layer: ZsuiFrameworkLayer,
    pub layer_name: &'static str,
    pub owner_name: &'static str,
    pub allowed_modules: Vec<&'static str>,
    pub owns: Vec<&'static str>,
    pub must_not_own: Vec<&'static str>,
    pub handoff_to: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiCompletionArea {
    pub area_name: &'static str,
    pub percent_complete: u8,
    pub status_name: &'static str,
    pub source_path: &'static str,
    pub missing_before_complete: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiReuseReadinessReport {
    pub platform_names: Vec<&'static str>,
    pub toolkit_names: Vec<&'static str>,
    pub declaration_audit_surface_names: Vec<&'static str>,
    pub native_runtime_ready_platforms: Vec<&'static str>,
    pub first_pass_native_host_platforms: Vec<&'static str>,
    pub scaffold_platforms: Vec<&'static str>,
    pub native_adapter_capability_names: Vec<&'static str>,
    pub required_host_surface_names: Vec<&'static str>,
    pub shared_non_host_protocol_names: Vec<&'static str>,
    pub native_runtime_driver_operation_names: Vec<&'static str>,
    pub native_status_item_host_operation_names: Vec<&'static str>,
    pub native_settings_page_model_host_operation_names: Vec<&'static str>,
    pub runtime_harness_stage_names: Vec<&'static str>,
    pub product_adapter_surface_names: Vec<&'static str>,
    pub product_adapter_task_names: Vec<&'static str>,
    pub product_adapter_smoke_example_names: Vec<&'static str>,
    pub ai_provider_family_names: Vec<&'static str>,
    pub ai_executor_boundary_names: Vec<&'static str>,
    pub native_smoke_artifact_names: Vec<&'static str>,
    pub native_smoke_command_names: Vec<&'static str>,
    pub mobile_runtime_host_scaffold_module_paths: Vec<&'static str>,
    pub agent_skill_path: &'static str,
    pub docs_paths: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiNativeRuntimeGatePlan {
    pub gate_name: &'static str,
    pub required_adapter_capability_names: Vec<&'static str>,
    pub required_host_surface_names: Vec<&'static str>,
    pub required_shared_protocol_names: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiNativeRuntimeGateCompletion {
    pub gate_names: Vec<&'static str>,
    pub missing_gate_names: Vec<&'static str>,
    pub next_gate_name: Option<&'static str>,
    pub total_gate_count: usize,
    pub completed_gate_count: usize,
    pub missing_gate_count: usize,
    pub completion_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiReuseBootstrapPlan {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub backend_status: NativeUiBackendStatus,
    pub backend_status_name: &'static str,
    pub adapter_boundary: &'static str,
    pub adapter_module_path: &'static str,
    pub native_adapter_capability_names: Vec<&'static str>,
    pub platform_binding_names: Vec<&'static str>,
    pub native_runtime_gate_names: Vec<&'static str>,
    pub missing_native_runtime_gate_names: Vec<&'static str>,
    pub next_native_runtime_gate_name: Option<&'static str>,
    pub native_runtime_gate_plans: Vec<ZsuiNativeRuntimeGatePlan>,
    pub native_runtime_gate_completion: ZsuiNativeRuntimeGateCompletion,
}

impl ZsuiReuseBootstrapPlan {
    pub const fn native_runtime_ready(&self) -> bool {
        self.backend_status.is_native_runtime_ready()
    }

    pub const fn scaffolded(&self) -> bool {
        self.backend_status.is_scaffold()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiAgentIntegrationStep {
    pub step_name: &'static str,
    pub owner_name: &'static str,
    pub required_names: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiAgentContext {
    pub framework_name: &'static str,
    pub context_version: &'static str,
    pub framework_layers: Vec<ZsuiFrameworkLayer>,
    pub boundary_rules: Vec<ZsuiFrameworkBoundaryRule>,
    pub completion_areas: Vec<ZsuiCompletionArea>,
    pub readiness: ZsuiReuseReadinessReport,
    pub platform_bootstrap: Vec<ZsuiReuseBootstrapPlan>,
    pub runtime_gate_plans: Vec<ZsuiNativeRuntimeGatePlan>,
    pub integration_steps: Vec<ZsuiAgentIntegrationStep>,
}

pub fn zsui_framework_layers() -> Vec<ZsuiFrameworkLayer> {
    vec![
        ZsuiFrameworkLayer::CoreContracts,
        ZsuiFrameworkLayer::DeclarationApi,
        ZsuiFrameworkLayer::SharedProtocols,
        ZsuiFrameworkLayer::AdapterBoundary,
        ZsuiFrameworkLayer::NativeHost,
        ZsuiFrameworkLayer::ProductAdapterBoundary,
    ]
}

pub fn zsui_framework_boundary_rules() -> Vec<ZsuiFrameworkBoundaryRule> {
    use ZsuiFrameworkLayer::{
        AdapterBoundary, CoreContracts, DeclarationApi, NativeHost, ProductAdapterBoundary,
        SharedProtocols,
    };

    vec![
        ZsuiFrameworkBoundaryRule {
            layer: CoreContracts,
            layer_name: CoreContracts.layer_name(),
            owner_name: "zsui_core_contracts",
            allowed_modules: vec!["src/core.rs", "src/capability.rs", "src/host.rs"],
            owns: vec!["stable ids", "errors", "host trait", "capability reporting"],
            must_not_own: vec![
                "platform handles",
                "product database access",
                "AI provider clients",
            ],
            handoff_to: vec!["declaration_api", "shared_protocols", "native_host"],
        },
        ZsuiFrameworkBoundaryRule {
            layer: DeclarationApi,
            layer_name: DeclarationApi.layer_name(),
            owner_name: "zsui_declarations",
            allowed_modules: vec![
                "src/app.rs",
                "src/window.rs",
                "src/tray.rs",
                "src/menu.rs",
                "src/hotkey.rs",
                "src/settings.rs",
                "src/clipboard.rs",
            ],
            owns: vec![
                "windows",
                "tray/status menus",
                "menus",
                "hotkeys",
                "settings specs",
            ],
            must_not_own: vec!["native widgets", "message loops", "product side effects"],
            handoff_to: vec!["native_host", "product_adapter_boundary"],
        },
        ZsuiFrameworkBoundaryRule {
            layer: SharedProtocols,
            layer_name: SharedProtocols.layer_name(),
            owner_name: "zsui_shared_protocols",
            allowed_modules: vec![
                "src/geometry.rs",
                "src/command_protocol.rs",
                "src/event_protocol.rs",
                "src/component_protocol.rs",
                "src/control_protocol.rs",
                "src/render_protocol.rs",
                "src/ui_surface_protocol.rs",
                "src/timer_protocol.rs",
            ],
            owns: vec![
                "geometry",
                "commands",
                "events",
                "components",
                "control specs",
                "render traits",
            ],
            must_not_own: vec!["OS windows", "clipboard writes", "file system dialogs"],
            handoff_to: vec!["adapter_boundary", "native_host"],
        },
        ZsuiFrameworkBoundaryRule {
            layer: AdapterBoundary,
            layer_name: AdapterBoundary.layer_name(),
            owner_name: "zsui_adapter_boundary",
            allowed_modules: vec![
                "src/native_adapter_manifest.rs",
                "src/native_host_launch.rs",
            ],
            owns: vec![
                "backend descriptors",
                "toolkit names",
                "capability matrix",
                "launch plans",
            ],
            must_not_own: vec!["real event-loop side effects", "product command execution"],
            handoff_to: vec!["native_host"],
        },
        ZsuiFrameworkBoundaryRule {
            layer: NativeHost,
            layer_name: NativeHost.layer_name(),
            owner_name: "zsui_native_host",
            allowed_modules: vec![
                "src/native.rs",
                "src/native_hosts.rs",
                "src/host_protocol.rs",
            ],
            owns: vec![
                "native window runtime",
                "native host traits",
                "platform service contracts",
            ],
            must_not_own: vec!["product storage", "sync", "product AI execution"],
            handoff_to: vec!["product_adapter_boundary"],
        },
        ZsuiFrameworkBoundaryRule {
            layer: ProductAdapterBoundary,
            layer_name: ProductAdapterBoundary.layer_name(),
            owner_name: "application_crate",
            allowed_modules: vec!["outside_zsui_crate"],
            owns: vec![
                "domain state",
                "persistence",
                "settings storage",
                "AI/tool execution",
            ],
            must_not_own: vec!["ZSUI framework internals", "generic platform binding names"],
            handoff_to: vec!["zsui_host_contracts"],
        },
    ]
}

pub fn zsui_completion_areas() -> Vec<ZsuiCompletionArea> {
    vec![
        ZsuiCompletionArea {
            area_name: "foundation_contracts",
            percent_complete: 70,
            status_name: "code_level_ready",
            source_path: "src/core.rs",
            missing_before_complete: vec!["broader examples", "stable semver policy"],
        },
        ZsuiCompletionArea {
            area_name: "declaration_api",
            percent_complete: 72,
            status_name: "declaration_audit_ready",
            source_path: "src/app.rs",
            missing_before_complete: vec![
                "theming API",
                "native component bindings",
                "layout measurement",
                "full menu/settings native binding",
            ],
        },
        ZsuiCompletionArea {
            area_name: "minimal_native_window_runtime",
            percent_complete: 45,
            status_name: "status_settings_operation_surface_ready",
            source_path: "src/native.rs",
            missing_before_complete: vec![
                "real native menus",
                "dialogs",
                "clipboard",
                "captured target smoke artifacts",
            ],
        },
        ZsuiCompletionArea {
            area_name: "full_desktop_native_hosts",
            percent_complete: 15,
            status_name: "extraction_in_progress",
            source_path: "src/native_hosts.rs",
            missing_before_complete: vec![
                "Win32 split",
                "AppKit split",
                "GTK split",
                "target smoke artifacts",
            ],
        },
        ZsuiCompletionArea {
            area_name: "android_and_harmony",
            percent_complete: 10,
            status_name: "host_scaffold_manifest_ready",
            source_path: "src/mobile_host.rs",
            missing_before_complete: vec![
                "Android Activity FFI bridge",
                "Harmony Ability FFI bridge",
                "device smoke artifacts",
            ],
        },
        ZsuiCompletionArea {
            area_name: "product_adapter_runtime_harness",
            percent_complete: 57,
            status_name: "native_operation_projection_ready",
            source_path: "src/product_adapter.rs",
            missing_before_complete: vec![
                "real product integration",
                "target host smoke through harness",
                "native driver target artifacts",
            ],
        },
        ZsuiCompletionArea {
            area_name: "native_smoke_verification",
            percent_complete: 55,
            status_name: "windows_target_smoke_verified",
            source_path: "src/native_smoke.rs",
            missing_before_complete: vec![
                "macOS/Linux screenshot capture",
                "macOS/Linux target smoke artifacts",
                "device artifact review",
            ],
        },
    ]
}

pub fn zsui_reuse_readiness_report() -> ZsuiReuseReadinessReport {
    let matrix = native_ui_backend_capability_matrix();
    let product_adapter = product_adapter_reuse_checklist();

    ZsuiReuseReadinessReport {
        platform_names: SUPPORTED_NATIVE_UI_PLATFORMS
            .iter()
            .map(|platform| platform.platform_name())
            .collect(),
        toolkit_names: SUPPORTED_NATIVE_UI_TOOLKITS
            .iter()
            .map(|toolkit| toolkit.toolkit_name())
            .collect(),
        declaration_audit_surface_names: zsui_declaration_audit_surface_names(),
        native_runtime_ready_platforms: matrix
            .iter()
            .filter(|entry| entry.backend.status.is_native_runtime_ready())
            .map(|entry| entry.backend.platform_name())
            .collect(),
        first_pass_native_host_platforms: matrix
            .iter()
            .filter(|entry| entry.backend.status.is_first_pass_native_host())
            .map(|entry| entry.backend.platform_name())
            .collect(),
        scaffold_platforms: matrix
            .iter()
            .filter(|entry| entry.backend.status.is_scaffold())
            .map(|entry| entry.backend.platform_name())
            .collect(),
        native_adapter_capability_names: REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES
            .iter()
            .map(|capability| capability.capability_name())
            .collect(),
        required_host_surface_names: REQUIRED_UI_HOST_SURFACES
            .iter()
            .map(|surface| surface.adapter_name())
            .collect(),
        shared_non_host_protocol_names: SHARED_NON_HOST_UI_PROTOCOLS
            .iter()
            .map(|protocol| protocol.protocol_name())
            .collect(),
        native_runtime_driver_operation_names: required_native_runtime_driver_operation_names(),
        native_status_item_host_operation_names: required_native_status_item_host_operation_names(),
        native_settings_page_model_host_operation_names:
            required_native_settings_page_model_host_operation_names(),
        runtime_harness_stage_names: zsui_reusable_runtime_harness_stage_names(),
        product_adapter_surface_names: product_adapter.surface_names,
        product_adapter_task_names: product_adapter.task_names,
        product_adapter_smoke_example_names: product_adapter_runtime_smoke_example_names(),
        ai_provider_family_names: product_adapter.ai_provider_family_names,
        ai_executor_boundary_names: product_adapter.ai_executor_boundary_names,
        native_smoke_artifact_names: native_host_smoke_artifact_names(),
        native_smoke_command_names: native_host_smoke_command_names(),
        mobile_runtime_host_scaffold_module_paths: mobile_runtime_host_scaffold_module_paths(),
        agent_skill_path: "docs/skills/zsui-native-ui/",
        docs_paths: vec![
            "README.md",
            "docs/ai-agent.md",
            "docs/architecture.md",
            "docs/porting.md",
            "docs/native-host-smoke.md",
            "docs/skills/zsui-native-ui/SKILL.md",
            "docs/skills/zsui-native-ui/references/native-ui-entrypoints.md",
            "src/mobile_host.rs",
            "src/android_activity_host.rs",
            "src/harmony_ability_host.rs",
        ],
    }
}

pub fn zsui_native_runtime_gate_plans() -> Vec<ZsuiNativeRuntimeGatePlan> {
    vec![
        ZsuiNativeRuntimeGatePlan {
            gate_name: "adapter_manifest",
            required_adapter_capability_names: vec!["main_window", "main_execution_plan_bridge"],
            required_host_surface_names: Vec::new(),
            required_shared_protocol_names: Vec::new(),
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "native_event_loop",
            required_adapter_capability_names: vec!["main_execution_plan_bridge"],
            required_host_surface_names: Vec::new(),
            required_shared_protocol_names: vec!["Command"],
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "native_window_surface",
            required_adapter_capability_names: vec!["main_window", "transient_window"],
            required_host_surface_names: vec!["main_window_host_event_from_message"],
            required_shared_protocol_names: vec!["LayoutProtocol"],
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "native_control_mapping",
            required_adapter_capability_names: vec![
                "main_search_control",
                "settings_window",
                "settings_dropdown",
                "popup_menu",
                "status_item",
            ],
            required_host_surface_names: vec![
                "settings_window_host_event_from_message",
                "dropdown_window_host_event_from_message",
            ],
            required_shared_protocol_names: vec!["Component"],
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "renderer_text_layout",
            required_adapter_capability_names: vec!["renderer", "text_layout"],
            required_host_surface_names: Vec::new(),
            required_shared_protocol_names: vec!["LayoutProtocol", "Component"],
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "native_service_bridges",
            required_adapter_capability_names: vec![
                "clipboard",
                "input_dialog",
                "edit_dialog",
                "shell_open",
                "file_dialog",
                "paste_target",
                "window_identity",
                "ime",
            ],
            required_host_surface_names: vec![
                "input_dialog_host_event_from_message",
                "edit_dialog_host_event_from_message",
            ],
            required_shared_protocol_names: vec!["Command"],
        },
        ZsuiNativeRuntimeGatePlan {
            gate_name: "target_smoke_verification",
            required_adapter_capability_names: REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES
                .iter()
                .map(|capability| capability.capability_name())
                .collect(),
            required_host_surface_names: REQUIRED_UI_HOST_SURFACES
                .iter()
                .map(|surface| surface.adapter_name())
                .collect(),
            required_shared_protocol_names: SHARED_NON_HOST_UI_PROTOCOLS
                .iter()
                .map(|protocol| protocol.protocol_name())
                .collect(),
        },
    ]
}

pub fn zsui_reuse_bootstrap_plan(platform: NativeUiPlatform) -> Option<ZsuiReuseBootstrapPlan> {
    let matrix = native_ui_backend_capability_matrix_for_platform(platform)?;
    let gate_plans = zsui_native_runtime_gate_plans();
    let gate_names: Vec<_> = gate_plans.iter().map(|gate| gate.gate_name).collect();
    let missing_gate_names = missing_native_runtime_gate_names(matrix.backend.status, &gate_names);
    let completion = native_runtime_gate_completion(gate_names.clone(), missing_gate_names.clone());
    let native_adapter_capability_names = matrix.required_capability_names();
    let platform_binding_names = native_adapter_capability_names
        .iter()
        .filter_map(|capability| platform_binding_name_for_capability(platform, capability))
        .collect();

    Some(ZsuiReuseBootstrapPlan {
        platform,
        platform_name: matrix.backend.platform_name(),
        toolkit_name: matrix.backend.toolkit_name(),
        backend_status: matrix.backend.status,
        backend_status_name: matrix.backend.status_name(),
        adapter_boundary: matrix.backend.adapter_boundary,
        adapter_module_path: matrix.backend.module_path,
        native_adapter_capability_names,
        platform_binding_names,
        native_runtime_gate_names: gate_names,
        missing_native_runtime_gate_names: missing_gate_names,
        next_native_runtime_gate_name: completion.next_gate_name,
        native_runtime_gate_plans: gate_plans,
        native_runtime_gate_completion: completion,
    })
}

pub fn zsui_agent_context() -> ZsuiAgentContext {
    ZsuiAgentContext {
        framework_name: ZSUI_FRAMEWORK_NAME,
        context_version: ZSUI_AGENT_CONTEXT_VERSION,
        framework_layers: zsui_framework_layers(),
        boundary_rules: zsui_framework_boundary_rules(),
        completion_areas: zsui_completion_areas(),
        readiness: zsui_reuse_readiness_report(),
        platform_bootstrap: SUPPORTED_NATIVE_UI_PLATFORMS
            .iter()
            .filter_map(|platform| zsui_reuse_bootstrap_plan(*platform))
            .collect(),
        runtime_gate_plans: zsui_native_runtime_gate_plans(),
        integration_steps: zsui_agent_integration_steps(),
    }
}

pub fn zsui_agent_context_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&zsui_agent_context())
}

fn missing_native_runtime_gate_names(
    status: NativeUiBackendStatus,
    gate_names: &[&'static str],
) -> Vec<&'static str> {
    match status {
        NativeUiBackendStatus::NativeHostIntegrated => Vec::new(),
        NativeUiBackendStatus::NativeHostFirstPass => gate_names
            .iter()
            .copied()
            .filter(|gate| {
                matches!(
                    *gate,
                    "native_control_mapping"
                        | "renderer_text_layout"
                        | "native_service_bridges"
                        | "target_smoke_verification"
                )
            })
            .collect(),
        NativeUiBackendStatus::AdapterBoundaryScaffold => gate_names
            .iter()
            .copied()
            .filter(|gate| *gate != "adapter_manifest")
            .collect(),
    }
}

fn native_runtime_gate_completion(
    gate_names: Vec<&'static str>,
    missing_gate_names: Vec<&'static str>,
) -> ZsuiNativeRuntimeGateCompletion {
    let total_gate_count = gate_names.len();
    let missing_gate_count = missing_gate_names.len();
    let completed_gate_count = total_gate_count.saturating_sub(missing_gate_count);
    let completion_percent = if total_gate_count == 0 {
        100
    } else {
        ((completed_gate_count * 100) / total_gate_count) as u8
    };

    ZsuiNativeRuntimeGateCompletion {
        gate_names,
        next_gate_name: missing_gate_names.first().copied(),
        missing_gate_names,
        total_gate_count,
        completed_gate_count,
        missing_gate_count,
        completion_percent,
    }
}

fn zsui_agent_integration_steps() -> Vec<ZsuiAgentIntegrationStep> {
    let readiness = zsui_reuse_readiness_report();
    vec![
        ZsuiAgentIntegrationStep {
            step_name: "select_native_adapter",
            owner_name: "application_crate",
            required_names: readiness.platform_names.clone(),
        },
        ZsuiAgentIntegrationStep {
            step_name: "verify_adapter_capabilities",
            owner_name: "zsui_adapter_boundary",
            required_names: readiness.native_adapter_capability_names.clone(),
        },
        ZsuiAgentIntegrationStep {
            step_name: "implement_product_adapter",
            owner_name: "application_crate",
            required_names: readiness.product_adapter_task_names.clone(),
        },
        ZsuiAgentIntegrationStep {
            step_name: "run_target_smoke",
            owner_name: "native_host",
            required_names: vec![
                "window_screenshot",
                "menu_interaction",
                "dialog_interaction",
                "clipboard_roundtrip",
            ],
        },
    ]
}

fn platform_binding_name_for_capability(
    platform: NativeUiPlatform,
    capability_name: &str,
) -> Option<&'static str> {
    match (platform, capability_name) {
        (NativeUiPlatform::Windows, "main_window")
        | (NativeUiPlatform::Macos, "main_window")
        | (NativeUiPlatform::Linux, "main_window") => Some("winit_native_window"),
        (NativeUiPlatform::Windows, "main_execution_plan_bridge")
        | (NativeUiPlatform::Macos, "main_execution_plan_bridge")
        | (NativeUiPlatform::Linux, "main_execution_plan_bridge") => {
            Some("zsui_native_window_builder")
        }
        (NativeUiPlatform::Android, "main_window") => Some("android_activity_surface"),
        (NativeUiPlatform::Android, "settings_window") => Some("android_settings_fragment"),
        (NativeUiPlatform::Android, "settings_dropdown") => Some("android_spinner_or_menu"),
        (NativeUiPlatform::Android, "input_dialog") => Some("android_text_input_dialog"),
        (NativeUiPlatform::Android, "edit_dialog") => Some("android_text_editor_activity"),
        (NativeUiPlatform::Android, "clipboard") => Some("android_clipboard_manager"),
        (NativeUiPlatform::Android, "popup_menu") => Some("android_popup_menu"),
        (NativeUiPlatform::Android, "status_item") => Some("android_notification_surface"),
        (NativeUiPlatform::Android, "renderer") => Some("android_canvas_or_compose_renderer"),
        (NativeUiPlatform::Android, "text_layout") => Some("android_static_layout"),
        (NativeUiPlatform::Android, "main_search_control") => Some("android_search_view"),
        (NativeUiPlatform::Android, "transient_window") => Some("android_popup_window"),
        (NativeUiPlatform::Android, "ime") => Some("android_input_method_manager"),
        (NativeUiPlatform::Android, "shell_open") => Some("android_intent_launcher"),
        (NativeUiPlatform::Android, "file_dialog") => Some("android_storage_access_framework"),
        (NativeUiPlatform::Android, "paste_target") => Some("android_accessibility_paste_target"),
        (NativeUiPlatform::Android, "window_identity") => Some("android_task_identity"),
        (NativeUiPlatform::Android, "main_execution_plan_bridge") => {
            Some("shared_main_execution_plan_bridge")
        }
        (NativeUiPlatform::Harmony, "main_window") => Some("harmony_ability_window"),
        (NativeUiPlatform::Harmony, "settings_window") => Some("harmony_settings_page"),
        (NativeUiPlatform::Harmony, "settings_dropdown") => Some("harmony_selector_or_menu"),
        (NativeUiPlatform::Harmony, "input_dialog") => Some("harmony_text_input_dialog"),
        (NativeUiPlatform::Harmony, "edit_dialog") => Some("harmony_text_editor_ability"),
        (NativeUiPlatform::Harmony, "clipboard") => Some("harmony_pasteboard"),
        (NativeUiPlatform::Harmony, "popup_menu") => Some("harmony_menu"),
        (NativeUiPlatform::Harmony, "status_item") => Some("harmony_notification_surface"),
        (NativeUiPlatform::Harmony, "renderer") => Some("harmony_canvas_renderer"),
        (NativeUiPlatform::Harmony, "text_layout") => Some("harmony_text_layout"),
        (NativeUiPlatform::Harmony, "main_search_control") => Some("harmony_search_component"),
        (NativeUiPlatform::Harmony, "transient_window") => Some("harmony_popup_component"),
        (NativeUiPlatform::Harmony, "ime") => Some("harmony_input_method_bridge"),
        (NativeUiPlatform::Harmony, "shell_open") => Some("harmony_want_launcher"),
        (NativeUiPlatform::Harmony, "file_dialog") => Some("harmony_document_picker"),
        (NativeUiPlatform::Harmony, "paste_target") => Some("harmony_accessibility_paste_target"),
        (NativeUiPlatform::Harmony, "window_identity") => Some("harmony_ability_identity"),
        (NativeUiPlatform::Harmony, "main_execution_plan_bridge") => {
            Some("shared_main_execution_plan_bridge")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_context_exposes_all_standalone_platforms() {
        let context = zsui_agent_context();

        assert_eq!(context.framework_name, "zsui");
        assert_eq!(
            context.readiness.platform_names,
            vec!["windows", "macos", "linux", "android", "harmony"]
        );
        assert_eq!(context.platform_bootstrap.len(), 5);
        assert!(context.readiness.scaffold_platforms.contains(&"android"));
        assert!(context.readiness.scaffold_platforms.contains(&"harmony"));
        assert!(context
            .readiness
            .declaration_audit_surface_names
            .contains(&"settings_pages"));
        assert!(context
            .readiness
            .product_adapter_task_names
            .contains(&"execute_ai_plan"));
        assert!(context
            .readiness
            .product_adapter_smoke_example_names
            .contains(&"product_adapter_smoke"));
        assert!(context
            .readiness
            .product_adapter_smoke_example_names
            .contains(&"product_adapter_native_driver"));
        assert!(context
            .readiness
            .runtime_harness_stage_names
            .contains(&"start_native_runtime"));
        assert!(context
            .readiness
            .native_status_item_host_operation_names
            .contains(&"create_status_item"));
        assert!(context
            .readiness
            .native_settings_page_model_host_operation_names
            .contains(&"bind_settings_pages"));
        assert!(context
            .readiness
            .native_smoke_artifact_names
            .contains(&"manifest.json"));
        assert!(context
            .readiness
            .native_smoke_command_names
            .contains(&"native_smoke_review"));
        assert!(context
            .readiness
            .mobile_runtime_host_scaffold_module_paths
            .contains(&"src/android_activity_host.rs"));
    }

    #[test]
    fn bootstrap_plan_names_next_gate_for_mobile_scaffolds() {
        let android = zsui_reuse_bootstrap_plan(NativeUiPlatform::Android)
            .expect("android bootstrap should exist");
        let harmony = zsui_reuse_bootstrap_plan(NativeUiPlatform::Harmony)
            .expect("harmony bootstrap should exist");

        assert!(android.scaffolded());
        assert_eq!(
            android.next_native_runtime_gate_name,
            Some("native_event_loop")
        );
        assert!(android
            .platform_binding_names
            .contains(&"android_activity_surface"));
        assert!(harmony
            .platform_binding_names
            .contains(&"harmony_ability_window"));
    }

    #[test]
    fn desktop_bootstrap_reports_only_current_winit_bindings() {
        let windows = zsui_reuse_bootstrap_plan(NativeUiPlatform::Windows)
            .expect("windows bootstrap should exist");

        assert!(!windows.native_runtime_ready());
        assert_eq!(windows.toolkit_name, "winit_desktop");
        assert!(windows
            .platform_binding_names
            .contains(&"winit_native_window"));
        assert!(windows
            .platform_binding_names
            .contains(&"zsui_native_window_builder"));
        assert!(!windows
            .platform_binding_names
            .contains(&"win32_main_window"));
        assert!(windows
            .missing_native_runtime_gate_names
            .contains(&"native_service_bridges"));
    }

    #[test]
    fn agent_context_serializes_for_tools() {
        let json = zsui_agent_context_json().expect("agent context should serialize");

        assert!(json.contains("\"framework_name\": \"zsui\""));
        assert!(json.contains("docs/skills/zsui-native-ui/"));
        assert!(json.contains("winit_desktop"));
        assert!(json.contains("android_activity"));
        assert!(json.contains("harmony_ability"));
        assert!(json.contains("src/harmony_ability_host.rs"));
    }
}
