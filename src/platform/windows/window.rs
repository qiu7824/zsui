pub fn set_windows_win32_window_draw_plan(hwnd: HWND, plan: NativeDrawPlan) -> bool {
    if hwnd.is_null() {
        return false;
    }
    apply_windows_win32_window_theme(hwnd, plan.theme_mode);
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    let hwnd = hwnd as isize;
    let plan = Arc::new(plan);
    if let Some(record) = plans.iter_mut().find(|record| record.hwnd == hwnd) {
        record.plan = plan;
    } else {
        plans.push(WindowsWindowDrawPlanRecord {
            hwnd,
            plan,
            renderer_resources: WindowsGdiResourceCache::default(),
        });
    }
    true
}

pub fn clear_windows_win32_window_draw_plan(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_draw_plans() {
    let mut plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans.clear();
}

fn window_draw_plans() -> &'static Mutex<Vec<WindowsWindowDrawPlanRecord>> {
    WINDOW_DRAW_PLANS.get_or_init(|| Mutex::new(Vec::new()))
}

fn window_draw_plan(hwnd: HWND) -> Option<Arc<NativeDrawPlan>> {
    window_paint_state(hwnd).map(|(plan, _)| plan)
}

fn window_paint_state(
    hwnd: HWND,
) -> Option<(Arc<NativeDrawPlan>, WindowsGdiResourceCache)> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let plans = window_draw_plans()
        .lock()
        .expect("window draw plan registry should not be poisoned");
    plans
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| (record.plan.clone(), record.renderer_resources.clone()))
}

pub struct WindowsWin32MainWindowHost {
    class_names: WindowsWin32ClassNames,
    window_proc: WNDPROC,
    operation_log: Vec<NativeMainWindowHostOperation>,
}

impl WindowsWin32MainWindowHost {
    pub fn new() -> Self {
        Self::with_window_proc(Some(zsui_win32_default_window_proc))
    }

    pub fn with_window_proc(window_proc: WNDPROC) -> Self {
        Self::with_class_names(WindowsWin32ClassNames::default(), window_proc)
    }

    pub fn with_class_names(class_names: WindowsWin32ClassNames, window_proc: WNDPROC) -> Self {
        Self {
            class_names,
            window_proc,
            operation_log: Vec::new(),
        }
    }

    pub const fn class_names(&self) -> WindowsWin32ClassNames {
        self.class_names
    }

    pub fn operation_log(&self) -> &[NativeMainWindowHostOperation] {
        &self.operation_log
    }

    fn record(&mut self, operation: NativeMainWindowHostOperation) {
        self.operation_log.push(operation);
    }

    unsafe fn module_handle() -> HINSTANCE {
        GetModuleHandleW(null()) as HINSTANCE
    }

    unsafe fn arrow_cursor() -> HCURSOR {
        LoadCursorW(null_mut(), IDC_ARROW)
    }

    unsafe fn register_window_class(
        &self,
        role: WindowsWindowRole,
        module: HINSTANCE,
        cursor: HCURSOR,
    ) -> bool {
        if self.window_proc.is_none() {
            return false;
        }
        let class_name = wide_null(role.class_name(self.class_names));
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: self.window_proc,
            hInstance: module,
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS
    }

    unsafe fn create_window(
        &self,
        role: WindowsWindowRole,
        title: &[u16],
        width: i32,
        height: i32,
        module: HINSTANCE,
        options: &NativeWindowOptions,
    ) -> HWND {
        let style_plan = windows_win32_main_window_style_plan(role, options);
        let (outer_width, outer_height) = windows_win32_outer_size_for_client(
            width,
            height,
            style_plan.style,
            style_plan.ex_style,
        );
        let class_name = wide_null(role.class_name(self.class_names));
        let create_params = WindowsWindowCreateParams::new(role, options.min_size);
        CreateWindowExW(
            style_plan.ex_style,
            class_name.as_ptr(),
            title.as_ptr(),
            style_plan.style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            outer_width,
            outer_height,
            null_mut(),
            null_mut(),
            module,
            &create_params as *const WindowsWindowCreateParams as _,
        )
    }
}

unsafe fn windows_win32_outer_size_for_client(
    width: i32,
    height: i32,
    style: u32,
    ex_style: u32,
) -> (i32, i32) {
    let width = width.max(1);
    let height = height.max(1);
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    let dpi = GetDpiForSystem().max(96);
    if AdjustWindowRectExForDpi(&mut rect, style, 0, ex_style, dpi) == 0 {
        (width, height)
    } else {
        (
            (rect.right - rect.left).max(width),
            (rect.bottom - rect.top).max(height),
        )
    }
}

impl Default for WindowsWin32MainWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeMainWindowHost for WindowsWin32MainWindowHost {
    type Handle = HWND;
    type AppIcon = isize;

