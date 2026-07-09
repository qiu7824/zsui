use std::{
    mem::{size_of, zeroed},
    path::Path,
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicI32, Ordering},
        Mutex, OnceLock,
    },
};

use crate::windows_gdi_renderer::{
    rect_from_win, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette, WindowsGdiRenderer,
};
use crate::{
    native_status_menu_command_from_menu, Command, HostCapabilities, MenuItemSpec, MenuSpec,
    NativeAppIconResource, NativeDrawPlan, NativeMainWindowHandles, NativeMainWindowHost,
    NativeMainWindowHostOperation, NativeMainWindowPresentMode, NativeMainWindowPresentation,
    NativeMainWindowRequest, NativeStatusItemHost, NativeStatusItemHostOperation,
    NativeStatusItemPresentation, NativeStatusItemRequest, NativeStatusMenuCommandHost,
    NativeStatusMenuCommandHostOperation, NativeStatusMenuCommandRequest,
    NativeStatusMenuCommandResult, NativeTransientWindowHost, NativeTransientWindowHostOperation,
    NativeTransientWindowPresentation, NativeTransientWindowRequest, NativeWindowOptions, Renderer,
    Size, TraySpec, UiCommand, UiRect, View, ViewEventCx, ViewInteractionPlan, ViewNode,
    WindowSpec, ZsuiError, ZsuiResult,
};
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT,
        WPARAM,
    },
    Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, UpdateWindow, PAINTSTRUCT},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{
            ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_HOVER, TME_LEAVE,
            TRACKMOUSEEVENT,
        },
        Shell::{
            Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            NOTIFYICONDATAW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
            DestroyMenu, DestroyWindow, DispatchMessageW, GetClientRect, GetCursorPos, GetMessageW,
            GetSystemMetrics, GetWindowLongPtrW, GetWindowLongW, GetWindowRect, IsWindow,
            LoadCursorW, LoadImageW, PostMessageW, PostQuitMessage, RegisterClassExW, SendMessageW,
            SetForegroundWindow, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, ShowWindow,
            TrackPopupMenu, TranslateMessage, CREATESTRUCTW, CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, GWLP_USERDATA, GWL_EXSTYLE, HCURSOR, HICON, HMENU, HTCAPTION,
            HWND_TOPMOST, ICON_BIG, ICON_SMALL, IDC_ARROW, IMAGE_ICON, LR_DEFAULTCOLOR,
            LR_LOADFROMFILE, MF_CHECKED, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG,
            SC_MOVE, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, SWP_FRAMECHANGED,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
            SW_SHOWNOACTIVATE, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP, WM_CHAR,
            WM_CLOSE, WM_ERASEBKGND, WM_LBUTTONUP, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SETICON,
            WM_SYSCOMMAND, WNDCLASSEXW, WNDPROC, WS_CAPTION, WS_CLIPCHILDREN, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED,
            WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

static ACTIVE_MAIN_WINDOW_COUNT: AtomicI32 = AtomicI32::new(0);
static WINDOW_DRAW_PLANS: OnceLock<Mutex<Vec<WindowsWindowDrawPlanRecord>>> = OnceLock::new();
static WINDOW_VIEW_INPUT_ROUTES: OnceLock<Mutex<Vec<WindowsWindowViewInputRouteRecord>>> =
    OnceLock::new();

const HOVER_DEFAULT: u32 = u32::MAX;
const DEFAULT_MAIN_CLASS_NAME: &str = "ZsuiMainWindow";
const DEFAULT_QUICK_CLASS_NAME: &str = "ZsuiQuickWindow";
const DEFAULT_TRANSIENT_CLASS_NAME: &str = "ZsuiTransientWindow";
pub const ZSUI_WIN32_TRAY_CALLBACK_MESSAGE: u32 = WM_APP + 0x0551;
pub const ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS: u32 = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsWindowDrawPlanRecord {
    hwnd: isize,
    plan: NativeDrawPlan,
}

#[derive(Debug, Clone)]
struct WindowsWindowViewInputRouteRecord {
    hwnd: isize,
    route: WindowsWin32ViewInputRoute,
    report: WindowsWin32ViewInputDispatchReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(isize)]
pub enum WindowsWindowRole {
    Main = 1,
    Quick = 2,
}

impl WindowsWindowRole {
    pub const fn from_create_param(value: isize) -> Self {
        match value {
            value if value == Self::Quick as isize => Self::Quick,
            _ => Self::Main,
        }
    }

    pub const fn class_name(self, class_names: WindowsWin32ClassNames) -> &'static str {
        match self {
            Self::Main => class_names.main,
            Self::Quick => class_names.quick,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWin32ClassNames {
    pub main: &'static str,
    pub quick: &'static str,
}

impl WindowsWin32ClassNames {
    pub const fn new(main: &'static str, quick: &'static str) -> Self {
        Self { main, quick }
    }
}

impl Default for WindowsWin32ClassNames {
    fn default() -> Self {
        Self::new(DEFAULT_MAIN_CLASS_NAME, DEFAULT_QUICK_CLASS_NAME)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWindowCreateParams {
    pub role: WindowsWindowRole,
    pub min_size: Option<Size>,
}

impl WindowsWindowCreateParams {
    pub const fn new(role: WindowsWindowRole, min_size: Option<Size>) -> Self {
        Self { role, min_size }
    }

    pub fn from_create_param(value: isize) -> Self {
        if value == WindowsWindowRole::Quick as isize || value == WindowsWindowRole::Main as isize {
            return Self::new(WindowsWindowRole::from_create_param(value), None);
        }
        let params = value as *const Self;
        if params.is_null() {
            Self::new(WindowsWindowRole::Main, None)
        } else {
            unsafe { *params }
        }
    }

    pub unsafe fn from_create_struct(create_struct: *const CREATESTRUCTW) -> Self {
        if create_struct.is_null() {
            return Self::new(WindowsWindowRole::Main, None);
        }
        Self::from_create_param((*create_struct).lpCreateParams as isize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWin32WindowStylePlan {
    pub ex_style: u32,
    pub style: u32,
}

pub fn windows_win32_main_window_style_plan(
    role: WindowsWindowRole,
    options: &NativeWindowOptions,
) -> WindowsWin32WindowStylePlan {
    let mut ex_style = 0;
    if !options.decorations {
        ex_style |= WS_EX_TOOLWINDOW;
    }
    if options.always_on_top {
        ex_style |= WS_EX_TOPMOST;
    }
    if matches!(role, WindowsWindowRole::Quick) {
        ex_style |= WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE;
    }

    let style = if options.decorations {
        let mut style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        if options.resizable {
            style |= WS_MAXIMIZEBOX | WS_THICKFRAME;
        }
        style
    } else {
        WS_POPUP | WS_CLIPCHILDREN
    };

    WindowsWin32WindowStylePlan { ex_style, style }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsWin32MessageLoopResult {
    Quit(i32),
    Failed,
}

pub struct WindowsWin32MessageLoop;

impl WindowsWin32MessageLoop {
    pub fn run() -> WindowsWin32MessageLoopResult {
        let mut msg: MSG = unsafe { zeroed() };
        loop {
            let code = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if code == -1 {
                return WindowsWin32MessageLoopResult::Failed;
            }
            if code == 0 {
                return WindowsWin32MessageLoopResult::Quit(msg.wParam as i32);
            }
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

pub fn create_windows_for_specs(
    specs: &[WindowSpec],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    create_windows_for_specs_with_draw_plans(specs, &[])
}

pub fn create_windows_for_specs_with_draw_plans(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    create_windows_for_specs_with_draw_plans_and_input_routes(specs, draw_plans, &[])
}

pub fn create_windows_for_specs_with_draw_plans_and_input_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    let owned = create_owned_windows_for_specs_with_draw_plans_and_input_routes(
        specs,
        draw_plans,
        input_routes,
    )?;
    Ok(owned
        .into_iter()
        .map(WindowsWin32OwnedMainWindowHandles::into_handles)
        .collect())
}

pub fn create_owned_windows_for_specs(
    specs: &[WindowSpec],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    create_owned_windows_for_specs_with_draw_plans(specs, &[])
}

pub fn create_owned_windows_for_specs_with_draw_plans(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    create_owned_windows_for_specs_with_draw_plans_and_input_routes(specs, draw_plans, &[])
}

pub fn create_owned_windows_for_specs_with_draw_plans_and_input_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    ACTIVE_MAIN_WINDOW_COUNT.store(0, Ordering::SeqCst);
    clear_windows_win32_window_draw_plans();
    clear_windows_win32_window_view_input_routes();
    let capabilities = HostCapabilities::windows_native_window_host();
    let mut host = WindowsWin32MainWindowHost::new();
    let mut handles = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let request = NativeMainWindowRequest::from_zsui_window_for_host(spec, &capabilities);
        let icon_path = request.icon_path.clone();
        match host.create_main_windows(request) {
            NativeMainWindowPresentation::Created(created) => {
                if let Some(Some(plan)) = draw_plans.get(index) {
                    set_windows_win32_window_draw_plan(created.main, plan.clone());
                    host.request_main_window_area_repaint(created.main, None, false);
                    unsafe {
                        UpdateWindow(created.main);
                    }
                }
                if let Some(Some(route)) = input_routes.get(index) {
                    set_windows_win32_window_view_input_route(created.main, route.clone());
                }
                host.apply_main_window_appearance(created.main);
                host.apply_main_window_appearance(created.quick);
                let mut owned = WindowsWin32OwnedMainWindowHandles::new(created);
                if let Some(icon_path) = icon_path.as_deref() {
                    let icon = WindowsWin32OwnedAppIconResource::from_icon_path(icon_path)?;
                    owned.set_main_owned_app_icon(&mut host, icon);
                }
                handles.push(owned);
            }
            NativeMainWindowPresentation::Failed => {
                return Err(ZsuiError::host(
                    "create_windows_win32",
                    "failed to create Win32 main/quick windows",
                ));
            }
        }
    }
    Ok(handles)
}

#[derive(Debug)]
pub struct WindowsWin32OwnedMainWindowHandles {
    handles: NativeMainWindowHandles<HWND>,
    app_icons: Vec<WindowsWin32OwnedAppIconResource>,
    destroy_on_drop: bool,
}

impl WindowsWin32OwnedMainWindowHandles {
    pub fn new(handles: NativeMainWindowHandles<HWND>) -> Self {
        Self {
            handles,
            app_icons: Vec::new(),
            destroy_on_drop: true,
        }
    }

    pub const fn handles(&self) -> NativeMainWindowHandles<HWND> {
        self.handles
    }

    pub const fn main(&self) -> HWND {
        self.handles.main
    }

    pub const fn quick(&self) -> HWND {
        self.handles.quick
    }

    pub fn app_icon_count(&self) -> usize {
        self.app_icons.len()
    }

    pub fn set_main_owned_app_icon(
        &mut self,
        host: &mut WindowsWin32MainWindowHost,
        icon: WindowsWin32OwnedAppIconResource,
    ) {
        host.set_main_window_app_icon(self.main(), icon.as_native_resource());
        self.app_icons.push(icon);
    }

    pub fn into_handles(mut self) -> NativeMainWindowHandles<HWND> {
        self.destroy_on_drop = false;
        let handles = self.handles;
        std::mem::forget(self);
        handles
    }
}

impl Drop for WindowsWin32OwnedMainWindowHandles {
    fn drop(&mut self) {
        if !self.destroy_on_drop {
            return;
        }
        for handle in [self.handles.quick, self.handles.main] {
            if handle.is_null() {
                continue;
            }
            clear_windows_win32_window_draw_plan(handle);
            clear_windows_win32_window_view_input_route(handle);
            unsafe {
                if IsWindow(handle) != 0 {
                    DestroyWindow(handle);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedIcon {
    icon: HICON,
}

impl WindowsWin32OwnedIcon {
    pub fn from_raw(icon: HICON) -> Option<Self> {
        if icon.is_null() {
            None
        } else {
            Some(Self { icon })
        }
    }

    pub fn from_icon_path(path: impl AsRef<Path>, width: i32, height: i32) -> ZsuiResult<Self> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(ZsuiError::invalid_spec(
                "window.icon_path",
                "window icon path cannot be empty",
            ));
        }
        let path_wide = wide_path_null(path);
        let icon = unsafe {
            LoadImageW(
                null_mut(),
                path_wide.as_ptr(),
                IMAGE_ICON,
                width.max(1),
                height.max(1),
                LR_LOADFROMFILE | LR_DEFAULTCOLOR,
            )
        } as HICON;
        Self::from_raw(icon).ok_or_else(|| {
            ZsuiError::host(
                "windows_win32_load_icon",
                format!("failed to load icon from {}", path.display()),
            )
        })
    }

    pub const fn handle(&self) -> HICON {
        self.icon
    }

    pub fn into_raw(mut self) -> HICON {
        let icon = self.icon;
        self.icon = null_mut();
        icon
    }
}

impl Drop for WindowsWin32OwnedIcon {
    fn drop(&mut self) {
        if !self.icon.is_null() {
            unsafe {
                DestroyIcon(self.icon);
            }
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedAppIconResource {
    small: WindowsWin32OwnedIcon,
    big: Option<WindowsWin32OwnedIcon>,
}

impl WindowsWin32OwnedAppIconResource {
    pub fn from_raw(small: HICON, big: HICON) -> Option<Self> {
        let small_icon = WindowsWin32OwnedIcon::from_raw(small)?;
        let big_icon = if small == big {
            None
        } else {
            Some(WindowsWin32OwnedIcon::from_raw(big)?)
        };
        Some(Self {
            small: small_icon,
            big: big_icon,
        })
    }

    pub fn from_icon_path(path: impl AsRef<Path>) -> ZsuiResult<Self> {
        let path = path.as_ref();
        let small = WindowsWin32OwnedIcon::from_icon_path(
            path,
            system_metric(SM_CXSMICON),
            system_metric(SM_CYSMICON),
        )?;
        let big = WindowsWin32OwnedIcon::from_icon_path(
            path,
            system_metric(SM_CXICON),
            system_metric(SM_CYICON),
        )?;
        Ok(Self::from_owned_icons(small, big))
    }

    pub fn from_owned_icons(small: WindowsWin32OwnedIcon, big: WindowsWin32OwnedIcon) -> Self {
        let big_icon = if small.handle() == big.handle() {
            let _shared = big.into_raw();
            None
        } else {
            Some(big)
        };
        Self {
            small,
            big: big_icon,
        }
    }

    pub const fn small(&self) -> HICON {
        self.small.handle()
    }

    pub fn big(&self) -> HICON {
        self.big
            .as_ref()
            .map(WindowsWin32OwnedIcon::handle)
            .unwrap_or_else(|| self.small.handle())
    }

    pub fn as_native_resource(&self) -> NativeAppIconResource<isize> {
        NativeAppIconResource {
            small: self.small() as isize,
            big: self.big() as isize,
        }
    }

    pub fn into_raw_pair(self) -> (HICON, HICON) {
        let Self { small, big } = self;
        let small = small.into_raw();
        let big = big.map(WindowsWin32OwnedIcon::into_raw).unwrap_or(small);
        (small, big)
    }
}

fn system_metric(metric: i32) -> i32 {
    unsafe { GetSystemMetrics(metric).max(1) }
}

pub const ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID: u32 = 0x5800;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsWin32StatusMenuCommandEntry {
    pub native_id: u32,
    pub item_id: Option<String>,
    pub label: String,
    pub command: Command,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsWin32StatusMenuCommandTable {
    entries: Vec<WindowsWin32StatusMenuCommandEntry>,
}

impl WindowsWin32StatusMenuCommandTable {
    pub fn from_menu(menu: &MenuSpec) -> Self {
        let mut entries = Vec::new();
        let mut next_id = ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID;
        collect_status_menu_commands(menu, true, &mut next_id, &mut entries);
        Self { entries }
    }

    pub fn entries(&self) -> &[WindowsWin32StatusMenuCommandEntry] {
        &self.entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn first_native_id(&self) -> Option<u32> {
        self.entries.first().map(|entry| entry.native_id)
    }

    pub fn resolve_native_command_id(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.native_id == native_id)
        else {
            return NativeStatusMenuCommandResult::NotFound;
        };

        if entry.enabled {
            NativeStatusMenuCommandResult::Dispatched(entry.command.clone())
        } else {
            NativeStatusMenuCommandResult::Disabled
        }
    }
}

fn collect_status_menu_commands(
    menu: &MenuSpec,
    inherited_enabled: bool,
    next_id: &mut u32,
    entries: &mut Vec<WindowsWin32StatusMenuCommandEntry>,
) {
    for item in &menu.items {
        match item {
            MenuItemSpec::Command {
                id,
                label,
                command,
                enabled,
                ..
            } => {
                entries.push(WindowsWin32StatusMenuCommandEntry {
                    native_id: *next_id,
                    item_id: id.clone(),
                    label: label.clone(),
                    command: command.clone(),
                    enabled: inherited_enabled && *enabled,
                });
                *next_id = (*next_id).saturating_add(1);
            }
            MenuItemSpec::Submenu { enabled, menu, .. } => {
                collect_status_menu_commands(menu, inherited_enabled && *enabled, next_id, entries);
            }
            MenuItemSpec::Separator => {}
        }
    }
}

pub struct WindowsWin32OwnedPopupMenu {
    menu: HMENU,
    command_table: WindowsWin32StatusMenuCommandTable,
    active: bool,
}

impl WindowsWin32OwnedPopupMenu {
    pub fn from_menu(menu: &MenuSpec) -> ZsuiResult<Self> {
        let command_table = WindowsWin32StatusMenuCommandTable::from_menu(menu);
        let handle = create_status_popup_menu(menu, &command_table)?;
        Ok(Self {
            menu: handle,
            command_table,
            active: true,
        })
    }

    pub const fn handle(&self) -> HMENU {
        self.menu
    }

    pub fn command_entry_count(&self) -> usize {
        self.command_table.entry_count()
    }

    pub fn dispatch_native_menu_command(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        self.command_table.resolve_native_command_id(native_id)
    }

    pub fn present_at(
        &self,
        owner: HWND,
        x: i32,
        y: i32,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "status_item.owner",
                "Win32 popup menu owner window cannot be null",
            ));
        }
        unsafe {
            SetForegroundWindow(owner);
            let selected = TrackPopupMenu(
                self.menu,
                ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS,
                x,
                y,
                0,
                owner,
                null(),
            );
            if selected == 0 {
                Ok(NativeStatusMenuCommandResult::NotFound)
            } else {
                Ok(self.dispatch_native_menu_command(selected as u32))
            }
        }
    }

    pub fn destroy(mut self) -> bool {
        let destroyed = self.destroy_active();
        self.active = false;
        destroyed
    }

    fn destroy_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        let destroyed = unsafe { DestroyMenu(self.menu) != 0 };
        if destroyed {
            self.active = false;
        }
        destroyed
    }
}

impl Drop for WindowsWin32OwnedPopupMenu {
    fn drop(&mut self) {
        let _ = self.destroy_active();
    }
}

fn create_status_popup_menu(
    menu: &MenuSpec,
    command_table: &WindowsWin32StatusMenuCommandTable,
) -> ZsuiResult<HMENU> {
    let handle = create_empty_status_popup_menu()?;

    let mut command_index = 0;
    if let Err(err) =
        append_status_popup_menu_items(menu, handle, command_table, &mut command_index)
    {
        unsafe {
            DestroyMenu(handle);
        }
        return Err(err);
    }
    Ok(handle)
}

fn create_empty_status_popup_menu() -> ZsuiResult<HMENU> {
    let handle = unsafe { CreatePopupMenu() };
    if handle.is_null() {
        Err(ZsuiError::host(
            "windows_win32_create_status_popup_menu",
            "CreatePopupMenu failed",
        ))
    } else {
        Ok(handle)
    }
}

fn append_status_popup_menu_items(
    menu: &MenuSpec,
    handle: HMENU,
    command_table: &WindowsWin32StatusMenuCommandTable,
    command_index: &mut usize,
) -> ZsuiResult<()> {
    for item in &menu.items {
        match item {
            MenuItemSpec::Command { label, checked, .. } => {
                let entry = command_table.entries().get(*command_index).ok_or_else(|| {
                    ZsuiError::host(
                        "windows_win32_status_popup_menu",
                        "status menu command table is missing a command entry",
                    )
                })?;
                *command_index += 1;
                let label = wide_null(label);
                let mut flags = MF_STRING;
                if !entry.enabled {
                    flags |= MF_GRAYED;
                }
                if *checked {
                    flags |= MF_CHECKED;
                }
                let appended =
                    unsafe { AppendMenuW(handle, flags, entry.native_id as usize, label.as_ptr()) };
                if appended == 0 {
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_item",
                        "AppendMenuW command item failed",
                    ));
                }
            }
            MenuItemSpec::Separator => {
                let appended = unsafe { AppendMenuW(handle, MF_SEPARATOR, 0, null()) };
                if appended == 0 {
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_separator",
                        "AppendMenuW separator failed",
                    ));
                }
            }
            MenuItemSpec::Submenu {
                label,
                enabled,
                menu,
                ..
            } => {
                let submenu = create_empty_status_popup_menu()?;
                if let Err(err) =
                    append_status_popup_menu_items(menu, submenu, command_table, command_index)
                {
                    unsafe {
                        DestroyMenu(submenu);
                    }
                    return Err(err);
                }
                let label = wide_null(label);
                let mut flags = MF_POPUP | MF_STRING;
                if !enabled {
                    flags |= MF_GRAYED;
                }
                let appended =
                    unsafe { AppendMenuW(handle, flags, submenu as usize, label.as_ptr()) };
                if appended == 0 {
                    unsafe {
                        DestroyMenu(submenu);
                    }
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_submenu",
                        "AppendMenuW submenu failed",
                    ));
                }
            }
        }
    }
    Ok(())
}

pub struct WindowsWin32OwnedTrayIcon {
    data: NOTIFYICONDATAW,
    icon: Option<WindowsWin32OwnedIcon>,
    menu: MenuSpec,
    menu_command_table: WindowsWin32StatusMenuCommandTable,
    active: bool,
}

impl WindowsWin32OwnedTrayIcon {
    pub fn create(
        owner: HWND,
        id: u32,
        request: NativeStatusItemRequest,
        callback_message: u32,
    ) -> ZsuiResult<Self> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "status_item.owner",
                "Win32 tray icon owner window cannot be null",
            ));
        }
        let icon = request
            .icon_path
            .as_deref()
            .map(|path| {
                WindowsWin32OwnedIcon::from_icon_path(
                    path,
                    system_metric(SM_CXSMICON),
                    system_metric(SM_CYSMICON),
                )
            })
            .transpose()?;
        let mut data = tray_notify_data(
            owner,
            id,
            request.tooltip.as_deref(),
            icon.as_ref().map(WindowsWin32OwnedIcon::handle),
            callback_message,
        );
        let created = unsafe { Shell_NotifyIconW(NIM_ADD, &mut data) != 0 };
        if !created {
            return Err(ZsuiError::host(
                "windows_win32_create_tray_icon",
                "Shell_NotifyIconW(NIM_ADD) failed",
            ));
        }
        let menu_command_table = WindowsWin32StatusMenuCommandTable::from_menu(&request.menu);
        Ok(Self {
            data,
            icon,
            menu: request.menu,
            menu_command_table,
            active: true,
        })
    }

    pub const fn id(&self) -> u32 {
        self.data.uID
    }

    pub const fn owner(&self) -> HWND {
        self.data.hWnd
    }

    pub const fn callback_message(&self) -> u32 {
        self.data.uCallbackMessage
    }

    pub fn menu(&self) -> &MenuSpec {
        &self.menu
    }

    pub fn menu_command_table(&self) -> &WindowsWin32StatusMenuCommandTable {
        &self.menu_command_table
    }

    pub fn has_icon(&self) -> bool {
        self.icon.is_some()
    }

    pub fn set_tooltip(&mut self, tooltip: Option<&str>) -> bool {
        clear_wide_buffer(&mut self.data.szTip);
        self.data.uFlags |= NIF_TIP;
        if let Some(tooltip) = tooltip {
            copy_wide_truncated(tooltip, &mut self.data.szTip);
        }
        unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut self.data) != 0 }
    }

    pub fn set_menu(&mut self, menu: MenuSpec) {
        self.menu_command_table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);
        self.menu = menu;
    }

    pub fn dispatch_native_menu_command(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        self.menu_command_table.resolve_native_command_id(native_id)
    }

    pub fn create_popup_menu(&self) -> ZsuiResult<WindowsWin32OwnedPopupMenu> {
        WindowsWin32OwnedPopupMenu::from_menu(&self.menu)
    }

    pub fn delete(mut self) -> bool {
        let deleted = self.delete_active();
        self.active = false;
        deleted
    }

    fn delete_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        let deleted = unsafe { Shell_NotifyIconW(NIM_DELETE, &mut self.data) != 0 };
        if deleted {
            self.active = false;
        }
        deleted
    }
}

