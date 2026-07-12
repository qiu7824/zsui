use std::path::{Path, PathBuf};

use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel, NSSavePanel};
use objc2_foundation::{NSArray, NSString, NSURL};

use crate::native_file_dialog::{
    native_file_dialog_extensions, native_file_dialog_initial_directory,
    native_save_dialog_suggested_name,
};
use crate::{FileDialogService, FileDialogSpec, SaveFileDialogSpec, ZsuiError, ZsuiResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct MacosAppKitFileDialogService;

impl FileDialogService for MacosAppKitFileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        macos_appkit_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        macos_appkit_save_file_dialog(spec)
    }
}

pub fn macos_appkit_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    let mtm = appkit_main_thread_marker("NSOpenPanel")?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(spec.allow_multiple);
    panel.setTitle(Some(&NSString::from_str(&spec.title)));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    if let Some(allowed) = appkit_allowed_file_types(&spec.filters) {
        #[allow(deprecated)]
        panel.setAllowedFileTypes(Some(&allowed));
    }
    appkit_set_initial_directory(&panel, spec.current_path.as_deref().map(Path::new));

    if panel.runModal() != NSModalResponseOK {
        return Ok(None);
    }

    let urls = panel.URLs();
    let mut paths = Vec::with_capacity(urls.len());
    for index in 0..urls.len() {
        let url = unsafe { urls.objectAtIndex_unchecked(index) };
        let path = url.to_file_path().ok_or_else(|| {
            ZsuiError::host(
                "macos_open_file_dialog",
                "NSOpenPanel returned a non-file URL",
            )
        })?;
        paths.push(path);
    }
    if paths.is_empty() {
        return Err(ZsuiError::host(
            "macos_open_file_dialog",
            "NSOpenPanel accepted without returning a selected file",
        ));
    }
    Ok(Some(paths))
}

pub fn macos_appkit_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    let mtm = appkit_main_thread_marker("NSSavePanel")?;
    let panel = NSSavePanel::savePanel(mtm);
    panel.setCanCreateDirectories(true);
    panel.setTitle(Some(&NSString::from_str(&spec.title)));
    panel.setPrompt(Some(&NSString::from_str("Save")));
    if let Some(name) = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    ) {
        panel.setNameFieldStringValue(&NSString::from_str(&name));
    }
    if let Some(allowed) = appkit_allowed_file_types(&spec.filters) {
        #[allow(deprecated)]
        panel.setAllowedFileTypes(Some(&allowed));
    }
    appkit_set_initial_directory(&panel, spec.current_path.as_deref());

    if panel.runModal() != NSModalResponseOK {
        return Ok(None);
    }
    panel
        .URL()
        .map(|url| {
            url.to_file_path().ok_or_else(|| {
                ZsuiError::host(
                    "macos_save_file_dialog",
                    "NSSavePanel returned a non-file URL",
                )
            })
        })
        .transpose()
}

fn appkit_main_thread_marker(operation: &'static str) -> ZsuiResult<MainThreadMarker> {
    MainThreadMarker::new().ok_or_else(|| {
        ZsuiError::host(
            operation,
            "AppKit file dialogs must be presented from the macOS main thread",
        )
    })
}

fn appkit_allowed_file_types(
    filters: &[crate::FileDialogFilter],
) -> Option<Retained<NSArray<NSString>>> {
    let values = native_file_dialog_extensions(filters)
        .into_iter()
        .map(|extension| NSString::from_str(&extension))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let references = values.iter().map(AsRef::as_ref).collect::<Vec<_>>();
    Some(NSArray::from_slice(&references))
}

fn appkit_set_initial_directory(panel: &NSSavePanel, current_path: Option<&Path>) {
    let Some(directory) = native_file_dialog_initial_directory(current_path) else {
        return;
    };
    if let Some(url) = NSURL::from_directory_path(directory) {
        panel.setDirectoryURL(Some(&url));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appkit_file_dialog_service_implements_safe_public_contract() {
        fn assert_service<T: FileDialogService>() {}
        assert_service::<MacosAppKitFileDialogService>();
    }
}
