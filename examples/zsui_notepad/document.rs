use std::{fs, path::PathBuf};

#[derive(Debug, Default)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    pub dirty: bool,
}

impl Document {
    pub fn untitled(initial_text: impl Into<String>) -> Self {
        Self {
            path: None,
            text: initial_text.into(),
            dirty: false,
        }
    }

    pub fn open(path: PathBuf) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|error| error.to_string())?;
        Ok(Self {
            path: Some(path),
            text: decode_text(&bytes)?,
            dirty: false,
        })
    }

    pub fn save(&mut self) -> Result<(), String> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| "document has no save path".to_string())?;
        fs::write(path, self.text.as_bytes()).map_err(|error| error.to_string())?;
        self.dirty = false;
        Ok(())
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<(), String> {
        self.path = Some(path);
        self.save()
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

fn decode_text(bytes: &[u8]) -> Result<String, String> {
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(rest.to_vec()).map_err(|error| error.to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        return decode_utf16(rest, u16::from_le_bytes);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        return decode_utf16(rest, u16::from_be_bytes);
    }
    String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("the file is not valid UTF-8 or BOM-tagged UTF-16: {error}"))
}

fn decode_utf16(bytes: &[u8], decode: fn([u8; 2]) -> u16) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("UTF-16 file has an odd byte length".to_string());
    }
    let units = bytes
        .chunks_exact(2)
        .map(|pair| decode([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&units).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::decode_text;

    #[test]
    fn decodes_utf8_and_utf16_bom_documents() {
        assert_eq!(decode_text(b"hello").unwrap(), "hello");
        assert_eq!(decode_text(b"\xef\xbb\xbfhello").unwrap(), "hello");
        assert_eq!(decode_text(&[0xff, 0xfe, b'h', 0, b'i', 0]).unwrap(), "hi");
    }
}
