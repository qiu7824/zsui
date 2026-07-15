pub struct WindowsWin32OwnedPopupMenu {
    menu: HMENU,
    command_table: WindowsWin32StatusMenuCommandTable,
    active: bool,
}

impl WindowsWin32OwnedPopupMenu {
    pub fn from_menu(menu: &MenuSpec) -> ZsuiResult<Self> {
        let command_table = WindowsWin32StatusMenuCommandTable::from_menu(menu);
        let handle = create_status_popup_menu(menu, &command_table)?;
        Ok(Self {
            menu: handle,
            command_table,
            active: true,
        })
    }

    pub const fn handle(&self) -> HMENU {
        self.menu
    }

    pub fn command_entry_count(&self) -> usize {
        self.command_table.entry_count()
    }

    pub fn dispatch_native_menu_command(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        self.command_table.resolve_native_command_id(native_id)
    }

    pub fn present_at(
        &self,
        owner: HWND,
        x: i32,
        y: i32,
    ) -> ZsuiResult<NativeStatusMenuCommandResult> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "status_item.owner",
                "Win32 popup menu owner window cannot be null",
            ));
        }
        unsafe {
            SetForegroundWindow(owner);
            let selected = TrackPopupMenu(
                self.menu,
                ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS,
                x,
                y,
                0,
                owner,
                null(),
            );
            if selected == 0 {
                Ok(NativeStatusMenuCommandResult::NotFound)
            } else {
                Ok(self.dispatch_native_menu_command(selected as u32))
            }
        }
    }

    pub fn destroy(mut self) -> bool {
        let destroyed = self.destroy_active();
        self.active = false;
        destroyed
    }

    fn destroy_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        let destroyed = unsafe { DestroyMenu(self.menu) != 0 };
        if destroyed {
            self.active = false;
        }
        destroyed
    }
}

impl Drop for WindowsWin32OwnedPopupMenu {
    fn drop(&mut self) {
        let _ = self.destroy_active();
    }
}

pub struct WindowsWin32TransientWindowHost {
    class_name: &'static str,
    window_proc: WNDPROC,
    operation_log: Vec<NativeTransientWindowHostOperation>,
}

impl WindowsWin32TransientWindowHost {
    pub fn new() -> Self {
        Self::with_window_proc(
            DEFAULT_TRANSIENT_CLASS_NAME,
            Some(zsui_win32_default_window_proc),
        )
    }

    pub fn with_window_proc(class_name: &'static str, window_proc: WNDPROC) -> Self {
        Self {
            class_name,
            window_proc,
            operation_log: Vec::new(),
        }
    }

    pub const fn class_name(&self) -> &'static str {
        self.class_name
    }

    pub fn operation_log(&self) -> &[NativeTransientWindowHostOperation] {
        &self.operation_log
    }

    fn record(&mut self, operation: NativeTransientWindowHostOperation) {
        self.operation_log.push(operation);
    }

    unsafe fn register_transient_class(&self, module: HINSTANCE, cursor: HCURSOR) -> bool {
        if self.class_name.is_empty() || self.window_proc.is_none() {
            return false;
        }
        let class_name = wide_null(self.class_name);
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: self.window_proc,
            hInstance: module,
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS
    }
}

impl Default for WindowsWin32TransientWindowHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeTransientWindowHost for WindowsWin32TransientWindowHost {
    type Handle = HWND;
    type Owner = HWND;

    fn create_transient_window(
        &mut self,
        request: NativeTransientWindowRequest<Self::Owner>,
    ) -> NativeTransientWindowPresentation<Self::Handle> {
        self.record(NativeTransientWindowHostOperation::CreateTransientWindow);
        unsafe {
            let module = WindowsWin32MainWindowHost::module_handle();
            let cursor = WindowsWin32MainWindowHost::arrow_cursor();
            if module.is_null()
                || cursor.is_null()
                || !self.register_transient_class(module, cursor)
            {
                return NativeTransientWindowPresentation::Failed;
            }

            let class_name = wide_null(self.class_name);
            let empty_title = wide_null("");
            let handle = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                empty_title.as_ptr(),
                WS_POPUP | WS_THICKFRAME,
                request.bounds.left,
                request.bounds.top,
                request.bounds.right - request.bounds.left,
                request.bounds.bottom - request.bounds.top,
                null_mut(),
                null_mut(),
                module,
                request.owner as _,
            );
            if handle.is_null() {
                NativeTransientWindowPresentation::Failed
            } else {
                NativeTransientWindowPresentation::Created(handle)
            }
        }
    }

    fn present_transient_window(&mut self, handle: Self::Handle, bounds: UiRect) {
        self.record(NativeTransientWindowHostOperation::PresentTransientWindow);
        unsafe {
            SetWindowPos(
                handle,
                HWND_TOPMOST,
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    fn hide_transient_window(&mut self, handle: Self::Handle) {
        self.record(NativeTransientWindowHostOperation::HideTransientWindow);
        unsafe {
            ShowWindow(handle, SW_HIDE);
        }
    }

    fn destroy_transient_window(&mut self, handle: Self::Handle) {
        self.record(NativeTransientWindowHostOperation::DestroyTransientWindow);
        unsafe {
            DestroyWindow(handle);
        }
    }
}
