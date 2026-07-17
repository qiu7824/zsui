#[derive(Debug, Clone)]
pub struct ViewPaintCx {
    pub dpi: Dpi,
    plan: NativeDrawPlan,
    paint_depth: usize,
    animation_elapsed_ms: u64,
}

impl ViewPaintCx {
    pub fn new(dpi: Dpi) -> Self {
        Self {
            dpi,
            plan: NativeDrawPlan::default(),
            paint_depth: 0,
            animation_elapsed_ms: 0,
        }
    }

    pub(crate) fn with_animation_elapsed(dpi: Dpi, elapsed: std::time::Duration) -> Self {
        let mut cx = Self::new(dpi);
        cx.animation_elapsed_ms = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
        cx
    }

    pub fn draw(&mut self, command: NativeDrawCommand) {
        self.plan.push(command);
    }

    pub fn plan(&self) -> &NativeDrawPlan {
        &self.plan
    }

    pub fn into_plan(self) -> NativeDrawPlan {
        self.plan
    }

    pub fn set_theme_mode(&mut self, theme_mode: ZsuiThemeMode) {
        self.plan.theme_mode = theme_mode;
    }

    fn finish_node<Msg>(&mut self, _root: &ViewNode<Msg>) {
        self.paint_depth = self.paint_depth.saturating_sub(1);
        #[cfg(any(
            feature = "auto-suggest",
            feature = "breadcrumb",
            feature = "color-picker",
            feature = "command-palette",
            feature = "combo",
            feature = "date-picker",
            feature = "dialog",
            feature = "teaching-tip",
            feature = "time-picker",
            feature = "toast"
        ))]
        if self.paint_depth == 0 {
            _root.paint_overlays(self, None);
        }
    }
}

pub trait View<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput;
    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent);
    fn paint(&self, cx: &mut ViewPaintCx);
}

