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
