#[derive(Debug)]
pub struct WindowsWin32OwnedAcceleratorTable {
    handle: HACCEL,
    entry_count: usize,
}

impl WindowsWin32OwnedAcceleratorTable {
    fn from_command_table(table: &WindowsWin32StatusMenuCommandTable) -> ZsuiResult<Option<Self>> {
        let mut bindings = Vec::new();
        for command in table.entries().iter().filter(|entry| entry.enabled) {
            let Some(accelerator) = command.accelerator.as_ref() else {
                continue;
            };
            let cmd = u16::try_from(command.native_id).map_err(|_| {
                ZsuiError::invalid_spec(
                    "menu.accelerator",
                    "Win32 menu command id does not fit an accelerator table",
                )
            })?;
            bindings.push((cmd, *accelerator));
        }
        if bindings.is_empty() {
            Ok(None)
        } else {
            Self::from_bindings(&bindings).map(Some)
        }
    }

    pub fn from_bindings(bindings: &[(u16, ZsAccelerator)]) -> ZsuiResult<Self> {
        if bindings.is_empty() {
            return Err(ZsuiError::invalid_spec(
                "accelerator.bindings",
                "Win32 accelerator bindings cannot be empty",
            ));
        }
        let mut entries = Vec::with_capacity(bindings.len());
        for (command, accelerator) in bindings {
            accelerator.validate()?;
            let flags = windows_accelerator_flags(accelerator);
            let key = windows_accelerator_virtual_key(accelerator)?;
            if entries
                .iter()
                .any(|entry: &ACCEL| entry.fVirt == flags && entry.key == key)
            {
                return Err(ZsuiError::invalid_spec(
                    "accelerator.bindings",
                    format!("duplicate accelerator `{accelerator}`"),
                ));
            }
            entries.push(ACCEL {
                fVirt: flags,
                key,
                cmd: *command,
            });
        }
        let handle = unsafe { CreateAcceleratorTableW(entries.as_ptr(), entries.len() as i32) };
        if handle.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_create_accelerator_table",
                "CreateAcceleratorTableW failed",
            ));
        }
        Ok(Self {
            handle,
            entry_count: entries.len(),
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub fn translate(&self, window: HWND, message: &MSG) -> bool {
        unsafe { TranslateAcceleratorW(window, self.handle, message) != 0 }
    }
}

fn windows_accelerator_flags(accelerator: &ZsAccelerator) -> u8 {
    let mut flags = FVIRTKEY;
    if accelerator.uses_primary() || accelerator.uses_super() {
        flags |= FCONTROL;
    }
    if accelerator.uses_alt() {
        flags |= FALT;
    }
    if accelerator.uses_shift() {
        flags |= FSHIFT;
    }
    flags
}

fn windows_accelerator_virtual_key(accelerator: &ZsAccelerator) -> ZsuiResult<u16> {
    accelerator.validate()?;
    let key = match accelerator.key() {
        ZsAcceleratorKey::Character(key) => key.to_ascii_uppercase() as u16,
        ZsAcceleratorKey::Enter => VK_RETURN,
        ZsAcceleratorKey::Tab => VK_TAB,
        ZsAcceleratorKey::Escape => VK_ESCAPE,
        ZsAcceleratorKey::Space => VK_SPACE,
        ZsAcceleratorKey::Backspace => VK_BACK,
        ZsAcceleratorKey::Delete => VK_DELETE,
        ZsAcceleratorKey::Up => VK_UP,
        ZsAcceleratorKey::Down => VK_DOWN,
        ZsAcceleratorKey::Left => VK_LEFT,
        ZsAcceleratorKey::Right => VK_RIGHT,
        ZsAcceleratorKey::Home => VK_HOME,
        ZsAcceleratorKey::End => VK_END,
        ZsAcceleratorKey::PageUp => VK_PRIOR,
        ZsAcceleratorKey::PageDown => VK_NEXT,
        ZsAcceleratorKey::Function(number) => VK_F1 + u16::from(number) - 1,
    };
    Ok(key)
}

impl Drop for WindowsWin32OwnedAcceleratorTable {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                DestroyAcceleratorTable(self.handle);
            }
        }
    }
}