    fn create_main_windows(
        &mut self,
        request: NativeMainWindowRequest,
    ) -> NativeMainWindowPresentation<Self::Handle> {
        self.record(NativeMainWindowHostOperation::CreateMainWindows);
        unsafe {
            let module = Self::module_handle();
            if module.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            let cursor = Self::arrow_cursor();
            if cursor.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            for role in [WindowsWindowRole::Main, WindowsWindowRole::Quick] {
                if !self.register_window_class(role, module, cursor) {
                    return NativeMainWindowPresentation::Failed;
                }
            }

            let title = wide_null(&request.title);
            let width = request.size.width.max(1);
            let height = request.size.height.max(1);
            let main = self.create_window(
                WindowsWindowRole::Main,
                &title,
                width,
                height,
                module,
                &request.options,
            );
            if main.is_null() {
                return NativeMainWindowPresentation::Failed;
            }
            ACTIVE_MAIN_WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

            let quick_options = NativeWindowOptions::tool_window();
            let quick = self.create_window(
                WindowsWindowRole::Quick,
                &title,
                width,
                height,
                module,
                &quick_options,
            );
            if quick.is_null() {
                DestroyWindow(main);
                return NativeMainWindowPresentation::Failed;
            }

            ShowWindow(
                main,
                if request.main_visible {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
            if request.main_visible {
                UpdateWindow(main);
            }
            ShowWindow(quick, SW_HIDE);
            NativeMainWindowPresentation::Created(NativeMainWindowHandles { main, quick })
        }
    }

    fn apply_main_window_appearance(&mut self, _handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ApplyMainWindowAppearance);
    }

    fn set_main_window_app_icon(
        &mut self,
        handle: Self::Handle,
        icon: NativeAppIconResource<Self::AppIcon>,
    ) {
        self.record(NativeMainWindowHostOperation::SetMainWindowAppIcon);
        unsafe {
            SendMessageW(handle, WM_SETICON, ICON_SMALL as WPARAM, icon.small);
            SendMessageW(handle, WM_SETICON, ICON_BIG as WPARAM, icon.big);
        }
    }

    fn hide_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::HideMainWindow);
        unsafe {
            ShowWindow(handle, SW_HIDE);
        }
    }

    fn present_main_window(&mut self, handle: Self::Handle, mode: NativeMainWindowPresentMode) {
        self.record(NativeMainWindowHostOperation::PresentMainWindow);
        unsafe {
            match mode {
                NativeMainWindowPresentMode::ActivateAndFocus => {
                    ShowWindow(handle, SW_SHOW);
                    SetWindowPos(
                        handle,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                    );
                    SetForegroundWindow(handle);
                    SetFocus(handle);
                }
                NativeMainWindowPresentMode::NoActivate => {
                    ShowWindow(handle, SW_SHOWNOACTIVATE);
                    SetWindowPos(
                        handle,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                }
            }
        }
    }

    fn set_main_window_bounds(&mut self, handle: Self::Handle, bounds: UiRect) {
        self.record(NativeMainWindowHostOperation::SetMainWindowBounds);
        unsafe {
            SetWindowPos(
                handle,
                null_mut(),
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn activate_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ActivateMainWindow);
        unsafe {
            ShowWindow(handle, SW_SHOW);
            SetWindowPos(
                handle,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            SetForegroundWindow(handle);
            SetFocus(handle);
        }
    }

    fn foreground_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ForegroundMainWindow);
        unsafe {
            SetForegroundWindow(handle);
        }
    }

    fn restore_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::RestoreMainWindow);
        unsafe {
            ShowWindow(handle, SW_SHOW);
        }
    }

    fn close_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::CloseMainWindow);
        unsafe {
            PostMessageW(handle, WM_CLOSE, 0, 0);
        }
    }

    fn set_main_window_activation_policy(&mut self, handle: Self::Handle, allow_activation: bool) {
        self.record(NativeMainWindowHostOperation::SetMainWindowActivationPolicy);
        if handle.is_null() {
            return;
        }
        unsafe {
            let ex_style = GetWindowLongW(handle, GWL_EXSTYLE) as u32;
            let desired = if allow_activation {
                ex_style & !WS_EX_NOACTIVATE
            } else {
                ex_style | WS_EX_NOACTIVATE
            };
            if desired != ex_style {
                SetWindowLongW(handle, GWL_EXSTYLE, desired as i32);
                SetWindowPos(
                    handle,
                    null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
    }

    fn request_main_window_close(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::RequestMainWindowClose);
        unsafe {
            SendMessageW(handle, WM_CLOSE, 0, 0);
        }
    }

    fn destroy_main_window(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::DestroyMainWindow);
        unsafe {
            DestroyWindow(handle);
        }
    }

    fn capture_main_pointer(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::CaptureMainPointer);
        unsafe {
            SetCapture(handle);
        }
    }

    fn release_main_pointer(&mut self, _handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::ReleaseMainPointer);
        unsafe {
            ReleaseCapture();
        }
    }

    fn begin_main_window_drag(&mut self, handle: Self::Handle) {
        self.record(NativeMainWindowHostOperation::BeginMainWindowDrag);
        unsafe {
            ReleaseCapture();
            SendMessageW(
                handle,
                WM_SYSCOMMAND,
                (SC_MOVE as usize | HTCAPTION as usize) as WPARAM,
                0,
            );
        }
    }

    fn track_main_pointer_leave(&mut self, handle: Self::Handle) -> bool {
        self.record(NativeMainWindowHostOperation::TrackMainPointerLeave);
        if handle.is_null() {
            return false;
        }
        let mut event = TRACKMOUSEEVENT {
            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE | TME_HOVER,
            hwndTrack: handle,
            dwHoverTime: HOVER_DEFAULT,
        };
        unsafe { TrackMouseEvent(&mut event) != 0 }
    }

    fn request_main_window_area_repaint(
        &mut self,
        handle: Self::Handle,
        area: Option<UiRect>,
        erase: bool,
    ) -> bool {
        self.record(NativeMainWindowHostOperation::RequestMainWindowAreaRepaint);
        let rect = area.map(RECT::from);
        unsafe {
            InvalidateRect(
                handle,
                rect.as_ref().map_or(null(), |rect| rect as *const RECT),
                erase as i32,
            ) != 0
        }
    }

    fn main_window_layout_dpi(&mut self, handle: Self::Handle) -> u32 {
        self.record(NativeMainWindowHostOperation::MainWindowLayoutDpi);
        if handle.is_null() {
            96
        } else {
            unsafe { GetDpiForWindow(handle).max(1) }
        }
    }

    fn main_window_client_bounds(&mut self, handle: Self::Handle) -> Option<UiRect> {
        self.record(NativeMainWindowHostOperation::MainWindowClientBounds);
        if handle.is_null() {
            return None;
        }
        let mut rect: RECT = unsafe { zeroed() };
        let ok = unsafe { GetClientRect(handle, &mut rect) != 0 };
        ok.then(|| UiRect::from(rect))
    }

    fn main_window_bounds(&mut self, handle: Self::Handle) -> Option<UiRect> {
        self.record(NativeMainWindowHostOperation::MainWindowBounds);
        if handle.is_null() {
            return None;
        }
        let mut rect: RECT = unsafe { zeroed() };
        let ok = unsafe { GetWindowRect(handle, &mut rect) != 0 };
        ok.then(|| UiRect::from(rect))
    }
}

