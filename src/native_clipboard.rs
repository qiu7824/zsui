use crate::{ClipboardData, ZsuiError, ZsuiResult};

#[cfg(any(
    test,
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeClipboardTextWrite<'a> {
    Clear,
    Text(&'a str),
}

#[cfg(any(
    test,
    all(target_os = "macos", feature = "macos-appkit"),
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
pub(crate) fn native_clipboard_text_write(
    data: &ClipboardData,
) -> ZsuiResult<NativeClipboardTextWrite<'_>> {
    match data {
        ClipboardData::Empty => Ok(NativeClipboardTextWrite::Clear),
        ClipboardData::Text(text) => Ok(NativeClipboardTextWrite::Text(text)),
        ClipboardData::ImageRgba { .. } => Err(ZsuiError::unsupported(
            "clipboard_image",
            "the native image clipboard service is not connected",
        )),
        ClipboardData::Files(_) => Err(ZsuiError::unsupported(
            "clipboard_files",
            "the native file clipboard service is not connected",
        )),
    }
}

#[cfg(feature = "clipboard")]
fn arboard_read_image(
    operation: &'static str,
    clipboard: &mut arboard::Clipboard,
) -> ZsuiResult<Option<ClipboardData>> {
    const IMAGE_READ_ATTEMPTS: usize = 4;
    const IMAGE_READ_RETRY_MS: u64 = 8;

    for attempt in 0..IMAGE_READ_ATTEMPTS {
        match clipboard.get_image() {
            Ok(image) => {
                let bytes = image.bytes.into_owned();
                return ClipboardData::image_rgba(image.width, image.height, bytes).map(Some);
            }
            Err(arboard::Error::ContentNotAvailable) if attempt + 1 < IMAGE_READ_ATTEMPTS => {
                std::thread::sleep(std::time::Duration::from_millis(IMAGE_READ_RETRY_MS));
            }
            Err(arboard::Error::ContentNotAvailable) => return Ok(None),
            Err(error) => return Err(ZsuiError::host(operation, error.to_string())),
        }
    }

    Ok(None)
}

#[cfg(feature = "clipboard")]
pub(crate) fn arboard_read_clipboard(operation: &'static str) -> ZsuiResult<Option<ClipboardData>> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| ZsuiError::host(operation, error.to_string()))?;
    match clipboard.get_text() {
        Ok(text) => Ok(Some(ClipboardData::Text(text))),
        Err(arboard::Error::ContentNotAvailable) => arboard_read_image(operation, &mut clipboard),
        Err(error) => Err(ZsuiError::host(operation, error.to_string())),
    }
}

#[cfg(all(
    feature = "clipboard",
    any(
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )
))]
pub(crate) fn arboard_read_clipboard_image(
    operation: &'static str,
) -> ZsuiResult<Option<ClipboardData>> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| ZsuiError::host(operation, error.to_string()))?;
    arboard_read_image(operation, &mut clipboard)
}

#[cfg(all(
    not(feature = "clipboard"),
    any(
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )
))]
pub(crate) fn arboard_read_clipboard_image(
    _operation: &'static str,
) -> ZsuiResult<Option<ClipboardData>> {
    Err(ZsuiError::unsupported(
        "clipboard_image",
        "enable the clipboard feature to compile native image clipboard support",
    ))
}

#[cfg(feature = "clipboard")]
pub(crate) fn arboard_write_clipboard(
    operation: &'static str,
    data: &ClipboardData,
) -> ZsuiResult<()> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| ZsuiError::host(operation, error.to_string()))?;
    match data {
        ClipboardData::Empty => clipboard
            .clear()
            .map_err(|error| ZsuiError::host(operation, error.to_string())),
        ClipboardData::Text(text) => clipboard
            .set_text(text.clone())
            .map_err(|error| ZsuiError::host(operation, error.to_string())),
        ClipboardData::ImageRgba {
            width,
            height,
            bytes,
        } => {
            ClipboardData::validate_image_rgba(*width, *height, bytes)?;
            clipboard
                .set_image(arboard::ImageData {
                    width: *width,
                    height: *height,
                    bytes: std::borrow::Cow::Borrowed(bytes),
                })
                .map_err(|error| ZsuiError::host(operation, error.to_string()))
        }
        ClipboardData::Files(_) => Err(ZsuiError::unsupported(
            "clipboard_files",
            "the native file clipboard service is not connected",
        )),
    }
}

#[cfg(all(
    not(feature = "clipboard"),
    any(
        target_os = "macos",
        all(target_os = "linux", not(target_env = "ohos"))
    )
))]
pub(crate) fn arboard_write_clipboard(
    _operation: &'static str,
    data: &ClipboardData,
) -> ZsuiResult<()> {
    match data {
        ClipboardData::ImageRgba { .. } => Err(ZsuiError::unsupported(
            "clipboard_image",
            "enable the clipboard feature to compile native image clipboard support",
        )),
        ClipboardData::Files(_) => Err(ZsuiError::unsupported(
            "clipboard_files",
            "the native file clipboard service is not connected",
        )),
        ClipboardData::Empty | ClipboardData::Text(_) => Err(ZsuiError::unsupported(
            "clipboard_text",
            "enable the clipboard feature to compile native text clipboard support",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_clipboard_write_contract_separates_clear_text_and_rich_data() {
        assert_eq!(
            native_clipboard_text_write(&ClipboardData::Empty).unwrap(),
            NativeClipboardTextWrite::Clear
        );
        assert_eq!(
            native_clipboard_text_write(&ClipboardData::text("ZSUI")).unwrap(),
            NativeClipboardTextWrite::Text("ZSUI")
        );
        assert!(native_clipboard_text_write(&ClipboardData::files(["notes.txt"])).is_err());
        assert!(native_clipboard_text_write(&ClipboardData::ImageRgba {
            width: 1,
            height: 1,
            bytes: vec![0, 0, 0, 255],
        })
        .is_err());
    }
}
