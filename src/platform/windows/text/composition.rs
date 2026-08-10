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

unsafe fn windows_ime_composition_selection(
    hwnd: HWND,
    text: &str,
) -> Option<(usize, usize)> {
    let context = ImmGetContext(hwnd);
    if context.is_null() {
        return None;
    }
    let cursor = ImmGetCompositionStringW(context, GCS_CURSORPOS, null_mut(), 0);
    ImmReleaseContext(hwnd, context);
    if cursor < 0 {
        None
    } else {
        let cursor = windows_utf16_offset_to_char_index(text, cursor as usize);
        Some((cursor, cursor))
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
    let caret = windows_win32_window_ime_caret_rect(hwnd).unwrap_or(target.bounds);
    let form = CANDIDATEFORM {
        dwIndex: 0,
        dwStyle: CFS_EXCLUDE,
        ptCurrentPos: POINT {
            x: caret.x,
            y: caret.y + caret.height,
        },
        rcArea: RECT {
            left: caret.x,
            top: caret.y,
            right: caret.x + caret.width.max(1),
            bottom: caret.y + caret.height.max(1),
        },
    };
    ImmSetCandidateWindow(context, &form);
    ImmReleaseContext(hwnd, context);
}
