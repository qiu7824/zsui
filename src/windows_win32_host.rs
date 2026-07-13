use std::{
    ffi::OsString,
    mem::{size_of, zeroed},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicI32, Ordering},
        Mutex, OnceLock,
    },
};

use crate::native_input_visuals::{
    decorate_native_focus_ring, decorate_native_text_edit_visuals, native_text_index_for_point,
};
use crate::native_text_edit::{
    delete_backward, delete_forward, insert_text, move_selection, set_pointer_selection,
    NativeTextDragState, NativeTextEditState, NativeTextMovement,
};
use crate::view::SharedLiveViewRuntime;
use crate::windows_gdi_renderer::{
    rect_from_win, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette, WindowsGdiRenderer,
};
use crate::{
    native_status_menu_command_from_menu, Command, FileDialogService, FileDialogSpec,
    HostCapabilities, MenuItemSpec, MenuSpec, NativeAppIconResource, NativeDrawPlan,
    NativeMainWindowHandles, NativeMainWindowHost, NativeMainWindowHostOperation,
    NativeMainWindowPresentMode, NativeMainWindowPresentation, NativeMainWindowRequest,
    NativeStatusItemHost, NativeStatusItemHostOperation, NativeStatusItemPresentation,
    NativeStatusItemRequest, NativeStatusMenuCommandHost, NativeStatusMenuCommandHostOperation,
    NativeStatusMenuCommandRequest, NativeStatusMenuCommandResult, NativeTransientWindowHost,
    NativeTransientWindowHostOperation, NativeTransientWindowPresentation,
    NativeTransientWindowRequest, NativeWindowOptions, Renderer, SaveFileDialogSpec,
    SharedAppCommandExecutor, SharedUiCommandExecutor, Size, TraySpec, UiCommand, UiRect, View,
    ViewEventCx, ViewInteractionPlan, ViewNode, ViewPaintCx, WindowSpec, ZsShellInteractionEvent,
    ZsShellInteractionUpdate, ZsShellRuntime, ZsuiError, ZsuiResult,
};
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT,
        WPARAM,
    },
    Graphics::{
        Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
        Gdi::{BeginPaint, EndPaint, InvalidateRect, ScreenToClient, UpdateWindow, PAINTSTRUCT},
    },
    System::LibraryLoader::GetModuleHandleW,
    System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    UI::{
        Controls::Dialogs::{
            CommDlgExtendedError, GetOpenFileNameW, GetSaveFileNameW, OFN_ALLOWMULTISELECT,
            OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT,
            OFN_PATHMUSTEXIST, OPENFILENAMEW,
        },
        HiDpi::{AdjustWindowRectExForDpi, GetDpiForSystem, GetDpiForWindow},
        Input::{
            Ime::{
                ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
                CANDIDATEFORM, CFS_EXCLUDE, GCS_RESULTSTR,
            },
            KeyboardAndMouse::{
                GetKeyState, ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_HOVER,
                TME_LEAVE, TRACKMOUSEEVENT, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
                VK_HOME, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE,
                VK_TAB, VK_UP,
            },
        },
        Shell::{
            Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            NOTIFYICONDATAW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CreateAcceleratorTableW, CreateMenu, CreatePopupMenu, CreateWindowExW,
            DefWindowProcW, DestroyAcceleratorTable, DestroyIcon, DestroyMenu, DestroyWindow,
            DispatchMessageW, DrawMenuBar, GetClientRect, GetCursorPos, GetMessageW,
            GetSystemMetrics, GetWindowLongPtrW, GetWindowLongW, GetWindowRect, IsWindow,
            KillTimer, LoadCursorW, LoadImageW, PostMessageW, PostQuitMessage, RegisterClassExW,
            SendMessageW, SetForegroundWindow, SetMenu, SetTimer, SetWindowLongPtrW,
            SetWindowLongW, SetWindowPos, ShowWindow, TrackPopupMenu, TranslateAcceleratorW,
            TranslateMessage, ACCEL, CREATESTRUCTW, CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, FALT, FCONTROL, FSHIFT, FVIRTKEY, GWLP_USERDATA, GWL_EXSTYLE, HACCEL,
            HCURSOR, HICON, HMENU, HTCAPTION, HWND_TOPMOST, ICON_BIG, ICON_SMALL, IDC_ARROW,
            IMAGE_ICON, LR_DEFAULTCOLOR, LR_LOADFROMFILE, MF_CHECKED, MF_GRAYED, MF_POPUP,
            MF_SEPARATOR, MF_STRING, MSG, SC_MOVE, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
            SW_HIDE, SW_SHOW, SW_SHOWNOACTIVATE, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON,
            WM_APP, WM_CAPTURECHANGED, WM_CHAR, WM_CLOSE, WM_COMMAND, WM_DPICHANGED, WM_ERASEBKGND,
            WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION, WM_KEYDOWN,
            WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE,
            WM_NCDESTROY, WM_PAINT, WM_SETICON, WM_SETTINGCHANGE, WM_SIZE, WM_SYSCOMMAND,
            WM_THEMECHANGED, WM_TIMER, WNDCLASSEXW, WNDPROC, WS_CAPTION, WS_CLIPCHILDREN,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
            WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

static ACTIVE_MAIN_WINDOW_COUNT: AtomicI32 = AtomicI32::new(0);
static WINDOW_DRAW_PLANS: OnceLock<Mutex<Vec<WindowsWindowDrawPlanRecord>>> = OnceLock::new();
static WINDOW_VIEW_INPUT_ROUTES: OnceLock<Mutex<Vec<WindowsWindowViewInputRouteRecord>>> =
    OnceLock::new();
static WINDOW_COMPLETED_VIEW_INPUT_REPORTS: OnceLock<
    Mutex<Vec<WindowsCompletedViewInputReportRecord>>,
> = OnceLock::new();
static WINDOW_SHELL_INPUT_ROUTES: OnceLock<Mutex<Vec<WindowsWindowShellInputRouteRecord>>> =
    OnceLock::new();
static WINDOW_MENU_COMMAND_TABLES: OnceLock<Mutex<Vec<WindowsWindowMenuCommandTableRecord>>> =
    OnceLock::new();

const HOVER_DEFAULT: u32 = u32::MAX;
const WM_MOUSELEAVE: u32 = 0x02A3;
const DEFAULT_MAIN_CLASS_NAME: &str = "ZsuiMainWindow";
const DEFAULT_QUICK_CLASS_NAME: &str = "ZsuiQuickWindow";
const DEFAULT_TRANSIENT_CLASS_NAME: &str = "ZsuiTransientWindow";
pub const ZSUI_WIN32_TRAY_CALLBACK_MESSAGE: u32 = WM_APP + 0x0551;
pub const ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS: u32 = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON;
const ZSUI_WIN32_VK_RETURN: u32 = 0x0d;
const ZSUI_WIN32_VK_TAB: u32 = 0x09;
const ZSUI_WIN32_VK_SPACE: u32 = 0x20;
const ZSUI_WIN32_LIVE_VIEW_POLL_TIMER_ID: usize = 0x5a51;
#[cfg(feature = "list")]
const ZSUI_WIN32_VK_UP: u32 = 0x26;
#[cfg(feature = "list")]
const ZSUI_WIN32_VK_DOWN: u32 = 0x28;

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

#[derive(Debug, Clone)]
struct WindowsCompletedViewInputReportRecord {
    hwnd: isize,
    report: WindowsWin32ViewInputDispatchReport,
}

#[derive(Debug, Clone)]
struct WindowsWindowShellInputRouteRecord {
    hwnd: isize,
    route: WindowsWin32ShellInputRoute,
}

#[derive(Debug, Clone)]
struct WindowsWindowMenuCommandTableRecord {
    hwnd: isize,
    command_table: WindowsWin32StatusMenuCommandTable,
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
        Self::run_with_windows(&[])
    }

    pub fn run_with_windows(
        windows: &[WindowsWin32OwnedMainWindowHandles],
    ) -> WindowsWin32MessageLoopResult {
        let mut msg: MSG = unsafe { zeroed() };
        loop {
            let code = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if code == -1 {
                return WindowsWin32MessageLoopResult::Failed;
            }
            if code == 0 {
                return WindowsWin32MessageLoopResult::Quit(msg.wParam as i32);
            }
            if windows
                .iter()
                .any(|window| window.translate_accelerator(&msg))
            {
                continue;
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
    create_windows_for_specs_with_routes(specs, draw_plans, input_routes, &[])
}

pub fn create_windows_for_specs_with_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    let owned =
        create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, shell_routes)?;
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
    create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, &[])
}

pub fn create_owned_windows_for_specs_with_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    ACTIVE_MAIN_WINDOW_COUNT.store(0, Ordering::SeqCst);
    clear_windows_win32_window_draw_plans();
    clear_windows_win32_window_view_input_routes();
    clear_windows_win32_window_shell_input_routes();
    clear_windows_win32_window_menu_command_tables();
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
                if let Some(Some(route)) = shell_routes.get(index) {
                    set_windows_win32_window_shell_input_route(created.main, route.clone());
                }
                host.apply_main_window_appearance(created.main);
                host.apply_main_window_appearance(created.quick);
                let mut owned = WindowsWin32OwnedMainWindowHandles::new(created);
                if let Some(icon_path) = icon_path.as_deref() {
                    let icon = WindowsWin32OwnedAppIconResource::from_icon_path(icon_path)?;
                    owned.set_main_owned_app_icon(&mut host, icon);
                }
                if let Some(menu) = spec.menu.as_ref() {
                    owned.set_main_owned_menu(WindowsWin32OwnedWindowMenu::attach(
                        created.main,
                        menu,
                    )?);
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
    window_menu: Option<WindowsWin32OwnedWindowMenu>,
    destroy_on_drop: bool,
}

impl WindowsWin32OwnedMainWindowHandles {
    pub fn new(handles: NativeMainWindowHandles<HWND>) -> Self {
        Self {
            handles,
            app_icons: Vec::new(),
            window_menu: None,
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

    pub fn set_main_owned_menu(&mut self, menu: WindowsWin32OwnedWindowMenu) {
        self.window_menu = Some(menu);
    }

    pub fn translate_accelerator(&self, message: &MSG) -> bool {
        self.window_menu
            .as_ref()
            .is_some_and(|menu| menu.translate_accelerator(message))
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
        if let Some(menu) = self.window_menu.take() {
            let _ = menu.detach_and_destroy();
        }
        for handle in [self.handles.quick, self.handles.main] {
            if handle.is_null() {
                continue;
            }
            clear_windows_win32_window_draw_plan(handle);
            clear_windows_win32_window_view_input_route(handle);
            clear_windows_win32_window_shell_input_route(handle);
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
    pub accelerator: Option<String>,
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
                accelerator,
                ..
            } => {
                entries.push(WindowsWin32StatusMenuCommandEntry {
                    native_id: *next_id,
                    item_id: id.clone(),
                    label: label.clone(),
                    command: command.clone(),
                    enabled: inherited_enabled && *enabled,
                    accelerator: accelerator.clone(),
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

#[derive(Debug)]
struct WindowsWin32OwnedAcceleratorTable {
    handle: HACCEL,
    entry_count: usize,
}

impl WindowsWin32OwnedAcceleratorTable {
    fn from_command_table(table: &WindowsWin32StatusMenuCommandTable) -> ZsuiResult<Option<Self>> {
        let mut entries = Vec::new();
        for command in table.entries().iter().filter(|entry| entry.enabled) {
            let Some(accelerator) = command.accelerator.as_deref() else {
                continue;
            };
            let accelerator = crate::native_menu::NativeMenuAccelerator::parse(accelerator)?;
            let cmd = u16::try_from(command.native_id).map_err(|_| {
                ZsuiError::invalid_spec(
                    "menu.accelerator",
                    "Win32 menu command id does not fit an accelerator table",
                )
            })?;
            entries.push(ACCEL {
                fVirt: windows_accelerator_flags(&accelerator),
                key: windows_accelerator_virtual_key(&accelerator)?,
                cmd,
            });
        }
        if entries.is_empty() {
            return Ok(None);
        }
        let handle = unsafe { CreateAcceleratorTableW(entries.as_ptr(), entries.len() as i32) };
        if handle.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_create_accelerator_table",
                "CreateAcceleratorTableW failed",
            ));
        }
        Ok(Some(Self {
            handle,
            entry_count: entries.len(),
        }))
    }

    fn translate(&self, window: HWND, message: &MSG) -> bool {
        unsafe { TranslateAcceleratorW(window, self.handle, message) != 0 }
    }
}

fn windows_accelerator_flags(accelerator: &crate::native_menu::NativeMenuAccelerator) -> u8 {
    let mut flags = FVIRTKEY;
    if accelerator.primary || accelerator.super_key {
        flags |= FCONTROL;
    }
    if accelerator.alt {
        flags |= FALT;
    }
    if accelerator.shift {
        flags |= FSHIFT;
    }
    flags
}

fn windows_accelerator_virtual_key(
    accelerator: &crate::native_menu::NativeMenuAccelerator,
) -> ZsuiResult<u16> {
    let key = match accelerator.key.as_str() {
        "Enter" | "Return" => VK_RETURN,
        "Tab" => VK_TAB,
        "Escape" => VK_ESCAPE,
        "Space" => VK_SPACE,
        "Backspace" => VK_BACK,
        "Delete" => VK_DELETE,
        "Up" => VK_UP,
        "Down" => VK_DOWN,
        "Left" => VK_LEFT,
        "Right" => VK_RIGHT,
        "Home" => VK_HOME,
        "End" => VK_END,
        "PageUp" => VK_PRIOR,
        "PageDown" => VK_NEXT,
        key if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
            key.as_bytes()[0] as u16
        }
        key if key.starts_with('F') => {
            let number = key[1..].parse::<u16>().map_err(|_| {
                ZsuiError::invalid_spec(
                    "menu.accelerator",
                    format!("unsupported Win32 accelerator key `{key}`"),
                )
            })?;
            VK_F1 + number - 1
        }
        key => {
            return Err(ZsuiError::invalid_spec(
                "menu.accelerator",
                format!("unsupported Win32 accelerator key `{key}`"),
            ));
        }
    };
    Ok(key)
}

impl Drop for WindowsWin32OwnedAcceleratorTable {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                DestroyAcceleratorTable(self.handle);
            }
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedWindowMenu {
    owner: HWND,
    menu: HMENU,
    accelerator_table: Option<WindowsWin32OwnedAcceleratorTable>,
    active: bool,
}

impl WindowsWin32OwnedWindowMenu {
    pub fn attach(owner: HWND, menu: &MenuSpec) -> ZsuiResult<Self> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "window.menu.owner",
                "Win32 window menu owner cannot be null",
            ));
        }
        let command_table = WindowsWin32StatusMenuCommandTable::from_menu(menu);
        let accelerator_table =
            WindowsWin32OwnedAcceleratorTable::from_command_table(&command_table)?;
        let handle = create_empty_window_menu()?;
        let mut command_index = 0;
        if let Err(err) =
            append_status_popup_menu_items(menu, handle, &command_table, &mut command_index)
        {
            unsafe {
                DestroyMenu(handle);
            }
            return Err(err);
        }
        if unsafe { SetMenu(owner, handle) } == 0 {
            unsafe {
                DestroyMenu(handle);
            }
            return Err(ZsuiError::host(
                "windows_win32_set_window_menu",
                "SetMenu failed",
            ));
        }
        unsafe {
            DrawMenuBar(owner);
        }
        set_windows_win32_window_menu_command_table(owner, command_table);
        Ok(Self {
            owner,
            menu: handle,
            accelerator_table,
            active: true,
        })
    }

    pub const fn handle(&self) -> HMENU {
        self.menu
    }

    pub fn accelerator_count(&self) -> usize {
        self.accelerator_table
            .as_ref()
            .map(|table| table.entry_count)
            .unwrap_or(0)
    }

    fn translate_accelerator(&self, message: &MSG) -> bool {
        self.active
            && self
                .accelerator_table
                .as_ref()
                .is_some_and(|table| table.translate(self.owner, message))
    }

    pub fn detach_and_destroy(mut self) -> bool {
        self.detach_and_destroy_active()
    }

    fn detach_and_destroy_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        clear_windows_win32_window_menu_command_table(self.owner);
        if !self.owner.is_null() && unsafe { IsWindow(self.owner) } != 0 {
            unsafe {
                SetMenu(self.owner, null_mut());
                DrawMenuBar(self.owner);
            }
        }
        let destroyed = unsafe { DestroyMenu(self.menu) != 0 };
        if destroyed {
            self.active = false;
        }
        destroyed
    }
}

