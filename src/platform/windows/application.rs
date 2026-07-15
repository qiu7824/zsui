#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsWindowDrawPlanRecord {
    hwnd: isize,
    plan: NativeDrawPlan,
}

#[derive(Debug, Clone)]
struct WindowsWindowViewInputRouteRecord {
    hwnd: isize,
    route: WindowsWin32ViewInputRoute,
    report: WindowsWin32ViewInputDispatchReport,
}

#[derive(Debug, Clone)]
struct WindowsCompletedViewInputReportRecord {
    hwnd: isize,
    report: WindowsWin32ViewInputDispatchReport,
}

#[derive(Debug, Clone)]
struct WindowsWindowShellInputRouteRecord {
    hwnd: isize,
    route: WindowsWin32ShellInputRoute,
}

#[derive(Debug, Clone)]
struct WindowsWindowMenuCommandTableRecord {
    hwnd: isize,
    command_table: WindowsWin32StatusMenuCommandTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(isize)]
pub enum WindowsWindowRole {
    Main = 1,
    Quick = 2,
}

impl WindowsWindowRole {
    pub const fn from_create_param(value: isize) -> Self {
        match value {
            value if value == Self::Quick as isize => Self::Quick,
            _ => Self::Main,
        }
    }

    pub const fn class_name(self, class_names: WindowsWin32ClassNames) -> &'static str {
        match self {
            Self::Main => class_names.main,
            Self::Quick => class_names.quick,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWin32ClassNames {
    pub main: &'static str,
    pub quick: &'static str,
}

impl WindowsWin32ClassNames {
    pub const fn new(main: &'static str, quick: &'static str) -> Self {
        Self { main, quick }
    }
}

impl Default for WindowsWin32ClassNames {
    fn default() -> Self {
        Self::new(DEFAULT_MAIN_CLASS_NAME, DEFAULT_QUICK_CLASS_NAME)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWindowCreateParams {
    pub role: WindowsWindowRole,
    pub min_size: Option<Size>,
}

impl WindowsWindowCreateParams {
    pub const fn new(role: WindowsWindowRole, min_size: Option<Size>) -> Self {
        Self { role, min_size }
    }

    pub fn from_create_param(value: isize) -> Self {
        if value == WindowsWindowRole::Quick as isize || value == WindowsWindowRole::Main as isize {
            return Self::new(WindowsWindowRole::from_create_param(value), None);
        }
        let params = value as *const Self;
        if params.is_null() {
            Self::new(WindowsWindowRole::Main, None)
        } else {
            unsafe { *params }
        }
    }

    pub unsafe fn from_create_struct(create_struct: *const CREATESTRUCTW) -> Self {
        if create_struct.is_null() {
            return Self::new(WindowsWindowRole::Main, None);
        }
        Self::from_create_param((*create_struct).lpCreateParams as isize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWin32WindowStylePlan {
    pub ex_style: u32,
    pub style: u32,
}

pub fn windows_win32_main_window_style_plan(
    role: WindowsWindowRole,
    options: &NativeWindowOptions,
) -> WindowsWin32WindowStylePlan {
    let mut ex_style = 0;
    if !options.decorations {
        ex_style |= WS_EX_TOOLWINDOW;
    }
    if options.always_on_top {
        ex_style |= WS_EX_TOPMOST;
    }
    if matches!(role, WindowsWindowRole::Quick) {
        ex_style |= WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE;
    }

    let style = if options.decorations {
        let mut style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        if options.resizable {
            style |= WS_MAXIMIZEBOX | WS_THICKFRAME;
        }
        style
    } else {
        WS_POPUP | WS_CLIPCHILDREN
    };

    WindowsWin32WindowStylePlan { ex_style, style }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsWin32MessageLoopResult {
    Quit(i32),
    Failed,
}

pub struct WindowsWin32MessageLoop;

impl WindowsWin32MessageLoop {
    pub fn run() -> WindowsWin32MessageLoopResult {
        Self::run_with_windows(&[])
    }

    pub fn run_with_windows(
        windows: &[WindowsWin32OwnedMainWindowHandles],
    ) -> WindowsWin32MessageLoopResult {
        let mut msg: MSG = unsafe { zeroed() };
        loop {
            let code = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if code == -1 {
                return WindowsWin32MessageLoopResult::Failed;
            }
            if code == 0 {
                return WindowsWin32MessageLoopResult::Quit(msg.wParam as i32);
            }
            if windows
                .iter()
                .any(|window| window.translate_accelerator(&msg))
            {
                continue;
            }
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

pub fn create_windows_for_specs(
    specs: &[WindowSpec],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    create_windows_for_specs_with_draw_plans(specs, &[])
}

pub fn create_windows_for_specs_with_draw_plans(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    create_windows_for_specs_with_draw_plans_and_input_routes(specs, draw_plans, &[])
}

pub fn create_windows_for_specs_with_draw_plans_and_input_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    create_windows_for_specs_with_routes(specs, draw_plans, input_routes, &[])
}

pub fn create_windows_for_specs_with_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
) -> ZsuiResult<Vec<NativeMainWindowHandles<HWND>>> {
    let owned =
        create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, shell_routes)?;
    Ok(owned
        .into_iter()
        .map(WindowsWin32OwnedMainWindowHandles::into_handles)
        .collect())
}

pub fn create_owned_windows_for_specs(
    specs: &[WindowSpec],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    create_owned_windows_for_specs_with_draw_plans(specs, &[])
}

pub fn create_owned_windows_for_specs_with_draw_plans(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    create_owned_windows_for_specs_with_draw_plans_and_input_routes(specs, draw_plans, &[])
}

pub fn create_owned_windows_for_specs_with_draw_plans_and_input_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, &[])
}

pub fn create_owned_windows_for_specs_with_routes(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
) -> ZsuiResult<Vec<WindowsWin32OwnedMainWindowHandles>> {
    ACTIVE_MAIN_WINDOW_COUNT.store(0, Ordering::SeqCst);
    clear_windows_win32_window_draw_plans();
    clear_windows_win32_window_view_input_routes();
    clear_windows_win32_window_shell_input_routes();
    clear_windows_win32_window_menu_command_tables();
    let capabilities = HostCapabilities::windows_native_window_host();
    let mut host = WindowsWin32MainWindowHost::new();
    let mut handles = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let mut request = NativeMainWindowRequest::from_zsui_window_for_host(spec, &capabilities);
        let main_visible = request.main_visible;
        request.main_visible = false;
        let icon_path = request.icon_path.clone();
        match host.create_main_windows(request) {
            NativeMainWindowPresentation::Created(created) => {
                if let Some(Some(plan)) = draw_plans.get(index) {
                    set_windows_win32_window_draw_plan(created.main, plan.clone());
                    host.request_main_window_area_repaint(created.main, None, false);
                    unsafe {
                        UpdateWindow(created.main);
                    }
                }
                if let Some(Some(route)) = input_routes.get(index) {
                    set_windows_win32_window_view_input_route(created.main, route.clone());
                }
                if let Some(Some(route)) = shell_routes.get(index) {
                    set_windows_win32_window_shell_input_route(created.main, route.clone());
                }
                host.apply_main_window_appearance(created.main);
                host.apply_main_window_appearance(created.quick);
                let mut owned = WindowsWin32OwnedMainWindowHandles::new(created);
                if let Some(icon_path) = icon_path.as_deref() {
                    let icon = WindowsWin32OwnedAppIconResource::from_icon_path(icon_path)?;
                    owned.set_main_owned_app_icon(&mut host, icon);
                }
                if let Some(menu) = spec.menu.as_ref() {
                    owned.set_main_owned_menu(WindowsWin32OwnedWindowMenu::attach(
                        created.main,
                        menu,
                    )?);
                }
                if main_visible {
                    unsafe {
                        ShowWindow(created.main, SW_SHOW);
                        UpdateWindow(created.main);
                    }
                }
                handles.push(owned);
            }
            NativeMainWindowPresentation::Failed => {
                return Err(ZsuiError::host(
                    "create_windows_win32",
                    "failed to create Win32 main/quick windows",
                ));
            }
        }
    }
    Ok(handles)
}

#[derive(Debug)]
pub struct WindowsWin32OwnedMainWindowHandles {
    handles: NativeMainWindowHandles<HWND>,
    app_icons: Vec<WindowsWin32OwnedAppIconResource>,
    window_menu: Option<WindowsWin32OwnedWindowMenu>,
    destroy_on_drop: bool,
}

impl WindowsWin32OwnedMainWindowHandles {
    pub fn new(handles: NativeMainWindowHandles<HWND>) -> Self {
        Self {
            handles,
            app_icons: Vec::new(),
            window_menu: None,
            destroy_on_drop: true,
        }
    }

    pub const fn handles(&self) -> NativeMainWindowHandles<HWND> {
        self.handles
    }

    pub const fn main(&self) -> HWND {
        self.handles.main
    }

    pub const fn quick(&self) -> HWND {
        self.handles.quick
    }

    pub fn app_icon_count(&self) -> usize {
        self.app_icons.len()
    }

    pub fn set_main_owned_app_icon(
        &mut self,
        host: &mut WindowsWin32MainWindowHost,
        icon: WindowsWin32OwnedAppIconResource,
    ) {
        host.set_main_window_app_icon(self.main(), icon.as_native_resource());
        self.app_icons.push(icon);
    }

    pub fn set_main_owned_menu(&mut self, menu: WindowsWin32OwnedWindowMenu) {
        self.window_menu = Some(menu);
    }

    pub fn translate_accelerator(&self, message: &MSG) -> bool {
        self.window_menu
            .as_ref()
            .is_some_and(|menu| menu.translate_accelerator(message))
    }

    pub fn into_handles(mut self) -> NativeMainWindowHandles<HWND> {
        self.destroy_on_drop = false;
        let handles = self.handles;
        std::mem::forget(self);
        handles
    }
}

impl Drop for WindowsWin32OwnedMainWindowHandles {
    fn drop(&mut self) {
        if !self.destroy_on_drop {
            return;
        }
        if let Some(menu) = self.window_menu.take() {
            let _ = menu.detach_and_destroy();
        }
        for handle in [self.handles.quick, self.handles.main] {
            if handle.is_null() {
                continue;
            }
            clear_windows_win32_window_draw_plan(handle);
            clear_windows_win32_window_view_input_route(handle);
            clear_windows_win32_window_shell_input_route(handle);
            unsafe {
                if IsWindow(handle) != 0 {
                    DestroyWindow(handle);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedIcon {
    icon: HICON,
}

impl WindowsWin32OwnedIcon {
    pub fn from_raw(icon: HICON) -> Option<Self> {
        if icon.is_null() {
            None
        } else {
            Some(Self { icon })
        }
    }

    pub fn from_icon_path(path: impl AsRef<Path>, width: i32, height: i32) -> ZsuiResult<Self> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(ZsuiError::invalid_spec(
                "window.icon_path",
                "window icon path cannot be empty",
            ));
        }
        let path_wide = wide_path_null(path);
        let icon = unsafe {
            LoadImageW(
                null_mut(),
                path_wide.as_ptr(),
                IMAGE_ICON,
                width.max(1),
                height.max(1),
                LR_LOADFROMFILE | LR_DEFAULTCOLOR,
            )
        } as HICON;
        Self::from_raw(icon).ok_or_else(|| {
            ZsuiError::host(
                "windows_win32_load_icon",
                format!("failed to load icon from {}", path.display()),
            )
        })
    }

    pub const fn handle(&self) -> HICON {
        self.icon
    }

    pub fn into_raw(mut self) -> HICON {
        let icon = self.icon;
        self.icon = null_mut();
        icon
    }
}

impl Drop for WindowsWin32OwnedIcon {
    fn drop(&mut self) {
        if !self.icon.is_null() {
            unsafe {
                DestroyIcon(self.icon);
            }
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedAppIconResource {
    small: WindowsWin32OwnedIcon,
    big: Option<WindowsWin32OwnedIcon>,
}

impl WindowsWin32OwnedAppIconResource {
    pub fn from_raw(small: HICON, big: HICON) -> Option<Self> {
        let small_icon = WindowsWin32OwnedIcon::from_raw(small)?;
        let big_icon = if small == big {
            None
        } else {
            Some(WindowsWin32OwnedIcon::from_raw(big)?)
        };
        Some(Self {
            small: small_icon,
            big: big_icon,
        })
    }

    pub fn from_icon_path(path: impl AsRef<Path>) -> ZsuiResult<Self> {
        let path = path.as_ref();
        let small = WindowsWin32OwnedIcon::from_icon_path(
            path,
            system_metric(SM_CXSMICON),
            system_metric(SM_CYSMICON),
        )?;
        let big = WindowsWin32OwnedIcon::from_icon_path(
            path,
            system_metric(SM_CXICON),
            system_metric(SM_CYICON),
        )?;
        Ok(Self::from_owned_icons(small, big))
    }

    pub fn from_owned_icons(small: WindowsWin32OwnedIcon, big: WindowsWin32OwnedIcon) -> Self {
        let big_icon = if small.handle() == big.handle() {
            let _shared = big.into_raw();
            None
        } else {
            Some(big)
        };
        Self {
            small,
            big: big_icon,
        }
    }

    pub const fn small(&self) -> HICON {
        self.small.handle()
    }

    pub fn big(&self) -> HICON {
        self.big
            .as_ref()
            .map(WindowsWin32OwnedIcon::handle)
            .unwrap_or_else(|| self.small.handle())
    }

    pub fn as_native_resource(&self) -> NativeAppIconResource<isize> {
        NativeAppIconResource {
            small: self.small() as isize,
            big: self.big() as isize,
        }
    }

    pub fn into_raw_pair(self) -> (HICON, HICON) {
        let Self { small, big } = self;
        let small = small.into_raw();
        let big = big.map(WindowsWin32OwnedIcon::into_raw).unwrap_or(small);
        (small, big)
    }
}

fn system_metric(metric: i32) -> i32 {
    unsafe { GetSystemMetrics(metric).max(1) }
}


impl WindowsWin32ViewInputRoute {
    fn dispatch_app_command(&mut self, command: Command) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        report
            .app_command_names
            .push(crate::app_command_name(&command));
        report
            .events
            .push(format!("win32_window_menu_command:{command:?}"));

        if let Some(live_view) = &self.live_view {
            let update = live_view.dispatch_app_command(&command);
            if update.message_count > 0 {
                #[cfg(feature = "textbox")]
                let text_edit_commands = update.text_edit_commands.clone();
                report.handled = true;
                report.event_count = 1;
                report.message_count = update.message_count;
                report.app_command_count = update.commands.len();
                report.ui_command_count = update.ui_commands.len();
                report.live_view_revision = update.revision;
                report.quit_requested = update.quit_requested;
                for effect in update.commands {
                    report
                        .app_command_names
                        .push(crate::app_command_name(&effect));
                    if effect == Command::Quit {
                        report.quit_requested = true;
                        self.quit_requested = true;
                    }
                    self.pending_app_commands.push(effect);
                }
                for effect in update.ui_commands {
                    report.ui_command_ids.push(effect.id.0);
                    self.pending_ui_commands.push(effect);
                }
                if update.redraw {
                    self.interaction_plan = live_view.interaction_plan();
                    self.rebuild_pending_draw_plan();
                    report.hit_target_count = self.hit_target_count();
                    report
                        .events
                        .push(format!("win32_live_view_menu_repaint:{}", update.revision));
                }
                self.quit_requested |= update.quit_requested;
                #[cfg(feature = "textbox")]
                self.dispatch_text_edit_commands(text_edit_commands, &mut report);
                return report;
            }
        }

        report.app_command_count = 1;
        if command == Command::Quit {
            report.handled = true;
            report.quit_requested = true;
            self.quit_requested = true;
        }
        self.pending_app_commands.push(command);
        report
    }

    fn dispatch_window_close_requested(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let Some(command) = self.window_close_request_command.clone() else {
            return WindowsWin32ViewInputDispatchReport {
                window_close_request_count: 1,
                hit_target_count: self.hit_target_count(),
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        };
        let mut report = self.dispatch_app_command(command);
        report.window_close_request_count = 1;
        report
            .events
            .push("win32_window_close_requested".to_string());
        report
    }

    fn approve_next_close(&mut self) {
        self.close_approved = true;
    }

    fn take_close_approved(&mut self) -> bool {
        std::mem::take(&mut self.close_approved)
    }

}

pub fn run_windows_win32_native_window_event_loop(specs: &[WindowSpec]) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_status_items(specs, &[])
}

pub fn run_windows_win32_native_window_event_loop_with_status_items(
    specs: &[WindowSpec],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items(
        specs,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_draw_plans_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_draw_plans_input_routes_and_status_items(
        specs,
        draw_plans,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_draw_plans_input_routes_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    run_windows_win32_native_window_event_loop_with_routes_and_status_items(
        specs,
        draw_plans,
        input_routes,
        &[],
        status_items,
    )
}

pub fn run_windows_win32_native_window_event_loop_with_routes_and_status_items(
    specs: &[WindowSpec],
    draw_plans: &[Option<NativeDrawPlan>],
    input_routes: &[Option<WindowsWin32ViewInputRoute>],
    shell_routes: &[Option<WindowsWin32ShellInputRoute>],
    status_items: &[TraySpec],
) -> ZsuiResult<()> {
    if specs.is_empty() {
        return Ok(());
    }
    let _handles =
        create_owned_windows_for_specs_with_routes(specs, draw_plans, input_routes, shell_routes)?;
    let mut _status_item_host = None;
    if !status_items.is_empty() {
        let mut host = WindowsWin32StatusItemHost::new(_handles[0].main());
        for status_item in status_items {
            match host.create_status_item(NativeStatusItemRequest::from_tray_spec(status_item)) {
                NativeStatusItemPresentation::Created(_) => {}
                NativeStatusItemPresentation::Failed => {
                    return Err(ZsuiError::host(
                        "windows_win32_status_item",
                        host.last_error()
                            .unwrap_or("Win32 status item creation failed")
                            .to_string(),
                    ));
                }
            }
        }
        _status_item_host = Some(host);
    }
    match WindowsWin32MessageLoop::run_with_windows(&_handles) {
        WindowsWin32MessageLoopResult::Quit(_) => Ok(()),
        WindowsWin32MessageLoopResult::Failed => Err(ZsuiError::host(
            "windows_win32_message_loop",
            "GetMessageW failed",
        )),
    }
}
