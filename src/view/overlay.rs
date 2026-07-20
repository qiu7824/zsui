impl<Msg> ViewNode<Msg> {
    fn event_targets_self(&self, event: &ViewEvent) -> bool {
        match (self.id, event) {
            (Some(id), ViewEvent::Click { widget })
            | (Some(id), ViewEvent::TextChanged { widget, .. })
            | (Some(id), ViewEvent::Toggled { widget, .. }) => id == *widget,
            #[cfg(feature = "canvas")]
            (Some(id), ViewEvent::CanvasPointer { event }) => id == event.widget,
            #[cfg(feature = "textbox")]
            (Some(id), ViewEvent::TextEdited { widget, .. })
            | (Some(id), ViewEvent::TextSelectionChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "password-box")]
            (Some(id), ViewEvent::PasswordChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "slider")]
            (Some(id), ViewEvent::SliderChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "number-box")]
            (Some(id), ViewEvent::NumberBoxStep { widget, .. })
            | (Some(id), ViewEvent::NumberBoxCommit { widget })
            | (Some(id), ViewEvent::NumberBoxReset { widget }) => id == *widget,
            #[cfg(feature = "radio")]
            (Some(id), ViewEvent::RadioSelected { widget }) => id == *widget,
            #[cfg(feature = "auto-suggest")]
            (Some(id), ViewEvent::AutoSuggestExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::AutoSuggestHighlighted { widget, .. })
            | (Some(id), ViewEvent::AutoSuggestCleared { widget })
            | (Some(id), ViewEvent::AutoSuggestSubmitted { widget, .. }) => id == *widget,
            #[cfg(feature = "tree")]
            (Some(id), ViewEvent::TreeNodeExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::TreeNodeSelected { widget, .. })
            | (Some(id), ViewEvent::TreeNodeInvoked { widget, .. }) => id == *widget,
            #[cfg(feature = "grid-view")]
            (Some(id), ViewEvent::GridViewItemSelected { widget, .. })
            | (Some(id), ViewEvent::GridViewItemInvoked { widget, .. }) => id == *widget,
            #[cfg(feature = "table")]
            (Some(id), ViewEvent::TableRowSelected { widget, .. })
            | (Some(id), ViewEvent::TableSorted { widget, .. })
            | (Some(id), ViewEvent::TableRowInvoked { widget, .. }) => id == *widget,
            #[cfg(feature = "dialog")]
            (Some(id), ViewEvent::ContentDialogFocused { widget, .. })
            | (Some(id), ViewEvent::ContentDialogResponded { widget, .. }) => id == *widget,
            #[cfg(feature = "command-palette")]
            (Some(id), ViewEvent::CommandPaletteHighlighted { widget, .. })
            | (Some(id), ViewEvent::CommandPaletteInvoked { widget, .. })
            | (Some(id), ViewEvent::CommandPaletteOpenChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "toast")]
            (Some(id), ViewEvent::ToastFocused { widget, .. })
            | (Some(id), ViewEvent::ToastResponded { widget, .. }) => id == *widget,
            #[cfg(feature = "teaching-tip")]
            (Some(id), ViewEvent::TeachingTipFocused { widget, .. })
            | (Some(id), ViewEvent::TeachingTipResponded { widget, .. }) => id == *widget,
            #[cfg(feature = "flyout")]
            (Some(id), ViewEvent::FlyoutDismissed { widget, .. }) => id == *widget,
            #[cfg(feature = "info-bar")]
            (Some(id), ViewEvent::InfoBarFocused { widget, .. })
            | (Some(id), ViewEvent::InfoBarInvoked { widget, .. }) => id == *widget,
            #[cfg(feature = "breadcrumb")]
            (Some(id), ViewEvent::BreadcrumbFocused { widget, .. })
            | (Some(id), ViewEvent::BreadcrumbExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::BreadcrumbSelected { widget, .. }) => id == *widget,
            #[cfg(feature = "combo")]
            (Some(id), ViewEvent::ComboBoxExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::ComboBoxSelected { widget, .. })
            | (Some(id), ViewEvent::ComboBoxScrolled { widget, .. }) => id == *widget,
            #[cfg(feature = "date-picker")]
            (Some(id), ViewEvent::DatePickerExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::DatePickerMonthChanged { widget, .. })
            | (Some(id), ViewEvent::DateChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "time-picker")]
            (Some(id), ViewEvent::TimePickerExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::TimeChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "color-picker")]
            (Some(id), ViewEvent::ColorPickerExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::ColorPickerChannelChanged { widget, .. })
            | (Some(id), ViewEvent::ColorChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "tabs")]
            (Some(id), ViewEvent::TabSelected { widget, .. }) => id == *widget,
            #[cfg(feature = "scroll")]
            (Some(id), ViewEvent::ScrollBy { widget, .. }) => id == *widget,
            #[cfg(any(
                feature = "auto-suggest",
                feature = "breadcrumb",
                feature = "color-picker",
                feature = "combo",
                feature = "date-picker",
                feature = "flyout",
                feature = "time-picker"
            ))]
            (Some(_), ViewEvent::DismissPopupOverlays { .. }) => false,
            (None, _) => false,
        }
    }

    #[cfg(any(feature = "list", feature = "scroll", feature = "tabs"))]
    fn contains_widget(&self, widget: WidgetId) -> bool {
        self.id == Some(widget)
            || self
                .children
                .iter()
                .any(|child| child.contains_widget(widget))
    }

    #[cfg(any(feature = "flyout", feature = "teaching-tip"))]
    fn widget_layout_bounds(&self, widget: WidgetId) -> Option<Rect> {
        if self.id == Some(widget) {
            return self.bounds;
        }
        self.children
            .iter()
            .find_map(|child| child.widget_layout_bounds(widget))
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        let mut hit_targets = Vec::new();
        self.collect_hit_targets(&mut hit_targets, None);
        #[cfg(any(
            feature = "auto-suggest",
            feature = "breadcrumb",
            feature = "color-picker",
            feature = "command-palette",
            feature = "combo",
            feature = "date-picker",
            feature = "dialog",
            feature = "flyout",
            feature = "teaching-tip",
            feature = "time-picker",
            feature = "toast"
        ))]
        self.collect_overlay_hit_targets(&mut hit_targets, None);
        #[cfg(feature = "tooltip")]
        let mut tooltip_targets = Vec::new();
        #[cfg(feature = "tooltip")]
        self.collect_tooltip_targets(&mut tooltip_targets, None);
        ViewInteractionPlan {
            hit_targets,
            #[cfg(feature = "tooltip")]
            tooltip_targets,
        }
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<&str> {
        if self.id == Some(widget) {
            #[cfg(feature = "textbox")]
            if let ViewNodeKind::Textbox { value, .. } = &self.kind {
                return Some(value);
            }
            #[cfg(feature = "number-box")]
            if let ViewNodeKind::NumberBox { draft, .. } = &self.kind {
                return Some(draft);
            }
            #[cfg(feature = "auto-suggest")]
            if let ViewNodeKind::AutoSuggestBox { query, .. } = &self.kind {
                return Some(query);
            }
            #[cfg(feature = "command-palette")]
            if let ViewNodeKind::CommandPalette { query, .. } = &self.kind {
                return Some(query);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_text_value(widget))
    }

    #[cfg(feature = "textbox")]
    pub fn widget_text_wrap(&self, widget: WidgetId) -> Option<crate::TextWrap> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Textbox { wrap, .. } = &self.kind {
                return Some(*wrap);
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_text_wrap(widget))
    }

    #[cfg(feature = "password-box")]
    pub fn widget_password_value(&self, widget: WidgetId) -> Option<&crate::ZsPassword> {
        if self.id == Some(widget) {
            if let ViewNodeKind::PasswordBox { value, .. } = &self.kind {
                return Some(value);
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_password_value(widget))
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        if self.id == Some(widget) {
            #[cfg(feature = "checkbox")]
            if let ViewNodeKind::Checkbox { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "toggle-button")]
            if let ViewNodeKind::ToggleButton { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "toggle")]
            if let ViewNodeKind::Toggle { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "radio")]
            if let ViewNodeKind::RadioButton { selected, .. } = &self.kind {
                return Some(*selected);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_checked_value(widget))
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        if matches!(&self.kind, ViewNodeKind::Stack { .. }) {
            let mut radio_widgets = self.children.iter().filter_map(|child| {
                if let ViewNodeKind::RadioButton { selected, .. } = &child.kind {
                    child.id.map(|id| (id, *selected))
                } else {
                    None
                }
            });
            if let Some(first) = radio_widgets.next() {
                let mut group = vec![first];
                group.extend(radio_widgets);
                if group.iter().any(|(candidate, _)| *candidate == widget) {
                    let tab_stop = group
                        .iter()
                        .find_map(|(candidate, selected)| selected.then_some(*candidate))
                        .unwrap_or(first.0);
                    return Some(widget == tab_stop);
                }
            }
        }
        if self.id == Some(widget) && matches!(&self.kind, ViewNodeKind::RadioButton { .. }) {
            return Some(true);
        }
        self.children
            .iter()
            .find_map(|child| child.widget_radio_is_tab_stop(widget))
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        if let ViewNodeKind::Stack { direction } = &self.kind {
            let navigation_supported = match direction {
                ViewStackDirection::Column => navigation == ViewStackDirection::Column,
                ViewStackDirection::Row => true,
            };
            let radio_widgets = self
                .children
                .iter()
                .filter_map(|child| {
                    matches!(&child.kind, ViewNodeKind::RadioButton { .. })
                        .then_some(child.id)
                        .flatten()
                })
                .collect::<Vec<_>>();
            if let Some(index) = radio_widgets
                .iter()
                .position(|candidate| *candidate == widget)
            {
                if navigation_supported {
                    let next_index = index as isize + offset;
                    return Some(
                        usize::try_from(next_index)
                            .ok()
                            .and_then(|index| radio_widgets.get(index).copied())
                            .unwrap_or(widget),
                    );
                }
                return Some(widget);
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_radio_relative_widget(widget, navigation, offset))
    }

    #[cfg(feature = "slider")]
    pub fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Slider { value, range, .. } = &self.kind {
                return Some((*value, *range));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_slider_state(widget))
    }

    #[cfg(feature = "number-box")]
    pub fn widget_number_box_state(&self, widget: WidgetId) -> Option<ZsNumberBoxState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::NumberBox {
                value,
                draft,
                range,
                format,
                ..
            } = &self.kind
            {
                let valid = draft.trim().is_empty()
                    || format
                        .parse(draft)
                        .is_some_and(|candidate| range.contains(candidate));
                return Some(ZsNumberBoxState {
                    value: *value,
                    draft: draft.clone(),
                    valid,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_number_box_state(widget))
    }

    #[cfg(feature = "combo")]
    pub fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded,
                ..
            } = &self.kind
            {
                return Some((*selected_index, options.len(), *expanded));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_combo_state(widget))
    }

    #[cfg(feature = "auto-suggest")]
    pub fn widget_auto_suggest_state(&self, widget: WidgetId) -> Option<crate::ZsAutoSuggestState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::AutoSuggestBox {
                query,
                suggestions,
                highlighted,
                expanded,
                ..
            } = &self.kind
            {
                return Some(crate::ZsAutoSuggestState {
                    query: query.clone(),
                    suggestion_ids: suggestions.iter().map(|item| item.id()).collect(),
                    highlighted: *highlighted,
                    expanded: *expanded,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_auto_suggest_state(widget))
    }

    #[cfg(feature = "tree")]
    pub fn widget_tree_view_state(&self, widget: WidgetId) -> Option<crate::ZsTreeViewState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::TreeView {
                roots,
                expanded,
                selected,
                ..
            } = &self.kind
            {
                return Some(crate::tree::tree_view_state(roots, expanded, *selected));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tree_view_state(widget))
    }

    #[cfg(feature = "grid-view")]
    pub fn widget_grid_view_state(&self, widget: WidgetId) -> Option<crate::ZsGridViewState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::GridView {
                items, selected, ..
            } = &self.kind
            {
                let column_count = self
                    .bounds
                    .map(|bounds| {
                        crate::zs_grid_view_render_plan(
                            bounds,
                            items,
                            *selected,
                            crate::ZsGridViewPlatformStyle::current(),
                            self.layout_dpi,
                        )
                        .column_count
                    })
                    .unwrap_or(1);
                return Some(crate::grid_view::grid_view_state(
                    items,
                    *selected,
                    column_count,
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_grid_view_state(widget))
    }

    #[cfg(feature = "table")]
    pub fn widget_table_state(&self, widget: WidgetId) -> Option<crate::ZsTableViewState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::DataGrid {
                rows,
                selected,
                sort,
                ..
            } = &self.kind
            {
                return Some(crate::table::table_view_state(rows, *selected, *sort));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_table_state(widget))
    }

    #[cfg(feature = "dialog")]
    pub fn widget_content_dialog_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsContentDialogState, crate::ZsContentDialogSpec)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ContentDialog {
                spec,
                open,
                focused_button,
                ..
            } = &self.kind
            {
                return Some((
                    crate::ZsContentDialogState {
                        open: *open,
                        focused_button: *focused_button,
                    },
                    spec.clone(),
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_content_dialog_state(widget))
    }

    #[cfg(feature = "command-palette")]
    pub fn widget_command_palette_state(
        &self,
        widget: WidgetId,
    ) -> Option<crate::ZsCommandPaletteState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::CommandPalette {
                items,
                query,
                highlighted,
                open,
                ..
            } = &self.kind
            {
                return Some(crate::command_palette::command_palette_state(
                    *open,
                    query,
                    items,
                    *highlighted,
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_command_palette_state(widget))
    }

    #[cfg(feature = "flyout")]
    pub fn widget_flyout_state(&self, widget: WidgetId) -> Option<crate::ZsFlyoutState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Flyout { open, target, .. } = &self.kind {
                return Some(crate::ZsFlyoutState {
                    open: *open,
                    target: *target,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_flyout_state(widget))
    }

    #[cfg(feature = "toast")]
    pub fn widget_toast_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsToastState, crate::ZsToastSpec)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ToastPresenter {
                toast: Some(toast),
                focused_control,
                ..
            } = &self.kind
            {
                return Some((
                    crate::ZsToastState {
                        toast: Some(toast.id()),
                        focused_control: *focused_control,
                    },
                    toast.clone(),
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_toast_state(widget))
    }

    #[cfg(feature = "teaching-tip")]
    pub fn widget_teaching_tip_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsTeachingTipState, crate::ZsTeachingTipSpec)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::TeachingTip {
                spec,
                open,
                target,
                focused_control,
                ..
            } = &self.kind
            {
                return Some((
                    crate::ZsTeachingTipState {
                        open: *open,
                        target: *target,
                        focused_control: *focused_control,
                    },
                    spec.clone(),
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_teaching_tip_state(widget))
    }

    #[cfg(feature = "info-bar")]
    pub fn widget_info_bar_state(
        &self,
        widget: WidgetId,
    ) -> Option<(crate::ZsInfoBarState, crate::ZsInfoBarSpec)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::InfoBar {
                spec,
                focused_control,
                ..
            } = &self.kind
            {
                return Some((
                    crate::ZsInfoBarState {
                        focused_control: *focused_control,
                    },
                    spec.clone(),
                ));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_info_bar_state(widget))
    }

    #[cfg(feature = "breadcrumb")]
    pub fn widget_breadcrumb_state(&self, widget: WidgetId) -> Option<crate::ZsBreadcrumbState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::BreadcrumbBar {
                items,
                overflow_open,
                focused,
                ..
            } = &self.kind
            {
                return Some(crate::ZsBreadcrumbState {
                    items: items.clone(),
                    overflow_open: *overflow_open,
                    focused: *focused,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_breadcrumb_state(widget))
    }

    #[cfg(feature = "combo")]
    pub(crate) fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ComboBox { options, .. } = &self.kind {
                if query.is_empty() || options.is_empty() {
                    return None;
                }
                let query = query.to_lowercase();
                let start = start_after
                    .filter(|index| *index < options.len())
                    .map_or(0, |index| (index + 1) % options.len());
                return (0..options.len())
                    .map(|offset| (start + offset) % options.len())
                    .find(|index| options[*index].to_lowercase().starts_with(&query));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_combo_type_ahead_match(widget, query, start_after))
    }

    #[cfg(feature = "date-picker")]
    pub fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                expanded,
                ..
            } = &self.kind
            {
                return Some(ZsDatePickerState {
                    value: *value,
                    minimum: *minimum,
                    maximum: *maximum,
                    visible_month: *visible_month,
                    expanded: *expanded,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_date_picker_state(widget))
    }

    #[cfg(feature = "time-picker")]
    pub fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded,
                ..
            } = &self.kind
            {
                return Some(ZsTimePickerState {
                    value: *value,
                    minute_increment: *minute_increment,
                    clock: *clock,
                    expanded: *expanded,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_time_picker_state(widget))
    }

    #[cfg(feature = "color-picker")]
    pub fn widget_color_picker_state(&self, widget: WidgetId) -> Option<ZsColorPickerState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ColorPicker { state, .. } = &self.kind {
                return Some(state.normalized());
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_color_picker_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub fn widget_tab_view_state(&self, widget: WidgetId) -> Option<ZsTabViewState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Tabs { tabs, selected, .. } = &self.kind {
                return Some(ZsTabViewState {
                    selected: *selected,
                    tab_count: tabs.len(),
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_view_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        if let (Some(tab_view), ViewNodeKind::Tabs { tabs, selected, .. }) = (self.id, &self.kind) {
            if let Some((index, tab)) = tabs
                .iter()
                .enumerate()
                .find(|(_, tab)| WidgetId(tab.id.0) == widget)
            {
                return Some(ZsTabHeaderState {
                    tab_view,
                    tab: tab.id,
                    selected: *selected == Some(tab.id),
                    previous: index
                        .checked_sub(1)
                        .and_then(|index| tabs.get(index))
                        .map(|tab| WidgetId(tab.id.0)),
                    next: tabs
                        .get(index.saturating_add(1))
                        .map(|tab| WidgetId(tab.id.0)),
                    first: WidgetId(tabs.first().expect("matched tab list is non-empty").id.0),
                    last: WidgetId(tabs.last().expect("matched tab list is non-empty").id.0),
                });
            }
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            return selected_index
                .and_then(|index| self.children.get(index))
                .and_then(|child| child.widget_tab_header_state(widget));
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_header_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        if let (Some(tab_view), ViewNodeKind::Tabs { tabs, selected, .. }) = (self.id, &self.kind) {
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            let focused_header = tabs.iter().any(|tab| WidgetId(tab.id.0) == focused);
            let focused_content = selected_index
                .and_then(|index| self.children.get(index))
                .is_some_and(|child| child.contains_widget(focused));
            if (focused_header || focused_content) && !tabs.is_empty() {
                let current = selected_index.unwrap_or(0) as isize;
                let next = (current + offset).rem_euclid(tabs.len() as isize) as usize;
                return Some((tab_view, tabs[next].id));
            }
            return selected_index
                .and_then(|index| self.children.get(index))
                .and_then(|child| child.widget_tab_cycle_target(focused, offset));
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_cycle_target(focused, offset))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            return self
                .children
                .iter()
                .position(|child| child.contains_widget(widget));
        }
        #[cfg(feature = "virtual-list")]
        if let ViewNodeKind::VirtualList { row_indices, .. } = &self.kind {
            let position = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            return row_indices.get(position).copied();
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_index(widget))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            let current = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            let next = current
                .saturating_add_signed(offset)
                .min(self.children.len().saturating_sub(1));
            if next == current {
                return None;
            }
            return self.children[next]
                .first_widget_id()
                .map(|widget| (widget, next));
        }
        #[cfg(feature = "virtual-list")]
        if let ViewNodeKind::VirtualList { row_indices, .. } = &self.kind {
            let current = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            let next = current
                .saturating_add_signed(offset)
                .min(self.children.len().saturating_sub(1));
            if next == current {
                return None;
            }
            let index = *row_indices.get(next)?;
            return self.children[next]
                .first_widget_id()
                .map(|widget| (widget, index));
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_relative_widget(widget, offset))
    }

    #[cfg(feature = "list")]
    fn first_widget_id(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id))
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        let is_scroll_target = matches!(self.kind, ViewNodeKind::Scroll { .. });
        #[cfg(feature = "virtual-list")]
        let is_scroll_target =
            is_scroll_target || matches!(self.kind, ViewNodeKind::VirtualList { .. });
        if is_scroll_target && self.contains_widget(widget) {
            return self.id.or_else(|| self.first_widget_id_any());
        }

        self.children
            .iter()
            .find_map(|child| child.widget_scroll_target(widget))
    }

    #[cfg(feature = "scroll")]
    fn first_widget_id_any(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id_any))
    }

    fn collect_hit_targets(&self, hit_targets: &mut Vec<ViewHitTarget>, clip: Option<Rect>) {
        #[cfg(feature = "flyout")]
        if matches!(self.kind, ViewNodeKind::Flyout { .. }) {
            if let Some(page) = self.children.first() {
                page.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "label")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::NavigationView {
                item_count,
                footer_count,
                pane_open,
                pane_width,
                minimum_content_width,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let platform = self.resolved_platform_style();
            let layout = zs_navigation_view_layout(
                bounds,
                platform,
                *pane_width,
                *minimum_content_width,
                *pane_open,
                self.layout_dpi,
                1.0,
            );
            if let Some(scrim) = layout
                .scrim_bounds
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    scrim,
                    ViewHitTargetKind::NavigationViewScrim,
                ));
            }
            if let Some(toggle) = layout
                .toggle_bounds
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    toggle,
                    ViewHitTargetKind::NavigationViewToggle,
                ));
            }
            let content_index = item_count.saturating_add(*footer_count);
            let children = if layout.overlay_open {
                &self.children[..content_index.min(self.children.len())]
            } else {
                self.children.as_slice()
            };
            for child in children {
                child.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "breadcrumb")]
        if let (Some(widget), Some(bounds), ViewNodeKind::BreadcrumbBar { items, .. }) =
            (self.id, self.bounds, &self.kind)
        {
            if items.is_empty() {
                return;
            }
            let plan = crate::zs_breadcrumb_render_plan(
                bounds,
                items,
                false,
                crate::ZsBreadcrumbPlatformStyle::current(),
                self.layout_dpi,
                None,
            );
            if let Some(bounds) = clipped_rect(bounds, clip) {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::BreadcrumbBar,
                ));
            }
            for item in &plan.items {
                if let (Some(spec), Some(bounds)) =
                    (items.get(item.item_index), clipped_rect(item.bounds, clip))
                {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        ViewHitTargetKind::BreadcrumbItem { item: spec.id() },
                    ));
                }
            }
            if let Some(bounds) = plan
                .overflow_bounds
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::BreadcrumbOverflow,
                ));
            }
            return;
        }

        #[cfg(feature = "grid-view")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::GridView {
                items, selected, ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            if items.is_empty() {
                return;
            }
            let grid_clip = clipped_rect(bounds, clip);
            if let Some(root_bounds) = grid_clip {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    root_bounds,
                    ViewHitTargetKind::GridView,
                ));
            }
            let plan = crate::zs_grid_view_render_plan(
                bounds,
                items,
                *selected,
                crate::ZsGridViewPlatformStyle::current(),
                self.layout_dpi,
            );
            for item in plan
                .items
                .iter()
                .filter(|item| item.selected)
                .chain(plan.items.iter().filter(|item| !item.selected))
            {
                if let (Some(spec), Some(item_bounds)) = (
                    items.get(item.item_index),
                    clipped_rect(item.bounds, grid_clip),
                ) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        item_bounds,
                        ViewHitTargetKind::GridViewItem { item: spec.id() },
                    ));
                }
            }
            return;
        }

        #[cfg(feature = "info-bar")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::InfoBar {
                spec,
                focused_control,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            if focused_control.is_some() {
                if let Some(root_bounds) = clipped_rect(bounds, clip) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        root_bounds,
                        ViewHitTargetKind::InfoBar,
                    ));
                }
                let plan = crate::zs_info_bar_render_plan(
                    bounds,
                    spec,
                    *focused_control,
                    crate::ZsInfoBarPlatformStyle::current(),
                    self.layout_dpi,
                );
                if let Some(action_bounds) = plan
                    .action_bounds
                    .and_then(|bounds| clipped_rect(bounds, clip))
                {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        action_bounds,
                        ViewHitTargetKind::InfoBarAction,
                    ));
                }
                if let Some(close_bounds) = plan
                    .close_bounds
                    .and_then(|bounds| clipped_rect(bounds, clip))
                {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        close_bounds,
                        ViewHitTargetKind::InfoBarClose,
                    ));
                }
            }
            return;
        }

        #[cfg(feature = "teaching-tip")]
        if matches!(self.kind, ViewNodeKind::TeachingTip { .. }) {
            for child in &self.children {
                child.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "toast")]
        if matches!(self.kind, ViewNodeKind::ToastPresenter { .. }) {
            for child in &self.children {
                child.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "command-palette")]
        if matches!(self.kind, ViewNodeKind::CommandPalette { .. }) {
            for child in &self.children {
                child.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "dialog")]
        if matches!(self.kind, ViewNodeKind::ContentDialog { .. }) {
            for child in &self.children {
                child.collect_hit_targets(hit_targets, clip);
            }
            return;
        }

        #[cfg(feature = "tabs")]
        if let (Some(tab_view), Some(bounds), ViewNodeKind::Tabs { tabs, selected, .. }) =
            (self.id, self.bounds, &self.kind)
        {
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            let plan = crate::zs_tab_view_render_plan_for_tabs(
                bounds,
                tabs,
                selected_index,
                crate::ZsTabPlatformStyle::current(),
                self.layout_dpi,
            );
            hit_targets.extend(plan.headers.iter().zip(tabs).filter_map(|(header, tab)| {
                clipped_rect(header.bounds, clip).map(|bounds| {
                    ViewHitTarget::with_kind(
                        WidgetId(tab.id.0),
                        bounds,
                        ViewHitTargetKind::Tab {
                            tab_view,
                            tab: tab.id,
                            index: tabs
                                .iter()
                                .position(|candidate| candidate.id == tab.id)
                                .unwrap_or(0),
                        },
                    )
                })
            }));
            if let Some(child) = selected_index.and_then(|index| self.children.get(index)) {
                child.collect_hit_targets(hit_targets, clipped_rect(plan.content_bounds, clip));
            }
            return;
        }

        #[allow(unused_mut)]
        let mut accepts_input = true;
        #[cfg(feature = "button")]
        {
            accepts_input &= !matches!(
                self.kind,
                ViewNodeKind::Button { enabled: false, .. }
            );
        }
        #[cfg(feature = "progress")]
        {
            accepts_input &= !matches!(self.kind, ViewNodeKind::ProgressBar { .. });
        }
        #[cfg(feature = "progress-ring")]
        {
            accepts_input &= !matches!(self.kind, ViewNodeKind::ProgressRing { .. });
        }
        if accepts_input {
            if let (Some(widget), Some(bounds)) = (self.id, self.bounds) {
                if let Some(bounds) = clipped_rect(bounds, clip) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        self.hit_target_kind(),
                    ));
                }
            }
        }

        #[cfg(feature = "number-box")]
        if let (Some(widget), Some(bounds), ViewNodeKind::NumberBox { .. }) =
            (self.id, self.bounds, &self.kind)
        {
            let plan = crate::zs_number_box_render_plan(
                bounds,
                crate::ZsNumberBoxPlatformStyle::current(),
                self.layout_dpi,
            );
            if let Some(bounds) = clipped_rect(plan.decrement_button, clip) {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::NumberBoxDecrement,
                ));
            }
            if let Some(bounds) = clipped_rect(plan.increment_button, clip) {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::NumberBoxIncrement,
                ));
            }
        }

        #[cfg(feature = "password-box")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::PasswordBox {
                value, reveal_mode, ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = crate::zs_password_box_render_plan(
                bounds,
                *reveal_mode,
                !value.is_empty(),
                crate::ZsPasswordBoxPlatformStyle::current(),
                self.layout_dpi,
            );
            if let Some(bounds) = plan
                .reveal_button
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::PasswordBoxReveal,
                ));
            }
        }

        #[cfg(feature = "table")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::DataGrid {
                columns,
                rows,
                selected,
                sort,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = crate::zs_table_render_plan(
                bounds,
                columns,
                rows,
                *selected,
                *sort,
                crate::ZsTablePlatformStyle::current(),
                self.layout_dpi,
            );
            let table_clip = clipped_rect(bounds, clip);
            for column in plan.columns {
                if column.sortable {
                    if let Some(header_bounds) = clipped_rect(column.bounds, table_clip) {
                        hit_targets.push(ViewHitTarget::with_kind(
                            widget,
                            header_bounds,
                            ViewHitTargetKind::TableHeader {
                                column: column.column,
                            },
                        ));
                    }
                }
            }
            for row in plan.rows {
                if let Some(row_bounds) = clipped_rect(row.bounds, table_clip) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        row_bounds,
                        ViewHitTargetKind::TableRow { row: row.row },
                    ));
                }
            }
        }

        #[cfg(feature = "auto-suggest")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::AutoSuggestBox {
                query,
                suggestions,
                highlighted,
                no_results_text,
                query_icon,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let row_count = if suggestions.is_empty() && no_results_text.is_some() {
                1
            } else {
                suggestions.len()
            };
            let highlighted_index = highlighted.and_then(|highlighted| {
                suggestions
                    .iter()
                    .position(|candidate| candidate.id() == highlighted)
            });
            let plan = crate::zs_auto_suggest_render_plan(
                bounds,
                row_count,
                highlighted_index,
                false,
                query.is_empty(),
                *query_icon,
                crate::ZsAutoSuggestPlatformStyle::current(),
                self.layout_dpi,
            );
            if let Some(bounds) = plan
                .search_button
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::AutoSuggestSearch,
                ));
            }
            if let Some(bounds) = plan
                .clear_button
                .and_then(|bounds| clipped_rect(bounds, clip))
            {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::AutoSuggestClear,
                ));
            }
        }

        #[cfg(feature = "tree")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::TreeView {
                roots,
                expanded,
                selected,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = crate::zs_tree_view_render_plan(
                bounds,
                roots,
                expanded,
                *selected,
                crate::ZsTreePlatformStyle::current(),
                self.layout_dpi,
            );
            let tree_clip = clipped_rect(bounds, clip);
            for row in plan.rows {
                if let Some(row_bounds) = clipped_rect(row.bounds, tree_clip) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        row_bounds,
                        ViewHitTargetKind::TreeNode { node: row.node },
                    ));
                }
                if let Some(disclosure) = row
                    .disclosure_bounds
                    .and_then(|bounds| clipped_rect(bounds, tree_clip))
                {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        disclosure,
                        ViewHitTargetKind::TreeNodeExpander { node: row.node },
                    ));
                }
            }
        }

        #[cfg(feature = "scroll")]
        let clips_children = matches!(self.kind, ViewNodeKind::Scroll { .. });
        #[cfg(all(feature = "scroll", feature = "virtual-list"))]
        let clips_children =
            clips_children || matches!(self.kind, ViewNodeKind::VirtualList { .. });
        #[cfg(feature = "scroll")]
        let child_clip = if clips_children {
            self.bounds.and_then(|bounds| clipped_rect(bounds, clip))
        } else {
            clip
        };
        #[cfg(not(feature = "scroll"))]
        let child_clip = clip;

        for child in &self.children {
            child.collect_hit_targets(hit_targets, child_clip);
        }
    }

    #[cfg(feature = "tooltip")]
    fn collect_tooltip_targets(
        &self,
        tooltip_targets: &mut Vec<ViewTooltipTarget>,
        clip: Option<Rect>,
    ) {
        if let (Some(widget), Some(bounds), Some(spec)) = (self.id, self.bounds, &self.tooltip) {
            if !spec.is_empty() {
                if let Some(bounds) = clipped_rect(bounds, clip) {
                    tooltip_targets.push(ViewTooltipTarget {
                        widget,
                        bounds,
                        spec: spec.clone(),
                    });
                }
            }
        }

        #[cfg(feature = "tabs")]
        if let (Some(bounds), ViewNodeKind::Tabs { tabs, selected, .. }) = (self.bounds, &self.kind)
        {
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            let plan = crate::zs_tab_view_render_plan_for_tabs(
                bounds,
                tabs,
                selected_index,
                crate::ZsTabPlatformStyle::current(),
                self.layout_dpi,
            );
            if let Some(child) = selected_index.and_then(|index| self.children.get(index)) {
                child.collect_tooltip_targets(
                    tooltip_targets,
                    clipped_rect(plan.content_bounds, clip),
                );
            }
            return;
        }

        #[cfg(feature = "scroll")]
        let clips_children = matches!(self.kind, ViewNodeKind::Scroll { .. });
        #[cfg(all(feature = "scroll", feature = "virtual-list"))]
        let clips_children =
            clips_children || matches!(self.kind, ViewNodeKind::VirtualList { .. });
        #[cfg(feature = "scroll")]
        let child_clip = if clips_children {
            self.bounds.and_then(|bounds| clipped_rect(bounds, clip))
        } else {
            clip
        };
        #[cfg(not(feature = "scroll"))]
        let child_clip = clip;

        for child in &self.children {
            child.collect_tooltip_targets(tooltip_targets, child_clip);
        }
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
        feature = "command-palette",
        feature = "combo",
        feature = "date-picker",
        feature = "dialog",
        feature = "flyout",
        feature = "teaching-tip",
        feature = "time-picker",
        feature = "toast"
    ))]
    fn collect_overlay_hit_targets(
        &self,
        hit_targets: &mut Vec<ViewHitTarget>,
        viewport: Option<Rect>,
    ) {
        #[cfg(feature = "flyout")]
        if let ViewNodeKind::Flyout {
            spec,
            open,
            target,
            ..
        } = &self.kind
        {
            let flyout_viewport = viewport.or(self.bounds);
            if let Some(page) = self.children.first() {
                page.collect_overlay_hit_targets(hit_targets, flyout_viewport);
            }
            let target_bounds = self
                .children
                .first()
                .and_then(|page| page.widget_layout_bounds(*target));
            if let (true, Some(widget), Some(viewport), Some(target_bounds)) =
                (*open, self.id, flyout_viewport, target_bounds)
            {
                let plan = crate::zs_flyout_render_plan(
                    viewport,
                    target_bounds,
                    *spec,
                    crate::ZsFlyoutPlatformStyle::current(),
                    self.layout_dpi,
                );
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    viewport,
                    ViewHitTargetKind::FlyoutScrim,
                ));
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.surface,
                    ViewHitTargetKind::Flyout,
                ));
                if let Some(content) = self.children.get(1) {
                    content.collect_hit_targets(hit_targets, Some(plan.content));
                    content.collect_overlay_hit_targets(hit_targets, Some(plan.content));
                }
            }
            return;
        }

        #[cfg(feature = "breadcrumb")]
        if let ViewNodeKind::BreadcrumbBar {
            items,
            overflow_open,
            ..
        } = &self.kind
        {
            if let (Some(widget), Some(bounds)) = (self.id, self.bounds) {
                let plan = crate::zs_breadcrumb_render_plan(
                    bounds,
                    items,
                    *overflow_open,
                    crate::ZsBreadcrumbPlatformStyle::current(),
                    self.layout_dpi,
                    viewport.or(self.bounds),
                );
                for row in &plan.popup_rows {
                    if let Some(item) = items.get(row.item_index) {
                        hit_targets.push(ViewHitTarget::with_kind(
                            widget,
                            row.bounds,
                            ViewHitTargetKind::BreadcrumbOverflowItem { item: item.id() },
                        ));
                    }
                }
            }
            return;
        }

        #[cfg(feature = "teaching-tip")]
        if let ViewNodeKind::TeachingTip {
            spec,
            open,
            target,
            focused_control,
            ..
        } = &self.kind
        {
            let tip_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.collect_overlay_hit_targets(hit_targets, tip_viewport);
            }
            let target_bounds = self
                .children
                .iter()
                .find_map(|child| child.widget_layout_bounds(*target));
            if let (true, Some(widget), Some(viewport), Some(target_bounds)) =
                (*open, self.id, tip_viewport, target_bounds)
            {
                let plan = crate::zs_teaching_tip_render_plan(
                    viewport,
                    target_bounds,
                    spec,
                    *focused_control,
                    crate::ZsTeachingTipPlatformStyle::current(),
                    self.layout_dpi,
                );
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.surface,
                    ViewHitTargetKind::TeachingTip,
                ));
                if let Some(bounds) = plan.action_bounds {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        ViewHitTargetKind::TeachingTipAction,
                    ));
                }
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.close_bounds,
                    ViewHitTargetKind::TeachingTipClose,
                ));
            }
            return;
        }

        #[cfg(feature = "toast")]
        if let ViewNodeKind::ToastPresenter {
            toast,
            focused_control,
            ..
        } = &self.kind
        {
            let toast_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.collect_overlay_hit_targets(hit_targets, toast_viewport);
            }
            if let (Some(spec), Some(widget), Some(viewport)) =
                (toast.as_ref(), self.id, toast_viewport)
            {
                let plan = crate::zs_toast_render_plan(
                    viewport,
                    spec,
                    *focused_control,
                    crate::ZsToastPlatformStyle::current(),
                    self.layout_dpi,
                );
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.surface,
                    ViewHitTargetKind::Toast,
                ));
                if let Some(bounds) = plan.action_bounds {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        ViewHitTargetKind::ToastAction,
                    ));
                }
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.close_bounds,
                    ViewHitTargetKind::ToastClose,
                ));
            }
            return;
        }

        #[cfg(feature = "command-palette")]
        if let ViewNodeKind::CommandPalette {
            items,
            query,
            highlighted,
            open,
            ..
        } = &self.kind
        {
            let palette_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.collect_overlay_hit_targets(hit_targets, palette_viewport);
            }
            if let (true, Some(widget), Some(viewport)) = (*open, self.id, palette_viewport) {
                let plan = crate::zs_command_palette_render_plan(
                    viewport,
                    query,
                    items,
                    *highlighted,
                    crate::ZsCommandPalettePlatformStyle::current(),
                    self.layout_dpi,
                );
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    viewport,
                    ViewHitTargetKind::CommandPaletteScrim,
                ));
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.search_bounds,
                    ViewHitTargetKind::CommandPalette,
                ));
                if let Some(bounds) = plan.clear_bounds {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        ViewHitTargetKind::CommandPaletteClear,
                    ));
                }
                hit_targets.extend(plan.rows.into_iter().filter(|row| row.enabled).map(|row| {
                    ViewHitTarget::with_kind(
                        widget,
                        row.bounds,
                        ViewHitTargetKind::CommandPaletteItem { item: row.item },
                    )
                }));
            }
            return;
        }

        #[cfg(feature = "dialog")]
        if let ViewNodeKind::ContentDialog {
            spec,
            open,
            focused_button,
            ..
        } = &self.kind
        {
            let dialog_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.collect_overlay_hit_targets(hit_targets, dialog_viewport);
            }
            if let (true, Some(widget), Some(viewport)) = (*open, self.id, dialog_viewport) {
                let plan = crate::zs_content_dialog_render_plan(
                    viewport,
                    spec,
                    *focused_button,
                    crate::ZsContentDialogPlatformStyle::current(),
                    self.layout_dpi,
                );
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    viewport,
                    ViewHitTargetKind::ContentDialogScrim,
                ));
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    plan.surface,
                    ViewHitTargetKind::ContentDialog,
                ));
                hit_targets.extend(plan.buttons.into_iter().map(|button| {
                    ViewHitTarget::with_kind(
                        widget,
                        button.bounds,
                        ViewHitTargetKind::ContentDialogButton {
                            button: button.button,
                        },
                    )
                }));
            }
            return;
        }

        #[cfg(feature = "auto-suggest")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::AutoSuggestBox {
                query,
                suggestions,
                highlighted,
                expanded: true,
                no_results_text,
                query_icon,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let row_count = if suggestions.is_empty() && no_results_text.is_some() {
                1
            } else {
                suggestions.len()
            };
            let highlighted_index = highlighted.and_then(|highlighted| {
                suggestions
                    .iter()
                    .position(|candidate| candidate.id() == highlighted)
            });
            let plan = viewport.map_or_else(
                || {
                    crate::zs_auto_suggest_render_plan(
                        bounds,
                        row_count,
                        highlighted_index,
                        true,
                        query.is_empty(),
                        *query_icon,
                        crate::ZsAutoSuggestPlatformStyle::current(),
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_auto_suggest_render_plan_in_viewport(
                        bounds,
                        row_count,
                        highlighted_index,
                        true,
                        query.is_empty(),
                        *query_icon,
                        crate::ZsAutoSuggestPlatformStyle::current(),
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            hit_targets.extend(
                suggestions
                    .iter()
                    .skip(plan.first_visible_suggestion)
                    .zip(plan.suggestion_rows)
                    .map(|(suggestion, bounds)| {
                        ViewHitTarget::with_kind(
                            widget,
                            bounds,
                            ViewHitTargetKind::AutoSuggestSuggestion {
                                suggestion: suggestion.id(),
                            },
                        )
                    }),
            );
        }
        #[cfg(feature = "combo")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_combo_box_render_plan_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_combo_box_render_plan_in_viewport_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            hit_targets.extend(
                plan.option_rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, bounds)| {
                        ViewHitTarget::with_kind(
                            widget,
                            bounds,
                            ViewHitTargetKind::ComboBoxOption {
                                index: plan.first_visible_option.saturating_add(index),
                            },
                        )
                    }),
            );
        }
        #[cfg(feature = "date-picker")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_date_picker_render_plan_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_date_picker_render_plan_in_viewport_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            if let Some(bounds) = plan.previous_button {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::DatePickerPreviousMonth,
                ));
            }
            if let Some(bounds) = plan.next_button {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::DatePickerNextMonth,
                ));
            }
            hit_targets.extend(plan.day_cells.into_iter().filter(|cell| cell.enabled).map(
                |cell| {
                    ViewHitTarget::with_kind(
                        widget,
                        cell.bounds,
                        ViewHitTargetKind::DatePickerDay { date: cell.date },
                    )
                },
            ));
        }
        #[cfg(feature = "time-picker")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_time_picker_render_plan(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_time_picker_render_plan_in_viewport(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            hit_targets.extend(plan.choices.into_iter().map(|choice| {
                ViewHitTarget::with_kind(
                    widget,
                    choice.bounds,
                    ViewHitTargetKind::TimePickerChoice {
                        value: choice.value,
                    },
                )
            }));
        }
        #[cfg(feature = "color-picker")]
        if let (Some(widget), Some(bounds), ViewNodeKind::ColorPicker { state, .. }) =
            (self.id, self.bounds, &self.kind)
        {
            if state.expanded {
                let plan = viewport.map_or_else(
                    || {
                        crate::zs_color_picker_render_plan(
                            bounds,
                            *state,
                            ZsColorPickerPlatformStyle::current(),
                            self.layout_dpi,
                        )
                    },
                    |viewport| {
                        crate::zs_color_picker_render_plan_in_viewport(
                            bounds,
                            *state,
                            ZsColorPickerPlatformStyle::current(),
                            self.layout_dpi,
                            viewport,
                        )
                    },
                );
                if let Some(popup) = plan.popup {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        popup,
                        ViewHitTargetKind::ColorPickerPopup,
                    ));
                }
                if let Some(spectrum) = plan.spectrum_bounds {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        spectrum,
                        ViewHitTargetKind::ColorPickerSpectrum,
                    ));
                }
                if let Some(hue) = plan.hue_track {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        Rect {
                            x: hue.x,
                            y: hue.y.saturating_sub(6),
                            width: hue.width,
                            height: hue.height.saturating_add(12),
                        },
                        ViewHitTargetKind::ColorPickerHue,
                    ));
                }
                hit_targets.extend(plan.channels.into_iter().map(|row| {
                    ViewHitTarget::with_kind(
                        widget,
                        row.bounds,
                        ViewHitTargetKind::ColorPickerChannel {
                            channel: row.channel,
                        },
                    )
                }));
            }
        }
        let child_viewport = viewport.or(self.bounds);
        for child in &self.children {
            child.collect_overlay_hit_targets(hit_targets, child_viewport);
        }
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
        feature = "command-palette",
        feature = "combo",
        feature = "date-picker",
        feature = "dialog",
        feature = "flyout",
        feature = "teaching-tip",
        feature = "time-picker",
        feature = "toast"
    ))]
    fn paint_overlays(&self, cx: &mut ViewPaintCx, viewport: Option<Rect>)
    where
        Msg: Clone,
    {
        #[cfg(feature = "flyout")]
        if let ViewNodeKind::Flyout {
            spec,
            open,
            target,
            ..
        } = &self.kind
        {
            let flyout_viewport = viewport.or(self.bounds);
            if let Some(page) = self.children.first() {
                page.paint_overlays(cx, flyout_viewport);
            }
            let target_bounds = self
                .children
                .first()
                .and_then(|page| page.widget_layout_bounds(*target));
            if let (true, Some(viewport), Some(target_bounds)) =
                (*open, flyout_viewport, target_bounds)
            {
                let plan = crate::zs_flyout_render_plan(
                    viewport,
                    target_bounds,
                    *spec,
                    crate::ZsFlyoutPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_flyout_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
                if let Some(content) = self.children.get(1) {
                    content.paint(cx);
                }
            }
            return;
        }

        #[cfg(feature = "breadcrumb")]
        if let ViewNodeKind::BreadcrumbBar {
            items,
            overflow_open,
            focused,
            ..
        } = &self.kind
        {
            if let Some(bounds) = self.bounds {
                let plan = crate::zs_breadcrumb_render_plan(
                    bounds,
                    items,
                    *overflow_open,
                    crate::ZsBreadcrumbPlatformStyle::current(),
                    cx.dpi,
                    viewport.or(self.bounds),
                );
                for command in
                    crate::zs_breadcrumb_popup_native_draw_plan(&plan, items, *focused).commands
                {
                    cx.draw(command);
                }
            }
            return;
        }

        #[cfg(feature = "teaching-tip")]
        if let ViewNodeKind::TeachingTip {
            spec,
            open,
            target,
            focused_control,
            ..
        } = &self.kind
        {
            let tip_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.paint_overlays(cx, tip_viewport);
            }
            let target_bounds = self
                .children
                .iter()
                .find_map(|child| child.widget_layout_bounds(*target));
            if let (true, Some(viewport), Some(target_bounds)) =
                (*open, tip_viewport, target_bounds)
            {
                let plan = crate::zs_teaching_tip_render_plan(
                    viewport,
                    target_bounds,
                    spec,
                    *focused_control,
                    crate::ZsTeachingTipPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_teaching_tip_native_draw_plan(&plan, spec).commands {
                    cx.draw(command);
                }
            }
            return;
        }

        #[cfg(feature = "toast")]
        if let ViewNodeKind::ToastPresenter {
            toast,
            focused_control,
            ..
        } = &self.kind
        {
            let toast_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.paint_overlays(cx, toast_viewport);
            }
            if let (Some(spec), Some(viewport)) = (toast.as_ref(), toast_viewport) {
                let plan = crate::zs_toast_render_plan(
                    viewport,
                    spec,
                    *focused_control,
                    crate::ZsToastPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_toast_native_draw_plan(&plan, spec).commands {
                    cx.draw(command);
                }
            }
            return;
        }

        #[cfg(feature = "command-palette")]
        if let ViewNodeKind::CommandPalette {
            items,
            query,
            highlighted,
            open,
            placeholder,
            no_results_text,
            ..
        } = &self.kind
        {
            let palette_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.paint_overlays(cx, palette_viewport);
            }
            if let (true, Some(viewport)) = (*open, palette_viewport) {
                let plan = crate::zs_command_palette_render_plan(
                    viewport,
                    query,
                    items,
                    *highlighted,
                    crate::ZsCommandPalettePlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_command_palette_native_draw_plan(
                    &plan,
                    query,
                    placeholder,
                    no_results_text,
                    items,
                )
                .commands
                {
                    cx.draw(command);
                }
            }
            return;
        }

        #[cfg(feature = "dialog")]
        if let ViewNodeKind::ContentDialog {
            spec,
            open,
            focused_button,
            ..
        } = &self.kind
        {
            let dialog_viewport = viewport.or(self.bounds);
            for child in &self.children {
                child.paint_overlays(cx, dialog_viewport);
            }
            if let (true, Some(viewport)) = (*open, dialog_viewport) {
                let plan = crate::zs_content_dialog_render_plan(
                    viewport,
                    spec,
                    *focused_button,
                    crate::ZsContentDialogPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_content_dialog_native_draw_plan(&plan, spec).commands {
                    cx.draw(command);
                }
            }
            return;
        }

        #[cfg(feature = "auto-suggest")]
        if let (
            Some(bounds),
            ViewNodeKind::AutoSuggestBox {
                query,
                suggestions,
                highlighted,
                expanded: true,
                no_results_text,
                query_icon,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let row_count = if suggestions.is_empty() && no_results_text.is_some() {
                1
            } else {
                suggestions.len()
            };
            let highlighted_index = highlighted.and_then(|highlighted| {
                suggestions
                    .iter()
                    .position(|candidate| candidate.id() == highlighted)
            });
            let plan = viewport.map_or_else(
                || {
                    crate::zs_auto_suggest_render_plan(
                        bounds,
                        row_count,
                        highlighted_index,
                        true,
                        query.is_empty(),
                        *query_icon,
                        crate::ZsAutoSuggestPlatformStyle::current(),
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_auto_suggest_render_plan_in_viewport(
                        bounds,
                        row_count,
                        highlighted_index,
                        true,
                        query.is_empty(),
                        *query_icon,
                        crate::ZsAutoSuggestPlatformStyle::current(),
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in crate::zs_auto_suggest_popup_native_draw_plan(
                &plan,
                suggestions,
                *highlighted,
                no_results_text.as_deref(),
                cx.dpi,
            )
            .commands
            {
                cx.draw(command);
            }
        }
        #[cfg(feature = "combo")]
        if let (
            Some(bounds),
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_combo_box_render_plan_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_combo_box_render_plan_in_viewport_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in
                crate::zs_combo_box_popup_native_draw_plan(&plan, options, *selected_index, cx.dpi)
                    .commands
            {
                cx.draw(command);
            }
        }
        #[cfg(feature = "date-picker")]
        if let (
            Some(bounds),
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_date_picker_render_plan_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_date_picker_render_plan_in_viewport_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in
                crate::zs_date_picker_popup_native_draw_plan(&plan, *visible_month, cx.dpi).commands
            {
                cx.draw(command);
            }
        }
        #[cfg(feature = "time-picker")]
        if let (
            Some(bounds),
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_time_picker_render_plan(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_time_picker_render_plan_in_viewport(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in crate::zs_time_picker_popup_native_draw_plan(&plan).commands {
                cx.draw(command);
            }
        }
        #[cfg(feature = "color-picker")]
        if let (Some(bounds), ViewNodeKind::ColorPicker { state, .. }) = (self.bounds, &self.kind) {
            if state.expanded {
                let plan = viewport.map_or_else(
                    || {
                        crate::zs_color_picker_render_plan(
                            bounds,
                            *state,
                            ZsColorPickerPlatformStyle::current(),
                            cx.dpi,
                        )
                    },
                    |viewport| {
                        crate::zs_color_picker_render_plan_in_viewport(
                            bounds,
                            *state,
                            ZsColorPickerPlatformStyle::current(),
                            cx.dpi,
                            viewport,
                        )
                    },
                );
                for command in crate::zs_color_picker_popup_native_draw_plan(&plan, *state).commands
                {
                    cx.draw(command);
                }
            }
        }
        let child_viewport = viewport.or(self.bounds);
        for child in &self.children {
            child.paint_overlays(cx, child_viewport);
        }
    }

    fn hit_target_kind(&self) -> ViewHitTargetKind {
        match &self.kind {
            #[cfg(feature = "canvas")]
            ViewNodeKind::Canvas { .. } => ViewHitTargetKind::Canvas,
            #[cfg(feature = "button")]
            ViewNodeKind::Button { .. } => ViewHitTargetKind::Button,
            #[cfg(feature = "breadcrumb")]
            ViewNodeKind::BreadcrumbBar { .. } => ViewHitTargetKind::BreadcrumbBar,
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { .. } => ViewHitTargetKind::ToggleButton,
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { multiline, .. } => {
                if *multiline {
                    ViewHitTargetKind::TextEditor
                } else {
                    ViewHitTargetKind::Textbox
                }
            }
            #[cfg(feature = "password-box")]
            ViewNodeKind::PasswordBox { .. } => ViewHitTargetKind::PasswordBox,
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { .. } => ViewHitTargetKind::Checkbox,
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { .. } => ViewHitTargetKind::Toggle,
            #[cfg(feature = "slider")]
            ViewNodeKind::Slider { .. } => ViewHitTargetKind::Slider,
            #[cfg(feature = "number-box")]
            ViewNodeKind::NumberBox { .. } => ViewHitTargetKind::NumberBox,
            #[cfg(feature = "radio")]
            ViewNodeKind::RadioButton { .. } => ViewHitTargetKind::RadioButton,
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox { .. } => ViewHitTargetKind::AutoSuggestBox,
            #[cfg(feature = "tree")]
            ViewNodeKind::TreeView { .. } => ViewHitTargetKind::TreeView,
            #[cfg(feature = "grid-view")]
            ViewNodeKind::GridView { .. } => ViewHitTargetKind::GridView,
            #[cfg(feature = "table")]
            ViewNodeKind::DataGrid { .. } => ViewHitTargetKind::DataGrid,
            #[cfg(feature = "dialog")]
            ViewNodeKind::ContentDialog { .. } => ViewHitTargetKind::ContentDialog,
            #[cfg(feature = "flyout")]
            ViewNodeKind::Flyout { .. } => ViewHitTargetKind::Flyout,
            #[cfg(feature = "command-palette")]
            ViewNodeKind::CommandPalette { .. } => ViewHitTargetKind::CommandPalette,
            #[cfg(feature = "toast")]
            ViewNodeKind::ToastPresenter { .. } => ViewHitTargetKind::Toast,
            #[cfg(feature = "teaching-tip")]
            ViewNodeKind::TeachingTip { .. } => ViewHitTargetKind::TeachingTip,
            #[cfg(feature = "info-bar")]
            ViewNodeKind::InfoBar { .. } => ViewHitTargetKind::InfoBar,
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { .. } => ViewHitTargetKind::ComboBox,
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker { .. } => ViewHitTargetKind::DatePicker,
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker { .. } => ViewHitTargetKind::TimePicker,
            #[cfg(feature = "color-picker")]
            ViewNodeKind::ColorPicker { .. } => ViewHitTargetKind::ColorPicker,
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => ViewHitTargetKind::Scroll,
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { .. } => ViewHitTargetKind::Scroll,
            _ => ViewHitTargetKind::Unknown,
        }
    }
}
