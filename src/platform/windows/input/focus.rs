impl WindowsWin32ViewInputRoute {
    fn dispatch_focus_traversal(
        &mut self,
        offset: isize,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        let Some(target) =
            self.interaction_plan
                .next_focus_target_where(self.focused_widget, offset, |target| {
                    self.widget_accepts_tab_focus(target)
                })
        else {
            report.unhandled_key_count = 1;
            report
                .events
                .push(format!("win32_view_key_focus_unavailable:{offset}"));
            return;
        };

        self.dismiss_popup_overlays_except(Some(target.widget), report);
        self.focus_target(target, report);
        #[cfg(feature = "tooltip")]
        self.show_keyboard_tooltip(target.widget);
        report.focus_traversal_count = 1;
        report.events.push(format!(
            "win32_view_key_focus:{}:{}",
            target.widget.0, offset
        ));
    }

    fn focus_target(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        if self.focused_widget == Some(target.widget) {
            self.ensure_text_edit_for_target(target);
            report.focused_widget = Some(target.widget.0);
            report.text_caret = self.text_edit.map(|state| state.selection.caret);
            return;
        }
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.interaction_plan
                .hit_target_for_widget(*widget)
                .is_some_and(|current| current.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            self.dispatch_event(crate::ViewEvent::NumberBoxCommit { widget }, report);
        }
        self.text_drag = None;
        #[cfg(feature = "password-box")]
        {
            self.password_peek = None;
        }
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        #[cfg(feature = "color-picker")]
        {
            self.color_picker_drag = None;
        }
        self.focused_widget = Some(target.widget);
        self.ensure_text_edit_for_target(target);
        report.focus_count = 1;
        report.focused_widget = Some(target.widget.0);
        report.text_caret = self.text_edit.map(|state| state.selection.caret);
        if self.rebuild_pending_draw_plan() {
            report.focus_visual_count = 1;
            report
                .events
                .push(format!("win32_view_focus_visual:{}", target.widget.0));
        }
        report
            .events
            .push(format!("win32_view_focus:{}", target.widget.0));
    }

    #[cfg(feature = "tooltip")]
    fn show_keyboard_tooltip(&mut self, widget: crate::WidgetId) {
        if self
            .tooltip
            .focus_widget(&self.interaction_plan, widget, std::time::Instant::now())
        {
            self.rebuild_pending_draw_plan();
        }
    }

    fn dispatch_blur(&mut self) -> WindowsWin32ViewInputDispatchReport {
        self.pending_utf16_high_surrogate = None;
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        self.dismiss_popup_overlays_except(None, &mut report);
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(feature = "password-box")]
        {
            self.password_peek = None;
        }
        #[cfg(any(
            feature = "auto-suggest",
            feature = "breadcrumb",
            feature = "color-picker",
            feature = "date-picker",
            feature = "dialog",
            feature = "grid-view",
            feature = "info-bar",
            feature = "teaching-tip",
            feature = "password-box",
            feature = "tabs",
            feature = "time-picker",
            feature = "toast",
            feature = "toggle-button",
            feature = "table",
            feature = "tree"
        ))]
        self.update_pointer_visual_state(None, None, &mut report);
        #[cfg(feature = "number-box")]
        if let Some(widget) = self.focused_widget.filter(|widget| {
            self.interaction_plan
                .hit_target_for_widget(*widget)
                .is_some_and(|target| target.kind == crate::ViewHitTargetKind::NumberBox)
        }) {
            self.dispatch_event(crate::ViewEvent::NumberBoxCommit { widget }, &mut report);
        }
        let Some(widget) = self.focused_widget.take() else {
            return report;
        };
        self.text_edit = None;
        self.text_drag = None;
        #[cfg(feature = "combo")]
        self.combo_type_ahead.reset();
        #[cfg(feature = "slider")]
        {
            self.slider_drag = None;
        }
        #[cfg(feature = "color-picker")]
        {
            self.color_picker_drag = None;
        }
        if self.rebuild_pending_draw_plan() {
            report.focus_visual_count = 1;
        }
        report
            .events
            .push(format!("win32_view_focus_visual_cleared:{}", widget.0));
        report
    }

    fn focused_target(&self) -> Option<crate::ViewHitTarget> {
        self.focused_widget
            .and_then(|widget| self.interaction_plan.hit_target_for_widget(widget))
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn focused_text_accessibility_snapshot(
        &self,
    ) -> Option<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        let target = self.focused_target()?;
        if !target.kind.accepts_text_input() {
            return None;
        }
        let value = self.widget_display_text_value(target.widget)?;
        let state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let caret = native_text_visual_geometry_in_viewport_with_backend(
            visual_target,
            &value,
            state.selection,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        )
        .caret;
        crate::native_accessibility::NativeTextAccessibilitySnapshot::new(
            target,
            value,
            state.selection,
            caret,
        )
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_range_rectangles(
        &self,
        widget: crate::WidgetId,
        selection: crate::native_text_edit::NativeTextSelection,
    ) -> Option<Vec<crate::Rect>> {
        let target = self.focused_target()?;
        if target.widget != widget || !target.kind.accepts_text_input() {
            return None;
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return None;
        }
        let value = self.widget_text_value(widget)?;
        let state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let selection = selection.clamp(&value);
        let geometry = native_text_visual_geometry_in_viewport_with_backend(
            visual_target,
            &value,
            selection,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(widget),
            self.dpi,
            &self.text_shaping,
        );
        if selection.is_collapsed() {
            Some(vec![geometry.caret])
        } else {
            Some(geometry.selections)
        }
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_visible_range(
        &self,
    ) -> Option<(crate::WidgetId, std::ops::Range<usize>)> {
        let target = self.focused_target()?;
        if !target.kind.accepts_text_input() {
            return None;
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return None;
        }
        let value = self.widget_text_value(target.widget)?;
        let state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        Some((
            target.widget,
            native_text_visible_range_with_backend(
                native_text_visual_target(target, &self.interaction_plan),
                &value,
                state.first_visible_visual_row,
                self.widget_text_wrap(target.widget),
                self.dpi,
                &self.text_shaping,
            ),
        ))
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn text_accessibility_index_for_point(
        &self,
        point: crate::Point,
    ) -> Option<(crate::WidgetId, usize)> {
        let target = self.focused_target()?;
        if !target.kind.accepts_text_input() {
            return None;
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return None;
        }
        let value = self.widget_text_value(target.widget)?;
        let state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let index = native_text_index_for_point_in_viewport_with_backend(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        Some((target.widget, index))
    }

    fn ensure_text_edit_for_target(&mut self, target: crate::ViewHitTarget) {
        if !target.kind.accepts_text_input() {
            self.text_drag = None;
            return;
        }
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        state.clamp(&value);
        if target.kind != crate::ViewHitTargetKind::TextEditor
            || self.widget_text_wrap(target.widget) != crate::TextWrap::NoWrap
        {
            state.horizontal_scroll_px = 0;
        }
        self.text_edit = Some(state);
    }

}
