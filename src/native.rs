use serde::Serialize;

#[cfg(feature = "combo")]
use std::time::{Duration, Instant};

#[cfg(feature = "workbench")]
use crate::workbench::ZsWorkbenchSpec;

use crate::native_input_visuals::{
    decorate_native_focus_ring, decorate_native_text_edit_visuals, native_text_index_for_point,
    native_text_visual_geometry,
};
#[cfg(any(feature = "date-picker", feature = "tabs"))]
use crate::native_input_visuals::{
    decorate_native_pointer_visuals, native_pointer_visual_key, NativePointerVisualKey,
};
use crate::native_text_edit::{
    char_to_byte_index, delete_backward, delete_forward, insert_text, move_selection,
    set_pointer_selection, NativeTextDragState, NativeTextEditState, NativeTextMovement,
    NativeTextSelection,
};
use crate::{
    app::{app, ZsuiApp, ZsuiAppRuntime},
    app_command::{app_command_name, AppCommandExecutor, SharedAppCommandExecutor},
    capability::HostCapabilities,
    clipboard::ClipboardData,
    command_protocol::{SharedUiCommandExecutor, UiCommand, UiCommandExecutor},
    core::{
        AppEvent, Command, DialogResponse, FileDialogSpec, HotkeyId, NativeDialogSpec, TrayId,
        WindowId, ZsuiError, ZsuiResult,
    },
    geometry::{Dp, Dpi, Point, Rect},
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
    render_protocol::{NativeDrawCommand, NativeDrawFill, NativeDrawPlan},
    settings::SettingsPageSpec,
    shell_layout::{ZsShellLayoutSpec, ZsShellRuntime},
    tray::TraySpec,
    view::{
        live_view_runtime, AppCx, SharedLiveViewRuntime, View, ViewEvent, ViewEventCx,
        ViewInteractionPlan, ViewLayoutCx, ViewNode, ViewPaintCx,
    },
    window::{Window, WindowSpec},
};

pub fn native_window(title: impl Into<String>) -> NativeWindowBuilder {
    NativeWindowBuilder::new(title)
}

