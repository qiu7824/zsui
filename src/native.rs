use serde::Serialize;

#[cfg(feature = "combo")]
use std::time::{Duration, Instant};

#[cfg(feature = "workbench")]
use crate::workbench::ZsWorkbenchSpec;

use crate::native_input_visuals::{
    decorate_native_focus_ring, decorate_native_text_edit_visuals_in_viewport_with_backend,
    move_native_text_selection_horizontally_with_backend,
    native_text_drag_viewport_for_point_with_backend,
    native_text_first_visible_row_for_caret_with_backend,
    native_text_horizontal_scroll_for_caret_with_backend,
    native_text_index_for_point_in_viewport_with_backend,
    native_text_index_for_vertical_move_with_backend,
    native_text_index_for_vertical_page_move_with_backend,
    native_text_scroll_visual_rows_with_backend,
    native_text_visual_geometry_in_viewport_with_backend, native_text_visual_target,
    native_text_wheel_row_delta, NativeTextVisualDirection, NativeTextVisualHorizontalDirection,
};
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
use crate::native_input_visuals::{
    decorate_native_pointer_visuals, native_pointer_visual_key, NativePointerVisualKey,
};
#[cfg(feature = "textbox")]
use crate::native_text_edit::{apply_text_edit_command, NativeTextHistory};
use crate::native_text_edit::{
    apply_text_input, char_to_byte_index, move_selection, move_selection_to, set_pointer_selection,
    snap_grapheme_index, NativeTextDragState, NativeTextEditState, NativeTextMovement,
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
        live_view_runtime, live_view_runtime_with_app_commands, AppCx, SharedLiveViewRuntime, View,
        ViewEvent, ViewEventCx, ViewInteractionPlan, ViewLayoutCx, ViewNode, ViewPaintCx,
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

/// Controls how much state a native window retains while it cannot be seen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeWindowResourcePolicy {
    /// Keeps the stateful View tree and its transient input resources alive.
    #[default]
    RetainView,
    /// Drops the stateful View tree, draw/hit data and transient input caches
    /// while the window is hidden or minimized, then rebuilds it from retained
    /// application state when the window becomes visible again.
    ReleaseViewWhenHidden,
}

