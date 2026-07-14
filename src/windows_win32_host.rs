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

#[cfg(feature = "combo")]
use crate::native::NativeComboTypeAheadState;
use crate::native_file_dialog::{
    native_file_dialog_initial_directory, native_save_dialog_suggested_name,
};
use crate::native_input_visuals::{
    decorate_native_focus_ring, decorate_native_text_edit_visuals_in_viewport,
    native_text_first_visible_column_for_caret, native_text_first_visible_row_for_caret,
    native_text_index_for_point_in_viewport, native_text_index_for_vertical_move,
    native_text_index_for_vertical_page_move, native_text_scroll_visual_rows,
    native_text_visual_target, native_text_wheel_row_delta, NativeTextVisualDirection,
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
use crate::native_input_visuals::{
    decorate_native_pointer_visuals, native_pointer_visual_key, NativePointerVisualKey,
};
#[cfg(feature = "textbox")]
use crate::native_text_edit::{apply_text_edit_command, NativeTextHistory};
use crate::native_text_edit::{
    apply_text_input, move_selection, move_selection_to, set_pointer_selection,
    NativeTextDragState, NativeTextEditState, NativeTextMovement,
};
use crate::view::SharedLiveViewRuntime;
use crate::windows_gdi_renderer::{
    rect_from_win, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette, WindowsGdiRenderer,
};
use crate::{
    native_status_menu_command_from_menu, Color, Command, FileDialogService, FileDialogSpec,
    HostCapabilities, MenuItemSpec, MenuSpec, NativeAppIconResource, NativeDrawPlan,
    NativeMainWindowHandles, NativeMainWindowHost, NativeMainWindowHostOperation,
    NativeMainWindowPresentMode, NativeMainWindowPresentation, NativeMainWindowRequest,
    NativeStatusItemHost, NativeStatusItemHostOperation, NativeStatusItemPresentation,
    NativeStatusItemRequest, NativeStatusMenuCommandHost, NativeStatusMenuCommandHostOperation,
    NativeStatusMenuCommandRequest, NativeStatusMenuCommandResult, NativeTransientWindowHost,
    NativeTransientWindowHostOperation, NativeTransientWindowPresentation,
    NativeTransientWindowRequest, NativeWindowOptions, Renderer, SaveFileDialogSpec,
    SharedAppCommandExecutor, SharedUiCommandExecutor, Size, TraySpec, UiCommand, UiRect, View,
    ViewEventCx, ViewInteractionPlan, ViewNode, ViewPaintCx, WindowSpec, ZsAccelerator,
    ZsAcceleratorKey, ZsShellInteractionEvent, ZsShellInteractionUpdate, ZsShellRuntime, ZsuiError,
    ZsuiResult,
};
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT,
        WPARAM,
    },
    Graphics::{
        Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
        Gdi::{
            BeginPaint, EndPaint, GdiFlush, GetSysColor, InvalidateRect, ScreenToClient,
            UpdateWindow, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT,
            PAINTSTRUCT,
        },
    },
    System::LibraryLoader::GetModuleHandleW,
    System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    UI::{
        Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
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
                GetActiveWindow, GetKeyState, ReleaseCapture, SetCapture, SetFocus,
                TrackMouseEvent, TME_HOVER, TME_LEAVE, TRACKMOUSEEVENT, VK_BACK, VK_CONTROL,
                VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_HOME, VK_LEFT, VK_NEXT, VK_PRIOR,
                VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
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
            SetWindowLongW, SetWindowPos, ShowWindow, SystemParametersInfoW, TrackPopupMenu,
            TranslateAcceleratorW, TranslateMessage, ACCEL, CREATESTRUCTW, CS_DBLCLKS, CS_HREDRAW,
            CS_VREDRAW, CW_USEDEFAULT, FALT, FCONTROL, FSHIFT, FVIRTKEY, GWLP_USERDATA,
            GWL_EXSTYLE, HACCEL, HCURSOR, HICON, HMENU, HTCAPTION, HWND_TOPMOST, ICON_BIG,
            ICON_SMALL, IDC_ARROW, IMAGE_ICON, LR_DEFAULTCOLOR, LR_LOADFROMFILE, MF_CHECKED,
            MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, SC_MOVE, SM_CXICON, SM_CXSMICON,
            SM_CYICON, SM_CYSMICON, SPI_GETHIGHCONTRAST, SWP_FRAMECHANGED, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
            SW_SHOWNOACTIVATE, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP,
            WM_CAPTURECHANGED, WM_CHAR, WM_CLOSE, WM_COMMAND, WM_DPICHANGED, WM_ERASEBKGND,
            WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION, WM_KEYDOWN,
            WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE,
            WM_NCDESTROY, WM_PAINT, WM_PRINTCLIENT, WM_SETICON, WM_SETTINGCHANGE, WM_SIZE,
            WM_SYSCOLORCHANGE, WM_SYSCOMMAND, WM_THEMECHANGED, WM_TIMER, WNDCLASSEXW, WNDPROC,
            WS_CAPTION, WS_CLIPCHILDREN, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
            WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

#[cfg(feature = "tooltip")]
use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETMESSAGEDURATION, SPI_GETMOUSEHOVERTIME};

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

#[cfg(feature = "tooltip")]
fn windows_tooltip_timing() -> crate::tooltip::ZsTooltipTiming {
    let mut hover_ms = 500u32;
    let mut message_seconds = 5u32;
    unsafe {
        let mut value = 0u32;
        if SystemParametersInfoW(SPI_GETMOUSEHOVERTIME, 0, (&mut value as *mut u32).cast(), 0) != 0
            && value > 0
        {
            hover_ms = value;
        }
        value = 0;
        if SystemParametersInfoW(
            SPI_GETMESSAGEDURATION,
            0,
            (&mut value as *mut u32).cast(),
            0,
        ) != 0
            && value > 0
        {
            message_seconds = value;
        }
    }
    crate::tooltip::ZsTooltipTiming {
        open_delay: std::time::Duration::from_millis(u64::from(hover_ms)),
        visible_duration: std::time::Duration::from_secs(u64::from(message_seconds)),
    }
}
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
    pub accelerator: Option<ZsAccelerator>,
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
pub struct WindowsWin32OwnedAcceleratorTable {
    handle: HACCEL,
    entry_count: usize,
}

impl WindowsWin32OwnedAcceleratorTable {
    fn from_command_table(table: &WindowsWin32StatusMenuCommandTable) -> ZsuiResult<Option<Self>> {
        let mut bindings = Vec::new();
        for command in table.entries().iter().filter(|entry| entry.enabled) {
            let Some(accelerator) = command.accelerator.as_ref() else {
                continue;
            };
            let cmd = u16::try_from(command.native_id).map_err(|_| {
                ZsuiError::invalid_spec(
                    "menu.accelerator",
                    "Win32 menu command id does not fit an accelerator table",
                )
            })?;
            bindings.push((cmd, *accelerator));
        }
        if bindings.is_empty() {
            Ok(None)
        } else {
            Self::from_bindings(&bindings).map(Some)
        }
    }

