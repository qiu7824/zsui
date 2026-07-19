#[derive(Debug, Clone, Default)]
pub struct LiveViewUpdate {
    pub redraw: bool,
    pub message_count: usize,
    pub commands: Vec<Command>,
    pub ui_commands: Vec<UiCommand>,
    #[cfg(feature = "textbox")]
    pub text_edit_commands: Vec<ZsTextEditCommandRequest>,
    pub quit_requested: bool,
    pub revision: u64,
}

trait LiveViewDriver: Send {
    #[allow(dead_code)]
    fn surface(&self) -> (Rect, Dpi);
    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool;
    fn set_typography_scale(&mut self, scale: f32) -> bool;
    fn suspend(&mut self) -> bool;
    fn resume(&mut self) -> LiveViewUpdate;
    fn is_suspended(&self) -> bool;
    fn refresh(&mut self) -> LiveViewUpdate;
    fn background_poll_interval_ms(&self) -> Option<u64>;
    fn draw_plan(&self) -> NativeDrawPlan;
    fn interaction_plan(&self) -> ViewInteractionPlan;
    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate;
    fn dispatch_app_command(&mut self, command: &Command) -> LiveViewUpdate;
    fn widget_text_value(&self, widget: WidgetId) -> Option<String>;
    #[cfg(feature = "textbox")]
    fn widget_text_wrap(&self, widget: WidgetId) -> Option<crate::TextWrap>;
    #[cfg(feature = "password-box")]
    fn widget_password_value(&self, widget: WidgetId) -> Option<crate::ZsPassword>;
    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool>;
    #[cfg(feature = "radio")]
    fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool>;
    #[cfg(feature = "radio")]
    fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId>;
    #[cfg(feature = "slider")]
    fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)>;
    #[cfg(feature = "auto-suggest")]
    fn widget_auto_suggest_state(&self, widget: WidgetId) -> Option<crate::ZsAutoSuggestState>;
    #[cfg(feature = "tree")]
    fn widget_tree_view_state(&self, widget: WidgetId) -> Option<crate::ZsTreeViewState>;
    #[cfg(feature = "grid-view")]
    fn widget_grid_view_state(&self, widget: WidgetId) -> Option<crate::ZsGridViewState>;
    #[cfg(feature = "table")]
    fn widget_table_state(&self, widget: WidgetId) -> Option<crate::ZsTableViewState>;
    #[cfg(feature = "dialog")]
    fn widget_content_dialog_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)>;
    #[cfg(feature = "command-palette")]
    fn widget_command_palette_state(
        &self,
        widget: WidgetId,
    ) -> Option<crate::ZsCommandPaletteState>;
    #[cfg(feature = "toast")]
    fn widget_toast_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)>;
    #[cfg(feature = "teaching-tip")]
    fn widget_teaching_tip_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)>;
    #[cfg(feature = "info-bar")]
    fn widget_info_bar_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)>;
    #[cfg(feature = "breadcrumb")]
    fn widget_breadcrumb_state(&self, widget: WidgetId) -> Option<crate::ZsBreadcrumbState>;
    #[cfg(feature = "combo")]
    fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)>;
    #[cfg(feature = "combo")]
    fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize>;
    #[cfg(feature = "date-picker")]
    fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState>;
    #[cfg(feature = "time-picker")]
    fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState>;
    #[cfg(feature = "color-picker")]
    fn widget_color_picker_state(&self, widget: WidgetId) -> Option<ZsColorPickerState>;
    #[cfg(feature = "tabs")]
    fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState>;
    #[cfg(all(test, feature = "tabs"))]
    fn widget_tab_view_state(&self, widget: WidgetId) -> Option<ZsTabViewState>;
    #[cfg(feature = "tabs")]
    fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)>;
    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)>;
    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize>;
    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId>;
    fn revision(&self) -> u64;
}

#[derive(Clone)]
pub struct SharedLiveViewRuntime {
    inner: Arc<Mutex<Box<dyn LiveViewDriver>>>,
}

