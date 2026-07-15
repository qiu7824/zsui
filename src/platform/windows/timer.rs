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
