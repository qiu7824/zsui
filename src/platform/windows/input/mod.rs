#[derive(Debug, Clone)]
pub struct WindowsWin32ViewInputRoute {
    shared_runtime: crate::native::NativeViewInputRuntime,
    shared_text_drag_active: bool,
    #[cfg(feature = "canvas")]
    shared_canvas_pointer_drag_active: bool,
    #[cfg(feature = "canvas")]
    shared_canvas_pointer_drag_moved: bool,
    #[cfg(feature = "slider")]
    shared_slider_drag_active: bool,
    #[cfg(feature = "color-picker")]
    shared_color_picker_drag_active: bool,
    pending_utf16_high_surrogate: Option<u16>,
    pending_draw_plan: Option<NativeDrawPlan>,
    quit_requested: bool,
    close_approved: bool,
    resource_policy: NativeWindowResourcePolicy,
    view_suspended: bool,
}

pub(crate) fn windows_win32_view_input_route(
    runtime: &crate::native::NativeViewInputRuntime,
) -> Option<WindowsWin32ViewInputRoute> {
    runtime
        .backend_runtime()
        .map(WindowsWin32ViewInputRoute::from_shared_runtime)
}

impl WindowsWin32ViewInputRoute {
    pub fn new(
        interaction_plan: ViewInteractionPlan,
        ui_command_view: ViewNode<UiCommand>,
    ) -> Self {
        let surface = ui_command_view.bounds().unwrap_or_else(|| {
            let width = interaction_plan
                .hit_targets
                .iter()
                .map(|target| target.bounds.x.saturating_add(target.bounds.width))
                .max()
                .unwrap_or(1)
                .max(1);
            let height = interaction_plan
                .hit_targets
                .iter()
                .map(|target| target.bounds.y.saturating_add(target.bounds.height))
                .max()
                .unwrap_or(1)
                .max(1);
            crate::Rect {
                x: 0,
                y: 0,
                width,
                height,
            }
        });
        Self::from_shared_runtime(crate::native::NativeViewInputRuntime::new(
            surface,
            Some(interaction_plan),
            Some(ui_command_view),
            None,
            NativeWindowResourcePolicy::default(),
            None,
            None,
            None,
        ))
    }

    pub fn from_live_view(live_view: SharedLiveViewRuntime) -> Self {
        let (surface, dpi) = live_view.surface();
        let mut runtime = crate::native::NativeViewInputRuntime::new(
            surface,
            Some(live_view.interaction_plan()),
            None,
            Some(live_view),
            NativeWindowResourcePolicy::default(),
            None,
            None,
            None,
        );
        let _ = runtime.set_surface(surface, dpi);
        Self::from_shared_runtime(runtime)
    }

    fn from_shared_runtime(mut shared_runtime: crate::native::NativeViewInputRuntime) -> Self {
        shared_runtime.set_text_shaping_backend(
            crate::windows_gdi_renderer::windows_gdi_text_shaping_backend(),
        );
        #[cfg(feature = "tooltip")]
        shared_runtime.set_tooltip_timing(windows_tooltip_timing());
        #[cfg(feature = "menu-flyout")]
        shared_runtime.set_menu_flyout_open_delay(windows_menu_flyout_open_delay());
        shared_runtime.defer_app_command_execution();
        shared_runtime.defer_ui_command_execution();
        Self {
            resource_policy: shared_runtime.resource_policy(),
            view_suspended: shared_runtime.is_view_suspended(),
            shared_runtime,
            shared_text_drag_active: false,
            #[cfg(feature = "canvas")]
            shared_canvas_pointer_drag_active: false,
            #[cfg(feature = "canvas")]
            shared_canvas_pointer_drag_moved: false,
            #[cfg(feature = "slider")]
            shared_slider_drag_active: false,
            #[cfg(feature = "color-picker")]
            shared_color_picker_drag_active: false,
            pending_utf16_high_surrogate: None,
            pending_draw_plan: None,
            quit_requested: false,
            close_approved: false,
        }
    }

    pub fn app_command_executor(mut self, executor: SharedAppCommandExecutor) -> Self {
        self.shared_runtime.set_app_command_executor(executor);
        self
    }

    pub fn resource_policy(mut self, policy: NativeWindowResourcePolicy) -> Self {
        self.shared_runtime.set_resource_policy(policy);
        self.resource_policy = policy;
        self
    }

    pub fn window_close_request_command(mut self, command: Option<Command>) -> Self {
        self.shared_runtime.set_window_close_request_command(command);
        self
    }

