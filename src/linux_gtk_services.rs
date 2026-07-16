use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gtk::gio;
use gtk::glib::MainContext;
use gtk::prelude::*;
#[allow(deprecated)]
use gtk::{gdk, FileChooserAction, FileChooserNative, FileFilter, ResponseType};
use gtk4 as gtk;

use crate::native_clipboard::{native_clipboard_text_write, NativeClipboardTextWrite};
use crate::native_file_dialog::{
    native_file_dialog_initial_directory, native_save_dialog_suggested_name,
};
use crate::{
    ClipboardData, ClipboardService, DesktopEvent, FileDialogService, FileDialogSpec, MenuService,
    SaveFileDialogSpec, WindowId, WindowService, WindowSpec, ZsuiError, ZsuiResult,
};

struct LinuxGtkRuntimeState {
    _windows: LinuxGtkWindowService,
    _menu: Option<crate::linux_gtk_menu::LinuxGtkMenuService>,
}

pub(crate) fn run_linux_gtk_native_window_event_loop(
    specs: &[WindowSpec],
    draw_plans: &[Option<crate::NativeDrawPlan>],
    view_runtimes: &[crate::native::NativeViewInputRuntime],
    auto_close_after_ms: Option<u64>,
) -> ZsuiResult<usize> {
    if specs.is_empty() {
        return Ok(0);
    }
    let application = gtk::Application::builder()
        .application_id("io.github.qiu7824.zsui")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let specs = Rc::new(specs.to_vec());
    let draw_plans = Rc::new(draw_plans.to_vec());
    let view_runtimes = Rc::new(view_runtimes.to_vec());
    let state = Rc::new(RefCell::new(None::<LinuxGtkRuntimeState>));
    let startup_error = Rc::new(RefCell::new(None::<String>));
    let created_count = Rc::new(RefCell::new(0_usize));

    application.connect_activate({
        let specs = Rc::clone(&specs);
        let draw_plans = Rc::clone(&draw_plans);
        let view_runtimes = Rc::clone(&view_runtimes);
        let state = Rc::clone(&state);
        let startup_error = Rc::clone(&startup_error);
        let created_count = Rc::clone(&created_count);
        move |application| {
            if state.borrow().is_some() {
                return;
            }
            let mut windows = LinuxGtkWindowService::from_application(application.clone());
            let mut ids = Vec::with_capacity(specs.len());
            for (index, spec) in specs.iter().enumerate() {
                match windows.create_window(&spec.clone().visible(false)) {
                    Ok(id) => {
                        if let Some(plan) = draw_plans.get(index).and_then(Clone::clone) {
                            if let Err(error) = windows.set_window_view_content(
                                id,
                                plan,
                                view_runtimes.get(index).cloned().unwrap_or_default(),
                            ) {
                                *startup_error.borrow_mut() = Some(error.to_string());
                                application.quit();
                                return;
                            }
                        }
                        ids.push(id)
                    }
                    Err(error) => {
                        *startup_error.borrow_mut() = Some(error.to_string());
                        application.quit();
                        return;
                    }
                }
            }

            let mut menu =
                crate::linux_gtk_menu::LinuxGtkMenuService::from_application(application.clone());
            let menu = if let Some((window, menu_spec)) = ids
                .first()
                .copied()
                .zip(specs.first().and_then(|spec| spec.menu.as_ref()))
            {
                if let Err(error) = menu.set_window_menu(window, Some(menu_spec)) {
                    *startup_error.borrow_mut() = Some(error.to_string());
                    application.quit();
                    return;
                }
                if let Some(view_host) = windows.view_hosts.get(&window).cloned() {
                    menu.set_event_handler(move |event| {
                        if let DesktopEvent::MenuCommand { command, .. } = event {
                            view_host.dispatch_app_command(command);
                        }
                    });
                }
                Some(menu)
            } else {
                None
            };
            for (id, spec) in ids.iter().copied().zip(specs.iter()) {
                if spec.visible {
                    if let Err(error) = windows.set_window_visible(id, true) {
                        *startup_error.borrow_mut() = Some(error.to_string());
                        application.quit();
                        return;
                    }
                }
            }
            *created_count.borrow_mut() = ids.len();
            *state.borrow_mut() = Some(LinuxGtkRuntimeState {
                _windows: windows,
                _menu: menu,
            });

            if let Some(delay) = auto_close_after_ms {
                let application = application.clone();
                gtk::glib::timeout_add_local_once(Duration::from_millis(delay.max(1)), move || {
                    application.quit()
                });
            }
        }
    });

    application.run();
    state.borrow_mut().take();
    if let Some(error) = startup_error.borrow_mut().take() {
        return Err(ZsuiError::host("linux_gtk_event_loop", error));
    }
    let created_count = *created_count.borrow();
    Ok(created_count)
}

