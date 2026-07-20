use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSFloatingWindowLevel, NSModalResponseOK, NSOpenPanel, NSPasteboard, NSPasteboardTypeString,
    NSSavePanel, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    NSArray, NSDate, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSRunLoop,
    NSSize, NSString, NSURL,
};

use crate::native_clipboard::{native_clipboard_text_write, NativeClipboardTextWrite};
use crate::native_file_dialog::{
    native_file_dialog_extensions, native_file_dialog_initial_directory,
    native_save_dialog_suggested_name,
};
use crate::{
    ClipboardData, ClipboardService, DesktopEvent, FileDialogService, FileDialogSpec, MenuService,
    SaveFileDialogSpec, WindowId, WindowService, WindowSpec, ZsuiError, ZsuiResult,
};

struct ZsuiAppKitRuntimeDelegateIvars {
    open_windows: Cell<usize>,
    close_handlers: HashMap<usize, crate::macos_appkit_renderer::MacosAppKitDrawViewHost>,
    capture_handler: Option<crate::macos_appkit_renderer::MacosAppKitDrawViewHost>,
    capture_path: Option<PathBuf>,
    capture_result: RefCell<Option<Result<crate::NativeViewCaptureEvidence, String>>>,
    proof_inputs: Vec<crate::NativeViewSmokeInput>,
    proof_input_reports: RefCell<Vec<crate::native::NativeViewInputDispatchReport>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ZsuiAppKitRuntimeDelegateIvars]
    struct ZsuiAppKitRuntimeDelegate;

    unsafe impl NSObjectProtocol for ZsuiAppKitRuntimeDelegate {}

    unsafe impl NSApplicationDelegate for ZsuiAppKitRuntimeDelegate {}

    unsafe impl NSWindowDelegate for ZsuiAppKitRuntimeDelegate {
        #[unsafe(method(windowShouldClose:))]
        fn window_should_close(&self, sender: &NSWindow) -> bool {
            self.ivars()
                .close_handlers
                .get(&(sender as *const NSWindow as usize))
                .is_none_or(|handler| handler.dispatch_window_close_requested())
        }

        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            let remaining = self.ivars().open_windows.get().saturating_sub(1);
            self.ivars().open_windows.set(remaining);
            if remaining == 0 {
                NSApplication::sharedApplication(self.mtm()).stop(None);
            }
        }

        #[unsafe(method(windowDidMiniaturize:))]
        fn window_did_miniaturize(&self, notification: &NSNotification) {
            self.set_window_suspended(notification, true);
        }

        #[unsafe(method(windowDidDeminiaturize:))]
        fn window_did_deminiaturize(&self, notification: &NSNotification) {
            self.set_window_suspended(notification, false);
        }
    }
);

impl ZsuiAppKitRuntimeDelegate {
    fn run_proof_inputs_and_capture(&self) {
        if let Some(handler) = self.ivars().capture_handler.as_ref() {
            *self.ivars().proof_input_reports.borrow_mut() =
                handler.dispatch_proof_inputs(&self.ivars().proof_inputs);
        }
        if let Some(path) = self.ivars().capture_path.as_deref() {
            let result = self
                .ivars()
                .capture_handler
                .as_ref()
                .ok_or_else(|| "the AppKit proof window has no ZSUI NSView".to_string())
                .and_then(|handler| handler.capture_png(path));
            *self.ivars().capture_result.borrow_mut() = Some(result);
        }
    }

    fn set_window_suspended(&self, notification: &NSNotification, suspended: bool) {
        let window = notification
            .object()
            .map(|object| Retained::as_ptr(&object).cast::<NSWindow>() as usize);
        if let Some(handler) = window.and_then(|window| self.ivars().close_handlers.get(&window)) {
            handler.set_window_suspended(suspended);
        }
    }

