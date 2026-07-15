impl WindowsWin32ViewInputRoute {
    fn dispatch_click(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            click_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        let target = self.interaction_plan.hit_target_at(point);
        self.dismiss_popup_overlays_except(target.map(|target| target.widget), &mut report);
        let Some(target) = target else {
            if !report.handled {
                report.unhandled_click_count = 1;
                report
                    .events
                    .push(format!("win32_view_click_missed:{}:{}", point.x, point.y));
            }
            return report;
        };

        report.handled = true;
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBoxReveal {
            return report;
        }
        #[cfg(feature = "combo")]
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::ComboBox | crate::ViewHitTargetKind::ComboBoxOption { .. }
        ) {
            self.combo_type_ahead.reset();
        }
        self.focus_target(target, &mut report);
        if target.kind.accepts_text_input() {
            return report;
        }

        self.dispatch_activation(target, &mut report);
        report
    }

    fn dispatch_pointer_down(
        &mut self,
        point: crate::Point,
        shift: bool,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            pointer_down_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        let target = self.interaction_plan.hit_target_at(point);
        self.dismiss_popup_overlays_except(target.map(|target| target.widget), &mut report);
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
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
        self.update_pointer_visual_state(
            target.and_then(native_pointer_visual_key),
            target.and_then(native_pointer_visual_key),
            &mut report,
        );
        let Some(target) = target else {
            return report;
        };
        #[cfg(feature = "password-box")]
        if target.kind == crate::ViewHitTargetKind::PasswordBoxReveal {
            self.text_drag = None;
            self.password_peek = Some(target.widget);
            report.handled = true;
            self.rebuild_pending_draw_plan();
            return report;
        }
        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            self.text_drag = None;
            self.focus_target(target, &mut report);
            self.slider_drag = Some(target.widget);
            report.slider_drag_active = true;
            return self.dispatch_slider_pointer(target, point, report);
        }
        #[cfg(feature = "color-picker")]
        if matches!(
            target.kind,
            crate::ViewHitTargetKind::ColorPickerSpectrum
                | crate::ViewHitTargetKind::ColorPickerHue
                | crate::ViewHitTargetKind::ColorPickerChannel { .. }
        ) {
            self.text_drag = None;
            self.focus_target(target, &mut report);
            self.color_picker_drag = Some((target.widget, target.kind));
            report.color_picker_drag_active = true;
            return self.dispatch_color_picker_pointer(target, point, report);
        }
        if !target.kind.accepts_text_input() {
            self.text_drag = None;
            #[cfg(feature = "slider")]
            {
                self.slider_drag = None;
            }
            #[cfg(feature = "color-picker")]
            {
                self.color_picker_drag = None;
            }
            return report;
        }

        self.focus_target(target, &mut report);
        let value = self
            .widget_display_text_value(target.widget)
            .unwrap_or_default();
        let mut state = self
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
        let anchor = if shift { state.selection.anchor } else { index };
        let edit = set_pointer_selection(&value, &mut state.selection, anchor, index);
        state.preferred_visual_x = None;
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
        self.text_drag = Some(NativeTextDragState {
            widget: target.widget,
            anchor,
        });
        report.handled = true;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report.text_drag_active = true;
        report.events.push(format!(
            "win32_view_text_pointer_down:{}:{}",
            target.widget.0, index
        ));
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            self.dispatch_event(
                crate::ViewEvent::TextSelectionChanged {
                    widget: target.widget,
                    selection: state.selection.into(),
                },
                &mut report,
            );
        }
        self.rebuild_pending_draw_plan();
        report
    }

    fn dispatch_pointer_move(
        &mut self,
        point: crate::Point,
    ) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_pointer_move_at(point, std::time::Instant::now())
    }

    fn dispatch_pointer_move_at(
        &mut self,
        point: crate::Point,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self
            .tooltip
            .pointer_moved(&self.interaction_plan, point, now)
        {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(not(feature = "tooltip"))]
        let _ = now;
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
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
        {
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, self.pointer_pressed, &mut report);
        }
        #[cfg(feature = "password-box")]
        if let Some(widget) = self.password_peek {
            let still_peeking = self
                .interaction_plan
                .hit_target_at(point)
                .is_some_and(|target| {
                    target.widget == widget
                        && target.kind == crate::ViewHitTargetKind::PasswordBoxReveal
                });
            if !still_peeking {
                self.password_peek = None;
                report.handled = true;
                self.rebuild_pending_draw_plan();
            }
            return report;
        }
        let Some(drag) = self.text_drag else {
            #[cfg(feature = "color-picker")]
            if let Some((widget, kind)) = self.color_picker_drag {
                if let Some(target) = self
                    .interaction_plan
                    .hit_targets
                    .iter()
                    .copied()
                    .find(|target| target.widget == widget && target.kind == kind)
                {
                    report.pointer_move_count = 1;
                    report.color_picker_drag_active = true;
                    return self.dispatch_color_picker_pointer(target, point, report);
                }
                self.color_picker_drag = None;
            }
            #[cfg(feature = "slider")]
            if let Some(widget) = self.slider_drag {
                if let Some(target) = self.interaction_plan.hit_target_for_widget(widget) {
                    report.pointer_move_count = 1;
                    report.slider_drag_active = true;
                    return self.dispatch_slider_pointer(target, point, report);
                }
                self.slider_drag = None;
            }
            return report;
        };
        let Some(target) = self.interaction_plan.hit_target_for_widget(drag.widget) else {
            self.text_drag = None;
            return report;
        };
        let value = self
            .widget_display_text_value(drag.widget)
            .unwrap_or_default();
        let mut state = self
            .text_edit
            .filter(|state| state.widget == drag.widget)
            .unwrap_or_else(|| NativeTextEditState::at_end(drag.widget, &value));
        let visual_target = native_text_visual_target(target, &self.interaction_plan);
        let drag_viewport = native_text_drag_viewport_for_point_with_backend(
            visual_target,
            &value,
            point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        state.first_visible_visual_row = drag_viewport.first_visible_row;
        state.horizontal_scroll_px = drag_viewport.horizontal_scroll_px;
        let index = native_text_index_for_point_in_viewport_with_backend(
            visual_target,
            &value,
            drag_viewport.point,
            state.first_visible_visual_row,
            state.horizontal_scroll_px,
            self.widget_text_wrap(target.widget),
            self.dpi,
            &self.text_shaping,
        );
        let edit = set_pointer_selection(&value, &mut state.selection, drag.anchor, index);
        state.preferred_visual_x = None;
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
        report.pointer_move_count = 1;
        report.text_selection_change_count = usize::from(edit.selection_changed);
        report.text_caret = Some(state.selection.caret);
        report.text_drag_active = true;
        report.text_drag_scroll_count = usize::from(drag_viewport.scrolled);
        if edit.selection_changed || drag_viewport.scrolled {
            self.rebuild_pending_draw_plan();
        }
        report.events.push(format!(
            "win32_view_text_pointer_move:{}:{}",
            drag.widget.0, index
        ));
        if drag_viewport.scrolled {
            report.events.push(format!(
                "win32_view_text_drag_scroll:{}:{}:{}",
                drag.widget.0, state.first_visible_visual_row, state.horizontal_scroll_px
            ));
        }
        #[cfg(feature = "textbox")]
        if edit.selection_changed
            && matches!(
                target.kind,
                crate::ViewHitTargetKind::Textbox | crate::ViewHitTargetKind::TextEditor
            )
        {
            self.dispatch_event(
                crate::ViewEvent::TextSelectionChanged {
                    widget: drag.widget,
                    selection: state.selection.into(),
                },
                &mut report,
            );
        }
        report
    }

    fn dispatch_pointer_up(&mut self, point: crate::Point) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        if self.password_peek.take().is_some() {
            let mut report = WindowsWin32ViewInputDispatchReport {
                handled: true,
                hit_target_count: self.hit_target_count(),
                pointer_up_count: 1,
                ..WindowsWin32ViewInputDispatchReport::default()
            };
            self.rebuild_pending_draw_plan();
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, None, &mut report);
            return report;
        }
        if self.text_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            let completed_selection = self
                .text_edit
                .is_some_and(|state| !state.selection.is_collapsed());
            self.text_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.text_drag_count = usize::from(completed_selection);
            report.text_drag_active = false;
            report.events.push("win32_view_text_pointer_up".to_string());
            #[cfg(any(
                feature = "auto-suggest",
                feature = "button",
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "slider")]
        if self.slider_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            self.slider_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.slider_drag_count = 1;
            report.slider_drag_active = false;
            report
                .events
                .push("win32_view_slider_pointer_up".to_string());
            #[cfg(any(
                feature = "auto-suggest",
                feature = "button",
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
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        #[cfg(feature = "color-picker")]
        if self.color_picker_drag.is_some() {
            let mut report = self.dispatch_pointer_move(point);
            report.pointer_move_count = 0;
            self.color_picker_drag = None;
            report.handled = true;
            report.pointer_up_count = 1;
            report.color_picker_drag_count = 1;
            report.color_picker_drag_active = false;
            report
                .events
                .push("win32_view_color_picker_pointer_up".to_string());
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            return report;
        }
        let mut report = self.dispatch_click(point);
        report.pointer_up_count = 1;
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
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
        {
            let hovered = self
                .interaction_plan
                .hit_target_at(point)
                .and_then(native_pointer_visual_key);
            self.update_pointer_visual_state(hovered, None, &mut report);
        }
        report
    }

    fn cancel_pointer_drag(&mut self) -> WindowsWin32ViewInputDispatchReport {
        let had_drag = self.text_drag.take().is_some();
        #[cfg(feature = "password-box")]
        let had_drag = had_drag | self.password_peek.take().is_some();
        #[cfg(feature = "slider")]
        let had_drag = had_drag | self.slider_drag.take().is_some();
        #[cfg(feature = "color-picker")]
        let had_drag = had_drag | self.color_picker_drag.take().is_some();
        let report = WindowsWin32ViewInputDispatchReport {
            handled: had_drag,
            hit_target_count: self.hit_target_count(),
            events: had_drag
                .then(|| "win32_view_text_pointer_cancel".to_string())
                .into_iter()
                .collect(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
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
        {
            let mut report = report;
            self.update_pointer_visual_state(self.pointer_hover, None, &mut report);
            report
        }
        #[cfg(not(any(
            feature = "auto-suggest",
            feature = "button",
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
        )))]
        {
            report
        }
    }

    fn dispatch_pointer_leave(&mut self) -> WindowsWin32ViewInputDispatchReport {
        #[cfg(feature = "password-box")]
        let had_password_peek = self.password_peek.take().is_some();
        #[allow(unused_mut)]
        let mut report = WindowsWin32ViewInputDispatchReport {
            #[cfg(feature = "password-box")]
            handled: had_password_peek,
            hit_target_count: self.hit_target_count(),
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(any(
            feature = "auto-suggest",
            feature = "button",
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
        {
            self.update_pointer_visual_state(None, None, &mut report);
            report
        }
        #[cfg(not(any(
            feature = "auto-suggest",
            feature = "button",
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
        )))]
        {
            report
        }
    }

    #[cfg(feature = "slider")]
    fn dispatch_slider_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: crate::Point,
        mut report: WindowsWin32ViewInputDispatchReport,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some((current, range)) = self.widget_slider_state(target.widget) else {
            self.slider_drag = None;
            return report;
        };
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, self.dpi).track;
        let fraction = point.x.saturating_sub(track.x) as f32 / track.width.max(1) as f32;
        let value = range.value_at_fraction(fraction);
        report.handled = true;
        report.slider_drag_active = self.slider_drag.is_some();
        if (value - current).abs() <= f32::EPSILON {
            return report;
        }
        report.slider_value_change_count = 1;
        report.events.push(format!(
            "win32_view_slider_changed:{}:{value}",
            target.widget.0
        ));
        report.event_count = 1;
        self.dispatch_event(
            crate::ViewEvent::SliderChanged {
                widget: target.widget,
                value,
            },
            &mut report,
        );
        report
    }

    #[cfg(feature = "color-picker")]
    fn dispatch_color_picker_pointer(
        &mut self,
        target: crate::ViewHitTarget,
        point: crate::Point,
        mut report: WindowsWin32ViewInputDispatchReport,
    ) -> WindowsWin32ViewInputDispatchReport {
        let Some(state) = self.widget_color_picker_state(target.widget) else {
            self.color_picker_drag = None;
            return report;
        };
        let root_bounds = self
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|candidate| {
                candidate.widget == target.widget
                    && candidate.kind == crate::ViewHitTargetKind::ColorPicker
            })
            .map(|target| target.bounds)
            .unwrap_or(target.bounds);
        let plan = self.surface.map_or_else(
            || {
                crate::zs_color_picker_render_plan(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::Windows,
                    self.dpi,
                )
            },
            |viewport| {
                crate::zs_color_picker_render_plan_in_viewport(
                    root_bounds,
                    state,
                    crate::ZsColorPickerPlatformStyle::Windows,
                    self.dpi,
                    viewport,
                )
            },
        );
        let (color, channel) = match target.kind {
            crate::ViewHitTargetKind::ColorPickerSpectrum => {
                (plan.spectrum_color_at(state, point), None)
            }
            crate::ViewHitTargetKind::ColorPickerHue => (plan.hue_color_at(state, point), None),
            crate::ViewHitTargetKind::ColorPickerChannel { channel } => {
                let Some(row) = plan.channels.iter().find(|row| row.channel == channel) else {
                    self.color_picker_drag = None;
                    return report;
                };
                let fraction =
                    point.x.saturating_sub(row.track.x) as f32 / row.track.width.max(1) as f32;
                let value = (fraction.clamp(0.0, 1.0) * 255.0).round() as u8;
                (Some(channel.with_value(state.color, value)), Some(channel))
            }
            _ => (None, None),
        };
        let Some(color) = color else {
            self.color_picker_drag = None;
            return report;
        };
        report.handled = true;
        report.color_picker_drag_active = self.color_picker_drag.is_some();
        if let Some(channel) = channel.filter(|channel| state.active_channel != *channel) {
            report.color_picker_channel_change_count = 1;
            report.event_count += 1;
            report.events.push(format!(
                "win32_view_color_picker_channel:{}:{channel:?}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ColorPickerChannelChanged {
                    widget: target.widget,
                    channel,
                },
                &mut report,
            );
        }
        if color == state.color {
            return report;
        }
        report.color_picker_value_change_count = 1;
        report.event_count += 1;
        report.events.push(format!(
            "win32_view_color_picker_changed:{}:{}",
            target.widget.0,
            crate::ZsColorPickerState::new(color).hex_label()
        ));
        self.dispatch_event(
            crate::ViewEvent::ColorChanged {
                widget: target.widget,
                color,
            },
            &mut report,
        );
        report
    }

}

