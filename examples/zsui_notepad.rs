#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "zsui_notepad/document.rs"]
mod document;

#[cfg(windows)]
#[path = "zsui_notepad/windows.rs"]
mod windows;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows::run().map_err(Into::into)
}

#[cfg(not(windows))]
fn main() {
    eprintln!("The current ZSUI notepad demo uses the Windows native text service.");
}