#[derive(Debug)]
pub struct LinuxGtkWindowService {
    application: gtk::Application,
    windows: HashMap<WindowId, gtk::ApplicationWindow>,
    view_hosts: HashMap<WindowId, crate::linux_gtk_renderer::LinuxGtkDrawViewHost>,
    close_handlers: HashMap<WindowId, gtk::glib::SignalHandlerId>,
    next_window_id: u64,
}

impl LinuxGtkWindowService {
    pub fn for_current_application() -> ZsuiResult<Self> {
        ensure_gtk_main_thread("gtk_window_service")?;
        let application = gio::Application::default()
            .and_then(|application| application.downcast::<gtk::Application>().ok())
            .ok_or_else(|| {
                ZsuiError::host(
                    "gtk_window_service",
                    "a running GTK Application is required before creating native windows",
                )
            })?;
        Ok(Self::from_application(application))
    }

    pub(crate) fn from_application(application: gtk::Application) -> Self {
        Self {
            application,
            windows: HashMap::new(),
            view_hosts: HashMap::new(),
            close_handlers: HashMap::new(),
            next_window_id: 1,
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn set_window_draw_plan(
        &mut self,
        window: WindowId,
        plan: crate::NativeDrawPlan,
    ) -> ZsuiResult<()> {
        self.set_window_view_content(
            window,
            plan,
            crate::native::NativeViewInputRuntime::default(),
        )
    }

    pub(crate) fn set_window_view_content(
        &mut self,
        window: WindowId,
        plan: crate::NativeDrawPlan,
        runtime: crate::native::NativeViewInputRuntime,
    ) -> ZsuiResult<()> {
        ensure_gtk_main_thread("gtk_set_window_draw_plan")?;
        let view_host = crate::linux_gtk_renderer::install_linux_gtk_draw_plan(
            self.window(window, "gtk_set_window_draw_plan")?,
            plan,
            runtime,
        );
        let native_window = self.window(window, "gtk_set_window_draw_plan")?.clone();
        if let Some(handler) = self.close_handlers.remove(&window) {
            native_window.disconnect(handler);
        }
        let close_host = view_host.clone();
        let handler = native_window.connect_close_request(move |_| {
            if close_host.dispatch_window_close_requested() {
                gtk::glib::Propagation::Proceed
            } else {
                gtk::glib::Propagation::Stop
            }
        });
        let map_host = view_host.clone();
        native_window.connect_map(move |_| map_host.set_window_suspended(false));
        let unmap_host = view_host.clone();
        native_window.connect_unmap(move |_| unmap_host.set_window_suspended(true));
        self.close_handlers.insert(window, handler);
        self.view_hosts.insert(window, view_host);
        Ok(())
    }

    fn window(&self, id: WindowId, operation: &'static str) -> ZsuiResult<&gtk::ApplicationWindow> {
        self.windows
            .get(&id)
            .ok_or_else(|| ZsuiError::host(operation, format!("unknown window id {}", id.0)))
    }

    fn allocate_window_id(&mut self) -> ZsuiResult<WindowId> {
        let id = WindowId(self.next_window_id);
        self.next_window_id = self.next_window_id.checked_add(1).ok_or_else(|| {
            ZsuiError::host(
                "gtk_create_window",
                "the native window id range is exhausted",
            )
        })?;
        Ok(id)
    }
}

impl Drop for LinuxGtkWindowService {
    fn drop(&mut self) {
        for (id, handler) in self.close_handlers.drain() {
            if let Some(window) = self.windows.get(&id) {
                window.disconnect(handler);
            }
        }
        self.view_hosts.clear();
        for (_, window) in self.windows.drain() {
            window.close();
        }
    }
}

impl WindowService for LinuxGtkWindowService {
    fn create_window(&mut self, spec: &WindowSpec) -> ZsuiResult<WindowId> {
        ensure_gtk_main_thread("gtk_create_window")?;
        if spec.always_on_top {
            return Err(ZsuiError::unsupported(
                "window_always_on_top",
                "GTK4 cannot guarantee always-on-top behavior across Wayland compositors",
            ));
        }
        if spec.transparent {
            return Err(ZsuiError::unsupported(
                "window_transparency",
                "the GTK4 transparent window surface is not connected",
            ));
        }
        let window = gtk::ApplicationWindow::builder()
            .application(&self.application)
            .title(&spec.title)
            .default_width(spec.width.min(i32::MAX as u32).max(1) as i32)
            .default_height(spec.height.min(i32::MAX as u32).max(1) as i32)
            .build();
        window.set_resizable(spec.resizable);
        window.set_decorated(spec.decorations);
        if spec.min_width.is_some() || spec.min_height.is_some() {
            window.set_size_request(
                spec.min_width.map(gtk_dimension).unwrap_or(-1),
                spec.min_height.map(gtk_dimension).unwrap_or(-1),
            );
        }
        if spec.visible {
            window.present();
        }
        let id = self.allocate_window_id()?;
        self.windows.insert(id, window);
        Ok(id)
    }

    fn set_window_title(&mut self, window: WindowId, title: &str) -> ZsuiResult<()> {
        ensure_gtk_main_thread("gtk_set_window_title")?;
        self.window(window, "gtk_set_window_title")?
            .set_title(Some(title));
        Ok(())
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()> {
        ensure_gtk_main_thread("gtk_set_window_visible")?;
        let window_id = window;
        let window = self.window(window_id, "gtk_set_window_visible")?;
        if visible {
            if let Some(host) = self.view_hosts.get(&window_id) {
                host.set_window_suspended(false);
            }
            window.present();
        } else {
            if let Some(host) = self.view_hosts.get(&window_id) {
                host.set_window_suspended(true);
            }
            window.set_visible(false);
        }
        Ok(())
    }

    fn request_window_redraw(&mut self, window: WindowId) -> ZsuiResult<()> {
        ensure_gtk_main_thread("gtk_request_window_redraw")?;
        self.window(window, "gtk_request_window_redraw")?
            .queue_draw();
        Ok(())
    }

    fn close_window(&mut self, window: WindowId) -> ZsuiResult<()> {
        ensure_gtk_main_thread("gtk_close_window")?;
        self.view_hosts.remove(&window);
        if let Some(handler) = self.close_handlers.remove(&window) {
            self.window(window, "gtk_close_window")?.disconnect(handler);
        }
        let window = self.windows.remove(&window).ok_or_else(|| {
            ZsuiError::host(
                "gtk_close_window",
                format!("unknown window id {}", window.0),
            )
        })?;
        window.close();
        Ok(())
    }
}

fn gtk_dimension(value: u32) -> i32 {
    value.min(i32::MAX as u32).max(1) as i32
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxGtkClipboardService;

impl ClipboardService for LinuxGtkClipboardService {
    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        let clipboard = gtk_system_clipboard("gtk_read_clipboard")?;
        MainContext::default()
            .block_on(clipboard.read_text_future())
            .map(|text| text.map(|text| ClipboardData::Text(text.to_string())))
            .map_err(|error| ZsuiError::host("gtk_read_clipboard", error.to_string()))
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        let write = native_clipboard_text_write(data)?;
        let clipboard = gtk_system_clipboard("gtk_write_clipboard")?;
        match write {
            NativeClipboardTextWrite::Clear => clipboard
                .set_content(None::<&gdk::ContentProvider>)
                .map_err(|error| ZsuiError::host("gtk_write_clipboard", error.to_string())),
            NativeClipboardTextWrite::Text(text) => {
                clipboard.set_text(text);
                Ok(())
            }
        }
    }
}

fn gtk_system_clipboard(operation: &'static str) -> ZsuiResult<gdk::Clipboard> {
    ensure_gtk_main_thread(operation)?;
    let display = gdk::Display::default().ok_or_else(|| {
        ZsuiError::host(operation, "GTK has no default display for clipboard access")
    })?;
    Ok(display.clipboard())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxGtkFileDialogService;

impl FileDialogService for LinuxGtkFileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        linux_gtk_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        linux_gtk_save_file_dialog(spec)
    }
}

#[allow(deprecated)]
pub fn linux_gtk_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    ensure_gtk_main_thread("gtk_open_file_dialog")?;
    let dialog = FileChooserNative::builder()
        .title(&spec.title)
        .action(FileChooserAction::Open)
        .accept_label("Open")
        .cancel_label("Cancel")
        .modal(true)
        .select_multiple(spec.allow_multiple)
        .build();
    gtk_bind_file_dialog_to_active_window(&dialog);
    add_gtk_file_filters(&dialog, &spec.filters);
    if let Some(directory) = native_file_dialog_initial_directory(spec.current_path.as_deref()) {
        let _ = dialog.set_current_folder(Some(&gio::File::for_path(directory)));
    }

    let response = MainContext::default().block_on(dialog.run_future());
    let result = if response == ResponseType::Accept {
        gtk_selected_local_paths(&dialog).map(Some)
    } else {
        Ok(None)
    };
    dialog.destroy();
    result
}

#[allow(deprecated)]
pub fn linux_gtk_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    ensure_gtk_main_thread("gtk_save_file_dialog")?;
    let dialog = FileChooserNative::builder()
        .title(&spec.title)
        .action(FileChooserAction::Save)
        .accept_label("Save")
        .cancel_label("Cancel")
        .modal(true)
        .select_multiple(false)
        .build();
    gtk_bind_file_dialog_to_active_window(&dialog);
    add_gtk_file_filters(&dialog, &spec.filters);
    if let Some(directory) = native_file_dialog_initial_directory(spec.current_path.as_deref()) {
        let _ = dialog.set_current_folder(Some(&gio::File::for_path(directory)));
    }
    if let Some(name) = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    ) {
        dialog.set_current_name(&name);
    }

    let response = MainContext::default().block_on(dialog.run_future());
    let result = if response == ResponseType::Accept {
        (|| {
            let file = dialog.file().ok_or_else(|| {
                ZsuiError::host(
                    "gtk_save_file_dialog",
                    "GTK file chooser returned no selected file",
                )
            })?;
            let path = file.path().ok_or_else(|| {
                ZsuiError::host(
                    "gtk_save_file_dialog",
                    "GTK file chooser returned a non-local file",
                )
            })?;
            Ok(Some(path))
        })()
    } else {
        Ok(None)
    };
    dialog.destroy();
    result
}