impl WindowsWin32ViewInputRoute {
    fn dispatch_scroll(
        &mut self,
        point: crate::Point,
        delta_y: crate::Dp,
    ) -> WindowsWin32ViewInputDispatchReport {
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            scroll_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        let Some(target) = self.interaction_plan.hit_target_at(point) else {
            report.unhandled_scroll_count = 1;
            report
                .events
                .push(format!("win32_view_scroll_missed:{}:{}", point.x, point.y));
            return report;
        };

        if target.kind == crate::ViewHitTargetKind::TextEditor
            && self.focused_widget == Some(target.widget)
            && delta_y.0 != 0.0
        {
            let value = self
                .widget_display_text_value(target.widget)
                .unwrap_or_default();
            let mut state = self
                .text_edit
                .filter(|state| state.widget == target.widget)
                .unwrap_or_else(|| NativeTextEditState::at_end(target.widget, &value));
            let previous = state.first_visible_visual_row;
            state.first_visible_visual_row = native_text_scroll_visual_rows_with_backend(
                target,
                &value,
                previous,
                native_text_wheel_row_delta(delta_y),
                self.widget_text_wrap(target.widget),
                self.dpi,
                &self.text_shaping,
            );
            self.text_edit = Some(state);
            report.handled = true;
            report.events.push(format!(
                "win32_view_text_scroll:{}:{}",
                target.widget.0, state.first_visible_visual_row
            ));
            if state.first_visible_visual_row != previous {
                self.rebuild_pending_draw_plan();
            }
            return report;
        }

        #[cfg(feature = "combo")]
        if matches!(target.kind, crate::ViewHitTargetKind::ComboBoxOption { .. })
            && delta_y.0 != 0.0
        {
            let Some((_, option_count, true)) = self.widget_combo_state(target.widget) else {
                return report;
            };
            let Some(visible_range) = self
                .interaction_plan
                .combo_visible_option_range(target.widget)
            else {
                return report;
            };
            let visible_count = visible_range.len();
            let maximum_first = option_count.saturating_sub(visible_count);
            let next_first = if delta_y.0 > 0.0 {
                visible_range.start.saturating_add(1).min(maximum_first)
            } else {
                visible_range.start.saturating_sub(1)
            };
            report.handled = true;
            if next_first == visible_range.start {
                return report;
            }
            report.combo_scroll_count = 1;
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_combo_scroll:{}:{next_first}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ComboBoxScrolled {
                    widget: target.widget,
                    first_visible_index: next_first,
                },
                &mut report,
            );
            return report;
        }