    fn new(
        mtm: MainThreadMarker,
        open_windows: usize,
        close_handlers: HashMap<usize, crate::macos_appkit_renderer::MacosAppKitDrawViewHost>,
        capture_handler: Option<crate::macos_appkit_renderer::MacosAppKitDrawViewHost>,
        capture_path: Option<PathBuf>,
        proof_inputs: Vec<crate::NativeViewSmokeInput>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ZsuiAppKitRuntimeDelegateIvars {
            open_windows: Cell::new(open_windows),
            close_handlers,
            capture_handler,
            capture_path,
            capture_result: RefCell::new(None),
            proof_inputs,
            proof_input_reports: RefCell::new(Vec::new()),
        });
        unsafe { msg_send![super(this), init] }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MacosAppKitNativeWindowRunReport {
    pub created_window_count: usize,
    pub native_view_capture: Option<Result<crate::NativeViewCaptureEvidence, String>>,
    pub proof_input_reports: Vec<crate::native::NativeViewInputDispatchReport>,
    pub menu_command_routed: bool,
    pub accessibility_backend: Option<&'static str>,
    pub accessibility_node_count: usize,
    pub accessibility_evidence_event: Option<String>,
}

pub(crate) fn run_macos_appkit_native_window_event_loop(
    specs: &[WindowSpec],
    draw_plans: &[Option<crate::NativeDrawPlan>],
    view_runtimes: &[crate::native::NativeViewInputRuntime],
    auto_close_after_ms: Option<u64>,
    capture_path: Option<&Path>,
    proof_inputs: &[crate::NativeViewSmokeInput],
) -> ZsuiResult<MacosAppKitNativeWindowRunReport> {
    if specs.is_empty() {
        return Ok(MacosAppKitNativeWindowRunReport {
            created_window_count: 0,
            native_view_capture: None,
            proof_input_reports: Vec::new(),
            menu_command_routed: false,
            accessibility_backend: None,
            accessibility_node_count: 0,
            accessibility_evidence_event: None,
        });
    }
    let mtm = appkit_main_thread_marker("macos_native_event_loop")?;
    let mut window_service = MacosAppKitWindowService::new()?;
    let mut ids = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        let id = window_service.create_window(&spec.clone().visible(false))?;
        if let Some(plan) = draw_plans.get(index).and_then(Clone::clone) {
            window_service.set_window_view_content(
                id,
                plan,
                view_runtimes.get(index).cloned().unwrap_or_default(),
            )?;
        }
        ids.push(id);
    }

    let mut menu_service = crate::macos_appkit_menu::MacosAppKitMenuService::new()?;
    if let Some((window, menu)) = ids
        .first()
        .copied()
        .zip(specs.first().and_then(|spec| spec.menu.as_ref()))
    {
        menu_service.set_window_menu(window, Some(menu))?;
        if let Some(view_host) = window_service.view_hosts.get(&window).cloned() {
            menu_service.set_event_handler(move |event| {
                if let DesktopEvent::MenuCommand { command, .. } = event {
                    view_host.dispatch_app_command(command);
                }
            });
        }
    }

    let close_handlers = ids
        .iter()
        .filter_map(|id| {
            window_service
                .windows
                .get(id)
                .zip(window_service.view_hosts.get(id))
        })
        .map(|(window, host)| (Retained::as_ptr(window) as usize, host.clone()))
        .collect();
    let capture_handler = ids
        .first()
        .and_then(|id| window_service.view_hosts.get(id))
        .cloned();
    let delegate = ZsuiAppKitRuntimeDelegate::new(
        mtm,
        ids.len(),
        close_handlers,
        capture_handler,
        capture_path.map(Path::to_path_buf),
        proof_inputs.to_vec(),
    );
    let application = window_service._application.clone();
    let application_delegate: &ProtocolObject<dyn NSApplicationDelegate> =
        ProtocolObject::from_ref(&*delegate);
    let window_delegate: &ProtocolObject<dyn NSWindowDelegate> =
        ProtocolObject::from_ref(&*delegate);
    application.setDelegate(Some(application_delegate));
    for window in window_service.windows.values() {
        window.setDelegate(Some(window_delegate));
    }
    #[allow(deprecated)]
    application.activateIgnoringOtherApps(true);
    for (id, spec) in ids.iter().copied().zip(specs) {
        if spec.visible {
            window_service.set_window_visible(id, true)?;
        }
    }
    let menu_command_routed =
        auto_close_after_ms.is_some() && menu_service.invoke_first_enabled_command_for_proof();

