use std::cell::Cell;
use std::rc::Rc;

const ZSUI_WIN32_SYNC_NATIVE_HOSTS: u32 = WM_APP + 0x0553;
const ZSUI_WIN32_NATIVE_EVENTS: u32 = WM_APP + 0x0554;
const ZSUI_WINDOW_STATE_PROPERTY: *const u16 = windows_sys::core::w!("ZSUI.NativeWindowState.3");

struct WindowsWindowState {
    params: WindowsWindowCreateParams,
    services: Rc<WindowsWindowServices>,
}

struct WindowsWindowServices {
    hwnd: HWND,
    configured: Cell<bool>,
    closed: Cell<bool>,
    integration: RefCell<crate::WindowIntegration>,
    error: RefCell<Option<String>>,
    #[cfg(all(feature = "windows-shell", feature = "native-smoke"))]
    menu_message_count: Cell<usize>,
    palette: Cell<Option<crate::PlatformPalette>>,
    control_brushes: RefCell<Option<NativeControlBrushes>>,
    #[cfg(feature = "dark-mode")]
    dark_mode: dark_mode_compatibility::Lease,
    #[cfg(feature = "windows-shell")]
    shell: RefCell<Option<shell::ShellWindow>>,
    #[cfg(feature = "windows-shell")]
    menu_hook: RefCell<Option<Rc<dyn menu_bridge::NativeMenuHook>>>,
    #[cfg(feature = "desktop-layer")]
    desktop: RefCell<Option<Rc<WindowsDesktopLayer>>>,
    #[cfg(feature = "windows-direct2d")]
    direct2d: RefCell<Option<direct2d::Direct2dPainter>>,
    #[cfg(feature = "window-chrome")]
    caption_metrics: Cell<Option<crate::CaptionMetrics>>,
    #[cfg(feature = "window-chrome")]
    caption_composited: Cell<bool>,
    #[cfg(feature = "native-window-events")]
    events: RefCell<std::collections::VecDeque<crate::NativeWindowEvent>>,
    #[cfg(feature = "native-host")]
    native_hosts: RefCell<Option<interop::NativeHostRuntime>>,
    #[cfg(feature = "hotkey")]
    hotkeys: RefCell<Option<WindowsGlobalHotkeys>>,
}

impl std::fmt::Debug for WindowsWindowServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsWindowServices")
            .field("configured", &self.configured.get())
            .field("closed", &self.closed.get())
            .finish()
    }
}

