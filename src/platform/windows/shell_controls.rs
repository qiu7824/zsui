struct NamespaceTreeChild {
    tree: INameSpaceTreeControl,
    container: interop::NativeChildWindow,
    _apartment: OleApartment,
}
impl interop::WindowsNativeChild for NamespaceTreeChild {
    fn window(&self) -> &interop::NativeChildWindow {
        &self.container
    }
    fn layout(
        &mut self,
        placement: crate::NativeHostPlacement,
        _dpi: crate::Dpi,
    ) -> ZsuiResult<()> {
        unsafe {
            let child = windows_sys::Win32::UI::WindowsAndMessaging::GetWindow(
                self.container.hwnd(),
                windows_sys::Win32::UI::WindowsAndMessaging::GW_CHILD,
            );
            if !child.is_null()
                && SetWindowPos(
                    child,
                    null_mut(),
                    0,
                    0,
                    placement.bounds.width.max(1),
                    placement.bounds.height.max(1),
                    SWP_NOACTIVATE | SWP_NOZORDER,
                ) == 0
            {
                return Err(ZsuiError::host(
                    "namespace_tree.layout",
                    "SetWindowPos failed",
                ));
            }
        }
        Ok(())
    }
    fn focus(&mut self) {
        unsafe extern "system" fn focus_tree(hwnd: HWND, _context: LPARAM) -> i32 {
            let mut class = [0u16; 64];
            let length = windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(
                hwnd,
                class.as_mut_ptr(),
                class.len() as i32,
            );
            if String::from_utf16_lossy(&class[..length.max(0) as usize]) == "SysTreeView32" {
                SetFocus(hwnd);
                return 0;
            }
            1
        }
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::EnumChildWindows(
                self.container.hwnd(),
                Some(focus_tree),
                0,
            );
        }
    }
}
impl Drop for NamespaceTreeChild {
    fn drop(&mut self) {
        unsafe {
            let _ = self.tree.RemoveAllRoots();
        }
    }
}

impl interop::WindowsWindowHooks {
    /// Registers a native Shell namespace tree rooted at the supplied folder.
    pub fn namespace_tree(
        self,
        id: crate::NativeHostId,
        folder: impl Into<PathBuf>,
    ) -> ZsuiResult<Self> {
        let folder = folder.into();
        self.child(id, move |parent| {
            #[cfg(feature = "dark-mode")]
            let _appearance = dark_mode_compatibility::AppearanceScope::new(
                windows_platform_palette(
                    window_draw_plan(parent.hwnd())
                        .map(|plan| plan.theme_mode)
                        .unwrap_or_default(),
                )
                .mode,
            );
            let apartment = OleApartment::new()?;
            let container = parent.container()?;
            let tree: INameSpaceTreeControl =
                unsafe { CoCreateInstance(&NamespaceTreeControl, None, CLSCTX_INPROC_SERVER) }
                    .map_err(shell_error)?;
            let child = NamespaceTreeChild {
                tree,
                container,
                _apartment: apartment,
            };
            let name = shell_parsing_name(&folder)?;
            unsafe {
                let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(name.as_ptr()), None)
                    .map_err(shell_error)?;
                child
                    .tree
                    .Initialize(
                        WinHwnd(child.container.hwnd()),
                        &windows::Win32::Foundation::RECT {
                            left: 0,
                            top: 0,
                            right: 1,
                            bottom: 1,
                        },
                        (NSTCS_HASEXPANDOS.0 | NSTCS_HASLINES.0 | NSTCS_SHOWSELECTIONALWAYS.0)
                            as u32,
                    )
                    .map_err(shell_error)?;
                child
                    .tree
                    .AppendRoot(
                        &item,
                        SHCONTF_FOLDERS.0 as u32,
                        NSTCRS_VISIBLE.0 as u32,
                        None,
                    )
                    .map_err(shell_error)?;
            }
            Ok(Box::new(child))
        })
    }
}
