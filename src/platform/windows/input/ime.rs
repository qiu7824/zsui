impl WindowsWin32ViewInputRoute {
    fn dispatch_text_input(&mut self, text: &str) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_text_input_at(text, std::time::Instant::now())
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_set_text_value(
        &mut self,
        text: &str,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some(target) = self.focused_target() else {
            return WindowsWin32ViewInputDispatchReport::default();
        };
        if !target.kind.accepts_text_input() {
            return WindowsWin32ViewInputDispatchReport::default();
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return WindowsWin32ViewInputDispatchReport::default();
        }
        let current = self.widget_text_value(target.widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &current));
        state.selection = crate::native_text_edit::NativeTextSelection {
            anchor: 0,
            caret: current.chars().count(),
        };
        self.text_edit = Some(state);
        self.dispatch_text_input(text)
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_set_text_selection(
        &mut self,
        selection: crate::native_text_edit::NativeTextSelection,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport::default();
        let Some(target) = self.focused_target() else {
            return report;
        };
        if !target.kind.accepts_text_input() {
            return report;
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return report;
        }
        let value = self.widget_text_value(target.widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == target.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
        let previous = state.selection;
        state.selection = selection.clamp(&value);
        state.preferred_visual_x = None;
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        state.first_visible_visual_row = native_text_first_visible_row_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            visual_target,
            &value,
            state.selection.caret,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        self.text_edit = Some(state);
        report.handled = true;
        report.text_selection_change_count = usize::from(previous != state.selection);
        report.text_caret = Some(state.selection.caret);
        if previous != state.selection {
            #[cfg(feature = "textbox")]
            if matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            ) {
                self.dispatch_event(
                    crate::ViewEvent::TextSelectionChanged {
                        widget: target.widget,
                        selection: state.selection.into(),
                    },
                    &mut report,
                );
            }
            self.rebuild_pending_draw_plan();
        }
        report
    }

    #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
    fn dispatch_accessibility_scroll_text_range(
        &mut self,
        widget: crate::WidgetId,
        selection: crate::native_text_edit::NativeTextSelection,
        align_to_top: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport::default();
        let Some(target) = self.focused_target() else {
            return report;
        };
        if target.widget != widget || !target.kind.accepts_text_input() {
            return report;
        }
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBox {
            return report;
        }
        let value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        let previous_row = state.first_visible_visual_row;
        let previous_horizontal_scroll = state.horizontal_scroll_px;
        let selection = selection.clamp(&value);
        let (start, end) = selection.ordered();
        let index = if align_to_top { start } else { end };
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        state.first_visible_visual_row =
            native_text_first_visible_row_for_index_alignment_with_backend(
                visual_target,
                &value,
                index,
                align_to_top,
                self.widget_text_wrap(widget),
                self.dpi,
                &self.text_shaping,
            );
        state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            visual_target,
            &value,
            index,
            state.horizontal_scroll_px,
            self.widget_text_wrap(widget),
            self.dpi,
            &self.text_shaping,
        );
        self.text_edit = Some(state);
        report.handled = true;
        report.events.push(format!(
            "win32_view_accessibility_text_scroll:{}:{}:{}",
            widget.0, state.first_visible_visual_row, state.horizontal_scroll_px
        ));
        if state.first_visible_visual_row != previous_row
            || state.horizontal_scroll_px != previous_horizontal_scroll
        {
            self.rebuild_pending_draw_plan();
        }
        report
    }

    fn dispatch_utf16_input_unit(&mut self, unit: u16) -> WindowsWin32ViewInputDispatchReport {
        if (0xd800..=0xdbff).contains(&unit) {
            self.pending_utf16_high_surrogate = Some(unit);
            return WindowsWin32ViewInputDispatchReport {
                handled: true,
                hit_target_count: self.hit_target_count(),
                events: vec!["win32_view_text_utf16_high_surrogate".to_string()],
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        }
        let text = if (0xdc00..=0xdfff).contains(&unit) {
            self.pending_utf16_high_surrogate
                .take()
                .and_then(|high| char::decode_utf16([high, unit]).next())
                .and_then(Result::ok)
                .map(|character| character.to_string())
        } else {
            self.pending_utf16_high_surrogate = None;
            text_from_char_wparam(unit as usize)
        };
        match text {
            Some(text) => self.dispatch_text_input(&text),
            None => WindowsWin32ViewInputDispatchReport {
                hit_target_count: self.hit_target_count(),
                events: vec!["win32_view_text_invalid_utf16_unit".to_string()],
                ..WindowsWin32ViewInputDispatchReport::default()
            },
        }
    }

    fn dispatch_text_input_at(
        &mut self,
        text: &str,
        _now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(widget) = self.focused_widget else {
            report
                .events
                .push("win32_view_text_without_focus".to_string());
            return report;
        };
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            report
                .events
                .push(format!("win32_view_text_without_target:{}", widget.0));
            return report;
        };
        #[cfg(feature = "dialog")]
        if self
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| state.open)
        {
            report.handled = true;
            report.events.push(format!(
                "win32_view_content_dialog_text_suppressed:{}",
                widget.0
            ));
            return report;
        }
        #[cfg(feature = "toast")]
        if self.widget_toast_state(widget).is_some() {
            report.handled = true;
            report
                .events
                .push(format!("win32_view_toast_text_suppressed:{}", widget.0));
            return report;
        }
        #[cfg(feature = "info-bar")]
        if self.widget_info_bar_state(widget).is_some() {
            report.handled = true;
            report
                .events
                .push(format!("win32_view_info_bar_text_suppressed:{}", widget.0));
            return report;
        }
        #[cfg(feature = "teaching-tip")]
        if self.widget_teaching_tip_state(widget).is_some() {
            report.handled = true;
            report.events.push(format!(
                "win32_view_teaching_tip_text_suppressed:{}",
                widget.0
            ));
            return report;
        }
        #[cfg(feature = "combo")]
        if target.kind == crate::ViewHitTargetKind::ComboBox {
            let Some(query) = self.combo_type_ahead.push_text(widget, text, _now) else {
                return report;
            };
            report.handled = true;
            let Some((selected, option_count, expanded)) = self.widget_combo_state(widget) else {
                self.combo_type_ahead.reset();
                return report;
            };
            let start_after = query.match_start_after(selected, option_count);
            let Some(index) = self.widget_combo_type_ahead_match(widget, &query.text, start_after)
            else {
                report.events.push(format!(
                    "win32_view_combo_type_ahead_no_match:{}:{}",
                    widget.0, query.text
                ));
                return report;
            };
            report.combo_type_ahead_match_count = 1;
            report.events.push(format!(
                "win32_view_combo_type_ahead_match:{}:{}:{index}",
                widget.0, query.text
            ));
            if selected == Some(index) {
                return report;
            }
            report.combo_selection_count = 1;
            report.combo_keyboard_selection_count = 1;
            report.combo_expanded_change_count = usize::from(expanded);
            report.event_count = 1;
            self.dispatch_event(
                crate::ViewEvent::ComboBoxSelected { widget, index },
                &mut report,
            );
            return report;
        }
        if !target.kind.accepts_text_input() {
            report.events.push(format!(
                "win32_view_text_without_textbox_focus:{}",
                widget.0
            ));
            return report;
        }

        #[cfg(feature = "password-box")]
        let mut password = (target.kind == crate::ViewHitTargetKind::PasswordBox)
            .then(|| self.widget_password_value(widget).unwrap_or_default());
        #[cfg(feature = "password-box")]
        let mut value = zeroize::Zeroizing::new(
            password
                .as_ref()
                .map(|password| password.as_str().to_owned())
                .unwrap_or_else(|| self.widget_text_value(widget).unwrap_or_default()),
        );
        #[cfg(not(feature = "password-box"))]
        let mut value = self.widget_text_value(widget).unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
        state.clamp(&value);
        #[cfg(feature = "textbox")]
        let history_before = matches!(
            target.kind,
            crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
        )
        .then(|| (value.as_str().to_owned(), state.selection));
        let multiline = target.kind == crate::ViewHitTargetKind::TextEditor;
        let mut previous_was_carriage_return = false;
        let accepted = text
            .chars()
            .filter(|ch| {
                let accepted = matches!(*ch, '\u{8}' | '\u{7f}')
                    || (multiline
                        && (*ch == '\r' || (*ch == '\n' && !previous_was_carriage_return)))
                    || !ch.is_control();
                previous_was_carriage_return = *ch == '\r';
                accepted
            })
            .count();
        let edit = apply_text_input(&mut value, &mut state.selection, text, multiline);

        if !edit.handled {
            return report;
        }

        state.preferred_visual_x = None;
        state.first_visible_visual_row = native_text_first_visible_row_for_caret_with_backend(
            target,
            &value,
            state.selection.caret,
            state.first_visible_visual_row,
            self.widget_text_wrap(widget),
            self.dpi,
            &self.text_shaping,
        );
        state.horizontal_scroll_px = native_text_horizontal_scroll_for_caret_with_backend(
            target,
            &value,
            state.selection.caret,
            state.horizontal_scroll_px,
            self.widget_text_wrap(widget),
            self.dpi,
            &self.text_shaping,
        );
        self.text_edit = Some(state);
        #[cfg(feature = "textbox")]
        if edit.text_changed {
            if let Some((before_value, before_selection)) = history_before {
                self.text_history.record_text_change(
                    widget,
                    &before_value,
                    before_selection,
                    value.as_str(),
                );
            }
        }
        report.text_input_count = accepted;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report
            .events
            .push(format!("win32_view_text_changed:{}", widget.0));
        if edit.text_changed {
            report.event_count = 1;
            #[cfg(feature = "command-palette")]
            if target.kind == crate::ViewHitTargetKind::CommandPalette {
                report.command_palette_query_change_count = 1;
            }
            #[cfg(feature = "auto-suggest")]
            if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
                report.auto_suggest_expanded_change_count = usize::from(
                    self.widget_auto_suggest_state(widget)
                        .is_some_and(|state| !state.expanded),
                );
            }
            #[cfg(feature = "password-box")]
            if let Some(password) = &mut password {
                *password.as_string_mut() = std::mem::take(&mut *value);
                self.dispatch_event(
                    crate::ViewEvent::PasswordChanged {
                        widget,
                        value: password.clone(),
                    },
                    &mut report,
                );
                return report;
            }
            #[cfg(feature = "password-box")]
            let value = std::mem::take(&mut *value);
            #[cfg(feature = "textbox")]
            if edit.selection_changed
                && matches!(
                    target.kind,
                    crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
                )
            {
                self.dispatch_event(
                    crate::ViewEvent::TextEdited {
                        widget,
                        value,
                        selection: state.selection.into(),
                    },
                    &mut report,
                );
                return report;
            }
            self.dispatch_event(crate::ViewEvent::TextChanged { widget, value }, &mut report);
        } else if edit.selection_changed {
            #[cfg(feature = "textbox")]
            if matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            ) {
                self.dispatch_event(
                    crate::ViewEvent::TextSelectionChanged {
                        widget,
                        selection: state.selection.into(),
                    },
                    &mut report,
                );
            }
            self.rebuild_pending_draw_plan();
        }
        report
    }

}
