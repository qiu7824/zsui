use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[cfg(feature = "linux-direct")]
use cairo::{Context as CairoContext, Format, ImageSurface};
#[cfg(feature = "linux-system-icons")]
use gdk_pixbuf::prelude::*;
#[cfg(feature = "linux-direct")]
use pango::prelude::*;
use softbuffer::{Context as SoftBufferContext, Surface as SoftBufferSurface};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use winit::window::{
    Theme as WinitTheme, Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId,
    WindowLevel,
};

#[cfg(feature = "linux-direct")]
use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
#[cfg(feature = "linux-system-icons")]
use crate::NativeIconColorMode;
#[cfg(feature = "linux-direct")]
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawImageCommand, NativeDrawTextCommand, NativeImageInterpolation, NativeStyleResolver,
    Size as ZsSize, TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};
use crate::{
    MenuItemSpec, MenuSpec, NativeDrawPlan, Point, Rect, WindowSpec, ZsAccelerator,
    ZsAcceleratorKey, ZsIcon, ZsuiError, ZsuiResult,
};

type LinuxDisplayHandle = OwnedDisplayHandle;
type LinuxSoftBufferContext = SoftBufferContext<LinuxDisplayHandle>;
type LinuxSoftBufferSurface = SoftBufferSurface<LinuxDisplayHandle, Rc<WinitWindow>>;

#[cfg(feature = "linux-direct")]
type LinuxDirectTextContext = pango::Context;
#[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
type LinuxDirectTextContext = crate::linux_direct_lite::LinuxLiteTextSystem;

#[derive(Debug)]
pub(crate) struct LinuxDirectNativeWindowRunReport {
    pub created_window_count: usize,
    pub native_view_capture: Option<Result<crate::NativeViewCaptureEvidence, String>>,
    pub proof_input_reports: Vec<crate::native::NativeViewInputDispatchReport>,
    pub native_window_resize: Option<crate::NativeWindowResizeEvidence>,
    pub native_window_resize_error: Option<String>,
    pub menu_command_routed: bool,
    pub menu_surface_created: bool,
    pub menu_surface_height: u32,
    pub menu_surface_open_at_capture: bool,
    pub accessibility_bridge_created: bool,
    pub accessibility_node_count: usize,
    pub accessibility_action_count: usize,
    pub process_memory: Option<crate::NativeProofProcessMemoryEvidence>,
}

pub(crate) fn run_linux_direct_native_window_event_loop(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    view_runtimes: &[crate::native::NativeViewInputRuntime],
    auto_close_after_ms: Option<u64>,
    capture_path: Option<&Path>,
    proof_inputs: &[crate::NativeViewSmokeInput],
    proof_resize: Option<crate::Size>,
) -> ZsuiResult<LinuxDirectNativeWindowRunReport> {
    if specs.is_empty() {
        return Ok(LinuxDirectNativeWindowRunReport {
            created_window_count: 0,
            native_view_capture: None,
            proof_input_reports: Vec::new(),
            native_window_resize: None,
            native_window_resize_error: None,
            menu_command_routed: false,
            menu_surface_created: false,
            menu_surface_height: 0,
            menu_surface_open_at_capture: false,
            accessibility_bridge_created: false,
            accessibility_node_count: 0,
            accessibility_action_count: 0,
            process_memory: None,
        });
    }

    let event_loop = EventLoop::new()
        .map_err(|error| ZsuiError::host("linux_direct_event_loop", error.to_string()))?;
    let mut app = LinuxDirectApp::new(
        specs,
        draw_plans,
        view_runtimes,
        auto_close_after_ms,
        capture_path,
        proof_inputs,
        proof_resize,
    );
    event_loop
        .run_app(&mut app)
        .map_err(|error| ZsuiError::host("linux_direct_event_loop", error.to_string()))?;

    if let Some(error) = app.startup_error.take() {
        return Err(ZsuiError::host("linux_direct_window", error));
    }
    Ok(LinuxDirectNativeWindowRunReport {
        created_window_count: app.created_window_count,
        native_view_capture: app.capture_result.take(),
        proof_input_reports: std::mem::take(&mut app.proof_input_reports),
        native_window_resize: app.native_window_resize.take(),
        native_window_resize_error: app.native_window_resize_error.take(),
        menu_command_routed: app.menu_command_routed,
        menu_surface_created: app.menu_surface_created,
        menu_surface_height: app.menu_surface_height,
        menu_surface_open_at_capture: app.menu_surface_open_at_capture,
        accessibility_bridge_created: app.accessibility_bridge_created,
        accessibility_node_count: app.accessibility_node_count,
        accessibility_action_count: app.accessibility_action_count,
        process_memory: app.process_memory.take(),
    })
}

struct LinuxDirectApp {
    specs: Vec<WindowSpec>,
    initial_plans: Vec<Option<NativeDrawPlan>>,
    initial_runtimes: Vec<crate::native::NativeViewInputRuntime>,
    windows: HashMap<WinitWindowId, LinuxDirectWindow>,
    context: Option<LinuxSoftBufferContext>,
    created_window_count: usize,
    startup_error: Option<String>,
    proof_at: Option<Instant>,
    close_at: Option<Instant>,
    proof_dispatched: bool,
    proof_inputs: Vec<crate::NativeViewSmokeInput>,
    proof_input_reports: Vec<crate::native::NativeViewInputDispatchReport>,
    proof_resize: Option<crate::Size>,
    proof_resize_requested: bool,
    proof_resize_window: Option<WinitWindowId>,
    proof_resize_initial_size: Option<crate::Size>,
    proof_resize_event_count: usize,
    native_window_resize: Option<crate::NativeWindowResizeEvidence>,
    native_window_resize_error: Option<String>,
    capture_path: Option<PathBuf>,
    capture_result: Option<Result<crate::NativeViewCaptureEvidence, String>>,
    menu_command_routed: bool,
    menu_surface_created: bool,
    menu_surface_height: u32,
    menu_surface_open_at_capture: bool,
    accessibility_bridge_created: bool,
    accessibility_node_count: usize,
    accessibility_action_count: usize,
    process_memory: Option<crate::NativeProofProcessMemoryEvidence>,
}