/// Starts the opt-in typestate builder. Content must be attached before
/// `build`, `run` or `run_smoke` becomes available.
///
/// ```compile_fail
/// zsui::typed_native_window("Missing content").run().unwrap();
/// ```
pub fn typed_native_window(
    title: impl Into<String>,
) -> TypedNativeWindowBuilder<NativeWindowContentMissing> {
    TypedNativeWindowBuilder::new(title)
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
    pub app_command_count: usize,
    pub app_command_names: Vec<&'static str>,
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
    app_commands: Vec<Command>,
    events: Vec<AppEvent>,
    handles: Option<NativeMainWindowHandles<NativeWindowRuntimeHandle>>,
    next_handle: u64,
    shutdown_requested: bool,
    main_window_visible: bool,
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
            app_commands: Vec::new(),
            events: Vec::new(),
            handles: None,
            next_handle: 1,
            shutdown_requested: false,
            main_window_visible: false,
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

    pub fn app_commands(&self) -> &[Command] {
        &self.app_commands
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
            app_command_count: self.app_commands.len(),
            app_command_names: self.app_commands.iter().map(app_command_name).collect(),
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
            None,
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
        let main_window_visible = window.visible;
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
        self.main_window_visible = main_window_visible;
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

impl AppCommandExecutor for NativeWindowRuntimeDriver {
    fn execute_app_command(&mut self, command: Command) -> ZsuiResult<Vec<AppEvent>> {
        self.app_commands.push(command.clone());
        let events = match command {
            Command::ShowMainWindow => {
                self.main_window_visible = true;
                vec![AppEvent::WindowShown {
                    window: self.main_window_id()?,
                }]
            }
            Command::HideMainWindow => {
                self.main_window_visible = false;
                vec![AppEvent::WindowHidden {
                    window: self.main_window_id()?,
                }]
            }
            Command::ToggleMainWindow => {
                self.main_window_visible = !self.main_window_visible;
                let window = self.main_window_id()?;
                if self.main_window_visible {
                    vec![AppEvent::WindowShown { window }]
                } else {
                    vec![AppEvent::WindowHidden { window }]
                }
            }
            Command::OpenQuickPanel => vec![AppEvent::WindowShown {
                window: self.quick_window_id()?,
            }],
            Command::Quit => {
                self.request_shutdown();
                return Ok(vec![AppEvent::QuitRequested]);
            }
            Command::Custom { id, payload } => vec![AppEvent::Custom { id, payload }],
            command => vec![AppEvent::Custom {
                id: format!("zsui.command.{}", app_command_name(&command)),
                payload: None,
            }],
        };
        self.events.extend(events.iter().cloned());
        Ok(events)
    }
}

impl UiCommandExecutor for NativeWindowRuntimeDriver {
    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        let event = AppEvent::Custom {
            id: command.id.0.to_string(),
            payload: None,
        };
        <Self as NativeRuntimeDriver<UiCommand, AppEvent>>::dispatch_ui_command(self, command);
        Ok(vec![event])
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

    fn main_window_id(&self) -> ZsuiResult<WindowId> {
        self.handles
            .map(|handles| WindowId(handles.main.0))
            .ok_or_else(|| ZsuiError::host("execute_app_command", "native runtime is not started"))
    }

    fn quick_window_id(&self) -> ZsuiResult<WindowId> {
        self.handles
            .map(|handles| WindowId(handles.quick.0))
            .ok_or_else(|| ZsuiError::host("execute_app_command", "native runtime is not started"))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NativeViewKey {
    Enter,
    Escape,
    Tab,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

#[cfg(feature = "tabs")]
fn native_tab_cycle_offset(
    platform: crate::ZsTabPlatformStyle,
    key: NativeViewKey,
    shift: bool,
    control: bool,
) -> Option<isize> {
    match (platform, key, control) {
        (crate::ZsTabPlatformStyle::Windows, NativeViewKey::Tab, true) => {
            Some(if shift { -1 } else { 1 })
        }
        (crate::ZsTabPlatformStyle::Gtk, NativeViewKey::PageUp, true) => Some(-1),
        (crate::ZsTabPlatformStyle::Gtk, NativeViewKey::PageDown, true) => Some(1),
        _ => None,
    }
}

#[cfg(feature = "combo")]
const NATIVE_COMBO_TYPE_AHEAD_TIMEOUT: Duration = Duration::from_millis(1_000);

#[cfg(feature = "combo")]
#[derive(Debug, Clone, Default)]
pub(crate) struct NativeComboTypeAheadState {
    widget: Option<crate::WidgetId>,
    query: String,
    last_input: Option<Instant>,
}

#[cfg(feature = "combo")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeComboTypeAheadQuery {
    pub(crate) text: String,
    cycle_from_selection: bool,
}

#[cfg(feature = "combo")]
impl NativeComboTypeAheadQuery {
    pub(crate) fn match_start_after(
        &self,
        selected: Option<usize>,
        option_count: usize,
    ) -> Option<usize> {
        if self.cycle_from_selection {
            return selected;
        }
        selected.filter(|index| *index < option_count).map(|index| {
            if index == 0 {
                option_count.saturating_sub(1)
            } else {
                index - 1
            }
        })
    }
}

#[cfg(feature = "combo")]
impl NativeComboTypeAheadState {
    pub(crate) fn push_text(
        &mut self,
        widget: crate::WidgetId,
        text: &str,
        now: Instant,
    ) -> Option<NativeComboTypeAheadQuery> {
        let normalized = text
            .chars()
            .filter(|character| !character.is_control() && !character.is_whitespace())
            .collect::<String>()
            .to_lowercase();
        if normalized.is_empty() {
            return None;
        }

        let continues = self.widget == Some(widget)
            && self.last_input.is_some_and(|last_input| {
                now.checked_duration_since(last_input)
                    .is_some_and(|elapsed| elapsed <= NATIVE_COMBO_TYPE_AHEAD_TIMEOUT)
            });
        let repeated_single_character = continues
            && normalized.chars().count() == 1
            && !self.query.is_empty()
            && self
                .query
                .chars()
                .all(|character| normalized.starts_with(character));
        let cycle_from_selection = !continues || repeated_single_character;
        if !cycle_from_selection {
            self.query.push_str(&normalized);
        } else {
            self.query = normalized;
        }
        self.widget = Some(widget);
        self.last_input = Some(now);
        Some(NativeComboTypeAheadQuery {
            text: self.query.clone(),
            cycle_from_selection,
        })
    }

    pub(crate) fn reset(&mut self) {
        self.widget = None;
        self.query.clear();
        self.last_input = None;
    }
}

impl NativeViewKey {
    pub const fn key_name(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Escape => "escape",
            Self::Tab => "tab",
            Self::Space => "space",
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::Home => "home",
            Self::End => "end",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum NativeViewSmokeInput {
    Click(Point),
    Drag { start: Point, end: Point },
    Text(String),
    KeyDown(NativeViewKey),
    Scroll { point: Point, delta_y: i32 },
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
    pub native_view_key_downs: Vec<NativeViewKey>,
    pub native_view_scroll_inputs: Vec<(Point, i32)>,
    pub native_view_inputs: Vec<NativeViewSmokeInput>,
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
            native_view_key_downs: Vec::new(),
            native_view_scroll_inputs: Vec::new(),
            native_view_inputs: Vec::new(),
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
        self.native_view_inputs
            .push(NativeViewSmokeInput::Click(point));
        self
    }

    pub fn native_view_clicks(mut self, points: impl IntoIterator<Item = Point>) -> Self {
        for point in points {
            self = self.native_view_click(point);
        }
        self
    }

    pub fn native_view_drag(mut self, start: Point, end: Point) -> Self {
        self.native_view_inputs
            .push(NativeViewSmokeInput::Drag { start, end });
        self
    }

    pub fn native_view_text_input(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.native_view_text_inputs.push(text.clone());
        self.native_view_inputs
            .push(NativeViewSmokeInput::Text(text));
        self
    }

    pub fn native_view_text_inputs<I, S>(mut self, texts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for text in texts {
            self = self.native_view_text_input(text);
        }
        self
    }

    pub fn native_view_key_down(mut self, key: NativeViewKey) -> Self {
        self.native_view_key_downs.push(key);
        self.native_view_inputs
            .push(NativeViewSmokeInput::KeyDown(key));
        self
    }

    pub fn native_view_key_downs(mut self, keys: impl IntoIterator<Item = NativeViewKey>) -> Self {
        for key in keys {
            self = self.native_view_key_down(key);
        }
        self
    }

    pub fn native_view_scroll(mut self, point: Point, delta_y: i32) -> Self {
        self.native_view_scroll_inputs.push((point, delta_y));
        self.native_view_inputs
            .push(NativeViewSmokeInput::Scroll { point, delta_y });
        self
    }

    pub fn native_view_scrolls(mut self, inputs: impl IntoIterator<Item = (Point, i32)>) -> Self {
        for (point, delta_y) in inputs {
            self = self.native_view_scroll(point, delta_y);
        }
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
    pub window_menu_requested_count: usize,
    pub window_menu_attached_count: usize,
    pub window_menu_native_command_count: usize,
    pub window_menu_command_routed: bool,
    pub window_menu_command_error: Option<String>,
    pub close_requested_count: usize,
    pub auto_close_after_ms: u64,
    pub exited_by_auto_close: bool,
    pub startup_error: Option<String>,
    pub screenshot_file: Option<String>,
    pub screenshot_captured: bool,
    pub screenshot_error: Option<String>,
    pub draw_plan_requested: bool,
    pub draw_plan_window_count: usize,
    pub high_contrast_draw_plan_window_count: usize,
    pub draw_command_count: usize,
    pub text_command_count: usize,
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
    pub native_view_radio_selection_count: usize,
    pub native_view_radio_keyboard_selection_count: usize,
    pub native_view_radio_keyboard_focus_only_count: usize,
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
            window_menu_requested_count: 0,
            window_menu_attached_count: 0,
            window_menu_native_command_count: 0,
            window_menu_command_routed: false,
            window_menu_command_error: None,
            close_requested_count: 0,
            auto_close_after_ms: options.auto_close_after_ms,
            exited_by_auto_close: false,
            startup_error: None,
            screenshot_file: options.screenshot_file,
            screenshot_captured: false,
            screenshot_error: None,
            draw_plan_requested: false,
            draw_plan_window_count: 0,
            high_contrast_draw_plan_window_count: 0,
            draw_command_count: 0,
            text_command_count: 0,
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
            native_view_radio_selection_count: 0,
            native_view_radio_keyboard_selection_count: 0,
            native_view_radio_keyboard_focus_only_count: 0,
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
pub(crate) struct NativeViewInputRuntime {
    surface: Option<Rect>,
    dpi: Dpi,
    interaction_plan: Option<ViewInteractionPlan>,
    ui_command_view: Option<ViewNode<UiCommand>>,
    live_view: Option<SharedLiveViewRuntime>,
    focused_widget: Option<crate::WidgetId>,
    text_edit: Option<NativeTextEditState>,
    text_drag: Option<NativeTextDragState>,
    #[cfg(feature = "combo")]
    combo_type_ahead: NativeComboTypeAheadState,
    #[cfg(feature = "slider")]
    slider_drag: Option<crate::WidgetId>,
    #[cfg(any(feature = "date-picker", feature = "tabs"))]
    pointer_hover: Option<NativePointerVisualKey>,
    #[cfg(any(feature = "date-picker", feature = "tabs"))]
    pointer_pressed: Option<NativePointerVisualKey>,
    ime_preedit: Option<NativeViewImePreedit>,
    app_command_executor: Option<SharedAppCommandExecutor>,
    ui_command_executor: Option<SharedUiCommandExecutor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeViewImePreedit {
    widget: crate::WidgetId,
    text: String,
    selection: Option<(usize, usize)>,
    replacement: NativeTextSelection,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NativeViewInputDispatchReport {
    pub handled: bool,
    pub surface_changed: bool,
    pub focus_visual_changed: bool,
    #[cfg(any(feature = "date-picker", feature = "tabs"))]
    pub pointer_visual_changed: bool,
    pub hit_target_count: usize,
    pub message_count: usize,
    pub app_command_count: usize,
    pub ui_command_count: usize,
    pub ui_command_ids: Vec<&'static str>,
    pub focused_widget: Option<u64>,
    pub text_selection: Option<(usize, usize)>,
    pub text_caret: Option<usize>,
    pub text_selection_changed: bool,
    pub text_drag_active: bool,
    #[cfg(feature = "slider")]
    pub slider_value: Option<f32>,
    #[cfg(feature = "slider")]
    pub slider_value_changed: bool,
    #[cfg(feature = "slider")]
    pub slider_drag_active: bool,
    #[cfg(feature = "radio")]
    pub radio_selection_changed: bool,
    #[cfg(feature = "radio")]
    pub radio_keyboard_selection_changed: bool,
    #[cfg(feature = "radio")]
    pub radio_keyboard_focus_only: bool,
    #[cfg(feature = "combo")]
    pub combo_expanded_changed: bool,
    #[cfg(feature = "combo")]
    pub combo_selection_changed: bool,
    #[cfg(feature = "combo")]
    pub combo_keyboard_selection_changed: bool,
    #[cfg(feature = "combo")]
    pub combo_type_ahead_matched: bool,
    #[cfg(feature = "combo")]
    pub combo_scrolled: bool,
    #[cfg(feature = "tabs")]
    pub tab_selection_changed: bool,
    #[cfg(feature = "tabs")]
    pub tab_keyboard_selection_changed: bool,
    #[cfg(feature = "tabs")]
    pub tab_keyboard_focus_only: bool,
    pub ime_preedit_text: Option<String>,
    pub ime_selection: Option<(usize, usize)>,
    pub ime_caret_rect: Option<Rect>,
    pub redraw_plan: Option<NativeDrawPlan>,
    pub quit_requested: bool,
    pub errors: Vec<String>,
}

#[allow(dead_code)]
impl NativeViewInputRuntime {
    fn new(
        surface: Rect,
        interaction_plan: Option<ViewInteractionPlan>,
        ui_command_view: Option<ViewNode<UiCommand>>,
        live_view: Option<SharedLiveViewRuntime>,
        app_command_executor: Option<SharedAppCommandExecutor>,
        ui_command_executor: Option<SharedUiCommandExecutor>,
    ) -> Self {
        Self {
            surface: Some(surface),
            dpi: Dpi::standard(),
            interaction_plan,
            ui_command_view,
            live_view,
            focused_widget: None,
            text_edit: None,
            text_drag: None,
            #[cfg(feature = "combo")]
            combo_type_ahead: NativeComboTypeAheadState::default(),
            #[cfg(feature = "slider")]
            slider_drag: None,
            #[cfg(any(feature = "date-picker", feature = "tabs"))]
            pointer_hover: None,
            #[cfg(any(feature = "date-picker", feature = "tabs"))]
            pointer_pressed: None,
            ime_preedit: None,
            app_command_executor,
            ui_command_executor,
        }
    }

    fn hit_target_count(&self) -> usize {
        self.current_interaction_plan()
            .as_ref()
            .map(ViewInteractionPlan::hit_target_count)
            .unwrap_or(0)
    }

    fn current_interaction_plan(&self) -> Option<ViewInteractionPlan> {
        self.live_view
            .as_ref()
            .map(SharedLiveViewRuntime::interaction_plan)
            .or_else(|| self.interaction_plan.clone())
    }

    pub(crate) fn has_focused_text_input(&self) -> bool {
        self.focused_text_input_target().is_some()
    }

    pub(crate) fn accepts_committed_text_input(&self) -> bool {
        if self.has_focused_text_input() {
            return true;
        }
        #[cfg(feature = "combo")]
        {
            return self
                .focused_widget
                .and_then(|widget| {
                    self.current_interaction_plan()
                        .and_then(|plan| plan.hit_target_for_widget(widget))
                })
                .is_some_and(|target| target.kind == crate::ViewHitTargetKind::ComboBox);
        }
        #[cfg(not(feature = "combo"))]
        false
    }

    pub(crate) fn set_surface(&mut self, surface: Rect, dpi: Dpi) -> NativeViewInputDispatchReport {
        let surface = Rect {
            x: surface.x,
            y: surface.y,
            width: surface.width.max(0),
            height: surface.height.max(0),
        };
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ime_preedit_text: self.ime_preedit.as_ref().map(|state| state.text.clone()),
            ime_selection: self.ime_preedit.as_ref().and_then(|state| state.selection),
            ..NativeViewInputDispatchReport::default()
        };
        if self.surface == Some(surface) && self.dpi == dpi {
            report.ime_caret_rect = self.text_input_caret_rect();
            self.populate_text_report(&mut report);
            return report;
        }

        self.surface = Some(surface);
        self.dpi = dpi;
        report.surface_changed = true;
        report.handled = true;
        if let Some(runtime) = &self.live_view {
            runtime.set_surface(surface, dpi);
            self.interaction_plan = Some(runtime.interaction_plan());
            report.redraw_plan = Some(runtime.draw_plan());
        } else if let Some(view) = &mut self.ui_command_view {
            let mut layout_cx = ViewLayoutCx::new(surface, dpi);
            view.layout(&mut layout_cx);
            let interaction_plan = view.interaction_plan();
            report.hit_target_count = interaction_plan.hit_target_count();
            self.interaction_plan = Some(interaction_plan);
            let mut paint_cx = ViewPaintCx::new(dpi);
            view.paint(&mut paint_cx);
            report.redraw_plan = Some(paint_cx.into_plan());
        }

        if self.focused_widget.is_some_and(|widget| {
            self.current_interaction_plan()
                .map_or(true, |plan| plan.hit_target_for_widget(widget).is_none())
        }) {
            self.focused_widget = None;
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "combo")]
            self.combo_type_ahead.reset();
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            self.ime_preedit = None;
            report.focus_visual_changed = true;
        }
        self.sync_text_edit();
        if let Some(plan) = report.redraw_plan.take() {
            report.redraw_plan = Some(self.compose_input_visuals(plan));
        }
        report.hit_target_count = self.hit_target_count();
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        report.ime_preedit_text = self.ime_preedit.as_ref().map(|state| state.text.clone());
        report.ime_selection = self.ime_preedit.as_ref().and_then(|state| state.selection);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn focused_text_input_value(&self) -> Option<String> {
        let target = self.focused_text_input_target()?;
        self.widget_text_value(target.widget)
    }

    pub(crate) fn focused_text_input_snapshot(&self) -> Option<(String, NativeTextSelection)> {
        let target = self.focused_text_input_target()?;
        let value = self.widget_text_value(target.widget)?;
        let selection = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .map(|state| state.selection.clamp(&value))
            .unwrap_or_else(|| NativeTextSelection::collapsed(value.chars().count()));
        Some((value, selection))
    }

    pub(crate) fn ime_replacement_selection(&self) -> Option<NativeTextSelection> {
        self.ime_preedit.as_ref().map(|preedit| preedit.replacement)
    }

    pub(crate) fn text_input_caret_rect(&self) -> Option<Rect> {
        let target = self.focused_text_input_target()?;
        let (value, selection) = self.focused_text_input_snapshot()?;
        Some(native_text_visual_geometry(target, &value, selection, self.dpi).caret)
    }

    pub(crate) fn dispatch_pointer_click(&mut self, point: Point) -> NativeViewInputDispatchReport {
        let report = self.dispatch_pointer_down(point, false);
        if self.text_drag.take().is_some() {
            return report;
        }
        #[cfg(feature = "slider")]
        if self.slider_drag.take().is_some() {
            return report;
        }
        self.dispatch_pointer_up(point)
    }

    pub(crate) fn dispatch_pointer_down(
        &mut self,
        point: Point,
        shift: bool,
    ) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..NativeViewInputDispatchReport::default()
        };
        let interaction_plan = self.current_interaction_plan();
        let target = interaction_plan.and_then(|plan| plan.hit_target_at(point));
        report = self.dismiss_popup_overlays_except(target.map(|target| target.widget), report);
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        self.update_pointer_visual_state(
            target.and_then(native_pointer_visual_key),
            target.and_then(native_pointer_visual_key),
            &mut report,
        );
        let Some(target) = target else {
            return report;
        };
        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            self.text_drag = None;
            self.focus_target(target, &mut report);
            self.slider_drag = Some(target.widget);
            report.slider_drag_active = true;
            return self.dispatch_slider_pointer(target, point, report);
        }
        if !matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            self.text_drag = None;
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            return report;
        }

        report.handled = true;
        self.ime_preedit = None;
        self.focus_target(target, &mut report);
        let value = self.widget_text_value(target.widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let index = native_text_index_for_point(target, &value, point, self.dpi);
        let anchor = if shift { state.selection.anchor } else { index };
        let edit = set_pointer_selection(&value, &mut state.selection, anchor, index);
        self.text_edit = Some(state);
        self.text_drag = Some(NativeTextDragState {
            widget: target.widget,
            anchor,
        });
        report.text_selection_changed = edit.selection_changed;
        report.text_drag_active = true;
        report.redraw_plan = self.current_composed_draw_plan();
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn dispatch_pointer_move(&mut self, point: Point) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        {
            let hovered = self
                .current_interaction_plan()
                .and_then(|plan| plan.hit_target_at(point))
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, self.pointer_pressed, &mut report);
        }
        let Some(drag) = self.text_drag else {
            #[cfg(feature = "slider")]
            if let Some(widget) = self.slider_drag {
                if let Some(target) = self
                    .current_interaction_plan()
                    .and_then(|plan| plan.hit_target_for_widget(widget))
                {
                    report.slider_drag_active = true;
                    return self.dispatch_slider_pointer(target, point, report);
                }
                self.slider_drag = None;
            }
            return report;
        };
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(drag.widget))
        else {
            self.text_drag = None;
            return report;
        };
        let value = self.widget_text_value(drag.widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == drag.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(drag.widget, &value));
        let index = native_text_index_for_point(target, &value, point, self.dpi);
        let edit = set_pointer_selection(&value, &mut state.selection, drag.anchor, index);
        self.text_edit = Some(state);
        report.handled = true;
        report.text_selection_changed = edit.selection_changed;
        report.text_drag_active = true;
        if edit.selection_changed {
            report.redraw_plan = self.current_composed_draw_plan();
        }
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn dispatch_pointer_up(&mut self, point: Point) -> NativeViewInputDispatchReport {
        if self.text_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            self.text_drag = None;
            report.handled = true;
            report.text_drag_active = false;
            #[cfg(any(feature = "date-picker", feature = "tabs"))]
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "slider")]
        if self.slider_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            self.slider_drag = None;
            report.handled = true;
            report.slider_drag_active = false;
            #[cfg(any(feature = "date-picker", feature = "tabs"))]
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }

        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..NativeViewInputDispatchReport::default()
        };
        let interaction_plan = self.current_interaction_plan();
        let target = interaction_plan.and_then(|plan| plan.hit_target_at(point));
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        self.update_pointer_visual_state(
            target.and_then(native_pointer_visual_key),
            None,
            &mut report,
        );
        let Some(target) = target else {
            return report;
        };
        report.handled = true;
        #[cfg(feature = "combo")]
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::ComboBox | crate::ViewHitTargetKind::ComboBoxOption { .. }
        ) {
            self.combo_type_ahead.reset();
        }
        self.focus_target(target, &mut report);
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            return report;
        }

        let event = self.activation_event(target);

        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            report.radio_selection_changed = true;
        }
        #[cfg(feature = "combo")]
        match target.kind {
            crate::ViewHitTargetKind::ComboBox => report.combo_expanded_changed = true,
            crate::ViewHitTargetKind::ComboBoxOption { .. } => {
                report.combo_selection_changed = true;
                report.combo_expanded_changed = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
            }
            _ => {}
        }
        #[cfg(feature = "tabs")]
        if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
            report.tab_selection_changed = self
                .widget_tab_header_state(target.widget)
                .is_some_and(|state| !state.selected);
        }

        self.dispatch_view_event(event, report)
    }

    pub(crate) fn cancel_pointer_drag(&mut self) -> NativeViewInputDispatchReport {
        let had_drag = self.text_drag.take().is_some();
        #[cfg(feature = "slider")]
        let had_drag = had_drag | self.slider_drag.take().is_some();
        let mut report = NativeViewInputDispatchReport {
            handled: had_drag,
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            text_drag_active: false,
            #[cfg(feature = "slider")]
            slider_drag_active: false,
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn dispatch_pointer_leave(&mut self) -> NativeViewInputDispatchReport {
        let report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        {
            let mut report = report;
            self.update_pointer_visual_state(None, None, &mut report);
            report
        }
        #[cfg(not(any(feature = "date-picker", feature = "tabs")))]
        {
            report
        }
    }

    pub(crate) fn dispatch_key(&mut self, key: NativeViewKey) -> NativeViewInputDispatchReport {
        self.dispatch_key_with_modifiers(key, false, false)
    }

    pub(crate) fn dispatch_key_with_shift(
        &mut self,
        key: NativeViewKey,
        shift: bool,
    ) -> NativeViewInputDispatchReport {
        self.dispatch_key_with_modifiers(key, shift, false)
    }

    pub(crate) fn dispatch_key_with_modifiers(
        &mut self,
        key: NativeViewKey,
        shift: bool,
        control: bool,
    ) -> NativeViewInputDispatchReport {
        #[cfg(not(any(feature = "radio", feature = "tabs")))]
        let _ = control;
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        let Some(interaction_plan) = self.current_interaction_plan() else {
            return report;
        };
        if key == NativeViewKey::Tab && !control {
            let offset = if shift { -1 } else { 1 };
            if let Some(target) =
                interaction_plan.next_focus_target_where(self.focused_widget, offset, |target| {
                    self.widget_accepts_tab_focus(target)
                })
            {
                report.handled = true;
                report = self.dismiss_popup_overlays_except(Some(target.widget), report);
                self.focus_target(target, &mut report);
            }
            return report;
        }

        let Some(widget) = self.focused_widget else {
            return report;
        };

        #[cfg(feature = "tabs")]
        let tab_cycle_offset =
            native_tab_cycle_offset(crate::ZsTabPlatformStyle::current(), key, shift, control);
        #[cfg(feature = "tabs")]
        if let Some(offset) = tab_cycle_offset {
            let Some((tab_view, tab)) = self.widget_tab_cycle_target(widget, offset) else {
                return report;
            };
            report.handled = true;
            report.tab_selection_changed = true;
            report.tab_keyboard_selection_changed = true;
            if let Some(target) = interaction_plan.hit_target_for_widget(crate::WidgetId(tab.0)) {
                self.focus_target(target, &mut report);
            }
            return self.dispatch_view_event(
                ViewEvent::TabSelected {
                    widget: tab_view,
                    tab,
                },
                report,
            );
        }

        let Some(target) = interaction_plan.hit_target_for_widget(widget) else {
            self.focused_widget = None;
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "combo")]
            self.combo_type_ahead.reset();
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            report.focused_widget = None;
            return report;
        };

        if matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            let movement = match key {
                NativeViewKey::Left => Some(NativeTextMovement::Left),
                NativeViewKey::Right => Some(NativeTextMovement::Right),
                NativeViewKey::Home => Some(NativeTextMovement::Home),
                NativeViewKey::End => Some(NativeTextMovement::End),
                _ => None,
            };
            if let Some(movement) = movement {
                let value = self.widget_text_value(widget).unwrap_or_default();
                let mut state = self
                    .text_edit
                    .filter(|state| state.widget == widget)
                    .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
                let edit = move_selection(
                    &value,
                    &mut state.selection,
                    movement,
                    shift,
                    target.kind == crate::ViewHitTargetKind::TextEditor,
                );
                self.text_edit = Some(state);
                report.handled = edit.handled;
                report.text_selection_changed = edit.selection_changed;
                report.redraw_plan = self.current_composed_draw_plan();
                self.populate_text_report(&mut report);
                return report;
            }
        }

        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            let delta = if shift { 10 } else { 1 };
            let Some((current, range)) = self.widget_slider_state(widget) else {
                return report;
            };
            let value = match key {
                NativeViewKey::Left | NativeViewKey::Down => {
                    Some(range.offset_steps(current, -delta))
                }
                NativeViewKey::Right | NativeViewKey::Up => {
                    Some(range.offset_steps(current, delta))
                }
                NativeViewKey::Home => Some(range.min()),
                NativeViewKey::End => Some(range.max()),
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                report.slider_value = Some(value);
                if (value - current).abs() <= f32::EPSILON {
                    return report;
                }
                report.slider_value_changed = true;
                return self
                    .dispatch_view_event(ViewEvent::SliderChanged { widget, value }, report);
            }
        }

        #[cfg(feature = "combo")]
        if target.kind == crate::ViewHitTargetKind::ComboBox {
            self.combo_type_ahead.reset();
            let Some((selected, option_count, expanded)) = self.widget_combo_state(widget) else {
                return report;
            };
            let expanded_event = match key {
                NativeViewKey::Enter | NativeViewKey::Space => Some(!expanded),
                NativeViewKey::Escape if expanded => Some(false),
                _ => None,
            };
            if let Some(next_expanded) = expanded_event {
                report.handled = true;
                report.combo_expanded_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::ComboBoxExpandedChanged {
                        widget,
                        expanded: next_expanded,
                    },
                    report,
                );
            }

            let next_index = match key {
                NativeViewKey::Up if option_count > 0 => {
                    Some(selected.unwrap_or(option_count).saturating_sub(1))
                }
                NativeViewKey::Down if option_count > 0 => {
                    Some(selected.map_or(0, |index| index.saturating_add(1).min(option_count - 1)))
                }
                NativeViewKey::Home if option_count > 0 => Some(0),
                NativeViewKey::End if option_count > 0 => Some(option_count - 1),
                _ => None,
            };
            if let Some(index) = next_index {
                report.handled = true;
                if selected == Some(index) {
                    return report;
                }
                report.combo_selection_changed = true;
                report.combo_keyboard_selection_changed = true;
                report.combo_expanded_changed = expanded;
                return self
                    .dispatch_view_event(ViewEvent::ComboBoxSelected { widget, index }, report);
            }
        }

        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            let navigation = match key {
                NativeViewKey::Up => Some((crate::ViewStackDirection::Column, -1)),
                NativeViewKey::Down => Some((crate::ViewStackDirection::Column, 1)),
                NativeViewKey::Left => Some((crate::ViewStackDirection::Row, -1)),
                NativeViewKey::Right => Some((crate::ViewStackDirection::Row, 1)),
                _ => None,
            };
            if let Some((navigation, offset)) = navigation {
                let Some(next_widget) =
                    self.widget_radio_relative_widget(widget, navigation, offset)
                else {
                    return report;
                };
                report.handled = true;
                if next_widget == widget {
                    return report;
                }
                let Some(next_target) = interaction_plan.hit_target_for_widget(next_widget) else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                if control {
                    report.radio_keyboard_focus_only = true;
                    return report;
                }
                report.radio_selection_changed = true;
                report.radio_keyboard_selection_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::RadioSelected {
                        widget: next_widget,
                    },
                    report,
                );
            }
        }

        #[cfg(feature = "tabs")]
        if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
            let Some(state) = self.widget_tab_header_state(widget) else {
                return report;
            };
            let platform = crate::ZsTabPlatformStyle::current();
            let next_widget = match key {
                NativeViewKey::Left => state.previous,
                NativeViewKey::Right => state.next,
                NativeViewKey::Home if platform.supports_home_end_focus() => Some(state.first),
                NativeViewKey::End if platform.supports_home_end_focus() => Some(state.last),
                _ => None,
            };
            let navigation_key = matches!(key, NativeViewKey::Left | NativeViewKey::Right)
                || (platform.supports_home_end_focus()
                    && matches!(key, NativeViewKey::Home | NativeViewKey::End));
            if navigation_key {
                report.handled = true;
                let Some(next_widget) = next_widget else {
                    return report;
                };
                if next_widget == widget {
                    return report;
                }
                let Some(next_target) = interaction_plan.hit_target_for_widget(next_widget) else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                let Some(next_state) = self.widget_tab_header_state(next_widget) else {
                    return report;
                };
                if platform.arrow_selects() {
                    report.tab_selection_changed = !next_state.selected;
                    report.tab_keyboard_selection_changed = !next_state.selected;
                    if !next_state.selected {
                        return self.dispatch_view_event(
                            ViewEvent::TabSelected {
                                widget: next_state.tab_view,
                                tab: next_state.tab,
                            },
                            report,
                        );
                    }
                } else {
                    report.tab_keyboard_focus_only = true;
                }
                return report;
            }
        }

        #[cfg(feature = "date-picker")]
        if target.kind == crate::ViewHitTargetKind::DatePicker {
            let Some(state) = self.widget_date_picker_state(widget) else {
                return report;
            };
            let expanded = match key {
                NativeViewKey::Enter | NativeViewKey::Space => Some(!state.expanded),
                NativeViewKey::Escape if state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = expanded {
                report.handled = true;
                return self.dispatch_view_event(
                    ViewEvent::DatePickerExpandedChanged { widget, expanded },
                    report,
                );
            }
            let value = match key {
                NativeViewKey::Left => Some(state.value.add_days(-1)),
                NativeViewKey::Right => Some(state.value.add_days(1)),
                NativeViewKey::Up => Some(state.value.add_days(-7)),
                NativeViewKey::Down => Some(state.value.add_days(7)),
                NativeViewKey::Home => Some(state.value.first_day_of_month()),
                NativeViewKey::End => {
                    Some(state.value.first_day_of_month().add_months(1).add_days(-1))
                }
                _ => None,
            };
            if let Some(value) = value {
                let value = value.clamp(state.minimum, state.maximum);
                report.handled = true;
                if value == state.value {
                    return report;
                }
                return self.dispatch_view_event(ViewEvent::DateChanged { widget, value }, report);
            }
        }

        #[cfg(feature = "list")]
        if matches!(key, NativeViewKey::Up | NativeViewKey::Down) {
            let offset = if key == NativeViewKey::Up { -1 } else { 1 };
            if let Some((next_widget, _index)) = self.widget_list_relative_widget(widget, offset) {
                if let Some(next_target) = interaction_plan.hit_target_for_widget(next_widget) {
                    report.handled = true;
                    self.focus_target(next_target, &mut report);
                    return self.dispatch_view_event(
                        ViewEvent::Click {
                            widget: next_widget,
                        },
                        report,
                    );
                }
            }
        }

        let activates = matches!(
            (target.kind, key),
            (
                crate::ViewHitTargetKind::Button | crate::ViewHitTargetKind::Unknown,
                NativeViewKey::Enter | NativeViewKey::Space
            ) | (
                crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle,
                NativeViewKey::Space
            )
        );
        #[cfg(feature = "radio")]
        let activates = activates
            || matches!(
                (target.kind, key),
                (crate::ViewHitTargetKind::RadioButton, NativeViewKey::Space)
            );
        #[cfg(feature = "tabs")]
        let activates = activates
            || matches!(
                (target.kind, key),
                (
                    crate::ViewHitTargetKind::Tab { .. },
                    NativeViewKey::Enter | NativeViewKey::Space
                )
            );
        if activates {
            report.handled = true;
            #[cfg(feature = "radio")]
            if target.kind == crate::ViewHitTargetKind::RadioButton {
                report.radio_selection_changed = true;
            }
            #[cfg(feature = "tabs")]
            if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
                let changed = self
                    .widget_tab_header_state(target.widget)
                    .is_some_and(|state| !state.selected);
                report.tab_selection_changed = changed;
                report.tab_keyboard_selection_changed = changed;
            }
            return self.dispatch_view_event(self.activation_event(target), report);
        }
        report
    }

    pub(crate) fn dispatch_text_input(&mut self, text: &str) -> NativeViewInputDispatchReport {
        self.dispatch_text_input_at(text, std::time::Instant::now())
    }

    fn dispatch_text_input_at(
        &mut self,
        text: &str,
        _now: std::time::Instant,
    ) -> NativeViewInputDispatchReport {
        self.ime_preedit = None;
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        let Some(widget) = self.focused_widget else {
            return report;
        };
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
        else {
            self.focused_widget = None;
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "combo")]
            self.combo_type_ahead.reset();
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            report.focused_widget = None;
            return report;
        };
        #[cfg(feature = "combo")]
        if target.kind == crate::ViewHitTargetKind::ComboBox {
            let Some(query) = self.combo_type_ahead.push_text(widget, text, _now) else {
                return report;
            };
            report.handled = true;
            let Some((selected, option_count, expanded)) = self.widget_combo_state(widget) else {
                self.combo_type_ahead.reset();
                return report;
            };
            let start_after = query.match_start_after(selected, option_count);
            let Some(index) = self.widget_combo_type_ahead_match(widget, &query.text, start_after)
            else {
                return report;
            };
            report.combo_type_ahead_matched = true;
            if selected == Some(index) {
                return report;
            }
            report.combo_selection_changed = true;
            report.combo_keyboard_selection_changed = true;
            report.combo_expanded_changed = expanded;
            return self.dispatch_view_event(ViewEvent::ComboBoxSelected { widget, index }, report);
        }
        if !matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            return report;
        }

        let mut value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        state.clamp(&value);
        let mut handled = false;
        let mut text_changed = false;
        let mut selection_changed = false;
        let mut previous_was_carriage_return = false;
        for ch in text.chars() {
            let edit = match ch {
                '\u{8}' => delete_backward(&mut value, &mut state.selection),
                '\u{7f}' => delete_forward(&mut value, &mut state.selection),
                '\r' if target.kind == crate::ViewHitTargetKind::TextEditor => {
                    previous_was_carriage_return = true;
                    insert_text(&mut value, &mut state.selection, "\n")
                }
                '\n' if target.kind == crate::ViewHitTargetKind::TextEditor
                    && !previous_was_carriage_return =>
                {
                    insert_text(&mut value, &mut state.selection, "\n")
                }
                ch if !ch.is_control() => {
                    let mut buffer = [0_u8; 4];
                    insert_text(
                        &mut value,
                        &mut state.selection,
                        ch.encode_utf8(&mut buffer),
                    )
                }
                _ => Default::default(),
            };
            handled |= edit.handled;
            text_changed |= edit.text_changed;
            selection_changed |= edit.selection_changed;
            if ch != '\r' {
                previous_was_carriage_return = false;
            }
        }
        if !handled {
            return report;
        }
        report.handled = true;
        report.text_selection_changed = selection_changed;
        self.text_edit = Some(state);
        if text_changed {
            self.dispatch_view_event(ViewEvent::TextChanged { widget, value }, report)
        } else {
            report.redraw_plan = selection_changed
                .then(|| self.current_composed_draw_plan())
                .flatten();
            self.populate_text_report(&mut report);
            report
        }
    }

    pub(crate) fn dispatch_ime_preedit(
        &mut self,
        text: &str,
        selection: Option<(usize, usize)>,
    ) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ime_caret_rect: self.text_input_caret_rect(),
            ..NativeViewInputDispatchReport::default()
        };
        let Some(target) = self.focused_text_input_target() else {
            self.ime_preedit = None;
            return report;
        };
        if text.is_empty() {
            return self.cancel_ime_preedit();
        }

        let char_count = text.chars().count();
        let selection = selection.map(|(start, end)| {
            let start = start.min(char_count);
            let end = end.min(char_count);
            (start.min(end), start.max(end))
        });
        let value = self.widget_text_value(target.widget).unwrap_or_default();
        let mut edit_state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        edit_state.clamp(&value);
        self.text_edit = Some(edit_state);
        let replacement = self
            .ime_preedit
            .as_ref()
            .filter(|preedit| preedit.widget == target.widget)
            .map(|preedit| preedit.replacement)
            .unwrap_or(edit_state.selection);
        self.ime_preedit = Some(NativeViewImePreedit {
            widget: target.widget,
            text: text.to_string(),
            selection,
            replacement,
        });
        report.handled = true;
        report.ime_preedit_text = Some(text.to_string());
        report.ime_selection = selection;
        report.redraw_plan = self.current_composed_draw_plan();
        report
    }

    pub(crate) fn dispatch_ime_commit(&mut self, text: &str) -> NativeViewInputDispatchReport {
        let preedit = self.ime_preedit.take();
        let had_preedit = preedit.is_some();
        if let Some(preedit) = preedit {
            self.text_edit = Some(NativeTextEditState {
                widget: preedit.widget,
                selection: preedit.replacement,
            });
        }
        let mut report = self.dispatch_text_input(text);
        if had_preedit && !report.handled {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
        }
        report.ime_preedit_text = None;
        report.ime_selection = None;
        report.ime_caret_rect = self.text_input_caret_rect();
        report
    }

    pub(crate) fn cancel_ime_preedit(&mut self) -> NativeViewInputDispatchReport {
        let had_preedit = self.ime_preedit.take().is_some();
        let mut report = NativeViewInputDispatchReport {
            handled: had_preedit,
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ime_caret_rect: self.text_input_caret_rect(),
            redraw_plan: had_preedit
                .then(|| self.current_composed_draw_plan())
                .flatten(),
            ..NativeViewInputDispatchReport::default()
        };
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn blur_focus(&mut self) -> NativeViewInputDispatchReport {
        let mut report = self.dismiss_popup_overlays_except(
            None,
            NativeViewInputDispatchReport {
                hit_target_count: self.hit_target_count(),
                ..NativeViewInputDispatchReport::default()
            },
        );
        let had_focus = self.focused_widget.take().is_some();
        self.text_edit = None;
        self.text_drag = None;
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        let had_preedit = self.ime_preedit.take().is_some();
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        self.update_pointer_visual_state(None, None, &mut report);
        report.handled |= had_focus || had_preedit;
        report.focus_visual_changed = had_focus;
        report.hit_target_count = self.hit_target_count();
        if report.handled {
            report.redraw_plan = self.current_composed_draw_plan();
        }
        report.focused_widget = None;
        report
    }

    pub(crate) fn dispatch_pointer_scroll(
        &mut self,
        point: Point,
        delta_y: Dp,
    ) -> NativeViewInputDispatchReport {
        let report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..NativeViewInputDispatchReport::default()
        };
        let interaction_plan = self
            .live_view
            .as_ref()
            .map(SharedLiveViewRuntime::interaction_plan)
            .or_else(|| self.interaction_plan.clone());
        let Some(interaction_plan) = interaction_plan else {
            return report;
        };
        let Some(target) = interaction_plan.hit_target_at(point) else {
            return report;
        };

        #[cfg(feature = "combo")]
        if matches!(target.kind, crate::ViewHitTargetKind::ComboBoxOption { .. })
            && delta_y.0 != 0.0
        {
            let Some((_, option_count, true)) = self.widget_combo_state(target.widget) else {
                return report;
            };
            let Some(visible_range) = interaction_plan.combo_visible_option_range(target.widget)
            else {
                return report;
            };
            let visible_count = visible_range.len();
            let maximum_first = option_count.saturating_sub(visible_count);
            let next_first = if delta_y.0 > 0.0 {
                visible_range.start.saturating_add(1).min(maximum_first)
            } else {
                visible_range.start.saturating_sub(1)
            };
            let mut report = report;
            report.handled = true;
            if next_first == visible_range.start {
                return report;
            }
            report.combo_scrolled = true;
            return self.dispatch_view_event(
                ViewEvent::ComboBoxScrolled {
                    widget: target.widget,
                    first_visible_index: next_first,
                },
                report,
            );
        }

        #[cfg(feature = "scroll")]
        {
            let scroll_widget = self
                .live_view
                .as_ref()
                .and_then(|runtime| runtime.widget_scroll_target(target.widget))
                .or_else(|| {
                    self.ui_command_view
                        .as_ref()
                        .and_then(|view| view.widget_scroll_target(target.widget))
                });
            if let Some(widget) = scroll_widget {
                let mut report = report;
                report.handled = true;
                return self.dispatch_view_event(ViewEvent::ScrollBy { widget, delta_y }, report);
            }
        }

        let _ = (target, delta_y);
        report
    }

    fn focus_target(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut NativeViewInputDispatchReport,
    ) {
        if self.focused_widget == Some(target.widget) {
            self.ensure_text_edit_for_target(target);
            report.focused_widget = Some(target.widget.0);
            report.ime_caret_rect = self.text_input_caret_rect();
            self.populate_text_report(report);
            return;
        }
        self.ime_preedit = None;
        self.text_drag = None;
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        self.focused_widget = Some(target.widget);
        self.ensure_text_edit_for_target(target);
        report.focused_widget = Some(target.widget.0);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(report);
        report.focus_visual_changed = true;
        report.redraw_plan = self.current_composed_draw_plan();
    }

    fn focused_text_input_target(&self) -> Option<crate::ViewHitTarget> {
        let widget = self.focused_widget?;
        self.current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .filter(|target| {
                matches!(
                    target.kind,
                    crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                )
            })
    }

    fn current_composed_draw_plan(&self) -> Option<NativeDrawPlan> {
        let plan = if let Some(runtime) = &self.live_view {
            runtime.draw_plan()
        } else {
            let view = self.ui_command_view.as_ref()?;
            let mut paint_cx = ViewPaintCx::new(self.dpi);
            view.paint(&mut paint_cx);
            paint_cx.into_plan()
        };
        Some(self.compose_input_visuals(plan))
    }

    fn compose_input_visuals(&self, plan: NativeDrawPlan) -> NativeDrawPlan {
        let mut plan = plan;
        #[cfg(any(feature = "date-picker", feature = "tabs"))]
        if let Some(interaction_plan) = self.current_interaction_plan() {
            decorate_native_pointer_visuals(
                &mut plan,
                &interaction_plan,
                self.pointer_hover,
                self.pointer_pressed,
                self.dpi,
            );
        }
        if let (Some(target), Some((value, selection))) = (
            self.focused_text_input_target(),
            self.focused_text_input_snapshot(),
        ) {
            decorate_native_text_edit_visuals(&mut plan, target, &value, selection, self.dpi);
        }
        let mut plan = self.compose_ime_preedit(plan);
        if let Some(interaction_plan) = self.current_interaction_plan() {
            decorate_native_focus_ring(&mut plan, &interaction_plan, self.focused_widget, self.dpi);
        }
        plan
    }

    #[cfg(any(feature = "date-picker", feature = "tabs"))]
    fn update_pointer_visual_state(
        &mut self,
        hovered: Option<NativePointerVisualKey>,
        pressed: Option<NativePointerVisualKey>,
        report: &mut NativeViewInputDispatchReport,
    ) {
        if self.pointer_hover == hovered && self.pointer_pressed == pressed {
            return;
        }
        self.pointer_hover = hovered;
        self.pointer_pressed = pressed;
        report.handled = true;
        report.pointer_visual_changed = true;
        report.redraw_plan = self.current_composed_draw_plan();
    }

    fn compose_ime_preedit(&self, mut plan: NativeDrawPlan) -> NativeDrawPlan {
        let Some(preedit) = &self.ime_preedit else {
            return plan;
        };
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|interaction| interaction.hit_target_for_widget(preedit.widget))
        else {
            return plan;
        };
        let committed = self.widget_text_value(preedit.widget).unwrap_or_default();
        let (start, end) = preedit.replacement.clamp(&committed).ordered();
        let start_byte = char_to_byte_index(&committed, start);
        let end_byte = char_to_byte_index(&committed, end);
        let mut composed = committed.clone();
        composed.replace_range(start_byte..end_byte, &preedit.text);
        let mut decorated = false;
        for command in plan.commands.iter_mut().rev() {
            let NativeDrawCommand::Text(text) = command else {
                continue;
            };
            if text.text == committed && rect_contains_rect(target.bounds, text.bounds) {
                text.text = composed.clone();
                decorated = true;
                break;
            }
        }
        if decorated {
            plan.push(NativeDrawCommand::StrokeRect {
                rect: target.bounds,
                stroke: NativeDrawFill::Role(crate::ColorRole::Accent),
                width: 2,
            });
        }
        plan
    }

    fn ensure_text_edit_for_target(&mut self, target: crate::ViewHitTarget) {
        if !matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            self.text_edit = None;
            self.text_drag = None;
            return;
        }
        let value = self.widget_text_value(target.widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        state.clamp(&value);
        self.text_edit = Some(state);
    }

    fn sync_text_edit(&mut self) {
        let Some(widget) = self.focused_widget else {
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            return;
        };
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
        else {
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            return;
        };
        self.ensure_text_edit_for_target(target);
    }

    fn populate_text_report(&self, report: &mut NativeViewInputDispatchReport) {
        let Some((_value, selection)) = self.focused_text_input_snapshot() else {
            report.text_selection = None;
            report.text_caret = None;
            return;
        };
        report.text_selection = Some(selection.ordered());
        report.text_caret = Some(selection.caret);
    }

    #[cfg(feature = "slider")]
    fn dispatch_slider_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: Point,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let Some((current, range)) = self.widget_slider_state(target.widget) else {
            self.slider_drag = None;
            return report;
        };
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, self.dpi).track;
        let fraction = point.x.saturating_sub(track.x) as f32 / track.width.max(1) as f32;
        let value = range.value_at_fraction(fraction);
        report.handled = true;
        report.slider_value = Some(value);
        report.slider_drag_active = self.slider_drag.is_some();
        if (value - current).abs() <= f32::EPSILON {
            return report;
        }
        report.slider_value_changed = true;
        self.dispatch_view_event(
            ViewEvent::SliderChanged {
                widget: target.widget,
                value,
            },
            report,
        )
    }

    fn activation_event(&self, target: crate::ViewHitTarget) -> ViewEvent {
        #[cfg(feature = "date-picker")]
        match target.kind {
            crate::ViewHitTargetKind::DatePickerDay { date } => {
                return ViewEvent::DateChanged {
                    widget: target.widget,
                    value: date,
                };
            }
            crate::ViewHitTargetKind::DatePickerPreviousMonth => {
                let month = self
                    .widget_date_picker_state(target.widget)
                    .map(|state| state.visible_month.add_months(-1))
                    .unwrap_or_else(|| {
                        crate::ZsDate::new(crate::ZsDate::MIN_YEAR, 1, 1)
                            .expect("minimum date is valid")
                    });
                return ViewEvent::DatePickerMonthChanged {
                    widget: target.widget,
                    month,
                };
            }
            crate::ViewHitTargetKind::DatePickerNextMonth => {
                let month = self
                    .widget_date_picker_state(target.widget)
                    .map(|state| state.visible_month.add_months(1))
                    .unwrap_or_else(|| {
                        crate::ZsDate::new(crate::ZsDate::MAX_YEAR, 12, 1)
                            .expect("maximum date is valid")
                    });
                return ViewEvent::DatePickerMonthChanged {
                    widget: target.widget,
                    month,
                };
            }
            crate::ViewHitTargetKind::DatePicker => {
                let expanded = self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                return ViewEvent::DatePickerExpandedChanged {
                    widget: target.widget,
                    expanded: !expanded,
                };
            }
            _ => {}
        }
        #[cfg(feature = "combo")]
        match target.kind {
            crate::ViewHitTargetKind::ComboBoxOption { index } => {
                return ViewEvent::ComboBoxSelected {
                    widget: target.widget,
                    index,
                };
            }
            crate::ViewHitTargetKind::ComboBox => {
                let expanded = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
                return ViewEvent::ComboBoxExpandedChanged {
                    widget: target.widget,
                    expanded: !expanded,
                };
            }
            _ => {}
        }
        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            return ViewEvent::RadioSelected {
                widget: target.widget,
            };
        }
        #[cfg(feature = "tabs")]
        if let crate::ViewHitTargetKind::Tab { tab_view, tab, .. } = target.kind {
            return ViewEvent::TabSelected {
                widget: tab_view,
                tab,
            };
        }
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle
        ) {
            ViewEvent::Toggled {
                widget: target.widget,
                checked: !self.widget_checked_value(target.widget).unwrap_or(false),
            }
        } else {
            ViewEvent::Click {
                widget: target.widget,
            }
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

    #[cfg(any(feature = "combo", feature = "date-picker"))]
    fn dismiss_popup_overlays_except(
        &mut self,
        except: Option<crate::WidgetId>,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let Some(interaction_plan) = self.current_interaction_plan() else {
            return report;
        };
        let should_dismiss = interaction_plan.hit_targets.iter().any(|target| {
            if Some(target.widget) == except {
                return false;
            }
            match target.kind {
                #[cfg(feature = "combo")]
                crate::ViewHitTargetKind::ComboBox => self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded),
                #[cfg(feature = "date-picker")]
                crate::ViewHitTargetKind::DatePicker => self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                _ => false,
            }
        });
        if !should_dismiss {
            return report;
        }
        #[cfg(feature = "combo")]
        {
            report.combo_expanded_changed |= interaction_plan.hit_targets.iter().any(|target| {
                Some(target.widget) != except
                    && target.kind == crate::ViewHitTargetKind::ComboBox
                    && self
                        .widget_combo_state(target.widget)
                        .is_some_and(|(_, _, expanded)| expanded)
            });
        }
        report.handled = true;
        self.dispatch_view_event(crate::ViewEvent::DismissPopupOverlays { except }, report)
    }

    #[cfg(not(any(feature = "combo", feature = "date-picker")))]
    fn dismiss_popup_overlays_except(
        &mut self,
        _except: Option<crate::WidgetId>,
        report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        report
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

    fn dispatch_view_event(
        &mut self,
        event: ViewEvent,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let (commands, ui_commands, quit_requested) = if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_event(&event);
            report.message_count = update.message_count;
            if update.redraw {
                report.redraw_plan = Some(live_view.draw_plan());
                report.hit_target_count = live_view.interaction_plan().hit_target_count();
            }
            (update.commands, update.ui_commands, update.quit_requested)
        } else {
            let mut event_cx = ViewEventCx::new();
            if let Some(view) = &mut self.ui_command_view {
                view.event(&mut event_cx, &event);
                if let Some(surface) = self.surface {
                    let mut layout_cx = ViewLayoutCx::new(surface, self.dpi);
                    view.layout(&mut layout_cx);
                    let interaction_plan = view.interaction_plan();
                    report.hit_target_count = interaction_plan.hit_target_count();
                    self.interaction_plan = Some(interaction_plan);
                }
                let mut paint_cx = ViewPaintCx::new(self.dpi);
                view.paint(&mut paint_cx);
                report.redraw_plan = Some(paint_cx.into_plan());
            }
            let messages = event_cx.into_messages();
            report.message_count = messages.len();
            (Vec::new(), messages, false)
        };

        report.app_command_count = commands.len();
        report.ui_command_count = ui_commands.len();
        report.quit_requested = quit_requested || commands.contains(&Command::Quit);
        let app_executor = self.app_command_executor.clone();
        for command in commands {
            if let Some(executor) = &app_executor {
                if let Err(error) = executor.dispatch(command) {
                    report.errors.push(error.to_string());
                }
            }
        }
        let ui_executor = self.ui_command_executor.clone();
        for command in ui_commands {
            report.ui_command_ids.push(command.id.0);
            if let Some(executor) = &ui_executor {
                if let Err(error) = executor.dispatch(command) {
                    report.errors.push(error.to_string());
                }
            }
        }
        if self.focused_widget.is_some_and(|widget| {
            self.current_interaction_plan()
                .map_or(true, |plan| plan.hit_target_for_widget(widget).is_none())
        }) {
            self.focused_widget = None;
            self.text_edit = None;
            self.text_drag = None;
            #[cfg(feature = "combo")]
            self.combo_type_ahead.reset();
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            self.ime_preedit = None;
            report.focus_visual_changed = true;
        }
        self.sync_text_edit();
        if let Some(plan) = report.redraw_plan.take() {
            report.redraw_plan = Some(self.compose_input_visuals(plan));
        }
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        report.ime_preedit_text = self.ime_preedit.as_ref().map(|state| state.text.clone());
        report.ime_selection = self.ime_preedit.as_ref().and_then(|state| state.selection);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    #[cfg(all(windows, feature = "windows-win32"))]
    fn windows_win32_route(&self) -> Option<crate::windows_win32_host::WindowsWin32ViewInputRoute> {
        let route = if let Some(runtime) = &self.live_view {
            crate::windows_win32_host::WindowsWin32ViewInputRoute::from_live_view(runtime.clone())
        } else {
            crate::windows_win32_host::WindowsWin32ViewInputRoute::new(
                self.interaction_plan.clone()?,
                self.ui_command_view.clone()?,
            )
        };
        let route = match &self.app_command_executor {
            Some(executor) => route.app_command_executor(executor.clone()),
            None => route,
        };
        Some(match &self.ui_command_executor {
            Some(executor) => route.ui_command_executor(executor.clone()),
            None => route,
        })
    }
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x.saturating_add(inner.width) <= outer.x.saturating_add(outer.width)
        && inner.y.saturating_add(inner.height) <= outer.y.saturating_add(outer.height)
}