impl Drop for WindowsWin32OwnedTrayIcon {
    fn drop(&mut self) {
        let _ = self.delete_active();
    }
}

pub struct WindowsWin32StatusItemHost {
    owner: HWND,
    callback_message: u32,
    next_id: u32,
    items: Vec<WindowsWin32OwnedTrayIcon>,
    operation_log: Vec<NativeStatusItemHostOperation>,
    status_menu_operation_log: Vec<NativeStatusMenuCommandHostOperation>,
    last_error: Option<String>,
}

impl WindowsWin32StatusItemHost {
    pub fn new(owner: HWND) -> Self {
        Self::with_callback_message(owner, ZSUI_WIN32_TRAY_CALLBACK_MESSAGE)
    }

    pub fn with_callback_message(owner: HWND, callback_message: u32) -> Self {
        Self {
            owner,
            callback_message,
            next_id: 1,
            items: Vec::new(),
            operation_log: Vec::new(),
            status_menu_operation_log: Vec::new(),
            last_error: None,
        }
    }

    pub const fn owner(&self) -> HWND {
        self.owner
    }

    pub fn operation_log(&self) -> &[NativeStatusItemHostOperation] {
        &self.operation_log
    }

    pub fn status_menu_operation_log(&self) -> &[NativeStatusMenuCommandHostOperation] {
        &self.status_menu_operation_log
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn first_native_menu_command_id(&self, status_item_index: usize) -> Option<u32> {
        self.items
            .get(status_item_index)
            .and_then(|item| item.menu_command_table().first_native_id())
    }

    pub fn native_menu_command_count(&self, status_item_index: usize) -> usize {
        self.items
            .get(status_item_index)
            .map(|item| item.menu_command_table().entry_count())
            .unwrap_or(0)
    }

    pub fn dispatch_native_menu_command(
        &mut self,
        status_item_index: usize,
        native_command_id: u32,
    ) -> NativeStatusMenuCommandResult {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        self.items
            .get(status_item_index)
            .map(|item| item.dispatch_native_menu_command(native_command_id))
            .unwrap_or(NativeStatusMenuCommandResult::NotFound)
    }

    pub fn create_popup_menu_for_status_item(
        &self,
        status_item_index: usize,
    ) -> ZsuiResult<WindowsWin32OwnedPopupMenu> {
        self.items
            .get(status_item_index)
            .ok_or_else(|| {
                ZsuiError::invalid_spec(
                    "status_item_index",
                    "Win32 status item index does not exist",
                )
            })?
            .create_popup_menu()
    }

    pub fn present_status_item_menu_at(
        &mut self,
        status_item_index: usize,
        x: i32,
        y: i32,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        let popup = self.create_popup_menu_for_status_item(status_item_index)?;
        popup.present_at(self.owner, x, y)
    }

    pub fn present_status_item_menu_at_cursor(
        &mut self,
        status_item_index: usize,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        let mut point = POINT { x: 0, y: 0 };
        let ok = unsafe { GetCursorPos(&mut point) != 0 };
        if !ok {
            return Err(ZsuiError::host(
                "windows_win32_status_popup_menu_position",
                "GetCursorPos failed",
            ));
        }
        self.present_status_item_menu_at(status_item_index, point.x, point.y)
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    fn record(&mut self, operation: NativeStatusItemHostOperation) {
        self.operation_log.push(operation);
    }
}

impl NativeStatusItemHost for WindowsWin32StatusItemHost {
    type Handle = u32;

    fn create_status_item(
        &mut self,
        request: NativeStatusItemRequest,
    ) -> NativeStatusItemPresentation<Self::Handle> {
        self.record(NativeStatusItemHostOperation::CreateStatusItem);
        self.last_error = None;
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);
        match WindowsWin32OwnedTrayIcon::create(self.owner, id, request, self.callback_message) {
            Ok(item) => {
                self.items.push(item);
                NativeStatusItemPresentation::Created(id)
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
                NativeStatusItemPresentation::Failed
            }
        }
    }

    fn set_status_item_tooltip(&mut self, handle: Self::Handle, tooltip: Option<String>) {
        self.record(NativeStatusItemHostOperation::SetStatusItemTooltip);
        if let Some(item) = self.items.iter_mut().find(|item| item.id() == handle) {
            let _ = item.set_tooltip(tooltip.as_deref());
        }
    }

    fn set_status_item_menu(&mut self, handle: Self::Handle, menu: MenuSpec) {
        self.record(NativeStatusItemHostOperation::SetStatusItemMenu);
        if let Some(item) = self.items.iter_mut().find(|item| item.id() == handle) {
            item.set_menu(menu);
        }
    }

    fn destroy_status_item(&mut self, handle: Self::Handle) {
        self.record(NativeStatusItemHostOperation::DestroyStatusItem);
        if let Some(index) = self.items.iter().position(|item| item.id() == handle) {
            let item = self.items.remove(index);
            let _ = item.delete();
        }
    }
}

impl NativeStatusMenuCommandHost for WindowsWin32StatusItemHost {
    fn dispatch_status_menu_command(
        &mut self,
        request: NativeStatusMenuCommandRequest,
    ) -> NativeStatusMenuCommandResult {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        let Some(item) = self.items.get(request.status_item_index) else {
            return NativeStatusMenuCommandResult::NotFound;
        };
        native_status_menu_command_from_menu(item.menu(), &request)
    }
}

fn tray_notify_data(
    owner: HWND,
    id: u32,
    tooltip: Option<&str>,
    icon: Option<HICON>,
    callback_message: u32,
) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { zeroed() };
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = owner;
    data.uID = id;
    data.uFlags = NIF_MESSAGE;
    data.uCallbackMessage = callback_message;
    if let Some(tooltip) = tooltip {
        data.uFlags |= NIF_TIP;
        copy_wide_truncated(tooltip, &mut data.szTip);
    }
    if let Some(icon) = icon {
        data.uFlags |= NIF_ICON;
        data.hIcon = icon;
    }
    data
}

