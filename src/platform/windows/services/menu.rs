pub const ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID: u32 = 0x5800;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsWin32StatusMenuCommandEntry {
    pub native_id: u32,
    pub item_id: Option<String>,
    pub label: String,
    pub command: Command,
    pub enabled: bool,
    pub accelerator: Option<ZsAccelerator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsWin32StatusMenuCommandTable {
    entries: Vec<WindowsWin32StatusMenuCommandEntry>,
}

impl WindowsWin32StatusMenuCommandTable {
    pub fn from_menu(menu: &MenuSpec) -> Self {
        let mut entries = Vec::new();
        let mut next_id = ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID;
        collect_status_menu_commands(menu, true, &mut next_id, &mut entries);
        Self { entries }
    }

    pub fn entries(&self) -> &[WindowsWin32StatusMenuCommandEntry] {
        &self.entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn first_native_id(&self) -> Option<u32> {
        self.entries.first().map(|entry| entry.native_id)
    }

    pub fn resolve_native_command_id(&self, native_id: u32) -> NativeStatusMenuCommandResult {
        let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.native_id == native_id)
        else {
            return NativeStatusMenuCommandResult::NotFound;
        };

        if entry.enabled {
            NativeStatusMenuCommandResult::Dispatched(entry.command.clone())
        } else {
            NativeStatusMenuCommandResult::Disabled
        }
    }
}

fn collect_status_menu_commands(
    menu: &MenuSpec,
    inherited_enabled: bool,
    next_id: &mut u32,
    entries: &mut Vec<WindowsWin32StatusMenuCommandEntry>,
) {
    for item in &menu.items {
        match item {
            MenuItemSpec::Command {
                id,
                label,
                command,
                enabled,
                accelerator,
                ..
            } => {
                entries.push(WindowsWin32StatusMenuCommandEntry {
                    native_id: *next_id,
                    item_id: id.clone(),
                    label: label.clone(),
                    command: command.clone(),
                    enabled: inherited_enabled && *enabled,
                    accelerator: accelerator.clone(),
                });
                *next_id = (*next_id).saturating_add(1);
            }
            MenuItemSpec::Submenu { enabled, menu, .. } => {
                collect_status_menu_commands(menu, inherited_enabled && *enabled, next_id, entries);
            }
            MenuItemSpec::Separator => {}
        }
    }
}

#[derive(Debug)]
pub struct WindowsWin32OwnedWindowMenu {
    owner: HWND,
    menu: HMENU,
    accelerator_table: Option<WindowsWin32OwnedAcceleratorTable>,
    active: bool,
}

impl WindowsWin32OwnedWindowMenu {
    pub fn attach(owner: HWND, menu: &MenuSpec) -> ZsuiResult<Self> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "window.menu.owner",
                "Win32 window menu owner cannot be null",
            ));
        }
        let command_table = WindowsWin32StatusMenuCommandTable::from_menu(menu);
        let accelerator_table =
            WindowsWin32OwnedAcceleratorTable::from_command_table(&command_table)?;
        let handle = create_empty_window_menu()?;
        let mut command_index = 0;
        if let Err(err) =
            append_status_popup_menu_items(menu, handle, &command_table, &mut command_index)
        {
            unsafe {
                DestroyMenu(handle);
            }
            return Err(err);
        }
        if unsafe { SetMenu(owner, handle) } == 0 {
            unsafe {
                DestroyMenu(handle);
            }
            return Err(ZsuiError::host(
                "windows_win32_set_window_menu",
                "SetMenu failed",
            ));
        }
        unsafe {
            DrawMenuBar(owner);
        }
        set_windows_win32_window_menu_command_table(owner, command_table);
        Ok(Self {
            owner,
            menu: handle,
            accelerator_table,
            active: true,
        })
    }

    pub const fn handle(&self) -> HMENU {
        self.menu
    }

    pub fn accelerator_count(&self) -> usize {
        self.accelerator_table
            .as_ref()
            .map(WindowsWin32OwnedAcceleratorTable::entry_count)
            .unwrap_or(0)
    }

    fn translate_accelerator(&self, message: &MSG) -> bool {
        self.active
            && self
                .accelerator_table
                .as_ref()
                .is_some_and(|table| table.translate(self.owner, message))
    }

    pub fn detach_and_destroy(mut self) -> bool {
        self.detach_and_destroy_active()
    }

    fn detach_and_destroy_active(&mut self) -> bool {
        if !self.active {
            return true;
        }
        clear_windows_win32_window_menu_command_table(self.owner);
        if !self.owner.is_null() && unsafe { IsWindow(self.owner) } != 0 {
            unsafe {
                SetMenu(self.owner, null_mut());
                DrawMenuBar(self.owner);
            }
        }
        let destroyed = unsafe { DestroyMenu(self.menu) != 0 };
        if destroyed {
            self.active = false;
        }
        destroyed
    }
}

impl Drop for WindowsWin32OwnedWindowMenu {
    fn drop(&mut self) {
        let _ = self.detach_and_destroy_active();
    }
}

