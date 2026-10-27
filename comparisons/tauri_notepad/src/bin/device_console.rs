#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    let devices: Vec<_> = model::DEVICES.iter().enumerate().map(|(id, d)| {
        serde_json::json!({"id":id,"name":d.name,"kind":d.kind,"online":d.online,"latency":d.latency})
    }).collect();
    let initialization = format!(
        "window.DEVICE_DEMO = {};",
        serde_json::json!({
            "devices":devices,"empty":model::has_flag("--benchmark-empty"),"repaint":model::has_flag("--benchmark-repaint")
        })
    );
    tauri::Builder::default()
        .setup(move |app| {
            if let Some(bootstrap) = app.get_webview_window("bootstrap") {
                bootstrap.destroy()?;
            }
            WebviewWindowBuilder::new(app, "device-console", WebviewUrl::App("index.html".into()))
                .title(format!("{} · Tauri", model::TITLE))
                .inner_size(model::WIDTH as f64, model::HEIGHT as f64)
                .initialization_script(&initialization)
                .center()
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Device console runtime");
}
