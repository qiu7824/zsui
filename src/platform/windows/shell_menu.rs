struct ShellMenuHook {
    third: Option<IContextMenu3>,
    second: Option<IContextMenu2>,
}
impl menu_bridge::NativeMenuHook for ShellMenuHook {
    fn handle_message(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        unsafe {
            let wp = windows::Win32::Foundation::WPARAM(wparam);
            let lp = windows::Win32::Foundation::LPARAM(lparam);
            if let Some(menu) = &self.third {
                let mut result = windows::Win32::Foundation::LRESULT(0);
                if menu
                    .HandleMenuMsg2(message, wp, lp, Some(&mut result))
                    .is_ok()
                {
                    return Some(result.0);
                }
            }
            if let Some(menu) = &self.second {
                if menu.HandleMenuMsg(message, wp, lp).is_ok() {
                    return Some(0);
                }
            }
            None
        }
    }
}

pub(super) fn context_menu(
    hwnd: HWND,
    spec: &crate::ShellMenuSpec,
) -> ZsuiResult<crate::ShellMenuOutcome> {
    #[cfg(feature = "dark-mode")]
    let _appearance = dark_mode_compatibility::AppearanceScope::new(
        windows_platform_palette(
            window_draw_plan(hwnd)
                .map(|plan| plan.theme_mode)
                .unwrap_or_default(),
        )
        .mode,
    );
    let _apartment = OleApartment::new()?;
    let context: IContextMenu =
        unsafe { shell_items(&spec.items)?.BindToHandler(None, &BHID_SFUIObject) }
            .map_err(shell_error)?;
    struct OwnedMenu(HMENU);
    impl Drop for OwnedMenu {
        fn drop(&mut self) {
            unsafe {
                DestroyMenu(self.0);
            }
        }
    }
    let menu = OwnedMenu(unsafe { windows_sys::Win32::UI::WindowsAndMessaging::CreatePopupMenu() });
    if menu.0.is_null() {
        return Err(ZsuiError::host("shell_menu", "CreatePopupMenu failed"));
    }
    unsafe {
        context
            .QueryContextMenu(
                windows::Win32::UI::WindowsAndMessaging::HMENU(menu.0),
                0,
                1,
                0x7fff,
                CMF_NORMAL,
            )
            .ok()
            .map_err(shell_error)?;
    }
    let mut borrowed = unsafe { menu_bridge::NativeMenu::from_handle(menu.0)? };
    let position = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetMenuItemCount(menu.0) }
        .max(0) as u32;
    let inserted = spec
        .application
        .merge_into(&mut borrowed, position, 0x8000)?;
    let hook = Rc::new(ShellMenuHook {
        third: context.cast().ok(),
        second: context.cast().ok(),
    });
    let selected = unsafe { inserted.track(hwnd, spec.position, hook)? };
    if selected == 0 {
        return Ok(crate::ShellMenuOutcome::Cancelled);
    }
    if let Some(command) = inserted.command(selected) {
        return Ok(crate::ShellMenuOutcome::Application(command.clone()));
    }
    if selected >= 0x8000 {
        return Err(ZsuiError::host(
            "shell_menu",
            "native menu selected an unknown application command",
        ));
    }
    let index = (selected - 1) as usize;
    let invoke = CMINVOKECOMMANDINFOEX {
        cbSize: size_of::<CMINVOKECOMMANDINFOEX>() as u32,
        fMask: SEE_MASK_UNICODE,
        hwnd: WinHwnd(hwnd),
        lpVerb: windows_core::PCSTR(index as *const u8),
        lpVerbW: PCWSTR(index as *const u16),
        nShow: windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL.0,
        ..Default::default()
    };
    unsafe {
        context
            .InvokeCommand((&invoke as *const CMINVOKECOMMANDINFOEX).cast())
            .map_err(shell_error)?;
    }
    Ok(crate::ShellMenuOutcome::ShellInvoked)
}

pub(super) fn execute_effect(hwnd: HWND, effect: &crate::AppEffect) -> crate::AppEffectOutcome {
    match effect {
        crate::AppEffect::StartDrag { payload, effects } => {
            crate::AppEffectOutcome::StartDrag(start_drag(hwnd, payload, *effects))
        }
        crate::AppEffect::ShellMenu(spec) => {
            crate::AppEffectOutcome::ShellMenu(context_menu(hwnd, spec))
        }
        _ => crate::app_effect::execute_native_app_effect(effect),
    }
}
