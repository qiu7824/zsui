#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsWin32StatusItemRouteRecord {
    owner: isize,
    callback_message: u32,
    tray_id: u32,
    tooltip: Option<String>,
    icon: Option<isize>,
    menu: MenuSpec,
}

impl WindowsWin32StatusItemRouteRecord {
    fn notify_data(&self) -> NOTIFYICONDATAW {
        tray_notify_data(
            self.owner as HWND,
            self.tray_id,
            self.tooltip.as_deref(),
            self.icon.map(|icon| icon as HICON),
            self.callback_message,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsWin32StatusItemCallbackTarget {
    tray_id: u32,
    event_message: u32,
    menu: MenuSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WindowsWin32StatusItemCallbackDispatch {
    Ignored,
    Menu(NativeStatusMenuCommandResult),
    Failed(String),
}

static WINDOWS_WIN32_STATUS_ITEM_ROUTES: OnceLock<
    Mutex<Vec<WindowsWin32StatusItemRouteRecord>>,
> = OnceLock::new();
static WINDOWS_WIN32_TASKBAR_CREATED_MESSAGE: OnceLock<u32> = OnceLock::new();

pub struct WindowsWin32OwnedTrayIcon {
    data: NOTIFYICONDATAW,
    icon: Option<WindowsWin32OwnedIcon>,
    tooltip: Option<String>,
    menu: MenuSpec,
    menu_command_table: WindowsWin32StatusMenuCommandTable,
    active: bool,
}

impl WindowsWin32OwnedTrayIcon {
    pub fn create(
        owner: HWND,
        id: u32,
        request: NativeStatusItemRequest,
        callback_message: u32,
    ) -> ZsuiResult<Self> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "status_item.owner",
                "Win32 tray icon owner window cannot be null",
            ));
        }
        let icon = request
            .icon_path
            .as_deref()
            .map(|path| {
                WindowsWin32OwnedIcon::from_icon_path(
                    path,
                    system_metric(SM_CXSMICON),
                    system_metric(SM_CYSMICON),
                )
            })
            .transpose()?;
        let mut data = tray_notify_data(
            owner,
            id,
            request.tooltip.as_deref(),
            icon.as_ref().map(WindowsWin32OwnedIcon::handle),
            callback_message,
        );
        let created = unsafe { Shell_NotifyIconW(NIM_ADD, &mut data) != 0 };
        if !created {
            return Err(ZsuiError::host(
                "windows_win32_create_tray_icon",
                "Shell_NotifyIconW(NIM_ADD) failed",
            ));
        }
        let menu_command_table = WindowsWin32StatusMenuCommandTable::from_menu(&request.menu);
        let item = Self {
            data,
            icon,
            tooltip: request.tooltip,
            menu: request.menu,
            menu_command_table,
            active: true,
        };
        set_windows_win32_status_item_route(item.route_record());
        Ok(item)
    }

    pub const fn id(&self) -> u32 {
        self.data.uID
    }

    pub const fn owner(&self) -> HWND {
        self.data.hWnd
    }

    pub const fn callback_message(&self) -> u32 {
        self.data.uCallbackMessage
    }

    pub fn menu(&self) -> &MenuSpec {
        &self.menu
    }

    pub fn menu_command_table(&self) -> &WindowsWin32StatusMenuCommandTable {
        &self.menu_command_table
    }

    pub fn has_icon(&self) -> bool {
        self.icon.is_some()
    }

