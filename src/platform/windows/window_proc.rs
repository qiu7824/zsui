thread_local! {
    /// Pairs a handled WM_SYSKEYDOWN shortcut with its following WM_SYSCHAR.
    static ZSUI_WIN32_SYSCHAR_HANDLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub unsafe extern "system" fn zsui_win32_default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(result) = native_control_color(hwnd, msg, wparam as _) {
        return result;
    }
    #[cfg(feature = "windows-shell")]
    if let Some(result) = menu_bridge::dispatch_menu_hook(hwnd, msg, wparam, lparam) {
        return result;
    }
    #[cfg(feature = "desktop-layer")]
    if let Some(result) = desktop_window_message(hwnd, msg, wparam, lparam) {
        return result;
    }
    #[cfg(feature = "window-chrome")]
    if let Some(result) = integrated_caption_message(hwnd, msg, wparam, lparam) {
        return result;
    }
    let taskbar_created_message = windows_win32_taskbar_created_message();
    if taskbar_created_message != 0 && msg == taskbar_created_message {
        restore_windows_win32_status_items(hwnd);
        #[cfg(feature = "desktop-layer")]
        if let Some(services) = window_services(hwnd) {
            if let Some(desktop) = services.desktop_layer() {
                desktop.refresh(true);
                services.queue_event(crate::NativeWindowEvent::DesktopRecreated);
            }
        }
        return 0;
    }
    if dispatch_windows_win32_status_item_callback(hwnd, msg, wparam, lparam).is_some() {
        return 0;
    }
    match msg {
        WM_NCCREATE => {
            let create_params =
                WindowsWindowCreateParams::from_create_struct(lparam as *const CREATESTRUCTW);
            let state = Box::into_raw(Box::new(WindowsWindowState {
                params: create_params,
                services: Rc::new(WindowsWindowServices::new(hwnd)),
            }));
            if windows_sys::Win32::UI::WindowsAndMessaging::SetPropW(
                hwnd,
                ZSUI_WINDOW_STATE_PROPERTY,
                state.cast(),
            ) == 0
            {
                drop(Box::from_raw(state));
                return 0;
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            let result = DefWindowProcW(hwnd, msg, wparam, lparam);
            if result == 0 {
                windows_sys::Win32::UI::WindowsAndMessaging::RemovePropW(
                    hwnd,
                    ZSUI_WINDOW_STATE_PROPERTY,
                );
                let state = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut WindowsWindowState;
                if !state.is_null() {
                    drop(Box::from_raw(state));
                }
            }
            result
        }
        WM_GETMINMAXINFO => {
            let state = windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(
                hwnd,
                ZSUI_WINDOW_STATE_PROPERTY,
            ) as *const WindowsWindowState;
            let minmax = lparam as *mut MINMAXINFO;
            if !state.is_null() && !minmax.is_null() {
                if let Some(min_size) = (*state).params.min_size {
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
            clear_windows_win32_status_item_routes_for_owner(hwnd);
            let state = windows_sys::Win32::UI::WindowsAndMessaging::RemovePropW(
                hwnd,
                ZSUI_WINDOW_STATE_PROPERTY,
            ) as *mut WindowsWindowState;
            let tracked = !state.is_null();
            if GetWindowLongPtrW(hwnd, GWLP_USERDATA) == state as isize {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            let role = if state.is_null() {
                WindowsWindowRole::Main
            } else {
                let state = Box::from_raw(state);
                state.services.shutdown();
                state.params.role
            };
            #[cfg(feature = "accessibility")]
            crate::windows_semantic_uia::disconnect(hwnd);
            clear_windows_win32_window_draw_plan(hwnd);
            archive_windows_win32_window_view_input_report(hwnd);
            clear_windows_win32_window_shell_input_route(hwnd);
            clear_windows_win32_window_menu_command_table(hwnd);
            if tracked
                && matches!(role, WindowsWindowRole::Main)
                && ACTIVE_MAIN_WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst) <= 1
            {
                PostQuitMessage(0);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        #[cfg(feature = "window-chrome")]
        WM_NCCALCSIZE if windows_win32_window_has_custom_title_bar(hwnd) => {
            // Both forms of WM_NCCALCSIZE start with the proposed window
            // rectangle. Keep the default side and bottom resize frame but
            // give the caption to the client so the View draws the title bar.
            let proposed = lparam as *mut RECT;
            let original_top = (*proposed).top;
            let result = DefWindowProcW(hwnd, msg, wparam, lparam);
            if result != 0 {
                return result;
            }
            (*proposed).top = original_top;
            if IsZoomed(hwnd) != 0 {
                // A maximized window hangs its frame off-screen.
                (*proposed).top += windows_win32_resize_border_height(hwnd);
            }
            0
        }
        #[cfg(feature = "window-chrome")]
        WM_NCHITTEST if windows_win32_window_has_custom_title_bar(hwnd) => {
            let hit = DefWindowProcW(hwnd, msg, wparam, lparam);
            if hit != HTCLIENT as LRESULT {
                return hit;
            }
            let screen = point_from_lparam(lparam);
            let mut point = POINT {
                x: screen.x,
                y: screen.y,
            };
            ScreenToClient(hwnd, &mut point);
            if IsZoomed(hwnd) == 0
                && point.y >= 0
                && point.y < windows_win32_resize_border_height(hwnd)
            {
                return HTTOP as LRESULT;
            }
            if windows_win32_window_view_drag_region_at(
                hwnd,
                crate::Point {
                    x: point.x,
                    y: point.y,
                },
            ) {
                HTCAPTION as LRESULT
            } else {
                HTCLIENT as LRESULT
            }
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
            #[cfg(feature = "native-smoke")]
            crate::native::record_native_display_scale(
                "win32",
                "WM_DPICHANGED",
                (wparam & 0xffff) as f64 / 96.0,
            );
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
            #[cfg(feature = "window-chrome")]
            update_integrated_caption(hwnd);
            #[cfg(feature = "window-chrome")]
            {
                let maximized = IsZoomed(hwnd) != 0;
                dispatch_windows_win32_window_view_input(hwnd, |route| {
                    route.dispatch_window_state(maximized)
                });
            }
            let minimized = wparam == SIZE_MINIMIZED as usize;
            let lifecycle_handled = sync_windows_win32_window_view_visibility(hwnd, !minimized);
            let shell_handled =
                !minimized && refresh_windows_win32_window_shell_surface(hwnd).is_some();
            let live_view_handled =
                !minimized && refresh_windows_win32_window_live_view_surface(hwnd);
            if !minimized {
                if let Some(services) = window_services(hwnd) {
                    if let Err(error) = services.present_layered() {
                        services.record_error(error);
                    }
                }
            }
            if lifecycle_handled || shell_handled || live_view_handled {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        #[cfg(feature = "window-chrome")]
        windows_sys::Win32::UI::WindowsAndMessaging::WM_DWMCOMPOSITIONCHANGED => {
            update_integrated_caption(hwnd);
            InvalidateRect(hwnd, null(), 0);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_SHOWWINDOW => {
            if sync_windows_win32_window_view_visibility(hwnd, wparam != 0) {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_SETTINGCHANGE | WM_SYSCOLORCHANGE | WM_THEMECHANGED => {
            if let Some(services) = window_services(hwnd) {
                services.sync_appearance();
            }
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
            let result = if dispatch_windows_win32_window_shell_pointer_up(hwnd).is_some() {
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
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            };
            // A captured target may disappear during a controlled rebuild.
            // Always end this window's capture on the matching button release.
            if windows_sys::Win32::UI::Input::KeyboardAndMouse::GetCapture() == hwnd {
                ReleaseCapture();
            }
            result
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
                if msg == WM_XBUTTONDOWN {
                    1
                } else {
                    0
                }
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
                if msg == WM_XBUTTONUP {
                    1
                } else {
                    0
                }
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
                    routed |=
                        dispatch_windows_win32_window_view_ime_preedit(hwnd, &text, selection)
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
        WM_KILLFOCUS
            if windows_sys::Win32::UI::WindowsAndMessaging::IsChild(hwnd, wparam as HWND) != 0 =>
        {
            0
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
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            match dispatch_windows_win32_window_view_key_down_with_modifiers(
                hwnd,
                wparam as u32,
                (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0,
                (GetKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0,
                msg == WM_SYSKEYDOWN || (GetKeyState(VK_MENU as i32) as u16 & 0x8000) != 0,
                (GetKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0
                    || (GetKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0,
            ) {
                Some(report) if report.unhandled_key_count == 0 => {
                    // TranslateMessage still posts WM_SYSCHAR for a handled Alt
                    // chord; swallow it so DefWindowProc does not beep.
                    ZSUI_WIN32_SYSCHAR_HANDLED.with(|handled| handled.set(msg == WM_SYSKEYDOWN));
                    0
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
        WM_SYSCHAR if ZSUI_WIN32_SYSCHAR_HANDLED.with(|handled| handled.replace(false)) => 0,
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
        ZSUI_WIN32_INVALIDATE_VIEW_MESSAGE => {
            if refresh_windows_win32_window_invalidated_view(hwnd).is_some() {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        ZSUI_WIN32_SYNC_NATIVE_HOSTS => {
            if let Some(services) = window_services(hwnd) {
                if let Err(error) = services.sync_native_hosts() {
                    services.record_error(error);
                }
                services.sync_appearance();
            }
            0
        }
        ZSUI_WIN32_NATIVE_EVENTS => {
            if let Some(services) = window_services(hwnd) {
                services.dispatch_events();
            }
            0
        }
        #[cfg(feature = "hotkey")]
        windows_sys::Win32::UI::WindowsAndMessaging::WM_HOTKEY => {
            let command = window_services(hwnd).and_then(|services| {
                services.hotkeys.borrow().as_ref().and_then(|hotkeys| {
                    hotkeys
                        .commands
                        .iter()
                        .find(|(id, _)| *id == wparam as i32)
                        .map(|(_, command)| command.clone())
                })
            });
            if let Some(command) = command {
                #[cfg(feature = "native-window-events")]
                if let Some(services) = window_services(hwnd) {
                    services.queue_event(crate::NativeWindowEvent::GlobalHotkey(command.clone()));
                }
                dispatch_windows_win32_window_view_input(hwnd, |route| {
                    route.dispatch_app_command(command)
                });
            }
            0
        }
        WM_PAINT => paint_no_flicker_background(hwnd),
        WM_PRINTCLIENT => paint_window_client_to_dc(hwnd, wparam as _),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(feature = "window-chrome")]
fn windows_win32_window_has_custom_title_bar(hwnd: HWND) -> bool {
    let state = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(hwnd, ZSUI_WINDOW_STATE_PROPERTY)
    } as *const WindowsWindowState;
    !state.is_null() && unsafe { (*state).params.custom_title_bar }
}

#[cfg(feature = "window-chrome")]
fn windows_win32_resize_border_height(hwnd: HWND) -> i32 {
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    unsafe {
        GetSystemMetricsForDpi(SM_CYSIZEFRAME as _, dpi)
            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER as _, dpi)
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
