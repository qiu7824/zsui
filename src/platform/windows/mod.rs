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
    decorate_native_focus_ring, decorate_native_text_edit_visuals_in_viewport_with_backend,
    move_native_text_selection_horizontally_with_backend,
    native_text_drag_viewport_for_point_with_backend,
    native_text_first_visible_row_for_caret_with_backend,
    native_text_horizontal_scroll_for_caret_with_backend,
    native_text_index_for_point_in_viewport_with_backend,
    native_text_index_for_vertical_move_with_backend,
    native_text_index_for_vertical_page_move_with_backend,
    native_text_scroll_visual_rows_with_backend, native_text_visual_target,
    native_text_wheel_row_delta, NativeTextVisualDirection, NativeTextVisualHorizontalDirection,
};
#[cfg(any(
    feature = "auto-suggest",
    feature = "button",
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
#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
use crate::native_input_visuals::{
    native_text_first_visible_row_for_index_alignment_with_backend,
    native_text_visible_range_with_backend, native_text_visual_geometry_in_viewport_with_backend,
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

#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
use windows_sys::Win32::UI::WindowsAndMessaging::WM_GETOBJECT;

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

include!("application.rs");
include!("services/menu.rs");
include!("popup.rs");
include!("services/hotkey.rs");
include!("services/dialog.rs");
include!("services/tray.rs");
include!("services/clipboard.rs");
include!("window.rs");
include!("input/mod.rs");
include!("input/pointer.rs");
include!("input/ime.rs");
include!("input/keyboard.rs");
include!("input/focus.rs");
include!("window_proc.rs");
include!("text/composition.rs");
include!("timer.rs");
include!("dpi.rs");
include!("tests.rs");