impl SharedLiveViewRuntime {
    #[allow(dead_code)]
    pub(crate) fn surface(&self) -> (Rect, Dpi) {
        self.lock().surface()
    }

    pub fn set_surface(&self, bounds: Rect, dpi: Dpi) -> bool {
        self.lock().set_surface(bounds, dpi)
    }

    pub(crate) fn set_typography_scale(&self, scale: f32) -> bool {
        self.lock().set_typography_scale(scale)
    }

    pub fn draw_plan(&self) -> NativeDrawPlan {
        self.lock().draw_plan()
    }

    /// Drops the current View tree while retaining application state, update
    /// functions and command routing for a later rebuild.
    pub fn suspend(&self) -> bool {
        self.lock().suspend()
    }

    /// Rebuilds a previously suspended View tree from the retained state.
    pub fn resume(&self) -> LiveViewUpdate {
        self.lock().resume()
    }

    pub fn is_suspended(&self) -> bool {
        self.lock().is_suspended()
    }

    pub fn refresh(&self) -> LiveViewUpdate {
        self.lock().refresh()
    }

    pub fn background_poll_interval_ms(&self) -> Option<u64> {
        self.lock().background_poll_interval_ms()
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        self.lock().interaction_plan()
    }

    pub fn dispatch_event(&self, event: &ViewEvent) -> LiveViewUpdate {
        self.lock().dispatch_event(event)
    }

    pub fn dispatch_app_command(&self, command: &Command) -> LiveViewUpdate {
        self.lock().dispatch_app_command(command)
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.lock().widget_text_value(widget)
    }

    #[cfg(feature = "textbox")]
    pub fn widget_text_wrap(&self, widget: WidgetId) -> Option<crate::TextWrap> {
        self.lock().widget_text_wrap(widget)
    }

    #[cfg(feature = "password-box")]
    pub fn widget_password_value(&self, widget: WidgetId) -> Option<crate::ZsPassword> {
        self.lock().widget_password_value(widget)
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.lock().widget_checked_value(widget)
    }

    #[cfg(feature = "auto-suggest")]
    pub fn widget_auto_suggest_state(&self, widget: WidgetId) -> Option<crate::ZsAutoSuggestState> {
        self.lock().widget_auto_suggest_state(widget)
    }

    #[cfg(feature = "tree")]
    pub fn widget_tree_view_state(&self, widget: WidgetId) -> Option<crate::ZsTreeViewState> {
        self.lock().widget_tree_view_state(widget)
    }

    #[cfg(feature = "grid-view")]
    pub fn widget_grid_view_state(&self, widget: WidgetId) -> Option<crate::ZsGridViewState> {
        self.lock().widget_grid_view_state(widget)
    }

    #[cfg(feature = "table")]
    pub fn widget_table_state(&self, widget: WidgetId) -> Option<crate::ZsTableViewState> {
        self.lock().widget_table_state(widget)
    }

