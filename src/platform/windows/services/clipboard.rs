#[cfg(feature = "clipboard")]
pub(crate) fn windows_read_clipboard() -> ZsuiResult<Option<crate::ClipboardData>> {
    #[cfg(feature = "windows-shell")]
    if let Some(files) = shell::read_file_clipboard()? {
        return Ok(Some(files));
    }
    crate::native_clipboard::arboard_read_clipboard("windows_read_clipboard")
}

#[cfg(feature = "clipboard")]
pub(crate) fn windows_write_clipboard(data: &crate::ClipboardData) -> ZsuiResult<()> {
    #[cfg(feature = "windows-shell")]
    if let crate::ClipboardData::Files(files) = data {
        return shell::write_file_clipboard(files);
    }
    crate::native_clipboard::arboard_write_clipboard("windows_write_clipboard", data)
}
