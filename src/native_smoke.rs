use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    native_host_launch_plan_for_platform, native_ui_backend_for_platform,
    native_ui_platform_for_current_target, HostCapabilities, NativeHostLaunchMode,
    NativeUiBackendStatus, NativeUiPlatform, NativeWindowSmokeRunReport, ZsuiError, ZsuiResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NativeHostSmokeArtifactKind {
    ManifestJson,
    LaunchLog,
    WindowScreenshot,
    InteractionLog,
    CapabilityReport,
    AgentContextJson,
}

impl NativeHostSmokeArtifactKind {
    pub const fn kind_name(self) -> &'static str {
        match self {
            Self::ManifestJson => "manifest_json",
            Self::LaunchLog => "launch_log",
            Self::WindowScreenshot => "window_screenshot",
            Self::InteractionLog => "interaction_log",
            Self::CapabilityReport => "capability_report",
            Self::AgentContextJson => "agent_context_json",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokeArtifactRequirement {
    pub file_name: &'static str,
    pub kind: NativeHostSmokeArtifactKind,
    pub kind_name: &'static str,
    pub required_for_target_smoke: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokePlan {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub toolkit_name: &'static str,
    pub backend_status_name: &'static str,
    pub launch_mode_name: &'static str,
    pub artifact_dir: String,
    pub manifest_file: String,
    pub manifest_command: String,
    pub artifact_record_command: String,
    pub interactive_launch_command: Option<String>,
    pub can_run_on_current_target: bool,
    pub target_smoke_ready: bool,
    pub blocking_reason: Option<String>,
    pub artifact_requirements: Vec<NativeHostSmokeArtifactRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokeInteractionReport {
    pub platform_name: &'static str,
    pub launch_mode_name: &'static str,
    pub artifact_writer_opened_real_window: bool,
    pub native_window_created_count: usize,
    pub close_requested_count: usize,
    pub exited_by_auto_close: bool,
    pub native_window_events: Vec<String>,
    pub screenshot_captured: bool,
    pub draw_plan_requested: bool,
    pub draw_plan_window_count: usize,
    pub high_contrast_draw_plan_window_count: usize,
    pub draw_command_count: usize,
    pub text_command_count: usize,
    pub window_menu_requested_count: usize,
    pub window_menu_attached_count: usize,
    pub window_menu_native_command_count: usize,
    pub window_menu_command_routed: bool,
    pub window_menu_command_error: Option<String>,
    pub native_view_hit_target_count: usize,
    pub native_view_click_count: usize,
    pub native_view_event_count: usize,
    pub native_view_message_count: usize,
    pub native_view_ui_command_count: usize,
    pub native_view_ui_command_executed_count: usize,
    pub native_view_ui_command_failed_count: usize,
    pub native_view_ui_command_unhandled_count: usize,
    pub native_view_ui_command_event_count: usize,
    pub native_view_ui_command_errors: Vec<String>,
    pub native_view_app_command_count: usize,
    pub native_view_app_command_executed_count: usize,
    pub native_view_app_command_failed_count: usize,
    pub native_view_app_command_unhandled_count: usize,
    pub native_view_app_command_event_count: usize,
    pub native_view_app_command_names: Vec<&'static str>,
    pub native_view_app_command_errors: Vec<String>,
    pub native_view_ui_command_ids: Vec<&'static str>,
    pub native_view_live_revision: u64,
    pub native_view_quit_requested: bool,
    pub native_view_unhandled_click_count: usize,
    pub native_view_focus_count: usize,
    pub native_view_focus_visual_count: usize,
    pub native_view_focus_traversal_count: usize,
    pub native_view_text_input_count: usize,
    pub native_view_text_navigation_count: usize,
    pub native_view_text_selection_change_count: usize,
    pub native_view_text_caret: Option<usize>,
    pub native_view_pointer_down_count: usize,
    pub native_view_pointer_move_count: usize,
    pub native_view_pointer_up_count: usize,
    pub native_view_pointer_visual_change_count: usize,
    pub native_view_text_drag_count: usize,
    pub native_view_slider_value_change_count: usize,
    pub native_view_slider_keyboard_change_count: usize,
    pub native_view_slider_drag_count: usize,
    pub native_view_color_picker_value_change_count: usize,
    pub native_view_color_picker_channel_change_count: usize,
    pub native_view_color_picker_expanded_change_count: usize,
    pub native_view_color_picker_drag_count: usize,
    pub native_view_radio_selection_count: usize,
    pub native_view_radio_keyboard_selection_count: usize,
    pub native_view_radio_keyboard_focus_only_count: usize,
    pub native_view_auto_suggest_expanded_change_count: usize,
    pub native_view_auto_suggest_highlight_change_count: usize,
    pub native_view_auto_suggest_submit_count: usize,
    pub native_view_auto_suggest_clear_count: usize,
    pub native_view_tree_expansion_change_count: usize,
    pub native_view_tree_selection_count: usize,
    pub native_view_tree_invoke_count: usize,
    pub native_view_grid_view_selection_count: usize,
    pub native_view_grid_view_invoke_count: usize,
    pub native_view_table_sort_count: usize,
    pub native_view_table_selection_count: usize,
    pub native_view_table_invoke_count: usize,
    pub native_view_content_dialog_focus_count: usize,
    pub native_view_content_dialog_response_count: usize,
    pub native_view_command_palette_query_change_count: usize,
    pub native_view_command_palette_highlight_change_count: usize,
    pub native_view_command_palette_invoke_count: usize,
    pub native_view_command_palette_open_change_count: usize,
    pub native_view_command_palette_clear_count: usize,
    pub native_view_toast_focus_count: usize,
    pub native_view_toast_response_count: usize,
    pub native_view_toast_timeout_count: usize,
    pub native_view_info_bar_focus_count: usize,
    pub native_view_info_bar_event_count: usize,
    pub native_view_teaching_tip_focus_count: usize,
    pub native_view_teaching_tip_response_count: usize,
    pub native_view_breadcrumb_focus_count: usize,
    pub native_view_breadcrumb_expanded_change_count: usize,
    pub native_view_breadcrumb_selection_count: usize,
    pub native_view_combo_expanded_change_count: usize,
    pub native_view_combo_selection_count: usize,
    pub native_view_combo_keyboard_selection_count: usize,
    pub native_view_combo_type_ahead_match_count: usize,
    pub native_view_combo_scroll_count: usize,
    pub native_view_tab_selection_count: usize,
    pub native_view_tab_keyboard_selection_count: usize,
    pub native_view_tab_keyboard_focus_only_count: usize,
    pub native_view_toggle_count: usize,
    pub native_view_selection_count: usize,
    pub native_view_keyboard_selection_count: usize,
    pub native_view_key_down_count: usize,
    pub native_view_keyboard_activation_count: usize,
    pub native_view_unhandled_key_count: usize,
    pub native_view_scroll_count: usize,
    pub native_view_unhandled_scroll_count: usize,
    pub status_item_requested: bool,
    pub status_item_created: bool,
    pub status_item_menu_item_count: usize,
    pub status_item_error: Option<String>,
    pub status_menu_native_command_count: usize,
    pub status_menu_command_routed: bool,
    pub status_menu_command_error: Option<String>,
    pub status_menu_popup_created: bool,
    pub status_menu_popup_command_count: usize,
    pub status_menu_popup_destroyed: bool,
    pub status_menu_popup_error: Option<String>,
    pub interaction_artifacts_captured: bool,
    pub notes: Vec<String>,
}

impl NativeHostSmokeInteractionReport {
    pub fn contract_only(platform_name: &'static str, launch_mode_name: &'static str) -> Self {
        Self {
            platform_name,
            launch_mode_name,
            artifact_writer_opened_real_window: false,
            native_window_created_count: 0,
            close_requested_count: 0,
            exited_by_auto_close: false,
            native_window_events: Vec::new(),
            screenshot_captured: false,
            draw_plan_requested: false,
            draw_plan_window_count: 0,
            high_contrast_draw_plan_window_count: 0,
            draw_command_count: 0,
            text_command_count: 0,
            window_menu_requested_count: 0,
            window_menu_attached_count: 0,
            window_menu_native_command_count: 0,
            window_menu_command_routed: false,
            window_menu_command_error: None,
            native_view_hit_target_count: 0,
            native_view_click_count: 0,
            native_view_event_count: 0,
            native_view_message_count: 0,
            native_view_ui_command_count: 0,
            native_view_ui_command_executed_count: 0,
            native_view_ui_command_failed_count: 0,
            native_view_ui_command_unhandled_count: 0,
            native_view_ui_command_event_count: 0,
            native_view_ui_command_errors: Vec::new(),
            native_view_app_command_count: 0,
            native_view_app_command_executed_count: 0,
            native_view_app_command_failed_count: 0,
            native_view_app_command_unhandled_count: 0,
            native_view_app_command_event_count: 0,
            native_view_app_command_names: Vec::new(),
            native_view_app_command_errors: Vec::new(),
            native_view_ui_command_ids: Vec::new(),
            native_view_live_revision: 0,
            native_view_quit_requested: false,
            native_view_unhandled_click_count: 0,
            native_view_focus_count: 0,
            native_view_focus_visual_count: 0,
            native_view_focus_traversal_count: 0,
            native_view_text_input_count: 0,
            native_view_text_navigation_count: 0,
            native_view_text_selection_change_count: 0,
            native_view_text_caret: None,
            native_view_pointer_down_count: 0,
            native_view_pointer_move_count: 0,
            native_view_pointer_up_count: 0,
            native_view_pointer_visual_change_count: 0,
            native_view_text_drag_count: 0,
            native_view_slider_value_change_count: 0,
            native_view_slider_keyboard_change_count: 0,
            native_view_slider_drag_count: 0,
            native_view_color_picker_value_change_count: 0,
            native_view_color_picker_channel_change_count: 0,
            native_view_color_picker_expanded_change_count: 0,
            native_view_color_picker_drag_count: 0,
            native_view_radio_selection_count: 0,
            native_view_radio_keyboard_selection_count: 0,
            native_view_radio_keyboard_focus_only_count: 0,
            native_view_auto_suggest_expanded_change_count: 0,
            native_view_auto_suggest_highlight_change_count: 0,
            native_view_auto_suggest_submit_count: 0,
            native_view_auto_suggest_clear_count: 0,
            native_view_tree_expansion_change_count: 0,
            native_view_tree_selection_count: 0,
            native_view_tree_invoke_count: 0,
            native_view_grid_view_selection_count: 0,
            native_view_grid_view_invoke_count: 0,
            native_view_table_sort_count: 0,
            native_view_table_selection_count: 0,
            native_view_table_invoke_count: 0,
            native_view_content_dialog_focus_count: 0,
            native_view_content_dialog_response_count: 0,
            native_view_command_palette_query_change_count: 0,
            native_view_command_palette_highlight_change_count: 0,
            native_view_command_palette_invoke_count: 0,
            native_view_command_palette_open_change_count: 0,
            native_view_command_palette_clear_count: 0,
            native_view_toast_focus_count: 0,
            native_view_toast_response_count: 0,
            native_view_toast_timeout_count: 0,
            native_view_info_bar_focus_count: 0,
            native_view_info_bar_event_count: 0,
            native_view_teaching_tip_focus_count: 0,
            native_view_teaching_tip_response_count: 0,
            native_view_breadcrumb_focus_count: 0,
            native_view_breadcrumb_expanded_change_count: 0,
            native_view_breadcrumb_selection_count: 0,
            native_view_combo_expanded_change_count: 0,
            native_view_combo_selection_count: 0,
            native_view_combo_keyboard_selection_count: 0,
            native_view_combo_type_ahead_match_count: 0,
            native_view_combo_scroll_count: 0,
            native_view_tab_selection_count: 0,
            native_view_tab_keyboard_selection_count: 0,
            native_view_tab_keyboard_focus_only_count: 0,
            native_view_toggle_count: 0,
            native_view_selection_count: 0,
            native_view_keyboard_selection_count: 0,
            native_view_key_down_count: 0,
            native_view_keyboard_activation_count: 0,
            native_view_unhandled_key_count: 0,
            native_view_scroll_count: 0,
            native_view_unhandled_scroll_count: 0,
            status_item_requested: false,
            status_item_created: false,
            status_item_menu_item_count: 0,
            status_item_error: None,
            status_menu_native_command_count: 0,
            status_menu_command_routed: false,
            status_menu_command_error: None,
            status_menu_popup_created: false,
            status_menu_popup_command_count: 0,
            status_menu_popup_destroyed: false,
            status_menu_popup_error: None,
            interaction_artifacts_captured: false,
            notes: vec![
                "artifact writer records contract-level smoke context only".to_string(),
                "run interactive_launch_command on the target and capture window.png before target-smoke is complete".to_string(),
            ],
        }
    }

    pub fn from_native_window_smoke(
        platform_name: &'static str,
        launch_mode_name: &'static str,
        report: &NativeWindowSmokeRunReport,
    ) -> Self {
        let mut notes =
            vec!["native smoke runner opened a real native window and auto-closed it".to_string()];
        if report.screenshot_captured {
            notes.push("window.png was captured by the native smoke runner".to_string());
        } else {
            notes.push(
                "window.png still requires platform screenshot capture before target-smoke is complete"
                    .to_string(),
            );
        }
        if report.draw_plan_requested {
            notes.push(format!(
                "native draw plan attached to {} window(s) with {} command(s)",
                report.draw_plan_window_count, report.draw_command_count
            ));
        }
        if report.high_contrast_draw_plan_window_count > 0 {
            notes.push(format!(
                "native draw plan requested high-contrast rendering for {} window(s)",
                report.high_contrast_draw_plan_window_count
            ));
        }
        if report.window_menu_requested_count > 0 {
            notes.push(format!(
                "native window menu attached to {} window(s) with {} command(s)",
                report.window_menu_attached_count, report.window_menu_native_command_count
            ));
            if report.window_menu_command_routed {
                notes.push("window menu command routing was exercised".to_string());
            }
        }
        if report.native_view_click_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} click(s) into {} UI command(s)",
                report.native_view_event_count, report.native_view_ui_command_count
            ));
        }
        if report.native_view_ui_command_count > 0 {
            notes.push(format!(
                "UI command executor completed {}, failed {} and left {} unhandled",
                report.native_view_ui_command_executed_count,
                report.native_view_ui_command_failed_count,
                report.native_view_ui_command_unhandled_count
            ));
        }
        if report.native_view_text_input_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} text character(s)",
                report.native_view_text_input_count
            ));
        }
        if report.native_view_focus_traversal_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard focus traversal(s)",
                report.native_view_focus_traversal_count
            ));
        }
        if report.native_view_pointer_visual_change_count > 0 {
            notes.push(format!(
                "native view input smoke rendered {} transient pointer visual change(s)",
                report.native_view_pointer_visual_change_count
            ));
        }
        if report.native_view_toggle_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} toggle event(s)",
                report.native_view_toggle_count
            ));
        }
        if report.native_view_radio_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} radio selection event(s)",
                report.native_view_radio_selection_count
            ));
        }
        if report.native_view_radio_keyboard_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard radio selection event(s)",
                report.native_view_radio_keyboard_selection_count
            ));
        }
        if report.native_view_radio_keyboard_focus_only_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard radio focus-only event(s)",
                report.native_view_radio_keyboard_focus_only_count
            ));
        }
        if report.native_view_auto_suggest_expanded_change_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} auto-suggest expansion event(s)",
                report.native_view_auto_suggest_expanded_change_count
            ));
        }
        if report.native_view_auto_suggest_highlight_change_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} auto-suggest strong-id highlight event(s)",
                report.native_view_auto_suggest_highlight_change_count
            ));
        }
        if report.native_view_auto_suggest_submit_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} auto-suggest submission event(s)",
                report.native_view_auto_suggest_submit_count
            ));
        }
        if report.native_view_auto_suggest_clear_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} auto-suggest clear event(s)",
                report.native_view_auto_suggest_clear_count
            ));
        }
        if report.native_view_command_palette_query_change_count > 0 {
            notes.push(format!(
                "native command-palette query changes: {}",
                report.native_view_command_palette_query_change_count
            ));
        }
        if report.native_view_command_palette_highlight_change_count > 0 {
            notes.push(format!(
                "native command-palette strong-id highlight changes: {}",
                report.native_view_command_palette_highlight_change_count
            ));
        }
        if report.native_view_command_palette_invoke_count > 0 {
            notes.push(format!(
                "native command-palette typed invocations: {}",
                report.native_view_command_palette_invoke_count
            ));
        }
        if report.native_view_command_palette_open_change_count > 0 {
            notes.push(format!(
                "native command-palette open changes: {}",
                report.native_view_command_palette_open_change_count
            ));
        }
        if report.native_view_command_palette_clear_count > 0 {
            notes.push(format!(
                "native command-palette clear actions: {}",
                report.native_view_command_palette_clear_count
            ));
        }
        if report.native_view_tree_expansion_change_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} tree expansion event(s)",
                report.native_view_tree_expansion_change_count
            ));
        }
        if report.native_view_tree_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} strong-id tree selection event(s)",
                report.native_view_tree_selection_count
            ));
        }
        if report.native_view_tree_invoke_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} tree invocation event(s)",
                report.native_view_tree_invoke_count
            ));
        }
        if report.native_view_grid_view_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} strong-id grid-view selection event(s)",
                report.native_view_grid_view_selection_count
            ));
        }
        if report.native_view_grid_view_invoke_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} grid-view invocation event(s)",
                report.native_view_grid_view_invoke_count
            ));
        }
        if report.native_view_table_sort_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} typed table sort event(s)",
                report.native_view_table_sort_count
            ));
        }
        if report.native_view_table_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} strong-id table selection event(s)",
                report.native_view_table_selection_count
            ));
        }
        if report.native_view_table_invoke_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} table invocation event(s)",
                report.native_view_table_invoke_count
            ));
        }
        if report.native_view_content_dialog_focus_count > 0 {
            notes.push(format!(
                "native content-dialog semantic focus changes: {}",
                report.native_view_content_dialog_focus_count
            ));
        }
        if report.native_view_content_dialog_response_count > 0 {
            notes.push(format!(
                "native content-dialog typed responses: {}",
                report.native_view_content_dialog_response_count
            ));
        }
        if report.native_view_toast_focus_count > 0 {
            notes.push(format!(
                "native toast semantic focus changes: {}",
                report.native_view_toast_focus_count
            ));
        }
        if report.native_view_toast_response_count > 0 {
            notes.push(format!(
                "native toast typed responses: {}",
                report.native_view_toast_response_count
            ));
        }
        if report.native_view_toast_timeout_count > 0 {
            notes.push(format!(
                "native toast owned timeouts: {}",
                report.native_view_toast_timeout_count
            ));
        }
        if report.native_view_info_bar_focus_count > 0 {
            notes.push(format!(
                "native info-bar semantic focus changes: {}",
                report.native_view_info_bar_focus_count
            ));
        }
        if report.native_view_info_bar_event_count > 0 {
            notes.push(format!(
                "native info-bar typed events: {}",
                report.native_view_info_bar_event_count
            ));
        }
        if report.native_view_teaching_tip_focus_count > 0 {
            notes.push(format!(
                "native teaching-tip semantic focus changes: {}",
                report.native_view_teaching_tip_focus_count
            ));
        }
        if report.native_view_teaching_tip_response_count > 0 {
            notes.push(format!(
                "native teaching-tip typed responses: {}",
                report.native_view_teaching_tip_response_count
            ));
        }
        if report.native_view_breadcrumb_focus_count > 0 {
            notes.push(format!(
                "native breadcrumb semantic focus changes: {}",
                report.native_view_breadcrumb_focus_count
            ));
        }
        if report.native_view_breadcrumb_expanded_change_count > 0 {
            notes.push(format!(
                "native breadcrumb expansion changes: {}",
                report.native_view_breadcrumb_expanded_change_count
            ));
        }
        if report.native_view_breadcrumb_selection_count > 0 {
            notes.push(format!(
                "native breadcrumb typed selections: {}",
                report.native_view_breadcrumb_selection_count
            ));
        }
        if report.native_view_combo_expanded_change_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} combo expansion event(s)",
                report.native_view_combo_expanded_change_count
            ));
        }
        if report.native_view_combo_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} combo selection event(s)",
                report.native_view_combo_selection_count
            ));
        }
        if report.native_view_combo_keyboard_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard combo selection event(s)",
                report.native_view_combo_keyboard_selection_count
            ));
        }
        if report.native_view_combo_type_ahead_match_count > 0 {
            notes.push(format!(
                "native view input smoke matched {} combo type-ahead query(s)",
                report.native_view_combo_type_ahead_match_count
            ));
        }
        if report.native_view_combo_scroll_count > 0 {
            notes.push(format!(
                "native view input smoke scrolled {} combo popup window(s)",
                report.native_view_combo_scroll_count
            ));
        }
        if report.native_view_tab_selection_count > 0 {
            notes.push(format!(
                "native tab selection changes: {}",
                report.native_view_tab_selection_count
            ));
        }
        if report.native_view_tab_keyboard_selection_count > 0 {
            notes.push(format!(
                "native tab keyboard selection changes: {}",
                report.native_view_tab_keyboard_selection_count
            ));
        }
        if report.native_view_tab_keyboard_focus_only_count > 0 {
            notes.push(format!(
                "native tab keyboard focus-only moves: {}",
                report.native_view_tab_keyboard_focus_only_count
            ));
        }
        if report.native_view_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} list selection event(s)",
                report.native_view_selection_count
            ));
        }
        if report.native_view_keyboard_selection_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard list selection event(s)",
                report.native_view_keyboard_selection_count
            ));
        }
        if report.native_view_keyboard_activation_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} keyboard activation(s)",
                report.native_view_keyboard_activation_count
            ));
        }
        if report.native_view_scroll_count > 0 {
            notes.push(format!(
                "native view input smoke routed {} scroll event(s)",
                report.native_view_scroll_count
            ));
        }
        if report.status_item_requested && report.status_item_created {
            notes.push("status item was created by the native smoke runner".to_string());
            if report.status_menu_command_routed {
                notes.push("status menu command routing was exercised".to_string());
            }
            if report.status_menu_popup_created && report.status_menu_popup_destroyed {
                notes.push(
                    "status popup menu was created and destroyed by the native smoke runner"
                        .to_string(),
                );
            }
        } else if report.status_item_requested {
            notes.push(
                "status item was requested but still needs target host proof before completion"
                    .to_string(),
            );
        }

        Self {
            platform_name,
            launch_mode_name,
            artifact_writer_opened_real_window: report.visible_window_was_created(),
            native_window_created_count: report.created_window_count,
            close_requested_count: report.close_requested_count,
            exited_by_auto_close: report.exited_by_auto_close,
            native_window_events: report.events.clone(),
            screenshot_captured: report.screenshot_captured,
            draw_plan_requested: report.draw_plan_requested,
            draw_plan_window_count: report.draw_plan_window_count,
            high_contrast_draw_plan_window_count: report.high_contrast_draw_plan_window_count,
            draw_command_count: report.draw_command_count,
            text_command_count: report.text_command_count,
            window_menu_requested_count: report.window_menu_requested_count,
            window_menu_attached_count: report.window_menu_attached_count,
            window_menu_native_command_count: report.window_menu_native_command_count,
            window_menu_command_routed: report.window_menu_command_routed,
            window_menu_command_error: report.window_menu_command_error.clone(),
            native_view_hit_target_count: report.native_view_hit_target_count,
            native_view_click_count: report.native_view_click_count,
            native_view_event_count: report.native_view_event_count,
            native_view_message_count: report.native_view_message_count,
            native_view_ui_command_count: report.native_view_ui_command_count,
            native_view_ui_command_executed_count: report.native_view_ui_command_executed_count,
            native_view_ui_command_failed_count: report.native_view_ui_command_failed_count,
            native_view_ui_command_unhandled_count: report.native_view_ui_command_unhandled_count,
            native_view_ui_command_event_count: report.native_view_ui_command_event_count,
            native_view_ui_command_errors: report.native_view_ui_command_errors.clone(),
            native_view_app_command_count: report.native_view_app_command_count,
            native_view_app_command_executed_count: report.native_view_app_command_executed_count,
            native_view_app_command_failed_count: report.native_view_app_command_failed_count,
            native_view_app_command_unhandled_count: report.native_view_app_command_unhandled_count,
            native_view_app_command_event_count: report.native_view_app_command_event_count,
            native_view_app_command_names: report.native_view_app_command_names.clone(),
            native_view_app_command_errors: report.native_view_app_command_errors.clone(),
            native_view_ui_command_ids: report.native_view_ui_command_ids.clone(),
            native_view_live_revision: report.native_view_live_revision,
            native_view_quit_requested: report.native_view_quit_requested,
            native_view_unhandled_click_count: report.native_view_unhandled_click_count,
            native_view_focus_count: report.native_view_focus_count,
            native_view_focus_visual_count: report.native_view_focus_visual_count,
            native_view_focus_traversal_count: report.native_view_focus_traversal_count,
            native_view_text_input_count: report.native_view_text_input_count,
            native_view_text_navigation_count: report.native_view_text_navigation_count,
            native_view_text_selection_change_count: report.native_view_text_selection_change_count,
            native_view_text_caret: report.native_view_text_caret,
            native_view_pointer_down_count: report.native_view_pointer_down_count,
            native_view_pointer_move_count: report.native_view_pointer_move_count,
            native_view_pointer_up_count: report.native_view_pointer_up_count,
            native_view_pointer_visual_change_count: report.native_view_pointer_visual_change_count,
            native_view_text_drag_count: report.native_view_text_drag_count,
            native_view_slider_value_change_count: report.native_view_slider_value_change_count,
            native_view_slider_keyboard_change_count: report
                .native_view_slider_keyboard_change_count,
            native_view_slider_drag_count: report.native_view_slider_drag_count,
            native_view_color_picker_value_change_count: report
                .native_view_color_picker_value_change_count,
            native_view_color_picker_channel_change_count: report
                .native_view_color_picker_channel_change_count,
            native_view_color_picker_expanded_change_count: report
                .native_view_color_picker_expanded_change_count,
            native_view_color_picker_drag_count: report.native_view_color_picker_drag_count,
            native_view_radio_selection_count: report.native_view_radio_selection_count,
            native_view_radio_keyboard_selection_count: report
                .native_view_radio_keyboard_selection_count,
            native_view_radio_keyboard_focus_only_count: report
                .native_view_radio_keyboard_focus_only_count,
            native_view_auto_suggest_expanded_change_count: report
                .native_view_auto_suggest_expanded_change_count,
            native_view_auto_suggest_highlight_change_count: report
                .native_view_auto_suggest_highlight_change_count,
            native_view_auto_suggest_submit_count: report.native_view_auto_suggest_submit_count,
            native_view_auto_suggest_clear_count: report.native_view_auto_suggest_clear_count,
            native_view_tree_expansion_change_count: report.native_view_tree_expansion_change_count,
            native_view_tree_selection_count: report.native_view_tree_selection_count,
            native_view_tree_invoke_count: report.native_view_tree_invoke_count,
            native_view_grid_view_selection_count: report.native_view_grid_view_selection_count,
            native_view_grid_view_invoke_count: report.native_view_grid_view_invoke_count,
            native_view_table_sort_count: report.native_view_table_sort_count,
            native_view_table_selection_count: report.native_view_table_selection_count,
            native_view_table_invoke_count: report.native_view_table_invoke_count,
            native_view_content_dialog_focus_count: report.native_view_content_dialog_focus_count,
            native_view_content_dialog_response_count: report
                .native_view_content_dialog_response_count,
            native_view_command_palette_query_change_count: report
                .native_view_command_palette_query_change_count,
            native_view_command_palette_highlight_change_count: report
                .native_view_command_palette_highlight_change_count,
            native_view_command_palette_invoke_count: report
                .native_view_command_palette_invoke_count,
            native_view_command_palette_open_change_count: report
                .native_view_command_palette_open_change_count,
            native_view_command_palette_clear_count: report.native_view_command_palette_clear_count,
            native_view_toast_focus_count: report.native_view_toast_focus_count,
            native_view_toast_response_count: report.native_view_toast_response_count,
            native_view_toast_timeout_count: report.native_view_toast_timeout_count,
            native_view_info_bar_focus_count: report.native_view_info_bar_focus_count,
            native_view_info_bar_event_count: report.native_view_info_bar_event_count,
            native_view_teaching_tip_focus_count: report.native_view_teaching_tip_focus_count,
            native_view_teaching_tip_response_count: report.native_view_teaching_tip_response_count,
            native_view_breadcrumb_focus_count: report.native_view_breadcrumb_focus_count,
            native_view_breadcrumb_expanded_change_count: report
                .native_view_breadcrumb_expanded_change_count,
            native_view_breadcrumb_selection_count: report.native_view_breadcrumb_selection_count,
            native_view_combo_expanded_change_count: report.native_view_combo_expanded_change_count,
            native_view_combo_selection_count: report.native_view_combo_selection_count,
            native_view_combo_keyboard_selection_count: report
                .native_view_combo_keyboard_selection_count,
            native_view_combo_type_ahead_match_count: report
                .native_view_combo_type_ahead_match_count,
            native_view_combo_scroll_count: report.native_view_combo_scroll_count,
            native_view_tab_selection_count: report.native_view_tab_selection_count,
            native_view_tab_keyboard_selection_count: report
                .native_view_tab_keyboard_selection_count,
            native_view_tab_keyboard_focus_only_count: report
                .native_view_tab_keyboard_focus_only_count,
            native_view_toggle_count: report.native_view_toggle_count,
            native_view_selection_count: report.native_view_selection_count,
            native_view_keyboard_selection_count: report.native_view_keyboard_selection_count,
            native_view_key_down_count: report.native_view_key_down_count,
            native_view_keyboard_activation_count: report.native_view_keyboard_activation_count,
            native_view_unhandled_key_count: report.native_view_unhandled_key_count,
            native_view_scroll_count: report.native_view_scroll_count,
            native_view_unhandled_scroll_count: report.native_view_unhandled_scroll_count,
            status_item_requested: report.status_item_requested,
            status_item_created: report.status_item_created,
            status_item_menu_item_count: report.status_item_menu_item_count,
            status_item_error: report.status_item_error.clone(),
            status_menu_native_command_count: report.status_menu_native_command_count,
            status_menu_command_routed: report.status_menu_command_routed,
            status_menu_command_error: report.status_menu_command_error.clone(),
            status_menu_popup_created: report.status_menu_popup_created,
            status_menu_popup_command_count: report.status_menu_popup_command_count,
            status_menu_popup_destroyed: report.status_menu_popup_destroyed,
            status_menu_popup_error: report.status_menu_popup_error.clone(),
            interaction_artifacts_captured: report.visible_window_was_created(),
            notes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokeWriteReport {
    pub platform_name: &'static str,
    pub artifact_dir: String,
    pub written_files: Vec<String>,
    pub missing_required_artifacts: Vec<String>,
    pub target_smoke_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokeArtifactStatus {
    pub file_name: &'static str,
    pub kind: NativeHostSmokeArtifactKind,
    pub kind_name: &'static str,
    pub required_for_target_smoke: bool,
    pub path: String,
    pub exists: bool,
    pub byte_len: Option<u64>,
    pub non_empty: bool,
    pub json_valid: Option<bool>,
    pub validation_error: Option<String>,
}

impl NativeHostSmokeArtifactStatus {
    pub fn target_smoke_satisfied(&self) -> bool {
        self.exists
            && self.non_empty
            && self
                .json_valid
                .map(|json_valid| json_valid && self.validation_error.is_none())
                .unwrap_or_else(|| self.validation_error.is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeHostSmokeReviewReport {
    pub platform: NativeUiPlatform,
    pub platform_name: &'static str,
    pub artifact_dir: String,
    pub reviewed_at_unix_ms: u128,
    pub artifact_statuses: Vec<NativeHostSmokeArtifactStatus>,
    pub required_artifact_count: usize,
    pub present_required_artifact_count: usize,
    pub valid_required_artifact_count: usize,
    pub missing_required_artifacts: Vec<String>,
    pub invalid_required_artifacts: Vec<String>,
    pub target_smoke_complete: bool,
}

impl NativeHostSmokePlan {
    pub fn required_artifact_file_names(&self) -> Vec<&'static str> {
        self.artifact_requirements
            .iter()
            .filter(|artifact| artifact.required_for_target_smoke)
            .map(|artifact| artifact.file_name)
            .collect()
    }
}

pub fn native_host_smoke_artifact_requirements() -> Vec<NativeHostSmokeArtifactRequirement> {
    use NativeHostSmokeArtifactKind::{
        AgentContextJson, CapabilityReport, InteractionLog, LaunchLog, ManifestJson,
        WindowScreenshot,
    };

    vec![
        artifact_requirement(
            "manifest.json",
            ManifestJson,
            true,
            "serialized smoke plan for the target platform",
        ),
        artifact_requirement(
            "launch.log",
            LaunchLog,
            true,
            "native runtime launch output and exit status",
        ),
        artifact_requirement(
            "window.png",
            WindowScreenshot,
            true,
            "screenshot proving the native window was visible on the target",
        ),
        artifact_requirement(
            "interaction.json",
            InteractionLog,
            true,
            "structured record of close/menu/dialog/clipboard interactions attempted",
        ),
        artifact_requirement(
            "capabilities.json",
            CapabilityReport,
            true,
            "host capabilities observed on the target run",
        ),
        artifact_requirement(
            "agent-context.json",
            AgentContextJson,
            true,
            "matching zsui_agent_context_json output captured with the smoke run",
        ),
    ]
}

pub fn native_host_smoke_artifact_names() -> Vec<&'static str> {
    native_host_smoke_artifact_requirements()
        .iter()
        .map(|artifact| artifact.file_name)
        .collect()
}

pub fn native_host_smoke_command_names() -> Vec<&'static str> {
    vec![
        "native_smoke_manifest",
        "native_smoke_record",
        "native_smoke_run",
        "native_smoke_review",
    ]
}

pub fn native_host_smoke_plan(platform: NativeUiPlatform) -> Option<NativeHostSmokePlan> {
    let backend = native_ui_backend_for_platform(platform)?;
    let launch = native_host_launch_plan_for_platform(platform)?;
    let platform_name = platform.platform_name();
    let artifact_dir = format!("target/native-host-smoke/{platform_name}");
    let target_smoke_ready = launch.mode == NativeHostLaunchMode::RealNativeHost
        && backend.status != NativeUiBackendStatus::AdapterBoundaryScaffold;

    Some(NativeHostSmokePlan {
        platform,
        platform_name,
        toolkit_name: backend.toolkit_name(),
        backend_status_name: backend.status_name(),
        launch_mode_name: launch.mode_name(),
        manifest_file: format!("{artifact_dir}/manifest.json"),
        manifest_command: format!("cargo run --example native_smoke_manifest -- {platform_name}"),
        artifact_record_command: format!(
            "cargo run --example native_smoke_record -- {platform_name}"
        ),
        interactive_launch_command: if target_smoke_ready {
            Some(format!(
                "cargo run --example native_smoke_run -- {platform_name}"
            ))
        } else {
            None
        },
        can_run_on_current_target: native_ui_platform_for_current_target() == Some(platform),
        target_smoke_ready,
        blocking_reason: if target_smoke_ready {
            None
        } else {
            Some(format!(
                "{platform_name} is still `{}` and needs a real runtime host before target smoke",
                backend.status_name()
            ))
        },
        artifact_dir,
        artifact_requirements: native_host_smoke_artifact_requirements(),
    })
}

pub fn native_host_smoke_plan_with_artifact_root(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> Option<NativeHostSmokePlan> {
    let mut plan = native_host_smoke_plan(platform)?;
    let artifact_dir = artifact_root.as_ref().join(plan.platform_name);
    plan.artifact_dir = path_to_manifest_string(&artifact_dir);
    plan.manifest_file = path_to_manifest_string(artifact_dir.join("manifest.json"));
    Some(plan)
}

pub fn native_host_smoke_plan_for_current_target() -> Option<NativeHostSmokePlan> {
    native_host_smoke_plan(native_ui_platform_for_current_target()?)
}

pub fn native_host_smoke_plans() -> Vec<NativeHostSmokePlan> {
    crate::SUPPORTED_NATIVE_UI_PLATFORMS
        .iter()
        .filter_map(|platform| native_host_smoke_plan(*platform))
        .collect()
}

pub fn native_host_smoke_plan_json(
    platform: NativeUiPlatform,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&native_host_smoke_plan(platform))
}

pub fn native_host_smoke_plans_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&native_host_smoke_plans())
}