impl Drop for WindowsWin32OwnedWindowMenu {
    fn drop(&mut self) {
        let _ = self.detach_and_destroy_active();
    }
}

#[derive(Debug, Default)]
pub struct WindowsWin32FileDialogService;

impl FileDialogService for WindowsWin32FileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        windows_win32_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        windows_win32_save_file_dialog(spec)
    }
}

pub fn windows_win32_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    const FILE_BUFFER_LEN: usize = 32_768;
    let mut file_buffer = vec![0u16; FILE_BUFFER_LEN];
    let title = wide_null(&spec.title);
    let filter = windows_file_dialog_filter(&spec.filters);
    let initial_dir = spec.current_path.as_deref().map(wide_null);
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = null_mut();
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = file_buffer.as_mut_ptr();
    dialog.nMaxFile = file_buffer.len() as u32;
    dialog.lpstrInitialDir = initial_dir
        .as_ref()
        .map(|path| path.as_ptr())
        .unwrap_or(null());
    dialog.lpstrTitle = title.as_ptr();
    dialog.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if spec.allow_multiple {
        dialog.Flags |= OFN_ALLOWMULTISELECT;
    }

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return windows_common_dialog_cancel_or_error("windows_open_file_dialog");
    }
    Ok(Some(parse_windows_open_file_buffer(&file_buffer)))
}

pub fn windows_win32_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    const FILE_BUFFER_LEN: usize = 32_768;
    let mut file_buffer = vec![0u16; FILE_BUFFER_LEN];
    if let Some(name) = spec.suggested_name.as_deref() {
        let encoded = name.encode_utf16().collect::<Vec<_>>();
        if encoded.len() + 1 > file_buffer.len() {
            return Err(ZsuiError::invalid_spec(
                "save_file_dialog.suggested_name",
                "suggested file name is too long",
            ));
        }
        file_buffer[..encoded.len()].copy_from_slice(&encoded);
    }
    let title = wide_null(&spec.title);
    let filter = windows_file_dialog_filter(&spec.filters);
    let initial_dir = spec.current_path.as_ref().map(|path| {
        path.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>()
    });
    let default_extension = windows_file_dialog_default_extension(&spec.filters);
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = null_mut();
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = file_buffer.as_mut_ptr();
    dialog.nMaxFile = file_buffer.len() as u32;
    dialog.lpstrInitialDir = initial_dir
        .as_ref()
        .map(|path| path.as_ptr())
        .unwrap_or(null());
    dialog.lpstrTitle = title.as_ptr();
    dialog.lpstrDefExt = default_extension
        .as_ref()
        .map(|extension| extension.as_ptr())
        .unwrap_or(null());
    dialog.Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;

    if unsafe { GetSaveFileNameW(&mut dialog) } == 0 {
        return windows_common_dialog_cancel_or_error("windows_save_file_dialog");
    }
    Ok(parse_windows_utf16_segments(&file_buffer)
        .into_iter()
        .next()
        .map(PathBuf::from))
}

fn windows_common_dialog_cancel_or_error<T>(operation: &'static str) -> ZsuiResult<Option<T>> {
    let error = unsafe { CommDlgExtendedError() };
    if error == 0 {
        Ok(None)
    } else {
        Err(ZsuiError::host(
            operation,
            format!("common dialog error 0x{error:08x}"),
        ))
    }
}