#[cfg(feature = "tabs")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_tabs(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let (tabs, selected) = match &self.kind {
            ViewNodeKind::Tabs { tabs, selected, .. } => (tabs, *selected),
            _ => unreachable!("tab layout requires a tab view node"),
        };
        let selected_index = selected
            .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
        let plan = crate::zs_tab_view_render_plan_for_tabs(
            cx.bounds,
            tabs,
            selected_index,
            crate::ZsTabPlatformStyle::current(),
            cx.dpi,
        );
        for child in &mut self.children {
            child.clear_layout_bounds();
        }
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }
        if let Some(child) = selected_index.and_then(|index| self.children.get_mut(index)) {
            let mut child_cx = ViewLayoutCx {
                bounds: plan.content_bounds,
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

#[cfg(feature = "virtual-list")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_virtual_list(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let (total_count, row_height, overscan_rows, row_indices, current_offset) = match &self.kind
        {
            ViewNodeKind::VirtualList {
                total_count,
                row_height,
                overscan_rows,
                row_indices,
                offset_y,
                ..
            } => (
                *total_count,
                *row_height,
                *overscan_rows,
                row_indices.clone(),
                *offset_y,
            ),
            _ => unreachable!("virtual list layout requires a virtual list node"),
        };
        let content_bounds = inset_bounds(cx.bounds, self.style.padding, cx.dpi);
        let viewport_height =
            Dp::new(content_bounds.height.max(0) as f32 / cx.dpi.scale_factor().max(f32::EPSILON));
        let viewport = virtual_list_viewport(
            total_count,
            row_height,
            current_offset,
            viewport_height,
            overscan_rows,
            VirtualListScrollDirection::Stationary,
        );
        if let ViewNodeKind::VirtualList {
            offset_y,
            visible_range,
            materialized_range,
            ..
        } = &mut self.kind
        {
            *offset_y = viewport.offset_y;
            *visible_range = viewport.visible_range;
            *materialized_range = viewport.materialized_range;
        }

        for child in &mut self.children {
            child.clear_layout_bounds();
        }
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }
        let row_height_px = row_height.to_px(cx.dpi).round_i32().max(1);
        let offset_px = viewport.offset_y.to_px(cx.dpi).round_i32().max(0);
        for (index, child) in row_indices.into_iter().zip(self.children.iter_mut()) {
            if !viewport.materialized_range.contains(index) {
                continue;
            }
            let row_top = (index as i64)
                .saturating_mul(row_height_px as i64)
                .saturating_sub(offset_px as i64)
                .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            let mut child_cx = ViewLayoutCx {
                bounds: Rect {
                    x: content_bounds.x,
                    y: content_bounds.y.saturating_add(row_top),
                    width: content_bounds.width,
                    height: row_height_px,
                },
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

impl<Msg: Clone> View<Msg> for ViewNode<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        #[cfg(feature = "tabs")]
        if matches!(self.kind, ViewNodeKind::Tabs { .. }) {
            return self.layout_tabs(cx);
        }
        #[cfg(feature = "virtual-list")]
        if matches!(self.kind, ViewNodeKind::VirtualList { .. }) {
            return self.layout_virtual_list(cx);
        }

        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }

        let child_bounds = split_child_bounds(
            inset_bounds(cx.bounds, self.style.padding, cx.dpi),
            &self.kind,
            &self.children,
            self.style.gap,
            cx.dpi,
        );
        for (child, bounds) in self.children.iter_mut().zip(child_bounds) {
            let mut child_cx = ViewLayoutCx {
                bounds,
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }

        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }

    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent) {
        #[cfg(feature = "teaching-tip")]
        if matches!(self.kind, ViewNodeKind::TeachingTip { .. }) {
            let mut handled = false;
            if let ViewNodeKind::TeachingTip {
                spec,
                open,
                focused_control,
                on_result,
                ..
            } = &mut self.kind
            {
                if *open {
                    match event {
                        ViewEvent::TeachingTipFocused { widget, control }
                            if self.id == Some(*widget) && spec.has_control(*control) =>
                        {
                            *focused_control = *control;
                            handled = true;
                        }
                        ViewEvent::TeachingTipResponded { widget, response }
                            if self.id == Some(*widget) =>
                        {
                            *open = false;
                            if let Some(message) = on_result {
                                cx.emit(message(crate::ZsTeachingTipResult {
                                    response: *response,
                                }));
                            }
                            handled = true;
                        }
                        _ => {}
                    }
                }
            }
            if handled {
                return;
            }
            for child in &mut self.children {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(feature = "info-bar")]
        if let ViewNodeKind::InfoBar {
            spec,
            focused_control,
            on_event,
        } = &mut self.kind
        {
            match event {
                ViewEvent::InfoBarFocused { widget, control }
                    if self.id == Some(*widget) && spec.has_control(*control) =>
                {
                    *focused_control = Some(*control);
                }
                ViewEvent::InfoBarInvoked { widget, event }
                    if self.id == Some(*widget)
                        && match event {
                            crate::ZsInfoBarEvent::Action => {
                                spec.has_control(crate::ZsInfoBarControl::Action)
                            }
                            crate::ZsInfoBarEvent::Close => {
                                spec.has_control(crate::ZsInfoBarControl::Close)
                            }
                        } =>
                {
                    if let Some(message) = on_event {
                        cx.emit(message(*event));
                    }
                }
                _ => {}
            }
            return;
        }

        #[cfg(feature = "breadcrumb")]
        if let ViewNodeKind::BreadcrumbBar {
            items,
            overflow_open,
            focused,
            on_select,
            on_expanded_change,
        } = &mut self.kind
        {
            match event {
                ViewEvent::BreadcrumbFocused { widget, target }
                    if self.id == Some(*widget)
                        && match target {
                            crate::ZsBreadcrumbFocusTarget::Overflow => true,
                            crate::ZsBreadcrumbFocusTarget::Item(id) => {
                                items.iter().any(|item| item.id() == *id)
                            }
                        } =>
                {
                    *focused = Some(*target);
                }
                ViewEvent::BreadcrumbExpandedChanged { widget, expanded }
                    if self.id == Some(*widget) =>
                {
                    *overflow_open = *expanded;
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(*expanded));
                    }
                }
                ViewEvent::BreadcrumbSelected { widget, item }
                    if self.id == Some(*widget)
                        && items.iter().any(|candidate| candidate.id() == *item) =>
                {
                    *focused = Some(crate::ZsBreadcrumbFocusTarget::Item(*item));
                    if *overflow_open {
                        *overflow_open = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                    if let Some(message) = on_select {
                        cx.emit(message(*item));
                    }
                }
                ViewEvent::DismissPopupOverlays { except }
                    if self.id.is_some() && self.id != *except && *overflow_open =>
                {
                    *overflow_open = false;
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(false));
                    }
                }
                _ => {}
            }
            return;
        }

        #[cfg(feature = "toast")]
        if matches!(self.kind, ViewNodeKind::ToastPresenter { .. }) {
            let mut handled = false;
            if let ViewNodeKind::ToastPresenter {
                toast,
                focused_control,
                on_result,
            } = &mut self.kind
            {
                if let Some(active) = toast.as_ref() {
                    match event {
                        ViewEvent::ToastFocused {
                            widget,
                            toast: toast_id,
                            control,
                        } if self.id == Some(*widget)
                            && active.id() == *toast_id
                            && active.has_control(*control) =>
                        {
                            *focused_control = *control;
                            handled = true;
                        }
                        ViewEvent::ToastResponded {
                            widget,
                            toast: toast_id,
                            response,
                        } if self.id == Some(*widget) && active.id() == *toast_id => {
                            let result = crate::ZsToastResult {
                                id: *toast_id,
                                response: *response,
                            };
                            *toast = None;
                            if let Some(message) = on_result {
                                cx.emit(message(result));
                            }
                            handled = true;
                        }
                        _ => {}
                    }
                }
            }
            if handled {
                return;
            }
            for child in &mut self.children {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(feature = "command-palette")]
        if matches!(self.kind, ViewNodeKind::CommandPalette { .. }) {
            let mut handled = false;
            let mut palette_open = false;
            if let ViewNodeKind::CommandPalette {
                items,
                query,
                highlighted,
                open,
                on_query_change,
                on_highlight_change,
                on_invoke,
                on_open_change,
                ..
            } = &mut self.kind
            {
                palette_open = *open;
                if let ViewEvent::CommandPaletteOpenChanged {
                    widget,
                    open: requested,
                } = event
                {
                    if self.id == Some(*widget) {
                        *open = *requested;
                        if *requested {
                            let state = crate::command_palette::command_palette_state(
                                true,
                                query,
                                items,
                                *highlighted,
                            );
                            *highlighted = state.highlighted.or_else(|| state.first_enabled());
                        }
                        if let Some(message) = on_open_change {
                            cx.emit(message(*requested));
                        }
                        handled = true;
                    }
                }
                if !handled && *open {
                    match event {
                        ViewEvent::TextChanged { widget, value }
                            if self.id == Some(*widget) && *query != *value =>
                        {
                            *query = value.clone();
                            let state = crate::command_palette::command_palette_state(
                                true,
                                query,
                                items,
                                *highlighted,
                            );
                            let next = state.highlighted.or_else(|| state.first_enabled());
                            if *highlighted != next {
                                *highlighted = next;
                                if let (Some(message), Some(item)) = (on_highlight_change, next) {
                                    cx.emit(message(item));
                                }
                            }
                            if let Some(message) = on_query_change {
                                cx.emit(message(value.clone()));
                            }
                            handled = true;
                        }
                        ViewEvent::CommandPaletteHighlighted { widget, item }
                            if self.id == Some(*widget)
                                && crate::command_palette::command_palette_state(
                                    true,
                                    query,
                                    items,
                                    Some(*item),
                                )
                                .highlighted
                                    == Some(*item) =>
                        {
                            if *highlighted != Some(*item) {
                                *highlighted = Some(*item);
                                if let Some(message) = on_highlight_change {
                                    cx.emit(message(*item));
                                }
                            }
                            handled = true;
                        }
                        ViewEvent::CommandPaletteInvoked { widget, item }
                            if self.id == Some(*widget)
                                && crate::command_palette::command_palette_state(
                                    true,
                                    query,
                                    items,
                                    Some(*item),
                                )
                                .highlighted
                                    == Some(*item) =>
                        {
                            *highlighted = Some(*item);
                            *open = false;
                            if let Some(message) = on_invoke {
                                cx.emit(message(*item));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message(false));
                            }
                            handled = true;
                        }
                        _ => {}
                    }
                }
            }
            if handled || palette_open {
                return;
            }
            for child in &mut self.children {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(feature = "dialog")]
        if matches!(self.kind, ViewNodeKind::ContentDialog { .. }) {
            let mut handled = false;
            let mut modal_open = false;
            if let ViewNodeKind::ContentDialog {
                spec,
                open,
                focused_button,
                on_result,
            } = &mut self.kind
            {
                modal_open = *open;
                if *open {
                    match event {
                        ViewEvent::ContentDialogFocused { widget, button }
                            if self.id == Some(*widget) && spec.has_button(*button) =>
                        {
                            *focused_button = *button;
                            handled = true;
                        }
                        ViewEvent::ContentDialogResponded { widget, button }
                            if self.id == Some(*widget) && spec.has_button(*button) =>
                        {
                            *open = false;
                            if let Some(message) = on_result {
                                cx.emit(message((*button).into()));
                            }
                            handled = true;
                        }
                        _ => {}
                    }
                }
            }
            if handled || modal_open {
                return;
            }
            for child in &mut self.children {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(feature = "tabs")]
        if let ViewNodeKind::Tabs {
            tabs,
            selected,
            on_select,
        } = &mut self.kind
        {
            if let ViewEvent::TabSelected { widget, tab } = event {
                if self.id == Some(*widget)
                    && tabs.iter().any(|candidate| candidate.id == *tab)
                    && *selected != Some(*tab)
                {
                    *selected = Some(*tab);
                    if let Some(message) = on_select {
                        cx.emit(message(*tab));
                    }
                }
            }
            let selected_index = (*selected)
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            if let Some(child) = selected_index.and_then(|index| self.children.get_mut(index)) {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(any(
            feature = "auto-suggest",
            feature = "color-picker",
            feature = "combo",
            feature = "date-picker",
            feature = "time-picker"
        ))]
        if let ViewEvent::DismissPopupOverlays { except } = event {
            let should_dismiss = self.id.is_some() && self.id != *except;
            #[cfg(feature = "auto-suggest")]
            if should_dismiss {
                if let ViewNodeKind::AutoSuggestBox {
                    highlighted,
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *highlighted = None;
                        *expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "combo")]
            if should_dismiss {
                if let ViewNodeKind::ComboBox {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        self.combo_first_visible_option = None;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "time-picker")]
            if should_dismiss {
                if let ViewNodeKind::TimePicker {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "date-picker")]
            if should_dismiss {
                if let ViewNodeKind::DatePicker {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "color-picker")]
            if should_dismiss {
                if let ViewNodeKind::ColorPicker {
                    state,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if state.expanded {
                        state.expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
        }

        #[cfg(feature = "list")]
        if let (
            ViewNodeKind::List {
                selected_index,
                on_select,
            },
            ViewEvent::Click { widget },
        ) = (&mut self.kind, event)
        {
            if let Some(index) = self
                .children
                .iter()
                .position(|child| child.contains_widget(*widget))
            {
                *selected_index = Some(index);
                if let Some(message) = on_select {
                    cx.emit(message(index));
                }
            }
        }

        #[cfg(feature = "radio")]
        if let ViewEvent::RadioSelected { widget } = event {
            let contains_target = self.children.iter().any(|child| {
                child.id == Some(*widget) && matches!(&child.kind, ViewNodeKind::RadioButton { .. })
            });
            if contains_target && matches!(&self.kind, ViewNodeKind::Stack { .. }) {
                for child in &mut self.children {
                    if let ViewNodeKind::RadioButton { selected, .. } = &mut child.kind {
                        *selected = child.id == Some(*widget);
                    }
                }
            }
        }

        #[cfg(feature = "virtual-list")]
        if let (
            ViewNodeKind::VirtualList {
                row_indices,
                selected_index,
                on_select,
                ..
            },
            ViewEvent::Click { widget },
        ) = (&mut self.kind, event)
        {
            if let Some(position) = self
                .children
                .iter()
                .position(|child| child.contains_widget(*widget))
            {
                if let Some(index) = row_indices.get(position).copied() {
                    *selected_index = Some(index);
                    if let Some(message) = on_select {
                        cx.emit(message(index));
                    }
                }
            }
        }

        #[cfg(feature = "combo")]
        if let (
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded,
                on_select,
                on_expanded_change,
                ..
            },
            ViewEvent::ComboBoxSelected { index, .. },
        ) = (&mut self.kind, event)
        {
            if *index < options.len() {
                *selected_index = Some(*index);
                let was_expanded = *expanded;
                *expanded = false;
                self.combo_first_visible_option = None;
                if let Some(message) = on_select {
                    cx.emit(message(*index));
                }
                if was_expanded {
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(false));
                    }
                }
            }
        }

        #[cfg(feature = "combo")]
        if let (
            ViewNodeKind::ComboBox {
                options,
                expanded: true,
                ..
            },
            ViewEvent::ComboBoxScrolled {
                first_visible_index,
                ..
            },
        ) = (&mut self.kind, event)
        {
            self.combo_first_visible_option =
                Some((*first_visible_index).min(options.len().saturating_sub(1)));
        }

        #[cfg(feature = "date-picker")]
        if let (
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                expanded,
                on_date_change,
                on_expanded_change,
                ..
            },
            ViewEvent::DateChanged {
                value: next_value, ..
            },
        ) = (&mut self.kind, event)
        {
            let next_value = (*next_value).clamp(*minimum, *maximum);
            let changed = *value != next_value;
            let was_expanded = *expanded;
            *value = next_value;
            *visible_month = next_value.first_day_of_month();
            *expanded = false;
            if changed {
                if let Some(message) = on_date_change {
                    cx.emit(message(next_value));
                }
            }
            if was_expanded {
                if let Some(message) = on_expanded_change {
                    cx.emit(message(false));
                }
            }
        }

        #[cfg(feature = "time-picker")]
        if let (
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                on_time_change,
                ..
            },
            ViewEvent::TimeChanged {
                widget,
                value: next_value,
            },
        ) = (&mut self.kind, event)
        {
            if self.id == Some(*widget) {
                let next_value = next_value.snap(*minute_increment);
                if *value != next_value {
                    *value = next_value;
                    if let Some(message) = on_time_change {
                        cx.emit(message(next_value));
                    }
                }
            }
        }

        #[cfg(feature = "color-picker")]
        if let ViewNodeKind::ColorPicker {
            state,
            on_color_change,
            on_expanded_change,
            on_channel_change,
        } = &mut self.kind
        {
            match event {
                ViewEvent::ColorPickerExpandedChanged { widget, expanded }
                    if self.id == Some(*widget) =>
                {
                    let changed = state.expanded != *expanded;
                    state.expanded = *expanded;
                    if changed {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(*expanded));
                        }
                    }
                }
                ViewEvent::ColorPickerChannelChanged { widget, channel }
                    if self.id == Some(*widget)
                        && (state.alpha_enabled || *channel != ZsColorChannel::Alpha) =>
                {
                    if state.active_channel != *channel {
                        state.active_channel = *channel;
                        if let Some(message) = on_channel_change {
                            cx.emit(message(*channel));
                        }
                    }
                }
                ViewEvent::ColorChanged { widget, color } if self.id == Some(*widget) => {
                    let color = if state.alpha_enabled {
                        *color
                    } else {
                        crate::Color::rgb(color.r, color.g, color.b)
                    };
                    if state.color != color {
                        state.color = color;
                        if let Some(message) = on_color_change {
                            cx.emit(message(color));
                        }
                    }
                }
                _ => {}
            }
        }

        if self.event_targets_self(event) {
            #[cfg(feature = "virtual-list")]
            let list_bounds = self
                .bounds
                .map(|bounds| inset_bounds(bounds, self.style.padding, self.layout_dpi));
            match (&mut self.kind, event) {
                #[cfg(feature = "button")]
                (ViewNodeKind::Button { on_click, .. }, ViewEvent::Click { .. }) => {
                    if let Some(message) = on_click.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "toggle-button")]
                (
                    ViewNodeKind::ToggleButton {
                        checked, on_toggle, ..
                    },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "textbox")]
                (
                    ViewNodeKind::Textbox {
                        value, on_change, ..
                    },
                    ViewEvent::TextChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = next_value.clone();
                    if let Some(message) = on_change {
                        cx.emit(message(next_value.clone()));
                    }
                }
                #[cfg(feature = "textbox")]
                (
                    ViewNodeKind::Textbox {
                        value,
                        on_change,
                        on_selection_change,
                        ..
                    },
                    ViewEvent::TextEdited {
                        value: next_value,
                        selection,
                        ..
                    },
                ) => {
                    *value = next_value.clone();
                    if let Some(message) = on_change {
                        cx.emit(message(next_value.clone()));
                    }
                    if let Some(message) = on_selection_change {
                        cx.emit(message(*selection));
                    }
                }
                #[cfg(feature = "textbox")]
                (
                    ViewNodeKind::Textbox {
                        on_selection_change,
                        ..
                    },
                    ViewEvent::TextSelectionChanged { selection, .. },
                ) => {
                    if let Some(message) = on_selection_change {
                        cx.emit(message(*selection));
                    }
                }
                #[cfg(feature = "auto-suggest")]
                (
                    ViewNodeKind::AutoSuggestBox {
                        query,
                        highlighted,
                        expanded,
                        on_text_change,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::AutoSuggestCleared { .. },
                ) => {
                    let query_changed = !query.is_empty();
                    let was_expanded = *expanded;
                    query.clear();
                    *highlighted = None;
                    *expanded = false;
                    if query_changed {
                        if let Some(message) = on_text_change {
                            cx.emit(message(crate::ZsAutoSuggestTextChange::new(
                                String::new(),
                                crate::ZsAutoSuggestTextChangeReason::UserInput,
                            )));
                        }
                    }
                    if was_expanded {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
                #[cfg(feature = "auto-suggest")]
                (
                    ViewNodeKind::AutoSuggestBox {
                        query,
                        suggestions,
                        highlighted,
                        expanded,
                        no_results_text,
                        on_text_change,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::TextChanged {
                        value: next_query, ..
                    },
                ) => {
                    *query = next_query.clone();
                    *highlighted = None;
                    let next_expanded = !suggestions.is_empty() || no_results_text.is_some();
                    let expanded_changed = *expanded != next_expanded;
                    *expanded = next_expanded;
                    if let Some(message) = on_text_change {
                        cx.emit(message(crate::ZsAutoSuggestTextChange::new(
                            next_query.clone(),
                            crate::ZsAutoSuggestTextChangeReason::UserInput,
                        )));
                    }
                    if expanded_changed {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(next_expanded));
                        }
                    }
                }
                #[cfg(feature = "auto-suggest")]
                (
                    ViewNodeKind::AutoSuggestBox {
                        highlighted,
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::AutoSuggestExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    let changed = *expanded != *next_expanded;
                    *expanded = *next_expanded;
                    if !*next_expanded {
                        *highlighted = None;
                    }
                    if changed {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(*next_expanded));
                        }
                    }
                }
                #[cfg(feature = "auto-suggest")]
                (
                    ViewNodeKind::AutoSuggestBox {
                        query,
                        suggestions,
                        highlighted,
                        on_text_change,
                        on_suggestion_chosen,
                        ..
                    },
                    ViewEvent::AutoSuggestHighlighted { suggestion, .. },
                ) => {
                    if let Some(candidate) = suggestions
                        .iter()
                        .find(|candidate| candidate.id() == *suggestion)
                    {
                        let changed =
                            *highlighted != Some(*suggestion) || query != candidate.text();
                        *highlighted = Some(*suggestion);
                        *query = candidate.text().to_string();
                        if changed {
                            if let Some(message) = on_text_change {
                                cx.emit(message(crate::ZsAutoSuggestTextChange::new(
                                    query.clone(),
                                    crate::ZsAutoSuggestTextChangeReason::SuggestionChosen,
                                )));
                            }
                            if let Some(message) = on_suggestion_chosen {
                                cx.emit(message(*suggestion));
                            }
                        }
                    }
                }
                #[cfg(feature = "auto-suggest")]
                (
                    ViewNodeKind::AutoSuggestBox {
                        query,
                        suggestions,
                        highlighted,
                        expanded,
                        on_text_change,
                        on_suggestion_chosen,
                        on_query_submit,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::AutoSuggestSubmitted { suggestion, .. },
                ) => {
                    let chosen = suggestion
                        .filter(|id| suggestions.iter().any(|candidate| candidate.id() == *id));
                    if let Some(chosen) = chosen {
                        if let Some(candidate) = suggestions
                            .iter()
                            .find(|candidate| candidate.id() == chosen)
                        {
                            let changed = *highlighted != Some(chosen) || query != candidate.text();
                            *highlighted = Some(chosen);
                            *query = candidate.text().to_string();
                            if changed {
                                if let Some(message) = on_text_change {
                                    cx.emit(message(crate::ZsAutoSuggestTextChange::new(
                                        query.clone(),
                                        crate::ZsAutoSuggestTextChangeReason::SuggestionChosen,
                                    )));
                                }
                                if let Some(message) = on_suggestion_chosen {
                                    cx.emit(message(chosen));
                                }
                            }
                        }
                    }
                    if let Some(message) = on_query_submit {
                        cx.emit(message(crate::ZsAutoSuggestSubmission::new(
                            query.clone(),
                            chosen,
                        )));
                    }
                    let was_expanded = *expanded;
                    *expanded = false;
                    *highlighted = None;
                    if was_expanded {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
                #[cfg(feature = "tree")]
                (
                    ViewNodeKind::TreeView {
                        roots,
                        expanded,
                        on_expansion_change,
                        ..
                    },
                    ViewEvent::TreeNodeExpandedChanged {
                        node,
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    let expandable = crate::tree::find_tree_node(roots, *node)
                        .is_some_and(crate::ZsTreeNode::is_expandable);
                    if expandable {
                        let changed = if *next_expanded {
                            expanded.insert(*node)
                        } else {
                            expanded.remove(node)
                        };
                        if changed {
                            if let Some(message) = on_expansion_change {
                                cx.emit(message(crate::ZsTreeExpansionChange::new(
                                    *node,
                                    *next_expanded,
                                )));
                            }
                        }
                    }
                }
                #[cfg(feature = "tree")]
                (
                    ViewNodeKind::TreeView {
                        roots,
                        expanded,
                        selected,
                        on_select,
                        ..
                    },
                    ViewEvent::TreeNodeSelected { node, .. },
                ) => {
                    let visible = crate::tree::tree_view_state(roots, expanded, *selected)
                        .rows
                        .iter()
                        .any(|row| row.node == *node);
                    if visible && *selected != Some(*node) {
                        *selected = Some(*node);
                        if let Some(message) = on_select {
                            cx.emit(message(*node));
                        }
                    }
                }
                #[cfg(feature = "tree")]
                (
                    ViewNodeKind::TreeView {
                        roots,
                        expanded,
                        selected,
                        on_invoke,
                        ..
                    },
                    ViewEvent::TreeNodeInvoked { node, .. },
                ) => {
                    let visible = crate::tree::tree_view_state(roots, expanded, *selected)
                        .rows
                        .iter()
                        .any(|row| row.node == *node);
                    if visible {
                        if let Some(message) = on_invoke {
                            cx.emit(message(*node));
                        }
                    }
                }
                #[cfg(feature = "grid-view")]
                (
                    ViewNodeKind::GridView {
                        items,
                        selected,
                        on_select,
                        ..
                    },
                    ViewEvent::GridViewItemSelected { item, .. },
                ) => {
                    let contains = crate::grid_view::unique_grid_view_items(items)
                        .iter()
                        .any(|candidate| candidate.id() == *item);
                    if contains && *selected != Some(*item) {
                        *selected = Some(*item);
                        if let Some(message) = on_select {
                            cx.emit(message(*item));
                        }
                    }
                }
                #[cfg(feature = "grid-view")]
                (
                    ViewNodeKind::GridView {
                        items, on_invoke, ..
                    },
                    ViewEvent::GridViewItemInvoked { item, .. },
                ) => {
                    let contains = crate::grid_view::unique_grid_view_items(items)
                        .iter()
                        .any(|candidate| candidate.id() == *item);
                    if contains {
                        if let Some(message) = on_invoke {
                            cx.emit(message(*item));
                        }
                    }
                }
                #[cfg(feature = "table")]
                (
                    ViewNodeKind::DataGrid {
                        rows,
                        selected,
                        on_select,
                        ..
                    },
                    ViewEvent::TableRowSelected { row, .. },
                ) => {
                    let visible = crate::table::unique_table_rows(rows)
                        .into_iter()
                        .any(|candidate| candidate.id() == *row);
                    if visible && *selected != Some(*row) {
                        *selected = Some(*row);
                        if let Some(message) = on_select {
                            cx.emit(message(*row));
                        }
                    }
                }
                #[cfg(feature = "table")]
                (
                    ViewNodeKind::DataGrid {
                        columns,
                        sort,
                        on_sort,
                        ..
                    },
                    ViewEvent::TableSorted { column, .. },
                ) => {
                    if let Some(next) = crate::table::next_table_sort(columns, *sort, *column) {
                        if *sort != Some(next) {
                            *sort = Some(next);
                            if let Some(message) = on_sort {
                                cx.emit(message(next));
                            }
                        }
                    }
                }
                #[cfg(feature = "table")]
                (
                    ViewNodeKind::DataGrid {
                        rows, on_invoke, ..
                    },
                    ViewEvent::TableRowInvoked { row, .. },
                ) => {
                    let visible = crate::table::unique_table_rows(rows)
                        .into_iter()
                        .any(|candidate| candidate.id() == *row);
                    if visible {
                        if let Some(message) = on_invoke {
                            cx.emit(message(*row));
                        }
                    }
                }
                #[cfg(feature = "password-box")]
                (
                    ViewNodeKind::PasswordBox {
                        value, on_change, ..
                    },
                    ViewEvent::PasswordChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = next_value.clone();
                    if let Some(message) = on_change {
                        cx.emit(message(next_value.clone()));
                    }
                }
                #[cfg(feature = "checkbox")]
                (
                    ViewNodeKind::Checkbox {
                        checked, on_toggle, ..
                    },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "toggle")]
                (
                    ViewNodeKind::Toggle { checked, on_toggle },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "slider")]
                (
                    ViewNodeKind::Slider {
                        value,
                        range,
                        on_slide,
                    },
                    ViewEvent::SliderChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = range.snap(*next_value);
                    if let Some(message) = on_slide {
                        cx.emit(message(*value));
                    }
                }
                #[cfg(feature = "number-box")]
                (
                    ViewNodeKind::NumberBox { draft, .. },
                    ViewEvent::TextChanged {
                        value: next_draft, ..
                    },
                ) => {
                    *draft = next_draft.clone();
                }
                #[cfg(feature = "number-box")]
                (
                    ViewNodeKind::NumberBox {
                        value,
                        draft,
                        range,
                        format,
                        wraps,
                        on_change,
                    },
                    ViewEvent::NumberBoxStep { steps, large, .. },
                ) => {
                    let current = format
                        .parse(draft)
                        .filter(|candidate| range.contains(*candidate))
                        .or(*value)
                        .unwrap_or_else(|| range.min());
                    let next_value = Some(range.offset(current, *steps, *large, *wraps));
                    let changed = *value != next_value;
                    *value = next_value;
                    *draft = format.format(next_value);
                    if changed {
                        if let Some(message) = on_change {
                            cx.emit(message(next_value));
                        }
                    }
                }
                #[cfg(feature = "number-box")]
                (
                    ViewNodeKind::NumberBox {
                        value,
                        draft,
                        range,
                        format,
                        on_change,
                        ..
                    },
                    ViewEvent::NumberBoxCommit { .. },
                ) => {
                    let next_value = if draft.trim().is_empty() {
                        None
                    } else {
                        format
                            .parse(draft)
                            .map(|candidate| range.clamp(candidate))
                            .or(*value)
                    };
                    let changed = *value != next_value;
                    *value = next_value;
                    *draft = format.format(next_value);
                    if changed {
                        if let Some(message) = on_change {
                            cx.emit(message(next_value));
                        }
                    }
                }
                #[cfg(feature = "number-box")]
                (
                    ViewNodeKind::NumberBox {
                        value,
                        draft,
                        format,
                        ..
                    },
                    ViewEvent::NumberBoxReset { .. },
                ) => {
                    *draft = format.format(*value);
                }
                #[cfg(feature = "radio")]
                (
                    ViewNodeKind::RadioButton {
                        selected,
                        on_choose,
                        ..
                    },
                    ViewEvent::RadioSelected { .. },
                ) => {
                    *selected = true;
                    if let Some(message) = on_choose.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "combo")]
                (
                    ViewNodeKind::ComboBox {
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::ComboBoxExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    *expanded = *next_expanded;
                    self.combo_first_visible_option = None;
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        value,
                        visible_month,
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::DatePickerExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    *expanded = *next_expanded;
                    if *next_expanded {
                        *visible_month = value.first_day_of_month();
                    }
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        minimum,
                        maximum,
                        visible_month,
                        ..
                    },
                    ViewEvent::DatePickerMonthChanged { month, .. },
                ) => {
                    *visible_month = clamp_visible_month(*month, *minimum, *maximum);
                }
                #[cfg(feature = "time-picker")]
                (
                    ViewNodeKind::TimePicker {
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::TimePickerExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    if *expanded != *next_expanded {
                        *expanded = *next_expanded;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(*next_expanded));
                        }
                    }
                }
                #[cfg(feature = "scroll")]
                (
                    ViewNodeKind::Scroll {
                        offset_y,
                        content_height,
                        on_scroll,
                    },
                    ViewEvent::ScrollBy { delta_y, .. },
                ) => {
                    let max_offset =
                        scroll_max_offset_y(self.bounds, *content_height, self.layout_dpi);
                    let next = Dp::new((offset_y.0 + delta_y.0).clamp(0.0, max_offset.0));
                    *offset_y = next;
                    if let Some(message) = on_scroll {
                        cx.emit(message(next));
                    }
                }
                #[cfg(feature = "virtual-list")]
                (
                    ViewNodeKind::VirtualList {
                        total_count,
                        row_height,
                        overscan_rows,
                        offset_y,
                        visible_range,
                        materialized_range,
                        on_viewport_changed,
                        ..
                    },
                    ViewEvent::ScrollBy { delta_y, .. },
                ) => {
                    let viewport_height = list_bounds
                        .map(|bounds| {
                            Dp::new(
                                bounds.height.max(0) as f32
                                    / self.layout_dpi.scale_factor().max(f32::EPSILON),
                            )
                        })
                        .unwrap_or(Dp::new(0.0));
                    let requested = Dp::new(offset_y.0 + delta_y.0);
                    let direction = if requested.0 > offset_y.0 {
                        VirtualListScrollDirection::Forward
                    } else if requested.0 < offset_y.0 {
                        VirtualListScrollDirection::Backward
                    } else {
                        VirtualListScrollDirection::Stationary
                    };
                    let viewport = virtual_list_viewport(
                        *total_count,
                        *row_height,
                        requested,
                        viewport_height,
                        *overscan_rows,
                        direction,
                    );
                    *offset_y = viewport.offset_y;
                    *visible_range = viewport.visible_range;
                    *materialized_range = viewport.materialized_range;
                    if let Some(message) = on_viewport_changed {
                        cx.emit(message(viewport));
                    }
                }
                _ => {}
            }
        }

        for child in &mut self.children {
            child.event(cx, event);
        }
    }

    fn paint(&self, cx: &mut ViewPaintCx) {
        let Some(bounds) = self.bounds else {
            return;
        };
        cx.paint_depth = cx.paint_depth.saturating_add(1);

        if let Some(theme_mode) = self.style.theme_mode {
            cx.set_theme_mode(theme_mode);
        }

        if let Some(background) = self.style.background {
            let fill = NativeDrawFill::Role(color_role_for_token(background));
            let radius = radius_px(self.style.radius, cx.dpi);
            if radius == 0 {
                cx.draw(NativeDrawCommand::FillRect { rect: bounds, fill });
            } else {
                cx.draw(NativeDrawCommand::RoundFill {
                    rect: bounds,
                    fill,
                    radius,
                });
            }
        }

        match &self.kind {
            #[cfg(feature = "label")]
            ViewNodeKind::Text { text, style } => {
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    text,
                    padded_bounds(bounds, self.style.padding, cx.dpi),
                    *style,
                )));
            }
            #[cfg(feature = "image-preview")]
            ViewNodeKind::ImagePreview {
                snapshot,
                fit,
                interpolation,
            } => {
                if self.style.background.is_none() {
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: bounds,
                        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                    });
                }
                let content_bounds = inset_bounds(bounds, self.style.padding, cx.dpi);
                if let Some(frame) = snapshot.frame.clone() {
                    if let Some(command) = crate::zs_image_native_draw_command(
                        frame,
                        content_bounds,
                        *fit,
                        *interpolation,
                    ) {
                        cx.draw(NativeDrawCommand::PushClip {
                            rect: content_bounds,
                        });
                        cx.draw(NativeDrawCommand::Image(command));
                        cx.draw(NativeDrawCommand::PopClip);
                    }
                } else {
                    let side = content_bounds
                        .width
                        .min(content_bounds.height)
                        .min(Dp::new(32.0).to_px(cx.dpi).round_i32())
                        .max(1);
                    cx.draw(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
                        crate::ZsIcon::Image,
                        Rect {
                            x: content_bounds.x + (content_bounds.width - side) / 2,
                            y: content_bounds.y + (content_bounds.height - side) / 2,
                            width: side,
                            height: side,
                        },
                        NativeIconColorMode::ThemeAware,
                    )));
                }
            }
            #[cfg(feature = "button")]
            ViewNodeKind::Button {
                label,
                presentation,
                ..
            } => {
                match presentation {
                    ZsButtonPresentation::Standard => {
                        let platform = crate::ZsBaseControlPlatformStyle::current();
                        let metrics = crate::ZsBaseControlMetrics::for_platform(platform);
                        // Standard buttons deliberately keep their platform
                        // bezel grammar: WinUI uses a bordered control,
                        // AppKit uses a clean bezel without an outline, and
                        // Adwaita keeps a raised surface with a subtle edge.
                        let (fill, stroke) = match platform {
                            crate::ZsBaseControlPlatformStyle::Windows => (
                                NativeDrawFill::Role(ColorRole::Control),
                                Some(NativeDrawFill::Role(ColorRole::Border)),
                            ),
                            crate::ZsBaseControlPlatformStyle::Macos => (
                                NativeDrawFill::Role(ColorRole::Control),
                                None,
                            ),
                            crate::ZsBaseControlPlatformStyle::Gtk => (
                                NativeDrawFill::Role(ColorRole::SurfaceRaised),
                                Some(NativeDrawFill::Role(ColorRole::Border)),
                            ),
                        };
                        cx.draw(NativeDrawCommand::RoundRect {
                            rect: bounds,
                            fill,
                            stroke,
                            radius: radius_px(
                                self.style.radius.or(Some(metrics.button_radius)),
                                cx.dpi,
                            ),
                        });
                        let mut text_style = SemanticTextStyle::body();
                        text_style.role = TextRole::Button;
                        text_style.horizontal_align = crate::HorizontalAlign::Center;
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            label,
                            button_content_bounds(bounds, self.style.padding, cx.dpi),
                            text_style,
                        )));
                    }
                    ZsButtonPresentation::Toolbar {
                        icon,
                        show_label,
                        platform,
                    } => {
                        let metrics = crate::ZsBaseControlMetrics::for_platform(*platform);
                        let icon_size = Dp::new(match platform {
                            // Keep the Windows command surface on the WinUI
                            // AppBarButton 20 epx icon metric. AppKit and GTK
                            // keep their denser 16-point symbolic actions.
                            crate::ZsBaseControlPlatformStyle::Windows => 20.0,
                            crate::ZsBaseControlPlatformStyle::Macos
                            | crate::ZsBaseControlPlatformStyle::Gtk => 16.0,
                        })
                            .to_px(cx.dpi)
                            .round_i32()
                            .max(1)
                            .min(bounds.height.max(1));
                        let padding = metrics
                            .button_padding_left
                            .to_px(cx.dpi)
                            .round_i32()
                            .max(0);
                        let content_gap = Dp::new(match platform {
                            crate::ZsBaseControlPlatformStyle::Windows => 8.0,
                            crate::ZsBaseControlPlatformStyle::Macos
                            | crate::ZsBaseControlPlatformStyle::Gtk => 6.0,
                        })
                        .to_px(cx.dpi)
                        .round_i32()
                        .max(0);
                        let icon_bounds = Rect {
                            x: if *show_label {
                                bounds.x.saturating_add(padding)
                            } else {
                                bounds
                                    .x
                                    .saturating_add(bounds.width.saturating_sub(icon_size) / 2)
                            },
                            y: bounds
                                .y
                                .saturating_add(bounds.height.saturating_sub(icon_size) / 2),
                            width: icon_size,
                            height: icon_size,
                        };
                        cx.draw(NativeDrawCommand::Icon(
                            crate::NativeDrawIconCommand::new(
                                *icon,
                                icon_bounds,
                                crate::NativeIconColorMode::ThemeAware,
                            )
                            .with_color(ColorRole::PrimaryText),
                        ));
                        if *show_label {
                            let text_x = icon_bounds
                                .x
                                .saturating_add(icon_bounds.width)
                                .saturating_add(content_gap);
                            let mut text_style = SemanticTextStyle::body();
                            // A label placed to the right of a Windows command
                            // icon is control content, not caption metadata.
                            // Body selects Segoe UI Variable Text instead of
                            // the smaller optical master used by Caption.
                            text_style.role = TextRole::Body;
                            text_style.horizontal_align = crate::HorizontalAlign::Start;
                            cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                                label,
                                Rect {
                                    x: text_x,
                                    y: bounds.y,
                                    width: bounds
                                        .x
                                        .saturating_add(bounds.width)
                                        .saturating_sub(padding)
                                        .saturating_sub(text_x)
                                        .max(0),
                                    height: bounds.height,
                                },
                                text_style,
                            )));
                        }
                    }
                    ZsButtonPresentation::NavigationItem { icon, selected } => {
                        let plan = crate::zs_navigation_item_render_plan(
                            bounds,
                            *selected,
                            crate::ZsBaseControlPlatformStyle::current(),
                            cx.dpi,
                        );
                        for command in
                            crate::zs_navigation_item_native_draw_plan(&plan, label, *icon).commands
                        {
                            cx.draw(command);
                        }
                    }
                }
            }
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { label, checked, .. } => {
                let plan = crate::zs_toggle_button_render_plan(
                    bounds,
                    *checked,
                    crate::ZsToggleButtonPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_toggle_button_native_draw_plan(&plan, label).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "breadcrumb")]
            ViewNodeKind::BreadcrumbBar { items, focused, .. } => {
                let plan = crate::zs_breadcrumb_render_plan(
                    bounds,
                    items,
                    false,
                    crate::ZsBreadcrumbPlatformStyle::current(),
                    cx.dpi,
                    None,
                );
                for command in
                    crate::zs_breadcrumb_native_draw_plan(&plan, items, *focused).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox {
                value,
                multiline,
                wrap,
                ..
            } => {
                let metrics = crate::ZsBaseControlMetrics::for_platform(
                    crate::ZsBaseControlPlatformStyle::current(),
                );
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Surface),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                    radius: radius_px(
                        self.style.radius.or(Some(metrics.text_input_radius)),
                        cx.dpi,
                    ),
                });
                let mut text_style = SemanticTextStyle::body();
                if *multiline {
                    text_style.vertical_align = crate::VerticalAlign::Start;
                    text_style.wrap = *wrap;
                    text_style.ellipsis = false;
                }
                let text_bounds = text_input_content_bounds(bounds, self.style.padding, cx.dpi);
                if *multiline && *wrap == crate::TextWrap::NoWrap {
                    let line_height = Dp::new(
                        crate::TextRole::Body
                            .metrics_for(crate::ZsTypographyPlatformStyle::current())
                            .line_height,
                    )
                    .to_px(cx.dpi)
                    .round_i32()
                    .max(1);
                    let bottom = text_bounds.y.saturating_add(text_bounds.height);
                    for (row, line) in value.split('\n').enumerate() {
                        let y = text_bounds.y.saturating_add(
                            i32::try_from(row)
                                .unwrap_or(i32::MAX)
                                .saturating_mul(line_height),
                        );
                        if y >= bottom {
                            break;
                        }
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            line,
                            Rect {
                                x: text_bounds.x,
                                y,
                                width: text_bounds.width,
                                height: line_height.min(bottom.saturating_sub(y)).max(1),
                            },
                            text_style,
                        )));
                    }
                } else {
                    cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                        value,
                        text_bounds,
                        text_style,
                    )));
                }
            }
            #[cfg(feature = "password-box")]
            ViewNodeKind::PasswordBox {
                value, reveal_mode, ..
            } => {
                let plan = crate::zs_password_box_render_plan(
                    bounds,
                    *reveal_mode,
                    !value.is_empty(),
                    crate::ZsPasswordBoxPlatformStyle::current(),
                    cx.dpi,
                );
                for command in
                    crate::zs_password_box_native_draw_plan(&plan, value, *reveal_mode, false)
                        .commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { label, checked, .. } => {
                let metrics = crate::ZsBaseControlMetrics::for_platform(
                    crate::ZsBaseControlPlatformStyle::current(),
                );
                let indicator_size = metrics
                    .check_indicator_size
                    .to_px(cx.dpi)
                    .round_i32()
                    .min(bounds.height.max(1))
                    .max(1);
                let check_bounds = Rect {
                    x: bounds.x,
                    y: bounds.y + (bounds.height - indicator_size) / 2,
                    width: indicator_size,
                    height: indicator_size,
                };
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: check_bounds,
                    fill: NativeDrawFill::Role(if *checked {
                        ColorRole::Accent
                    } else {
                        ColorRole::Control
                    }),
                    stroke: Some(NativeDrawFill::Role(if *checked {
                        ColorRole::Accent
                    } else {
                        ColorRole::Border
                    })),
                    radius: metrics
                        .button_radius
                        .to_px(cx.dpi)
                        .round_i32()
                        .max(1),
                });
                if *checked {
                    let glyph_size = Dp::new(12.0)
                        .to_px(cx.dpi)
                        .round_i32()
                        .min(indicator_size)
                        .max(1);
                    cx.draw(NativeDrawCommand::Icon(
                        crate::NativeDrawIconCommand::new(
                            crate::ZsIcon::Check,
                            Rect {
                                x: check_bounds.x + (check_bounds.width - glyph_size) / 2,
                                y: check_bounds.y + (check_bounds.height - glyph_size) / 2,
                                width: glyph_size,
                                height: glyph_size,
                            },
                            crate::NativeIconColorMode::ThemeAware,
                        )
                        .with_color(ColorRole::AccentText),
                    ));
                }
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    Rect {
                        x: bounds.x + check_bounds.width + 8,
                        y: bounds.y,
                        width: (bounds.width - check_bounds.width - 8).max(0),
                        height: bounds.height,
                    },
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { checked, .. } => {
                let plan = crate::zs_toggle_render_plan(bounds, false, *checked, cx.dpi);
                for command in crate::zs_toggle_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                ..
            } => {
                let plan = crate::zs_time_picker_render_plan(
                    bounds,
                    *value,
                    *minute_increment,
                    *clock,
                    false,
                    ZsTimePickerPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_time_picker_header_native_draw_plan(&plan, *value).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "color-picker")]
            ViewNodeKind::ColorPicker { state, .. } => {
                let plan = crate::zs_color_picker_render_plan(
                    bounds,
                    *state,
                    ZsColorPickerPlatformStyle::current(),
                    cx.dpi,
                );
                for command in
                    crate::zs_color_picker_header_native_draw_plan(&plan, *state).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                ..
            } => {
                let plan = crate::zs_date_picker_render_plan_with_today(
                    bounds,
                    *value,
                    *visible_month,
                    *minimum,
                    *maximum,
                    *today,
                    false,
                    cx.dpi,
                );
                for command in crate::zs_date_picker_header_native_draw_plan(&plan, *value).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "slider")]
            ViewNodeKind::Slider { value, range, .. } => {
                let plan = crate::zs_slider_render_plan(bounds, range.fraction(*value), cx.dpi);
                for command in crate::zs_slider_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "number-box")]
            ViewNodeKind::NumberBox {
                value,
                draft,
                range,
                format,
                wraps,
                ..
            } => {
                let valid = draft.trim().is_empty()
                    || format
                        .parse(draft)
                        .is_some_and(|candidate| range.contains(candidate));
                let plan = crate::zs_number_box_render_plan(
                    bounds,
                    crate::ZsNumberBoxPlatformStyle::current(),
                    cx.dpi,
                );
                let current = format
                    .parse(draft)
                    .filter(|candidate| range.contains(*candidate))
                    .or(*value);
                let decrement_enabled =
                    *wraps || current.is_some_and(|current| current > range.min());
                let increment_enabled =
                    *wraps || current.map_or(true, |current| current < range.max());
                for command in crate::zs_number_box_native_draw_plan(
                    &plan,
                    draft,
                    valid,
                    decrement_enabled,
                    increment_enabled,
                )
                .commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "radio")]
            ViewNodeKind::RadioButton {
                label, selected, ..
            } => {
                let plan = crate::zs_radio_render_plan(bounds, *selected, cx.dpi);
                for command in crate::zs_radio_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
                let gap = Dp::new(8.0).to_px(cx.dpi).round_i32().max(0);
                let label_x = plan
                    .indicator
                    .x
                    .saturating_add(plan.indicator.width)
                    .saturating_add(gap);
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    Rect {
                        x: label_x,
                        y: bounds.y,
                        width: bounds
                            .x
                            .saturating_add(bounds.width)
                            .saturating_sub(label_x)
                            .max(0),
                        height: bounds.height,
                    },
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "progress")]
            ViewNodeKind::ProgressBar { value, range } => {
                let plan =
                    crate::zs_progress_bar_render_plan(bounds, range.fraction(*value), cx.dpi);
                for command in crate::zs_progress_bar_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "progress-ring")]
            ViewNodeKind::ProgressRing { spec } => {
                let plan = crate::zs_progress_ring_render_plan(
                    *spec,
                    bounds,
                    crate::ZsProgressRingPlatformStyle::current(),
                    cx.dpi,
                    cx.animation_elapsed_ms,
                );
                for command in crate::zs_progress_ring_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox {
                query,
                suggestions,
                highlighted,
                placeholder,
                no_results_text,
                query_icon,
                ..
            } => {
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
                    cx.dpi,
                );
                for command in crate::zs_auto_suggest_header_native_draw_plan(
                    &plan,
                    query,
                    placeholder.as_deref(),
                )
                .commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "tree")]
            ViewNodeKind::TreeView {
                roots,
                expanded,
                selected,
                ..
            } => {
                let plan = crate::zs_tree_view_render_plan(
                    bounds,
                    roots,
                    expanded,
                    *selected,
                    crate::ZsTreePlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_tree_view_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "grid-view")]
            ViewNodeKind::GridView {
                items, selected, ..
            } => {
                let plan = crate::zs_grid_view_render_plan(
                    bounds,
                    items,
                    *selected,
                    crate::ZsGridViewPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_grid_view_native_draw_plan(&plan, items).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "table")]
            ViewNodeKind::DataGrid {
                columns,
                rows,
                selected,
                sort,
                ..
            } => {
                let plan = crate::zs_table_render_plan(
                    bounds,
                    columns,
                    rows,
                    *selected,
                    *sort,
                    crate::ZsTablePlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_table_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "info-bar")]
            ViewNodeKind::InfoBar {
                spec,
                focused_control,
                ..
            } => {
                let plan = crate::zs_info_bar_render_plan(
                    bounds,
                    spec,
                    *focused_control,
                    crate::ZsInfoBarPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_info_bar_native_draw_plan(&plan, spec).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                placeholder,
                ..
            } => {
                let plan = crate::zs_combo_box_render_plan(bounds, options.len(), false, cx.dpi);
                let selected_text = selected_index
                    .and_then(|index| options.get(index))
                    .map(String::as_str);
                for command in crate::zs_combo_box_header_native_draw_plan(
                    &plan,
                    selected_text,
                    placeholder.as_deref(),
                )
                .commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "tabs")]
            ViewNodeKind::Tabs { tabs, selected, .. } => {
                let selected_index = selected.and_then(|selected| {
                    tabs.iter().position(|candidate| candidate.id == selected)
                });
                let plan = crate::zs_tab_view_render_plan_for_tabs(
                    bounds,
                    tabs,
                    selected_index,
                    crate::ZsTabPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_tab_view_native_draw_plan_for_tabs(&plan, tabs).commands {
                    cx.draw(command);
                }
                if let Some(child) = selected_index.and_then(|index| self.children.get(index)) {
                    cx.draw(NativeDrawCommand::PushClip {
                        rect: plan.content_bounds,
                    });
                    child.paint(cx);
                    cx.draw(NativeDrawCommand::PopClip);
                }
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "list")]
            ViewNodeKind::List { selected_index, .. } => {
                if let Some(bounds) = selected_index
                    .and_then(|index| self.children.get(index))
                    .and_then(ViewNode::bounds)
                {
                    cx.draw(NativeDrawCommand::RoundFill {
                        rect: bounds,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Accent,
                            alpha: 36,
                        },
                        radius: radius_px(self.style.radius.or(Some(Dp::new(4.0))), cx.dpi),
                    });
                }
            }
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList {
                row_height,
                row_indices,
                selected_index,
                offset_y,
                visible_range,
                show_placeholders,
                ..
            } => {
                cx.draw(NativeDrawCommand::PushClip { rect: bounds });
                if let Some(selected_bounds) = selected_index
                    .and_then(|index| row_indices.binary_search(&index).ok())
                    .and_then(|position| self.children.get(position))
                    .and_then(ViewNode::bounds)
                {
                    cx.draw(NativeDrawCommand::RoundFill {
                        rect: selected_bounds,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Accent,
                            alpha: 36,
                        },
                        radius: radius_px(self.style.radius.or(Some(Dp::new(4.0))), cx.dpi),
                    });
                }
                if *show_placeholders {
                    let content_bounds = inset_bounds(bounds, self.style.padding, cx.dpi);
                    for index in visible_range.start..visible_range.end {
                        if row_indices.binary_search(&index).is_ok() {
                            continue;
                        }
                        let row_bounds = virtual_list_row_bounds(
                            content_bounds,
                            index,
                            *row_height,
                            *offset_y,
                            cx.dpi,
                        );
                        let inset_x = 8.min(row_bounds.width / 4).max(0);
                        let inset_y = 6.min(row_bounds.height / 4).max(0);
                        let placeholder = Rect {
                            x: row_bounds.x + inset_x,
                            y: row_bounds.y + inset_y,
                            width: (row_bounds.width - inset_x * 2).max(0),
                            height: (row_bounds.height - inset_y * 2).max(0),
                        };
                        if placeholder.width > 0 && placeholder.height > 0 {
                            cx.draw(NativeDrawCommand::RoundFill {
                                rect: placeholder,
                                fill: NativeDrawFill::RoleWithAlpha {
                                    role: ColorRole::Control,
                                    alpha: 96,
                                },
                                radius: 4,
                            });
                        }
                    }
                }
                for child in &self.children {
                    child.paint(cx);
                }
                cx.draw(NativeDrawCommand::PopClip);
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => {
                cx.draw(NativeDrawCommand::PushClip { rect: bounds });
                for child in &self.children {
                    child.paint(cx);
                }
                cx.draw(NativeDrawCommand::PopClip);
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "grid")]
            ViewNodeKind::Grid { .. } => {}
            #[cfg(feature = "dialog")]
            ViewNodeKind::ContentDialog { .. } => {}
            #[cfg(feature = "command-palette")]
            ViewNodeKind::CommandPalette { .. } => {}
            #[cfg(feature = "toast")]
            ViewNodeKind::ToastPresenter { .. } => {}
            #[cfg(feature = "teaching-tip")]
            ViewNodeKind::TeachingTip { .. } => {}
            ViewNodeKind::Stack { .. } | ViewNodeKind::Spacer | ViewNodeKind::__Message(_) => {}
        }

        for child in &self.children {
            child.paint(cx);
        }
        cx.finish_node(self);
    }
}