pub fn write_native_host_smoke_artifacts(
    platform: NativeUiPlatform,
) -> ZsuiResult<NativeHostSmokeWriteReport> {
    write_native_host_smoke_artifacts_to(platform, "target/native-host-smoke")
}

pub fn write_native_host_smoke_artifacts_to(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<NativeHostSmokeWriteReport> {
    let plan =
        native_host_smoke_plan_with_artifact_root(platform, &artifact_root).ok_or_else(|| {
            ZsuiError::unsupported(
                "native_host_smoke",
                format!("no smoke plan exists for `{}`", platform.platform_name()),
            )
        })?;
    let interaction =
        NativeHostSmokeInteractionReport::contract_only(plan.platform_name, plan.launch_mode_name);
    write_native_host_smoke_artifacts_with_interaction_to(platform, artifact_root, interaction)
}

pub fn write_native_host_smoke_artifacts_with_interaction_to(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
    interaction: NativeHostSmokeInteractionReport,
) -> ZsuiResult<NativeHostSmokeWriteReport> {
    let plan =
        native_host_smoke_plan_with_artifact_root(platform, artifact_root).ok_or_else(|| {
            ZsuiError::unsupported(
                "native_host_smoke",
                format!("no smoke plan exists for `{}`", platform.platform_name()),
            )
        })?;
    let artifact_dir = PathBuf::from(&plan.artifact_dir);
    fs::create_dir_all(&artifact_dir)
        .map_err(|err| smoke_io_error("create_artifact_dir", &artifact_dir, err))?;

    let mut written_files = Vec::new();
    write_json_artifact(&artifact_dir, "manifest.json", &plan, &mut written_files)?;
    write_json_artifact(
        &artifact_dir,
        "capabilities.json",
        &smoke_capabilities_for_platform(platform),
        &mut written_files,
    )?;
    let agent_context = crate::zsui_agent_context();
    write_json_artifact(
        &artifact_dir,
        "agent-context.json",
        &agent_context,
        &mut written_files,
    )?;
    write_json_artifact(
        &artifact_dir,
        "interaction.json",
        &interaction,
        &mut written_files,
    )?;
    write_text_artifact(
        &artifact_dir,
        "launch.log",
        &format!(
            "platform={}\ntoolkit={}\nbackend_status={}\nlaunch_mode={}\nrecorded_at_unix_ms={}\nreal_window_opened_by_artifact_writer={}\ninteractive_launch_command={}\n",
            plan.platform_name,
            plan.toolkit_name,
            plan.backend_status_name,
            plan.launch_mode_name,
            unix_ms_now(),
            interaction.artifact_writer_opened_real_window,
            plan.interactive_launch_command.as_deref().unwrap_or("none")
        ),
        &mut written_files,
    )?;

    let missing_required_artifacts: Vec<String> = plan
        .artifact_requirements
        .iter()
        .filter(|artifact| artifact.required_for_target_smoke)
        .filter(|artifact| !artifact_dir.join(artifact.file_name).exists())
        .map(|artifact| artifact.file_name.to_string())
        .collect();

    Ok(NativeHostSmokeWriteReport {
        platform_name: plan.platform_name,
        artifact_dir: plan.artifact_dir,
        written_files,
        target_smoke_complete: missing_required_artifacts.is_empty(),
        missing_required_artifacts,
    })
}

pub fn review_native_host_smoke_artifacts(
    platform: NativeUiPlatform,
) -> ZsuiResult<NativeHostSmokeReviewReport> {
    review_native_host_smoke_artifacts_at(platform, "target/native-host-smoke")
}

pub fn review_native_host_smoke_artifacts_at(
    platform: NativeUiPlatform,
    artifact_root: impl AsRef<Path>,
) -> ZsuiResult<NativeHostSmokeReviewReport> {
    let plan =
        native_host_smoke_plan_with_artifact_root(platform, artifact_root).ok_or_else(|| {
            ZsuiError::unsupported(
                "native_host_smoke_review",
                format!("no smoke plan exists for `{}`", platform.platform_name()),
            )
        })?;
    let artifact_dir = PathBuf::from(&plan.artifact_dir);
    let artifact_statuses: Vec<_> = plan
        .artifact_requirements
        .iter()
        .map(|requirement| review_smoke_artifact(&artifact_dir, requirement))
        .collect();
    let required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_target_smoke)
        .count();
    let present_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_target_smoke && artifact.exists)
        .count();
    let valid_required_artifact_count = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_target_smoke && artifact.target_smoke_satisfied())
        .count();
    let missing_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| artifact.required_for_target_smoke && !artifact.exists)
        .map(|artifact| artifact.file_name.to_string())
        .collect();
    let invalid_required_artifacts = artifact_statuses
        .iter()
        .filter(|artifact| {
            artifact.required_for_target_smoke
                && artifact.exists
                && !artifact.target_smoke_satisfied()
        })
        .map(|artifact| artifact.file_name.to_string())
        .collect();

    Ok(NativeHostSmokeReviewReport {
        platform,
        platform_name: plan.platform_name,
        artifact_dir: plan.artifact_dir,
        reviewed_at_unix_ms: unix_ms_now(),
        target_smoke_complete: valid_required_artifact_count == required_artifact_count,
        artifact_statuses,
        required_artifact_count,
        present_required_artifact_count,
        valid_required_artifact_count,
        missing_required_artifacts,
        invalid_required_artifacts,
    })
}