fn windows_file_dialog_filter(filters: &[crate::FileDialogFilter]) -> Vec<u16> {
    let mut output = Vec::new();
    if filters.is_empty() {
        append_windows_filter_part(&mut output, "All files");
        append_windows_filter_part(&mut output, "*.*");
    } else {
        for filter in filters {
            append_windows_filter_part(&mut output, &filter.name);
            let patterns = if filter.patterns.is_empty() {
                "*.*".to_string()
            } else {
                filter.patterns.join(";")
            };
            append_windows_filter_part(&mut output, &patterns);
        }
    }
    output.push(0);
    output
}

fn append_windows_filter_part(output: &mut Vec<u16>, value: &str) {
    output.extend(value.encode_utf16());
    output.push(0);
}

fn windows_file_dialog_default_extension(filters: &[crate::FileDialogFilter]) -> Option<Vec<u16>> {
    filters
        .iter()
        .flat_map(|filter| &filter.patterns)
        .find_map(|pattern| {
            pattern
                .strip_prefix("*.")
                .filter(|extension| !extension.is_empty() && !extension.contains(['*', '?', ';']))
        })
        .map(|extension| extension.encode_utf16().chain(Some(0)).collect())
}

fn parse_windows_open_file_buffer(buffer: &[u16]) -> Vec<PathBuf> {
    let parts = parse_windows_utf16_segments(buffer);
    match parts.as_slice() {
        [] => Vec::new(),
        [path] => vec![PathBuf::from(path)],
        [directory, names @ ..] => names
            .iter()
            .map(|name| PathBuf::from(directory).join(name))
            .collect(),
    }
}

fn parse_windows_utf16_segments(buffer: &[u16]) -> Vec<OsString> {
    buffer
        .split(|unit| *unit == 0)
        .take_while(|segment| !segment.is_empty())
        .map(OsString::from_wide)
        .collect()
}

fn create_empty_window_menu() -> ZsuiResult<HMENU> {
    let handle = unsafe { CreateMenu() };
    if handle.is_null() {
        Err(ZsuiError::host(
            "windows_win32_create_window_menu",
            "CreateMenu failed",
        ))
    } else {
        Ok(handle)
    }
}

fn set_windows_win32_window_menu_command_table(
    hwnd: HWND,
    command_table: WindowsWin32StatusMenuCommandTable,
) {
    let hwnd = hwnd as isize;
    let mut records = window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned");
    records.retain(|record| record.hwnd != hwnd);
    records.push(WindowsWindowMenuCommandTableRecord {
        hwnd,
        command_table,
    });
}

fn clear_windows_win32_window_menu_command_table(hwnd: HWND) {
    let hwnd = hwnd as isize;
    window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .retain(|record| record.hwnd != hwnd);
}

fn clear_windows_win32_window_menu_command_tables() {
    window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .clear();
}

