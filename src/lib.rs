//! ZSUI public framework surface.
//!
//! ZSUI is a Rust-first native system UI framework shape. It is not yet a full
//! self-drawing widget kit: applications declare windows, tray/status menus,
//! shortcuts, settings pages, reusable navigation/card shell layouts and
//! commands in Rust, while platform hosts map those declarations to Win32,
//! AppKit or GTK/libadwaita backends.

pub mod agent_context;
pub mod android_activity_host;
pub mod app;
pub mod app_command;
#[cfg(feature = "auto-suggest")]
pub mod auto_suggest;
#[cfg(feature = "breadcrumb")]
pub mod breadcrumb;
#[cfg(feature = "calculator")]
pub mod calculator;
pub mod capability;
pub mod clipboard;
#[cfg(feature = "color-picker")]
pub mod color_picker;
#[cfg(feature = "command-palette")]
pub mod command_palette;
pub mod command_protocol;
pub mod component_catalog;
pub mod component_protocol;
pub mod components;
#[cfg(feature = "dialog")]
pub mod content_dialog;
pub mod control_protocol;
pub mod core;
#[cfg(feature = "date-picker")]
pub mod date;
pub mod desktop_services;
#[cfg(feature = "document-shell")]
pub mod document_shell;
pub mod event_protocol;
pub mod feature_manifest;
pub mod framework_goals;
pub mod geometry;
#[cfg(feature = "grid-view")]
pub mod grid_view;
pub mod harmony_ability_host;
pub mod host;
pub mod host_protocol;
pub mod hotkey;
pub mod icon;
#[cfg(feature = "image-preview")]
pub mod image_preview;
#[cfg(feature = "info-bar")]
pub mod info_bar;
#[cfg(all(
    target_os = "linux",
    not(target_env = "ohos"),
    feature = "linux-direct"
))]
mod linux_direct;
#[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
pub mod linux_gtk_menu;
#[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
mod linux_gtk_renderer;
#[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
pub mod linux_gtk_services;
#[cfg(feature = "localization")]
pub mod localization;
#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
pub mod macos_appkit_menu;
#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
mod macos_appkit_renderer;
#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
pub mod macos_appkit_services;
pub mod menu;
pub mod mobile_host;
pub mod native;
#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
mod native_accessibility;
pub mod native_adapter_manifest;
#[cfg(any(
    test,
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
mod native_clipboard;
#[cfg(any(
    test,
    all(target_os = "macos", feature = "macos-appkit"),
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        any(feature = "linux-direct", feature = "linux-gtk")
    )
))]
mod native_draw_support;
#[cfg(any(
    test,
    all(windows, feature = "windows-win32"),
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
mod native_file_dialog;
pub mod native_host_actions;
pub mod native_host_launch;
pub mod native_hosts;
pub mod native_icons;
mod native_input_visuals;
#[cfg(any(
    test,
    all(windows, feature = "windows-win32"),
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
mod native_menu;
pub mod native_proof;
pub mod native_smoke;
mod native_text_edit;
#[cfg(feature = "paged-list")]
pub mod paged_list;
#[cfg(feature = "password-box")]
pub mod password_box;
pub mod product_adapter;
#[cfg(any(feature = "progress", feature = "progress-ring"))]
pub mod progress;
pub mod render_protocol;
pub mod settings;
pub mod shell_layout;
pub mod style;
#[cfg(feature = "table")]
pub mod table;
#[cfg(feature = "teaching-tip")]
pub mod teaching_tip;
#[cfg(feature = "time-picker")]
pub mod time;
pub mod timer_protocol;
#[cfg(feature = "toast")]
pub mod toast;
#[cfg(feature = "tooltip")]
pub mod tooltip;
pub mod tray;
#[cfg(feature = "tree")]
pub mod tree;
pub mod ui_surface_protocol;
pub mod view;
pub mod widget_render;
pub mod window;
#[cfg(all(windows, feature = "windows-gdi"))]
pub mod windows_gdi_renderer;
#[cfg(all(
    windows,
    feature = "accessibility",
    feature = "text-input-core",
    feature = "windows-win32"
))]
mod windows_uia;
#[cfg(all(windows, feature = "windows-win32"))]
#[path = "platform/windows/mod.rs"]
pub mod windows_win32_host;
#[cfg(all(windows, feature = "windows-win32", feature = "document-shell"))]
#[path = "platform/windows/text/edit_host.rs"]
pub mod windows_win32_text_editor;
#[cfg(feature = "workbench")]
pub mod workbench;