#[allow(dead_code)]
fn record_draw_plan_smoke(
    report: &mut NativeWindowSmokeRunReport,
    draw_plans: &[Option<NativeDrawPlan>],
) {
    report.draw_plan_window_count = draw_plans.iter().filter(|plan| plan.is_some()).count();
    report.high_contrast_draw_plan_window_count = draw_plans
        .iter()
        .filter_map(|plan| plan.as_ref())
        .filter(|plan| plan.theme_mode == crate::ZsuiThemeMode::HighContrast)
        .count();
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
    let ui_command_executor = runtime.ui_command_executor.clone();
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
            match &ui_command_executor {
                Some(executor) => match executor.dispatch(command) {
                    Ok(events) => {
                        report.native_view_ui_command_executed_count += 1;
                        report.native_view_ui_command_event_count += events.len();
                    }
                    Err(err) => {
                        report.native_view_ui_command_failed_count += 1;
                        report.native_view_ui_command_errors.push(err.to_string());
                    }
                },
                None => report.native_view_ui_command_unhandled_count += 1,
            }
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
    report.native_view_ui_command_executed_count += input.ui_command_executed_count;
    report.native_view_ui_command_failed_count += input.ui_command_failed_count;
    report.native_view_ui_command_unhandled_count += input.ui_command_unhandled_count;
    report.native_view_ui_command_event_count += input.ui_command_event_count;
    report
        .native_view_ui_command_errors
        .extend(input.ui_command_errors.iter().cloned());
    report.native_view_app_command_count += input.app_command_count;
    report.native_view_app_command_executed_count += input.app_command_executed_count;
    report.native_view_app_command_failed_count += input.app_command_failed_count;
    report.native_view_app_command_unhandled_count += input.app_command_unhandled_count;
    report.native_view_app_command_event_count += input.app_command_event_count;
    report
        .native_view_app_command_names
        .extend(input.app_command_names.iter().copied());
    report
        .native_view_app_command_errors
        .extend(input.app_command_errors.iter().cloned());
    report
        .native_view_ui_command_ids
        .extend(input.ui_command_ids.iter().copied());
    report.native_view_live_revision = report
        .native_view_live_revision
        .max(input.live_view_revision);
    report.native_view_quit_requested |= input.quit_requested;
    report.native_view_unhandled_click_count += input.unhandled_click_count;
    report.native_view_focus_count += input.focus_count;
    report.native_view_focus_visual_count += input.focus_visual_count;
    report.native_view_focus_traversal_count += input.focus_traversal_count;
    report.native_view_text_input_count += input.text_input_count;
    report.native_view_text_navigation_count += input.text_navigation_count;
    report.native_view_text_selection_change_count += input.text_selection_change_count;
    report.native_view_text_caret = input.text_caret.or(report.native_view_text_caret);
    report.native_view_pointer_down_count += input.pointer_down_count;
    report.native_view_pointer_move_count += input.pointer_move_count;
    report.native_view_pointer_up_count += input.pointer_up_count;
    report.native_view_pointer_visual_change_count += input.pointer_visual_change_count;
    report.native_view_text_drag_count += input.text_drag_count;
    report.native_view_slider_value_change_count += input.slider_value_change_count;
    report.native_view_slider_keyboard_change_count += input.slider_keyboard_change_count;
    report.native_view_slider_drag_count += input.slider_drag_count;
    report.native_view_radio_selection_count += input.radio_selection_count;
    report.native_view_radio_keyboard_selection_count += input.radio_keyboard_selection_count;
    report.native_view_radio_keyboard_focus_only_count += input.radio_keyboard_focus_only_count;
    report.native_view_combo_expanded_change_count += input.combo_expanded_change_count;
    report.native_view_combo_selection_count += input.combo_selection_count;
    report.native_view_combo_keyboard_selection_count += input.combo_keyboard_selection_count;
    report.native_view_combo_type_ahead_match_count += input.combo_type_ahead_match_count;
    report.native_view_combo_scroll_count += input.combo_scroll_count;
    report.native_view_tab_selection_count += input.tab_selection_count;
    report.native_view_tab_keyboard_selection_count += input.tab_keyboard_selection_count;
    report.native_view_tab_keyboard_focus_only_count += input.tab_keyboard_focus_only_count;
    report.native_view_toggle_count += input.toggle_count;
    report.native_view_selection_count += input.selection_count;
    report.native_view_keyboard_selection_count += input.keyboard_selection_count;
    report.native_view_key_down_count += input.key_down_count;
    report.native_view_keyboard_activation_count += input.keyboard_activation_count;
    report.native_view_unhandled_key_count += input.unhandled_key_count;
    report.native_view_scroll_count += input.scroll_count;
    report.native_view_unhandled_scroll_count += input.unhandled_scroll_count;
    report.events.extend(input.events.clone());
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_lparam_from_point(point: Point) -> isize {
    let x = point.x as i16 as u16 as u32;
    let y = point.y as i16 as u16 as u32;
    ((y << 16) | x) as isize
}

#[cfg(all(windows, feature = "windows-win32"))]
fn post_windows_native_view_input(
    hwnd: windows_sys::Win32::Foundation::HWND,
    input: &NativeViewSmokeInput,
) {
    use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        PostMessageW, WM_CHAR, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
        WM_MOUSEWHEEL,
    };

    match input {
        NativeViewSmokeInput::Click(point) => unsafe {
            let lparam = windows_lparam_from_point(*point);
            PostMessageW(hwnd, WM_LBUTTONDOWN, 0, lparam);
            PostMessageW(hwnd, WM_LBUTTONUP, 0, lparam);
        },
        NativeViewSmokeInput::Drag { start, end } => unsafe {
            PostMessageW(hwnd, WM_LBUTTONDOWN, 0, windows_lparam_from_point(*start));
            PostMessageW(hwnd, WM_MOUSEMOVE, 0, windows_lparam_from_point(*end));
            PostMessageW(hwnd, WM_LBUTTONUP, 0, windows_lparam_from_point(*end));
        },
        NativeViewSmokeInput::Text(text) => {
            for ch in text.chars() {
                unsafe {
                    PostMessageW(hwnd, WM_CHAR, ch as usize, 0);
                }
            }
        }
        NativeViewSmokeInput::KeyDown(key) => unsafe {
            PostMessageW(
                hwnd,
                WM_KEYDOWN,
                windows_wparam_from_native_view_key(*key),
                0,
            );
        },
        NativeViewSmokeInput::Scroll { point, delta_y } => unsafe {
            let mut screen_point = windows_sys::Win32::Foundation::POINT {
                x: point.x,
                y: point.y,
            };
            ClientToScreen(hwnd, &mut screen_point);
            PostMessageW(
                hwnd,
                WM_MOUSEWHEEL,
                windows_wparam_from_scroll_delta_y(*delta_y),
                windows_lparam_from_point(Point {
                    x: screen_point.x,
                    y: screen_point.y,
                }),
            );
        },
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_wparam_from_scroll_delta_y(delta_y: i32) -> usize {
    let wheel_delta = ((-(delta_y as f32) / 48.0) * 120.0).round() as i16;
    ((wheel_delta as u16 as usize) << 16) & 0xffff_0000
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_wparam_from_native_view_key(key: NativeViewKey) -> usize {
    match key {
        NativeViewKey::Enter => 0x0d,
        NativeViewKey::Escape => 0x1b,
        NativeViewKey::Tab => 0x09,
        NativeViewKey::Space => 0x20,
        NativeViewKey::Up => 0x26,
        NativeViewKey::Down => 0x28,
        NativeViewKey::Left => 0x25,
        NativeViewKey::Right => 0x27,
        NativeViewKey::Home => 0x24,
        NativeViewKey::End => 0x23,
        NativeViewKey::PageUp => 0x21,
        NativeViewKey::PageDown => 0x22,
    }
}

#[derive(Debug, Clone)]
pub struct NativeWindowBuilder {
    app_name: String,
    window: WindowSpec,
    draw_plan: Option<NativeDrawPlan>,
    view_interaction_plan: Option<ViewInteractionPlan>,
    view_ui_command_tree: Option<ViewNode<UiCommand>>,
    view_layout_node_count: usize,
    shell_runtime: Option<ZsShellRuntime>,
    live_view_runtime: Option<SharedLiveViewRuntime>,
    app_command_executor: Option<SharedAppCommandExecutor>,
    ui_command_executor: Option<SharedUiCommandExecutor>,
}

impl PartialEq for NativeWindowBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.app_name == other.app_name
            && self.window == other.window
            && self.draw_plan == other.draw_plan
            && self.view_interaction_plan == other.view_interaction_plan
            && self.view_ui_command_tree.is_some() == other.view_ui_command_tree.is_some()
            && self.view_layout_node_count == other.view_layout_node_count
            && self.shell_runtime == other.shell_runtime
            && self.live_view_runtime == other.live_view_runtime
            && self.app_command_executor == other.app_command_executor
            && self.ui_command_executor == other.ui_command_executor
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
            shell_runtime: None,
            live_view_runtime: None,
            app_command_executor: None,
            ui_command_executor: None,
        }
    }

    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window = self.window.size(width, height);
        if let Some(runtime) = &mut self.shell_runtime {
            runtime.set_surface(
                Rect {
                    x: 0,
                    y: 0,
                    width: width as i32,
                    height: height as i32,
                },
                Dpi::standard(),
            );
            self.draw_plan = Some(runtime.draw_plan());
        }
        if let Some(runtime) = &self.live_view_runtime {
            runtime.set_surface(
                Rect {
                    x: 0,
                    y: 0,
                    width: width as i32,
                    height: height as i32,
                },
                Dpi::standard(),
            );
            self.view_interaction_plan = Some(runtime.interaction_plan());
            self.draw_plan = Some(runtime.draw_plan());
        }
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

    pub fn icon_path(mut self, icon_path: impl Into<String>) -> Self {
        self.window = self.window.icon_path(icon_path);
        self
    }

    pub fn menu(mut self, menu: MenuSpec) -> Self {
        self.window = self.window.menu(menu);
        self
    }

    pub fn draw_plan(mut self, draw_plan: NativeDrawPlan) -> Self {
        self.draw_plan = Some(draw_plan);
        self.view_interaction_plan = None;
        self.view_ui_command_tree = None;
        self.view_layout_node_count = 0;
        self.shell_runtime = None;
        self.live_view_runtime = None;
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
        self.shell_runtime = None;
        self.live_view_runtime = None;
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
        self.shell_runtime = None;
        self.live_view_runtime = None;
        self
    }

    pub fn stateful_view<State, Msg, ViewFn, UpdateFn>(
        mut self,
        state: State,
        view_fn: ViewFn,
        update_fn: UpdateFn,
    ) -> Self
    where
        State: Send + 'static,
        Msg: Clone + Send + 'static,
        ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
        UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
    {
        let runtime = live_view_runtime(
            state,
            view_fn,
            update_fn,
            Rect {
                x: 0,
                y: 0,
                width: self.window.width as i32,
                height: self.window.height as i32,
            },
            Dpi::standard(),
        );
        let interaction_plan = runtime.interaction_plan();
        self.view_layout_node_count = interaction_plan.hit_target_count();
        self.view_interaction_plan = Some(interaction_plan);
        self.draw_plan = Some(runtime.draw_plan());
        self.view_ui_command_tree = None;
        self.shell_runtime = None;
        self.live_view_runtime = Some(runtime);
        self
    }

    pub fn app_command_executor(mut self, executor: impl AppCommandExecutor + 'static) -> Self {
        self.app_command_executor = Some(SharedAppCommandExecutor::new(executor));
        self
    }

    pub fn shared_app_command_executor(mut self, executor: SharedAppCommandExecutor) -> Self {
        self.app_command_executor = Some(executor);
        self
    }

    pub fn ui_command_executor(mut self, executor: impl UiCommandExecutor + 'static) -> Self {
        self.ui_command_executor = Some(SharedUiCommandExecutor::new(executor));
        self
    }

    pub fn shared_ui_command_executor(mut self, executor: SharedUiCommandExecutor) -> Self {
        self.ui_command_executor = Some(executor);
        self
    }

    pub fn shell_layout(mut self, spec: ZsShellLayoutSpec) -> Self {
        let runtime = ZsShellRuntime::new(
            spec,
            Rect {
                x: 0,
                y: 0,
                width: self.window.width as i32,
                height: self.window.height as i32,
            },
            Dpi::standard(),
        );
        self.draw_plan = Some(runtime.draw_plan());
        self.view_interaction_plan = None;
        self.view_ui_command_tree = None;
        self.view_layout_node_count = 0;
        self.shell_runtime = Some(runtime);
        self.live_view_runtime = None;
        self
    }

    #[cfg(feature = "workbench")]
    pub fn workbench(self, spec: ZsWorkbenchSpec) -> Self {
        let surface = Rect {
            x: 0,
            y: 0,
            width: self.window.width as i32,
            height: self.window.height as i32,
        };
        self.draw_plan(spec.native_draw_plan(surface, Dpi::standard()))
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

    pub fn native_shell_runtime(&self) -> Option<&ZsShellRuntime> {
        self.shell_runtime.as_ref()
    }

    pub fn native_live_view_runtime(&self) -> Option<&SharedLiveViewRuntime> {
        self.live_view_runtime.as_ref()
    }

    pub fn native_app_command_executor(&self) -> Option<&SharedAppCommandExecutor> {
        self.app_command_executor.as_ref()
    }

    pub fn native_ui_command_executor(&self) -> Option<&SharedUiCommandExecutor> {
        self.ui_command_executor.as_ref()
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        app(self.app_name).window(self.window).build()
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        let draw_plan = self.draw_plan.clone();
        let view_runtime = self.native_view_input_runtime();
        let shell_runtime = self.shell_runtime.clone();
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        host.set_next_window_draw_plan(draw_plan);
        host.set_next_window_view_runtime(view_runtime);
        host.set_next_window_shell_runtime(shell_runtime);
        app.run_with_host(&mut host)
    }

    pub fn run_smoke(
        self,
        options: NativeWindowSmokeRunOptions,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        let draw_plan = self.draw_plan.clone();
        let view_runtime = self.native_view_input_runtime();
        let shell_runtime = self.shell_runtime.clone();
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
            shell_runtime,
            options,
        )
    }

    fn native_view_input_runtime(&self) -> NativeViewInputRuntime {
        NativeViewInputRuntime::new(
            Rect {
                x: 0,
                y: 0,
                width: self.window.width as i32,
                height: self.window.height as i32,
            },
            self.view_interaction_plan.clone(),
            self.view_ui_command_tree.clone(),
            self.live_view_runtime.clone(),
            self.app_command_executor.clone(),
            self.ui_command_executor.clone(),
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeWindowContentMissing;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeWindowContentReady;

#[derive(Debug, Clone)]
pub struct TypedNativeWindowBuilder<ContentState> {
    inner: NativeWindowBuilder,
    content_state: std::marker::PhantomData<fn() -> ContentState>,
}

impl<ContentState> PartialEq for TypedNativeWindowBuilder<ContentState> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl TypedNativeWindowBuilder<NativeWindowContentMissing> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            inner: NativeWindowBuilder::new(title),
            content_state: std::marker::PhantomData,
        }
    }
}