    pub fn set_tooltip(&mut self, tooltip: Option<&str>) -> bool {
        clear_wide_buffer(&mut self.data.szTip);
        self.data.uFlags |= NIF_TIP;
        if let Some(tooltip) = tooltip {
            copy_wide_truncated(tooltip, &mut self.data.szTip);
        }
        self.tooltip = tooltip.map(str::to_owned);
        let updated = unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut self.data) != 0 };
        set_windows_win32_status_item_route(self.route_record());
        updated
    }

    pub fn set_menu(&mut self, menu: MenuSpec) {
        self.menu_command_table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);
        self.menu = menu;
        set_windows_win32_status_item_route(self.route_record());
    }

    pub fn dispatch_native_menu_command(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        self.menu_command_table.resolve_native_command_id(native_id)
    }

    pub fn create_popup_menu(&self) -> ZsuiResult<WindowsWin32OwnedPopupMenu> {
        WindowsWin32OwnedPopupMenu::from_menu(&self.menu)
    }

    pub fn delete(mut self) -> bool {
        let deleted = self.delete_active();
        self.active = false;
        deleted
    }

    fn delete_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        let deleted = unsafe { Shell_NotifyIconW(NIM_DELETE, &mut self.data) != 0 };
        if deleted {
            self.active = false;
        }
        deleted
    }

    fn route_record(&self) -> WindowsWin32StatusItemRouteRecord {
        WindowsWin32StatusItemRouteRecord {
            owner: self.owner() as isize,
            callback_message: self.callback_message(),
            tray_id: self.id(),
            tooltip: self.tooltip.clone(),
            icon: self.icon.as_ref().map(|icon| icon.handle() as isize),
            menu: self.menu.clone(),
        }
    }
}

impl Drop for WindowsWin32OwnedTrayIcon {
    fn drop(&mut self) {
        clear_windows_win32_status_item_route(self.owner(), self.callback_message(), self.id());
        let _ = self.delete_active();
    }
}

pub struct WindowsWin32StatusItemHost {
    owner: HWND,
    callback_message: u32,
    next_id: u32,
    items: Vec<WindowsWin32OwnedTrayIcon>,
    operation_log: Vec<NativeStatusItemHostOperation>,
    status_menu_operation_log: Vec<NativeStatusMenuCommandHostOperation>,
    last_error: Option<String>,
}

impl WindowsWin32StatusItemHost {
    pub fn new(owner: HWND) -> Self {
        Self::with_callback_message(owner, ZSUI_WIN32_TRAY_CALLBACK_MESSAGE)
    }

    pub fn with_callback_message(owner: HWND, callback_message: u32) -> Self {
        Self {
            owner,
            callback_message,
            next_id: 1,
            items: Vec::new(),
            operation_log: Vec::new(),
            status_menu_operation_log: Vec::new(),
            last_error: None,
        }
    }

    pub const fn owner(&self) -> HWND {
        self.owner
    }

    pub fn operation_log(&self) -> &[NativeStatusItemHostOperation] {
        &self.operation_log
    }