fn copy_wide_truncated(value: &str, target: &mut [u16]) {
    if target.is_empty() {
        return;
    }
    clear_wide_buffer(target);
    let max_chars = target.len().saturating_sub(1);
    for (slot, value) in target.iter_mut().take(max_chars).zip(value.encode_utf16()) {
        *slot = value;
    }
}

fn clear_wide_buffer(target: &mut [u16]) {
    for value in target {
        *value = 0;
    }
}

pub fn set_windows_win32_window_draw_plan(hwnd: HWND, plan: NativeDrawPlan) -> bool {
    if hwnd.is_null() {
        return false;
    }
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    let hwnd = hwnd as isize;
    if let Some(record) = plans.iter_mut().find(|record| record.hwnd == hwnd) {
        record.plan = plan;
    } else {
        plans.push(WindowsWindowDrawPlanRecord { hwnd, plan });
    }
    true
}

pub fn clear_windows_win32_window_draw_plan(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_draw_plans() {
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans.clear();
}

fn window_draw_plans() -> &'static Mutex<Vec<WindowsWindowDrawPlanRecord>> {
    WINDOW_DRAW_PLANS.get_or_init(|| Mutex::new(Vec::new()))
}

fn window_draw_plan(hwnd: HWND) -> Option<NativeDrawPlan> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.plan.clone())
}