impl<ContentState> TypedNativeWindowBuilder<ContentState> {
    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.inner = self.inner.app_name(app_name);
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.inner = self.inner.size(width, height);
        self
    }

    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.inner = self.inner.min_size(width, height);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.inner = self.inner.visible(visible);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.inner = self.inner.resizable(resizable);
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.inner = self.inner.decorations(decorations);
        self
    }

    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.inner = self.inner.always_on_top(always_on_top);
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.inner = self.inner.transparent(transparent);
        self
    }

    pub fn icon_path(mut self, icon_path: impl Into<String>) -> Self {
        self.inner = self.inner.icon_path(icon_path);
        self
    }

    pub fn menu(mut self, menu: MenuSpec) -> Self {
        self.inner = self.inner.menu(menu);
        self
    }

    pub fn app_command_executor(mut self, executor: impl AppCommandExecutor + 'static) -> Self {
        self.inner = self.inner.app_command_executor(executor);
        self
    }

    pub fn shared_app_command_executor(mut self, executor: SharedAppCommandExecutor) -> Self {
        self.inner = self.inner.shared_app_command_executor(executor);
        self
    }

    pub fn ui_command_executor(mut self, executor: impl UiCommandExecutor + 'static) -> Self {
        self.inner = self.inner.ui_command_executor(executor);
        self
    }

    pub fn shared_ui_command_executor(mut self, executor: SharedUiCommandExecutor) -> Self {
        self.inner = self.inner.shared_ui_command_executor(executor);
        self
    }

    pub fn draw_plan(
        self,
        draw_plan: NativeDrawPlan,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady> {
        TypedNativeWindowBuilder::ready(self.inner.draw_plan(draw_plan))
    }

    pub fn view<Msg: Clone>(
        self,
        view: ViewNode<Msg>,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady> {
        TypedNativeWindowBuilder::ready(self.inner.view(view))
    }

    pub fn ui_command_view(
        self,
        view: ViewNode<UiCommand>,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady> {
        TypedNativeWindowBuilder::ready(self.inner.ui_command_view(view))
    }

    pub fn stateful_view<State, Msg, ViewFn, UpdateFn>(
        self,
        state: State,
        view_fn: ViewFn,
        update_fn: UpdateFn,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady>
    where
        State: Send + 'static,
        Msg: Clone + Send + 'static,
        ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
        UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
    {
        TypedNativeWindowBuilder::ready(self.inner.stateful_view(state, view_fn, update_fn))
    }

    pub fn shell_layout(
        self,
        spec: ZsShellLayoutSpec,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady> {
        TypedNativeWindowBuilder::ready(self.inner.shell_layout(spec))
    }

    #[cfg(feature = "workbench")]
    pub fn workbench(
        self,
        spec: ZsWorkbenchSpec,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady> {
        TypedNativeWindowBuilder::ready(self.inner.workbench(spec))
    }

    pub fn window_spec(&self) -> &WindowSpec {
        self.inner.window_spec()
    }
}

impl TypedNativeWindowBuilder<NativeWindowContentReady> {
    fn ready(inner: NativeWindowBuilder) -> Self {
        Self {
            inner,
            content_state: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        self.inner.build()
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        self.inner.run()
    }

    pub fn run_smoke(
        self,
        options: NativeWindowSmokeRunOptions,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        self.inner.run_smoke(options)
    }

    pub fn into_builder(self) -> NativeWindowBuilder {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub struct NativeWindowHost {
    inner: MemoryHost,
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    next_window_draw_plan: Option<NativeDrawPlan>,
    view_runtimes: Vec<NativeViewInputRuntime>,
    next_window_view_runtime: NativeViewInputRuntime,
    shell_runtimes: Vec<Option<ZsShellRuntime>>,
    next_window_shell_runtime: Option<ZsShellRuntime>,
}

impl NativeWindowHost {
    pub fn new() -> Self {
        Self {
            inner: MemoryHost::with_capabilities(HostCapabilities::current_native_window_host()),
            windows: Vec::new(),
            trays: Vec::new(),
            draw_plans: Vec::new(),
            next_window_draw_plan: None,
            view_runtimes: Vec::new(),
            next_window_view_runtime: NativeViewInputRuntime::default(),
            shell_runtimes: Vec::new(),
            next_window_shell_runtime: None,
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

    fn set_next_window_view_runtime(&mut self, runtime: NativeViewInputRuntime) {
        self.next_window_view_runtime = runtime;
    }

    fn set_next_window_shell_runtime(&mut self, runtime: Option<ZsShellRuntime>) {
        self.next_window_shell_runtime = runtime;
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

#[cfg(any(
    all(windows, feature = "windows-win32"),
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
fn menu_command_count(menu: &MenuSpec) -> usize {
    menu.items
        .iter()
        .map(|item| match item {
            MenuItemSpec::Command { .. } => 1,
            MenuItemSpec::Separator => 0,
            MenuItemSpec::Submenu { menu, .. } => menu_command_count(menu),
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
        self.view_runtimes
            .push(std::mem::take(&mut self.next_window_view_runtime));
        self.shell_runtimes
            .push(self.next_window_shell_runtime.take());
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
        #[cfg(feature = "clipboard")]
        {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|err| ZsuiError::host("read_clipboard", err.to_string()))?;
            return match clipboard.get_text() {
                Ok(text) => Ok(Some(ClipboardData::Text(text))),
                Err(arboard::Error::ContentNotAvailable) => Ok(None),
                Err(err) => Err(ZsuiError::host("read_clipboard", err.to_string())),
            };
        }

        #[cfg(not(feature = "clipboard"))]
        {
            Err(ZsuiError::unsupported(
                "clipboard_text",
                "enable the clipboard feature to compile the native text clipboard service",
            ))
        }
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        #[cfg(feature = "clipboard")]
        {
            let text = match data {
                ClipboardData::Text(text) => text.clone(),
                ClipboardData::Empty => String::new(),
                ClipboardData::ImageRgba { .. } => {
                    return Err(ZsuiError::unsupported(
                        "clipboard_image",
                        "the native image clipboard service is not connected",
                    ));
                }
                ClipboardData::Files(_) => {
                    return Err(ZsuiError::unsupported(
                        "clipboard_files",
                        "the native file clipboard service is not connected",
                    ));
                }
            };
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|err| ZsuiError::host("write_clipboard", err.to_string()))?;
            return clipboard
                .set_text(text)
                .map_err(|err| ZsuiError::host("write_clipboard", err.to_string()));
        }

        #[cfg(not(feature = "clipboard"))]
        {
            let _ = data;
            Err(ZsuiError::unsupported(
                "clipboard_text",
                "enable the clipboard feature to compile the native text clipboard service",
            ))
        }
    }

    fn open_file_picker(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<String>>> {
        #[cfg(all(windows, feature = "windows-win32"))]
        {
            return crate::windows_win32_host::windows_win32_open_file_dialog(spec).map(
                |selection| {
                    selection.map(|paths| {
                        paths
                            .into_iter()
                            .map(|path| path.to_string_lossy().into_owned())
                            .collect()
                    })
                },
            );
        }

        #[cfg(all(target_os = "macos", feature = "macos-appkit"))]
        {
            return crate::macos_appkit_services::macos_appkit_open_file_dialog(spec).map(
                |selection| {
                    selection.map(|paths| {
                        paths
                            .into_iter()
                            .map(|path| path.to_string_lossy().into_owned())
                            .collect()
                    })
                },
            );
        }

        #[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
        {
            return crate::linux_gtk_services::linux_gtk_open_file_dialog(spec).map(|selection| {
                selection.map(|paths| {
                    paths
                        .into_iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect()
                })
            });
        }

        #[cfg(not(any(
            all(windows, feature = "windows-win32"),
            all(target_os = "macos", feature = "macos-appkit"),
            all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
        )))]
        {
            self.inner.open_file_picker(spec)
        }
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
            self.view_runtimes.clone(),
            self.shell_runtimes.clone(),
        )
    }
}

impl crate::ClipboardService for NativeWindowHost {
    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        <Self as ZsuiHost>::read_clipboard(self)
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        <Self as ZsuiHost>::write_clipboard(self, data)
    }
}

impl crate::FileDialogService for NativeWindowHost {
    fn open_file_dialog(
        &mut self,
        spec: &FileDialogSpec,
    ) -> ZsuiResult<Option<Vec<std::path::PathBuf>>> {
        <Self as ZsuiHost>::open_file_picker(self, spec).map(|selection| {
            selection.map(|paths| paths.into_iter().map(std::path::PathBuf::from).collect())
        })
    }

    fn save_file_dialog(
        &mut self,
        spec: &crate::SaveFileDialogSpec,
    ) -> ZsuiResult<Option<std::path::PathBuf>> {
        #[cfg(all(windows, feature = "windows-win32"))]
        {
            return crate::windows_win32_host::windows_win32_save_file_dialog(spec);
        }

        #[cfg(all(target_os = "macos", feature = "macos-appkit"))]
        {
            return crate::macos_appkit_services::macos_appkit_save_file_dialog(spec);
        }

        #[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
        {
            return crate::linux_gtk_services::linux_gtk_save_file_dialog(spec);
        }

        #[cfg(not(any(
            all(windows, feature = "windows-win32"),
            all(target_os = "macos", feature = "macos-appkit"),
            all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
        )))]
        {
            let _ = spec;
            Err(ZsuiError::unsupported(
                "save_file_dialog",
                "the selected desktop backend does not implement a native save dialog",
            ))
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtimes: Vec<NativeViewInputRuntime>,
    shell_runtimes: Vec<Option<ZsShellRuntime>>,
) -> ZsuiResult<()> {
    let input_routes = view_runtimes
        .iter()
        .map(NativeViewInputRuntime::windows_win32_route)
        .collect::<Vec<_>>();
    let shell_routes = shell_runtimes
        .into_iter()
        .map(|runtime| runtime.map(crate::windows_win32_host::WindowsWin32ShellInputRoute::new))
        .collect::<Vec<_>>();
    crate::windows_win32_host::run_windows_win32_native_window_event_loop_with_routes_and_status_items(
        &windows,
        &draw_plans,
        &input_routes,
        &shell_routes,
        &trays,
    )
}

#[cfg(all(windows, not(feature = "windows-win32")))]
fn run_native_window_event_loop(
    _windows: Vec<WindowSpec>,
    _trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtimes: Vec<NativeViewInputRuntime>,
    _shell_runtimes: Vec<Option<ZsShellRuntime>>,
) -> ZsuiResult<()> {
    Err(ZsuiError::unsupported(
        "native_window",
        "enable the windows-win32 feature to compile the direct Win32 native window host",
    ))
}

#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtimes: Vec<NativeViewInputRuntime>,
    _shell_runtimes: Vec<Option<ZsShellRuntime>>,
) -> ZsuiResult<()> {
    if !trays.is_empty() {
        return Err(ZsuiError::unsupported(
            "native_window_status_item",
            "the AppKit NSStatusItem runtime is not connected to the unified event loop",
        ));
    }
    crate::macos_appkit_services::run_macos_appkit_native_window_event_loop(
        &windows,
        &draw_plans,
        &view_runtimes,
        None,
    )
    .map(|_| ())
}

#[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtimes: Vec<NativeViewInputRuntime>,
    _shell_runtimes: Vec<Option<ZsShellRuntime>>,
) -> ZsuiResult<()> {
    if !trays.is_empty() {
        return Err(ZsuiError::unsupported(
            "native_window_status_item",
            "the GTK4 status-item runtime is not connected to the unified event loop",
        ));
    }
    crate::linux_gtk_services::run_linux_gtk_native_window_event_loop(
        &windows,
        &draw_plans,
        &view_runtimes,
        None,
    )
    .map(|_| ())
}

#[cfg(any(
    all(
        feature = "desktop-winit",
        not(feature = "macos-appkit"),
        target_os = "macos"
    ),
    all(
        feature = "desktop-winit",
        not(feature = "linux-gtk"),
        target_os = "linux",
        not(target_env = "ohos")
    )
))]
fn run_native_window_event_loop(
    windows: Vec<WindowSpec>,
    trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtimes: Vec<NativeViewInputRuntime>,
    _shell_runtimes: Vec<Option<ZsShellRuntime>>,
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
    if windows.iter().any(|window| window.menu.is_some()) {
        return Err(ZsuiError::unsupported(
            "native_menu",
            "the first-pass Winit host does not implement a native application menu",
        ));
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
    shell_runtime: Option<ZsShellRuntime>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    use std::{thread, time::Duration};
    use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }

    let mut report = NativeWindowSmokeRunReport {
        requested_window_count: windows.len(),
        window_menu_requested_count: windows
            .iter()
            .filter(|window| window.menu.is_some())
            .count(),
        window_menu_native_command_count: windows
            .iter()
            .filter_map(|window| window.menu.as_ref())
            .map(menu_command_count)
            .sum(),
        auto_close_after_ms: options.auto_close_after_ms,
        ..NativeWindowSmokeRunReport::empty(options.clone())
    };
    record_draw_plan_smoke(&mut report, &draw_plans);
    report.native_view_hit_target_count = view_runtime.hit_target_count();
    let input_routes = match view_runtime.windows_win32_route() {
        Some(route) => vec![Some(route)],
        None => Vec::new(),
    };
    let shell_routes = match shell_runtime {
        Some(runtime) => vec![Some(
            crate::windows_win32_host::WindowsWin32ShellInputRoute::new(runtime),
        )],
        None => Vec::new(),
    };
    let handles = crate::windows_win32_host::create_owned_windows_for_specs_with_routes(
        &windows,
        &draw_plans,
        &input_routes,
        &shell_routes,
    )
    .map_err(|err| {
        report.startup_error = Some(err.to_string());
        report.events.push("startup_error".to_string());
        err
    })?;

    report.created_window_count = handles.len();
    report.window_menu_attached_count = report.window_menu_requested_count;
    report.events.extend(
        windows
            .iter()
            .map(|spec| format!("window_created:{}", spec.title)),
    );

    if let Some(menu) = windows.first().and_then(|window| window.menu.as_ref()) {
        let table = crate::windows_win32_host::WindowsWin32StatusMenuCommandTable::from_menu(menu);
        if let Some(native_id) = table.first_native_id() {
            match crate::windows_win32_host::dispatch_windows_win32_window_menu_command(
                handles[0].main(),
                native_id,
            ) {
                Some(NativeStatusMenuCommandResult::Dispatched(command)) => {
                    report.window_menu_command_routed = true;
                    report
                        .events
                        .push(format!("window_menu_command_dispatched:{command:?}"));
                }
                Some(NativeStatusMenuCommandResult::Disabled) => {
                    report.window_menu_command_error =
                        Some("first window menu command is disabled".to_string());
                }
                Some(NativeStatusMenuCommandResult::NotFound) | None => {
                    report.window_menu_command_error =
                        Some("first window menu command was not found".to_string());
                }
            }
        }
    }

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

    if options.native_view_inputs.is_empty() {
        let mut click_points = options.native_view_click_points.iter();
        if !options.native_view_text_inputs.is_empty() {
            if let Some(point) = click_points.next() {
                post_windows_native_view_input(
                    handles[0].main(),
                    &NativeViewSmokeInput::Click(*point),
                );
            }
        }
        for text in &options.native_view_text_inputs {
            post_windows_native_view_input(
                handles[0].main(),
                &NativeViewSmokeInput::Text(text.clone()),
            );
        }
        for point in click_points {
            post_windows_native_view_input(handles[0].main(), &NativeViewSmokeInput::Click(*point));
        }
        for key in &options.native_view_key_downs {
            post_windows_native_view_input(handles[0].main(), &NativeViewSmokeInput::KeyDown(*key));
        }
        for (point, delta_y) in &options.native_view_scroll_inputs {
            post_windows_native_view_input(
                handles[0].main(),
                &NativeViewSmokeInput::Scroll {
                    point: *point,
                    delta_y: *delta_y,
                },
            );
        }
    } else {
        for input in &options.native_view_inputs {
            post_windows_native_view_input(handles[0].main(), input);
        }
    }

    let close_handles: Vec<isize> = handles
        .iter()
        .map(|handles| handles.main() as isize)
        .collect();
    let auto_close_after = Duration::from_millis(options.auto_close_after_ms.max(1));
    let screenshot_file = report.screenshot_file.clone();
    let screenshot_handle = handles[0].main() as isize;
    let capture_delay = screenshot_file
        .as_ref()
        .map(|_| Duration::from_millis(options.auto_close_after_ms.clamp(1, 250)))
        .unwrap_or_default();
    let worker = thread::spawn(move || {
        if !capture_delay.is_zero() {
            thread::sleep(capture_delay);
        }
        let screenshot_result = screenshot_file.map(|path| {
            let result = capture_win32_hwnd_png(screenshot_handle as _, &path);
            (path, result)
        });
        let remaining = auto_close_after.saturating_sub(capture_delay);
        if !remaining.is_zero() {
            thread::sleep(remaining);
        }
        for handle in close_handles {
            unsafe {
                PostMessageW(handle as _, WM_CLOSE, 0, 0);
            }
        }
        screenshot_result
    });

    match crate::windows_win32_host::WindowsWin32MessageLoop::run_with_windows(&handles) {
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

    match worker.join() {
        Ok(Some((path, Ok(())))) => {
            report.screenshot_captured = true;
            report.events.push(format!("screenshot_captured:{path}"));
        }
        Ok(Some((_path, Err(err)))) => {
            report.screenshot_error = Some(err);
            report.events.push("screenshot_error".to_string());
        }
        Ok(None) => {}
        Err(_) => {
            report.screenshot_error = Some("native smoke worker panicked".to_string());
            report.events.push("smoke_worker_error".to_string());
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
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    mut view_runtime: NativeViewInputRuntime,
    _shell_runtime: Option<ZsShellRuntime>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }
    let mut report = NativeWindowSmokeRunReport {
        requested_window_count: windows.len(),
        window_menu_requested_count: windows
            .iter()
            .filter(|window| window.menu.is_some())
            .count(),
        window_menu_native_command_count: windows
            .iter()
            .filter_map(|window| window.menu.as_ref())
            .map(menu_command_count)
            .sum(),
        auto_close_after_ms: options.auto_close_after_ms,
        ..NativeWindowSmokeRunReport::empty(options.clone())
    };
    record_draw_plan_smoke(&mut report, &draw_plans);
    record_native_view_input_smoke(&mut report, &mut view_runtime, &options);

    #[cfg(all(target_os = "macos", feature = "macos-appkit"))]
    let created = crate::macos_appkit_services::run_macos_appkit_native_window_event_loop(
        &windows,
        &draw_plans,
        std::slice::from_ref(&view_runtime),
        Some(options.auto_close_after_ms),
    )?;
    #[cfg(all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk"))]
    let created = crate::linux_gtk_services::run_linux_gtk_native_window_event_loop(
        &windows,
        &draw_plans,
        std::slice::from_ref(&view_runtime),
        Some(options.auto_close_after_ms),
    )?;

    report.created_window_count = created;
    report.window_menu_attached_count = report.window_menu_requested_count.min(created);
    report.close_requested_count = created;
    report.exited_by_auto_close = true;
    report.events.extend(
        windows
            .iter()
            .take(created)
            .map(|spec| format!("window_created:{}", spec.title)),
    );
    report.events.push("auto_close_elapsed".to_string());

    if options.screenshot_file.is_some() {
        report.screenshot_error = Some(
            "native screenshot capture still requires target AppKit/GTK4 integration".to_string(),
        );
        report.events.push("screenshot_error".to_string());
    }
    if options.status_item.is_some() {
        report.status_item_error = Some(
            "status-item smoke is not connected to the AppKit/GTK4 unified event loop".to_string(),
        );
        report.events.push("status_item_unsupported".to_string());
    }
    if options.require_visible_window && !report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_status_item {
        return Err(ZsuiError::unsupported(
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
    all(
        feature = "desktop-winit",
        not(feature = "macos-appkit"),
        target_os = "macos"
    ),
    all(
        feature = "desktop-winit",
        not(feature = "linux-gtk"),
        target_os = "linux",
        not(target_env = "ohos")
    )
))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    mut view_runtime: NativeViewInputRuntime,
    _shell_runtime: Option<ZsShellRuntime>,
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
        all(target_os = "macos", not(feature = "macos-appkit")),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            not(feature = "linux-gtk")
        )
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
    _shell_runtime: Option<ZsShellRuntime>,
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
    all(
        target_os = "macos",
        not(feature = "macos-appkit"),
        not(feature = "desktop-winit")
    ),
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        not(feature = "linux-gtk"),
        not(feature = "desktop-winit")
    )
))]
fn run_native_window_event_loop(
    _windows: Vec<WindowSpec>,
    _trays: Vec<TraySpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtimes: Vec<NativeViewInputRuntime>,
    _shell_runtimes: Vec<Option<ZsShellRuntime>>,
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
    all(
        target_os = "macos",
        not(feature = "macos-appkit"),
        not(feature = "desktop-winit")
    ),
    all(
        target_os = "linux",
        not(target_env = "ohos"),
        not(feature = "linux-gtk"),
        not(feature = "desktop-winit")
    )
))]
fn run_native_window_smoke_event_loop(
    _windows: Vec<WindowSpec>,
    _draw_plans: Vec<Option<NativeDrawPlan>>,
    _view_runtime: NativeViewInputRuntime,
    _shell_runtime: Option<ZsShellRuntime>,
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

    #[cfg(feature = "label")]
    #[test]
    fn typed_native_window_builder_requires_content_transition() {
        let missing: TypedNativeWindowBuilder<NativeWindowContentMissing> =
            typed_native_window("Typed Window").size(640, 420);
        let ready: TypedNativeWindowBuilder<NativeWindowContentReady> =
            missing.view(crate::text::<()>("Ready"));

        assert_eq!(ready.window_spec().width, 640);
        let app = ready.build().unwrap();
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.windows[0].title, "Typed Window");
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
    fn native_window_builder_stateful_view_rebuilds_after_update() {
        #[derive(Clone)]
        enum Msg {
            Increment,
        }
        struct State {
            count: u32,
        }

        let button_id = crate::WidgetId::new(70);
        let builder = native_window("Stateful View").size(360, 220).stateful_view(
            State { count: 0 },
            move |state| {
                crate::column([
                    crate::text(format!("Count: {}", state.count)),
                    crate::button("Increment")
                        .id(button_id)
                        .on_click(Msg::Increment),
                ])
            },
            |state, message, _cx| match message {
                Msg::Increment => state.count += 1,
            },
        );
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful view should keep a live runtime");

        let update = runtime.dispatch_event(&ViewEvent::Click { widget: button_id });

        assert!(update.redraw);
        assert_eq!(update.revision, 1);
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Count: 1"
        )));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_window_builder_routes_ui_command_view_clicks_for_smoke() {
        let executor = SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let builder = native_window("View Command Example")
            .size(360, 220)
            .ui_command_view(crate::column(vec![
                crate::text::<UiCommand>("Settings"),
                crate::button("Save")
                    .id(crate::WidgetId::new(7))
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.save"))),
            ]))
            .shared_ui_command_executor(executor.clone());
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
        assert_eq!(report.native_view_ui_command_executed_count, 1);
        assert_eq!(report.native_view_ui_command_event_count, 1);
        assert_eq!(executor.report().executed_count, 1);
        assert_eq!(report.native_view_ui_command_ids, vec!["zsui.test.save"]);
        assert_eq!(report.native_view_unhandled_click_count, 0);
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_view_runtime_dispatches_platform_click_and_returns_repaint() {
        #[derive(Clone)]
        enum Msg {
            Increment,
        }
        struct State {
            count: u32,
        }

        let button_id = crate::WidgetId::new(71);
        let builder = native_window("Platform Click")
            .size(360, 220)
            .stateful_view(
                State { count: 0 },
                move |state| {
                    crate::column([
                        crate::text(format!("Count: {}", state.count)),
                        crate::button("Increment")
                            .id(button_id)
                            .on_click(Msg::Increment),
                    ])
                },
                |state, message, _cx| match message {
                    Msg::Increment => state.count += 1,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(button_id))
            .expect("button should have a platform hit target");
        let mut runtime = builder.native_view_input_runtime();

        let report = runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        });

        assert!(report.handled);
        assert_eq!(report.message_count, 1);
        assert!(report.errors.is_empty());
        assert!(report.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(
                    command,
                    crate::NativeDrawCommand::Text(text) if text.text == "Count: 1"
                )
            })
        }));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_view_runtime_dispatches_platform_click_to_shared_executor() {
        let executor = SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let button_id = crate::WidgetId::new(72);
        let builder = native_window("Platform Command")
            .size(360, 220)
            .ui_command_view(crate::column([
                crate::text::<UiCommand>("Settings"),
                crate::button("Save")
                    .id(button_id)
                    .on_click(UiCommand::app(crate::CommandId("zsui.platform.save"))),
            ]))
            .shared_ui_command_executor(executor.clone());
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(button_id))
            .expect("button should have a platform hit target");
        let mut runtime = builder.native_view_input_runtime();

        let report = runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        });

        assert!(report.handled);
        assert_eq!(report.ui_command_count, 1);
        assert_eq!(report.ui_command_ids, vec!["zsui.platform.save"]);
        assert!(report.errors.is_empty());
        assert_eq!(executor.report().executed_count, 1);
    }

    #[cfg(all(feature = "label", feature = "scroll"))]
    #[test]
    fn native_view_runtime_dispatches_platform_scroll_and_repaints() {
        fn scrolled(_offset: Dp) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.platform.scrolled"))
        }

        let executor = SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let scroll_id = crate::WidgetId::new(73);
        let builder = native_window("Platform Scroll")
            .size(360, 220)
            .ui_command_view(
                crate::scroll(crate::column([
                    crate::text::<UiCommand>("First"),
                    crate::text::<UiCommand>("Second"),
                    crate::text::<UiCommand>("Third"),
                ]))
                .id(scroll_id)
                .content_height(Dp::new(600.0))
                .on_scroll(scrolled),
            )
            .shared_ui_command_executor(executor.clone());
        let before = builder
            .native_draw_plan()
            .cloned()
            .expect("scroll view should have an initial draw plan");
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(scroll_id))
            .expect("scroll view should have a platform hit target");
        let mut runtime = builder.native_view_input_runtime();

        let report = runtime.dispatch_pointer_scroll(
            Point {
                x: target.bounds.x + target.bounds.width / 2,
                y: target.bounds.y + target.bounds.height / 2,
            },
            Dp::new(48.0),
        );

        assert!(report.handled);
        assert_eq!(report.ui_command_ids, vec!["zsui.platform.scrolled"]);
        assert!(report.errors.is_empty());
        assert!(report.redraw_plan.is_some_and(|plan| plan != before));
        assert_eq!(executor.report().executed_count, 1);
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn native_view_runtime_traverses_focus_and_activates_from_keyboard() {
        let executor = SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let first = crate::WidgetId::new(74);
        let second = crate::WidgetId::new(75);
        let builder = native_window("Platform Keyboard")
            .size(360, 220)
            .ui_command_view(crate::column([
                crate::button("First")
                    .id(first)
                    .on_click(UiCommand::app(crate::CommandId("zsui.keyboard.first"))),
                crate::button("Second")
                    .id(second)
                    .on_click(UiCommand::app(crate::CommandId("zsui.keyboard.second"))),
            ]))
            .shared_ui_command_executor(executor.clone());
        let first_bounds = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(first))
            .expect("first button should have focus geometry")
            .bounds;
        let second_bounds = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(second))
            .expect("second button should have focus geometry")
            .bounds;
        let mut runtime = builder.native_view_input_runtime();

        let first_focus = runtime.dispatch_key(NativeViewKey::Tab);
        let second_focus = runtime.dispatch_key(NativeViewKey::Tab);
        let activated = runtime.dispatch_key(NativeViewKey::Enter);

        assert!(first_focus.handled);
        assert!(first_focus.focus_visual_changed);
        assert_eq!(first_focus.focused_widget, Some(first.0));
        assert!(first_focus.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                    if rect.x == first_bounds.x + 1 && rect.y == first_bounds.y + 1)
            })
        }));
        assert!(second_focus.focus_visual_changed);
        assert_eq!(second_focus.focused_widget, Some(second.0));
        assert!(second_focus.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                    if rect.x == second_bounds.x + 1 && rect.y == second_bounds.y + 1)
            }) && !plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                    if rect.x == first_bounds.x + 1 && rect.y == first_bounds.y + 1)
            })
        }));
        assert!(activated.handled);
        assert_eq!(activated.ui_command_ids, vec!["zsui.keyboard.second"]);
        assert_eq!(executor.report().executed_count, 1);

        let blurred = runtime.blur_focus();
        assert!(blurred.focus_visual_changed);
        assert_eq!(blurred.focused_widget, None);
        assert!(blurred.redraw_plan.as_ref().is_some_and(|plan| {
            !plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                    if rect.x == first_bounds.x + 1 || rect.x == second_bounds.x + 1)
            })
        }));
    }

    #[cfg(all(feature = "label", feature = "textbox"))]
    #[test]
    fn native_view_runtime_routes_unicode_selection_and_replacement() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
        }
        struct State {
            value: String,
        }

        let textbox_id = crate::WidgetId::new(76);
        let builder = native_window("Platform Text").size(360, 220).stateful_view(
            State {
                value: String::new(),
            },
            move |state| {
                crate::textbox(&state.value)
                    .id(textbox_id)
                    .on_change(Msg::Changed)
            },
            |state, message, _cx| match message {
                Msg::Changed(value) => state.value = value,
            },
        );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have a platform hit target");
        let mut runtime = builder.native_view_input_runtime();

        let focus = runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        });
        let typed = runtime.dispatch_text_input("A中文Z");
        runtime.dispatch_key(NativeViewKey::Home);
        runtime.dispatch_key(NativeViewKey::Right);
        runtime.dispatch_key_with_shift(NativeViewKey::Right, true);
        let selected = runtime.dispatch_key_with_shift(NativeViewKey::Right, true);
        let replaced = runtime.dispatch_text_input("🙂");

        assert_eq!(focus.focused_widget, Some(textbox_id.0));
        assert!(typed.handled);
        assert_eq!(selected.text_selection, Some((1, 3)));
        assert_eq!(selected.text_caret, Some(3));
        assert!(selected.text_selection_changed);
        assert!(selected.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(
                    command,
                    NativeDrawCommand::FillRect {
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: crate::ColorRole::Accent,
                            alpha: 64,
                        },
                        ..
                    }
                )
            })
        }));
        assert!(replaced.handled);
        assert_eq!(replaced.text_selection, Some((2, 2)));
        assert!(replaced.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(
                    command, crate::NativeDrawCommand::Text(text) if text.text == "A🙂Z"
                )
            })
        }));
    }

    #[cfg(all(feature = "label", feature = "textbox"))]
    #[test]
    fn native_view_runtime_drags_unicode_text_selection_and_replaces_it() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
        }

        let textbox_id = crate::WidgetId::new(80);
        let builder = native_window("Platform Pointer Selection")
            .size(360, 220)
            .stateful_view(
                "A中文Z".to_string(),
                move |value| crate::textbox(value).id(textbox_id).on_change(Msg::Changed),
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have pointer selection geometry");
        let mut runtime = builder.native_view_input_runtime();

        let pressed = runtime.dispatch_pointer_down(
            Point {
                x: target.bounds.x + 16,
                y: target.bounds.y + 12,
            },
            false,
        );
        let dragged = runtime.dispatch_pointer_move(Point {
            x: target.bounds.x + 32,
            y: target.bounds.y + 12,
        });
        let released = runtime.dispatch_pointer_up(Point {
            x: target.bounds.x + 32,
            y: target.bounds.y + 12,
        });

        assert_eq!(pressed.text_selection, Some((1, 1)));
        assert!(pressed.text_drag_active);
        assert_eq!(dragged.text_selection, Some((1, 3)));
        assert!(dragged.text_selection_changed);
        assert!(dragged.text_drag_active);
        assert_eq!(released.text_selection, Some((1, 3)));
        assert!(!released.text_drag_active);
        assert!(dragged.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(
                    command,
                    NativeDrawCommand::FillRect {
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: crate::ColorRole::Accent,
                            alpha: 64,
                        },
                        ..
                    }
                )
            })
        }));

        let replaced = runtime.dispatch_text_input("🙂");

        assert_eq!(replaced.text_selection, Some((2, 2)));
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("A🙂Z"));
    }

    #[cfg(feature = "slider")]
    #[test]
    fn native_view_runtime_routes_slider_drag_and_keyboard_steps() {
        #[derive(Clone)]
        enum Msg {
            Changed(f32),
        }

        let slider_id = crate::WidgetId::new(81);
        let range = crate::SliderRange::new(0.0, 100.0).step(5.0);
        let builder = native_window("Platform Slider")
            .size(360, 220)
            .stateful_view(
                0.0_f32,
                move |value| {
                    crate::slider(*value, range)
                        .id(slider_id)
                        .on_slide(Msg::Changed)
                },
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(slider_id))
            .expect("slider should have pointer geometry");
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, Dpi::standard()).track;
        let mut runtime = builder.native_view_input_runtime();

        let pressed = runtime.dispatch_pointer_down(
            Point {
                x: track.x + track.width / 4,
                y: target.bounds.y + target.bounds.height / 2,
            },
            false,
        );
        let dragged = runtime.dispatch_pointer_move(Point {
            x: track.x + track.width * 3 / 4,
            y: target.bounds.y + target.bounds.height / 2,
        });
        let released = runtime.dispatch_pointer_up(Point {
            x: track.x + track.width * 3 / 4,
            y: target.bounds.y + target.bounds.height / 2,
        });
        let left = runtime.dispatch_key(NativeViewKey::Left);
        let coarse_right = runtime.dispatch_key_with_shift(NativeViewKey::Right, true);

        assert!(pressed.handled);
        assert_eq!(pressed.slider_value, Some(25.0));
        assert!(pressed.slider_drag_active);
        assert_eq!(dragged.slider_value, Some(75.0));
        assert!(dragged.slider_value_changed);
        assert!(!released.slider_drag_active);
        assert_eq!(left.slider_value, Some(70.0));
        assert_eq!(coarse_right.slider_value, Some(100.0));
        assert_eq!(runtime.widget_slider_state(slider_id), Some((100.0, range)));
    }

    #[cfg(feature = "radio")]
    #[test]
    fn native_view_runtime_selects_radio_from_pointer_and_keyboard() {
        #[derive(Clone)]
        enum Msg {
            Choose(usize),
        }

        let first = crate::WidgetId::new(82);
        let second = crate::WidgetId::new(83);
        let builder = native_window("Platform Radio")
            .size(360, 220)
            .stateful_view(
                0usize,
                move |selected| {
                    crate::column([
                        crate::radio_button("Balanced", *selected == 0)
                            .id(first)
                            .height(Dp::new(36.0))
                            .on_choose(Msg::Choose(0)),
                        crate::radio_button("Performance", *selected == 1)
                            .id(second)
                            .height(Dp::new(36.0))
                            .on_choose(Msg::Choose(1)),
                    ])
                },
                |selected, message, _cx| match message {
                    Msg::Choose(index) => *selected = index,
                },
            );
        let second_bounds = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(second))
            .expect("second radio should have pointer geometry")
            .bounds;
        let mut runtime = builder.native_view_input_runtime();

        let selected = runtime.dispatch_pointer_click(Point {
            x: second_bounds.x + 8,
            y: second_bounds.y + second_bounds.height / 2,
        });
        let keyboard = runtime.dispatch_key(NativeViewKey::Space);
        let moved = runtime.dispatch_key(NativeViewKey::Up);
        let boundary = runtime.dispatch_key(NativeViewKey::Up);
        let horizontal = runtime.dispatch_key(NativeViewKey::Left);
        let focus_only = runtime.dispatch_key_with_modifiers(NativeViewKey::Down, false, true);
        let tabbed = runtime.dispatch_key(NativeViewKey::Tab);

        assert!(selected.handled);
        assert!(selected.radio_selection_changed);
        assert!(keyboard.handled);
        assert!(keyboard.radio_selection_changed);
        assert!(moved.handled);
        assert!(moved.radio_selection_changed);
        assert!(moved.radio_keyboard_selection_changed);
        assert_eq!(moved.focused_widget, Some(first.0));
        assert!(boundary.handled);
        assert!(!boundary.radio_selection_changed);
        assert!(horizontal.handled);
        assert!(!horizontal.radio_selection_changed);
        assert!(focus_only.handled);
        assert!(focus_only.radio_keyboard_focus_only);
        assert!(!focus_only.radio_selection_changed);
        assert_eq!(focus_only.focused_widget, Some(second.0));
        assert!(tabbed.handled);
        assert_eq!(tabbed.focused_widget, Some(first.0));
        assert_eq!(runtime.widget_checked_value(first), Some(true));
        assert_eq!(runtime.widget_checked_value(second), Some(false));
    }

    #[cfg(all(feature = "tabs", feature = "label"))]
    #[test]
    fn native_view_runtime_routes_platform_tab_pointer_focus_and_selection() {
        #[derive(Clone)]
        enum Msg {
            Selected(crate::ZsTabId),
        }

        let tab_view_id = crate::WidgetId::new(90);
        let general = crate::ZsTabId::new(91);
        let advanced = crate::ZsTabId::new(92);
        let builder = native_window("Platform Tabs").size(420, 260).stateful_view(
            general,
            move |selected| {
                crate::tab_view(
                    [
                        crate::ZsTabItem::new(general, "General", crate::text("General content")),
                        crate::ZsTabItem::new(
                            advanced,
                            "Advanced",
                            crate::text("Advanced content"),
                        ),
                    ],
                    Some(*selected),
                )
                .id(tab_view_id)
                .on_tab_select(Msg::Selected)
            },
            |selected, message, _cx| match message {
                Msg::Selected(tab) => *selected = tab,
            },
        );
        let interaction = builder
            .native_view_interaction_plan()
            .expect("tabs should expose header interaction geometry");
        let first = interaction
            .hit_target_for_widget(crate::WidgetId(general.0))
            .expect("first tab should be interactive");
        let second = interaction
            .hit_target_for_widget(crate::WidgetId(advanced.0))
            .expect("second tab should be interactive");
        let second_point = Point {
            x: second.bounds.x + second.bounds.width / 2,
            y: second.bounds.y + second.bounds.height / 2,
        };
        let mut runtime = builder.native_view_input_runtime();

        let hovered = runtime.dispatch_pointer_move(second_point);
        let pressed = runtime.dispatch_pointer_down(second_point, false);
        let selected = runtime.dispatch_pointer_up(second_point);

        assert!(hovered.pointer_visual_changed);
        assert!(pressed.pointer_visual_changed);
        assert!(selected.tab_selection_changed);
        assert_eq!(selected.focused_widget, Some(advanced.0));
        assert_eq!(
            runtime
                .live_view
                .as_ref()
                .and_then(|view| view.widget_tab_header_state(crate::WidgetId(advanced.0)))
                .map(|state| state.selected),
            Some(true)
        );
        runtime.dispatch_pointer_down(second_point, false);
        let reselected = runtime.dispatch_pointer_up(second_point);
        assert!(!reselected.tab_selection_changed);
        assert_eq!(reselected.message_count, 0);

        let left = runtime.dispatch_key(NativeViewKey::Left);
        assert!(left.handled);
        assert_eq!(left.focused_widget, Some(general.0));

        match crate::ZsTabPlatformStyle::current() {
            crate::ZsTabPlatformStyle::Windows => {
                assert!(left.tab_keyboard_focus_only);
                assert!(!left.tab_selection_changed);
                let activated = runtime.dispatch_key(NativeViewKey::Space);
                assert!(activated.tab_selection_changed);
                assert!(activated.tab_keyboard_selection_changed);
                let cycled = runtime.dispatch_key_with_modifiers(NativeViewKey::Tab, false, true);
                assert!(cycled.handled);
                assert!(cycled.tab_selection_changed);
                assert_eq!(cycled.focused_widget, Some(advanced.0));
            }
            crate::ZsTabPlatformStyle::Macos => {
                assert!(left.tab_selection_changed);
                assert!(left.tab_keyboard_selection_changed);
                let right = runtime.dispatch_key(NativeViewKey::Right);
                assert!(right.tab_selection_changed);
                assert_eq!(right.focused_widget, Some(advanced.0));
            }
            crate::ZsTabPlatformStyle::Gtk => {
                assert!(left.tab_keyboard_focus_only);
                assert!(!left.tab_selection_changed);
                let activated = runtime.dispatch_key(NativeViewKey::Space);
                assert!(activated.tab_selection_changed);
                let cycled =
                    runtime.dispatch_key_with_modifiers(NativeViewKey::PageDown, false, true);
                assert!(cycled.handled);
                assert!(cycled.tab_selection_changed);
                assert_eq!(cycled.focused_widget, Some(advanced.0));
            }
        }
        assert!(first.bounds.x < second.bounds.x);
    }

    #[cfg(feature = "tabs")]
    #[test]
    fn tab_cycle_shortcuts_follow_each_platform_contract() {
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Windows,
                NativeViewKey::Tab,
                false,
                true,
            ),
            Some(1)
        );
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Windows,
                NativeViewKey::Tab,
                true,
                true,
            ),
            Some(-1)
        );
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Gtk,
                NativeViewKey::PageUp,
                false,
                true,
            ),
            Some(-1)
        );
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Gtk,
                NativeViewKey::PageDown,
                false,
                true,
            ),
            Some(1)
        );
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Gtk,
                NativeViewKey::Tab,
                false,
                true,
            ),
            None
        );
        assert_eq!(
            native_tab_cycle_offset(
                crate::ZsTabPlatformStyle::Macos,
                NativeViewKey::Tab,
                false,
                true,
            ),
            None
        );
    }

    #[cfg(feature = "combo")]
    #[test]
    fn native_view_runtime_selects_combo_overlay_and_routes_keyboard_state() {
        #[derive(Clone)]
        enum Msg {
            Selected(usize),
            Expanded(bool),
        }
        struct State {
            selected: Option<usize>,
            expanded: bool,
        }

        let combo_id = crate::WidgetId::new(84);
        let builder = native_window("Platform Combo")
            .size(360, 240)
            .stateful_view(
                State {
                    selected: Some(0),
                    expanded: true,
                },
                move |state| {
                    crate::column([
                        crate::combo_box(["Balanced", "Fast", "Quiet"], state.selected)
                            .id(combo_id)
                            .height(Dp::new(36.0))
                            .expanded(state.expanded)
                            .on_select(Msg::Selected)
                            .on_expanded_change(Msg::Expanded),
                        crate::spacer(),
                    ])
                },
                |state, message, _cx| match message {
                    Msg::Selected(index) => state.selected = Some(index),
                    Msg::Expanded(expanded) => state.expanded = expanded,
                },
            );
        let option = builder
            .native_view_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind == crate::ViewHitTargetKind::ComboBoxOption { index: 1 }
                })
            })
            .expect("expanded combo should expose option hit geometry");
        let header = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(combo_id))
            .expect("combo should expose header hit geometry");
        let mut runtime = builder.native_view_input_runtime();

        let dismissed = runtime.dispatch_pointer_down(Point { x: 340, y: 220 }, false);
        assert!(dismissed.handled);
        assert!(dismissed.combo_expanded_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(0), 3, false))
        );
        runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + 8,
            y: header.bounds.y + header.bounds.height / 2,
        });

        let pointer = runtime.dispatch_pointer_click(Point {
            x: option.bounds.x + 8,
            y: option.bounds.y + option.bounds.height / 2,
        });
        assert!(pointer.handled);
        assert!(pointer.combo_selection_changed);
        assert!(pointer.combo_expanded_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(1), 3, false))
        );

        let opened = runtime.dispatch_key(NativeViewKey::Space);
        assert!(opened.handled);
        assert!(opened.combo_expanded_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(1), 3, true))
        );

        let selected = runtime.dispatch_key(NativeViewKey::Down);
        assert!(selected.handled);
        assert!(selected.combo_selection_changed);
        assert!(selected.combo_keyboard_selection_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(2), 3, false))
        );

        let typed = runtime.dispatch_text_input("B");
        assert!(typed.handled);
        assert!(typed.combo_type_ahead_matched);
        assert!(typed.combo_selection_changed);
        assert!(typed.combo_keyboard_selection_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(0), 3, false))
        );

        runtime.dispatch_key(NativeViewKey::Space);
        let blurred = runtime.blur_focus();
        assert!(blurred.handled);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(0), 3, false))
        );
        runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + 12,
            y: header.bounds.y + header.bounds.height / 2,
        });
        let closed = runtime.dispatch_key(NativeViewKey::Escape);
        assert!(closed.handled);
        assert!(closed.combo_expanded_changed);
        assert_eq!(
            runtime.widget_combo_state(combo_id),
            Some((Some(0), 3, false))
        );
    }

    #[cfg(feature = "combo")]
    #[test]
    fn native_view_runtime_routes_wheel_input_to_long_combo_popup() {
        let combo_id = crate::WidgetId::new(92);
        let options = (0..30)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let builder = native_window("Scrollable Combo")
            .size(320, 220)
            .ui_command_view(crate::column([
                crate::combo_box::<_, crate::UiCommand>(options, Some(0))
                    .id(combo_id)
                    .height(Dp::new(36.0))
                    .expanded(true),
                crate::spacer(),
            ]));
        let option = builder
            .native_view_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind == crate::ViewHitTargetKind::ComboBoxOption { index: 0 }
                })
            })
            .expect("long combo should expose its first visible option");
        let mut runtime = builder.native_view_input_runtime();

        let report = runtime.dispatch_pointer_scroll(
            Point {
                x: option.bounds.x + 8,
                y: option.bounds.y + option.bounds.height / 2,
            },
            Dp::new(48.0),
        );

        assert!(report.handled);
        assert!(report.combo_scrolled);
        assert!(report.redraw_plan.is_some());
        assert_eq!(
            runtime
                .current_interaction_plan()
                .and_then(|plan| plan.combo_visible_option_range(combo_id))
                .map(|range| range.start),
            Some(1)
        );

        let first_scrolled_row = runtime
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind == crate::ViewHitTargetKind::ComboBoxOption { index: 1 }
                })
            })
            .expect("scrolled combo should expose the next global option");
        let upward = runtime.dispatch_pointer_scroll(
            Point {
                x: first_scrolled_row.bounds.x + 8,
                y: first_scrolled_row.bounds.y + first_scrolled_row.bounds.height / 2,
            },
            Dp::new(-48.0),
        );
        assert!(upward.combo_scrolled);
        assert_eq!(
            runtime
                .current_interaction_plan()
                .and_then(|plan| plan.combo_visible_option_range(combo_id))
                .map(|range| range.start),
            Some(0)
        );
    }

    #[cfg(feature = "combo")]
    #[test]
    fn native_combo_type_ahead_accumulates_resets_and_cycles() {
        let widget = crate::WidgetId::new(90);
        let started = Instant::now();
        let mut state = NativeComboTypeAheadState::default();

        let first = state
            .push_text(widget, "Q", started)
            .expect("first printable character should start a query");
        assert_eq!(first.text, "q");
        assert_eq!(first.match_start_after(Some(2), 3), Some(2));

        let continued = state
            .push_text(widget, "u", started + Duration::from_millis(200))
            .expect("nearby characters should continue the query");
        assert_eq!(continued.text, "qu");
        assert_eq!(continued.match_start_after(Some(0), 3), Some(2));

        let longer = state
            .push_text(widget, "q", started + Duration::from_millis(400))
            .expect("different nearby characters should remain in the query");
        assert_eq!(longer.text, "quq");

        state.reset();
        state.push_text(widget, "q", started);
        let repeated = state
            .push_text(widget, "Q", started + Duration::from_millis(200))
            .expect("repeated characters should keep a cycling query");
        assert_eq!(repeated.text, "q");
        assert_eq!(repeated.match_start_after(Some(0), 3), Some(0));

        let restarted = state
            .push_text(widget, "B", started + Duration::from_millis(1_500))
            .expect("expired queries should restart");
        assert_eq!(restarted.text, "b");
        assert_eq!(restarted.match_start_after(Some(1), 3), Some(1));
        assert_eq!(
            state.push_text(widget, " \r\n", started + Duration::from_millis(1_600)),
            None
        );
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn native_view_runtime_opens_selects_and_navigates_date_picker() {
        #[derive(Clone)]
        enum Msg {
            Changed(crate::ZsDate),
        }

        let widget = crate::WidgetId::new(85);
        let initial = crate::ZsDate::new(2026, 7, 13).unwrap();
        let builder = native_window("Platform DatePicker")
            .size(520, 480)
            .stateful_view(
                initial,
                move |value| {
                    crate::date_picker(*value)
                        .id(widget)
                        .height(Dp::new(32.0))
                        .on_date_change(Msg::Changed)
                },
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let header = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .expect("date picker header should have hit geometry");
        let mut runtime = builder.native_view_input_runtime();

        let opened = runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + 12,
            y: header.bounds.y + header.bounds.height / 2,
        });
        assert!(opened.handled);
        assert!(opened.redraw_plan.is_some());
        assert!(
            runtime
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );

        let next = crate::ZsDate::new(2026, 7, 14).unwrap();
        let day = runtime
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.into_iter().find(|target| {
                    target.kind == crate::ViewHitTargetKind::DatePickerDay { date: next }
                })
            })
            .expect("expanded date picker should expose day hit geometry");
        let day_point = Point {
            x: day.bounds.x + day.bounds.width / 2,
            y: day.bounds.y + day.bounds.height / 2,
        };
        let hovered = runtime.dispatch_pointer_move(day_point);
        assert!(hovered.handled);
        assert!(hovered.pointer_visual_changed);
        assert!(hovered
            .redraw_plan
            .is_some_and(|plan| plan.commands.iter().any(|command| matches!(
                command,
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::PrimaryText,
                        alpha: 14,
                    },
                    ..
                }
            ))));
        let pressed = runtime.dispatch_pointer_down(day_point, false);
        assert!(pressed.pointer_visual_changed);
        assert!(pressed
            .redraw_plan
            .is_some_and(|plan| plan.commands.iter().any(|command| matches!(
                command,
                NativeDrawCommand::RoundFill {
                    fill: NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::PrimaryText,
                        alpha: 28,
                    },
                    ..
                }
            ))));
        let left = runtime.dispatch_pointer_leave();
        assert!(left.pointer_visual_changed);
        assert!(left.redraw_plan.is_some());

        let selected = runtime.dispatch_pointer_click(day_point);
        assert!(selected.handled);
        assert_eq!(selected.message_count, 1);
        assert_eq!(
            runtime
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(next)
        );

        runtime.dispatch_key(NativeViewKey::Space);
        let blurred = runtime.blur_focus();
        assert!(blurred.handled);
        assert!(
            !runtime
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
        runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + 12,
            y: header.bounds.y + header.bounds.height / 2,
        });
        let closed = runtime.dispatch_key(NativeViewKey::Escape);
        assert!(closed.handled);
        assert!(
            !runtime
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
        let moved = runtime.dispatch_key(NativeViewKey::Right);
        assert!(moved.handled);
        assert_eq!(
            runtime
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsDate::new(2026, 7, 15).unwrap())
        );
    }

    #[cfg(all(feature = "label", feature = "textbox"))]
    #[test]
    fn native_view_runtime_keeps_ime_preedit_provisional_until_commit() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
        }
        struct State {
            value: String,
        }

        let textbox_id = crate::WidgetId::new(77);
        let builder = native_window("Platform IME").size(360, 220).stateful_view(
            State {
                value: "A".to_string(),
            },
            move |state| {
                crate::textbox(&state.value)
                    .id(textbox_id)
                    .on_change(Msg::Changed)
            },
            |state, message, _cx| match message {
                Msg::Changed(value) => state.value = value,
            },
        );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have an IME target");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        });

        let preedit = runtime.dispatch_ime_preedit("中文", Some((1, 1)));

        assert!(preedit.handled);
        assert_eq!(preedit.message_count, 0);
        assert_eq!(preedit.ime_preedit_text.as_deref(), Some("中文"));
        assert_eq!(preedit.ime_selection, Some((1, 1)));
        assert!(preedit.ime_caret_rect.is_some());
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("A"));
        assert!(preedit.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::Text(text) if text.text == "A中文")
            }) && plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. } if *rect == target.bounds)
            })
        }));

        let committed = runtime.dispatch_ime_commit("中文");

        assert!(committed.handled);
        assert_eq!(committed.message_count, 1);
        assert_eq!(committed.ime_preedit_text, None);
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("A中文"));
        assert!(committed.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::Text(text) if text.text == "A中文")
            }) && !plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::StrokeRect { rect, width: 2, .. } if *rect == target.bounds)
            })
        }));
    }

    #[cfg(all(feature = "label", feature = "textbox"))]
    #[test]
    fn native_view_runtime_ime_commit_replaces_preserved_selection() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
        }

        let textbox_id = crate::WidgetId::new(79);
        let builder = native_window("Platform IME Selection")
            .size(360, 220)
            .stateful_view(
                "A中文Z".to_string(),
                move |value| crate::textbox(value).id(textbox_id).on_change(Msg::Changed),
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have an IME selection target");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(crate::Point {
            x: target.bounds.x + 1,
            y: target.bounds.y + 1,
        });
        runtime.dispatch_key(NativeViewKey::Home);
        runtime.dispatch_key(NativeViewKey::Right);
        runtime.dispatch_key_with_shift(NativeViewKey::Right, true);
        runtime.dispatch_key_with_shift(NativeViewKey::Right, true);

        let preedit = runtime.dispatch_ime_preedit("🙂", Some((1, 1)));

        assert_eq!(
            runtime.focused_text_input_value().as_deref(),
            Some("A中文Z")
        );
        assert!(preedit.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "A🙂Z"),
            )
        }));

        let committed = runtime.dispatch_ime_commit("🙂");

        assert_eq!(committed.text_selection, Some((2, 2)));
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("A🙂Z"));
    }

    #[cfg(all(feature = "label", feature = "textbox"))]
    #[test]
    fn native_view_runtime_relayouts_live_view_and_input_geometry_on_resize() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
        }

        let textbox_id = crate::WidgetId::new(78);
        let builder = native_window("Platform Resize")
            .size(240, 120)
            .stateful_view(
                String::new(),
                move |value| crate::textbox(value).id(textbox_id).on_change(Msg::Changed),
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let mut runtime = builder.native_view_input_runtime();
        let initial = runtime
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have initial resize geometry");
        runtime.dispatch_pointer_click(Point {
            x: initial.bounds.x + 1,
            y: initial.bounds.y + 1,
        });
        runtime.dispatch_ime_preedit("中", Some((1, 1)));

        let resized = runtime.set_surface(
            Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 360,
            },
            Dpi::standard(),
        );
        let resized_target = runtime
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(textbox_id))
            .expect("textbox should have resized geometry");

        assert!(resized.handled);
        assert!(resized.surface_changed);
        assert_ne!(initial.bounds, resized_target.bounds);
        assert_eq!(resized.ime_preedit_text.as_deref(), Some("中"));
        assert_eq!(resized.focused_widget, Some(textbox_id.0));
        assert!(resized.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "中"),
            )
        }));

        let unchanged = runtime.set_surface(
            Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 360,
            },
            Dpi::standard(),
        );
        assert!(!unchanged.surface_changed);
        assert!(unchanged.redraw_plan.is_none());
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn native_view_runtime_relayouts_static_command_view_on_resize() {
        let button_id = crate::WidgetId::new(79);
        let builder = native_window("Static Resize")
            .size(200, 100)
            .ui_command_view(
                crate::button("Resize")
                    .id(button_id)
                    .on_click(UiCommand::app(crate::CommandId("zsui.resize.static"))),
            );
        let mut runtime = builder.native_view_input_runtime();
        let initial = runtime
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(button_id))
            .expect("button should have initial static geometry");

        let report = runtime.set_surface(
            Rect {
                x: 0,
                y: 0,
                width: 520,
                height: 240,
            },
            Dpi::standard(),
        );
        let resized = runtime
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(button_id))
            .expect("button should have resized static geometry");

        assert!(report.surface_changed);
        assert!(report.redraw_plan.is_some());
        assert_ne!(initial.bounds, resized.bounds);
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
        assert!(options.native_view_key_downs.is_empty());
        assert!(options.native_view_scroll_inputs.is_empty());
        assert!(options.native_view_inputs.is_empty());
        assert_eq!(report.native_view_focus_count, 0);
        assert_eq!(report.native_view_focus_visual_count, 0);
        assert_eq!(report.native_view_focus_traversal_count, 0);
        assert_eq!(report.native_view_text_input_count, 0);
        assert_eq!(report.native_view_text_navigation_count, 0);
        assert_eq!(report.native_view_text_selection_change_count, 0);
        assert_eq!(report.native_view_text_caret, None);
        assert_eq!(report.native_view_pointer_down_count, 0);
        assert_eq!(report.native_view_pointer_move_count, 0);
        assert_eq!(report.native_view_pointer_up_count, 0);
        assert_eq!(report.native_view_text_drag_count, 0);
        assert_eq!(report.native_view_slider_value_change_count, 0);
        assert_eq!(report.native_view_slider_keyboard_change_count, 0);
        assert_eq!(report.native_view_slider_drag_count, 0);
        assert_eq!(report.native_view_radio_selection_count, 0);
        assert_eq!(report.native_view_radio_keyboard_selection_count, 0);
        assert_eq!(report.native_view_radio_keyboard_focus_only_count, 0);
        assert_eq!(report.native_view_combo_scroll_count, 0);
        assert_eq!(report.native_view_toggle_count, 0);
        assert_eq!(report.native_view_selection_count, 0);
        assert_eq!(report.native_view_keyboard_selection_count, 0);
        assert_eq!(report.native_view_key_down_count, 0);
        assert_eq!(report.native_view_keyboard_activation_count, 0);
        assert_eq!(report.native_view_unhandled_key_count, 0);
        assert_eq!(report.native_view_scroll_count, 0);
        assert_eq!(report.native_view_unhandled_scroll_count, 0);
        assert!(!report.visible_window_was_created());
    }

    #[test]
    fn native_window_smoke_options_preserve_pointer_drag_sequence() {
        let start = Point { x: 16, y: 24 };
        let end = Point { x: 48, y: 24 };

        let options = NativeWindowSmokeRunOptions::quick().native_view_drag(start, end);

        assert_eq!(
            options.native_view_inputs,
            vec![NativeViewSmokeInput::Drag { start, end }]
        );
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
    fn native_window_runtime_driver_executes_typed_app_commands() {
        let mut driver = NativeWindowRuntimeDriver::new();
        let startup = NativeRuntimeStartupRequest {
            app_name: "Command Example".to_string(),
            main_window: crate::NativeMainWindowRequest::from_zsui_window(&Window::new(
                "Command Example",
            )),
            status_item_tooltip: None,
            status_item: None,
            settings_pages: Vec::new(),
        };
        assert!(matches!(
            driver.start_runtime(startup),
            NativeRuntimeStartupResult::Started(_)
        ));

        let events = driver.execute_app_command(Command::HideMainWindow).unwrap();

        assert_eq!(
            events,
            vec![AppEvent::WindowHidden {
                window: WindowId(1)
            }]
        );
        assert_eq!(driver.app_commands(), &[Command::HideMainWindow]);
        assert_eq!(driver.report().app_command_count, 1);
        assert_eq!(driver.report().app_command_names, vec!["hide_main_window"]);
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
