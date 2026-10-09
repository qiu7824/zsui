impl WindowsWin32ViewInputRoute {
    fn dispatch_key_down(&mut self, virtual_key: u32) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_modifiers(virtual_key, false, false)
    }

    fn dispatch_key_down_with_shift(
        &mut self,
        virtual_key: u32,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_modifiers(virtual_key, shift, false)
    }

    fn dispatch_key_down_with_modifiers(
        &mut self,
        virtual_key: u32,
        shift: bool,
        control: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_key_down_with_all_modifiers(virtual_key, shift, control, false)
    }

    fn dispatch_key_down_with_all_modifiers(
        &mut self,
        virtual_key: u32,
        shift: bool,
        control: bool,
        alt: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(feature = "shortcuts")]
        if let Some(accelerator) = windows_shortcut_accelerator(virtual_key, shift, control, alt) {
            let report = self.shared_runtime.dispatch_shortcut(accelerator);
            if report.handled {
                return self.adapt_shared_report(report, WindowsSharedInputKind::Shortcut);
            }
        }
        if alt {
            // Alt chords that are not application shortcuts keep their system
            // meaning (menus, Alt+F4) instead of moving carets or selection.
            return WindowsWin32ViewInputDispatchReport {
                hit_target_count: self.hit_target_count(),
                key_down_count: 1,
                unhandled_key_count: 1,
                events: vec![format!("win32_view_key_unhandled:{virtual_key}")],
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        }
        let Some(key) = windows_native_view_key(virtual_key) else {
            return WindowsWin32ViewInputDispatchReport {
                hit_target_count: self.hit_target_count(),
                key_down_count: 1,
                unhandled_key_count: 1,
                events: vec![format!("win32_view_key_unhandled:{virtual_key}")],
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        };
        let target = self.shared_focused_target();
        let report = self
            .shared_runtime
            .dispatch_key_with_modifiers(key, shift, control);
        self.adapt_shared_report(report, WindowsSharedInputKind::Key { key, target })
    }
}

#[cfg(feature = "shortcuts")]
fn windows_shortcut_accelerator(
    virtual_key: u32,
    shift: bool,
    control: bool,
    alt: bool,
) -> Option<crate::ZsAccelerator> {
    use crate::ZsAcceleratorKey as Key;
    let key = match virtual_key {
        0x30..=0x39 | 0x41..=0x5a => Key::Character(char::from_u32(virtual_key)?),
        key if (u32::from(VK_F1)..u32::from(VK_F1) + 24).contains(&key) => {
            Key::Function(u8::try_from(key - u32::from(VK_F1) + 1).ok()?)
        }
        ZSUI_WIN32_VK_RETURN => Key::Enter,
        key if key == u32::from(VK_ESCAPE) => Key::Escape,
        ZSUI_WIN32_VK_TAB => Key::Tab,
        ZSUI_WIN32_VK_SPACE => Key::Space,
        key if key == u32::from(VK_BACK) => Key::Backspace,
        key if key == u32::from(VK_DELETE) => Key::Delete,
        key if key == u32::from(VK_UP) => Key::Up,
        key if key == u32::from(VK_DOWN) => Key::Down,
        key if key == u32::from(VK_LEFT) => Key::Left,
        key if key == u32::from(VK_RIGHT) => Key::Right,
        key if key == u32::from(VK_HOME) => Key::Home,
        key if key == u32::from(VK_END) => Key::End,
        key if key == u32::from(VK_PRIOR) => Key::PageUp,
        key if key == u32::from(VK_NEXT) => Key::PageDown,
        _ => return None,
    };
    let mut accelerator = crate::ZsAccelerator::new(key);
    if control {
        accelerator = crate::ZsAccelerator::primary(key);
    }
    if shift {
        accelerator = accelerator.shifted();
    }
    if alt {
        accelerator = accelerator.with_alt();
    }
    Some(accelerator)
}

fn windows_native_view_key(virtual_key: u32) -> Option<crate::native::NativeViewKey> {
    match virtual_key {
        ZSUI_WIN32_VK_RETURN => Some(crate::native::NativeViewKey::Enter),
        key if key == u32::from(VK_ESCAPE) => Some(crate::native::NativeViewKey::Escape),
        ZSUI_WIN32_VK_TAB => Some(crate::native::NativeViewKey::Tab),
        ZSUI_WIN32_VK_SPACE => Some(crate::native::NativeViewKey::Space),
        key if key == u32::from(VK_UP) => Some(crate::native::NativeViewKey::Up),
        key if key == u32::from(VK_DOWN) => Some(crate::native::NativeViewKey::Down),
        key if key == u32::from(VK_LEFT) => Some(crate::native::NativeViewKey::Left),
        key if key == u32::from(VK_RIGHT) => Some(crate::native::NativeViewKey::Right),
        key if key == u32::from(VK_HOME) => Some(crate::native::NativeViewKey::Home),
        key if key == u32::from(VK_END) => Some(crate::native::NativeViewKey::End),
        key if key == u32::from(VK_PRIOR) => Some(crate::native::NativeViewKey::PageUp),
        key if key == u32::from(VK_NEXT) => Some(crate::native::NativeViewKey::PageDown),
        _ => None,
    }
}