    #[cfg(feature = "dialog")]
    pub fn widget_content_dialog_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)> {
        self.lock().widget_content_dialog_state(widget)
    }

    #[cfg(feature = "command-palette")]
    pub fn widget_command_palette_state(
        &self,
        widget: WidgetId,
    ) -> Option<crate::ZsCommandPaletteState> {
        self.lock().widget_command_palette_state(widget)
    }

    #[cfg(feature = "toast")]
    pub fn widget_toast_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)> {
        self.lock().widget_toast_state(widget)
    }

    #[cfg(feature = "teaching-tip")]
    pub fn widget_teaching_tip_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)> {
        self.lock().widget_teaching_tip_state(widget)
    }

    #[cfg(feature = "info-bar")]
    pub fn widget_info_bar_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)> {
        self.lock().widget_info_bar_state(widget)
    }

    #[cfg(feature = "breadcrumb")]
    pub fn widget_breadcrumb_state(&self, widget: WidgetId) -> Option<crate::ZsBreadcrumbState> {
        self.lock().widget_breadcrumb_state(widget)
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        self.lock().widget_radio_is_tab_stop(widget)
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        self.lock()
            .widget_radio_relative_widget(widget, navigation, offset)
    }

    #[cfg(feature = "slider")]
    pub fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        self.lock().widget_slider_state(widget)
    }

    #[cfg(feature = "combo")]
    pub fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        self.lock().widget_combo_state(widget)
    }

    #[cfg(feature = "combo")]
    pub(crate) fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        self.lock()
            .widget_combo_type_ahead_match(widget, query, start_after)
    }

    #[cfg(feature = "date-picker")]
    pub fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        self.lock().widget_date_picker_state(widget)
    }

    #[cfg(feature = "time-picker")]
    pub fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        self.lock().widget_time_picker_state(widget)
    }

    #[cfg(feature = "color-picker")]
    pub fn widget_color_picker_state(&self, widget: WidgetId) -> Option<ZsColorPickerState> {
        self.lock().widget_color_picker_state(widget)
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        self.lock().widget_tab_header_state(widget)
    }

    #[cfg(all(test, feature = "tabs"))]
    pub(crate) fn widget_tab_view_state(&self, widget: WidgetId) -> Option<ZsTabViewState> {
        self.lock().widget_tab_view_state(widget)
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        self.lock().widget_tab_cycle_target(focused, offset)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.lock().widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.lock().widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.lock().widget_scroll_target(widget)
    }

    pub fn revision(&self) -> u64 {
        self.lock().revision()
    }

    fn lock(&self) -> MutexGuard<'_, Box<dyn LiveViewDriver>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for SharedLiveViewRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedLiveViewRuntime")
            .field("revision", &self.revision())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SharedLiveViewRuntime {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

struct TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    app_command_mapper: Option<Box<dyn Fn(&Command) -> Option<Msg> + Send>>,
    view: ViewNode<Msg>,
    bounds: Rect,
    dpi: Dpi,
    typography_scale_per_mille: u16,
    revision: u64,
    animation_epoch: std::time::Instant,
    suspended: bool,
}

impl<State, Msg, ViewFn, UpdateFn> TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    fn new(
        state: State,
        view_fn: ViewFn,
        update_fn: UpdateFn,
        app_command_mapper: Option<Box<dyn Fn(&Command) -> Option<Msg> + Send>>,
        bounds: Rect,
        dpi: Dpi,
    ) -> Self {
        let view = view_fn(&state);
        let mut driver = Self {
            state,
            view_fn,
            update_fn,
            app_command_mapper,
            view,
            bounds,
            dpi,
            typography_scale_per_mille:
                crate::render_protocol::default_typography_scale_per_mille(),
            revision: 0,
            animation_epoch: std::time::Instant::now(),
            suspended: false,
        };
        driver.layout_current_view();
        driver
    }

    fn rebuild_and_layout(&mut self) {
        if self.suspended {
            return;
        }
        self.view = (self.view_fn)(&self.state);
        self.layout_current_view();
    }

    fn layout_current_view(&mut self) {
        let mut cx = ViewLayoutCx::new(self.bounds, self.dpi)
            .with_typography_scale(self.typography_scale());
        self.view.layout(&mut cx);
    }

    fn typography_scale(&self) -> f32 {
        f32::from(self.typography_scale_per_mille) / 1_000.0
    }

    fn apply_messages(&mut self, messages: Vec<Msg>) -> LiveViewUpdate {
        let message_count = messages.len();
        let mut app_cx = AppCx::new();
        for message in messages {
            (self.update_fn)(&mut self.state, message, &mut app_cx);
        }
        self.rebuild_and_layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: !self.suspended,
            message_count,
            commands: app_cx.commands().to_vec(),
            ui_commands: app_cx.ui_commands().to_vec(),
            #[cfg(feature = "textbox")]
            text_edit_commands: app_cx.text_edit_commands().to_vec(),
            quit_requested: app_cx.quit_requested(),
            revision: self.revision,
        }
    }
}

