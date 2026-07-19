
#[derive(Debug, Clone, Copy)]
enum WindowsSharedInputKind {
    PointerDown(Option<crate::ViewHitTarget>),
    PointerMove,
    PointerUp(Option<crate::ViewHitTarget>),
    PointerLeave,
    Text {
        accepted: usize,
        target: Option<crate::ViewHitTarget>,
    },
    Key {
        key: crate::native::NativeViewKey,
        target: Option<crate::ViewHitTarget>,
    },
    Scroll,
    Blur,
    Background,
    AppCommand,
    WindowClose,
    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    Accessibility,
    Surface,
}

impl WindowsSharedInputKind {
    const fn name(self) -> &'static str {
        match self {
            Self::PointerDown(_) => "pointer_down",
            Self::PointerMove => "pointer_move",
            Self::PointerUp(_) => "pointer_up",
            Self::PointerLeave => "pointer_leave",
            Self::Text { .. } => "text",
            Self::Key { .. } => "key_down",
            Self::Scroll => "scroll",
            Self::Blur => "blur",
            Self::Background => "background",
            Self::AppCommand => "app_command",
            Self::WindowClose => "window_close",
            #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
            Self::Accessibility => "accessibility",
            Self::Surface => "surface",
        }
    }

    const fn target(self) -> Option<crate::ViewHitTarget> {
        match self {
            Self::PointerDown(target) | Self::PointerUp(target) => target,
            Self::Text { target, .. } | Self::Key { target, .. } => target,
            _ => None,
        }
    }
}

impl WindowsWin32ViewInputRoute {
    pub fn hit_target_count(&self) -> usize {
        self.shared_runtime.hit_target_count()
    }

    fn has_live_view(&self) -> bool {
        self.shared_runtime.has_live_view()
    }

