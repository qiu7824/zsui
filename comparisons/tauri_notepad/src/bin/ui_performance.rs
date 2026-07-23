#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf, thread, time::Duration};

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Profile {
    Minimal,
    Full,
    Viewer,
}

#[cfg(feature = "perf-viewer")]
const PROFILE: Profile = Profile::Viewer;
#[cfg(all(not(feature = "perf-viewer"), feature = "perf-full"))]
const PROFILE: Profile = Profile::Full;
#[cfg(all(not(feature = "perf-viewer"), not(feature = "perf-full")))]
const PROFILE: Profile = Profile::Minimal;

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let empty = arguments
        .iter()
        .any(|argument| argument == "--benchmark-empty");
    let repaint = arguments
        .iter()
        .any(|argument| argument == "--benchmark-repaint");
    let auto_close = arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(Duration::from_secs);
    let document = arguments
        .windows(2)
        .find(|pair| pair[0] == "--document")
        .map(|pair| PathBuf::from(&pair[1]));
    let title = match PROFILE {
        Profile::Minimal => "UI 性能矩阵 · Minimal · Tauri 2",
        Profile::Full => "UI 性能矩阵 · Full Native App · Tauri 2",
        Profile::Viewer => "UI 性能矩阵 · Viewer · Tauri 2",
    };
    let mut query = Vec::new();
    if empty {
        query.push("empty=1");
    }
    if repaint {
        query.push("repaint=1");
    }
    let route = if query.is_empty() {
        "index.html".to_owned()
    } else {
        format!("index.html?{}", query.join("&"))
    };

    tauri::Builder::default()
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                window.destroy()?;
            }
            let window =
                WebviewWindowBuilder::new(app, "performance", WebviewUrl::App(route.into()))
                    .title(title)
                    .inner_size(1000.0, 700.0)
                    .min_inner_size(820.0, 560.0)
                    .center()
                    .build()?;

            if repaint {
                let repaint_window = window.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(300));
                    let _ = repaint_window.eval(
                        r#"(() => {
                          const marker = document.createElement('i');
                          marker.setAttribute('aria-hidden', 'true');
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

            if matches!(PROFILE, Profile::Viewer) {
                if let Some(document) = document {
                    let viewer = window.clone();
                    thread::spawn(move || {
                        let mut last_modified = None;
                        let mut revision = 1_u64;
                        loop {
                            thread::sleep(Duration::from_millis(250));
                            let modified = fs::metadata(&document)
                                .and_then(|metadata| metadata.modified())
                                .ok();
                            if modified.is_some() && modified != last_modified {
                                last_modified = modified;
                                revision = revision.saturating_add(1);
                                if viewer
                                    .eval(format!("window.setViewerRevision?.({revision})"))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    });
                }
            }
            if let Some(duration) = auto_close {
                let handle = app.handle().clone();
                thread::spawn(move || {
                    thread::sleep(duration);
                    handle.exit(0);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri UI performance runtime failed");
}