    pub fn from_bindings(bindings: &[(u16, ZsAccelerator)]) -> ZsuiResult<Self> {
        if bindings.is_empty() {
            return Err(ZsuiError::invalid_spec(
                "accelerator.bindings",
                "Win32 accelerator bindings cannot be empty",
            ));
        }
        let mut entries = Vec::with_capacity(bindings.len());
        for (command, accelerator) in bindings {
            accelerator.validate()?;
            let flags = windows_accelerator_flags(accelerator);
            let key = windows_accelerator_virtual_key(accelerator)?;
            if entries
                .iter()
                .any(|entry: &ACCEL| entry.fVirt == flags && entry.key == key)
            {
                return Err(ZsuiError::invalid_spec(
                    "accelerator.bindings",
                    format!("duplicate accelerator `{accelerator}`"),
                ));
            }
            entries.push(ACCEL {
                fVirt: flags,
                key,
                cmd: *command,
            });
        }
        let handle = unsafe { CreateAcceleratorTableW(entries.as_ptr(), entries.len() as i32) };
        if handle.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_create_accelerator_table",
                "CreateAcceleratorTableW failed",
            ));
        }
        Ok(Self {
            handle,
            entry_count: entries.len(),
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub fn translate(&self, window: HWND, message: &MSG) -> bool {
        unsafe { TranslateAcceleratorW(window, self.handle, message) != 0 }
    }
}

fn windows_accelerator_flags(accelerator: &ZsAccelerator) -> u8 {
    let mut flags = FVIRTKEY;
    if accelerator.uses_primary() || accelerator.uses_super() {
        flags |= FCONTROL;
    }
    if accelerator.uses_alt() {
        flags |= FALT;
    }
    if accelerator.uses_shift() {
        flags |= FSHIFT;
    }
    flags
}

fn windows_accelerator_virtual_key(accelerator: &ZsAccelerator) -> ZsuiResult<u16> {
    accelerator.validate()?;
    let key = match accelerator.key() {
        ZsAcceleratorKey::Character(key) => key.to_ascii_uppercase() as u16,
        ZsAcceleratorKey::Enter => VK_RETURN,
        ZsAcceleratorKey::Tab => VK_TAB,
        ZsAcceleratorKey::Escape => VK_ESCAPE,
        ZsAcceleratorKey::Space => VK_SPACE,
        ZsAcceleratorKey::Backspace => VK_BACK,
        ZsAcceleratorKey::Delete => VK_DELETE,
        ZsAcceleratorKey::Up => VK_UP,
        ZsAcceleratorKey::Down => VK_DOWN,
        ZsAcceleratorKey::Left => VK_LEFT,
        ZsAcceleratorKey::Right => VK_RIGHT,
        ZsAcceleratorKey::Home => VK_HOME,
        ZsAcceleratorKey::End => VK_END,
        ZsAcceleratorKey::PageUp => VK_PRIOR,
        ZsAcceleratorKey::PageDown => VK_NEXT,
        ZsAcceleratorKey::Function(number) => VK_F1 + u16::from(number) - 1,
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
            .map(WindowsWin32OwnedAcceleratorTable::entry_count)
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
    let initial_dir =
        native_file_dialog_initial_directory(spec.current_path.as_deref()).map(|path| {
            path.as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>()
        });
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = unsafe { GetActiveWindow() };
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
    let suggested_name = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    );
    if let Some(name) = suggested_name.as_deref() {
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
    let initial_dir =
        native_file_dialog_initial_directory(spec.current_path.as_deref()).map(|path| {
            path.as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>()
        });
    let default_extension = windows_file_dialog_default_extension(&spec.filters);
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = unsafe { GetActiveWindow() };
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
                    .as_ref()
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
            ui_command_view: Some(ui_command_view),
            live_view: None,
            focused_widget: None,
            #[cfg(feature = "tooltip")]
            tooltip: crate::tooltip::ZsTooltipRuntime::new(windows_tooltip_timing()),
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
            ui_command_view: None,
            live_view: Some(live_view),
            focused_widget: None,
            #[cfg(feature = "tooltip")]
            tooltip: crate::tooltip::ZsTooltipRuntime::new(windows_tooltip_timing()),
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

    fn dispatch_click(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            click_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        let target = self.interaction_plan.hit_target_at(point);
        self.dismiss_popup_overlays_except(target.map(|target| target.widget), &mut report);
        let Some(target) = target else {
            if !report.handled {
                report.unhandled_click_count = 1;
                report
                    .events
                    .push(format!("win32_view_click_missed:{}:{}", point.x, point.y));
            }
            return report;
        };

        report.handled = true;
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBoxReveal {
            return report;
        }
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
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        let target = self.interaction_plan.hit_target_at(point);
        self.dismiss_popup_overlays_except(target.map(|target| target.widget), &mut report);
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
            self.rebuild_pending_draw_plan();
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

        self.focus_target(target, &mut report);
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let index = native_text_index_for_point_in_viewport(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.first_visible_visual_column,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
        let anchor = if shift { state.selection.anchor } else { index };
        let edit = set_pointer_selection(&value, &mut state.selection, anchor, index);
        state.preferred_visual_column = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
        state.first_visible_visual_column = native_text_first_visible_column_for_caret(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_column,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
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
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            self.dispatch_event(
                crate::ViewEvent::TextSelectionChanged {
                    widget: target.widget,
                    selection: state.selection.into(),
                },
                &mut report,
            );
        }
        self.rebuild_pending_draw_plan();
        report
    }

    fn dispatch_pointer_move(
        &mut self,
        point: crate::Point,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_pointer_move_at(point, std::time::Instant::now())
    }

    fn dispatch_pointer_move_at(
        &mut self,
        point: crate::Point,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self
            .tooltip
            .pointer_moved(&self.interaction_plan, point, now)
        {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(not(feature = "tooltip"))]
        let _ = now;
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
        {
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, self.pointer_pressed, &mut report);
        }
        #[cfg(feature = "password-box")]
        if let Some(widget) = self.password_peek {
            let still_peeking = self
                .interaction_plan
                .hit_target_at(point)
                .is_some_and(|target| {
                    target.widget == widget
                        && target.kind == crate::ViewHitTargetKind::PasswordBoxReveal
                });
            if !still_peeking {
                self.password_peek = None;
                report.handled = true;
                self.rebuild_pending_draw_plan();
            }
            return report;
        }
        let Some(drag) = self.text_drag else {
            #[cfg(feature = "color-picker")]
            if let Some((widget, kind)) = self.color_picker_drag {
                if let Some(target) = self
                    .interaction_plan
                    .hit_targets
                    .iter()
                    .copied()
                    .find(|target| target.widget == widget && target.kind == kind)
                {
                    report.pointer_move_count = 1;
                    report.color_picker_drag_active = true;
                    return self.dispatch_color_picker_pointer(target, point, report);
                }
                self.color_picker_drag = None;
            }
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
        let value = self
            .widget_display_text_value(drag.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == drag.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(drag.widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let index = native_text_index_for_point_in_viewport(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.first_visible_visual_column,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
        let edit = set_pointer_selection(&value, &mut state.selection, drag.anchor, index);
        state.preferred_visual_column = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
        state.first_visible_visual_column = native_text_first_visible_column_for_caret(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_column,
            self.widget_text_wrap(target.widget),
            self.dpi,
        );
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
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            self.dispatch_event(
                crate::ViewEvent::TextSelectionChanged {
                    widget: drag.widget,
                    selection: state.selection.into(),
                },
                &mut report,
            );
        }
        report
    }

    fn dispatch_pointer_up(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        if self.password_peek.take().is_some() {
            let mut report = WindowsWin32ViewInputDispatchReport {
                handled: true,
                hit_target_count: self.hit_target_count(),
                pointer_up_count: 1,
                ..WindowsWin32ViewInputDispatchReport::default()
            };
            self.rebuild_pending_draw_plan();
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, None, &mut report);
            return report;
        }
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "color-picker")]
        if self.color_picker_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            self.color_picker_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.color_picker_drag_count = 1;
            report.color_picker_drag_active = false;
            report
                .events
                .push("win32_view_color_picker_pointer_up".to_string());
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        let mut report = self.dispatch_click(point);
        report.pointer_up_count = 1;
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
        {
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, None, &mut report);
        }
        report
    }

    fn cancel_pointer_drag(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let had_drag = self.text_drag.take().is_some();
        #[cfg(feature = "password-box")]
        let had_drag = had_drag | self.password_peek.take().is_some();
        #[cfg(feature = "slider")]
        let had_drag = had_drag | self.slider_drag.take().is_some();
        #[cfg(feature = "color-picker")]
        let had_drag = had_drag | self.color_picker_drag.take().is_some();
        let report = WindowsWin32ViewInputDispatchReport {
            handled: had_drag,
            hit_target_count: self.hit_target_count(),
            events: had_drag
                .then(|| "win32_view_text_pointer_cancel".to_string())
                .into_iter()
                .collect(),
            ..WindowsWin32ViewInputDispatchReport::default()
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
        {
            let mut report = report;
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            report
        }
        #[cfg(not(any(
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
        )))]
        {
            report
        }
    }

    fn dispatch_pointer_leave(&mut self) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        let had_password_peek = self.password_peek.take().is_some();
        #[allow(unused_mut)]
        let mut report = WindowsWin32ViewInputDispatchReport {
            #[cfg(feature = "password-box")]
            handled: had_password_peek,
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
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
        {
            self.update_pointer_visual_state(None, None, &mut report);
            report
        }
        #[cfg(not(any(
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
        )))]
        {
            report
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

    #[cfg(feature = "color-picker")]
    fn dispatch_color_picker_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: crate::Point,
        mut report: WindowsWin32ViewInputDispatchReport,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some(state) = self.widget_color_picker_state(target.widget) else {
            self.color_picker_drag = None;
            return report;
        };
        let root_bounds = self
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|candidate| {
                candidate.widget == target.widget
                    && candidate.kind == crate::ViewHitTargetKind::ColorPicker
            })
            .map(|target| target.bounds)
            .unwrap_or(target.bounds);
        let plan = self.surface.map_or_else(
            || {
                crate::zs_color_picker_render_plan(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::Windows,
                    self.dpi,
                )
            },
            |viewport| {
                crate::zs_color_picker_render_plan_in_viewport(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::Windows,
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
        report.color_picker_drag_active = self.color_picker_drag.is_some();
        if let Some(channel) = channel.filter(|channel| state.active_channel != *channel) {
            report.color_picker_channel_change_count = 1;
            report.event_count += 1;
            report.events.push(format!(
                "win32_view_color_picker_channel:{}:{channel:?}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ColorPickerChannelChanged {
                    widget: target.widget,
                    channel,
                },
                &mut report,
            );
        }
        if color == state.color {
            return report;
        }
        report.color_picker_value_change_count = 1;
        report.event_count += 1;
        report.events.push(format!(
            "win32_view_color_picker_changed:{}:{}",
            target.widget.0,
            crate::ZsColorPickerState::new(color).hex_label()
        ));
        self.dispatch_event(
            crate::ViewEvent::ColorChanged {
                widget: target.widget,
                color,
            },
            &mut report,
        );
        report
    }

    fn dispatch_text_input(&mut self, text: &str) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_text_input_at(text, std::time::Instant::now())
    }

    fn dispatch_text_input_at(
        &mut self,
        text: &str,
        _now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
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
        #[cfg(feature = "dialog")]
        if self
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| state.open)
        {
            report.handled = true;
            report.events.push(format!(
                "win32_view_content_dialog_text_suppressed:{}",
                widget.0
            ));
            return report;
        }
        #[cfg(feature = "toast")]
        if self.widget_toast_state(widget).is_some() {
            report.handled = true;
            report
                .events
                .push(format!("win32_view_toast_text_suppressed:{}", widget.0));
            return report;
        }
        #[cfg(feature = "info-bar")]
        if self.widget_info_bar_state(widget).is_some() {
            report.handled = true;
            report
                .events
                .push(format!("win32_view_info_bar_text_suppressed:{}", widget.0));
            return report;
        }
        #[cfg(feature = "teaching-tip")]
        if self.widget_teaching_tip_state(widget).is_some() {
            report.handled = true;
            report.events.push(format!(
                "win32_view_teaching_tip_text_suppressed:{}",
                widget.0
            ));
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
                report.events.push(format!(
                    "win32_view_combo_type_ahead_no_match:{}:{}",
                    widget.0, query.text
                ));
                return report;
            };
            report.combo_type_ahead_match_count = 1;
            report.events.push(format!(
                "win32_view_combo_type_ahead_match:{}:{}:{index}",
                widget.0, query.text
            ));
            if selected == Some(index) {
                return report;
            }
            report.combo_selection_count = 1;
            report.combo_keyboard_selection_count = 1;
            report.combo_expanded_change_count = usize::from(expanded);
            report.event_count = 1;
            self.dispatch_event(
                crate::ViewEvent::ComboBoxSelected { widget, index },
                &mut report,
            );
            return report;
        }
        if !target.kind.accepts_text_input() {
            report.events.push(format!(
                "win32_view_text_without_textbox_focus:{}",
                widget.0
            ));
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
        let multiline = target.kind == crate::ViewHitTargetKind::TextEditor;
        let mut previous_was_carriage_return = false;
        let accepted = text
            .chars()
            .filter(|ch| {
                let accepted = matches!(*ch, '\u{8}' | '\u{7f}')
                    || (multiline
                        && (*ch == '\r' || (*ch == '\n' && !previous_was_carriage_return)))
                    || !ch.is_control();
                previous_was_carriage_return = *ch == '\r';
                accepted
            })
            .count();
        let edit = apply_text_input(&mut value, &mut state.selection, text, multiline);

        if !edit.handled {
            return report;
        }

        state.preferred_visual_column = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret(
            target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(widget),
            self.dpi,
        );
        state.first_visible_visual_column = native_text_first_visible_column_for_caret(
            target,
            &value,
            state.selection.caret,
            state.first_visible_visual_column,
            self.widget_text_wrap(widget),
            self.dpi,
        );
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
        report.text_input_count = accepted;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report
            .events
            .push(format!("win32_view_text_changed:{}", widget.0));
        if edit.text_changed {
            report.event_count = 1;
            #[cfg(feature = "command-palette")]
            if target.kind == crate::ViewHitTargetKind::CommandPalette {
                report.command_palette_query_change_count = 1;
            }
            #[cfg(feature = "auto-suggest")]
            if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
                report.auto_suggest_expanded_change_count = usize::from(
                    self.widget_auto_suggest_state(widget)
                        .is_some_and(|state| !state.expanded),
                );
            }
            #[cfg(feature = "password-box")]
            if let Some(password) = &mut password {
                *password.as_string_mut() = std::mem::take(&mut *value);
                self.dispatch_event(
                    crate::ViewEvent::PasswordChanged {
                        widget,
                        value: password.clone(),
                    },
                    &mut report,
                );
                return report;
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
                self.dispatch_event(
                    crate::ViewEvent::TextEdited {
                        widget,
                        value,
                        selection: state.selection.into(),
                    },
                    &mut report,
                );
                return report;
            }
            self.dispatch_event(crate::ViewEvent::TextChanged { widget, value }, &mut report);
        } else if edit.selection_changed {
            #[cfg(feature = "textbox")]
            if matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            ) {
                self.dispatch_event(
                    crate::ViewEvent::TextSelectionChanged {
                        widget,
                        selection: state.selection.into(),
                    },
                    &mut report,
                );
            }
            self.rebuild_pending_draw_plan();
        }
        report
    }

    fn dispatch_key_down(&mut self, virtual_key: u32) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_modifiers(virtual_key, false, false)
    }

    fn dispatch_key_down_with_shift(
        &mut self,
        virtual_key: u32,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_modifiers(virtual_key, shift, false)
    }

    fn dispatch_key_down_with_modifiers(
        &mut self,
        virtual_key: u32,
        shift: bool,
        control: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(not(any(feature = "radio", feature = "tabs")))]
        let _ = control;
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            key_down_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(feature = "toast")]
        if let Some(toast_target) = self
            .interaction_plan
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
            if virtual_key == u32::from(VK_ESCAPE) {
                report.handled = true;
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:EscapeKey",
                    toast_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: toast_target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::EscapeKey,
                        ),
                    },
                    &mut report,
                );
                return report;
            }
            if self.focused_widget == Some(toast_target.widget) {
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) => Some(-1),
                    key if key == u32::from(VK_RIGHT) => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.toast_focus_change_count = usize::from(next != state.focused_control);
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_toast_focus:{}:{next:?}",
                        toast_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::ToastFocused {
                            widget: toast_target.widget,
                            toast,
                            control: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let response = match state.focused_control {
                        crate::ZsToastControl::Action if spec.action_label().is_some() => {
                            crate::ZsToastResponse::Action
                        }
                        _ => crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.toast_response_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_toast_response:{}:{toast:?}:{response:?}",
                        toast_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::ToastResponded {
                            widget: toast_target.widget,
                            toast,
                            response,
                        },
                        &mut report,
                    );
                    return report;
                }
            }
        }
        #[cfg(feature = "teaching-tip")]
        if let Some(tip_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTip)
        {
            let Some((state, spec)) = self.widget_teaching_tip_state(tip_target.widget) else {
                return report;
            };
            if virtual_key == u32::from(VK_ESCAPE) {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::EscapeKey,
                );
                report.handled = true;
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:{response:?}",
                    tip_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: tip_target.widget,
                        response,
                    },
                    &mut report,
                );
                return report;
            }
            if self.focused_widget == Some(tip_target.widget) {
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) => Some(-1),
                    key if key == u32::from(VK_RIGHT) => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.teaching_tip_focus_change_count =
                        usize::from(next != state.focused_control);
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_teaching_tip_focus:{}:{next:?}",
                        tip_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TeachingTipFocused {
                            widget: tip_target.widget,
                            control: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let response = match state.focused_control {
                        crate::ZsTeachingTipControl::Action if spec.action_label().is_some() => {
                            crate::ZsTeachingTipResponse::Action
                        }
                        _ => crate::ZsTeachingTipResponse::Dismissed(
                            crate::ZsTeachingTipDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.teaching_tip_response_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_teaching_tip_response:{}:{response:?}",
                        tip_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TeachingTipResponded {
                            widget: tip_target.widget,
                            response,
                        },
                        &mut report,
                    );
                    return report;
                }
            }
        }
        #[cfg(feature = "info-bar")]
        if let Some(widget) = self.focused_widget {
            if let Some((state, spec)) = self.widget_info_bar_state(widget) {
                if virtual_key == u32::from(VK_ESCAPE) && spec.is_closable() {
                    report.handled = true;
                    report.info_bar_event_count = 1;
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_info_bar_event:{}:Close", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::InfoBarInvoked {
                            widget,
                            event: crate::ZsInfoBarEvent::Close,
                        },
                        &mut report,
                    );
                    return report;
                }
                if let Some(current) = state.focused_control {
                    let focus_offset = match virtual_key {
                        key if key == u32::from(VK_LEFT) => Some(-1),
                        key if key == u32::from(VK_RIGHT) => Some(1),
                        _ => None,
                    };
                    if let Some(offset) = focus_offset {
                        let next = spec.relative_control(current, offset);
                        report.handled = true;
                        report.info_bar_focus_change_count = usize::from(next != current);
                        report.event_count = 1;
                        report
                            .events
                            .push(format!("win32_view_info_bar_focus:{}:{next:?}", widget.0));
                        self.dispatch_event(
                            crate::ViewEvent::InfoBarFocused {
                                widget,
                                control: next,
                            },
                            &mut report,
                        );
                        return report;
                    }
                    if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                        let event = match current {
                            crate::ZsInfoBarControl::Action => crate::ZsInfoBarEvent::Action,
                            crate::ZsInfoBarControl::Close => crate::ZsInfoBarEvent::Close,
                        };
                        if spec.has_control(current) {
                            report.handled = true;
                            report.info_bar_event_count = 1;
                            report.event_count = 1;
                            report
                                .events
                                .push(format!("win32_view_info_bar_event:{}:{event:?}", widget.0));
                            self.dispatch_event(
                                crate::ViewEvent::InfoBarInvoked { widget, event },
                                &mut report,
                            );
                            return report;
                        }
                    }
                }
            }
        }
        #[cfg(feature = "breadcrumb")]
        if let Some(widget) = self.focused_widget {
            if let Some(state) = self.widget_breadcrumb_state(widget) {
                let mut visible = self
                    .interaction_plan
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
                let mut hidden = self
                    .interaction_plan
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

                if virtual_key == u32::from(VK_ESCAPE) && state.overflow_open {
                    report.handled = true;
                    report.breadcrumb_expanded_change_count = 1;
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_breadcrumb_expanded:{}:false", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::BreadcrumbExpandedChanged {
                            widget,
                            expanded: false,
                        },
                        &mut report,
                    );
                    return report;
                }

                let focus_list = if state.overflow_open
                    && matches!(virtual_key, key if key == u32::from(VK_UP) || key == u32::from(VK_DOWN))
                    && !hidden.is_empty()
                {
                    &hidden
                } else {
                    &visible
                };
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) || key == u32::from(VK_UP) => Some(-1),
                    key if key == u32::from(VK_RIGHT) || key == u32::from(VK_DOWN) => Some(1),
                    key if key == u32::from(VK_HOME) => Some(isize::MIN),
                    key if key == u32::from(VK_END) => Some(isize::MAX),
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
                    report.breadcrumb_focus_change_count = usize::from(state.focused != Some(next));
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_breadcrumb_focus:{}:{next:?}", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::BreadcrumbFocused {
                            widget,
                            target: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let active = state
                        .focused
                        .or_else(|| visible.first().copied())
                        .or_else(|| state.current().map(crate::ZsBreadcrumbFocusTarget::Item));
                    match active {
                        Some(crate::ZsBreadcrumbFocusTarget::Overflow) => {
                            report.handled = true;
                            report.breadcrumb_expanded_change_count = 1;
                            report.event_count = 1;
                            report.events.push(format!(
                                "win32_view_breadcrumb_expanded:{}:{}",
                                widget.0, !state.overflow_open
                            ));
                            self.dispatch_event(
                                crate::ViewEvent::BreadcrumbExpandedChanged {
                                    widget,
                                    expanded: !state.overflow_open,
                                },
                                &mut report,
                            );
                            return report;
                        }
                        Some(crate::ZsBreadcrumbFocusTarget::Item(item)) => {
                            report.handled = true;
                            report.breadcrumb_selection_count = 1;
                            report.breadcrumb_expanded_change_count =
                                usize::from(state.overflow_open);
                            report.event_count = 1;
                            report.events.push(format!(
                                "win32_view_breadcrumb_selected:{}:{}",
                                widget.0,
                                item.get()
                            ));
                            self.dispatch_event(
                                crate::ViewEvent::BreadcrumbSelected { widget, item },
                                &mut report,
                            );
                            return report;
                        }
                        None => {}
                    }
                }
            }
        }
        #[cfg(feature = "command-palette")]
        if let Some(palette_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::CommandPalette)
        {
            if self.focused_widget != Some(palette_target.widget) {
                self.focus_target(palette_target, &mut report);
            }
            report.focused_widget = Some(palette_target.widget.0);
            let Some(state) = self.widget_command_palette_state(palette_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let next = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_highlight(-1),
                key if key == u32::from(VK_DOWN) => state.relative_highlight(1),
                key if key == u32::from(VK_HOME) => state.first_enabled(),
                key if key == u32::from(VK_END) => state.last_enabled(),
                _ => None,
            };
            if let Some(item) = next {
                report.handled = true;
                report.command_palette_highlight_change_count =
                    usize::from(state.highlighted != Some(item));
                report.event_count = usize::from(report.command_palette_highlight_change_count > 0);
                if report.command_palette_highlight_change_count > 0 {
                    report.events.push(format!(
                        "win32_view_command_palette_highlight:{}:{}",
                        palette_target.widget.0,
                        item.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::CommandPaletteHighlighted {
                            widget: palette_target.widget,
                            item,
                        },
                        &mut report,
                    );
                }
                return report;
            }
            match virtual_key {
                ZSUI_WIN32_VK_RETURN => {
                    if let Some(item) = state.highlighted.or_else(|| state.first_enabled()) {
                        report.handled = true;
                        report.command_palette_invoke_count = 1;
                        report.command_palette_open_change_count = 1;
                        report.event_count = 1;
                        report.events.push(format!(
                            "win32_view_command_palette_invoke:{}:{}",
                            palette_target.widget.0,
                            item.get()
                        ));
                        self.dispatch_event(
                            crate::ViewEvent::CommandPaletteInvoked {
                                widget: palette_target.widget,
                                item,
                            },
                            &mut report,
                        );
                        return report;
                    }
                }
                key if key == u32::from(VK_ESCAPE) => {
                    report.handled = true;
                    report.command_palette_open_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_command_palette_dismissed:{}",
                        palette_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::CommandPaletteOpenChanged {
                            widget: palette_target.widget,
                            open: false,
                        },
                        &mut report,
                    );
                    return report;
                }
                ZSUI_WIN32_VK_TAB => {
                    report.handled = true;
                    return report;
                }
                _ => {}
            }
        }

        #[cfg(feature = "dialog")]
        if let Some(dialog_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialog)
        {
            if self.focused_widget != Some(dialog_target.widget) {
                self.focus_target(dialog_target, &mut report);
            }
            report.focused_widget = Some(dialog_target.widget.0);
            let Some((state, spec)) = self.widget_content_dialog_state(dialog_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let focus_offset = match virtual_key {
                ZSUI_WIN32_VK_TAB => Some(if shift { -1 } else { 1 }),
                key if key == u32::from(VK_LEFT) => Some(-1),
                key if key == u32::from(VK_RIGHT) => Some(1),
                _ => None,
            };
            if let Some(offset) = focus_offset {
                let button = spec.relative_button(state.focused_button, offset);
                report.handled = true;
                report.content_dialog_focus_change_count =
                    usize::from(button != state.focused_button);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_focus:{}:{button:?}",
                    dialog_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogFocused {
                        widget: dialog_target.widget,
                        button,
                    },
                    &mut report,
                );
                return report;
            }
            let response = match virtual_key {
                key if key == u32::from(VK_ESCAPE) => Some(crate::ZsContentDialogButton::Close),
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(state.focused_button),
                _ => None,
            };
            if let Some(button) = response.filter(|button| spec.has_button(*button)) {
                report.handled = true;
                report.content_dialog_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_response:{}:{button:?}",
                    dialog_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogResponded {
                        widget: dialog_target.widget,
                        button,
                    },
                    &mut report,
                );
                return report;
            }
            return report;
        }
        if virtual_key == ZSUI_WIN32_VK_TAB && !control {
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

        #[cfg(feature = "tabs")]
        if virtual_key == ZSUI_WIN32_VK_TAB && control {
            let offset = if shift { -1 } else { 1 };
            let Some((tab_view, tab)) = self.widget_tab_cycle_target(widget, offset) else {
                report.unhandled_key_count = 1;
                return report;
            };
            report.handled = true;
            report.tab_selection_count = 1;
            report.tab_keyboard_selection_count = 1;
            report.event_count = 1;
            if let Some(target) = self
                .interaction_plan
                .hit_target_for_widget(crate::WidgetId(tab.0))
            {
                self.focus_target(target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(target.widget);
            }
            report
                .events
                .push(format!("win32_view_tab_cycle:{}:{}", tab_view.0, tab.0));
            self.dispatch_event(
                crate::ViewEvent::TabSelected {
                    widget: tab_view,
                    tab,
                },
                &mut report,
            );
            return report;
        }
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            report.unhandled_key_count = 1;
            report.events.push(format!(
                "win32_view_key_without_target:{widget:?}:{virtual_key}"
            ));
            return report;
        };

        if target.kind.accepts_text_input() {
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
            let visual_navigation = (target.kind == crate::ViewHitTargetKind::TextEditor)
                .then(|| match virtual_key {
                    key if key == u32::from(VK_UP) => Some((NativeTextVisualDirection::Up, false)),
                    key if key == u32::from(VK_DOWN) => {
                        Some((NativeTextVisualDirection::Down, false))
                    }
                    key if key == u32::from(VK_PRIOR) => {
                        Some((NativeTextVisualDirection::Up, true))
                    }
                    key if key == u32::from(VK_NEXT) => {
                        Some((NativeTextVisualDirection::Down, true))
                    }
                    _ => None,
                })
                .flatten();
            if movement.is_some() || visual_navigation.is_some() {
                let value = self.widget_display_text_value(widget).unwrap_or_default();
                let mut state = self
                    .text_edit
                    .filter(|state| state.widget == widget)
                    .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
                let edit = if let Some((direction, page)) = visual_navigation {
                    let (target_index, preferred_column) = if page {
                        let (target_index, preferred_column, first_visible_row) =
                            native_text_index_for_vertical_page_move(
                                target,
                                &value,
                                state.selection.caret,
                                direction,
                                state.preferred_visual_column,
                                state.first_visible_visual_row,
                                self.widget_text_wrap(widget),
                                self.dpi,
                            );
                        state.first_visible_visual_row = first_visible_row;
                        (target_index, preferred_column)
                    } else {
                        native_text_index_for_vertical_move(
                            target,
                            &value,
                            state.selection.caret,
                            direction,
                            state.preferred_visual_column,
                            self.widget_text_wrap(widget),
                            self.dpi,
                        )
                    };
                    state.preferred_visual_column = Some(preferred_column);
                    move_selection_to(&value, &mut state.selection, target_index, shift)
                } else {
                    state.preferred_visual_column = None;
                    move_selection(
                        &value,
                        &mut state.selection,
                        movement.expect("text movement should be present"),
                        shift,
                        target.kind == crate::ViewHitTargetKind::TextEditor,
                    )
                };
                if !visual_navigation.is_some_and(|(_, page)| page) {
                    state.first_visible_visual_row = native_text_first_visible_row_for_caret(
                        target,
                        &value,
                        state.selection.caret,
                        state.first_visible_visual_row,
                        self.widget_text_wrap(widget),
                        self.dpi,
                    );
                }
                state.first_visible_visual_column = native_text_first_visible_column_for_caret(
                    target,
                    &value,
                    state.selection.caret,
                    state.first_visible_visual_column,
                    self.widget_text_wrap(widget),
                    self.dpi,
                );
                self.text_edit = Some(state);
                report.handled = edit.handled;
                report.text_navigation_count = 1;
                report.text_selection_change_count = usize::from(edit.selection_changed);
                report.text_caret = Some(state.selection.caret);
                report.events.push(format!(
                    "win32_view_text_navigate:{}:{virtual_key}:{}",
                    widget.0, state.selection.caret
                ));
                #[cfg(feature = "textbox")]
                if edit.selection_changed
                    && matches!(
                        target.kind,
                        crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                    )
                {
                    self.dispatch_event(
                        crate::ViewEvent::TextSelectionChanged {
                            widget,
                            selection: state.selection.into(),
                        },
                        &mut report,
                    );
                }
                self.rebuild_pending_draw_plan();
                return report;
            }
        }

        #[cfg(feature = "auto-suggest")]
        if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
            let Some(state) = self.widget_auto_suggest_state(widget) else {
                return report;
            };
            if (virtual_key == u32::from(VK_UP) || virtual_key == u32::from(VK_DOWN))
                && !state.suggestion_ids.is_empty()
            {
                let offset = if virtual_key == u32::from(VK_UP) {
                    -1
                } else {
                    1
                };
                let Some(suggestion) = state.next_highlight(offset) else {
                    return report;
                };
                report.handled = true;
                report.auto_suggest_highlight_change_count =
                    usize::from(state.highlighted != Some(suggestion));
                report.auto_suggest_expanded_change_count = usize::from(!state.expanded);
                report.event_count = 1;
                if !state.expanded {
                    self.dispatch_event(
                        crate::ViewEvent::AutoSuggestExpandedChanged {
                            widget,
                            expanded: true,
                        },
                        &mut report,
                    );
                    report.event_count += 1;
                }
                report.events.push(format!(
                    "win32_view_auto_suggest_highlight:{}:{}",
                    widget.0,
                    suggestion.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestHighlighted { widget, suggestion },
                    &mut report,
                );
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                report.handled = true;
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_keyboard_submit:{}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget,
                        suggestion: state.highlighted,
                    },
                    &mut report,
                );
                return report;
            }
            if virtual_key == u32::from(VK_ESCAPE) && state.expanded {
                report.handled = true;
                report.auto_suggest_expanded_change_count = 1;
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_auto_suggest_escape:{}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestExpandedChanged {
                        widget,
                        expanded: false,
                    },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "tree")]
        if target.kind == crate::ViewHitTargetKind::TreeView {
            let Some(state) = self.widget_tree_view_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_visible(-1),
                key if key == u32::from(VK_DOWN) => state.relative_visible(1),
                key if key == u32::from(VK_HOME) => state.first_visible(),
                key if key == u32::from(VK_END) => state.last_visible(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first_visible()),
                _ => None,
            };
            if let Some(node) = select {
                report.handled = true;
                if state.selected != Some(node) {
                    report.tree_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_select:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected { widget, node },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == u32::from(VK_LEFT) {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && row.expanded {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_collapse:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: false,
                        },
                        &mut report,
                    );
                } else if let Some(parent) = row.parent {
                    report.tree_selection_count = usize::from(state.selected != Some(parent));
                    report.event_count = report.tree_selection_count;
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected {
                            widget,
                            node: parent,
                        },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == u32::from(VK_RIGHT) {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && !row.expanded {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_expand:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: true,
                        },
                        &mut report,
                    );
                } else if let Some(child) = state.first_visible_child(node) {
                    report.tree_selection_count = usize::from(state.selected != Some(child));
                    report.event_count = report.tree_selection_count;
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected {
                            widget,
                            node: child,
                        },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(node) = state
                    .selected
                    .filter(|selected| state.row(*selected).is_some())
                else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.tree_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_tree_key_invoke:{}:{}",
                    widget.0,
                    node.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeInvoked { widget, node },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "grid-view")]
        if target.kind == crate::ViewHitTargetKind::GridView {
            let Some(state) = self.widget_grid_view_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_LEFT) => state.relative_horizontal(-1),
                key if key == u32::from(VK_RIGHT) => state.relative_horizontal(1),
                key if key == u32::from(VK_UP) => state.relative_vertical(-1),
                key if key == u32::from(VK_DOWN) => state.relative_vertical(1),
                key if key == u32::from(VK_HOME) => state.first(),
                key if key == u32::from(VK_END) => state.last(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first()),
                _ => None,
            };
            if let Some(item) = select {
                report.handled = true;
                if state.selected != Some(item) {
                    report.grid_view_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_grid_view_key_select:{}:{}",
                        widget.0,
                        item.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::GridViewItemSelected { widget, item },
                        &mut report,
                    );
                }
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(item) = state
                    .selected
                    .filter(|selected| state.contains(*selected))
                    .or_else(|| state.first())
                else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.grid_view_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_grid_view_key_invoke:{}:{}",
                    widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemInvoked { widget, item },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "table")]
        if target.kind == crate::ViewHitTargetKind::DataGrid {
            let Some(state) = self.widget_table_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_row(-1),
                key if key == u32::from(VK_DOWN) => state.relative_row(1),
                key if key == u32::from(VK_HOME) => state.first_row(),
                key if key == u32::from(VK_END) => state.last_row(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first_row()),
                _ => None,
            };
            if let Some(row) = select {
                report.handled = true;
                if state.selected != Some(row) {
                    report.table_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_table_key_select:{}:{}",
                        widget.0,
                        row.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TableRowSelected { widget, row },
                        &mut report,
                    );
                }
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(row) = state.selected.filter(|row| state.contains_row(*row)) else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.table_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_table_key_invoke:{}:{}",
                    widget.0,
                    row.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableRowInvoked { widget, row },
                    &mut report,
                );
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

        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let Some(state) = self.widget_color_picker_state(widget) else {
                return report;
            };
            let next_expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = next_expanded {
                report.handled = true;
                report.color_picker_expanded_change_count = 1;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorPickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            if !state.expanded {
                return report;
            }

            let next_channel = match virtual_key {
                key if key == u32::from(VK_UP) => {
                    Some(state.active_channel.previous(state.alpha_enabled))
                }
                key if key == u32::from(VK_DOWN) => {
                    Some(state.active_channel.next(state.alpha_enabled))
                }
                _ => None,
            };
            if let Some(channel) = next_channel {
                report.handled = true;
                if channel == state.active_channel {
                    return report;
                }
                report.color_picker_channel_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_channel:{}:{channel:?}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorPickerChannelChanged { widget, channel },
                    &mut report,
                );
                return report;
            }

            let current = state.channel_value(state.active_channel);
            let delta = if shift { 10_i16 } else { 1_i16 };
            let next = match virtual_key {
                key if key == u32::from(VK_LEFT) => {
                    Some((i16::from(current) - delta).clamp(0, 255) as u8)
                }
                key if key == u32::from(VK_RIGHT) => {
                    Some((i16::from(current) + delta).clamp(0, 255) as u8)
                }
                key if key == u32::from(VK_HOME) => Some(0),
                key if key == u32::from(VK_END) => Some(255),
                _ => None,
            };
            if let Some(value) = next {
                report.handled = true;
                let color = state.active_channel.with_value(state.color, value);
                if color == state.color {
                    return report;
                }
                report.color_picker_value_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_key:{}:{}",
                    widget.0,
                    crate::ZsColorPickerState::new(color).hex_label()
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorChanged { widget, color },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "number-box")]
        if target.kind == crate::ViewHitTargetKind::NumberBox {
            let event = match virtual_key {
                key if key == u32::from(VK_DOWN) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: shift,
                }),
                key if key == u32::from(VK_UP) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: shift,
                }),
                key if key == u32::from(VK_NEXT) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: true,
                }),
                key if key == u32::from(VK_PRIOR) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: true,
                }),
                ZSUI_WIN32_VK_RETURN => Some(crate::ViewEvent::NumberBoxCommit { widget }),
                key if key == u32::from(VK_ESCAPE) => {
                    Some(crate::ViewEvent::NumberBoxReset { widget })
                }
                _ => None,
            };
            if let Some(event) = event {
                report.handled = true;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_number_box_key:{}:{virtual_key}",
                    widget.0
                ));
                self.dispatch_event(event, &mut report);
                return report;
            }
        }

        #[cfg(feature = "combo")]
        if target.kind == crate::ViewHitTargetKind::ComboBox {
            let Some((selected, option_count, expanded)) = self.widget_combo_state(widget) else {
                return report;
            };
            if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE)
                || matches!(
                    virtual_key,
                    key if key == u32::from(VK_ESCAPE)
                        || key == u32::from(VK_UP)
                        || key == u32::from(VK_DOWN)
                        || key == u32::from(VK_HOME)
                        || key == u32::from(VK_END)
                )
            {
                self.combo_type_ahead.reset();
            }
            let expanded_event = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!expanded),
                key if key == u32::from(VK_ESCAPE) && expanded => Some(false),
                _ => None,
            };
            if let Some(next_expanded) = expanded_event {
                report.handled = true;
                report.combo_expanded_change_count = 1;
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    report.keyboard_activation_count = 1;
                }
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_expanded:{}:{next_expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxExpandedChanged {
                        widget,
                        expanded: next_expanded,
                    },
                    &mut report,
                );
                return report;
            }

            let next_index = match virtual_key {
                key if key == u32::from(VK_UP) && option_count > 0 => {
                    Some(selected.unwrap_or(option_count).saturating_sub(1))
                }
                key if key == u32::from(VK_DOWN) && option_count > 0 => {
                    Some(selected.map_or(0, |index| index.saturating_add(1).min(option_count - 1)))
                }
                key if key == u32::from(VK_HOME) && option_count > 0 => Some(0),
                key if key == u32::from(VK_END) && option_count > 0 => Some(option_count - 1),
                _ => None,
            };
            if let Some(index) = next_index {
                report.handled = true;
                if selected == Some(index) {
                    return report;
                }
                report.combo_selection_count = 1;
                report.combo_keyboard_selection_count = 1;
                report.combo_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_combo_key_select:{}:{index}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxSelected { widget, index },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            let navigation = match virtual_key {
                key if key == u32::from(VK_UP) => Some((crate::ViewStackDirection::Column, -1)),
                key if key == u32::from(VK_DOWN) => Some((crate::ViewStackDirection::Column, 1)),
                key if key == u32::from(VK_LEFT) => Some((crate::ViewStackDirection::Row, -1)),
                key if key == u32::from(VK_RIGHT) => Some((crate::ViewStackDirection::Row, 1)),
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
                let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget);
                if control {
                    report.radio_keyboard_focus_only_count = 1;
                    report.events.push(format!(
                        "win32_view_radio_key_focus_only:{}:{}",
                        widget.0, next_widget.0
                    ));
                    return report;
                }
                report.radio_selection_count = 1;
                report.radio_keyboard_selection_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_radio_key_select:{}:{}",
                    widget.0, next_widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::RadioSelected {
                        widget: next_widget,
                    },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "tabs")]
        if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
            let Some(state) = self.widget_tab_header_state(widget) else {
                return report;
            };
            let next_widget = match virtual_key {
                key if key == u32::from(VK_LEFT) => state.previous,
                key if key == u32::from(VK_RIGHT) => state.next,
                _ => None,
            };
            if matches!(
                virtual_key,
                key if key == u32::from(VK_LEFT)
                    || key == u32::from(VK_RIGHT)
            ) {
                report.handled = true;
                let Some(next_widget) = next_widget else {
                    return report;
                };
                if next_widget == widget {
                    return report;
                }
                let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget);
                report.tab_keyboard_focus_only_count = 1;
                report.events.push(format!(
                    "win32_view_tab_key_focus:{}:{}",
                    widget.0, next_widget.0
                ));
                return report;
            }
        }

        #[cfg(feature = "date-picker")]
        if target.kind == crate::ViewHitTargetKind::DatePicker {
            let Some(state) = self.widget_date_picker_state(widget) else {
                return report;
            };
            let expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = expanded {
                report.handled = true;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(state.value.add_days(-1)),
                key if key == u32::from(VK_RIGHT) => Some(state.value.add_days(1)),
                key if key == u32::from(VK_UP) => Some(state.value.add_days(-7)),
                key if key == u32::from(VK_DOWN) => Some(state.value.add_days(7)),
                key if key == u32::from(VK_HOME) => Some(state.value.first_day_of_month()),
                key if key == u32::from(VK_END) => {
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
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_date_picker_key:{}:{value}", widget.0));
                self.dispatch_event(crate::ViewEvent::DateChanged { widget, value }, &mut report);
                return report;
            }
        }

        #[cfg(feature = "time-picker")]
        if target.kind == crate::ViewHitTargetKind::TimePicker {
            let Some(state) = self.widget_time_picker_state(widget) else {
                return report;
            };
            let expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = expanded {
                report.handled = true;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimePickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            let minute_step = i32::from(state.minute_increment.get());
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(state.value.add_minutes_wrapping(-60)),
                key if key == u32::from(VK_RIGHT) => Some(state.value.add_minutes_wrapping(60)),
                key if key == u32::from(VK_UP) => {
                    Some(state.value.add_minutes_wrapping(-minute_step))
                }
                key if key == u32::from(VK_DOWN) => {
                    Some(state.value.add_minutes_wrapping(minute_step))
                }
                key if key == u32::from(VK_HOME) => Some(crate::ZsTime::MIDNIGHT),
                key if key == u32::from(VK_END) => {
                    crate::ZsTime::new(23, 60 - state.minute_increment.get()).ok()
                }
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                if value == state.value {
                    return report;
                }
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_time_picker_key:{}:{value}", widget.0));
                self.dispatch_event(crate::ViewEvent::TimeChanged { widget, value }, &mut report);
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
                    #[cfg(feature = "tooltip")]
                    self.show_keyboard_tooltip(next_target.widget);
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

        let activates = matches!(
            (target.kind, virtual_key),
            (
                crate::ViewHitTargetKind::Button | crate::ViewHitTargetKind::Unknown,
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE,
            ) | (
                crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle,
                ZSUI_WIN32_VK_SPACE,
            )
        );
        #[cfg(feature = "toggle-button")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (crate::ViewHitTargetKind::ToggleButton, ZSUI_WIN32_VK_SPACE)
            );
        #[cfg(feature = "radio")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (crate::ViewHitTargetKind::RadioButton, ZSUI_WIN32_VK_SPACE)
            );
        #[cfg(feature = "tabs")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (
                    crate::ViewHitTargetKind::Tab { .. },
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                )
            );
        if activates {
            report.keyboard_activation_count = 1;
            #[cfg(feature = "tabs")]
            if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
                let changed = self
                    .widget_tab_header_state(target.widget)
                    .is_some_and(|state| !state.selected);
                report.tab_selection_count = usize::from(changed);
                report.tab_keyboard_selection_count = usize::from(changed);
            }
            report.events.push(format!(
                "win32_view_key_activate:{}:{virtual_key}",
                target.widget.0
            ));
            self.dispatch_activation(target, &mut report);
        } else {
            report.unhandled_key_count = 1;
            report.events.push(format!(
                "win32_view_key_unhandled:{}:{virtual_key}",
                target.widget.0
            ));
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
            state.first_visible_visual_row = native_text_scroll_visual_rows(
                target,
                &value,
                previous,
                native_text_wheel_row_delta(delta_y),
                self.widget_text_wrap(target.widget),
                self.dpi,
            );
            self.text_edit = Some(state);
            report.handled = true;
            report.events.push(format!(
                "win32_view_text_scroll:{}:{}",
                target.widget.0, state.first_visible_visual_row
            ));
            if state.first_visible_visual_row != previous {
                self.rebuild_pending_draw_plan();
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
            let Some(visible_range) = self
                .interaction_plan
                .combo_visible_option_range(target.widget)
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
            report.handled = true;
            if next_first == visible_range.start {
                return report;
            }
            report.combo_scroll_count = 1;
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_combo_scroll:{}:{next_first}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ComboBoxScrolled {
                    widget: target.widget,
                    first_visible_index: next_first,
                },
                &mut report,
            );
            return report;
        }

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
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        report
            .app_command_names
            .push(crate::app_command_name(&command));
        report
            .events
            .push(format!("win32_window_menu_command:{command:?}"));

        if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_app_command(&command);
            if update.message_count > 0 {
                #[cfg(feature = "textbox")]
                let text_edit_commands = update.text_edit_commands.clone();
                report.handled = true;
                report.event_count = 1;
                report.message_count = update.message_count;
                report.app_command_count = update.commands.len();
                report.ui_command_count = update.ui_commands.len();
                report.live_view_revision = update.revision;
                report.quit_requested = update.quit_requested;
                for effect in update.commands {
                    report
                        .app_command_names
                        .push(crate::app_command_name(&effect));
                    if effect == Command::Quit {
                        report.quit_requested = true;
                        self.quit_requested = true;
                    }
                    self.pending_app_commands.push(effect);
                }
                for effect in update.ui_commands {
                    report.ui_command_ids.push(effect.id.0);
                    self.pending_ui_commands.push(effect);
                }
                if update.redraw {
                    self.interaction_plan = live_view.interaction_plan();
                    self.rebuild_pending_draw_plan();
                    report.hit_target_count = self.hit_target_count();
                    report
                        .events
                        .push(format!("win32_live_view_menu_repaint:{}", update.revision));
                }
                self.quit_requested |= update.quit_requested;
                #[cfg(feature = "textbox")]
                self.dispatch_text_edit_commands(text_edit_commands, &mut report);
                return report;
            }
        }

        report.app_command_count = 1;
        if command == Command::Quit {
            report.handled = true;
            report.quit_requested = true;
            self.quit_requested = true;
        }
        self.pending_app_commands.push(command);
        report
    }

    fn dispatch_window_close_requested(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let Some(command) = self.window_close_request_command.clone() else {
            return WindowsWin32ViewInputDispatchReport {
                window_close_request_count: 1,
                hit_target_count: self.hit_target_count(),
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        };
        let mut report = self.dispatch_app_command(command);
        report.window_close_request_count = 1;
        report
            .events
            .push("win32_window_close_requested".to_string());
        report
    }

    fn approve_next_close(&mut self) {
        self.close_approved = true;
    }

    fn take_close_approved(&mut self) -> bool {
        std::mem::take(&mut self.close_approved)
    }

    fn dispatch_focus_traversal(
        &mut self,
        offset: isize,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let Some(target) =
            self.interaction_plan
                .next_focus_target_where(self.focused_widget, offset, |target| {
                    self.widget_accepts_tab_focus(target)
                })
        else {
            report.unhandled_key_count = 1;
            report
                .events
                .push(format!("win32_view_key_focus_unavailable:{offset}"));
            return;
        };

        self.dismiss_popup_overlays_except(Some(target.widget), report);
        self.focus_target(target, report);
        #[cfg(feature = "tooltip")]
        self.show_keyboard_tooltip(target.widget);
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
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.interaction_plan
                .hit_target_for_widget(*widget)
                .is_some_and(|current| current.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            self.dispatch_event(crate::ViewEvent::NumberBoxCommit { widget }, report);
        }
        self.text_drag = None;
        #[cfg(feature = "password-box")]
        {
            self.password_peek = None;
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

    #[cfg(feature = "tooltip")]
    fn show_keyboard_tooltip(&mut self, widget: crate::WidgetId) {
        if self
            .tooltip
            .focus_widget(&self.interaction_plan, widget, std::time::Instant::now())
        {
            self.rebuild_pending_draw_plan();
        }
    }

    fn dispatch_blur(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        self.dismiss_popup_overlays_except(None, &mut report);
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(feature = "password-box")]
        {
            self.password_peek = None;
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
        self.update_pointer_visual_state(None, None, &mut report);
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.interaction_plan
                .hit_target_for_widget(*widget)
                .is_some_and(|target| target.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            self.dispatch_event(crate::ViewEvent::NumberBoxCommit { widget }, &mut report);
        }
        let Some(widget) = self.focused_widget.take() else {
            return report;
        };
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
            state.first_visible_visual_column = 0;
        }
        self.text_edit = Some(state);
    }

    fn dispatch_activation(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let expanded = self
                .widget_color_picker_state(target.widget)
                .is_none_or(|state| !state.expanded);
            report.color_picker_expanded_change_count = 1;
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_color_picker_expanded:{}:{expanded}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ColorPickerExpandedChanged {
                    widget: target.widget,
                    expanded,
                },
                report,
            );
            return;
        }
        #[cfg(feature = "command-palette")]
        match target.kind {
            crate::ViewHitTargetKind::CommandPaletteItem { item } => {
                report.command_palette_invoke_count = 1;
                report.command_palette_open_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_invoke:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::CommandPaletteInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPaletteClear => {
                report.command_palette_query_change_count = 1;
                report.command_palette_clear_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_cleared:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TextChanged {
                        widget: target.widget,
                        value: String::new(),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPaletteScrim => {
                report.command_palette_open_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_dismissed:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::CommandPaletteOpenChanged {
                        widget: target.widget,
                        open: false,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPalette => return,
            _ => {}
        }

        #[cfg(feature = "dialog")]
        match target.kind {
            crate::ViewHitTargetKind::ContentDialogButton { button } => {
                report.content_dialog_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_response:{}:{button:?}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogResponded {
                        widget: target.widget,
                        button,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ContentDialog
            | crate::ViewHitTargetKind::ContentDialogScrim => return,
            _ => {}
        }
        #[cfg(feature = "toast")]
        match target.kind {
            crate::ViewHitTargetKind::ToastAction => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return;
                };
                let Some(toast) = state.toast else {
                    return;
                };
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ToastClose => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return;
                };
                let Some(toast) = state.toast else {
                    return;
                };
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:CloseButton",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::Toast => return,
            _ => {}
        }
        #[cfg(feature = "info-bar")]
        match target.kind {
            crate::ViewHitTargetKind::InfoBarAction => {
                report.info_bar_event_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_info_bar_event:{}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::InfoBarClose => {
                report.info_bar_event_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_info_bar_event:{}:Close",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Close,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::InfoBar => return,
            _ => {}
        }
        #[cfg(feature = "teaching-tip")]
        match target.kind {
            crate::ViewHitTargetKind::TeachingTipAction => {
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response: crate::ZsTeachingTipResponse::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TeachingTipClose => {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::CloseButton,
                );
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:{response:?}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TeachingTip => return,
            _ => {}
        }
        #[cfg(feature = "breadcrumb")]
        match target.kind {
            crate::ViewHitTargetKind::BreadcrumbOverflow => {
                let expanded = self
                    .widget_breadcrumb_state(target.widget)
                    .map_or(true, |state| !state.overflow_open);
                report.breadcrumb_expanded_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_breadcrumb_expanded:{}:{}",
                    target.widget.0, expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::BreadcrumbExpandedChanged {
                        widget: target.widget,
                        expanded,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::BreadcrumbItem { item }
            | crate::ViewHitTargetKind::BreadcrumbOverflowItem { item } => {
                report.breadcrumb_selection_count = 1;
                report.breadcrumb_expanded_change_count = usize::from(
                    self.widget_breadcrumb_state(target.widget)
                        .is_some_and(|state| state.overflow_open),
                );
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_breadcrumb_selected:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::BreadcrumbSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::BreadcrumbBar => return,
            _ => {}
        }
        #[cfg(feature = "tree")]
        match target.kind {
            crate::ViewHitTargetKind::TreeNodeExpander { node } => {
                let Some(row) = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.row(node))
                else {
                    return;
                };
                if row.expandable {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_expanded:{}:{}:{}",
                        target.widget.0,
                        node.get(),
                        !row.expanded
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget: target.widget,
                            node,
                            expanded: !row.expanded,
                        },
                        report,
                    );
                }
                return;
            }
            crate::ViewHitTargetKind::TreeNode { node } => {
                let selected = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.tree_selection_count = usize::from(selected != Some(node));
                report.tree_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_tree_invoke:{}:{}",
                    target.widget.0,
                    node.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeSelected {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeInvoked {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "grid-view")]
        match target.kind {
            crate::ViewHitTargetKind::GridViewItem { item } => {
                let selected = self
                    .widget_grid_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.grid_view_selection_count = usize::from(selected != Some(item));
                report.grid_view_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_grid_view_invoke:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::GridView => return,
            _ => {}
        }
        #[cfg(feature = "table")]
        match target.kind {
            crate::ViewHitTargetKind::TableHeader { column } => {
                report.table_sort_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_table_sort:{}:{}",
                    target.widget.0,
                    column.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableSorted {
                        widget: target.widget,
                        column,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TableRow { row } => {
                let selected = self
                    .widget_table_state(target.widget)
                    .and_then(|state| state.selected);
                report.table_selection_count = usize::from(selected != Some(row));
                report.table_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_table_invoke:{}:{}",
                    target.widget.0,
                    row.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableRowSelected {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::TableRowInvoked {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "auto-suggest")]
        match target.kind {
            crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion } => {
                let expanded = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_submit:{}:{}",
                    target.widget.0,
                    suggestion.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget: target.widget,
                        suggestion: Some(suggestion),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::AutoSuggestSearch => {
                let state = self.widget_auto_suggest_state(target.widget);
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count =
                    usize::from(state.as_ref().is_some_and(|state| state.expanded));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_query_submit:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget: target.widget,
                        suggestion: state.and_then(|state| state.highlighted),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::AutoSuggestClear => {
                let expanded = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.auto_suggest_clear_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_cleared:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestCleared {
                        widget: target.widget,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "number-box")]
        match target.kind {
            crate::ViewHitTargetKind::NumberBoxDecrement
            | crate::ViewHitTargetKind::NumberBoxIncrement => {
                let steps = if target.kind == crate::ViewHitTargetKind::NumberBoxIncrement {
                    1
                } else {
                    -1
                };
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_number_box_step:{}:{steps}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::NumberBoxStep {
                        widget: target.widget,
                        steps,
                        large: false,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "time-picker")]
        match target.kind {
            crate::ViewHitTargetKind::TimePickerChoice { value } => {
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_selected:{}:{value}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimeChanged {
                        widget: target.widget,
                        value,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TimePicker => {
                let expanded = self
                    .widget_time_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimePickerExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "date-picker")]
        match target.kind {
            crate::ViewHitTargetKind::DatePickerDay { date } => {
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_selected:{}:{date}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DateChanged {
                        widget: target.widget,
                        value: date,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::DatePickerPreviousMonth
            | crate::ViewHitTargetKind::DatePickerNextMonth => {
                let Some(state) = self.widget_date_picker_state(target.widget) else {
                    return;
                };
                let offset = if target.kind == crate::ViewHitTargetKind::DatePickerPreviousMonth {
                    -1
                } else {
                    1
                };
                let month = state.visible_month.add_months(offset);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_month:{}:{month}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerMonthChanged {
                        widget: target.widget,
                        month,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::DatePicker => {
                let expanded = self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "combo")]
        match target.kind {
            crate::ViewHitTargetKind::ComboBoxOption { index } => {
                let expanded = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
                report.combo_selection_count = 1;
                report.combo_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_selected:{}:{index}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxSelected {
                        widget: target.widget,
                        index,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ComboBox => {
                let expanded = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
                report.combo_expanded_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            report.radio_selection_count = 1;
            report
                .events
                .push(format!("win32_view_radio_selected:{}", target.widget.0));
            report.event_count = 1;
            self.dispatch_event(
                crate::ViewEvent::RadioSelected {
                    widget: target.widget,
                },
                report,
            );
            return;
        }
        #[cfg(feature = "tabs")]
        if let crate::ViewHitTargetKind::Tab { tab_view, tab, .. } = target.kind {
            report.tab_selection_count = usize::from(
                self.widget_tab_header_state(target.widget)
                    .is_some_and(|state| !state.selected),
            );
            report.event_count = 1;
            report
                .events
                .push(format!("win32_view_tab_selected:{}:{}", tab_view.0, tab.0));
            self.dispatch_event(
                crate::ViewEvent::TabSelected {
                    widget: tab_view,
                    tab,
                },
                report,
            );
            return;
        }
        let toggles = matches!(
            target.kind,
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle
        );
        #[cfg(feature = "toggle-button")]
        let toggles = toggles || target.kind == crate::ViewHitTargetKind::ToggleButton;
        let event = if toggles {
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
            state.preferred_visual_column = None;
            state.first_visible_visual_row = native_text_first_visible_row_for_caret(
                target,
                &value,
                state.selection.caret,
                state.first_visible_visual_row,
                self.widget_text_wrap(widget),
                self.dpi,
            );
            state.first_visible_visual_column = native_text_first_visible_column_for_caret(
                target,
                &value,
                state.selection.caret,
                state.first_visible_visual_column,
                self.widget_text_wrap(widget),
                self.dpi,
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
                decorate_native_text_edit_visuals_in_viewport(
                    &mut plan,
                    target,
                    &value,
                    state.selection.clamp(&value),
                    state.first_visible_visual_row,
                    state.first_visible_visual_column,
                    self.widget_text_wrap(target.widget),
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
        WM_CLOSE => {
            if take_windows_win32_window_close_approval(hwnd) {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            } else if dispatch_windows_win32_window_close_requested(hwnd)
                .is_some_and(|report| report.handled && !report.quit_requested)
            {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
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
        WM_SETTINGCHANGE | WM_SYSCOLORCHANGE | WM_THEMECHANGED => {
            if let Some(plan) = window_draw_plan(hwnd) {
                apply_windows_win32_window_theme(hwnd, plan.theme_mode);
                InvalidateRect(hwnd, null(), 0);
                return 0;
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
            let shell_handled = dispatch_windows_win32_window_shell_pointer_leave(hwnd).is_some();
            let view_handled = dispatch_windows_win32_window_view_pointer_leave(hwnd)
                .is_some_and(|report| report.handled);
            if shell_handled || view_handled {
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
        WM_KEYDOWN => match dispatch_windows_win32_window_view_key_down_with_modifiers(
            hwnd,
            wparam as u32,
            (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0,
            (GetKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0,
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
        WM_PRINTCLIENT => paint_window_client_to_dc(hwnd, wparam as _),
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
    if !target.kind.accepts_text_input() {
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
        if let Some(buffered) = WindowsBufferedPaint::begin(target, &rect) {
            paint_window_client_rect_to_dc(hwnd, buffered.hdc(), rect);
        } else {
            paint_window_client_rect_to_dc(hwnd, target, rect);
        }
    }

    EndPaint(hwnd, &ps);
    0
}

unsafe fn paint_window_client_to_dc(
    hwnd: HWND,
    target: windows_sys::Win32::Graphics::Gdi::HDC,
) -> LRESULT {
    if target.is_null() {
        return 0;
    }
    let mut rect: RECT = zeroed();
    if GetClientRect(hwnd, &mut rect) != 0 {
        paint_window_client_rect_to_dc(hwnd, target, rect);
        GdiFlush();
    }
    0
}

unsafe fn paint_window_client_rect_to_dc(
    hwnd: HWND,
    target: windows_sys::Win32::Graphics::Gdi::HDC,
    rect: RECT,
) {
    let draw_plan = window_draw_plan(hwnd);
    let palette = windows_palette_for_draw_plan(draw_plan.as_ref());
    let high_contrast = resolved_windows_theme_mode(
        draw_plan
            .as_ref()
            .map(|plan| plan.theme_mode)
            .unwrap_or(crate::ZsuiThemeMode::System),
    ) == crate::ZsuiThemeMode::HighContrast;
    paint_win32_surface(target, rect, palette, high_contrast, draw_plan.as_ref());
}

fn windows_palette_for_draw_plan(draw_plan: Option<&NativeDrawPlan>) -> WindowsGdiPalette {
    match resolved_windows_theme_mode(
        draw_plan
            .map(|plan| plan.theme_mode)
            .unwrap_or(crate::ZsuiThemeMode::System),
    ) {
        crate::ZsuiThemeMode::HighContrast => windows_high_contrast_palette(),
        crate::ZsuiThemeMode::Dark => WindowsGdiPalette::from_theme(&crate::ZsuiTheme::dark()),
        _ => WindowsGdiPalette::default(),
    }
}

fn resolved_windows_theme_mode(theme_mode: crate::ZsuiThemeMode) -> crate::ZsuiThemeMode {
    resolved_windows_theme_mode_for_system(theme_mode, windows_system_theme_mode())
}

fn resolved_windows_theme_mode_for_system(
    theme_mode: crate::ZsuiThemeMode,
    system_mode: crate::ZsuiThemeMode,
) -> crate::ZsuiThemeMode {
    if system_mode == crate::ZsuiThemeMode::HighContrast {
        crate::ZsuiThemeMode::HighContrast
    } else if theme_mode == crate::ZsuiThemeMode::System {
        system_mode
    } else {
        theme_mode
    }
}

pub fn windows_system_theme_mode() -> crate::ZsuiThemeMode {
    if windows_system_high_contrast() {
        return crate::ZsuiThemeMode::HighContrast;
    }
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

pub fn windows_system_high_contrast() -> bool {
    let mut high_contrast = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        dwFlags: 0,
        lpszDefaultScheme: null_mut(),
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            high_contrast.cbSize,
            &mut high_contrast as *mut HIGHCONTRASTW as _,
            0,
        ) != 0
            && high_contrast.dwFlags & HCF_HIGHCONTRASTON != 0
    }
}

fn windows_high_contrast_palette() -> WindowsGdiPalette {
    let surface = windows_system_color(COLOR_WINDOW);
    let primary_text = windows_system_color(COLOR_WINDOWTEXT);
    WindowsGdiPalette {
        primary_text,
        secondary_text: primary_text,
        disabled_text: primary_text,
        accent: windows_system_color(COLOR_HIGHLIGHT),
        accent_text: windows_system_color(COLOR_HIGHLIGHTTEXT),
        surface,
        surface_raised: surface,
        control: surface,
        border: primary_text,
        success: primary_text,
        warning: primary_text,
        danger: primary_text,
    }
}

fn windows_system_color(index: i32) -> Color {
    let color = unsafe { GetSysColor(index) };
    Color::rgb(
        (color & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        ((color >> 16) & 0xff) as u8,
    )
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
    high_contrast: bool,
    draw_plan: Option<&NativeDrawPlan>,
) {
    let mut renderer = WindowsGdiRenderer::new(dc);
    renderer.fill_rect(rect_from_win(rect), palette.surface);
    drop(renderer);
    if let Some(plan) = draw_plan {
        let mut sink = WindowsGdiDrawSink::with_palette_and_contrast(dc, palette, high_contrast);
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
    fn high_contrast_system_mode_overrides_explicit_light_or_dark_preferences() {
        assert_eq!(
            resolved_windows_theme_mode_for_system(
                crate::ZsuiThemeMode::Light,
                crate::ZsuiThemeMode::HighContrast,
            ),
            crate::ZsuiThemeMode::HighContrast
        );
        assert_eq!(
            resolved_windows_theme_mode_for_system(
                crate::ZsuiThemeMode::System,
                crate::ZsuiThemeMode::Dark,
            ),
            crate::ZsuiThemeMode::Dark
        );
    }

    #[test]
    fn high_contrast_palette_uses_user_selected_system_color_pairs() {
        let palette = windows_high_contrast_palette();
        assert_eq!(palette.surface, windows_system_color(COLOR_WINDOW));
        assert_eq!(palette.primary_text, windows_system_color(COLOR_WINDOWTEXT));
        assert_eq!(palette.accent, windows_system_color(COLOR_HIGHLIGHT));
        assert_eq!(
            palette.accent_text,
            windows_system_color(COLOR_HIGHLIGHTTEXT)
        );
        assert_eq!(palette.border, palette.primary_text);
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
        assert_eq!(report.live_view_revision, 2);
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_live_view_repaint:1"));
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_live_view_app_effect_refresh:2"));
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

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn window_menu_command_updates_typed_live_view_and_repaints() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_view_input_routes();
        clear_windows_win32_window_menu_command_tables();
        let hwnd = 0x5454isize as HWND;

        #[derive(Clone)]
        enum Msg {
            Open,
        }
        struct State {
            status: &'static str,
        }

        let builder = crate::native_window("Menu State").stateful_view_with_app_commands(
            State { status: "Ready" },
            |state| crate::text::<Msg>(state.status),
            |state, message, _cx| match message {
                Msg::Open => state.status = "Opened from native menu",
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
        assert!(set_windows_win32_window_view_input_route(
            hwnd,
            WindowsWin32ViewInputRoute::from_live_view(runtime.clone()),
        ));
        let table = WindowsWin32StatusMenuCommandTable::from_menu(
            &MenuSpec::new().item("Open", Command::custom("document.open")),
        );
        let native_id = table
            .first_native_id()
            .expect("menu should allocate a native command id");
        set_windows_win32_window_menu_command_table(hwnd, table);

        assert!(matches!(
            dispatch_windows_win32_window_menu_command(hwnd, native_id),
            Some(NativeStatusMenuCommandResult::Dispatched(Command::Custom { id, .. }))
                if id == "document.open"
        ));
        assert_eq!(runtime.revision(), 1);
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Opened from native menu"
        )));
        assert!(window_draw_plan(hwnd).is_some());

        clear_windows_win32_window_menu_command_table(hwnd);
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
    #[cfg(all(feature = "tooltip", feature = "button"))]
    fn window_view_input_route_ticks_delayed_tooltip_into_buffered_draw_plan() {
        let widget = crate::WidgetId::new(1009);
        let mut view: crate::ViewNode<UiCommand> = crate::button("Save")
            .id(widget)
            .tooltip_spec(crate::ZsTooltipSpec::new("Save document").open_delay_ms(100));
        let surface = crate::Rect {
            x: 0,
            y: 0,
            width: 240,
            height: 120,
        };
        view.layout(&mut crate::ViewLayoutCx::new(
            surface,
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let target = interaction
            .tooltip_for_widget(widget)
            .expect("tooltip target should be collected");
        let point = crate::Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        };
        let start = std::time::Instant::now();
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        route.dispatch_pointer_move_at(point, start);
        let tick = route.refresh_background_view_at(start + std::time::Duration::from_millis(100));
        let draw = route
            .take_pending_draw_plan()
            .expect("tooltip tick should rebuild the buffered draw plan");

        assert!(tick.handled);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Save document"
        )));
        assert_eq!(route.hit_target_count(), 1);
    }

    #[test]
    #[cfg(feature = "password-box")]
    fn window_view_input_route_keeps_password_text_and_peek_plans_redacted() {
        fn password_changed(_: crate::ZsPassword) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.password_changed"))
        }

        let widget = crate::WidgetId::new(1010);
        let initial_secret = "A🙂";
        let mut view = crate::password_box(initial_secret)
            .id(widget)
            .height(crate::Dp::new(36.0))
            .reveal_mode(crate::ZsPasswordRevealMode::Peek)
            .on_password_change(password_changed);
        let mut layout = crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 220,
                height: 36,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let interaction = view.interaction_plan();
        let input = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBox)
            .expect("password box should expose a Win32 input target");
        let reveal = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBoxReveal)
            .expect("password box should expose a Win32 reveal target");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);
        route.dispatch_click(crate::Point {
            x: input.bounds.x + 2,
            y: input.bounds.y + input.bounds.height / 2,
        });

        let typed = route.dispatch_text_input("中");
        let current_secret = "A🙂中";
        assert_eq!(typed.text_input_count, 1);
        assert_eq!(
            typed.ui_command_ids,
            vec!["zsui.test.win32.password_changed"]
        );
        assert_eq!(
            route
                .widget_password_value(widget)
                .map(|value| value.as_str().to_owned())
                .as_deref(),
            Some(current_secret)
        );
        assert!(!format!("{typed:?}").contains(current_secret));
        let _ = route.take_pending_draw_plan();

        let reveal_point = crate::Point {
            x: reveal.bounds.x + reveal.bounds.width / 2,
            y: reveal.bounds.y + reveal.bounds.height / 2,
        };
        let pressed = route.dispatch_pointer_down(reveal_point, false);
        let pressed_plan = route
            .take_pending_draw_plan()
            .expect("Win32 reveal press should rebuild the draw plan");
        assert!(pressed.handled);
        assert!(pressed_plan.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::SecureText(command) if command.character_count() == 3
        )));
        assert!(!format!("{pressed_plan:?}").contains(current_secret));
        assert!(!serde_json::to_string(&pressed_plan)
            .expect("Win32 peek plan should serialize redacted")
            .contains(current_secret));

        let released = route.dispatch_pointer_up(reveal_point);
        let released_plan = route
            .take_pending_draw_plan()
            .expect("Win32 reveal release should restore the mask");
        assert!(released.handled);
        assert!(released_plan.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "•••"
        )));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_replaces_unicode_keyboard_selection() {
        fn selection_changed(_: crate::ZsTextSelection) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.text_selection"))
        }

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
            crate::textbox("A中文Z")
                .id(widget)
                .on_text_selection_change(selection_changed),
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
        assert_eq!(
            selected.ui_command_ids,
            vec!["zsui.test.win32.text_selection"]
        );
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
        assert_eq!(
            replaced.ui_command_ids,
            vec!["zsui.test.win32.text_selection"]
        );
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A🙂Z"));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_moves_wrapped_editor_caret_by_visual_row() {
        #[derive(Clone)]
        enum Msg {
            Selection(crate::ZsTextSelection),
        }

        let widget = crate::WidgetId::new(320);
        let builder = crate::native_window("Win32 wrapped navigation")
            .size(48, 140)
            .stateful_view(
                crate::ZsTextSelection::default(),
                move |_selection| {
                    crate::text_editor("abcdef\nx\nuvwxyz")
                        .id(widget)
                        .width(crate::Dp::new(48.0))
                        .height(crate::Dp::new(120.0))
                        .on_text_selection_change(Msg::Selection)
                },
                |selection, message, _cx| match message {
                    Msg::Selection(next) => *selection = next,
                },
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("wrapped editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("wrapped editor should expose Win32 geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        let point = crate::Point {
            x: target.bounds.x + 24,
            y: target.bounds.y + 10,
        };
        route.dispatch_pointer_down(point, false);
        route.dispatch_pointer_up(point);

        let second_visual_row = route.dispatch_key_down(u32::from(VK_DOWN));
        let short_hard_line = route.dispatch_key_down(u32::from(VK_DOWN));
        let next_wrapped_line = route.dispatch_key_down(u32::from(VK_DOWN));
        let extended = route.dispatch_key_down_with_shift(u32::from(VK_UP), true);

        assert_eq!(second_visual_row.text_caret, Some(6));
        assert_eq!(short_hard_line.text_caret, Some(8));
        assert_eq!(next_wrapped_line.text_caret, Some(11));
        assert_eq!(extended.text_caret, Some(8));
        assert_eq!(extended.text_selection_change_count, 1);
        assert_eq!(extended.message_count, 1);
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_scrolls_editor_and_reveals_keyboard_caret() {
        let widget = crate::WidgetId::new(321);
        let value = "row0\nrow1\nrow2\nrow3\nrow4\nrow5";
        let builder = crate::native_window("Win32 editor viewport")
            .size(160, 80)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>(value)
                        .id(widget)
                        .width(crate::Dp::new(120.0))
                        .height(crate::Dp::new(52.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("editor should expose Win32 viewport geometry");
        let point = crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(point, false);
        route.dispatch_pointer_up(point);

        let scrolled = route.dispatch_scroll(point, crate::Dp::new(48.0));
        let scrolled_plan = route
            .take_pending_draw_plan()
            .expect("editor scroll should rebuild the Win32 draw plan");
        route.dispatch_key_down(u32::from(VK_RIGHT));
        let revealed_plan = route
            .take_pending_draw_plan()
            .expect("keyboard navigation should reveal the caret row");

        assert!(scrolled.handled);
        assert_eq!(scrolled.scroll_count, 1);
        assert_eq!(scrolled.unhandled_scroll_count, 0);
        assert!(scrolled_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row3"),
        ));
        assert!(!scrolled_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row0"),
        ));
        assert!(revealed_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row0"),
        ));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_reveals_no_wrap_columns_for_pointer_hits() {
        let widget = crate::WidgetId::new(322);
        let builder = crate::native_window("Win32 horizontal editor viewport")
            .size(48, 70)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>("0123456789")
                        .id(widget)
                        .text_wrap(crate::TextWrap::NoWrap)
                        .width(crate::Dp::new(48.0))
                        .height(crate::Dp::new(52.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("no-wrap editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("no-wrap editor should expose Win32 viewport geometry");
        let left = crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(left, false);
        route.dispatch_pointer_up(left);

        let revealed = route.dispatch_key_down(u32::from(VK_END));
        let revealed_plan = route
            .take_pending_draw_plan()
            .expect("End should reveal the no-wrap caret column");
        let clicked = route.dispatch_pointer_down(left, false);

        assert_eq!(revealed.text_caret, Some(10));
        assert!(revealed_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "789"),
        ));
        assert_eq!(clicked.text_caret, Some(7));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_pages_editor_by_visible_rows() {
        let widget = crate::WidgetId::new(323);
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let builder = crate::native_window("Win32 paged editor viewport")
            .size(160, 70)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>(value)
                        .id(widget)
                        .text_wrap(crate::TextWrap::NoWrap)
                        .width(crate::Dp::new(160.0))
                        .height(crate::Dp::new(70.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("paged editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("paged editor should expose Win32 viewport geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(
            crate::Point {
                x: target.bounds.x + 16,
                y: target.bounds.y + 10,
            },
            false,
        );
        route.dispatch_pointer_up(crate::Point {
            x: target.bounds.x + 16,
            y: target.bounds.y + 10,
        });

        let page_down = route.dispatch_key_down(u32::from(VK_NEXT));
        let page_plan = route
            .take_pending_draw_plan()
            .expect("PageDown should rebuild the paged editor viewport");
        let shift_page_down = route.dispatch_key_down_with_shift(u32::from(VK_NEXT), true);
        let page_up = route.dispatch_key_down(u32::from(VK_PRIOR));

        assert_eq!(page_down.text_caret, Some(10));
        assert!(page_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "d3"),
        ));
        assert_eq!(shift_page_down.text_caret, Some(19));
        assert_eq!(page_up.text_caret, Some(10));
        assert_eq!(
            route.text_edit.map(|state| state.selection),
            Some(crate::native_text_edit::NativeTextSelection::collapsed(10))
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_live_view_routes_typed_undo_command_to_focused_editor() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
            Undo,
        }

        let widget = crate::WidgetId::new(34);
        let builder = crate::native_window("Win32 typed editor command")
            .size(320, 160)
            .stateful_view_with_app_commands(
                String::new(),
                move |value| crate::text_editor(value).id(widget).on_change(Msg::Changed),
                |value, message, cx| match message {
                    Msg::Changed(next) => *value = next,
                    Msg::Undo => cx.text_edit_command(crate::ZsTextEditCommand::Undo),
                },
                |command| (command == &Command::custom("edit.undo")).then_some(Msg::Undo),
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("editor should expose Win32 focus geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_click(crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 8,
        });
        route.dispatch_text_input("A");
        route.dispatch_text_input("中");

        let undone = route.dispatch_app_command(Command::custom("edit.undo"));

        assert_eq!(undone.text_edit_command_count, 1);
        assert_eq!(undone.text_undo_count, 1);
        assert!(undone.text_edit_command_errors.is_empty());
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A"));
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
    #[cfg(feature = "color-picker")]
    fn window_view_input_route_drags_and_keys_color_picker_channels() {
        fn color_changed(_: crate::Color) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_changed"))
        }
        fn channel_changed(_: crate::ZsColorChannel) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_channel"))
        }
        fn expanded_changed(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_expanded"))
        }

        let widget = crate::WidgetId::new(341);
        let viewport = crate::Rect {
            x: 0,
            y: 0,
            width: 480,
            height: 680,
        };
        let state = crate::ZsColorPickerState::new(crate::Color::rgba(32, 96, 160, 224))
            .with_expanded(true);
        let mut view = crate::column([
            crate::color_picker(state)
                .id(widget)
                .height(crate::Dp::new(32.0))
                .on_color_change(color_changed)
                .on_color_channel_change(channel_changed)
                .on_expanded_change(expanded_changed),
            crate::spacer(),
        ])
        .padding(crate::Dp::new(24.0))
        .gap(crate::Dp::new(12.0));
        view.layout(&mut crate::ViewLayoutCx::new(
            viewport,
            crate::Dpi::standard(),
        ));
        let plan = view.interaction_plan();
        let root = plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.widget == widget && target.kind == crate::ViewHitTargetKind::ColorPicker
            })
            .expect("color picker root");
        let render = crate::zs_color_picker_render_plan_in_viewport(
            root.bounds,
            state,
            crate::ZsColorPickerPlatformStyle::Windows,
            crate::Dpi::standard(),
            viewport,
        );
        let red = render
            .channels
            .iter()
            .find(|row| row.channel == crate::ZsColorChannel::Red)
            .expect("red row");
        let mut route = WindowsWin32ViewInputRoute::new(plan, view);

        let pressed = route.dispatch_pointer_down(
            crate::Point {
                x: red.track.x + red.track.width / 4,
                y: red.track.y + red.track.height / 2,
            },
            false,
        );
        let dragged = route.dispatch_pointer_move(crate::Point {
            x: red.track.x + red.track.width * 9 / 10,
            y: red.track.y + red.track.height / 2,
        });
        let released = route.dispatch_pointer_up(crate::Point {
            x: red.track.x + red.track.width * 9 / 10,
            y: red.track.y + red.track.height / 2,
        });

        assert!(pressed.handled);
        assert!(pressed.color_picker_drag_active);
        assert_eq!(pressed.color_picker_value_change_count, 1);
        assert!(dragged.handled);
        assert!(dragged.color_picker_drag_active);
        assert_eq!(dragged.color_picker_value_change_count, 1);
        assert_eq!(released.color_picker_drag_count, 1);
        assert!(!released.color_picker_drag_active);
        assert!(route
            .widget_color_picker_state(widget)
            .is_some_and(|state| state.color.r > 220));

        let channel = route.dispatch_key_down(u32::from(VK_DOWN));
        let maximum = route.dispatch_key_down(u32::from(VK_END));
        let closed = route.dispatch_key_down(u32::from(VK_ESCAPE));
        let reopened = route.dispatch_key_down(ZSUI_WIN32_VK_SPACE);

        assert_eq!(channel.color_picker_channel_change_count, 1);
        assert_eq!(maximum.color_picker_value_change_count, 1);
        assert_eq!(closed.color_picker_expanded_change_count, 1);
        assert_eq!(reopened.color_picker_expanded_change_count, 1);
        assert!(route
            .widget_color_picker_state(widget)
            .is_some_and(|state| {
                state.active_channel == crate::ZsColorChannel::Green
                    && state.color.g == 255
                    && state.expanded
            }));
        assert_eq!(route.pending_ui_commands.len(), 6);
    }

    #[test]
    #[cfg(feature = "number-box")]
    fn window_view_input_route_edits_commits_and_steps_number_box() {
        fn changed(_: Option<f64>) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.number_box_changed"))
        }

        let widget = crate::WidgetId::new(340);
        let bounds = crate::Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 36,
        };
        let render = crate::zs_number_box_render_plan(
            bounds,
            crate::ZsNumberBoxPlatformStyle::Windows,
            crate::Dpi::standard(),
        );
        let plan = crate::ViewInteractionPlan::new([
            crate::ViewHitTarget::with_kind(widget, bounds, crate::ViewHitTargetKind::NumberBox),
            crate::ViewHitTarget::with_kind(
                widget,
                render.decrement_button,
                crate::ViewHitTargetKind::NumberBoxDecrement,
            ),
            crate::ViewHitTarget::with_kind(
                widget,
                render.increment_button,
                crate::ViewHitTargetKind::NumberBoxIncrement,
            ),
        ]);
        let range = crate::ZsNumberRange::new(0.0, 10.0)
            .step(0.5)
            .large_step(5.0);
        let mut route = WindowsWin32ViewInputRoute::new(
            plan,
            crate::number_box(Some(2.5), range)
                .id(widget)
                .fraction_digits(1)
                .on_number_change(changed),
        );

        let incremented = route.dispatch_click(crate::Point {
            x: render.increment_button.x + render.increment_button.width / 2,
            y: render.increment_button.y + render.increment_button.height / 2,
        });
        let stepped = route.dispatch_key_down(u32::from(VK_UP));
        let cleared = route.dispatch_text_input("\u{8}\u{8}\u{8}");
        let typed = route.dispatch_text_input("9.5");
        let committed = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);

        assert!(incremented.handled);
        assert!(stepped.handled);
        assert_eq!(cleared.text_input_count, 3);
        assert_eq!(typed.text_input_count, 3);
        assert!(committed.handled);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("9.5"));
        assert_eq!(route.pending_ui_commands.len(), 3);
    }

    #[test]
    #[cfg(feature = "radio")]
    fn window_view_input_route_selects_radio_from_pointer_and_space() {
        let first = crate::WidgetId::new(35);
        let second = crate::WidgetId::new(36);
        let selected = UiCommand::app(crate::CommandId("zsui.test.win32.radio_selected"));
        let mut view = crate::column([
            crate::radio_button("Balanced", true)
                .id(first)
                .height(crate::Dp::new(36.0))
                .on_choose(selected.clone()),
            crate::radio_button("Performance", false)
                .id(second)
                .height(crate::Dp::new(36.0))
                .on_choose(selected),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 72,
            },
            crate::Dpi::standard(),
        ));
        let interaction_plan = view.interaction_plan();
        let second_bounds = interaction_plan
            .hit_target_for_widget(second)
            .expect("second radio should have hit geometry")
            .bounds;
        let mut route = WindowsWin32ViewInputRoute::new(interaction_plan, view);

        let pointer = route.dispatch_click(crate::Point {
            x: second_bounds.x + 10,
            y: second_bounds.y + second_bounds.height / 2,
        });
        let keyboard = route.dispatch_key_down(u32::from(VK_SPACE));
        let arrow = route.dispatch_key_down(u32::from(VK_UP));
        let focus_only = route.dispatch_key_down_with_modifiers(u32::from(VK_DOWN), false, true);
        let tabbed = route.dispatch_key_down(u32::from(VK_TAB));

        assert_eq!(pointer.radio_selection_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(keyboard.radio_selection_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
        assert_eq!(arrow.radio_selection_count, 1);
        assert_eq!(arrow.radio_keyboard_selection_count, 1);
        assert_eq!(arrow.focused_widget, Some(first.0));
        assert!(arrow
            .events
            .iter()
            .any(|event| event == "win32_view_radio_key_select:36:35"));
        assert_eq!(focus_only.radio_keyboard_focus_only_count, 1);
        assert_eq!(focus_only.radio_selection_count, 0);
        assert_eq!(focus_only.focused_widget, Some(second.0));
        assert!(focus_only
            .events
            .iter()
            .any(|event| event == "win32_view_radio_key_focus_only:35:36"));
        assert_eq!(tabbed.focus_traversal_count, 1);
        assert_eq!(tabbed.focused_widget, Some(first.0));
        assert_eq!(route.widget_checked_value(first), Some(true));
        assert_eq!(route.widget_checked_value(second), Some(false));
    }

    #[test]
    #[cfg(all(feature = "tabs", feature = "label"))]
    fn window_view_input_route_routes_tab_pointer_focus_activation_and_ctrl_tab() {
        fn selected(_: crate::ZsTabId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.tabs.selected"))
        }

        let tab_view_id = crate::WidgetId::new(340);
        let general = crate::ZsTabId::new(341);
        let advanced = crate::ZsTabId::new(342);
        let mut view = crate::tab_view(
            [
                crate::ZsTabItem::new(general, "General", crate::text("General content")),
                crate::ZsTabItem::new(advanced, "Advanced", crate::text("Advanced content")),
            ],
            Some(general),
        )
        .id(tab_view_id)
        .on_tab_select(selected);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let second = interaction
            .hit_target_for_widget(crate::WidgetId(advanced.0))
            .expect("second tab should expose a hit target");
        let second_point = crate::Point {
            x: second.bounds.x + second.bounds.width / 2,
            y: second.bounds.y + second.bounds.height / 2,
        };
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let hovered = route.dispatch_pointer_move(second_point);
        let pressed = route.dispatch_pointer_down(second_point, false);
        let pointer = route.dispatch_pointer_up(second_point);

        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert_eq!(pressed.pointer_visual_change_count, 1);
        assert_eq!(pointer.tab_selection_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(pointer.focused_widget, Some(advanced.0));
        assert_eq!(
            route
                .ui_command_view
                .as_ref()
                .and_then(|view| view.widget_tab_view_state(tab_view_id))
                .and_then(|state| state.selected),
            Some(advanced)
        );

        let focus_only = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(focus_only.tab_keyboard_focus_only_count, 1);
        assert_eq!(focus_only.tab_selection_count, 0);
        assert_eq!(focus_only.focused_widget, Some(general.0));

        let keyboard = route.dispatch_key_down(u32::from(VK_SPACE));
        assert_eq!(keyboard.tab_selection_count, 1);
        assert_eq!(keyboard.tab_keyboard_selection_count, 1);
        assert_eq!(
            route
                .ui_command_view
                .as_ref()
                .and_then(|view| view.widget_tab_view_state(tab_view_id))
                .and_then(|state| state.selected),
            Some(general)
        );

        let cycled = route.dispatch_key_down_with_modifiers(u32::from(VK_TAB), false, true);
        assert_eq!(cycled.tab_selection_count, 1);
        assert_eq!(cycled.tab_keyboard_selection_count, 1);
        assert_eq!(cycled.focused_widget, Some(advanced.0));
        assert!(route.take_pending_draw_plan().is_some());
    }

    #[test]
    #[cfg(feature = "auto-suggest")]
    fn window_view_input_route_closes_auto_suggest_with_pointer_and_keyboard_submission() {
        fn text(_change: crate::ZsAutoSuggestTextChange) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_text"))
        }
        fn chosen(_suggestion: crate::ZsAutoSuggestionId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_chosen"))
        }
        fn submitted(_submission: crate::ZsAutoSuggestSubmission) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_submitted"))
        }
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_expanded"))
        }

        let widget = crate::WidgetId::new(136);
        let beta = crate::ZsAutoSuggestionId::new(2);
        let mut view = crate::column([
            crate::auto_suggest_box(
                "B",
                [
                    crate::ZsAutoSuggestion::new(1_u64, "Alpha"),
                    crate::ZsAutoSuggestion::new(beta, "Beta"),
                    crate::ZsAutoSuggestion::new(3_u64, "Bravo"),
                ],
            )
            .id(widget)
            .expanded(true)
            .on_auto_suggest_text_change(text)
            .on_suggestion_chosen(chosen)
            .on_query_submit(submitted)
            .on_expanded_change(expanded),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let suggestion = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind == crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion: beta }
            })
            .expect("expanded auto-suggest should expose Beta row");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let pointer = route.dispatch_click(crate::Point {
            x: suggestion.bounds.x + 8,
            y: suggestion.bounds.y + suggestion.bounds.height / 2,
        });
        assert_eq!(pointer.auto_suggest_submit_count, 1);
        assert_eq!(pointer.auto_suggest_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 4);
        assert!(route
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.query == "Beta" && !state.expanded));

        let typed = route.dispatch_text_input("x");
        assert_eq!(typed.text_input_count, 1);
        assert_eq!(typed.auto_suggest_expanded_change_count, 1);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("Betax"));
        let highlighted = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(highlighted.auto_suggest_highlight_change_count, 1);
        assert_eq!(
            route
                .widget_auto_suggest_state(widget)
                .and_then(|state| state.highlighted),
            Some(1_u64.into())
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.auto_suggest_submit_count, 1);
        assert_eq!(keyboard.auto_suggest_expanded_change_count, 1);
        assert!(route
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.query == "Alpha" && !state.expanded));

        let clear = route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::AutoSuggestClear)
            .expect("non-empty query should expose clear button");
        let cleared = route.dispatch_click(crate::Point {
            x: clear.bounds.x + clear.bounds.width / 2,
            y: clear.bounds.y + clear.bounds.height / 2,
        });
        assert_eq!(cleared.auto_suggest_clear_count, 1);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some(""));
    }

    #[test]
    #[cfg(feature = "command-palette")]
    fn window_view_input_route_filters_navigates_and_invokes_command_palette() {
        fn query(_query: String) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_query"))
        }
        fn highlight(_item: crate::ZsCommandPaletteItemId) -> UiCommand {
            UiCommand::app(crate::CommandId(
                "zsui.test.win32.command_palette_highlight",
            ))
        }
        fn invoke(_item: crate::ZsCommandPaletteItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_invoke"))
        }
        fn open(_open: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_open"))
        }

        let widget = crate::WidgetId::new(341);
        let first = crate::ZsCommandPaletteItemId::new(1);
        let settings = crate::ZsCommandPaletteItemId::new(2);
        let mut view = crate::command_palette(
            widget,
            true,
            "",
            [
                crate::ZsCommandPaletteItem::new(first, "Open file"),
                crate::ZsCommandPaletteItem::new(settings, "Open settings")
                    .keywords(["preferences"]),
                crate::ZsCommandPaletteItem::new(3_u64, "Unavailable").enabled(false),
            ],
            crate::spacer(),
        )
        .highlighted_command(Some(first))
        .on_command_palette_query_change(query)
        .on_command_palette_highlight_change(highlight)
        .on_command_palette_invoke(invoke)
        .on_command_palette_open_change(open);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 900,
                height: 620,
            },
            crate::Dpi::standard(),
        ));
        let mut route = WindowsWin32ViewInputRoute::new(view.interaction_plan(), view);

        let moved = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(moved.command_palette_highlight_change_count, 1);
        assert_eq!(
            route
                .widget_command_palette_state(widget)
                .and_then(|state| state.highlighted),
            Some(settings)
        );

        let typed = route.dispatch_text_input("settings");
        assert_eq!(typed.command_palette_query_change_count, 1);
        assert!(route
            .widget_command_palette_state(widget)
            .is_some_and(
                |state| state.query == "settings" && state.visible_items == vec![settings]
            ));

        let invoked = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(invoked.command_palette_invoke_count, 1);
        assert_eq!(invoked.command_palette_open_change_count, 1);
        assert!(route
            .widget_command_palette_state(widget)
            .is_some_and(|state| !state.open));
        assert!(invoked
            .events
            .iter()
            .any(|event| event == "win32_view_command_palette_invoke:341:2"));
    }

    #[test]
    #[cfg(feature = "tree")]
    fn window_view_input_route_handles_tree_disclosure_rows_and_keyboard_hierarchy() {
        fn selected(_node: crate::ZsTreeNodeId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_selected"))
        }
        fn expanded(_change: crate::ZsTreeExpansionChange) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_expanded"))
        }
        fn invoked(_node: crate::ZsTreeNodeId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_invoked"))
        }

        let widget = crate::WidgetId::new(137);
        let root = crate::ZsTreeNodeId::new(1);
        let folder = crate::ZsTreeNodeId::new(2);
        let leaf = crate::ZsTreeNodeId::new(3);
        let mut view = crate::tree_view([crate::ZsTreeNode::new(root, "Workspace")
            .icon(crate::ZsIcon::Folder)
            .children([
                crate::ZsTreeNode::new(folder, "src")
                    .icon(crate::ZsIcon::Folder)
                    .children([crate::ZsTreeNode::new(leaf, "lib.rs")]),
                crate::ZsTreeNode::new(4, "Cargo.toml"),
            ])])
        .id(widget)
        .expanded_tree_nodes([root])
        .selected_tree_node(Some(folder))
        .on_tree_select(selected)
        .on_tree_expansion_change(expanded)
        .on_tree_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let disclosure = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind == crate::ViewHitTargetKind::TreeNodeExpander { node: folder }
            })
            .expect("folder should expose a disclosure target");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let opened = route.dispatch_click(crate::Point {
            x: disclosure.bounds.x + disclosure.bounds.width / 2,
            y: disclosure.bounds.y + disclosure.bounds.height / 2,
        });
        assert_eq!(opened.tree_expansion_change_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        let leaf_row = route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TreeNode { node: leaf })
            .expect("expanded folder should expose leaf row");
        let pointer = route.dispatch_click(crate::Point {
            x: leaf_row.bounds.x + leaf_row.bounds.width / 2,
            y: leaf_row.bounds.y + leaf_row.bounds.height / 2,
        });
        assert_eq!(pointer.tree_selection_count, 1);
        assert_eq!(pointer.tree_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let parent = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(parent.tree_selection_count, 1);
        assert_eq!(
            route
                .widget_tree_view_state(widget)
                .and_then(|state| state.selected),
            Some(folder)
        );
        let collapsed = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(collapsed.tree_expansion_change_count, 1);
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.tree_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "grid-view")]
    fn window_view_input_route_handles_grid_view_tiles_and_two_axis_keyboard_navigation() {
        fn selected(_item: crate::ZsGridViewItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.grid_view_selected"))
        }
        fn invoked(_item: crate::ZsGridViewItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.grid_view_invoked"))
        }

        let widget = crate::WidgetId::new(151);
        let first = crate::ZsGridViewItemId::new(1);
        let fifth = crate::ZsGridViewItemId::new(5);
        let mut view = crate::grid_view([
            crate::ZsGridViewItem::new(1, "One"),
            crate::ZsGridViewItem::new(2, "Two"),
            crate::ZsGridViewItem::new(3, "Three"),
            crate::ZsGridViewItem::new(4, "Four"),
            crate::ZsGridViewItem::new(5, "Five"),
            crate::ZsGridViewItem::new(6, "Six"),
        ])
        .id(widget)
        .selected_grid_view_item(Some(first))
        .on_grid_view_select(selected)
        .on_grid_view_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let fifth_tile = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::GridViewItem { item: fifth })
            .expect("fifth grid-view tile");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let pointer = route.dispatch_click(crate::Point {
            x: fifth_tile.bounds.x + fifth_tile.bounds.width / 2,
            y: fifth_tile.bounds.y + fifth_tile.bounds.height / 2,
        });
        assert_eq!(pointer.grid_view_selection_count, 1);
        assert_eq!(pointer.grid_view_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_HOME))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_RIGHT))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_DOWN))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .widget_grid_view_state(widget)
                .and_then(|state| state.selected),
            Some(fifth)
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.grid_view_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "table")]
    fn window_view_input_route_handles_table_sort_rows_and_keyboard_navigation() {
        fn selected(_row: crate::ZsTableRowId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_selected"))
        }
        fn sorted(_sort: crate::ZsTableSort) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_sorted"))
        }
        fn invoked(_row: crate::ZsTableRowId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_invoked"))
        }

        let widget = crate::WidgetId::new(138);
        let name = crate::ZsTableColumnId::new(1);
        let first = crate::ZsTableRowId::new(10);
        let second = crate::ZsTableRowId::new(11);
        let mut view = crate::data_grid(
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
        .selected_table_row(Some(first))
        .on_table_select(selected)
        .on_table_sort(sorted)
        .on_table_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let header = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableHeader { column: name })
            .expect("sortable table header");
        let second_row = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableRow { row: second })
            .expect("second table row");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let sort = route.dispatch_click(crate::Point {
            x: header.bounds.x + header.bounds.width / 2,
            y: header.bounds.y + header.bounds.height / 2,
        });
        assert_eq!(sort.table_sort_count, 1);
        assert_eq!(sort.ui_command_count, 1);
        assert_eq!(
            route
                .widget_table_state(widget)
                .and_then(|state| state.sort),
            Some(crate::ZsTableSort::new(
                name,
                crate::ZsTableSortDirection::Ascending
            ))
        );

        let pointer = route.dispatch_click(crate::Point {
            x: second_row.bounds.x + second_row.bounds.width / 2,
            y: second_row.bounds.y + second_row.bounds.height / 2,
        });
        assert_eq!(pointer.table_selection_count, 1);
        assert_eq!(pointer.table_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let moved = route.dispatch_key_down(u32::from(VK_UP));
        assert_eq!(moved.table_selection_count, 1);
        assert_eq!(
            route
                .widget_table_state(widget)
                .and_then(|state| state.selected),
            Some(first)
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.table_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "dialog")]
    fn window_view_input_route_traps_modal_focus_and_routes_dialog_buttons() {
        fn responded(_result: crate::ZsContentDialogResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.dialog_responded"))
        }

        let widget = crate::WidgetId::new(139);
        let spec = crate::ZsContentDialogSpec::new("Choose a response.", "Cancel")
            .title("Continue?")
            .primary_button("Continue")
            .secondary_button("Review")
            .default_button(crate::ZsContentDialogButton::Primary);
        let mut view =
            crate::content_dialog(widget, true, spec, crate::spacer()).on_dialog_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 400,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let primary = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind
                    == crate::ViewHitTargetKind::ContentDialogButton {
                        button: crate::ZsContentDialogButton::Primary,
                    }
            })
            .expect("primary dialog button");
        let scrim = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialogScrim)
            .expect("dialog scrim");

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let caught = keyboard_route.dispatch_click(crate::Point {
            x: scrim.bounds.x + 2,
            y: scrim.bounds.y + 2,
        });
        assert!(caught.handled);
        assert_eq!(caught.content_dialog_response_count, 0);
        let suppressed = keyboard_route.dispatch_text_input("x");
        assert!(suppressed.handled);
        assert_eq!(suppressed.ui_command_count, 0);
        let focused = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focused.content_dialog_focus_change_count, 1);
        assert_eq!(focused.focused_widget, Some(widget.0));
        assert_eq!(
            keyboard_route
                .widget_content_dialog_state(widget)
                .map(|(state, _)| state.focused_button),
            Some(crate::ZsContentDialogButton::Secondary)
        );
        let keyboard = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.content_dialog_response_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert!(keyboard_route
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| !state.open));

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: primary.bounds.x + primary.bounds.width / 2,
            y: primary.bounds.y + primary.bounds.height / 2,
        });
        assert_eq!(pointer.content_dialog_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "toast")]
    fn window_view_input_route_routes_toast_action_and_owned_timeout() {
        fn responded(_result: crate::ZsToastResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.toast_responded"))
        }

        let widget = crate::WidgetId::new(149);
        let mut view = crate::toast_presenter(
            widget,
            Some(crate::ZsToastSpec::new(51, "File deleted").action("Undo")),
            crate::spacer(),
        )
        .on_toast_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 400,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ToastAction)
            .expect("toast action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.toast_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert!(pointer_route.widget_toast_state(widget).is_none());

        let start = std::time::Instant::now();
        let mut timeout_route = WindowsWin32ViewInputRoute::new(interaction, view);
        assert!(timeout_route.background_poll_interval_ms().is_some());
        let timeout =
            timeout_route.refresh_background_view_at(start + std::time::Duration::from_secs(6));
        assert_eq!(timeout.toast_response_count, 1);
        assert_eq!(timeout.toast_timeout_count, 1);
        assert_eq!(timeout.ui_command_count, 1);
        assert!(timeout_route.widget_toast_state(widget).is_none());
    }

    #[test]
    #[cfg(feature = "info-bar")]
    fn window_view_input_route_routes_info_bar_action_and_keyboard_close() {
        fn invoked(event: crate::ZsInfoBarEvent) -> UiCommand {
            UiCommand::app(crate::CommandId(match event {
                crate::ZsInfoBarEvent::Action => "zsui.test.win32.info_bar_action",
                crate::ZsInfoBarEvent::Close => "zsui.test.win32.info_bar_close",
            }))
        }

        let widget = crate::WidgetId::new(150);
        let mut view = crate::column([
            crate::info_bar(
                widget,
                crate::ZsInfoBarSpec::new("Renew to keep all functionality.")
                    .title("Subscription expires soon")
                    .severity(crate::ZsInfoBarSeverity::Warning)
                    .action("Renew"),
            )
            .on_info_bar_event(invoked),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 240,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::InfoBarAction)
            .expect("info-bar action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.info_bar_event_count, 1);
        assert_eq!(pointer.ui_command_count, 1);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let next = keyboard_route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(next.info_bar_focus_change_count, 1);
        assert_eq!(
            keyboard_route
                .widget_info_bar_state(widget)
                .map(|(state, _)| state.focused_control),
            Some(Some(crate::ZsInfoBarControl::Close))
        );
        let close = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(close.info_bar_event_count, 1);
        assert_eq!(close.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "teaching-tip")]
    fn window_view_input_route_routes_teaching_tip_action_and_keyboard_close() {
        fn responded(_result: crate::ZsTeachingTipResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.teaching_tip_responded"))
        }

        let widget = crate::WidgetId::new(151);
        let target = crate::WidgetId::new(152);
        let mut view = crate::teaching_tip(
            widget,
            true,
            target,
            crate::ZsTeachingTipSpec::new(
                "Save automatically",
                "Your changes are saved as you work.",
            )
            .action("Review settings"),
            crate::spacer().id(target),
        )
        .on_teaching_tip_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 420,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTipAction)
            .expect("teaching-tip action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.teaching_tip_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(target.0));
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let next = keyboard_route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(next.teaching_tip_focus_change_count, 1);
        assert_eq!(
            keyboard_route
                .widget_teaching_tip_state(widget)
                .map(|(state, _)| state.focused_control),
            Some(crate::ZsTeachingTipControl::Close)
        );
        let close = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(close.teaching_tip_response_count, 1);
        assert_eq!(close.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "breadcrumb")]
    fn window_view_input_route_routes_breadcrumb_overflow_focus_and_selection() {
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.breadcrumb_expanded"))
        }
        fn selected(_item: crate::ZsBreadcrumbId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.breadcrumb_selected"))
        }

        let widget = crate::WidgetId::new(153);
        let mut view = crate::breadcrumb_bar([
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(1), "Home"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(2), "Projects"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(3), "ZSUI Framework"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(4), "Documentation"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(5), "BreadcrumbBar"),
        ])
        .id(widget)
        .width(crate::Dp::new(240.0))
        .expanded(false)
        .on_expanded_change(expanded)
        .on_breadcrumb_select(selected);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let overflow = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::BreadcrumbOverflow)
            .expect("narrow breadcrumb overflow");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let open = pointer_route.dispatch_click(crate::Point {
            x: overflow.bounds.x + overflow.bounds.width / 2,
            y: overflow.bounds.y + overflow.bounds.height / 2,
        });
        assert_eq!(open.breadcrumb_expanded_change_count, 1);
        assert_eq!(open.ui_command_count, 1);
        let row = pointer_route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                matches!(
                    target.kind,
                    crate::ViewHitTargetKind::BreadcrumbOverflowItem { .. }
                )
            })
            .expect("open overflow row");
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: row.bounds.x + row.bounds.width / 2,
            y: row.bounds.y + row.bounds.height / 2,
        });
        assert_eq!(pointer.breadcrumb_selection_count, 1);
        assert_eq!(pointer.breadcrumb_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let home = keyboard_route.dispatch_key_down(u32::from(VK_HOME));
        assert_eq!(home.breadcrumb_focus_change_count, 1);
        let open = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(open.breadcrumb_expanded_change_count, 1);
        let down = keyboard_route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(down.breadcrumb_focus_change_count, 1);
        let selection = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(selection.breadcrumb_selection_count, 1);
        assert_eq!(selection.breadcrumb_expanded_change_count, 1);
        assert_eq!(selection.ui_command_count, 2);
    }

    #[test]
    #[cfg(feature = "combo")]
    fn window_view_input_route_selects_combo_overlay_and_keyboard_option() {
        fn selected(_index: usize) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.combo_selected"))
        }
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.combo_expanded"))
        }

        let widget = crate::WidgetId::new(36);
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 36,
            },
            crate::ViewHitTargetKind::ComboBox,
        );
        let option = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 76,
                width: 200,
                height: 36,
            },
            crate::ViewHitTargetKind::ComboBoxOption { index: 1 },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, option]),
            crate::combo_box(["Balanced", "Fast", "Quiet"], Some(0))
                .id(widget)
                .expanded(true)
                .on_select(selected)
                .on_expanded_change(expanded),
        );

        let pointer = route.dispatch_click(crate::Point { x: 10, y: 90 });
        assert_eq!(pointer.combo_selection_count, 1);
        assert_eq!(pointer.combo_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 2);
        assert_eq!(route.widget_combo_state(widget), Some((Some(1), 3, false)));

        let opened = route.dispatch_key_down(u32::from(VK_SPACE));
        assert_eq!(opened.combo_expanded_change_count, 1);
        assert_eq!(opened.keyboard_activation_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(1), 3, true)));

        let keyboard = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(keyboard.combo_selection_count, 1);
        assert_eq!(keyboard.combo_keyboard_selection_count, 1);
        assert_eq!(keyboard.combo_expanded_change_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(2), 3, false)));

        let typed = route.dispatch_text_input("B");
        assert!(typed.handled);
        assert_eq!(typed.combo_type_ahead_match_count, 1);
        assert_eq!(typed.combo_selection_count, 1);
        assert_eq!(typed.combo_keyboard_selection_count, 1);
        assert_eq!(typed.ui_command_count, 1);
        assert!(typed
            .events
            .iter()
            .any(|event| event == "win32_view_combo_type_ahead_match:36:b:0"));
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));

        route.dispatch_key_down(u32::from(VK_SPACE));
        let outside = route.dispatch_pointer_down(crate::Point { x: 260, y: 200 }, false);
        assert!(outside.handled);
        assert_eq!(outside.event_count, 1);
        assert_eq!(outside.ui_command_count, 1);
        assert_eq!(outside.combo_expanded_change_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));

        route.dispatch_click(crate::Point { x: 10, y: 18 });
        let blurred = route.dispatch_blur();
        assert!(blurred.handled);
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));
    }

    #[test]
    #[cfg(feature = "combo")]
    fn window_view_input_route_scrolls_long_combo_popup() {
        let widget = crate::WidgetId::new(93);
        let options = (0..30)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let mut view = crate::column([
            crate::combo_box::<_, UiCommand>(options, Some(0))
                .id(widget)
                .height(crate::Dp::new(36.0))
                .expanded(true),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction_plan = view.interaction_plan();
        let option = interaction_plan
            .hit_targets
            .iter()
            .find(|target| target.kind == crate::ViewHitTargetKind::ComboBoxOption { index: 0 })
            .copied()
            .expect("long combo should expose a visible option");
        let mut route = WindowsWin32ViewInputRoute::new(interaction_plan, view);

        let report = route.dispatch_scroll(
            crate::Point {
                x: option.bounds.x + 8,
                y: option.bounds.y + option.bounds.height / 2,
            },
            crate::Dp::new(48.0),
        );

        assert!(report.handled);
        assert_eq!(report.combo_scroll_count, 1);
        assert_eq!(report.scroll_count, 1);
        assert_eq!(report.unhandled_scroll_count, 0);
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_view_combo_scroll:93:1"));
        assert_eq!(
            route
                .interaction_plan
                .combo_visible_option_range(widget)
                .map(|range| range.start),
            Some(1)
        );
    }

    #[test]
    #[cfg(feature = "date-picker")]
    fn window_view_input_route_selects_and_navigates_date_picker() {
        fn changed(_: crate::ZsDate) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.date_changed"))
        }
        fn expanded(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.date_expanded"))
        }

        let widget = crate::WidgetId::new(37);
        let initial = crate::ZsDate::new(2026, 7, 13).unwrap();
        let selected = crate::ZsDate::new(2026, 7, 14).unwrap();
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            crate::ViewHitTargetKind::DatePicker,
        );
        let previous = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 160,
                y: 40,
                width: 40,
                height: 48,
            },
            crate::ViewHitTargetKind::DatePickerPreviousMonth,
        );
        let day = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 80,
                y: 120,
                width: 40,
                height: 40,
            },
            crate::ViewHitTargetKind::DatePickerDay { date: selected },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, previous, day]),
            crate::date_picker(initial)
                .id(widget)
                .on_date_change(changed)
                .on_expanded_change(expanded),
        );

        let header_point = crate::Point { x: 20, y: 16 };
        let hovered = route.dispatch_pointer_move(header_point);
        assert!(hovered.handled);
        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert!(route.take_pending_draw_plan().is_some());
        let pressed = route.dispatch_pointer_down(header_point, false);
        assert_eq!(pressed.pointer_visual_change_count, 1);
        assert!(route.take_pending_draw_plan().is_some());
        let opened = route.dispatch_pointer_up(header_point);
        assert_eq!(opened.event_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        assert_eq!(opened.pointer_visual_change_count, 1);
        assert!(
            route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
        let left = route.dispatch_pointer_leave();
        assert!(left.handled);
        assert_eq!(left.pointer_visual_change_count, 1);

        let previous_month = route.dispatch_click(crate::Point { x: 180, y: 64 });
        assert_eq!(previous_month.event_count, 1);
        assert_eq!(previous_month.ui_command_count, 0);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .visible_month,
            crate::ZsDate::new(2026, 6, 1).unwrap()
        );

        let selection = route.dispatch_click(crate::Point { x: 100, y: 140 });
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.ui_command_count, 2);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(selected)
        );

        let keyboard = route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(keyboard.event_count, 1);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsDate::new(2026, 7, 15).unwrap())
        );

        route.dispatch_click(crate::Point { x: 20, y: 16 });
        let blurred = route.dispatch_blur();
        assert!(blurred.handled);
        assert!(
            !route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
    }

    #[test]
    #[cfg(feature = "time-picker")]
    fn window_view_input_route_selects_and_navigates_time_picker() {
        fn changed(_: crate::ZsTime) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.time_changed"))
        }
        fn expanded(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.time_expanded"))
        }

        let widget = crate::WidgetId::new(38);
        let initial = crate::ZsTime::new(9, 30).unwrap();
        let selected = crate::ZsTime::new(9, 45).unwrap();
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            crate::ViewHitTargetKind::TimePicker,
        );
        let choice = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 80,
                y: 120,
                width: 80,
                height: 40,
            },
            crate::ViewHitTargetKind::TimePickerChoice { value: selected },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, choice]),
            crate::time_picker(initial)
                .id(widget)
                .minute_increment(crate::ZsMinuteIncrement::FIFTEEN)
                .clock_format(crate::ZsClockFormat::TwentyFourHour)
                .on_time_change(changed)
                .on_expanded_change(expanded),
        );

        let header_point = crate::Point { x: 20, y: 16 };
        let hovered = route.dispatch_pointer_move(header_point);
        assert!(hovered.handled);
        assert_eq!(hovered.pointer_visual_change_count, 1);
        let opened = route.dispatch_click(header_point);
        assert_eq!(opened.event_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        assert!(
            route
                .widget_time_picker_state(widget)
                .expect("time picker state")
                .expanded
        );

        let selection = route.dispatch_click(crate::Point { x: 100, y: 140 });
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| (state.value, state.expanded)),
            Some((selected, true))
        );

        let closed = route.dispatch_key_down(u32::from(VK_ESCAPE));
        assert_eq!(closed.event_count, 1);
        assert_eq!(closed.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| state.expanded),
            Some(false)
        );
        let keyboard = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(keyboard.event_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsTime::new(10, 0).unwrap())
        );
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
    #[cfg(feature = "toggle-button")]
    fn window_view_input_route_dispatches_toggle_button_pointer_and_keyboard() {
        let widget = crate::WidgetId::new(15);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 36,
                },
                crate::ViewHitTargetKind::ToggleButton,
            )]),
            crate::toggle_button("Pin", false)
                .id(widget)
                .on_toggle(|_| UiCommand::app(crate::CommandId("zsui.test.pin_changed"))),
        );

        let hovered = route.dispatch_pointer_move(crate::Point { x: 20, y: 18 });
        let pointer = route.dispatch_click(crate::Point { x: 20, y: 18 });
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_SPACE);

        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert_eq!(pointer.toggle_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
        assert_eq!(keyboard.toggle_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert_eq!(route.widget_checked_value(widget), Some(false));
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
            MenuItemSpec::command("Open", Command::custom("file.open"))
                .accelerator(ZsAccelerator::primary_character('O')),
        );
        menu.items.push(
            MenuItemSpec::command("Save As", Command::custom("file.save_as"))
                .accelerator(ZsAccelerator::primary_character('S').shifted()),
        );
        let table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);
        let accelerators = WindowsWin32OwnedAcceleratorTable::from_command_table(&table)
            .expect("valid Win32 accelerators")
            .expect("accelerator table should be created");

        assert!(std::mem::needs_drop::<WindowsWin32OwnedAcceleratorTable>());
        assert_eq!(accelerators.entry_count(), 2);
        assert_eq!(
            table.entries()[0].accelerator,
            Some(ZsAccelerator::primary_character('O'))
        );

        let duplicate = WindowsWin32OwnedAcceleratorTable::from_bindings(&[
            (1, ZsAccelerator::primary_character('O')),
            (2, ZsAccelerator::primary_character('O')),
        ])
        .expect_err("duplicate native bindings must be rejected");
        assert!(matches!(
            duplicate,
            ZsuiError::InvalidSpec { field, .. } if field == "accelerator.bindings"
        ));
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
