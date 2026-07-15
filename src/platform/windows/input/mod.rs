#[derive(Debug, Clone)]
pub struct WindowsWin32ViewInputRoute {
    interaction_plan: ViewInteractionPlan,
    text_shaping: crate::native_input_visuals::NativeTextShapingBackend,
    ui_command_view: Option<ViewNode<UiCommand>>,
    live_view: Option<SharedLiveViewRuntime>,
    focused_widget: Option<crate::WidgetId>,
    #[cfg(feature = "tooltip")]
    tooltip: crate::tooltip::ZsTooltipRuntime,
    #[cfg(feature = "toast")]
    toast: crate::toast::ZsToastRuntime,
    text_edit: Option<NativeTextEditState>,
    pending_utf16_high_surrogate: Option<u16>,
    #[cfg(feature = "textbox")]
    text_history: NativeTextHistory,
    #[cfg(feature = "textbox")]
    processing_text_edit_commands: bool,
    text_drag: Option<NativeTextDragState>,
    #[cfg(feature = "combo")]
    combo_type_ahead: NativeComboTypeAheadState,
    #[cfg(feature = "slider")]
    slider_drag: Option<crate::WidgetId>,
    #[cfg(feature = "color-picker")]
    color_picker_drag: Option<(crate::WidgetId, crate::ViewHitTargetKind)>,
    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
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
    pointer_hover: Option<NativePointerVisualKey>,
    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
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
    pointer_pressed: Option<NativePointerVisualKey>,
    #[cfg(feature = "password-box")]
    password_peek: Option<crate::WidgetId>,
    surface: Option<crate::Rect>,
    dpi: crate::Dpi,
    pending_draw_plan: Option<NativeDrawPlan>,
    quit_requested: bool,
    window_close_request_command: Option<Command>,
    close_approved: bool,
    app_command_executor: Option<SharedAppCommandExecutor>,
    pending_app_commands: Vec<Command>,
    ui_command_executor: Option<SharedUiCommandExecutor>,
    pending_ui_commands: Vec<UiCommand>,
}

impl WindowsWin32ViewInputRoute {
    pub fn new(
        interaction_plan: ViewInteractionPlan,
        ui_command_view: ViewNode<UiCommand>,
    ) -> Self {
        let surface = ui_command_view.bounds();
        #[cfg(feature = "toast")]
        let now = std::time::Instant::now();
        #[allow(unused_mut)]
        let mut route = Self {
            interaction_plan,
            text_shaping: crate::native_input_visuals::NativeTextShapingBackend::windows_gdi(),
            ui_command_view: Some(ui_command_view),
            live_view: None,
            focused_widget: None,
            #[cfg(feature = "tooltip")]
            tooltip: crate::tooltip::ZsTooltipRuntime::new(windows_tooltip_timing()),
            #[cfg(feature = "toast")]
            toast: crate::toast::ZsToastRuntime::default(),
            text_edit: None,
            pending_utf16_high_surrogate: None,
            #[cfg(feature = "textbox")]
            text_history: NativeTextHistory::default(),
            #[cfg(feature = "textbox")]
            processing_text_edit_commands: false,
            text_drag: None,
            #[cfg(feature = "combo")]
            combo_type_ahead: NativeComboTypeAheadState::default(),
            #[cfg(feature = "slider")]
            slider_drag: None,
            #[cfg(feature = "color-picker")]
            color_picker_drag: None,
            #[cfg(any(
                feature = "auto-suggest",
                feature = "breadcrumb",
                feature = "color-picker",
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
            pointer_hover: None,
            #[cfg(any(
                feature = "auto-suggest",
                feature = "breadcrumb",
                feature = "color-picker",
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
            pointer_pressed: None,
            #[cfg(feature = "password-box")]
            password_peek: None,
            surface,
            dpi: crate::Dpi::standard(),
            pending_draw_plan: None,
            quit_requested: false,
            window_close_request_command: None,
            close_approved: false,
            app_command_executor: None,
            pending_app_commands: Vec::new(),
            ui_command_executor: None,
            pending_ui_commands: Vec::new(),
        };
        #[cfg(feature = "toast")]
        route.sync_toast_runtime(now);
        route
    }