    if let Some(delay) = auto_close_after_ms {
        application.finishLaunching();
        let deadline = NSDate::dateWithTimeIntervalSinceNow(delay.max(1) as f64 / 1_000.0);
        NSRunLoop::mainRunLoop().runUntilDate(&deadline);
        delegate.run_proof_inputs_and_capture();
    } else {
        application.run();
    }
    for window in window_service.windows.values() {
        window.setDelegate(None);
    }
    application.setDelegate(None);
    let native_view_capture = delegate.ivars().capture_result.borrow_mut().take();
    let proof_input_reports =
        std::mem::take(&mut *delegate.ivars().proof_input_reports.borrow_mut());
    let accessibility_evidence = delegate
        .ivars()
        .capture_handler
        .as_ref()
        .and_then(|handler| handler.accessibility_evidence());
    let accessibility_node_count = accessibility_evidence
        .map(|evidence| evidence.node_count)
        .unwrap_or(0);
    Ok(MacosAppKitNativeWindowRunReport {
        created_window_count: ids.len(),
        native_view_capture,
        proof_input_reports,
        menu_command_routed,
        accessibility_backend: accessibility_evidence
            .is_some_and(|evidence| evidence.verified())
            .then_some("appkit_nsaccessibility"),
        accessibility_node_count,
        accessibility_evidence_event: accessibility_evidence.map(|evidence| evidence.event()),
    })
}

#[derive(Debug)]
pub struct MacosAppKitWindowService {
    _application: Retained<NSApplication>,
    windows: HashMap<WindowId, Retained<NSWindow>>,
    view_hosts: HashMap<WindowId, crate::macos_appkit_renderer::MacosAppKitDrawViewHost>,
    next_window_id: u64,
}

impl MacosAppKitWindowService {
    pub fn new() -> ZsuiResult<Self> {
        let mtm = appkit_main_thread_marker("NSApplication")?;
        let application = NSApplication::sharedApplication(mtm);
        application.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        Ok(Self {
            _application: application,
            windows: HashMap::new(),
            view_hosts: HashMap::new(),
            next_window_id: 1,
        })
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
        appkit_main_thread_marker("macos_set_window_draw_plan")?;
        let view_host = crate::macos_appkit_renderer::install_macos_appkit_draw_plan(
            self.window(window, "macos_set_window_draw_plan")?,
            plan,
            runtime,
        );
        self.view_hosts.insert(window, view_host);
        Ok(())
    }

    fn window(&self, id: WindowId, operation: &'static str) -> ZsuiResult<&NSWindow> {
        self.windows
            .get(&id)
            .map(AsRef::as_ref)
            .ok_or_else(|| ZsuiError::host(operation, format!("unknown window id {}", id.0)))
    }

    fn allocate_window_id(&mut self) -> ZsuiResult<WindowId> {
        let id = WindowId(self.next_window_id);
        self.next_window_id = self.next_window_id.checked_add(1).ok_or_else(|| {
            ZsuiError::host(
                "macos_create_window",
                "the native window id range is exhausted",
            )
        })?;
        Ok(id)
    }
}

impl Drop for MacosAppKitWindowService {
    fn drop(&mut self) {
        self.view_hosts.clear();
        for (_, window) in self.windows.drain() {
            window.close();
        }
    }
}