    pub fn ui_command_executor(mut self, executor: SharedUiCommandExecutor) -> Self {
        self.shared_runtime.set_ui_command_executor(executor);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsWin32ViewInputDispatchReport {
    pub handled: bool,
    pub window_close_request_count: usize,
    pub window_close_veto_count: usize,
    pub hit_target_count: usize,
    pub click_count: usize,
    pub pointer_down_count: usize,
    pub pointer_move_count: usize,
    pub pointer_up_count: usize,
    pub canvas_pointer_event_count: usize,
    pub canvas_pointer_drag_count: usize,
    pub canvas_pointer_drag_active: bool,
    pub pointer_visual_change_count: usize,
    pub event_count: usize,
    pub message_count: usize,
    pub ui_command_count: usize,
    pub ui_command_executed_count: usize,
    pub ui_command_failed_count: usize,
    pub ui_command_unhandled_count: usize,
    pub ui_command_event_count: usize,
    pub ui_command_errors: Vec<String>,
    pub app_command_count: usize,
    pub app_command_executed_count: usize,
    pub app_command_failed_count: usize,
    pub app_command_unhandled_count: usize,
    pub app_command_event_count: usize,
    pub app_command_names: Vec<&'static str>,
    pub app_command_errors: Vec<String>,
    pub ui_command_ids: Vec<&'static str>,
    pub live_view_revision: u64,
    pub background_refresh_count: usize,
    pub quit_requested: bool,
    pub unhandled_click_count: usize,
    pub focus_count: usize,
    pub focus_visual_count: usize,
    pub focused_widget: Option<u64>,
    pub focus_traversal_count: usize,
    pub text_input_count: usize,
    pub text_navigation_count: usize,
    pub text_navigation_evidence: Vec<crate::NativeTextNavigationEvidence>,
    pub text_selection_change_count: usize,
    pub text_selection: Option<(usize, usize)>,
    pub text_caret: Option<usize>,
    #[cfg(feature = "textbox")]
    pub text_edit_command_count: usize,
    #[cfg(feature = "textbox")]
    pub text_clipboard_read_count: usize,
    #[cfg(feature = "textbox")]
    pub text_clipboard_write_count: usize,
    #[cfg(feature = "textbox")]
    pub text_undo_count: usize,
    #[cfg(feature = "textbox")]
    pub text_edit_command_errors: Vec<String>,
    pub text_drag_count: usize,
    pub text_drag_scroll_count: usize,
    pub text_drag_active: bool,
    pub slider_value_change_count: usize,
    pub slider_keyboard_change_count: usize,
    pub slider_drag_count: usize,
    pub slider_drag_active: bool,
    pub color_picker_value_change_count: usize,
    pub color_picker_channel_change_count: usize,
    pub color_picker_expanded_change_count: usize,
    pub color_picker_drag_count: usize,
    pub color_picker_drag_active: bool,
    pub radio_selection_count: usize,
    pub radio_keyboard_selection_count: usize,
    pub radio_keyboard_focus_only_count: usize,
    pub auto_suggest_expanded_change_count: usize,
    pub auto_suggest_highlight_change_count: usize,
    pub auto_suggest_submit_count: usize,
    pub auto_suggest_clear_count: usize,
    pub tree_expansion_change_count: usize,
    pub tree_selection_count: usize,
    pub tree_invoke_count: usize,
    pub grid_view_selection_count: usize,
    pub grid_view_invoke_count: usize,
    pub table_sort_count: usize,
    pub table_selection_count: usize,
    pub table_invoke_count: usize,
    pub content_dialog_focus_change_count: usize,
    pub content_dialog_response_count: usize,
    pub command_palette_query_change_count: usize,
    pub command_palette_highlight_change_count: usize,
    pub command_palette_invoke_count: usize,
    pub command_palette_open_change_count: usize,
    pub command_palette_clear_count: usize,
    pub menu_flyout_highlight_change_count: usize,
    pub menu_flyout_submenu_change_count: usize,
    pub menu_flyout_invoke_count: usize,
    pub menu_flyout_open_change_count: usize,
    pub toast_focus_change_count: usize,
    pub toast_response_count: usize,
    pub toast_timeout_count: usize,
    pub info_bar_focus_change_count: usize,
    pub info_bar_event_count: usize,
    pub teaching_tip_focus_change_count: usize,
    pub teaching_tip_response_count: usize,
    pub breadcrumb_focus_change_count: usize,
    pub breadcrumb_expanded_change_count: usize,
    pub breadcrumb_selection_count: usize,
    pub combo_expanded_change_count: usize,
    pub combo_selection_count: usize,
    pub combo_keyboard_selection_count: usize,
    pub combo_type_ahead_match_count: usize,
    pub combo_scroll_count: usize,
    pub tab_selection_count: usize,
    pub tab_keyboard_selection_count: usize,
    pub tab_keyboard_focus_only_count: usize,
    pub toggle_count: usize,
    pub selection_count: usize,
    pub keyboard_selection_count: usize,
    pub key_down_count: usize,
    pub keyboard_activation_count: usize,
    pub unhandled_key_count: usize,
    pub scroll_count: usize,
    pub unhandled_scroll_count: usize,
    pub events: Vec<String>,
}

impl WindowsWin32ViewInputDispatchReport {
    fn merge(&mut self, next: WindowsWin32ViewInputDispatchReport) {
        self.handled |= next.handled;
        self.window_close_request_count += next.window_close_request_count;
        self.window_close_veto_count += next.window_close_veto_count;
        self.hit_target_count = next.hit_target_count;
        self.click_count += next.click_count;
        self.pointer_down_count += next.pointer_down_count;
        self.pointer_move_count += next.pointer_move_count;
        self.pointer_up_count += next.pointer_up_count;
        self.canvas_pointer_event_count += next.canvas_pointer_event_count;
        self.canvas_pointer_drag_count += next.canvas_pointer_drag_count;
        self.canvas_pointer_drag_active = next.canvas_pointer_drag_active;
        self.pointer_visual_change_count += next.pointer_visual_change_count;
        self.event_count += next.event_count;
        self.message_count += next.message_count;
        self.ui_command_count += next.ui_command_count;
        self.ui_command_executed_count += next.ui_command_executed_count;
        self.ui_command_failed_count += next.ui_command_failed_count;
        self.ui_command_unhandled_count += next.ui_command_unhandled_count;
        self.ui_command_event_count += next.ui_command_event_count;
        self.ui_command_errors.extend(next.ui_command_errors);
        self.app_command_count += next.app_command_count;
        self.app_command_executed_count += next.app_command_executed_count;
        self.app_command_failed_count += next.app_command_failed_count;
        self.app_command_unhandled_count += next.app_command_unhandled_count;
        self.app_command_event_count += next.app_command_event_count;
        self.app_command_names.extend(next.app_command_names);
        self.app_command_errors.extend(next.app_command_errors);
        self.ui_command_ids.extend(next.ui_command_ids);
        self.live_view_revision = self.live_view_revision.max(next.live_view_revision);
        self.background_refresh_count += next.background_refresh_count;
        self.quit_requested |= next.quit_requested;
        self.unhandled_click_count += next.unhandled_click_count;
        self.focus_count += next.focus_count;
        self.focus_visual_count += next.focus_visual_count;
        self.focused_widget = next.focused_widget.or(self.focused_widget);
        self.focus_traversal_count += next.focus_traversal_count;
        self.text_input_count += next.text_input_count;
        self.text_navigation_count += next.text_navigation_count;
        self.text_navigation_evidence
            .extend(next.text_navigation_evidence);
        self.text_selection_change_count += next.text_selection_change_count;
        self.text_selection = next.text_selection.or(self.text_selection);
        self.text_caret = next.text_caret.or(self.text_caret);
        #[cfg(feature = "textbox")]
        {
            self.text_edit_command_count += next.text_edit_command_count;
            self.text_clipboard_read_count += next.text_clipboard_read_count;
            self.text_clipboard_write_count += next.text_clipboard_write_count;
            self.text_undo_count += next.text_undo_count;
            self.text_edit_command_errors
                .extend(next.text_edit_command_errors);
        }
        self.text_drag_count += next.text_drag_count;
        self.text_drag_scroll_count += next.text_drag_scroll_count;
        self.text_drag_active = next.text_drag_active;
        self.slider_value_change_count += next.slider_value_change_count;
        self.slider_keyboard_change_count += next.slider_keyboard_change_count;
        self.slider_drag_count += next.slider_drag_count;
        self.slider_drag_active = next.slider_drag_active;
        self.color_picker_value_change_count += next.color_picker_value_change_count;
        self.color_picker_channel_change_count += next.color_picker_channel_change_count;
        self.color_picker_expanded_change_count += next.color_picker_expanded_change_count;
        self.color_picker_drag_count += next.color_picker_drag_count;
        self.color_picker_drag_active = next.color_picker_drag_active;
        self.radio_selection_count += next.radio_selection_count;
        self.radio_keyboard_selection_count += next.radio_keyboard_selection_count;
        self.radio_keyboard_focus_only_count += next.radio_keyboard_focus_only_count;
        self.auto_suggest_expanded_change_count += next.auto_suggest_expanded_change_count;
        self.auto_suggest_highlight_change_count += next.auto_suggest_highlight_change_count;
        self.auto_suggest_submit_count += next.auto_suggest_submit_count;
        self.auto_suggest_clear_count += next.auto_suggest_clear_count;
        self.tree_expansion_change_count += next.tree_expansion_change_count;
        self.tree_selection_count += next.tree_selection_count;
        self.tree_invoke_count += next.tree_invoke_count;
        self.grid_view_selection_count += next.grid_view_selection_count;
        self.grid_view_invoke_count += next.grid_view_invoke_count;
        self.table_sort_count += next.table_sort_count;
        self.table_selection_count += next.table_selection_count;
        self.table_invoke_count += next.table_invoke_count;
        self.content_dialog_focus_change_count += next.content_dialog_focus_change_count;
        self.content_dialog_response_count += next.content_dialog_response_count;
        self.command_palette_query_change_count += next.command_palette_query_change_count;
        self.command_palette_highlight_change_count += next.command_palette_highlight_change_count;
        self.command_palette_invoke_count += next.command_palette_invoke_count;
        self.command_palette_open_change_count += next.command_palette_open_change_count;
        self.command_palette_clear_count += next.command_palette_clear_count;
        self.menu_flyout_highlight_change_count += next.menu_flyout_highlight_change_count;
        self.menu_flyout_submenu_change_count += next.menu_flyout_submenu_change_count;
        self.menu_flyout_invoke_count += next.menu_flyout_invoke_count;
        self.menu_flyout_open_change_count += next.menu_flyout_open_change_count;
        self.toast_focus_change_count += next.toast_focus_change_count;
        self.toast_response_count += next.toast_response_count;
        self.toast_timeout_count += next.toast_timeout_count;
        self.info_bar_focus_change_count += next.info_bar_focus_change_count;
        self.info_bar_event_count += next.info_bar_event_count;
        self.teaching_tip_focus_change_count += next.teaching_tip_focus_change_count;
        self.teaching_tip_response_count += next.teaching_tip_response_count;
        self.breadcrumb_focus_change_count += next.breadcrumb_focus_change_count;
        self.breadcrumb_expanded_change_count += next.breadcrumb_expanded_change_count;
        self.breadcrumb_selection_count += next.breadcrumb_selection_count;
        self.combo_expanded_change_count += next.combo_expanded_change_count;
        self.combo_selection_count += next.combo_selection_count;
        self.combo_keyboard_selection_count += next.combo_keyboard_selection_count;
        self.combo_type_ahead_match_count += next.combo_type_ahead_match_count;
        self.combo_scroll_count += next.combo_scroll_count;
        self.tab_selection_count += next.tab_selection_count;
        self.tab_keyboard_selection_count += next.tab_keyboard_selection_count;
        self.tab_keyboard_focus_only_count += next.tab_keyboard_focus_only_count;
        self.toggle_count += next.toggle_count;
        self.selection_count += next.selection_count;
        self.keyboard_selection_count += next.keyboard_selection_count;
        self.key_down_count += next.key_down_count;
        self.keyboard_activation_count += next.keyboard_activation_count;
        self.unhandled_key_count += next.unhandled_key_count;
        self.scroll_count += next.scroll_count;
        self.unhandled_scroll_count += next.unhandled_scroll_count;
        self.events.extend(next.events);
    }
}
pub fn set_windows_win32_window_view_input_route(
    hwnd: HWND,
    mut route: WindowsWin32ViewInputRoute,
) -> bool {
    if hwnd.is_null() {
        return false;
    }
    if let Some((bounds, dpi)) = windows_win32_shell_surface(hwnd) {
        route.set_surface(bounds, dpi);
    }
    let draw_plan = route.take_pending_draw_plan();
    let poll_interval_ms = route.background_poll_interval_ms();
    let hwnd_value = hwnd as isize;
    completed_window_view_input_reports()
        .lock()
        .expect("completed window view input report registry should not be poisoned")
        .retain(|record| record.hwnd != hwnd_value);
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    if let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) {
        record.route = route;
        record.report = WindowsWin32ViewInputDispatchReport::default();
    } else {
        let report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: route.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        routes.push(WindowsWindowViewInputRouteRecord {
            hwnd: hwnd_value,
            route,
            report,
        });
    }
    drop(routes);
    if let Some(draw_plan) = draw_plan {
        set_windows_win32_window_draw_plan(hwnd, draw_plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    sync_windows_win32_live_view_poll_timer(hwnd, poll_interval_ms);
    true
}

pub fn clear_windows_win32_window_view_input_route(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    unsafe {
        KillTimer(hwnd, ZSUI_WIN32_LIVE_VIEW_POLL_TIMER_ID);
    }
    let hwnd = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    routes.retain(|record| record.hwnd != hwnd);
    completed_window_view_input_reports()
        .lock()
        .expect("completed window view input report registry should not be poisoned")
        .retain(|record| record.hwnd != hwnd);
}

fn archive_windows_win32_window_view_input_report(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    let Some(index) = routes.iter().position(|record| record.hwnd == hwnd) else {
        return;
    };
    let record = routes.remove(index);
    drop(routes);
    let mut completed = completed_window_view_input_reports()
        .lock()
        .expect("completed window view input report registry should not be poisoned");
    completed.retain(|record| record.hwnd != hwnd);
    completed.push(WindowsCompletedViewInputReportRecord {
        hwnd,
        report: record.report,
    });
}

pub fn clear_windows_win32_window_view_input_routes() {
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    routes.clear();
    completed_window_view_input_reports()
        .lock()
        .expect("completed window view input report registry should not be poisoned")
        .clear();
}


pub fn windows_win32_window_view_input_report(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    if let Some(report) = routes
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.report.clone())
    {
        return Some(report);
    }
    drop(routes);
    completed_window_view_input_reports()
        .lock()
        .expect("completed window view input report registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.report.clone())
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn windows_win32_window_text_accessibility_snapshot(
    hwnd: HWND,
) -> Option<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd)
        .and_then(|record| record.route.focused_text_accessibility_snapshot())
}

#[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
pub(crate) fn windows_win32_window_menu_flyout_accessibility_snapshot(
    hwnd: HWND,
) -> Option<crate::native_menu_accessibility::NativeMenuFlyoutAccessibilitySnapshot> {
    if hwnd.is_null() {
        return None;
    }
    let (interaction, menu) = {
        let routes = window_view_input_routes()
            .lock()
            .expect("window view input route registry should not be poisoned");
        let record = routes.iter().find(|record| record.hwnd == hwnd as isize)?;
        let interaction = record.route.shared_runtime.current_interaction_plan()?;
        let widget = interaction.hit_targets.iter().find_map(|target| {
            matches!(
                target.kind,
                crate::ViewHitTargetKind::MenuFlyoutItem { .. }
            )
            .then_some(target.widget)
        })?;
        let (_, menu) = record
            .route
            .shared_runtime
            .widget_menu_flyout_state(widget)?;
        (interaction, menu)
    };
    let plan = window_draw_plan(hwnd)?;
    crate::native_menu_accessibility::native_menu_flyout_accessibility_snapshot(
        &plan,
        &interaction,
        Some(&menu),
    )
}

#[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
pub(crate) fn focus_windows_win32_window_accessible_menu_flyout_item(
    hwnd: HWND,
    path: crate::ZsMenuFlyoutPath,
) -> bool {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_menu_flyout_focus(path)
    })
    .is_some_and(|report| report.handled)
}