    pub fn from_live_view(live_view: SharedLiveViewRuntime) -> Self {
        #[cfg(feature = "toast")]
        let now = std::time::Instant::now();
        #[allow(unused_mut)]
        let mut route = Self {
            interaction_plan: live_view.interaction_plan(),
            text_shaping: crate::native_input_visuals::NativeTextShapingBackend::windows_gdi(),
            ui_command_view: None,
            live_view: Some(live_view),
            focused_widget: None,
            #[cfg(feature = "tooltip")]
            tooltip: crate::tooltip::ZsTooltipRuntime::new(windows_tooltip_timing()),
            #[cfg(feature = "toast")]
            toast: crate::toast::ZsToastRuntime::default(),
            text_edit: None,
            pending_utf16_high_surrogate: None,
            #[cfg(feature = "textbox")]
            text_history: NativeTextHistory::default(),
            #[cfg(feature = "textbox")]
            processing_text_edit_commands: false,
            text_drag: None,
            #[cfg(feature = "combo")]
            combo_type_ahead: NativeComboTypeAheadState::default(),
            #[cfg(feature = "slider")]
            slider_drag: None,
            #[cfg(feature = "color-picker")]
            color_picker_drag: None,
            #[cfg(any(
                feature = "auto-suggest",
                feature = "breadcrumb",
                feature = "color-picker",
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
            pointer_hover: None,
            #[cfg(any(
                feature = "auto-suggest",
                feature = "breadcrumb",
                feature = "color-picker",
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
            pointer_pressed: None,
            #[cfg(feature = "password-box")]
            password_peek: None,
            surface: None,
            dpi: crate::Dpi::standard(),
            pending_draw_plan: None,
            quit_requested: false,
            window_close_request_command: None,
            close_approved: false,
            app_command_executor: None,
            pending_app_commands: Vec::new(),
            ui_command_executor: None,
            pending_ui_commands: Vec::new(),
        };
        #[cfg(feature = "toast")]
        route.sync_toast_runtime(now);
        route
    }

    pub fn app_command_executor(mut self, executor: SharedAppCommandExecutor) -> Self {
        self.app_command_executor = Some(executor);
        self
    }

    pub fn window_close_request_command(mut self, command: Option<Command>) -> Self {
        self.window_close_request_command = command;
        self
    }

    pub fn ui_command_executor(mut self, executor: SharedUiCommandExecutor) -> Self {
        self.ui_command_executor = Some(executor);
        self
    }

    pub fn hit_target_count(&self) -> usize {
        self.interaction_plan.hit_target_count()
    }

}

impl WindowsWin32ViewInputRoute {
    fn dispatch_event(
        &mut self,
        event: crate::ViewEvent,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_event(&event);
            #[cfg(feature = "textbox")]
            let text_edit_commands = update.text_edit_commands.clone();
            report.message_count += update.message_count;
            report.ui_command_count += update.ui_commands.len();
            report.app_command_count += update.commands.len();
            report.live_view_revision = update.revision;
            report.quit_requested |= update.quit_requested;
            for command in update.commands {
                report
                    .app_command_names
                    .push(crate::app_command_name(&command));
                report
                    .events
                    .push(format!("win32_live_view_command:{command:?}"));
                if command == Command::Quit {
                    report.quit_requested = true;
                    self.quit_requested = true;
                }
                self.pending_app_commands.push(command);
            }
            for command in update.ui_commands {
                report.ui_command_ids.push(command.id.0);
                report
                    .events
                    .push(format!("win32_live_view_ui_command:{}", command.id.0));
                self.pending_ui_commands.push(command);
            }
            if update.redraw {
                self.interaction_plan = live_view.interaction_plan();
                self.rebuild_pending_draw_plan();
                report.hit_target_count = self.hit_target_count();
                report
                    .events
                    .push(format!("win32_live_view_repaint:{}", update.revision));
            }
            self.quit_requested |= update.quit_requested;
            #[cfg(feature = "toast")]
            self.sync_toast_runtime(std::time::Instant::now());
            #[cfg(feature = "textbox")]
            self.dispatch_text_edit_commands(text_edit_commands, report);
            return;
        }