fn artifact_requirement(
    file_name: &'static str,
    kind: NativeHostSmokeArtifactKind,
    required_for_target_smoke: bool,
    description: &'static str,
) -> NativeHostSmokeArtifactRequirement {
    NativeHostSmokeArtifactRequirement {
        file_name,
        kind,
        kind_name: kind.kind_name(),
        required_for_target_smoke,
        description,
    }
}

fn review_smoke_artifact(
    artifact_dir: &Path,
    requirement: &NativeHostSmokeArtifactRequirement,
) -> NativeHostSmokeArtifactStatus {
    let path = artifact_dir.join(requirement.file_name);
    let metadata = fs::metadata(&path);
    let (exists, byte_len, mut validation_error) = match metadata {
        Ok(metadata) => {
            let byte_len = metadata.len();
            let validation_error = if byte_len == 0 {
                Some("artifact is empty".to_string())
            } else {
                None
            };
            (true, Some(byte_len), validation_error)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (false, None, None),
        Err(err) => (false, None, Some(err.to_string())),
    };
    let non_empty = byte_len.map(|len| len > 0).unwrap_or(false);
    let json_valid = if requirement.file_name.ends_with(".json") && exists && non_empty {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => Some(true),
                Err(err) => {
                    validation_error = Some(err.to_string());
                    Some(false)
                }
            },
            Err(err) => {
                validation_error = Some(err.to_string());
                Some(false)
            }
        }
    } else if requirement.file_name.ends_with(".json") && exists {
        Some(false)
    } else {
        None
    };
    if requirement.kind == NativeHostSmokeArtifactKind::WindowScreenshot && exists && non_empty {
        match fs::read(&path) {
            Ok(bytes) if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) => {}
            Ok(_) => validation_error = Some("window screenshot is not a PNG file".to_string()),
            Err(err) => validation_error = Some(err.to_string()),
        }
    }

    NativeHostSmokeArtifactStatus {
        file_name: requirement.file_name,
        kind: requirement.kind,
        kind_name: requirement.kind_name,
        required_for_target_smoke: requirement.required_for_target_smoke,
        path: path_to_manifest_string(path),
        exists,
        byte_len,
        non_empty,
        json_valid,
        validation_error,
    }
}