impl LinuxDirectApp {
    fn new(
        specs: &[WindowSpec],
        draw_plans: &[Option<NativeDrawPlan>],
        view_runtimes: &[crate::native::NativeViewInputRuntime],
        auto_close_after_ms: Option<u64>,
        capture_path: Option<&Path>,
        proof_inputs: &[crate::NativeViewSmokeInput],
        proof_resize: Option<crate::Size>,
    ) -> Self {
        let started_at = Instant::now();
        let close_after = auto_close_after_ms.map(|delay| Duration::from_millis(delay.max(1)));
        Self {
            specs: specs.to_vec(),
            initial_plans: draw_plans.to_vec(),
            initial_runtimes: view_runtimes.to_vec(),
            windows: HashMap::new(),
            context: None,
            created_window_count: 0,
            startup_error: None,
            proof_at: (!proof_inputs.is_empty() || proof_resize.is_some())
                .then(|| close_after.map(|duration| started_at + duration / 2))
                .flatten(),
            close_at: close_after.map(|duration| started_at + duration),
            proof_dispatched: false,
            proof_inputs: proof_inputs.to_vec(),
            proof_input_reports: Vec::new(),
            proof_resize,
            proof_resize_requested: false,
            proof_resize_window: None,
            proof_resize_initial_size: None,
            proof_resize_event_count: 0,
            native_window_resize: None,
            native_window_resize_error: None,
            capture_path: capture_path.map(PathBuf::from),
            capture_result: None,
            menu_command_routed: false,
            menu_surface_created: false,
            menu_surface_height: 0,
            menu_surface_open_at_capture: false,
            accessibility_bridge_created: false,
            accessibility_node_count: 0,
            accessibility_action_count: 0,
            process_memory: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, operation: &str, error: impl ToString) {
        self.startup_error = Some(format!("{operation}: {}", error.to_string()));
        event_loop.exit();
    }

    fn create_windows(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let context = SoftBufferContext::new(event_loop.owned_display_handle())
            .map_err(|error| format!("could not initialize software presentation: {error}"))?;
        self.context = Some(context);

        let specs = std::mem::take(&mut self.specs);
        let mut initial_plans = std::mem::take(&mut self.initial_plans);
        let mut initial_runtimes = std::mem::take(&mut self.initial_runtimes);
        for (index, spec) in specs.into_iter().enumerate() {
            let mut attributes = WindowAttributes::default()
                .with_title(spec.title.clone())
                .with_inner_size(Size::Logical(LogicalSize::new(
                    spec.width as f64,
                    spec.height as f64,
                )))
                .with_visible(false)
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

            let window = Rc::new(
                event_loop
                    .create_window(attributes)
                    .map_err(|error| format!("could not create `{}`: {error}", spec.title))?,
            );
            let surface = SoftBufferSurface::new(
                self.context
                    .as_ref()
                    .ok_or_else(|| "software presentation context is missing".to_string())?,
                Rc::clone(&window),
            )
            .map_err(|error| format!("could not create `{}` surface: {error}", spec.title))?;
            let title = spec.title;
            let menu = spec.menu;
            let visible = spec.visible;
            let initial_width = spec.width;
            let initial_height = spec.height;
            let mut runtime = initial_runtimes
                .get_mut(index)
                .map(|runtime| std::mem::take(runtime))
                .unwrap_or_default();
            let text_context = linux_direct_text_context();
            #[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
            runtime
                .set_text_shaping_backend(linux_direct_text_shaping_backend(text_context.clone()));
            #[cfg(all(
                feature = "text-input-core",
                feature = "linux-direct-lite",
                not(feature = "linux-direct")
            ))]
            runtime.set_text_shaping_backend(linux_direct_lite_text_shaping_backend(
                text_context.clone(),
            ));
            runtime.defer_app_command_execution();
            let mut direct = LinuxDirectWindow::new(
                event_loop,
                title,
                menu,
                initial_width,
                initial_height,
                Rc::clone(&window),
                surface,
                initial_plans
                    .get_mut(index)
                    .and_then(Option::take)
                    .unwrap_or_default(),
                runtime,
                text_context,
                self.capture_path.is_some(),
            );
            direct.synchronize_initial_surface(window.inner_size());
            direct.sync_text_input();
            window.set_visible(visible);
            window.request_redraw();
            self.windows.insert(window.id(), direct);
            self.created_window_count += 1;
        }
        Ok(())
    }

    fn dispatch_proof_inputs(&mut self, event_loop: &ActiveEventLoop) {
        if self.proof_dispatched {
            return;
        }
        self.proof_dispatched = true;
        let Some(window_id) = self.windows.keys().next().copied() else {
            return;
        };
        let Some(window) = self.windows.get_mut(&window_id) else {
            return;
        };
        let proof_menu_command = (!self.proof_inputs.is_empty())
            .then(|| {
                window
                    .menu_surface
                    .as_mut()
                    .and_then(|menu| menu.proof_command(&window.draw_text_context))
            })
            .flatten();
        if let Some(command) = proof_menu_command {
            let report = window.runtime.dispatch_app_command(command);
            let _ = window.apply_report(report, event_loop);
            window.menu_surface_command_count = window.menu_surface_command_count.saturating_add(1);
            self.menu_command_routed = true;
        }
        // Match the Win32, AppKit and GTK proof order: verify the menu route
        // before the scripted interaction. Some applications intentionally
        // leave their final state in the last input (for example a vetoed
        // native close request that opens an unsaved-changes dialog).
        for input in &self.proof_inputs {
            let reports = window.dispatch_proof_input(input, event_loop);
            self.proof_input_reports.extend(reports);
        }
        let open_menu_for_capture = std::env::var("ZSUI_NATIVE_PROOF_OPEN_MENU")
            .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes"));
        if open_menu_for_capture {
            if let Some(menu) = window.menu_surface.as_mut() {
                menu.open_for_capture(&window.draw_text_context);
            }
            #[cfg(feature = "accessibility")]
            window.sync_accessibility();
        }
        window.window.request_redraw();
    }

    fn request_proof_resize(&mut self) {
        if self.proof_resize_requested {
            return;
        }
        let Some(requested) = self.proof_resize else {
            return;
        };
        let Some(window_id) = self.windows.keys().next().copied() else {
            self.native_window_resize_error =
                Some("the Linux direct proof has no native window to resize".to_string());
            self.proof_resize_requested = true;
            return;
        };
        let Some(window) = self.windows.get_mut(&window_id) else {
            return;
        };
        self.proof_resize_initial_size = Some(window.logical_window_size());
        self.proof_resize_window = Some(window_id);
        self.proof_resize_requested = true;
        if let Some(physical_size) = window.window.request_inner_size(LogicalSize::new(
            f64::from(requested.width.max(1)),
            f64::from(requested.height.max(1)),
        )) {
            window.resize(physical_size);
            window.window.request_redraw();
            self.proof_resize_event_count = self.proof_resize_event_count.saturating_add(1);
        }
    }

    fn capture_and_exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(requested_size) = self.proof_resize {
            let final_size = self
                .proof_resize_window
                .and_then(|window_id| self.windows.get(&window_id))
                .map(LinuxDirectWindow::logical_window_size);
            let applied = final_size == Some(requested_size) && self.proof_resize_event_count > 0;
            self.native_window_resize = Some(crate::NativeWindowResizeEvidence {
                backend: "winit_request_inner_size_window_event",
                requested_size,
                initial_size: self.proof_resize_initial_size,
                final_size,
                native_event_count: self.proof_resize_event_count,
                applied,
            });
            if !applied && self.native_window_resize_error.is_none() {
                self.native_window_resize_error = Some(match final_size {
                    Some(final_size) => format!(
                        "Linux native resize requested {}x{} but finished at {}x{} after {} Resized events",
                        requested_size.width,
                        requested_size.height,
                        final_size.width,
                        final_size.height,
                        self.proof_resize_event_count
                    ),
                    None => "the Linux direct proof window closed before resize evidence was captured"
                        .to_string(),
                });
            }
        }
        self.process_memory =
            crate::NativeProofProcessMemoryEvidence::capture_at("native_window_before_teardown");
        #[cfg(feature = "accessibility")]
        {
            self.accessibility_bridge_created = !self.windows.is_empty();
            self.accessibility_node_count = self
                .windows
                .values()
                .map(LinuxDirectWindow::accessibility_node_count)
                .sum();
            self.accessibility_action_count = self
                .windows
                .values()
                .map(LinuxDirectWindow::accessibility_action_count)
                .sum();
        }
        self.menu_surface_created = self
            .windows
            .values()
            .any(LinuxDirectWindow::has_menu_surface);
        self.menu_surface_height = self
            .windows
            .values()
            .map(LinuxDirectWindow::menu_content_offset_y)
            .max()
            .unwrap_or(0) as u32;
        self.menu_surface_open_at_capture = self
            .windows
            .values()
            .any(LinuxDirectWindow::menu_surface_is_open);
        self.menu_command_routed |= self
            .windows
            .values()
            .any(|window| window.menu_surface_command_count > 0);
        if self.capture_result.is_none() {
            if let Some(path) = self.capture_path.as_deref() {
                self.capture_result = Some(
                    self.windows
                        .values()
                        .next()
                        .ok_or_else(|| "the Linux direct host has no window to capture".to_string())
                        .and_then(|window| window.capture_png(path)),
                );
            }
        }
        self.windows.clear();
        event_loop.exit();
    }

    fn schedule_control_flow(&self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let mut next = self.close_at;
        if !self.proof_dispatched {
            next = next.min_some(self.proof_at);
        }
        for window in self.windows.values() {
            if let Some(candidate) = window.next_tick {
                next = next.min_some(Some(candidate));
            }
        }
        #[cfg(feature = "accessibility")]
        if !self.windows.is_empty() {
            next = next.min_some(Some(now + Duration::from_millis(50)));
        }
        match next {
            Some(deadline) if deadline <= now => event_loop.set_control_flow(ControlFlow::Poll),
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }
}

trait OptionInstantExt {
    fn min_some(self, other: Option<Instant>) -> Option<Instant>;
}

impl OptionInstantExt for Option<Instant> {
    fn min_some(self, other: Option<Instant>) -> Option<Instant> {
        match (self, other) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(value), None) | (None, Some(value)) => Some(value),
            (None, None) => None,
        }
    }
}