#[derive(Debug, Clone)]
pub struct WindowsWin32ViewInputRoute {
    interaction_plan: ViewInteractionPlan,
    ui_command_view: ViewNode<UiCommand>,
    focused_widget: Option<crate::WidgetId>,
}

impl WindowsWin32ViewInputRoute {
    pub fn new(
        interaction_plan: ViewInteractionPlan,
        ui_command_view: ViewNode<UiCommand>,
    ) -> Self {
        Self {
            interaction_plan,
            ui_command_view,
            focused_widget: None,
        }
    }

    pub fn hit_target_count(&self) -> usize {
        self.interaction_plan.hit_target_count()
    }

    fn dispatch_click(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            click_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(target) = self.interaction_plan.hit_target_at(point) else {
            report.unhandled_click_count = 1;
            report
                .events
                .push(format!("win32_view_click_missed:{}:{}", point.x, point.y));
            return report;
        };

        if target.kind == crate::ViewHitTargetKind::Textbox {
            self.focused_widget = Some(target.widget);
            report.focus_count = 1;
            report.focused_widget = Some(target.widget.0);
            report
                .events
                .push(format!("win32_view_focus:{}", target.widget.0));
            return report;
        }

        let event = if target.kind == crate::ViewHitTargetKind::Checkbox {
            let checked = !self
                .ui_command_view
                .widget_checked_value(target.widget)
                .unwrap_or(false);
            report.toggle_count = 1;
            report
                .events
                .push(format!("win32_view_toggle:{}:{checked}", target.widget.0));
            crate::ViewEvent::Toggled {
                widget: target.widget,
                checked,
            }
        } else {
            crate::ViewEvent::Click {
                widget: target.widget,
            }
        };
        report.event_count = 1;
        if let crate::ViewEvent::Click { widget } = &event {
            report.events.push(format!("win32_view_click:{}", widget.0));
        }

        self.dispatch_event(event, &mut report);
        report
    }