fn create_empty_window_menu() -> ZsuiResult<HMENU> {
    let handle = unsafe { CreateMenu() };
    if handle.is_null() {
        Err(ZsuiError::host(
            "windows_win32_create_window_menu",
            "CreateMenu failed",
        ))
    } else {
        Ok(handle)
    }
}

fn set_windows_win32_window_menu_command_table(
    hwnd: HWND,
    command_table: WindowsWin32StatusMenuCommandTable,
) {
    let hwnd = hwnd as isize;
    let mut records = window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned");
    records.retain(|record| record.hwnd != hwnd);
    records.push(WindowsWindowMenuCommandTableRecord {
        hwnd,
        command_table,
    });
}

fn clear_windows_win32_window_menu_command_table(hwnd: HWND) {
    let hwnd = hwnd as isize;
    window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .retain(|record| record.hwnd != hwnd);
}

fn clear_windows_win32_window_menu_command_tables() {
    window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .clear();
}

fn window_menu_command_tables() -> &'static Mutex<Vec<WindowsWindowMenuCommandTableRecord>> {
    WINDOW_MENU_COMMAND_TABLES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn dispatch_windows_win32_window_menu_command(
    hwnd: HWND,
    native_id: u32,
) -> Option<NativeStatusMenuCommandResult> {
    if hwnd.is_null() {
        return None;
    }
    let result = window_menu_command_tables()
        .lock()
        .expect("window menu command table registry should not be poisoned")
        .iter()
        .find(|record| record.hwnd == hwnd as isize)
        .map(|record| record.command_table.resolve_native_command_id(native_id))?;

    if let NativeStatusMenuCommandResult::Dispatched(command) = &result {
        if dispatch_windows_win32_window_view_input(hwnd, |route| {
            route.dispatch_app_command(command.clone())
        })
        .is_none()
            && *command == Command::Quit
        {
            unsafe {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    }
    Some(result)
}

fn create_status_popup_menu(
    menu: &MenuSpec,
    command_table: &WindowsWin32StatusMenuCommandTable,
) -> ZsuiResult<HMENU> {
    let handle = create_empty_status_popup_menu()?;

    let mut command_index = 0;
    if let Err(err) =
        append_status_popup_menu_items(menu, handle, command_table, &mut command_index)
    {
        unsafe {
            DestroyMenu(handle);
        }
        return Err(err);
    }
    Ok(handle)
}

fn create_empty_status_popup_menu() -> ZsuiResult<HMENU> {
    let handle = unsafe { CreatePopupMenu() };
    if handle.is_null() {
        Err(ZsuiError::host(
            "windows_win32_create_status_popup_menu",
            "CreatePopupMenu failed",
        ))
    } else {
        Ok(handle)
    }
}

fn append_status_popup_menu_items(
    menu: &MenuSpec,
    handle: HMENU,
    command_table: &WindowsWin32StatusMenuCommandTable,
    command_index: &mut usize,
) -> ZsuiResult<()> {
    for item in &menu.items {
        match item {
            MenuItemSpec::Command {
                label,
                checked,
                accelerator,
                ..
            } => {
                let entry = command_table.entries().get(*command_index).ok_or_else(|| {
                    ZsuiError::host(
                        "windows_win32_status_popup_menu",
                        "status menu command table is missing a command entry",
                    )
                })?;
                *command_index += 1;
                let display_label = accelerator
                    .as_ref()
                    .map(|accelerator| format!("{label}\t{accelerator}"))
                    .unwrap_or_else(|| label.clone());
                let label = wide_null(&display_label);
                let mut flags = MF_STRING;
                if !entry.enabled {
                    flags |= MF_GRAYED;
                }
                if *checked {
                    flags |= MF_CHECKED;
                }
                let appended =
                    unsafe { AppendMenuW(handle, flags, entry.native_id as usize, label.as_ptr()) };
                if appended == 0 {
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_item",
                        "AppendMenuW command item failed",
                    ));
                }
            }
            MenuItemSpec::Separator => {
                let appended = unsafe { AppendMenuW(handle, MF_SEPARATOR, 0, null()) };
                if appended == 0 {
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_separator",
                        "AppendMenuW separator failed",
                    ));
                }
            }
            MenuItemSpec::Submenu {
                label,
                enabled,
                menu,
                ..
            } => {
                let submenu = create_empty_status_popup_menu()?;
                if let Err(err) =
                    append_status_popup_menu_items(menu, submenu, command_table, command_index)
                {
                    unsafe {
                        DestroyMenu(submenu);
                    }
                    return Err(err);
                }
                let label = wide_null(label);
                let mut flags = MF_POPUP | MF_STRING;
                if !enabled {
                    flags |= MF_GRAYED;
                }
                let appended =
                    unsafe { AppendMenuW(handle, flags, submenu as usize, label.as_ptr()) };
                if appended == 0 {
                    unsafe {
                        DestroyMenu(submenu);
                    }
                    return Err(ZsuiError::host(
                        "windows_win32_append_status_popup_submenu",
                        "AppendMenuW submenu failed",
                    ));
                }
            }
        }
    }
    Ok(())
}