fn smoke_capabilities_for_platform(platform: NativeUiPlatform) -> HostCapabilities {
    match platform {
        NativeUiPlatform::Windows => HostCapabilities::windows_native_window_host(),
        NativeUiPlatform::Macos => HostCapabilities::macos_native_window_host(),
        NativeUiPlatform::Linux => HostCapabilities::linux_native_window_host(),
        NativeUiPlatform::Android => HostCapabilities::android_native_window_host(),
        NativeUiPlatform::Harmony => HostCapabilities::harmony_native_window_host(),
    }
}

fn write_json_artifact<T: Serialize>(
    artifact_dir: &Path,
    file_name: &str,
    value: &T,
    written_files: &mut Vec<String>,
) -> ZsuiResult<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| ZsuiError::host("serialize_smoke_artifact", err.to_string()))?;
    write_text_artifact(artifact_dir, file_name, &(json + "\n"), written_files)
}

fn write_text_artifact(
    artifact_dir: &Path,
    file_name: &str,
    content: &str,
    written_files: &mut Vec<String>,
) -> ZsuiResult<()> {
    let path = artifact_dir.join(file_name);
    fs::write(&path, content).map_err(|err| smoke_io_error("write_artifact", &path, err))?;
    written_files.push(path_to_manifest_string(path));
    Ok(())
}