#[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
pub(crate) fn invoke_windows_win32_window_accessible_menu_flyout_item(
    hwnd: HWND,
    path: crate::ZsMenuFlyoutPath,
) -> bool {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_menu_flyout_invoke(path)
    })
    .is_some_and(|report| report.handled)
}

#[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
pub(crate) fn set_windows_win32_window_accessible_menu_flyout_item_expanded(
    hwnd: HWND,
    path: crate::ZsMenuFlyoutPath,
    expanded: bool,
) -> bool {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_menu_flyout_expanded(path, expanded)
    })
    .is_some_and(|report| report.handled)
}

pub fn refresh_windows_win32_window_live_view_surface(hwnd: HWND) -> bool {
    let Some((bounds, dpi)) = windows_win32_shell_surface(hwnd) else {
        return false;
    };
    let hwnd_value = hwnd as isize;
    let draw_plan = {
        let mut routes = window_view_input_routes()
            .lock()
            .expect("window view input route registry should not be poisoned");
        let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) else {
            return false;
        };
        if !record.route.set_surface(bounds, dpi) {
            return true;
        }
        record.route.take_pending_draw_plan()
    };
    if let Some(draw_plan) = draw_plan {
        set_windows_win32_window_draw_plan(hwnd, draw_plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    true
}

pub fn sync_windows_win32_window_view_visibility(hwnd: HWND, visible: bool) -> bool {
    if hwnd.is_null() {
        return false;
    }
    let hwnd_value = hwnd as isize;
    let (eligible, released, draw_plan, poll_interval_ms) = {
        let mut routes = window_view_input_routes()
            .lock()
            .expect("window view input route registry should not be poisoned");
        let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) else {
            return false;
        };
        let eligible = record.route.has_live_view()
            && record.route.resource_policy.releases_view_when_hidden();
        if !eligible {
            return false;
        }
        if visible {
            record.route.resume_view_when_visible();
        } else {
            record.route.suspend_view_when_hidden();
        }
        (
            true,
            record.route.view_suspended,
            record.route.take_pending_draw_plan(),
            record.route.background_poll_interval_ms(),
        )
    };
    if released {
        clear_windows_win32_window_draw_plan(hwnd);
    } else if let Some(draw_plan) = draw_plan {
        set_windows_win32_window_draw_plan(hwnd, draw_plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    sync_windows_win32_live_view_poll_timer(hwnd, poll_interval_ms);
    eligible
}

pub fn dispatch_windows_win32_window_view_click(
    hwnd: HWND,
    point: crate::Point,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_click(point))
}

pub fn dispatch_windows_win32_window_view_pointer_down(
    hwnd: HWND,
    point: crate::Point,
    shift: bool,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_pointer_down(point, shift)
    })
}