impl WindowsWindowServices {
    #[cfg(feature = "desktop-layer")]
    fn desktop_layer(&self) -> Option<Rc<WindowsDesktopLayer>> {
        self.desktop.borrow().clone()
    }
    fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            configured: Cell::new(false),
            closed: Cell::new(false),
            integration: RefCell::new(crate::WindowIntegration::default()),
            error: RefCell::new(None),
            #[cfg(all(feature = "windows-shell", feature = "native-smoke"))]
            menu_message_count: Cell::new(0),
            palette: Cell::new(None),
            control_brushes: RefCell::new(None),
            #[cfg(feature = "dark-mode")]
            dark_mode: dark_mode_compatibility::Lease::new(),
            #[cfg(feature = "windows-shell")]
            shell: RefCell::new(None),
            #[cfg(feature = "windows-shell")]
            menu_hook: RefCell::new(None),
            #[cfg(feature = "desktop-layer")]
            desktop: RefCell::new(None),
            #[cfg(feature = "windows-direct2d")]
            direct2d: RefCell::new(None),
            #[cfg(feature = "window-chrome")]
            caption_metrics: Cell::new(None),
            #[cfg(feature = "window-chrome")]
            caption_composited: Cell::new(false),
            #[cfg(feature = "native-window-events")]
            events: RefCell::new(std::collections::VecDeque::new()),
            #[cfg(feature = "native-host")]
            native_hosts: RefCell::new(None),
            #[cfg(feature = "hotkey")]
            hotkeys: RefCell::new(None),
        }
    }

    fn record_error(&self, error: impl std::fmt::Display) {
        let message = error.to_string();
        *self.error.borrow_mut() = Some(message.clone());
        eprintln!("ZSUI native integration: {message}");
        #[cfg(feature = "native-window-events")]
        self.queue_event(crate::NativeWindowEvent::Error {
            operation: "native_integration".into(),
            message,
        });
    }

    #[cfg(feature = "native-window-events")]
    fn queue_event(&self, event: crate::NativeWindowEvent) {
        if self.closed.get() {
            return;
        }
        let mut events = self.events.borrow_mut();
        let coalescible = matches!(
            event,
            crate::NativeWindowEvent::ThemeChanged(_) | crate::NativeWindowEvent::CaptionChanged(_)
        );
        if let Some(previous) = events.iter_mut().rev().find(|previous| {
            coalescible && std::mem::discriminant(*previous) == std::mem::discriminant(&event)
        }) {
            *previous = event;
        } else if events.len() < 64 {
            events.push_back(event);
        } else {
            *self.error.borrow_mut() = Some("native window event queue is full".into());
        }
        drop(events);
        unsafe {
            PostMessageW(self.hwnd, ZSUI_WIN32_NATIVE_EVENTS, 0, 0);
        }
    }

    fn dispatch_events(&self) {
        #[cfg(feature = "native-window-events")]
        {
            let events = std::mem::take(&mut *self.events.borrow_mut());
            for event in events {
                if self.closed.get() {
                    break;
                }
                dispatch_windows_win32_window_view_input(self.hwnd, |route| {
                    let report = route.shared_runtime.dispatch_native_window_event(event);
                    route.adapt_shared_report(report, WindowsSharedInputKind::Surface)
                });
            }
        }
    }

    fn configure(&self, spec: &WindowSpec) -> ZsuiResult<()> {
        spec.integration
            .validate(spec.decorations, spec.always_on_top)?;
        *self.integration.borrow_mut() = spec.integration.clone();
        #[cfg(feature = "windows-shell")]
        {
            *self.shell.borrow_mut() = Some(shell::ShellWindow::new(self.hwnd)?);
        }
        #[cfg(not(feature = "window-chrome"))]
        if spec.integration.title_bar.is_some() {
            return Err(ZsuiError::unsupported(
                "integrated_title_bar",
                "enable window-chrome",
            ));
        }
        #[cfg(not(feature = "window-surface"))]
        if spec.integration.surface != crate::WindowSurface::Opaque {
            return Err(ZsuiError::unsupported(
                "window_surface",
                "enable window-surface",
            ));
        }
        #[cfg(not(feature = "desktop-layer"))]
        if spec.integration.layer != crate::WindowLayer::Normal {
            return Err(ZsuiError::unsupported(
                "desktop_layer",
                "enable desktop-layer",
            ));
        }
        #[cfg(not(feature = "windows-direct2d"))]
        if spec.integration.renderer != crate::WindowRenderer::Gdi {
            return Err(ZsuiError::unsupported(
                "window_renderer",
                "enable windows-direct2d",
            ));
        }
        #[cfg(feature = "window-surface")]
        {
            if spec.transparent && self.integration.borrow().surface == crate::WindowSurface::Opaque
            {
                if spec.decorations {
                    return Err(ZsuiError::invalid_spec(
                        "window.surface",
                        "per-pixel transparency requires an undecorated window",
                    ));
                }
                self.integration.borrow_mut().surface = crate::WindowSurface::PerPixelAlpha {
                    hit_test: crate::AlphaHitTest::TransparentPixels,
                };
            }
            if let crate::WindowSurface::PerPixelAlpha { hit_test } =
                self.integration.borrow().surface
            {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    WS_EX_LAYERED, WS_EX_TRANSPARENT,
                };
                let style = unsafe { GetWindowLongW(self.hwnd, GWL_EXSTYLE) } as u32
                    | WS_EX_LAYERED
                    | if hit_test == crate::AlphaHitTest::PassThrough {
                        WS_EX_TRANSPARENT
                    } else {
                        0
                    };
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongW(
                        self.hwnd,
                        GWL_EXSTYLE,
                        style as i32,
                    );
                }
            }
        }
        #[cfg(feature = "native-host")]
        {
            let mut hooks = spec.windows_hooks.clone().unwrap_or_default();
            for (id, provider) in &spec.native_hosts.providers {
                hooks = match provider {
                    crate::NativeChildSpec::Edit { text, multiline } => {
                        hooks.edit(*id, text.clone(), *multiline)?
                    }
                    #[cfg(feature = "windows-shell")]
                    crate::NativeChildSpec::Explorer { folder } => {
                        hooks.explorer(*id, folder.clone())?
                    }
                    #[cfg(feature = "windows-shell")]
                    crate::NativeChildSpec::NamespaceTree { folder } => {
                        hooks.namespace_tree(*id, folder.clone())?
                    }
                    #[cfg(not(feature = "windows-shell"))]
                    _ => {
                        return Err(ZsuiError::unsupported(
                            "native_host.shell",
                            "enable windows-shell for Shell child controls",
                        ))
                    }
                };
            }
            *self.native_hosts.borrow_mut() = Some(interop::NativeHostRuntime::new(hooks));
        }
        #[cfg(feature = "hotkey")]
        {
            *self.hotkeys.borrow_mut() = Some(WindowsGlobalHotkeys::register(
                self.hwnd,
                &spec.global_hotkeys,
            )?);
        }
        #[cfg(not(feature = "hotkey"))]
        if spec.global_hotkeys.iter().any(|hotkey| hotkey.enabled) {
            return Err(ZsuiError::unsupported(
                "global_hotkeys",
                "enable the hotkey feature",
            ));
        }
        self.configured.set(true);
        #[cfg(feature = "desktop-layer")]
        if let crate::WindowLayer::Desktop {
            follow_show_desktop,
        } = spec.integration.layer
        {
            let layer = WindowsDesktopLayer::attach(self.hwnd, spec.visible, follow_show_desktop)?;
            *self.desktop.borrow_mut() = Some(Rc::new(layer));
        }
        self.sync_native_hosts()?;
        self.sync_appearance();
        #[cfg(feature = "window-chrome")]
        update_integrated_caption(self.hwnd);
        self.present_layered()?;
        unsafe {
            InvalidateRect(self.hwnd, null(), 0);
        }
        Ok(())
    }

    fn sync_appearance(&self) {
        if !self.configured.get() || self.closed.get() {
            return;
        }
        let palette = windows_platform_palette(
            window_draw_plan(self.hwnd)
                .map(|plan| plan.theme_mode)
                .unwrap_or_default(),
        );
        if self.palette.replace(Some(palette)) == Some(palette) {
            return;
        }
        *self.control_brushes.borrow_mut() = Some(NativeControlBrushes::new(palette));
        #[cfg(feature = "dark-mode")]
        self.dark_mode
            .apply(self.hwnd, palette.mode == crate::ZsuiThemeMode::Dark);
        #[cfg(feature = "native-host")]
        if let Ok(mut hosts) = self.native_hosts.try_borrow_mut() {
            if let Some(hosts) = hosts.as_mut() {
                if let Err(error) = hosts.theme_changed(palette.mode) {
                    self.record_error(error);
                }
            }
        }
        #[cfg(feature = "native-window-events")]
        self.queue_event(crate::NativeWindowEvent::ThemeChanged(palette));
        #[cfg(feature = "window-chrome")]
        update_integrated_caption(self.hwnd);
    }

    #[cfg(feature = "windows-direct2d")]
    fn uses_direct2d(&self) -> bool {
        self.configured.get()
            && (self.integration.borrow().surface != crate::WindowSurface::Opaque
                || self.integration.borrow().renderer != crate::WindowRenderer::Gdi)
    }

    fn present_layered(&self) -> ZsuiResult<()> {
        #[cfg(feature = "window-surface")]
        if self.configured.get()
            && !self.closed.get()
            && self.integration.borrow().surface != crate::WindowSurface::Opaque
            && unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsIconic(self.hwnd) } == 0
        {
            let mut bounds: RECT = unsafe { zeroed() };
            if unsafe { GetClientRect(self.hwnd, &mut bounds) } != 0
                && bounds.right > 0
                && bounds.bottom > 0
            {
                return self.paint_direct2d(
                    null_mut(),
                    bounds.right as u32,
                    bounds.bottom as u32,
                    false,
                );
            }
        }
        Ok(())
    }

    #[cfg(feature = "windows-direct2d")]
    fn paint_direct2d(
        &self,
        target: windows_sys::Win32::Graphics::Gdi::HDC,
        width: u32,
        height: u32,
        capture: bool,
    ) -> ZsuiResult<()> {
        let configuration = self.integration.borrow().clone();
        let Ok(mut painter) = self.direct2d.try_borrow_mut() else {
            return Ok(());
        };
        if painter.is_none() {
            *painter = Some(direct2d::Direct2dPainter::new(
                width,
                height,
                configuration.renderer == crate::WindowRenderer::Direct2dHardware,
            )?);
        }
        let surface = unsafe { window_paint_surface(self.hwnd) };
        let transparent = configuration.surface != crate::WindowSurface::Opaque;
        let renderer = painter.as_mut().expect("painter initialized");
        renderer.render(
            width,
            height,
            surface.draw_plan.as_deref(),
            surface.palette,
            surface.high_contrast,
            surface.dpi,
            surface.resources,
            transparent,
        )?;
        unsafe {
            if capture {
                renderer.copy_to_dc(target)
            } else {
                renderer.present(self.hwnd, target, transparent)
            }
        }
    }

    fn sync_native_hosts(&self) -> ZsuiResult<()> {
        if !self.configured.get() || self.closed.get() {
            return Ok(());
        }
        #[cfg(feature = "native-host")]
        {
            let Some(plan) = window_draw_plan(self.hwnd) else {
                return Ok(());
            };
            let Ok(mut hosts) = self.native_hosts.try_borrow_mut() else {
                unsafe {
                    PostMessageW(self.hwnd, ZSUI_WIN32_SYNC_NATIVE_HOSTS, 0, 0);
                }
                return Ok(());
            };
            if let Some(hosts) = hosts.as_mut() {
                if self.integration.borrow().surface != crate::WindowSurface::Opaque
                    && !plan.native_hosts.is_empty()
                {
                    return Err(ZsuiError::invalid_spec(
                        "native_host.surface",
                        "native child slots require an opaque parent surface",
                    ));
                }
                hosts.reconcile(self.hwnd, &plan.native_hosts, true, &plan.native_occlusions)?;
            }
        }
        Ok(())
    }

    fn pre_translate(&self, _message: &MSG) -> bool {
        if self.closed.get() {
            return false;
        }
        #[cfg(feature = "native-host")]
        {
            let message = _message;
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetAncestor, GA_ROOT};
            if unsafe { GetAncestor(message.hwnd, GA_ROOT) } != self.hwnd {
                return false;
            }
            let filtered = {
                let mut hosts = self.native_hosts.try_borrow_mut().ok();
                hosts
                    .as_mut()
                    .and_then(|hosts| hosts.as_mut())
                    .is_some_and(|hosts| hosts.pre_translate(message))
            };
            if self.closed.get() {
                self.shutdown();
                return true;
            }
            if filtered {
                return true;
            }
            let has_children = self
                .native_hosts
                .try_borrow()
                .ok()
                .is_some_and(|hosts| hosts.as_ref().is_some_and(|hosts| hosts.has_children()));
            let focused = unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
            let child = self.native_hosts.try_borrow().ok().and_then(|hosts| {
                hosts
                    .as_ref()
                    .and_then(|hosts| hosts.child_at_focus(focused))
            });
            // A child or application filter can consume internal traversal first.
            // Otherwise native surfaces participate in the existing View chain.
            if has_children
                && message.message == WM_KEYDOWN
                && (message.wParam == VK_TAB as usize
                    || message.wParam
                        == windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F6 as usize)
            {
                if let Some(child) = child {
                    if let Ok(mut hosts) = self.native_hosts.try_borrow_mut() {
                        if let Some(hosts) = hosts.as_mut() {
                            hosts.blur(child.id);
                        }
                    }
                    dispatch_windows_win32_window_view_input(self.hwnd, |route| {
                        let report = route
                            .shared_runtime
                            .dispatch_accessibility_focus(child.widget);
                        route.adapt_shared_report(report, WindowsSharedInputKind::Surface)
                    });
                    unsafe {
                        SetFocus(self.hwnd);
                    }
                }
                let shift = unsafe { GetKeyState(VK_SHIFT as i32) } < 0;
                let handled = dispatch_windows_win32_window_view_input(self.hwnd, |route| {
                    route.dispatch_key_down_with_shift(VK_TAB as u32, shift)
                })
                .is_some_and(|report| report.handled);
                self.sync_native_focus();
                if handled {
                    return true;
                }
            }
            return false;
        }
        #[cfg(not(feature = "native-host"))]
        false
    }

    fn shutdown(&self) {
        self.closed.set(true);
        #[cfg(feature = "native-host")]
        if let Ok(mut hosts) = self.native_hosts.try_borrow_mut() {
            hosts.take();
        }
        #[cfg(feature = "hotkey")]
        if let Ok(mut hotkeys) = self.hotkeys.try_borrow_mut() {
            hotkeys.take();
        }
        #[cfg(feature = "windows-shell")]
        {
            if let Ok(mut shell) = self.shell.try_borrow_mut() {
                shell.take();
            }
            self.menu_hook.borrow_mut().take();
        }
        #[cfg(feature = "desktop-layer")]
        if let Ok(mut desktop) = self.desktop.try_borrow_mut() {
            desktop.take();
        }
        #[cfg(feature = "windows-direct2d")]
        if let Ok(mut renderer) = self.direct2d.try_borrow_mut() {
            renderer.take();
        }
        self.control_brushes.borrow_mut().take();
        #[cfg(feature = "native-window-events")]
        self.events.borrow_mut().clear();
    }

    fn sync_native_focus(&self) {
        if self.closed.get() {
            return;
        }
        #[cfg(feature = "native-host")]
        {
            if let Some(crate::ViewHitTarget {
                kind: crate::ViewHitTargetKind::NativeHost(id),
                ..
            }) = windows_win32_window_focused_target(self.hwnd)
            {
                if let Ok(mut hosts) = self.native_hosts.try_borrow_mut() {
                    if let Some(hosts) = hosts.as_mut() {
                        hosts.focus(id);
                    }
                }
            } else {
                let focus = unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
                let is_child = self
                    .native_hosts
                    .try_borrow()
                    .ok()
                    .and_then(|hosts| hosts.as_ref().and_then(|hosts| hosts.child_at_focus(focus)))
                    .is_some();
                if is_child {
                    unsafe {
                        SetFocus(self.hwnd);
                    }
                }
            }
        }
    }
}