impl WindowService for MacosAppKitWindowService {
    fn create_window(&mut self, spec: &WindowSpec) -> ZsuiResult<WindowId> {
        let mtm = appkit_main_thread_marker("macos_create_window")?;
        if spec.transparent {
            return Err(ZsuiError::unsupported(
                "window_transparency",
                "the AppKit transparent window surface is not connected",
            ));
        }
        let mut style = if spec.decorations {
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable
        } else {
            NSWindowStyleMask::Borderless
        };
        if spec.resizable {
            style |= NSWindowStyleMask::Resizable;
        }
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(spec.width.max(1) as f64, spec.height.max(1) as f64),
                ),
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe { window.setReleasedWhenClosed(false) };
        window.setTitle(&NSString::from_str(&spec.title));
        if let (Some(width), Some(height)) = (spec.min_width, spec.min_height) {
            window.setMinSize(NSSize::new(width.max(1) as f64, height.max(1) as f64));
        }
        if spec.always_on_top {
            window.setLevel(NSFloatingWindowLevel);
        }
        window.center();
        if spec.visible {
            window.makeKeyAndOrderFront(None);
        }
        let id = self.allocate_window_id()?;
        self.windows.insert(id, window);
        Ok(id)
    }

    fn set_window_title(&mut self, window: WindowId, title: &str) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_set_window_title")?;
        self.window(window, "macos_set_window_title")?
            .setTitle(&NSString::from_str(title));
        Ok(())
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_set_window_visible")?;
        let window_id = window;
        let window = self.window(window_id, "macos_set_window_visible")?;
        if visible {
            if let Some(host) = self.view_hosts.get(&window_id) {
                host.set_window_suspended(false);
            }
            window.makeKeyAndOrderFront(None);
        } else {
            if let Some(host) = self.view_hosts.get(&window_id) {
                host.set_window_suspended(true);
            }
            window.orderOut(None);
        }
        Ok(())
    }

    fn request_window_redraw(&mut self, window: WindowId) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_request_window_redraw")?;
        let window = self.window(window, "macos_request_window_redraw")?;
        if let Some(view) = window.contentView() {
            view.setNeedsDisplay(true);
        }
        window.displayIfNeeded();
        Ok(())
    }

    fn close_window(&mut self, window: WindowId) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_close_window")?;
        self.view_hosts.remove(&window);
        let window = self.windows.remove(&window).ok_or_else(|| {
            ZsuiError::host(
                "macos_close_window",
                format!("unknown window id {}", window.0),
            )
        })?;
        window.close();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MacosAppKitClipboardService;