    fn dispatch_text_input(&mut self, text: &str) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(widget) = self.focused_widget else {
            report
                .events
                .push("win32_view_text_without_focus".to_string());
            return report;
        };

        let mut value = self
            .ui_command_view
            .widget_text_value(widget)
            .unwrap_or_default()
            .to_string();
        for ch in text.chars() {
            match ch {
                '\u{8}' => {
                    value.pop();
                }
                ch if !ch.is_control() => value.push(ch),
                _ => {}
            }
        }

        report.text_input_count = text.chars().count();
        report.event_count = 1;
        report
            .events
            .push(format!("win32_view_text_changed:{}", widget.0));
        self.dispatch_event(crate::ViewEvent::TextChanged { widget, value }, &mut report);
        report
    }

    fn dispatch_event(
        &mut self,
        event: crate::ViewEvent,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let mut event_cx = ViewEventCx::new();
        self.ui_command_view.event(&mut event_cx, &event);
        let commands = event_cx.into_messages();
        report.message_count = commands.len();
        report.ui_command_count = commands.len();
        for command in commands {
            report.ui_command_ids.push(command.id.0);
            report
                .events
                .push(format!("win32_view_ui_command:{}", command.id.0));
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsWin32ViewInputDispatchReport {
    pub hit_target_count: usize,
    pub click_count: usize,
    pub event_count: usize,
    pub message_count: usize,
    pub ui_command_count: usize,
    pub ui_command_ids: Vec<&'static str>,
    pub unhandled_click_count: usize,
    pub focus_count: usize,
    pub focused_widget: Option<u64>,
    pub text_input_count: usize,
    pub toggle_count: usize,
    pub events: Vec<String>,
}

impl WindowsWin32ViewInputDispatchReport {
    fn merge(&mut self, next: WindowsWin32ViewInputDispatchReport) {
        self.hit_target_count = next.hit_target_count;
        self.click_count += next.click_count;
        self.event_count += next.event_count;
        self.message_count += next.message_count;
        self.ui_command_count += next.ui_command_count;
        self.ui_command_ids.extend(next.ui_command_ids);
        self.unhandled_click_count += next.unhandled_click_count;
        self.focus_count += next.focus_count;
        self.focused_widget = next.focused_widget.or(self.focused_widget);
        self.text_input_count += next.text_input_count;
        self.toggle_count += next.toggle_count;
        self.events.extend(next.events);
    }
}

pub fn set_windows_win32_window_view_input_route(
    hwnd: HWND,
    route: WindowsWin32ViewInputRoute,
) -> bool {
    if hwnd.is_null() {
        return false;
    }
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    let hwnd = hwnd as isize;
    if let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd) {
        record.route = route;
        record.report = WindowsWin32ViewInputDispatchReport::default();
    } else {
        let report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: route.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        routes.push(WindowsWindowViewInputRouteRecord {
            hwnd,
            route,
            report,
        });
    }
    true
}

pub fn clear_windows_win32_window_view_input_route(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    routes.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_view_input_routes() {
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    routes.clear();
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
    routes
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.report.clone())
}

pub fn dispatch_windows_win32_window_view_click(
    hwnd: HWND,
    point: crate::Point,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    let record = routes.iter_mut().find(|record| record.hwnd == hwnd)?;
    let report = record.route.dispatch_click(point);
    record.report.merge(report.clone());
    Some(report)
}

pub fn dispatch_windows_win32_window_view_text_input(
    hwnd: HWND,
    text: &str,
) -> Option<WindowsWin32ViewInputDispatchReport> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned");
    let record = routes.iter_mut().find(|record| record.hwnd == hwnd)?;
    let report = record.route.dispatch_text_input(text);
    record.report.merge(report.clone());
    Some(report)
}

fn window_view_input_routes() -> &'static Mutex<Vec<WindowsWindowViewInputRouteRecord>> {
    WINDOW_VIEW_INPUT_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn run_windows_win32_native_window_event_loop(specs: &[WindowSpec]) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_status_items(specs, &[])
}