fn window_services(hwnd: HWND) -> Option<Rc<WindowsWindowServices>> {
    if unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, null_mut())
    } != unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() }
    {
        return None;
    }
    let state = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(hwnd, ZSUI_WINDOW_STATE_PROPERTY)
    } as *const WindowsWindowState;
    if state.is_null() {
        None
    } else {
        Some(unsafe { (*state).services.clone() })
    }
}

fn sync_windows_native_hosts(hwnd: HWND) {
    if let Some(services) = window_services(hwnd) {
        if let Err(error) = services.sync_native_hosts() {
            services.record_error(error);
        }
        if let Err(error) = services.present_layered() {
            services.record_error(error);
        }
    } else if !unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(hwnd, ZSUI_WINDOW_STATE_PROPERTY)
    }
    .is_null()
    {
        unsafe {
            PostMessageW(hwnd, ZSUI_WIN32_SYNC_NATIVE_HOSTS, 0, 0);
        }
    }
}

#[cfg(feature = "hotkey")]
struct WindowsGlobalHotkeys {
    hwnd: HWND,
    commands: Vec<(i32, Command)>,
}

#[cfg(feature = "hotkey")]
impl WindowsGlobalHotkeys {
    fn register(hwnd: HWND, specs: &[crate::HotkeySpec]) -> ZsuiResult<Self> {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
        };
        let mut owned = Self {
            hwnd,
            commands: Vec::new(),
        };
        let mut seen = std::collections::BTreeSet::new();
        for spec in specs.iter().filter(|spec| spec.enabled) {
            let accelerator: crate::ZsAccelerator = spec.accelerator.parse()?;
            let mut modifiers = MOD_NOREPEAT;
            if accelerator.uses_primary() {
                modifiers |= MOD_CONTROL;
            }
            if accelerator.uses_super() {
                modifiers |= MOD_WIN;
            }
            if accelerator.uses_alt() {
                modifiers |= MOD_ALT;
            }
            if accelerator.uses_shift() {
                modifiers |= MOD_SHIFT;
            }
            let key = u32::from(windows_accelerator_virtual_key(&accelerator)?);
            if !seen.insert((modifiers, key)) {
                return Err(ZsuiError::invalid_spec(
                    "global_hotkeys",
                    "duplicate global shortcut",
                ));
            }
            let id = i32::try_from(owned.commands.len() + 1).unwrap_or(i32::MAX);
            if id > 0xbfff {
                return Err(ZsuiError::invalid_spec(
                    "global_hotkeys",
                    "too many global shortcuts",
                ));
            }
            if unsafe { RegisterHotKey(hwnd, id, modifiers, key) } == 0 {
                return Err(ZsuiError::host(
                    "RegisterHotKey",
                    format!(
                        "shortcut '{}' could not be registered (Windows error {})",
                        spec.accelerator,
                        unsafe { GetLastError() }
                    ),
                ));
            }
            owned.commands.push((id, spec.command.clone()));
        }
        Ok(owned)
    }
}

#[cfg(feature = "hotkey")]
impl Drop for WindowsGlobalHotkeys {
    fn drop(&mut self) {
        for (id, _) in &self.commands {
            unsafe {
                windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey(self.hwnd, *id);
            }
        }
    }
}
