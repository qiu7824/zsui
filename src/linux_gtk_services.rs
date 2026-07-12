use std::path::{Path, PathBuf};

use gtk::gio;
use gtk::glib::MainContext;
use gtk::prelude::*;
use gtk::{FileChooserAction, FileChooserNative, FileFilter, ResponseType};
use gtk4 as gtk;

use crate::native_file_dialog::{
    native_file_dialog_initial_directory, native_save_dialog_suggested_name,
};
use crate::{FileDialogService, FileDialogSpec, SaveFileDialogSpec, ZsuiError, ZsuiResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxGtkFileDialogService;

impl FileDialogService for LinuxGtkFileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        linux_gtk_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        linux_gtk_save_file_dialog(spec)
    }
}

pub fn linux_gtk_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    ensure_gtk_main_thread("gtk_open_file_dialog")?;
    let dialog = FileChooserNative::builder()
        .title(&spec.title)
        .action(FileChooserAction::Open)
        .accept_label("Open")
        .cancel_label("Cancel")
        .modal(true)
        .select_multiple(spec.allow_multiple)
        .build();
    add_gtk_file_filters(&dialog, &spec.filters);
    if let Some(directory) =
        native_file_dialog_initial_directory(spec.current_path.as_deref().map(Path::new))
    {
        let _ = dialog.set_current_folder(Some(&gio::File::for_path(directory)));
    }

    let response = MainContext::default().block_on(dialog.run_future());
    let result = if response == ResponseType::Accept {
        gtk_selected_local_paths(&dialog).map(Some)
    } else {
        Ok(None)
    };
    dialog.destroy();
    result
}

pub fn linux_gtk_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    ensure_gtk_main_thread("gtk_save_file_dialog")?;
    let dialog = FileChooserNative::builder()
        .title(&spec.title)
        .action(FileChooserAction::Save)
        .accept_label("Save")
        .cancel_label("Cancel")
        .modal(true)
        .select_multiple(false)
        .build();
    add_gtk_file_filters(&dialog, &spec.filters);
    if let Some(directory) = native_file_dialog_initial_directory(spec.current_path.as_deref()) {
        let _ = dialog.set_current_folder(Some(&gio::File::for_path(directory)));
    }
    if let Some(name) = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    ) {
        dialog.set_current_name(&name);
    }

    let response = MainContext::default().block_on(dialog.run_future());
    let result = if response == ResponseType::Accept {
        (|| {
            let file = dialog.file().ok_or_else(|| {
                ZsuiError::host(
                    "gtk_save_file_dialog",
                    "GTK file chooser returned no selected file",
                )
            })?;
            let path = file.path().ok_or_else(|| {
                ZsuiError::host(
                    "gtk_save_file_dialog",
                    "GTK file chooser returned a non-local file",
                )
            })?;
            Ok(Some(path))
        })()
    } else {
        Ok(None)
    };
    dialog.destroy();
    result
}

pub(crate) fn ensure_gtk_main_thread(operation: &'static str) -> ZsuiResult<()> {
    if gtk::is_initialized() && !gtk::is_initialized_main_thread() {
        return Err(ZsuiError::host(
            operation,
            "GTK file dialogs must be presented from the GTK main thread",
        ));
    }
    if !gtk::is_initialized_main_thread() {
        gtk::init().map_err(|error| ZsuiError::host(operation, error.to_string()))?;
    }
    Ok(())
}

fn add_gtk_file_filters(dialog: &FileChooserNative, filters: &[crate::FileDialogFilter]) {
    for filter_spec in filters {
        if filter_spec.patterns.is_empty() {
            continue;
        }
        let filter = FileFilter::new();
        filter.set_name(Some(&filter_spec.name));
        for pattern in &filter_spec.patterns {
            filter.add_pattern(pattern);
        }
        dialog.add_filter(&filter);
    }
}

fn gtk_selected_local_paths(dialog: &FileChooserNative) -> ZsuiResult<Vec<PathBuf>> {
    let files = dialog.files();
    let mut paths = Vec::with_capacity(files.n_items() as usize);
    for index in 0..files.n_items() {
        let file = files
            .item(index)
            .and_then(|item| item.downcast::<gio::File>().ok())
            .ok_or_else(|| {
                ZsuiError::host(
                    "gtk_open_file_dialog",
                    "GTK file chooser returned an invalid file object",
                )
            })?;
        let path = file.path().ok_or_else(|| {
            ZsuiError::host(
                "gtk_open_file_dialog",
                "GTK file chooser returned a non-local file",
            )
        })?;
        paths.push(path);
    }
    if paths.is_empty() {
        return Err(ZsuiError::host(
            "gtk_open_file_dialog",
            "GTK file chooser accepted without returning a selected file",
        ));
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk_file_dialog_service_implements_safe_public_contract() {
        fn assert_service<T: FileDialogService>() {}
        assert_service::<LinuxGtkFileDialogService>();
    }
}
