use serde::Serialize;

use crate::{
    app::{app, ZsuiApp, ZsuiAppRuntime},
    capability::HostCapabilities,
    clipboard::ClipboardData,
    command_protocol::UiCommand,
    core::{
        AppEvent, DialogResponse, FileDialogSpec, HotkeyId, NativeDialogSpec, TrayId, WindowId,
        ZsuiError, ZsuiResult,
    },
    geometry::{Dpi, Point, Rect},
    host::{MemoryHost, TrayRecord, WindowRecord, ZsuiHost},
    hotkey::HotkeySpec,
    menu::{MenuItemSpec, MenuSpec},
    native_hosts::{
        native_status_menu_command_from_menu, NativeMainWindowHandles, NativeRuntimeDriver,
        NativeRuntimeStartupRequest, NativeRuntimeStartupResult, NativeSettingsItemUpdateHost,
        NativeSettingsItemUpdateRequest, NativeSettingsItemUpdateResult,
        NativeSettingsPageModelHost, NativeSettingsPageModelPresentation,
        NativeSettingsPageModelRequest, NativeStatusItemHost, NativeStatusItemPresentation,
        NativeStatusItemRequest, NativeStatusMenuCommandHost, NativeStatusMenuCommandRequest,
        NativeStatusMenuCommandResult,
    },
    render_protocol::NativeDrawPlan,
    settings::SettingsPageSpec,
    tray::TraySpec,
    view::{
        View, ViewEvent, ViewEventCx, ViewInteractionPlan, ViewLayoutCx, ViewNode, ViewPaintCx,
    },
    window::{Window, WindowSpec},
};

pub fn native_window(title: impl Into<String>) -> NativeWindowBuilder {
    NativeWindowBuilder::new(title)
}

pub fn run_native_window(title: impl Into<String>) -> ZsuiResult<ZsuiAppRuntime> {
    native_window(title).run()
}

