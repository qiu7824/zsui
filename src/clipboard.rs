use serde::{Deserialize, Serialize};

use crate::{ZsuiError, ZsuiResult};

const RGBA_CHANNEL_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardData {
    Empty,
    Text(String),
    ImageRgba {
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    },
    Files(Vec<String>),
}

impl ClipboardData {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn files(paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Files(paths.into_iter().map(Into::into).collect())
    }

    pub fn image_rgba(width: usize, height: usize, bytes: impl Into<Vec<u8>>) -> ZsuiResult<Self> {
        let bytes = bytes.into();
        Self::validate_image_rgba(width, height, &bytes)?;
        Ok(Self::ImageRgba {
            width,
            height,
            bytes,
        })
    }

    pub(crate) fn validate_image_rgba(width: usize, height: usize, bytes: &[u8]) -> ZsuiResult<()> {
        if width == 0 || height == 0 {
            return Err(ZsuiError::invalid_spec(
                "clipboard.image",
                "RGBA clipboard image dimensions must be greater than zero",
            ));
        }
        let expected = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(RGBA_CHANNEL_COUNT))
            .ok_or_else(|| {
                ZsuiError::invalid_spec(
                    "clipboard.image",
                    "RGBA clipboard image dimensions overflow addressable storage",
                )
            })?;
        if bytes.len() != expected {
            return Err(ZsuiError::invalid_spec(
                "clipboard.image",
                format!(
                    "RGBA clipboard image requires {expected} bytes for {width}x{height}, received {}",
                    bytes.len()
                ),
            ));
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_clipboard_images_require_exact_nonzero_storage() {
        assert!(ClipboardData::image_rgba(2, 1, [0; 8]).is_ok());
        assert!(ClipboardData::image_rgba(0, 1, []).is_err());
        assert!(ClipboardData::image_rgba(1, 0, []).is_err());
        assert!(ClipboardData::image_rgba(2, 1, [0; 7]).is_err());
        assert!(ClipboardData::image_rgba(usize::MAX, 2, []).is_err());
    }
}
