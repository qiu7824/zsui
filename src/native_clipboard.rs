use crate::{ClipboardData, ZsuiError, ZsuiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeClipboardTextWrite<'a> {
    Clear,
    Text(&'a str),
}

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