fn window_menu_command_tables() -> &'static Mutex<Vec<WindowsWindowMenuCommandTableRecord>> {
    WINDOW_MENU_COMMAND_TABLES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn dispatch_windows_win32_window_menu_command(
    hwnd: HWND,
    native_id: u32,
) -> Option<NativeStatusMenuCommandResult> {
    if hwnd.is_null() {
        return None;
    }
    let result = window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd as isize)
        .map(|record| record.command_table.resolve_native_command_id(native_id))?;

    if let NativeStatusMenuCommandResult::Dispatched(command) = &result {
        if dispatch_windows_win32_window_view_input(hwnd, |route| {
            route.dispatch_app_command(command.clone())
        })
        .is_none()
            && *command == Command::Quit
        {
            unsafe {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    }
    Some(result)
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
            MenuItemSpec::Command {
                label,
                checked,
                accelerator,
                ..
            } => {
                let entry = command_table.entries().get(*command_index).ok_or_else(|| {
                    ZsuiError::host(
                        "windows_win32_status_popup_menu",
                        "status menu command table is missing a command entry",
                    )
                })?;
                *command_index += 1;
                let display_label = accelerator
                    .as_deref()
                    .map(|accelerator| format!("{label}\t{accelerator}"))
                    .unwrap_or_else(|| label.clone());
                let label = wide_null(&display_label);
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
    apply_windows_win32_window_theme(hwnd, plan.theme_mode);
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
    ui_command_view: Option<ViewNode<UiCommand>>,
    live_view: Option<SharedLiveViewRuntime>,
    focused_widget: Option<crate::WidgetId>,
    text_edit: Option<NativeTextEditState>,
    text_drag: Option<NativeTextDragState>,
    #[cfg(feature = "slider")]
    slider_drag: Option<crate::WidgetId>,
    dpi: crate::Dpi,
    pending_draw_plan: Option<NativeDrawPlan>,
    quit_requested: bool,
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
        Self {
            interaction_plan,
            ui_command_view: Some(ui_command_view),
            live_view: None,
            focused_widget: None,
            text_edit: None,
            text_drag: None,
            #[cfg(feature = "slider")]
            slider_drag: None,
            dpi: crate::Dpi::standard(),
            pending_draw_plan: None,
            quit_requested: false,
            app_command_executor: None,
            pending_app_commands: Vec::new(),
            ui_command_executor: None,
            pending_ui_commands: Vec::new(),
        }
    }

    pub fn from_live_view(live_view: SharedLiveViewRuntime) -> Self {
        Self {
            interaction_plan: live_view.interaction_plan(),
            ui_command_view: None,
            live_view: Some(live_view),
            focused_widget: None,
            text_edit: None,
            text_drag: None,
            #[cfg(feature = "slider")]
            slider_drag: None,
            dpi: crate::Dpi::standard(),
            pending_draw_plan: None,
            quit_requested: false,
            app_command_executor: None,
            pending_app_commands: Vec::new(),
            ui_command_executor: None,
            pending_ui_commands: Vec::new(),
        }
    }

    pub fn app_command_executor(mut self, executor: SharedAppCommandExecutor) -> Self {
        self.app_command_executor = Some(executor);
        self
    }

    pub fn ui_command_executor(mut self, executor: SharedUiCommandExecutor) -> Self {
        self.ui_command_executor = Some(executor);
        self
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

        report.handled = true;
        self.focus_target(target, &mut report);
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            return report;
        }

        self.dispatch_activation(target, &mut report);
        report
    }

    fn dispatch_pointer_down(
        &mut self,
        point: crate::Point,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            pointer_down_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(target) = self.interaction_plan.hit_target_at(point) else {
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
        report.handled = true;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report.text_drag_active = true;
        report.events.push(format!(
            "win32_view_text_pointer_down:{}:{}",
            target.widget.0, index
        ));
        self.rebuild_pending_draw_plan();
        report
    }

    fn dispatch_pointer_move(
        &mut self,
        point: crate::Point,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(drag) = self.text_drag else {
            #[cfg(feature = "slider")]
            if let Some(widget) = self.slider_drag {
                if let Some(target) = self.interaction_plan.hit_target_for_widget(widget) {
                    report.pointer_move_count = 1;
                    report.slider_drag_active = true;
                    return self.dispatch_slider_pointer(target, point, report);
                }
                self.slider_drag = None;
            }
            return report;
        };
        let Some(target) = self.interaction_plan.hit_target_for_widget(drag.widget) else {
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
        report.pointer_move_count = 1;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report.text_drag_active = true;
        if edit.selection_changed {
            self.rebuild_pending_draw_plan();
        }
        report.events.push(format!(
            "win32_view_text_pointer_move:{}:{}",
            drag.widget.0, index
        ));
        report
    }

    fn dispatch_pointer_up(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        if self.text_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            let completed_selection = self
                .text_edit
                .is_some_and(|state| !state.selection.is_collapsed());
            self.text_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.text_drag_count = usize::from(completed_selection);
            report.text_drag_active = false;
            report.events.push("win32_view_text_pointer_up".to_string());
            return report;
        }
        #[cfg(feature = "slider")]
        if self.slider_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            self.slider_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.slider_drag_count = 1;
            report.slider_drag_active = false;
            report
                .events
                .push("win32_view_slider_pointer_up".to_string());
            return report;
        }
        let mut report = self.dispatch_click(point);
        report.pointer_up_count = 1;
        report
    }

    fn cancel_pointer_drag(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let had_drag = self.text_drag.take().is_some();
        #[cfg(feature = "slider")]
        let had_drag = had_drag | self.slider_drag.take().is_some();
        WindowsWin32ViewInputDispatchReport {
            handled: had_drag,
            hit_target_count: self.hit_target_count(),
            events: had_drag
                .then(|| "win32_view_text_pointer_cancel".to_string())
                .into_iter()
                .collect(),
            ..WindowsWin32ViewInputDispatchReport::default()
        }
    }

    #[cfg(feature = "slider")]
    fn dispatch_slider_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: crate::Point,
        mut report: WindowsWin32ViewInputDispatchReport,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some((current, range)) = self.widget_slider_state(target.widget) else {
            self.slider_drag = None;
            return report;
        };
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, self.dpi).track;
        let fraction = point.x.saturating_sub(track.x) as f32 / track.width.max(1) as f32;
        let value = range.value_at_fraction(fraction);
        report.handled = true;
        report.slider_drag_active = self.slider_drag.is_some();
        if (value - current).abs() <= f32::EPSILON {
            return report;
        }
        report.slider_value_change_count = 1;
        report.events.push(format!(
            "win32_view_slider_changed:{}:{value}",
            target.widget.0
        ));
        report.event_count = 1;
        self.dispatch_event(
            crate::ViewEvent::SliderChanged {
                widget: target.widget,
                value,
            },
            &mut report,
        );
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
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            report
                .events
                .push(format!("win32_view_text_without_target:{}", widget.0));
            return report;
        };
        if !matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            report.events.push(format!(
                "win32_view_text_without_textbox_focus:{}",
                widget.0
            ));
            return report;
        }

        let mut value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        state.clamp(&value);
        let mut accepted = 0;
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
            accepted += usize::from(edit.handled);
            text_changed |= edit.text_changed;
            selection_changed |= edit.selection_changed;
            if ch != '\r' {
                previous_was_carriage_return = false;
            }
        }
        if accepted == 0 {
            return report;
        }

        self.text_edit = Some(state);
        report.text_input_count = accepted;
        report.text_selection_change_count = usize::from(selection_changed);
        report.text_caret = Some(state.selection.caret);
        report
            .events
            .push(format!("win32_view_text_changed:{}", widget.0));
        if text_changed {
            report.event_count = 1;
            self.dispatch_event(crate::ViewEvent::TextChanged { widget, value }, &mut report);
        } else if selection_changed {
            self.rebuild_pending_draw_plan();
        }
        report
    }

    fn dispatch_key_down(&mut self, virtual_key: u32) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_shift(virtual_key, false)
    }

    fn dispatch_key_down_with_shift(
        &mut self,
        virtual_key: u32,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            key_down_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        if virtual_key == ZSUI_WIN32_VK_TAB {
            self.dispatch_focus_traversal(if shift { -1 } else { 1 }, &mut report);
            return report;
        }

        let Some(widget) = self.focused_widget else {
            report.unhandled_key_count = 1;
            report
                .events
                .push(format!("win32_view_key_without_focus:{virtual_key}"));
            return report;
        };
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            report.unhandled_key_count = 1;
            report.events.push(format!(
                "win32_view_key_without_target:{widget:?}:{virtual_key}"
            ));
            return report;
        };

        if matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        ) {
            if virtual_key == u32::from(VK_DELETE) {
                let mut edit = self.dispatch_text_input("\u{7f}");
                edit.key_down_count = 1;
                return edit;
            }
            let movement = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(NativeTextMovement::Left),
                key if key == u32::from(VK_RIGHT) => Some(NativeTextMovement::Right),
                key if key == u32::from(VK_HOME) => Some(NativeTextMovement::Home),
                key if key == u32::from(VK_END) => Some(NativeTextMovement::End),
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
                report.text_navigation_count = 1;
                report.text_selection_change_count = usize::from(edit.selection_changed);
                report.text_caret = Some(state.selection.caret);
                report.events.push(format!(
                    "win32_view_text_navigate:{}:{virtual_key}:{}",
                    widget.0, state.selection.caret
                ));
                self.rebuild_pending_draw_plan();
                return report;
            }
        }

        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            let Some((current, range)) = self.widget_slider_state(widget) else {
                return report;
            };
            let delta = if shift { 10 } else { 1 };
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) || key == u32::from(VK_DOWN) => {
                    Some(range.offset_steps(current, -delta))
                }
                key if key == u32::from(VK_RIGHT) || key == u32::from(VK_UP) => {
                    Some(range.offset_steps(current, delta))
                }
                key if key == u32::from(VK_HOME) => Some(range.min()),
                key if key == u32::from(VK_END) => Some(range.max()),
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                report.slider_keyboard_change_count = 1;
                if (value - current).abs() <= f32::EPSILON {
                    return report;
                }
                report.slider_value_change_count = 1;
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_slider_key:{}:{value}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::SliderChanged { widget, value },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "list")]
        if matches!(virtual_key, ZSUI_WIN32_VK_UP | ZSUI_WIN32_VK_DOWN) {
            let offset = if virtual_key == ZSUI_WIN32_VK_UP {
                -1
            } else {
                1
            };
            if let Some((next_widget, index)) =
                self.widget_list_relative_widget(target.widget, offset)
            {
                if let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                {
                    self.focus_target(next_target, &mut report);
                    report.selection_count = 1;
                    report.keyboard_selection_count = 1;
                    report.events.push(format!(
                        "win32_view_key_select:{}:{}:{index}",
                        target.widget.0, next_widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::Click {
                            widget: next_widget,
                        },
                        &mut report,
                    );
                    report.event_count = 1;
                    return report;
                }
            }
        }

        match (target.kind, virtual_key) {
            (
                crate::ViewHitTargetKind::Button | crate::ViewHitTargetKind::Unknown,
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE,
            )
            | (
                crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle,
                ZSUI_WIN32_VK_SPACE,
            ) => {
                report.keyboard_activation_count = 1;
                report.events.push(format!(
                    "win32_view_key_activate:{}:{virtual_key}",
                    target.widget.0
                ));
                self.dispatch_activation(target, &mut report);
            }
            _ => {
                report.unhandled_key_count = 1;
                report.events.push(format!(
                    "win32_view_key_unhandled:{}:{virtual_key}",
                    target.widget.0
                ));
            }
        }
        report
    }

    fn dispatch_scroll(
        &mut self,
        point: crate::Point,
        delta_y: crate::Dp,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            scroll_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(target) = self.interaction_plan.hit_target_at(point) else {
            report.unhandled_scroll_count = 1;
            report
                .events
                .push(format!("win32_view_scroll_missed:{}:{}", point.x, point.y));
            return report;
        };

        #[cfg(feature = "scroll")]
        if let Some(scroll_widget) = self.widget_scroll_target(target.widget) {
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_scroll:{}:{}",
                scroll_widget.0, delta_y.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ScrollBy {
                    widget: scroll_widget,
                    delta_y,
                },
                &mut report,
            );
            return report;
        }

        let _ = delta_y;
        report.unhandled_scroll_count = 1;
        report.events.push(format!(
            "win32_view_scroll_without_scroll_target:{}",
            target.widget.0
        ));
        report
    }

    fn dispatch_app_command(&mut self, command: Command) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            app_command_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        report
            .app_command_names
            .push(crate::app_command_name(&command));
        report
            .events
            .push(format!("win32_window_menu_command:{command:?}"));
        if command == Command::Quit {
            report.quit_requested = true;
            self.quit_requested = true;
        }
        self.pending_app_commands.push(command);
        report
    }

    fn dispatch_focus_traversal(
        &mut self,
        offset: isize,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let Some(target) = self
            .interaction_plan
            .next_focus_target(self.focused_widget, offset)
        else {
            report.unhandled_key_count = 1;
            report
                .events
                .push(format!("win32_view_key_focus_unavailable:{offset}"));
            return;
        };

        self.focus_target(target, report);
        report.focus_traversal_count = 1;
        report.events.push(format!(
            "win32_view_key_focus:{}:{}",
            target.widget.0, offset
        ));
    }

    fn focus_target(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if self.focused_widget == Some(target.widget) {
            self.ensure_text_edit_for_target(target);
            report.focused_widget = Some(target.widget.0);
            report.text_caret = self.text_edit.map(|state| state.selection.caret);
            return;
        }
        self.text_drag = None;
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        self.focused_widget = Some(target.widget);
        self.ensure_text_edit_for_target(target);
        report.focus_count = 1;
        report.focused_widget = Some(target.widget.0);
        report.text_caret = self.text_edit.map(|state| state.selection.caret);
        if self.rebuild_pending_draw_plan() {
            report.focus_visual_count = 1;
            report
                .events
                .push(format!("win32_view_focus_visual:{}", target.widget.0));
        }
        report
            .events
            .push(format!("win32_view_focus:{}", target.widget.0));
    }

    fn dispatch_blur(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(widget) = self.focused_widget.take() else {
            return report;
        };
        self.text_edit = None;
        self.text_drag = None;
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        if self.rebuild_pending_draw_plan() {
            report.focus_visual_count = 1;
        }
        report
            .events
            .push(format!("win32_view_focus_visual_cleared:{}", widget.0));
        report
    }

    fn focused_target(&self) -> Option<crate::ViewHitTarget> {
        self.focused_widget
            .and_then(|widget| self.interaction_plan.hit_target_for_widget(widget))
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

    fn dispatch_activation(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let event = if matches!(
            target.kind,
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle
        ) {
            let checked = !self.widget_checked_value(target.widget).unwrap_or(false);
            report.toggle_count = 1;
            report
                .events
                .push(format!("win32_view_toggle:{}:{checked}", target.widget.0));
            crate::ViewEvent::Toggled {
                widget: target.widget,
                checked,
            }
        } else {
            report
                .events
                .push(format!("win32_view_click:{}", target.widget.0));
            #[cfg(feature = "list")]
            if let Some(index) = self.widget_list_index(target.widget) {
                report.selection_count = 1;
                report
                    .events
                    .push(format!("win32_view_select:{}:{index}", target.widget.0));
            }
            crate::ViewEvent::Click {
                widget: target.widget,
            }
        };
        report.event_count = 1;
        self.dispatch_event(event, report);
    }

    fn dispatch_event(
        &mut self,
        event: crate::ViewEvent,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_event(&event);
            report.message_count = update.message_count;
            report.ui_command_count = update.ui_commands.len();
            report.app_command_count = update.commands.len();
            report.live_view_revision = update.revision;
            report.quit_requested = update.quit_requested;
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
            return;
        }

        let mut event_cx = ViewEventCx::new();
        let Some(view) = &mut self.ui_command_view else {
            return;
        };
        view.event(&mut event_cx, &event);
        let commands = event_cx.into_messages();
        report.message_count = commands.len();
        report.ui_command_count = commands.len();
        for command in commands {
            report.ui_command_ids.push(command.id.0);
            report
                .events
                .push(format!("win32_view_ui_command:{}", command.id.0));
            self.pending_ui_commands.push(command);
        }
        self.rebuild_pending_draw_plan();
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
        if let (Some(target), Some(state)) = (self.focused_target(), self.text_edit) {
            if let Some(value) = self.widget_text_value(target.widget) {
                decorate_native_text_edit_visuals(
                    &mut plan,
                    target,
                    &value,
                    state.selection.clamp(&value),
                    self.dpi,
                );
            }
        }
        decorate_native_focus_ring(
            &mut plan,
            &self.interaction_plan,
            self.focused_widget,
            self.dpi,
        );
        self.pending_draw_plan = Some(plan);
        true
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
        self.live_view
            .as_ref()
            .and_then(SharedLiveViewRuntime::background_poll_interval_ms)
    }

    fn refresh_background_view(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let Some(live_view) = &self.live_view else {
            return WindowsWin32ViewInputDispatchReport::default();
        };
        let update = live_view.refresh();
        self.interaction_plan = live_view.interaction_plan();
        self.rebuild_pending_draw_plan();
        WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            background_refresh_count: 1,
            live_view_revision: update.revision,
            events: vec![format!(
                "win32_live_view_background_refresh:{}",
                update.revision
            )],
            ..WindowsWin32ViewInputDispatchReport::default()
        }
    }

    fn set_surface(&mut self, bounds: crate::Rect, dpi: crate::Dpi) -> bool {
        self.dpi = dpi;
        let Some(live_view) = &self.live_view else {
            return false;
        };
        if !live_view.set_surface(bounds, dpi) {
            return false;
        }
        self.interaction_plan = live_view.interaction_plan();
        self.rebuild_pending_draw_plan();
        true
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsWin32ViewInputDispatchReport {
    pub handled: bool,
    pub hit_target_count: usize,
    pub click_count: usize,
    pub pointer_down_count: usize,
    pub pointer_move_count: usize,
    pub pointer_up_count: usize,
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
    pub text_drag_count: usize,
    pub text_drag_active: bool,
    pub slider_value_change_count: usize,
    pub slider_keyboard_change_count: usize,
    pub slider_drag_count: usize,
    pub slider_drag_active: bool,
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
        self.hit_target_count = next.hit_target_count;
        self.click_count += next.click_count;
        self.pointer_down_count += next.pointer_down_count;
        self.pointer_move_count += next.pointer_move_count;
        self.pointer_up_count += next.pointer_up_count;
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
        self.text_drag_count += next.text_drag_count;
        self.text_drag_active = next.text_drag_active;
        self.slider_value_change_count += next.slider_value_change_count;
        self.slider_keyboard_change_count += next.slider_keyboard_change_count;
        self.slider_drag_count += next.slider_drag_count;
        self.slider_drag_active = next.slider_drag_active;
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
    dispatch_windows_win32_window_view_input(hwnd, |route| route.dispatch_pointer_move(point))
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
    if hwnd.is_null() {
        return None;
    }
    let hwnd_value = hwnd as isize;
    let (
        mut report,
        draw_plan,
        quit_requested,
        app_executor,
        app_commands,
        ui_executor,
        ui_commands,
        poll_interval_ms,
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

    dispatch_windows_win32_app_commands(&mut report, app_executor, app_commands);
    dispatch_windows_win32_ui_commands(&mut report, ui_executor, ui_commands);
    if let Some(record) = window_view_input_routes()
        .lock()
        .expect("window view input route registry should not be poisoned")
        .iter_mut()
        .find(|record| record.hwnd == hwnd_value)
    {
        record.report.merge(report.clone());
    }

    if let Some(draw_plan) = draw_plan {
        set_windows_win32_window_draw_plan(hwnd, draw_plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    if quit_requested {
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
    sync_windows_win32_live_view_poll_timer(hwnd, poll_interval_ms);
    Some(report)
}

fn sync_windows_win32_live_view_poll_timer(hwnd: HWND, interval_ms: Option<u64>) {
    if hwnd.is_null() {
        return;
    }
    unsafe {
        if let Some(interval_ms) = interval_ms {
            SetTimer(
                hwnd,
                ZSUI_WIN32_LIVE_VIEW_POLL_TIMER_ID,
                interval_ms.clamp(1, u32::MAX as u64) as u32,
                None,
            );
        } else {
            KillTimer(hwnd, ZSUI_WIN32_LIVE_VIEW_POLL_TIMER_ID);
        }
    }
}

fn dispatch_windows_win32_app_commands(
    report: &mut WindowsWin32ViewInputDispatchReport,
    executor: Option<SharedAppCommandExecutor>,
    commands: Vec<Command>,
) {
    let Some(executor) = executor else {
        report.app_command_unhandled_count += commands.len();
        return;
    };
    for command in commands {
        match executor.dispatch(command) {
            Ok(events) => {
                report.app_command_executed_count += 1;
                report.app_command_event_count += events.len();
            }
            Err(err) => {
                report.app_command_failed_count += 1;
                report.app_command_errors.push(err.to_string());
            }
        }
    }
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

fn completed_window_view_input_reports(
) -> &'static Mutex<Vec<WindowsCompletedViewInputReportRecord>> {
    WINDOW_COMPLETED_VIEW_INPUT_REPORTS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug, Clone)]
pub struct WindowsWin32ShellInputRoute {
    runtime: ZsShellRuntime,
    events: Vec<ZsShellInteractionEvent>,
}

impl WindowsWin32ShellInputRoute {
    pub fn new(runtime: ZsShellRuntime) -> Self {
        Self {
            runtime,
            events: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &ZsShellRuntime {
        &self.runtime
    }

    pub fn events(&self) -> &[ZsShellInteractionEvent] {
        &self.events
    }
}

pub fn set_windows_win32_window_shell_input_route(
    hwnd: HWND,
    mut route: WindowsWin32ShellInputRoute,
) -> bool {
    if hwnd.is_null() {
        return false;
    }
    if let Some((bounds, dpi)) = windows_win32_shell_surface(hwnd) {
        route.runtime.set_surface(bounds, dpi);
    }
    let plan = route.runtime.draw_plan();
    let hwnd_value = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    if let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) {
        record.route = route;
    } else {
        routes.push(WindowsWindowShellInputRouteRecord {
            hwnd: hwnd_value,
            route,
        });
    }
    drop(routes);
    set_windows_win32_window_draw_plan(hwnd, plan);
    unsafe {
        InvalidateRect(hwnd, null(), 0);
    }
    true
}

pub fn clear_windows_win32_window_shell_input_route(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_shell_input_routes() {
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.clear();
}

pub fn windows_win32_window_shell_input_events(hwnd: HWND) -> Option<Vec<ZsShellInteractionEvent>> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.route.events.clone())
}

pub fn dispatch_windows_win32_window_shell_pointer_move(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_move(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_leave(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_leave)
}

pub fn dispatch_windows_win32_window_shell_pointer_down(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_down(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_up(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_up)
}

pub fn dispatch_windows_win32_window_shell_pointer_cancel(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_cancel)
}

pub fn dispatch_windows_win32_window_shell_scroll(
    hwnd: HWND,
    delta_y: i32,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.scroll_by(delta_y))
}

pub fn refresh_windows_win32_window_shell_surface(hwnd: HWND) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |_| ZsShellInteractionUpdate::default())
}

fn dispatch_windows_win32_window_shell_update(
    hwnd: HWND,
    update: impl FnOnce(&mut ZsShellRuntime) -> ZsShellInteractionUpdate,
) -> Option<ZsShellInteractionUpdate> {
    if hwnd.is_null() {
        return None;
    }
    let surface = windows_win32_shell_surface(hwnd);
    let hwnd_value = hwnd as isize;
    let (result, plan) = {
        let mut routes = window_shell_input_routes()
            .lock()
            .expect("window shell input route registry should not be poisoned");
        let record = routes.iter_mut().find(|record| record.hwnd == hwnd_value)?;
        let surface_changed = surface
            .map(|(bounds, dpi)| record.route.runtime.set_surface(bounds, dpi))
            .unwrap_or(false);
        let mut result = update(&mut record.route.runtime);
        if surface_changed {
            result.redraw = true;
        }
        record.route.events.extend(result.events.iter().cloned());
        let plan = result.redraw.then(|| record.route.runtime.draw_plan());
        (result, plan)
    };

    if let Some(plan) = plan {
        set_windows_win32_window_draw_plan(hwnd, plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    Some(result)
}

fn window_shell_input_routes() -> &'static Mutex<Vec<WindowsWindowShellInputRouteRecord>> {
    WINDOW_SHELL_INPUT_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

fn windows_win32_shell_surface(hwnd: HWND) -> Option<(crate::Rect, crate::Dpi)> {
    let mut rect: RECT = unsafe { zeroed() };
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
        return None;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32;
    Some((rect_from_win(rect), crate::Dpi(dpi)))
}

fn track_windows_win32_shell_pointer_leave(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let mut event = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: HOVER_DEFAULT,
    };
    unsafe {
        TrackMouseEvent(&mut event);
    }
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
    run_windows_win32_native_window_event_loop_with_draw_plans_input_routes_and_status_items(
        specs,
        draw_plans,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_draw_plans_input_routes_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_routes_and_status_items(
        specs,
        draw_plans,
        input_routes,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_routes_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    if specs.is_empty() {
        return Ok(());
    }
    let _handles =
        create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, shell_routes)?;
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
    match WindowsWin32MessageLoop::run_with_windows(&_handles) {
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
        let (outer_width, outer_height) = windows_win32_outer_size_for_client(
            width,
            height,
            style_plan.style,
            style_plan.ex_style,
        );
        let class_name = wide_null(role.class_name(self.class_names));
        let create_params = WindowsWindowCreateParams::new(role, options.min_size);
        CreateWindowExW(
            style_plan.ex_style,
            class_name.as_ptr(),
            title.as_ptr(),
            style_plan.style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            outer_width,
            outer_height,
            null_mut(),
            null_mut(),
            module,
            &create_params as *const WindowsWindowCreateParams as _,
        )
    }
}

unsafe fn windows_win32_outer_size_for_client(
    width: i32,
    height: i32,
    style: u32,
    ex_style: u32,
) -> (i32, i32) {
    let width = width.max(1);
    let height = height.max(1);
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    let dpi = GetDpiForSystem().max(96);
    if AdjustWindowRectExForDpi(&mut rect, style, 0, ex_style, dpi) == 0 {
        (width, height)
    } else {
        (
            (rect.right - rect.left).max(width),
            (rect.bottom - rect.top).max(height),
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
            archive_windows_win32_window_view_input_report(hwnd);
            clear_windows_win32_window_shell_input_route(hwnd);
            clear_windows_win32_window_menu_command_table(hwnd);
            if matches!(role, WindowsWindowRole::Main)
                && ACTIVE_MAIN_WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst) <= 1
            {
                PostQuitMessage(0);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_ERASEBKGND => 1,
        WM_DPICHANGED => {
            let suggested = lparam as *const RECT;
            if !suggested.is_null() {
                let suggested = *suggested;
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    suggested.left,
                    suggested.top,
                    (suggested.right - suggested.left).max(1),
                    (suggested.bottom - suggested.top).max(1),
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
            }
            let shell_handled = refresh_windows_win32_window_shell_surface(hwnd).is_some();
            let live_view_handled = refresh_windows_win32_window_live_view_surface(hwnd);
            if shell_handled || live_view_handled {
                InvalidateRect(hwnd, null(), 0);
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_SIZE => {
            let shell_handled = refresh_windows_win32_window_shell_surface(hwnd).is_some();
            let live_view_handled = refresh_windows_win32_window_live_view_surface(hwnd);
            if shell_handled || live_view_handled {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_SETTINGCHANGE | WM_THEMECHANGED => {
            if let Some(plan) = window_draw_plan(hwnd) {
                if plan.theme_mode == crate::ZsuiThemeMode::System {
                    apply_windows_win32_window_theme(hwnd, plan.theme_mode);
                    InvalidateRect(hwnd, null(), 0);
                    return 0;
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MOUSEMOVE => {
            if dispatch_windows_win32_window_shell_pointer_move(hwnd, point_from_lparam(lparam))
                .is_some()
            {
                0
            } else if dispatch_windows_win32_window_view_pointer_move(
                hwnd,
                point_from_lparam(lparam),
            )
            .is_some_and(|report| report.handled)
            {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_MOUSELEAVE => {
            if dispatch_windows_win32_window_shell_pointer_leave(hwnd).is_some() {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_LBUTTONDOWN => {
            if dispatch_windows_win32_window_shell_pointer_down(hwnd, point_from_lparam(lparam))
                .is_some()
            {
                SetCapture(hwnd);
                0
            } else if dispatch_windows_win32_window_view_pointer_down(
                hwnd,
                point_from_lparam(lparam),
                (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0,
            )
            .is_some_and(|report| report.handled)
            {
                SetFocus(hwnd);
                SetCapture(hwnd);
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_LBUTTONUP => {
            if dispatch_windows_win32_window_shell_pointer_up(hwnd).is_some() {
                ReleaseCapture();
                0
            } else if dispatch_windows_win32_window_view_pointer_up(hwnd, point_from_lparam(lparam))
                .is_some_and(|report| report.handled)
            {
                SetFocus(hwnd);
                ReleaseCapture();
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_CAPTURECHANGED => {
            if dispatch_windows_win32_window_shell_pointer_cancel(hwnd).is_some() {
                0
            } else if cancel_windows_win32_window_view_pointer_drag(hwnd)
                .is_some_and(|report| report.handled)
            {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_IME_STARTCOMPOSITION => {
            position_windows_ime_candidate(hwnd);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_IME_COMPOSITION => {
            if (lparam as u32 & GCS_RESULTSTR) != 0 {
                if let Some(text) = windows_ime_composition_text(hwnd, GCS_RESULTSTR) {
                    if dispatch_windows_win32_window_view_text_input(hwnd, &text).is_some() {
                        return 0;
                    }
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_IME_ENDCOMPOSITION => DefWindowProcW(hwnd, msg, wparam, lparam),
        WM_KILLFOCUS => match dispatch_windows_win32_window_view_blur(hwnd) {
            Some(report) if !report.events.is_empty() => 0,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        },
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
        WM_COMMAND => {
            let native_id = (wparam & 0xffff) as u32;
            if dispatch_windows_win32_window_menu_command(hwnd, native_id).is_some() {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_KEYDOWN => match dispatch_windows_win32_window_view_key_down_with_shift(
            hwnd,
            wparam as u32,
            (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0,
        ) {
            Some(report) if report.unhandled_key_count == 0 => 0,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        },
        WM_MOUSEWHEEL => {
            let point = mouse_wheel_point_from_lparam(hwnd, lparam);
            let delta_y = mouse_wheel_scroll_delta_from_wparam(wparam);
            if dispatch_windows_win32_window_shell_scroll(hwnd, delta_y.0.round() as i32).is_some()
            {
                0
            } else {
                match dispatch_windows_win32_window_view_scroll(hwnd, point, delta_y) {
                    Some(report) if report.unhandled_scroll_count == 0 => 0,
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
        }
        WM_TIMER if wparam == ZSUI_WIN32_LIVE_VIEW_POLL_TIMER_ID => {
            if refresh_windows_win32_window_background_view(hwnd).is_some() {
                0
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

unsafe fn mouse_wheel_point_from_lparam(hwnd: HWND, lparam: LPARAM) -> crate::Point {
    let raw = lparam as u32;
    let mut point = POINT {
        x: (raw & 0xffff) as i16 as i32,
        y: ((raw >> 16) & 0xffff) as i16 as i32,
    };
    ScreenToClient(hwnd, &mut point);
    crate::Point {
        x: point.x,
        y: point.y,
    }
}

fn mouse_wheel_scroll_delta_from_wparam(wparam: WPARAM) -> crate::Dp {
    let wheel_delta = ((wparam >> 16) & 0xffff) as u16 as i16 as f32;
    crate::Dp::new(-(wheel_delta / 120.0) * 48.0)
}

fn text_from_char_wparam(wparam: WPARAM) -> Option<String> {
    match char::from_u32(wparam as u32) {
        Some(ch @ ('\u{8}' | '\r' | '\n')) => Some(ch.to_string()),
        Some(ch) if !ch.is_control() => Some(ch.to_string()),
        _ => None,
    }
}

unsafe fn windows_ime_composition_text(
    hwnd: HWND,
    kind: windows_sys::Win32::UI::Input::Ime::IME_COMPOSITION_STRING,
) -> Option<String> {
    let context = ImmGetContext(hwnd);
    if context.is_null() {
        return None;
    }
    let byte_len = ImmGetCompositionStringW(context, kind, null_mut(), 0);
    if byte_len <= 0 {
        ImmReleaseContext(hwnd, context);
        return None;
    }
    let mut utf16 = vec![0u16; byte_len as usize / size_of::<u16>()];
    let copied = ImmGetCompositionStringW(context, kind, utf16.as_mut_ptr() as _, byte_len as u32);
    ImmReleaseContext(hwnd, context);
    if copied <= 0 {
        None
    } else {
        Some(String::from_utf16_lossy(
            &utf16[..copied as usize / size_of::<u16>()],
        ))
    }
}

unsafe fn position_windows_ime_candidate(hwnd: HWND) {
    let Some(target) = windows_win32_window_focused_target(hwnd) else {
        return;
    };
    if !matches!(
        target.kind,
        crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
    ) {
        return;
    }
    let context = ImmGetContext(hwnd);
    if context.is_null() {
        return;
    }
    let form = CANDIDATEFORM {
        dwIndex: 0,
        dwStyle: CFS_EXCLUDE,
        ptCurrentPos: POINT {
            x: target.bounds.x,
            y: target.bounds.y + target.bounds.height,
        },
        rcArea: RECT {
            left: target.bounds.x,
            top: target.bounds.y,
            right: target.bounds.x + target.bounds.width,
            bottom: target.bounds.y + target.bounds.height,
        },
    };
    ImmSetCandidateWindow(context, &form);
    ImmReleaseContext(hwnd, context);
}

unsafe fn paint_no_flicker_background(hwnd: HWND) -> LRESULT {
    let mut ps: PAINTSTRUCT = zeroed();
    let target = BeginPaint(hwnd, &mut ps);
    if target.is_null() {
        return 0;
    }

    let mut rect: RECT = zeroed();
    if GetClientRect(hwnd, &mut rect) != 0 {
        let draw_plan = window_draw_plan(hwnd);
        let palette = windows_palette_for_draw_plan(draw_plan.as_ref());
        if let Some(buffered) = WindowsBufferedPaint::begin(target, &rect) {
            paint_win32_surface(buffered.hdc(), rect, palette, draw_plan.as_ref());
        } else {
            paint_win32_surface(target, rect, palette, draw_plan.as_ref());
        }
    }

    EndPaint(hwnd, &ps);
    0
}

fn windows_palette_for_draw_plan(draw_plan: Option<&NativeDrawPlan>) -> WindowsGdiPalette {
    match resolved_windows_theme_mode(
        draw_plan
            .map(|plan| plan.theme_mode)
            .unwrap_or(crate::ZsuiThemeMode::System),
    ) {
        crate::ZsuiThemeMode::Dark => WindowsGdiPalette::from_theme(&crate::ZsuiTheme::dark()),
        _ => WindowsGdiPalette::default(),
    }
}

fn resolved_windows_theme_mode(theme_mode: crate::ZsuiThemeMode) -> crate::ZsuiThemeMode {
    match theme_mode {
        crate::ZsuiThemeMode::System => windows_system_theme_mode(),
        mode => mode,
    }
}

pub fn windows_system_theme_mode() -> crate::ZsuiThemeMode {
    let subkey = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = wide_null("AppsUseLightTheme");
    let mut value = 1u32;
    let mut value_size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as _,
            &mut value_size,
        )
    };
    if status == 0 && value == 0 {
        crate::ZsuiThemeMode::Dark
    } else {
        crate::ZsuiThemeMode::Light
    }
}

fn apply_windows_win32_window_theme(hwnd: HWND, theme_mode: crate::ZsuiThemeMode) {
    if hwnd.is_null() {
        return;
    }
    let dark = i32::from(matches!(
        resolved_windows_theme_mode(theme_mode),
        crate::ZsuiThemeMode::Dark
    ));
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &dark as *const i32 as _,
            size_of::<i32>() as u32,
        );
    }
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
    fn draw_plan_theme_mode_selects_shared_dark_palette() {
        let plan = NativeDrawPlan::default().theme_mode(crate::ZsuiThemeMode::Dark);
        let palette = windows_palette_for_draw_plan(Some(&plan));

        assert_eq!(palette.surface, crate::ZsuiTheme::dark().colors.surface);
        assert_eq!(
            palette.primary_text,
            crate::ZsuiTheme::dark().colors.text_primary
        );
    }

    #[test]
    fn file_dialog_filter_and_multi_select_buffer_are_structured_utf16() {
        let filters = vec![
            crate::FileDialogFilter::new("Text", ["*.txt", "*.md"]),
            crate::FileDialogFilter::new("All", ["*.*"]),
        ];
        let filter_buffer = windows_file_dialog_filter(&filters);
        let filter_parts = parse_windows_utf16_segments(&filter_buffer);

        assert_eq!(
            filter_parts,
            vec![
                OsString::from("Text"),
                OsString::from("*.txt;*.md"),
                OsString::from("All"),
                OsString::from("*.*"),
            ]
        );
        assert_eq!(
            windows_file_dialog_default_extension(&filters),
            Some(vec!['t' as u16, 'x' as u16, 't' as u16, 0])
        );

        let mut selection = Vec::new();
        for part in ["C:\\docs", "one.txt", "two.md"] {
            selection.extend(part.encode_utf16());
            selection.push(0);
        }
        selection.push(0);
        assert_eq!(
            parse_windows_open_file_buffer(&selection),
            vec![
                PathBuf::from("C:\\docs\\one.txt"),
                PathBuf::from("C:\\docs\\two.md")
            ]
        );
    }

    fn view_input_route_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static VIEW_INPUT_ROUTE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        VIEW_INPUT_ROUTE_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("view input route test lock should not be poisoned")
    }

    #[test]
    fn main_window_styles_map_to_win32_flags() {
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
    fn tool_window_shape_maps_to_popup_topmost_flags() {
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
    fn decorated_window_converts_requested_client_size_to_outer_size() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::standard(),
        );
        let (width, height) =
            unsafe { windows_win32_outer_size_for_client(1280, 800, plan.style, plan.ex_style) };

        assert!(width >= 1280);
        assert!(height > 800);
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
        let _guard = view_input_route_test_lock();
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
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 77isize as HWND;
        let widget = crate::WidgetId::new(9);
        let executor = crate::SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![crate::AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 48,
                },
                crate::ViewHitTargetKind::Button,
            )]),
            crate::button("Save")
                .id(widget)
                .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.save"))),
        )
        .ui_command_executor(executor.clone());

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
        assert_eq!(dispatched.ui_command_executed_count, 1);
        assert_eq!(dispatched.ui_command_event_count, 1);
        assert_eq!(executor.report().executed_count, 1);
        assert_eq!(dispatched.ui_command_ids, vec!["zsui.test.win32.save"]);
        assert_eq!(dispatched.focus_count, 1);
        assert_eq!(dispatched.focused_widget, Some(widget.0));
        assert_eq!(aggregate.click_count, 1);
        assert_eq!(aggregate.focus_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
        assert!(windows_win32_window_view_input_report(hwnd).is_none());
    }

    #[test]
    fn window_shell_route_updates_navigation_and_draw_plan() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_shell_input_routes();
        let hwnd = 0x5252isize as HWND;
        let spec = crate::ZsShellLayoutSpec::new("gallery", "Gallery")
            .selected_nav("general")
            .nav_item(crate::ZsShellNavItemSpec::new("general", "General"))
            .nav_item(crate::ZsShellNavItemSpec::new("controls", "Controls"));
        let runtime = crate::ZsShellRuntime::new(
            spec,
            crate::Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            crate::Dpi::standard(),
        );

        assert!(set_windows_win32_window_shell_input_route(
            hwnd,
            WindowsWin32ShellInputRoute::new(runtime),
        ));
        let update =
            dispatch_windows_win32_window_shell_pointer_down(hwnd, crate::Point { x: 40, y: 140 })
                .expect("shell route should handle pointer input");

        assert_eq!(
            update.events,
            vec![crate::ZsShellInteractionEvent::NavigationSelected {
                id: "controls".to_string(),
            }]
        );
        assert!(window_draw_plan(hwnd).is_some());
        assert_eq!(
            windows_win32_window_shell_input_events(hwnd).unwrap(),
            update.events
        );

        clear_windows_win32_window_shell_input_route(hwnd);
        clear_windows_win32_window_draw_plan(hwnd);
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn window_live_view_route_updates_state_and_repaints_draw_plan() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 0x5353isize as HWND;
        let button_id = crate::WidgetId::new(501);

        #[derive(Clone)]
        enum Msg {
            Increment,
        }
        struct State {
            count: u32,
        }

        let runtime = crate::live_view_runtime(
            State { count: 0 },
            move |state| {
                crate::column([
                    crate::text(format!("Count: {}", state.count)),
                    crate::button("Increment")
                        .id(button_id)
                        .on_click(Msg::Increment),
                ])
            },
            |state, message, cx| match message {
                Msg::Increment => {
                    state.count += 1;
                    cx.command(crate::Command::custom("counter.incremented"));
                    cx.ui_command(UiCommand::app(crate::CommandId("counter.persist")));
                }
            },
            crate::Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 120,
            },
            crate::Dpi::standard(),
        );
        let executor = crate::SharedAppCommandExecutor::new(|command| {
            Ok(vec![crate::AppEvent::MenuCommand { command }])
        });
        let ui_executor = crate::SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![crate::AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        assert!(set_windows_win32_window_view_input_route(
            hwnd,
            WindowsWin32ViewInputRoute::from_live_view(runtime)
                .app_command_executor(executor.clone())
                .ui_command_executor(ui_executor.clone()),
        ));

        let report = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 150, y: 90 })
            .expect("live view route should handle click");

        assert_eq!(report.message_count, 1);
        assert_eq!(report.app_command_count, 1);
        assert_eq!(report.app_command_executed_count, 1);
        assert_eq!(report.app_command_event_count, 1);
        assert_eq!(executor.report().executed_count, 1);
        assert_eq!(report.ui_command_count, 1);
        assert_eq!(report.ui_command_executed_count, 1);
        assert_eq!(report.ui_command_event_count, 1);
        assert_eq!(ui_executor.report().executed_count, 1);
        assert_eq!(report.live_view_revision, 1);
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_live_view_repaint:1"));
        assert!(window_draw_plan(hwnd)
            .unwrap()
            .commands
            .iter()
            .any(|command| matches!(
                command,
                crate::NativeDrawCommand::Text(text) if text.text == "Count: 1"
            )));

        clear_windows_win32_window_view_input_route(hwnd);
        clear_windows_win32_window_draw_plan(hwnd);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_keyboard_activation_to_ui_command() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 80isize as HWND;
        let widget = crate::WidgetId::new(12);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 48,
                },
                crate::ViewHitTargetKind::Button,
            )]),
            crate::button("Save")
                .id(widget)
                .on_click(UiCommand::app(crate::CommandId(
                    "zsui.test.win32.keyboard_save",
                ))),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should focus button");
        let key = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_RETURN)
            .expect("focused button should dispatch keyboard activation");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(key.key_down_count, 1);
        assert_eq!(key.keyboard_activation_count, 1);
        assert_eq!(key.event_count, 1);
        assert_eq!(key.ui_command_count, 1);
        assert_eq!(key.ui_command_ids, vec!["zsui.test.win32.keyboard_save"]);
        assert_eq!(aggregate.key_down_count, 1);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_tab_focus_traversal() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 82isize as HWND;
        let first = crate::WidgetId::new(15);
        let second = crate::WidgetId::new(16);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([
                crate::ViewHitTarget::with_kind(
                    first,
                    crate::Rect {
                        x: 0,
                        y: 0,
                        width: 120,
                        height: 48,
                    },
                    crate::ViewHitTargetKind::Button,
                ),
                crate::ViewHitTarget::with_kind(
                    second,
                    crate::Rect {
                        x: 0,
                        y: 48,
                        width: 120,
                        height: 48,
                    },
                    crate::ViewHitTargetKind::Button,
                ),
            ]),
            crate::column([
                crate::button("First")
                    .id(first)
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.first"))),
                crate::button("Second")
                    .id(second)
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.second"))),
            ]),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let first_focus = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_TAB)
            .expect("registered route should focus first target from Tab");
        let second_focus = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_TAB)
            .expect("registered route should focus next target from Tab");
        let key = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_RETURN)
            .expect("focused second button should dispatch keyboard activation");
        let focused_plan = window_draw_plan(hwnd).expect("focus should publish a draw plan");
        let blur = dispatch_windows_win32_window_view_blur(hwnd)
            .expect("registered route should clear focus visuals");
        let blurred_plan = window_draw_plan(hwnd).expect("blur should publish a clean draw plan");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(first_focus.focus_traversal_count, 1);
        assert_eq!(first_focus.focus_visual_count, 1);
        assert_eq!(first_focus.focused_widget, Some(first.0));
        assert_eq!(second_focus.focus_traversal_count, 1);
        assert_eq!(second_focus.focus_visual_count, 1);
        assert_eq!(second_focus.focused_widget, Some(second.0));
        assert!(focused_plan.commands.iter().any(|command| {
            matches!(command, crate::NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                if rect.x == 1 && rect.y == 49 && rect.width == 118 && rect.height == 46)
        }));
        assert_eq!(key.ui_command_ids, vec!["zsui.test.win32.second"]);
        assert_eq!(blur.focus_visual_count, 1);
        assert!(!blurred_plan
            .commands
            .iter()
            .any(|command| { matches!(command, crate::NativeDrawCommand::StrokeRect { .. }) }));
        assert_eq!(aggregate.focus_traversal_count, 2);
        assert_eq!(aggregate.key_down_count, 3);
        assert_eq!(aggregate.focus_count, 2);
        assert_eq!(aggregate.focus_visual_count, 3);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(all(feature = "list", feature = "label"))]
    fn window_view_input_route_dispatches_list_selection_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn selected(_: usize) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.list_selected"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 81isize as HWND;
        let first = crate::WidgetId::new(13);
        let second = crate::WidgetId::new(14);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([
                crate::ViewHitTarget::new(
                    first,
                    crate::Rect {
                        x: 0,
                        y: 0,
                        width: 180,
                        height: 40,
                    },
                ),
                crate::ViewHitTarget::new(
                    second,
                    crate::Rect {
                        x: 0,
                        y: 40,
                        width: 180,
                        height: 40,
                    },
                ),
            ]),
            crate::list([(first, "One"), (second, "Two")], |(id, label)| {
                crate::text(label).id(id)
            })
            .on_select(selected),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let selection =
            dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 60 })
                .expect("registered route should select list row");
        let keyboard_selection =
            dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_UP)
                .expect("registered route should move list selection from keyboard");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(selection.click_count, 1);
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.selection_count, 1);
        assert_eq!(selection.ui_command_count, 1);
        assert_eq!(
            selection.ui_command_ids,
            vec!["zsui.test.win32.list_selected"]
        );
        assert_eq!(keyboard_selection.key_down_count, 1);
        assert_eq!(keyboard_selection.selection_count, 1);
        assert_eq!(keyboard_selection.keyboard_selection_count, 1);
        assert_eq!(keyboard_selection.ui_command_count, 1);
        assert_eq!(
            keyboard_selection.ui_command_ids,
            vec!["zsui.test.win32.list_selected"]
        );
        assert_eq!(aggregate.selection_count, 2);
        assert_eq!(aggregate.keyboard_selection_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn window_view_input_route_dispatches_scroll_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn scrolled(_: crate::Dp) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.scrolled"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 83isize as HWND;
        let scroll_id = crate::WidgetId::new(17);
        let row = crate::WidgetId::new(18);
        let mut view = crate::scroll(crate::text("Row").id(row))
            .id(scroll_id)
            .content_height(crate::Dp::new(120.0))
            .on_scroll(scrolled);
        let mut layout = crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 40,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let route = WindowsWin32ViewInputRoute::new(view.interaction_plan(), view);

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let scroll = dispatch_windows_win32_window_view_scroll(
            hwnd,
            crate::Point { x: 20, y: 20 },
            crate::Dp::new(48.0),
        )
        .expect("registered route should dispatch scroll");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(scroll.scroll_count, 1);
        assert_eq!(scroll.unhandled_scroll_count, 0);
        assert_eq!(scroll.event_count, 1);
        assert_eq!(scroll.ui_command_count, 1);
        assert_eq!(scroll.ui_command_ids, vec!["zsui.test.win32.scrolled"]);
        assert_eq!(aggregate.scroll_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_dispatches_text_input_to_ui_command() {
        let _guard = view_input_route_test_lock();
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
    #[cfg(feature = "textbox")]
    fn window_view_input_route_replaces_unicode_keyboard_selection() {
        let widget = crate::WidgetId::new(32);
        let mut route = WindowsWin32ViewInputRoute::new(
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
            crate::textbox("A中文Z").id(widget),
        );
        route.dispatch_click(crate::Point { x: 20, y: 20 });
        route.dispatch_key_down(u32::from(VK_HOME));
        route.dispatch_key_down(u32::from(VK_RIGHT));
        route.dispatch_key_down_with_shift(u32::from(VK_RIGHT), true);
        let selected = route.dispatch_key_down_with_shift(u32::from(VK_RIGHT), true);
        let selection_plan = route
            .take_pending_draw_plan()
            .expect("selection navigation should rebuild the draw plan");

        let replaced = route.dispatch_text_input("🙂");

        assert_eq!(selected.text_navigation_count, 1);
        assert_eq!(selected.text_selection_change_count, 1);
        assert_eq!(selected.text_caret, Some(3));
        assert!(selection_plan.commands.iter().any(|command| {
            matches!(
                command,
                crate::NativeDrawCommand::FillRect {
                    fill: crate::NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::Accent,
                        alpha: 64,
                    },
                    ..
                }
            )
        }));
        assert_eq!(replaced.text_caret, Some(2));
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A🙂Z"));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_captures_unicode_pointer_drag_selection() {
        let widget = crate::WidgetId::new(33);
        let mut route = WindowsWin32ViewInputRoute::new(
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
            crate::textbox("A中文Z").id(widget),
        );

        let pressed = route.dispatch_pointer_down(crate::Point { x: 16, y: 12 }, false);
        let dragged = route.dispatch_pointer_move(crate::Point { x: 32, y: 12 });
        let released = route.dispatch_pointer_up(crate::Point { x: 32, y: 12 });

        assert!(pressed.handled);
        assert_eq!(pressed.pointer_down_count, 1);
        assert_eq!(pressed.text_caret, Some(1));
        assert!(pressed.text_drag_active);
        assert_eq!(dragged.pointer_move_count, 1);
        assert_eq!(dragged.text_caret, Some(3));
        assert_eq!(dragged.text_selection_change_count, 1);
        assert!(dragged.text_drag_active);
        assert_eq!(released.pointer_up_count, 1);
        assert_eq!(released.text_drag_count, 1);
        assert!(!released.text_drag_active);

        let replaced = route.dispatch_text_input("🙂");

        assert_eq!(replaced.text_caret, Some(2));
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A🙂Z"));
    }

    #[test]
    #[cfg(feature = "slider")]
    fn window_view_input_route_drags_and_steps_slider() {
        fn changed(_: f32) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.slider_changed"))
        }

        let widget = crate::WidgetId::new(34);
        let range = crate::SliderRange::new(0.0, 100.0).step(5.0);
        let target = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            crate::ViewHitTargetKind::Slider,
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([target]),
            crate::slider(0.0, range).id(widget).on_slide(changed),
        );
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, crate::Dpi::standard()).track;

        let pressed = route.dispatch_pointer_down(
            crate::Point {
                x: track.x + track.width / 4,
                y: 16,
            },
            false,
        );
        let dragged = route.dispatch_pointer_move(crate::Point {
            x: track.x + track.width * 3 / 4,
            y: 16,
        });
        let released = route.dispatch_pointer_up(crate::Point {
            x: track.x + track.width * 3 / 4,
            y: 16,
        });
        let stepped = route.dispatch_key_down(u32::from(VK_LEFT));

        assert!(pressed.handled);
        assert_eq!(pressed.slider_value_change_count, 1);
        assert!(pressed.slider_drag_active);
        assert_eq!(dragged.slider_value_change_count, 1);
        assert_eq!(dragged.pointer_move_count, 1);
        assert_eq!(released.slider_drag_count, 1);
        assert!(!released.slider_drag_active);
        assert_eq!(stepped.slider_keyboard_change_count, 1);
        assert_eq!(stepped.slider_value_change_count, 1);
        assert_eq!(route.widget_slider_state(widget), Some((70.0, range)));
        assert_eq!(route.pending_ui_commands.len(), 3);
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_normalizes_multiline_text_and_ignores_single_line_enter() {
        let editor = crate::WidgetId::new(30);
        let mut editor_route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                editor,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 120,
                },
                crate::ViewHitTargetKind::TextEditor,
            )]),
            crate::text_editor("").id(editor),
        );
        editor_route.dispatch_click(crate::Point { x: 20, y: 20 });

        let editor_report = editor_route.dispatch_text_input("A\r\nB\n\nC");

        assert_eq!(editor_report.text_input_count, 6);
        assert_eq!(
            editor_route.widget_text_value(editor).as_deref(),
            Some("A\nB\n\nC")
        );

        let input = crate::WidgetId::new(31);
        let mut input_route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                input,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("").id(input),
        );
        input_route.dispatch_click(crate::Point { x: 20, y: 20 });

        let input_report = input_route.dispatch_text_input("\r");

        assert_eq!(input_report.text_input_count, 0);
        assert_eq!(input_report.event_count, 0);
        assert_eq!(input_route.widget_text_value(input).as_deref(), Some(""));
        assert_eq!(text_from_char_wparam('\r' as usize).as_deref(), Some("\r"));
    }

    #[test]
    #[cfg(feature = "checkbox")]
    fn window_view_input_route_dispatches_checkbox_toggle_to_ui_command() {
        let _guard = view_input_route_test_lock();
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
        let key_toggle = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_SPACE)
            .expect("focused checkbox should toggle from keyboard");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(toggle.toggle_count, 1);
        assert_eq!(toggle.event_count, 1);
        assert_eq!(toggle.ui_command_count, 1);
        assert_eq!(
            toggle.ui_command_ids,
            vec!["zsui.test.win32.toggle_changed"]
        );
        assert_eq!(key_toggle.key_down_count, 1);
        assert_eq!(key_toggle.keyboard_activation_count, 1);
        assert_eq!(key_toggle.toggle_count, 1);
        assert_eq!(key_toggle.ui_command_count, 1);
        assert_eq!(aggregate.toggle_count, 2);
        assert_eq!(aggregate.key_down_count, 1);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
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
    fn owned_accelerator_table_uses_typed_menu_commands_and_raii() {
        let mut menu = MenuSpec::new();
        menu.items.push(
            MenuItemSpec::command("Open", Command::custom("file.open")).accelerator("Ctrl+O"),
        );
        menu.items.push(
            MenuItemSpec::command("Save As", Command::custom("file.save_as"))
                .accelerator("Ctrl+Shift+S"),
        );
        let table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);
        let accelerators = WindowsWin32OwnedAcceleratorTable::from_command_table(&table)
            .expect("valid Win32 accelerators")
            .expect("accelerator table should be created");

        assert!(std::mem::needs_drop::<WindowsWin32OwnedAcceleratorTable>());
        assert_eq!(accelerators.entry_count, 2);
        assert_eq!(table.entries()[0].accelerator.as_deref(), Some("Ctrl+O"));
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
    fn transient_host_preserves_topmost_noactivate_window_shape() {
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