pub fn run_windows_win32_native_window_event_loop_with_status_items(
    specs: &[WindowSpec],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items(
        specs,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    if specs.is_empty() {
        return Ok(());
    }
    let _handles = create_owned_windows_for_specs_with_draw_plans(specs, draw_plans)?;
    let mut _status_item_host = None;
    if !status_items.is_empty() {
        let mut host = WindowsWin32StatusItemHost::new(_handles[0].main());
        for status_item in status_items {
            match host.create_status_item(NativeStatusItemRequest::from_tray_spec(status_item)) {
                NativeStatusItemPresentation::Created(_) => {}
                NativeStatusItemPresentation::Failed => {
                    return Err(ZsuiError::host(
                        "windows_win32_status_item",
                        host.last_error()
                            .unwrap_or("Win32 status item creation failed")
                            .to_string(),
                    ));
                }
            }
        }
        _status_item_host = Some(host);
    }
    match WindowsWin32MessageLoop::run() {
        WindowsWin32MessageLoopResult::Quit(_) => Ok(()),
        WindowsWin32MessageLoopResult::Failed => Err(ZsuiError::host(
            "windows_win32_message_loop",
            "GetMessageW failed",
        )),
    }
}

pub struct WindowsWin32MainWindowHost {
    class_names: WindowsWin32ClassNames,
    window_proc: WNDPROC,
    operation_log: Vec<NativeMainWindowHostOperation>,
}

impl WindowsWin32MainWindowHost {
    pub fn new() -> Self {
        Self::with_window_proc(Some(zsui_win32_default_window_proc))
    }

    pub fn with_window_proc(window_proc: WNDPROC) -> Self {
        Self::with_class_names(WindowsWin32ClassNames::default(), window_proc)
    }

    pub fn with_class_names(class_names: WindowsWin32ClassNames, window_proc: WNDPROC) -> Self {
        Self {
            class_names,
            window_proc,
            operation_log: Vec::new(),
        }
    }

    pub const fn class_names(&self) -> WindowsWin32ClassNames {
        self.class_names
    }

    pub fn operation_log(&self) -> &[NativeMainWindowHostOperation] {
        &self.operation_log
    }

    fn record(&mut self, operation: NativeMainWindowHostOperation) {
        self.operation_log.push(operation);
    }

    unsafe fn module_handle() -> HINSTANCE {
        GetModuleHandleW(null()) as HINSTANCE
    }

    unsafe fn arrow_cursor() -> HCURSOR {
        LoadCursorW(null_mut(), IDC_ARROW)
    }

    unsafe fn register_window_class(
        &self,
        role: WindowsWindowRole,
        module: HINSTANCE,
        cursor: HCURSOR,
    ) -> bool {
        if self.window_proc.is_none() {
            return false;
        }
        let class_name = wide_null(role.class_name(self.class_names));
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: self.window_proc,
            hInstance: module,
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS
    }

    unsafe fn create_window(
        &self,
        role: WindowsWindowRole,
        title: &[u16],
        width: i32,
        height: i32,
        module: HINSTANCE,
        options: &NativeWindowOptions,
    ) -> HWND {
        let style_plan = windows_win32_main_window_style_plan(role, options);
        let class_name = wide_null(role.class_name(self.class_names));
        let create_params = WindowsWindowCreateParams::new(role, options.min_size);
        CreateWindowExW(
            style_plan.ex_style,
            class_name.as_ptr(),
            title.as_ptr(),
            style_plan.style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            null_mut(),
            null_mut(),
            module,
            &create_params as *const WindowsWindowCreateParams as _,
        )
    }
}

impl Default for WindowsWin32MainWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeMainWindowHost for WindowsWin32MainWindowHost {
    type Handle = HWND;
    type AppIcon = isize;

    fn create_main_windows(
        &mut self,
        request: NativeMainWindowRequest,
    ) -> NativeMainWindowPresentation<Self::Handle> {
        self.record(NativeMainWindowHostOperation::CreateMainWindows);
        unsafe {
            let module = Self::module_handle();
            if module.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            let cursor = Self::arrow_cursor();
            if cursor.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            for role in [WindowsWindowRole::Main, WindowsWindowRole::Quick] {
                if !self.register_window_class(role, module, cursor) {
                    return NativeMainWindowPresentation::Failed;
                }
            }

            let title = wide_null(&request.title);
            let width = request.size.width.max(1);
            let height = request.size.height.max(1);
            let main = self.create_window(
                WindowsWindowRole::Main,
                &title,
                width,
                height,
                module,
                &request.options,
            );
            if main.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            ACTIVE_MAIN_WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

            let quick_options = NativeWindowOptions::tool_window();
            let quick = self.create_window(
                WindowsWindowRole::Quick,
                &title,
                width,
                height,
                module,
                &quick_options,
            );
            if quick.is_null() {
                DestroyWindow(main);
                return NativeMainWindowPresentation::Failed;
            }

            ShowWindow(
                main,
                if request.main_visible {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
            if request.main_visible {
                UpdateWindow(main);
            }
            ShowWindow(quick, SW_HIDE);
            NativeMainWindowPresentation::Created(NativeMainWindowHandles { main, quick })
        }
    }

    fn apply_main_window_appearance(&mut self, _handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ApplyMainWindowAppearance);
    }

    fn set_main_window_app_icon(
        &mut self,
        handle: Self::Handle,
        icon: NativeAppIconResource<Self::AppIcon>,
    ) {
        self.record(NativeMainWindowHostOperation::SetMainWindowAppIcon);
        unsafe {
            SendMessageW(handle, WM_SETICON, ICON_SMALL as WPARAM, icon.small);
            SendMessageW(handle, WM_SETICON, ICON_BIG as WPARAM, icon.big);
        }
    }

    fn hide_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::HideMainWindow);
        unsafe {
            ShowWindow(handle, SW_HIDE);
        }
    }

    fn present_main_window(&mut self, handle: Self::Handle, mode: NativeMainWindowPresentMode) {
        self.record(NativeMainWindowHostOperation::PresentMainWindow);
        unsafe {
            match mode {
                NativeMainWindowPresentMode::ActivateAndFocus => {
                    ShowWindow(handle, SW_SHOW);
                    SetWindowPos(
                        handle,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                    );
                    SetForegroundWindow(handle);
                    SetFocus(handle);
                }
                NativeMainWindowPresentMode::NoActivate => {
                    ShowWindow(handle, SW_SHOWNOACTIVATE);
                    SetWindowPos(
                        handle,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                }
            }
        }
    }

    fn set_main_window_bounds(&mut self, handle: Self::Handle, bounds: UiRect) {
        self.record(NativeMainWindowHostOperation::SetMainWindowBounds);
        unsafe {
            SetWindowPos(
                handle,
                null_mut(),
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn activate_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ActivateMainWindow);
        unsafe {
            ShowWindow(handle, SW_SHOW);
            SetWindowPos(
                handle,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            SetForegroundWindow(handle);
            SetFocus(handle);
        }
    }

    fn foreground_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ForegroundMainWindow);
        unsafe {
            SetForegroundWindow(handle);
        }
    }

    fn restore_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::RestoreMainWindow);
        unsafe {
            ShowWindow(handle, SW_SHOW);
        }
    }

    fn close_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::CloseMainWindow);
        unsafe {
            PostMessageW(handle, WM_CLOSE, 0, 0);
        }
    }

    fn set_main_window_activation_policy(&mut self, handle: Self::Handle, allow_activation: bool) {
        self.record(NativeMainWindowHostOperation::SetMainWindowActivationPolicy);
        if handle.is_null() {
            return;
        }
        unsafe {
            let ex_style = GetWindowLongW(handle, GWL_EXSTYLE) as u32;
            let desired = if allow_activation {
                ex_style & !WS_EX_NOACTIVATE
            } else {
                ex_style | WS_EX_NOACTIVATE
            };
            if desired != ex_style {
                SetWindowLongW(handle, GWL_EXSTYLE, desired as i32);
                SetWindowPos(
                    handle,
                    null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
    }

    fn request_main_window_close(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::RequestMainWindowClose);
        unsafe {
            SendMessageW(handle, WM_CLOSE, 0, 0);
        }
    }

    fn destroy_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::DestroyMainWindow);
        unsafe {
            DestroyWindow(handle);
        }
    }

    fn capture_main_pointer(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::CaptureMainPointer);
        unsafe {
            SetCapture(handle);
        }
    }

    fn release_main_pointer(&mut self, _handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ReleaseMainPointer);
        unsafe {
            ReleaseCapture();
        }
    }

    fn begin_main_window_drag(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::BeginMainWindowDrag);
        unsafe {
            ReleaseCapture();
            SendMessageW(
                handle,
                WM_SYSCOMMAND,
                (SC_MOVE as usize | HTCAPTION as usize) as WPARAM,
                0,
            );
        }
    }

    fn track_main_pointer_leave(&mut self, handle: Self::Handle) -> bool {
        self.record(NativeMainWindowHostOperation::TrackMainPointerLeave);
        if handle.is_null() {
            return false;
        }
        let mut event = TRACKMOUSEEVENT {
            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE | TME_HOVER,
            hwndTrack: handle,
            dwHoverTime: HOVER_DEFAULT,
        };
        unsafe { TrackMouseEvent(&mut event) != 0 }
    }

    fn request_main_window_area_repaint(
        &mut self,
        handle: Self::Handle,
        area: Option<UiRect>,
        erase: bool,
    ) -> bool {
        self.record(NativeMainWindowHostOperation::RequestMainWindowAreaRepaint);
        let rect = area.map(RECT::from);
        unsafe {
            InvalidateRect(
                handle,
                rect.as_ref().map_or(null(), |rect| rect as *const RECT),
                erase as i32,
            ) != 0
        }
    }

    fn main_window_layout_dpi(&mut self, handle: Self::Handle) -> u32 {
        self.record(NativeMainWindowHostOperation::MainWindowLayoutDpi);
        if handle.is_null() {
            96
        } else {
            unsafe { GetDpiForWindow(handle).max(1) }
        }
    }

    fn main_window_client_bounds(&mut self, handle: Self::Handle) -> Option<UiRect> {
        self.record(NativeMainWindowHostOperation::MainWindowClientBounds);
        if handle.is_null() {
            return None;
        }
        let mut rect: RECT = unsafe { zeroed() };
        let ok = unsafe { GetClientRect(handle, &mut rect) != 0 };
        ok.then(|| UiRect::from(rect))
    }

    fn main_window_bounds(&mut self, handle: Self::Handle) -> Option<UiRect> {
        self.record(NativeMainWindowHostOperation::MainWindowBounds);
        if handle.is_null() {
            return None;
        }
        let mut rect: RECT = unsafe { zeroed() };
        let ok = unsafe { GetWindowRect(handle, &mut rect) != 0 };
        ok.then(|| UiRect::from(rect))
    }
}

pub struct WindowsWin32TransientWindowHost {
    class_name: &'static str,
    window_proc: WNDPROC,
    operation_log: Vec<NativeTransientWindowHostOperation>,
}

impl WindowsWin32TransientWindowHost {
    pub fn new() -> Self {
        Self::with_window_proc(
            DEFAULT_TRANSIENT_CLASS_NAME,
            Some(zsui_win32_default_window_proc),
        )
    }