pub fn dispatch_windows_win32_window_view_pointer_down_with_button(
    hwnd: HWND,
    point: crate::Point,
    button: crate::ZsPointerButton,
    modifiers: crate::ZsPointerModifiers,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_pointer_down_with_button(point, button, modifiers)
    })
}

pub fn dispatch_windows_win32_window_view_pointer_move(
    hwnd: HWND,
    point: crate::Point,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_pointer_move(point))
}

pub fn dispatch_windows_win32_window_view_pointer_move_with_modifiers(
    hwnd: HWND,
    point: crate::Point,
    modifiers: crate::ZsPointerModifiers,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_pointer_move_with_modifiers(point, modifiers)
    })
}

pub fn dispatch_windows_win32_window_view_pointer_leave(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_pointer_leave())
}

pub fn dispatch_windows_win32_window_view_pointer_up(
    hwnd: HWND,
    point: crate::Point,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_pointer_up(point))
}

pub fn dispatch_windows_win32_window_view_pointer_up_with_button(
    hwnd: HWND,
    point: crate::Point,
    button: crate::ZsPointerButton,
    modifiers: crate::ZsPointerModifiers,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_pointer_up_with_button(point, button, modifiers)
    })
}

pub fn cancel_windows_win32_window_view_pointer_drag(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.cancel_pointer_drag())
}