fn smoke_io_error(operation: &str, path: &Path, err: std::io::Error) -> ZsuiError {
    ZsuiError::host(
        operation,
        format!("{}: {}", path_to_manifest_string(path), err),
    )
}

fn path_to_manifest_string(path: impl AsRef<Path>) -> String {
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

    #[test]
    fn desktop_smoke_plan_names_required_artifacts() {
        let plan = native_host_smoke_plan(NativeUiPlatform::Windows)
            .expect("windows smoke plan should exist");

        assert_eq!(plan.platform_name, "windows");
        assert_eq!(plan.toolkit_name, "win32_gdi");
        assert!(plan.target_smoke_ready);
        assert_eq!(plan.artifact_dir, "target/native-host-smoke/windows");
        assert!(plan.required_artifact_file_names().contains(&"window.png"));
        assert!(plan
            .manifest_command
            .contains("native_smoke_manifest -- windows"));
    }

    #[test]
    fn mobile_smoke_plan_is_blocked_until_runtime_host_exists() {
        let plan = native_host_smoke_plan(NativeUiPlatform::Android)
            .expect("android smoke plan should exist");

        assert!(!plan.target_smoke_ready);
        assert_eq!(plan.launch_mode_name, "contract_scaffold_fallback");
        assert!(plan.interactive_launch_command.is_none());
        assert!(plan
            .blocking_reason
            .as_deref()
            .unwrap_or_default()
            .contains("needs a real runtime host"));
    }

    #[test]
    fn smoke_plans_serialize_for_artifact_manifests() {
        let json = native_host_smoke_plans_json().expect("smoke plans should serialize");

        assert!(json.contains("target/native-host-smoke/windows"));
        assert!(json.contains("agent-context.json"));
        assert!(json.contains("harmony"));
    }

    #[test]
    fn smoke_artifact_writer_records_contract_files_without_faking_screenshot() {
        let root = std::env::temp_dir().join(format!(
            "zsui-smoke-artifacts-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));

        let report = write_native_host_smoke_artifacts_to(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should write");

        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("manifest.json")));
        assert!(report
            .written_files
            .iter()
            .any(|path| path.ends_with("agent-context.json")));
        assert!(report
            .missing_required_artifacts
            .contains(&"window.png".to_string()));
        assert!(!report.target_smoke_complete);
        assert!(root.join("windows").join("interaction.json").exists());
    }

    #[test]
    fn smoke_artifact_writer_accepts_real_window_interaction_report() {
        let root = std::env::temp_dir().join(format!(
            "zsui-smoke-run-artifacts-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        let run = NativeWindowSmokeRunReport {
            requested_window_count: 1,
            created_window_count: 1,
            window_menu_requested_count: 0,
            window_menu_attached_count: 0,
            window_menu_native_command_count: 0,
            window_menu_command_routed: false,
            window_menu_command_error: None,
            close_requested_count: 0,
            auto_close_after_ms: 10,
            exited_by_auto_close: true,
            startup_error: None,
            screenshot_file: None,
            screenshot_captured: false,
            screenshot_error: None,
            draw_plan_requested: true,
            draw_plan_window_count: 1,
            high_contrast_draw_plan_window_count: 1,
            draw_command_count: 3,
            text_command_count: 1,
            native_view_hit_target_count: 1,
            native_view_click_count: 1,
            native_view_event_count: 1,
            native_view_message_count: 1,
            native_view_ui_command_count: 1,
            native_view_ui_command_executed_count: 0,
            native_view_ui_command_failed_count: 0,
            native_view_ui_command_unhandled_count: 1,
            native_view_ui_command_event_count: 0,
            native_view_ui_command_errors: Vec::new(),
            native_view_app_command_count: 0,
            native_view_app_command_executed_count: 0,
            native_view_app_command_failed_count: 0,
            native_view_app_command_unhandled_count: 0,
            native_view_app_command_event_count: 0,
            native_view_app_command_names: Vec::new(),
            native_view_app_command_errors: Vec::new(),
            native_view_ui_command_ids: vec!["zsui.test.save"],
            native_view_live_revision: 0,
            native_view_quit_requested: false,
            native_view_unhandled_click_count: 0,
            native_view_focus_count: 1,
            native_view_focus_visual_count: 1,
            native_view_focus_traversal_count: 1,
            native_view_text_input_count: 0,
            native_view_text_navigation_count: 0,
            native_view_text_selection_change_count: 0,
            native_view_text_caret: None,
            native_view_pointer_down_count: 0,
            native_view_pointer_move_count: 0,
            native_view_pointer_up_count: 0,
            native_view_pointer_visual_change_count: 2,
            native_view_text_drag_count: 0,
            native_view_slider_value_change_count: 0,
            native_view_slider_keyboard_change_count: 0,
            native_view_slider_drag_count: 0,
            native_view_color_picker_value_change_count: 0,
            native_view_color_picker_channel_change_count: 0,
            native_view_color_picker_expanded_change_count: 0,
            native_view_color_picker_drag_count: 0,
            native_view_radio_selection_count: 0,
            native_view_radio_keyboard_selection_count: 1,
            native_view_radio_keyboard_focus_only_count: 1,
            native_view_auto_suggest_expanded_change_count: 0,
            native_view_auto_suggest_highlight_change_count: 0,
            native_view_auto_suggest_submit_count: 0,
            native_view_auto_suggest_clear_count: 0,
            native_view_tree_expansion_change_count: 0,
            native_view_tree_selection_count: 0,
            native_view_tree_invoke_count: 0,
            native_view_grid_view_selection_count: 1,
            native_view_grid_view_invoke_count: 1,
            native_view_table_sort_count: 0,
            native_view_table_selection_count: 0,
            native_view_table_invoke_count: 0,
            native_view_content_dialog_focus_count: 1,
            native_view_content_dialog_response_count: 1,
            native_view_command_palette_query_change_count: 0,
            native_view_command_palette_highlight_change_count: 0,
            native_view_command_palette_invoke_count: 0,
            native_view_command_palette_open_change_count: 0,
            native_view_command_palette_clear_count: 0,
            native_view_toast_focus_count: 1,
            native_view_toast_response_count: 1,
            native_view_toast_timeout_count: 1,
            native_view_info_bar_focus_count: 1,
            native_view_info_bar_event_count: 1,
            native_view_teaching_tip_focus_count: 1,
            native_view_teaching_tip_response_count: 1,
            native_view_breadcrumb_focus_count: 1,
            native_view_breadcrumb_expanded_change_count: 2,
            native_view_breadcrumb_selection_count: 1,
            native_view_combo_expanded_change_count: 0,
            native_view_combo_selection_count: 0,
            native_view_combo_keyboard_selection_count: 0,
            native_view_combo_type_ahead_match_count: 0,
            native_view_combo_scroll_count: 1,
            native_view_tab_selection_count: 1,
            native_view_tab_keyboard_selection_count: 1,
            native_view_tab_keyboard_focus_only_count: 1,
            native_view_toggle_count: 0,
            native_view_selection_count: 0,
            native_view_keyboard_selection_count: 0,
            native_view_key_down_count: 0,
            native_view_keyboard_activation_count: 0,
            native_view_unhandled_key_count: 0,
            native_view_scroll_count: 0,
            native_view_unhandled_scroll_count: 0,
            status_item_requested: true,
            status_item_required: false,
            status_item_created: true,
            status_item_menu_item_count: 2,
            status_item_error: None,
            status_menu_native_command_count: 2,
            status_menu_command_routed: true,
            status_menu_command_error: None,
            status_menu_popup_created: true,
            status_menu_popup_command_count: 2,
            status_menu_popup_destroyed: true,
            status_menu_popup_error: None,
            events: vec![
                "window_created:Smoke".to_string(),
                "status_item_created:1".to_string(),
                "status_menu_command_dispatched:ShowMainWindow".to_string(),
                "status_menu_popup_created:2".to_string(),
                "status_menu_popup_destroyed".to_string(),
                "auto_close_elapsed".to_string(),
            ],
        };
        let interaction = NativeHostSmokeInteractionReport::from_native_window_smoke(
            "windows",
            "real_native_host",
            &run,
        );

        let report = write_native_host_smoke_artifacts_with_interaction_to(
            NativeUiPlatform::Windows,
            &root,
            interaction,
        )
        .expect("smoke artifacts should write");
        let interaction_json = fs::read_to_string(root.join("windows").join("interaction.json"))
            .expect("interaction artifact should be readable");

        assert!(report
            .missing_required_artifacts
            .contains(&"window.png".to_string()));
        assert!(interaction_json.contains("\"artifact_writer_opened_real_window\": true"));
        assert!(interaction_json.contains("\"status_item_created\": true"));
        assert!(interaction_json.contains("\"status_menu_command_routed\": true"));
        assert!(interaction_json.contains("\"status_menu_popup_destroyed\": true"));
        assert!(interaction_json.contains("\"native_view_ui_command_count\": 1"));
        assert!(interaction_json.contains("\"native_view_ui_command_executed_count\": 0"));
        assert!(interaction_json.contains("\"native_view_ui_command_unhandled_count\": 1"));
        assert!(interaction_json.contains("\"native_view_ui_command_errors\": []"));
        assert!(interaction_json.contains("\"native_view_live_revision\": 0"));
        assert!(interaction_json.contains("\"native_view_focus_visual_count\": 1"));
        assert!(interaction_json.contains("\"high_contrast_draw_plan_window_count\": 1"));
        assert!(interaction_json.contains("\"native_view_pointer_visual_change_count\": 2"));
        assert!(interaction_json.contains("\"native_view_radio_keyboard_selection_count\": 1"));
        assert!(interaction_json.contains("\"native_view_radio_keyboard_focus_only_count\": 1"));
        assert!(interaction_json.contains("\"native_view_content_dialog_focus_count\": 1"));
        assert!(interaction_json.contains("\"native_view_content_dialog_response_count\": 1"));
        assert!(interaction_json.contains("\"native_view_toast_focus_count\": 1"));
        assert!(interaction_json.contains("\"native_view_toast_response_count\": 1"));
        assert!(interaction_json.contains("\"native_view_toast_timeout_count\": 1"));
        assert!(interaction_json.contains("\"native_view_info_bar_focus_count\": 1"));
        assert!(interaction_json.contains("\"native_view_info_bar_event_count\": 1"));
        assert!(interaction_json.contains("\"native_view_teaching_tip_focus_count\": 1"));
        assert!(interaction_json.contains("\"native_view_teaching_tip_response_count\": 1"));
        assert!(interaction_json.contains("\"native_view_breadcrumb_focus_count\": 1"));
        assert!(interaction_json.contains("\"native_view_breadcrumb_expanded_change_count\": 2"));
        assert!(interaction_json.contains("\"native_view_breadcrumb_selection_count\": 1"));
        assert!(interaction_json.contains("\"native_view_grid_view_selection_count\": 1"));
        assert!(interaction_json.contains("\"native_view_grid_view_invoke_count\": 1"));
        assert!(interaction_json.contains("\"native_view_combo_scroll_count\": 1"));
        assert!(interaction_json.contains("\"native_view_tab_selection_count\": 1"));
        assert!(interaction_json.contains("\"native_view_tab_keyboard_selection_count\": 1"));
        assert!(interaction_json.contains("\"native_view_tab_keyboard_focus_only_count\": 1"));
        assert!(interaction_json.contains("zsui.test.save"));
        assert!(interaction_json.contains("auto_close_elapsed"));
    }

    #[test]
    fn smoke_review_reports_missing_screenshot_without_marking_complete() {
        let root = std::env::temp_dir().join(format!(
            "zsui-smoke-review-artifacts-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        write_native_host_smoke_artifacts_to(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should write");

        let report = review_native_host_smoke_artifacts_at(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should review");

        assert_eq!(report.required_artifact_count, 6);
        assert_eq!(report.present_required_artifact_count, 5);
        assert_eq!(report.valid_required_artifact_count, 5);
        assert!(report
            .missing_required_artifacts
            .contains(&"window.png".to_string()));
        assert!(report.invalid_required_artifacts.is_empty());
        assert!(!report.target_smoke_complete);
        assert!(report
            .artifact_statuses
            .iter()
            .any(|artifact| artifact.file_name == "manifest.json"
                && artifact.json_valid == Some(true)));
    }

    #[test]
    fn smoke_review_rejects_invalid_json_artifact() {
        let root = std::env::temp_dir().join(format!(
            "zsui-smoke-review-invalid-json-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        write_native_host_smoke_artifacts_to(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should write");
        fs::write(root.join("windows").join("interaction.json"), "not json")
            .expect("interaction artifact should be replaceable");

        let report = review_native_host_smoke_artifacts_at(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should review");

        assert!(report
            .invalid_required_artifacts
            .contains(&"interaction.json".to_string()));
        assert!(!report.target_smoke_complete);
        assert!(report
            .artifact_statuses
            .iter()
            .any(|artifact| artifact.file_name == "interaction.json"
                && artifact.json_valid == Some(false)
                && artifact.validation_error.is_some()));
    }

    #[test]
    fn smoke_review_rejects_invalid_screenshot_artifact() {
        let root = std::env::temp_dir().join(format!(
            "zsui-smoke-review-invalid-png-{}-{}",
            std::process::id(),
            unix_ms_now()
        ));
        write_native_host_smoke_artifacts_to(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should write");
        fs::write(root.join("windows").join("window.png"), "not png")
            .expect("screenshot artifact should be replaceable");

        let report = review_native_host_smoke_artifacts_at(NativeUiPlatform::Windows, &root)
            .expect("smoke artifacts should review");

        assert!(report
            .invalid_required_artifacts
            .contains(&"window.png".to_string()));
        assert!(!report.target_smoke_complete);
        assert!(report
            .artifact_statuses
            .iter()
            .any(|artifact| artifact.file_name == "window.png"
                && artifact.validation_error.as_deref()
                    == Some("window screenshot is not a PNG file")));
    }
}
