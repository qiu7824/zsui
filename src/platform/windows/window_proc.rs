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
            let state = Box::into_raw(Box::new(create_params));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            let result = DefWindowProcW(hwnd, msg, wparam, lparam);
            if result == 0 {
                let state = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0)
                    as *mut WindowsWindowCreateParams;
                if !state.is_null() {
                    drop(Box::from_raw(state));
                }
            }
            result
        }
        WM_GETMINMAXINFO => {
            let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                as *const WindowsWindowCreateParams;
            let minmax = lparam as *mut MINMAXINFO;
            if !state.is_null() && !minmax.is_null() {
                if let Some(min_size) = (*state).min_size {
                    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
                    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                    let (width, height) = windows_win32_outer_size_for_client_at_dpi(
                        min_size.width,
                        min_size.height,
                        style,
                        ex_style,
                        !GetMenu(hwnd).is_null(),
                        GetDpiForWindow(hwnd).max(96),
                    );
                    (*minmax).ptMinTrackSize.x = (*minmax).ptMinTrackSize.x.max(width);
                    (*minmax).ptMinTrackSize.y = (*minmax).ptMinTrackSize.y.max(height);
                    return 0;
                }
            }
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
            let state = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0)
                as *mut WindowsWindowCreateParams;
            let role = if state.is_null() {
                WindowsWindowRole::Main
            } else {
                Box::from_raw(state).role
            };
            #[cfg(feature = "accessibility")]
            crate::windows_semantic_uia::disconnect(hwnd);
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
        #[cfg(feature = "accessibility")]
        WM_GETOBJECT => {
            #[cfg(feature = "menu-flyout")]
            if let Some(result) = crate::windows_menu_uia::handle_get_object(hwnd, wparam, lparam) {
                return result;
            }
            // Keep one fragment root for the complete retained View. Editable
            // semantic children delegate their Value/Text patterns to the
            // focused text provider instead of replacing every sibling in the
            // accessibility tree.
            if let Some(result) =
                crate::windows_semantic_uia::handle_get_object(hwnd, wparam, lparam)
            {
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
            let mut routed = false;
            if (lparam as u32 & GCS_RESULTSTR) != 0 {
                if let Some(text) = windows_ime_composition_text(hwnd, GCS_RESULTSTR) {
                    routed |= dispatch_windows_win32_window_view_ime_commit(hwnd, &text).is_some();
                }
            }
            if (lparam as u32 & GCS_COMPSTR) != 0 {
                if let Some(text) = windows_ime_composition_text(hwnd, GCS_COMPSTR) {
                    let selection = windows_ime_composition_selection(hwnd, &text);
                    routed |= dispatch_windows_win32_window_view_ime_preedit(
                        hwnd,
                        &text,
                        selection,
                    )
                    .is_some();
                    position_windows_ime_candidate(hwnd);
                }
            }
            if routed {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_IME_ENDCOMPOSITION => {
            if dispatch_windows_win32_window_view_ime_cancel(hwnd)
                .is_some_and(|report| report.handled)
            {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
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
