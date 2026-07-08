use crate::{
    AppEvent, HostCapabilities, MenuSpec, Point, SettingsPageSpec, Size, TraySpec, UiCommand,
    UiRect, WindowSpec,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeRuntimeStartupRequest {
    pub app_name: String,
    pub main_window: NativeMainWindowRequest,
    pub status_item_tooltip: Option<String>,
    pub status_item: Option<TraySpec>,
    pub settings_pages: Vec<SettingsPageSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRuntimeStartupResult<Handle: Copy + Eq> {
    Started(NativeMainWindowHandles<Handle>),
    Failed,
}

pub trait NativeRuntimeDriver<UiCommandT = UiCommand, ApplicationEventT = AppEvent> {
    type WindowHandle: Copy + Eq;

    fn start_runtime(
        &mut self,
        request: NativeRuntimeStartupRequest,
    ) -> NativeRuntimeStartupResult<Self::WindowHandle>;
    fn dispatch_ui_command(&mut self, command: UiCommandT);
    fn poll_application_event(&mut self) -> Option<ApplicationEventT>;
    fn request_shutdown(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRuntimeDriverOperation {
    StartRuntime,
    DispatchUiCommand,
    PollApplicationEvent,
    RequestShutdown,
}

impl NativeRuntimeDriverOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::StartRuntime => "start_runtime",
            Self::DispatchUiCommand => "dispatch_ui_command",
            Self::PollApplicationEvent => "poll_application_event",
            Self::RequestShutdown => "request_shutdown",
        }
    }
}

pub const REQUIRED_NATIVE_RUNTIME_DRIVER_OPERATIONS: [NativeRuntimeDriverOperation; 4] = [
    NativeRuntimeDriverOperation::StartRuntime,
    NativeRuntimeDriverOperation::DispatchUiCommand,
    NativeRuntimeDriverOperation::PollApplicationEvent,
    NativeRuntimeDriverOperation::RequestShutdown,
];

pub fn required_native_runtime_driver_operation_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_RUNTIME_DRIVER_OPERATIONS
        .iter()
        .map(|operation| operation.operation_name())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMainWindowRequest {
    pub title: String,
    pub size: Size,
    pub options: NativeWindowOptions,
    pub main_visible: bool,
    pub degraded_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWindowOptions {
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub transparent: bool,
}

impl NativeWindowOptions {
    pub const fn standard() -> Self {
        Self {
            min_size: None,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
        }
    }

    pub const fn tool_window() -> Self {
        Self {
            min_size: None,
            resizable: false,
            decorations: false,
            always_on_top: true,
            transparent: false,
        }
    }

    pub const fn from_parts(
        min_size: Option<Size>,
        resizable: bool,
        decorations: bool,
        always_on_top: bool,
        transparent: bool,
    ) -> Self {
        Self {
            min_size,
            resizable,
            decorations,
            always_on_top,
            transparent,
        }
    }

    pub const fn with_min_size(mut self, size: Size) -> Self {
        self.min_size = Some(size);
        self
    }

    pub fn from_zsui_window(window: &WindowSpec) -> Self {
        let min_size = match (window.min_width, window.min_height) {
            (Some(width), Some(height)) => Some(Size {
                width: u32_to_i32_saturating(width).max(1),
                height: u32_to_i32_saturating(height).max(1),
            }),
            _ => None,
        };
        Self {
            min_size,
            resizable: window.resizable,
            decorations: window.decorations,
            always_on_top: window.always_on_top,
            transparent: window.transparent,
        }
    }

    pub fn from_zsui_window_for_host(window: &WindowSpec, capabilities: &HostCapabilities) -> Self {
        let resolved = window.resolve_for(capabilities);
        Self::from_zsui_window(&resolved.effective)
    }
}

impl Default for NativeWindowOptions {
    fn default() -> Self {
        Self::standard()
    }
}

impl NativeMainWindowRequest {
    pub fn from_zsui_window(window: &WindowSpec) -> Self {
        Self {
            title: window.title.clone(),
            size: Size {
                width: u32_to_i32_saturating(window.width).max(1),
                height: u32_to_i32_saturating(window.height).max(1),
            },
            options: NativeWindowOptions::from_zsui_window(window),
            main_visible: window.visible,
            degraded_capabilities: Vec::new(),
        }
    }

    pub fn from_zsui_window_for_host(window: &WindowSpec, capabilities: &HostCapabilities) -> Self {
        let resolved = window.resolve_for(capabilities);
        let mut request = Self::from_zsui_window(&resolved.effective);
        request.degraded_capabilities = resolved.degraded_capabilities;
        request
    }
}

fn u32_to_i32_saturating(value: u32) -> i32 {
    value.min(i32::MAX as u32) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMainWindowHandles<Handle: Copy + Eq> {
    pub main: Handle,
    pub quick: Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainWindowPresentation<Handle: Copy + Eq> {
    Created(NativeMainWindowHandles<Handle>),
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainWindowPresentMode {
    ActivateAndFocus,
    NoActivate,
}

pub trait NativeMainWindowHost {
    type Handle: Copy + Eq;
    type AppIcon: Copy + Eq;

    fn create_main_windows(
        &mut self,
        request: NativeMainWindowRequest,
    ) -> NativeMainWindowPresentation<Self::Handle>;
    fn apply_main_window_appearance(&mut self, handle: Self::Handle);
    fn set_main_window_app_icon(
        &mut self,
        handle: Self::Handle,
        icon: NativeAppIconResource<Self::AppIcon>,
    );
    fn hide_main_window(&mut self, handle: Self::Handle);
    fn present_main_window(&mut self, handle: Self::Handle, mode: NativeMainWindowPresentMode);
    fn set_main_window_bounds(&mut self, handle: Self::Handle, bounds: UiRect);
    fn activate_main_window(&mut self, handle: Self::Handle);
    fn foreground_main_window(&mut self, handle: Self::Handle);
    fn restore_main_window(&mut self, handle: Self::Handle);
    fn close_main_window(&mut self, handle: Self::Handle);
    fn set_main_window_activation_policy(&mut self, handle: Self::Handle, allow_activation: bool);
    fn request_main_window_close(&mut self, handle: Self::Handle);
    fn destroy_main_window(&mut self, handle: Self::Handle);
    fn capture_main_pointer(&mut self, handle: Self::Handle);
    fn release_main_pointer(&mut self, handle: Self::Handle);
    fn begin_main_window_drag(&mut self, handle: Self::Handle);
    fn track_main_pointer_leave(&mut self, handle: Self::Handle) -> bool;
    fn request_main_window_area_repaint(
        &mut self,
        handle: Self::Handle,
        area: Option<UiRect>,
        erase: bool,
    ) -> bool;
    fn main_window_layout_dpi(&mut self, handle: Self::Handle) -> u32;
    fn main_window_client_bounds(&mut self, handle: Self::Handle) -> Option<UiRect>;
    fn main_window_bounds(&mut self, handle: Self::Handle) -> Option<UiRect>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainWindowHostOperation {
    CreateMainWindows,
    ApplyMainWindowAppearance,
    SetMainWindowAppIcon,
    HideMainWindow,
    PresentMainWindow,
    SetMainWindowBounds,
    ActivateMainWindow,
    ForegroundMainWindow,
    RestoreMainWindow,
    CloseMainWindow,
    SetMainWindowActivationPolicy,
    RequestMainWindowClose,
    DestroyMainWindow,
    CaptureMainPointer,
    ReleaseMainPointer,
    BeginMainWindowDrag,
    TrackMainPointerLeave,
    RequestMainWindowAreaRepaint,
    MainWindowLayoutDpi,
    MainWindowClientBounds,
    MainWindowBounds,
}

impl NativeMainWindowHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::CreateMainWindows => "create_main_windows",
            Self::ApplyMainWindowAppearance => "apply_main_window_appearance",
            Self::SetMainWindowAppIcon => "set_main_window_app_icon",
            Self::HideMainWindow => "hide_main_window",
            Self::PresentMainWindow => "present_main_window",
            Self::SetMainWindowBounds => "set_main_window_bounds",
            Self::ActivateMainWindow => "activate_main_window",
            Self::ForegroundMainWindow => "foreground_main_window",
            Self::RestoreMainWindow => "restore_main_window",
            Self::CloseMainWindow => "close_main_window",
            Self::SetMainWindowActivationPolicy => "set_main_window_activation_policy",
            Self::RequestMainWindowClose => "request_main_window_close",
            Self::DestroyMainWindow => "destroy_main_window",
            Self::CaptureMainPointer => "capture_main_pointer",
            Self::ReleaseMainPointer => "release_main_pointer",
            Self::BeginMainWindowDrag => "begin_main_window_drag",
            Self::TrackMainPointerLeave => "track_main_pointer_leave",
            Self::RequestMainWindowAreaRepaint => "request_main_window_area_repaint",
            Self::MainWindowLayoutDpi => "main_window_layout_dpi",
            Self::MainWindowClientBounds => "main_window_client_bounds",
            Self::MainWindowBounds => "main_window_bounds",
        }
    }
}

pub const REQUIRED_NATIVE_MAIN_WINDOW_HOST_OPERATIONS: [NativeMainWindowHostOperation; 21] = [
    NativeMainWindowHostOperation::CreateMainWindows,
    NativeMainWindowHostOperation::ApplyMainWindowAppearance,
    NativeMainWindowHostOperation::SetMainWindowAppIcon,
    NativeMainWindowHostOperation::HideMainWindow,
    NativeMainWindowHostOperation::PresentMainWindow,
    NativeMainWindowHostOperation::SetMainWindowBounds,
    NativeMainWindowHostOperation::ActivateMainWindow,
    NativeMainWindowHostOperation::ForegroundMainWindow,
    NativeMainWindowHostOperation::RestoreMainWindow,
    NativeMainWindowHostOperation::CloseMainWindow,
    NativeMainWindowHostOperation::SetMainWindowActivationPolicy,
    NativeMainWindowHostOperation::RequestMainWindowClose,
    NativeMainWindowHostOperation::DestroyMainWindow,
    NativeMainWindowHostOperation::CaptureMainPointer,
    NativeMainWindowHostOperation::ReleaseMainPointer,
    NativeMainWindowHostOperation::BeginMainWindowDrag,
    NativeMainWindowHostOperation::TrackMainPointerLeave,
    NativeMainWindowHostOperation::RequestMainWindowAreaRepaint,
    NativeMainWindowHostOperation::MainWindowLayoutDpi,
    NativeMainWindowHostOperation::MainWindowClientBounds,
    NativeMainWindowHostOperation::MainWindowBounds,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeAppIconResource<Icon: Copy + Eq> {
    pub small: Icon,
    pub big: Icon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeStatusItemRequest {
    pub tooltip: Option<String>,
    pub icon_path: Option<String>,
    pub menu: MenuSpec,
}

impl NativeStatusItemRequest {
    pub fn from_tray_spec(spec: &TraySpec) -> Self {
        Self {
            tooltip: spec.tooltip.clone(),
            icon_path: spec.icon_path.clone(),
            menu: spec.menu.clone(),
        }
    }

    pub fn into_tray_spec(self) -> TraySpec {
        TraySpec {
            tooltip: self.tooltip,
            icon_path: self.icon_path,
            menu: self.menu,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeStatusItemPresentation<Handle: Copy + Eq> {
    Created(Handle),
    Failed,
}

pub trait NativeStatusItemHost {
    type Handle: Copy + Eq;

    fn create_status_item(
        &mut self,
        request: NativeStatusItemRequest,
    ) -> NativeStatusItemPresentation<Self::Handle>;
    fn set_status_item_tooltip(&mut self, handle: Self::Handle, tooltip: Option<String>);
    fn set_status_item_menu(&mut self, handle: Self::Handle, menu: MenuSpec);
    fn destroy_status_item(&mut self, handle: Self::Handle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeStatusItemHostOperation {
    CreateStatusItem,
    SetStatusItemTooltip,
    SetStatusItemMenu,
    DestroyStatusItem,
}

impl NativeStatusItemHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::CreateStatusItem => "create_status_item",
            Self::SetStatusItemTooltip => "set_status_item_tooltip",
            Self::SetStatusItemMenu => "set_status_item_menu",
            Self::DestroyStatusItem => "destroy_status_item",
        }
    }
}

pub const REQUIRED_NATIVE_STATUS_ITEM_HOST_OPERATIONS: [NativeStatusItemHostOperation; 4] = [
    NativeStatusItemHostOperation::CreateStatusItem,
    NativeStatusItemHostOperation::SetStatusItemTooltip,
    NativeStatusItemHostOperation::SetStatusItemMenu,
    NativeStatusItemHostOperation::DestroyStatusItem,
];

pub fn required_native_status_item_host_operation_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_STATUS_ITEM_HOST_OPERATIONS
        .iter()
        .map(|operation| operation.operation_name())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMainSearchControlRequest<Owner: Copy + Eq> {
    pub owner: Owner,
    pub id: i64,
    pub bounds: UiRect,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMainSearchStyleRequest<Handle: Copy + Eq, StyleResource: Copy + Eq> {
    pub handle: Handle,
    pub font_family: String,
    pub font_px: i32,
    pub previous_resource: Option<StyleResource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainSearchStylePresentation<StyleResource: Copy + Eq> {
    Applied(Option<StyleResource>),
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainSearchControlPresentation<Handle: Copy + Eq> {
    Created(Handle),
    Failed,
}

pub trait NativeMainSearchControlHost {
    type Owner: Copy + Eq;
    type Handle: Copy + Eq;
    type StyleResource: Copy + Eq;

    fn create_search_control(
        &mut self,
        request: NativeMainSearchControlRequest<Self::Owner>,
    ) -> NativeMainSearchControlPresentation<Self::Handle>;
    fn apply_search_style(
        &mut self,
        request: NativeMainSearchStyleRequest<Self::Handle, Self::StyleResource>,
    ) -> NativeMainSearchStylePresentation<Self::StyleResource>;
    fn release_search_style_resource(&mut self, resource: Self::StyleResource);
    fn set_search_bounds(&mut self, handle: Self::Handle, bounds: UiRect);
    fn set_search_visible(&mut self, handle: Self::Handle, visible: bool);
    fn search_text(&self, handle: Self::Handle) -> String;
    fn set_search_text(&mut self, handle: Self::Handle, text: &str);
    fn focus_search(&mut self, handle: Self::Handle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMainSearchControlHostOperation {
    CreateSearchControl,
    ApplySearchStyle,
    ReleaseSearchStyleResource,
    SetSearchBounds,
    SetSearchVisible,
    SearchText,
    SetSearchText,
    FocusSearch,
}

impl NativeMainSearchControlHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::CreateSearchControl => "create_search_control",
            Self::ApplySearchStyle => "apply_search_style",
            Self::ReleaseSearchStyleResource => "release_search_style_resource",
            Self::SetSearchBounds => "set_search_bounds",
            Self::SetSearchVisible => "set_search_visible",
            Self::SearchText => "search_text",
            Self::SetSearchText => "set_search_text",
            Self::FocusSearch => "focus_search",
        }
    }
}

pub const REQUIRED_NATIVE_MAIN_SEARCH_CONTROL_HOST_OPERATIONS:
    [NativeMainSearchControlHostOperation; 8] = [
    NativeMainSearchControlHostOperation::CreateSearchControl,
    NativeMainSearchControlHostOperation::ApplySearchStyle,
    NativeMainSearchControlHostOperation::ReleaseSearchStyleResource,
    NativeMainSearchControlHostOperation::SetSearchBounds,
    NativeMainSearchControlHostOperation::SetSearchVisible,
    NativeMainSearchControlHostOperation::SearchText,
    NativeMainSearchControlHostOperation::SetSearchText,
    NativeMainSearchControlHostOperation::FocusSearch,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSettingsWindowRequest<Handle: Copy + Eq> {
    pub owner: Handle,
    pub existing: Option<Handle>,
    pub bounds: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsWindowPresentation<Handle: Copy + Eq> {
    FocusedExisting(Handle),
    Created(Handle),
    Failed,
}

pub trait NativeSettingsWindowHost {
    type Handle: Copy + Eq;

    fn present_settings_window(
        &mut self,
        request: NativeSettingsWindowRequest<Self::Handle>,
    ) -> NativeSettingsWindowPresentation<Self::Handle>;
    fn set_settings_window_bounds(&mut self, handle: Self::Handle, bounds: UiRect);
    fn destroy_settings_window(&mut self, handle: Self::Handle);
    fn focus_settings_window(&mut self, handle: Self::Handle);
    fn track_settings_pointer_leave(&mut self, handle: Self::Handle) -> bool;
    fn capture_settings_pointer(&mut self, handle: Self::Handle);
    fn release_settings_pointer(&mut self, handle: Self::Handle);
    fn request_settings_window_repaint(&mut self, handle: Self::Handle) -> bool;
    fn request_settings_window_area_repaint(
        &mut self,
        handle: Self::Handle,
        area: Option<UiRect>,
        erase: bool,
    ) -> bool;
    fn settings_window_layout_dpi(&mut self, handle: Self::Handle) -> u32;
    fn settings_window_client_to_screen(
        &mut self,
        handle: Self::Handle,
        point: Point,
    ) -> Option<Point>;
    fn settings_window_client_bounds(&mut self, handle: Self::Handle) -> Option<UiRect>;
    fn settings_window_bounds(&mut self, handle: Self::Handle) -> Option<UiRect>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsWindowHostOperation {
    PresentSettingsWindow,
    SetSettingsWindowBounds,
    DestroySettingsWindow,
    FocusSettingsWindow,
    TrackSettingsPointerLeave,
    CaptureSettingsPointer,
    ReleaseSettingsPointer,
    RequestSettingsWindowRepaint,
    RequestSettingsWindowAreaRepaint,
    SettingsWindowLayoutDpi,
    SettingsWindowClientToScreen,
    SettingsWindowClientBounds,
    SettingsWindowBounds,
}

impl NativeSettingsWindowHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::PresentSettingsWindow => "present_settings_window",
            Self::SetSettingsWindowBounds => "set_settings_window_bounds",
            Self::DestroySettingsWindow => "destroy_settings_window",
            Self::FocusSettingsWindow => "focus_settings_window",
            Self::TrackSettingsPointerLeave => "track_settings_pointer_leave",
            Self::CaptureSettingsPointer => "capture_settings_pointer",
            Self::ReleaseSettingsPointer => "release_settings_pointer",
            Self::RequestSettingsWindowRepaint => "request_settings_window_repaint",
            Self::RequestSettingsWindowAreaRepaint => "request_settings_window_area_repaint",
            Self::SettingsWindowLayoutDpi => "settings_window_layout_dpi",
            Self::SettingsWindowClientToScreen => "settings_window_client_to_screen",
            Self::SettingsWindowClientBounds => "settings_window_client_bounds",
            Self::SettingsWindowBounds => "settings_window_bounds",
        }
    }
}

pub const REQUIRED_NATIVE_SETTINGS_WINDOW_HOST_OPERATIONS: [NativeSettingsWindowHostOperation; 13] = [
    NativeSettingsWindowHostOperation::PresentSettingsWindow,
    NativeSettingsWindowHostOperation::SetSettingsWindowBounds,
    NativeSettingsWindowHostOperation::DestroySettingsWindow,
    NativeSettingsWindowHostOperation::FocusSettingsWindow,
    NativeSettingsWindowHostOperation::TrackSettingsPointerLeave,
    NativeSettingsWindowHostOperation::CaptureSettingsPointer,
    NativeSettingsWindowHostOperation::ReleaseSettingsPointer,
    NativeSettingsWindowHostOperation::RequestSettingsWindowRepaint,
    NativeSettingsWindowHostOperation::RequestSettingsWindowAreaRepaint,
    NativeSettingsWindowHostOperation::SettingsWindowLayoutDpi,
    NativeSettingsWindowHostOperation::SettingsWindowClientToScreen,
    NativeSettingsWindowHostOperation::SettingsWindowClientBounds,
    NativeSettingsWindowHostOperation::SettingsWindowBounds,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSettingsPageModelRequest {
    pub pages: Vec<SettingsPageSpec>,
}

impl NativeSettingsPageModelRequest {
    pub fn new(pages: impl IntoIterator<Item = SettingsPageSpec>) -> Self {
        Self {
            pages: pages.into_iter().collect(),
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn item_count(&self) -> usize {
        self.pages.iter().map(|page| page.items.len()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsPageModelPresentation {
    Bound {
        page_count: usize,
        item_count: usize,
    },
    Failed,
}

pub trait NativeSettingsPageModelHost {
    fn bind_settings_pages(
        &mut self,
        request: NativeSettingsPageModelRequest,
    ) -> NativeSettingsPageModelPresentation;
    fn update_settings_pages(&mut self, request: NativeSettingsPageModelRequest);
    fn clear_settings_pages(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsPageModelHostOperation {
    BindSettingsPages,
    UpdateSettingsPages,
    ClearSettingsPages,
}

impl NativeSettingsPageModelHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::BindSettingsPages => "bind_settings_pages",
            Self::UpdateSettingsPages => "update_settings_pages",
            Self::ClearSettingsPages => "clear_settings_pages",
        }
    }
}

pub const REQUIRED_NATIVE_SETTINGS_PAGE_MODEL_HOST_OPERATIONS:
    [NativeSettingsPageModelHostOperation; 3] = [
    NativeSettingsPageModelHostOperation::BindSettingsPages,
    NativeSettingsPageModelHostOperation::UpdateSettingsPages,
    NativeSettingsPageModelHostOperation::ClearSettingsPages,
];

pub fn required_native_settings_page_model_host_operation_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_SETTINGS_PAGE_MODEL_HOST_OPERATIONS
        .iter()
        .map(|operation| operation.operation_name())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSettingsDropdownRequest<Owner: Copy + Eq> {
    pub owner: Owner,
    pub control_id: isize,
    pub anchor: UiRect,
    pub items: Vec<String>,
    pub selected: usize,
    pub width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsDropdownPresentation<Handle: Copy + Eq> {
    Created(Handle),
    Failed,
}

pub trait NativeSettingsDropdownHost {
    type Handle: Copy + Eq;
    type Owner: Copy + Eq;

    fn present_settings_dropdown(
        &mut self,
        request: NativeSettingsDropdownRequest<Self::Owner>,
    ) -> NativeSettingsDropdownPresentation<Self::Handle>;
    fn destroy_settings_dropdown(&mut self, handle: Self::Handle);
    fn settings_dropdown_bounds(&self, handle: Self::Handle) -> Option<UiRect>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSettingsDropdownHostOperation {
    PresentSettingsDropdown,
    DestroySettingsDropdown,
    SettingsDropdownBounds,
}

impl NativeSettingsDropdownHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::PresentSettingsDropdown => "present_settings_dropdown",
            Self::DestroySettingsDropdown => "destroy_settings_dropdown",
            Self::SettingsDropdownBounds => "settings_dropdown_bounds",
        }
    }
}

pub const REQUIRED_NATIVE_SETTINGS_DROPDOWN_HOST_OPERATIONS: [NativeSettingsDropdownHostOperation;
    3] = [
    NativeSettingsDropdownHostOperation::PresentSettingsDropdown,
    NativeSettingsDropdownHostOperation::DestroySettingsDropdown,
    NativeSettingsDropdownHostOperation::SettingsDropdownBounds,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapabilitySupport, HostCapabilities, PlatformName, UiCommand};

    #[derive(Default)]
    struct RecordingRuntimeDriver {
        started: bool,
        commands: Vec<UiCommand>,
        events: Vec<crate::AppEvent>,
        shutdown_requested: bool,
    }

    impl NativeRuntimeDriver for RecordingRuntimeDriver {
        type WindowHandle = u32;

        fn start_runtime(
            &mut self,
            request: NativeRuntimeStartupRequest,
        ) -> NativeRuntimeStartupResult<Self::WindowHandle> {
            assert_eq!(request.app_name, "Example");
            assert_eq!(request.main_window.title, "Example");
            self.started = true;
            NativeRuntimeStartupResult::Started(NativeMainWindowHandles { main: 1, quick: 2 })
        }

        fn dispatch_ui_command(&mut self, command: UiCommand) {
            self.commands.push(command);
            self.events.push(crate::AppEvent::Started);
        }

        fn poll_application_event(&mut self) -> Option<crate::AppEvent> {
            self.events.pop()
        }

        fn request_shutdown(&mut self) {
            self.shutdown_requested = true;
        }
    }

    #[test]
    fn native_runtime_driver_contract_executes_framework_path() {
        let mut driver = RecordingRuntimeDriver::default();
        let startup = driver.start_runtime(NativeRuntimeStartupRequest {
            app_name: "Example".to_string(),
            main_window: NativeMainWindowRequest {
                title: "Example".to_string(),
                size: Size {
                    width: 320,
                    height: 240,
                },
                options: NativeWindowOptions::standard(),
                main_visible: true,
                degraded_capabilities: Vec::new(),
            },
            status_item_tooltip: Some("Example".to_string()),
            status_item: None,
            settings_pages: Vec::new(),
        });

        assert_eq!(
            startup,
            NativeRuntimeStartupResult::Started(NativeMainWindowHandles { main: 1, quick: 2 })
        );
        driver.dispatch_ui_command(UiCommand::app(crate::CommandId("example.open")));
        assert_eq!(driver.commands[0].id.0, "example.open");
        assert_eq!(
            driver.poll_application_event(),
            Some(crate::AppEvent::Started)
        );
        driver.request_shutdown();
        assert!(driver.started);
        assert!(driver.shutdown_requested);
    }

    #[test]
    fn native_window_request_resolves_unsupported_traits_for_host() {
        let mut capabilities = HostCapabilities::all_supported(PlatformName::Unknown);
        capabilities.window_always_on_top = CapabilitySupport::unsupported("topmost unavailable");
        capabilities.window_transparency =
            CapabilitySupport::unsupported("transparency unavailable");

        let window = WindowSpec::new("Example")
            .always_on_top(true)
            .transparent(true);
        let request = NativeMainWindowRequest::from_zsui_window_for_host(&window, &capabilities);

        assert_eq!(request.title, "Example");
        assert!(!request.options.always_on_top);
        assert!(!request.options.transparent);
        assert_eq!(request.degraded_capabilities.len(), 2);
    }

    #[test]
    fn native_main_window_host_operations_are_stable() {
        let names: Vec<_> = REQUIRED_NATIVE_MAIN_WINDOW_HOST_OPERATIONS
            .iter()
            .map(|operation| operation.operation_name())
            .collect();

        assert_eq!(names.len(), 21);
        assert_eq!(names[0], "create_main_windows");
        assert_eq!(names[20], "main_window_bounds");
    }

    #[test]
    fn status_item_and_settings_model_operations_are_stable() {
        assert_eq!(
            required_native_status_item_host_operation_names(),
            vec![
                "create_status_item",
                "set_status_item_tooltip",
                "set_status_item_menu",
                "destroy_status_item"
            ]
        );
        assert_eq!(
            required_native_settings_page_model_host_operation_names(),
            vec![
                "bind_settings_pages",
                "update_settings_pages",
                "clear_settings_pages"
            ]
        );
    }
}