    pub fn status_menu_operation_log(&self) -> &[NativeStatusMenuCommandHostOperation] {
        &self.status_menu_operation_log
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn first_native_menu_command_id(&self, status_item_index: usize) -> Option<u32> {
        self.items
            .get(status_item_index)
            .and_then(|item| item.menu_command_table().first_native_id())
    }

    pub fn native_menu_command_count(&self, status_item_index: usize) -> usize {
        self.items
            .get(status_item_index)
            .map(|item| item.menu_command_table().entry_count())
            .unwrap_or(0)
    }

    pub fn dispatch_native_menu_command(
        &mut self,
        status_item_index: usize,
        native_command_id: u32,
    ) -> NativeStatusMenuCommandResult {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        self.items
            .get(status_item_index)
            .map(|item| item.dispatch_native_menu_command(native_command_id))
            .unwrap_or(NativeStatusMenuCommandResult::NotFound)
    }

    pub fn create_popup_menu_for_status_item(
        &self,
        status_item_index: usize,
    ) -> ZsuiResult<WindowsWin32OwnedPopupMenu> {
        self.items
            .get(status_item_index)
            .ok_or_else(|| {
                ZsuiError::invalid_spec(
                    "status_item_index",
                    "Win32 status item index does not exist",
                )
            })?
            .create_popup_menu()
    }

    pub fn present_status_item_menu_at(
        &mut self,
        status_item_index: usize,
        x: i32,
        y: i32,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        let popup = self.create_popup_menu_for_status_item(status_item_index)?;
        popup.present_at(self.owner, x, y)
    }

    pub fn present_status_item_menu_at_cursor(
        &mut self,
        status_item_index: usize,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        let mut point = POINT { x: 0, y: 0 };
        let ok = unsafe { GetCursorPos(&mut point) != 0 };
        if !ok {
            return Err(ZsuiError::host(
                "windows_win32_status_popup_menu_position",
                "GetCursorPos failed",
            ));
        }
        self.present_status_item_menu_at(status_item_index, point.x, point.y)
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    fn record(&mut self, operation: NativeStatusItemHostOperation) {
        self.operation_log.push(operation);
    }
}

impl NativeStatusItemHost for WindowsWin32StatusItemHost {
    type Handle = u32;

    fn create_status_item(
        &mut self,
        request: NativeStatusItemRequest,
    ) -> NativeStatusItemPresentation<Self::Handle> {
        self.record(NativeStatusItemHostOperation::CreateStatusItem);
        self.last_error = None;
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);
        match WindowsWin32OwnedTrayIcon::create(self.owner, id, request, self.callback_message) {
            Ok(item) => {
                self.items.push(item);
                NativeStatusItemPresentation::Created(id)
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
                NativeStatusItemPresentation::Failed
            }
        }
    }

    fn set_status_item_tooltip(&mut self, handle: Self::Handle, tooltip: Option<String>) {
        self.record(NativeStatusItemHostOperation::SetStatusItemTooltip);
        if let Some(item) = self.items.iter_mut().find(|item| item.id() == handle) {
            let _ = item.set_tooltip(tooltip.as_deref());
        }
    }

    fn set_status_item_menu(&mut self, handle: Self::Handle, menu: MenuSpec) {
        self.record(NativeStatusItemHostOperation::SetStatusItemMenu);
        if let Some(item) = self.items.iter_mut().find(|item| item.id() == handle) {
            item.set_menu(menu);
        }
    }

    fn destroy_status_item(&mut self, handle: Self::Handle) {
        self.record(NativeStatusItemHostOperation::DestroyStatusItem);
        if let Some(index) = self.items.iter().position(|item| item.id() == handle) {
            let item = self.items.remove(index);
            let _ = item.delete();
        }
    }
}

impl NativeStatusMenuCommandHost for WindowsWin32StatusItemHost {
    fn dispatch_status_menu_command(
        &mut self,
        request: NativeStatusMenuCommandRequest,
    ) -> NativeStatusMenuCommandResult {
        self.status_menu_operation_log
            .push(NativeStatusMenuCommandHostOperation::DispatchStatusMenuCommand);
        let Some(item) = self.items.get(request.status_item_index) else {
            return NativeStatusMenuCommandResult::NotFound;
        };
        native_status_menu_command_from_menu(item.menu(), &request)
    }
}

fn windows_win32_status_item_routes(
) -> &'static Mutex<Vec<WindowsWin32StatusItemRouteRecord>> {
    WINDOWS_WIN32_STATUS_ITEM_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

fn set_windows_win32_status_item_route(route: WindowsWin32StatusItemRouteRecord) {
    let mut routes = windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned");
    routes.retain(|candidate| {
        candidate.owner != route.owner
            || candidate.callback_message != route.callback_message
            || candidate.tray_id != route.tray_id
    });
    routes.push(route);
}

fn clear_windows_win32_status_item_route(owner: HWND, callback_message: u32, tray_id: u32) {
    let owner = owner as isize;
    windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned")
        .retain(|route| {
            route.owner != owner
                || route.callback_message != callback_message
                || route.tray_id != tray_id
        });
}

fn clear_windows_win32_status_item_routes_for_owner(owner: HWND) {
    let owner = owner as isize;
    windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned")
        .retain(|route| route.owner != owner);
}

fn clear_windows_win32_status_item_routes() {
    windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned")
        .clear();
}

fn windows_win32_status_item_callback_target(
    owner: HWND,
    callback_message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<WindowsWin32StatusItemCallbackTarget> {
    if owner.is_null() {
        return None;
    }
    let owner = owner as isize;
    let tray_id = wparam as u32;
    let event_message = lparam as u32;
    windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned")
        .iter()
        .find(|route| {
            route.owner == owner
                && route.callback_message == callback_message
                && route.tray_id == tray_id
        })
        .map(|route| WindowsWin32StatusItemCallbackTarget {
            tray_id,
            event_message,
            menu: route.menu.clone(),
        })
}

fn dispatch_windows_win32_status_item_callback(
    owner: HWND,
    callback_message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<WindowsWin32StatusItemCallbackDispatch> {
    let target = windows_win32_status_item_callback_target(
        owner,
        callback_message,
        wparam,
        lparam,
    )?;
    if !matches!(target.event_message, WM_RBUTTONUP | WM_CONTEXTMENU) {
        return Some(WindowsWin32StatusItemCallbackDispatch::Ignored);
    }
    let popup = match WindowsWin32OwnedPopupMenu::from_menu(&target.menu) {
        Ok(popup) => popup,
        Err(err) => {
            return Some(WindowsWin32StatusItemCallbackDispatch::Failed(
                err.to_string(),
            ));
        }
    };
    let mut point = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut point) } == 0 {
        return Some(WindowsWin32StatusItemCallbackDispatch::Failed(
            "GetCursorPos failed".to_string(),
        ));
    }
    match popup.present_at(owner, point.x, point.y) {
        Ok(result) => {
            if let NativeStatusMenuCommandResult::Dispatched(command) = &result {
                dispatch_windows_win32_app_command(owner, command.clone());
            }
            Some(WindowsWin32StatusItemCallbackDispatch::Menu(result))
        }
        Err(err) => Some(WindowsWin32StatusItemCallbackDispatch::Failed(
            err.to_string(),
        )),
    }
}

fn windows_win32_taskbar_created_message() -> u32 {
    *WINDOWS_WIN32_TASKBAR_CREATED_MESSAGE.get_or_init(|| {
        let message_name = wide_null("TaskbarCreated");
        unsafe { RegisterWindowMessageW(message_name.as_ptr()) }
    })
}

fn restore_windows_win32_status_items(owner: HWND) -> usize {
    if owner.is_null() {
        return 0;
    }
    let routes = windows_win32_status_item_routes()
        .lock()
        .expect("Win32 status item route registry should not be poisoned")
        .iter()
        .filter(|route| route.owner == owner as isize)
        .cloned()
        .collect::<Vec<_>>();
    routes
        .into_iter()
        .filter(|route| {
            let mut data = route.notify_data();
            unsafe { Shell_NotifyIconW(NIM_ADD, &mut data) != 0 }
        })
        .count()
}

fn tray_notify_data(
    owner: HWND,
    id: u32,
    tooltip: Option<&str>,
    icon: Option<HICON>,
    callback_message: u32,
) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { zeroed() };
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = owner;
    data.uID = id;
    data.uFlags = NIF_MESSAGE;
    data.uCallbackMessage = callback_message;
    if let Some(tooltip) = tooltip {
        data.uFlags |= NIF_TIP;
        copy_wide_truncated(tooltip, &mut data.szTip);
    }
    if let Some(icon) = icon {
        data.uFlags |= NIF_ICON;
        data.hIcon = icon;
    }
    data
}

fn copy_wide_truncated(value: &str, target: &mut [u16]) {
    if target.is_empty() {
        return;
    }
    clear_wide_buffer(target);
    let max_chars = target.len().saturating_sub(1);
    for (slot, value) in target.iter_mut().take(max_chars).zip(value.encode_utf16()) {
        *slot = value;
    }
}

fn clear_wide_buffer(target: &mut [u16]) {
    for value in target {
        *value = 0;
    }
}
