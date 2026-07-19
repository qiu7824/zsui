use std::collections::HashMap;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
};

use super::{DesktopRuntimeBackend, DesktopRuntimeRequest};
use crate::{WindowSpec, ZsuiError, ZsuiResult};

#[derive(Default)]
pub(super) struct Backend;

impl DesktopRuntimeBackend for Backend {
    #[cfg(test)]
    fn backend_name(&self) -> &'static str {
        "winit_fallback"
    }

    fn run_event_loop(self, request: DesktopRuntimeRequest) -> ZsuiResult<()> {
        let _backend_owned_state = (
            request.draw_plans,
            request.view_runtimes,
            request.shell_runtimes,
        );
        if request.windows.is_empty() {
            return Ok(());
        }
        if request.windows.iter().any(|window| window.menu.is_some()) {
            return Err(ZsuiError::unsupported(
                "native_menu",
                "the first-pass Winit host does not implement a native application menu",
            ));
        }
        if !request.trays.is_empty() {
            return Err(ZsuiError::unsupported(
                "native_window_status_item",
                "the first-pass Winit host does not implement a native status item",
            ));
        }

        let event_loop = EventLoop::new()
            .map_err(|err| ZsuiError::host("native_window_event_loop", err.to_string()))?;
        let mut app = WinitNativeApp {
            specs: request.windows,
            windows: HashMap::new(),
            startup_error: None,
        };
        event_loop
            .run_app(&mut app)
            .map_err(|err| ZsuiError::host("native_window_event_loop", err.to_string()))?;

        if let Some(err) = app.startup_error {
            Err(ZsuiError::host("create_native_window", err))
        } else {
            Ok(())
        }
    }
}

struct WinitNativeApp {
    specs: Vec<WindowSpec>,
    windows: HashMap<WinitWindowId, WinitWindow>,
    startup_error: Option<String>,
}

impl ApplicationHandler for WinitNativeApp {
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
                    self.windows.insert(window.id(), window);
                }
                Err(err) => {
                    self.startup_error = Some(err.to_string());
                    event_loop.exit();
                    return;
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.windows.remove(&window_id);
            if self.windows.is_empty() {
                event_loop.exit();
            }
        }
    }
}