impl NativeWindowResourcePolicy {
    pub const fn releases_view_when_hidden(self) -> bool {
        matches!(self, Self::ReleaseViewWhenHidden)
    }
}

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
        crate::desktop_runtime::run_smoke_event_loop(
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
    Move(Point),
    Click(Point),
    Drag { start: Point, end: Point },
    Text(String),
    KeyDown(NativeViewKey),
    Scroll { point: Point, delta_y: i32 },
    WindowCloseRequest,
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

    pub fn native_view_pointer_move(mut self, point: Point) -> Self {
        self.native_view_inputs
            .push(NativeViewSmokeInput::Move(point));
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

    pub fn native_window_close_request(mut self) -> Self {
        self.native_view_inputs
            .push(NativeViewSmokeInput::WindowCloseRequest);
        self
    }
}

impl Default for NativeWindowSmokeRunOptions {
    fn default() -> Self {
        Self::quick()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NativeViewCaptureEvidence {
    pub platform: &'static str,
    pub backend: &'static str,
    pub display_server: Option<&'static str>,
    pub logical_width: u32,
    pub logical_height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub scale_factor: f64,
    pub typography_scale: f32,
    pub typography: crate::NativeTypographyProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NativeWindowSmokeRunReport {
    pub requested_window_count: usize,
    pub created_window_count: usize,
    pub window_menu_requested_count: usize,
    pub window_menu_attached_count: usize,
    pub window_menu_native_command_count: usize,
    pub window_menu_command_routed: bool,
    pub window_menu_command_error: Option<String>,
    pub window_menu_surface_created: bool,
    pub window_menu_surface_height: u32,
    pub window_menu_surface_open_at_capture: bool,
    pub close_requested_count: usize,
    pub native_view_window_close_request_count: usize,
    pub native_view_window_close_veto_count: usize,
    pub auto_close_after_ms: u64,
    pub exited_by_auto_close: bool,
    pub startup_error: Option<String>,
    pub screenshot_file: Option<String>,
    pub screenshot_captured: bool,
    pub screenshot_error: Option<String>,
    pub native_view_capture: Option<NativeViewCaptureEvidence>,
    pub process_memory_during_runtime: Option<crate::NativeProofProcessMemoryEvidence>,
    pub native_accessibility_backend: Option<&'static str>,
    pub native_accessibility_node_count: usize,
    pub native_accessibility_action_count: usize,
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
    pub native_view_focused_widget: Option<u64>,
    pub native_view_text_input_count: usize,
    pub native_view_text_navigation_count: usize,
    pub native_view_text_selection_change_count: usize,
    pub native_view_text_caret: Option<usize>,
    pub native_view_text_edit_command_count: usize,
    pub native_view_text_clipboard_read_count: usize,
    pub native_view_text_clipboard_write_count: usize,
    pub native_view_text_undo_count: usize,
    pub native_view_text_edit_command_errors: Vec<String>,
    pub native_view_pointer_down_count: usize,
    pub native_view_pointer_move_count: usize,
    pub native_view_pointer_up_count: usize,
    pub native_view_pointer_visual_change_count: usize,
    pub native_view_text_drag_count: usize,
    pub native_view_text_drag_scroll_count: usize,
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
            window_menu_surface_created: false,
            window_menu_surface_height: 0,
            window_menu_surface_open_at_capture: false,
            close_requested_count: 0,
            native_view_window_close_request_count: 0,
            native_view_window_close_veto_count: 0,
            auto_close_after_ms: options.auto_close_after_ms,
            exited_by_auto_close: false,
            startup_error: None,
            screenshot_file: options.screenshot_file,
            screenshot_captured: false,
            screenshot_error: None,
            native_view_capture: None,
            process_memory_during_runtime: None,
            native_accessibility_backend: None,
            native_accessibility_node_count: 0,
            native_accessibility_action_count: 0,
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
            native_view_focused_widget: None,
            native_view_text_input_count: 0,
            native_view_text_navigation_count: 0,
            native_view_text_selection_change_count: 0,
            native_view_text_caret: None,
            native_view_text_edit_command_count: 0,
            native_view_text_clipboard_read_count: 0,
            native_view_text_clipboard_write_count: 0,
            native_view_text_undo_count: 0,
            native_view_text_edit_command_errors: Vec::new(),
            native_view_pointer_down_count: 0,
            native_view_pointer_move_count: 0,
            native_view_pointer_up_count: 0,
            native_view_pointer_visual_change_count: 0,
            native_view_text_drag_count: 0,
            native_view_text_drag_scroll_count: 0,
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
    typography_scale_per_mille: u16,
    text_shaping: crate::native_input_visuals::NativeTextShapingBackend,
    interaction_plan: Option<ViewInteractionPlan>,
    ui_command_view: Option<ViewNode<UiCommand>>,
    live_view: Option<SharedLiveViewRuntime>,
    resource_policy: NativeWindowResourcePolicy,
    view_suspended: bool,
    animation_epoch: Option<std::time::Instant>,
    focused_widget: Option<crate::WidgetId>,
    #[cfg(any(feature = "command-palette", feature = "dialog"))]
    modal_restore_focus: Option<crate::WidgetId>,
    #[cfg(feature = "tooltip")]
    tooltip: crate::tooltip::ZsTooltipRuntime,
    #[cfg(feature = "toast")]
    toast: crate::toast::ZsToastRuntime,
    text_edit: Option<NativeTextEditState>,
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
    pointer_hover: Option<NativePointerVisualKey>,
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
    pointer_pressed: Option<NativePointerVisualKey>,
    #[cfg(feature = "password-box")]
    password_peek: Option<crate::WidgetId>,
    ime_preedit: Option<NativeViewImePreedit>,
    window_close_request_command: Option<Command>,
    app_command_executor: Option<SharedAppCommandExecutor>,
    defer_app_command_execution: bool,
    pending_app_commands: Vec<Command>,
    ui_command_executor: Option<SharedUiCommandExecutor>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) enum NativeViewInputBackendSource {
    Live(SharedLiveViewRuntime),
    Static {
        interaction_plan: ViewInteractionPlan,
        ui_command_view: ViewNode<UiCommand>,
    },
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct NativeViewInputBackendAttachment {
    pub(crate) source: NativeViewInputBackendSource,
    pub(crate) resource_policy: NativeWindowResourcePolicy,
    pub(crate) window_close_request_command: Option<Command>,
    pub(crate) app_command_executor: Option<SharedAppCommandExecutor>,
    pub(crate) ui_command_executor: Option<SharedUiCommandExecutor>,
}

#[derive(Clone, PartialEq, Eq)]
struct NativeViewImePreedit {
    widget: crate::WidgetId,
    text: NativeViewImeText,
    selection: Option<(usize, usize)>,
    replacement: NativeTextSelection,
}

#[derive(Clone, PartialEq, Eq)]
enum NativeViewImeText {
    Plain(String),
    #[cfg(feature = "password-box")]
    Secure(crate::ZsPassword),
}

impl NativeViewImeText {
    fn as_str(&self) -> &str {
        match self {
            Self::Plain(text) => text,
            #[cfg(feature = "password-box")]
            Self::Secure(text) => text.as_str(),
        }
    }

    fn report_text(&self) -> String {
        match self {
            Self::Plain(text) => text.clone(),
            #[cfg(feature = "password-box")]
            Self::Secure(text) => crate::mask_password(text.as_str()),
        }
    }

    #[cfg(feature = "password-box")]
    fn is_secure(&self) -> bool {
        matches!(self, Self::Secure(_))
    }
}

impl std::fmt::Debug for NativeViewImeText {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(text) => formatter.debug_tuple("Plain").field(text).finish(),
            #[cfg(feature = "password-box")]
            Self::Secure(_) => formatter.write_str("Secure(<redacted>)"),
        }
    }
}

impl std::fmt::Debug for NativeViewImePreedit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeViewImePreedit")
            .field("widget", &self.widget)
            .field("text", &self.text)
            .field("selection", &self.selection)
            .field("replacement", &self.replacement)
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NativeViewInputDispatchReport {
    pub handled: bool,
    pub window_close_request_count: usize,
    pub window_close_veto_count: usize,
    pub surface_changed: bool,
    pub focus_visual_changed: bool,
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
    #[cfg(feature = "textbox")]
    pub text_edit_command_count: usize,
    #[cfg(feature = "textbox")]
    pub text_clipboard_read_count: usize,
    #[cfg(feature = "textbox")]
    pub text_clipboard_write_count: usize,
    #[cfg(feature = "textbox")]
    pub text_undo_count: usize,
    pub text_drag_active: bool,
    pub text_drag_scroll_count: usize,
    #[cfg(feature = "slider")]
    pub slider_value: Option<f32>,
    #[cfg(feature = "slider")]
    pub slider_value_changed: bool,
    #[cfg(feature = "slider")]
    pub slider_drag_active: bool,
    #[cfg(feature = "color-picker")]
    pub color_picker_value: Option<crate::Color>,
    #[cfg(feature = "color-picker")]
    pub color_picker_value_changed: bool,
    #[cfg(feature = "color-picker")]
    pub color_picker_channel_changed: bool,
    #[cfg(feature = "color-picker")]
    pub color_picker_expanded_changed: bool,
    #[cfg(feature = "color-picker")]
    pub color_picker_drag_active: bool,
    #[cfg(feature = "radio")]
    pub radio_selection_changed: bool,
    #[cfg(feature = "radio")]
    pub radio_keyboard_selection_changed: bool,
    #[cfg(feature = "radio")]
    pub radio_keyboard_focus_only: bool,
    #[cfg(feature = "auto-suggest")]
    pub auto_suggest_expanded_changed: bool,
    #[cfg(feature = "auto-suggest")]
    pub auto_suggest_highlight_changed: bool,
    #[cfg(feature = "auto-suggest")]
    pub auto_suggest_submitted: bool,
    #[cfg(feature = "auto-suggest")]
    pub auto_suggest_cleared: bool,
    #[cfg(feature = "tree")]
    pub tree_expansion_changed: bool,
    #[cfg(feature = "tree")]
    pub tree_selection_changed: bool,
    #[cfg(feature = "tree")]
    pub tree_invoked: bool,
    #[cfg(feature = "grid-view")]
    pub grid_view_selection_changed: bool,
    #[cfg(feature = "grid-view")]
    pub grid_view_invoked: bool,
    #[cfg(feature = "table")]
    pub table_sort_changed: bool,
    #[cfg(feature = "table")]
    pub table_selection_changed: bool,
    #[cfg(feature = "table")]
    pub table_invoked: bool,
    #[cfg(feature = "dialog")]
    pub content_dialog_focus_changed: bool,
    #[cfg(feature = "dialog")]
    pub content_dialog_responded: bool,
    #[cfg(feature = "command-palette")]
    pub command_palette_query_changed: bool,
    #[cfg(feature = "command-palette")]
    pub command_palette_highlight_changed: bool,
    #[cfg(feature = "command-palette")]
    pub command_palette_invoked: bool,
    #[cfg(feature = "command-palette")]
    pub command_palette_open_changed: bool,
    #[cfg(feature = "command-palette")]
    pub command_palette_cleared: bool,
    #[cfg(feature = "toast")]
    pub toast_focus_changed: bool,
    #[cfg(feature = "toast")]
    pub toast_responded: bool,
    #[cfg(feature = "info-bar")]
    pub info_bar_focus_changed: bool,
    #[cfg(feature = "info-bar")]
    pub info_bar_event: Option<crate::ZsInfoBarEvent>,
    #[cfg(feature = "teaching-tip")]
    pub teaching_tip_focus_changed: bool,
    #[cfg(feature = "teaching-tip")]
    pub teaching_tip_response: Option<crate::ZsTeachingTipResponse>,
    #[cfg(feature = "breadcrumb")]
    pub breadcrumb_focus_changed: bool,
    #[cfg(feature = "breadcrumb")]
    pub breadcrumb_expanded_changed: bool,
    #[cfg(feature = "breadcrumb")]
    pub breadcrumb_selection: Option<crate::ZsBreadcrumbId>,
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
        resource_policy: NativeWindowResourcePolicy,
        window_close_request_command: Option<Command>,
        app_command_executor: Option<SharedAppCommandExecutor>,
        ui_command_executor: Option<SharedUiCommandExecutor>,
    ) -> Self {
        #[cfg(feature = "toast")]
        let now = std::time::Instant::now();
        #[allow(unused_mut)]
        let mut runtime = Self {
            surface: Some(surface),
            dpi: Dpi::standard(),
            typography_scale_per_mille: crate::render_protocol::default_typography_scale_per_mille(
            ),
            text_shaping: crate::native_input_visuals::NativeTextShapingBackend::default(),
            interaction_plan,
            ui_command_view,
            live_view,
            resource_policy,
            view_suspended: false,
            animation_epoch: Some(std::time::Instant::now()),
            focused_widget: None,
            #[cfg(any(feature = "command-palette", feature = "dialog"))]
            modal_restore_focus: None,
            #[cfg(feature = "tooltip")]
            tooltip: crate::tooltip::ZsTooltipRuntime::default(),
            #[cfg(feature = "toast")]
            toast: crate::toast::ZsToastRuntime::default(),
            text_edit: None,
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
            pointer_hover: None,
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
            pointer_pressed: None,
            #[cfg(feature = "password-box")]
            password_peek: None,
            ime_preedit: None,
            window_close_request_command,
            app_command_executor,
            defer_app_command_execution: false,
            pending_app_commands: Vec::new(),
            ui_command_executor,
        };
        runtime.reconcile_modal_focus(&mut NativeViewInputDispatchReport::default());
        #[cfg(feature = "toast")]
        runtime.sync_toast_runtime(now);
        runtime
    }

    pub(crate) fn suspend_view_when_hidden(&mut self) -> bool {
        if self.view_suspended || !self.resource_policy.releases_view_when_hidden() {
            return false;
        }
        let Some(runtime) = self.live_view.clone() else {
            return false;
        };
        if !runtime.suspend() {
            return false;
        }
        self.view_suspended = true;
        self.interaction_plan = None;
        self.animation_epoch = None;
        self.focused_widget = None;
        #[cfg(any(feature = "command-palette", feature = "dialog"))]
        {
            self.modal_restore_focus = None;
        }
        self.text_edit = None;
        self.text_drag = None;
        self.ime_preedit = None;
        #[cfg(feature = "textbox")]
        {
            self.text_history = NativeTextHistory::default();
            self.processing_text_edit_commands = false;
        }
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        #[cfg(feature = "color-picker")]
        {
            self.color_picker_drag = None;
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
            self.pointer_hover = None;
            self.pointer_pressed = None;
        }
        #[cfg(feature = "password-box")]
        {
            self.password_peek = None;
        }
        #[cfg(feature = "tooltip")]
        {
            self.tooltip = crate::tooltip::ZsTooltipRuntime::default();
        }
        #[cfg(feature = "toast")]
        {
            self.toast = crate::toast::ZsToastRuntime::default();
        }
        self.text_shaping.release_idle_memory();
        true
    }

    pub(crate) fn resume_view_when_visible(&mut self) -> Option<NativeDrawPlan> {
        if !self.view_suspended {
            return None;
        }
        let runtime = self.live_view.clone()?;
        let update = runtime.resume();
        self.view_suspended = false;
        self.animation_epoch = Some(std::time::Instant::now());
        self.interaction_plan = Some(runtime.interaction_plan());
        self.reconcile_modal_focus(&mut NativeViewInputDispatchReport::default());
        #[cfg(feature = "toast")]
        self.sync_toast_runtime(std::time::Instant::now());
        update
            .redraw
            .then(|| self.compose_input_visuals(runtime.draw_plan()))
    }
    pub(crate) fn set_text_shaping_backend(
        &mut self,
        backend: crate::native_input_visuals::NativeTextShapingBackend,
    ) {
        self.text_shaping = backend;
    }

    pub(crate) fn hit_target_count(&self) -> usize {
        self.current_interaction_plan()
            .as_ref()
            .map(ViewInteractionPlan::hit_target_count)
            .unwrap_or(0)
    }

    pub(crate) fn current_interaction_plan(&self) -> Option<ViewInteractionPlan> {
        self.live_view
            .as_ref()
            .map(SharedLiveViewRuntime::interaction_plan)
            .or_else(|| self.interaction_plan.clone())
    }

    pub(crate) const fn focused_widget(&self) -> Option<crate::WidgetId> {
        self.focused_widget
    }

    pub(crate) fn dispatch_accessibility_focus(
        &mut self,
        widget: crate::WidgetId,
    ) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport::default();
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|plan| plan.focus_target_for_widget(widget))
        else {
            return report;
        };
        report.handled = true;
        self.focus_target(target, &mut report);
        report
    }

    #[cfg(feature = "text-input-core")]
    pub(crate) fn dispatch_accessibility_set_value(
        &mut self,
        widget: crate::WidgetId,
        value: &str,
    ) -> NativeViewInputDispatchReport {
        let _ = self.dispatch_accessibility_focus(widget);
        let Some(target) = self.focused_text_input_target() else {
            return NativeViewInputDispatchReport::default();
        };
        let current = self.widget_text_value(widget).unwrap_or_default();
        let mut state = NativeTextEditState::at_end(widget, &current);
        state.selection = NativeTextSelection {
            anchor: 0,
            caret: current.chars().count(),
        };
        self.text_edit = Some(state);
        let mut report = self.dispatch_text_input(value);
        report.handled = true;
        report.focused_widget = Some(target.widget.0);
        report
    }

    fn reconcile_modal_focus(&mut self, report: &mut NativeViewInputDispatchReport) {
        #[cfg(any(feature = "command-palette", feature = "dialog"))]
        {
            if let Some(modal) = self
                .current_interaction_plan()
                .and_then(|plan| plan.modal_focus_target())
            {
                if self.focused_widget == Some(modal.widget) {
                    return;
                }
                if self.modal_restore_focus.is_none() {
                    self.modal_restore_focus = self.focused_widget.filter(|widget| {
                        *widget != modal.widget
                            && self
                                .current_interaction_plan()
                                .is_some_and(|plan| plan.hit_target_for_widget(*widget).is_some())
                    });
                }
                self.focused_widget = Some(modal.widget);
                self.ime_preedit = None;
                self.text_drag = None;
                #[cfg(feature = "combo")]
                self.combo_type_ahead.reset();
                #[cfg(feature = "slider")]
                {
                    self.slider_drag = None;
                }
                #[cfg(feature = "color-picker")]
                {
                    self.color_picker_drag = None;
                }
                #[cfg(feature = "password-box")]
                {
                    self.password_peek = None;
                }
                report.focus_visual_changed = true;
                report.focused_widget = Some(modal.widget.0);
                return;
            }

            let Some(previous) = self.modal_restore_focus.take() else {
                return;
            };
            let restored = self
                .current_interaction_plan()
                .and_then(|plan| plan.focus_target_for_widget(previous));
            self.focused_widget = restored.map(|target| target.widget);
            self.ime_preedit = None;
            self.text_drag = None;
            report.focus_visual_changed = true;
            report.focused_widget = self.focused_widget.map(|widget| widget.0);
        }
        #[cfg(not(any(feature = "command-palette", feature = "dialog")))]
        let _ = report;
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

    fn typography_scale(&self) -> f32 {
        f32::from(if self.typography_scale_per_mille == 0 {
            crate::render_protocol::default_typography_scale_per_mille()
        } else {
            self.typography_scale_per_mille
        }) / 1_000.0
    }

    pub(crate) fn set_typography_scale(&mut self, scale: f32) -> Option<NativeDrawPlan> {
        let scale_per_mille = crate::render_protocol::normalize_typography_scale_per_mille(scale);
        if self.typography_scale_per_mille == scale_per_mille {
            return None;
        }
        self.typography_scale_per_mille = scale_per_mille;
        self.text_shaping.release_idle_memory();
        let scale = self.typography_scale();
        let plan = if let Some(runtime) = &self.live_view {
            runtime.set_typography_scale(scale);
            if self.view_suspended {
                return None;
            }
            self.interaction_plan = Some(runtime.interaction_plan());
            runtime.draw_plan()
        } else {
            let surface = self.surface?;
            let view = self.ui_command_view.as_mut()?;
            let mut layout_cx = ViewLayoutCx::new(surface, self.dpi).with_typography_scale(scale);
            view.layout(&mut layout_cx);
            self.interaction_plan = Some(view.interaction_plan());
            let mut paint_cx = ViewPaintCx::new(self.dpi);
            paint_cx.set_typography_scale(scale);
            view.paint(&mut paint_cx);
            paint_cx.into_plan()
        };
        Some(self.compose_input_visuals(plan))
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
            ime_preedit_text: self
                .ime_preedit
                .as_ref()
                .map(|state| state.text.report_text()),
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
        let typography_scale = self.typography_scale();
        if let Some(runtime) = &self.live_view {
            runtime.set_surface(surface, dpi);
            if !self.view_suspended {
                self.interaction_plan = Some(runtime.interaction_plan());
                report.redraw_plan = Some(runtime.draw_plan());
            }
        } else if let Some(view) = &mut self.ui_command_view {
            let mut layout_cx =
                ViewLayoutCx::new(surface, dpi).with_typography_scale(typography_scale);
            view.layout(&mut layout_cx);
            let interaction_plan = view.interaction_plan();
            report.hit_target_count = interaction_plan.hit_target_count();
            self.interaction_plan = Some(interaction_plan);
            let mut paint_cx = ViewPaintCx::new(dpi);
            paint_cx.set_typography_scale(typography_scale);
            view.paint(&mut paint_cx);
            report.redraw_plan = Some(paint_cx.into_plan());
        }

        self.reconcile_modal_focus(&mut report);
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
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            self.ime_preedit = None;
            report.focus_visual_changed = true;
        }
        self.sync_text_edit();
        #[cfg(feature = "toast")]
        self.sync_toast_runtime(std::time::Instant::now());
        if let Some(plan) = report.redraw_plan.take() {
            report.redraw_plan = Some(self.compose_input_visuals(plan));
        }
        report.hit_target_count = self.hit_target_count();
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        report.ime_preedit_text = self
            .ime_preedit
            .as_ref()
            .map(|state| state.text.report_text());
        report.ime_selection = self.ime_preedit.as_ref().and_then(|state| state.selection);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn focused_text_input_value(&self) -> Option<String> {
        let target = self.focused_text_input_target()?;
        self.widget_display_text_value(target.widget)
    }

    pub(crate) fn focused_text_input_snapshot(&self) -> Option<(String, NativeTextSelection)> {
        let target = self.focused_text_input_target()?;
        let value = self.widget_display_text_value(target.widget)?;
        let selection = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .map(|state| state.selection.clamp(&value))
            .unwrap_or_else(|| NativeTextSelection::collapsed(value.chars().count()));
        Some((value, selection))
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    pub(crate) fn focused_text_accessibility_snapshot(
        &self,
    ) -> Option<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        let target = self.focused_text_input_target()?;
        let (value, selection) = self.focused_text_input_snapshot()?;
        let caret = self.text_input_caret_rect()?;
        crate::native_accessibility::NativeTextAccessibilitySnapshot::new(
            target, value, selection, caret,
        )
    }

    pub(crate) fn ime_replacement_selection(&self) -> Option<NativeTextSelection> {
        self.ime_preedit.as_ref().map(|preedit| preedit.replacement)
    }

    pub(crate) fn text_input_caret_rect(&self) -> Option<Rect> {
        let target = self.focused_text_input_target()?;
        let interaction = self.current_interaction_plan()?;
        let target = native_text_visual_target(target, &interaction);
        let (value, selection) = self.focused_text_input_snapshot()?;
        let first_visible_row = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .map(|state| state.first_visible_visual_row)
            .unwrap_or(0);
        let horizontal_scroll_px = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .map(|state| state.horizontal_scroll_px)
            .unwrap_or(0);
        Some(
            native_text_visual_geometry_in_viewport_with_backend(
                target,
                &value,
                selection,
                first_visible_row,
                horizontal_scroll_px,
                self.widget_text_wrap(target.widget),
                self.dpi,
                &self.text_shaping,
            )
            .caret,
        )
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
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
        }
        let interaction_plan = self.current_interaction_plan();
        let target = interaction_plan.and_then(|plan| plan.hit_target_at(point));
        report = self.dismiss_popup_overlays_except(target.map(|target| target.widget), report);
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
        self.update_pointer_visual_state(
            target.and_then(native_pointer_visual_key),
            target.and_then(native_pointer_visual_key),
            &mut report,
        );
        let Some(target) = target else {
            return report;
        };
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBoxReveal {
            self.text_drag = None;
            self.password_peek = Some(target.widget);
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
            return report;
        }
        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            self.text_drag = None;
            self.focus_target(target, &mut report);
            self.slider_drag = Some(target.widget);
            report.slider_drag_active = true;
            return self.dispatch_slider_pointer(target, point, report);
        }
        #[cfg(feature = "color-picker")]
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::ColorPickerSpectrum
                | crate::ViewHitTargetKind::ColorPickerHue
                | crate::ViewHitTargetKind::ColorPickerChannel { .. }
        ) {
            self.text_drag = None;
            self.focus_target(target, &mut report);
            self.color_picker_drag = Some((target.widget, target.kind));
            report.color_picker_drag_active = true;
            return self.dispatch_color_picker_pointer(target, point, report);
        }
        if !target.kind.accepts_text_input() {
            self.text_drag = None;
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            return report;
        }

        report.handled = true;
        self.ime_preedit = None;
        self.focus_target(target, &mut report);
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let visual_target = self
            .current_interaction_plan()
            .map(|interaction| native_text_visual_target(target, &interaction))
            .unwrap_or(target);
        let index = native_text_index_for_point_in_viewport_with_backend(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        let anchor = if shift { state.selection.anchor } else { index };
        let edit = set_pointer_selection(&value, &mut state.selection, anchor, index);
        state.preferred_visual_x = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
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
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            return self.dispatch_view_event(
                ViewEvent::TextSelectionChanged {
                    widget: target.widget,
                    selection: state.selection.into(),
                },
                report,
            );
        }
        report
    }

    pub(crate) fn dispatch_pointer_move(&mut self, point: Point) -> NativeViewInputDispatchReport {
        self.dispatch_pointer_move_at(point, std::time::Instant::now())
    }

    fn dispatch_pointer_move_at(
        &mut self,
        point: Point,
        now: std::time::Instant,
    ) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if let Some(interaction) = self.current_interaction_plan() {
            if self.tooltip.pointer_moved(&interaction, point, now) {
                report.handled = true;
                report.redraw_plan = self.current_composed_draw_plan();
            }
        }
        #[cfg(not(feature = "tooltip"))]
        let _ = now;
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
            let hovered = self
                .current_interaction_plan()
                .and_then(|plan| plan.hit_target_at(point))
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, self.pointer_pressed, &mut report);
        }
        #[cfg(feature = "password-box")]
        if let Some(widget) = self.password_peek {
            let still_peeking = self
                .current_interaction_plan()
                .and_then(|plan| plan.hit_target_at(point))
                .is_some_and(|target| {
                    target.widget == widget
                        && target.kind == crate::ViewHitTargetKind::PasswordBoxReveal
                });
            if !still_peeking {
                self.password_peek = None;
                report.handled = true;
                report.redraw_plan = self.current_composed_draw_plan();
            }
            return report;
        }
        let Some(drag) = self.text_drag else {
            #[cfg(feature = "color-picker")]
            if let Some((widget, kind)) = self.color_picker_drag {
                if let Some(target) = self.current_interaction_plan().and_then(|plan| {
                    plan.hit_targets
                        .into_iter()
                        .find(|target| target.widget == widget && target.kind == kind)
                }) {
                    report.color_picker_drag_active = true;
                    return self.dispatch_color_picker_pointer(target, point, report);
                }
                self.color_picker_drag = None;
            }
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
        let value = self
            .widget_display_text_value(drag.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == drag.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(drag.widget, &value));
        let visual_target = self
            .current_interaction_plan()
            .map(|interaction| native_text_visual_target(target, &interaction))
            .unwrap_or(target);
        let drag_viewport = native_text_drag_viewport_for_point_with_backend(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        state.first_visible_visual_row = drag_viewport.first_visible_row;
        state.horizontal_scroll_px = drag_viewport.horizontal_scroll_px;
        let index = native_text_index_for_point_in_viewport_with_backend(
            visual_target,
            &value,
            drag_viewport.point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        let edit = set_pointer_selection(&value, &mut state.selection, drag.anchor, index);
        state.preferred_visual_x = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        self.text_edit = Some(state);
        report.handled = true;
        report.text_selection_changed = edit.selection_changed;
        report.text_drag_active = true;
        report.text_drag_scroll_count = usize::from(drag_viewport.scrolled);
        if edit.selection_changed || drag_viewport.scrolled {
            report.redraw_plan = self.current_composed_draw_plan();
        }
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            return self.dispatch_view_event(
                ViewEvent::TextSelectionChanged {
                    widget: drag.widget,
                    selection: state.selection.into(),
                },
                report,
            );
        }
        report
    }

    pub(crate) fn dispatch_pointer_up(&mut self, point: Point) -> NativeViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        if self.password_peek.take().is_some() {
            let mut report = NativeViewInputDispatchReport {
                handled: true,
                hit_target_count: self.hit_target_count(),
                focused_widget: self.focused_widget.map(|widget| widget.0),
                redraw_plan: self.current_composed_draw_plan(),
                ..NativeViewInputDispatchReport::default()
            };
            let hovered = self
                .current_interaction_plan()
                .and_then(|plan| plan.hit_target_at(point))
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, None, &mut report);
            return report;
        }
        if self.text_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            self.text_drag = None;
            report.handled = true;
            report.text_drag_active = false;
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "color-picker")]
        if self.color_picker_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            self.color_picker_drag = None;
            report.handled = true;
            report.color_picker_drag_active = false;
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "slider")]
        if self.slider_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            self.slider_drag = None;
            report.handled = true;
            report.slider_drag_active = false;
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }

        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..NativeViewInputDispatchReport::default()
        };
        let interaction_plan = self.current_interaction_plan();
        let target = interaction_plan.and_then(|plan| plan.hit_target_at(point));
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
        if target.kind.accepts_text_input() {
            return report;
        }

        #[cfg(feature = "tree")]
        match target.kind {
            crate::ViewHitTargetKind::TreeNodeExpander { node } => {
                let Some(row) = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.row(node))
                else {
                    return report;
                };
                if row.expandable {
                    report.tree_expansion_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::TreeNodeExpandedChanged {
                            widget: target.widget,
                            node,
                            expanded: !row.expanded,
                        },
                        report,
                    );
                }
                return report;
            }
            crate::ViewHitTargetKind::TreeNode { node } => {
                let selected = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.tree_selection_changed = selected != Some(node);
                report.tree_invoked = true;
                let report = self.dispatch_view_event(
                    ViewEvent::TreeNodeSelected {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
                return self.dispatch_view_event(
                    ViewEvent::TreeNodeInvoked {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
            }
            _ => {}
        }

        #[cfg(feature = "grid-view")]
        match target.kind {
            crate::ViewHitTargetKind::GridViewItem { item } => {
                let selected = self
                    .widget_grid_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.grid_view_selection_changed = selected != Some(item);
                report.grid_view_invoked = true;
                let report = self.dispatch_view_event(
                    ViewEvent::GridViewItemSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return self.dispatch_view_event(
                    ViewEvent::GridViewItemInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::GridView => return report,
            _ => {}
        }

        #[cfg(feature = "table")]
        match target.kind {
            crate::ViewHitTargetKind::TableHeader { column } => {
                report.table_sort_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::TableSorted {
                        widget: target.widget,
                        column,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::TableRow { row } => {
                let selected = self
                    .widget_table_state(target.widget)
                    .and_then(|state| state.selected);
                report.table_selection_changed = selected != Some(row);
                report.table_invoked = true;
                let report = self.dispatch_view_event(
                    ViewEvent::TableRowSelected {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
                return self.dispatch_view_event(
                    ViewEvent::TableRowInvoked {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
            }
            _ => {}
        }

        #[cfg(feature = "command-palette")]
        match target.kind {
            crate::ViewHitTargetKind::CommandPaletteItem { item } => {
                report.command_palette_invoked = true;
                report.command_palette_open_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::CommandPaletteInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::CommandPaletteClear => {
                report.command_palette_query_changed = true;
                report.command_palette_cleared = true;
                return self.dispatch_view_event(
                    ViewEvent::TextChanged {
                        widget: target.widget,
                        value: String::new(),
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::CommandPaletteScrim => {
                report.command_palette_open_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::CommandPaletteOpenChanged {
                        widget: target.widget,
                        open: false,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::CommandPalette => return report,
            _ => {}
        }

        #[cfg(feature = "dialog")]
        match target.kind {
            crate::ViewHitTargetKind::ContentDialogButton { button } => {
                report.content_dialog_responded = true;
                return self.dispatch_view_event(
                    ViewEvent::ContentDialogResponded {
                        widget: target.widget,
                        button,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::ContentDialog
            | crate::ViewHitTargetKind::ContentDialogScrim => return report,
            _ => {}
        }

        #[cfg(feature = "toast")]
        match target.kind {
            crate::ViewHitTargetKind::ToastAction => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return report;
                };
                let Some(toast) = state.toast else {
                    return report;
                };
                report.toast_responded = true;
                return self.dispatch_view_event(
                    ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Action,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::ToastClose => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return report;
                };
                let Some(toast) = state.toast else {
                    return report;
                };
                report.toast_responded = true;
                return self.dispatch_view_event(
                    ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::Toast => return report,
            _ => {}
        }

        #[cfg(feature = "info-bar")]
        match target.kind {
            crate::ViewHitTargetKind::InfoBarAction => {
                report.info_bar_event = Some(crate::ZsInfoBarEvent::Action);
                return self.dispatch_view_event(
                    ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Action,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::InfoBarClose => {
                report.info_bar_event = Some(crate::ZsInfoBarEvent::Close);
                return self.dispatch_view_event(
                    ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Close,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::InfoBar => return report,
            _ => {}
        }

        #[cfg(feature = "teaching-tip")]
        match target.kind {
            crate::ViewHitTargetKind::TeachingTipAction => {
                report.teaching_tip_response = Some(crate::ZsTeachingTipResponse::Action);
                return self.dispatch_view_event(
                    ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response: crate::ZsTeachingTipResponse::Action,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::TeachingTipClose => {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::CloseButton,
                );
                report.teaching_tip_response = Some(response);
                return self.dispatch_view_event(
                    ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::TeachingTip => return report,
            _ => {}
        }

        #[cfg(feature = "breadcrumb")]
        match target.kind {
            crate::ViewHitTargetKind::BreadcrumbOverflow => {
                let expanded = self
                    .widget_breadcrumb_state(target.widget)
                    .map_or(true, |state| !state.overflow_open);
                report.breadcrumb_expanded_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::BreadcrumbExpandedChanged {
                        widget: target.widget,
                        expanded,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::BreadcrumbItem { item }
            | crate::ViewHitTargetKind::BreadcrumbOverflowItem { item } => {
                report.breadcrumb_selection = Some(item);
                report.breadcrumb_expanded_changed = self
                    .widget_breadcrumb_state(target.widget)
                    .is_some_and(|state| state.overflow_open);
                return self.dispatch_view_event(
                    ViewEvent::BreadcrumbSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
            }
            crate::ViewHitTargetKind::BreadcrumbBar => return report,
            _ => {}
        }

        let event = self.activation_event(target);

        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            report.radio_selection_changed = true;
        }
        #[cfg(feature = "auto-suggest")]
        match target.kind {
            crate::ViewHitTargetKind::AutoSuggestSuggestion { .. }
            | crate::ViewHitTargetKind::AutoSuggestSearch => {
                report.auto_suggest_submitted = true;
                report.auto_suggest_expanded_changed = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
            }
            crate::ViewHitTargetKind::AutoSuggestClear => {
                report.auto_suggest_cleared = true;
                report.auto_suggest_expanded_changed = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
            }
            _ => {}
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
        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            report.color_picker_expanded_changed = true;
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
        #[cfg(feature = "password-box")]
        let had_drag = had_drag | self.password_peek.take().is_some();
        #[cfg(feature = "slider")]
        let had_drag = had_drag | self.slider_drag.take().is_some();
        #[cfg(feature = "color-picker")]
        let had_drag = had_drag | self.color_picker_drag.take().is_some();
        let mut report = NativeViewInputDispatchReport {
            handled: had_drag,
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            text_drag_active: false,
            #[cfg(feature = "slider")]
            slider_drag_active: false,
            #[cfg(feature = "color-picker")]
            color_picker_drag_active: false,
            ..NativeViewInputDispatchReport::default()
        };
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
        self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn dispatch_pointer_leave(&mut self) -> NativeViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        let had_password_peek = self.password_peek.take().is_some();
        #[allow(unused_mut)]
        let mut report = NativeViewInputDispatchReport {
            #[cfg(feature = "password-box")]
            handled: had_password_peek,
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
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
            self.update_pointer_visual_state(None, None, &mut report);
            report
        }
        #[cfg(not(any(
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
        )))]
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
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
        }
        let Some(interaction_plan) = self.current_interaction_plan() else {
            return report;
        };
        #[cfg(feature = "toast")]
        if let Some(toast_target) = interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::Toast)
        {
            let Some((state, spec)) = self.widget_toast_state(toast_target.widget) else {
                return report;
            };
            let Some(toast) = state.toast else {
                return report;
            };
            if key == NativeViewKey::Escape {
                report.handled = true;
                report.toast_responded = true;
                return self.dispatch_view_event(
                    ViewEvent::ToastResponded {
                        widget: toast_target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::EscapeKey,
                        ),
                    },
                    report,
                );
            }
            if self.focused_widget == Some(toast_target.widget) {
                let focus_offset = match key {
                    NativeViewKey::Left => Some(-1),
                    NativeViewKey::Right => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.toast_focus_changed = next != state.focused_control;
                    return self.dispatch_view_event(
                        ViewEvent::ToastFocused {
                            widget: toast_target.widget,
                            toast,
                            control: next,
                        },
                        report,
                    );
                }
                if matches!(key, NativeViewKey::Enter | NativeViewKey::Space) {
                    let response = match state.focused_control {
                        crate::ZsToastControl::Action if spec.action_label().is_some() => {
                            crate::ZsToastResponse::Action
                        }
                        _ => crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.toast_responded = true;
                    return self.dispatch_view_event(
                        ViewEvent::ToastResponded {
                            widget: toast_target.widget,
                            toast,
                            response,
                        },
                        report,
                    );
                }
            }
        }
        #[cfg(feature = "teaching-tip")]
        if let Some(tip_target) = interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTip)
        {
            let Some((state, spec)) = self.widget_teaching_tip_state(tip_target.widget) else {
                return report;
            };
            if key == NativeViewKey::Escape {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::EscapeKey,
                );
                report.handled = true;
                report.teaching_tip_response = Some(response);
                return self.dispatch_view_event(
                    ViewEvent::TeachingTipResponded {
                        widget: tip_target.widget,
                        response,
                    },
                    report,
                );
            }
            if self.focused_widget == Some(tip_target.widget) {
                let focus_offset = match key {
                    NativeViewKey::Left => Some(-1),
                    NativeViewKey::Right => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.teaching_tip_focus_changed = next != state.focused_control;
                    return self.dispatch_view_event(
                        ViewEvent::TeachingTipFocused {
                            widget: tip_target.widget,
                            control: next,
                        },
                        report,
                    );
                }
                if matches!(key, NativeViewKey::Enter | NativeViewKey::Space) {
                    let response = match state.focused_control {
                        crate::ZsTeachingTipControl::Action if spec.action_label().is_some() => {
                            crate::ZsTeachingTipResponse::Action
                        }
                        _ => crate::ZsTeachingTipResponse::Dismissed(
                            crate::ZsTeachingTipDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.teaching_tip_response = Some(response);
                    return self.dispatch_view_event(
                        ViewEvent::TeachingTipResponded {
                            widget: tip_target.widget,
                            response,
                        },
                        report,
                    );
                }
            }
        }
        #[cfg(feature = "info-bar")]
        if let Some(widget) = self.focused_widget {
            if let Some((state, spec)) = self.widget_info_bar_state(widget) {
                if key == NativeViewKey::Escape && spec.is_closable() {
                    report.handled = true;
                    report.info_bar_event = Some(crate::ZsInfoBarEvent::Close);
                    return self.dispatch_view_event(
                        ViewEvent::InfoBarInvoked {
                            widget,
                            event: crate::ZsInfoBarEvent::Close,
                        },
                        report,
                    );
                }
                if let Some(current) = state.focused_control {
                    let focus_offset = match key {
                        NativeViewKey::Left => Some(-1),
                        NativeViewKey::Right => Some(1),
                        _ => None,
                    };
                    if let Some(offset) = focus_offset {
                        let next = spec.relative_control(current, offset);
                        report.handled = true;
                        report.info_bar_focus_changed = next != current;
                        return self.dispatch_view_event(
                            ViewEvent::InfoBarFocused {
                                widget,
                                control: next,
                            },
                            report,
                        );
                    }
                    if matches!(key, NativeViewKey::Enter | NativeViewKey::Space) {
                        let event = match current {
                            crate::ZsInfoBarControl::Action => crate::ZsInfoBarEvent::Action,
                            crate::ZsInfoBarControl::Close => crate::ZsInfoBarEvent::Close,
                        };
                        if spec.has_control(current) {
                            report.handled = true;
                            report.info_bar_event = Some(event);
                            return self.dispatch_view_event(
                                ViewEvent::InfoBarInvoked { widget, event },
                                report,
                            );
                        }
                    }
                }
            }
        }
        #[cfg(feature = "breadcrumb")]
        if let Some(widget) = self.focused_widget {
            if let Some(state) = self.widget_breadcrumb_state(widget) {
                let mut visible = interaction_plan
                    .hit_targets
                    .iter()
                    .filter_map(|target| match target.kind {
                        crate::ViewHitTargetKind::BreadcrumbOverflow if target.widget == widget => {
                            Some((target.bounds.x, crate::ZsBreadcrumbFocusTarget::Overflow))
                        }
                        crate::ViewHitTargetKind::BreadcrumbItem { item }
                            if target.widget == widget =>
                        {
                            Some((target.bounds.x, crate::ZsBreadcrumbFocusTarget::Item(item)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                visible.sort_by_key(|(x, _)| *x);
                let visible = visible
                    .into_iter()
                    .map(|(_, target)| target)
                    .collect::<Vec<_>>();
                let mut hidden = interaction_plan
                    .hit_targets
                    .iter()
                    .filter_map(|target| match target.kind {
                        crate::ViewHitTargetKind::BreadcrumbOverflowItem { item }
                            if target.widget == widget =>
                        {
                            Some((target.bounds.y, crate::ZsBreadcrumbFocusTarget::Item(item)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                hidden.sort_by_key(|(y, _)| *y);
                let hidden = hidden
                    .into_iter()
                    .map(|(_, target)| target)
                    .collect::<Vec<_>>();

                if key == NativeViewKey::Escape && state.overflow_open {
                    report.handled = true;
                    report.breadcrumb_expanded_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::BreadcrumbExpandedChanged {
                            widget,
                            expanded: false,
                        },
                        report,
                    );
                }

                let focus_list = if state.overflow_open
                    && matches!(key, NativeViewKey::Up | NativeViewKey::Down)
                    && !hidden.is_empty()
                {
                    &hidden
                } else {
                    &visible
                };
                let focus_offset = match key {
                    NativeViewKey::Left | NativeViewKey::Up => Some(-1),
                    NativeViewKey::Right | NativeViewKey::Down => Some(1),
                    NativeViewKey::Home => Some(isize::MIN),
                    NativeViewKey::End => Some(isize::MAX),
                    _ => None,
                };
                if let Some(offset) = focus_offset.filter(|_| !focus_list.is_empty()) {
                    let current_index = state.focused.and_then(|current| {
                        focus_list.iter().position(|target| *target == current)
                    });
                    let next_index = if offset == isize::MIN {
                        0
                    } else if offset == isize::MAX {
                        focus_list.len() - 1
                    } else {
                        match current_index {
                            Some(index) => (index as isize + offset)
                                .clamp(0, focus_list.len().saturating_sub(1) as isize)
                                as usize,
                            None if offset < 0 => focus_list.len() - 1,
                            None => 0,
                        }
                    };
                    let next = focus_list[next_index];
                    report.handled = true;
                    report.breadcrumb_focus_changed = state.focused != Some(next);
                    return self.dispatch_view_event(
                        ViewEvent::BreadcrumbFocused {
                            widget,
                            target: next,
                        },
                        report,
                    );
                }
                if matches!(key, NativeViewKey::Enter | NativeViewKey::Space) {
                    let active = state
                        .focused
                        .or_else(|| visible.first().copied())
                        .or_else(|| state.current().map(crate::ZsBreadcrumbFocusTarget::Item));
                    match active {
                        Some(crate::ZsBreadcrumbFocusTarget::Overflow) => {
                            report.handled = true;
                            report.breadcrumb_expanded_changed = true;
                            return self.dispatch_view_event(
                                ViewEvent::BreadcrumbExpandedChanged {
                                    widget,
                                    expanded: !state.overflow_open,
                                },
                                report,
                            );
                        }
                        Some(crate::ZsBreadcrumbFocusTarget::Item(item)) => {
                            report.handled = true;
                            report.breadcrumb_selection = Some(item);
                            report.breadcrumb_expanded_changed = state.overflow_open;
                            return self.dispatch_view_event(
                                ViewEvent::BreadcrumbSelected { widget, item },
                                report,
                            );
                        }
                        None => {}
                    }
                }
            }
        }
        #[cfg(feature = "command-palette")]
        if let Some(palette_target) = interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::CommandPalette)
        {
            if self.focused_widget != Some(palette_target.widget) {
                self.focus_target(palette_target, &mut report);
            }
            let Some(state) = self.widget_command_palette_state(palette_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let next = match key {
                NativeViewKey::Up => state.relative_highlight(-1),
                NativeViewKey::Down => state.relative_highlight(1),
                NativeViewKey::Home => state.first_enabled(),
                NativeViewKey::End => state.last_enabled(),
                _ => None,
            };
            if let Some(item) = next {
                report.handled = true;
                report.command_palette_highlight_changed = state.highlighted != Some(item);
                if report.command_palette_highlight_changed {
                    return self.dispatch_view_event(
                        ViewEvent::CommandPaletteHighlighted {
                            widget: palette_target.widget,
                            item,
                        },
                        report,
                    );
                }
                return report;
            }
            match key {
                NativeViewKey::Enter => {
                    if let Some(item) = state.highlighted.or_else(|| state.first_enabled()) {
                        report.handled = true;
                        report.command_palette_invoked = true;
                        report.command_palette_open_changed = true;
                        return self.dispatch_view_event(
                            ViewEvent::CommandPaletteInvoked {
                                widget: palette_target.widget,
                                item,
                            },
                            report,
                        );
                    }
                }
                NativeViewKey::Escape => {
                    report.handled = true;
                    report.command_palette_open_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::CommandPaletteOpenChanged {
                            widget: palette_target.widget,
                            open: false,
                        },
                        report,
                    );
                }
                NativeViewKey::Tab => {
                    report.handled = true;
                    return report;
                }
                _ => {}
            }
        }

        #[cfg(feature = "dialog")]
        if let Some(dialog_target) = interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialog)
        {
            if self.focused_widget != Some(dialog_target.widget) {
                self.focus_target(dialog_target, &mut report);
            }
            let Some((state, spec)) = self.widget_content_dialog_state(dialog_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let focus_offset = match key {
                NativeViewKey::Tab => Some(if shift { -1 } else { 1 }),
                NativeViewKey::Left => Some(-1),
                NativeViewKey::Right => Some(1),
                _ => None,
            };
            if let Some(offset) = focus_offset {
                let button = spec.relative_button(state.focused_button, offset);
                report.handled = true;
                report.content_dialog_focus_changed = button != state.focused_button;
                return self.dispatch_view_event(
                    ViewEvent::ContentDialogFocused {
                        widget: dialog_target.widget,
                        button,
                    },
                    report,
                );
            }
            let response = match key {
                NativeViewKey::Escape => Some(crate::ZsContentDialogButton::Close),
                NativeViewKey::Enter | NativeViewKey::Space => Some(state.focused_button),
                _ => None,
            };
            if let Some(button) = response.filter(|button| spec.has_button(*button)) {
                report.handled = true;
                report.content_dialog_responded = true;
                return self.dispatch_view_event(
                    ViewEvent::ContentDialogResponded {
                        widget: dialog_target.widget,
                        button,
                    },
                    report,
                );
            }
            return report;
        }
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
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(target.widget, &mut report);
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
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(target.widget, &mut report);
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
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            report.focused_widget = None;
            return report;
        };

        if target.kind.accepts_text_input() {
            let movement = match key {
                NativeViewKey::Home => Some(NativeTextMovement::Home),
                NativeViewKey::End => Some(NativeTextMovement::End),
                _ => None,
            };
            let horizontal_navigation = match key {
                NativeViewKey::Left => Some(NativeTextVisualHorizontalDirection::Left),
                NativeViewKey::Right => Some(NativeTextVisualHorizontalDirection::Right),
                _ => None,
            };
            let visual_navigation = (target.kind == crate::ViewHitTargetKind::TextEditor)
                .then(|| match key {
                    NativeViewKey::Up => Some((NativeTextVisualDirection::Up, false)),
                    NativeViewKey::Down => Some((NativeTextVisualDirection::Down, false)),
                    NativeViewKey::PageUp => Some((NativeTextVisualDirection::Up, true)),
                    NativeViewKey::PageDown => Some((NativeTextVisualDirection::Down, true)),
                    _ => None,
                })
                .flatten();
            if movement.is_some() || horizontal_navigation.is_some() || visual_navigation.is_some()
            {
                let value = self.widget_display_text_value(widget).unwrap_or_default();
                let mut state = self
                    .text_edit
                    .filter(|state| state.widget == widget)
                    .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
                let edit = if let Some((direction, page)) = visual_navigation {
                    let (target_index, preferred_x) = if page {
                        let (target_index, preferred_x, first_visible_row) =
                            native_text_index_for_vertical_page_move_with_backend(
                                target,
                                &value,
                                state.selection.caret,
                                direction,
                                state.preferred_visual_x,
                                state.first_visible_visual_row,
                                self.widget_text_wrap(widget),
                                self.dpi,
                                &self.text_shaping,
                            );
                        state.first_visible_visual_row = first_visible_row;
                        (target_index, preferred_x)
                    } else {
                        native_text_index_for_vertical_move_with_backend(
                            target,
                            &value,
                            state.selection.caret,
                            direction,
                            state.preferred_visual_x,
                            self.widget_text_wrap(widget),
                            self.dpi,
                            &self.text_shaping,
                        )
                    };
                    state.preferred_visual_x = Some(preferred_x);
                    move_selection_to(&value, &mut state.selection, target_index, shift)
                } else if let Some(direction) = horizontal_navigation {
                    state.preferred_visual_x = None;
                    move_native_text_selection_horizontally_with_backend(
                        target,
                        &value,
                        &mut state.selection,
                        direction,
                        shift,
                        self.widget_text_wrap(widget),
                        self.dpi,
                        &self.text_shaping,
                    )
                } else {
                    state.preferred_visual_x = None;
                    move_selection(
                        &value,
                        &mut state.selection,
                        movement.expect("text movement should be present"),
                        shift,
                        target.kind == crate::ViewHitTargetKind::TextEditor,
                    )
                };
                if !visual_navigation.is_some_and(|(_, page)| page) {
                    state.first_visible_visual_row =
                        native_text_first_visible_row_for_caret_with_backend(
                            target,
                            &value,
                            state.selection.caret,
                            state.first_visible_visual_row,
                            self.widget_text_wrap(widget),
                            self.dpi,
                            &self.text_shaping,
                        );
                }
                state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
                    target,
                    &value,
                    state.selection.caret,
                    state.horizontal_scroll_px,
                    self.widget_text_wrap(widget),
                    self.dpi,
                    &self.text_shaping,
                );
                self.text_edit = Some(state);
                report.handled = edit.handled;
                report.text_selection_changed = edit.selection_changed;
                report.redraw_plan = self.current_composed_draw_plan();
                self.populate_text_report(&mut report);
                #[cfg(feature = "textbox")]
                if edit.selection_changed
                    && matches!(
                        target.kind,
                        crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                    )
                {
                    return self.dispatch_view_event(
                        ViewEvent::TextSelectionChanged {
                            widget,
                            selection: state.selection.into(),
                        },
                        report,
                    );
                }
                return report;
            }
        }

        #[cfg(feature = "auto-suggest")]
        if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
            let Some(state) = self.widget_auto_suggest_state(widget) else {
                return report;
            };
            match key {
                NativeViewKey::Up | NativeViewKey::Down if !state.suggestion_ids.is_empty() => {
                    let offset = if key == NativeViewKey::Up { -1 } else { 1 };
                    let Some(suggestion) = state.next_highlight(offset) else {
                        return report;
                    };
                    report.handled = true;
                    report.auto_suggest_highlight_changed = state.highlighted != Some(suggestion);
                    report.auto_suggest_expanded_changed = !state.expanded;
                    if !state.expanded {
                        report = self.dispatch_view_event(
                            ViewEvent::AutoSuggestExpandedChanged {
                                widget,
                                expanded: true,
                            },
                            report,
                        );
                    }
                    return self.dispatch_view_event(
                        ViewEvent::AutoSuggestHighlighted { widget, suggestion },
                        report,
                    );
                }
                NativeViewKey::Enter => {
                    report.handled = true;
                    report.auto_suggest_submitted = true;
                    report.auto_suggest_expanded_changed = state.expanded;
                    return self.dispatch_view_event(
                        ViewEvent::AutoSuggestSubmitted {
                            widget,
                            suggestion: state.highlighted,
                        },
                        report,
                    );
                }
                NativeViewKey::Escape if state.expanded => {
                    report.handled = true;
                    report.auto_suggest_expanded_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::AutoSuggestExpandedChanged {
                            widget,
                            expanded: false,
                        },
                        report,
                    );
                }
                _ => {}
            }
        }

        #[cfg(feature = "tree")]
        if target.kind == crate::ViewHitTargetKind::TreeView {
            let Some(state) = self.widget_tree_view_state(widget) else {
                return report;
            };
            let select = match key {
                NativeViewKey::Up => state.relative_visible(-1),
                NativeViewKey::Down => state.relative_visible(1),
                NativeViewKey::Home => state.first_visible(),
                NativeViewKey::End => state.last_visible(),
                NativeViewKey::Space => state.selected.or_else(|| state.first_visible()),
                _ => None,
            };
            if let Some(node) = select {
                report.handled = true;
                report.tree_selection_changed = state.selected != Some(node);
                if report.tree_selection_changed {
                    return self
                        .dispatch_view_event(ViewEvent::TreeNodeSelected { widget, node }, report);
                }
                return report;
            }

            if key == NativeViewKey::Left {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && row.expanded {
                    report.tree_expansion_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: false,
                        },
                        report,
                    );
                }
                if let Some(parent) = row.parent {
                    report.tree_selection_changed = state.selected != Some(parent);
                    return self.dispatch_view_event(
                        ViewEvent::TreeNodeSelected {
                            widget,
                            node: parent,
                        },
                        report,
                    );
                }
                return report;
            }

            if key == NativeViewKey::Right {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && !row.expanded {
                    report.tree_expansion_changed = true;
                    return self.dispatch_view_event(
                        ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: true,
                        },
                        report,
                    );
                }
                if let Some(child) = state.first_visible_child(node) {
                    report.tree_selection_changed = state.selected != Some(child);
                    return self.dispatch_view_event(
                        ViewEvent::TreeNodeSelected {
                            widget,
                            node: child,
                        },
                        report,
                    );
                }
                return report;
            }

            if key == NativeViewKey::Enter {
                let Some(node) = state
                    .selected
                    .filter(|selected| state.row(*selected).is_some())
                else {
                    return report;
                };
                report.handled = true;
                report.tree_invoked = true;
                return self
                    .dispatch_view_event(ViewEvent::TreeNodeInvoked { widget, node }, report);
            }
        }

        #[cfg(feature = "grid-view")]
        if target.kind == crate::ViewHitTargetKind::GridView {
            let Some(state) = self.widget_grid_view_state(widget) else {
                return report;
            };
            let select = match key {
                NativeViewKey::Left => state.relative_horizontal(-1),
                NativeViewKey::Right => state.relative_horizontal(1),
                NativeViewKey::Up => state.relative_vertical(-1),
                NativeViewKey::Down => state.relative_vertical(1),
                NativeViewKey::Home => state.first(),
                NativeViewKey::End => state.last(),
                NativeViewKey::Space => state.selected.or_else(|| state.first()),
                _ => None,
            };
            if let Some(item) = select {
                report.handled = true;
                report.grid_view_selection_changed = state.selected != Some(item);
                if report.grid_view_selection_changed {
                    return self.dispatch_view_event(
                        ViewEvent::GridViewItemSelected { widget, item },
                        report,
                    );
                }
                return report;
            }
            if key == NativeViewKey::Enter {
                let Some(item) = state
                    .selected
                    .filter(|selected| state.contains(*selected))
                    .or_else(|| state.first())
                else {
                    return report;
                };
                report.handled = true;
                report.grid_view_invoked = true;
                return self
                    .dispatch_view_event(ViewEvent::GridViewItemInvoked { widget, item }, report);
            }
        }

        #[cfg(feature = "table")]
        if target.kind == crate::ViewHitTargetKind::DataGrid {
            let Some(state) = self.widget_table_state(widget) else {
                return report;
            };
            let select = match key {
                NativeViewKey::Up => state.relative_row(-1),
                NativeViewKey::Down => state.relative_row(1),
                NativeViewKey::Home => state.first_row(),
                NativeViewKey::End => state.last_row(),
                NativeViewKey::Space => state.selected.or_else(|| state.first_row()),
                _ => None,
            };
            if let Some(row) = select {
                report.handled = true;
                report.table_selection_changed = state.selected != Some(row);
                if report.table_selection_changed {
                    return self
                        .dispatch_view_event(ViewEvent::TableRowSelected { widget, row }, report);
                }
                return report;
            }
            if key == NativeViewKey::Enter {
                let Some(row) = state.selected.filter(|row| state.contains_row(*row)) else {
                    return report;
                };
                report.handled = true;
                report.table_invoked = true;
                return self
                    .dispatch_view_event(ViewEvent::TableRowInvoked { widget, row }, report);
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

        #[cfg(feature = "number-box")]
        if target.kind == crate::ViewHitTargetKind::NumberBox {
            let event = match key {
                NativeViewKey::Down => Some(ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: shift,
                }),
                NativeViewKey::Up => Some(ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: shift,
                }),
                NativeViewKey::PageDown => Some(ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: true,
                }),
                NativeViewKey::PageUp => Some(ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: true,
                }),
                NativeViewKey::Enter => Some(ViewEvent::NumberBoxCommit { widget }),
                NativeViewKey::Escape => Some(ViewEvent::NumberBoxReset { widget }),
                _ => None,
            };
            if let Some(event) = event {
                report.handled = true;
                return self.dispatch_view_event(event, report);
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

        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let Some(state) = self.widget_color_picker_state(widget) else {
                return report;
            };
            let next_expanded = match key {
                NativeViewKey::Enter | NativeViewKey::Space => Some(!state.expanded),
                NativeViewKey::Escape if state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = next_expanded {
                report.handled = true;
                report.color_picker_expanded_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::ColorPickerExpandedChanged { widget, expanded },
                    report,
                );
            }
            if !state.expanded {
                return report;
            }

            let next_channel = match key {
                NativeViewKey::Up => Some(state.active_channel.previous(state.alpha_enabled)),
                NativeViewKey::Down => Some(state.active_channel.next(state.alpha_enabled)),
                _ => None,
            };
            if let Some(channel) = next_channel {
                report.handled = true;
                if channel == state.active_channel {
                    return report;
                }
                report.color_picker_channel_changed = true;
                return self.dispatch_view_event(
                    ViewEvent::ColorPickerChannelChanged { widget, channel },
                    report,
                );
            }

            let current = state.channel_value(state.active_channel);
            let delta = if shift { 10_i16 } else { 1_i16 };
            let next = match key {
                NativeViewKey::Left => Some((i16::from(current) - delta).clamp(0, 255) as u8),
                NativeViewKey::Right => Some((i16::from(current) + delta).clamp(0, 255) as u8),
                NativeViewKey::Home => Some(0),
                NativeViewKey::End => Some(255),
                _ => None,
            };
            if let Some(value) = next {
                report.handled = true;
                let color = state.active_channel.with_value(state.color, value);
                report.color_picker_value = Some(color);
                if color == state.color {
                    return report;
                }
                report.color_picker_value_changed = true;
                return self.dispatch_view_event(ViewEvent::ColorChanged { widget, color }, report);
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
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget, &mut report);
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
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget, &mut report);
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

        #[cfg(feature = "time-picker")]
        if target.kind == crate::ViewHitTargetKind::TimePicker {
            let Some(state) = self.widget_time_picker_state(widget) else {
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
                    ViewEvent::TimePickerExpandedChanged { widget, expanded },
                    report,
                );
            }
            let minute_step = i32::from(state.minute_increment.get());
            let value = match key {
                NativeViewKey::Left => Some(state.value.add_minutes_wrapping(-60)),
                NativeViewKey::Right => Some(state.value.add_minutes_wrapping(60)),
                NativeViewKey::Up => Some(state.value.add_minutes_wrapping(-minute_step)),
                NativeViewKey::Down => Some(state.value.add_minutes_wrapping(minute_step)),
                NativeViewKey::Home => Some(crate::ZsTime::MIDNIGHT),
                NativeViewKey::End => {
                    crate::ZsTime::new(23, 60 - state.minute_increment.get()).ok()
                }
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                if value == state.value {
                    return report;
                }
                return self.dispatch_view_event(ViewEvent::TimeChanged { widget, value }, report);
            }
        }

        #[cfg(feature = "list")]
        if matches!(key, NativeViewKey::Up | NativeViewKey::Down) {
            let offset = if key == NativeViewKey::Up { -1 } else { 1 };
            if let Some((next_widget, _index)) = self.widget_list_relative_widget(widget, offset) {
                if let Some(next_target) = interaction_plan.hit_target_for_widget(next_widget) {
                    report.handled = true;
                    self.focus_target(next_target, &mut report);
                    #[cfg(feature = "tooltip")]
                    self.show_keyboard_tooltip(next_target.widget, &mut report);
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
        #[cfg(feature = "label")]
        let activates = activates
            || matches!(
                (target.kind, key),
                (
                    crate::ViewHitTargetKind::NavigationViewToggle,
                    NativeViewKey::Enter | NativeViewKey::Space
                )
            );
        #[cfg(feature = "toggle-button")]
        let activates = activates
            || matches!(
                (target.kind, key),
                (crate::ViewHitTargetKind::ToggleButton, NativeViewKey::Space)
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
            .and_then(|plan| plan.focus_target_for_widget(widget))
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
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            report.focused_widget = None;
            return report;
        };
        #[cfg(feature = "dialog")]
        if self
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| state.open)
        {
            report.handled = true;
            return report;
        }
        #[cfg(feature = "toast")]
        if self.widget_toast_state(widget).is_some() {
            report.handled = true;
            return report;
        }
        #[cfg(feature = "info-bar")]
        if self.widget_info_bar_state(widget).is_some() {
            report.handled = true;
            return report;
        }
        #[cfg(feature = "teaching-tip")]
        if self.widget_teaching_tip_state(widget).is_some() {
            report.handled = true;
            return report;
        }
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
        if !target.kind.accepts_text_input() {
            return report;
        }

        #[cfg(feature = "password-box")]
        let mut password = (target.kind == crate::ViewHitTargetKind::PasswordBox)
            .then(|| self.widget_password_value(widget).unwrap_or_default());
        #[cfg(feature = "password-box")]
        let mut value = zeroize::Zeroizing::new(
            password
                .as_ref()
                .map(|password| password.as_str().to_owned())
                .unwrap_or_else(|| self.widget_text_value(widget).unwrap_or_default()),
        );
        #[cfg(not(feature = "password-box"))]
        let mut value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        state.clamp(&value);
        #[cfg(feature = "textbox")]
        let history_before = matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        )
        .then(|| (value.as_str().to_owned(), state.selection));
        let edit = apply_text_input(
            &mut value,
            &mut state.selection,
            text,
            target.kind == crate::ViewHitTargetKind::TextEditor,
        );
        if !edit.handled {
            return report;
        }
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
        report.handled = true;
        report.text_selection_changed = edit.selection_changed;
        self.text_edit = Some(state);
        #[cfg(feature = "textbox")]
        if edit.text_changed {
            if let Some((before_value, before_selection)) = history_before {
                self.text_history.record_text_change(
                    widget,
                    &before_value,
                    before_selection,
                    value.as_str(),
                );
            }
        }
        if edit.text_changed {
            #[cfg(feature = "command-palette")]
            if target.kind == crate::ViewHitTargetKind::CommandPalette {
                report.command_palette_query_changed = true;
            }
            #[cfg(feature = "auto-suggest")]
            if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
                report.auto_suggest_expanded_changed = self
                    .widget_auto_suggest_state(widget)
                    .is_some_and(|state| !state.expanded);
            }
            #[cfg(feature = "password-box")]
            if let Some(password) = &mut password {
                *password.as_string_mut() = std::mem::take(&mut *value);
                return self.dispatch_view_event(
                    ViewEvent::PasswordChanged {
                        widget,
                        value: password.clone(),
                    },
                    report,
                );
            }
            #[cfg(feature = "password-box")]
            let value = std::mem::take(&mut *value);
            #[cfg(feature = "textbox")]
            if edit.selection_changed
                && matches!(
                    target.kind,
                    crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                )
            {
                return self.dispatch_view_event(
                    ViewEvent::TextEdited {
                        widget,
                        value,
                        selection: state.selection.into(),
                    },
                    report,
                );
            }
            self.dispatch_view_event(ViewEvent::TextChanged { widget, value }, report)
        } else {
            #[cfg(feature = "textbox")]
            if edit.selection_changed
                && matches!(
                    target.kind,
                    crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                )
            {
                return self.dispatch_view_event(
                    ViewEvent::TextSelectionChanged {
                        widget,
                        selection: state.selection.into(),
                    },
                    report,
                );
            }
            report.redraw_plan = edit
                .selection_changed
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

        let selection = selection.map(|(start, end)| {
            let start = snap_grapheme_index(text, start);
            let end = snap_grapheme_index(text, end);
            (start.min(end), start.max(end))
        });
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
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
        #[cfg(feature = "password-box")]
        let preedit_text = if target.kind == crate::ViewHitTargetKind::PasswordBox {
            NativeViewImeText::Secure(crate::ZsPassword::from(text))
        } else {
            NativeViewImeText::Plain(text.to_string())
        };
        #[cfg(not(feature = "password-box"))]
        let preedit_text = NativeViewImeText::Plain(text.to_string());
        let report_text = preedit_text.report_text();
        self.ime_preedit = Some(NativeViewImePreedit {
            widget: target.widget,
            text: preedit_text,
            selection,
            replacement,
        });
        report.handled = true;
        report.ime_preedit_text = Some(report_text);
        report.ime_selection = selection;
        report.redraw_plan = self.current_composed_draw_plan();
        report
    }

    pub(crate) fn dispatch_ime_commit(&mut self, text: &str) -> NativeViewInputDispatchReport {
        let preedit = self.ime_preedit.take();
        let had_preedit = preedit.is_some();
        if let Some(preedit) = preedit {
            let first_visible_visual_row = self
                .text_edit
                .filter(|state| state.widget == preedit.widget)
                .map(|state| state.first_visible_visual_row)
                .unwrap_or(0);
            let horizontal_scroll_px = self
                .text_edit
                .filter(|state| state.widget == preedit.widget)
                .map(|state| state.horizontal_scroll_px)
                .unwrap_or(0);
            self.text_edit = Some(NativeTextEditState {
                widget: preedit.widget,
                selection: preedit.replacement,
                preferred_visual_x: None,
                first_visible_visual_row,
                horizontal_scroll_px,
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
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
        }
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.current_interaction_plan()
                .and_then(|plan| plan.hit_target_for_widget(*widget))
                .is_some_and(|target| target.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            report = self.dispatch_view_event(ViewEvent::NumberBoxCommit { widget }, report);
        }
        let had_focus = self.focused_widget.take().is_some();
        self.text_edit = None;
        self.text_drag = None;
        #[cfg(feature = "password-box")]
        let had_password_peek = self.password_peek.take().is_some();
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        #[cfg(feature = "color-picker")]
        {
            self.color_picker_drag = None;
        }
        let had_preedit = self.ime_preedit.take().is_some();
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
            feature = "breadcrumb",
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
        self.update_pointer_visual_state(None, None, &mut report);
        #[cfg(feature = "password-box")]
        let had_preedit = had_preedit | had_password_peek;
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
        #[allow(unused_mut)]
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            report.redraw_plan = self.current_composed_draw_plan();
        }
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

        if target.kind == crate::ViewHitTargetKind::TextEditor
            && self.focused_widget == Some(target.widget)
            && delta_y.0 != 0.0
        {
            let value = self
                .widget_display_text_value(target.widget)
                .unwrap_or_default();
            let mut state = self
                .text_edit
                .filter(|state| state.widget == target.widget)
                .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
            let previous = state.first_visible_visual_row;
            state.first_visible_visual_row = native_text_scroll_visual_rows_with_backend(
                target,
                &value,
                previous,
                native_text_wheel_row_delta(delta_y),
                self.widget_text_wrap(target.widget),
                self.dpi,
                &self.text_shaping,
            );
            self.text_edit = Some(state);
            report.handled = true;
            if state.first_visible_visual_row != previous {
                report.redraw_plan = self.current_composed_draw_plan();
            }
            return report;
        }

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
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.current_interaction_plan()
                .and_then(|plan| plan.hit_target_for_widget(*widget))
                .is_some_and(|current| current.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            let current_report = std::mem::take(report);
            *report =
                self.dispatch_view_event(ViewEvent::NumberBoxCommit { widget }, current_report);
        }
        self.ime_preedit = None;
        self.text_drag = None;
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        #[cfg(feature = "color-picker")]
        {
            self.color_picker_drag = None;
        }
        self.focused_widget = Some(target.widget);
        self.ensure_text_edit_for_target(target);
        report.focused_widget = Some(target.widget.0);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(report);
        report.focus_visual_changed = true;
        report.redraw_plan = self.current_composed_draw_plan();
    }

    #[cfg(feature = "tooltip")]
    fn show_keyboard_tooltip(
        &mut self,
        widget: crate::WidgetId,
        report: &mut NativeViewInputDispatchReport,
    ) {
        let Some(interaction) = self.current_interaction_plan() else {
            return;
        };
        if self
            .tooltip
            .focus_widget(&interaction, widget, std::time::Instant::now())
        {
            report.redraw_plan = self.current_composed_draw_plan();
        }
    }

    fn focused_text_input_target(&self) -> Option<crate::ViewHitTarget> {
        let widget = self.focused_widget?;
        self.current_interaction_plan()
            .and_then(|plan| plan.focus_target_for_widget(widget))
            .filter(|target| target.kind.accepts_text_input())
    }

    fn current_composed_draw_plan(&self) -> Option<NativeDrawPlan> {
        if self.view_suspended {
            return None;
        }
        let plan = if let Some(runtime) = &self.live_view {
            runtime.draw_plan()
        } else {
            let view = self.ui_command_view.as_ref()?;
            let elapsed = self
                .animation_epoch
                .map(|epoch| epoch.elapsed())
                .unwrap_or_default();
            let mut paint_cx = ViewPaintCx::with_animation_elapsed(self.dpi, elapsed);
            paint_cx.set_typography_scale(self.typography_scale());
            view.paint(&mut paint_cx);
            paint_cx.into_plan()
        };
        Some(self.compose_input_visuals(plan))
    }

    fn compose_input_visuals(&self, plan: NativeDrawPlan) -> NativeDrawPlan {
        let mut plan = plan;
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
        if let Some(interaction_plan) = self.current_interaction_plan() {
            decorate_native_pointer_visuals(
                &mut plan,
                &interaction_plan,
                self.pointer_hover,
                self.pointer_pressed,
                self.dpi,
            );
        }
        #[cfg(feature = "password-box")]
        self.compose_password_peek(&mut plan);
        if let (Some(target), Some((value, selection))) = (
            self.focused_text_input_target(),
            self.focused_text_input_snapshot(),
        ) {
            let target = self
                .current_interaction_plan()
                .map(|interaction| native_text_visual_target(target, &interaction))
                .unwrap_or(target);
            let first_visible_row = self
                .text_edit
                .filter(|state| state.widget == target.widget)
                .map(|state| state.first_visible_visual_row)
                .unwrap_or(0);
            let horizontal_scroll_px = self
                .text_edit
                .filter(|state| state.widget == target.widget)
                .map(|state| state.horizontal_scroll_px)
                .unwrap_or(0);
            decorate_native_text_edit_visuals_in_viewport_with_backend(
                &mut plan,
                target,
                &value,
                selection,
                first_visible_row,
                horizontal_scroll_px,
                self.widget_text_wrap(target.widget),
                self.dpi,
                &self.text_shaping,
            );
        }
        let mut plan = self.compose_ime_preedit(plan);
        if let Some(interaction_plan) = self.current_interaction_plan() {
            decorate_native_focus_ring(&mut plan, &interaction_plan, self.focused_widget, self.dpi);
        }
        #[cfg(feature = "tooltip")]
        self.compose_tooltip(&mut plan);
        plan
    }

    #[cfg(feature = "tooltip")]
    fn compose_tooltip(&self, plan: &mut NativeDrawPlan) {
        let (Some(surface), Some(interaction)) = (self.surface, self.current_interaction_plan())
        else {
            return;
        };
        let Some(target) = self.tooltip.visible_target(&interaction) else {
            return;
        };
        let render = crate::zs_tooltip_render_plan(
            &target.spec,
            target.bounds,
            self.tooltip.anchor(),
            surface,
            crate::ZsTooltipPlatformStyle::current(),
            self.dpi,
        );
        let overlay = crate::zs_tooltip_native_draw_plan(&render, &target.spec);
        plan.commands.extend(overlay.commands);
    }

    pub(crate) fn transient_poll_interval_ms(&self) -> Option<u64> {
        let live_interval = self
            .live_view
            .as_ref()
            .and_then(SharedLiveViewRuntime::background_poll_interval_ms);
        let static_interval = self
            .ui_command_view
            .as_ref()
            .and_then(ViewNode::background_poll_interval_ms);
        let interval = live_interval.into_iter().chain(static_interval).min();
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

    pub(crate) fn refresh_transient_view(&mut self) -> NativeViewInputDispatchReport {
        self.refresh_transient_view_at(std::time::Instant::now())
    }

    fn refresh_transient_view_at(
        &mut self,
        now: std::time::Instant,
    ) -> NativeViewInputDispatchReport {
        #[cfg(not(any(feature = "tooltip", feature = "toast")))]
        let _ = now;
        let mut changed = false;
        if self
            .live_view
            .as_ref()
            .and_then(SharedLiveViewRuntime::background_poll_interval_ms)
            .is_some()
        {
            if let Some(runtime) = &self.live_view {
                let update = runtime.refresh();
                self.interaction_plan = Some(runtime.interaction_plan());
                changed |= update.redraw;
            }
        }
        changed |= self
            .ui_command_view
            .as_ref()
            .and_then(ViewNode::background_poll_interval_ms)
            .is_some();
        #[cfg(feature = "tooltip")]
        {
            changed |= self.tooltip.refresh(now);
        }
        #[cfg(feature = "toast")]
        if let Some((widget, toast)) = self.toast.take_expired(now) {
            let mut report = NativeViewInputDispatchReport {
                handled: true,
                toast_responded: true,
                hit_target_count: self.hit_target_count(),
                focused_widget: self.focused_widget.map(|widget| widget.0),
                ..NativeViewInputDispatchReport::default()
            };
            if changed {
                report.redraw_plan = self.current_composed_draw_plan();
            }
            return self.dispatch_view_event(
                ViewEvent::ToastResponded {
                    widget,
                    toast,
                    response: crate::ZsToastResponse::Dismissed(
                        crate::ZsToastDismissReason::Timeout,
                    ),
                },
                report,
            );
        }
        let report = NativeViewInputDispatchReport {
            handled: changed,
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            redraw_plan: changed.then(|| self.current_composed_draw_plan()).flatten(),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(feature = "toast")]
        self.sync_toast_runtime(now);
        report
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
        #[cfg(feature = "password-box")]
        if preedit.text.is_secure() {
            let mut committed = self
                .widget_password_value(preedit.widget)
                .unwrap_or_default();
            let (start, end) = preedit.replacement.clamp(committed.as_str()).ordered();
            let start_byte = char_to_byte_index(committed.as_str(), start);
            let end_byte = char_to_byte_index(committed.as_str(), end);
            committed
                .as_string_mut()
                .replace_range(start_byte..end_byte, preedit.text.as_str());
            let masked = crate::mask_password(committed.as_str());
            let mut decorated = false;
            for command in plan.commands.iter_mut().rev() {
                match command {
                    NativeDrawCommand::SecureText(command)
                        if rect_contains_rect(target.bounds, command.bounds) =>
                    {
                        command.replace_value(committed.clone());
                        decorated = true;
                        break;
                    }
                    NativeDrawCommand::Text(text)
                        if rect_contains_rect(target.bounds, text.bounds) =>
                    {
                        text.text = masked.clone();
                        decorated = true;
                        break;
                    }
                    _ => {}
                }
            }
            if decorated {
                plan.push(NativeDrawCommand::StrokeRect {
                    rect: target.bounds,
                    stroke: NativeDrawFill::Role(crate::ColorRole::Accent),
                    width: 2,
                });
            }
            return plan;
        }
        let committed = self.widget_text_value(preedit.widget).unwrap_or_default();
        let (start, end) = preedit.replacement.clamp(&committed).ordered();
        let start_byte = char_to_byte_index(&committed, start);
        let end_byte = char_to_byte_index(&committed, end);
        let mut composed = committed.clone();
        composed.replace_range(start_byte..end_byte, preedit.text.as_str());
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

    #[cfg(feature = "password-box")]
    fn compose_password_peek(&self, plan: &mut NativeDrawPlan) {
        let Some(widget) = self.password_peek else {
            return;
        };
        let Some(target) = self
            .current_interaction_plan()
            .and_then(|interaction| interaction.hit_target_for_widget(widget))
        else {
            return;
        };
        let Some(value) = self.widget_password_value(widget) else {
            return;
        };
        for command in plan.commands.iter_mut().rev() {
            let NativeDrawCommand::Text(text) = command else {
                continue;
            };
            if !rect_contains_rect(target.bounds, text.bounds) {
                continue;
            }
            let bounds = text.bounds;
            let style = text.style;
            *command = NativeDrawCommand::SecureText(crate::NativeDrawSecureTextCommand::new(
                value, bounds, style, true,
            ));
            break;
        }
    }

    fn ensure_text_edit_for_target(&mut self, target: crate::ViewHitTarget) {
        if !target.kind.accepts_text_input() {
            self.text_drag = None;
            return;
        }
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        state.clamp(&value);
        if target.kind != crate::ViewHitTargetKind::TextEditor
            || self.widget_text_wrap(target.widget) != crate::TextWrap::NoWrap
        {
            state.horizontal_scroll_px = 0;
        }
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
            .and_then(|plan| plan.focus_target_for_widget(widget))
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

    #[cfg(feature = "color-picker")]
    fn dispatch_color_picker_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: Point,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let Some(state) = self.widget_color_picker_state(target.widget) else {
            self.color_picker_drag = None;
            return report;
        };
        let root_bounds = self
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.into_iter().find(|candidate| {
                    candidate.widget == target.widget
                        && candidate.kind == crate::ViewHitTargetKind::ColorPicker
                })
            })
            .map(|target| target.bounds)
            .unwrap_or(target.bounds);
        let plan = self.surface.map_or_else(
            || {
                crate::zs_color_picker_render_plan(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::current(),
                    self.dpi,
                )
            },
            |viewport| {
                crate::zs_color_picker_render_plan_in_viewport(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::current(),
                    self.dpi,
                    viewport,
                )
            },
        );
        let (color, channel) = match target.kind {
            crate::ViewHitTargetKind::ColorPickerSpectrum => {
                (plan.spectrum_color_at(state, point), None)
            }
            crate::ViewHitTargetKind::ColorPickerHue => (plan.hue_color_at(state, point), None),
            crate::ViewHitTargetKind::ColorPickerChannel { channel } => {
                let Some(row) = plan.channels.iter().find(|row| row.channel == channel) else {
                    self.color_picker_drag = None;
                    return report;
                };
                let fraction =
                    point.x.saturating_sub(row.track.x) as f32 / row.track.width.max(1) as f32;
                let value = (fraction.clamp(0.0, 1.0) * 255.0).round() as u8;
                (Some(channel.with_value(state.color, value)), Some(channel))
            }
            _ => (None, None),
        };
        let Some(color) = color else {
            self.color_picker_drag = None;
            return report;
        };
        report.handled = true;
        report.color_picker_value = Some(color);
        report.color_picker_drag_active = self.color_picker_drag.is_some();
        if let Some(channel) = channel.filter(|channel| state.active_channel != *channel) {
            report.color_picker_channel_changed = true;
            report = self.dispatch_view_event(
                ViewEvent::ColorPickerChannelChanged {
                    widget: target.widget,
                    channel,
                },
                report,
            );
        }
        if color == state.color {
            return report;
        }
        report.color_picker_value_changed = true;
        self.dispatch_view_event(
            ViewEvent::ColorChanged {
                widget: target.widget,
                color,
            },
            report,
        )
    }

    fn activation_event(&self, target: crate::ViewHitTarget) -> ViewEvent {
        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let expanded = self
                .widget_color_picker_state(target.widget)
                .is_none_or(|state| !state.expanded);
            return ViewEvent::ColorPickerExpandedChanged {
                widget: target.widget,
                expanded,
            };
        }
        #[cfg(feature = "dialog")]
        if let crate::ViewHitTargetKind::ContentDialogButton { button } = target.kind {
            return ViewEvent::ContentDialogResponded {
                widget: target.widget,
                button,
            };
        }
        #[cfg(feature = "auto-suggest")]
        match target.kind {
            crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion } => {
                return ViewEvent::AutoSuggestSubmitted {
                    widget: target.widget,
                    suggestion: Some(suggestion),
                };
            }
            crate::ViewHitTargetKind::AutoSuggestSearch => {
                return ViewEvent::AutoSuggestSubmitted {
                    widget: target.widget,
                    suggestion: self
                        .widget_auto_suggest_state(target.widget)
                        .and_then(|state| state.highlighted),
                };
            }
            crate::ViewHitTargetKind::AutoSuggestClear => {
                return ViewEvent::AutoSuggestCleared {
                    widget: target.widget,
                };
            }
            _ => {}
        }
        #[cfg(feature = "number-box")]
        match target.kind {
            crate::ViewHitTargetKind::NumberBoxDecrement => {
                return ViewEvent::NumberBoxStep {
                    widget: target.widget,
                    steps: -1,
                    large: false,
                };
            }
            crate::ViewHitTargetKind::NumberBoxIncrement => {
                return ViewEvent::NumberBoxStep {
                    widget: target.widget,
                    steps: 1,
                    large: false,
                };
            }
            _ => {}
        }
        #[cfg(feature = "time-picker")]
        match target.kind {
            crate::ViewHitTargetKind::TimePickerChoice { value } => {
                return ViewEvent::TimeChanged {
                    widget: target.widget,
                    value,
                };
            }
            crate::ViewHitTargetKind::TimePicker => {
                let expanded = self
                    .widget_time_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                return ViewEvent::TimePickerExpandedChanged {
                    widget: target.widget,
                    expanded: !expanded,
                };
            }
            _ => {}
        }
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
        let toggles = matches!(
            target.kind,
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle
        );
        #[cfg(feature = "toggle-button")]
        let toggles = toggles || target.kind == crate::ViewHitTargetKind::ToggleButton;
        if toggles {
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
            .current_interaction_plan()?
            .hit_targets
            .into_iter()
            .rev()
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
                #[cfg(feature = "auto-suggest")]
                crate::ViewHitTargetKind::AutoSuggestBox => self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "combo")]
                crate::ViewHitTargetKind::ComboBox => self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded),
                #[cfg(feature = "date-picker")]
                crate::ViewHitTargetKind::DatePicker => self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "time-picker")]
                crate::ViewHitTargetKind::TimePicker => self
                    .widget_time_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                #[cfg(feature = "color-picker")]
                crate::ViewHitTargetKind::ColorPicker => self
                    .widget_color_picker_state(target.widget)
                    .is_some_and(|state| state.expanded),
                _ => false,
            }
        });
        if !should_dismiss {
            return report;
        }
        #[cfg(feature = "auto-suggest")]
        {
            report.auto_suggest_expanded_changed |=
                interaction_plan.hit_targets.iter().any(|target| {
                    Some(target.widget) != except
                        && target.kind == crate::ViewHitTargetKind::AutoSuggestBox
                        && self
                            .widget_auto_suggest_state(target.widget)
                            .is_some_and(|state| state.expanded)
                });
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
        #[cfg(feature = "color-picker")]
        {
            report.color_picker_expanded_changed |=
                interaction_plan.hit_targets.iter().any(|target| {
                    Some(target.widget) != except
                        && target.kind == crate::ViewHitTargetKind::ColorPicker
                        && self
                            .widget_color_picker_state(target.widget)
                            .is_some_and(|state| state.expanded)
                });
        }
        report.handled = true;
        self.dispatch_view_event(crate::ViewEvent::DismissPopupOverlays { except }, report)
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
        #[cfg(feature = "textbox")]
        let mut text_edit_commands = Vec::new();
        let (commands, ui_commands, quit_requested) = if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_event(&event);
            report.message_count += update.message_count;
            #[cfg(feature = "textbox")]
            text_edit_commands.extend(update.text_edit_commands.iter().copied());
            if update.redraw {
                report.redraw_plan = Some(live_view.draw_plan());
                report.hit_target_count = live_view.interaction_plan().hit_target_count();
            }
            (update.commands, update.ui_commands, update.quit_requested)
        } else {
            let mut event_cx = ViewEventCx::new();
            let typography_scale = self.typography_scale();
            if let Some(view) = &mut self.ui_command_view {
                view.event(&mut event_cx, &event);
                if let Some(surface) = self.surface {
                    let mut layout_cx = ViewLayoutCx::new(surface, self.dpi)
                        .with_typography_scale(typography_scale);
                    view.layout(&mut layout_cx);
                    let interaction_plan = view.interaction_plan();
                    report.hit_target_count = interaction_plan.hit_target_count();
                    self.interaction_plan = Some(interaction_plan);
                }
                let mut paint_cx = ViewPaintCx::new(self.dpi);
                paint_cx.set_typography_scale(typography_scale);
                view.paint(&mut paint_cx);
                report.redraw_plan = Some(paint_cx.into_plan());
            }
            let messages = event_cx.into_messages();
            report.message_count += messages.len();
            (Vec::new(), messages, false)
        };

        report.app_command_count += commands.len();
        report.ui_command_count += ui_commands.len();
        report.quit_requested |= quit_requested || commands.contains(&Command::Quit);
        let mut app_effect_executed = false;
        if self.defer_app_command_execution {
            self.pending_app_commands.extend(commands);
        } else {
            let app_executor = self.app_command_executor.clone();
            for command in commands {
                if let Some(executor) = &app_executor {
                    match executor.dispatch(command) {
                        Ok(_) => app_effect_executed = true,
                        Err(error) => report.errors.push(error.to_string()),
                    }
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
        if app_effect_executed {
            self.refresh_live_view_after_app_effect(&mut report);
        }
        self.reconcile_modal_focus(&mut report);
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
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            self.ime_preedit = None;
            report.focus_visual_changed = true;
        }
        self.sync_text_edit();
        #[cfg(feature = "textbox")]
        self.dispatch_text_edit_commands(text_edit_commands, &mut report);
        if let Some(plan) = report.redraw_plan.take() {
            report.redraw_plan = Some(self.compose_input_visuals(plan));
        }
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        report.ime_preedit_text = self
            .ime_preedit
            .as_ref()
            .map(|state| state.text.report_text());
        report.ime_selection = self.ime_preedit.as_ref().and_then(|state| state.selection);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    #[cfg(feature = "textbox")]
    fn dispatch_text_edit_commands(
        &mut self,
        commands: Vec<crate::ZsTextEditCommandRequest>,
        report: &mut NativeViewInputDispatchReport,
    ) {
        if commands.is_empty() {
            return;
        }
        if self.processing_text_edit_commands {
            report
                .errors
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
        report: &mut NativeViewInputDispatchReport,
    ) {
        let target = match request.widget {
            Some(widget) => self
                .current_interaction_plan()
                .and_then(|plan| plan.hit_target_for_widget(widget)),
            None => self.focused_text_input_target(),
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
                report.errors.push(error.to_string());
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
        report.text_selection_changed |= result.selection_changed;
        report.text_clipboard_read_count += usize::from(result.clipboard_read);
        report.text_clipboard_write_count += usize::from(result.clipboard_write);
        report.text_undo_count += usize::from(result.undo_applied);

        let event = if result.text_changed {
            Some(ViewEvent::TextEdited {
                widget,
                value,
                selection: state.selection.into(),
            })
        } else if result.selection_changed {
            Some(ViewEvent::TextSelectionChanged {
                widget,
                selection: state.selection.into(),
            })
        } else {
            None
        };
        if let Some(event) = event {
            *report = self.dispatch_view_event(event, std::mem::take(report));
        }
    }

    pub(crate) fn dispatch_app_command(
        &mut self,
        command: Command,
    ) -> NativeViewInputDispatchReport {
        let mut report = NativeViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            focused_widget: self.focused_widget.map(|widget| widget.0),
            ..NativeViewInputDispatchReport::default()
        };
        #[cfg(feature = "textbox")]
        let mut text_edit_commands = Vec::new();

        let update = self
            .live_view
            .as_ref()
            .map(|runtime| runtime.dispatch_app_command(&command));
        let mut app_effect_executed = false;
        if let Some(update) = update.filter(|update| update.message_count > 0) {
            #[cfg(feature = "textbox")]
            text_edit_commands.extend(update.text_edit_commands.iter().copied());
            report.handled = true;
            report.message_count = update.message_count;
            report.app_command_count = update.commands.len();
            report.ui_command_count = update.ui_commands.len();
            report.quit_requested =
                update.quit_requested || update.commands.contains(&Command::Quit);

            if self.defer_app_command_execution {
                self.pending_app_commands.extend(update.commands);
            } else {
                let app_executor = self.app_command_executor.clone();
                for effect in update.commands {
                    if let Some(executor) = &app_executor {
                        match executor.dispatch(effect) {
                            Ok(_) => app_effect_executed = true,
                            Err(error) => report.errors.push(error.to_string()),
                        }
                    }
                }
            }
            let ui_executor = self.ui_command_executor.clone();
            for effect in update.ui_commands {
                report.ui_command_ids.push(effect.id.0);
                if let Some(executor) = &ui_executor {
                    if let Err(error) = executor.dispatch(effect) {
                        report.errors.push(error.to_string());
                    }
                }
            }
            if let Some(runtime) = &self.live_view {
                self.interaction_plan = Some(runtime.interaction_plan());
                if update.redraw {
                    report.redraw_plan = Some(runtime.draw_plan());
                }
            }
        } else if command == Command::Quit {
            report.handled = true;
            report.quit_requested = true;
        } else if self.defer_app_command_execution {
            report.app_command_count = 1;
            self.pending_app_commands.push(command);
        } else if let Some(executor) = self.app_command_executor.clone() {
            report.app_command_count = 1;
            match executor.dispatch(command) {
                Ok(_) => {
                    report.handled = true;
                    app_effect_executed = true;
                }
                Err(error) => report.errors.push(error.to_string()),
            }
        }

        if app_effect_executed {
            self.refresh_live_view_after_app_effect(&mut report);
        }

        self.reconcile_modal_focus(&mut report);
        if self.focused_widget.is_some_and(|widget| {
            self.current_interaction_plan()
                .map_or(true, |plan| plan.hit_target_for_widget(widget).is_none())
        }) {
            self.focused_widget = None;
            self.text_edit = None;
            self.text_drag = None;
            self.ime_preedit = None;
            report.focus_visual_changed = true;
        }
        self.sync_text_edit();
        #[cfg(feature = "textbox")]
        self.dispatch_text_edit_commands(text_edit_commands, &mut report);
        if let Some(plan) = report.redraw_plan.take() {
            report.redraw_plan = Some(self.compose_input_visuals(plan));
        }
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        report.ime_preedit_text = self
            .ime_preedit
            .as_ref()
            .map(|state| state.text.report_text());
        report.ime_selection = self.ime_preedit.as_ref().and_then(|state| state.selection);
        report.ime_caret_rect = self.text_input_caret_rect();
        self.populate_text_report(&mut report);
        report
    }

    pub(crate) fn dispatch_window_close_requested(&mut self) -> NativeViewInputDispatchReport {
        let Some(command) = self.window_close_request_command.clone() else {
            return NativeViewInputDispatchReport {
                window_close_request_count: 1,
                hit_target_count: self.hit_target_count(),
                ..NativeViewInputDispatchReport::default()
            };
        };
        let mut report = self.dispatch_app_command(command);
        report.window_close_request_count = 1;
        if report.handled && !report.quit_requested {
            report.window_close_veto_count = 1;
        }
        report
    }

    pub(crate) fn defer_app_command_execution(&mut self) {
        self.defer_app_command_execution = true;
    }

    pub(crate) fn take_pending_app_command_dispatch(
        &mut self,
    ) -> (Option<SharedAppCommandExecutor>, Vec<Command>) {
        (
            self.app_command_executor.clone(),
            std::mem::take(&mut self.pending_app_commands),
        )
    }

    pub(crate) fn refresh_live_view_after_app_effect(
        &mut self,
        report: &mut NativeViewInputDispatchReport,
    ) {
        let Some(runtime) = &self.live_view else {
            return;
        };
        let update = runtime.refresh();
        if !update.redraw {
            return;
        }
        self.interaction_plan = Some(runtime.interaction_plan());
        report.hit_target_count = self.hit_target_count();
        report.redraw_plan = Some(runtime.draw_plan());
    }

    pub(crate) fn backend_attachment(&self) -> Option<NativeViewInputBackendAttachment> {
        let source = if let Some(runtime) = &self.live_view {
            NativeViewInputBackendSource::Live(runtime.clone())
        } else {
            NativeViewInputBackendSource::Static {
                interaction_plan: self.interaction_plan.clone()?,
                ui_command_view: self.ui_command_view.clone()?,
            }
        };
        Some(NativeViewInputBackendAttachment {
            source,
            resource_policy: self.resource_policy,
            window_close_request_command: self.window_close_request_command.clone(),
            app_command_executor: self.app_command_executor.clone(),
            ui_command_executor: self.ui_command_executor.clone(),
        })
    }
}

#[allow(dead_code)]
pub(crate) fn dispatch_deferred_native_view_app_commands(
    report: &mut NativeViewInputDispatchReport,
    executor: Option<SharedAppCommandExecutor>,
    commands: Vec<Command>,
) -> bool {
    let Some(executor) = executor else {
        return false;
    };
    let mut executed = false;
    for command in commands {
        match executor.dispatch(command) {
            Ok(_) => {
                report.handled = true;
                executed = true;
            }
            Err(error) => report.errors.push(error.to_string()),
        }
    }
    executed
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x.saturating_add(inner.width) <= outer.x.saturating_add(outer.width)
        && inner.y.saturating_add(inner.height) <= outer.y.saturating_add(outer.height)
}

#[allow(dead_code)]
pub(crate) fn record_draw_plan_smoke(
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
pub(crate) fn record_native_view_input_smoke(
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

#[allow(dead_code)]
pub(crate) fn record_native_view_input_reports(
    report: &mut NativeWindowSmokeRunReport,
    inputs: &[NativeViewSmokeInput],
    dispatches: &[NativeViewInputDispatchReport],
    backend: &str,
) {
    let mut dispatch_index: usize = 0;
    for input in inputs {
        let dispatch_count = match input {
            NativeViewSmokeInput::Click(_) => 2,
            NativeViewSmokeInput::Drag { .. } => 3,
            _ => 1,
        };
        let end = dispatch_index
            .saturating_add(dispatch_count)
            .min(dispatches.len());
        let input_dispatches = &dispatches[dispatch_index..end];
        dispatch_index = end;
        let handled = input_dispatches.iter().any(|dispatch| dispatch.handled);

        match input {
            NativeViewSmokeInput::Move(_) => report.native_view_pointer_move_count += 1,
            NativeViewSmokeInput::Click(_) => {
                report.native_view_click_count += 1;
                report.native_view_pointer_down_count += 1;
                report.native_view_pointer_up_count += 1;
            }
            NativeViewSmokeInput::Drag { .. } => {
                report.native_view_pointer_down_count += 1;
                report.native_view_pointer_move_count += 1;
                report.native_view_pointer_up_count += 1;
                if input_dispatches.iter().any(|dispatch| {
                    dispatch.text_drag_active || dispatch.text_drag_scroll_count > 0
                }) {
                    report.native_view_text_drag_count += 1;
                }
            }
            NativeViewSmokeInput::Text(_) => {
                report.native_view_text_input_count += usize::from(handled);
            }
            NativeViewSmokeInput::KeyDown(key) => {
                report.native_view_key_down_count += 1;
                if handled {
                    if matches!(
                        key,
                        NativeViewKey::Up
                            | NativeViewKey::Down
                            | NativeViewKey::Left
                            | NativeViewKey::Right
                            | NativeViewKey::Home
                            | NativeViewKey::End
                            | NativeViewKey::PageUp
                            | NativeViewKey::PageDown
                    ) {
                        report.native_view_text_navigation_count += 1;
                    }
                    if matches!(key, NativeViewKey::Enter | NativeViewKey::Space) {
                        report.native_view_keyboard_activation_count += 1;
                    }
                    if *key == NativeViewKey::Tab {
                        report.native_view_focus_traversal_count += 1;
                    }
                } else {
                    report.native_view_unhandled_key_count += 1;
                }
            }
            NativeViewSmokeInput::Scroll { .. } => {
                if handled {
                    report.native_view_scroll_count += 1;
                } else {
                    report.native_view_unhandled_scroll_count += 1;
                }
            }
            NativeViewSmokeInput::WindowCloseRequest => {}
        }
        report.events.push(format!(
            "{backend}_proof_input:{}:{}",
            match input {
                NativeViewSmokeInput::Move(_) => "move",
                NativeViewSmokeInput::Click(_) => "click",
                NativeViewSmokeInput::Drag { .. } => "drag",
                NativeViewSmokeInput::Text(_) => "text",
                NativeViewSmokeInput::KeyDown(_) => "key_down",
                NativeViewSmokeInput::Scroll { .. } => "scroll",
                NativeViewSmokeInput::WindowCloseRequest => "window_close_request",
            },
            if handled { "handled" } else { "unhandled" }
        ));
    }

    for dispatch in dispatches {
        report.native_view_event_count += usize::from(dispatch.handled);
        report.native_view_message_count += dispatch.message_count;
        report.native_view_app_command_count += dispatch.app_command_count;
        report.native_view_ui_command_count += dispatch.ui_command_count;
        report
            .native_view_ui_command_ids
            .extend(dispatch.ui_command_ids.iter().copied());
        report.native_view_window_close_request_count += dispatch.window_close_request_count;
        report.native_view_window_close_veto_count += dispatch.window_close_veto_count;
        report.native_view_hit_target_count = report
            .native_view_hit_target_count
            .max(dispatch.hit_target_count);
        report.native_view_focused_widget = dispatch.focused_widget;
        report.native_view_focus_count += usize::from(dispatch.focus_visual_changed);
        report.native_view_focus_visual_count += usize::from(dispatch.focus_visual_changed);
        report.native_view_text_selection_change_count +=
            usize::from(dispatch.text_selection_changed);
        report.native_view_text_caret = dispatch.text_caret.or(report.native_view_text_caret);
        report.native_view_text_drag_scroll_count += dispatch.text_drag_scroll_count;
        report.native_view_quit_requested |= dispatch.quit_requested;
        report
            .native_view_app_command_errors
            .extend(dispatch.errors.iter().cloned());
        #[cfg(feature = "textbox")]
        {
            report.native_view_text_edit_command_count += dispatch.text_edit_command_count;
            report.native_view_text_clipboard_read_count += dispatch.text_clipboard_read_count;
            report.native_view_text_clipboard_write_count += dispatch.text_clipboard_write_count;
            report.native_view_text_undo_count += dispatch.text_undo_count;
        }
        #[cfg(feature = "slider")]
        {
            report.native_view_slider_value_change_count +=
                usize::from(dispatch.slider_value_changed);
            report.native_view_slider_drag_count += usize::from(dispatch.slider_drag_active);
        }
        #[cfg(feature = "color-picker")]
        {
            report.native_view_color_picker_value_change_count +=
                usize::from(dispatch.color_picker_value_changed);
            report.native_view_color_picker_channel_change_count +=
                usize::from(dispatch.color_picker_channel_changed);
            report.native_view_color_picker_expanded_change_count +=
                usize::from(dispatch.color_picker_expanded_changed);
            report.native_view_color_picker_drag_count +=
                usize::from(dispatch.color_picker_drag_active);
        }
        #[cfg(feature = "radio")]
        {
            report.native_view_radio_selection_count +=
                usize::from(dispatch.radio_selection_changed);
            report.native_view_radio_keyboard_selection_count +=
                usize::from(dispatch.radio_keyboard_selection_changed);
            report.native_view_radio_keyboard_focus_only_count +=
                usize::from(dispatch.radio_keyboard_focus_only);
        }
        #[cfg(feature = "auto-suggest")]
        {
            report.native_view_auto_suggest_expanded_change_count +=
                usize::from(dispatch.auto_suggest_expanded_changed);
            report.native_view_auto_suggest_highlight_change_count +=
                usize::from(dispatch.auto_suggest_highlight_changed);
            report.native_view_auto_suggest_submit_count +=
                usize::from(dispatch.auto_suggest_submitted);
            report.native_view_auto_suggest_clear_count +=
                usize::from(dispatch.auto_suggest_cleared);
        }
        #[cfg(feature = "tree")]
        {
            report.native_view_tree_expansion_change_count +=
                usize::from(dispatch.tree_expansion_changed);
            report.native_view_tree_selection_count += usize::from(dispatch.tree_selection_changed);
            report.native_view_tree_invoke_count += usize::from(dispatch.tree_invoked);
        }
        #[cfg(feature = "grid-view")]
        {
            report.native_view_grid_view_selection_count +=
                usize::from(dispatch.grid_view_selection_changed);
            report.native_view_grid_view_invoke_count += usize::from(dispatch.grid_view_invoked);
        }
        #[cfg(feature = "table")]
        {
            report.native_view_table_sort_count += usize::from(dispatch.table_sort_changed);
            report.native_view_table_selection_count +=
                usize::from(dispatch.table_selection_changed);
            report.native_view_table_invoke_count += usize::from(dispatch.table_invoked);
        }
        #[cfg(feature = "dialog")]
        {
            report.native_view_content_dialog_focus_count +=
                usize::from(dispatch.content_dialog_focus_changed);
            report.native_view_content_dialog_response_count +=
                usize::from(dispatch.content_dialog_responded);
        }
        #[cfg(feature = "command-palette")]
        {
            report.native_view_command_palette_query_change_count +=
                usize::from(dispatch.command_palette_query_changed);
            report.native_view_command_palette_highlight_change_count +=
                usize::from(dispatch.command_palette_highlight_changed);
            report.native_view_command_palette_invoke_count +=
                usize::from(dispatch.command_palette_invoked);
            report.native_view_command_palette_open_change_count +=
                usize::from(dispatch.command_palette_open_changed);
            report.native_view_command_palette_clear_count +=
                usize::from(dispatch.command_palette_cleared);
        }
        #[cfg(feature = "toast")]
        {
            report.native_view_toast_focus_count += usize::from(dispatch.toast_focus_changed);
            report.native_view_toast_response_count += usize::from(dispatch.toast_responded);
        }
        #[cfg(feature = "info-bar")]
        {
            report.native_view_info_bar_focus_count += usize::from(dispatch.info_bar_focus_changed);
            report.native_view_info_bar_event_count +=
                usize::from(dispatch.info_bar_event.is_some());
        }
        #[cfg(feature = "teaching-tip")]
        {
            report.native_view_teaching_tip_focus_count +=
                usize::from(dispatch.teaching_tip_focus_changed);
            report.native_view_teaching_tip_response_count +=
                usize::from(dispatch.teaching_tip_response.is_some());
        }
        #[cfg(feature = "breadcrumb")]
        {
            report.native_view_breadcrumb_focus_count +=
                usize::from(dispatch.breadcrumb_focus_changed);
            report.native_view_breadcrumb_expanded_change_count +=
                usize::from(dispatch.breadcrumb_expanded_changed);
            report.native_view_breadcrumb_selection_count +=
                usize::from(dispatch.breadcrumb_selection.is_some());
        }
        #[cfg(feature = "combo")]
        {
            report.native_view_combo_expanded_change_count +=
                usize::from(dispatch.combo_expanded_changed);
            report.native_view_combo_selection_count +=
                usize::from(dispatch.combo_selection_changed);
            report.native_view_combo_keyboard_selection_count +=
                usize::from(dispatch.combo_keyboard_selection_changed);
            report.native_view_combo_type_ahead_match_count +=
                usize::from(dispatch.combo_type_ahead_matched);
            report.native_view_combo_scroll_count += usize::from(dispatch.combo_scrolled);
        }
        #[cfg(feature = "tabs")]
        {
            report.native_view_tab_selection_count += usize::from(dispatch.tab_selection_changed);
            report.native_view_tab_keyboard_selection_count +=
                usize::from(dispatch.tab_keyboard_selection_changed);
            report.native_view_tab_keyboard_focus_only_count +=
                usize::from(dispatch.tab_keyboard_focus_only);
        }
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
    resource_policy: NativeWindowResourcePolicy,
    window_close_request_command: Option<Command>,
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
            && self.resource_policy == other.resource_policy
            && self.window_close_request_command == other.window_close_request_command
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
            resource_policy: NativeWindowResourcePolicy::default(),
            window_close_request_command: None,
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

    pub fn resource_policy(mut self, policy: NativeWindowResourcePolicy) -> Self {
        self.resource_policy = policy;
        self
    }

    /// Releases a stateful View and transient UI caches while the native
    /// window is hidden or minimized. Application state, command routing and
    /// app-owned monitoring services stay alive.
    pub fn release_view_when_hidden(self) -> Self {
        self.resource_policy(NativeWindowResourcePolicy::ReleaseViewWhenHidden)
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

    /// Routes a native title-bar close request through the stateful view's
    /// typed application-command mapper. An unmapped command keeps normal OS
    /// behavior; a mapped update approves closing by calling [`AppCx::quit`]
    /// and otherwise vetoes the request.
    pub fn on_close_requested(mut self, command: Command) -> Self {
        self.window_close_request_command = Some(command);
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

    /// Installs a stateful view whose typed update function also receives
    /// commands produced by the native window menu.
    pub fn stateful_view_with_app_commands<State, Msg, ViewFn, UpdateFn, CommandFn>(
        mut self,
        state: State,
        view_fn: ViewFn,
        update_fn: UpdateFn,
        command_fn: CommandFn,
    ) -> Self
    where
        State: Send + 'static,
        Msg: Clone + Send + 'static,
        ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
        UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
        CommandFn: Fn(&Command) -> Option<Msg> + Send + 'static,
    {
        let runtime = live_view_runtime_with_app_commands(
            state,
            view_fn,
            update_fn,
            command_fn,
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

    pub const fn native_resource_policy(&self) -> NativeWindowResourcePolicy {
        self.resource_policy
    }

    pub fn native_window_close_request_command(&self) -> Option<&Command> {
        self.window_close_request_command.as_ref()
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
        crate::desktop_runtime::run_smoke_event_loop(
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
            self.resource_policy,
            self.window_close_request_command.clone(),
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

    pub fn resource_policy(mut self, policy: NativeWindowResourcePolicy) -> Self {
        self.inner = self.inner.resource_policy(policy);
        self
    }

    pub fn release_view_when_hidden(mut self) -> Self {
        self.inner = self.inner.release_view_when_hidden();
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

    pub fn on_close_requested(mut self, command: Command) -> Self {
        self.inner = self.inner.on_close_requested(command);
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

    pub fn stateful_view_with_app_commands<State, Msg, ViewFn, UpdateFn, CommandFn>(
        self,
        state: State,
        view_fn: ViewFn,
        update_fn: UpdateFn,
        command_fn: CommandFn,
    ) -> TypedNativeWindowBuilder<NativeWindowContentReady>
    where
        State: Send + 'static,
        Msg: Clone + Send + 'static,
        ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
        UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
        CommandFn: Fn(&Command) -> Option<Msg> + Send + 'static,
    {
        TypedNativeWindowBuilder::ready(
            self.inner
                .stateful_view_with_app_commands(state, view_fn, update_fn, command_fn),
        )
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

    pub fn native_window_close_request_command(&self) -> Option<&Command> {
        self.inner.native_window_close_request_command()
    }

    pub const fn native_resource_policy(&self) -> NativeWindowResourcePolicy {
        self.inner.native_resource_policy()
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

pub(crate) fn menu_command_count(menu: &MenuSpec) -> usize {
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
        crate::desktop_runtime::read_clipboard()
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        crate::desktop_runtime::write_clipboard(data)
    }

    fn open_file_picker(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<String>>> {
        if let Some(result) = crate::desktop_runtime::open_file_dialog(spec) {
            return result.map(|selection| {
                selection.map(|paths| {
                    paths
                        .into_iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect()
                })
            });
        }
        self.inner.open_file_picker(spec)
    }

    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        self.inner.show_native_dialog(spec)
    }

    fn poll_event(&mut self) -> ZsuiResult<Option<AppEvent>> {
        self.inner.poll_event()
    }

    fn run_event_loop(&mut self) -> ZsuiResult<()> {
        crate::desktop_runtime::run_event_loop(
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
        crate::desktop_runtime::save_file_dialog(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "textbox")]
    fn editor_height_for_visible_rows(rows: u32) -> u32 {
        let line_height = crate::TextRole::Body
            .metrics_for(crate::ZsTypographyPlatformStyle::current())
            .line_height;
        16_u32.saturating_add((line_height * rows.max(1) as f32).round() as u32)
    }

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
    fn native_view_backend_attachment_is_platform_neutral() {
        #[derive(Clone)]
        enum Msg {
            Save,
        }

        let static_builder = native_window("Static Backend Attachment")
            .release_view_when_hidden()
            .ui_command_view(crate::button("Save").id(crate::WidgetId::new(41)));
        let static_attachment = static_builder
            .native_view_input_runtime()
            .backend_attachment()
            .expect("static typed View should expose a backend attachment");
        assert_eq!(
            static_attachment.resource_policy,
            NativeWindowResourcePolicy::ReleaseViewWhenHidden
        );
        match static_attachment.source {
            NativeViewInputBackendSource::Static {
                interaction_plan,
                ui_command_view,
            } => {
                assert_eq!(interaction_plan.hit_target_count(), 1);
                assert_eq!(ui_command_view.interaction_plan().hit_target_count(), 1);
            }
            NativeViewInputBackendSource::Live(_) => {
                panic!("static typed View must not be lowered as a live source")
            }
        }

        let live_builder = native_window("Live Backend Attachment").stateful_view(
            false,
            |saved| {
                crate::button(if *saved { "Saved" } else { "Save" })
                    .id(crate::WidgetId::new(42))
                    .on_click(Msg::Save)
            },
            |saved, message, _cx| match message {
                Msg::Save => *saved = true,
            },
        );
        let live_attachment = live_builder
            .native_view_input_runtime()
            .backend_attachment()
            .expect("stateful View should expose a backend attachment");
        match live_attachment.source {
            NativeViewInputBackendSource::Live(runtime) => {
                assert_eq!(runtime.interaction_plan().hit_target_count(), 1)
            }
            NativeViewInputBackendSource::Static { .. } => {
                panic!("stateful View must remain a live backend source")
            }
        }
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_window_builder_stateful_view_rebuilds_after_update() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        #[derive(Clone)]
        enum Msg {
            Increment,
        }
        struct State {
            count: u32,
        }

        let button_id = crate::WidgetId::new(70);
        let view_build_count = Arc::new(AtomicUsize::new(0));
        let build_counter = Arc::clone(&view_build_count);
        let builder = native_window("Stateful View").size(360, 220).stateful_view(
            State { count: 0 },
            move |state| {
                build_counter.fetch_add(1, Ordering::SeqCst);
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
        assert_eq!(view_build_count.load(Ordering::SeqCst), 1);

        assert!(runtime.set_surface(
            crate::Rect {
                x: 0,
                y: 0,
                width: 340,
                height: 200,
            },
            crate::Dpi::standard(),
        ));
        assert_eq!(
            view_build_count.load(Ordering::SeqCst),
            1,
            "surface-only relayout must reuse the existing tree"
        );

        let update = runtime.dispatch_event(&ViewEvent::Click { widget: button_id });

        assert!(update.redraw);
        assert_eq!(update.revision, 2);
        assert_eq!(view_build_count.load(Ordering::SeqCst), 2);
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Count: 1"
        )));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn hidden_window_resource_policy_releases_and_rebuilds_the_view() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        #[derive(Clone)]
        enum Msg {
            Increment,
        }

        let builds = Arc::new(AtomicUsize::new(0));
        let build_counter = Arc::clone(&builds);
        let builder = native_window("Hibernating View")
            .release_view_when_hidden()
            .stateful_view(
                0_u32,
                move |count| {
                    build_counter.fetch_add(1, Ordering::SeqCst);
                    crate::column([
                        crate::text(format!("Count: {count}")),
                        crate::button("Increment")
                            .id(crate::WidgetId::new(801))
                            .on_click(Msg::Increment),
                    ])
                },
                |count, message, _cx| match message {
                    Msg::Increment => *count += 1,
                },
            );
        let live_view = builder.native_live_view_runtime().unwrap().clone();
        let mut runtime = builder.native_view_input_runtime();

        assert_eq!(
            builder.native_resource_policy(),
            NativeWindowResourcePolicy::ReleaseViewWhenHidden
        );
        assert_eq!(builds.load(Ordering::SeqCst), 1);
        assert_eq!(runtime.hit_target_count(), 1);
        assert!(runtime.suspend_view_when_hidden());
        assert!(live_view.is_suspended());
        assert_eq!(live_view.draw_plan().command_count(), 0);
        assert_eq!(runtime.hit_target_count(), 0);
        assert!(!runtime.suspend_view_when_hidden());

        let plan = runtime
            .resume_view_when_visible()
            .expect("showing the window should rebuild the released view");
        assert!(!live_view.is_suspended());
        assert_eq!(builds.load(Ordering::SeqCst), 2);
        assert_eq!(runtime.hit_target_count(), 1);
        assert!(plan.command_count() > 0);
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_window_menu_command_reenters_typed_stateful_update() {
        #[derive(Clone)]
        enum Msg {
            Open,
        }
        struct State {
            status: &'static str,
        }

        let builder = native_window("Menu State")
            .size(360, 220)
            .stateful_view_with_app_commands(
                State { status: "Ready" },
                |state| crate::text::<Msg>(state.status),
                |state, message, _cx| match message {
                    Msg::Open => state.status = "Opened from menu",
                },
                |command| match command {
                    Command::Custom { id, .. } if id == "document.open" => Some(Msg::Open),
                    _ => None,
                },
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful view should keep a live runtime")
            .clone();
        let mut input = builder.native_view_input_runtime();

        let report = input.dispatch_app_command(Command::custom("document.open"));

        assert!(report.handled);
        assert_eq!(report.message_count, 1);
        assert!(report.redraw_plan.is_some());
        assert_eq!(runtime.revision(), 1);
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Opened from menu"
        )));
    }

    #[cfg(feature = "label")]
    #[test]
    fn native_window_close_request_uses_typed_update_to_veto_or_approve() {
        #[derive(Clone)]
        enum Msg {
            Close,
        }
        struct State {
            dirty: bool,
        }
        let close_command = Command::custom("document.close");
        let build = |dirty| {
            native_window("Close Request")
                .on_close_requested(close_command.clone())
                .stateful_view_with_app_commands(
                    State { dirty },
                    |_state| crate::text::<Msg>("Document"),
                    |state, message, cx| match message {
                        Msg::Close if !state.dirty => cx.quit(),
                        Msg::Close => {}
                    },
                    |command| (command == &Command::custom("document.close")).then_some(Msg::Close),
                )
        };

        let dirty = build(true);
        assert_eq!(
            dirty.native_window_close_request_command(),
            Some(&close_command)
        );
        let dirty_report = dirty
            .native_view_input_runtime()
            .dispatch_window_close_requested();
        assert!(dirty_report.handled);
        assert!(!dirty_report.quit_requested);
        assert_eq!(dirty_report.window_close_request_count, 1);
        assert_eq!(dirty_report.window_close_veto_count, 1);

        let clean_report = build(false)
            .native_view_input_runtime()
            .dispatch_window_close_requested();
        assert!(clean_report.handled);
        assert!(clean_report.quit_requested);
        assert_eq!(clean_report.window_close_request_count, 1);
        assert_eq!(clean_report.window_close_veto_count, 0);
    }

    #[cfg(all(feature = "dialog", feature = "textbox"))]
    #[test]
    fn content_dialog_opened_by_close_request_preserves_overlay_text_and_restores_focus() {
        #[derive(Clone)]
        enum Msg {
            Close,
            Responded(crate::ZsContentDialogResult),
        }
        struct State {
            pending: bool,
        }

        let editor = crate::WidgetId::new(970);
        let dialog = crate::WidgetId::new(971);
        let close_command = Command::custom("document.close-with-dialog");
        let builder = native_window("Dialog Focus")
            .size(640, 400)
            .on_close_requested(close_command.clone())
            .stateful_view_with_app_commands(
                State { pending: false },
                move |state| {
                    crate::content_dialog(
                        dialog,
                        state.pending,
                        crate::ZsContentDialogSpec::new(
                            "Save the document before closing?",
                            "Cancel",
                        )
                        .title("Unsaved changes")
                        .primary_button("Save")
                        .secondary_button("Discard")
                        .default_button(crate::ZsContentDialogButton::Primary),
                        crate::text_editor::<Msg>("Document body").id(editor),
                    )
                    .on_dialog_result(Msg::Responded)
                },
                |state, message, _cx| match message {
                    Msg::Close => state.pending = true,
                    Msg::Responded(_result) => state.pending = false,
                },
                move |command| (command == &close_command).then_some(Msg::Close),
            );
        let editor_target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("editor hit target");
        let mut runtime = builder.native_view_input_runtime();
        let focused = runtime.dispatch_pointer_click(Point {
            x: editor_target.bounds.x + 12,
            y: editor_target.bounds.y + 12,
        });
        assert_eq!(focused.focused_widget, Some(editor.0));

        let opened = runtime.dispatch_window_close_requested();
        assert!(opened.handled);
        assert_eq!(opened.window_close_veto_count, 1);
        assert_eq!(opened.focused_widget, Some(dialog.0));
        let draw = opened.redraw_plan.expect("dialog redraw plan");
        for expected in [
            "Unsaved changes",
            "Save the document before closing?",
            "Save",
            "Discard",
            "Cancel",
        ] {
            assert!(
                draw.commands.iter().any(|command| {
                    matches!(command, crate::NativeDrawCommand::Text(text) if text.text == expected)
                }),
                "missing dialog text: {expected}"
            );
        }

        let closed = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(closed.content_dialog_responded);
        assert_eq!(closed.focused_widget, Some(editor.0));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn app_command_effect_refreshes_shared_state_after_executor_returns() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        enum Msg {
            Save,
        }

        let state = Arc::new(Mutex::new("Ready".to_string()));
        let executor_state = state.clone();
        let builder = native_window("External Effect")
            .size(360, 220)
            .stateful_view_with_app_commands(
                state,
                |state| {
                    crate::text::<Msg>(
                        state
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone(),
                    )
                },
                |_state, message, cx| match message {
                    Msg::Save => cx.command(Command::custom("effect.save")),
                },
                |command| match command {
                    Command::Custom { id, .. } if id == "document.save" => Some(Msg::Save),
                    _ => None,
                },
            )
            .app_command_executor(move |command| {
                assert_eq!(command, Command::custom("effect.save"));
                *executor_state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    "Saved externally".to_string();
                Ok(Vec::new())
            });
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful view should keep a live runtime")
            .clone();
        let mut input = builder.native_view_input_runtime();

        let report = input.dispatch_app_command(Command::custom("document.save"));

        assert!(report.handled);
        assert_eq!(report.message_count, 1);
        assert_eq!(runtime.revision(), 2);
        assert!(report
            .redraw_plan
            .expect("external effect should request a refreshed draw plan")
            .commands
            .iter()
            .any(|command| matches!(
                command,
                crate::NativeDrawCommand::Text(text) if text.text == "Saved externally"
            )));
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn native_hosts_can_defer_modal_app_effects_until_runtime_borrow_is_released() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        enum Msg {
            Open,
        }

        let state = Arc::new(Mutex::new("Ready".to_string()));
        let executor_state = state.clone();
        let executor = SharedAppCommandExecutor::new(move |command| {
            assert_eq!(command, Command::custom("effect.open"));
            *executor_state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = "Opened externally".to_string();
            Ok(Vec::new())
        });
        let builder = native_window("Deferred Effect")
            .size(360, 220)
            .stateful_view_with_app_commands(
                state.clone(),
                |state| {
                    crate::text::<Msg>(
                        state
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone(),
                    )
                },
                |_state, message, cx| match message {
                    Msg::Open => cx.command(Command::custom("effect.open")),
                },
                |command| match command {
                    Command::Custom { id, .. } if id == "document.open" => Some(Msg::Open),
                    _ => None,
                },
            )
            .shared_app_command_executor(executor.clone());
        let mut input = builder.native_view_input_runtime();
        input.defer_app_command_execution();

        let mut report = input.dispatch_app_command(Command::custom("document.open"));

        assert_eq!(*state.lock().unwrap(), "Ready");
        assert_eq!(executor.report().executed_count, 0);
        let (pending_executor, commands) = input.take_pending_app_command_dispatch();
        assert_eq!(commands, vec![Command::custom("effect.open")]);
        assert!(dispatch_deferred_native_view_app_commands(
            &mut report,
            pending_executor,
            commands,
        ));
        input.refresh_live_view_after_app_effect(&mut report);

        assert_eq!(executor.report().executed_count, 1);
        assert!(report
            .redraw_plan
            .expect("deferred effect should refresh after executor returns")
            .commands
            .iter()
            .any(|command| matches!(
                command,
                crate::NativeDrawCommand::Text(text) if text.text == "Opened externally"
            )));
    }

    #[cfg(all(feature = "tooltip", feature = "button"))]
    #[test]
    fn native_view_runtime_opens_delayed_tooltip_as_noninteractive_overlay() {
        let widget = crate::WidgetId::new(502);
        let builder = native_window("Tooltip runtime")
            .size(320, 180)
            .ui_command_view(
                crate::button("Save")
                    .id(widget)
                    .tooltip_spec(crate::ZsTooltipSpec::new("Save document").open_delay_ms(100)),
            );
        let target = builder
            .view_interaction_plan
            .as_ref()
            .and_then(|plan| plan.tooltip_for_widget(widget))
            .expect("tooltip owner should be present in interaction metadata");
        let point = Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        };
        let start = std::time::Instant::now();
        let mut runtime = builder.native_view_input_runtime();

        let hover = runtime.dispatch_pointer_move_at(point, start);
        let early = runtime.refresh_transient_view_at(start + std::time::Duration::from_millis(99));
        let shown =
            runtime.refresh_transient_view_at(start + std::time::Duration::from_millis(100));

        assert!(hover.handled);
        assert!(hover.pointer_visual_changed);
        assert!(early.redraw_plan.is_none());
        let shown = shown
            .redraw_plan
            .expect("hover deadline should draw tooltip");
        assert!(shown.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Save document"
        )));
        assert_eq!(runtime.hit_target_count(), 1);
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
            NativeWindowSmokeRunOptions::quick().native_view_click(Point { x: 60, y: 36 }),
        );

        record_native_view_input_smoke(
            &mut report,
            &mut builder.native_view_input_runtime(),
            &NativeWindowSmokeRunOptions::quick().native_view_click(Point { x: 60, y: 36 }),
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

    #[cfg(feature = "button")]
    #[test]
    fn native_view_runtime_decorates_default_button_hover_and_pressed_states() {
        let button_id = crate::WidgetId::new(73);
        let builder = native_window("Button states").size(240, 100).stateful_view(
            false,
            move |_| crate::button("Save").id(button_id).on_click(true),
            |clicked, message, _cx| *clicked = message,
        );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(button_id))
            .expect("button should expose pointer geometry");
        let point = Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        };
        let mut runtime = builder.native_view_input_runtime();

        let hovered = runtime.dispatch_pointer_move(point);
        let pressed = runtime.dispatch_pointer_down(point, false);

        assert!(hovered.pointer_visual_changed);
        assert!(pressed.pointer_visual_changed);
        assert!(hovered
            .redraw_plan
            .is_some_and(|plan| plan.commands.iter().any(|command| matches!(
                command,
                crate::NativeDrawCommand::RoundFill {
                    fill: crate::NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::PrimaryText,
                        alpha: 14,
                    },
                    ..
                }
            ))));
        assert!(pressed
            .redraw_plan
            .is_some_and(|plan| plan.commands.iter().any(|command| matches!(
                command,
                crate::NativeDrawCommand::RoundFill {
                    fill: crate::NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::PrimaryText,
                        alpha: 28,
                    },
                    ..
                }
            ))));
    }

    #[cfg(feature = "toggle-button")]
    #[test]
    fn native_view_runtime_toggles_button_from_pointer_and_space() {
        let widget = crate::WidgetId::new(74);
        let builder = native_window("Toggle Button").size(240, 100).stateful_view(
            false,
            move |checked| {
                crate::toggle_button("Pin", *checked)
                    .id(widget)
                    .on_toggle(|checked| checked)
            },
            |state, checked, _cx| *state = checked,
        );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .expect("toggle button should have a platform hit target");
        let point = Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        };
        let mut runtime = builder.native_view_input_runtime();

        let hovered = runtime.dispatch_pointer_move(point);
        let pointer = runtime.dispatch_pointer_click(point);
        let keyboard = runtime.dispatch_key(NativeViewKey::Space);

        assert!(hovered.pointer_visual_changed);
        assert_eq!(pointer.message_count, 1);
        assert_eq!(runtime.widget_checked_value(widget), Some(false));
        assert!(keyboard.handled);
        assert_eq!(keyboard.message_count, 1);
        assert!(keyboard.redraw_plan.is_some());
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
            SelectionChanged(crate::ZsTextSelection),
            Undo,
        }
        struct State {
            value: String,
            selection: crate::ZsTextSelection,
        }

        let textbox_id = crate::WidgetId::new(76);
        let builder = native_window("Platform Text")
            .size(360, 220)
            .stateful_view_with_app_commands(
                State {
                    value: String::new(),
                    selection: crate::ZsTextSelection::default(),
                },
                move |state| {
                    crate::column([
                        crate::textbox(&state.value)
                            .id(textbox_id)
                            .on_change(Msg::Changed)
                            .on_text_selection_change(Msg::SelectionChanged),
                        crate::text(format!("Caret: {}", state.selection.caret)),
                    ])
                },
                |state, message, cx| match message {
                    Msg::Changed(value) => state.value = value,
                    Msg::SelectionChanged(selection) => state.selection = selection,
                    Msg::Undo => cx.text_edit_command(crate::ZsTextEditCommand::Undo),
                },
                |command| (command == &Command::custom("edit.undo")).then_some(Msg::Undo),
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
        let undone = runtime.dispatch_app_command(Command::custom("edit.undo"));

        assert_eq!(focus.focused_widget, Some(textbox_id.0));
        assert!(typed.handled);
        assert_eq!(selected.text_selection, Some((1, 3)));
        assert_eq!(selected.text_caret, Some(3));
        assert!(selected.text_selection_changed);
        assert_eq!(selected.message_count, 1);
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
        assert_eq!(replaced.message_count, 2);
        assert!(replaced.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(
                    command, crate::NativeDrawCommand::Text(text) if text.text == "A🙂Z"
                )
            }) && plan.commands.iter().any(|command| {
                matches!(
                    command, crate::NativeDrawCommand::Text(text) if text.text == "Caret: 2"
                )
            })
        }));
        assert_eq!(undone.text_edit_command_count, 1);
        assert_eq!(undone.text_undo_count, 1);
        assert_eq!(undone.text_selection, Some((1, 3)));
        assert!(undone.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "A中文Z")
            })
        }));
    }

    #[cfg(feature = "textbox")]
    #[test]
    fn native_view_runtime_moves_wrapped_editor_caret_by_visual_row() {
        #[derive(Clone)]
        enum Msg {
            Selection(crate::ZsTextSelection),
        }

        let editor = crate::WidgetId::new(760);
        let builder = native_window("Wrapped keyboard navigation")
            .size(48, 140)
            .stateful_view(
                crate::ZsTextSelection::default(),
                move |_selection| {
                    crate::text_editor("abcdef\nx\nuvwxyz")
                        .id(editor)
                        .width(crate::Dp::new(48.0))
                        .height(crate::Dp::new(120.0))
                        .on_text_selection_change(Msg::Selection)
                },
                |selection, message, _cx| match message {
                    Msg::Selection(next) => *selection = next,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("wrapped editor should expose keyboard geometry");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + 24,
            y: target.bounds.y + 10,
        });

        let second_visual_row = runtime.dispatch_key(NativeViewKey::Down);
        let short_hard_line = runtime.dispatch_key(NativeViewKey::Down);
        let next_wrapped_line = runtime.dispatch_key(NativeViewKey::Down);
        let extended = runtime.dispatch_key_with_shift(NativeViewKey::Up, true);

        assert_eq!(second_visual_row.text_caret, Some(6));
        assert_eq!(short_hard_line.text_caret, Some(8));
        assert_eq!(next_wrapped_line.text_caret, Some(11));
        assert_eq!(extended.text_selection, Some((8, 11)));
        assert!(extended.text_selection_changed);
        assert_eq!(extended.message_count, 1);
    }

    #[cfg(feature = "textbox")]
    #[test]
    fn native_view_runtime_scrolls_editor_rows_and_reveals_keyboard_caret() {
        let editor = crate::WidgetId::new(761);
        let value = "row0\nrow1\nrow2\nrow3\nrow4\nrow5";
        let builder = native_window("Editor viewport")
            .size(160, 80)
            .ui_command_view(
                crate::text_editor::<UiCommand>(value)
                    .id(editor)
                    .width(crate::Dp::new(120.0))
                    .height(crate::Dp::new(52.0))
                    .on_text_selection_change(|_| UiCommand::app(crate::CommandId("selection"))),
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("editor should expose viewport geometry");
        let point = Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(point);

        let scrolled = runtime.dispatch_pointer_scroll(point, Dp::new(48.0));
        let scrolled_row = runtime
            .text_edit
            .map(|state| state.first_visible_visual_row);
        let revealed = runtime.dispatch_key(NativeViewKey::Right);

        assert!(scrolled.handled);
        assert!(
            scrolled.redraw_plan.is_some(),
            "scroll row {scrolled_row:?}, target {:?}",
            target.bounds
        );
        let scrolled_text = scrolled
            .redraw_plan
            .as_ref()
            .into_iter()
            .flat_map(|plan| &plan.commands)
            .filter_map(|command| match command {
                NativeDrawCommand::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            scrolled_text.contains(&"row3") && !scrolled_text.contains(&"row0"),
            "unexpected scrolled editor text: {scrolled_text:?}"
        );
        assert!(revealed.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "row0"),
            )
        }));
    }

    #[cfg(feature = "textbox")]
    #[test]
    fn native_view_runtime_reveals_no_wrap_columns_and_offsets_pointer_hits() {
        let editor = crate::WidgetId::new(762);
        let builder = native_window("Horizontal editor viewport")
            .size(48, 70)
            .ui_command_view(
                crate::text_editor::<UiCommand>("0123456789")
                    .id(editor)
                    .text_wrap(crate::TextWrap::NoWrap)
                    .width(crate::Dp::new(48.0))
                    .height(crate::Dp::new(52.0))
                    .on_text_selection_change(|_| UiCommand::app(crate::CommandId("selection"))),
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("no-wrap editor should expose viewport geometry");
        let left = Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(left);

        let revealed = runtime.dispatch_key(NativeViewKey::End);
        let clicked = runtime.dispatch_pointer_click(left);

        assert_eq!(revealed.text_caret, Some(10));
        assert!(revealed.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(|command| {
                matches!(command, NativeDrawCommand::Text(text)
                    if text.text == "0123456789" && text.bounds.x < target.bounds.x + 8)
            })
        }));
        assert_eq!(clicked.text_caret, Some(6));
    }

    #[cfg(feature = "textbox")]
    #[test]
    fn native_view_runtime_pages_editor_by_visible_rows_with_shift_selection() {
        let editor = crate::WidgetId::new(763);
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let editor_height = editor_height_for_visible_rows(3);
        let builder = native_window("Paged editor viewport")
            .size(160, editor_height)
            .ui_command_view(
                crate::text_editor::<UiCommand>(value)
                    .id(editor)
                    .text_wrap(crate::TextWrap::NoWrap)
                    .width(crate::Dp::new(160.0))
                    .height(crate::Dp::new(editor_height as f32))
                    .on_text_selection_change(|_| UiCommand::app(crate::CommandId("selection"))),
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("editor should expose page viewport geometry");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + 16,
            y: target.bounds.y + 10,
        });

        let page_down = runtime.dispatch_key(NativeViewKey::PageDown);
        let shift_page_down = runtime.dispatch_key_with_shift(NativeViewKey::PageDown, true);
        let page_up = runtime.dispatch_key(NativeViewKey::PageUp);

        assert_eq!(page_down.text_caret, Some(10));
        assert!(page_down.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "d3"),
            )
        }));
        assert_eq!(shift_page_down.text_selection, Some((10, 19)));
        assert_eq!(page_up.text_selection, Some((10, 10)));
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

    #[cfg(feature = "textbox")]
    #[test]
    fn native_view_runtime_scrolls_editor_viewport_during_edge_drag() {
        let editor = crate::WidgetId::new(764);
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let editor_height = editor_height_for_visible_rows(3);
        let builder = native_window("Editor edge drag")
            .size(160, editor_height)
            .ui_command_view(
                crate::text_editor::<UiCommand>(value)
                    .id(editor)
                    .text_wrap(crate::TextWrap::NoWrap)
                    .height(crate::Dp::new(editor_height as f32))
                    .on_text_selection_change(|_| UiCommand::app(crate::CommandId("selection"))),
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(editor))
            .expect("editor should expose edge-drag geometry");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_down(
            Point {
                x: target.bounds.x + 16,
                y: target.bounds.y + 10,
            },
            false,
        );
        let outside = Point {
            x: target.bounds.x + 16,
            y: target.bounds.y + target.bounds.height + 40,
        };

        let first = runtime.dispatch_pointer_move(outside);
        let second = runtime.dispatch_pointer_move(outside);

        assert_eq!(first.text_drag_scroll_count, 1);
        assert_eq!(first.text_selection, Some((1, 10)));
        assert_eq!(second.text_drag_scroll_count, 1);
        assert_eq!(second.text_selection, Some((1, 13)));
        assert!(second.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "c2"),
            )
        }));
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

    #[cfg(feature = "number-box")]
    #[test]
    fn native_view_runtime_edits_and_steps_number_box() {
        #[derive(Clone)]
        enum Msg {
            Changed(Option<f64>),
        }

        let number_id = crate::WidgetId::new(810);
        let range = crate::ZsNumberRange::new(-10.0, 10.0)
            .step(0.5)
            .large_step(5.0);
        let builder = native_window("Platform NumberBox")
            .size(360, 220)
            .stateful_view(
                Some(2.5_f64),
                move |value| {
                    crate::number_box(*value, range)
                        .id(number_id)
                        .height(Dp::new(36.0))
                        .fraction_digits(1)
                        .on_number_change(Msg::Changed)
                },
                |value, message, _cx| match message {
                    Msg::Changed(next) => *value = next,
                },
            );
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(number_id))
            .expect("number box should have text geometry");
        let render = crate::zs_number_box_render_plan(
            target.bounds,
            crate::ZsNumberBoxPlatformStyle::current(),
            Dpi::standard(),
        );
        let mut runtime = builder.native_view_input_runtime();

        let incremented = runtime.dispatch_pointer_click(Point {
            x: render.increment_button.x + render.increment_button.width / 2,
            y: render.increment_button.y + render.increment_button.height / 2,
        });
        let stepped = runtime.dispatch_key(NativeViewKey::Up);
        let page_stepped = runtime.dispatch_key(NativeViewKey::PageUp);
        let cleared = runtime.dispatch_text_input("\u{8}\u{8}\u{8}");
        let typed = runtime.dispatch_text_input("-1.5");
        let committed = runtime.dispatch_key(NativeViewKey::Enter);

        assert!(incremented.handled);
        assert!(stepped.handled);
        assert!(page_stepped.handled);
        assert!(cleared.handled);
        assert!(typed.handled);
        assert!(committed.handled);
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("-1.5"));
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

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn native_view_runtime_routes_auto_suggest_text_keyboard_pointer_and_clear() {
        #[derive(Clone)]
        enum Msg {
            Text(crate::ZsAutoSuggestTextChange),
            Chosen(crate::ZsAutoSuggestionId),
            Submitted(crate::ZsAutoSuggestSubmission),
            Expanded(bool),
        }
        struct State {
            query: String,
            expanded: bool,
            chosen: Option<crate::ZsAutoSuggestionId>,
            submission: Option<crate::ZsAutoSuggestSubmission>,
        }

        let widget = crate::WidgetId::new(184);
        let chosen = crate::ZsAutoSuggestionId::new(2);
        let builder = native_window("Platform Auto Suggest")
            .size(360, 240)
            .stateful_view(
                State {
                    query: "B".into(),
                    expanded: true,
                    chosen: None,
                    submission: None,
                },
                move |state| {
                    crate::column([
                        crate::auto_suggest_box(
                            state.query.clone(),
                            [
                                crate::ZsAutoSuggestion::new(1_u64, "Alpha"),
                                crate::ZsAutoSuggestion::new(chosen, "Beta"),
                                crate::ZsAutoSuggestion::new(3_u64, "Bravo"),
                            ],
                        )
                        .id(widget)
                        .expanded(state.expanded)
                        .highlighted_suggestion(state.expanded.then_some(state.chosen).flatten())
                        .on_auto_suggest_text_change(Msg::Text)
                        .on_suggestion_chosen(Msg::Chosen)
                        .on_query_submit(Msg::Submitted)
                        .on_expanded_change(Msg::Expanded),
                        crate::spacer(),
                    ])
                },
                |state, message, _cx| match message {
                    Msg::Text(change) => {
                        state.query = change.text;
                        if change.reason == crate::ZsAutoSuggestTextChangeReason::UserInput {
                            state.chosen = None;
                        }
                    }
                    Msg::Chosen(chosen) => state.chosen = Some(chosen),
                    Msg::Submitted(submission) => state.submission = Some(submission),
                    Msg::Expanded(expanded) => state.expanded = expanded,
                },
            );
        let suggestion = builder
            .native_view_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind
                        == crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion: chosen }
                })
            })
            .expect("expanded auto-suggest should expose strong-id row geometry");
        let mut runtime = builder.native_view_input_runtime();

        let pointer = runtime.dispatch_pointer_click(Point {
            x: suggestion.bounds.x + 8,
            y: suggestion.bounds.y + suggestion.bounds.height / 2,
        });
        assert!(pointer.handled);
        assert!(pointer.auto_suggest_submitted);
        assert!(pointer.auto_suggest_expanded_changed);
        assert_eq!(
            runtime.widget_auto_suggest_state(widget),
            Some(crate::ZsAutoSuggestState {
                query: "Beta".into(),
                suggestion_ids: vec![1_u64.into(), chosen, 3_u64.into()],
                highlighted: None,
                expanded: false,
            })
        );

        let typed = runtime.dispatch_text_input("x");
        assert!(typed.handled);
        assert!(typed.auto_suggest_expanded_changed);
        assert_eq!(runtime.widget_text_value(widget).as_deref(), Some("Betax"));
        let highlighted = runtime.dispatch_key(NativeViewKey::Down);
        assert!(highlighted.handled);
        assert!(highlighted.auto_suggest_highlight_changed);
        assert_eq!(
            runtime
                .widget_auto_suggest_state(widget)
                .and_then(|state| state.highlighted),
            Some(1_u64.into())
        );
        let submitted = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(submitted.handled);
        assert!(submitted.auto_suggest_submitted);
        assert!(runtime
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.query == "Alpha" && !state.expanded));

        let clear = runtime
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets
                    .iter()
                    .copied()
                    .find(|target| target.kind == crate::ViewHitTargetKind::AutoSuggestClear)
            })
            .expect("non-empty query should expose clear button");
        let cleared = runtime.dispatch_pointer_click(Point {
            x: clear.bounds.x + clear.bounds.width / 2,
            y: clear.bounds.y + clear.bounds.height / 2,
        });
        assert!(cleared.auto_suggest_cleared);
        assert_eq!(runtime.widget_text_value(widget).as_deref(), Some(""));
    }

    #[cfg(feature = "command-palette")]
    #[test]
    fn native_view_runtime_routes_command_palette_query_navigation_and_invocation() {
        #[derive(Clone)]
        enum Msg {
            Query(String),
            Highlight(crate::ZsCommandPaletteItemId),
            Invoke(crate::ZsCommandPaletteItemId),
            Open(bool),
        }
        struct State {
            query: String,
            highlighted: Option<crate::ZsCommandPaletteItemId>,
            invoked: Option<crate::ZsCommandPaletteItemId>,
            open: bool,
        }

        let widget = crate::WidgetId::new(204);
        let settings = crate::ZsCommandPaletteItemId::new(2);
        let builder = native_window("Platform Command Palette")
            .size(900, 620)
            .stateful_view(
                State {
                    query: String::new(),
                    highlighted: Some(1_u64.into()),
                    invoked: None,
                    open: true,
                },
                move |state| {
                    crate::command_palette(
                        widget,
                        state.open,
                        state.query.clone(),
                        [
                            crate::ZsCommandPaletteItem::new(1_u64, "Open file")
                                .icon(crate::ZsIcon::File),
                            crate::ZsCommandPaletteItem::new(settings, "Open settings")
                                .keywords(["preferences"])
                                .shortcut("Ctrl+,"),
                            crate::ZsCommandPaletteItem::new(3_u64, "Unavailable").enabled(false),
                        ],
                        crate::spacer(),
                    )
                    .highlighted_command(state.highlighted)
                    .on_command_palette_query_change(Msg::Query)
                    .on_command_palette_highlight_change(Msg::Highlight)
                    .on_command_palette_invoke(Msg::Invoke)
                    .on_command_palette_open_change(Msg::Open)
                },
                |state, message, _cx| match message {
                    Msg::Query(query) => state.query = query,
                    Msg::Highlight(item) => state.highlighted = Some(item),
                    Msg::Invoke(item) => {
                        state.invoked = Some(item);
                        state.open = false;
                    }
                    Msg::Open(open) => state.open = open,
                },
            );
        let mut runtime = builder.native_view_input_runtime();

        let moved = runtime.dispatch_key(NativeViewKey::Down);
        assert!(moved.handled);
        assert!(moved.command_palette_highlight_changed);
        assert_eq!(
            runtime
                .widget_command_palette_state(widget)
                .and_then(|state| state.highlighted),
            Some(settings)
        );

        let typed = runtime.dispatch_text_input("settings");
        assert!(typed.handled);
        assert!(typed.command_palette_query_changed);
        assert!(runtime
            .widget_command_palette_state(widget)
            .is_some_and(
                |state| state.query == "settings" && state.visible_items == vec![settings]
            ));

        let invoked = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(invoked.handled);
        assert!(invoked.command_palette_invoked);
        assert!(invoked.command_palette_open_changed);
        assert!(runtime
            .widget_command_palette_state(widget)
            .is_some_and(|state| !state.open));
    }

    #[cfg(feature = "tree")]
    #[test]
    fn native_view_runtime_routes_tree_pointer_and_hierarchical_keyboard_navigation() {
        #[derive(Clone)]
        enum Msg {
            Selected(crate::ZsTreeNodeId),
            Expanded(crate::ZsTreeExpansionChange),
            Invoked(crate::ZsTreeNodeId),
        }
        struct State {
            selected: Option<crate::ZsTreeNodeId>,
            expanded: std::collections::BTreeSet<crate::ZsTreeNodeId>,
            invoked: Option<crate::ZsTreeNodeId>,
        }

        let widget = crate::WidgetId::new(185);
        let root = crate::ZsTreeNodeId::new(1);
        let folder = crate::ZsTreeNodeId::new(2);
        let leaf = crate::ZsTreeNodeId::new(3);
        let builder = native_window("Platform Tree").size(360, 260).stateful_view(
            State {
                selected: Some(folder),
                expanded: std::collections::BTreeSet::from([root]),
                invoked: None,
            },
            move |state| {
                crate::tree_view([crate::ZsTreeNode::new(root, "Workspace")
                    .icon(crate::ZsIcon::Folder)
                    .children([
                        crate::ZsTreeNode::new(folder, "src")
                            .icon(crate::ZsIcon::Folder)
                            .children([crate::ZsTreeNode::new(leaf, "lib.rs")]),
                        crate::ZsTreeNode::new(4, "Cargo.toml"),
                    ])])
                .id(widget)
                .expanded_tree_nodes(state.expanded.iter().copied())
                .selected_tree_node(state.selected)
                .on_tree_select(Msg::Selected)
                .on_tree_expansion_change(Msg::Expanded)
                .on_tree_invoke(Msg::Invoked)
            },
            |state, message, _cx| match message {
                Msg::Selected(node) => state.selected = Some(node),
                Msg::Expanded(change) => {
                    if change.expanded {
                        state.expanded.insert(change.node);
                    } else {
                        state.expanded.remove(&change.node);
                    }
                }
                Msg::Invoked(node) => state.invoked = Some(node),
            },
        );
        let expander = builder
            .native_view_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind == crate::ViewHitTargetKind::TreeNodeExpander { node: folder }
                })
            })
            .expect("collapsed folder should expose disclosure geometry");
        let mut runtime = builder.native_view_input_runtime();

        let expanded = runtime.dispatch_pointer_click(Point {
            x: expander.bounds.x + expander.bounds.width / 2,
            y: expander.bounds.y + expander.bounds.height / 2,
        });
        assert!(expanded.tree_expansion_changed);
        assert!(runtime
            .widget_tree_view_state(widget)
            .is_some_and(|state| state.row(folder).is_some_and(|row| row.expanded)));

        let leaf_row = runtime
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets
                    .iter()
                    .copied()
                    .find(|target| target.kind == crate::ViewHitTargetKind::TreeNode { node: leaf })
            })
            .expect("expanded folder should expose its leaf row");
        let invoked = runtime.dispatch_pointer_click(Point {
            x: leaf_row.bounds.x + leaf_row.bounds.width / 2,
            y: leaf_row.bounds.y + leaf_row.bounds.height / 2,
        });
        assert!(invoked.tree_selection_changed);
        assert!(invoked.tree_invoked);
        assert_eq!(
            runtime
                .widget_tree_view_state(widget)
                .and_then(|state| state.selected),
            Some(leaf)
        );

        let parent = runtime.dispatch_key(NativeViewKey::Left);
        assert!(parent.tree_selection_changed);
        assert_eq!(
            runtime
                .widget_tree_view_state(widget)
                .and_then(|state| state.selected),
            Some(folder)
        );
        let collapsed = runtime.dispatch_key(NativeViewKey::Left);
        assert!(collapsed.tree_expansion_changed);
        let keyboard = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(keyboard.tree_invoked);
    }

    #[cfg(feature = "grid-view")]
    #[test]
    fn native_view_runtime_routes_grid_view_pointer_and_two_axis_keyboard_navigation() {
        #[derive(Clone)]
        enum Msg {
            Selected(crate::ZsGridViewItemId),
            Invoked(crate::ZsGridViewItemId),
        }
        struct State {
            selected: Option<crate::ZsGridViewItemId>,
            invoked: Option<crate::ZsGridViewItemId>,
        }

        let widget = crate::WidgetId::new(198);
        let first = crate::ZsGridViewItemId::new(1);
        let fifth = crate::ZsGridViewItemId::new(5);
        let builder = native_window("Platform GridView")
            .size(420, 300)
            .stateful_view(
                State {
                    selected: Some(first),
                    invoked: None,
                },
                move |state| {
                    crate::grid_view([
                        crate::ZsGridViewItem::new(1, "One"),
                        crate::ZsGridViewItem::new(2, "Two"),
                        crate::ZsGridViewItem::new(3, "Three"),
                        crate::ZsGridViewItem::new(4, "Four"),
                        crate::ZsGridViewItem::new(5, "Five"),
                        crate::ZsGridViewItem::new(6, "Six"),
                    ])
                    .id(widget)
                    .selected_grid_view_item(state.selected)
                    .on_grid_view_select(Msg::Selected)
                    .on_grid_view_invoke(Msg::Invoked)
                },
                |state, message, _cx| match message {
                    Msg::Selected(item) => state.selected = Some(item),
                    Msg::Invoked(item) => state.invoked = Some(item),
                },
            );
        let fifth_tile = builder
            .native_view_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.iter().copied().find(|target| {
                    target.kind == crate::ViewHitTargetKind::GridViewItem { item: fifth }
                })
            })
            .expect("grid view should expose item hit geometry");
        let mut runtime = builder.native_view_input_runtime();

        let pointer = runtime.dispatch_pointer_click(Point {
            x: fifth_tile.bounds.x + fifth_tile.bounds.width / 2,
            y: fifth_tile.bounds.y + fifth_tile.bounds.height / 2,
        });
        assert!(pointer.grid_view_selection_changed);
        assert!(pointer.grid_view_invoked);
        assert_eq!(
            runtime
                .widget_grid_view_state(widget)
                .and_then(|state| state.selected),
            Some(fifth)
        );

        assert!(
            runtime
                .dispatch_key(NativeViewKey::Home)
                .grid_view_selection_changed
        );
        assert!(
            runtime
                .dispatch_key(NativeViewKey::Right)
                .grid_view_selection_changed
        );
        let expected_after_down = runtime.widget_grid_view_state(widget).and_then(|state| {
            let selected_index = state
                .selected
                .and_then(|selected| state.items.iter().position(|item| *item == selected))?;
            let target = selected_index
                .saturating_add(state.column_count.max(1))
                .min(state.items.len().saturating_sub(1));
            state.items.get(target).copied()
        });
        assert!(
            runtime
                .dispatch_key(NativeViewKey::Down)
                .grid_view_selection_changed
        );
        assert_eq!(
            runtime
                .widget_grid_view_state(widget)
                .and_then(|state| state.selected),
            expected_after_down
        );
        assert!(runtime.dispatch_key(NativeViewKey::Enter).grid_view_invoked);
    }

    #[cfg(feature = "table")]
    #[test]
    fn native_view_runtime_routes_table_sort_row_pointer_and_keyboard_navigation() {
        #[derive(Clone)]
        enum Msg {
            Selected(crate::ZsTableRowId),
            Sorted(crate::ZsTableSort),
            Invoked(crate::ZsTableRowId),
        }
        struct State {
            selected: Option<crate::ZsTableRowId>,
            sort: Option<crate::ZsTableSort>,
            invoked: Option<crate::ZsTableRowId>,
        }

        let widget = crate::WidgetId::new(186);
        let name = crate::ZsTableColumnId::new(1);
        let first = crate::ZsTableRowId::new(10);
        let second = crate::ZsTableRowId::new(11);
        let builder = native_window("Platform Table")
            .size(360, 240)
            .stateful_view(
                State {
                    selected: Some(first),
                    sort: None,
                    invoked: None,
                },
                move |state| {
                    crate::data_grid(
                        [
                            crate::ZsTableColumn::new(name, "Name").sortable(true),
                            crate::ZsTableColumn::new(2, "Size").fixed_width(crate::Dp::new(80.0)),
                        ],
                        [
                            crate::ZsTableRow::new(first, ["Cargo.toml", "4 KB"]),
                            crate::ZsTableRow::new(second, ["src", "—"]),
                        ],
                    )
                    .id(widget)
                    .selected_table_row(state.selected)
                    .table_sort(state.sort)
                    .on_table_select(Msg::Selected)
                    .on_table_sort(Msg::Sorted)
                    .on_table_invoke(Msg::Invoked)
                },
                |state, message, _cx| match message {
                    Msg::Selected(row) => state.selected = Some(row),
                    Msg::Sorted(sort) => state.sort = Some(sort),
                    Msg::Invoked(row) => state.invoked = Some(row),
                },
            );
        let interaction = builder
            .native_view_interaction_plan()
            .expect("table interaction plan");
        let header = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableHeader { column: name })
            .expect("sortable header");
        let second_row = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableRow { row: second })
            .expect("second row");
        let mut runtime = builder.native_view_input_runtime();

        let sorted = runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + header.bounds.width / 2,
            y: header.bounds.y + header.bounds.height / 2,
        });
        assert!(sorted.table_sort_changed);
        assert_eq!(
            runtime
                .widget_table_state(widget)
                .and_then(|state| state.sort),
            Some(crate::ZsTableSort::new(
                name,
                crate::ZsTableSortDirection::Ascending
            ))
        );

        let invoked = runtime.dispatch_pointer_click(Point {
            x: second_row.bounds.x + second_row.bounds.width / 2,
            y: second_row.bounds.y + second_row.bounds.height / 2,
        });
        assert!(invoked.table_selection_changed);
        assert!(invoked.table_invoked);
        assert_eq!(
            runtime
                .widget_table_state(widget)
                .and_then(|state| state.selected),
            Some(second)
        );

        let moved = runtime.dispatch_key(NativeViewKey::Up);
        assert!(moved.table_selection_changed);
        let keyboard = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(keyboard.table_invoked);
    }

    #[cfg(feature = "dialog")]
    #[test]
    fn native_view_runtime_traps_modal_focus_and_routes_typed_dialog_response() {
        #[derive(Clone)]
        enum Msg {
            Responded(crate::ZsContentDialogResult),
        }
        struct State {
            open: bool,
            result: Option<crate::ZsContentDialogResult>,
        }

        let widget = crate::WidgetId::new(187);
        let background = crate::WidgetId::new(188);
        let builder = native_window("Platform Dialog")
            .size(640, 400)
            .stateful_view(
                State {
                    open: true,
                    result: None,
                },
                move |state| {
                    crate::content_dialog(
                        widget,
                        state.open,
                        crate::ZsContentDialogSpec::new(
                            "Choose whether to continue or cancel.",
                            "Cancel",
                        )
                        .title("Continue operation?")
                        .primary_button("Continue")
                        .secondary_button("Review")
                        .default_button(crate::ZsContentDialogButton::Primary),
                        crate::spacer().id(background),
                    )
                    .on_dialog_result(Msg::Responded)
                },
                |state, message, _cx| match message {
                    Msg::Responded(result) => {
                        state.open = false;
                        state.result = Some(result);
                    }
                },
            );
        let interaction = builder
            .native_view_interaction_plan()
            .expect("open dialog interaction plan");
        let scrim = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialogScrim)
            .expect("modal scrim");
        let mut runtime = builder.native_view_input_runtime();

        let caught = runtime.dispatch_pointer_click(Point {
            x: scrim.bounds.x + 2,
            y: scrim.bounds.y + 2,
        });
        assert!(caught.handled);
        assert!(!caught.content_dialog_responded);
        assert!(runtime
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| state.open));
        let suppressed = runtime.dispatch_text_input("x");
        assert!(suppressed.handled);
        assert_eq!(suppressed.message_count, 0);

        let focused = runtime.dispatch_key(NativeViewKey::Tab);
        assert!(focused.handled);
        assert!(focused.content_dialog_focus_changed);
        assert_eq!(focused.focused_widget, Some(widget.0));
        assert_eq!(
            runtime
                .widget_content_dialog_state(widget)
                .map(|(state, _)| state.focused_button),
            Some(crate::ZsContentDialogButton::Secondary)
        );

        let responded = runtime.dispatch_key(NativeViewKey::Enter);
        assert!(responded.handled);
        assert!(responded.content_dialog_responded);
        assert_eq!(responded.message_count, 1);
        assert!(runtime
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| !state.open));
        assert_eq!(
            runtime
                .current_interaction_plan()
                .and_then(|plan| plan.first_focus_target())
                .map(|target| target.widget),
            Some(background)
        );
    }

    #[cfg(feature = "toast")]
    #[test]
    fn native_view_runtime_routes_toast_action_and_timeout_as_typed_results() {
        #[derive(Clone)]
        enum Msg {
            Responded(crate::ZsToastResult),
        }
        struct State {
            toast: Option<crate::ZsToastSpec>,
            result: Option<crate::ZsToastResult>,
        }

        let widget = crate::WidgetId::new(197);
        let page = crate::WidgetId::new(198);
        let build = || {
            native_window("Platform Toast")
                .size(640, 400)
                .stateful_view(
                    State {
                        toast: Some(crate::ZsToastSpec::new(41, "File deleted").action("Undo")),
                        result: None,
                    },
                    move |state| {
                        crate::toast_presenter(
                            widget,
                            state.toast.clone(),
                            crate::spacer().id(page),
                        )
                        .on_toast_result(Msg::Responded)
                    },
                    |state, message, _cx| match message {
                        Msg::Responded(result) => {
                            state.toast = None;
                            state.result = Some(result);
                        }
                    },
                )
        };

        let builder = build();
        let action = builder
            .native_view_interaction_plan()
            .expect("toast interaction plan")
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ToastAction)
            .expect("toast action target");
        let mut runtime = builder.native_view_input_runtime();
        let action_report = runtime.dispatch_pointer_click(Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert!(action_report.handled);
        assert!(action_report.toast_responded);
        assert_eq!(action_report.message_count, 1);
        assert!(runtime.widget_toast_state(widget).is_none());

        let mut timeout_runtime = build().native_view_input_runtime();
        let timeout = timeout_runtime.refresh_transient_view_at(
            std::time::Instant::now() + std::time::Duration::from_secs(6),
        );
        assert!(timeout.handled);
        assert!(timeout.toast_responded);
        assert_eq!(timeout.message_count, 1);
        assert!(timeout_runtime.widget_toast_state(widget).is_none());
    }

    #[cfg(feature = "info-bar")]
    #[test]
    fn native_view_runtime_routes_info_bar_pointer_and_keyboard_events() {
        #[derive(Clone)]
        enum Msg {
            Event(crate::ZsInfoBarEvent),
        }
        struct State {
            last: Option<crate::ZsInfoBarEvent>,
        }

        let widget = crate::WidgetId::new(199);
        let build = || {
            native_window("Platform InfoBar")
                .size(640, 240)
                .stateful_view(
                    State { last: None },
                    move |_state| {
                        crate::column([
                            crate::info_bar(
                                widget,
                                crate::ZsInfoBarSpec::new("Renew to keep all functionality.")
                                    .title("Subscription expires soon")
                                    .severity(crate::ZsInfoBarSeverity::Warning)
                                    .action("Renew"),
                            )
                            .on_info_bar_event(Msg::Event),
                            crate::spacer(),
                        ])
                    },
                    |state, message, _cx| match message {
                        Msg::Event(event) => state.last = Some(event),
                    },
                )
        };

        let builder = build();
        let action = builder
            .native_view_interaction_plan()
            .expect("info-bar interaction plan")
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::InfoBarAction)
            .expect("info-bar action target");
        let mut pointer_runtime = builder.native_view_input_runtime();
        let action_report = pointer_runtime.dispatch_pointer_click(Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert!(action_report.handled);
        assert_eq!(
            action_report.info_bar_event,
            Some(crate::ZsInfoBarEvent::Action)
        );
        assert_eq!(action_report.message_count, 1);

        let mut keyboard_runtime = build().native_view_input_runtime();
        let focus = keyboard_runtime.dispatch_key(NativeViewKey::Tab);
        assert!(focus.handled);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let next = keyboard_runtime.dispatch_key(NativeViewKey::Right);
        assert!(next.handled);
        assert!(next.info_bar_focus_changed);
        let close = keyboard_runtime.dispatch_key(NativeViewKey::Enter);
        assert!(close.handled);
        assert_eq!(close.info_bar_event, Some(crate::ZsInfoBarEvent::Close));
        assert_eq!(close.message_count, 1);
    }

    #[cfg(feature = "teaching-tip")]
    #[test]
    fn native_view_runtime_routes_teaching_tip_pointer_and_keyboard_responses() {
        #[derive(Clone)]
        enum Msg {
            Result(crate::ZsTeachingTipResult),
        }
        struct State {
            open: bool,
            last: Option<crate::ZsTeachingTipResult>,
        }

        let tip = crate::WidgetId::new(201);
        let target = crate::WidgetId::new(202);
        let build = || {
            native_window("Platform TeachingTip")
                .size(640, 420)
                .stateful_view(
                    State {
                        open: true,
                        last: None,
                    },
                    move |state| {
                        crate::teaching_tip(
                            tip,
                            state.open,
                            target,
                            crate::ZsTeachingTipSpec::new(
                                "Save automatically",
                                "Your changes are saved as you work.",
                            )
                            .action("Review settings"),
                            crate::spacer().id(target),
                        )
                        .on_teaching_tip_result(Msg::Result)
                    },
                    |state, message, _cx| match message {
                        Msg::Result(result) => {
                            state.open = false;
                            state.last = Some(result);
                        }
                    },
                )
        };

        let builder = build();
        let action = builder
            .native_view_interaction_plan()
            .expect("teaching-tip interaction plan")
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTipAction)
            .expect("teaching-tip action target");
        let mut pointer_runtime = builder.native_view_input_runtime();
        let action_report = pointer_runtime.dispatch_pointer_click(Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert!(action_report.handled);
        assert_eq!(
            action_report.teaching_tip_response,
            Some(crate::ZsTeachingTipResponse::Action)
        );
        assert_eq!(action_report.message_count, 1);

        let mut keyboard_runtime = build().native_view_input_runtime();
        let focus = keyboard_runtime.dispatch_key(NativeViewKey::Tab);
        assert!(focus.handled);
        assert_eq!(focus.focused_widget, Some(target.0));
        let focus = keyboard_runtime.dispatch_key(NativeViewKey::Tab);
        assert!(focus.handled);
        assert_eq!(focus.focused_widget, Some(tip.0));
        let next = keyboard_runtime.dispatch_key(NativeViewKey::Right);
        assert!(next.handled);
        assert!(next.teaching_tip_focus_changed);
        let close = keyboard_runtime.dispatch_key(NativeViewKey::Enter);
        assert!(close.handled);
        assert_eq!(
            close.teaching_tip_response,
            Some(crate::ZsTeachingTipResponse::Dismissed(
                crate::ZsTeachingTipDismissReason::CloseButton,
            ))
        );
        assert_eq!(close.message_count, 1);
    }

    #[cfg(feature = "breadcrumb")]
    #[test]
    fn native_view_runtime_routes_breadcrumb_overflow_focus_and_selection() {
        #[derive(Clone)]
        enum Msg {
            Expanded(bool),
            Selected(crate::ZsBreadcrumbId),
        }
        struct State {
            expanded: bool,
            selected: Option<crate::ZsBreadcrumbId>,
        }

        let widget = crate::WidgetId::new(203);
        let selected = crate::ZsBreadcrumbId::new(2);
        let build = || {
            native_window("Platform Breadcrumb")
                .size(320, 220)
                .stateful_view(
                    State {
                        expanded: false,
                        selected: None,
                    },
                    move |state| {
                        crate::breadcrumb_bar([
                            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(1), "Home"),
                            crate::ZsBreadcrumbItem::new(selected, "Projects"),
                            crate::ZsBreadcrumbItem::new(
                                crate::ZsBreadcrumbId::new(3),
                                "ZSUI Framework",
                            ),
                            crate::ZsBreadcrumbItem::new(
                                crate::ZsBreadcrumbId::new(4),
                                "Documentation",
                            ),
                            crate::ZsBreadcrumbItem::new(
                                crate::ZsBreadcrumbId::new(5),
                                "BreadcrumbBar",
                            ),
                        ])
                        .id(widget)
                        .width(crate::Dp::new(240.0))
                        .expanded(state.expanded)
                        .on_expanded_change(Msg::Expanded)
                        .on_breadcrumb_select(Msg::Selected)
                    },
                    |state, message, _cx| match message {
                        Msg::Expanded(expanded) => state.expanded = expanded,
                        Msg::Selected(item) => state.selected = Some(item),
                    },
                )
        };

        let builder = build();
        let overflow = builder
            .native_view_interaction_plan()
            .expect("breadcrumb interaction plan")
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::BreadcrumbOverflow)
            .expect("narrow breadcrumb should expose overflow");
        let mut pointer_runtime = builder.native_view_input_runtime();
        let opened = pointer_runtime.dispatch_pointer_click(Point {
            x: overflow.bounds.x + overflow.bounds.width / 2,
            y: overflow.bounds.y + overflow.bounds.height / 2,
        });
        assert!(opened.handled);
        assert!(opened.breadcrumb_expanded_changed);
        let row = pointer_runtime
            .current_interaction_plan()
            .expect("open breadcrumb interaction plan")
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                matches!(
                    target.kind,
                    crate::ViewHitTargetKind::BreadcrumbOverflowItem { .. }
                )
            })
            .expect("open overflow should expose hidden rows");
        let selected_report = pointer_runtime.dispatch_pointer_click(Point {
            x: row.bounds.x + row.bounds.width / 2,
            y: row.bounds.y + row.bounds.height / 2,
        });
        assert!(selected_report.handled);
        assert!(selected_report.breadcrumb_selection.is_some());
        assert!(selected_report.breadcrumb_expanded_changed);

        let mut keyboard_runtime = build().native_view_input_runtime();
        assert!(keyboard_runtime.dispatch_key(NativeViewKey::Tab).handled);
        let home = keyboard_runtime.dispatch_key(NativeViewKey::Home);
        assert!(home.handled);
        assert!(home.breadcrumb_focus_changed);
        for _ in 0..5 {
            if keyboard_runtime
                .widget_breadcrumb_state(widget)
                .is_some_and(|state| {
                    state.focused == Some(crate::ZsBreadcrumbFocusTarget::Overflow)
                })
            {
                break;
            }
            assert!(keyboard_runtime.dispatch_key(NativeViewKey::Right).handled);
        }
        assert!(keyboard_runtime
            .widget_breadcrumb_state(widget)
            .is_some_and(|state| {
                state.focused == Some(crate::ZsBreadcrumbFocusTarget::Overflow)
            }));
        let open = keyboard_runtime.dispatch_key(NativeViewKey::Enter);
        assert!(open.handled);
        assert!(open.breadcrumb_expanded_changed);
        let down = keyboard_runtime.dispatch_key(NativeViewKey::Down);
        assert!(down.handled);
        assert!(down.breadcrumb_focus_changed);
        let select = keyboard_runtime.dispatch_key(NativeViewKey::Enter);
        assert!(select.handled);
        assert!(select.breadcrumb_selection.is_some());
        assert!(select.breadcrumb_expanded_changed);
        assert!(keyboard_runtime
            .widget_breadcrumb_state(widget)
            .is_some_and(|state| !state.overflow_open));
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
                        .today(initial)
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

    #[cfg(feature = "time-picker")]
    #[test]
    fn native_view_runtime_opens_selects_and_navigates_time_picker() {
        #[derive(Clone)]
        enum Msg {
            Changed(crate::ZsTime),
            Expanded(bool),
        }
        struct State {
            value: crate::ZsTime,
            expanded: bool,
        }

        let widget = crate::WidgetId::new(86);
        let initial = crate::ZsTime::new(9, 30).unwrap();
        let builder = native_window("Platform TimePicker")
            .size(420, 320)
            .stateful_view(
                State {
                    value: initial,
                    expanded: false,
                },
                move |state| {
                    crate::time_picker(state.value)
                        .id(widget)
                        .height(Dp::new(32.0))
                        .minute_increment(crate::ZsMinuteIncrement::FIFTEEN)
                        .clock_format(crate::ZsClockFormat::TwentyFourHour)
                        .expanded(state.expanded)
                        .on_time_change(Msg::Changed)
                        .on_expanded_change(Msg::Expanded)
                },
                |state, message, _cx| match message {
                    Msg::Changed(next) => state.value = next,
                    Msg::Expanded(expanded) => state.expanded = expanded,
                },
            );
        let header = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .expect("time picker header should have hit geometry");
        let mut runtime = builder.native_view_input_runtime();

        let opened = runtime.dispatch_pointer_click(Point {
            x: header.bounds.x + 12,
            y: header.bounds.y + header.bounds.height / 2,
        });
        assert!(opened.handled);
        assert!(
            runtime
                .widget_time_picker_state(widget)
                .expect("time picker state")
                .expanded
        );

        let selected = crate::ZsTime::new(9, 45).unwrap();
        let choice = runtime
            .current_interaction_plan()
            .and_then(|plan| {
                plan.hit_targets.into_iter().find(|target| {
                    target.kind == crate::ViewHitTargetKind::TimePickerChoice { value: selected }
                })
            })
            .expect("expanded time picker should expose minute choice geometry");
        let changed = runtime.dispatch_pointer_click(Point {
            x: choice.bounds.x + choice.bounds.width / 2,
            y: choice.bounds.y + choice.bounds.height / 2,
        });
        assert!(changed.handled);
        assert_eq!(changed.message_count, 1);
        assert_eq!(
            runtime
                .widget_time_picker_state(widget)
                .map(|state| (state.value, state.expanded)),
            Some((selected, true))
        );

        let closed = runtime.dispatch_key(NativeViewKey::Escape);
        assert!(closed.handled);
        assert_eq!(
            runtime
                .widget_time_picker_state(widget)
                .map(|state| state.expanded),
            Some(false)
        );
        let minute = runtime.dispatch_key(NativeViewKey::Down);
        assert!(minute.handled);
        assert_eq!(
            runtime
                .widget_time_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsTime::new(10, 0).unwrap())
        );
        let hour = runtime.dispatch_key(NativeViewKey::Right);
        assert!(hour.handled);
        assert_eq!(
            runtime
                .widget_time_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsTime::new(11, 0).unwrap())
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

    #[cfg(all(feature = "textbox", feature = "text-input-core"))]
    #[test]
    fn native_view_runtime_keeps_committed_and_preedit_selection_on_grapheme_boundaries() {
        let widget = crate::WidgetId::new(769);
        let builder = native_window("Platform grapheme input")
            .size(360, 220)
            .ui_command_view(crate::textbox::<UiCommand>("").id(widget));
        let target = builder
            .native_view_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .expect("textbox should expose grapheme input geometry");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(Point {
            x: target.bounds.x + 12,
            y: target.bounds.y + 12,
        });

        let typed = runtime.dispatch_text_input("A\u{65}\u{301}👩🏽‍💻Z");
        assert!(typed.handled);
        assert_eq!(typed.text_selection, Some((8, 8)));
        let left = runtime.dispatch_key(NativeViewKey::Left);
        assert_eq!(left.text_selection, Some((7, 7)));
        let left = runtime.dispatch_key(NativeViewKey::Left);
        assert_eq!(left.text_selection, Some((3, 3)));
        let deleted = runtime.dispatch_text_input("\u{8}");
        assert_eq!(deleted.text_selection, Some((1, 1)));
        assert_eq!(runtime.focused_text_input_value().as_deref(), Some("A👩🏽‍💻Z"));

        let preedit = runtime.dispatch_ime_preedit("\u{65}\u{301}", Some((1, 1)));
        assert_eq!(preedit.ime_selection, Some((0, 0)));
        let committed = runtime.dispatch_ime_commit("\u{65}\u{301}");
        assert_eq!(committed.text_selection, Some((3, 3)));
        assert_eq!(
            runtime.focused_text_input_value().as_deref(),
            Some("Ae\u{301}👩🏽‍💻Z")
        );
    }

    #[cfg(feature = "password-box")]
    #[test]
    fn native_view_runtime_keeps_password_input_ime_and_peek_redacted() {
        #[derive(Clone)]
        enum Msg {
            Changed(crate::ZsPassword),
        }

        let widget = crate::WidgetId::new(780);
        let initial_secret = "vault🙂";
        let builder = native_window("Secure input").size(360, 120).stateful_view(
            crate::ZsPassword::from(initial_secret),
            move |value| {
                crate::password_box(value)
                    .id(widget)
                    .height(crate::Dp::new(36.0))
                    .reveal_mode(crate::ZsPasswordRevealMode::Peek)
                    .on_password_change(Msg::Changed)
            },
            |value, message, _cx| match message {
                Msg::Changed(next) => *value = next,
            },
        );
        let interaction = builder
            .native_view_interaction_plan()
            .expect("password box should expose interaction geometry");
        let input = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBox)
            .expect("password box should expose an input target");
        let reveal = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBoxReveal)
            .expect("peek mode should expose a reveal target");
        let mut runtime = builder.native_view_input_runtime();
        runtime.dispatch_pointer_click(Point {
            x: reveal.bounds.x - 4,
            y: input.bounds.y + input.bounds.height / 2,
        });

        let typed = runtime.dispatch_text_input("中");
        let typed_secret = "vault🙂中";
        assert!(typed.handled);
        assert_eq!(typed.message_count, 1);
        assert_eq!(
            runtime
                .widget_password_value(widget)
                .map(|value| value.as_str().to_owned())
                .as_deref(),
            Some(typed_secret)
        );
        assert_eq!(
            runtime.focused_text_input_value().as_deref(),
            Some("•••••••")
        );
        assert!(!format!("{typed:?}").contains(typed_secret));
        assert!(!serde_json::to_string(
            typed
                .redraw_plan
                .as_ref()
                .expect("typing should redraw the password box")
        )
        .expect("password redraw should serialize redacted")
        .contains(typed_secret));

        let preedit = runtime.dispatch_ime_preedit("文", Some((1, 1)));
        assert_eq!(preedit.ime_preedit_text.as_deref(), Some("•"));
        assert!(!format!("{preedit:?}").contains('文'));
        assert!(preedit.redraw_plan.as_ref().is_some_and(|plan| {
            plan.commands.iter().any(
                |command| matches!(command, NativeDrawCommand::Text(text) if text.text == "••••••••"),
            )
        }));
        let committed = runtime.dispatch_ime_commit("文");
        let committed_secret = "vault🙂中文";
        assert!(committed.handled);
        assert_eq!(committed.message_count, 1);
        assert_eq!(
            runtime.focused_text_input_value().as_deref(),
            Some("••••••••")
        );

        let reveal_point = Point {
            x: reveal.bounds.x + reveal.bounds.width / 2,
            y: reveal.bounds.y + reveal.bounds.height / 2,
        };
        let pressed = runtime.dispatch_pointer_down(reveal_point, false);
        let pressed_plan = pressed
            .redraw_plan
            .as_ref()
            .expect("pressing reveal should redraw clear text at the renderer boundary");
        assert!(pressed_plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::SecureText(command) if command.character_count() == 8
        )));
        assert!(!format!("{pressed_plan:?}").contains(committed_secret));
        assert!(!serde_json::to_string(pressed_plan)
            .expect("peek draw plan should serialize redacted")
            .contains(committed_secret));

        let released = runtime.dispatch_pointer_up(reveal_point);
        let released_plan = released
            .redraw_plan
            .as_ref()
            .expect("releasing reveal should restore the mask");
        assert!(released_plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "••••••••"
        )));
        assert!(!released_plan
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::SecureText(_))));
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
        assert_eq!(report.native_view_window_close_request_count, 0);
        assert_eq!(report.native_view_window_close_veto_count, 0);
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
        assert_eq!(report.native_view_focused_widget, None);
        assert_eq!(report.native_view_capture, None);
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
    fn native_view_capture_evidence_serializes_native_geometry_and_scale() {
        let evidence = NativeViewCaptureEvidence {
            platform: "macos",
            backend: "appkit_nsview_bitmap_cache",
            display_server: None,
            logical_width: 960,
            logical_height: 640,
            pixel_width: 1920,
            pixel_height: 1280,
            scale_factor: 2.0,
            typography_scale: 1.0,
            typography: crate::NativeTypographyProfile::fallback(
                crate::ZsTypographyPlatformStyle::Macos,
                1.0,
            ),
        };

        let json = serde_json::to_value(&evidence).expect("capture evidence should serialize");
        assert_eq!(json["platform"], "macos");
        assert_eq!(json["backend"], "appkit_nsview_bitmap_cache");
        assert_eq!(json["logical_width"], 960);
        assert_eq!(json["pixel_width"], 1920);
        assert_eq!(json["scale_factor"], 2.0);
        assert_eq!(json["typography_scale"], 1.0);
        assert_eq!(json["typography"]["platform"], "Macos");
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
    fn native_window_smoke_options_preserve_close_request_sequence() {
        let options = NativeWindowSmokeRunOptions::quick().native_window_close_request();

        assert_eq!(
            options.native_view_inputs,
            vec![NativeViewSmokeInput::WindowCloseRequest]
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

    #[cfg(feature = "color-picker")]
    #[test]
    fn native_view_runtime_routes_color_picker_drag_and_keyboard_channels() {
        #[derive(Clone, Copy)]
        enum Msg {
            Color(crate::Color),
            Expanded(bool),
            Channel(crate::ZsColorChannel),
        }

        struct State {
            picker: crate::ZsColorPickerState,
        }

        let widget = crate::WidgetId::new(219);
        let build = || {
            native_window("Platform ColorPicker")
                .size(480, 680)
                .stateful_view(
                    State {
                        picker: crate::ZsColorPickerState::new(crate::Color::rgba(
                            32, 96, 160, 224,
                        ))
                        .with_expanded(true),
                    },
                    move |state| {
                        crate::column([
                            crate::color_picker(state.picker)
                                .id(widget)
                                .height(Dp::new(32.0))
                                .on_color_change(Msg::Color)
                                .on_expanded_change(Msg::Expanded)
                                .on_color_channel_change(Msg::Channel),
                            crate::spacer(),
                        ])
                        .padding(Dp::new(24.0))
                        .gap(Dp::new(12.0))
                    },
                    |state, message, _cx| match message {
                        Msg::Color(color) => state.picker.color = color,
                        Msg::Expanded(expanded) => state.picker.expanded = expanded,
                        Msg::Channel(channel) => state.picker.active_channel = channel,
                    },
                )
        };

        let builder = build();
        let interaction = builder
            .native_view_interaction_plan()
            .expect("color picker interaction plan");
        let root = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.widget == widget && target.kind == crate::ViewHitTargetKind::ColorPicker
            })
            .expect("color picker root target");
        let red = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind
                    == (crate::ViewHitTargetKind::ColorPickerChannel {
                        channel: crate::ZsColorChannel::Red,
                    })
            })
            .expect("red channel target");
        let plan = crate::zs_color_picker_render_plan_in_viewport(
            root.bounds,
            crate::ZsColorPickerState::new(crate::Color::rgba(32, 96, 160, 224))
                .with_expanded(true),
            crate::ZsColorPickerPlatformStyle::current(),
            Dpi::standard(),
            Rect {
                x: 0,
                y: 0,
                width: 480,
                height: 680,
            },
        );
        let red_track = plan
            .channels
            .iter()
            .find(|row| row.channel == crate::ZsColorChannel::Red)
            .expect("red channel geometry")
            .track;
        assert!(red.bounds.contains(Point {
            x: red_track.x,
            y: red_track.y,
        }));

        let mut runtime = builder.native_view_input_runtime();
        let pressed = runtime.dispatch_pointer_down(
            Point {
                x: red_track.x + red_track.width / 4,
                y: red_track.y + red_track.height / 2,
            },
            false,
        );
        assert!(pressed.handled);
        assert!(pressed.color_picker_drag_active);
        assert!(pressed.color_picker_value_changed);
        let moved = runtime.dispatch_pointer_move(Point {
            x: red_track.x + red_track.width * 9 / 10,
            y: red_track.y + red_track.height / 2,
        });
        assert!(moved.handled);
        assert!(moved.color_picker_drag_active);
        let released = runtime.dispatch_pointer_up(Point {
            x: red_track.x + red_track.width * 9 / 10,
            y: red_track.y + red_track.height / 2,
        });
        assert!(released.handled);
        assert!(!released.color_picker_drag_active);
        assert!(runtime
            .widget_color_picker_state(widget)
            .is_some_and(|state| state.color.r > 220));

        let mut keyboard = build().native_view_input_runtime();
        let focused = keyboard.dispatch_key(NativeViewKey::Tab);
        assert!(focused.handled);
        assert_eq!(focused.focused_widget, Some(widget.0));
        let channel = keyboard.dispatch_key(NativeViewKey::Down);
        assert!(channel.handled);
        assert!(channel.color_picker_channel_changed);
        assert_eq!(
            keyboard
                .widget_color_picker_state(widget)
                .map(|state| state.active_channel),
            Some(crate::ZsColorChannel::Green)
        );
        let maximum = keyboard.dispatch_key(NativeViewKey::End);
        assert!(maximum.handled);
        assert!(maximum.color_picker_value_changed);
        assert_eq!(
            keyboard
                .widget_color_picker_state(widget)
                .map(|state| state.color.g),
            Some(255)
        );
        let closed = keyboard.dispatch_key(NativeViewKey::Escape);
        assert!(closed.color_picker_expanded_changed);
        assert!(keyboard
            .widget_color_picker_state(widget)
            .is_some_and(|state| !state.expanded));
        let reopened = keyboard.dispatch_key(NativeViewKey::Space);
        assert!(reopened.color_picker_expanded_changed);
        assert!(keyboard
            .widget_color_picker_state(widget)
            .is_some_and(|state| state.expanded));
    }
}