pub fn run_native_window_smoke(
    title: impl Into<String>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    native_window(title).run_smoke(options)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NativeWindowRuntimeHandle(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeWindowRuntimeDriverReport {
    pub capabilities: HostCapabilities,
    pub started: bool,
    pub startup_request_count: usize,
    pub window_count: usize,
    pub status_item_count: usize,
    pub status_menu_entry_count: usize,
    pub settings_page_count: usize,
    pub status_item_handle_count: usize,
    pub settings_model_bound: bool,
    pub native_operation_names: Vec<&'static str>,
    pub command_ids: Vec<&'static str>,
    pub pending_event_count: usize,
    pub shutdown_requested: bool,
    pub handles_created: bool,
}

#[derive(Debug, Clone)]
pub struct NativeWindowRuntimeDriver {
    capabilities: HostCapabilities,
    startup_requests: Vec<NativeRuntimeStartupRequest>,
    windows: Vec<WindowSpec>,
    status_items: Vec<TraySpec>,
    status_item_handles: Vec<NativeWindowRuntimeHandle>,
    settings_pages: Vec<SettingsPageSpec>,
    settings_model_bound: bool,
    native_operation_names: Vec<&'static str>,
    command_ids: Vec<&'static str>,
    events: Vec<AppEvent>,
    handles: Option<NativeMainWindowHandles<NativeWindowRuntimeHandle>>,
    next_handle: u64,
    shutdown_requested: bool,
}

impl NativeWindowRuntimeDriver {
    pub fn new() -> Self {
        Self::with_capabilities(HostCapabilities::current_native_window_host())
    }

    pub fn with_capabilities(capabilities: HostCapabilities) -> Self {
        Self {
            capabilities,
            startup_requests: Vec::new(),
            windows: Vec::new(),
            status_items: Vec::new(),
            status_item_handles: Vec::new(),
            settings_pages: Vec::new(),
            settings_model_bound: false,
            native_operation_names: Vec::new(),
            command_ids: Vec::new(),
            events: Vec::new(),
            handles: None,
            next_handle: 1,
            shutdown_requested: false,
        }
    }

    pub fn capabilities(&self) -> HostCapabilities {
        self.capabilities.clone()
    }

    pub fn startup_requests(&self) -> &[NativeRuntimeStartupRequest] {
        &self.startup_requests
    }

    pub fn window_specs(&self) -> &[WindowSpec] {
        &self.windows
    }

    pub fn status_item_specs(&self) -> &[TraySpec] {
        &self.status_items
    }

    pub fn settings_page_specs(&self) -> &[SettingsPageSpec] {
        &self.settings_pages
    }

    pub fn status_item_handles(&self) -> &[NativeWindowRuntimeHandle] {
        &self.status_item_handles
    }

    pub const fn settings_model_bound(&self) -> bool {
        self.settings_model_bound
    }

    pub fn native_operation_names(&self) -> &[&'static str] {
        &self.native_operation_names
    }

    pub fn command_ids(&self) -> &[&'static str] {
        &self.command_ids
    }

    pub const fn handles(&self) -> Option<NativeMainWindowHandles<NativeWindowRuntimeHandle>> {
        self.handles
    }

    pub const fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    pub fn report(&self) -> NativeWindowRuntimeDriverReport {
        let handles_created = self.handles.is_some() || !self.startup_requests.is_empty();
        NativeWindowRuntimeDriverReport {
            capabilities: self.capabilities.clone(),
            started: !self.startup_requests.is_empty(),
            startup_request_count: self.startup_requests.len(),
            window_count: self.windows.len(),
            status_item_count: self.status_items.len(),
            status_menu_entry_count: self
                .status_items
                .iter()
                .map(|item| menu_entry_count(&item.menu))
                .sum(),
            settings_page_count: self.settings_pages.len(),
            status_item_handle_count: self.status_item_handles.len(),
            settings_model_bound: self.settings_model_bound,
            native_operation_names: self.native_operation_names.clone(),
            command_ids: self.command_ids.clone(),
            pending_event_count: self.events.len(),
            shutdown_requested: self.shutdown_requested,
            handles_created,
        }
    }

    pub fn run_started_window_smoke(
        &self,
        options: NativeWindowSmokeRunOptions,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        run_native_window_smoke_event_loop(
            self.windows.clone(),
            Vec::new(),
            NativeViewInputRuntime::default(),
            options,
        )
    }
}

impl Default for NativeWindowRuntimeDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRuntimeDriver<UiCommand, AppEvent> for NativeWindowRuntimeDriver {
    type WindowHandle = NativeWindowRuntimeHandle;

    fn start_runtime(
        &mut self,
        request: NativeRuntimeStartupRequest,
    ) -> NativeRuntimeStartupResult<Self::WindowHandle> {
        let window = window_spec_from_startup_request(&request);
        let status_item = request.status_item.clone();
        let settings_pages = request.settings_pages.clone();
        let handles = NativeMainWindowHandles {
            main: self.allocate_handle(),
            quick: self.allocate_handle(),
        };
        self.startup_requests.push(request);
        self.windows.push(window);
        if let Some(status_item) = status_item {
            let presentation =
                self.create_status_item(NativeStatusItemRequest::from_tray_spec(&status_item));
            if matches!(presentation, NativeStatusItemPresentation::Failed) {
                return NativeRuntimeStartupResult::Failed;
            }
        }
        if !settings_pages.is_empty() {
            let presentation =
                self.bind_settings_pages(NativeSettingsPageModelRequest::new(settings_pages));
            if matches!(presentation, NativeSettingsPageModelPresentation::Failed) {
                return NativeRuntimeStartupResult::Failed;
            }
        }
        self.handles = Some(handles);
        self.shutdown_requested = false;
        self.events.push(AppEvent::Started);
        self.events.push(AppEvent::WindowCreated {
            window: WindowId(handles.main.0),
        });
        NativeRuntimeStartupResult::Started(handles)
    }

    fn dispatch_ui_command(&mut self, command: UiCommand) {
        let command_id = command.id.0;
        self.command_ids.push(command_id);
        self.events.push(AppEvent::Custom {
            id: command_id.to_string(),
            payload: None,
        });
    }

    fn poll_application_event(&mut self) -> Option<AppEvent> {
        self.events.pop()
    }

    fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.handles = None;
        for handle in self.status_item_handles.clone() {
            self.destroy_status_item(handle);
        }
        self.clear_settings_pages();
        self.events.push(AppEvent::QuitRequested);
    }
}

impl NativeWindowRuntimeDriver {
    fn allocate_handle(&mut self) -> NativeWindowRuntimeHandle {
        let handle = NativeWindowRuntimeHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    fn record_native_operation(&mut self, operation_name: &'static str) {
        self.native_operation_names.push(operation_name);
    }
}

impl NativeStatusItemHost for NativeWindowRuntimeDriver {
    type Handle = NativeWindowRuntimeHandle;

    fn create_status_item(
        &mut self,
        request: NativeStatusItemRequest,
    ) -> NativeStatusItemPresentation<Self::Handle> {
        self.record_native_operation("create_status_item");
        let handle = self.allocate_handle();
        self.status_item_handles.push(handle);
        self.status_items.push(request.into_tray_spec());
        NativeStatusItemPresentation::Created(handle)
    }

    fn set_status_item_tooltip(&mut self, handle: Self::Handle, tooltip: Option<String>) {
        self.record_native_operation("set_status_item_tooltip");
        if let Some(index) = self
            .status_item_handles
            .iter()
            .position(|candidate| *candidate == handle)
        {
            self.status_items[index].tooltip = tooltip;
        }
    }

    fn set_status_item_menu(&mut self, handle: Self::Handle, menu: MenuSpec) {
        self.record_native_operation("set_status_item_menu");
        if let Some(index) = self
            .status_item_handles
            .iter()
            .position(|candidate| *candidate == handle)
        {
            self.status_items[index].menu = menu;
        }
    }

    fn destroy_status_item(&mut self, handle: Self::Handle) {
        self.record_native_operation("destroy_status_item");
        let _ = self
            .status_item_handles
            .iter()
            .position(|candidate| *candidate == handle);
    }
}

impl NativeSettingsPageModelHost for NativeWindowRuntimeDriver {
    fn bind_settings_pages(
        &mut self,
        request: NativeSettingsPageModelRequest,
    ) -> NativeSettingsPageModelPresentation {
        self.record_native_operation("bind_settings_pages");
        let page_count = request.page_count();
        let item_count = request.item_count();
        self.settings_pages = request.pages;
        self.settings_model_bound = true;
        NativeSettingsPageModelPresentation::Bound {
            page_count,
            item_count,
        }
    }

    fn update_settings_pages(&mut self, request: NativeSettingsPageModelRequest) {
        self.record_native_operation("update_settings_pages");
        self.settings_pages = request.pages;
        self.settings_model_bound = true;
    }

    fn clear_settings_pages(&mut self) {
        self.record_native_operation("clear_settings_pages");
        self.settings_model_bound = false;
    }
}

impl NativeStatusMenuCommandHost for NativeWindowRuntimeDriver {
    fn dispatch_status_menu_command(
        &mut self,
        request: NativeStatusMenuCommandRequest,
    ) -> NativeStatusMenuCommandResult {
        self.record_native_operation("dispatch_status_menu_command");
        let Some(status_item) = self.status_items.get(request.status_item_index) else {
            return NativeStatusMenuCommandResult::NotFound;
        };
        let result = native_status_menu_command_from_menu(&status_item.menu, &request);
        if let NativeStatusMenuCommandResult::Dispatched(command) = &result {
            self.events.push(AppEvent::TrayCommand {
                command: command.clone(),
            });
        }
        result
    }
}

impl NativeSettingsItemUpdateHost for NativeWindowRuntimeDriver {
    fn update_settings_item_value(
        &mut self,
        request: NativeSettingsItemUpdateRequest,
    ) -> NativeSettingsItemUpdateResult {
        self.record_native_operation("update_settings_item_value");
        if !self.settings_model_bound {
            return NativeSettingsItemUpdateResult::NotBound;
        }

        let page_id = request.page_id;
        let item_id = request.item_id;
        let value = request.value;
        let Some(page) = self
            .settings_pages
            .iter_mut()
            .find(|page| page.id == page_id.as_str())
        else {
            return NativeSettingsItemUpdateResult::PageNotFound;
        };
        let Some(item) = page
            .items
            .iter_mut()
            .find(|item| item.id == item_id.as_str())
        else {
            return NativeSettingsItemUpdateResult::ItemNotFound;
        };

        item.default_value = Some(value);
        self.events.push(AppEvent::SettingsChanged {
            page: page_id,
            item: item_id,
        });
        NativeSettingsItemUpdateResult::Updated
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeWindowSmokeRunOptions {
    pub auto_close_after_ms: u64,
    pub require_visible_window: bool,
    pub screenshot_file: Option<String>,
    pub require_screenshot: bool,
    pub status_item: Option<TraySpec>,
    pub require_status_item: bool,
    pub native_view_click_points: Vec<Point>,
    pub native_view_text_inputs: Vec<String>,
}

impl NativeWindowSmokeRunOptions {
    pub const fn new(auto_close_after_ms: u64) -> Self {
        Self {
            auto_close_after_ms,
            require_visible_window: true,
            screenshot_file: None,
            require_screenshot: false,
            status_item: None,
            require_status_item: false,
            native_view_click_points: Vec::new(),
            native_view_text_inputs: Vec::new(),
        }
    }

    pub const fn quick() -> Self {
        Self::new(750)
    }

    pub const fn require_visible_window(mut self, require_visible_window: bool) -> Self {
        self.require_visible_window = require_visible_window;
        self
    }

    pub fn screenshot_file(mut self, screenshot_file: impl Into<String>) -> Self {
        self.screenshot_file = Some(screenshot_file.into());
        self
    }

    pub const fn require_screenshot(mut self, require_screenshot: bool) -> Self {
        self.require_screenshot = require_screenshot;
        self
    }

    pub fn status_item(mut self, status_item: TraySpec) -> Self {
        self.status_item = Some(status_item);
        self
    }

    pub const fn require_status_item(mut self, require_status_item: bool) -> Self {
        self.require_status_item = require_status_item;
        self
    }

    pub fn native_view_click(mut self, point: Point) -> Self {
        self.native_view_click_points.push(point);
        self
    }

    pub fn native_view_clicks(mut self, points: impl IntoIterator<Item = Point>) -> Self {
        self.native_view_click_points.extend(points);
        self
    }

    pub fn native_view_text_input(mut self, text: impl Into<String>) -> Self {
        self.native_view_text_inputs.push(text.into());
        self
    }

    pub fn native_view_text_inputs<I, S>(mut self, texts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.native_view_text_inputs
            .extend(texts.into_iter().map(Into::into));
        self
    }
}

impl Default for NativeWindowSmokeRunOptions {
    fn default() -> Self {
        Self::quick()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeWindowSmokeRunReport {
    pub requested_window_count: usize,
    pub created_window_count: usize,
    pub close_requested_count: usize,
    pub auto_close_after_ms: u64,
    pub exited_by_auto_close: bool,
    pub startup_error: Option<String>,
    pub screenshot_file: Option<String>,
    pub screenshot_captured: bool,
    pub screenshot_error: Option<String>,
    pub draw_plan_requested: bool,
    pub draw_plan_window_count: usize,
    pub draw_command_count: usize,
    pub text_command_count: usize,
    pub native_view_hit_target_count: usize,
    pub native_view_click_count: usize,
    pub native_view_event_count: usize,
    pub native_view_message_count: usize,
    pub native_view_ui_command_count: usize,
    pub native_view_ui_command_ids: Vec<&'static str>,
    pub native_view_unhandled_click_count: usize,
    pub native_view_focus_count: usize,
    pub native_view_text_input_count: usize,
    pub native_view_toggle_count: usize,
    pub status_item_requested: bool,
    pub status_item_required: bool,
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
    pub events: Vec<String>,
}

impl NativeWindowSmokeRunReport {
    pub fn empty(options: NativeWindowSmokeRunOptions) -> Self {
        let status_item_requested = options.status_item.is_some();
        let status_item_menu_item_count = options
            .status_item
            .as_ref()
            .map(|status_item| status_item.menu.items.len())
            .unwrap_or(0);
        Self {
            requested_window_count: 0,
            created_window_count: 0,
            close_requested_count: 0,
            auto_close_after_ms: options.auto_close_after_ms,
            exited_by_auto_close: false,
            startup_error: None,
            screenshot_file: options.screenshot_file,
            screenshot_captured: false,
            screenshot_error: None,
            draw_plan_requested: false,
            draw_plan_window_count: 0,
            draw_command_count: 0,
            text_command_count: 0,
            native_view_hit_target_count: 0,
            native_view_click_count: 0,
            native_view_event_count: 0,
            native_view_message_count: 0,
            native_view_ui_command_count: 0,
            native_view_ui_command_ids: Vec::new(),
            native_view_unhandled_click_count: 0,
            native_view_focus_count: 0,
            native_view_text_input_count: 0,
            native_view_toggle_count: 0,
            status_item_requested,
            status_item_required: options.require_status_item,
            status_item_created: false,
            status_item_menu_item_count,
            status_item_error: None,
            status_menu_native_command_count: 0,
            status_menu_command_routed: false,
            status_menu_command_error: None,
            status_menu_popup_created: false,
            status_menu_popup_command_count: 0,
            status_menu_popup_destroyed: false,
            status_menu_popup_error: None,
            events: Vec::new(),
        }
    }

    pub fn visible_window_was_created(&self) -> bool {
        self.created_window_count > 0 && self.startup_error.is_none()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
struct NativeViewInputRuntime {
    interaction_plan: Option<ViewInteractionPlan>,
    ui_command_view: Option<ViewNode<UiCommand>>,
}

#[allow(dead_code)]
impl NativeViewInputRuntime {
    fn new(
        interaction_plan: Option<ViewInteractionPlan>,
        ui_command_view: Option<ViewNode<UiCommand>>,
    ) -> Self {
        Self {
            interaction_plan,
            ui_command_view,
        }
    }

    fn hit_target_count(&self) -> usize {
        self.interaction_plan
            .as_ref()
            .map(ViewInteractionPlan::hit_target_count)
            .unwrap_or(0)
    }

    #[cfg(all(windows, feature = "windows-win32"))]
    fn windows_win32_route(&self) -> Option<crate::windows_win32_host::WindowsWin32ViewInputRoute> {
        Some(crate::windows_win32_host::WindowsWin32ViewInputRoute::new(
            self.interaction_plan.clone()?,
            self.ui_command_view.clone()?,
        ))
    }
}

#[allow(dead_code)]
fn record_draw_plan_smoke(
    report: &mut NativeWindowSmokeRunReport,
    draw_plans: &[Option<NativeDrawPlan>],
) {
    report.draw_plan_window_count = draw_plans.iter().filter(|plan| plan.is_some()).count();
    report.draw_plan_requested = report.draw_plan_window_count > 0;
    report.draw_command_count = draw_plans
        .iter()
        .filter_map(|plan| plan.as_ref())
        .map(NativeDrawPlan::command_count)
        .sum();
    report.text_command_count = draw_plans
        .iter()
        .filter_map(|plan| plan.as_ref())
        .map(NativeDrawPlan::text_count)
        .sum();
    if report.draw_plan_requested {
        report.events.push(format!(
            "draw_plan_attached:{}:{}",
            report.draw_plan_window_count, report.draw_command_count
        ));
    }
}

#[allow(dead_code)]
fn record_native_view_input_smoke(
    report: &mut NativeWindowSmokeRunReport,
    runtime: &mut NativeViewInputRuntime,
    options: &NativeWindowSmokeRunOptions,
) {
    report.native_view_hit_target_count = runtime.hit_target_count();
    if options.native_view_click_points.is_empty() {
        return;
    }

    for point in &options.native_view_click_points {
        report.native_view_click_count += 1;
        let Some(event) = runtime
            .interaction_plan
            .as_ref()
            .and_then(|plan| plan.click_event_at(*point))
        else {
            report.native_view_unhandled_click_count += 1;
            report
                .events
                .push(format!("native_view_click_missed:{}:{}", point.x, point.y));
            continue;
        };

        report.native_view_event_count += 1;
        if let ViewEvent::Click { widget } = &event {
            report
                .events
                .push(format!("native_view_click:{}", widget.0));
        }

        let Some(view) = &mut runtime.ui_command_view else {
            report
                .events
                .push("native_view_event_without_ui_command_view".to_string());
            continue;
        };

        let mut event_cx = ViewEventCx::new();
        view.event(&mut event_cx, &event);
        let commands = event_cx.into_messages();
        report.native_view_message_count += commands.len();
        report.native_view_ui_command_count += commands.len();
        for command in commands {
            report.native_view_ui_command_ids.push(command.id.0);
            report
                .events
                .push(format!("native_view_ui_command:{}", command.id.0));
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn record_windows_win32_view_input_report(
    report: &mut NativeWindowSmokeRunReport,
    input: &crate::windows_win32_host::WindowsWin32ViewInputDispatchReport,
) {
    report.native_view_hit_target_count = input.hit_target_count;
    report.native_view_click_count += input.click_count;
    report.native_view_event_count += input.event_count;
    report.native_view_message_count += input.message_count;
    report.native_view_ui_command_count += input.ui_command_count;
    report
        .native_view_ui_command_ids
        .extend(input.ui_command_ids.iter().copied());
    report.native_view_unhandled_click_count += input.unhandled_click_count;
    report.native_view_focus_count += input.focus_count;
    report.native_view_text_input_count += input.text_input_count;
    report.native_view_toggle_count += input.toggle_count;
    report.events.extend(input.events.clone());
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_lparam_from_point(point: Point) -> isize {
    let x = point.x as i16 as u16 as u32;
    let y = point.y as i16 as u16 as u32;
    ((y << 16) | x) as isize
}

#[derive(Debug, Clone)]
pub struct NativeWindowBuilder {
    app_name: String,
    window: WindowSpec,
    draw_plan: Option<NativeDrawPlan>,
    view_interaction_plan: Option<ViewInteractionPlan>,
    view_ui_command_tree: Option<ViewNode<UiCommand>>,
    view_layout_node_count: usize,
}

impl PartialEq for NativeWindowBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.app_name == other.app_name
            && self.window == other.window
            && self.draw_plan == other.draw_plan
            && self.view_interaction_plan == other.view_interaction_plan
            && self.view_ui_command_tree.is_some() == other.view_ui_command_tree.is_some()
            && self.view_layout_node_count == other.view_layout_node_count
    }
}

impl NativeWindowBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            app_name: title.clone(),
            window: Window::new(title),
            draw_plan: None,
            view_interaction_plan: None,
            view_ui_command_tree: None,
            view_layout_node_count: 0,
        }
    }

    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window = self.window.size(width, height);
        self
    }

    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.window = self.window.min_size(width, height);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.window = self.window.visible(visible);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window = self.window.resizable(resizable);
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.window = self.window.decorations(decorations);
        self
    }

    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.window = self.window.always_on_top(always_on_top);
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.window = self.window.transparent(transparent);
        self
    }

    pub fn draw_plan(mut self, draw_plan: NativeDrawPlan) -> Self {
        self.draw_plan = Some(draw_plan);
        self.view_interaction_plan = None;
        self.view_ui_command_tree = None;
        self.view_layout_node_count = 0;
        self
    }

    pub fn view<Msg: Clone>(mut self, mut view: ViewNode<Msg>) -> Self {
        let mut layout_cx = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: self.window.width as i32,
                height: self.window.height as i32,
            },
            Dpi::standard(),
        );
        let layout = view.layout(&mut layout_cx);
        let mut paint_cx = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint_cx);
        self.view_interaction_plan = Some(view.interaction_plan());
        self.view_ui_command_tree = None;
        self.view_layout_node_count = layout.children.len();
        self.draw_plan = Some(paint_cx.into_plan());
        self
    }

    pub fn ui_command_view(mut self, mut view: ViewNode<UiCommand>) -> Self {
        let mut layout_cx = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: self.window.width as i32,
                height: self.window.height as i32,
            },
            Dpi::standard(),
        );
        let layout = view.layout(&mut layout_cx);
        let mut paint_cx = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint_cx);
        self.view_interaction_plan = Some(view.interaction_plan());
        self.view_layout_node_count = layout.children.len();
        self.draw_plan = Some(paint_cx.into_plan());
        self.view_ui_command_tree = Some(view);
        self
    }

    pub fn window_spec(&self) -> &WindowSpec {
        &self.window
    }

    pub fn native_draw_plan(&self) -> Option<&NativeDrawPlan> {
        self.draw_plan.as_ref()
    }

    pub fn native_view_interaction_plan(&self) -> Option<&ViewInteractionPlan> {
        self.view_interaction_plan.as_ref()
    }

    pub const fn native_view_has_ui_command_routing(&self) -> bool {
        self.view_ui_command_tree.is_some()
    }

    pub const fn view_layout_node_count(&self) -> usize {
        self.view_layout_node_count
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        app(self.app_name).window(self.window).build()
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        let draw_plan = self.draw_plan.clone();
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        host.set_next_window_draw_plan(draw_plan);
        app.run_with_host(&mut host)
    }

    pub fn run_smoke(
        self,
        options: NativeWindowSmokeRunOptions,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        let draw_plan = self.draw_plan.clone();
        let view_runtime = self.native_view_input_runtime();
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        for window in &app.windows {
            host.create_main_window(window)?;
        }
        host.set_window_draw_plan(0, draw_plan);
        run_native_window_smoke_event_loop(
            host.windows.clone(),
            host.draw_plans.clone(),
            view_runtime,
            options,
        )
    }

    fn native_view_input_runtime(&self) -> NativeViewInputRuntime {
        NativeViewInputRuntime::new(
            self.view_interaction_plan.clone(),
            self.view_ui_command_tree.clone(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct NativeWindowHost {
    inner: MemoryHost,
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    next_window_draw_plan: Option<NativeDrawPlan>,
}

impl NativeWindowHost {
    pub fn new() -> Self {
        Self {
            inner: MemoryHost::with_capabilities(HostCapabilities::current_native_window_host()),
            windows: Vec::new(),
            trays: Vec::new(),
            draw_plans: Vec::new(),
            next_window_draw_plan: None,
        }
    }

    pub fn recorded_windows(&self) -> &[WindowRecord] {
        self.inner.windows()
    }

    pub fn recorded_trays(&self) -> &[TrayRecord] {
        self.inner.trays()
    }

    pub fn window_draw_plans(&self) -> &[Option<NativeDrawPlan>] {
        &self.draw_plans
    }

    pub fn set_next_window_draw_plan(&mut self, draw_plan: Option<NativeDrawPlan>) {
        self.next_window_draw_plan = draw_plan;
    }

    pub fn set_window_draw_plan(
        &mut self,
        window_index: usize,
        draw_plan: Option<NativeDrawPlan>,
    ) -> bool {
        let Some(slot) = self.draw_plans.get_mut(window_index) else {
            return false;
        };
        *slot = draw_plan;
        true
    }
}

impl Default for NativeWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

fn window_spec_from_startup_request(request: &NativeRuntimeStartupRequest) -> WindowSpec {
    let main = &request.main_window;
    let options = &main.options;
    let mut window = Window::new(main.title.clone())
        .size(
            i32_to_u32_window_size(main.size.width),
            i32_to_u32_window_size(main.size.height),
        )
        .visible(main.main_visible)
        .resizable(options.resizable)
        .decorations(options.decorations)
        .always_on_top(options.always_on_top)
        .transparent(options.transparent);

    if let Some(min_size) = &options.min_size {
        window = window.min_size(
            i32_to_u32_window_size(min_size.width),
            i32_to_u32_window_size(min_size.height),
        );
    }
    if let Some(icon_path) = &main.icon_path {
        window = window.icon_path(icon_path.clone());
    }

    window
}

fn i32_to_u32_window_size(value: i32) -> u32 {
    value.max(1) as u32
}

fn menu_entry_count(menu: &MenuSpec) -> usize {
    menu.items
        .iter()
        .map(|item| match item {
            MenuItemSpec::Command { .. } | MenuItemSpec::Separator => 1,
            MenuItemSpec::Submenu { menu, .. } => 1 + menu_entry_count(menu),
        })
        .sum()
}

impl ZsuiHost for NativeWindowHost {
    fn capabilities(&self) -> HostCapabilities {
        self.inner.capabilities()
    }

    fn create_main_window(&mut self, spec: &WindowSpec) -> ZsuiResult<WindowId> {
        let id = self.inner.create_main_window(spec)?;
        let capabilities = self.capabilities();
        self.windows.push(spec.resolve_for(&capabilities).effective);
        self.draw_plans.push(self.next_window_draw_plan.take());
        Ok(id)
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()> {
        self.inner.set_window_visible(window, visible)
    }

    fn create_tray(&mut self, spec: &TraySpec) -> ZsuiResult<TrayId> {
        let id = self.inner.create_tray(spec)?;
        self.trays.push(spec.clone());
        Ok(id)
    }

    fn register_global_hotkey(&mut self, spec: &HotkeySpec) -> ZsuiResult<HotkeyId> {
        self.inner.register_global_hotkey(spec)
    }

    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        self.inner.read_clipboard()
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        self.inner.write_clipboard(data)
    }

    fn open_file_picker(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<String>>> {
        self.inner.open_file_picker(spec)
    }

    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        self.inner.show_native_dialog(spec)
    }

    fn poll_event(&mut self) -> ZsuiResult<Option<AppEvent>> {
        self.inner.poll_event()
    }

    fn run_event_loop(&mut self) -> ZsuiResult<()> {
        run_native_window_event_loop(
            self.windows.clone(),
            self.trays.clone(),
            self.draw_plans.clone(),
        )
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
) -> ZsuiResult<()> {
    crate::windows_win32_host::run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items(
        &windows,
        &draw_plans,
        &trays,
    )
}

#[cfg(all(windows, not(feature = "windows-win32")))]
fn run_native_window_event_loop(
    _windows: Vec<WindowSpec>,
    _trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
) -> ZsuiResult<()> {
    Err(ZsuiError::unsupported(
        "native_window",
        "enable the windows-win32 feature to compile the direct Win32 native window host",
    ))
}

#[cfg(any(
    all(feature = "desktop-winit", target_os = "macos"),
    all(
        feature = "desktop-winit",
        target_os = "linux",
        not(target_env = "ohos")
    )
))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    _trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
) -> ZsuiResult<()> {
    use std::collections::HashMap;
    use winit::{
        application::ApplicationHandler,
        dpi::{LogicalSize, Size},
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
    };

    struct WinitNativeApp {
        specs: Vec<WindowSpec>,
        windows: HashMap<WinitWindowId, WinitWindow>,
        startup_error: Option<String>,
    }

    impl ApplicationHandler for WinitNativeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if !self.windows.is_empty() {
                return;
            }

            for spec in &self.specs {
                let mut attributes = WindowAttributes::default()
                    .with_title(spec.title.clone())
                    .with_inner_size(Size::Logical(LogicalSize::new(
                        spec.width as f64,
                        spec.height as f64,
                    )))
                    .with_visible(spec.visible)
                    .with_resizable(spec.resizable)
                    .with_decorations(spec.decorations)
                    .with_transparent(spec.transparent);

                if let (Some(width), Some(height)) = (spec.min_width, spec.min_height) {
                    attributes = attributes.with_min_inner_size(Size::Logical(LogicalSize::new(
                        width as f64,
                        height as f64,
                    )));
                }
                if spec.always_on_top {
                    attributes = attributes.with_window_level(WindowLevel::AlwaysOnTop);
                }

                match event_loop.create_window(attributes) {
                    Ok(window) => {
                        self.windows.insert(window.id(), window);
                    }
                    Err(err) => {
                        self.startup_error = Some(err.to_string());
                        event_loop.exit();
                        return;
                    }
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WinitWindowId,
            event: WindowEvent,
        ) {
            if matches!(event, WindowEvent::CloseRequested) {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
        }
    }

    if windows.is_empty() {
        return Ok(());
    }
    if !trays.is_empty() {
        return Err(ZsuiError::unsupported(
            "native_window_status_item",
            "native tray/status item runtime is wired only for the direct Windows Win32 host",
        ));
    }

    let event_loop = EventLoop::new()
        .map_err(|err| ZsuiError::host("native_window_event_loop", err.to_string()))?;
    let mut app = WinitNativeApp {
        specs: windows,
        windows: HashMap::new(),
        startup_error: None,
    };
    event_loop
        .run_app(&mut app)
        .map_err(|err| ZsuiError::host("native_window_event_loop", err.to_string()))?;

    if let Some(err) = app.startup_error {
        Err(ZsuiError::host("create_native_window", err))
    } else {
        Ok(())
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtime: NativeViewInputRuntime,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    use std::{thread, time::Duration};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        PostMessageW, WM_CHAR, WM_CLOSE, WM_LBUTTONUP,
    };

    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }

    let mut report = NativeWindowSmokeRunReport {
        requested_window_count: windows.len(),
        auto_close_after_ms: options.auto_close_after_ms,
        ..NativeWindowSmokeRunReport::empty(options.clone())
    };
    record_draw_plan_smoke(&mut report, &draw_plans);
    report.native_view_hit_target_count = view_runtime.hit_target_count();
    let input_routes = match view_runtime.windows_win32_route() {
        Some(route) => vec![Some(route)],
        None => Vec::new(),
    };
    let handles =
        crate::windows_win32_host::create_owned_windows_for_specs_with_draw_plans_and_input_routes(
            &windows,
            &draw_plans,
            &input_routes,
        )
        .map_err(|err| {
            report.startup_error = Some(err.to_string());
            report.events.push("startup_error".to_string());
            err
        })?;

    report.created_window_count = handles.len();
    report.events.extend(
        windows
            .iter()
            .map(|spec| format!("window_created:{}", spec.title)),
    );

    let mut _status_item_host = None;
    if let Some(status_item) = options.status_item.clone() {
        let mut host =
            crate::windows_win32_host::WindowsWin32StatusItemHost::new(handles[0].main());
        match host.create_status_item(NativeStatusItemRequest::from_tray_spec(&status_item)) {
            NativeStatusItemPresentation::Created(handle) => {
                report.status_item_created = true;
                report.events.push(format!("status_item_created:{handle}"));
                report.events.push(format!(
                    "status_item_menu_items:{}",
                    status_item.menu.items.len()
                ));
                report.status_menu_native_command_count = host.native_menu_command_count(0);
                if let Some(native_command_id) = host.first_native_menu_command_id(0) {
                    match host.dispatch_native_menu_command(0, native_command_id) {
                        NativeStatusMenuCommandResult::Dispatched(command) => {
                            report.status_menu_command_routed = true;
                            report
                                .events
                                .push(format!("status_menu_command_dispatched:{command:?}"));
                        }
                        NativeStatusMenuCommandResult::Disabled => {
                            report.status_menu_command_error =
                                Some("first status menu command is disabled".to_string());
                            report
                                .events
                                .push("status_menu_command_disabled".to_string());
                        }
                        NativeStatusMenuCommandResult::NotFound => {
                            report.status_menu_command_error =
                                Some("first status menu command was not found".to_string());
                            report
                                .events
                                .push("status_menu_command_not_found".to_string());
                        }
                    }
                } else if !status_item.menu.items.is_empty() {
                    report.status_menu_command_error =
                        Some("status item menu has no native command entries".to_string());
                    report
                        .events
                        .push("status_menu_command_missing".to_string());
                }
                match host.create_popup_menu_for_status_item(0) {
                    Ok(popup_menu) => {
                        report.status_menu_popup_created = true;
                        report.status_menu_popup_command_count = popup_menu.command_entry_count();
                        report.events.push(format!(
                            "status_menu_popup_created:{}",
                            report.status_menu_popup_command_count
                        ));
                        report.status_menu_popup_destroyed = popup_menu.destroy();
                        if report.status_menu_popup_destroyed {
                            report
                                .events
                                .push("status_menu_popup_destroyed".to_string());
                        } else {
                            report.status_menu_popup_error =
                                Some("DestroyMenu failed for status popup menu".to_string());
                            report
                                .events
                                .push("status_menu_popup_destroy_error".to_string());
                        }
                    }
                    Err(err) => {
                        report.status_menu_popup_error = Some(err.to_string());
                        report.events.push("status_menu_popup_error".to_string());
                    }
                }
            }
            NativeStatusItemPresentation::Failed => {
                let error = host
                    .last_error()
                    .unwrap_or("Win32 status item creation failed")
                    .to_string();
                report.status_item_error = Some(error);
                report.events.push("status_item_error".to_string());
            }
        }
        _status_item_host = Some(host);
    }

    if let Some(path) = report.screenshot_file.clone() {
        match capture_win32_hwnd_png(handles[0].main(), &path) {
            Ok(()) => {
                report.screenshot_captured = true;
                report.events.push(format!("screenshot_captured:{path}"));
            }
            Err(err) => {
                report.screenshot_error = Some(err);
                report.events.push("screenshot_error".to_string());
            }
        }
    }

    let mut click_points = options.native_view_click_points.iter();
    if !options.native_view_text_inputs.is_empty() {
        if let Some(point) = click_points.next() {
            unsafe {
                PostMessageW(
                    handles[0].main(),
                    WM_LBUTTONUP,
                    0,
                    windows_lparam_from_point(*point),
                );
            }
        }
    }
    for text in &options.native_view_text_inputs {
        for ch in text.chars() {
            unsafe {
                PostMessageW(handles[0].main(), WM_CHAR, ch as usize, 0);
            }
        }
    }
    for point in click_points {
        unsafe {
            PostMessageW(
                handles[0].main(),
                WM_LBUTTONUP,
                0,
                windows_lparam_from_point(*point),
            );
        }
    }

    let close_handles: Vec<isize> = handles
        .iter()
        .map(|handles| handles.main() as isize)
        .collect();
    let auto_close_after = Duration::from_millis(options.auto_close_after_ms.max(1));
    thread::spawn(move || {
        thread::sleep(auto_close_after);
        for handle in close_handles {
            unsafe {
                PostMessageW(handle as _, WM_CLOSE, 0, 0);
            }
        }
    });

    match crate::windows_win32_host::WindowsWin32MessageLoop::run() {
        crate::windows_win32_host::WindowsWin32MessageLoopResult::Quit(_) => {
            report.exited_by_auto_close = true;
            report.close_requested_count = report.created_window_count;
            report.events.push("auto_close_elapsed".to_string());
        }
        crate::windows_win32_host::WindowsWin32MessageLoopResult::Failed => {
            report.startup_error = Some("GetMessageW failed".to_string());
            report.events.push("message_loop_error".to_string());
        }
    }

    for handles in &handles {
        if let Some(input_report) =
            crate::windows_win32_host::windows_win32_window_view_input_report(handles.main())
        {
            record_windows_win32_view_input_report(&mut report, &input_report);
        }
    }

    if options.require_visible_window && !report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot && !report.screenshot_captured {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_status_item && !report.status_item_created {
        return Err(ZsuiError::host(
            "native_window_smoke_status_item",
            report
                .status_item_error
                .clone()
                .unwrap_or_else(|| "status item was not created".to_string()),
        ));
    }

    Ok(report)
}

#[cfg(any(
    all(feature = "desktop-winit", target_os = "macos"),
    all(
        feature = "desktop-winit",
        target_os = "linux",
        not(target_env = "ohos")
    )
))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    mut view_runtime: NativeViewInputRuntime,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };
    use winit::{
        application::ApplicationHandler,
        dpi::{LogicalSize, Size},
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
    };

    struct WinitNativeSmokeApp {
        specs: Vec<WindowSpec>,
        windows: HashMap<WinitWindowId, WinitWindow>,
        started_at: Instant,
        auto_close_after: Duration,
        report: NativeWindowSmokeRunReport,
        screenshot_attempted: bool,
        auto_close_reported: bool,
    }

    impl ApplicationHandler for WinitNativeSmokeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if !self.windows.is_empty() {
                return;
            }

            for spec in &self.specs {
                let mut attributes = WindowAttributes::default()
                    .with_title(spec.title.clone())
                    .with_inner_size(Size::Logical(LogicalSize::new(
                        spec.width as f64,
                        spec.height as f64,
                    )))
                    .with_visible(spec.visible)
                    .with_resizable(spec.resizable)
                    .with_decorations(spec.decorations)
                    .with_transparent(spec.transparent);

                if let (Some(width), Some(height)) = (spec.min_width, spec.min_height) {
                    attributes = attributes.with_min_inner_size(Size::Logical(LogicalSize::new(
                        width as f64,
                        height as f64,
                    )));
                }
                if spec.always_on_top {
                    attributes = attributes.with_window_level(WindowLevel::AlwaysOnTop);
                }

                match event_loop.create_window(attributes) {
                    Ok(window) => {
                        self.report.created_window_count += 1;
                        self.report
                            .events
                            .push(format!("window_created:{}", spec.title));
                        self.windows.insert(window.id(), window);
                    }
                    Err(err) => {
                        self.report.startup_error = Some(err.to_string());
                        self.report.events.push("startup_error".to_string());
                        event_loop.exit();
                        return;
                    }
                }
            }

            event_loop.set_control_flow(ControlFlow::WaitUntil(
                self.started_at + self.auto_close_after,
            ));
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WinitWindowId,
            event: WindowEvent,
        ) {
            if matches!(event, WindowEvent::CloseRequested) {
                self.report.close_requested_count += 1;
                self.report.events.push("close_requested".to_string());
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            if self.report.created_window_count == 0 {
                return;
            }

            if !self.screenshot_attempted {
                self.screenshot_attempted = true;
                if let Some(path) = self.report.screenshot_file.clone() {
                    match capture_first_native_window_png(&self.windows, &path) {
                        Ok(()) => {
                            self.report.screenshot_captured = true;
                            self.report
                                .events
                                .push(format!("screenshot_captured:{path}"));
                        }
                        Err(err) => {
                            self.report.screenshot_error = Some(err.clone());
                            self.report.events.push("screenshot_error".to_string());
                        }
                    }
                }
            }

            let target = self.started_at + self.auto_close_after;
            if Instant::now() >= target {
                if !self.auto_close_reported {
                    self.auto_close_reported = true;
                    self.report.exited_by_auto_close = true;
                    self.report.events.push("auto_close_elapsed".to_string());
                }
                event_loop.exit();
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(target));
            }
        }
    }

    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }

    let event_loop = EventLoop::new()
        .map_err(|err| ZsuiError::host("native_window_smoke_event_loop", err.to_string()))?;
    let mut app = WinitNativeSmokeApp {
        report: NativeWindowSmokeRunReport {
            requested_window_count: windows.len(),
            auto_close_after_ms: options.auto_close_after_ms,
            ..NativeWindowSmokeRunReport::empty(options.clone())
        },
        specs: windows,
        windows: HashMap::new(),
        started_at: Instant::now(),
        auto_close_after: Duration::from_millis(options.auto_close_after_ms.max(1)),
        screenshot_attempted: false,
        auto_close_reported: false,
    };
    record_draw_plan_smoke(&mut app.report, &draw_plans);
    record_native_view_input_smoke(&mut app.report, &mut view_runtime, &options);
    event_loop
        .run_app(&mut app)
        .map_err(|err| ZsuiError::host("native_window_smoke_event_loop", err.to_string()))?;

    if options.status_item.is_some() {
        app.report.status_item_error = Some(
            "status item smoke is currently implemented only for the direct Windows Win32 host"
                .to_string(),
        );
        app.report
            .events
            .push("status_item_unsupported".to_string());
    }

    if let Some(err) = &app.report.startup_error {
        return Err(ZsuiError::host("create_native_window", err.clone()));
    }
    if options.require_visible_window && !app.report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot && !app.report.screenshot_captured {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            app.report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_status_item && !app.report.status_item_created {
        return Err(ZsuiError::unsupported(
            "native_window_smoke_status_item",
            app.report
                .status_item_error
                .clone()
                .unwrap_or_else(|| "status item was not created".to_string()),
        ));
    }

    Ok(app.report)
}