impl ApplicationHandler for LinuxDirectApp {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() || self.startup_error.is_some() {
            return;
        }
        if let Err(error) = self.create_windows(event_loop) {
            self.fail(event_loop, "create_windows", error);
            return;
        }
        self.schedule_control_flow(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        let mut remove_window = false;
        let Some(window) = self.windows.get_mut(&window_id) else {
            return;
        };
        #[cfg(feature = "accessibility")]
        window.process_accessibility_event(&event);
        match event {
            WindowEvent::CloseRequested => {
                let report = window.runtime.dispatch_window_close_requested();
                let allow = !report.handled || report.quit_requested;
                window.apply_report(report, event_loop);
                remove_window = allow;
            }
            WindowEvent::Resized(size) => {
                window.resize(size);
                window.window.request_redraw();
                if self.proof_resize_requested && self.proof_resize_window == Some(window_id) {
                    self.proof_resize_event_count = self.proof_resize_event_count.saturating_add(1);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                window.scale_factor = scale_factor.max(0.1);
                window.resize(window.window.inner_size());
                window.window.request_redraw();
            }
            WindowEvent::ThemeChanged(theme) => {
                window.theme = theme;
                window.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = window.redraw() {
                    self.fail(event_loop, "redraw", error);
                    return;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = window.logical_point(position);
                window.cursor = point;
                let mut menu_changed = false;
                let mut menu_captures = false;
                if let Some(menu) = window.menu_surface.as_mut() {
                    menu_changed = menu.pointer_move(point, &window.draw_text_context);
                    menu_captures = menu.captures_pointer(point);
                }
                if menu_changed {
                    window.window.request_redraw();
                    #[cfg(feature = "accessibility")]
                    window.sync_accessibility();
                }
                if !menu_captures {
                    let report = window.runtime.dispatch_pointer_move_with_modifiers(
                        window.content_point(point),
                        linux_pointer_modifiers(window.modifiers),
                    );
                    window.apply_report(report, event_loop);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                let report = window.runtime.dispatch_pointer_leave();
                window.apply_report(report, event_loop);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let cursor = window.cursor;
                let pointer_button = linux_pointer_button(button);
                let primary = pointer_button == crate::ZsPointerButton::Primary;
                if state == ElementState::Pressed {
                    let menu_consumed = primary
                        && window
                            .menu_surface
                            .as_mut()
                            .is_some_and(|menu| menu.pointer_down(cursor));
                    if menu_consumed {
                        window.window.request_redraw();
                    } else {
                        let report = window.runtime.dispatch_pointer_down_with_button(
                            window.content_point(cursor),
                            pointer_button,
                            linux_pointer_modifiers(window.modifiers),
                        );
                        window.apply_report(report, event_loop);
                    }
                } else {
                    let menu_result = if primary {
                        window
                            .menu_surface
                            .as_mut()
                            .map(|menu| menu.pointer_up(cursor, &window.draw_text_context))
                            .unwrap_or(crate::linux_direct_menu::LinuxMenuInputResult::Ignored)
                    } else {
                        crate::linux_direct_menu::LinuxMenuInputResult::Ignored
                    };
                    match menu_result {
                        crate::linux_direct_menu::LinuxMenuInputResult::Ignored => {
                            let report = window.runtime.dispatch_pointer_up_with_button(
                                window.content_point(cursor),
                                pointer_button,
                                linux_pointer_modifiers(window.modifiers),
                            );
                            window.apply_report(report, event_loop);
                        }
                        crate::linux_direct_menu::LinuxMenuInputResult::Redraw => {
                            window.window.request_redraw();
                            #[cfg(feature = "accessibility")]
                            window.sync_accessibility();
                        }
                        crate::linux_direct_menu::LinuxMenuInputResult::Command(command) => {
                            window.menu_surface_command_count =
                                window.menu_surface_command_count.saturating_add(1);
                            let report = window.runtime.dispatch_app_command(command);
                            window.apply_report(report, event_loop);
                            window.window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let logical_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 48.0,
                    MouseScrollDelta::PixelDelta(position) => {
                        (position.y / window.scale_factor) as f32
                    }
                };
                if logical_delta.abs() > f32::EPSILON {
                    let menu_captures = window
                        .menu_surface
                        .as_ref()
                        .is_some_and(|menu| menu.captures_pointer(window.cursor));
                    if !menu_captures {
                        let report = window.runtime.dispatch_pointer_scroll(
                            window.content_point(window.cursor),
                            crate::Dp::new(logical_delta),
                        );
                        window.apply_report(report, event_loop);
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                window.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let menu_result = window
                    .menu_surface
                    .as_mut()
                    .map(|menu| menu.key(&event.logical_key, &window.draw_text_context))
                    .unwrap_or(crate::linux_direct_menu::LinuxMenuInputResult::Ignored);
                match menu_result {
                    crate::linux_direct_menu::LinuxMenuInputResult::Command(command) => {
                        window.menu_surface_command_count =
                            window.menu_surface_command_count.saturating_add(1);
                        let report = window.runtime.dispatch_app_command(command);
                        window.apply_report(report, event_loop);
                        window.window.request_redraw();
                    }
                    crate::linux_direct_menu::LinuxMenuInputResult::Redraw => {
                        window.window.request_redraw();
                        #[cfg(feature = "accessibility")]
                        window.sync_accessibility();
                    }
                    crate::linux_direct_menu::LinuxMenuInputResult::Ignored => {
                        if let Some(command) = window.menu_surface.as_ref().and_then(|surface| {
                            menu_command_for_key(
                                surface.spec(),
                                &event.logical_key,
                                window.modifiers,
                            )
                        }) {
                            let report = window.runtime.dispatch_app_command(command);
                            window.apply_report(report, event_loop);
                        } else {
                            let report = window
                                .dispatch_key_event(&event.logical_key, event.text.as_deref());
                            window.apply_report(report, event_loop);
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                let report = match ime {
                    Ime::Preedit(text, selection) => {
                        let selection = selection.map(|(start, end)| {
                            (
                                byte_to_char_index(&text, start),
                                byte_to_char_index(&text, end),
                            )
                        });
                        window.runtime.dispatch_ime_preedit(&text, selection)
                    }
                    Ime::Commit(text) => window.runtime.dispatch_ime_commit(&text),
                    Ime::Disabled => window.runtime.cancel_ime_preedit(),
                    Ime::Enabled => crate::native::NativeViewInputDispatchReport::default(),
                };
                window.apply_report(report, event_loop);
            }
            WindowEvent::Focused(focused) => {
                if focus_transition_lost(&mut window.window_focused, focused) {
                    let report = window.runtime.blur_focus();
                    window.apply_report(report, event_loop);
                }
            }
            WindowEvent::Occluded(true) => {
                if window.runtime.suspend_view_when_hidden() {
                    window.plan = NativeDrawPlan::default();
                    window.window.request_redraw();
                }
            }
            WindowEvent::Occluded(false) => {
                if let Some(plan) = window.runtime.resume_view_when_visible() {
                    window.plan = plan;
                    window.window.request_redraw();
                }
            }
            _ => {}
        }
        if remove_window {
            self.windows.remove(&window_id);
            if self.windows.is_empty() {
                event_loop.exit();
            }
        }
        self.schedule_control_flow(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self.proof_at.is_some_and(|deadline| now >= deadline) {
            if self.proof_resize.is_some() && !self.proof_resize_requested {
                self.request_proof_resize();
            } else if self.proof_resize.is_none() || self.proof_resize_event_count > 0 {
                self.dispatch_proof_inputs(event_loop);
            }
        }
        for window in self.windows.values_mut() {
            #[cfg(feature = "accessibility")]
            window.dispatch_accessibility_actions(event_loop);
            if window.next_tick.is_some_and(|deadline| now >= deadline) {
                let report = window.runtime.refresh_transient_view();
                window.apply_report(report, event_loop);
            }
        }
        if self.close_at.is_some_and(|deadline| now >= deadline) {
            self.capture_and_exit(event_loop);
            return;
        }
        self.schedule_control_flow(event_loop);
    }
}

struct LinuxDirectWindow {
    #[allow(dead_code)]
    title: String,
    window: Rc<WinitWindow>,
    surface: LinuxSoftBufferSurface,
    plan: NativeDrawPlan,
    runtime: crate::native::NativeViewInputRuntime,
    text_context: LinuxDirectTextContext,
    draw_text_context: LinuxDirectTextContext,
    display_server: &'static str,
    scale_factor: f64,
    physical_size: PhysicalSize<u32>,
    cursor: Point,
    modifiers: ModifiersState,
    window_focused: bool,
    presented_once: bool,
    theme: WinitTheme,
    retain_frame: bool,
    last_frame: Vec<u32>,
    icon_cache: HashMap<(ZsIcon, u32, bool), LinuxIconRaster>,
    next_tick: Option<Instant>,
    menu_surface: Option<crate::linux_direct_menu::LinuxDirectMenuSurface>,
    menu_surface_command_count: usize,
    #[cfg(feature = "accessibility")]
    accessibility: crate::linux_direct_accessibility::LinuxDirectAccessibility,
}

impl LinuxDirectWindow {
    fn logical_window_size(&self) -> crate::Size {
        let logical = self.physical_size.to_logical::<f64>(self.scale_factor);
        crate::Size {
            width: logical.width.round().clamp(1.0, f64::from(i32::MAX)) as i32,
            height: logical.height.round().clamp(1.0, f64::from(i32::MAX)) as i32,
        }
    }

    fn new(
        event_loop: &ActiveEventLoop,
        title: String,
        menu: Option<MenuSpec>,
        initial_width: u32,
        initial_height: u32,
        window: Rc<WinitWindow>,
        surface: LinuxSoftBufferSurface,
        plan: NativeDrawPlan,
        runtime: crate::native::NativeViewInputRuntime,
        text_context: LinuxDirectTextContext,
        retain_frame: bool,
    ) -> Self {
        #[cfg(not(feature = "accessibility"))]
        let _ = (event_loop, initial_height);
        let scale_factor = window.scale_factor().max(0.1);
        let theme = window.theme().unwrap_or(WinitTheme::Light);
        let display_server = linux_direct_display_server(&window);
        let mut menu_surface = menu.map(crate::linux_direct_menu::LinuxDirectMenuSurface::new);
        if let Some(menu) = menu_surface.as_mut() {
            menu.layout(initial_width.max(1) as i32, &text_context);
        }
        #[cfg(feature = "accessibility")]
        let accessibility = crate::linux_direct_accessibility::LinuxDirectAccessibility::new(
            event_loop,
            &window,
            &title,
            Rect {
                x: 0,
                y: 0,
                width: initial_width.max(1) as i32,
                height: initial_height.max(1) as i32,
            },
            scale_factor,
            menu_surface
                .as_ref()
                .map_or(0, |menu| menu.content_offset_y()),
            menu_surface
                .as_ref()
                .map(|menu| menu.accessibility_snapshot()),
            &plan,
            runtime.current_interaction_plan(),
            runtime.focused_widget(),
        );
        let draw_text_context = text_context.clone();
        Self {
            title,
            window,
            surface,
            plan,
            runtime,
            text_context,
            draw_text_context,
            display_server,
            scale_factor,
            physical_size: PhysicalSize::new(1, 1),
            cursor: Point { x: 0, y: 0 },
            modifiers: ModifiersState::default(),
            window_focused: false,
            presented_once: false,
            theme,
            retain_frame,
            last_frame: Vec::new(),
            icon_cache: HashMap::new(),
            next_tick: None,
            menu_surface,
            menu_surface_command_count: 0,
            #[cfg(feature = "accessibility")]
            accessibility,
        }
    }

    fn synchronize_initial_surface(&mut self, physical_size: PhysicalSize<u32>) {
        self.update_surface(physical_size, true);
    }

    fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.update_surface(physical_size, !self.presented_once);
    }

    fn update_surface(&mut self, physical_size: PhysicalSize<u32>, initial_attachment: bool) {
        self.physical_size =
            PhysicalSize::new(physical_size.width.max(1), physical_size.height.max(1));
        let logical = self.physical_size.to_logical::<f64>(self.scale_factor);
        if let Some(menu) = self.menu_surface.as_mut() {
            menu.layout(
                logical.width.round().clamp(1.0, f64::from(i32::MAX)) as i32,
                &self.draw_text_context,
            );
        }
        let content_height = logical.height.round().clamp(1.0, f64::from(i32::MAX)) as i32
            - self.menu_content_offset_y();
        let surface = Rect {
            x: 0,
            y: 0,
            width: logical.width.round().clamp(1.0, f64::from(i32::MAX)) as i32,
            height: content_height.max(1),
        };
        let report = if initial_attachment {
            self.runtime
                .synchronize_surface(surface, crate::Dpi::standard())
        } else {
            self.runtime.set_surface(surface, crate::Dpi::standard())
        };
        if let Some(plan) = report.redraw_plan {
            self.plan = plan;
        }
        self.sync_text_input();
        #[cfg(feature = "accessibility")]
        self.sync_accessibility();
        self.schedule_tick();
    }

    fn logical_point(&self, position: PhysicalPosition<f64>) -> Point {
        let logical = position.to_logical::<f64>(self.scale_factor);
        Point {
            x: logical
                .x
                .round()
                .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
            y: logical
                .y
                .round()
                .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
        }
    }

    fn content_point(&self, point: Point) -> Point {
        Point {
            x: point.x,
            y: point.y.saturating_sub(self.menu_content_offset_y()),
        }
    }

    fn menu_content_offset_y(&self) -> i32 {
        self.menu_surface
            .as_ref()
            .map(|menu| menu.content_offset_y())
            .unwrap_or(0)
    }

    fn has_menu_surface(&self) -> bool {
        self.menu_surface.is_some()
    }

    fn menu_surface_is_open(&self) -> bool {
        self.menu_surface
            .as_ref()
            .is_some_and(|menu| menu.is_open())
    }

    fn dispatch_key_event(
        &mut self,
        logical_key: &Key,
        event_text: Option<&str>,
    ) -> crate::native::NativeViewInputDispatchReport {
        let shift = self.modifiers.shift_key();
        let control = self.modifiers.control_key();
        let command_modifier = control || self.modifiers.super_key() || self.modifiers.alt_key();
        let named = match logical_key {
            Key::Named(NamedKey::Tab) => Some(crate::NativeViewKey::Tab),
            Key::Named(NamedKey::Enter) => Some(crate::NativeViewKey::Enter),
            Key::Named(NamedKey::Space) => Some(crate::NativeViewKey::Space),
            Key::Named(NamedKey::ArrowUp) => Some(crate::NativeViewKey::Up),
            Key::Named(NamedKey::ArrowDown) => Some(crate::NativeViewKey::Down),
            Key::Named(NamedKey::ArrowLeft) => Some(crate::NativeViewKey::Left),
            Key::Named(NamedKey::ArrowRight) => Some(crate::NativeViewKey::Right),
            Key::Named(NamedKey::Home) => Some(crate::NativeViewKey::Home),
            Key::Named(NamedKey::End) => Some(crate::NativeViewKey::End),
            Key::Named(NamedKey::PageUp) => Some(crate::NativeViewKey::PageUp),
            Key::Named(NamedKey::PageDown) => Some(crate::NativeViewKey::PageDown),
            Key::Named(NamedKey::Escape) => Some(crate::NativeViewKey::Escape),
            _ => None,
        };
        if let Some(key) = named {
            let report = self
                .runtime
                .dispatch_key_with_modifiers(key, shift, control);
            if report.handled {
                return report;
            }
            return match logical_key {
                Key::Named(NamedKey::Enter) => self.runtime.dispatch_text_input("\r"),
                Key::Named(NamedKey::Space) if !command_modifier => {
                    self.runtime.dispatch_text_input(" ")
                }
                _ => report,
            };
        }
        match logical_key {
            Key::Named(NamedKey::Backspace) => self.runtime.dispatch_text_input("\u{8}"),
            Key::Named(NamedKey::Delete) => self.runtime.dispatch_text_input("\u{7f}"),
            _ if !command_modifier => event_text
                .filter(|text| !text.is_empty() && !text.chars().all(char::is_control))
                .map(|text| self.runtime.dispatch_text_input(text))
                .unwrap_or_default(),
            _ => crate::native::NativeViewInputDispatchReport::default(),
        }
    }

    fn dispatch_proof_input(
        &mut self,
        input: &crate::NativeViewSmokeInput,
        event_loop: &ActiveEventLoop,
    ) -> Vec<crate::native::NativeViewInputDispatchReport> {
        let mut reports = Vec::new();
        match input {
            crate::NativeViewSmokeInput::Move(point) => {
                self.cursor = *point;
                let report = self.runtime.dispatch_pointer_move(*point);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::Click(point) => {
                self.cursor = *point;
                let report = self.runtime.dispatch_pointer_down(*point, false);
                reports.push(self.apply_report(report, event_loop));
                let report = self.runtime.dispatch_pointer_up(*point);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::Drag { start, end } => {
                self.cursor = *start;
                let report = self.runtime.dispatch_pointer_down(*start, false);
                reports.push(self.apply_report(report, event_loop));
                self.cursor = *end;
                let report = self.runtime.dispatch_pointer_move(*end);
                reports.push(self.apply_report(report, event_loop));
                let report = self.runtime.dispatch_pointer_up(*end);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::PointerDrag {
                start,
                end,
                button,
                modifiers,
            } => {
                self.cursor = *start;
                let report = self
                    .runtime
                    .dispatch_pointer_down_with_button(*start, *button, *modifiers);
                reports.push(self.apply_report(report, event_loop));
                self.cursor = *end;
                let report = self
                    .runtime
                    .dispatch_pointer_move_with_modifiers(*end, *modifiers);
                reports.push(self.apply_report(report, event_loop));
                let report = self
                    .runtime
                    .dispatch_pointer_up_with_button(*end, *button, *modifiers);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::Text(text) => {
                let report = self.runtime.dispatch_ime_commit(text);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::KeyDown(key) => {
                let report = self.runtime.dispatch_key(*key);
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::Scroll { point, delta_y } => {
                let report = self
                    .runtime
                    .dispatch_pointer_scroll(*point, crate::Dp::new(*delta_y as f32));
                reports.push(self.apply_report(report, event_loop));
            }
            crate::NativeViewSmokeInput::WindowCloseRequest => {
                let report = self.runtime.dispatch_window_close_requested();
                reports.push(self.apply_report(report, event_loop));
            }
        }
        reports
    }

    fn apply_report(
        &mut self,
        mut report: crate::native::NativeViewInputDispatchReport,
        event_loop: &ActiveEventLoop,
    ) -> crate::native::NativeViewInputDispatchReport {
        let (executor, commands) = self.runtime.take_pending_app_command_dispatch();
        let effect_executed = crate::native::dispatch_deferred_native_view_app_commands(
            &mut report,
            executor,
            commands,
        );
        if effect_executed {
            self.runtime.refresh_live_view_after_app_effect(&mut report);
        }
        if let Some(plan) = report.redraw_plan.take() {
            self.plan = plan;
            self.window.request_redraw();
        }
        if report.quit_requested {
            event_loop.exit();
        }
        self.sync_text_input();
        #[cfg(feature = "accessibility")]
        self.sync_accessibility();
        self.schedule_tick();
        report
    }

    #[cfg(feature = "accessibility")]
    fn process_accessibility_event(&mut self, event: &WindowEvent) {
        self.accessibility.process_event(&self.window, event);
    }

    #[cfg(feature = "accessibility")]
    fn sync_accessibility(&mut self) {
        let logical = self.physical_size.to_logical::<f64>(self.scale_factor);
        let content_offset_y = self.menu_content_offset_y();
        let menu = self
            .menu_surface
            .as_ref()
            .map(|menu| menu.accessibility_snapshot());
        self.accessibility.update(
            &self.title,
            Rect {
                x: 0,
                y: 0,
                width: logical.width.round().clamp(1.0, f64::from(i32::MAX)) as i32,
                height: logical.height.round().clamp(1.0, f64::from(i32::MAX)) as i32,
            },
            self.scale_factor,
            content_offset_y,
            menu,
            &self.plan,
            self.runtime.current_interaction_plan(),
            self.runtime.focused_widget(),
        );
    }

    #[cfg(feature = "accessibility")]
    fn dispatch_accessibility_actions(&mut self, event_loop: &ActiveEventLoop) {
        use crate::linux_direct_accessibility::LinuxAccessibilityTarget;
        use crate::linux_direct_menu::{LinuxMenuAccessibilityTarget, LinuxMenuInputResult};
        use accesskit::{Action, ActionData};
        let text_context = self.draw_text_context.clone();

        for action in self.accessibility.take_actions() {
            match action.target {
                LinuxAccessibilityTarget::View(target) => {
                    let reports = match (action.request.action, action.request.data.as_ref()) {
                        (Action::Focus, _) => {
                            vec![self.runtime.dispatch_accessibility_focus(target.widget)]
                        }
                        (Action::Click, _) => {
                            vec![self.runtime.dispatch_pointer_click(Point {
                                x: target.bounds.x + target.bounds.width.max(1) / 2,
                                y: target.bounds.y + target.bounds.height.max(1) / 2,
                            })]
                        }
                        #[cfg(feature = "text-input-core")]
                        (Action::SetValue, Some(ActionData::Value(value))) => vec![self
                            .runtime
                            .dispatch_accessibility_set_value(target.widget, value)],
                        #[cfg(feature = "text-input-core")]
                        (Action::ReplaceSelectedText, Some(ActionData::Value(value))) => {
                            let focus = self.runtime.dispatch_accessibility_focus(target.widget);
                            let replace = self.runtime.dispatch_text_input(value);
                            vec![focus, replace]
                        }
                        _ => Vec::new(),
                    };
                    for report in reports {
                        self.apply_report(report, event_loop);
                    }
                }
                LinuxAccessibilityTarget::Menu(target) => {
                    let result = match (action.request.action, target) {
                        (Action::Focus, LinuxMenuAccessibilityTarget::Root(root)) => self
                            .menu_surface
                            .as_mut()
                            .is_some_and(|menu| menu.accessibility_focus_root(root, &text_context))
                            .then_some(LinuxMenuInputResult::Redraw),
                        (Action::Focus, LinuxMenuAccessibilityTarget::Row(row)) => self
                            .menu_surface
                            .as_mut()
                            .is_some_and(|menu| menu.accessibility_focus_row(row))
                            .then_some(LinuxMenuInputResult::Redraw),
                        (Action::Click, LinuxMenuAccessibilityTarget::Root(root)) => self
                            .menu_surface
                            .as_mut()
                            .map(|menu| menu.accessibility_activate_root(root, &text_context)),
                        (Action::Click, LinuxMenuAccessibilityTarget::Row(row)) => self
                            .menu_surface
                            .as_mut()
                            .map(|menu| menu.accessibility_activate_row(row, &text_context)),
                        _ => None,
                    };
                    match result {
                        Some(LinuxMenuInputResult::Command(command)) => {
                            self.menu_surface_command_count =
                                self.menu_surface_command_count.saturating_add(1);
                            let report = self.runtime.dispatch_app_command(command);
                            self.apply_report(report, event_loop);
                        }
                        Some(LinuxMenuInputResult::Redraw) => {
                            self.window.request_redraw();
                        }
                        Some(LinuxMenuInputResult::Ignored) | None => {}
                    }
                    self.sync_accessibility();
                }
            }
        }
    }

    #[cfg(feature = "accessibility")]
    const fn accessibility_node_count(&self) -> usize {
        self.accessibility.node_count()
    }

    #[cfg(feature = "accessibility")]
    const fn accessibility_action_count(&self) -> usize {
        self.accessibility.action_count()
    }

    fn schedule_tick(&mut self) {
        self.next_tick = self
            .runtime
            .transient_poll_interval_ms()
            .map(|delay| Instant::now() + Duration::from_millis(delay.max(1)));
    }

    fn sync_text_input(&self) {
        let accepts = self.runtime.accepts_committed_text_input();
        self.window.set_ime_allowed(accepts);
        if accepts {
            if let Some(rect) = self.runtime.text_input_caret_rect() {
                self.window.set_ime_cursor_area(
                    Position::Logical(LogicalPosition::new(
                        f64::from(rect.x),
                        f64::from(rect.y.saturating_add(self.menu_content_offset_y())),
                    )),
                    Size::Logical(LogicalSize::new(
                        f64::from(rect.width.max(1)),
                        f64::from(rect.height.max(1)),
                    )),
                );
            }
        }
    }

    fn redraw(&mut self) -> Result<(), String> {
        let width = NonZeroU32::new(self.physical_size.width.max(1))
            .ok_or_else(|| "Linux direct surface width is zero".to_string())?;
        let height = NonZeroU32::new(self.physical_size.height.max(1))
            .ok_or_else(|| "Linux direct surface height is zero".to_string())?;
        self.surface
            .resize(width, height)
            .map_err(|error| format!("could not resize software surface: {error}"))?;
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|error| format!("could not acquire software buffer: {error}"))?;
        let expected_len = (width.get() as usize)
            .checked_mul(height.get() as usize)
            .ok_or_else(|| "software buffer dimensions overflow usize".to_string())?;
        if buffer.len() != expected_len {
            return Err(format!(
                "software buffer length {} does not match frame size {expected_len}",
                buffer.len()
            ));
        }
        render_linux_direct_frame(
            &mut buffer,
            &self.plan,
            self.menu_surface.as_ref(),
            width.get(),
            height.get(),
            self.scale_factor,
            self.theme,
            &self.draw_text_context,
            &mut self.icon_cache,
        )?;
        if self.retain_frame {
            self.last_frame.clear();
            self.last_frame.extend_from_slice(&buffer);
        }
        self.window.pre_present_notify();
        buffer
            .present()
            .map_err(|error| format!("could not present software buffer: {error}"))?;
        self.presented_once = true;
        Ok(())
    }

    fn capture_png(&self, path: &Path) -> Result<crate::NativeViewCaptureEvidence, String> {
        if self.last_frame.is_empty() {
            return Err("the Linux direct host has not presented a frame".to_string());
        }
        write_linux_direct_png(
            path,
            self.physical_size.width,
            self.physical_size.height,
            &self.last_frame,
        )?;
        let logical = self.physical_size.to_logical::<f64>(self.scale_factor);
        #[cfg(feature = "linux-direct")]
        let backend = "winit_softbuffer_pango_cairo";
        #[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
        let backend = "winit_softbuffer_cosmic_text_tiny_skia";
        #[cfg(feature = "linux-direct")]
        let typography = linux_direct_native_typography_profile(
            self.plan.typography_scale(),
            Some(&self.text_context),
        );
        #[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
        let typography = crate::linux_direct_lite::linux_lite_typography_profile(
            self.plan.typography_scale(),
            &self.text_context,
        );
        Ok(crate::NativeViewCaptureEvidence {
            platform: "linux",
            backend,
            display_server: Some(self.display_server),
            logical_width: logical.width.round().max(1.0) as u32,
            logical_height: logical.height.round().max(1.0) as u32,
            pixel_width: self.physical_size.width.max(1),
            pixel_height: self.physical_size.height.max(1),
            scale_factor: self.scale_factor,
            typography_scale: self.plan.typography_scale(),
            typography,
        })
    }
}

fn linux_direct_display_server(window: &WinitWindow) -> &'static str {
    match window.display_handle().map(|handle| handle.as_raw()) {
        Ok(RawDisplayHandle::Wayland(_)) => "wayland",
        Ok(RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_)) => "x11",
        Ok(_) | Err(_) => "unknown",
    }
}

fn byte_to_char_index(text: &str, byte: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < byte.min(text.len()))
        .count()
}

fn menu_command_for_key(
    menu: &MenuSpec,
    key: &Key,
    modifiers: ModifiersState,
) -> Option<crate::Command> {
    for item in &menu.items {
        match item {
            MenuItemSpec::Command {
                command,
                enabled: true,
                accelerator: Some(accelerator),
                ..
            } if accelerator_matches(*accelerator, key, modifiers) => {
                return Some(command.clone());
            }
            MenuItemSpec::Submenu {
                enabled: true,
                menu,
                ..
            } => {
                if let Some(command) = menu_command_for_key(menu, key, modifiers) {
                    return Some(command);
                }
            }
            _ => {}
        }
    }
    None
}

fn accelerator_matches(accelerator: ZsAccelerator, key: &Key, modifiers: ModifiersState) -> bool {
    if accelerator.uses_primary() != modifiers.control_key()
        || accelerator.uses_shift() != modifiers.shift_key()
        || accelerator.uses_alt() != modifiers.alt_key()
        || accelerator.uses_super() != modifiers.super_key()
    {
        return false;
    }
    match (accelerator.key(), key) {
        (ZsAcceleratorKey::Character(expected), Key::Character(actual)) => actual
            .chars()
            .next()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(&expected)),
        (ZsAcceleratorKey::Enter, Key::Named(NamedKey::Enter))
        | (ZsAcceleratorKey::Escape, Key::Named(NamedKey::Escape))
        | (ZsAcceleratorKey::Tab, Key::Named(NamedKey::Tab))
        | (ZsAcceleratorKey::Space, Key::Named(NamedKey::Space))
        | (ZsAcceleratorKey::Backspace, Key::Named(NamedKey::Backspace))
        | (ZsAcceleratorKey::Delete, Key::Named(NamedKey::Delete))
        | (ZsAcceleratorKey::Up, Key::Named(NamedKey::ArrowUp))
        | (ZsAcceleratorKey::Down, Key::Named(NamedKey::ArrowDown))
        | (ZsAcceleratorKey::Left, Key::Named(NamedKey::ArrowLeft))
        | (ZsAcceleratorKey::Right, Key::Named(NamedKey::ArrowRight))
        | (ZsAcceleratorKey::Home, Key::Named(NamedKey::Home))
        | (ZsAcceleratorKey::End, Key::Named(NamedKey::End))
        | (ZsAcceleratorKey::PageUp, Key::Named(NamedKey::PageUp))
        | (ZsAcceleratorKey::PageDown, Key::Named(NamedKey::PageDown)) => true,
        _ => false,
    }
}

fn linux_pointer_modifiers(modifiers: ModifiersState) -> crate::ZsPointerModifiers {
    crate::ZsPointerModifiers::new(
        modifiers.shift_key(),
        modifiers.control_key(),
        modifiers.alt_key(),
        modifiers.super_key(),
    )
}

fn linux_pointer_button(button: MouseButton) -> crate::ZsPointerButton {
    match button {
        MouseButton::Left => crate::ZsPointerButton::Primary,
        MouseButton::Right => crate::ZsPointerButton::Secondary,
        MouseButton::Middle => crate::ZsPointerButton::Middle,
        MouseButton::Back => crate::ZsPointerButton::Auxiliary(4),
        MouseButton::Forward => crate::ZsPointerButton::Auxiliary(5),
        MouseButton::Other(button) => crate::ZsPointerButton::Auxiliary(button),
    }
}

fn focus_transition_lost(previously_focused: &mut bool, focused: bool) -> bool {
    let lost = *previously_focused && !focused;
    *previously_focused = focused;
    lost
}

#[derive(Clone)]
#[allow(dead_code)]
struct LinuxIconRaster {
    width: i32,
    height: i32,
    premultiplied_bgra: Vec<u8>,
}

#[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
fn render_linux_direct_frame(
    frame: &mut [u32],
    plan: &NativeDrawPlan,
    menu_surface: Option<&crate::linux_direct_menu::LinuxDirectMenuSurface>,
    width: u32,
    height: u32,
    scale_factor: f64,
    theme: WinitTheme,
    text_context: &LinuxDirectTextContext,
    _icon_cache: &mut HashMap<(ZsIcon, u32, bool), LinuxIconRaster>,
) -> Result<(), String> {
    crate::linux_direct_lite::render_linux_direct_lite_frame(
        frame,
        plan,
        menu_surface,
        width,
        height,
        scale_factor,
        matches!(theme, WinitTheme::Dark),
        text_context,
    )
}

#[cfg(feature = "linux-direct")]
fn render_linux_direct_frame(
    frame: &mut [u32],
    plan: &NativeDrawPlan,
    menu_surface: Option<&crate::linux_direct_menu::LinuxDirectMenuSurface>,
    width: u32,
    height: u32,
    scale_factor: f64,
    theme: WinitTheme,
    pango_context: &pango::Context,
    icon_cache: &mut HashMap<(ZsIcon, u32, bool), LinuxIconRaster>,
) -> Result<(), String> {
    let width_i32 = i32::try_from(width).map_err(|_| "frame width exceeds i32".to_string())?;
    let height_i32 = i32::try_from(height).map_err(|_| "frame height exceeds i32".to_string())?;
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| "software frame dimensions overflow usize".to_string())?;
    if frame.len() != expected_len {
        return Err(format!(
            "software frame length {} does not match {width}x{height}",
            frame.len()
        ));
    }
    let stride = Format::Rgb24
        .stride_for_width(width)
        .map_err(|error| format!("could not compute Cairo image stride: {error}"))?;
    if usize::try_from(stride).ok() != Some(width as usize * std::mem::size_of::<u32>()) {
        return Err(format!(
            "Cairo RGB24 stride {stride} does not match the software buffer width {width}"
        ));
    }
    // SAFETY: `frame` owns `width * height` native-endian RGB24 pixels and
    // remains borrowed for the complete lifetime of `image`. All Cairo state
    // and the image surface are dropped before this function returns, so no
    // Cairo reference can outlive the softbuffer guard that supplied the slice.
    let image = unsafe {
        ImageSurface::create_for_data_unsafe(
            frame.as_mut_ptr().cast::<u8>(),
            Format::Rgb24,
            width_i32,
            height_i32,
            stride,
        )
    }
    .map_err(|error| format!("could not bind Cairo to the software buffer: {error}"))?;
    let context = CairoContext::new(&image)
        .map_err(|error| format!("could not create Cairo drawing context: {error}"))?;
    let palette = NativeDrawPalette::for_mode(plan.theme_mode, matches!(theme, WinitTheme::Dark));
    set_cairo_source(&context, palette.surface);
    context
        .paint()
        .map_err(|error| format!("could not clear Cairo image surface: {error}"))?;
    context.scale(scale_factor, scale_factor);
    let draw_context = pango_context.clone();
    pangocairo::functions::update_context(&context, &draw_context);
    if let Some(menu) = menu_surface {
        context
            .save()
            .map_err(|error| format!("could not save Cairo menu state: {error}"))?;
        let content_offset = menu.content_offset_y();
        context.rectangle(
            0.0,
            f64::from(content_offset),
            f64::from(width) / scale_factor,
            (f64::from(height) / scale_factor - f64::from(content_offset)).max(1.0),
        );
        context.clip();
        context.translate(0.0, f64::from(content_offset));
        pangocairo::functions::update_context(&context, &draw_context);
    }
    let mut sink = LinuxDirectDrawSink::new(
        &context,
        draw_context.clone(),
        palette,
        plan.typography_scale(),
        scale_factor,
        icon_cache,
    );
    sink.draw_plan(plan);
    drop(sink);
    if menu_surface.is_some() {
        context
            .restore()
            .map_err(|error| format!("could not restore Cairo menu state: {error}"))?;
    }
    if let Some(menu) = menu_surface {
        pangocairo::functions::update_context(&context, &draw_context);
        let mut canvas =
            crate::linux_direct_menu::LinuxCairoMenuCanvas::new(&context, &draw_context);
        menu.draw(&mut canvas, palette);
    }
    drop(context);
    image.flush();
    drop(image);
    Ok(())
}

#[cfg(feature = "linux-direct")]
struct LinuxDirectTextLayout {
    context: pango::Context,
}

#[cfg(feature = "linux-direct")]
impl LinuxDirectTextLayout {
    fn new(context: pango::Context) -> Self {
        Self { context }
    }

    fn pango_layout(&self, text: &str, style: &TextStyle, bounds: Option<Rect>) -> pango::Layout {
        let layout = pango::Layout::new(&self.context);
        configure_linux_pango_layout(&layout, text, style, bounds);
        layout
    }
}

#[cfg(feature = "linux-direct")]
impl TextLayout for LinuxDirectTextLayout {
    fn measure(&self, text: &str, style: &TextStyle, max_width: i32) -> ZsSize {
        if text.is_empty() {
            return ZsSize {
                width: 0,
                height: 0,
            };
        }
        let bounds = (max_width > 0).then(|| Rect {
            x: 0,
            y: 0,
            width: max_width,
            height: i32::MAX / 4,
        });
        let layout = self.pango_layout(text, style, bounds);
        let (width, height) = layout.pixel_size();
        ZsSize {
            width: width.max(0),
            height: height.max(0),
        }
    }

    fn layout_runs(&self, text: &str, _style: &TextStyle, bounds: Rect) -> Vec<TextRun> {
        if text.is_empty() {
            Vec::new()
        } else {
            vec![TextRun {
                text: text.to_string(),
                bounds,
            }]
        }
    }
}

#[cfg(feature = "linux-direct")]
struct LinuxDirectDrawSink<'a> {
    context: &'a CairoContext,
    palette: NativeDrawPalette,
    style_resolver: NativeDrawTextStyleResolver,
    text_layout: LinuxDirectTextLayout,
    scale_factor: f64,
    icon_cache: &'a mut HashMap<(ZsIcon, u32, bool), LinuxIconRaster>,
    clip_depth: usize,
}

#[cfg(feature = "linux-direct")]
impl<'a> LinuxDirectDrawSink<'a> {
    fn new(
        context: &'a CairoContext,
        pango_context: pango::Context,
        palette: NativeDrawPalette,
        typography_scale: f32,
        scale_factor: f64,
        icon_cache: &'a mut HashMap<(ZsIcon, u32, bool), LinuxIconRaster>,
    ) -> Self {
        Self {
            context,
            palette,
            style_resolver: NativeDrawTextStyleResolver::from_profile(
                linux_direct_native_typography_profile(typography_scale, Some(&pango_context)),
                palette,
            ),
            text_layout: LinuxDirectTextLayout::new(pango_context),
            scale_factor,
            icon_cache,
            clip_depth: 0,
        }
    }

    fn set_source(&self, color: Color) {
        set_cairo_source(self.context, color);
    }

    fn add_rect(&self, rect: Rect) {
        self.context.rectangle(
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.width.max(0)),
            f64::from(rect.height.max(0)),
        );
    }

    fn add_round_rect(&self, rect: Rect, radius: i32) {
        let x = f64::from(rect.x);
        let y = f64::from(rect.y);
        let width = f64::from(rect.width.max(0));
        let height = f64::from(rect.height.max(0));
        let radius = f64::from(radius.max(0)).min(width / 2.0).min(height / 2.0);
        if radius <= 0.0 {
            self.context.rectangle(x, y, width, height);
            return;
        }
        let right = x + width;
        let bottom = y + height;
        self.context.new_sub_path();
        self.context.arc(
            right - radius,
            y + radius,
            radius,
            -std::f64::consts::FRAC_PI_2,
            0.0,
        );
        self.context.arc(
            right - radius,
            bottom - radius,
            radius,
            0.0,
            std::f64::consts::FRAC_PI_2,
        );
        self.context.arc(
            x + radius,
            bottom - radius,
            radius,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
        );
        self.context.arc(
            x + radius,
            y + radius,
            radius,
            std::f64::consts::PI,
            std::f64::consts::PI * 1.5,
        );
        self.context.close_path();
    }

    fn draw_text(&self, command: &NativeDrawTextCommand) {
        let style = self.style_resolver.resolve_text_style(command.style);
        let layout = self
            .text_layout
            .pango_layout(&command.text, &style, Some(command.bounds));
        let (text_width, text_height) = layout.pixel_size();
        let x = if style.wrap == TextWrap::NoWrap && !style.ellipsis {
            match style.horizontal_align {
                HorizontalAlign::Start => command.bounds.x,
                HorizontalAlign::Center => {
                    command.bounds.x + (command.bounds.width - text_width).max(0) / 2
                }
                HorizontalAlign::End => {
                    command.bounds.x + (command.bounds.width - text_width).max(0)
                }
            }
        } else {
            command.bounds.x
        };
        let y = match style.vertical_align {
            VerticalAlign::Start => command.bounds.y,
            VerticalAlign::Center => {
                command.bounds.y + (command.bounds.height - text_height).max(0) / 2
            }
            VerticalAlign::End => command.bounds.y + (command.bounds.height - text_height).max(0),
        };
        if self.context.save().is_err() {
            return;
        }
        self.add_rect(command.bounds);
        self.context.clip();
        self.set_source(style.color);
        self.context.move_to(f64::from(x), f64::from(y));
        pangocairo::functions::show_layout(self.context, &layout);
        let _ = self.context.restore();
    }

    fn draw_icon(&mut self, command: &NativeDrawIconCommand) {
        #[cfg(feature = "linux-system-icons")]
        {
            let logical_size = command.bounds.width.min(command.bounds.height).max(1) as u32;
            let physical_size = (f64::from(logical_size) * self.scale_factor)
                .ceil()
                .clamp(1.0, f64::from(u16::MAX)) as u32;
            let theme_aware = command.color_mode == NativeIconColorMode::ThemeAware;
            let key = (command.icon, physical_size, theme_aware);
            if !self.icon_cache.contains_key(&key) {
                if let Some(raster) = load_linux_icon_raster(
                    command.icon,
                    physical_size,
                    theme_aware.then(|| self.palette.resolve(command.color)),
                ) {
                    self.icon_cache.insert(key, raster);
                }
            }
            if let Some(raster) = self.icon_cache.get(&key) {
                let Ok(surface) = ImageSurface::create_for_data(
                    raster.premultiplied_bgra.clone(),
                    Format::ARgb32,
                    raster.width,
                    raster.height,
                    raster.width.saturating_mul(4),
                ) else {
                    return;
                };
                if self.context.save().is_err() {
                    return;
                }
                self.add_rect(command.bounds);
                self.context.clip();
                self.context
                    .translate(f64::from(command.bounds.x), f64::from(command.bounds.y));
                self.context.scale(
                    f64::from(command.bounds.width.max(1)) / f64::from(raster.width.max(1)),
                    f64::from(command.bounds.height.max(1)) / f64::from(raster.height.max(1)),
                );
                if self.context.set_source_surface(&surface, 0.0, 0.0).is_ok() {
                    let _ = self.context.paint();
                }
                let _ = self.context.restore();
                return;
            }
        }

        crate::linux_direct_icons::draw_symbolic_icon(
            self.context,
            command,
            self.palette.resolve(command.color),
        );
    }

    fn draw_image(&self, command: &NativeDrawImageCommand) {
        let Ok(width) = i32::try_from(command.frame.width()) else {
            return;
        };
        let Ok(height) = i32::try_from(command.frame.height()) else {
            return;
        };
        if width <= 0
            || height <= 0
            || command.source.width <= 0
            || command.source.height <= 0
            || command.bounds.width <= 0
            || command.bounds.height <= 0
        {
            return;
        }
        let Ok(stride) = Format::ARgb32.stride_for_width(command.frame.width()) else {
            return;
        };
        if usize::try_from(stride).ok().and_then(|stride| {
            usize::try_from(height)
                .ok()
                .and_then(|height| stride.checked_mul(height))
        }) != Some(command.frame.decoded_bytes())
        {
            return;
        }
        let Ok(surface) = ImageSurface::create_for_data(
            command.frame.premultiplied_bgra8().to_vec(),
            Format::ARgb32,
            width,
            height,
            stride,
        ) else {
            return;
        };
        if self.context.save().is_err() {
            return;
        }
        self.add_rect(command.bounds);
        self.context.clip();
        self.context
            .translate(f64::from(command.bounds.x), f64::from(command.bounds.y));
        self.context.scale(
            f64::from(command.bounds.width) / f64::from(command.source.width),
            f64::from(command.bounds.height) / f64::from(command.source.height),
        );
        if self
            .context
            .set_source_surface(
                &surface,
                -f64::from(command.source.x),
                -f64::from(command.source.y),
            )
            .is_ok()
        {
            self.context
                .source()
                .set_filter(match command.interpolation {
                    NativeImageInterpolation::Nearest => cairo::Filter::Nearest,
                    NativeImageInterpolation::Smooth => cairo::Filter::Good,
                });
            let _ = self.context.paint();
        }
        let _ = self.context.restore();
    }

    fn push_clip(&mut self, rect: Rect) {
        if self.context.save().is_ok() {
            self.add_rect(rect);
            self.context.clip();
            self.clip_depth += 1;
        }
    }

    fn pop_clip(&mut self) {
        if self.clip_depth > 0 {
            let _ = self.context.restore();
            self.clip_depth -= 1;
        }
    }
}

#[cfg(feature = "linux-direct")]
impl Drop for LinuxDirectDrawSink<'_> {
    fn drop(&mut self) {
        while self.clip_depth > 0 {
            self.pop_clip();
        }
    }
}

#[cfg(feature = "linux-direct")]
impl NativeDrawCommandSink for LinuxDirectDrawSink<'_> {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                self.set_source(self.palette.resolve_source_fill(*fill));
                self.add_rect(*rect);
                let _ = self.context.fill();
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => {
                self.set_source(self.palette.resolve_source_fill(*stroke));
                self.context.set_line_width(f64::from((*width).max(1)));
                self.add_rect(*rect);
                let _ = self.context.stroke();
            }
            NativeDrawCommand::StrokeArc {
                rect,
                stroke,
                width,
                start_degrees,
                sweep_degrees,
            } => {
                self.set_source(self.palette.resolve_source_fill(*stroke));
                self.context.set_line_width(f64::from((*width).max(1)));
                let start = f64::from(*start_degrees).to_radians();
                let end = f64::from(start_degrees.saturating_add(*sweep_degrees)).to_radians();
                self.context.arc(
                    f64::from(rect.x) + f64::from(rect.width) / 2.0,
                    f64::from(rect.y) + f64::from(rect.height) / 2.0,
                    f64::from(rect.width.min(rect.height).max(0)) / 2.0,
                    start,
                    end,
                );
                let _ = self.context.stroke();
            }
            NativeDrawCommand::FillTriangle { points, fill } => {
                self.set_source(self.palette.resolve_source_fill(*fill));
                self.context
                    .move_to(f64::from(points[0].x), f64::from(points[0].y));
                for point in &points[1..] {
                    self.context.line_to(f64::from(point.x), f64::from(point.y));
                }
                self.context.close_path();
                let _ = self.context.fill();
            }
            NativeDrawCommand::RoundRect {
                rect,
                fill,
                stroke,
                radius,
            } => {
                self.add_round_rect(*rect, *radius);
                self.set_source(self.palette.resolve_source_fill(*fill));
                if stroke.is_some() {
                    let _ = self.context.fill_preserve();
                } else {
                    let _ = self.context.fill();
                }
                if let Some(stroke) = stroke {
                    self.set_source(self.palette.resolve_source_fill(*stroke));
                    self.context.set_line_width(1.0);
                    let _ = self.context.stroke();
                }
            }
            NativeDrawCommand::RoundFill { rect, fill, radius } => {
                self.add_round_rect(*rect, *radius);
                self.set_source(self.palette.resolve_source_fill(*fill));
                let _ = self.context.fill();
            }
            NativeDrawCommand::Text(command) => self.draw_text(command),
            #[cfg(feature = "password-box")]
            NativeDrawCommand::SecureText(command) => {
                let rendered = command.rendered_text();
                self.draw_text(&NativeDrawTextCommand::new(
                    rendered.as_str(),
                    command.bounds,
                    command.style,
                ));
            }
            NativeDrawCommand::Icon(command) => self.draw_icon(command),
            NativeDrawCommand::Image(command) => self.draw_image(command),
            NativeDrawCommand::PushClip { rect } => self.push_clip(*rect),
            NativeDrawCommand::PopClip => self.pop_clip(),
        }
    }
}

#[cfg(feature = "linux-direct")]
fn set_cairo_source(context: &CairoContext, color: Color) {
    context.set_source_rgba(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    );
}

#[cfg(feature = "linux-system-icons")]
fn load_linux_icon_raster(
    icon: ZsIcon,
    physical_size: u32,
    recolor: Option<Color>,
) -> Option<LinuxIconRaster> {
    let lookup_size = u16::try_from(physical_size.min(u32::from(u16::MAX))).ok()?;
    let theme = linux_direct_icon_theme();
    let path = freedesktop_icons::lookup(icon.gtk_symbolic_name())
        .with_theme(theme)
        .with_size(lookup_size)
        .with_cache()
        .find();
    let pixbuf = path
        .and_then(|path| {
            gdk_pixbuf::Pixbuf::from_file_at_scale(
                path,
                physical_size as i32,
                physical_size as i32,
                true,
            )
            .ok()
        })
        .or_else(|| {
            let loader = gdk_pixbuf::PixbufLoader::with_type("svg").ok()?;
            loader.set_size(physical_size as i32, physical_size as i32);
            loader.write(crate::bundled_fluent_icon_svg(icon)).ok()?;
            loader.close().ok()?;
            loader.pixbuf()
        })?;
    pixbuf_to_linux_icon_raster(&pixbuf, recolor)
}

#[cfg(feature = "linux-system-icons")]
fn pixbuf_to_linux_icon_raster(
    pixbuf: &gdk_pixbuf::Pixbuf,
    recolor: Option<Color>,
) -> Option<LinuxIconRaster> {
    let width = pixbuf.width();
    let height = pixbuf.height();
    let channels = pixbuf.n_channels();
    let stride = usize::try_from(pixbuf.rowstride()).ok()?;
    if width <= 0 || height <= 0 || !(channels == 3 || channels == 4) {
        return None;
    }
    let bytes = pixbuf.read_pixel_bytes();
    let source = bytes.as_ref();
    let mut target = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        let start = row.checked_mul(stride)?;
        let needed = width as usize * channels as usize;
        let pixels = source.get(start..start.checked_add(needed)?)?;
        for pixel in pixels.chunks_exact(channels as usize) {
            let source_alpha = if channels == 4 { pixel[3] } else { 255 };
            let (red, green, blue, alpha) = if let Some(color) = recolor {
                let alpha = ((u16::from(source_alpha) * u16::from(color.a) + 127) / 255) as u8;
                (color.r, color.g, color.b, alpha)
            } else {
                (pixel[0], pixel[1], pixel[2], source_alpha)
            };
            let premultiply =
                |channel: u8| ((u16::from(channel) * u16::from(alpha) + 127) / 255) as u8;
            target.extend_from_slice(&[
                premultiply(blue),
                premultiply(green),
                premultiply(red),
                alpha,
            ]);
        }
    }
    Some(LinuxIconRaster {
        width,
        height,
        premultiplied_bgra: target,
    })
}

#[cfg(feature = "linux-system-icons")]
fn linux_direct_icon_theme() -> &'static str {
    static THEME: OnceLock<String> = OnceLock::new();
    THEME
        .get_or_init(|| {
            std::env::var("ZSUI_LINUX_ICON_THEME")
                .ok()
                .filter(|theme| !theme.trim().is_empty())
                .or_else(freedesktop_icons::default_theme_gtk)
                .unwrap_or_else(|| "Adwaita".to_string())
        })
        .as_str()
}

pub(crate) fn linux_direct_configured_font_name() -> &'static str {
    static FONT: OnceLock<String> = OnceLock::new();
    FONT.get_or_init(|| {
        std::env::var("ZSUI_LINUX_UI_FONT")
            .ok()
            .filter(|font| !font.trim().is_empty())
            .or_else(|| {
                std::process::Command::new("gsettings")
                    .args(["get", "org.gnome.desktop.interface", "font-name"])
                    .output()
                    .ok()
                    .filter(|output| output.status.success())
                    .and_then(|output| String::from_utf8(output.stdout).ok())
                    .map(|value| value.trim().trim_matches('\'').to_string())
                    .filter(|font| !font.is_empty())
            })
            .unwrap_or_else(|| "Sans 10.5".to_string())
    })
}

#[cfg(feature = "linux-direct")]
fn linux_direct_font_description() -> pango::FontDescription {
    pango::FontDescription::from_string(linux_direct_configured_font_name())
}

#[cfg(feature = "linux-direct")]
fn linux_direct_ui_font_family() -> String {
    linux_direct_font_description()
        .family()
        .map(|family| family.to_string())
        .filter(|family| !family.trim().is_empty())
        .unwrap_or_else(|| "Sans".to_string())
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
pub(crate) fn linux_direct_ui_font_scale() -> f32 {
    let description = linux_direct_font_description();
    let size = description.size();
    if size <= 0 {
        return 1.0;
    }
    let logical_pixels = if description.is_size_absolute() {
        size as f32 / pango::SCALE as f32
    } else {
        size as f32 / pango::SCALE as f32 * (96.0 / 72.0)
    };
    (logical_pixels / 14.0).clamp(0.75, 3.0)
}

#[cfg(feature = "linux-direct")]
fn linux_direct_text_context() -> LinuxDirectTextContext {
    pangocairo::FontMap::default().create_context()
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
fn linux_direct_text_shaping_backend(
    context: pango::Context,
) -> crate::native_input_visuals::NativeTextShapingBackend {
    crate::native_input_visuals::NativeTextShapingBackend::platform(LinuxDirectPangoTextShaper {
        context,
    })
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
struct LinuxDirectPangoTextShaper {
    context: pango::Context,
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
impl crate::native_input_visuals::NativeTextShaper for LinuxDirectPangoTextShaper {
    fn debug_name(&self) -> &'static str {
        "LinuxDirect(PangoContext)"
    }

    fn typography_scale(&self) -> f32 {
        linux_direct_ui_font_scale()
    }

    fn shape_line(&self, text: &str) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
        shape_linux_direct_text_line(&self.context, text)
    }
}

#[cfg(all(feature = "linux-direct-lite", not(feature = "linux-direct")))]
fn linux_direct_text_context() -> LinuxDirectTextContext {
    crate::linux_direct_lite::LinuxLiteTextSystem::new(linux_direct_configured_font_name())
}

#[cfg(all(
    feature = "text-input-core",
    feature = "linux-direct-lite",
    not(feature = "linux-direct")
))]
fn linux_direct_lite_text_shaping_backend(
    system: crate::linux_direct_lite::LinuxLiteTextSystem,
) -> crate::native_input_visuals::NativeTextShapingBackend {
    crate::native_input_visuals::NativeTextShapingBackend::platform(LinuxDirectLiteTextShaper {
        system,
    })
}

