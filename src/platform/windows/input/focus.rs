impl WindowsWin32ViewInputRoute {
    fn dispatch_blur(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.blur_focus();
        self.adapt_shared_report(report, WindowsSharedInputKind::Blur)
    }

    fn focused_target(&self) -> Option<crate::ViewHitTarget> {
        self.shared_focused_target()
    }

    #[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
    fn dispatch_accessibility_menu_flyout_focus(
        &mut self,
        path: crate::ZsMenuFlyoutPath,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self
            .shared_runtime
            .dispatch_accessibility_menu_flyout_focus(path);
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
    fn dispatch_accessibility_menu_flyout_invoke(
        &mut self,
        path: crate::ZsMenuFlyoutPath,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self
            .shared_runtime
            .dispatch_accessibility_menu_flyout_invoke(path);
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "menu-flyout"))]
    fn dispatch_accessibility_menu_flyout_expanded(
        &mut self,
        path: crate::ZsMenuFlyoutPath,
        expanded: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self
            .shared_runtime
            .dispatch_accessibility_menu_flyout_expanded(path, expanded);
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_set_text_value(
        &mut self,
        text: &str,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some(widget) = self.shared_runtime.focused_widget() else {
            return WindowsWin32ViewInputDispatchReport::default();
        };
        let report = self
            .shared_runtime
            .dispatch_accessibility_set_value(widget, text);
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_set_text_selection(
        &mut self,
        selection: crate::native_text_edit::NativeTextSelection,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self
            .shared_runtime
            .dispatch_accessibility_set_selection(selection);
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_scroll_text_range(
        &mut self,
        widget: crate::WidgetId,
        selection: crate::native_text_edit::NativeTextSelection,
        align_to_top: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let report = self.shared_runtime.dispatch_accessibility_scroll_text_range(
            widget,
            selection,
            align_to_top,
        );
        self.adapt_shared_report(report, WindowsSharedInputKind::Accessibility)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn focused_text_accessibility_snapshot(
        &self,
    ) -> Option<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        self.shared_runtime.focused_text_accessibility_snapshot()
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_range_rectangles(
        &self,
        widget: crate::WidgetId,
        selection: crate::native_text_edit::NativeTextSelection,
    ) -> Option<Vec<crate::Rect>> {
        self.shared_runtime
            .text_accessibility_range_rectangles(widget, selection)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_visible_range(
        &self,
    ) -> Option<(crate::WidgetId, std::ops::Range<usize>)> {
        self.shared_runtime.text_accessibility_visible_range()
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_index_for_point(
        &self,
        point: crate::Point,
    ) -> Option<(crate::WidgetId, usize)> {
        self.shared_runtime
            .text_accessibility_index_for_point(point)
    }
}
