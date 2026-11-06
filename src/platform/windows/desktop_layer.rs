// Non-activating desktop surfaces with window-owned shell-event subscriptions.
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GetClassNameW, GetForegroundWindow, HWND_BOTTOM, HWND_TOP,
    SWP_NOOWNERZORDER, WINDOWPOS, WM_WINDOWPOSCHANGING,
};

const ZSUI_WIN32_DESKTOP_REFRESH: u32 = WM_APP + 0x0555;

struct WindowsDesktopLayer {
    hwnd: HWND,
    owner: std::rc::Weak<WindowsWindowServices>,
    hook: HWINEVENTHOOK,
    visible: Cell<bool>,
    follow_show_desktop: bool,
}

impl WindowsDesktopLayer {
    fn attach(hwnd: HWND, visible: bool, follow_show_desktop: bool) -> ZsuiResult<Self> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        };
        let style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32
            | WS_EX_TOOLWINDOW
            | WS_EX_NOACTIVATE;
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongW(
                hwnd,
                GWL_EXSTYLE,
                style as i32,
            );
        }
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                null_mut(),
                Some(desktop_foreground_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.is_null() {
            return Err(ZsuiError::host(
                "desktop_layer",
                "could not subscribe to desktop foreground events",
            ));
        }
        let owner = window_services(hwnd)
            .map(|services| Rc::downgrade(&services))
            .unwrap_or_default();
        let layer = Self {
            hwnd,
            owner,
            hook,
            visible: Cell::new(visible),
            follow_show_desktop,
        };
        layer.refresh(false);
        Ok(layer)
    }

    fn refresh(&self, shell_restarted: bool) {
        if self.owner.upgrade().is_none_or(|owner| owner.closed.get())
            || unsafe { IsWindow(self.hwnd) } == 0
        {
            return;
        }
        unsafe {
            if self.visible.get()
                && (shell_restarted || (self.follow_show_desktop && desktop_is_foreground()))
            {
                ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
            if self.owner.upgrade().is_none_or(|owner| owner.closed.get()) {
                return;
            }
            SetWindowPos(
                self.hwnd,
                desktop_insert_after(self.hwnd),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            );
        }
    }
}

impl Drop for WindowsDesktopLayer {
    fn drop(&mut self) {
        unsafe {
            UnhookWinEvent(self.hook);
        }
    }
}

unsafe fn desktop_insert_after(own: HWND) -> HWND {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, IsWindowVisible, GW_HWNDPREV,
    };
    unsafe extern "system" fn find(hwnd: HWND, context: LPARAM) -> i32 {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let mut class = [0u16; 64];
        let length = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
        if matches!(
            String::from_utf16_lossy(&class[..length.max(0) as usize]).as_str(),
            "Progman" | "WorkerW"
        ) {
            *(context as *mut HWND) = hwnd;
            return 0;
        }
        1
    }
    let mut shell: HWND = null_mut();
    EnumWindows(Some(find), (&mut shell as *mut HWND) as LPARAM);
    if shell.is_null() {
        return HWND_BOTTOM;
    }
    let mut previous = GetWindow(shell, GW_HWNDPREV);
    if previous == own {
        previous = GetWindow(previous, GW_HWNDPREV);
    }
    if previous.is_null() || GetWindowLongW(previous, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST != 0 {
        HWND_TOP
    } else {
        previous
    }
}

unsafe fn desktop_is_foreground() -> bool {
    let foreground = GetForegroundWindow();
    let mut class = [0u16; 64];
    let length = GetClassNameW(foreground, class.as_mut_ptr(), class.len() as i32);
    matches!(
        String::from_utf16_lossy(&class[..length.max(0) as usize]).as_str(),
        "Progman" | "WorkerW"
    )
}

unsafe extern "system" fn desktop_foreground_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    // OUTOFCONTEXT callbacks arrive on the registering UI thread. Enumerate
    // that thread's windows instead of maintaining a global control registry.
    EnumThreadWindows(
        windows_sys::Win32::System::Threading::GetCurrentThreadId(),
        Some(notify_desktop_window),
        0,
    );
}

unsafe extern "system" fn notify_desktop_window(hwnd: HWND, _context: LPARAM) -> i32 {
    let mut name = [0u16; 64];
    let length = GetClassNameW(hwnd, name.as_mut_ptr(), name.len() as i32);
    if String::from_utf16_lossy(&name[..length.max(0) as usize]) == DEFAULT_MAIN_CLASS_NAME {
        if window_services(hwnd).is_some_and(|services| services.desktop.borrow().is_some()) {
            PostMessageW(hwnd, ZSUI_WIN32_DESKTOP_REFRESH, 0, 0);
        }
    }
    1
}

unsafe fn desktop_window_message(
    hwnd: HWND,
    msg: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let services = window_services(hwnd)?;
    let layer = services.desktop_layer()?;
    if msg == WM_WINDOWPOSCHANGING {
        let position = lparam as *mut WINDOWPOS;
        if !position.is_null() {
            (*position).hwndInsertAfter = desktop_insert_after(hwnd);
            (*position).flags =
                ((*position).flags & !SWP_NOZORDER) | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
        }
    }
    if msg == ZSUI_WIN32_DESKTOP_REFRESH {
        layer.refresh(false);
        return Some(0);
    }
    None
}

fn set_desktop_window_visibility(hwnd: HWND, visible: bool) -> bool {
    let Some(layer) = window_services(hwnd).and_then(|services| services.desktop_layer()) else {
        return false;
    };
    layer.visible.set(visible);
    if visible {
        layer.refresh(true);
    } else {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    true
}
