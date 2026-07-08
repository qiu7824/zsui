use crate::{
    app::{app, ZsuiApp, ZsuiAppRuntime},
    capability::HostCapabilities,
    clipboard::ClipboardData,
    core::{
        AppEvent, DialogResponse, FileDialogSpec, HotkeyId, NativeDialogSpec, TrayId, WindowId,
        ZsuiError, ZsuiResult,
    },
    host::{MemoryHost, TrayRecord, WindowRecord, ZsuiHost},
    hotkey::HotkeySpec,
    tray::TraySpec,
    window::{Window, WindowSpec},
};

pub fn native_window(title: impl Into<String>) -> NativeWindowBuilder {
    NativeWindowBuilder::new(title)
}

pub fn run_native_window(title: impl Into<String>) -> ZsuiResult<ZsuiAppRuntime> {
    native_window(title).run()
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWindowBuilder {
    app_name: String,
    window: WindowSpec,
}

impl NativeWindowBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            app_name: title.clone(),
            window: Window::new(title),
        }
    }

    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window = self.window.size(width, height);
        self
    }

    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.window = self.window.min_size(width, height);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.window = self.window.visible(visible);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window = self.window.resizable(resizable);
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.window = self.window.decorations(decorations);
        self
    }

    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.window = self.window.always_on_top(always_on_top);
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.window = self.window.transparent(transparent);
        self
    }

    pub fn window_spec(&self) -> &WindowSpec {
        &self.window
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        app(self.app_name).window(self.window).build()
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        let app = self.build()?;
        let mut host = NativeWindowHost::new();
        app.run_with_host(&mut host)
    }
}

#[derive(Debug, Clone)]
pub struct NativeWindowHost {
    inner: MemoryHost,
    windows: Vec<WindowSpec>,
}

impl NativeWindowHost {
    pub fn new() -> Self {
        Self {
            inner: MemoryHost::with_capabilities(HostCapabilities::current_native_window_host()),
            windows: Vec::new(),
        }
    }

    pub fn recorded_windows(&self) -> &[WindowRecord] {
        self.inner.windows()
    }

    pub fn recorded_trays(&self) -> &[TrayRecord] {
        self.inner.trays()
    }
}

impl Default for NativeWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

impl ZsuiHost for NativeWindowHost {
    fn capabilities(&self) -> HostCapabilities {
        self.inner.capabilities()
    }

    fn create_main_window(&mut self, spec: &WindowSpec) -> ZsuiResult<WindowId> {
        let id = self.inner.create_main_window(spec)?;
        let capabilities = self.capabilities();
        self.windows.push(spec.resolve_for(&capabilities).effective);
        Ok(id)
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()> {
        self.inner.set_window_visible(window, visible)
    }

    fn create_tray(&mut self, spec: &TraySpec) -> ZsuiResult<TrayId> {
        self.inner.create_tray(spec)
    }

    fn register_global_hotkey(&mut self, spec: &HotkeySpec) -> ZsuiResult<HotkeyId> {
        self.inner.register_global_hotkey(spec)
    }

    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        self.inner.read_clipboard()
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        self.inner.write_clipboard(data)
    }

    fn open_file_picker(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<String>>> {
        self.inner.open_file_picker(spec)
    }

    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        self.inner.show_native_dialog(spec)
    }

    fn poll_event(&mut self) -> ZsuiResult<Option<AppEvent>> {
        self.inner.poll_event()
    }

    fn run_event_loop(&mut self) -> ZsuiResult<()> {
        run_native_window_event_loop(self.windows.clone())
    }
}

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
fn run_native_window_event_loop(windows: Vec<WindowSpec>) -> ZsuiResult<()> {
    use std::collections::HashMap;
    use winit::{
        application::ApplicationHandler,
        dpi::{LogicalSize, Size},
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
    };

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

    if windows.is_empty() {
        return Ok(());
    }

    let event_loop = EventLoop::new()
        .map_err(|err| ZsuiError::host("native_window_event_loop", err.to_string()))?;
    let mut app = WinitNativeApp {
        specs: windows,
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

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
fn run_native_window_event_loop(_windows: Vec<WindowSpec>) -> ZsuiResult<()> {
    Err(ZsuiError::unsupported(
        "native_window",
        "desktop native windows are implemented for Windows, macOS and Linux; Android and Harmony need mobile runtime hosts",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_window_builder_declares_single_window_app() {
        let app = native_window("Example")
            .size(800, 520)
            .min_size(480, 320)
            .always_on_top(true)
            .build()
            .expect("native builder should create an app declaration");

        assert_eq!(app.name, "Example");
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.windows[0].title, "Example");
        assert_eq!(app.windows[0].width, 800);
        assert_eq!(app.windows[0].min_width, Some(480));
        assert!(app.windows[0].always_on_top);
    }
}
