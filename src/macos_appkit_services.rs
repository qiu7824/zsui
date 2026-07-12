use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSFloatingWindowLevel, NSModalResponseOK, NSOpenPanel, NSPasteboard, NSPasteboardTypeString,
    NSSavePanel, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    NSArray, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
    NSTimer, NSURL,
};

use crate::native_clipboard::{native_clipboard_text_write, NativeClipboardTextWrite};
use crate::native_file_dialog::{
    native_file_dialog_extensions, native_file_dialog_initial_directory,
    native_save_dialog_suggested_name,
};
use crate::{
    ClipboardData, ClipboardService, FileDialogService, FileDialogSpec, MenuService,
    SaveFileDialogSpec, WindowId, WindowService, WindowSpec, ZsuiError, ZsuiResult,
};

struct ZsuiAppKitRuntimeDelegateIvars {
    open_windows: Cell<usize>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ZsuiAppKitRuntimeDelegateIvars]
    struct ZsuiAppKitRuntimeDelegate;

    unsafe impl NSObjectProtocol for ZsuiAppKitRuntimeDelegate {}

    unsafe impl NSApplicationDelegate for ZsuiAppKitRuntimeDelegate {}

    unsafe impl NSWindowDelegate for ZsuiAppKitRuntimeDelegate {
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            let remaining = self.ivars().open_windows.get().saturating_sub(1);
            self.ivars().open_windows.set(remaining);
            if remaining == 0 {
                NSApplication::sharedApplication(self.mtm()).stop(None);
            }
        }
    }
);

impl ZsuiAppKitRuntimeDelegate {
    fn new(mtm: MainThreadMarker, open_windows: usize) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ZsuiAppKitRuntimeDelegateIvars {
            open_windows: Cell::new(open_windows),
        });
        unsafe { msg_send![super(this), init] }
    }
}

pub(crate) fn run_macos_appkit_native_window_event_loop(
    specs: &[WindowSpec],
    auto_close_after_ms: Option<u64>,
) -> ZsuiResult<usize> {
    if specs.is_empty() {
        return Ok(0);
    }
    let mtm = appkit_main_thread_marker("macos_native_event_loop")?;
    let mut window_service = MacosAppKitWindowService::new()?;
    let mut ids = Vec::with_capacity(specs.len());
    for spec in specs {
        ids.push(window_service.create_window(spec)?);
    }

    let mut menu_service = crate::macos_appkit_menu::MacosAppKitMenuService::new()?;
    if let Some((window, menu)) = ids
        .first()
        .copied()
        .zip(specs.first().and_then(|spec| spec.menu.as_ref()))
    {
        menu_service.set_window_menu(window, Some(menu))?;
    }

    let delegate = ZsuiAppKitRuntimeDelegate::new(mtm, ids.len());
    let application = &window_service._application;
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

    let timer = auto_close_after_ms.map(|delay| unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            delay.max(1) as f64 / 1_000.0,
            application.as_ref(),
            objc2::sel!(stop:),
            None,
            false,
        )
    });
    application.run();
    if let Some(timer) = timer {
        timer.invalidate();
    }
    for window in window_service.windows.values() {
        window.setDelegate(None);
    }
    application.setDelegate(None);
    Ok(ids.len())
}

#[derive(Debug)]
pub struct MacosAppKitWindowService {
    _application: Retained<NSApplication>,
    windows: HashMap<WindowId, Retained<NSWindow>>,
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
            next_window_id: 1,
        })
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
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
        let window = self.window(window, "macos_set_window_visible")?;
        if visible {
            window.makeKeyAndOrderFront(None);
        } else {
            window.orderOut(None);
        }
        Ok(())
    }

    fn request_window_redraw(&mut self, window: WindowId) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_request_window_redraw")?;
        self.window(window, "macos_request_window_redraw")?
            .displayIfNeeded();
        Ok(())
    }

    fn close_window(&mut self, window: WindowId) -> ZsuiResult<()> {
        appkit_main_thread_marker("macos_close_window")?;
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
    appkit_set_initial_directory(&panel, spec.current_path.as_deref().map(Path::new));

    if panel.runModal() != NSModalResponseOK {
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

    if panel.runModal() != NSModalResponseOK {
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