impl<State, Msg, ViewFn, UpdateFn> LiveViewDriver
    for TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    fn surface(&self) -> (Rect, Dpi) {
        (self.bounds, self.dpi)
    }

    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool {
        if self.bounds == bounds && self.dpi == dpi {
            return false;
        }
        self.bounds = bounds;
        self.dpi = dpi;
        if !self.suspended {
            self.layout_current_view();
        }
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn set_typography_scale(&mut self, scale: f32) -> bool {
        let scale = crate::render_protocol::normalize_typography_scale_per_mille(scale);
        if self.typography_scale_per_mille == scale {
            return false;
        }
        self.typography_scale_per_mille = scale;
        if !self.suspended {
            self.layout_current_view();
        }
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn suspend(&mut self) -> bool {
        if self.suspended {
            return false;
        }
        self.view = ViewNode::new(ViewNodeKind::Spacer);
        self.suspended = true;
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn resume(&mut self) -> LiveViewUpdate {
        if !self.suspended {
            return LiveViewUpdate {
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }
        self.suspended = false;
        self.animation_epoch = std::time::Instant::now();
        self.rebuild_and_layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: true,
            revision: self.revision,
            ..LiveViewUpdate::default()
        }
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn refresh(&mut self) -> LiveViewUpdate {
        if self.suspended {
            return LiveViewUpdate {
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }
        self.rebuild_and_layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: true,
            revision: self.revision,
            ..LiveViewUpdate::default()
        }
    }

    fn background_poll_interval_ms(&self) -> Option<u64> {
        (!self.suspended)
            .then(|| self.view.background_poll_interval_ms())
            .flatten()
    }

    fn draw_plan(&self) -> NativeDrawPlan {
        if self.suspended {
            return NativeDrawPlan::default();
        }
        let mut cx = ViewPaintCx::with_animation_elapsed(self.dpi, self.animation_epoch.elapsed());
        cx.set_typography_scale(self.typography_scale());
        self.view.paint(&mut cx);
        cx.into_plan()
    }

    fn interaction_plan(&self) -> ViewInteractionPlan {
        if self.suspended {
            ViewInteractionPlan::default()
        } else {
            self.view.interaction_plan()
        }
    }

    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate {
        if self.suspended {
            return LiveViewUpdate {
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }
        let mut event_cx = ViewEventCx::new();
        self.view.event(&mut event_cx, event);
        let messages = event_cx.into_messages();
        if messages.is_empty() {
            self.revision = self.revision.saturating_add(1);
            return LiveViewUpdate {
                redraw: true,
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }

        self.apply_messages(messages)
    }

    fn dispatch_app_command(&mut self, command: &Command) -> LiveViewUpdate {
        let Some(message) = self
            .app_command_mapper
            .as_ref()
            .and_then(|mapper| mapper(command))
        else {
            return LiveViewUpdate {
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        };
        self.apply_messages(vec![message])
    }

    fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.view.widget_text_value(widget).map(str::to_string)
    }

    #[cfg(feature = "textbox")]
    fn widget_text_wrap(&self, widget: WidgetId) -> Option<crate::TextWrap> {
        self.view.widget_text_wrap(widget)
    }

    #[cfg(feature = "password-box")]
    fn widget_password_value(&self, widget: WidgetId) -> Option<crate::ZsPassword> {
        self.view.widget_password_value(widget).cloned()
    }

    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.view.widget_checked_value(widget)
    }

    #[cfg(feature = "radio")]
    fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        self.view.widget_radio_is_tab_stop(widget)
    }

    #[cfg(feature = "radio")]
    fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        self.view
            .widget_radio_relative_widget(widget, navigation, offset)
    }

    #[cfg(feature = "slider")]
    fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        self.view.widget_slider_state(widget)
    }

    #[cfg(feature = "combo")]
    fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        self.view.widget_combo_state(widget)
    }

    #[cfg(feature = "auto-suggest")]
    fn widget_auto_suggest_state(&self, widget: WidgetId) -> Option<crate::ZsAutoSuggestState> {
        self.view.widget_auto_suggest_state(widget)
    }

    #[cfg(feature = "tree")]
    fn widget_tree_view_state(&self, widget: WidgetId) -> Option<crate::ZsTreeViewState> {
        self.view.widget_tree_view_state(widget)
    }

    #[cfg(feature = "grid-view")]
    fn widget_grid_view_state(&self, widget: WidgetId) -> Option<crate::ZsGridViewState> {
        self.view.widget_grid_view_state(widget)
    }

    #[cfg(feature = "table")]
    fn widget_table_state(&self, widget: WidgetId) -> Option<crate::ZsTableViewState> {
        self.view.widget_table_state(widget)
    }

    #[cfg(feature = "dialog")]
    fn widget_content_dialog_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)> {
        self.view.widget_content_dialog_state(widget)
    }

    #[cfg(feature = "command-palette")]
    fn widget_command_palette_state(
        &self,
        widget: WidgetId,
    ) -> Option<crate::ZsCommandPaletteState> {
        self.view.widget_command_palette_state(widget)
    }

    #[cfg(feature = "toast")]
    fn widget_toast_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)> {
        self.view.widget_toast_state(widget)
    }

    #[cfg(feature = "teaching-tip")]
    fn widget_teaching_tip_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)> {
        self.view.widget_teaching_tip_state(widget)
    }

    #[cfg(feature = "info-bar")]
    fn widget_info_bar_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)> {
        self.view.widget_info_bar_state(widget)
    }

    #[cfg(feature = "breadcrumb")]
    fn widget_breadcrumb_state(&self, widget: WidgetId) -> Option<crate::ZsBreadcrumbState> {
        self.view.widget_breadcrumb_state(widget)
    }

    #[cfg(feature = "combo")]
    fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        self.view
            .widget_combo_type_ahead_match(widget, query, start_after)
    }

    #[cfg(feature = "date-picker")]
    fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        self.view.widget_date_picker_state(widget)
    }

    #[cfg(feature = "time-picker")]
    fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        self.view.widget_time_picker_state(widget)
    }

    #[cfg(feature = "color-picker")]
    fn widget_color_picker_state(&self, widget: WidgetId) -> Option<ZsColorPickerState> {
        self.view.widget_color_picker_state(widget)
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        self.view.widget_tab_header_state(widget)
    }

    #[cfg(all(test, feature = "tabs"))]
    fn widget_tab_view_state(&self, widget: WidgetId) -> Option<ZsTabViewState> {
        self.view.widget_tab_view_state(widget)
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        self.view.widget_tab_cycle_target(focused, offset)
    }

    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.view.widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.view.widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.view.widget_scroll_target(widget)
    }

    fn revision(&self) -> u64 {
        self.revision
    }
}

pub fn live_view_runtime<State, Msg, ViewFn, UpdateFn>(
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    bounds: Rect,
    dpi: Dpi,
) -> SharedLiveViewRuntime
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    SharedLiveViewRuntime {
        inner: Arc::new(Mutex::new(Box::new(TypedLiveViewDriver::new(
            state, view_fn, update_fn, None, bounds, dpi,
        )))),
    }
}

pub fn live_view_runtime_with_app_commands<State, Msg, ViewFn, UpdateFn, CommandFn>(
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    command_fn: CommandFn,
    bounds: Rect,
    dpi: Dpi,
) -> SharedLiveViewRuntime
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
    CommandFn: Fn(&Command) -> Option<Msg> + Send + 'static,
{
    SharedLiveViewRuntime {
        inner: Arc::new(Mutex::new(Box::new(TypedLiveViewDriver::new(
            state,
            view_fn,
            update_fn,
            Some(Box::new(command_fn)),
            bounds,
            dpi,
        )))),
    }
}