#[cfg(all(
    feature = "text-input-core",
    feature = "linux-direct-lite",
    not(feature = "linux-direct")
))]
struct LinuxDirectLiteTextShaper {
    system: crate::linux_direct_lite::LinuxLiteTextSystem,
}

#[cfg(all(
    feature = "text-input-core",
    feature = "linux-direct-lite",
    not(feature = "linux-direct")
))]
impl crate::native_input_visuals::NativeTextShaper for LinuxDirectLiteTextShaper {
    fn debug_name(&self) -> &'static str {
        "LinuxDirectLite(CosmicText)"
    }

    fn typography_scale(&self) -> f32 {
        self.system.ui_scale()
    }

    fn shape_line(&self, text: &str) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
        crate::linux_direct_lite::shape_linux_lite_text_line(&self.system, text)
    }
}

#[cfg(feature = "linux-direct")]
fn linux_direct_native_typography_profile(
    typography_scale: f32,
    context: Option<&pango::Context>,
) -> crate::NativeTypographyProfile {
    let font_family = linux_direct_ui_font_family();
    let mut profile = crate::NativeTypographyProfile::new(
        crate::ZsTypographyPlatformStyle::Gtk,
        "fontconfig_pango_context",
        font_family.clone(),
        "Monospace",
        font_family,
        typography_scale,
        "pango_cairo_softbuffer",
    )
    .with_configured_ui_font(linux_direct_configured_font_name());
    if let Some(context) = context {
        let mut font = pango::FontDescription::new();
        font.set_family(&profile.ui_font_family);
        font.set_absolute_size(f64::from(profile.body_metrics.size) * f64::from(pango::SCALE));
        let metrics = context.metrics(Some(&font), None);
        let ascent = metrics.ascent() as f32 / pango::SCALE as f32;
        let descent = metrics.descent() as f32 / pango::SCALE as f32;
        let leading = (profile.body_metrics.line_height - ascent - descent).max(0.0);
        profile = profile.with_body_vertical_metrics(ascent, descent, leading);
    }
    profile
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
pub(crate) fn shape_linux_direct_text_line(
    context: &pango::Context,
    text: &str,
) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
    use crate::native_input_visuals::{
        NativeShapedTextCaret, NativeShapedTextCluster, NativeShapedTextLine,
    };

    if text.is_empty() {
        return None;
    }
    let body = crate::TextRole::Body.metrics_for(crate::ZsTypographyPlatformStyle::Gtk);
    let typography_scale = linux_direct_ui_font_scale();
    let mut style = TextStyle::line(
        linux_direct_ui_font_family(),
        body.size * typography_scale,
        Color::rgb(0, 0, 0),
    );
    style.line_height = body.line_height * typography_scale;
    style.semantic_role = Some(crate::TextRole::Body);
    let layout = pango::Layout::new(context);
    configure_linux_pango_layout(&layout, text, &style, None);
    let boundaries = crate::native_text_edit::grapheme_boundaries(text);
    let byte_offsets = boundaries
        .iter()
        .map(|index| i32::try_from(crate::native_text_edit::char_to_byte_index(text, *index)).ok())
        .collect::<Option<Vec<_>>>()?;
    let carets = boundaries
        .iter()
        .copied()
        .zip(byte_offsets.iter().copied())
        .map(|(index, byte)| {
            let (strong, weak) = layout.cursor_pos(byte);
            NativeShapedTextCaret {
                index,
                primary_x: linux_pango_coordinate(strong.x()),
                secondary_x: linux_pango_coordinate(weak.x()),
            }
        })
        .collect::<Vec<_>>();
    let clusters = boundaries
        .windows(2)
        .zip(byte_offsets.iter().copied())
        .map(|(scalar, byte)| {
            let position = layout.index_to_pos(byte);
            NativeShapedTextCluster {
                start: scalar[0],
                end: scalar[1],
                start_x: linux_pango_coordinate(position.x()),
                end_x: linux_pango_coordinate(position.x().saturating_add(position.width())),
            }
        })
        .collect::<Vec<_>>();
    let (width, _) = layout.pixel_size();
    NativeShapedTextLine::new(width, clusters, carets)
}

