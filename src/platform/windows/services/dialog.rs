#[derive(Debug, Default)]
pub struct WindowsWin32FileDialogService;

impl FileDialogService for WindowsWin32FileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        windows_win32_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        windows_win32_save_file_dialog(spec)
    }
}

pub fn windows_win32_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    const FILE_BUFFER_LEN: usize = 32_768;
    let mut file_buffer = vec![0u16; FILE_BUFFER_LEN];
    let title = wide_null(&spec.title);
    let filter = windows_file_dialog_filter(&spec.filters);
    let initial_dir =
        native_file_dialog_initial_directory(spec.current_path.as_deref()).map(|path| {
            path.as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>()
        });
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = unsafe { GetActiveWindow() };
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = file_buffer.as_mut_ptr();
    dialog.nMaxFile = file_buffer.len() as u32;
    dialog.lpstrInitialDir = initial_dir
        .as_ref()
        .map(|path| path.as_ptr())
        .unwrap_or(null());
    dialog.lpstrTitle = title.as_ptr();
    dialog.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if spec.allow_multiple {
        dialog.Flags |= OFN_ALLOWMULTISELECT;
    }

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return windows_common_dialog_cancel_or_error("windows_open_file_dialog");
    }
    Ok(Some(parse_windows_open_file_buffer(&file_buffer)))
}

pub fn windows_win32_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    const FILE_BUFFER_LEN: usize = 32_768;
    let mut file_buffer = vec![0u16; FILE_BUFFER_LEN];
    let suggested_name = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    );
    if let Some(name) = suggested_name.as_deref() {
        let encoded = name.encode_utf16().collect::<Vec<_>>();
        if encoded.len() + 1 > file_buffer.len() {
            return Err(ZsuiError::invalid_spec(
                "save_file_dialog.suggested_name",
                "suggested file name is too long",
            ));
        }
        file_buffer[..encoded.len()].copy_from_slice(&encoded);
    }
    let title = wide_null(&spec.title);
    let filter = windows_file_dialog_filter(&spec.filters);
    let initial_dir =
        native_file_dialog_initial_directory(spec.current_path.as_deref()).map(|path| {
            path.as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>()
        });
    let default_extension = windows_file_dialog_default_extension(&spec.filters);
    let mut dialog: OPENFILENAMEW = unsafe { zeroed() };
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = unsafe { GetActiveWindow() };
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = file_buffer.as_mut_ptr();
    dialog.nMaxFile = file_buffer.len() as u32;
    dialog.lpstrInitialDir = initial_dir
        .as_ref()
        .map(|path| path.as_ptr())
        .unwrap_or(null());
    dialog.lpstrTitle = title.as_ptr();
    dialog.lpstrDefExt = default_extension
        .as_ref()
        .map(|extension| extension.as_ptr())
        .unwrap_or(null());
    dialog.Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;

    if unsafe { GetSaveFileNameW(&mut dialog) } == 0 {
        return windows_common_dialog_cancel_or_error("windows_save_file_dialog");
    }
    Ok(parse_windows_utf16_segments(&file_buffer)
        .into_iter()
        .next()
        .map(PathBuf::from))
}

#[derive(Debug, Default)]
pub struct WindowsWin32DialogService;

impl NativeDialogService for WindowsWin32DialogService {
    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        windows_win32_show_native_dialog(spec)
    }
}

pub fn windows_win32_show_native_dialog(
    spec: &NativeDialogSpec,
) -> ZsuiResult<DialogResponse> {
    let owner = unsafe { GetActiveWindow() };
    let mut style = windows_native_dialog_button_style(spec.buttons)
        | windows_native_dialog_level_style(spec.level);
    if owner.is_null() {
        style |= MB_TASKMODAL;
    }
    let title = wide_null(&spec.title);
    let message = wide_null(&spec.message);
    let response = unsafe { MessageBoxW(owner, message.as_ptr(), title.as_ptr(), style) };
    match response {
        IDOK => Ok(DialogResponse::Ok),
        IDCANCEL => Ok(DialogResponse::Cancel),
        IDYES => Ok(DialogResponse::Yes),
        IDNO => Ok(DialogResponse::No),
        0 => Err(ZsuiError::host(
            "windows_native_dialog",
            format!("MessageBoxW failed with Win32 error {}", unsafe {
                GetLastError()
            }),
        )),
        other => Err(ZsuiError::host(
            "windows_native_dialog",
            format!("MessageBoxW returned unexpected response {other}"),
        )),
    }
}

fn windows_native_dialog_button_style(buttons: DialogButtons) -> u32 {
    match buttons {
        DialogButtons::Ok => MB_OK,
        DialogButtons::OkCancel => MB_OKCANCEL,
        DialogButtons::YesNo => MB_YESNO,
        DialogButtons::YesNoCancel => MB_YESNOCANCEL,
    }
}

fn windows_native_dialog_level_style(level: DialogLevel) -> u32 {
    match level {
        DialogLevel::Info | DialogLevel::Question => MB_ICONINFORMATION,
        DialogLevel::Warning => MB_ICONWARNING,
        DialogLevel::Error => MB_ICONERROR,
    }
}

fn windows_common_dialog_cancel_or_error<T>(operation: &'static str) -> ZsuiResult<Option<T>> {
    let error = unsafe { CommDlgExtendedError() };
    if error == 0 {
        Ok(None)
    } else {
        Err(ZsuiError::host(
            operation,
            format!("common dialog error 0x{error:08x}"),
        ))
    }
}

fn windows_file_dialog_filter(filters: &[crate::FileDialogFilter]) -> Vec<u16> {
    let mut output = Vec::new();
    if filters.is_empty() {
        append_windows_filter_part(&mut output, "All files");
        append_windows_filter_part(&mut output, "*.*");
    } else {
        for filter in filters {
            append_windows_filter_part(&mut output, &filter.name);
            let patterns = if filter.patterns.is_empty() {
                "*.*".to_string()
            } else {
                filter.patterns.join(";")
            };
            append_windows_filter_part(&mut output, &patterns);
        }
    }
    output.push(0);
    output
}

fn append_windows_filter_part(output: &mut Vec<u16>, value: &str) {
    output.extend(value.encode_utf16());
    output.push(0);
}

fn windows_file_dialog_default_extension(filters: &[crate::FileDialogFilter]) -> Option<Vec<u16>> {
    filters
        .iter()
        .flat_map(|filter| &filter.patterns)
        .find_map(|pattern| {
            pattern
                .strip_prefix("*.")
                .filter(|extension| !extension.is_empty() && !extension.contains(['*', '?', ';']))
        })
        .map(|extension| extension.encode_utf16().chain(Some(0)).collect())
}

fn parse_windows_open_file_buffer(buffer: &[u16]) -> Vec<PathBuf> {
    let parts = parse_windows_utf16_segments(buffer);
    match parts.as_slice() {
        [] => Vec::new(),
        [path] => vec![PathBuf::from(path)],
        [directory, names @ ..] => names
            .iter()
            .map(|name| PathBuf::from(directory).join(name))
            .collect(),
    }
}

fn parse_windows_utf16_segments(buffer: &[u16]) -> Vec<OsString> {
    buffer
        .split(|unit| *unit == 0)
        .take_while(|segment| !segment.is_empty())
        .map(OsString::from_wide)
        .collect()
}