        #[cfg(feature = "scroll")]
        if let Some(scroll_widget) = self.widget_scroll_target(target.widget) {
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_scroll:{}:{}",
                scroll_widget.0, delta_y.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ScrollBy {
                    widget: scroll_widget,
                    delta_y,
                },
                &mut report,
            );
            return report;
        }

        let _ = delta_y;
        report.unhandled_scroll_count = 1;
        report.events.push(format!(
            "win32_view_scroll_without_scroll_target:{}",
            target.widget.0
        ));
        report
    }

}

#[derive(Debug, Clone)]
pub struct WindowsWin32ShellInputRoute {
    runtime: ZsShellRuntime,
    events: Vec<ZsShellInteractionEvent>,
}

impl WindowsWin32ShellInputRoute {
    pub fn new(runtime: ZsShellRuntime) -> Self {
        Self {
            runtime,
            events: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &ZsShellRuntime {
        &self.runtime
    }

    pub fn events(&self) -> &[ZsShellInteractionEvent] {
        &self.events
    }
}

pub fn set_windows_win32_window_shell_input_route(
    hwnd: HWND,
    mut route: WindowsWin32ShellInputRoute,
) -> bool {
    if hwnd.is_null() {
        return false;
    }
    if let Some((bounds, dpi)) = windows_win32_shell_surface(hwnd) {
        route.runtime.set_surface(bounds, dpi);
    }
    let plan = route.runtime.draw_plan();
    let hwnd_value = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    if let Some(record) = routes.iter_mut().find(|record| record.hwnd == hwnd_value) {
        record.route = route;
    } else {
        routes.push(WindowsWindowShellInputRouteRecord {
            hwnd: hwnd_value,
            route,
        });
    }
    drop(routes);
    set_windows_win32_window_draw_plan(hwnd, plan);
    unsafe {
        InvalidateRect(hwnd, null(), 0);
    }
    true
}

pub fn clear_windows_win32_window_shell_input_route(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let hwnd = hwnd as isize;
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.retain(|record| record.hwnd != hwnd);
}

pub fn clear_windows_win32_window_shell_input_routes() {
    let mut routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes.clear();
}

pub fn windows_win32_window_shell_input_events(hwnd: HWND) -> Option<Vec<ZsShellInteractionEvent>> {
    if hwnd.is_null() {
        return None;
    }
    let hwnd = hwnd as isize;
    let routes = window_shell_input_routes()
        .lock()
        .expect("window shell input route registry should not be poisoned");
    routes
        .iter()
        .find(|record| record.hwnd == hwnd)
        .map(|record| record.route.events.clone())
}

pub fn dispatch_windows_win32_window_shell_pointer_move(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    track_windows_win32_shell_pointer_leave(hwnd);
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_move(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_leave(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_leave)
}

pub fn dispatch_windows_win32_window_shell_pointer_down(
    hwnd: HWND,
    point: crate::Point,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.pointer_down(point))
}

pub fn dispatch_windows_win32_window_shell_pointer_up(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_up)
}

pub fn dispatch_windows_win32_window_shell_pointer_cancel(
    hwnd: HWND,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, ZsShellRuntime::pointer_cancel)
}

