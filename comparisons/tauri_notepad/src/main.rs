#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Serialize;
use tauri::WebviewWindow;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentPayload {
    path: String,
    display_name: String,
    text: String,
}

#[tauri::command]
fn open_document() -> Result<Option<DocumentPayload>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md", "log"])
        .pick_file()
    else {
        return Ok(None);
    };
    let text = read_text(&path)?;
    Ok(Some(DocumentPayload {
        display_name: display_name(&path),
        path: path.to_string_lossy().into_owned(),
        text,
    }))
}

#[tauri::command]
fn save_document(
    path: Option<String>,
    text: String,
    force_picker: bool,
) -> Result<Option<DocumentPayload>, String> {
    let current = path.map(PathBuf::from);
    let path = if force_picker || current.is_none() {
        let suggested = current
            .as_deref()
            .map(display_name)
            .unwrap_or_else(|| "Untitled.txt".to_string());
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md", "log"])
            .set_file_name(suggested)
            .save_file()
        else {
            return Ok(None);
        };
        path
    } else {
        current.expect("path checked above")
    };
    fs::write(&path, text.as_bytes()).map_err(|error| error.to_string())?;
    Ok(Some(DocumentPayload {
        display_name: display_name(&path),
        path: path.to_string_lossy().into_owned(),
        text,
    }))
}

#[tauri::command]
fn set_window_title(window: WebviewWindow, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|error| error.to_string())
}

fn main() {
    let auto_close = benchmark_timeout();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_document,
            save_document,
            set_window_title
        ])
        .setup(move |app| {
            if let Some(duration) = auto_close {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(duration);
                    handle.exit(0);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed");
}

fn benchmark_timeout() -> Option<Duration> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs)
        .or_else(|| {
            arguments
                .iter()
                .any(|argument| argument == "--smoke")
                .then_some(Duration::from_millis(1200))
        })
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn read_text(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(rest.to_vec()).map_err(|error| error.to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        if rest.len() % 2 != 0 {
            return Err("UTF-16 file has an odd byte length".to_string());
        }
        let units = rest
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units).map_err(|error| error.to_string());
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}
