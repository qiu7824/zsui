#[cfg(feature = "clipboard")]
pub(crate) fn windows_read_clipboard() -> ZsuiResult<Option<crate::ClipboardData>> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| ZsuiError::host("windows_read_clipboard", error.to_string()))?;
    match clipboard.get_text() {
        Ok(text) => Ok(Some(crate::ClipboardData::Text(text))),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(error) => Err(ZsuiError::host("windows_read_clipboard", error.to_string())),
    }
}

#[cfg(feature = "clipboard")]
pub(crate) fn windows_write_clipboard(data: &crate::ClipboardData) -> ZsuiResult<()> {
    let text = match data {
        crate::ClipboardData::Empty => String::new(),
        crate::ClipboardData::Text(text) => text.clone(),
        crate::ClipboardData::ImageRgba { .. } => {
            return Err(ZsuiError::unsupported(
                "clipboard_image",
                "the native image clipboard service is not connected",
            ));
        }
        crate::ClipboardData::Files(_) => {
            return Err(ZsuiError::unsupported(
                "clipboard_files",
                "the native file clipboard service is not connected",
            ));
        }
    };
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| ZsuiError::host("windows_write_clipboard", error.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|error| ZsuiError::host("windows_write_clipboard", error.to_string()))
}