pub fn dispatch_windows_win32_window_shell_scroll(
    hwnd: HWND,
    delta_y: i32,
) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |runtime| runtime.scroll_by(delta_y))
}

pub fn refresh_windows_win32_window_shell_surface(hwnd: HWND) -> Option<ZsShellInteractionUpdate> {
    dispatch_windows_win32_window_shell_update(hwnd, |_| ZsShellInteractionUpdate::default())
}

fn dispatch_windows_win32_window_shell_update(
    hwnd: HWND,
    update: impl FnOnce(&mut ZsShellRuntime) -> ZsShellInteractionUpdate,
) -> Option<ZsShellInteractionUpdate> {
    if hwnd.is_null() {
        return None;
    }
    let surface = windows_win32_shell_surface(hwnd);
    let hwnd_value = hwnd as isize;
    let (result, plan) = {
        let mut routes = window_shell_input_routes()
            .lock()
            .expect("window shell input route registry should not be poisoned");
        let record = routes.iter_mut().find(|record| record.hwnd == hwnd_value)?;
        let surface_changed = surface
            .map(|(bounds, dpi)| record.route.runtime.set_surface(bounds, dpi))
            .unwrap_or(false);
        let mut result = update(&mut record.route.runtime);
        if surface_changed {
            result.redraw = true;
        }
        record.route.events.extend(result.events.iter().cloned());
        let plan = result.redraw.then(|| record.route.runtime.draw_plan());
        (result, plan)
    };

    if let Some(plan) = plan {
        set_windows_win32_window_draw_plan(hwnd, plan);
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }
    Some(result)
}

fn window_shell_input_routes() -> &'static Mutex<Vec<WindowsWindowShellInputRouteRecord>> {
    WINDOW_SHELL_INPUT_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

fn windows_win32_shell_surface(hwnd: HWND) -> Option<(crate::Rect, crate::Dpi)> {
    let mut rect: RECT = unsafe { zeroed() };
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
        return None;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32;
    Some((rect_from_win(rect), crate::Dpi(dpi)))
}

fn track_windows_win32_shell_pointer_leave(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let mut event = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: HOVER_DEFAULT,
    };
    unsafe {
        TrackMouseEvent(&mut event);
    }
}