pub(crate) fn ensure_gtk_main_thread(operation: &'static str) -> ZsuiResult<()> {
    if gtk::is_initialized() && !gtk::is_initialized_main_thread() {
        return Err(ZsuiError::host(
            operation,
            "GTK desktop services must run on the GTK main thread",
        ));
    }
    if !gtk::is_initialized_main_thread() {
        gtk::init().map_err(|error| ZsuiError::host(operation, error.to_string()))?;
    }
    Ok(())
}

#[allow(deprecated)]
fn add_gtk_file_filters(dialog: &FileChooserNative, filters: &[crate::FileDialogFilter]) {
    for filter_spec in filters {
        if filter_spec.patterns.is_empty() {
            continue;
        }
        let filter = FileFilter::new();
        filter.set_name(Some(&filter_spec.name));
        for pattern in &filter_spec.patterns {
            filter.add_pattern(pattern);
        }
        dialog.add_filter(&filter);
    }
}

#[allow(deprecated)]
fn gtk_bind_file_dialog_to_active_window(dialog: &FileChooserNative) {
    let owner = gio::Application::default()
        .and_then(|application| application.downcast::<gtk::Application>().ok())
        .and_then(|application| application.active_window());
    dialog.set_transient_for(owner.as_ref());
}

#[allow(deprecated)]
fn gtk_selected_local_paths(dialog: &FileChooserNative) -> ZsuiResult<Vec<PathBuf>> {
    let files = dialog.files();
    let mut paths = Vec::with_capacity(files.n_items() as usize);
    for index in 0..files.n_items() {
        let file = files
            .item(index)
            .and_then(|item| item.downcast::<gio::File>().ok())
            .ok_or_else(|| {
                ZsuiError::host(
                    "gtk_open_file_dialog",
                    "GTK file chooser returned an invalid file object",
                )
            })?;
        let path = file.path().ok_or_else(|| {
            ZsuiError::host(
                "gtk_open_file_dialog",
                "GTK file chooser returned a non-local file",
            )
        })?;
        paths.push(path);
    }
    if paths.is_empty() {
        return Err(ZsuiError::host(
            "gtk_open_file_dialog",
            "GTK file chooser accepted without returning a selected file",
        ));
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk_file_dialog_service_implements_safe_public_contract() {
        fn assert_service<T: FileDialogService>() {}
        assert_service::<LinuxGtkFileDialogService>();
    }

    #[test]
    fn gtk_clipboard_service_implements_safe_public_contract() {
        fn assert_service<T: ClipboardService>() {}
        assert_service::<LinuxGtkClipboardService>();
    }

    #[test]
    fn gtk_window_service_implements_safe_public_contract() {
        fn assert_service<T: WindowService>() {}
        assert_service::<LinuxGtkWindowService>();
    }
}