unsafe fn paint_no_flicker_background(hwnd: HWND) -> LRESULT {
    let mut ps: PAINTSTRUCT = zeroed();
    let target = BeginPaint(hwnd, &mut ps);
    if target.is_null() {
        return 0;
    }

    let mut rect: RECT = zeroed();
    if GetClientRect(hwnd, &mut rect) != 0 {
        if let Some(buffered) = WindowsBufferedPaint::begin(target, &rect) {
            paint_window_client_rect_to_dc(hwnd, buffered.hdc(), rect);
        } else {
            paint_window_client_rect_to_dc(hwnd, target, rect);
        }
    }

    EndPaint(hwnd, &ps);
    0
}

unsafe fn paint_window_client_to_dc(
    hwnd: HWND,
    target: windows_sys::Win32::Graphics::Gdi::HDC,
) -> LRESULT {
    if target.is_null() {
        return 0;
    }
    let mut rect: RECT = zeroed();
    if GetClientRect(hwnd, &mut rect) != 0 {
        paint_window_client_rect_to_dc(hwnd, target, rect);
        GdiFlush();
    }
    0
}

unsafe fn paint_window_client_rect_to_dc(
    hwnd: HWND,
    target: windows_sys::Win32::Graphics::Gdi::HDC,
    rect: RECT,
) {
    let paint_state = window_paint_state(hwnd);
    let draw_plan = paint_state.as_ref().map(|(plan, _)| plan.as_ref());
    let resources = paint_state
        .as_ref()
        .map(|(_, resources)| resources.clone())
        .unwrap_or_default();
    let palette = windows_palette_for_draw_plan(draw_plan);
    let high_contrast = resolved_windows_theme_mode(
        draw_plan
            .map(|plan| plan.theme_mode)
            .unwrap_or(crate::ZsuiThemeMode::System),
    ) == crate::ZsuiThemeMode::HighContrast;
    let dpi = crate::Dpi::new(GetDpiForWindow(hwnd).max(96) as f32);
    paint_win32_surface(
        target,
        rect,
        palette,
        high_contrast,
        dpi,
        draw_plan,
        resources,
    );
}

unsafe fn paint_win32_surface(
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
    rect: RECT,
    palette: WindowsGdiPalette,
    high_contrast: bool,
    dpi: crate::Dpi,
    draw_plan: Option<&NativeDrawPlan>,
    resources: WindowsGdiResourceCache,
) {
    let mut renderer = WindowsGdiRenderer::with_dpi_and_resources(dc, dpi, resources.clone());
    renderer.fill_rect(rect_from_win(rect), palette.surface);
    drop(renderer);
    if let Some(plan) = draw_plan {
        let mut sink = WindowsGdiDrawSink::with_palette_contrast_dpi_and_resources(
            dc,
            palette,
            high_contrast,
            dpi,
            resources,
        );
        sink.draw_native_plan(plan);
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path_null(path: &Path) -> Vec<u16> {
    path.to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}
