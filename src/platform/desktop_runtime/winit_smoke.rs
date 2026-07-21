use super::DesktopSmokeRequest;
use crate::{
    native::{record_draw_plan_smoke, record_native_view_input_smoke, NativeViewInputRuntime},
    NativeDrawPlan, NativeWindowSmokeRunOptions, NativeWindowSmokeRunReport, WindowSpec,
    ZsShellRuntime, ZsuiError, ZsuiResult,
};

pub(super) fn run(request: DesktopSmokeRequest) -> ZsuiResult<NativeWindowSmokeRunReport> {
    run_native_window_smoke_event_loop(
        request.windows,
        request.draw_plans,
        request.view_runtime,
        request.shell_runtime,
        request.options,
    )
}

fn run_native_window_smoke_event_loop(
    windows: Vec<WindowSpec>,
    draw_plans: Vec<Option<NativeDrawPlan>>,
    mut view_runtime: NativeViewInputRuntime,
    _shell_runtime: Option<ZsShellRuntime>,
    options: NativeWindowSmokeRunOptions,
) -> ZsuiResult<NativeWindowSmokeRunReport> {
    use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };
    use winit::{
        application::ApplicationHandler,
        dpi::{LogicalSize, Size},
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
    };

    struct WinitNativeSmokeApp {
        specs: Vec<WindowSpec>,
        windows: HashMap<WinitWindowId, WinitWindow>,
        started_at: Instant,
        auto_close_after: Duration,
        report: NativeWindowSmokeRunReport,
        screenshot_attempted: bool,
        auto_close_reported: bool,
    }

    impl ApplicationHandler for WinitNativeSmokeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if !self.windows.is_empty() {
                return;
            }

            for spec in &self.specs {
                let mut attributes = WindowAttributes::default()
                    .with_title(spec.title.clone())
                    .with_inner_size(Size::Logical(LogicalSize::new(
                        spec.width as f64,
                        spec.height as f64,
                    )))
                    .with_visible(spec.visible)
                    .with_resizable(spec.resizable)
                    .with_decorations(spec.decorations)
                    .with_transparent(spec.transparent);

                if let (Some(width), Some(height)) = (spec.min_width, spec.min_height) {
                    attributes = attributes.with_min_inner_size(Size::Logical(LogicalSize::new(
                        width as f64,
                        height as f64,
                    )));
                }
                if spec.always_on_top {
                    attributes = attributes.with_window_level(WindowLevel::AlwaysOnTop);
                }

                match event_loop.create_window(attributes) {
                    Ok(window) => {
                        self.report.created_window_count += 1;
                        self.report
                            .events
                            .push(format!("window_created:{}", spec.title));
                        self.windows.insert(window.id(), window);
                    }
                    Err(err) => {
                        self.report.startup_error = Some(err.to_string());
                        self.report.events.push("startup_error".to_string());
                        event_loop.exit();
                        return;
                    }
                }
            }

            event_loop.set_control_flow(ControlFlow::WaitUntil(
                self.started_at + self.auto_close_after,
            ));
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WinitWindowId,
            event: WindowEvent,
        ) {
            if matches!(event, WindowEvent::CloseRequested) {
                self.report.close_requested_count += 1;
                self.report.events.push("close_requested".to_string());
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            if self.report.created_window_count == 0 {
                return;
            }

            if !self.screenshot_attempted {
                self.screenshot_attempted = true;
                if let Some(path) = self.report.screenshot_file.clone() {
                    match capture_first_native_window_png(&self.windows, &path) {
                        Ok(()) => {
                            self.report.screenshot_captured = true;
                            self.report
                                .events
                                .push(format!("screenshot_captured:{path}"));
                        }
                        Err(err) => {
                            self.report.screenshot_error = Some(err.clone());
                            self.report.events.push("screenshot_error".to_string());
                        }
                    }
                }
            }

            let target = self.started_at + self.auto_close_after;
            if Instant::now() >= target {
                if !self.auto_close_reported {
                    self.auto_close_reported = true;
                    self.report.exited_by_auto_close = true;
                    self.report.events.push("auto_close_elapsed".to_string());
                }
                event_loop.exit();
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(target));
            }
        }
    }

    if windows.is_empty() {
        return Ok(NativeWindowSmokeRunReport::empty(options));
    }

    let event_loop = EventLoop::new()
        .map_err(|err| ZsuiError::host("native_window_smoke_event_loop", err.to_string()))?;
    let mut app = WinitNativeSmokeApp {
        report: NativeWindowSmokeRunReport {
            requested_window_count: windows.len(),
            auto_close_after_ms: options.auto_close_after_ms,
            ..NativeWindowSmokeRunReport::empty(options.clone())
        },
        specs: windows,
        windows: HashMap::new(),
        started_at: Instant::now(),
        auto_close_after: Duration::from_millis(options.auto_close_after_ms.max(1)),
        screenshot_attempted: false,
        auto_close_reported: false,
    };
    record_draw_plan_smoke(&mut app.report, &draw_plans);
    record_native_view_input_smoke(&mut app.report, &mut view_runtime, &options);
    event_loop
        .run_app(&mut app)
        .map_err(|err| ZsuiError::host("native_window_smoke_event_loop", err.to_string()))?;

    if options.status_item.is_some() {
        app.report.status_item_error = Some(
            "status item smoke is currently implemented only for the direct Windows Win32 host"
                .to_string(),
        );
        app.report
            .events
            .push("status_item_unsupported".to_string());
    }
    if options.native_window_resize.is_some() {
        app.report.native_window_resize_error = Some(
            "native resize proof is not connected to the desktop transport fallback".to_string(),
        );
    }

    if let Some(err) = &app.report.startup_error {
        return Err(ZsuiError::host("create_native_window", err.clone()));
    }
    if options.require_visible_window && !app.report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "native_window_smoke",
            "no visible native window was created",
        ));
    }
    if options.require_screenshot && !app.report.screenshot_captured {
        return Err(ZsuiError::host(
            "native_window_smoke_screenshot",
            app.report
                .screenshot_error
                .clone()
                .unwrap_or_else(|| "window screenshot was not captured".to_string()),
        ));
    }
    if options.require_native_window_resize {
        return Err(ZsuiError::unsupported(
            "native_window_smoke_resize",
            app.report
                .native_window_resize_error
                .clone()
                .unwrap_or_else(|| "native resize proof is unavailable".to_string()),
        ));
    }
    if options.require_status_item && !app.report.status_item_created {
        return Err(ZsuiError::unsupported(
            "native_window_smoke_status_item",
            app.report
                .status_item_error
                .clone()
                .unwrap_or_else(|| "status item was not created".to_string()),
        ));
    }

    Ok(app.report)
}

fn capture_first_native_window_png(
    _windows: &std::collections::HashMap<winit::window::WindowId, winit::window::Window>,
    _path: &str,
) -> Result<(), String> {
    Err("native smoke screenshot capture is currently implemented for Windows only".to_string())
}
