//! Scoped insertion into borrowed native menus and owner-bound message hooks.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DeleteMenu, GetMenuItemCount, GetMenuItemInfoW, InsertMenuItemW, IsMenu,
    MENUITEMINFOW, MFS_CHECKED, MFS_DISABLED, MFT_SEPARATOR, MF_BYPOSITION, MIIM_DATA, MIIM_FTYPE,
    MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU, WM_DRAWITEM, WM_INITMENUPOPUP, WM_MEASUREITEM,
    WM_MENUCHAR,
};

/// A borrowed menu. The host never takes ownership of the external HMENU.
pub struct NativeMenu<'a> {
    handle: HMENU,
    _owner: PhantomData<&'a mut ()>,
}
impl NativeMenu<'_> {
    /// Borrows a live native menu without transferring ownership.
    ///
    /// # Safety
    /// The owner must keep the menu alive on the current UI thread until the
    /// borrow and all insertion guards have ended.
    pub unsafe fn from_handle(handle: HMENU) -> ZsuiResult<Self> {
        if IsMenu(handle) == 0 {
            return Err(ZsuiError::invalid_spec(
                "native_menu",
                "menu handle is not valid",
            ));
        }
        Ok(Self {
            handle,
            _owner: PhantomData,
        })
    }
}

struct InsertedItem {
    parent: HMENU,
    marker: Box<u8>,
}

/// Owns only the inserted application rows; dropping restores the borrowed menu.
pub struct NativeMenuInsertion<'a> {
    menu: HMENU,
    rows: Vec<InsertedItem>,
    commands: BTreeMap<u32, Command>,
    _menu: PhantomData<&'a mut ()>,
}

impl NativeMenuInsertion<'_> {
    /// Tracks this merged menu on its owning ZSUI window.
    ///
    /// # Safety
    /// `owner` must be a live ZSUI HWND on the calling UI thread.
    pub unsafe fn track(
        &self,
        owner: HWND,
        point: crate::Point,
        hook: Rc<dyn NativeMenuHook>,
    ) -> ZsuiResult<u32> {
        track_native_menu(
            owner,
            &NativeMenu {
                handle: self.menu,
                _owner: PhantomData,
            },
            point,
            hook,
        )
    }
    /// Resolves a selected native ID to its typed application command.
    pub fn command(&self, id: u32) -> Option<&Command> {
        self.commands.get(&id)
    }
    /// Returns the application command ID map.
    pub fn commands(&self) -> &BTreeMap<u32, Command> {
        &self.commands
    }

    fn insert(
        &mut self,
        handle: HMENU,
        position: u32,
        menu: &MenuSpec,
        next: &mut u32,
        occupied: &mut BTreeSet<u32>,
        parent_enabled: bool,
    ) -> ZsuiResult<()> {
        for (offset, item) in menu.items.iter().enumerate() {
            let marker = Box::new(0u8);
            let mut native: MENUITEMINFOW = unsafe { zeroed() };
            native.cbSize = size_of::<MENUITEMINFOW>() as u32;
            native.fMask = MIIM_DATA | MIIM_FTYPE;
            native.dwItemData = (&*marker as *const u8) as usize;
            let mut label = Vec::new();
            let mut submenu = null_mut();
            match item {
                MenuItemSpec::Separator => native.fType = MFT_SEPARATOR,
                MenuItemSpec::Command {
                    label: text,
                    command,
                    enabled,
                    checked,
                    accelerator,
                    ..
                } => {
                    while occupied.contains(next) && *next <= 0xffff {
                        *next += 1;
                    }
                    if *next == 0 || *next > 0xffff {
                        return Err(ZsuiError::invalid_spec(
                            "native_menu.id",
                            "native command ID range exhausted",
                        ));
                    }
                    native.fMask |= MIIM_ID | MIIM_STATE | MIIM_STRING;
                    native.wID = *next;
                    occupied.insert(*next);
                    if *enabled && parent_enabled {
                        self.commands.insert(*next, command.clone());
                    } else {
                        native.fState |= MFS_DISABLED;
                    }
                    if *checked {
                        native.fState |= MFS_CHECKED;
                    }
                    *next += 1;
                    label = wide_null(&accelerator.map_or_else(
                        || text.clone(),
                        |accelerator| format!("{text}\t{accelerator}"),
                    ));
                }
                MenuItemSpec::Submenu {
                    label: text,
                    enabled,
                    menu,
                    ..
                } => {
                    submenu = unsafe { CreatePopupMenu() };
                    if submenu.is_null() {
                        return Err(ZsuiError::host("native_menu", "CreatePopupMenu failed"));
                    }
                    if let Err(error) =
                        self.insert(submenu, 0, menu, next, occupied, parent_enabled && *enabled)
                    {
                        unsafe {
                            DestroyMenu(submenu);
                        }
                        return Err(error);
                    }
                    native.fMask |= MIIM_SUBMENU | MIIM_STRING | MIIM_STATE;
                    native.hSubMenu = submenu;
                    if !enabled || !parent_enabled {
                        native.fState |= MFS_DISABLED;
                    }
                    label = wide_null(text);
                }
            }
            if !label.is_empty() {
                native.dwTypeData = label.as_mut_ptr();
                native.cch = label.len().saturating_sub(1) as u32;
            }
            if unsafe { InsertMenuItemW(handle, position + offset as u32, 1, &native) } == 0 {
                if !submenu.is_null() {
                    unsafe {
                        DestroyMenu(submenu);
                    }
                }
                return Err(ZsuiError::host("native_menu", "InsertMenuItemW failed"));
            }
            self.rows.push(InsertedItem {
                parent: handle,
                marker,
            });
        }
        Ok(())
    }
}