#[cfg(all(feature = "text-input-core", feature = "linux-direct"))]
fn linux_pango_coordinate(value: i32) -> i32 {
    if value >= 0 {
        value.saturating_add(pango::SCALE / 2) / pango::SCALE
    } else {
        value.saturating_sub(pango::SCALE / 2) / pango::SCALE
    }
}

#[cfg(feature = "linux-direct")]
fn configure_linux_pango_layout(
    layout: &pango::Layout,
    text: &str,
    style: &TextStyle,
    bounds: Option<Rect>,
) {
    layout.set_text(text);
    let mut font = pango::FontDescription::new();
    font.set_family(&style.font_family);
    font.set_absolute_size(f64::from(style.size) * f64::from(pango::SCALE));
    font.set_weight(match style.weight {
        TextWeight::Automatic | TextWeight::Regular => pango::Weight::Normal,
        TextWeight::Medium => pango::Weight::Medium,
        TextWeight::Semibold => pango::Weight::Semibold,
        TextWeight::Bold => pango::Weight::Bold,
    });
    layout.set_font_description(Some(&font));
    if style.line_height > 0.0 {
        let metrics = layout.context().metrics(Some(&font), None);
        let natural_line_height = metrics.ascent().saturating_add(metrics.descent());
        let target_line_height = (f64::from(style.line_height.max(style.size))
            * f64::from(pango::SCALE))
        .round()
        .clamp(0.0, f64::from(i32::MAX)) as i32;
        layout.set_spacing(target_line_height.saturating_sub(natural_line_height));
    }
    layout.set_alignment(match style.horizontal_align {
        HorizontalAlign::Start => pango::Alignment::Left,
        HorizontalAlign::Center => pango::Alignment::Center,
        HorizontalAlign::End => pango::Alignment::Right,
    });
    // Pango layouts are reused across draw commands. Clear any width/height
    // constraint left by a previous wrapped or ellipsized run before applying
    // the current style.
    layout.set_width(-1);
    layout.set_height(-1);
    layout.set_wrap(pango::WrapMode::WordChar);
    if let Some(bounds) = bounds.filter(|_| style.wrap == TextWrap::Word || style.ellipsis) {
        layout.set_width(bounds.width.max(0).saturating_mul(pango::SCALE));
        if style.wrap == TextWrap::Word {
            layout.set_height(bounds.height.max(0).saturating_mul(pango::SCALE));
        }
    }
    layout.set_single_paragraph_mode(style.wrap == TextWrap::NoWrap);
    layout.set_ellipsize(if style.ellipsis {
        pango::EllipsizeMode::End
    } else {
        pango::EllipsizeMode::None
    });
}