pub use agent_context::{
    zsui_agent_context, zsui_agent_context_json, zsui_completion_areas,
    zsui_framework_boundary_rules, zsui_framework_layers, zsui_native_runtime_gate_plans,
    zsui_reuse_bootstrap_plan, zsui_reuse_readiness_report, ZsuiAgentContext,
    ZsuiAgentIntegrationStep, ZsuiCompletionArea, ZsuiFrameworkBoundaryRule, ZsuiFrameworkLayer,
    ZsuiNativeRuntimeGateCompletion, ZsuiNativeRuntimeGatePlan, ZsuiReuseBootstrapPlan,
    ZsuiReuseReadinessReport, ZSUI_AGENT_CONTEXT_VERSION, ZSUI_FRAMEWORK_NAME,
};
pub use android_activity_host::{
    android_activity_bridge_callbacks, android_activity_bridge_contract,
    android_activity_bridge_entry_points, android_activity_capability_bindings,
    android_activity_device_smoke_artifacts, android_activity_host_scaffold,
    android_activity_lifecycle_bindings, android_activity_required_permissions,
};
pub use app::{
    app, audit_app_declaration, zsui_declaration_audit_surface_names, AppBuilder, ZsuiApp,
    ZsuiAppDeclarationReport, ZsuiAppRuntime, ZsuiDeclarationIssue, ZsuiDeclarationIssueLevel,
    ZSUI_DECLARATION_AUDIT_SURFACES,
};
pub use app_command::{
    app_command_name, AppCommandDispatchReport, AppCommandExecutor, SharedAppCommandExecutor,
};
#[cfg(feature = "auto-suggest")]
pub use auto_suggest::{
    ZsAutoSuggestState, ZsAutoSuggestSubmission, ZsAutoSuggestTextChange,
    ZsAutoSuggestTextChangeReason, ZsAutoSuggestion, ZsAutoSuggestionId,
};
#[cfg(feature = "breadcrumb")]
pub use breadcrumb::{
    ZsBreadcrumbFocusTarget, ZsBreadcrumbId, ZsBreadcrumbItem, ZsBreadcrumbState,
};
#[cfg(feature = "calculator")]
pub use calculator::{
    ZsCalculatorAction, ZsCalculatorBinaryOperator, ZsCalculatorButtonKind,
    ZsCalculatorButtonRegion, ZsCalculatorEngine, ZsCalculatorHistoryEntry,
    ZsCalculatorInteraction, ZsCalculatorLayout, ZsCalculatorShellSpec,
};
pub use capability::{CapabilityStatus, CapabilitySupport, HostCapabilities, PlatformName};
pub use clipboard::ClipboardData;
#[cfg(feature = "color-picker")]
pub use color_picker::{ZsColorChannel, ZsColorPickerState, ZsHsvColor};
#[cfg(feature = "command-palette")]
pub use command_palette::{ZsCommandPaletteItem, ZsCommandPaletteItemId, ZsCommandPaletteState};
pub use command_protocol::{
    CommandId, CommandPayload, CommandQueue, CommandScope, SharedUiCommandExecutor, UiCommand,
    UiCommandDispatchReport, UiCommandExecutor,
};
pub use component_catalog::{
    zsui_component_catalog, zsui_component_catalog_summary, ZsuiComponentCatalogSummary,
    ZsuiComponentCategory, ZsuiComponentDescriptor, ZsuiComponentStatus, ZSUI_COMPONENT_CATALOG,
};
pub use component_protocol::Component;
#[cfg(feature = "label")]
pub use components::Label;
pub use components::{UiNode, UiNodeKind, UiStackDirection};
#[cfg(feature = "tabs")]
pub use components::{ZsTabId, ZsTabSpec};
#[cfg(feature = "dialog")]
pub use content_dialog::{
    ZsContentDialogButton, ZsContentDialogResult, ZsContentDialogSpec, ZsContentDialogState,
};
pub use control_protocol::{
    NativeControlFamily, NativeControlMapper, NativeControlMapperOperation,
    NativeSettingsControlHost, SettingsComponentKind, SettingsControlHostOperation,
    SettingsControlSpec, REQUIRED_NATIVE_CONTROL_MAPPER_OPERATIONS,
    REQUIRED_SETTINGS_CONTROL_HOST_OPERATIONS,
};
pub use core::{
    AppEvent, Command, DialogButtons, DialogLevel, DialogResponse, FileDialogFilter,
    FileDialogSpec, HotkeyId, NativeDialogSpec, TrayId, WindowId, ZsuiError, ZsuiResult,
};
#[cfg(feature = "date-picker")]
pub use date::{days_in_month, is_leap_year, ZsDate};
pub use desktop_services::{
    ClipboardService, DesktopCapabilities, DesktopCapability, DesktopCapabilityEntry, DesktopEvent,
    DesktopHost, DesktopKey, DesktopTheme, FileDialogService, IconService, KeyModifiers,
    MenuService, NativeClipboardService, NativeFileDialogService, SaveFileDialogSpec,
    TextInputRequest, TextInputService, ThemePreference, ThemeService, WindowService,
    REQUIRED_DESKTOP_CAPABILITIES,
};
#[cfg(feature = "document-shell")]
pub use document_shell::{
    ZsDocumentShellCommand, ZsDocumentShellCommandRegion, ZsDocumentShellInteraction,
    ZsDocumentShellLayout, ZsDocumentShellSpec, ZsTextCursorStatus, ZsTextDocument,
    ZsTextDocumentEncoding,
};
pub use event_protocol::{
    ComponentPhase, KeyState, LifecycleEvent, LifecycleState, MouseButton, UiEvent,
};
pub use feature_manifest::{
    zsui_default_feature_names, zsui_feature_manifest, zsui_optional_dependency_feature_names,
    ZsuiCargoFeature, ZsuiFeatureCategory,
};
pub use framework_goals::{zsui_rust_first_goal_names, zsui_rust_first_goals, ZsuiRustFirstGoal};
pub use geometry::{
    clamp_window_pos_to_rect, dpi_compensated_size, ComponentId, Dp, Dpi, DpiCompensationPlan,
    DpiCompensationState, LayoutInput, LayoutNode, LayoutOutput, LayoutProtocol, Point, Px, Rect,
    SharedUiProtocol, Size, UiLength, UiRect, SHARED_NON_HOST_UI_PROTOCOLS,
};
#[cfg(feature = "grid-view")]
pub use grid_view::{ZsGridViewItem, ZsGridViewItemId, ZsGridViewState};
pub use harmony_ability_host::{
    harmony_ability_bridge_callbacks, harmony_ability_bridge_contract,
    harmony_ability_bridge_entry_points, harmony_ability_capability_bindings,
    harmony_ability_device_smoke_artifacts, harmony_ability_host_scaffold,
    harmony_ability_lifecycle_bindings, harmony_ability_required_permissions,
};
pub use host::{MemoryHost, PlatformHost, TrayRecord, WindowRecord, ZsuiHost};
pub use host_protocol::{
    clipboard_monitor_poll_result_for_sequence, native_paste_target_activation_snapshot,
    native_window_identity_snapshot, poll_clipboard_monitor, ClipboardHost,
    ClipboardMonitorPollResult, ClipboardMonitorState, NativeAutostartApplyResult,
    NativeAutostartHost, NativeAutostartStatus, NativeDialogButtons, NativeDialogHost,
    NativeDialogHostOperation, NativeDialogLevel, NativeDialogResponse, NativeEditTextDialogHost,
    NativeEditTextDialogHostOperation, NativeEditTextDialogRequest, NativeEditTextDialogResult,
    NativeEditTextSaveHandler, NativeFileDialogHost, NativeFileDialogHostOperation,
    NativeFileDialogRequest, NativeImeCandidateAnchor, NativeImeCompositionAnchor, NativeImeHost,
    NativeImeHostOperation, NativeMailMergeWindowHost, NativeMailMergeWindowHostOperation,
    NativeMailMergeWindowRequest, NativePasteTargetActivationSnapshot, NativePasteTargetHost,
    NativePasteTargetHostOperation, NativePopupMenuEntry, NativePopupMenuHost,
    NativePopupMenuHostOperation, NativePopupMenuPlacement, NativeShellOpenHost,
    NativeShellOpenHostOperation, NativeTextCaretAnchor, NativeTextCaretHost,
    NativeTextCaretHostOperation, NativeTextInputDialogHost, NativeTextInputDialogHostOperation,
    NativeTextInputDialogRequest, NativeTransientWindowHost, NativeTransientWindowHostOperation,
    NativeTransientWindowPresentation, NativeTransientWindowRequest, NativeWindowIdentityHost,
    NativeWindowIdentityHostOperation, NativeWindowIdentitySnapshot, PasteTargetFocusStatus,
    PasteTargetTextInputCapabilities, REQUIRED_NATIVE_DIALOG_HOST_OPERATIONS,
    REQUIRED_NATIVE_EDIT_TEXT_DIALOG_HOST_OPERATIONS, REQUIRED_NATIVE_FILE_DIALOG_HOST_OPERATIONS,
    REQUIRED_NATIVE_IME_HOST_OPERATIONS, REQUIRED_NATIVE_MAIL_MERGE_WINDOW_HOST_OPERATIONS,
    REQUIRED_NATIVE_PASTE_TARGET_HOST_OPERATIONS, REQUIRED_NATIVE_POPUP_MENU_HOST_OPERATIONS,
    REQUIRED_NATIVE_SHELL_OPEN_HOST_OPERATIONS, REQUIRED_NATIVE_TEXT_CARET_HOST_OPERATIONS,
    REQUIRED_NATIVE_TEXT_INPUT_DIALOG_HOST_OPERATIONS,
    REQUIRED_NATIVE_TRANSIENT_WINDOW_HOST_OPERATIONS,
    REQUIRED_NATIVE_WINDOW_IDENTITY_HOST_OPERATIONS,
};
pub use hotkey::HotkeySpec;
pub use icon::ZsIcon;
#[cfg(feature = "image-preview")]
pub use image_preview::{
    zs_image_native_draw_command, zs_image_render_geometry, ZsImageFit, ZsImagePreviewConfig,
    ZsImagePreviewSnapshot, ZsImagePreviewState, ZsImageRenderGeometry,
};
#[cfg(feature = "info-bar")]
pub use info_bar::{
    ZsInfoBarControl, ZsInfoBarEvent, ZsInfoBarSeverity, ZsInfoBarSpec, ZsInfoBarState,
};
#[cfg(feature = "localization")]
pub use localization::{ZsLocale, ZsLocalizer, ZsMessageArgs, ZsMessageValue, ZsTextDirection};
pub use menu::{MenuItemSpec, MenuSpec, ZsAccelerator, ZsAcceleratorKey};
pub use mobile_host::{
    mobile_runtime_bridge_callback_symbol_names, mobile_runtime_bridge_contract,
    mobile_runtime_bridge_contract_artifact_file_names,
    mobile_runtime_bridge_contract_artifact_requirements, mobile_runtime_bridge_contract_json,
    mobile_runtime_bridge_contract_module_paths, mobile_runtime_bridge_contract_smoke_report,
    mobile_runtime_bridge_contract_smoke_report_json, mobile_runtime_bridge_contract_smoke_reports,
    mobile_runtime_bridge_contract_smoke_reports_json, mobile_runtime_bridge_contracts,
    mobile_runtime_bridge_contracts_json, mobile_runtime_bridge_dispatch_report,
    mobile_runtime_bridge_dispatch_report_json, mobile_runtime_bridge_dispatch_reports,
    mobile_runtime_bridge_dispatch_reports_json, mobile_runtime_bridge_dispatch_steps,
    mobile_runtime_bridge_parity_report, mobile_runtime_bridge_parity_report_json,
    mobile_runtime_bridge_parity_reports, mobile_runtime_bridge_parity_reports_json,
    mobile_runtime_device_smoke_artifact_names, mobile_runtime_device_smoke_command_names,
    mobile_runtime_device_smoke_plan, mobile_runtime_device_smoke_plan_json,
    mobile_runtime_device_smoke_plan_with_artifact_root, mobile_runtime_device_smoke_plans,
    mobile_runtime_device_smoke_plans_json, mobile_runtime_device_smoke_trace_template,
    mobile_runtime_device_smoke_trace_template_json, mobile_runtime_device_smoke_trace_templates,
    mobile_runtime_device_smoke_trace_templates_json, mobile_runtime_host_scaffold,
    mobile_runtime_host_scaffold_json, mobile_runtime_host_scaffold_module_paths,
    mobile_runtime_host_scaffolds, mobile_runtime_host_scaffolds_json,
    mobile_runtime_required_bridge_callback_kind_names,
    mobile_runtime_required_bridge_dispatch_operation_names,
    review_mobile_runtime_bridge_contract_artifacts,
    review_mobile_runtime_bridge_contract_artifacts_at,
    review_mobile_runtime_bridge_contract_artifacts_for_all,
    review_mobile_runtime_bridge_contract_artifacts_for_all_at,
    review_mobile_runtime_device_smoke_artifacts, review_mobile_runtime_device_smoke_artifacts_at,
    write_mobile_runtime_bridge_contract_artifacts,
    write_mobile_runtime_bridge_contract_artifacts_for_all,
    write_mobile_runtime_bridge_contract_artifacts_for_all_to,
    write_mobile_runtime_bridge_contract_artifacts_to, MobileRuntimeBridgeCallback,
    MobileRuntimeBridgeCallbackKind, MobileRuntimeBridgeContract,
    MobileRuntimeBridgeContractArtifactRequirement,
    MobileRuntimeBridgeContractArtifactReviewReport, MobileRuntimeBridgeContractArtifactStatus,
    MobileRuntimeBridgeContractArtifactWriteReport, MobileRuntimeBridgeContractSmokeReport,
    MobileRuntimeBridgeContractSmokeStep, MobileRuntimeBridgeDispatchReport,
    MobileRuntimeBridgeDispatchStep, MobileRuntimeBridgeEntryPoint,
    MobileRuntimeBridgeParityReport, MobileRuntimeCapabilityBinding,
    MobileRuntimeDeviceSmokeArtifact, MobileRuntimeDeviceSmokeArtifactStatus,
    MobileRuntimeDeviceSmokePlan, MobileRuntimeDeviceSmokeReviewReport,
    MobileRuntimeDeviceSmokeTrace, MobileRuntimeDeviceSmokeTraceKind, MobileRuntimeHostScaffold,
    MobileRuntimeLifecycleBinding, MobileRuntimePermission,
};
pub use native::{
    native_window, run_native_window, run_native_window_smoke, typed_native_window,
    NativeViewCaptureEvidence, NativeViewKey, NativeViewSmokeInput, NativeWindowBuilder,
    NativeWindowContentMissing, NativeWindowContentReady, NativeWindowHost,
    NativeWindowResourcePolicy, NativeWindowRuntimeDriver, NativeWindowRuntimeDriverReport,
    NativeWindowRuntimeHandle, NativeWindowSmokeRunOptions, NativeWindowSmokeRunReport,
    TypedNativeWindowBuilder,
};
pub use native_adapter_manifest::{
    native_ui_adapter_parity_report, native_ui_backend_capability_matrix,
    native_ui_backend_capability_matrix_for_platform, native_ui_backend_for_current_target,
    native_ui_backend_for_platform, native_ui_backend_for_toolkit,
    native_ui_platform_for_current_target, native_ui_platform_readiness,
    native_ui_platform_readiness_reports, NativeUiAdapterBindingPlan, NativeUiAdapterCapability,
    NativeUiAdapterManifest, NativeUiAdapterParityReport, NativeUiAdapterReusePackage,
    NativeUiBackendCapabilityMatrix, NativeUiBackendDescriptor, NativeUiBackendStatus,
    NativeUiCapabilityReadiness, NativeUiCapabilityReadinessLevel, NativeUiPlatform,
    NativeUiPlatformReadinessReport, NativeUiToolkit, REQUIRED_NATIVE_UI_ADAPTER_CAPABILITIES,
    SUPPORTED_NATIVE_UI_BACKENDS, SUPPORTED_NATIVE_UI_PLATFORMS, SUPPORTED_NATIVE_UI_TOOLKITS,
};
pub use native_host_actions::{
    command_ids as native_command_ids, dispatch_settings_action, main_menu_command_for_id,
    main_tray_action_plan, main_tray_menu_plan, menu_ids as native_menu_ids,
    native_host_status_menu_entries, native_status_menu_action_icon_name,
    required_native_host_settings_action_names, required_native_host_settings_control_action_names,
    required_native_host_status_menu_action_names, settings_action_for_route,
    settings_action_route, settings_command_for_control_role, settings_command_id_for_role,
    MainTrayActionInput, MainTrayActionPlan, MainTrayMenuAction, MainTrayMenuInput,
    MainTrayMenuItem, MainTrayMenuText, NativeHostSearchControlAction, NativeHostSearchTextAction,
    NativeHostSettingsAction, NativeHostSettingsControlAction, NativeHostSettingsGroupAction,
    NativeHostSettingsPlatformAction, NativeHostStatusMenuAction, NativeHostUiAction,
    SettingsAction, SettingsActionExecutor, SettingsActionRoute, SettingsControlRole,
    StatusMenuEntry, REQUIRED_NATIVE_HOST_SEARCH_CONTROL_ACTIONS,
    REQUIRED_NATIVE_HOST_SETTINGS_ACTIONS, REQUIRED_NATIVE_HOST_SETTINGS_CONTROL_ACTIONS,
    REQUIRED_NATIVE_HOST_SETTINGS_GROUP_ACTIONS, REQUIRED_NATIVE_HOST_SETTINGS_PLATFORM_ACTIONS,
    REQUIRED_NATIVE_HOST_STATUS_MENU_ACTIONS, REQUIRED_NATIVE_HOST_UI_ACTIONS,
};
pub use native_host_launch::{
    native_host_launch_plan_for_current_target, native_host_launch_plan_for_platform,
    NativeHostLaunchMode, NativeHostLaunchPlan,
};
pub use native_hosts::{
    native_status_menu_command_from_menu, required_native_runtime_driver_operation_names,
    required_native_settings_item_update_host_operation_names,
    required_native_settings_page_model_host_operation_names,
    required_native_status_item_host_operation_names,
    required_native_status_menu_command_host_operation_names, NativeAppIconResource,
    NativeMainSearchControlHost, NativeMainSearchControlHostOperation,
    NativeMainSearchControlPresentation, NativeMainSearchControlRequest,
    NativeMainSearchStylePresentation, NativeMainSearchStyleRequest, NativeMainWindowHandles,
    NativeMainWindowHost, NativeMainWindowHostOperation, NativeMainWindowPresentMode,
    NativeMainWindowPresentation, NativeMainWindowRequest, NativeRuntimeDriver,
    NativeRuntimeDriverOperation, NativeRuntimeStartupRequest, NativeRuntimeStartupResult,
    NativeSettingsDropdownHost, NativeSettingsDropdownHostOperation,
    NativeSettingsDropdownPresentation, NativeSettingsDropdownRequest,
    NativeSettingsItemUpdateHost, NativeSettingsItemUpdateHostOperation,
    NativeSettingsItemUpdateRequest, NativeSettingsItemUpdateResult, NativeSettingsPageModelHost,
    NativeSettingsPageModelHostOperation, NativeSettingsPageModelPresentation,
    NativeSettingsPageModelRequest, NativeSettingsWindowHost, NativeSettingsWindowHostOperation,
    NativeSettingsWindowPresentation, NativeSettingsWindowRequest, NativeStatusItemHost,
    NativeStatusItemHostOperation, NativeStatusItemPresentation, NativeStatusItemRequest,
    NativeStatusMenuCommandHost, NativeStatusMenuCommandHostOperation,
    NativeStatusMenuCommandRequest, NativeStatusMenuCommandResult, NativeWindowOptions,
    REQUIRED_NATIVE_MAIN_SEARCH_CONTROL_HOST_OPERATIONS,
    REQUIRED_NATIVE_MAIN_WINDOW_HOST_OPERATIONS, REQUIRED_NATIVE_RUNTIME_DRIVER_OPERATIONS,
    REQUIRED_NATIVE_SETTINGS_DROPDOWN_HOST_OPERATIONS,
    REQUIRED_NATIVE_SETTINGS_ITEM_UPDATE_HOST_OPERATIONS,
    REQUIRED_NATIVE_SETTINGS_PAGE_MODEL_HOST_OPERATIONS,
    REQUIRED_NATIVE_SETTINGS_WINDOW_HOST_OPERATIONS, REQUIRED_NATIVE_STATUS_ITEM_HOST_OPERATIONS,
    REQUIRED_NATIVE_STATUS_MENU_COMMAND_HOST_OPERATIONS,
};
#[cfg(any(
    feature = "fluent-icons",
    all(target_os = "macos", feature = "macos-appkit"),
    all(
        target_os = "linux",
        any(feature = "linux-direct", feature = "linux-gtk")
    )
))]
pub use native_icons::{
    bundled_fluent_icon_svg, FLUENT_SYSTEM_ICONS_LICENSE, FLUENT_SYSTEM_ICONS_NOTICE,
};
pub use native_icons::{
    native_icon_candidates, resolve_native_icon, NativeIconLookup, NativeIconSource,
    NativeIconSourceKind, WINDOWS_FLUENT_ICON_FONT_FAMILY, WINDOWS_MDL2_ICON_FONT_FAMILY,
};
pub use native_proof::{
    NativeProofDocument, NativeProofProcessMemoryEvidence, NativeProofRunnerEvidence,
    NativeProofWidgetEvidence, NativeProofWindowEvidence, NATIVE_PROOF_SCHEMA,
    NATIVE_PROOF_SCHEMA_VERSION,
};
pub use native_smoke::{
    native_host_smoke_artifact_names, native_host_smoke_artifact_requirements,
    native_host_smoke_command_names, native_host_smoke_plan,
    native_host_smoke_plan_for_current_target, native_host_smoke_plan_json,
    native_host_smoke_plan_with_artifact_root, native_host_smoke_plans,
    native_host_smoke_plans_json, review_native_host_smoke_artifacts,
    review_native_host_smoke_artifacts_at, write_native_host_smoke_artifacts,
    write_native_host_smoke_artifacts_to, write_native_host_smoke_artifacts_with_interaction_to,
    NativeHostSmokeArtifactKind, NativeHostSmokeArtifactRequirement, NativeHostSmokeArtifactStatus,
    NativeHostSmokeInteractionReport, NativeHostSmokePlan, NativeHostSmokeReviewReport,
    NativeHostSmokeWriteReport,
};
#[cfg(feature = "paged-list")]
pub use paged_list::{
    paged_list, Page, PageIndex, PageLoadError, PageRequest, PagedDataSource, PagedItem,
    PagedListAnchor, PagedListConfig, PagedListSnapshot, PagedListState, PagedListSyncReconcile,
};
#[cfg(feature = "password-box")]
pub use password_box::{
    mask_password, zs_password_box_native_draw_plan, zs_password_box_render_plan, ZsPassword,
    ZsPasswordBoxMetrics, ZsPasswordBoxPlatformStyle, ZsPasswordBoxRenderPlan,
    ZsPasswordRevealMode,
};
pub use product_adapter::{
    product_adapter_reuse_checklist, product_adapter_runtime_smoke_example_names,
    required_product_adapter_surface_names, required_product_adapter_task_names,
    ui_command_id_name, zsui_reusable_runtime_harness_stage_names, ProductAdapterHost,
    ProductAdapterIdentity, ProductAdapterReuseChecklist, ProductAdapterRuntimeSmokeReport,
    ProductAdapterRuntimeSmokeRequest, ProductAdapterSurface, ProductAdapterTask,
    ProductAdapterUiCommandExecutor, ProductAiCapabilityDescriptor, ProductAiExecutionPlan,
    ProductAiExecutorBoundary, ProductAiInvocation, ProductAiProviderFamily, ProductAiResult,
    ProductUiProjection, ProductViewAdapterHost, ProductViewRuntimeSmokeReport,
    ProductViewRuntimeSmokeRequest, ZsuiReusableRuntimeHarness, ZsuiReusableRuntimeHarnessStage,
    PRODUCT_ADAPTER_SMOKE_COMMAND, REQUIRED_PRODUCT_ADAPTER_SURFACES,
    REQUIRED_PRODUCT_ADAPTER_TASKS, ZSUI_REUSABLE_RUNTIME_HARNESS_STAGES,
};
#[cfg(any(feature = "progress", feature = "progress-ring"))]
pub use progress::ProgressRange;
#[cfg(feature = "progress-ring")]
pub use progress::{
    zs_progress_ring_metrics, zs_progress_ring_native_draw_plan, zs_progress_ring_render_plan,
    ZsProgressRingMetrics, ZsProgressRingMode, ZsProgressRingPlatformStyle,
    ZsProgressRingRenderPlan, ZsProgressRingSize, ZsProgressRingSpec,
};
#[cfg(feature = "password-box")]
pub use render_protocol::NativeDrawSecureTextCommand;
pub use render_protocol::{
    required_native_draw_command_operation_names, Color, ColorRole, HorizontalAlign,
    NativeDrawCommand, NativeDrawCommandOperation, NativeDrawCommandSink, NativeDrawFill,
    NativeDrawIconCommand, NativeDrawImageCommand, NativeDrawPlan, NativeDrawTextCommand,
    NativeFontMetrics, NativeIconColorMode, NativeImageInterpolation, NativeStyleHostOperation,
    NativeStyleResolver, NativeTypographyProfile, Renderer, RendererHostOperation,
    SemanticTextStyle, TextLayout, TextLayoutHostOperation, TextRole, TextRun, TextStyle,
    TextWeight, TextWrap, VerticalAlign, ZsImageFrame, ZsImageFrameId, ZsTypographyMetrics,
    ZsTypographyPlatformStyle, REQUIRED_NATIVE_DRAW_COMMAND_OPERATIONS,
    REQUIRED_NATIVE_STYLE_HOST_OPERATIONS, REQUIRED_RENDERER_HOST_OPERATIONS,
    REQUIRED_TEXT_LAYOUT_HOST_OPERATIONS,
};
pub use settings::{SettingsItemKind, SettingsItemSpec, SettingsPageSpec, SettingsValue};
pub use shell_layout::{
    ZsActionAreaSpec, ZsActionButtonKind, ZsActionButtonSpec, ZsContentRowSpec, ZsGroupCardSpec,
    ZsNavItemSpec, ZsNavigationLayoutMetrics, ZsNavigationLayoutPlan, ZsNavigationLayoutRegion,
    ZsNavigationLayoutRegionKind, ZsNavigationScaffoldAudit, ZsNavigationScaffoldSpec,
    ZsRowAccessory, ZsShellActionAreaSpec, ZsShellActionButtonKind, ZsShellActionButtonSpec,
    ZsShellContentRowSpec, ZsShellGroupCardSpec, ZsShellInteractionEvent, ZsShellInteractionUpdate,
    ZsShellLayoutAudit, ZsShellLayoutMetrics, ZsShellLayoutPlan, ZsShellLayoutRegion,
    ZsShellLayoutRegionKind, ZsShellLayoutSpec, ZsShellNavHoverTransition, ZsShellNavItemSpec,
    ZsShellPointerDownTarget, ZsShellPointerMoveTransition, ZsShellRowAccessory, ZsShellRuntime,
};
pub use style::{
    ControlMetricToken, RadiusToken, SpacingToken, ThemeColorToken, TypographyToken,
    ZsuiColorTokens, ZsuiControlMetrics, ZsuiRadiusTokens, ZsuiSpacingTokens, ZsuiTheme,
    ZsuiThemeMode, ZsuiTypographyStyle, ZsuiTypographyTokens, ZSUI_FLUENT_CARD_RADIUS,
    ZSUI_FLUENT_COMPACT_CONTROL_HEIGHT, ZSUI_FLUENT_CONTROL_RADIUS, ZSUI_FLUENT_GRID_UNIT,
    ZSUI_FLUENT_NAVIGATION_ROW_HEIGHT, ZSUI_FLUENT_SMALL_ICON_SIZE,
    ZSUI_FLUENT_STANDARD_CONTROL_HEIGHT, ZSUI_FLUENT_STANDARD_ICON_SIZE, ZSUI_FLUENT_TOUCH_TARGET,
};
#[cfg(feature = "table")]
pub use table::{
    ZsTableColumn, ZsTableColumnId, ZsTableColumnWidth, ZsTableRow, ZsTableRowId, ZsTableSort,
    ZsTableSortDirection, ZsTableViewState,
};
#[cfg(feature = "teaching-tip")]
pub use teaching_tip::{
    ZsTeachingTipControl, ZsTeachingTipDismissReason, ZsTeachingTipPlacement,
    ZsTeachingTipResponse, ZsTeachingTipResult, ZsTeachingTipSpec, ZsTeachingTipState,
};
#[cfg(feature = "time-picker")]
pub use time::{ZsClockFormat, ZsMinuteIncrement, ZsTime};
pub use timer_protocol::{
    main_timer_task_for_id, settings_timer_task_for_id, MainTimerIds, MainTimerTask,
    SettingsTimerIds, SettingsTimerTask,
};
#[cfg(feature = "toast")]
pub use toast::{
    ZsToastControl, ZsToastDismissReason, ZsToastDuration, ZsToastId, ZsToastResponse,
    ZsToastResult, ZsToastSpec, ZsToastState,
};
#[cfg(feature = "tooltip")]
pub use tooltip::{
    zs_tooltip_native_draw_plan, zs_tooltip_render_plan, ZsTooltipMetrics, ZsTooltipPlacement,
    ZsTooltipPlatformStyle, ZsTooltipRenderPlan, ZsTooltipSpec,
};
pub use tray::TraySpec;
#[cfg(feature = "tree")]
pub use tree::{ZsTreeExpansionChange, ZsTreeNode, ZsTreeNodeId, ZsTreeRowState, ZsTreeViewState};
pub use ui_surface_protocol::{UiHostSurface, REQUIRED_UI_HOST_SURFACES};
#[cfg(feature = "auto-suggest")]
pub use view::auto_suggest_box;
#[cfg(feature = "breadcrumb")]
pub use view::breadcrumb_bar;
#[cfg(feature = "checkbox")]
pub use view::checkbox;
#[cfg(feature = "color-picker")]
pub use view::color_picker;
#[cfg(feature = "combo")]
pub use view::combo_box;
#[cfg(feature = "command-palette")]
pub use view::command_palette;
#[cfg(feature = "dialog")]
pub use view::content_dialog;
#[cfg(feature = "table")]
pub use view::data_grid;
#[cfg(feature = "date-picker")]
pub use view::date_picker;
#[cfg(feature = "grid")]
pub use view::grid;
#[cfg(feature = "grid-view")]
pub use view::grid_view;
#[cfg(feature = "image-preview")]
pub use view::image_preview;
#[cfg(feature = "info-bar")]
pub use view::info_bar;
#[cfg(feature = "list")]
pub use view::list;
#[cfg(feature = "number-box")]
pub use view::number_box;
#[cfg(feature = "password-box")]
pub use view::password_box;
#[cfg(feature = "progress")]
pub use view::progress_bar;
#[cfg(feature = "progress-ring")]
pub use view::progress_ring;
#[cfg(feature = "radio")]
pub use view::radio_button;
#[cfg(feature = "scroll")]
pub use view::scroll;
#[cfg(feature = "teaching-tip")]
pub use view::teaching_tip;
#[cfg(feature = "textbox")]
pub use view::text_editor;
#[cfg(feature = "textbox")]
pub use view::textbox;
#[cfg(feature = "time-picker")]
pub use view::time_picker;
#[cfg(feature = "toast")]
pub use view::toast_presenter;
#[cfg(feature = "toggle")]
pub use view::toggle;
#[cfg(feature = "toggle-button")]
pub use view::toggle_button;
#[cfg(feature = "tree")]
pub use view::tree_view;
#[cfg(feature = "tooltip")]
pub use view::ViewTooltipTarget;
#[cfg(feature = "date-picker")]
pub use view::ZsDatePickerState;
#[cfg(feature = "time-picker")]
pub use view::ZsTimePickerState;
#[cfg(feature = "button")]
pub use view::{button, navigation_item, toolbar_button, ZsButtonPresentation};
pub use view::{
    column, live_view_runtime, live_view_runtime_with_app_commands, row, spacer, AppCx,
    LiveViewUpdate, SharedLiveViewRuntime, View, ViewEvent, ViewEventCx, ViewHitTarget,
    ViewHitTargetKind, ViewInteractionPlan, ViewLayoutCx, ViewNode, ViewNodeKind, ViewPaintCx,
    ViewStackDirection, ViewStyle, WidgetId,
};
#[cfg(feature = "button")]
pub use view::{command_bar, ZsCommandBarSpec};
#[cfg(feature = "label")]
pub use view::{navigation_view, section, ZsNavigationViewSpec};
#[cfg(feature = "slider")]
pub use view::{slider, SliderRange};
#[cfg(feature = "label")]
pub use view::{styled_text, text};
#[cfg(feature = "tabs")]
pub use view::{tab_view, ZsTabItem, ZsTabViewState};
#[cfg(feature = "virtual-list")]
pub use view::{virtual_list, virtual_list_viewport};
#[cfg(feature = "virtual-list")]
pub use view::{VirtualListRange, VirtualListScrollDirection, VirtualListViewport};
#[cfg(feature = "grid")]
pub use view::{ZsGridCell, ZsGridFraction, ZsGridPlacement, ZsGridSpan, ZsGridTrack};
#[cfg(feature = "number-box")]
pub use view::{ZsNumberBoxState, ZsNumberFormat, ZsNumberRange};
#[cfg(feature = "textbox")]
pub use view::{ZsTextEditCommand, ZsTextEditCommandRequest, ZsTextSelection};
#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "color-picker",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker"
))]
pub use widget_render::ZsPopupPlacement;
#[cfg(feature = "auto-suggest")]
pub use widget_render::{
    zs_auto_suggest_header_native_draw_plan, zs_auto_suggest_popup_native_draw_plan,
    zs_auto_suggest_render_plan, zs_auto_suggest_render_plan_in_viewport, ZsAutoSuggestMetrics,
    ZsAutoSuggestPlatformStyle, ZsAutoSuggestRenderPlan, ZS_AUTO_SUGGEST_MAX_VISIBLE_ITEMS,
};
#[cfg(feature = "breadcrumb")]
pub use widget_render::{
    zs_breadcrumb_native_draw_plan, zs_breadcrumb_popup_native_draw_plan,
    zs_breadcrumb_render_plan, ZsBreadcrumbItemRenderPlan, ZsBreadcrumbMetrics,
    ZsBreadcrumbOverflowRowRenderPlan, ZsBreadcrumbPlatformStyle, ZsBreadcrumbRenderPlan,
};
#[cfg(feature = "color-picker")]
pub use widget_render::{
    zs_color_picker_header_native_draw_plan, zs_color_picker_popup_native_draw_plan,
    zs_color_picker_render_plan, zs_color_picker_render_plan_in_viewport,
    ZsColorPickerChannelRenderPlan, ZsColorPickerMetrics, ZsColorPickerPlatformStyle,
    ZsColorPickerRenderPlan,
};
#[cfg(feature = "combo")]
pub use widget_render::{
    zs_combo_box_header_native_draw_plan, zs_combo_box_popup_native_draw_plan,
    zs_combo_box_render_plan, zs_combo_box_render_plan_in_viewport,
    zs_combo_box_render_plan_in_viewport_with_scroll, zs_combo_box_render_plan_with_scroll,
    ZsComboBoxRenderPlan, ZS_COMBO_BOX_MAX_VISIBLE_OPTIONS,
};
#[cfg(feature = "command-palette")]
pub use widget_render::{
    zs_command_palette_native_draw_plan, zs_command_palette_render_plan, ZsCommandPaletteMetrics,
    ZsCommandPalettePlatformStyle, ZsCommandPaletteRenderPlan, ZsCommandPaletteRowRenderPlan,
    ZS_COMMAND_PALETTE_MAX_VISIBLE_ITEMS,
};
#[cfg(feature = "dialog")]
pub use widget_render::{
    zs_content_dialog_native_draw_plan, zs_content_dialog_render_plan,
    ZsContentDialogButtonRenderPlan, ZsContentDialogMetrics, ZsContentDialogPlatformStyle,
    ZsContentDialogRenderPlan,
};
#[cfg(feature = "date-picker")]
pub use widget_render::{
    zs_date_picker_header_native_draw_plan, zs_date_picker_popup_native_draw_plan,
    zs_date_picker_render_plan, zs_date_picker_render_plan_in_viewport,
    zs_date_picker_render_plan_in_viewport_with_today, zs_date_picker_render_plan_with_today,
    ZsDatePickerDayCell, ZsDatePickerRenderPlan,
};
#[cfg(feature = "grid-view")]
pub use widget_render::{
    zs_grid_view_native_draw_plan, zs_grid_view_render_plan, ZsGridViewItemRenderPlan,
    ZsGridViewMetrics, ZsGridViewPlatformStyle, ZsGridViewRenderPlan,
};
#[cfg(feature = "info-bar")]
pub use widget_render::{
    zs_info_bar_native_draw_plan, zs_info_bar_render_plan, ZsInfoBarMetrics,
    ZsInfoBarPlatformStyle, ZsInfoBarRenderPlan,
};
#[cfg(feature = "button")]
pub use widget_render::{
    zs_navigation_item_native_draw_plan, zs_navigation_item_render_plan, ZsNavigationItemMetrics,
    ZsNavigationItemRenderPlan,
};
#[cfg(feature = "number-box")]
pub use widget_render::{
    zs_number_box_native_draw_plan, zs_number_box_render_plan, ZsNumberBoxMetrics,
    ZsNumberBoxPlatformStyle, ZsNumberBoxRenderPlan,
};
#[cfg(feature = "progress")]
pub use widget_render::{
    zs_progress_bar_native_draw_plan, zs_progress_bar_render_plan,
    zs_progress_bar_render_plan_for_platform, ZsProgressBarRenderPlan,
};
#[cfg(feature = "radio")]
pub use widget_render::{
    zs_radio_native_draw_plan, zs_radio_render_plan, zs_radio_render_plan_for_platform,
    ZsRadioRenderPlan,
};
#[cfg(feature = "slider")]
pub use widget_render::{
    zs_slider_native_draw_plan, zs_slider_render_plan, zs_slider_render_plan_for_platform,
    ZsSliderRenderPlan,
};
#[cfg(feature = "tabs")]
pub use widget_render::{
    zs_tab_view_native_draw_plan, zs_tab_view_native_draw_plan_for_tabs, zs_tab_view_render_plan,
    zs_tab_view_render_plan_for_tabs, ZsTabHeaderRenderPlan, ZsTabPlatformStyle, ZsTabViewMetrics,
    ZsTabViewRenderPlan,
};
#[cfg(feature = "table")]
pub use widget_render::{
    zs_table_native_draw_plan, zs_table_render_plan, ZsTableCellRenderPlan,
    ZsTableColumnRenderPlan, ZsTableMetrics, ZsTablePlatformStyle, ZsTableRenderPlan,
    ZsTableRowRenderPlan,
};
#[cfg(feature = "teaching-tip")]
pub use widget_render::{
    zs_teaching_tip_native_draw_plan, zs_teaching_tip_render_plan, ZsTeachingTipMetrics,
    ZsTeachingTipPlatformStyle, ZsTeachingTipRenderPlan,
};
#[cfg(feature = "time-picker")]
pub use widget_render::{
    zs_time_picker_header_native_draw_plan, zs_time_picker_popup_native_draw_plan,
    zs_time_picker_render_plan, zs_time_picker_render_plan_in_viewport, ZsTimePickerChoice,
    ZsTimePickerMetrics, ZsTimePickerPlatformStyle, ZsTimePickerRenderPlan, ZsTimePickerSegment,
};
#[cfg(feature = "toast")]
pub use widget_render::{
    zs_toast_native_draw_plan, zs_toast_render_plan, ZsToastMetrics, ZsToastPlatformStyle,
    ZsToastRenderPlan,
};
#[cfg(feature = "toggle-button")]
pub use widget_render::{
    zs_toggle_button_native_draw_plan, zs_toggle_button_render_plan, ZsToggleButtonMetrics,
    ZsToggleButtonPlatformStyle, ZsToggleButtonRenderPlan,
};
pub use widget_render::{
    zs_toggle_native_draw_plan, zs_toggle_render_plan, zs_toggle_render_plan_for_platform,
    ZsToggleRenderPlan,
};
#[cfg(feature = "tree")]
pub use widget_render::{
    zs_tree_view_native_draw_plan, zs_tree_view_render_plan, ZsTreePlatformStyle,
    ZsTreeRowRenderPlan, ZsTreeViewMetrics, ZsTreeViewRenderPlan,
};
pub use widget_render::{ZsBaseControlMetrics, ZsBaseControlPlatformStyle};
pub use window::{Window, WindowNativeOptions, WindowResolvedSpec, WindowSpec};
#[cfg(all(windows, feature = "windows-gdi"))]
pub use windows_gdi_renderer::{
    windows_no_flicker_paint_strategy, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette,
    WindowsGdiRenderer, WindowsGdiStyleResolver, WindowsGdiTextLayout,
    WindowsNoFlickerPaintStrategy,
};
#[cfg(all(windows, feature = "windows-win32"))]
pub use windows_win32_host::{
    clear_windows_win32_window_draw_plan, clear_windows_win32_window_draw_plans,
    clear_windows_win32_window_view_input_route, clear_windows_win32_window_view_input_routes,
    create_owned_windows_for_specs as create_owned_windows_win32_for_specs,
    create_owned_windows_for_specs_with_draw_plans as create_owned_windows_win32_for_specs_with_draw_plans,
    create_owned_windows_for_specs_with_draw_plans_and_input_routes as create_owned_windows_win32_for_specs_with_draw_plans_and_input_routes,
    create_windows_for_specs as create_windows_win32_for_specs,
    create_windows_for_specs_with_draw_plans as create_windows_win32_for_specs_with_draw_plans,
    create_windows_for_specs_with_draw_plans_and_input_routes as create_windows_win32_for_specs_with_draw_plans_and_input_routes,
    dispatch_windows_win32_window_menu_command, dispatch_windows_win32_window_view_click,
    dispatch_windows_win32_window_view_key_down, dispatch_windows_win32_window_view_scroll,
    dispatch_windows_win32_window_view_text_input, refresh_windows_win32_window_background_view,
    run_windows_win32_native_window_event_loop,
    run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items,
    run_windows_win32_native_window_event_loop_with_status_items,
    set_windows_win32_window_draw_plan, set_windows_win32_window_view_input_route,
    windows_system_theme_mode, windows_win32_main_window_style_plan,
    windows_win32_open_file_dialog, windows_win32_save_file_dialog,
    windows_win32_window_view_input_report, zsui_win32_default_window_proc, WindowsWin32ClassNames,
    WindowsWin32FileDialogService, WindowsWin32MainWindowHost, WindowsWin32MessageLoop,
    WindowsWin32MessageLoopResult, WindowsWin32OwnedAcceleratorTable,
    WindowsWin32OwnedAppIconResource, WindowsWin32OwnedIcon, WindowsWin32OwnedMainWindowHandles,
    WindowsWin32OwnedPopupMenu, WindowsWin32OwnedTrayIcon, WindowsWin32OwnedWindowMenu,
    WindowsWin32StatusItemHost, WindowsWin32StatusMenuCommandEntry,
    WindowsWin32StatusMenuCommandTable, WindowsWin32TransientWindowHost,
    WindowsWin32ViewInputDispatchReport, WindowsWin32ViewInputRoute, WindowsWin32WindowStylePlan,
    WindowsWindowCreateParams, WindowsWindowRole, ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID,
    ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS, ZSUI_WIN32_TRAY_CALLBACK_MESSAGE,
};
#[cfg(all(windows, feature = "windows-win32", feature = "document-shell"))]
pub use windows_win32_text_editor::WindowsWin32OwnedTextEditor;
#[cfg(feature = "workbench")]
pub use workbench::{
    zs_workbench_event_for_region, zs_workbench_layout, zs_workbench_native_draw_plan,
    ZsWorkbenchActionSpec, ZsWorkbenchBlockLayout, ZsWorkbenchComposerSpec,
    ZsWorkbenchContentBlock, ZsWorkbenchConversationGroupSpec, ZsWorkbenchConversationSpec,
    ZsWorkbenchIcon, ZsWorkbenchInspectorSpec, ZsWorkbenchInteractionEvent,
    ZsWorkbenchInteractionUpdate, ZsWorkbenchLayoutMetrics, ZsWorkbenchLayoutPlan,
    ZsWorkbenchLayoutRegion, ZsWorkbenchMessageLayout, ZsWorkbenchMessageRole,
    ZsWorkbenchMessageSpec, ZsWorkbenchNoticeLevel, ZsWorkbenchRegionKind, ZsWorkbenchRuntime,
    ZsWorkbenchSidebarSpec, ZsWorkbenchSpec, ZsWorkbenchToolStatus,
    ZS_WORKBENCH_BASE_SIDEBAR_WIDTH, ZS_WORKBENCH_COLLAPSED_SIDEBAR_WIDTH,
    ZS_WORKBENCH_COMPOSER_HEIGHT, ZS_WORKBENCH_CONTENT_MAX_WIDTH, ZS_WORKBENCH_INSPECTOR_WIDTH,
    ZS_WORKBENCH_TOP_BAR_HEIGHT,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluent_declaration_registers_window_tray_and_hotkey() {
        let mut host = MemoryHost::new();

        let runtime = app("Example")
            .window(Window::new("Example").size(900, 620))
            .tray(
                TraySpec::new()
                    .tooltip("Example")
                    .item("Open", Command::ShowMainWindow)
                    .separator()
                    .item("Quit", Command::Quit),
            )
            .global_hotkey("Alt+V", Command::OpenQuickPanel)
            .run_with_host(&mut host)
            .expect("memory host should accept the demo declaration");

        assert_eq!(runtime.app_name, "Example");
        assert_eq!(host.windows()[0].spec.title, "Example");
        assert_eq!(host.windows()[0].spec.width, 900);
        assert_eq!(host.trays()[0].spec.menu.items.len(), 3);
        assert_eq!(host.hotkeys()[0].spec.accelerator, "Alt+V");
    }

    #[test]
    fn unsupported_host_capability_returns_error_instead_of_panicking() {
        let capabilities = HostCapabilities::all_unsupported(PlatformName::Unknown);
        let mut host = MemoryHost::with_capabilities(capabilities);

        let err = app("Example")
            .window(WindowSpec::new("Example"))
            .run_with_host(&mut host)
            .expect_err("unsupported window creation should be reported");

        assert!(matches!(err, ZsuiError::Unsupported { .. }));
    }

    #[test]
    fn window_alias_supports_standard_builder_shape() {
        let window = Window::new("Example")
            .size(900, 620)
            .min_size(640, 420)
            .resizable(true)
            .decorations(true);

        assert_eq!(window.title, "Example");
        assert_eq!(window.width, 900);
        assert_eq!(window.height, 620);
        assert_eq!(window.min_width, Some(640));
        assert!(window.resizable);
        assert!(window.decorations);
    }

    #[test]
    fn window_native_options_snapshot_matches_builder_fields() {
        let options = Window::new("Example")
            .min_size(640, 420)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .transparent(true)
            .native_options();

        assert_eq!(options.min_width, Some(640));
        assert_eq!(options.min_height, Some(420));
        assert!(!options.resizable);
        assert!(!options.decorations);
        assert!(options.always_on_top);
        assert!(options.transparent);
    }

    #[test]
    fn native_window_host_capabilities_do_not_overstate_backend_completion() {
        let windows = HostCapabilities::windows_native_window_host();
        assert_eq!(windows.windows.status, CapabilityStatus::Supported);
        assert_eq!(windows.window_resizing.status, CapabilityStatus::Supported);
        assert_eq!(
            windows.window_decorations.status,
            CapabilityStatus::Supported
        );
        assert_eq!(
            windows.window_always_on_top.status,
            CapabilityStatus::Supported
        );
        assert_eq!(
            windows.window_transparency.status,
            CapabilityStatus::Unsupported
        );
        assert_eq!(windows.menus.status, CapabilityStatus::Supported);
        assert_eq!(
            windows.clipboard_text.status,
            if cfg!(feature = "clipboard") {
                CapabilityStatus::Supported
            } else {
                CapabilityStatus::Unsupported
            }
        );

        let macos = HostCapabilities::macos_native_window_host();
        assert_eq!(macos.windows.status, CapabilityStatus::Partial);
        assert_eq!(macos.window_resizing.status, CapabilityStatus::Partial);
        assert_eq!(macos.window_decorations.status, CapabilityStatus::Partial);
        assert_eq!(macos.window_always_on_top.status, CapabilityStatus::Partial);
        assert_eq!(
            macos.window_transparency.status,
            CapabilityStatus::Unsupported
        );
        assert_eq!(
            macos.menus.status,
            if cfg!(feature = "macos-appkit") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert_eq!(
            macos.clipboard_text.status,
            if cfg!(feature = "macos-appkit") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert_eq!(
            macos.file_picker.status,
            if cfg!(feature = "macos-appkit") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );

        let linux = HostCapabilities::linux_native_window_host();
        assert_eq!(linux.windows.status, CapabilityStatus::Partial);
        assert_eq!(linux.window_resizing.status, CapabilityStatus::Partial);
        assert_eq!(linux.window_decorations.status, CapabilityStatus::Partial);
        assert_eq!(
            linux.window_always_on_top.status,
            CapabilityStatus::Unsupported
        );
        assert_eq!(
            linux.window_transparency.status,
            CapabilityStatus::Unsupported
        );
        assert_eq!(
            linux.menus.status,
            if cfg!(all(feature = "linux-gtk", not(feature = "linux-direct"))) {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert_eq!(
            linux.clipboard_text.status,
            if cfg!(any(
                all(feature = "linux-direct", feature = "clipboard"),
                feature = "linux-gtk"
            )) {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert_eq!(
            linux.file_picker.status,
            if cfg!(any(feature = "linux-direct", feature = "linux-gtk")) {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );

        for capabilities in [
            HostCapabilities::windows_native_window_host(),
            HostCapabilities::macos_native_window_host(),
            HostCapabilities::linux_native_window_host(),
        ] {
            let resolved = Window::new("Example")
                .transparent(true)
                .resolve_for(&capabilities);
            assert!(resolved.requested.transparent);
            assert!(!resolved.effective.transparent);
        }
    }

    #[test]
    fn mobile_platform_capabilities_are_explicit_scaffolds() {
        assert_eq!(PlatformName::Android.as_str(), "android");
        assert_eq!(PlatformName::Harmony.as_str(), "harmony");

        let android = HostCapabilities::android_scaffold();
        assert_eq!(android.platform, PlatformName::Android);
        assert_eq!(android.windows.status, CapabilityStatus::Partial);
        assert_eq!(
            HostCapabilities::android_native_window_host()
                .windows
                .status,
            CapabilityStatus::Unsupported
        );

        let harmony = HostCapabilities::harmony_scaffold();
        assert_eq!(harmony.platform, PlatformName::Harmony);
        assert_eq!(harmony.windows.status, CapabilityStatus::Partial);
        assert_eq!(
            HostCapabilities::harmony_native_window_host()
                .windows
                .status,
            CapabilityStatus::Unsupported
        );
    }

    #[test]
    fn unsupported_window_traits_resolve_to_standard_native_fallbacks() {
        let mut capabilities = HostCapabilities::all_supported(PlatformName::Unknown);
        capabilities.window_resizing = CapabilitySupport::unsupported("resize policy unavailable");
        capabilities.window_decorations =
            CapabilitySupport::unsupported("decoration policy unavailable");
        capabilities.window_always_on_top = CapabilitySupport::unsupported("topmost unavailable");
        capabilities.window_transparency =
            CapabilitySupport::unsupported("transparency unavailable");

        let resolved = Window::new("Example")
            .min_size(640, 420)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .transparent(true)
            .resolve_for(&capabilities);

        assert!(!resolved.requested.resizable);
        assert!(!resolved.requested.decorations);
        assert!(resolved.requested.always_on_top);
        assert!(resolved.requested.transparent);
        assert!(resolved.effective.resizable);
        assert_eq!(resolved.effective.min_width, None);
        assert!(resolved.effective.decorations);
        assert!(!resolved.effective.always_on_top);
        assert!(!resolved.effective.transparent);
    }

    #[test]
    fn memory_host_records_requested_and_effective_window_specs() {
        let mut capabilities = HostCapabilities::all_supported(PlatformName::Unknown);
        capabilities.window_always_on_top = CapabilitySupport::unsupported("topmost unavailable");
        capabilities.window_transparency =
            CapabilitySupport::unsupported("transparency unavailable");
        let mut host = MemoryHost::with_capabilities(capabilities);

        app("Example")
            .window(Window::new("Example").always_on_top(true).transparent(true))
            .run_with_host(&mut host)
            .expect("window should fall back instead of failing");

        let record = &host.windows()[0];
        assert!(record.spec.always_on_top);
        assert!(record.spec.transparent);
        assert!(!record.effective_spec.always_on_top);
        assert!(!record.effective_spec.transparent);
        assert!(record
            .degraded_capabilities
            .iter()
            .any(|detail| detail.contains("window_always_on_top")));
    }

    #[test]
    fn requested_window_features_report_host_degradation() {
        let mut host = MemoryHost::with_capabilities(HostCapabilities::linux_scaffold());

        let runtime = app("Example")
            .window(Window::new("Example").always_on_top(true).transparent(true))
            .run_with_host(&mut host)
            .expect("partial Linux scaffold should accept window declarations");

        assert!(runtime
            .degraded_capabilities
            .iter()
            .any(|detail| detail.contains("window[0].window_always_on_top")));
        assert!(runtime
            .degraded_capabilities
            .iter()
            .any(|detail| detail.contains("window[0].window_transparency")));
    }

    #[test]
    fn specs_are_serializable_for_ai_and_tooling_contexts() {
        let spec = TraySpec::new()
            .item("Open", Command::ShowMainWindow)
            .item("Settings", Command::OpenSettings);

        let json = serde_json::to_string(&spec).expect("tray spec should serialize");
        assert!(json.contains("ShowMainWindow"));
        assert!(json.contains("OpenSettings"));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn window_can_carry_a_declarative_component_tree() {
        let mut host = MemoryHost::new();
        let content = UiNode::column("root")
            .gap(10)
            .child(UiNode::text("title", "Example"))
            .child(UiNode::button(
                "refresh",
                "Refresh",
                Command::custom("example.refresh"),
            ));

        app("Example")
            .window(Window::new("Example").content(content))
            .run_with_host(&mut host)
            .expect("component tree should be a valid window declaration");

        let content = host.windows()[0]
            .spec
            .content
            .as_ref()
            .expect("content tree should be recorded");
        assert_eq!(content.node_count(), 3);
        assert!(content.contains_node_id("refresh"));
    }
}
