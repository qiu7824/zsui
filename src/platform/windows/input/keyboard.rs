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
        #[cfg(not(any(feature = "radio", feature = "tabs")))]
        let _ = control;
        let mut report = WindowsWin32ViewInputDispatchReport {
            hit_target_count: self.hit_target_count(),
            key_down_count: 1,
            ..WindowsWin32ViewInputDispatchReport::default()
        };
        #[cfg(feature = "tooltip")]
        if self.tooltip.dismiss() {
            report.handled = true;
            self.rebuild_pending_draw_plan();
        }
        #[cfg(feature = "toast")]
        if let Some(toast_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::Toast)
        {
            let Some((state, spec)) = self.widget_toast_state(toast_target.widget) else {
                return report;
            };
            let Some(toast) = state.toast else {
                return report;
            };
            if virtual_key == u32::from(VK_ESCAPE) {
                report.handled = true;
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:EscapeKey",
                    toast_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: toast_target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::EscapeKey,
                        ),
                    },
                    &mut report,
                );
                return report;
            }
            if self.focused_widget == Some(toast_target.widget) {
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) => Some(-1),
                    key if key == u32::from(VK_RIGHT) => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.toast_focus_change_count = usize::from(next != state.focused_control);
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_toast_focus:{}:{next:?}",
                        toast_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::ToastFocused {
                            widget: toast_target.widget,
                            toast,
                            control: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let response = match state.focused_control {
                        crate::ZsToastControl::Action if spec.action_label().is_some() => {
                            crate::ZsToastResponse::Action
                        }
                        _ => crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.toast_response_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_toast_response:{}:{toast:?}:{response:?}",
                        toast_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::ToastResponded {
                            widget: toast_target.widget,
                            toast,
                            response,
                        },
                        &mut report,
                    );
                    return report;
                }
            }
        }
        #[cfg(feature = "teaching-tip")]
        if let Some(tip_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTip)
        {
            let Some((state, spec)) = self.widget_teaching_tip_state(tip_target.widget) else {
                return report;
            };
            if virtual_key == u32::from(VK_ESCAPE) {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::EscapeKey,
                );
                report.handled = true;
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:{response:?}",
                    tip_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: tip_target.widget,
                        response,
                    },
                    &mut report,
                );
                return report;
            }
            if self.focused_widget == Some(tip_target.widget) {
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) => Some(-1),
                    key if key == u32::from(VK_RIGHT) => Some(1),
                    _ => None,
                };
                if let Some(offset) = focus_offset {
                    let next = spec.relative_control(state.focused_control, offset);
                    report.handled = true;
                    report.teaching_tip_focus_change_count =
                        usize::from(next != state.focused_control);
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_teaching_tip_focus:{}:{next:?}",
                        tip_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TeachingTipFocused {
                            widget: tip_target.widget,
                            control: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let response = match state.focused_control {
                        crate::ZsTeachingTipControl::Action if spec.action_label().is_some() => {
                            crate::ZsTeachingTipResponse::Action
                        }
                        _ => crate::ZsTeachingTipResponse::Dismissed(
                            crate::ZsTeachingTipDismissReason::CloseButton,
                        ),
                    };
                    report.handled = true;
                    report.teaching_tip_response_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_teaching_tip_response:{}:{response:?}",
                        tip_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TeachingTipResponded {
                            widget: tip_target.widget,
                            response,
                        },
                        &mut report,
                    );
                    return report;
                }
            }
        }
        #[cfg(feature = "info-bar")]
        if let Some(widget) = self.focused_widget {
            if let Some((state, spec)) = self.widget_info_bar_state(widget) {
                if virtual_key == u32::from(VK_ESCAPE) && spec.is_closable() {
                    report.handled = true;
                    report.info_bar_event_count = 1;
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_info_bar_event:{}:Close", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::InfoBarInvoked {
                            widget,
                            event: crate::ZsInfoBarEvent::Close,
                        },
                        &mut report,
                    );
                    return report;
                }
                if let Some(current) = state.focused_control {
                    let focus_offset = match virtual_key {
                        key if key == u32::from(VK_LEFT) => Some(-1),
                        key if key == u32::from(VK_RIGHT) => Some(1),
                        _ => None,
                    };
                    if let Some(offset) = focus_offset {
                        let next = spec.relative_control(current, offset);
                        report.handled = true;
                        report.info_bar_focus_change_count = usize::from(next != current);
                        report.event_count = 1;
                        report
                            .events
                            .push(format!("win32_view_info_bar_focus:{}:{next:?}", widget.0));
                        self.dispatch_event(
                            crate::ViewEvent::InfoBarFocused {
                                widget,
                                control: next,
                            },
                            &mut report,
                        );
                        return report;
                    }
                    if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                        let event = match current {
                            crate::ZsInfoBarControl::Action => crate::ZsInfoBarEvent::Action,
                            crate::ZsInfoBarControl::Close => crate::ZsInfoBarEvent::Close,
                        };
                        if spec.has_control(current) {
                            report.handled = true;
                            report.info_bar_event_count = 1;
                            report.event_count = 1;
                            report
                                .events
                                .push(format!("win32_view_info_bar_event:{}:{event:?}", widget.0));
                            self.dispatch_event(
                                crate::ViewEvent::InfoBarInvoked { widget, event },
                                &mut report,
                            );
                            return report;
                        }
                    }
                }
            }
        }
        #[cfg(feature = "breadcrumb")]
        if let Some(widget) = self.focused_widget {
            if let Some(state) = self.widget_breadcrumb_state(widget) {
                let mut visible = self
                    .interaction_plan
                    .hit_targets
                    .iter()
                    .filter_map(|target| match target.kind {
                        crate::ViewHitTargetKind::BreadcrumbOverflow if target.widget == widget => {
                            Some((target.bounds.x, crate::ZsBreadcrumbFocusTarget::Overflow))
                        }
                        crate::ViewHitTargetKind::BreadcrumbItem { item }
                            if target.widget == widget =>
                        {
                            Some((target.bounds.x, crate::ZsBreadcrumbFocusTarget::Item(item)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                visible.sort_by_key(|(x, _)| *x);
                let visible = visible
                    .into_iter()
                    .map(|(_, target)| target)
                    .collect::<Vec<_>>();
                let mut hidden = self
                    .interaction_plan
                    .hit_targets
                    .iter()
                    .filter_map(|target| match target.kind {
                        crate::ViewHitTargetKind::BreadcrumbOverflowItem { item }
                            if target.widget == widget =>
                        {
                            Some((target.bounds.y, crate::ZsBreadcrumbFocusTarget::Item(item)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                hidden.sort_by_key(|(y, _)| *y);
                let hidden = hidden
                    .into_iter()
                    .map(|(_, target)| target)
                    .collect::<Vec<_>>();

                if virtual_key == u32::from(VK_ESCAPE) && state.overflow_open {
                    report.handled = true;
                    report.breadcrumb_expanded_change_count = 1;
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_breadcrumb_expanded:{}:false", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::BreadcrumbExpandedChanged {
                            widget,
                            expanded: false,
                        },
                        &mut report,
                    );
                    return report;
                }

                let focus_list = if state.overflow_open
                    && matches!(virtual_key, key if key == u32::from(VK_UP) || key == u32::from(VK_DOWN))
                    && !hidden.is_empty()
                {
                    &hidden
                } else {
                    &visible
                };
                let focus_offset = match virtual_key {
                    key if key == u32::from(VK_LEFT) || key == u32::from(VK_UP) => Some(-1),
                    key if key == u32::from(VK_RIGHT) || key == u32::from(VK_DOWN) => Some(1),
                    key if key == u32::from(VK_HOME) => Some(isize::MIN),
                    key if key == u32::from(VK_END) => Some(isize::MAX),
                    _ => None,
                };
                if let Some(offset) = focus_offset.filter(|_| !focus_list.is_empty()) {
                    let current_index = state.focused.and_then(|current| {
                        focus_list.iter().position(|target| *target == current)
                    });
                    let next_index = if offset == isize::MIN {
                        0
                    } else if offset == isize::MAX {
                        focus_list.len() - 1
                    } else {
                        match current_index {
                            Some(index) => (index as isize + offset)
                                .clamp(0, focus_list.len().saturating_sub(1) as isize)
                                as usize,
                            None if offset < 0 => focus_list.len() - 1,
                            None => 0,
                        }
                    };
                    let next = focus_list[next_index];
                    report.handled = true;
                    report.breadcrumb_focus_change_count = usize::from(state.focused != Some(next));
                    report.event_count = 1;
                    report
                        .events
                        .push(format!("win32_view_breadcrumb_focus:{}:{next:?}", widget.0));
                    self.dispatch_event(
                        crate::ViewEvent::BreadcrumbFocused {
                            widget,
                            target: next,
                        },
                        &mut report,
                    );
                    return report;
                }
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    let active = state
                        .focused
                        .or_else(|| visible.first().copied())
                        .or_else(|| state.current().map(crate::ZsBreadcrumbFocusTarget::Item));
                    match active {
                        Some(crate::ZsBreadcrumbFocusTarget::Overflow) => {
                            report.handled = true;
                            report.breadcrumb_expanded_change_count = 1;
                            report.event_count = 1;
                            report.events.push(format!(
                                "win32_view_breadcrumb_expanded:{}:{}",
                                widget.0, !state.overflow_open
                            ));
                            self.dispatch_event(
                                crate::ViewEvent::BreadcrumbExpandedChanged {
                                    widget,
                                    expanded: !state.overflow_open,
                                },
                                &mut report,
                            );
                            return report;
                        }
                        Some(crate::ZsBreadcrumbFocusTarget::Item(item)) => {
                            report.handled = true;
                            report.breadcrumb_selection_count = 1;
                            report.breadcrumb_expanded_change_count =
                                usize::from(state.overflow_open);
                            report.event_count = 1;
                            report.events.push(format!(
                                "win32_view_breadcrumb_selected:{}:{}",
                                widget.0,
                                item.get()
                            ));
                            self.dispatch_event(
                                crate::ViewEvent::BreadcrumbSelected { widget, item },
                                &mut report,
                            );
                            return report;
                        }
                        None => {}
                    }
                }
            }
        }
        #[cfg(feature = "command-palette")]
        if let Some(palette_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::CommandPalette)
        {
            if self.focused_widget != Some(palette_target.widget) {
                self.focus_target(palette_target, &mut report);
            }
            report.focused_widget = Some(palette_target.widget.0);
            let Some(state) = self.widget_command_palette_state(palette_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let next = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_highlight(-1),
                key if key == u32::from(VK_DOWN) => state.relative_highlight(1),
                key if key == u32::from(VK_HOME) => state.first_enabled(),
                key if key == u32::from(VK_END) => state.last_enabled(),
                _ => None,
            };
            if let Some(item) = next {
                report.handled = true;
                report.command_palette_highlight_change_count =
                    usize::from(state.highlighted != Some(item));
                report.event_count = usize::from(report.command_palette_highlight_change_count > 0);
                if report.command_palette_highlight_change_count > 0 {
                    report.events.push(format!(
                        "win32_view_command_palette_highlight:{}:{}",
                        palette_target.widget.0,
                        item.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::CommandPaletteHighlighted {
                            widget: palette_target.widget,
                            item,
                        },
                        &mut report,
                    );
                }
                return report;
            }
            match virtual_key {
                ZSUI_WIN32_VK_RETURN => {
                    if let Some(item) = state.highlighted.or_else(|| state.first_enabled()) {
                        report.handled = true;
                        report.command_palette_invoke_count = 1;
                        report.command_palette_open_change_count = 1;
                        report.event_count = 1;
                        report.events.push(format!(
                            "win32_view_command_palette_invoke:{}:{}",
                            palette_target.widget.0,
                            item.get()
                        ));
                        self.dispatch_event(
                            crate::ViewEvent::CommandPaletteInvoked {
                                widget: palette_target.widget,
                                item,
                            },
                            &mut report,
                        );
                        return report;
                    }
                }
                key if key == u32::from(VK_ESCAPE) => {
                    report.handled = true;
                    report.command_palette_open_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_command_palette_dismissed:{}",
                        palette_target.widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::CommandPaletteOpenChanged {
                            widget: palette_target.widget,
                            open: false,
                        },
                        &mut report,
                    );
                    return report;
                }
                ZSUI_WIN32_VK_TAB => {
                    report.handled = true;
                    return report;
                }
                _ => {}
            }
        }

        #[cfg(feature = "dialog")]
        if let Some(dialog_target) = self
            .interaction_plan
            .hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialog)
        {
            if self.focused_widget != Some(dialog_target.widget) {
                self.focus_target(dialog_target, &mut report);
            }
            report.focused_widget = Some(dialog_target.widget.0);
            let Some((state, spec)) = self.widget_content_dialog_state(dialog_target.widget) else {
                return report;
            };
            if !state.open {
                return report;
            }
            let focus_offset = match virtual_key {
                ZSUI_WIN32_VK_TAB => Some(if shift { -1 } else { 1 }),
                key if key == u32::from(VK_LEFT) => Some(-1),
                key if key == u32::from(VK_RIGHT) => Some(1),
                _ => None,
            };
            if let Some(offset) = focus_offset {
                let button = spec.relative_button(state.focused_button, offset);
                report.handled = true;
                report.content_dialog_focus_change_count =
                    usize::from(button != state.focused_button);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_focus:{}:{button:?}",
                    dialog_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogFocused {
                        widget: dialog_target.widget,
                        button,
                    },
                    &mut report,
                );
                return report;
            }
            let response = match virtual_key {
                key if key == u32::from(VK_ESCAPE) => Some(crate::ZsContentDialogButton::Close),
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(state.focused_button),
                _ => None,
            };
            if let Some(button) = response.filter(|button| spec.has_button(*button)) {
                report.handled = true;
                report.content_dialog_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_response:{}:{button:?}",
                    dialog_target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogResponded {
                        widget: dialog_target.widget,
                        button,
                    },
                    &mut report,
                );
                return report;
            }
            return report;
        }
        if virtual_key == ZSUI_WIN32_VK_TAB && !control {
            self.dispatch_focus_traversal(if shift { -1 } else { 1 }, &mut report);
            return report;
        }

        let Some(widget) = self.focused_widget else {
            report.unhandled_key_count = 1;
            report
                .events
                .push(format!("win32_view_key_without_focus:{virtual_key}"));
            return report;
        };

        #[cfg(feature = "tabs")]
        if virtual_key == ZSUI_WIN32_VK_TAB && control {
            let offset = if shift { -1 } else { 1 };
            let Some((tab_view, tab)) = self.widget_tab_cycle_target(widget, offset) else {
                report.unhandled_key_count = 1;
                return report;
            };
            report.handled = true;
            report.tab_selection_count = 1;
            report.tab_keyboard_selection_count = 1;
            report.event_count = 1;
            if let Some(target) = self
                .interaction_plan
                .hit_target_for_widget(crate::WidgetId(tab.0))
            {
                self.focus_target(target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(target.widget);
            }
            report
                .events
                .push(format!("win32_view_tab_cycle:{}:{}", tab_view.0, tab.0));
            self.dispatch_event(
                crate::ViewEvent::TabSelected {
                    widget: tab_view,
                    tab,
                },
                &mut report,
            );
            return report;
        }
        let Some(target) = self.interaction_plan.hit_target_for_widget(widget) else {
            report.unhandled_key_count = 1;
            report.events.push(format!(
                "win32_view_key_without_target:{widget:?}:{virtual_key}"
            ));
            return report;
        };

        if target.kind.accepts_text_input() {
            if virtual_key == u32::from(VK_DELETE) {
                let mut edit = self.dispatch_text_input("\u{7f}");
                edit.key_down_count = 1;
                return edit;
            }
            let movement = match virtual_key {
                key if key == u32::from(VK_HOME) => Some(NativeTextMovement::Home),
                key if key == u32::from(VK_END) => Some(NativeTextMovement::End),
                _ => None,
            };
            let horizontal_navigation = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(NativeTextVisualHorizontalDirection::Left),
                key if key == u32::from(VK_RIGHT) => {
                    Some(NativeTextVisualHorizontalDirection::Right)
                }
                _ => None,
            };
            let visual_navigation = (target.kind == crate::ViewHitTargetKind::TextEditor)
                .then(|| match virtual_key {
                    key if key == u32::from(VK_UP) => Some((NativeTextVisualDirection::Up, false)),
                    key if key == u32::from(VK_DOWN) => {
                        Some((NativeTextVisualDirection::Down, false))
                    }
                    key if key == u32::from(VK_PRIOR) => {
                        Some((NativeTextVisualDirection::Up, true))
                    }
                    key if key == u32::from(VK_NEXT) => {
                        Some((NativeTextVisualDirection::Down, true))
                    }
                    _ => None,
                })
                .flatten();
            if movement.is_some() || horizontal_navigation.is_some() || visual_navigation.is_some()
            {
                let value = self.widget_display_text_value(widget).unwrap_or_default();
                let mut state = self
                    .text_edit
                    .filter(|state| state.widget == widget)
                    .unwrap_or_else(|| NativeTextEditState::at_end(widget, &value));
                let edit = if let Some((direction, page)) = visual_navigation {
                    let (target_index, preferred_x) = if page {
                        let (target_index, preferred_x, first_visible_row) =
                            native_text_index_for_vertical_page_move_with_backend(
                                target,
                                &value,
                                state.selection.caret,
                                direction,
                                state.preferred_visual_x,
                                state.first_visible_visual_row,
                                self.widget_text_wrap(widget),
                                self.dpi,
                                &self.text_shaping,
                            );
                        state.first_visible_visual_row = first_visible_row;
                        (target_index, preferred_x)
                    } else {
                        native_text_index_for_vertical_move_with_backend(
                            target,
                            &value,
                            state.selection.caret,
                            direction,
                            state.preferred_visual_x,
                            self.widget_text_wrap(widget),
                            self.dpi,
                            &self.text_shaping,
                        )
                    };
                    state.preferred_visual_x = Some(preferred_x);
                    move_selection_to(&value, &mut state.selection, target_index, shift)
                } else if let Some(direction) = horizontal_navigation {
                    state.preferred_visual_x = None;
                    move_native_text_selection_horizontally_with_backend(
                        target,
                        &value,
                        &mut state.selection,
                        direction,
                        shift,
                        self.widget_text_wrap(widget),
                        self.dpi,
                        &self.text_shaping,
                    )
                } else {
                    state.preferred_visual_x = None;
                    move_selection(
                        &value,
                        &mut state.selection,
                        movement.expect("text movement should be present"),
                        shift,
                        target.kind == crate::ViewHitTargetKind::TextEditor,
                    )
                };
                if !visual_navigation.is_some_and(|(_, page)| page) {
                    state.first_visible_visual_row =
                        native_text_first_visible_row_for_caret_with_backend(
                            target,
                            &value,
                            state.selection.caret,
                            state.first_visible_visual_row,
                            self.widget_text_wrap(widget),
                            self.dpi,
                            &self.text_shaping,
                        );
                }
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
                report.handled = edit.handled;
                report.text_navigation_count = 1;
                report.text_selection_change_count = usize::from(edit.selection_changed);
                report.text_caret = Some(state.selection.caret);
                report.events.push(format!(
                    "win32_view_text_navigate:{}:{virtual_key}:{}",
                    widget.0, state.selection.caret
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
                            widget,
                            selection: state.selection.into(),
                        },
                        &mut report,
                    );
                }
                self.rebuild_pending_draw_plan();
                return report;
            }
        }

        #[cfg(feature = "auto-suggest")]
        if target.kind == crate::ViewHitTargetKind::AutoSuggestBox {
            let Some(state) = self.widget_auto_suggest_state(widget) else {
                return report;
            };
            if (virtual_key == u32::from(VK_UP) || virtual_key == u32::from(VK_DOWN))
                && !state.suggestion_ids.is_empty()
            {
                let offset = if virtual_key == u32::from(VK_UP) {
                    -1
                } else {
                    1
                };
                let Some(suggestion) = state.next_highlight(offset) else {
                    return report;
                };
                report.handled = true;
                report.auto_suggest_highlight_change_count =
                    usize::from(state.highlighted != Some(suggestion));
                report.auto_suggest_expanded_change_count = usize::from(!state.expanded);
                report.event_count = 1;
                if !state.expanded {
                    self.dispatch_event(
                        crate::ViewEvent::AutoSuggestExpandedChanged {
                            widget,
                            expanded: true,
                        },
                        &mut report,
                    );
                    report.event_count += 1;
                }
                report.events.push(format!(
                    "win32_view_auto_suggest_highlight:{}:{}",
                    widget.0,
                    suggestion.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestHighlighted { widget, suggestion },
                    &mut report,
                );
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                report.handled = true;
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_keyboard_submit:{}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget,
                        suggestion: state.highlighted,
                    },
                    &mut report,
                );
                return report;
            }
            if virtual_key == u32::from(VK_ESCAPE) && state.expanded {
                report.handled = true;
                report.auto_suggest_expanded_change_count = 1;
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_auto_suggest_escape:{}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestExpandedChanged {
                        widget,
                        expanded: false,
                    },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "tree")]
        if target.kind == crate::ViewHitTargetKind::TreeView {
            let Some(state) = self.widget_tree_view_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_visible(-1),
                key if key == u32::from(VK_DOWN) => state.relative_visible(1),
                key if key == u32::from(VK_HOME) => state.first_visible(),
                key if key == u32::from(VK_END) => state.last_visible(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first_visible()),
                _ => None,
            };
            if let Some(node) = select {
                report.handled = true;
                if state.selected != Some(node) {
                    report.tree_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_select:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected { widget, node },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == u32::from(VK_LEFT) {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && row.expanded {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_collapse:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: false,
                        },
                        &mut report,
                    );
                } else if let Some(parent) = row.parent {
                    report.tree_selection_count = usize::from(state.selected != Some(parent));
                    report.event_count = report.tree_selection_count;
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected {
                            widget,
                            node: parent,
                        },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == u32::from(VK_RIGHT) {
                let Some(node) = state.selected.or_else(|| state.first_visible()) else {
                    return report;
                };
                let Some(row) = state.row(node) else {
                    return report;
                };
                report.handled = true;
                if row.expandable && !row.expanded {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_key_expand:{}:{}",
                        widget.0,
                        node.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget,
                            node,
                            expanded: true,
                        },
                        &mut report,
                    );
                } else if let Some(child) = state.first_visible_child(node) {
                    report.tree_selection_count = usize::from(state.selected != Some(child));
                    report.event_count = report.tree_selection_count;
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeSelected {
                            widget,
                            node: child,
                        },
                        &mut report,
                    );
                }
                return report;
            }

            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(node) = state
                    .selected
                    .filter(|selected| state.row(*selected).is_some())
                else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.tree_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_tree_key_invoke:{}:{}",
                    widget.0,
                    node.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeInvoked { widget, node },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "grid-view")]
        if target.kind == crate::ViewHitTargetKind::GridView {
            let Some(state) = self.widget_grid_view_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_LEFT) => state.relative_horizontal(-1),
                key if key == u32::from(VK_RIGHT) => state.relative_horizontal(1),
                key if key == u32::from(VK_UP) => state.relative_vertical(-1),
                key if key == u32::from(VK_DOWN) => state.relative_vertical(1),
                key if key == u32::from(VK_HOME) => state.first(),
                key if key == u32::from(VK_END) => state.last(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first()),
                _ => None,
            };
            if let Some(item) = select {
                report.handled = true;
                if state.selected != Some(item) {
                    report.grid_view_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_grid_view_key_select:{}:{}",
                        widget.0,
                        item.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::GridViewItemSelected { widget, item },
                        &mut report,
                    );
                }
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(item) = state
                    .selected
                    .filter(|selected| state.contains(*selected))
                    .or_else(|| state.first())
                else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.grid_view_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_grid_view_key_invoke:{}:{}",
                    widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemInvoked { widget, item },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "table")]
        if target.kind == crate::ViewHitTargetKind::DataGrid {
            let Some(state) = self.widget_table_state(widget) else {
                return report;
            };
            let select = match virtual_key {
                key if key == u32::from(VK_UP) => state.relative_row(-1),
                key if key == u32::from(VK_DOWN) => state.relative_row(1),
                key if key == u32::from(VK_HOME) => state.first_row(),
                key if key == u32::from(VK_END) => state.last_row(),
                ZSUI_WIN32_VK_SPACE => state.selected.or_else(|| state.first_row()),
                _ => None,
            };
            if let Some(row) = select {
                report.handled = true;
                if state.selected != Some(row) {
                    report.table_selection_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_table_key_select:{}:{}",
                        widget.0,
                        row.get()
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TableRowSelected { widget, row },
                        &mut report,
                    );
                }
                return report;
            }
            if virtual_key == ZSUI_WIN32_VK_RETURN {
                let Some(row) = state.selected.filter(|row| state.contains_row(*row)) else {
                    return report;
                };
                report.handled = true;
                report.keyboard_activation_count = 1;
                report.table_invoke_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_table_key_invoke:{}:{}",
                    widget.0,
                    row.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableRowInvoked { widget, row },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "slider")]
        if target.kind == crate::ViewHitTargetKind::Slider {
            let Some((current, range)) = self.widget_slider_state(widget) else {
                return report;
            };
            let delta = if shift { 10 } else { 1 };
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) || key == u32::from(VK_DOWN) => {
                    Some(range.offset_steps(current, -delta))
                }
                key if key == u32::from(VK_RIGHT) || key == u32::from(VK_UP) => {
                    Some(range.offset_steps(current, delta))
                }
                key if key == u32::from(VK_HOME) => Some(range.min()),
                key if key == u32::from(VK_END) => Some(range.max()),
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                report.slider_keyboard_change_count = 1;
                if (value - current).abs() <= f32::EPSILON {
                    return report;
                }
                report.slider_value_change_count = 1;
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_slider_key:{}:{value}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::SliderChanged { widget, value },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let Some(state) = self.widget_color_picker_state(widget) else {
                return report;
            };
            let next_expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = next_expanded {
                report.handled = true;
                report.color_picker_expanded_change_count = 1;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorPickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            if !state.expanded {
                return report;
            }

            let next_channel = match virtual_key {
                key if key == u32::from(VK_UP) => {
                    Some(state.active_channel.previous(state.alpha_enabled))
                }
                key if key == u32::from(VK_DOWN) => {
                    Some(state.active_channel.next(state.alpha_enabled))
                }
                _ => None,
            };
            if let Some(channel) = next_channel {
                report.handled = true;
                if channel == state.active_channel {
                    return report;
                }
                report.color_picker_channel_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_channel:{}:{channel:?}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorPickerChannelChanged { widget, channel },
                    &mut report,
                );
                return report;
            }

            let current = state.channel_value(state.active_channel);
            let delta = if shift { 10_i16 } else { 1_i16 };
            let next = match virtual_key {
                key if key == u32::from(VK_LEFT) => {
                    Some((i16::from(current) - delta).clamp(0, 255) as u8)
                }
                key if key == u32::from(VK_RIGHT) => {
                    Some((i16::from(current) + delta).clamp(0, 255) as u8)
                }
                key if key == u32::from(VK_HOME) => Some(0),
                key if key == u32::from(VK_END) => Some(255),
                _ => None,
            };
            if let Some(value) = next {
                report.handled = true;
                let color = state.active_channel.with_value(state.color, value);
                if color == state.color {
                    return report;
                }
                report.color_picker_value_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_color_picker_key:{}:{}",
                    widget.0,
                    crate::ZsColorPickerState::new(color).hex_label()
                ));
                self.dispatch_event(
                    crate::ViewEvent::ColorChanged { widget, color },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "number-box")]
        if target.kind == crate::ViewHitTargetKind::NumberBox {
            let event = match virtual_key {
                key if key == u32::from(VK_DOWN) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: shift,
                }),
                key if key == u32::from(VK_UP) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: shift,
                }),
                key if key == u32::from(VK_NEXT) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: -1,
                    large: true,
                }),
                key if key == u32::from(VK_PRIOR) => Some(crate::ViewEvent::NumberBoxStep {
                    widget,
                    steps: 1,
                    large: true,
                }),
                ZSUI_WIN32_VK_RETURN => Some(crate::ViewEvent::NumberBoxCommit { widget }),
                key if key == u32::from(VK_ESCAPE) => {
                    Some(crate::ViewEvent::NumberBoxReset { widget })
                }
                _ => None,
            };
            if let Some(event) = event {
                report.handled = true;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_number_box_key:{}:{virtual_key}",
                    widget.0
                ));
                self.dispatch_event(event, &mut report);
                return report;
            }
        }

        #[cfg(feature = "combo")]
        if target.kind == crate::ViewHitTargetKind::ComboBox {
            let Some((selected, option_count, expanded)) = self.widget_combo_state(widget) else {
                return report;
            };
            if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE)
                || matches!(
                    virtual_key,
                    key if key == u32::from(VK_ESCAPE)
                        || key == u32::from(VK_UP)
                        || key == u32::from(VK_DOWN)
                        || key == u32::from(VK_HOME)
                        || key == u32::from(VK_END)
                )
            {
                self.combo_type_ahead.reset();
            }
            let expanded_event = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!expanded),
                key if key == u32::from(VK_ESCAPE) && expanded => Some(false),
                _ => None,
            };
            if let Some(next_expanded) = expanded_event {
                report.handled = true;
                report.combo_expanded_change_count = 1;
                if matches!(virtual_key, ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE) {
                    report.keyboard_activation_count = 1;
                }
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_expanded:{}:{next_expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxExpandedChanged {
                        widget,
                        expanded: next_expanded,
                    },
                    &mut report,
                );
                return report;
            }

            let next_index = match virtual_key {
                key if key == u32::from(VK_UP) && option_count > 0 => {
                    Some(selected.unwrap_or(option_count).saturating_sub(1))
                }
                key if key == u32::from(VK_DOWN) && option_count > 0 => {
                    Some(selected.map_or(0, |index| index.saturating_add(1).min(option_count - 1)))
                }
                key if key == u32::from(VK_HOME) && option_count > 0 => Some(0),
                key if key == u32::from(VK_END) && option_count > 0 => Some(option_count - 1),
                _ => None,
            };
            if let Some(index) = next_index {
                report.handled = true;
                if selected == Some(index) {
                    return report;
                }
                report.combo_selection_count = 1;
                report.combo_keyboard_selection_count = 1;
                report.combo_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_combo_key_select:{}:{index}", widget.0));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxSelected { widget, index },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            let navigation = match virtual_key {
                key if key == u32::from(VK_UP) => Some((crate::ViewStackDirection::Column, -1)),
                key if key == u32::from(VK_DOWN) => Some((crate::ViewStackDirection::Column, 1)),
                key if key == u32::from(VK_LEFT) => Some((crate::ViewStackDirection::Row, -1)),
                key if key == u32::from(VK_RIGHT) => Some((crate::ViewStackDirection::Row, 1)),
                _ => None,
            };
            if let Some((navigation, offset)) = navigation {
                let Some(next_widget) =
                    self.widget_radio_relative_widget(widget, navigation, offset)
                else {
                    return report;
                };
                report.handled = true;
                if next_widget == widget {
                    return report;
                }
                let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget);
                if control {
                    report.radio_keyboard_focus_only_count = 1;
                    report.events.push(format!(
                        "win32_view_radio_key_focus_only:{}:{}",
                        widget.0, next_widget.0
                    ));
                    return report;
                }
                report.radio_selection_count = 1;
                report.radio_keyboard_selection_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_radio_key_select:{}:{}",
                    widget.0, next_widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::RadioSelected {
                        widget: next_widget,
                    },
                    &mut report,
                );
                return report;
            }
        }

        #[cfg(feature = "tabs")]
        if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
            let Some(state) = self.widget_tab_header_state(widget) else {
                return report;
            };
            let next_widget = match virtual_key {
                key if key == u32::from(VK_LEFT) => state.previous,
                key if key == u32::from(VK_RIGHT) => state.next,
                _ => None,
            };
            if matches!(
                virtual_key,
                key if key == u32::from(VK_LEFT)
                    || key == u32::from(VK_RIGHT)
            ) {
                report.handled = true;
                let Some(next_widget) = next_widget else {
                    return report;
                };
                if next_widget == widget {
                    return report;
                }
                let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                else {
                    return report;
                };
                self.focus_target(next_target, &mut report);
                #[cfg(feature = "tooltip")]
                self.show_keyboard_tooltip(next_target.widget);
                report.tab_keyboard_focus_only_count = 1;
                report.events.push(format!(
                    "win32_view_tab_key_focus:{}:{}",
                    widget.0, next_widget.0
                ));
                return report;
            }
        }

        #[cfg(feature = "date-picker")]
        if target.kind == crate::ViewHitTargetKind::DatePicker {
            let Some(state) = self.widget_date_picker_state(widget) else {
                return report;
            };
            let expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = expanded {
                report.handled = true;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(state.value.add_days(-1)),
                key if key == u32::from(VK_RIGHT) => Some(state.value.add_days(1)),
                key if key == u32::from(VK_UP) => Some(state.value.add_days(-7)),
                key if key == u32::from(VK_DOWN) => Some(state.value.add_days(7)),
                key if key == u32::from(VK_HOME) => Some(state.value.first_day_of_month()),
                key if key == u32::from(VK_END) => {
                    Some(state.value.first_day_of_month().add_months(1).add_days(-1))
                }
                _ => None,
            };
            if let Some(value) = value {
                let value = value.clamp(state.minimum, state.maximum);
                report.handled = true;
                if value == state.value {
                    return report;
                }
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_date_picker_key:{}:{value}", widget.0));
                self.dispatch_event(crate::ViewEvent::DateChanged { widget, value }, &mut report);
                return report;
            }
        }

        #[cfg(feature = "time-picker")]
        if target.kind == crate::ViewHitTargetKind::TimePicker {
            let Some(state) = self.widget_time_picker_state(widget) else {
                return report;
            };
            let expanded = match virtual_key {
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE => Some(!state.expanded),
                key if key == u32::from(VK_ESCAPE) && state.expanded => Some(false),
                _ => None,
            };
            if let Some(expanded) = expanded {
                report.handled = true;
                report.keyboard_activation_count = usize::from(matches!(
                    virtual_key,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                ));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_expanded:{}:{expanded}",
                    widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimePickerExpandedChanged { widget, expanded },
                    &mut report,
                );
                return report;
            }
            let minute_step = i32::from(state.minute_increment.get());
            let value = match virtual_key {
                key if key == u32::from(VK_LEFT) => Some(state.value.add_minutes_wrapping(-60)),
                key if key == u32::from(VK_RIGHT) => Some(state.value.add_minutes_wrapping(60)),
                key if key == u32::from(VK_UP) => {
                    Some(state.value.add_minutes_wrapping(-minute_step))
                }
                key if key == u32::from(VK_DOWN) => {
                    Some(state.value.add_minutes_wrapping(minute_step))
                }
                key if key == u32::from(VK_HOME) => Some(crate::ZsTime::MIDNIGHT),
                key if key == u32::from(VK_END) => {
                    crate::ZsTime::new(23, 60 - state.minute_increment.get()).ok()
                }
                _ => None,
            };
            if let Some(value) = value {
                report.handled = true;
                if value == state.value {
                    return report;
                }
                report.event_count = 1;
                report
                    .events
                    .push(format!("win32_view_time_picker_key:{}:{value}", widget.0));
                self.dispatch_event(crate::ViewEvent::TimeChanged { widget, value }, &mut report);
                return report;
            }
        }

        #[cfg(feature = "list")]
        if matches!(virtual_key, ZSUI_WIN32_VK_UP | ZSUI_WIN32_VK_DOWN) {
            let offset = if virtual_key == ZSUI_WIN32_VK_UP {
                -1
            } else {
                1
            };
            if let Some((next_widget, index)) =
                self.widget_list_relative_widget(target.widget, offset)
            {
                if let Some(next_target) = self.interaction_plan.hit_target_for_widget(next_widget)
                {
                    self.focus_target(next_target, &mut report);
                    #[cfg(feature = "tooltip")]
                    self.show_keyboard_tooltip(next_target.widget);
                    report.selection_count = 1;
                    report.keyboard_selection_count = 1;
                    report.events.push(format!(
                        "win32_view_key_select:{}:{}:{index}",
                        target.widget.0, next_widget.0
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::Click {
                            widget: next_widget,
                        },
                        &mut report,
                    );
                    report.event_count = 1;
                    return report;
                }
            }
        }

        let activates = matches!(
            (target.kind, virtual_key),
            (
                crate::ViewHitTargetKind::Button | crate::ViewHitTargetKind::Unknown,
                ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE,
            ) | (
                crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle,
                ZSUI_WIN32_VK_SPACE,
            )
        );
        #[cfg(feature = "label")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (
                    crate::ViewHitTargetKind::NavigationViewToggle,
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                )
            );
        #[cfg(feature = "toggle-button")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (crate::ViewHitTargetKind::ToggleButton, ZSUI_WIN32_VK_SPACE)
            );
        #[cfg(feature = "radio")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (crate::ViewHitTargetKind::RadioButton, ZSUI_WIN32_VK_SPACE)
            );
        #[cfg(feature = "tabs")]
        let activates = activates
            || matches!(
                (target.kind, virtual_key),
                (
                    crate::ViewHitTargetKind::Tab { .. },
                    ZSUI_WIN32_VK_RETURN | ZSUI_WIN32_VK_SPACE
                )
            );
        if activates {
            report.keyboard_activation_count = 1;
            #[cfg(feature = "tabs")]
            if matches!(target.kind, crate::ViewHitTargetKind::Tab { .. }) {
                let changed = self
                    .widget_tab_header_state(target.widget)
                    .is_some_and(|state| !state.selected);
                report.tab_selection_count = usize::from(changed);
                report.tab_keyboard_selection_count = usize::from(changed);
            }
            report.events.push(format!(
                "win32_view_key_activate:{}:{virtual_key}",
                target.widget.0
            ));
            self.dispatch_activation(target, &mut report);
        } else {
            report.unhandled_key_count = 1;
            report.events.push(format!(
                "win32_view_key_unhandled:{}:{virtual_key}",
                target.widget.0
            ));
        }
        report
    }

}