impl Drop for NativeMenuInsertion<'_> {
    fn drop(&mut self) {
        for row in self.rows.iter().rev() {
            unsafe {
                if IsMenu(row.parent) == 0 {
                    continue;
                }
                let marker = (&*row.marker as *const u8) as usize;
                for position in (0..GetMenuItemCount(row.parent).max(0)).rev() {
                    let mut item: MENUITEMINFOW = zeroed();
                    item.cbSize = size_of::<MENUITEMINFOW>() as u32;
                    item.fMask = MIIM_DATA;
                    if GetMenuItemInfoW(row.parent, position as u32, 1, &mut item) != 0
                        && item.dwItemData == marker
                    {
                        DeleteMenu(row.parent, position as u32, MF_BYPOSITION);
                        break;
                    }
                }
            }
        }
    }
}

fn collect_ids(menu: HMENU, ids: &mut BTreeSet<u32>, depth: usize) -> ZsuiResult<()> {
    if depth > 64 {
        return Err(ZsuiError::invalid_spec(
            "native_menu",
            "menu hierarchy is too deep",
        ));
    }
    unsafe {
        for index in 0..GetMenuItemCount(menu).max(0) {
            let mut item: MENUITEMINFOW = zeroed();
            item.cbSize = size_of::<MENUITEMINFOW>() as u32;
            item.fMask = MIIM_ID | MIIM_SUBMENU;
            if GetMenuItemInfoW(menu, index as u32, 1, &mut item) == 0 {
                return Err(ZsuiError::host("native_menu", "GetMenuItemInfoW failed"));
            }
            ids.insert(item.wID);
            if !item.hSubMenu.is_null() {
                collect_ids(item.hSubMenu, ids, depth + 1)?;
            }
        }
    }
    Ok(())
}

impl MenuSpec {
    /// Inserts application rows into a borrowed Win32 menu and returns an ID map
    /// plus a guard that removes only those rows when the tracking session ends.
    pub fn merge_into<'a>(
        &self,
        menu: &'a mut NativeMenu<'_>,
        position: u32,
        first_id: u16,
    ) -> ZsuiResult<NativeMenuInsertion<'a>> {
        let count = unsafe { GetMenuItemCount(menu.handle) };
        if count < 0 || position > count as u32 {
            return Err(ZsuiError::invalid_spec(
                "native_menu.position",
                "insertion position is outside the menu",
            ));
        }
        let mut occupied = BTreeSet::new();
        collect_ids(menu.handle, &mut occupied, 0)?;
        let mut insertion = NativeMenuInsertion {
            menu: menu.handle,
            rows: Vec::new(),
            commands: BTreeMap::new(),
            _menu: PhantomData,
        };
        insertion.insert(
            menu.handle,
            position,
            self,
            &mut u32::from(first_id),
            &mut occupied,
            true,
        )?;
        Ok(insertion)
    }
}