#[cfg(feature = "image")]
fn write_linux_direct_png(
    path: &Path,
    width: u32,
    height: u32,
    frame: &[u32],
) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("could not create Linux capture directory: {error}"))?;
    }
    let file = std::fs::File::create(path)
        .map_err(|error| format!("could not create Linux capture file: {error}"))?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("could not write Linux capture header: {error}"))?;
    let mut rgb = Vec::with_capacity(frame.len() * 3);
    for pixel in frame {
        rgb.extend_from_slice(&[
            ((pixel >> 16) & 0xff) as u8,
            ((pixel >> 8) & 0xff) as u8,
            (pixel & 0xff) as u8,
        ]);
    }
    writer
        .write_image_data(&rgb)
        .map_err(|error| format!("could not write Linux capture pixels: {error}"))
}

#[cfg(not(feature = "image"))]
fn write_linux_direct_png(
    _path: &Path,
    _width: u32,
    _height: u32,
    _frame: &[u32],
) -> Result<(), String> {
    Err("enable the image feature to capture a Linux native PNG".to_string())
}

pub(crate) fn linux_direct_open_file_dialog(
    spec: &crate::FileDialogSpec,
) -> ZsuiResult<Option<Vec<PathBuf>>> {
    let mut dialog = rfd::FileDialog::new().set_title(&spec.title);
    if let Some(path) = &spec.current_path {
        dialog = dialog.set_directory(path);
    }
    for filter in &spec.filters {
        let extensions = filter
            .patterns
            .iter()
            .map(|pattern| pattern.trim_start_matches("*.").trim_start_matches('.'))
            .filter(|extension| !extension.is_empty() && *extension != "*")
            .collect::<Vec<_>>();
        if !extensions.is_empty() {
            dialog = dialog.add_filter(&filter.name, &extensions);
        }
    }
    Ok(dialog.pick_files())
}

