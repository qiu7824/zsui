#[cfg(feature = "clipboard")]
pub(crate) fn windows_read_clipboard() -> ZsuiResult<Option<crate::ClipboardData>> {
    crate::native_clipboard::arboard_read_clipboard("windows_read_clipboard")
}

#[cfg(feature = "clipboard")]
pub(crate) fn windows_write_clipboard(data: &crate::ClipboardData) -> ZsuiResult<()> {
    crate::native_clipboard::arboard_write_clipboard("windows_write_clipboard", data)
}
