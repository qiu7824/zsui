#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, time::Duration};

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    let auto_close = benchmark_timeout();
    tauri::Builder::default()
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                window.destroy()?;
            }
            WebviewWindowBuilder::new(app, "invoice", WebviewUrl::App("invoice.html".into()))
                .title("发票工作台 · Tauri 2")
                .inner_size(1000.0, 700.0)
                .min_inner_size(820.0, 560.0)
                .center()
                .build()?;
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
        .expect("Tauri invoice runtime failed");
}

fn benchmark_timeout() -> Option<Duration> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs)
}
