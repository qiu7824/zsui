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
            #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
            crate::windows_uia::disconnect(hwnd);
            #[cfg(all(
                feature = "accessibility",
                feature = "menu-flyout",
                not(feature = "text-input-core")
            ))]
            crate::windows_menu_uia::disconnect(hwnd);
            #[cfg(all(
                feature = "accessibility",
                feature = "tabs",
                not(feature = "text-input-core"),
                not(feature = "menu-flyout")
            ))]
            crate::windows_tab_uia::disconnect(hwnd);
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
        #[cfg(all(
            feature = "accessibility",
            any(
                feature = "text-input-core",
                feature = "menu-flyout",
                feature = "tabs"
            )
        ))]
        WM_GETOBJECT => {
            #[cfg(feature = "menu-flyout")]
            if let Some(result) = crate::windows_menu_uia::handle_get_object(hwnd, wparam, lparam) {
                return result;
            }
            #[cfg(feature = "text-input-core")]
            if let Some(result) = crate::windows_uia::handle_get_object(hwnd, wparam, lparam) {
                return result;
            }
            #[cfg(feature = "tabs")]
            if let Some(result) = crate::windows_tab_uia::handle_get_object(hwnd, wparam, lparam) {
                return result;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
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
            let minimized = wparam == SIZE_MINIMIZED as usize;
            let lifecycle_handled = sync_windows_win32_window_view_visibility(hwnd, !minimized);
            let shell_handled =
                !minimized && refresh_windows_win32_window_shell_surface(hwnd).is_some();
            let live_view_handled =
                !minimized && refresh_windows_win32_window_live_view_surface(hwnd);
            if lifecycle_handled || shell_handled || live_view_handled {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_SHOWWINDOW => {
            if sync_windows_win32_window_view_visibility(hwnd, wparam != 0) {
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
            } else if dispatch_windows_win32_window_view_pointer_move_with_modifiers(
                hwnd,
                point_from_lparam(lparam),
                windows_pointer_modifiers(wparam),
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
            } else if dispatch_windows_win32_window_view_pointer_down_with_button(
                hwnd,
                point_from_lparam(lparam),
                crate::ZsPointerButton::Primary,
                windows_pointer_modifiers(wparam),
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
            } else if dispatch_windows_win32_window_view_pointer_up_with_button(
                hwnd,
                point_from_lparam(lparam),
                crate::ZsPointerButton::Primary,
                windows_pointer_modifiers(wparam),
            )
                .is_some_and(|report| report.handled)
            {
                SetFocus(hwnd);
                ReleaseCapture();
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
            let button = windows_pointer_button(msg, wparam);
            if dispatch_windows_win32_window_view_pointer_down_with_button(
                hwnd,
                point_from_lparam(lparam),
                button,
                windows_pointer_modifiers(wparam),
            )
            .is_some_and(|report| report.handled)
            {
                SetFocus(hwnd);
                SetCapture(hwnd);
                if msg == WM_XBUTTONDOWN { 1 } else { 0 }
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
            let button = windows_pointer_button(msg, wparam);
            if dispatch_windows_win32_window_view_pointer_up_with_button(
                hwnd,
                point_from_lparam(lparam),
                button,
                windows_pointer_modifiers(wparam),
            )
            .is_some_and(|report| report.handled)
            {
                ReleaseCapture();
                if msg == WM_XBUTTONUP { 1 } else { 0 }
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
        WM_CHAR => match dispatch_windows_win32_window_view_utf16_input_unit(hwnd, wparam as u16) {
            Some(report) if report.handled => 0,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        },
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

fn windows_pointer_modifiers(wparam: WPARAM) -> crate::ZsPointerModifiers {
    crate::ZsPointerModifiers::new(
        wparam & 0x0004 != 0 || unsafe { GetKeyState(VK_SHIFT as i32) as u16 & 0x8000 != 0 },
        wparam & 0x0008 != 0 || unsafe { GetKeyState(VK_CONTROL as i32) as u16 & 0x8000 != 0 },
        unsafe { GetKeyState(VK_MENU as i32) as u16 & 0x8000 != 0 },
        unsafe {
            GetKeyState(VK_LWIN as i32) as u16 & 0x8000 != 0
                || GetKeyState(VK_RWIN as i32) as u16 & 0x8000 != 0
        },
    )
}

fn windows_pointer_button(msg: u32, wparam: WPARAM) -> crate::ZsPointerButton {
    match msg {
        WM_RBUTTONDOWN | WM_RBUTTONUP => crate::ZsPointerButton::Secondary,
        WM_MBUTTONDOWN | WM_MBUTTONUP => crate::ZsPointerButton::Middle,
        WM_XBUTTONDOWN | WM_XBUTTONUP => {
            crate::ZsPointerButton::Auxiliary(((wparam >> 16) & 0xffff) as u16)
        }
        _ => crate::ZsPointerButton::Primary,
    }
}