    pub fn with_window_proc(class_name: &'static str, window_proc: WNDPROC) -> Self {
        Self {
            class_name,
            window_proc,
            operation_log: Vec::new(),
        }
    }

    pub const fn class_name(&self) -> &'static str {
        self.class_name
    }

    pub fn operation_log(&self) -> &[NativeTransientWindowHostOperation] {
        &self.operation_log
    }

    fn record(&mut self, operation: NativeTransientWindowHostOperation) {
        self.operation_log.push(operation);
    }

    unsafe fn register_transient_class(&self, module: HINSTANCE, cursor: HCURSOR) -> bool {
        if self.class_name.is_empty() || self.window_proc.is_none() {
            return false;
        }
        let class_name = wide_null(self.class_name);
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: self.window_proc,
            hInstance: module,
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS
    }
}

impl Default for WindowsWin32TransientWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeTransientWindowHost for WindowsWin32TransientWindowHost {
    type Handle = HWND;
    type Owner = HWND;

    fn create_transient_window(
        &mut self,
        request: NativeTransientWindowRequest<Self::Owner>,
    ) -> NativeTransientWindowPresentation<Self::Handle> {
        self.record(NativeTransientWindowHostOperation::CreateTransientWindow);
        unsafe {
            let module = WindowsWin32MainWindowHost::module_handle();
            let cursor = WindowsWin32MainWindowHost::arrow_cursor();
            if module.is_null()
                || cursor.is_null()
                || !self.register_transient_class(module, cursor)
            {
                return NativeTransientWindowPresentation::Failed;
            }

            let class_name = wide_null(self.class_name);
            let empty_title = wide_null("");
            let handle = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                empty_title.as_ptr(),
                WS_POPUP | WS_THICKFRAME,
                request.bounds.left,
                request.bounds.top,
                request.bounds.right - request.bounds.left,
                request.bounds.bottom - request.bounds.top,
                null_mut(),
                null_mut(),
                module,
                request.owner as _,
            );
            if handle.is_null() {
                NativeTransientWindowPresentation::Failed
            } else {
                NativeTransientWindowPresentation::Created(handle)
            }
        }
    }

    fn present_transient_window(&mut self, handle: Self::Handle, bounds: UiRect) {
        self.record(NativeTransientWindowHostOperation::PresentTransientWindow);
        unsafe {
            SetWindowPos(
                handle,
                HWND_TOPMOST,
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    fn hide_transient_window(&mut self, handle: Self::Handle) {
        self.record(NativeTransientWindowHostOperation::HideTransientWindow);
        unsafe {
            ShowWindow(handle, SW_HIDE);
        }
    }

    fn destroy_transient_window(&mut self, handle: Self::Handle) {
        self.record(NativeTransientWindowHostOperation::DestroyTransientWindow);
        unsafe {
            DestroyWindow(handle);
        }
    }
}

pub unsafe extern "system" fn zsui_win32_default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let create_params =
                WindowsWindowCreateParams::from_create_struct(lparam as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_params.role as isize);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_NCDESTROY => {
            let role = WindowsWindowRole::from_create_param(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
            clear_windows_win32_window_draw_plan(hwnd);
            if matches!(role, WindowsWindowRole::Main)
                && ACTIVE_MAIN_WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst) <= 1
            {
                PostQuitMessage(0);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_ERASEBKGND => 1,
        WM_LBUTTONUP => {
            if dispatch_windows_win32_window_view_click(hwnd, point_from_lparam(lparam)).is_some() {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_CHAR => {
            if let Some(text) = text_from_char_wparam(wparam) {
                if dispatch_windows_win32_window_view_text_input(hwnd, &text).is_some() {
                    0
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_PAINT => paint_no_flicker_background(hwnd),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn point_from_lparam(lparam: LPARAM) -> crate::Point {
    let raw = lparam as u32;
    crate::Point {
        x: (raw & 0xffff) as i16 as i32,
        y: ((raw >> 16) & 0xffff) as i16 as i32,
    }
}

fn text_from_char_wparam(wparam: WPARAM) -> Option<String> {
    match char::from_u32(wparam as u32) {
        Some('\u{8}') => Some('\u{8}'.to_string()),
        Some(ch) if !ch.is_control() => Some(ch.to_string()),
        _ => None,
    }
}

unsafe fn paint_no_flicker_background(hwnd: HWND) -> LRESULT {
    let mut ps: PAINTSTRUCT = zeroed();
    let target = BeginPaint(hwnd, &mut ps);
    if target.is_null() {
        return 0;
    }

    let mut rect: RECT = zeroed();
    if GetClientRect(hwnd, &mut rect) != 0 {
        let palette = WindowsGdiPalette::default();
        let draw_plan = window_draw_plan(hwnd);
        if let Some(buffered) = WindowsBufferedPaint::begin(target, &rect) {
            paint_win32_surface(buffered.hdc(), rect, palette, draw_plan.as_ref());
        } else {
            paint_win32_surface(target, rect, palette, draw_plan.as_ref());
        }
    }

    EndPaint(hwnd, &ps);
    0
}

unsafe fn paint_win32_surface(
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
    rect: RECT,
    palette: WindowsGdiPalette,
    draw_plan: Option<&NativeDrawPlan>,
) {
    let mut renderer = WindowsGdiRenderer::new(dc);
    renderer.fill_rect(rect_from_win(rect), palette.surface);
    drop(renderer);
    if let Some(plan) = draw_plan {
        let mut sink = WindowsGdiDrawSink::with_palette(dc, palette);
        sink.draw_native_plan(plan);
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path_null(path: &Path) -> Vec<u16> {
    path.to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsclip_main_window_styles_map_to_win32_flags() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::standard(),
        );

        assert_eq!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_eq!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.style & WS_CAPTION, 0);
        assert_ne!(plan.style & WS_SYSMENU, 0);
        assert_ne!(plan.style & WS_THICKFRAME, 0);
        assert_ne!(plan.style & WS_MAXIMIZEBOX, 0);
    }

    #[test]
    fn zsclip_tool_window_shape_maps_to_popup_topmost_flags() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::tool_window(),
        );

        assert_ne!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_ne!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.style & WS_POPUP, 0);
        assert_eq!(plan.style & WS_CAPTION, 0);
        assert_eq!(plan.style & WS_THICKFRAME, 0);
    }

    #[test]
    fn quick_window_forces_no_activate_topmost_tool_window() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Quick,
            &NativeWindowOptions::standard(),
        );

        assert_ne!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_ne!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.ex_style & WS_EX_NOACTIVATE, 0);
    }

    #[test]
    fn window_create_params_preserve_role_and_min_size_for_win32_create() {
        let params = WindowsWindowCreateParams::new(
            WindowsWindowRole::Main,
            Some(Size {
                width: 640,
                height: 420,
            }),
        );

        let decoded = WindowsWindowCreateParams::from_create_param(&params as *const _ as isize);
        assert_eq!(decoded, params);
        assert_eq!(
            WindowsWindowCreateParams::from_create_param(WindowsWindowRole::Quick as isize),
            WindowsWindowCreateParams::new(WindowsWindowRole::Quick, None)
        );
    }

    #[test]
    fn window_draw_plan_registry_tracks_native_paint_content() {
        let hwnd = 1isize as HWND;
        let plan = NativeDrawPlan::new([crate::NativeDrawCommand::FillRect {
            rect: crate::Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
            fill: crate::NativeDrawFill::Role(crate::ColorRole::Accent),
        }]);

        assert!(set_windows_win32_window_draw_plan(hwnd, plan.clone()));
        assert_eq!(window_draw_plan(hwnd), Some(plan));

        clear_windows_win32_window_draw_plan(hwnd);
        assert_eq!(window_draw_plan(hwnd), None);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_click_to_ui_command() {
        clear_windows_win32_window_view_input_routes();
        let hwnd = 77isize as HWND;
        let widget = crate::WidgetId::new(9);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::new(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 48,
                },
            )]),
            crate::button("Save")
                .id(widget)
                .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.save"))),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let dispatched =
            dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
                .expect("registered route should dispatch click");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(dispatched.hit_target_count, 1);
        assert_eq!(dispatched.click_count, 1);
        assert_eq!(dispatched.event_count, 1);
        assert_eq!(dispatched.ui_command_count, 1);
        assert_eq!(dispatched.ui_command_ids, vec!["zsui.test.win32.save"]);
        assert_eq!(aggregate.click_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
        assert!(windows_win32_window_view_input_report(hwnd).is_none());
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_dispatches_text_input_to_ui_command() {
        fn text_changed(_: String) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.text_changed"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 78isize as HWND;
        let widget = crate::WidgetId::new(10);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("").id(widget).on_change(text_changed),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let focus = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should focus textbox");
        let text = dispatch_windows_win32_window_view_text_input(hwnd, "ZS")
            .expect("focused textbox should dispatch text");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(focus.focus_count, 1);
        assert_eq!(focus.focused_widget, Some(widget.0));
        assert_eq!(text.text_input_count, 2);
        assert_eq!(text.event_count, 1);
        assert_eq!(text.ui_command_count, 1);
        assert_eq!(text.ui_command_ids, vec!["zsui.test.win32.text_changed"]);
        assert_eq!(aggregate.focus_count, 1);
        assert_eq!(aggregate.text_input_count, 2);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "checkbox")]
    fn window_view_input_route_dispatches_checkbox_toggle_to_ui_command() {
        fn toggled(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.toggle_changed"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 79isize as HWND;
        let widget = crate::WidgetId::new(11);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Checkbox,
            )]),
            crate::checkbox("Dark mode", false)
                .id(widget)
                .on_toggle(toggled),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let toggle = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should toggle checkbox");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(toggle.toggle_count, 1);
        assert_eq!(toggle.event_count, 1);
        assert_eq!(toggle.ui_command_count, 1);
        assert_eq!(
            toggle.ui_command_ids,
            vec!["zsui.test.win32.toggle_changed"]
        );
        assert_eq!(aggregate.toggle_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    fn owned_hwnd_wrapper_is_drop_backed_and_can_release_legacy_raw_handles() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedMainWindowHandles>());

        let handles = NativeMainWindowHandles {
            main: 1isize as HWND,
            quick: 2isize as HWND,
        };
        let owned = WindowsWin32OwnedMainWindowHandles::new(handles);

        assert_eq!(owned.handles(), handles);
        assert_eq!(owned.main(), handles.main);
        assert_eq!(owned.quick(), handles.quick);
        assert_eq!(owned.app_icon_count(), 0);
        assert_eq!(owned.into_handles(), handles);
    }

    #[test]
    fn owned_hicon_wrappers_model_raii_without_double_destroying_shared_sizes() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedIcon>());
        assert!(std::mem::needs_drop::<WindowsWin32OwnedAppIconResource>());
        assert!(WindowsWin32OwnedIcon::from_raw(null_mut()).is_none());
        assert!(WindowsWin32OwnedAppIconResource::from_raw(null_mut(), null_mut()).is_none());
        assert!(matches!(
            WindowsWin32OwnedIcon::from_icon_path("", 16, 16),
            Err(ZsuiError::InvalidSpec { field, .. }) if field == "window.icon_path"
        ));

        let icon = WindowsWin32OwnedIcon::from_raw(1isize as HICON)
            .expect("non-null HICON should be accepted");
        assert_eq!(icon.into_raw(), 1isize as HICON);

        let resource = WindowsWin32OwnedAppIconResource::from_raw(2isize as HICON, 2isize as HICON)
            .expect("shared small/big HICON should be accepted");
        assert_eq!(
            resource.as_native_resource(),
            NativeAppIconResource { small: 2, big: 2 }
        );
        assert_eq!(resource.into_raw_pair(), (2isize as HICON, 2isize as HICON));
    }

    #[test]
    fn owned_tray_icon_data_keeps_win32_notify_contract_and_raii_shape() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedTrayIcon>());

        let hwnd = 7isize as HWND;
        let data = tray_notify_data(
            hwnd,
            42,
            Some("ZSUI"),
            Some(9isize as HICON),
            ZSUI_WIN32_TRAY_CALLBACK_MESSAGE,
        );

        assert_eq!(data.hWnd, hwnd);
        assert_eq!(data.uID, 42);
        assert_eq!(data.uCallbackMessage, ZSUI_WIN32_TRAY_CALLBACK_MESSAGE);
        assert_ne!(data.uFlags & NIF_MESSAGE, 0);
        assert_ne!(data.uFlags & NIF_TIP, 0);
        assert_ne!(data.uFlags & NIF_ICON, 0);
        assert_eq!(data.szTip[0], 'Z' as u16);
    }

    #[test]
    fn status_menu_command_table_maps_nested_menu_to_native_ids() {
        let menu = MenuSpec::new()
            .item("Open", Command::ShowMainWindow)
            .submenu(
                "More",
                MenuSpec::new()
                    .item("Refresh", Command::custom("example.refresh"))
                    .separator()
                    .item("Quit", Command::Quit),
            );
        let table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);

        assert_eq!(table.entry_count(), 3);
        assert_eq!(
            table.first_native_id(),
            Some(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID)
        );
        assert_eq!(
            table.resolve_native_command_id(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID + 1),
            NativeStatusMenuCommandResult::Dispatched(Command::custom("example.refresh"))
        );
        assert_eq!(
            table.resolve_native_command_id(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID + 99),
            NativeStatusMenuCommandResult::NotFound
        );
    }

    #[test]
    fn owned_status_popup_menu_creates_native_menu_and_cleans_up() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedPopupMenu>());
        let menu = MenuSpec::new()
            .item("Open", Command::ShowMainWindow)
            .separator()
            .item("Quit", Command::Quit);
        let popup = WindowsWin32OwnedPopupMenu::from_menu(&menu)
            .expect("Win32 popup menu should be created from a status menu spec");

        assert!(!popup.handle().is_null());
        assert_eq!(popup.command_entry_count(), 2);
        assert_eq!(
            popup.dispatch_native_menu_command(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID),
            NativeStatusMenuCommandResult::Dispatched(Command::ShowMainWindow)
        );
        assert!(popup.destroy());
    }

    #[test]
    fn status_popup_menu_selection_uses_return_command_flags() {
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_RETURNCMD, 0);
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_NONOTIFY, 0);
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_RIGHTBUTTON, 0);

        let popup = WindowsWin32OwnedPopupMenu::from_menu(
            &MenuSpec::new().item("Open", Command::ShowMainWindow),
        )
        .expect("Win32 popup menu should be created");
        assert!(matches!(
            popup.present_at(null_mut(), 0, 0),
            Err(ZsuiError::InvalidSpec { field, .. }) if field == "status_item.owner"
        ));
    }

    #[test]
    fn win32_status_item_host_rejects_null_owner_without_leaking_tray_handle() {
        let mut host = WindowsWin32StatusItemHost::new(null_mut());
        let presentation = host.create_status_item(NativeStatusItemRequest::from_tray_spec(
            &crate::TraySpec::new()
                .tooltip("ZSUI")
                .item("Quit", crate::Command::Quit),
        ));

        assert!(matches!(presentation, NativeStatusItemPresentation::Failed));
        assert_eq!(host.item_count(), 0);
        assert!(host
            .last_error()
            .expect("failed status item should retain host error")
            .contains("status_item.owner"));
        assert_eq!(
            host.operation_log(),
            &[NativeStatusItemHostOperation::CreateStatusItem]
        );
    }

    #[test]
    fn win32_host_records_native_main_window_host_operations() {
        let mut host = WindowsWin32MainWindowHost::new();

        host.hide_main_window(null_mut());
        assert_eq!(
            host.operation_log(),
            &[NativeMainWindowHostOperation::HideMainWindow]
        );
    }

    #[test]
    fn transient_host_preserves_zsclip_topmost_noactivate_window_shape() {
        let mut host = WindowsWin32TransientWindowHost::new();

        host.present_transient_window(
            null_mut(),
            UiRect {
                left: 10,
                top: 20,
                right: 110,
                bottom: 70,
            },
        );
        host.hide_transient_window(null_mut());
        host.destroy_transient_window(null_mut());

        assert_eq!(host.class_name(), "ZsuiTransientWindow");
        assert_eq!(
            host.operation_log(),
            &[
                NativeTransientWindowHostOperation::PresentTransientWindow,
                NativeTransientWindowHostOperation::HideTransientWindow,
                NativeTransientWindowHostOperation::DestroyTransientWindow,
            ]
        );
    }
}
