use super::DesktopSmokeRequest;
use crate::{
    native::{
        menu_command_count, record_draw_plan_smoke, record_native_view_text_input_script_evidence,
        NativeViewInputRuntime,
    },
    NativeDrawPlan, NativeStatusItemHost, NativeStatusItemPresentation, NativeStatusItemRequest,
    NativeStatusMenuCommandResult, NativeViewKey, NativeViewSmokeInput,
    NativeWindowSmokeRunOptions, NativeWindowSmokeRunReport, Point, WindowSpec, ZsShellRuntime,
    ZsuiError, ZsuiResult,
};

pub(super) fn run(request: DesktopSmokeRequest) -> ZsuiResult<NativeWindowSmokeRunReport> {
    run_native_window_smoke_event_loop(
        request.windows,
        request.draw_plans,
        request.view_runtime,
        request.shell_runtime,
        request.options,
    )
}

#[cfg(all(windows, feature = "windows-win32"))]
fn record_windows_win32_view_input_report(
    report: &mut NativeWindowSmokeRunReport,
    input: &crate::windows_win32_host::WindowsWin32ViewInputDispatchReport,
) {
    report.native_view_window_close_request_count += input.window_close_request_count;
    report.native_view_window_close_veto_count += input.window_close_veto_count;
    report.native_view_hit_target_count = input.hit_target_count;
    report.native_view_click_count += input.click_count;
    report.native_view_event_count += input.event_count;
    report.native_view_message_count += input.message_count;
    report.native_view_ui_command_count += input.ui_command_count;
    report.native_view_ui_command_executed_count += input.ui_command_executed_count;
    report.native_view_ui_command_failed_count += input.ui_command_failed_count;
    report.native_view_ui_command_unhandled_count += input.ui_command_unhandled_count;
    report.native_view_ui_command_event_count += input.ui_command_event_count;
    report
        .native_view_ui_command_errors
        .extend(input.ui_command_errors.iter().cloned());
    report.native_view_app_command_count += input.app_command_count;
    report.native_view_app_command_executed_count += input.app_command_executed_count;
    report.native_view_app_command_failed_count += input.app_command_failed_count;
    report.native_view_app_command_unhandled_count += input.app_command_unhandled_count;
    report.native_view_app_command_event_count += input.app_command_event_count;
    report
        .native_view_app_command_names
        .extend(input.app_command_names.iter().copied());
    report
        .native_view_app_command_errors
        .extend(input.app_command_errors.iter().cloned());
    report
        .native_view_ui_command_ids
        .extend(input.ui_command_ids.iter().copied());
    report.native_view_live_revision = report
        .native_view_live_revision
        .max(input.live_view_revision);
    report.native_view_quit_requested |= input.quit_requested;
    report.native_view_unhandled_click_count += input.unhandled_click_count;
    report.native_view_focus_count += input.focus_count;
    report.native_view_focus_visual_count += input.focus_visual_count;
    report.native_view_focus_traversal_count += input.focus_traversal_count;
    report.native_view_focused_widget = input.focused_widget.or(report.native_view_focused_widget);
    report.native_view_text_input_count += input.text_input_count;
    report.native_view_text_navigation_count += input.text_navigation_count;
    report
        .native_view_text_navigation_evidence
        .extend(input.text_navigation_evidence.iter().cloned());
    report.native_view_text_selection_change_count += input.text_selection_change_count;
    report.native_view_text_selection = input.text_selection.or(report.native_view_text_selection);
    report.native_view_text_caret = input.text_caret.or(report.native_view_text_caret);
    #[cfg(feature = "textbox")]
    {
        report.native_view_text_edit_command_count += input.text_edit_command_count;
        report.native_view_text_clipboard_read_count += input.text_clipboard_read_count;
        report.native_view_text_clipboard_write_count += input.text_clipboard_write_count;
        report.native_view_text_undo_count += input.text_undo_count;
        report
            .native_view_text_edit_command_errors
            .extend(input.text_edit_command_errors.iter().cloned());
    }
    report.native_view_pointer_down_count += input.pointer_down_count;
    report.native_view_pointer_move_count += input.pointer_move_count;
    report.native_view_pointer_up_count += input.pointer_up_count;
    report.native_view_canvas_pointer_event_count += input.canvas_pointer_event_count;
    report.native_view_canvas_pointer_drag_count += input.canvas_pointer_drag_count;
    report.native_view_pointer_visual_change_count += input.pointer_visual_change_count;
    report.native_view_text_drag_count += input.text_drag_count;
    report.native_view_text_drag_scroll_count += input.text_drag_scroll_count;
    report.native_view_slider_value_change_count += input.slider_value_change_count;
    report.native_view_slider_keyboard_change_count += input.slider_keyboard_change_count;
    report.native_view_slider_drag_count += input.slider_drag_count;
    report.native_view_color_picker_value_change_count += input.color_picker_value_change_count;
    report.native_view_color_picker_channel_change_count += input.color_picker_channel_change_count;
    report.native_view_color_picker_expanded_change_count +=
        input.color_picker_expanded_change_count;
    report.native_view_color_picker_drag_count += input.color_picker_drag_count;
    report.native_view_radio_selection_count += input.radio_selection_count;
    report.native_view_radio_keyboard_selection_count += input.radio_keyboard_selection_count;
    report.native_view_radio_keyboard_focus_only_count += input.radio_keyboard_focus_only_count;
    report.native_view_auto_suggest_expanded_change_count +=
        input.auto_suggest_expanded_change_count;
    report.native_view_auto_suggest_highlight_change_count +=
        input.auto_suggest_highlight_change_count;
    report.native_view_auto_suggest_submit_count += input.auto_suggest_submit_count;
    report.native_view_auto_suggest_clear_count += input.auto_suggest_clear_count;
    report.native_view_tree_expansion_change_count += input.tree_expansion_change_count;
    report.native_view_tree_selection_count += input.tree_selection_count;
    report.native_view_tree_invoke_count += input.tree_invoke_count;
    report.native_view_grid_view_selection_count += input.grid_view_selection_count;
    report.native_view_grid_view_invoke_count += input.grid_view_invoke_count;
    report.native_view_table_sort_count += input.table_sort_count;
    report.native_view_table_selection_count += input.table_selection_count;
    report.native_view_table_invoke_count += input.table_invoke_count;
    report.native_view_content_dialog_focus_count += input.content_dialog_focus_change_count;
    report.native_view_content_dialog_response_count += input.content_dialog_response_count;
    report.native_view_command_palette_query_change_count +=
        input.command_palette_query_change_count;
    report.native_view_command_palette_highlight_change_count +=
        input.command_palette_highlight_change_count;
    report.native_view_command_palette_invoke_count += input.command_palette_invoke_count;
    report.native_view_command_palette_open_change_count += input.command_palette_open_change_count;
    report.native_view_command_palette_clear_count += input.command_palette_clear_count;
    report.native_view_menu_flyout_highlight_change_count +=
        input.menu_flyout_highlight_change_count;
    report.native_view_menu_flyout_submenu_change_count += input.menu_flyout_submenu_change_count;
    report.native_view_menu_flyout_invoke_count += input.menu_flyout_invoke_count;
    report.native_view_menu_flyout_open_change_count += input.menu_flyout_open_change_count;
    report.native_view_toast_focus_count += input.toast_focus_change_count;
    report.native_view_toast_response_count += input.toast_response_count;
    report.native_view_toast_timeout_count += input.toast_timeout_count;
    report.native_view_info_bar_focus_count += input.info_bar_focus_change_count;
    report.native_view_info_bar_event_count += input.info_bar_event_count;
    report.native_view_teaching_tip_focus_count += input.teaching_tip_focus_change_count;
    report.native_view_teaching_tip_response_count += input.teaching_tip_response_count;
    report.native_view_breadcrumb_focus_count += input.breadcrumb_focus_change_count;
    report.native_view_breadcrumb_expanded_change_count += input.breadcrumb_expanded_change_count;
    report.native_view_breadcrumb_selection_count += input.breadcrumb_selection_count;
    report.native_view_combo_expanded_change_count += input.combo_expanded_change_count;
    report.native_view_combo_selection_count += input.combo_selection_count;
    report.native_view_combo_keyboard_selection_count += input.combo_keyboard_selection_count;
    report.native_view_combo_type_ahead_match_count += input.combo_type_ahead_match_count;
    report.native_view_combo_scroll_count += input.combo_scroll_count;
    report.native_view_tab_selection_count += input.tab_selection_count;
    report.native_view_tab_keyboard_selection_count += input.tab_keyboard_selection_count;
    report.native_view_tab_keyboard_focus_only_count += input.tab_keyboard_focus_only_count;
    report.native_view_toggle_count += input.toggle_count;
    report.native_view_selection_count += input.selection_count;
    report.native_view_keyboard_selection_count += input.keyboard_selection_count;
    report.native_view_key_down_count += input.key_down_count;
    report.native_view_keyboard_activation_count += input.keyboard_activation_count;
    report.native_view_unhandled_key_count += input.unhandled_key_count;
    report.native_view_scroll_count += input.scroll_count;
    report.native_view_unhandled_scroll_count += input.unhandled_scroll_count;
    report.events.extend(input.events.clone());
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_lparam_from_point(point: Point) -> isize {
    let x = point.x as i16 as u16 as u32;
    let y = point.y as i16 as u16 as u32;
    ((y << 16) | x) as isize
}

#[cfg(all(windows, feature = "windows-win32"))]
fn post_windows_native_view_input(
    hwnd: windows_sys::Win32::Foundation::HWND,
    input: &NativeViewSmokeInput,
) {
    use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        PostMessageW, WM_CHAR, WM_CLOSE, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
        WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
        WM_XBUTTONUP,
    };

    match input {
        NativeViewSmokeInput::Move(point) => unsafe {
            PostMessageW(hwnd, WM_MOUSEMOVE, 0, windows_lparam_from_point(*point));
        },
        NativeViewSmokeInput::Click(point) => unsafe {
            let lparam = windows_lparam_from_point(*point);
            PostMessageW(hwnd, WM_LBUTTONDOWN, 0, lparam);
            PostMessageW(hwnd, WM_LBUTTONUP, 0, lparam);
        },
        NativeViewSmokeInput::Drag { start, end } => unsafe {
            PostMessageW(hwnd, WM_LBUTTONDOWN, 0, windows_lparam_from_point(*start));
            PostMessageW(hwnd, WM_MOUSEMOVE, 0, windows_lparam_from_point(*end));
            PostMessageW(hwnd, WM_LBUTTONUP, 0, windows_lparam_from_point(*end));
        },
        NativeViewSmokeInput::PointerDrag {
            start,
            end,
            button,
            modifiers,
        } => unsafe {
            let (down, up) = match button {
                crate::ZsPointerButton::Primary => (WM_LBUTTONDOWN, WM_LBUTTONUP),
                crate::ZsPointerButton::Secondary => (WM_RBUTTONDOWN, WM_RBUTTONUP),
                crate::ZsPointerButton::Middle => (WM_MBUTTONDOWN, WM_MBUTTONUP),
                crate::ZsPointerButton::Auxiliary(_) => (WM_XBUTTONDOWN, WM_XBUTTONUP),
            };
            let wparam = windows_wparam_from_pointer(*button, *modifiers);
            PostMessageW(hwnd, down, wparam, windows_lparam_from_point(*start));
            PostMessageW(
                hwnd,
                WM_MOUSEMOVE,
                wparam & 0xffff,
                windows_lparam_from_point(*end),
            );
            PostMessageW(hwnd, up, wparam, windows_lparam_from_point(*end));
        },
        NativeViewSmokeInput::Text(text) => {
            for unit in text.encode_utf16() {
                unsafe {
                    PostMessageW(hwnd, WM_CHAR, unit as usize, 0);
                }
            }
        }
        NativeViewSmokeInput::KeyDown(key) => unsafe {
            PostMessageW(
                hwnd,
                WM_KEYDOWN,
                windows_wparam_from_native_view_key(*key),
                0,
            );
        },
        NativeViewSmokeInput::Scroll { point, delta_y } => unsafe {
            let mut screen_point = windows_sys::Win32::Foundation::POINT {
                x: point.x,
                y: point.y,
            };
            ClientToScreen(hwnd, &mut screen_point);
            PostMessageW(
                hwnd,
                WM_MOUSEWHEEL,
                windows_wparam_from_scroll_delta_y(*delta_y),
                windows_lparam_from_point(Point {
                    x: screen_point.x,
                    y: screen_point.y,
                }),
            );
        },
        NativeViewSmokeInput::WindowCloseRequest => unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        },
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_wparam_from_pointer(
    button: crate::ZsPointerButton,
    modifiers: crate::ZsPointerModifiers,
) -> usize {
    let mut value = 0_usize;
    if modifiers.shift {
        value |= 0x0004;
    }
    if modifiers.control {
        value |= 0x0008;
    }
    if let crate::ZsPointerButton::Auxiliary(button) = button {
        value |= (usize::from(button.max(1)) & 0xffff) << 16;
    }
    value
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_wparam_from_scroll_delta_y(delta_y: i32) -> usize {
    let wheel_delta = ((-(delta_y as f32) / 48.0) * 120.0).round() as i16;
    ((wheel_delta as u16 as usize) << 16) & 0xffff_0000
}

#[cfg(all(windows, feature = "windows-win32"))]
fn windows_wparam_from_native_view_key(key: NativeViewKey) -> usize {
    match key {
        NativeViewKey::Enter => 0x0d,
        NativeViewKey::Escape => 0x1b,
        NativeViewKey::Tab => 0x09,
        NativeViewKey::Space => 0x20,
        NativeViewKey::Up => 0x26,
        NativeViewKey::Down => 0x28,
        NativeViewKey::Left => 0x25,
        NativeViewKey::Right => 0x27,
        NativeViewKey::Home => 0x24,
        NativeViewKey::End => 0x23,
        NativeViewKey::PageUp => 0x21,
        NativeViewKey::PageDown => 0x22,
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn win32_client_size(hwnd: windows_sys::Win32::Foundation::HWND) -> Result<crate::Size, String> {
    use windows_sys::Win32::{Foundation::RECT, UI::WindowsAndMessaging::GetClientRect};
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
        return Err("GetClientRect failed while observing the native resize".to_string());
    }
    Ok(crate::Size {
        width: (rect.right - rect.left).max(1),
        height: (rect.bottom - rect.top).max(1),
    })
}

#[cfg(all(windows, feature = "windows-win32"))]
fn request_win32_client_resize(
    hwnd: windows_sys::Win32::Foundation::HWND,
    requested: crate::Size,
) -> Result<crate::Size, String> {
    use std::ptr::null_mut;
    use windows_sys::Win32::{
        Foundation::RECT,
        UI::{
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            WindowsAndMessaging::{
                GetMenu, GetWindowLongW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE, SWP_NOACTIVATE,
                SWP_NOMOVE, SWP_NOZORDER,
            },
        },
    };

    let initial = win32_client_size(hwnd)?;
    let mut outer = RECT {
        left: 0,
        top: 0,
        right: requested.width.max(1),
        bottom: requested.height.max(1),
    };
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;
    let has_menu = i32::from(!unsafe { GetMenu(hwnd) }.is_null());
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    if unsafe { AdjustWindowRectExForDpi(&mut outer, style, has_menu, ex_style, dpi) } == 0 {
        return Err("AdjustWindowRectExForDpi failed for the native resize".to_string());
    }
    if unsafe {
        SetWindowPos(
            hwnd,
            null_mut(),
            0,
            0,
            (outer.right - outer.left).max(1),
            (outer.bottom - outer.top).max(1),
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    } == 0
    {
        return Err("SetWindowPos failed for the native resize".to_string());
    }
    Ok(initial)
}

#[cfg(all(windows, feature = "windows-win32"))]
fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    view_runtime: NativeViewInputRuntime,
    shell_runtime: Option<ZsShellRuntime>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    use std::{thread, time::Duration};
    use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }

    let mut report = NativeWindowSmokeRunReport {
        requested_window_count: windows.len(),
        window_menu_requested_count: windows
            .iter()
            .filter(|window| window.menu.is_some())
            .count(),
        window_menu_native_command_count: windows
            .iter()
            .filter_map(|window| window.menu.as_ref())
            .map(menu_command_count)
            .sum(),
        auto_close_after_ms: options.auto_close_after_ms,
        ..NativeWindowSmokeRunReport::empty(options.clone())
    };
    record_native_view_text_input_script_evidence(&mut report, &options.native_view_inputs);
    record_draw_plan_smoke(&mut report, &draw_plans);
    report.native_view_hit_target_count = view_runtime.hit_target_count();
    let input_routes =
        match crate::windows_win32_host::windows_win32_view_input_route(&view_runtime) {
            Some(route) => vec![Some(route)],
            None => Vec::new(),
        };
    let shell_routes = match shell_runtime {
        Some(runtime) => vec![Some(
            crate::windows_win32_host::WindowsWin32ShellInputRoute::new(runtime),
        )],
        None => Vec::new(),
    };
    let handles = crate::windows_win32_host::create_owned_windows_for_specs_with_routes(
        &windows,
        &draw_plans,
        &input_routes,
        &shell_routes,
    )
    .map_err(|err| {
        report.startup_error = Some(err.to_string());
        report.events.push("startup_error".to_string());
        err
    })?;

    report.created_window_count = handles.len();
    report.window_menu_attached_count = report.window_menu_requested_count;
    report.events.extend(
        windows
            .iter()
            .map(|spec| format!("window_created:{}", spec.title)),
    );

    if let Some(menu) = windows.first().and_then(|window| window.menu.as_ref()) {
        let table = crate::windows_win32_host::WindowsWin32StatusMenuCommandTable::from_menu(menu);
        if let Some(native_id) = table.first_native_id() {
            match crate::windows_win32_host::dispatch_windows_win32_window_menu_command(
                handles[0].main(),
                native_id,
            ) {
                Some(NativeStatusMenuCommandResult::Dispatched(command)) => {
                    report.window_menu_command_routed = true;
                    report
                        .events
                        .push(format!("window_menu_command_dispatched:{command:?}"));
                }
                Some(NativeStatusMenuCommandResult::Disabled) => {
                    report.window_menu_command_error =
                        Some("first window menu command is disabled".to_string());
                }
                Some(NativeStatusMenuCommandResult::NotFound) | None => {
                    report.window_menu_command_error =
                        Some("first window menu command was not found".to_string());
                }
            }
        }
    }

    let mut _status_item_host = None;
    if let Some(status_item) = options.status_item.clone() {
        let mut host =
            crate::windows_win32_host::WindowsWin32StatusItemHost::new(handles[0].main());
        match host.create_status_item(NativeStatusItemRequest::from_tray_spec(&status_item)) {
            NativeStatusItemPresentation::Created(handle) => {
                report.status_item_created = true;
                report.events.push(format!("status_item_created:{handle}"));
                report.events.push(format!(
                    "status_item_menu_items:{}",
                    status_item.menu.items.len()
                ));
                report.status_menu_native_command_count = host.native_menu_command_count(0);
                if let Some(native_command_id) = host.first_native_menu_command_id(0) {
                    match host.dispatch_native_menu_command(0, native_command_id) {
                        NativeStatusMenuCommandResult::Dispatched(command) => {
                            report.status_menu_command_routed = true;
                            report
                                .events
                                .push(format!("status_menu_command_dispatched:{command:?}"));
                        }
                        NativeStatusMenuCommandResult::Disabled => {
                            report.status_menu_command_error =
                                Some("first status menu command is disabled".to_string());
                            report
                                .events
                                .push("status_menu_command_disabled".to_string());
                        }
                        NativeStatusMenuCommandResult::NotFound => {
                            report.status_menu_command_error =
                                Some("first status menu command was not found".to_string());
                            report
                                .events
                                .push("status_menu_command_not_found".to_string());
                        }
                    }
                } else if !status_item.menu.items.is_empty() {
                    report.status_menu_command_error =
                        Some("status item menu has no native command entries".to_string());
                    report
                        .events
                        .push("status_menu_command_missing".to_string());
                }
                match host.create_popup_menu_for_status_item(0) {
                    Ok(popup_menu) => {
                        report.status_menu_popup_created = true;
                        report.status_menu_popup_command_count = popup_menu.command_entry_count();
                        report.events.push(format!(
                            "status_menu_popup_created:{}",
                            report.status_menu_popup_command_count
                        ));
                        report.status_menu_popup_destroyed = popup_menu.destroy();
                        if report.status_menu_popup_destroyed {
                            report
                                .events
                                .push("status_menu_popup_destroyed".to_string());
                        } else {
                            report.status_menu_popup_error =
                                Some("DestroyMenu failed for status popup menu".to_string());
                            report
                                .events
                                .push("status_menu_popup_destroy_error".to_string());
                        }
                    }
                    Err(err) => {
                        report.status_menu_popup_error = Some(err.to_string());
                        report.events.push("status_menu_popup_error".to_string());
                    }
                }
            }
            NativeStatusItemPresentation::Failed => {
                let error = host
                    .last_error()
                    .unwrap_or("Win32 status item creation failed")
                    .to_string();
                report.status_item_error = Some(error);
                report.events.push("status_item_error".to_string());
            }
        }
        _status_item_host = Some(host);
    }

    if options.native_view_inputs.is_empty() {
        let mut click_points = options.native_view_click_points.iter();
        if !options.native_view_text_inputs.is_empty() {
            if let Some(point) = click_points.next() {
                post_windows_native_view_input(
                    handles[0].main(),
                    &NativeViewSmokeInput::Click(*point),
                );
            }
        }
        for text in &options.native_view_text_inputs {
            post_windows_native_view_input(
                handles[0].main(),
                &NativeViewSmokeInput::Text(text.clone()),
            );
        }
        for point in click_points {
            post_windows_native_view_input(handles[0].main(), &NativeViewSmokeInput::Click(*point));
        }
        for key in &options.native_view_key_downs {
            post_windows_native_view_input(handles[0].main(), &NativeViewSmokeInput::KeyDown(*key));
        }
        for (point, delta_y) in &options.native_view_scroll_inputs {
            post_windows_native_view_input(
                handles[0].main(),
                &NativeViewSmokeInput::Scroll {
                    point: *point,
                    delta_y: *delta_y,
                },
            );
        }
    } else {
        for input in &options.native_view_inputs {
            post_windows_native_view_input(handles[0].main(), input);
        }
    }

    let close_handles: Vec<isize> = handles
        .iter()
        .map(|handles| handles.main() as isize)
        .collect();
    let auto_close_after = Duration::from_millis(options.auto_close_after_ms.max(1));
    let resize_request = options.native_window_resize;
    let screenshot_file = report.screenshot_file.clone();
    let screenshot_handle = handles[0].main() as isize;
    let typography_scale = draw_plans
        .first()
        .and_then(Option::as_ref)
        .map(NativeDrawPlan::typography_scale)
        .unwrap_or(1.0);
    let capture_delay = screenshot_file
        .as_ref()
        .map(|_| {
            Duration::from_millis(
                options
                    .auto_close_after_ms
                    .max(1)
                    .saturating_mul(3)
                    .checked_div(4)
                    .unwrap_or(1)
                    .max(1),
            )
        })
        .unwrap_or_default();
    let resize_delay = resize_request
        .map(|_| Duration::from_millis(options.auto_close_after_ms.max(1) / 2))
        .unwrap_or_default();
    let worker = thread::spawn(move || {
        if !resize_delay.is_zero() {
            thread::sleep(resize_delay);
        }
        let resize_start = resize_request.map(|requested| {
            let surface_events_before =
                crate::windows_win32_host::windows_win32_window_view_input_report(
                    screenshot_handle as _,
                )
                .map(|report| report.surface_change_count)
                .unwrap_or(0);
            request_win32_client_resize(screenshot_handle as _, requested)
                .map(|initial| (requested, initial, surface_events_before))
        });
        let capture_after_resize = capture_delay.saturating_sub(resize_delay);
        if !capture_after_resize.is_zero() {
            thread::sleep(capture_after_resize);
        }
        let screenshot_result = screenshot_file.map(|path| {
            let result = capture_win32_hwnd_png(screenshot_handle as _, &path, typography_scale);
            (path, result)
        });
        let elapsed = capture_delay.max(resize_delay);
        let remaining = auto_close_after.saturating_sub(elapsed);
        if !remaining.is_zero() {
            thread::sleep(remaining);
        }
        let resize_result = resize_start.map(|start| {
            start.and_then(|(requested, initial, surface_events_before)| {
                let final_size = win32_client_size(screenshot_handle as _)?;
                let surface_events_after =
                    crate::windows_win32_host::windows_win32_window_view_input_report(
                        screenshot_handle as _,
                    )
                    .map(|report| report.surface_change_count)
                    .unwrap_or(surface_events_before);
                Ok((
                    requested,
                    initial,
                    final_size,
                    surface_events_after.saturating_sub(surface_events_before),
                ))
            })
        });
        let process_memory =
            crate::desktop_runtime::capture_process_memory("native_window_before_teardown");
        for handle in close_handles {
            crate::windows_win32_host::approve_windows_win32_window_close(handle as _);
            unsafe {
                PostMessageW(handle as _, WM_CLOSE, 0, 0);
            }
        }
        (screenshot_result, process_memory, resize_result)
    });

    match crate::windows_win32_host::WindowsWin32MessageLoop::run_with_windows(&handles) {
        crate::windows_win32_host::WindowsWin32MessageLoopResult::Quit(_) => {
            report.exited_by_auto_close = true;
            report.close_requested_count = report.created_window_count;
            report.events.push("auto_close_elapsed".to_string());
        }
        crate::windows_win32_host::WindowsWin32MessageLoopResult::Failed => {
            report.startup_error = Some("GetMessageW failed".to_string());
            report.events.push("message_loop_error".to_string());
        }
    }

    match worker.join() {
        Ok((screenshot_result, process_memory, resize_result)) => {
            report.process_memory_during_runtime = process_memory;
            if let Some(resize_result) = resize_result {
                match resize_result {
                    Ok((requested_size, initial_size, final_size, native_event_count)) => {
                        let applied = final_size == requested_size && native_event_count > 0;
                        report.native_window_resize = Some(crate::NativeWindowResizeEvidence {
                            backend: "win32_set_window_pos_wm_size",
                            requested_size,
                            initial_size: Some(initial_size),
                            final_size: Some(final_size),
                            native_event_count,
                            applied,
                        });
                        if !applied {
                            report.native_window_resize_error = Some(format!(
                                "Win32 resize requested {}x{} but finished at {}x{} after {} WM_SIZE surface changes",
                                requested_size.width,
                                requested_size.height,
                                final_size.width,
                                final_size.height,
                                native_event_count
                            ));
                        }
                    }
                    Err(error) => report.native_window_resize_error = Some(error),
                }
            }
            match screenshot_result {
                Some((path, Ok(capture))) => {
                    report.screenshot_captured = true;
                    report.native_view_capture = Some(capture);
                    report.events.push(format!("screenshot_captured:{path}"));
                    report
                        .events
                        .push("screenshot_backend:win32_wm_printclient_dib_png".to_string());
                }
                Some((_path, Err(err))) => {
                    report.screenshot_error = Some(err);
                    report.events.push("screenshot_error".to_string());
                }
                None => {}
            }
        }
        Err(_) => {
            report.screenshot_error = Some("native smoke worker panicked".to_string());
            report.events.push("smoke_worker_error".to_string());
        }
    }

    for handles in &handles {
        if let Some(input_report) =
            crate::windows_win32_host::windows_win32_window_view_input_report(handles.main())
        {
            record_windows_win32_view_input_report(&mut report, &input_report);
        }
    }
    report.native_view_click_count += if options.native_view_inputs.is_empty() {
        options.native_view_click_points.len()
    } else {
        options
            .native_view_inputs
            .iter()
            .filter(|input| matches!(input, NativeViewSmokeInput::Click(_)))
            .count()
    };

    if options.require_visible_window && !report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot && !report.screenshot_captured {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_native_window_resize
        && !report
            .native_window_resize
            .as_ref()
            .is_some_and(|evidence| evidence.applied)
    {
        return Err(ZsuiError::host(
            "native_window_smoke_resize",
            report
                .native_window_resize_error
                .clone()
                .unwrap_or_else(|| "the requested Win32 resize was not observed".to_string()),
        ));
    }
    if options.require_status_item && !report.status_item_created {
        return Err(ZsuiError::host(
            "native_window_smoke_status_item",
            report
                .status_item_error
                .clone()
                .unwrap_or_else(|| "status item was not created".to_string()),
        ));
    }

    Ok(report)
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32WindowDeviceContext {
    hwnd: windows_sys::Win32::Foundation::HWND,
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32WindowDeviceContext {
    fn acquire(hwnd: windows_sys::Win32::Foundation::HWND) -> Result<Self, String> {
        let dc = unsafe { windows_sys::Win32::Graphics::Gdi::GetDC(hwnd) };
        if dc.is_null() {
            Err("GetDC failed".to_string())
        } else {
            Ok(Self { hwnd, dc })
        }
    }

    const fn hdc(&self) -> windows_sys::Win32::Graphics::Gdi::HDC {
        self.dc
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32WindowDeviceContext {
    fn drop(&mut self) {
        if !self.dc.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::ReleaseDC(self.hwnd, self.dc);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32CompatibleDeviceContext {
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32CompatibleDeviceContext {
    fn create(source: windows_sys::Win32::Graphics::Gdi::HDC) -> Result<Self, String> {
        let dc = unsafe { windows_sys::Win32::Graphics::Gdi::CreateCompatibleDC(source) };
        if dc.is_null() {
            Err("CreateCompatibleDC failed".to_string())
        } else {
            Ok(Self { dc })
        }
    }

    const fn hdc(&self) -> windows_sys::Win32::Graphics::Gdi::HDC {
        self.dc
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32CompatibleDeviceContext {
    fn drop(&mut self) {
        if !self.dc.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::DeleteDC(self.dc);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32CompatibleBitmap {
    bitmap: windows_sys::Win32::Graphics::Gdi::HBITMAP,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32CompatibleBitmap {
    fn create(
        dc: windows_sys::Win32::Graphics::Gdi::HDC,
        width: i32,
        height: i32,
    ) -> Result<Self, String> {
        let bitmap =
            unsafe { windows_sys::Win32::Graphics::Gdi::CreateCompatibleBitmap(dc, width, height) };
        if bitmap.is_null() {
            Err("CreateCompatibleBitmap failed".to_string())
        } else {
            Ok(Self { bitmap })
        }
    }

    const fn handle(&self) -> windows_sys::Win32::Graphics::Gdi::HBITMAP {
        self.bitmap
    }

    fn object(&self) -> windows_sys::Win32::Graphics::Gdi::HGDIOBJ {
        self.bitmap.cast()
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32CompatibleBitmap {
    fn drop(&mut self) {
        if !self.bitmap.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::DeleteObject(self.bitmap.cast());
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
struct Win32SelectedGdiObject {
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
    old: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Win32SelectedGdiObject {
    fn select(
        dc: windows_sys::Win32::Graphics::Gdi::HDC,
        object: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
    ) -> Option<Self> {
        if dc.is_null() || object.is_null() {
            return None;
        }
        let old = unsafe { windows_sys::Win32::Graphics::Gdi::SelectObject(dc, object) };
        if old.is_null() {
            None
        } else {
            Some(Self { dc, old })
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
impl Drop for Win32SelectedGdiObject {
    fn drop(&mut self) {
        if !self.dc.is_null() && !self.old.is_null() {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::SelectObject(self.dc, self.old);
            }
        }
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
fn capture_win32_hwnd_png(
    hwnd: windows_sys::Win32::Foundation::HWND,
    path: &str,
    typography_scale: f32,
) -> Result<crate::NativeViewCaptureEvidence, String> {
    use std::{ffi::c_void, mem, path::Path};
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD},
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{GetClientRect, SendMessageW, WM_PRINTCLIENT},
        },
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
    let window_dc = Win32WindowDeviceContext::acquire(hwnd)?;
    let memory_dc = Win32CompatibleDeviceContext::create(window_dc.hdc())?;
    let bitmap = Win32CompatibleBitmap::create(window_dc.hdc(), width, height)?;
    let selected_bitmap = Win32SelectedGdiObject::select(memory_dc.hdc(), bitmap.object());

    unsafe {
        SendMessageW(hwnd, WM_PRINTCLIENT, memory_dc.hdc() as usize, 0);
    }
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
    drop(selected_bitmap);
    let dib_lines = unsafe {
        GetDIBits(
            memory_dc.hdc(),
            bitmap.handle(),
            0,
            height as u32,
            bgra.as_mut_ptr().cast::<c_void>(),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };
    if dib_lines == 0 {
        return Err("GetDIBits failed".to_string());
    }

    let rgba = bgra_to_rgba(&bgra);
    let pixel_width = width as u32;
    let pixel_height = height as u32;
    write_rgba_png(Path::new(path), pixel_width, pixel_height, &rgba)?;
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    Ok(win32_capture_evidence(
        pixel_width,
        pixel_height,
        dpi,
        typography_scale,
    ))
}

#[cfg(all(windows, feature = "windows-win32"))]
fn win32_capture_evidence(
    pixel_width: u32,
    pixel_height: u32,
    dpi: u32,
    typography_scale: f32,
) -> crate::NativeViewCaptureEvidence {
    let scale_factor = f64::from(dpi.max(1)) / 96.0;
    let logical_width = (f64::from(pixel_width) / scale_factor).round().max(1.0) as u32;
    let logical_height = (f64::from(pixel_height) / scale_factor).round().max(1.0) as u32;
    crate::NativeViewCaptureEvidence {
        platform: "windows",
        backend: "win32_wm_printclient_dib_png",
        display_server: None,
        logical_width,
        logical_height,
        pixel_width,
        pixel_height,
        scale_factor,
        typography_scale,
        typography: crate::windows_gdi_renderer::windows_native_typography_profile()
            .with_typography_scale(typography_scale),
    }
}

#[cfg(all(windows, feature = "windows-win32"))]
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

#[cfg(all(windows, feature = "windows-win32"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_resources_are_drop_backed() {
        assert!(std::mem::needs_drop::<Win32WindowDeviceContext>());
        assert!(std::mem::needs_drop::<Win32CompatibleDeviceContext>());
        assert!(std::mem::needs_drop::<Win32CompatibleBitmap>());
        assert!(std::mem::needs_drop::<Win32SelectedGdiObject>());
        assert!(
            Win32SelectedGdiObject::select(std::ptr::null_mut(), std::ptr::null_mut()).is_none()
        );
    }

    #[test]
    fn capture_evidence_preserves_pixel_geometry_and_resolves_logical_dpi() {
        let evidence = win32_capture_evidence(1920, 1280, 192, 1.25);

        assert_eq!(evidence.platform, "windows");
        assert_eq!(evidence.backend, "win32_wm_printclient_dib_png");
        assert_eq!(evidence.logical_width, 960);
        assert_eq!(evidence.logical_height, 640);
        assert_eq!(evidence.pixel_width, 1920);
        assert_eq!(evidence.pixel_height, 1280);
        assert_eq!(evidence.scale_factor, 2.0);
        assert_eq!(evidence.typography_scale, 1.25);
        assert_eq!(evidence.typography.typography_scale, 1.25);
    }
}
