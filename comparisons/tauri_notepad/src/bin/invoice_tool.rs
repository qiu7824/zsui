#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, time::Duration};

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    let auto_close = benchmark_timeout();
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let empty = arguments
        .iter()
        .any(|argument| argument == "--benchmark-empty");
    let repaint = arguments
        .iter()
        .any(|argument| argument == "--benchmark-repaint");
    let route = if cfg!(feature = "perf-common") && (empty || repaint) {
        let mut query = Vec::new();
        if empty {
            query.push("empty=1");
        }
        if repaint {
            query.push("repaint=1");
        }
        format!("invoice.html?{}", query.join("&"))
    } else {
        "invoice.html".to_owned()
    };
    tauri::Builder::default()
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                window.destroy()?;
            }
            WebviewWindowBuilder::new(app, "invoice", WebviewUrl::App(route.into()))
                .title("发票工作台 · Tauri 2")
                .inner_size(1000.0, 700.0)
                .min_inner_size(820.0, 560.0)
                .center()
                .build()?;
            if repaint {
                let repaint_window = app
                    .get_webview_window("invoice")
                    .expect("invoice benchmark window exists");
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(300));
                    let _ = repaint_window.eval(
                        r#"(() => {
                          const marker = document.createElement('i');
                          marker.style.cssText = 'position:fixed;width:1px;height:1px;opacity:.01;pointer-events:none';
                          document.body.appendChild(marker);
                          let frame = 0;
                          const repaint = () => {
                            marker.style.transform = `translateX(${frame++ & 1}px)`;
                            requestAnimationFrame(repaint);
                          };
                          requestAnimationFrame(repaint);
                        })()"#,
                    );
                });
            }
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