impl WindowsWin32ViewInputRoute {
    fn dispatch_activation(
        &mut self,
        target: crate::ViewHitTarget,
        report: &mut WindowsWin32ViewInputDispatchReport,
    ) {
        #[cfg(feature = "color-picker")]
        if target.kind == crate::ViewHitTargetKind::ColorPicker {
            let expanded = self
                .widget_color_picker_state(target.widget)
                .is_none_or(|state| !state.expanded);
            report.color_picker_expanded_change_count = 1;
            report.event_count = 1;
            report.events.push(format!(
                "win32_view_color_picker_expanded:{}:{expanded}",
                target.widget.0
            ));
            self.dispatch_event(
                crate::ViewEvent::ColorPickerExpandedChanged {
                    widget: target.widget,
                    expanded,
                },
                report,
            );
            return;
        }
        #[cfg(feature = "command-palette")]
        match target.kind {
            crate::ViewHitTargetKind::CommandPaletteItem { item } => {
                report.command_palette_invoke_count = 1;
                report.command_palette_open_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_invoke:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::CommandPaletteInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPaletteClear => {
                report.command_palette_query_change_count = 1;
                report.command_palette_clear_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_cleared:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TextChanged {
                        widget: target.widget,
                        value: String::new(),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPaletteScrim => {
                report.command_palette_open_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_command_palette_dismissed:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::CommandPaletteOpenChanged {
                        widget: target.widget,
                        open: false,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::CommandPalette => return,
            _ => {}
        }

        #[cfg(feature = "dialog")]
        match target.kind {
            crate::ViewHitTargetKind::ContentDialogButton { button } => {
                report.content_dialog_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_content_dialog_response:{}:{button:?}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ContentDialogResponded {
                        widget: target.widget,
                        button,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ContentDialog
            | crate::ViewHitTargetKind::ContentDialogScrim => return,
            _ => {}
        }
        #[cfg(feature = "toast")]
        match target.kind {
            crate::ViewHitTargetKind::ToastAction => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return;
                };
                let Some(toast) = state.toast else {
                    return;
                };
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ToastClose => {
                let Some((state, _)) = self.widget_toast_state(target.widget) else {
                    return;
                };
                let Some(toast) = state.toast else {
                    return;
                };
                report.toast_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_toast_response:{}:{toast:?}:CloseButton",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ToastResponded {
                        widget: target.widget,
                        toast,
                        response: crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::Toast => return,
            _ => {}
        }
        #[cfg(feature = "info-bar")]
        match target.kind {
            crate::ViewHitTargetKind::InfoBarAction => {
                report.info_bar_event_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_info_bar_event:{}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::InfoBarClose => {
                report.info_bar_event_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_info_bar_event:{}:Close",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::InfoBarInvoked {
                        widget: target.widget,
                        event: crate::ZsInfoBarEvent::Close,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::InfoBar => return,
            _ => {}
        }
        #[cfg(feature = "teaching-tip")]
        match target.kind {
            crate::ViewHitTargetKind::TeachingTipAction => {
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:Action",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response: crate::ZsTeachingTipResponse::Action,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TeachingTipClose => {
                let response = crate::ZsTeachingTipResponse::Dismissed(
                    crate::ZsTeachingTipDismissReason::CloseButton,
                );
                report.teaching_tip_response_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_teaching_tip_response:{}:{response:?}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TeachingTipResponded {
                        widget: target.widget,
                        response,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TeachingTip => return,
            _ => {}
        }
        #[cfg(feature = "breadcrumb")]
        match target.kind {
            crate::ViewHitTargetKind::BreadcrumbOverflow => {
                let expanded = self
                    .widget_breadcrumb_state(target.widget)
                    .map_or(true, |state| !state.overflow_open);
                report.breadcrumb_expanded_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_breadcrumb_expanded:{}:{}",
                    target.widget.0, expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::BreadcrumbExpandedChanged {
                        widget: target.widget,
                        expanded,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::BreadcrumbItem { item }
            | crate::ViewHitTargetKind::BreadcrumbOverflowItem { item } => {
                report.breadcrumb_selection_count = 1;
                report.breadcrumb_expanded_change_count = usize::from(
                    self.widget_breadcrumb_state(target.widget)
                        .is_some_and(|state| state.overflow_open),
                );
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_breadcrumb_selected:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::BreadcrumbSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::BreadcrumbBar => return,
            _ => {}
        }
        #[cfg(feature = "tree")]
        match target.kind {
            crate::ViewHitTargetKind::TreeNodeExpander { node } => {
                let Some(row) = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.row(node))
                else {
                    return;
                };
                if row.expandable {
                    report.tree_expansion_change_count = 1;
                    report.event_count = 1;
                    report.events.push(format!(
                        "win32_view_tree_expanded:{}:{}:{}",
                        target.widget.0,
                        node.get(),
                        !row.expanded
                    ));
                    self.dispatch_event(
                        crate::ViewEvent::TreeNodeExpandedChanged {
                            widget: target.widget,
                            node,
                            expanded: !row.expanded,
                        },
                        report,
                    );
                }
                return;
            }
            crate::ViewHitTargetKind::TreeNode { node } => {
                let selected = self
                    .widget_tree_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.tree_selection_count = usize::from(selected != Some(node));
                report.tree_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_tree_invoke:{}:{}",
                    target.widget.0,
                    node.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeSelected {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::TreeNodeInvoked {
                        widget: target.widget,
                        node,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "grid-view")]
        match target.kind {
            crate::ViewHitTargetKind::GridViewItem { item } => {
                let selected = self
                    .widget_grid_view_state(target.widget)
                    .and_then(|state| state.selected);
                report.grid_view_selection_count = usize::from(selected != Some(item));
                report.grid_view_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_grid_view_invoke:{}:{}",
                    target.widget.0,
                    item.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemSelected {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::GridViewItemInvoked {
                        widget: target.widget,
                        item,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::GridView => return,
            _ => {}
        }
        #[cfg(feature = "table")]
        match target.kind {
            crate::ViewHitTargetKind::TableHeader { column } => {
                report.table_sort_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_table_sort:{}:{}",
                    target.widget.0,
                    column.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableSorted {
                        widget: target.widget,
                        column,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TableRow { row } => {
                let selected = self
                    .widget_table_state(target.widget)
                    .and_then(|state| state.selected);
                report.table_selection_count = usize::from(selected != Some(row));
                report.table_invoke_count = 1;
                report.event_count = 2;
                report.events.push(format!(
                    "win32_view_table_invoke:{}:{}",
                    target.widget.0,
                    row.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::TableRowSelected {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
                self.dispatch_event(
                    crate::ViewEvent::TableRowInvoked {
                        widget: target.widget,
                        row,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "auto-suggest")]
        match target.kind {
            crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion } => {
                let expanded = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_submit:{}:{}",
                    target.widget.0,
                    suggestion.get()
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget: target.widget,
                        suggestion: Some(suggestion),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::AutoSuggestSearch => {
                let state = self.widget_auto_suggest_state(target.widget);
                report.auto_suggest_submit_count = 1;
                report.auto_suggest_expanded_change_count =
                    usize::from(state.as_ref().is_some_and(|state| state.expanded));
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_query_submit:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestSubmitted {
                        widget: target.widget,
                        suggestion: state.and_then(|state| state.highlighted),
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::AutoSuggestClear => {
                let expanded = self
                    .widget_auto_suggest_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.auto_suggest_clear_count = 1;
                report.auto_suggest_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_auto_suggest_cleared:{}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::AutoSuggestCleared {
                        widget: target.widget,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "number-box")]
        match target.kind {
            crate::ViewHitTargetKind::NumberBoxDecrement
            | crate::ViewHitTargetKind::NumberBoxIncrement => {
                let steps = if target.kind == crate::ViewHitTargetKind::NumberBoxIncrement {
                    1
                } else {
                    -1
                };
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_number_box_step:{}:{steps}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::NumberBoxStep {
                        widget: target.widget,
                        steps,
                        large: false,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "time-picker")]
        match target.kind {
            crate::ViewHitTargetKind::TimePickerChoice { value } => {
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_selected:{}:{value}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimeChanged {
                        widget: target.widget,
                        value,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::TimePicker => {
                let expanded = self
                    .widget_time_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_time_picker_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::TimePickerExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "date-picker")]
        match target.kind {
            crate::ViewHitTargetKind::DatePickerDay { date } => {
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_selected:{}:{date}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DateChanged {
                        widget: target.widget,
                        value: date,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::DatePickerPreviousMonth
            | crate::ViewHitTargetKind::DatePickerNextMonth => {
                let Some(state) = self.widget_date_picker_state(target.widget) else {
                    return;
                };
                let offset = if target.kind == crate::ViewHitTargetKind::DatePickerPreviousMonth {
                    -1
                } else {
                    1
                };
                let month = state.visible_month.add_months(offset);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_month:{}:{month}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerMonthChanged {
                        widget: target.widget,
                        month,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::DatePicker => {
                let expanded = self
                    .widget_date_picker_state(target.widget)
                    .is_some_and(|state| state.expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_date_picker_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::DatePickerExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "combo")]
        match target.kind {
            crate::ViewHitTargetKind::ComboBoxOption { index } => {
                let expanded = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
                report.combo_selection_count = 1;
                report.combo_expanded_change_count = usize::from(expanded);
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_selected:{}:{index}",
                    target.widget.0
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxSelected {
                        widget: target.widget,
                        index,
                    },
                    report,
                );
                return;
            }
            crate::ViewHitTargetKind::ComboBox => {
                let expanded = self
                    .widget_combo_state(target.widget)
                    .is_some_and(|(_, _, expanded)| expanded);
                report.combo_expanded_change_count = 1;
                report.event_count = 1;
                report.events.push(format!(
                    "win32_view_combo_expanded:{}:{}",
                    target.widget.0, !expanded
                ));
                self.dispatch_event(
                    crate::ViewEvent::ComboBoxExpandedChanged {
                        widget: target.widget,
                        expanded: !expanded,
                    },
                    report,
                );
                return;
            }
            _ => {}
        }
        #[cfg(feature = "radio")]
        if target.kind == crate::ViewHitTargetKind::RadioButton {
            report.radio_selection_count = 1;
            report
                .events
                .push(format!("win32_view_radio_selected:{}", target.widget.0));
            report.event_count = 1;
            self.dispatch_event(
                crate::ViewEvent::RadioSelected {
                    widget: target.widget,
                },
                report,
            );
            return;
        }
        #[cfg(feature = "tabs")]
        if let crate::ViewHitTargetKind::Tab { tab_view, tab, .. } = target.kind {
            report.tab_selection_count = usize::from(
                self.widget_tab_header_state(target.widget)
                    .is_some_and(|state| !state.selected),
            );
            report.event_count = 1;
            report
                .events
                .push(format!("win32_view_tab_selected:{}:{}", tab_view.0, tab.0));
            self.dispatch_event(
                crate::ViewEvent::TabSelected {
                    widget: tab_view,
                    tab,
                },
                report,
            );
            return;
        }
        let toggles = matches!(
            target.kind,
            crate::ViewHitTargetKind::Checkbox | crate::ViewHitTargetKind::Toggle
        );
        #[cfg(feature = "toggle-button")]
        let toggles = toggles || target.kind == crate::ViewHitTargetKind::ToggleButton;
        let event = if toggles {
            let checked = !self.widget_checked_value(target.widget).unwrap_or(false);
            report.toggle_count = 1;
            report
                .events
                .push(format!("win32_view_toggle:{}:{checked}", target.widget.0));
            crate::ViewEvent::Toggled {
                widget: target.widget,
                checked,
            }
        } else {
            report
                .events
                .push(format!("win32_view_click:{}", target.widget.0));
            #[cfg(feature = "list")]
            if let Some(index) = self.widget_list_index(target.widget) {
                report.selection_count = 1;
                report
                    .events
                    .push(format!("win32_view_select:{}:{index}", target.widget.0));
            }
            crate::ViewEvent::Click {
                widget: target.widget,
            }
        };
        report.event_count = 1;
        self.dispatch_event(event, report);
    }

}
