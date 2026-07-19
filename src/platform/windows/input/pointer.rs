impl WindowsWin32ViewInputRoute {
    fn dispatch_click(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        let target = self.shared_target_at(point);
        let down = self.shared_runtime.dispatch_pointer_down(point, false);
        let mut report =
            self.adapt_shared_report(down, WindowsSharedInputKind::PointerDown(target));
        let up = self.shared_runtime.dispatch_pointer_up(point);
        report.merge(self.adapt_shared_report(
            up,
            WindowsSharedInputKind::PointerUp(target),
        ));
        report.click_count = 1;
        report
    }

    fn dispatch_pointer_down(
        &mut self,
        point: crate::Point,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let target = self.shared_target_at(point);
        let report = self.shared_runtime.dispatch_pointer_down(point, shift);
        self.adapt_shared_report(report, WindowsSharedInputKind::PointerDown(target))
    }

    fn dispatch_pointer_move(
        &mut self,
        point: crate::Point,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_pointer_move_at(point, std::time::Instant::now())
    }

    fn dispatch_pointer_move_at(
        &mut self,
        point: crate::Point,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_pointer_move_at(point, now);
        self.adapt_shared_report(report, WindowsSharedInputKind::PointerMove)
    }

    fn dispatch_pointer_up(
        &mut self,
        point: crate::Point,
    ) -> WindowsWin32ViewInputDispatchReport {
        let target = self.shared_target_at(point);
        let report = self.shared_runtime.dispatch_pointer_up(point);
        self.adapt_shared_report(report, WindowsSharedInputKind::PointerUp(target))
    }

    fn cancel_pointer_drag(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.cancel_pointer_drag();
        self.adapt_shared_report(report, WindowsSharedInputKind::PointerUp(None))
    }

    fn dispatch_pointer_leave(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_pointer_leave();
        self.adapt_shared_report(report, WindowsSharedInputKind::PointerLeave)
    }

    fn dispatch_scroll(
        &mut self,
        point: crate::Point,
        delta: crate::Dp,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_pointer_scroll(point, delta);
        self.adapt_shared_report(report, WindowsSharedInputKind::Scroll)
    }
}

#[derive(Debug, Clone)]
pub struct WindowsWin32ShellInputRoute {
    runtime: ZsShellRuntime,
    events: Vec<ZsShellInteractionEvent>,
}

impl WindowsWin32ShellInputRoute {
    pub fn new(runtime: ZsShellRuntime) -> Self {
        Self {
            runtime,
            events: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &ZsShellRuntime {
        &self.runtime
    }

    pub fn events(&self) -> &[ZsShellInteractionEvent] {
        &self.events
    }
}

pub fn set_windows_win32_window_shell_input_route(
    hwnd: HWND,
    mut route: WindowsWin32ShellInputRoute,
) -> bool {
    if hwnd.is_null() {
        return false;
    }
    if let Some((bounds, dpi)) = windows_win32_shell_surface(hwnd) {
        route.runtime.set_surface(bounds, dpi);
    }
    let plan = route.runtime.draw_plan();
    let hwnd_value = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    if let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) {
        record.route = route;
    } else {
        routes.push(WindowsWindowShellInputRouteRecord {
            hwnd: hwnd_value,
            route,
        });
    }
    drop(routes);
    set_windows_win32_window_draw_plan(hwnd, plan);
    unsafe {
        InvalidateRect(hwnd, null(), 0);
    }
    true
}

pub fn clear_windows_win32_window_shell_input_route(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_shell_input_routes() {
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.clear();
}

pub fn windows_win32_window_shell_input_events(hwnd: HWND) -> Option<Vec<ZsShellInteractionEvent>> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.route.events.clone())
}

pub fn dispatch_windows_win32_window_shell_pointer_move(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_move(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_leave(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_leave)
}

pub fn dispatch_windows_win32_window_shell_pointer_down(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_down(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_up(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_up)
}

pub fn dispatch_windows_win32_window_shell_pointer_cancel(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_cancel)
}

pub fn dispatch_windows_win32_window_shell_scroll(
    hwnd: HWND,
    delta_y: i32,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.scroll_by(delta_y))
}

pub fn refresh_windows_win32_window_shell_surface(hwnd: HWND) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |_| ZsShellInteractionUpdate::default())
}

fn dispatch_windows_win32_window_shell_update(
    hwnd: HWND,
    update: impl FnOnce(&mut ZsShellRuntime) -> ZsShellInteractionUpdate,
) -> Option<ZsShellInteractionUpdate> {
    if hwnd.is_null() {
        return None;
    }
    let surface = windows_win32_shell_surface(hwnd);
    let hwnd_value = hwnd as isize;
    let (result, plan) = {
        let mut routes = window_shell_input_routes()
            .lock()
            .expect("window shell input route registry should not be poisoned");
        let record = routes.iter_mut().find(|record| record.hwnd == hwnd_value)?;
        let surface_changed = surface
            .map(|(bounds, dpi)| record.route.runtime.set_surface(bounds, dpi))
            .unwrap_or(false);
        let mut result = update(&mut record.route.runtime);
        if surface_changed {
            result.redraw = true;
        }
        record.route.events.extend(result.events.iter().cloned());
        let plan = result.redraw.then(|| record.route.runtime.draw_plan());
        (result, plan)
    };

    if let Some(plan) = plan {
        set_windows_win32_window_draw_plan(hwnd, plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    Some(result)
}

fn window_shell_input_routes() -> &'static Mutex<Vec<WindowsWindowShellInputRouteRecord>> {
    WINDOW_SHELL_INPUT_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

fn windows_win32_shell_surface(hwnd: HWND) -> Option<(crate::Rect, crate::Dpi)> {
    let mut rect: RECT = unsafe { zeroed() };
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
        return None;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32;
    Some((rect_from_win(rect), crate::Dpi(dpi)))
}

fn track_windows_win32_shell_pointer_leave(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let mut event = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: HOVER_DEFAULT,
    };
    unsafe {
        TrackMouseEvent(&mut event);
    }
}