#[cfg(all(
    not(windows),
    feature = "desktop-winit",
    any(
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )
))]
fn capture_first_native_window_png(
    _windows: &std::collections::HashMap<winit::window::WindowId, winit::window::Window>,
    _path: &str,
) -> Result<(), String> {
    Err("native smoke screenshot capture is currently implemented for Windows only".to_string())
}

#[cfg(all(windows, not(feature = "windows-win32")))]
fn run_native_window_smoke_event_loop(
    _windows: Vec<WindowSpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtime: NativeViewInputRuntime,
    _options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    Err(ZsuiError::unsupported(
        "native_window_smoke",
        "enable the windows-win32 feature to compile the direct Win32 native smoke host",
    ))
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32WindowDeviceContext {
    hwnd: windows_sys::Win32::Foundation::HWND,
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32WindowDeviceContext {
    fn acquire(hwnd: windows_sys::Win32::Foundation::HWND) -> Result<Self, String> {
        let dc = unsafe { windows_sys::Win32::Graphics::Gdi::GetDC(hwnd) };
        if dc.is_null() {
            Err("GetDC failed".to_string())
        } else {
            Ok(Self { hwnd, dc })
        }
    }

    const fn hdc(&self) -> windows_sys::Win32::Graphics::Gdi::HDC {
        self.dc
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32WindowDeviceContext {
    fn drop(&mut self) {
        if !self.dc.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::ReleaseDC(self.hwnd, self.dc);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32CompatibleDeviceContext {
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32CompatibleDeviceContext {
    fn create(source: windows_sys::Win32::Graphics::Gdi::HDC) -> Result<Self, String> {
        let dc = unsafe { windows_sys::Win32::Graphics::Gdi::CreateCompatibleDC(source) };
        if dc.is_null() {
            Err("CreateCompatibleDC failed".to_string())
        } else {
            Ok(Self { dc })
        }
    }

    const fn hdc(&self) -> windows_sys::Win32::Graphics::Gdi::HDC {
        self.dc
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32CompatibleDeviceContext {
    fn drop(&mut self) {
        if !self.dc.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::DeleteDC(self.dc);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32CompatibleBitmap {
    bitmap: windows_sys::Win32::Graphics::Gdi::HBITMAP,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32CompatibleBitmap {
    fn create(
        dc: windows_sys::Win32::Graphics::Gdi::HDC,
        width: i32,
        height: i32,
    ) -> Result<Self, String> {
        let bitmap =
            unsafe { windows_sys::Win32::Graphics::Gdi::CreateCompatibleBitmap(dc, width, height) };
        if bitmap.is_null() {
            Err("CreateCompatibleBitmap failed".to_string())
        } else {
            Ok(Self { bitmap })
        }
    }

    const fn handle(&self) -> windows_sys::Win32::Graphics::Gdi::HBITMAP {
        self.bitmap
    }

    fn object(&self) -> windows_sys::Win32::Graphics::Gdi::HGDIOBJ {
        self.bitmap.cast()
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32CompatibleBitmap {
    fn drop(&mut self) {
        if !self.bitmap.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::DeleteObject(self.bitmap.cast());
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32SelectedGdiObject {
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
    old: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32SelectedGdiObject {
    fn select(
        dc: windows_sys::Win32::Graphics::Gdi::HDC,
        object: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
    ) -> Option<Self> {
        if dc.is_null() || object.is_null() {
            return None;
        }
        let old = unsafe { windows_sys::Win32::Graphics::Gdi::SelectObject(dc, object) };
        if old.is_null() {
            None
        } else {
            Some(Self { dc, old })
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32SelectedGdiObject {
    fn drop(&mut self) {
        if !self.dc.is_null() && !self.old.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::SelectObject(self.dc, self.old);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn capture_win32_hwnd_png(
    hwnd: windows_sys::Win32::Foundation::HWND,
    path: &str,
) -> Result<(), String> {
    use std::{ffi::c_void, mem, path::Path};
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{
            BitBlt, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD,
            SRCCOPY,
        },
        UI::WindowsAndMessaging::GetClientRect,
    };

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let has_rect = unsafe { GetClientRect(hwnd, &mut rect) };
    if has_rect == 0 {
        return Err("GetClientRect failed".to_string());
    }

    let width = (rect.right - rect.left).max(1);
    let height = (rect.bottom - rect.top).max(1);
    let window_dc = Win32WindowDeviceContext::acquire(hwnd)?;
    let memory_dc = Win32CompatibleDeviceContext::create(window_dc.hdc())?;
    let bitmap = Win32CompatibleBitmap::create(window_dc.hdc(), width, height)?;
    let _selected_bitmap = Win32SelectedGdiObject::select(memory_dc.hdc(), bitmap.object());

    let blit_ok = unsafe {
        BitBlt(
            memory_dc.hdc(),
            0,
            0,
            width,
            height,
            window_dc.hdc(),
            0,
            0,
            SRCCOPY,
        )
    };
    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }; 1],
    };
    let mut bgra = vec![0u8; width as usize * height as usize * 4];
    let dib_lines = if blit_ok != 0 {
        unsafe {
            GetDIBits(
                memory_dc.hdc(),
                bitmap.handle(),
                0,
                height as u32,
                bgra.as_mut_ptr().cast::<c_void>(),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        }
    } else {
        0
    };

    if blit_ok == 0 {
        return Err("BitBlt failed".to_string());
    }
    if dib_lines == 0 {
        return Err("GetDIBits failed".to_string());
    }

    let rgba = bgra_to_rgba(&bgra);
    write_rgba_png(Path::new(path), width as u32, height as u32, &rgba)
}

#[cfg(all(windows, feature = "windows-win32"))]
fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        rgba.push(pixel[2]);
        rgba.push(pixel[1]);
        rgba.push(pixel[0]);
        rgba.push(255);
    }
    rgba
}

#[cfg(all(windows, feature = "windows-win32"))]
fn write_rgba_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|err| err.to_string())?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().map_err(|err| err.to_string())?;
    png_writer
        .write_image_data(rgba)
        .map_err(|err| err.to_string())
}

#[cfg(any(
    not(any(
        target_os = "windows",
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )),
    all(target_os = "macos", not(feature = "desktop-winit")),
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        not(feature = "desktop-winit")
    )
))]
fn run_native_window_event_loop(
    _windows: Vec<WindowSpec>,
    _trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
) -> ZsuiResult<()> {
    Err(ZsuiError::unsupported(
        "native_window",
        "desktop native windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts",
    ))
}

#[cfg(any(
    not(any(
        target_os = "windows",
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )),
    all(target_os = "macos", not(feature = "desktop-winit")),
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        not(feature = "desktop-winit")
    )
))]
fn run_native_window_smoke_event_loop(
    _windows: Vec<WindowSpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtime: NativeViewInputRuntime,
    _options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    Err(ZsuiError::unsupported(
        "native_window_smoke",
        "desktop native smoke windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_window_builder_declares_single_window_app() {
        let app = native_window("Example")
            .size(800, 520)
            .min_size(480, 320)
            .always_on_top(true)
            .build()
            .expect("native builder should create an app declaration");

        assert_eq!(app.name, "Example");
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.windows[0].title, "Example");
        assert_eq!(app.windows[0].width, 800);
        assert_eq!(app.windows[0].min_width, Some(480));
        assert!(app.windows[0].always_on_top);
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_window_builder_projects_typed_view_into_draw_plan() {
        #[derive(Clone)]
        enum Msg {
            Save,
        }

        let builder = native_window("View Example")
            .size(360, 220)
            .view(crate::column(vec![
                crate::text::<Msg>("Settings"),
                crate::button("Save")
                    .id(crate::WidgetId::new(1))
                    .on_click(Msg::Save),
            ]));
        let draw_plan = builder
            .native_draw_plan()
            .expect("typed view should become a native draw plan");
        let interaction_plan = builder
            .native_view_interaction_plan()
            .expect("typed view should expose hit targets");

        assert_eq!(builder.view_layout_node_count(), 1);
        assert_eq!(interaction_plan.hit_target_count(), 1);
        assert!(draw_plan.command_count() >= 3);
        assert!(draw_plan.text_count() >= 2);
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_window_builder_routes_ui_command_view_clicks_for_smoke() {
        let builder = native_window("View Command Example")
            .size(360, 220)
            .ui_command_view(crate::column(vec![
                crate::text::<UiCommand>("Settings"),
                crate::button("Save")
                    .id(crate::WidgetId::new(7))
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.save"))),
            ]));
        let mut report = NativeWindowSmokeRunReport::empty(
            NativeWindowSmokeRunOptions::quick().native_view_click(Point { x: 180, y: 170 }),
        );

        record_native_view_input_smoke(
            &mut report,
            &mut builder.native_view_input_runtime(),
            &NativeWindowSmokeRunOptions::quick().native_view_click(Point { x: 180, y: 170 }),
        );

        assert!(builder.native_view_has_ui_command_routing());
        assert_eq!(report.native_view_hit_target_count, 1);
        assert_eq!(report.native_view_click_count, 1);
        assert_eq!(report.native_view_event_count, 1);
        assert_eq!(report.native_view_message_count, 1);
        assert_eq!(report.native_view_ui_command_count, 1);
        assert_eq!(report.native_view_ui_command_ids, vec!["zsui.test.save"]);
        assert_eq!(report.native_view_unhandled_click_count, 0);
    }

    #[test]
    fn native_window_smoke_options_have_short_default_runtime() {
        let options = NativeWindowSmokeRunOptions::quick();
        let report = NativeWindowSmokeRunReport::empty(options.clone());

        assert_eq!(options.auto_close_after_ms, 750);
        assert!(options.require_visible_window);
        assert_eq!(options.screenshot_file, None);
        assert!(!options.require_screenshot);
        assert_eq!(options.status_item, None);
        assert!(!options.require_status_item);
        assert_eq!(report.created_window_count, 0);
        assert!(!report.status_item_requested);
        assert_eq!(report.status_menu_native_command_count, 0);
        assert!(!report.status_menu_command_routed);
        assert!(!report.status_menu_popup_created);
        assert!(!report.status_menu_popup_destroyed);
        assert!(!report.draw_plan_requested);
        assert_eq!(report.draw_plan_window_count, 0);
        assert_eq!(report.draw_command_count, 0);
        assert_eq!(report.text_command_count, 0);
        assert!(options.native_view_click_points.is_empty());
        assert_eq!(report.native_view_hit_target_count, 0);
        assert_eq!(report.native_view_click_count, 0);
        assert_eq!(report.native_view_event_count, 0);
        assert_eq!(report.native_view_message_count, 0);
        assert_eq!(report.native_view_ui_command_count, 0);
        assert!(report.native_view_ui_command_ids.is_empty());
        assert_eq!(report.native_view_unhandled_click_count, 0);
        assert!(options.native_view_text_inputs.is_empty());
        assert_eq!(report.native_view_focus_count, 0);
        assert_eq!(report.native_view_text_input_count, 0);
        assert_eq!(report.native_view_toggle_count, 0);
        assert!(!report.visible_window_was_created());
    }

    #[test]
    fn native_window_smoke_options_can_request_status_item() {
        let status_item = crate::TraySpec::new()
            .tooltip("ZSUI Smoke")
            .item("Open", crate::Command::ShowMainWindow)
            .item("Quit", crate::Command::Quit);
        let options = NativeWindowSmokeRunOptions::quick()
            .status_item(status_item)
            .require_status_item(true);
        let report = NativeWindowSmokeRunReport::empty(options.clone());

        assert!(options.status_item.is_some());
        assert!(options.require_status_item);
        assert!(report.status_item_requested);
        assert!(report.status_item_required);
        assert_eq!(report.status_item_menu_item_count, 2);
        assert_eq!(report.status_menu_native_command_count, 0);
        assert_eq!(report.status_menu_popup_command_count, 0);
        assert!(!report.status_item_created);
    }

    #[cfg(all(windows, feature = "windows-win32"))]
    #[test]
    fn win32_capture_resources_are_drop_backed() {
        assert!(std::mem::needs_drop::<Win32WindowDeviceContext>());
        assert!(std::mem::needs_drop::<Win32CompatibleDeviceContext>());
        assert!(std::mem::needs_drop::<Win32CompatibleBitmap>());
        assert!(std::mem::needs_drop::<Win32SelectedGdiObject>());
        assert!(
            Win32SelectedGdiObject::select(std::ptr::null_mut(), std::ptr::null_mut()).is_none()
        );
    }

    #[test]
    fn native_window_runtime_driver_maps_startup_request_to_window_spec() {
        let mut driver = NativeWindowRuntimeDriver::with_capabilities(
            HostCapabilities::windows_native_window_host(),
        );
        let startup = NativeRuntimeStartupRequest {
            app_name: "Example".to_string(),
            main_window: crate::NativeMainWindowRequest::from_zsui_window(
                &Window::new("Example")
                    .size(640, 420)
                    .min_size(320, 240)
                    .icon_path("assets/app.ico")
                    .decorations(false),
            ),
            status_item_tooltip: Some("Example".to_string()),
            status_item: Some(
                TraySpec::new()
                    .tooltip("Example")
                    .item("Quit", crate::Command::Quit),
            ),
            settings_pages: vec![SettingsPageSpec::new("general", "General")],
        };

        let handles = driver.start_runtime(startup);

        assert!(matches!(handles, NativeRuntimeStartupResult::Started(_)));
        assert_eq!(driver.startup_requests().len(), 1);
        assert_eq!(driver.window_specs()[0].title, "Example");
        assert_eq!(driver.window_specs()[0].width, 640);
        assert_eq!(driver.window_specs()[0].min_width, Some(320));
        assert_eq!(
            driver.window_specs()[0].icon_path.as_deref(),
            Some("assets/app.ico")
        );
        assert!(!driver.window_specs()[0].decorations);
        assert_eq!(driver.status_item_specs().len(), 1);
        assert_eq!(driver.status_item_specs()[0].menu.items.len(), 1);
        assert_eq!(driver.settings_page_specs().len(), 1);
        assert_eq!(driver.report().status_item_count, 1);
        assert_eq!(driver.report().status_menu_entry_count, 1);
        assert_eq!(driver.report().settings_page_count, 1);
        assert_eq!(driver.report().status_item_handle_count, 1);
        assert!(driver.report().settings_model_bound);
        assert_eq!(
            driver.native_operation_names(),
            &["create_status_item", "bind_settings_pages"]
        );
        assert_eq!(
            driver.poll_application_event(),
            Some(AppEvent::WindowCreated {
                window: WindowId(1)
            })
        );
        assert_eq!(driver.poll_application_event(), Some(AppEvent::Started));
    }

    #[test]
    fn native_window_runtime_driver_records_commands_and_shutdown() {
        let mut driver = NativeWindowRuntimeDriver::new();
        driver.dispatch_ui_command(UiCommand::app(crate::CommandId("example.refresh")));

        assert_eq!(driver.command_ids(), &["example.refresh"]);
        assert_eq!(
            driver.poll_application_event(),
            Some(AppEvent::Custom {
                id: "example.refresh".to_string(),
                payload: None
            })
        );

        driver.request_shutdown();
        assert!(driver.shutdown_requested());
        assert!(driver
            .native_operation_names()
            .contains(&"clear_settings_pages"));
        assert_eq!(
            driver.poll_application_event(),
            Some(AppEvent::QuitRequested)
        );
    }

    #[test]
    fn native_window_runtime_driver_dispatches_status_menu_commands() {
        let mut driver = NativeWindowRuntimeDriver::new();
        let presentation = driver.create_status_item(NativeStatusItemRequest {
            tooltip: Some("Example".to_string()),
            icon_path: None,
            menu: MenuSpec::new()
                .item("Open", crate::Command::ShowMainWindow)
                .separator()
                .item("Quit", crate::Command::Quit),
        });

        assert!(matches!(
            presentation,
            NativeStatusItemPresentation::Created(_)
        ));
        assert_eq!(
            driver
                .dispatch_status_menu_command(NativeStatusMenuCommandRequest::by_label(0, "Open")),
            NativeStatusMenuCommandResult::Dispatched(crate::Command::ShowMainWindow)
        );
        assert_eq!(
            driver.poll_application_event(),
            Some(AppEvent::TrayCommand {
                command: crate::Command::ShowMainWindow
            })
        );
        assert_eq!(
            driver.dispatch_status_menu_command(NativeStatusMenuCommandRequest::by_label(
                0, "Missing"
            )),
            NativeStatusMenuCommandResult::NotFound
        );
        assert!(driver
            .native_operation_names()
            .contains(&"dispatch_status_menu_command"));
    }

    #[test]
    fn native_window_runtime_driver_updates_bound_settings_items() {
        let mut driver = NativeWindowRuntimeDriver::new();
        assert_eq!(
            driver.update_settings_item_value(NativeSettingsItemUpdateRequest::new(
                "general",
                "capture",
                crate::SettingsValue::Bool(false),
            )),
            NativeSettingsItemUpdateResult::NotBound
        );

        driver.bind_settings_pages(NativeSettingsPageModelRequest::new(vec![
            SettingsPageSpec::new("general", "General")
                .item(crate::SettingsItemSpec::toggle("capture", "Capture", true)),
        ]));
        assert_eq!(
            driver.update_settings_item_value(NativeSettingsItemUpdateRequest::new(
                "general",
                "capture",
                crate::SettingsValue::Bool(false),
            )),
            NativeSettingsItemUpdateResult::Updated
        );
        assert_eq!(
            driver.settings_page_specs()[0].items[0].default_value,
            Some(crate::SettingsValue::Bool(false))
        );
        assert_eq!(
            driver.poll_application_event(),
            Some(AppEvent::SettingsChanged {
                page: "general".to_string(),
                item: "capture".to_string()
            })
        );
        assert!(driver
            .native_operation_names()
            .contains(&"update_settings_item_value"));
    }
}
