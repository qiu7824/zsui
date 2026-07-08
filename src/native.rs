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
    host::{MemoryHost, TrayRecord, WindowRecord, ZsuiHost},
    hotkey::HotkeySpec,
    menu::{MenuItemSpec, MenuSpec},
    native_hosts::{
        NativeMainWindowHandles, NativeRuntimeDriver, NativeRuntimeStartupRequest,
        NativeRuntimeStartupResult, NativeSettingsPageModelHost,
        NativeSettingsPageModelPresentation, NativeSettingsPageModelRequest, NativeStatusItemHost,
        NativeStatusItemPresentation, NativeStatusItemRequest,
    },
    settings::SettingsPageSpec,
    tray::TraySpec,
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
        run_native_window_smoke_event_loop(self.windows.clone(), options)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeWindowSmokeRunOptions {
    pub auto_close_after_ms: u64,
    pub require_visible_window: bool,
    pub screenshot_file: Option<String>,
    pub require_screenshot: bool,
}

impl NativeWindowSmokeRunOptions {
    pub const fn new(auto_close_after_ms: u64) -> Self {
        Self {
            auto_close_after_ms,
            require_visible_window: true,
            screenshot_file: None,
            require_screenshot: false,
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
    pub events: Vec<String>,
}

impl NativeWindowSmokeRunReport {
    pub fn empty(options: NativeWindowSmokeRunOptions) -> Self {
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
            events: Vec::new(),
        }
    }

    pub fn visible_window_was_created(&self) -> bool {
        self.created_window_count > 0 && self.startup_error.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWindowBuilder {
    app_name: String,
    window: WindowSpec,
}

impl NativeWindowBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            app_name: title.clone(),
            window: Window::new(title),
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

    pub fn window_spec(&self) -> &WindowSpec {
        &self.window
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        app(self.app_name).window(self.window).build()
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        app.run_with_host(&mut host)
    }

    pub fn run_smoke(
        self,
        options: NativeWindowSmokeRunOptions,
    ) -> ZsuiResult<NativeWindowSmokeRunReport> {
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        for window in &app.windows {
            host.create_main_window(window)?;
        }
        run_native_window_smoke_event_loop(host.windows.clone(), options)
    }
}

#[derive(Debug, Clone)]
pub struct NativeWindowHost {
    inner: MemoryHost,
    windows: Vec<WindowSpec>,
}

impl NativeWindowHost {
    pub fn new() -> Self {
        Self {
            inner: MemoryHost::with_capabilities(HostCapabilities::current_native_window_host()),
            windows: Vec::new(),
        }
    }

    pub fn recorded_windows(&self) -> &[WindowRecord] {
        self.inner.windows()
    }

    pub fn recorded_trays(&self) -> &[TrayRecord] {
        self.inner.trays()
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
        Ok(id)
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()> {
        self.inner.set_window_visible(window, visible)
    }

    fn create_tray(&mut self, spec: &TraySpec) -> ZsuiResult<TrayId> {
        self.inner.create_tray(spec)
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
        run_native_window_event_loop(self.windows.clone())
    }
}

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
fn run_native_window_event_loop(windows: Vec<WindowSpec>) -> ZsuiResult<()> {
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

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
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
    event_loop
        .run_app(&mut app)
        .map_err(|err| ZsuiError::host("native_window_smoke_event_loop", err.to_string()))?;

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

    Ok(app.report)
}

#[cfg(windows)]
fn capture_first_native_window_png(
    windows: &std::collections::HashMap<winit::window::WindowId, winit::window::Window>,
    path: &str,
) -> Result<(), String> {
    let window = windows
        .values()
        .next()
        .ok_or_else(|| "no native window exists for screenshot capture".to_string())?;
    capture_winit_window_png(window, path)
}

#[cfg(all(
    not(windows),
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

#[cfg(windows)]
fn capture_winit_window_png(window: &winit::window::Window, path: &str) -> Result<(), String> {
    use std::{ffi::c_void, mem, path::Path};
    use windows_sys::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
            GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
            DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
        },
        UI::WindowsAndMessaging::GetClientRect,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let raw = window
        .window_handle()
        .map_err(|err| err.to_string())?
        .as_raw();
    let hwnd = match raw {
        RawWindowHandle::Win32(handle) => handle.hwnd.get() as isize as HWND,
        _ => return Err("native smoke screenshot requires a Win32 window handle".to_string()),
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
    let hdc = unsafe { GetDC(hwnd) };
    if hdc.is_null() {
        return Err("GetDC failed".to_string());
    }

    let result = (|| {
        let memory_dc = unsafe { CreateCompatibleDC(hdc) };
        if memory_dc.is_null() {
            return Err("CreateCompatibleDC failed".to_string());
        }

        let bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
        if bitmap.is_null() {
            unsafe {
                DeleteDC(memory_dc);
            }
            return Err("CreateCompatibleBitmap failed".to_string());
        }

        let old_object = unsafe { SelectObject(memory_dc, bitmap.cast()) };
        let blit_ok = unsafe { BitBlt(memory_dc, 0, 0, width, height, hdc, 0, 0, SRCCOPY) };
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
                    memory_dc,
                    bitmap,
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

        if !old_object.is_null() {
            unsafe {
                SelectObject(memory_dc, old_object);
            }
        }
        unsafe {
            DeleteObject(bitmap.cast());
            DeleteDC(memory_dc);
        }

        if blit_ok == 0 {
            return Err("BitBlt failed".to_string());
        }
        if dib_lines == 0 {
            return Err("GetDIBits failed".to_string());
        }

        let rgba = bgra_to_rgba(&bgra);
        write_rgba_png(Path::new(path), width as u32, height as u32, &rgba)
    })();

    unsafe {
        ReleaseDC(hwnd, hdc);
    }

    result
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
fn run_native_window_event_loop(_windows: Vec<WindowSpec>) -> ZsuiResult<()> {
    Err(ZsuiError::unsupported(
        "native_window",
        "desktop native windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts",
    ))
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
fn run_native_window_smoke_event_loop(
    _windows: Vec<WindowSpec>,
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

    #[test]
    fn native_window_smoke_options_have_short_default_runtime() {
        let options = NativeWindowSmokeRunOptions::quick();
        let report = NativeWindowSmokeRunReport::empty(options.clone());

        assert_eq!(options.auto_close_after_ms, 750);
        assert!(options.require_visible_window);
        assert_eq!(options.screenshot_file, None);
        assert!(!options.require_screenshot);
        assert_eq!(report.created_window_count, 0);
        assert!(!report.visible_window_was_created());
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
}