pub(crate) fn linux_direct_save_file_dialog(
    spec: &crate::SaveFileDialogSpec,
) -> ZsuiResult<Option<PathBuf>> {
    let mut dialog = rfd::FileDialog::new().set_title(&spec.title);
    if let Some(path) = &spec.current_path {
        dialog = dialog.set_directory(path);
    }
    if let Some(name) = &spec.suggested_name {
        dialog = dialog.set_file_name(name);
    }
    for filter in &spec.filters {
        let extensions = filter
            .patterns
            .iter()
            .map(|pattern| pattern.trim_start_matches("*.").trim_start_matches('.'))
            .filter(|extension| !extension.is_empty() && *extension != "*")
            .collect::<Vec<_>>();
        if !extensions.is_empty() {
            dialog = dialog.add_filter(&filter.name, &extensions);
        }
    }
    Ok(dialog.save_file())
}

pub(crate) fn linux_direct_show_native_dialog(
    spec: &crate::NativeDialogSpec,
) -> ZsuiResult<crate::DialogResponse> {
    let availability = std::process::Command::new("zenity")
        .arg("--version")
        .output();
    match availability {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            return Err(ZsuiError::unsupported(
                "native_dialogs",
                format!(
                    "the Linux direct message-dialog provider `zenity` is unavailable (exit status {})",
                    output.status
                ),
            ));
        }
        Err(error) => {
            return Err(ZsuiError::unsupported(
                "native_dialogs",
                format!(
                    "the Linux direct message-dialog provider `zenity` is unavailable: {error}"
                ),
            ));
        }
    }

    let level = match spec.level {
        crate::DialogLevel::Info | crate::DialogLevel::Question => rfd::MessageLevel::Info,
        crate::DialogLevel::Warning => rfd::MessageLevel::Warning,
        crate::DialogLevel::Error => rfd::MessageLevel::Error,
    };
    let buttons = match spec.buttons {
        crate::DialogButtons::Ok => rfd::MessageButtons::Ok,
        crate::DialogButtons::OkCancel => rfd::MessageButtons::OkCancel,
        crate::DialogButtons::YesNo => rfd::MessageButtons::YesNo,
        crate::DialogButtons::YesNoCancel => rfd::MessageButtons::YesNoCancel,
    };
    let response = rfd::MessageDialog::new()
        .set_title(&spec.title)
        .set_description(&spec.message)
        .set_level(level)
        .set_buttons(buttons)
        .show();
    match response {
        rfd::MessageDialogResult::Ok => Ok(crate::DialogResponse::Ok),
        rfd::MessageDialogResult::Cancel => Ok(crate::DialogResponse::Cancel),
        rfd::MessageDialogResult::Yes => Ok(crate::DialogResponse::Yes),
        rfd::MessageDialogResult::No => Ok(crate::DialogResponse::No),
        rfd::MessageDialogResult::Custom(label) => Err(ZsuiError::host(
            "linux_direct_native_dialog",
            format!("unexpected custom message-dialog response `{label}`"),
        )),
    }
}