/// A hook active only while one borrowed menu is being tracked.
pub trait NativeMenuHook {
    /// Handles native menu initialization, measurement, drawing or keyboard messages.
    fn handle_message(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT>;
}

pub(super) fn dispatch_menu_hook(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    if !matches!(
        message,
        WM_INITMENUPOPUP | WM_MEASUREITEM | WM_DRAWITEM | WM_MENUCHAR
    ) {
        return None;
    }
    let services = window_services(hwnd)?;
    let hook = services.menu_hook.borrow().clone()?;
    #[cfg(feature = "native-smoke")]
    services
        .menu_message_count
        .set(services.menu_message_count.get().saturating_add(1));
    hook.handle_message(message, wparam, lparam)
}

/// Tracks an external native menu with a window-owned, automatically removed hook.
///
/// # Safety
/// `owner` must be a live ZSUI HWND on the current UI thread, and `menu` must
/// remain alive throughout tracking. This function never destroys the menu.
pub unsafe fn track_native_menu(
    owner: HWND,
    menu: &NativeMenu<'_>,
    point: crate::Point,
    hook: Rc<dyn NativeMenuHook>,
) -> ZsuiResult<u32> {
    let services = window_services(owner)
        .ok_or_else(|| ZsuiError::host("native_menu", "owner window no longer exists"))?;
    #[cfg(feature = "dark-mode")]
    let _appearance = dark_mode_compatibility::AppearanceScope::new(
        services
            .palette
            .get()
            .map(|palette| palette.mode)
            .unwrap_or_default(),
    );
    if services.menu_hook.borrow().is_some() {
        return Err(ZsuiError::host(
            "native_menu",
            "a native menu is already being tracked by this window",
        ));
    }
    struct Guard(Rc<WindowsWindowServices>);
    impl Drop for Guard {
        fn drop(&mut self) {
            self.0.menu_hook.borrow_mut().take();
        }
    }
    *services.menu_hook.borrow_mut() = Some(hook);
    let _guard = Guard(services);
    let mut screen = POINT {
        x: point.x,
        y: point.y,
    };
    windows_sys::Win32::Graphics::Gdi::ClientToScreen(owner, &mut screen);
    // NONOTIFY would suppress the initialization/owner-draw messages required
    // by IContextMenu2/3. RETURNCMD owns command delivery through the return value.
    let selected = TrackPopupMenu(
        menu.handle,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        screen.x,
        screen.y,
        0,
        owner,
        null(),
    );
    Ok(selected as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merging_preserves_external_ids_and_restores_only_owned_rows() {
        super::super::integration_tests::on_ui(|| unsafe {
            struct Owned(HMENU);
            impl Drop for Owned {
                fn drop(&mut self) {
                    unsafe {
                        DestroyMenu(self.0);
                    }
                }
            }
            let owned = Owned(CreatePopupMenu());
            assert!(!owned.0.is_null());
            assert_ne!(
                AppendMenuW(
                    owned.0,
                    MF_STRING,
                    4,
                    wide_null("External command").as_ptr()
                ),
                0
            );
            let mut borrowed = NativeMenu::from_handle(owned.0).unwrap();
            {
                let menu = MenuSpec::new()
                    .item("Application command", Command::custom("app.action"))
                    .separator()
                    .submenu(
                        "More",
                        MenuSpec::new().item("Nested", Command::custom("app.nested")),
                    );
                let inserted = menu.merge_into(&mut borrowed, 0, 4).unwrap();
                assert_eq!(inserted.commands.len(), 2);
                assert!(inserted.command(4).is_none());
                assert_eq!(inserted.command(5), Some(&Command::custom("app.action")));
                assert_eq!(GetMenuItemCount(owned.0), 4);
                // External additions during tracking must survive guard cleanup.
                assert_ne!(
                    AppendMenuW(
                        owned.0,
                        MF_STRING,
                        99,
                        wide_null("Another external command").as_ptr()
                    ),
                    0
                );
            }
            assert_eq!(GetMenuItemCount(owned.0), 2);
            assert_eq!(
                windows_sys::Win32::UI::WindowsAndMessaging::GetMenuItemID(owned.0, 0),
                4
            );
            assert_eq!(
                windows_sys::Win32::UI::WindowsAndMessaging::GetMenuItemID(owned.0, 1),
                99
            );
        });
    }
}