pub fn dispatch_windows_win32_window_view_text_input(
    hwnd: HWND,
    text: &str,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_text_input(text))
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn set_windows_win32_window_accessible_text_value(hwnd: HWND, value: &str) -> bool {
    if windows_win32_window_text_accessibility_snapshot(hwnd)
        .is_none_or(|snapshot| snapshot.kind().is_protected())
    {
        return false;
    }
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_set_text_value(value)
    })
    .is_some()
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn set_windows_win32_window_accessible_text_selection(
    hwnd: HWND,
    widget: crate::WidgetId,
    selection: crate::native_text_edit::NativeTextSelection,
) -> bool {
    if windows_win32_window_text_accessibility_snapshot(hwnd)
        .is_none_or(|snapshot| snapshot.widget() != widget || snapshot.kind().is_protected())
    {
        return false;
    }
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_set_text_selection(selection)
    })
    .is_some()
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn scroll_windows_win32_window_accessible_text_range(
    hwnd: HWND,
    widget: crate::WidgetId,
    selection: crate::native_text_edit::NativeTextSelection,
    align_to_top: bool,
) -> bool {
    if windows_win32_window_text_accessibility_snapshot(hwnd)
        .is_none_or(|snapshot| snapshot.widget() != widget || snapshot.kind().is_protected())
    {
        return false;
    }
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_accessibility_scroll_text_range(widget, selection, align_to_top)
    })
    .is_some()
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn windows_win32_window_text_accessibility_range_rectangles(
    hwnd: HWND,
    widget: crate::WidgetId,
    selection: crate::native_text_edit::NativeTextSelection,
) -> Option<Vec<crate::Rect>> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd)
        .and_then(|record| {
            record
                .route
                .text_accessibility_range_rectangles(widget, selection)
        })
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn windows_win32_window_text_accessibility_visible_range(
    hwnd: HWND,
) -> Option<(crate::WidgetId, std::ops::Range<usize>)> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd)
        .and_then(|record| record.route.text_accessibility_visible_range())
}

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
pub(crate) fn windows_win32_window_text_accessibility_index_for_screen_point(
    hwnd: HWND,
    x: f64,
    y: f64,
) -> Option<(crate::WidgetId, usize)> {
    if hwnd.is_null() || !x.is_finite() || !y.is_finite() {
        return None;
    }
    let mut point = POINT {
        x: x.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
        y: y.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
    };
    if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
        return None;
    }
    let hwnd_key = hwnd as isize;
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd_key)
        .and_then(|record| {
            record
                .route
                .text_accessibility_index_for_point(crate::Point {
                    x: point.x,
                    y: point.y,
                })
        })
}