#[cfg(feature = "clipboard")]
pub(crate) fn linux_direct_read_clipboard() -> ZsuiResult<Option<crate::ClipboardData>> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| ZsuiError::host("linux_direct_read_clipboard", error.to_string()))?;
    match clipboard.get_text() {
        Ok(text) => Ok(Some(crate::ClipboardData::Text(text))),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(error) => Err(ZsuiError::host(
            "linux_direct_read_clipboard",
            error.to_string(),
        )),
    }
}

#[cfg(feature = "clipboard")]
pub(crate) fn linux_direct_write_clipboard(data: &crate::ClipboardData) -> ZsuiResult<()> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| ZsuiError::host("linux_direct_write_clipboard", error.to_string()))?;
    match data {
        crate::ClipboardData::Text(text) => clipboard
            .set_text(text.clone())
            .map_err(|error| ZsuiError::host("linux_direct_write_clipboard", error.to_string())),
        crate::ClipboardData::Empty => clipboard
            .clear()
            .map_err(|error| ZsuiError::host("linux_direct_write_clipboard", error.to_string())),
        crate::ClipboardData::ImageRgba { .. } => Err(ZsuiError::unsupported(
            "clipboard_image",
            "the lightweight Linux backend currently exposes text clipboard data",
        )),
        crate::ClipboardData::Files(_) => Err(ZsuiError::unsupported(
            "clipboard_files",
            "the lightweight Linux backend currently exposes text clipboard data",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accelerator_matching_uses_linux_primary_control_modifier() {
        let accelerator = ZsAccelerator::primary_character('s');
        let modifiers = ModifiersState::CONTROL;
        assert!(accelerator_matches(
            accelerator,
            &Key::Character("s".into()),
            modifiers,
        ));
    }

    #[test]
    fn initial_unfocused_event_is_not_a_focus_loss() {
        let mut focused = false;
        assert!(!focus_transition_lost(&mut focused, false));
        assert!(!focus_transition_lost(&mut focused, true));
        assert!(focus_transition_lost(&mut focused, false));
        assert!(!focus_transition_lost(&mut focused, false));
    }

    #[cfg(feature = "linux-direct")]
    #[test]
    fn direct_pango_no_wrap_does_not_constrain_width() {
        let context = linux_direct_text_context();
        let mut style = TextStyle::line("Sans", 14.0, Color::rgb(0, 0, 0));
        style.ellipsis = false;
        let layout = pango::Layout::new(&context);
        configure_linux_pango_layout(
            &layout,
            "a long single line",
            &style,
            Some(Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 20,
            }),
        );
        assert_eq!(layout.width(), -1);
        style.ellipsis = true;
        configure_linux_pango_layout(
            &layout,
            "a long single line",
            &style,
            Some(Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 20,
            }),
        );
        assert_eq!(layout.width(), pango::SCALE);
    }
}
