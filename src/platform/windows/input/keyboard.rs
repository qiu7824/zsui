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
        self.dispatch_key_down_with_all_modifiers(virtual_key, shift, control, false, false)
    }

    fn dispatch_key_down_with_all_modifiers(
        &mut self,
        virtual_key: u32,
        shift: bool,
        control: bool,
        alt: bool,
        super_key: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(not(feature = "text-input-core"))]
        let _ = (alt, super_key);
        #[cfg(feature = "text-input-core")]
        if let Some(command) =
            windows_text_edit_shortcut(virtual_key, shift, control, alt, super_key)
        {
            let target = self.shared_focused_target();
            let report = self.shared_runtime.dispatch_text_edit_shortcut(command);
            return self.adapt_shared_report(
                report,
                WindowsSharedInputKind::TextEditShortcut { target },
            );
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

#[cfg(feature = "text-input-core")]
fn windows_text_edit_shortcut(
    virtual_key: u32,
    shift: bool,
    control: bool,
    alt: bool,
    super_key: bool,
) -> Option<crate::ZsTextEditCommand> {
    if shift || !control || alt || super_key {
        return None;
    }
    char::from_u32(virtual_key)
        .and_then(crate::native_text_edit::text_edit_command_for_shortcut_character)
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