fn dispatch_windows_win32_window_view_utf16_input_unit(
    hwnd: HWND,
    unit: u16,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_utf16_input_unit(unit))
}

pub fn dispatch_windows_win32_window_view_key_down(
    hwnd: HWND,
    virtual_key: u32,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_key_down(virtual_key))
}

pub fn dispatch_windows_win32_window_view_key_down_with_shift(
    hwnd: HWND,
    virtual_key: u32,
    shift: bool,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_key_down_with_shift(virtual_key, shift)
    })
}

fn dispatch_windows_win32_window_view_key_down_with_modifiers(
    hwnd: HWND,
    virtual_key: u32,
    shift: bool,
    control: bool,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| {
        route.dispatch_key_down_with_modifiers(virtual_key, shift, control)
    })
}

pub fn dispatch_windows_win32_window_view_blur(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, WindowsWin32ViewInputRoute::dispatch_blur)
}

pub fn dispatch_windows_win32_window_view_scroll(

    hwnd: HWND,
    point: crate::Point,
    delta_y: crate::Dp,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_scroll(point, delta_y))
}

pub fn refresh_windows_win32_window_background_view(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input(hwnd, |route| route.refresh_background_view())
}

fn windows_win32_window_focused_target(hwnd: HWND) -> Option<crate::ViewHitTarget> {
    if hwnd.is_null() {
        return None;
    }
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd as isize)
        .and_then(|record| record.route.focused_target())
}