        let mut event_cx = ViewEventCx::new();
        let Some(view) = &mut self.ui_command_view else {
            return;
        };
        view.event(&mut event_cx, &event);
        let commands = event_cx.into_messages();
        report.message_count += commands.len();
        report.ui_command_count += commands.len();
        for command in commands {
            report.ui_command_ids.push(command.id.0);
            report
                .events
                .push(format!("win32_view_ui_command:{}", command.id.0));
            self.pending_ui_commands.push(command);
        }
        if let Some(surface) = self.surface {
            let mut layout = crate::ViewLayoutCx::new(surface, self.dpi);
            view.layout(&mut layout);
        }
        let next_interaction_plan = view.interaction_plan();
        if next_interaction_plan.hit_target_count() > 0 {
            self.interaction_plan = next_interaction_plan;
        }
        #[cfg(feature = "toast")]
        self.sync_toast_runtime(std::time::Instant::now());
        self.rebuild_pending_draw_plan();
    }

    #[cfg(feature = "textbox")]
    fn dispatch_text_edit_commands(
        &mut self,
        commands: Vec<crate::ZsTextEditCommandRequest>,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if commands.is_empty() {
            return;
        }
        if self.processing_text_edit_commands {
            report
                .text_edit_command_errors
                .push("nested text edit commands are not supported".to_string());
            return;
        }

        self.processing_text_edit_commands = true;
        for request in commands {
            report.text_edit_command_count += 1;
            self.dispatch_text_edit_command(request, report);
        }
        self.processing_text_edit_commands = false;
    }

    #[cfg(feature = "textbox")]
    fn dispatch_text_edit_command(
        &mut self,
        request: crate::ZsTextEditCommandRequest,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let target = match request.widget {
            Some(widget) => self.interaction_plan.hit_target_for_widget(widget),
            None => self.focused_target(),
        };
        let Some(target) = target else {
            return;
        };
        if !matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            return;
        }

        let widget = target.widget;
        let mut value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        state.clamp(&value);
        let mut clipboard = crate::NativeClipboardService::new();
        let result = apply_text_edit_command(
            request.command,
            widget,
            &mut value,
            &mut state.selection,
            &mut self.text_history,
            &mut clipboard,
        );
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                report.handled = true;
                report.text_edit_command_errors.push(error.to_string());
                return;
            }
        };

        if result.selection_changed || result.text_changed {
            state.preferred_visual_x = None;
            state.first_visible_visual_row = native_text_first_visible_row_for_caret_with_backend(
                target,
                &value,
                state.selection.caret,
                state.first_visible_visual_row,
                self.widget_text_wrap(widget),
                self.dpi,
                &self.text_shaping,
            );
            state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
                target,
                &value,
                state.selection.caret,
                state.horizontal_scroll_px,
                self.widget_text_wrap(widget),
                self.dpi,
                &self.text_shaping,
            );
        }
        self.text_edit = Some(state);
        report.handled |= result.handled;
        report.text_selection_change_count += usize::from(result.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report.text_clipboard_read_count += usize::from(result.clipboard_read);
        report.text_clipboard_write_count += usize::from(result.clipboard_write);
        report.text_undo_count += usize::from(result.undo_applied);
        report.events.push(format!(
            "win32_view_text_edit_command:{}:{:?}",
            widget.0, request.command
        ));

        let event = if result.text_changed {
            Some(crate::ViewEvent::TextEdited {
                widget,
                value,
                selection: state.selection.into(),
            })
        } else if result.selection_changed {
            Some(crate::ViewEvent::TextSelectionChanged {
                widget,
                selection: state.selection.into(),
            })
        } else {
            None
        };
        if let Some(event) = event {
            report.event_count += 1;
            self.dispatch_event(event, report);
        }
    }

    fn widget_text_value(&self, widget: crate::WidgetId) -> Option<String> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_text_value(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_text_value(widget).map(str::to_string))
            })
    }

    fn widget_text_wrap(&self, widget: crate::WidgetId) -> crate::TextWrap {
        #[cfg(feature = "textbox")]
        {
            if let Some(wrap) = self
                .live_view
                .as_ref()
                .and_then(|runtime| runtime.widget_text_wrap(widget))
                .or_else(|| {
                    self.ui_command_view
                        .as_ref()
                        .and_then(|view| view.widget_text_wrap(widget))
                })
            {
                return wrap;
            }
        }
        let _ = widget;
        crate::TextWrap::NoWrap
    }

    #[cfg(feature = "password-box")]
    fn widget_password_value(&self, widget: crate::WidgetId) -> Option<crate::ZsPassword> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_password_value(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_password_value(widget).cloned())
            })
    }

    fn widget_display_text_value(&self, widget: crate::WidgetId) -> Option<String> {
        #[cfg(feature = "password-box")]
        if let Some(password) = self.widget_password_value(widget) {
            return Some(crate::mask_password(password.as_str()));
        }
        self.widget_text_value(widget)
    }

    fn widget_checked_value(&self, widget: crate::WidgetId) -> Option<bool> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_checked_value(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_checked_value(widget))
            })
    }

    fn widget_accepts_tab_focus(&self, target: crate::ViewHitTarget) -> bool {
        #[cfg(not(any(feature = "radio", feature = "tabs")))]
        let _ = target;
        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            return self
                .live_view
                .as_ref()
                .and_then(|runtime| runtime.widget_radio_is_tab_stop(target.widget))
                .or_else(|| {
                    self.ui_command_view
                        .as_ref()
                        .and_then(|view| view.widget_radio_is_tab_stop(target.widget))
                })
                .unwrap_or(true);
        }
        #[cfg(feature = "tabs")]
        if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
            return self
                .widget_tab_header_state(target.widget)
                .is_none_or(|state| state.selected);
        }
        true
    }

    #[cfg(feature = "radio")]
    fn widget_radio_relative_widget(
        &self,
        widget: crate::WidgetId,
        navigation: crate::ViewStackDirection,
        offset: isize,
    ) -> Option<crate::WidgetId> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_radio_relative_widget(widget, navigation, offset))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_radio_relative_widget(widget, navigation, offset))
            })
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_header_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::view::ZsTabHeaderState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_tab_header_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_tab_header_state(widget))
            })
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_cycle_target(
        &self,
        focused: crate::WidgetId,
        offset: isize,
    ) -> Option<(crate::WidgetId, crate::ZsTabId)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_tab_cycle_target(focused, offset))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_tab_cycle_target(focused, offset))
            })
    }

    #[cfg(feature = "slider")]
    fn widget_slider_state(&self, widget: crate::WidgetId) -> Option<(f32, crate::SliderRange)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_slider_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_slider_state(widget))
            })
    }

    #[cfg(feature = "color-picker")]
    fn widget_color_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsColorPickerState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_color_picker_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_color_picker_state(widget))
            })
    }

    #[cfg(feature = "auto-suggest")]
    fn widget_auto_suggest_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsAutoSuggestState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_auto_suggest_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_auto_suggest_state(widget))
            })
    }

    #[cfg(feature = "tree")]
    fn widget_tree_view_state(&self, widget: crate::WidgetId) -> Option<crate::ZsTreeViewState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_tree_view_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_tree_view_state(widget))
            })
    }

    #[cfg(feature = "grid-view")]
    fn widget_grid_view_state(&self, widget: crate::WidgetId) -> Option<crate::ZsGridViewState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_grid_view_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_grid_view_state(widget))
            })
    }

    #[cfg(feature = "table")]
    fn widget_table_state(&self, widget: crate::WidgetId) -> Option<crate::ZsTableViewState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_table_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_table_state(widget))
            })
    }

    #[cfg(feature = "dialog")]
    fn widget_content_dialog_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_content_dialog_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_content_dialog_state(widget))
            })
    }

    #[cfg(feature = "command-palette")]
    fn widget_command_palette_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsCommandPaletteState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_command_palette_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_command_palette_state(widget))
            })
    }

    #[cfg(feature = "toast")]
    fn widget_toast_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_toast_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_toast_state(widget))
            })
    }

    #[cfg(feature = "info-bar")]
    fn widget_info_bar_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_info_bar_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_info_bar_state(widget))
            })
    }

    #[cfg(feature = "breadcrumb")]
    fn widget_breadcrumb_state(&self, widget: crate::WidgetId) -> Option<crate::ZsBreadcrumbState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_breadcrumb_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_breadcrumb_state(widget))
            })
    }

    #[cfg(feature = "teaching-tip")]
    fn widget_teaching_tip_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_teaching_tip_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_teaching_tip_state(widget))
            })
    }

    #[cfg(feature = "toast")]
    fn active_toast(&self) -> Option<(crate::WidgetId, crate::ZsToastSpec)> {
        let target = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::Toast)?;
        self.widget_toast_state(target.widget)
            .map(|(_, spec)| (target.widget, spec))
    }

    #[cfg(feature = "toast")]
    fn sync_toast_runtime(&mut self, now: std::time::Instant) -> bool {
        let active = self.active_toast();
        self.toast
            .sync(active.as_ref().map(|(widget, spec)| (*widget, spec)), now)
    }

    #[cfg(feature = "combo")]
    fn widget_combo_state(&self, widget: crate::WidgetId) -> Option<(Option<usize>, usize, bool)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_combo_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_combo_state(widget))
            })
    }

    #[cfg(feature = "combo")]
    fn widget_combo_type_ahead_match(
        &self,
        widget: crate::WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_combo_type_ahead_match(widget, query, start_after))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_combo_type_ahead_match(widget, query, start_after))
            })
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "color-picker",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker"
    ))]
    fn dismiss_popup_overlays_except(
        &mut self,
        except: Option<crate::WidgetId>,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        #[cfg(feature = "auto-suggest")]
        let auto_suggest_was_dismissed = self.interaction_plan.hit_targets.iter().any(|target| {
            Some(target.widget) != except
                && target.kind == crate::ViewHitTargetKind::AutoSuggestBox
                && self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded)
        });
        #[cfg(feature = "combo")]
        let combo_was_dismissed = self.interaction_plan.hit_targets.iter().any(|target| {
            Some(target.widget) != except
                && target.kind == crate::ViewHitTargetKind::ComboBox
                && self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded)
        });
        #[cfg(feature = "color-picker")]
        let color_picker_was_dismissed = self.interaction_plan.hit_targets.iter().any(|target| {
            Some(target.widget) != except
                && target.kind == crate::ViewHitTargetKind::ColorPicker
                && self
                    .widget_color_picker_state(target.widget)
                    .is_some_and(|state| state.expanded)
        });
        let should_dismiss = self.interaction_plan.hit_targets.iter().any(|target| {
            if Some(target.widget) == except {
                return false;
            }
            match target.kind {
                #[cfg(feature = "auto-suggest")]
                crate::ViewHitTargetKind::AutoSuggestBox => self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "combo")]
                crate::ViewHitTargetKind::ComboBox => self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded),
                #[cfg(feature = "color-picker")]
                crate::ViewHitTargetKind::ColorPicker => self
                    .widget_color_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "date-picker")]
                crate::ViewHitTargetKind::DatePicker => self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "time-picker")]
                crate::ViewHitTargetKind::TimePicker => self
                    .widget_time_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                _ => false,
            }
        });
        if !should_dismiss {
            return;
        }
        #[cfg(feature = "auto-suggest")]
        {
            report.auto_suggest_expanded_change_count += usize::from(auto_suggest_was_dismissed);
        }
        #[cfg(feature = "combo")]
        {
            report.combo_expanded_change_count += usize::from(combo_was_dismissed);
        }
        #[cfg(feature = "color-picker")]
        {
            report.color_picker_expanded_change_count += usize::from(color_picker_was_dismissed);
        }
        report.handled = true;
        report.event_count += 1;
        report
            .events
            .push("win32_view_popup_overlays_dismissed".to_string());
        self.dispatch_event(crate::ViewEvent::DismissPopupOverlays { except }, report);
    }

    #[cfg(not(any(
        feature = "auto-suggest",
        feature = "color-picker",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker"
    )))]
    fn dismiss_popup_overlays_except(
        &mut self,
        _except: Option<crate::WidgetId>,
        _report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
    }

    #[cfg(feature = "date-picker")]
    fn widget_date_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsDatePickerState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_date_picker_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_date_picker_state(widget))
            })
    }

    #[cfg(feature = "time-picker")]
    fn widget_time_picker_state(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsTimePickerState> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_time_picker_state(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_time_picker_state(widget))
            })
    }

    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: crate::WidgetId,
        offset: isize,
    ) -> Option<(crate::WidgetId, usize)> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_list_relative_widget(widget, offset))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_list_relative_widget(widget, offset))
            })
    }

    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: crate::WidgetId) -> Option<usize> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_list_index(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_list_index(widget))
            })
    }

    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: crate::WidgetId) -> Option<crate::WidgetId> {
        self.live_view
            .as_ref()
            .and_then(|runtime| runtime.widget_scroll_target(widget))
            .or_else(|| {
                self.ui_command_view
                    .as_ref()
                    .and_then(|view| view.widget_scroll_target(widget))
            })
    }

    fn take_pending_draw_plan(&mut self) -> Option<NativeDrawPlan> {
        self.pending_draw_plan.take()
    }

    fn rebuild_pending_draw_plan(&mut self) -> bool {
        let mut plan = if let Some(live_view) = &self.live_view {
            live_view.draw_plan()
        } else if let Some(view) = &self.ui_command_view {
            let mut paint_cx = ViewPaintCx::new(self.dpi);
            view.paint(&mut paint_cx);
            paint_cx.into_plan()
        } else {
            return false;
        };
        #[cfg(any(
            feature = "auto-suggest",
            feature = "breadcrumb",
            feature = "color-picker",
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
        decorate_native_pointer_visuals(
            &mut plan,
            &self.interaction_plan,
            self.pointer_hover,
            self.pointer_pressed,
            self.dpi,
        );
        #[cfg(feature = "password-box")]
        self.compose_password_peek(&mut plan);
        if let (Some(target), Some(state)) = (self.focused_target(), self.text_edit) {
            if let Some(value) = self.widget_display_text_value(target.widget) {
                let target = native_text_visual_target(target, &self.interaction_plan);
                decorate_native_text_edit_visuals_in_viewport_with_backend(
                    &mut plan,
                    target,
                    &value,
                    state.selection.clamp(&value),
                    state.first_visible_visual_row,
                    state.horizontal_scroll_px,
                    self.widget_text_wrap(target.widget),
                    self.dpi,
                    &self.text_shaping,
                );
            }
        }
        decorate_native_focus_ring(
            &mut plan,
            &self.interaction_plan,
            self.focused_widget,
            self.dpi,
        );
        #[cfg(feature = "tooltip")]
        self.compose_tooltip(&mut plan);
        self.pending_draw_plan = Some(plan);
        true
    }

    fn refresh_live_view_after_app_effect(&mut self) -> Option<u64> {
        let live_view = self.live_view.as_ref()?;
        let update = live_view.refresh();
        self.interaction_plan = live_view.interaction_plan();
        self.rebuild_pending_draw_plan();
        Some(update.revision)
    }

    #[cfg(feature = "tooltip")]
    fn compose_tooltip(&self, plan: &mut NativeDrawPlan) {
        let Some(surface) = self.surface else {
            return;
        };
        let Some(target) = self.tooltip.visible_target(&self.interaction_plan) else {
            return;
        };
        let render = crate::zs_tooltip_render_plan(
            &target.spec,
            target.bounds,
            self.tooltip.anchor(),
            surface,
            crate::ZsTooltipPlatformStyle::Windows,
            self.dpi,
        );
        let overlay = crate::zs_tooltip_native_draw_plan(&render, &target.spec);
        plan.commands.extend(overlay.commands);
    }

    #[cfg(feature = "password-box")]
    fn compose_password_peek(&self, plan: &mut NativeDrawPlan) {
        let Some(widget) = self.password_peek else {
            return;
        };
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            return;
        };
        let Some(value) = self.widget_password_value(widget) else {
            return;
        };
        for command in plan.commands.iter_mut().rev() {
            let crate::NativeDrawCommand::Text(text) = command else {
                continue;
            };
            if !crate::native_input_visuals::rect_contains(target.bounds, text.bounds) {
                continue;
            }
            let bounds = text.bounds;
            let style = text.style;
            *command = crate::NativeDrawCommand::SecureText(
                crate::NativeDrawSecureTextCommand::new(value, bounds, style, true),
            );
            break;
        }
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
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
    fn update_pointer_visual_state(
        &mut self,
        hovered: Option<NativePointerVisualKey>,
        pressed: Option<NativePointerVisualKey>,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if self.pointer_hover == hovered && self.pointer_pressed == pressed {
            return;
        }
        self.pointer_hover = hovered;
        self.pointer_pressed = pressed;
        report.handled = true;
        report.pointer_visual_change_count += 1;
        self.rebuild_pending_draw_plan();
    }

    fn take_quit_requested(&mut self) -> bool {
        std::mem::take(&mut self.quit_requested)
    }

    fn take_pending_app_command_dispatch(
        &mut self,
    ) -> (Option<SharedAppCommandExecutor>, Vec<Command>) {
        (
            self.app_command_executor.clone(),
            std::mem::take(&mut self.pending_app_commands),
        )
    }

    fn take_pending_ui_command_dispatch(
        &mut self,
    ) -> (Option<SharedUiCommandExecutor>, Vec<UiCommand>) {
        (
            self.ui_command_executor.clone(),
            std::mem::take(&mut self.pending_ui_commands),
        )
    }

    fn background_poll_interval_ms(&self) -> Option<u64> {
        let live_interval = self
            .live_view
            .as_ref()
            .and_then(SharedLiveViewRuntime::background_poll_interval_ms);
        let interval = live_interval;
        #[cfg(feature = "tooltip")]
        let interval = interval
            .into_iter()
            .chain(self.tooltip.poll_interval_ms(std::time::Instant::now()))
            .min();
        #[cfg(feature = "toast")]
        let interval = interval
            .into_iter()
            .chain(self.toast.poll_interval_ms(std::time::Instant::now()))
            .min();
        interval
    }

    fn refresh_background_view(&mut self) -> WindowsWin32ViewInputDispatchReport {
        self.refresh_background_view_at(std::time::Instant::now())
    }

    fn refresh_background_view_at(
        &mut self,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(not(any(feature = "tooltip", feature = "toast")))]
        let _ = now;
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let mut redraw = false;
        if let Some(live_view) = &self.live_view {
            let update = live_view.refresh();
            self.interaction_plan = live_view.interaction_plan();
            report.background_refresh_count = 1;
            report.live_view_revision = update.revision;
            report.events.push(format!(
                "win32_live_view_background_refresh:{}",
                update.revision
            ));
            redraw = true;
        }
        #[cfg(feature = "tooltip")]
        if self.tooltip.refresh(now) {
            report.handled = true;
            report.events.push("win32_tooltip_tick".to_string());
            redraw = true;
        }
        #[cfg(feature = "toast")]
        if let Some((widget, toast)) = self.toast.take_expired(now) {
            report.handled = true;
            report.toast_response_count = 1;
            report.toast_timeout_count = 1;
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_toast_response:{}:{toast:?}:Timeout",
                widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ToastResponded {
                    widget,
                    toast,
                    response: crate::ZsToastResponse::Dismissed(
                        crate::ZsToastDismissReason::Timeout,
                    ),
                },
                &mut report,
            );
            redraw = true;
        }
        #[cfg(feature = "toast")]
        self.sync_toast_runtime(now);
        if redraw {
            self.rebuild_pending_draw_plan();
        }
        report
    }

    fn set_surface(&mut self, bounds: crate::Rect, dpi: crate::Dpi) -> bool {
        self.dpi = dpi;
        self.surface = Some(bounds);
        let mut changed = false;
        if let Some(live_view) = &self.live_view {
            if live_view.set_surface(bounds, dpi) {
                self.interaction_plan = live_view.interaction_plan();
                changed = true;
            }
        }
        if let Some(view) = &mut self.ui_command_view {
            let mut layout = crate::ViewLayoutCx::new(bounds, dpi);
            view.layout(&mut layout);
            self.interaction_plan = view.interaction_plan();
            changed = true;
        }
        if changed {
            self.rebuild_pending_draw_plan();
        }
        changed
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
    pub text_selection_change_count: usize,
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
        self.text_selection_change_count += next.text_selection_change_count;
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

pub fn dispatch_windows_win32_window_view_pointer_move(
    hwnd: HWND,
    point: crate::Point,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_pointer_move(point))
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
        .expect("view input route test lock should not be poisoned")
}

fn completed_window_view_input_reports(
) -> &'static Mutex<Vec<WindowsCompletedViewInputReportRecord>> {
    WINDOW_COMPLETED_VIEW_INPUT_REPORTS.get_or_init(|| Mutex::new(Vec::new()))
}
