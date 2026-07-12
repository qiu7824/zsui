use std::path::{Path, PathBuf};

#[cfg(any(test, target_os = "macos"))]
use crate::FileDialogFilter;

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn native_file_dialog_extensions(filters: &[FileDialogFilter]) -> Vec<String> {
    let mut extensions = Vec::new();
    for pattern in filters.iter().flat_map(|filter| &filter.patterns) {
        let pattern = pattern.trim();
        if pattern.is_empty() || matches!(pattern, "*" | "*.*") {
            continue;
        }
        let extension = pattern
            .strip_prefix("*.")
            .or_else(|| pattern.strip_prefix('.'))
            .unwrap_or(pattern)
            .trim();
        if extension.is_empty()
            || extension
                .chars()
                .any(|character| matches!(character, '*' | '?' | '/' | '\\'))
            || extensions
                .iter()
                .any(|known: &String| known.eq_ignore_ascii_case(extension))
        {
            continue;
        }
        extensions.push(extension.to_string());
    }
    extensions
}

pub(crate) fn native_file_dialog_initial_directory(current_path: Option<&Path>) -> Option<PathBuf> {
    let path = current_path?.to_path_buf();
    if path.as_os_str().is_empty() {
        return None;
    }
    if path.is_dir() {
        return Some(path);
    }
    if path.is_file() || path.extension().is_some() {
        return path.parent().map(Path::to_path_buf);
    }
    Some(path)
}

pub(crate) fn native_save_dialog_suggested_name<'a>(
    suggested_name: Option<&'a str>,
    current_path: Option<&'a Path>,
) -> Option<String> {
    suggested_name
        .filter(|name| !name.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            current_path
                .and_then(Path::file_name)
                .map(|name| name.to_string_lossy().into_owned())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_extensions_drop_wildcards_invalid_patterns_and_duplicates() {
        let filters = vec![
            FileDialogFilter::new("Text", ["*.txt", ".MD", "*.tar.gz"]),
            FileDialogFilter::new("All", ["*.*", "*.TXT", "folder/*.rs"]),
        ];

        assert_eq!(
            native_file_dialog_extensions(&filters),
            vec!["txt", "MD", "tar.gz"]
        );
    }

    #[test]
    fn native_initial_directory_and_suggested_name_handle_file_paths() {
        let path = Path::new("workspace/notes/readme.md");

        assert_eq!(
            native_file_dialog_initial_directory(Some(path)),
            Some(PathBuf::from("workspace/notes"))
        );
        assert_eq!(
            native_save_dialog_suggested_name(None, Some(path)).as_deref(),
            Some("readme.md")
        );
        assert_eq!(
            native_save_dialog_suggested_name(Some("draft.txt"), Some(path)).as_deref(),
            Some("draft.txt")
        );
    }
}