fn dispatch_windows_win32_window_view_input(
    hwnd: HWND,
    dispatch: impl FnOnce(&mut WindowsWin32ViewInputRoute) -> WindowsWin32ViewInputDispatchReport,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    dispatch_windows_win32_window_view_input_with_quit_policy(hwnd, dispatch, true)
}

fn dispatch_windows_win32_window_view_input_with_quit_policy(
    hwnd: HWND,
    dispatch: impl FnOnce(&mut WindowsWin32ViewInputRoute) -> WindowsWin32ViewInputDispatchReport,
    post_close_on_quit: bool,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd_value = hwnd as isize;
    let (
        mut report,
        mut draw_plan,
        quit_requested,
        app_executor,
        app_commands,
        ui_executor,
        ui_commands,
        mut poll_interval_ms,
    ) = {
        let mut routes = window_view_input_routes()
            .lock()
            .expect("window view input route registry should not be poisoned");
        let record = routes.iter_mut().find(|record| record.hwnd == hwnd_value)?;
        let report = dispatch(&mut record.route);
        let draw_plan = record.route.take_pending_draw_plan();
        let quit_requested = record.route.take_quit_requested();
        let (app_executor, app_commands) = record.route.take_pending_app_command_dispatch();
        let (ui_executor, ui_commands) = record.route.take_pending_ui_command_dispatch();
        let poll_interval_ms = record.route.background_poll_interval_ms();
        (
            report,
            draw_plan,
            quit_requested,
            app_executor,
            app_commands,
            ui_executor,
            ui_commands,
            poll_interval_ms,
        )
    };

    let app_effect_executed =
        dispatch_windows_win32_app_commands(&mut report, app_executor, app_commands);
    dispatch_windows_win32_ui_commands(&mut report, ui_executor, ui_commands);
    if let Some(record) = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter_mut()
        .find(|record| record.hwnd == hwnd_value)
    {
        if app_effect_executed {
            if let Some(revision) = record.route.refresh_live_view_after_app_effect() {
                report.live_view_revision = revision;
                report.hit_target_count = record.route.hit_target_count();
                report
                    .events
                    .push(format!("win32_live_view_app_effect_refresh:{revision}"));
                if let Some(refreshed_plan) = record.route.take_pending_draw_plan() {
                    draw_plan = Some(refreshed_plan);
                }
                poll_interval_ms = record.route.background_poll_interval_ms();
            }
        }
        record.report.merge(report.clone());
    }

    if let Some(draw_plan) = draw_plan {
        set_windows_win32_window_draw_plan(hwnd, draw_plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    if quit_requested && post_close_on_quit {
        approve_windows_win32_window_close(hwnd);
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
    sync_windows_win32_live_view_poll_timer(hwnd, poll_interval_ms);
    Some(report)
}

pub fn approve_windows_win32_window_close(hwnd: HWND) -> bool {
    if hwnd.is_null() {
        return false;
    }
    let hwnd_value = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) else {
        return false;
    };
    record.route.approve_next_close();
    true
}

fn take_windows_win32_window_close_approval(hwnd: HWND) -> bool {
    if hwnd.is_null() {
        return false;
    }
    let hwnd_value = hwnd as isize;
    window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter_mut()
        .find(|record| record.hwnd == hwnd_value)
        .is_some_and(|record| record.route.take_close_approved())
}

fn dispatch_windows_win32_window_close_requested(
    hwnd: HWND,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    let mut report = dispatch_windows_win32_window_view_input_with_quit_policy(
        hwnd,
        WindowsWin32ViewInputRoute::dispatch_window_close_requested,
        false,
    )?;
    if report.handled && !report.quit_requested {
        report.window_close_veto_count = 1;
        report.events.push("win32_window_close_vetoed".to_string());
        let hwnd_value = hwnd as isize;
        if let Some(record) = window_view_input_routes()
            .lock()
            .expect("window view input route registry should not be poisoned")
            .iter_mut()
            .find(|record| record.hwnd == hwnd_value)
        {
            record.report.window_close_veto_count += 1;
            record
                .report
                .events
                .push("win32_window_close_vetoed".to_string());
        }
    }
    Some(report)
}

fn dispatch_windows_win32_app_commands(
    report: &mut WindowsWin32ViewInputDispatchReport,
    executor: Option<SharedAppCommandExecutor>,
    commands: Vec<Command>,
) -> bool {
    report
        .app_command_names
        .extend(commands.iter().map(crate::app_command_name));
    let Some(executor) = executor else {
        report.app_command_unhandled_count += commands.len();
        return false;
    };
    let mut executed = false;
    for command in commands {
        match executor.dispatch(command) {
            Ok(events) => {
                executed = true;
                report.app_command_executed_count += 1;
                report.app_command_event_count += events.len();
            }
            Err(err) => {
                report.app_command_failed_count += 1;
                report.app_command_errors.push(err.to_string());
            }
        }
    }
    executed
}

fn dispatch_windows_win32_ui_commands(
    report: &mut WindowsWin32ViewInputDispatchReport,
    executor: Option<SharedUiCommandExecutor>,
    commands: Vec<UiCommand>,
) {
    let Some(executor) = executor else {
        report.ui_command_unhandled_count += commands.len();
        return;
    };
    for command in commands {
        match executor.dispatch(command) {
            Ok(events) => {
                report.ui_command_executed_count += 1;
                report.ui_command_event_count += events.len();
            }
            Err(err) => {
                report.ui_command_failed_count += 1;
                report.ui_command_errors.push(err.to_string());
            }
        }
    }
}

fn window_view_input_routes() -> &'static Mutex<Vec<WindowsWindowViewInputRouteRecord>> {
    WINDOW_VIEW_INPUT_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(test)]
pub(crate) fn windows_win32_view_input_route_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static VIEW_INPUT_ROUTE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    VIEW_INPUT_ROUTE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn completed_window_view_input_reports(
) -> &'static Mutex<Vec<WindowsCompletedViewInputReportRecord>> {
    WINDOW_COMPLETED_VIEW_INPUT_REPORTS.get_or_init(|| Mutex::new(Vec::new()))
}