impl ClipboardService for MacosAppKitClipboardService {
    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        let _mtm = appkit_main_thread_marker("NSPasteboard")?;
        Ok(NSPasteboard::generalPasteboard()
            .stringForType(unsafe { NSPasteboardTypeString })
            .map(|text| ClipboardData::Text(text.to_string())))
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        let write = native_clipboard_text_write(data)?;
        let _mtm = appkit_main_thread_marker("NSPasteboard")?;
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        match write {
            NativeClipboardTextWrite::Clear => Ok(()),
            NativeClipboardTextWrite::Text(text) => {
                if pasteboard
                    .setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString })
                {
                    Ok(())
                } else {
                    Err(ZsuiError::host(
                        "macos_write_clipboard",
                        "NSPasteboard rejected the UTF-8 text value",
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MacosAppKitFileDialogService;

impl FileDialogService for MacosAppKitFileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        macos_appkit_open_file_dialog(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        macos_appkit_save_file_dialog(spec)
    }
}

pub fn macos_appkit_open_file_dialog(spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
    let mtm = appkit_main_thread_marker("NSOpenPanel")?;
    let owner = appkit_active_file_dialog_owner(mtm);
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(spec.allow_multiple);
    panel.setTitle(Some(&NSString::from_str(&spec.title)));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    if let Some(allowed) = appkit_allowed_file_types(&spec.filters) {
        #[allow(deprecated)]
        panel.setAllowedFileTypes(Some(&allowed));
    }
    appkit_set_initial_directory(&panel, spec.current_path.as_deref());

    if appkit_run_file_panel(&panel, owner.as_deref()) != NSModalResponseOK {
        return Ok(None);
    }

    let urls = panel.URLs();
    let mut paths = Vec::with_capacity(urls.len());
    for index in 0..urls.len() {
        let url = unsafe { urls.objectAtIndex_unchecked(index) };
        let path = url.to_file_path().ok_or_else(|| {
            ZsuiError::host(
                "macos_open_file_dialog",
                "NSOpenPanel returned a non-file URL",
            )
        })?;
        paths.push(path);
    }
    if paths.is_empty() {
        return Err(ZsuiError::host(
            "macos_open_file_dialog",
            "NSOpenPanel accepted without returning a selected file",
        ));
    }
    Ok(Some(paths))
}

pub fn macos_appkit_save_file_dialog(spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
    let mtm = appkit_main_thread_marker("NSSavePanel")?;
    let owner = appkit_active_file_dialog_owner(mtm);
    let panel = NSSavePanel::savePanel(mtm);
    panel.setCanCreateDirectories(true);
    panel.setTitle(Some(&NSString::from_str(&spec.title)));
    panel.setPrompt(Some(&NSString::from_str("Save")));
    if let Some(name) = native_save_dialog_suggested_name(
        spec.suggested_name.as_deref(),
        spec.current_path.as_deref(),
    ) {
        panel.setNameFieldStringValue(&NSString::from_str(&name));
    }
    if let Some(allowed) = appkit_allowed_file_types(&spec.filters) {
        #[allow(deprecated)]
        panel.setAllowedFileTypes(Some(&allowed));
    }
    appkit_set_initial_directory(&panel, spec.current_path.as_deref());

    if appkit_run_file_panel(&panel, owner.as_deref()) != NSModalResponseOK {
        return Ok(None);
    }
    panel
        .URL()
        .map(|url| {
            url.to_file_path().ok_or_else(|| {
                ZsuiError::host(
                    "macos_save_file_dialog",
                    "NSSavePanel returned a non-file URL",
                )
            })
        })
        .transpose()
}

fn appkit_active_file_dialog_owner(mtm: MainThreadMarker) -> Option<Retained<NSWindow>> {
    let application = NSApplication::sharedApplication(mtm);
    application.keyWindow().or_else(|| application.mainWindow())
}

fn appkit_run_file_panel(
    panel: &NSSavePanel,
    owner: Option<&NSWindow>,
) -> objc2_app_kit::NSModalResponse {
    let Some(owner) = owner else {
        return panel.runModal();
    };

    let response = Rc::new(Cell::new(None));
    let completed_response = Rc::clone(&response);
    let completion = RcBlock::new(move |value: objc2_app_kit::NSModalResponse| {
        completed_response.set(Some(value));
    });
    panel.beginSheetModalForWindow_completionHandler(owner, &completion);

    let run_loop = NSRunLoop::currentRunLoop();
    while response.get().is_none() {
        run_loop.runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(0.01));
    }
    panel.orderOut(None);
    response
        .get()
        .expect("AppKit sheet completion set a modal response")
}

fn appkit_main_thread_marker(operation: &'static str) -> ZsuiResult<MainThreadMarker> {
    MainThreadMarker::new().ok_or_else(|| {
        ZsuiError::host(
            operation,
            "AppKit desktop services must run on the macOS main thread",
        )
    })
}

fn appkit_allowed_file_types(
    filters: &[crate::FileDialogFilter],
) -> Option<Retained<NSArray<NSString>>> {
    let values = native_file_dialog_extensions(filters)
        .into_iter()
        .map(|extension| NSString::from_str(&extension))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let references = values.iter().map(AsRef::as_ref).collect::<Vec<_>>();
    Some(NSArray::from_slice(&references))
}

fn appkit_set_initial_directory(panel: &NSSavePanel, current_path: Option<&Path>) {
    let Some(directory) = native_file_dialog_initial_directory(current_path) else {
        return;
    };
    if let Some(url) = NSURL::from_directory_path(directory) {
        panel.setDirectoryURL(Some(&url));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appkit_file_dialog_service_implements_safe_public_contract() {
        fn assert_service<T: FileDialogService>() {}
        assert_service::<MacosAppKitFileDialogService>();
    }

    #[test]
    fn appkit_clipboard_service_implements_safe_public_contract() {
        fn assert_service<T: ClipboardService>() {}
        assert_service::<MacosAppKitClipboardService>();
    }

    #[test]
    fn appkit_window_service_implements_safe_public_contract() {
        fn assert_service<T: WindowService>() {}
        assert_service::<MacosAppKitWindowService>();
    }
}