    fn shared_target_at(&self, point: crate::Point) -> Option<crate::ViewHitTarget> {
        self.shared_runtime
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_at(point))
    }

    fn shared_focused_target(&self) -> Option<crate::ViewHitTarget> {
        let widget = self.shared_runtime.focused_widget()?;
        self.shared_runtime
            .current_interaction_plan()
            .and_then(|plan| plan.focus_target_for_widget(widget))
    }


    fn suspend_view_when_hidden(&mut self) -> bool {
        let changed = self.shared_runtime.suspend_view_when_hidden();
        if changed {
            self.pending_utf16_high_surrogate = None;
            self.pending_draw_plan = None;
            self.sync_shared_host_state();
        }
        changed
    }

    fn resume_view_when_visible(&mut self) -> bool {
        let plan = self.shared_runtime.resume_view_when_visible();
        let changed = plan.is_some();
        if let Some(plan) = plan {
            self.pending_draw_plan = Some(plan);
        }
        self.sync_shared_host_state();
        changed
    }

    fn take_pending_draw_plan(&mut self) -> Option<NativeDrawPlan> {
        self.pending_draw_plan.take()
    }

    fn take_quit_requested(&mut self) -> bool {
        std::mem::take(&mut self.quit_requested)
    }

    fn approve_next_close(&mut self) {
        self.close_approved = true;
    }

    fn take_close_approved(&mut self) -> bool {
        std::mem::take(&mut self.close_approved)
    }

    fn take_pending_app_command_dispatch(
        &mut self,
    ) -> (Option<SharedAppCommandExecutor>, Vec<Command>) {
        self.shared_runtime.take_pending_app_command_dispatch()
    }

    fn take_pending_ui_command_dispatch(
        &mut self,
    ) -> (Option<SharedUiCommandExecutor>, Vec<UiCommand>) {
        self.shared_runtime.take_pending_ui_command_dispatch()
    }

    fn background_poll_interval_ms(&self) -> Option<u64> {
        self.shared_runtime.transient_poll_interval_ms()
    }

    fn refresh_background_view(&mut self) -> WindowsWin32ViewInputDispatchReport {
        self.refresh_background_view_at(std::time::Instant::now())
    }

    fn refresh_background_view_at(
        &mut self,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.refresh_transient_view_at(now);
        self.adapt_shared_report(report, WindowsSharedInputKind::Background)
    }

    fn set_surface(&mut self, bounds: crate::Rect, dpi: crate::Dpi) -> bool {
        let report = self.shared_runtime.set_surface(bounds, dpi);
        let changed = report.surface_changed;
        let _ = self.adapt_shared_report(report, WindowsSharedInputKind::Surface);
        changed
    }

    fn dispatch_app_command(&mut self, command: Command) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_app_command(command);
        self.adapt_shared_report(report, WindowsSharedInputKind::AppCommand)
    }

    fn dispatch_window_close_requested(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_window_close_requested();
        self.adapt_shared_report(report, WindowsSharedInputKind::WindowClose)
    }

    fn refresh_live_view_after_app_effect(&mut self) -> Option<u64> {
        let mut report = crate::native::NativeViewInputDispatchReport::default();
        self.shared_runtime
            .refresh_live_view_after_app_effect(&mut report);
        if report.redraw_plan.is_none() {
            return None;
        }
        let revision = self.shared_runtime.live_view_revision();
        let _ = self.adapt_shared_report(report, WindowsSharedInputKind::AppCommand);
        Some(revision)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]

    #[cfg(test)]
    fn widget_text_value(&self, widget: crate::WidgetId) -> Option<String> {
        self.shared_runtime.widget_text_value(widget)
    }

    #[cfg(test)]
    fn widget_checked_value(&self, widget: crate::WidgetId) -> Option<bool> {
        self.shared_runtime.widget_checked_value(widget)
    }

    #[cfg(all(test, feature = "slider"))]
    fn widget_slider_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(f32, crate::SliderRange)> {
        self.shared_runtime.widget_slider_state(widget)
    }

    #[cfg(all(test, feature = "toast"))]
    fn widget_toast_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)> {
        self.shared_runtime.widget_toast_state(widget)
    }

    #[cfg(all(test, feature = "combo"))]
    fn widget_combo_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(Option<usize>, usize, bool)> {
        self.shared_runtime.widget_combo_state(widget)
    }

    #[cfg(all(test, feature = "password-box"))]
    fn widget_password_value(&self, widget: crate::WidgetId) -> Option<crate::ZsPassword> {
        self.shared_runtime.widget_password_value(widget)
    }

    #[cfg(all(test, feature = "color-picker"))]
    fn widget_color_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsColorPickerState> {
        self.shared_runtime.widget_color_picker_state(widget)
    }

    #[cfg(all(test, feature = "auto-suggest"))]
    fn widget_auto_suggest_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsAutoSuggestState> {
        self.shared_runtime.widget_auto_suggest_state(widget)
    }

    #[cfg(all(test, feature = "command-palette"))]
    fn widget_command_palette_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsCommandPaletteState> {
        self.shared_runtime.widget_command_palette_state(widget)
    }

    #[cfg(all(test, feature = "tree"))]
    fn widget_tree_view_state(&self, widget: crate::WidgetId) -> Option<crate::ZsTreeViewState> {
        self.shared_runtime.widget_tree_view_state(widget)
    }

    #[cfg(all(test, feature = "grid-view"))]
    fn widget_grid_view_state(&self, widget: crate::WidgetId) -> Option<crate::ZsGridViewState> {
        self.shared_runtime.widget_grid_view_state(widget)
    }

    #[cfg(all(test, feature = "table"))]
    fn widget_table_state(&self, widget: crate::WidgetId) -> Option<crate::ZsTableViewState> {
        self.shared_runtime.widget_table_state(widget)
    }

    #[cfg(all(test, feature = "dialog"))]
    fn widget_content_dialog_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)> {
        self.shared_runtime.widget_content_dialog_state(widget)
    }

    #[cfg(all(test, feature = "info-bar"))]
    fn widget_info_bar_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)> {
        self.shared_runtime.widget_info_bar_state(widget)
    }

    #[cfg(all(test, feature = "teaching-tip"))]
    fn widget_teaching_tip_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)> {
        self.shared_runtime.widget_teaching_tip_state(widget)
    }

    #[cfg(all(test, feature = "date-picker"))]
    fn widget_date_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsDatePickerState> {
        self.shared_runtime.widget_date_picker_state(widget)
    }

    #[cfg(all(test, feature = "time-picker"))]
    fn widget_time_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsTimePickerState> {
        self.shared_runtime.widget_time_picker_state(widget)
    }

    #[cfg(test)]
    fn interaction_plan(&self) -> Option<ViewInteractionPlan> {
        self.shared_runtime.current_interaction_plan()
    }

    #[cfg(test)]
    fn focused_widget(&self) -> Option<crate::WidgetId> {
        self.shared_runtime.focused_widget()
    }

    #[cfg(test)]
    fn text_edit_selection(&self) -> Option<crate::native_text_edit::NativeTextSelection> {
        self.shared_runtime.text_edit_selection()
    }

    #[cfg(test)]
    fn pending_ui_command_count(&self) -> usize {
        self.shared_runtime.pending_ui_command_count()
    }

    #[cfg(all(test, feature = "tabs"))]
    fn widget_tab_view_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::view::ZsTabViewState> {
        self.shared_runtime.widget_tab_view_state(widget)
    }


    fn sync_shared_host_state(&mut self) {
        self.view_suspended = self.shared_runtime.is_view_suspended();
    }

    fn adapt_shared_report(
        &mut self,
        mut shared: crate::native::NativeViewInputDispatchReport,
        kind: WindowsSharedInputKind,
    ) -> WindowsWin32ViewInputDispatchReport {
        let redraw = shared.redraw_plan.take();
        if let Some(plan) = redraw {
            self.pending_draw_plan = Some(plan);
        }
        self.quit_requested |= shared.quit_requested;
        self.sync_shared_host_state();

        let target = kind.target();
        let text_drag_ended = self.shared_text_drag_active && !shared.text_drag_active;
        self.shared_text_drag_active = shared.text_drag_active;
        #[cfg(feature = "slider")]
        let slider_drag_ended = self.shared_slider_drag_active && !shared.slider_drag_active;
        #[cfg(feature = "slider")]
        {
            self.shared_slider_drag_active = shared.slider_drag_active;
        }
        #[cfg(feature = "color-picker")]
        let color_picker_drag_ended =
            self.shared_color_picker_drag_active && !shared.color_picker_drag_active;
        #[cfg(feature = "color-picker")]
        {
            self.shared_color_picker_drag_active = shared.color_picker_drag_active;
        }

        let mut report = WindowsWin32ViewInputDispatchReport {
            handled: shared.handled,
            window_close_request_count: shared.window_close_request_count,
            window_close_veto_count: shared.window_close_veto_count,
            hit_target_count: shared.hit_target_count,
            event_count: shared.view_event_count,
            message_count: shared.message_count,
            ui_command_count: shared.ui_command_count,
            app_command_count: shared.app_command_count,
            ui_command_ids: shared.ui_command_ids,
            live_view_revision: self.shared_runtime.live_view_revision(),
            quit_requested: shared.quit_requested,
            focus_count: usize::from(
                shared.focus_visual_changed && !matches!(kind, WindowsSharedInputKind::Blur),
            ),
            focus_visual_count: usize::from(shared.focus_visual_changed),
            focused_widget: shared.focused_widget,
            text_selection_change_count: usize::from(shared.text_selection_changed),
            text_caret: shared.text_caret,
            text_drag_scroll_count: shared.text_drag_scroll_count,
            text_drag_active: shared.text_drag_active,
            text_drag_count: usize::from(text_drag_ended),
            events: vec![format!(
                "win32_shared_input:{}:{}",
                kind.name(),
                if shared.handled { "handled" } else { "unhandled" }
            )],
            ..WindowsWin32ViewInputDispatchReport::default()
        };

        if self.pending_draw_plan.is_some() && report.live_view_revision > 0 {
            report.events.push(format!(
                "win32_live_view_repaint:{}",
                report.live_view_revision
            ));
        }
        report
            .events
            .extend(shared.errors.iter().map(|error| format!("win32_shared_input_error:{error}")));
        #[cfg(feature = "textbox")]
        {
            report.text_edit_command_count = shared.text_edit_command_count;
            report.text_clipboard_read_count = shared.text_clipboard_read_count;
            report.text_clipboard_write_count = shared.text_clipboard_write_count;
            report.text_undo_count = shared.text_undo_count;
            report.text_edit_command_errors = shared.errors;
        }
        #[cfg(not(feature = "textbox"))]
        {
            report.app_command_errors = shared.errors;
        }

        match kind {
            WindowsSharedInputKind::PointerDown(_) => report.pointer_down_count = 1,
            WindowsSharedInputKind::PointerMove => report.pointer_move_count = 1,
            WindowsSharedInputKind::PointerUp(_) => report.pointer_up_count = 1,
            WindowsSharedInputKind::Text { accepted, .. } => {
                report.text_input_count = usize::from(shared.handled) * accepted;
            }
            WindowsSharedInputKind::Key { key, target } => {
                report.key_down_count = 1;
                report.unhandled_key_count = usize::from(!shared.handled);
                report.focus_traversal_count =
                    usize::from(shared.handled && key == crate::native::NativeViewKey::Tab);
                report.keyboard_activation_count = usize::from(
                    shared.handled
                        && matches!(
                            key,
                            crate::native::NativeViewKey::Enter
                                | crate::native::NativeViewKey::Space
                        ),
                );
                report.text_navigation_count = usize::from(
                    shared.handled
                        && shared.text_caret.is_some()
                        && matches!(
                            key,
                            crate::native::NativeViewKey::Up
                                | crate::native::NativeViewKey::Down
                                | crate::native::NativeViewKey::Left
                                | crate::native::NativeViewKey::Right
                                | crate::native::NativeViewKey::Home
                                | crate::native::NativeViewKey::End
                                | crate::native::NativeViewKey::PageUp
                                | crate::native::NativeViewKey::PageDown
                        ),
                );
                if target.is_some_and(|target| target.kind == crate::ViewHitTargetKind::Unknown)
                    && shared.message_count > 0
                {
                    report.selection_count = 1;
                    report.keyboard_selection_count = 1;
                }
            }
            WindowsSharedInputKind::Scroll => {
                report.scroll_count = usize::from(shared.handled);
                report.unhandled_scroll_count = usize::from(!shared.handled);
            }
            _ => {}
        }

        if matches!(kind, WindowsSharedInputKind::PointerUp(_))
            && target.is_some_and(|target| target.kind == crate::ViewHitTargetKind::Unknown)
            && shared.message_count > 0
        {
            report.selection_count = 1;
        }
        let toggle_target = target.is_some_and(|target| match target.kind {
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle => true,
            #[cfg(feature = "toggle-button")]
            crate::ViewHitTargetKind::ToggleButton => true,
            _ => false,
        });
        if toggle_target && shared.message_count > 0 {
            report.toggle_count = 1;
        }

        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
            feature = "breadcrumb",
            feature = "color-picker",
            feature = "command-palette",
            feature = "date-picker",
            feature = "dialog",
            feature = "grid-view",
            feature = "info-bar",
            feature = "teaching-tip",
            feature = "password-box",
            feature = "tabs",
            feature = "time-picker",
            feature = "toast",
            feature = "toggle-button",
            feature = "table",
            feature = "tree"
        ))]
        {
            report.pointer_visual_change_count = usize::from(shared.pointer_visual_changed);
        }
        #[cfg(feature = "slider")]
        {
            report.slider_value_change_count = usize::from(shared.slider_value_changed);
            report.slider_keyboard_change_count = usize::from(
                shared.slider_value_changed && matches!(kind, WindowsSharedInputKind::Key { .. }),
            );
            report.slider_drag_count = usize::from(slider_drag_ended);
            report.slider_drag_active = shared.slider_drag_active;
        }
        #[cfg(feature = "color-picker")]
        {
            report.color_picker_value_change_count =
                usize::from(shared.color_picker_value_changed);
            report.color_picker_channel_change_count =
                usize::from(shared.color_picker_channel_changed);
            report.color_picker_expanded_change_count =
                usize::from(shared.color_picker_expanded_changed);
            report.color_picker_drag_count = usize::from(color_picker_drag_ended);
            report.color_picker_drag_active = shared.color_picker_drag_active;
        }
        #[cfg(feature = "radio")]
        {
            report.radio_selection_count = usize::from(shared.radio_selection_changed);
            report.radio_keyboard_selection_count =
                usize::from(shared.radio_keyboard_selection_changed);
            report.radio_keyboard_focus_only_count =
                usize::from(shared.radio_keyboard_focus_only);
        }
        #[cfg(feature = "auto-suggest")]
        {
            report.auto_suggest_expanded_change_count =
                usize::from(shared.auto_suggest_expanded_changed);
            report.auto_suggest_highlight_change_count =
                usize::from(shared.auto_suggest_highlight_changed);
            report.auto_suggest_submit_count = usize::from(shared.auto_suggest_submitted);
            report.auto_suggest_clear_count = usize::from(shared.auto_suggest_cleared);
        }
        #[cfg(feature = "tree")]
        {
            report.tree_expansion_change_count = usize::from(shared.tree_expansion_changed);
            report.tree_selection_count = usize::from(shared.tree_selection_changed);
            report.tree_invoke_count = usize::from(shared.tree_invoked);
        }
        #[cfg(feature = "grid-view")]
        {
            report.grid_view_selection_count = usize::from(shared.grid_view_selection_changed);
            report.grid_view_invoke_count = usize::from(shared.grid_view_invoked);
        }
        #[cfg(feature = "table")]
        {
            report.table_sort_count = usize::from(shared.table_sort_changed);
            report.table_selection_count = usize::from(shared.table_selection_changed);
            report.table_invoke_count = usize::from(shared.table_invoked);
        }
        #[cfg(feature = "dialog")]
        {
            report.content_dialog_focus_change_count =
                usize::from(shared.content_dialog_focus_changed);
            report.content_dialog_response_count =
                usize::from(shared.content_dialog_responded);
        }
        #[cfg(feature = "command-palette")]
        {
            report.command_palette_query_change_count =
                usize::from(shared.command_palette_query_changed);
            report.command_palette_highlight_change_count =
                usize::from(shared.command_palette_highlight_changed);
            report.command_palette_invoke_count =
                usize::from(shared.command_palette_invoked);
            report.command_palette_open_change_count =
                usize::from(shared.command_palette_open_changed);
            report.command_palette_clear_count =
                usize::from(shared.command_palette_cleared);
        }
        #[cfg(feature = "toast")]
        {
            report.toast_focus_change_count = usize::from(shared.toast_focus_changed);
            report.toast_response_count = usize::from(shared.toast_responded);
            report.toast_timeout_count = usize::from(
                shared.toast_responded && matches!(kind, WindowsSharedInputKind::Background),
            );
        }
        #[cfg(feature = "info-bar")]
        {
            report.info_bar_focus_change_count = usize::from(shared.info_bar_focus_changed);
            report.info_bar_event_count = usize::from(shared.info_bar_event.is_some());
        }
        #[cfg(feature = "teaching-tip")]
        {
            report.teaching_tip_focus_change_count =
                usize::from(shared.teaching_tip_focus_changed);
            report.teaching_tip_response_count =
                usize::from(shared.teaching_tip_response.is_some());
        }
        #[cfg(feature = "breadcrumb")]
        {
            report.breadcrumb_focus_change_count =
                usize::from(shared.breadcrumb_focus_changed);
            report.breadcrumb_expanded_change_count =
                usize::from(shared.breadcrumb_expanded_changed);
            report.breadcrumb_selection_count =
                usize::from(shared.breadcrumb_selection.is_some());
        }
        #[cfg(feature = "combo")]
        {
            report.combo_expanded_change_count = usize::from(shared.combo_expanded_changed);
            report.combo_selection_count = usize::from(shared.combo_selection_changed);
            report.combo_keyboard_selection_count =
                usize::from(shared.combo_keyboard_selection_changed);
            report.combo_type_ahead_match_count =
                usize::from(shared.combo_type_ahead_matched);
            report.combo_scroll_count = usize::from(shared.combo_scrolled);
        }
        #[cfg(feature = "tabs")]
        {
            report.tab_selection_count = usize::from(shared.tab_selection_changed);
            report.tab_keyboard_selection_count =
                usize::from(shared.tab_keyboard_selection_changed);
            report.tab_keyboard_focus_only_count =
                usize::from(shared.tab_keyboard_focus_only);
        }
        report
    }
}
