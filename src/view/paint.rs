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

    pub(crate) fn set_typography_scale(&mut self, scale: f32) {
        self.plan.set_typography_scale(scale);
    }

    fn finish_node<Msg: Clone>(&mut self, _root: &ViewNode<Msg>) {
        self.paint_depth = self.paint_depth.saturating_sub(1);
        #[cfg(any(
            feature = "auto-suggest",
            feature = "breadcrumb",
            feature = "color-picker",
            feature = "command-palette",
            feature = "combo",
            feature = "date-picker",
            feature = "dialog",
            feature = "flyout",
            feature = "menu-flyout",
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
        let tab_bounds = tab_layout_bounds(cx.bounds, self.style.padding, cx.dpi);
        let plan = crate::zs_tab_view_render_plan_for_tabs(
            tab_bounds,
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
            let child_bounds = constrained_child_bounds(
                plan.content_bounds,
                child,
                cx.dpi,
                cx.typography_scale(),
            );
            let mut child_cx = cx.child(child_bounds);
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

#[cfg(feature = "flyout")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_flyout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
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
        let split = 1.min(self.children.len());
        let (page_children, overlay_children) = self.children.split_at_mut(split);
        let Some(page) = page_children.first_mut() else {
            return LayoutOutput {
                bounds: cx.bounds,
                children,
            };
        };
        let mut page_cx = cx.child(cx.bounds);
        children.extend(page.layout(&mut page_cx).children);

        let (spec, open, target) = match &self.kind {
            ViewNodeKind::Flyout {
                spec, open, target, ..
            } => (*spec, *open, *target),
            _ => unreachable!("flyout layout requires a flyout node"),
        };
        if open {
            if let (Some(content), Some(target_bounds)) =
                (overlay_children.first_mut(), page.widget_layout_bounds(target))
            {
                let plan = crate::zs_flyout_render_plan(
                    cx.bounds,
                    target_bounds,
                    spec,
                    crate::ZsFlyoutPlatformStyle::current(),
                    cx.dpi,
                );
                let mut content_cx = cx.child(plan.content);
                children.extend(content.layout(&mut content_cx).children);
            }
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
            let mut child_cx = cx.child(Rect {
                    x: content_bounds.x,
                    y: content_bounds.y.saturating_add(row_top),
                    width: content_bounds.width,
                    height: row_height_px,
                });
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

#[cfg(feature = "label")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_navigation_view(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let platform = self.resolved_platform_style();
        let (
            item_count,
            footer_count,
            pane_open,
            pane_width,
            minimum_content_width,
        ) = match &self.kind {
            ViewNodeKind::NavigationView {
                item_count,
                footer_count,
                pane_open,
                pane_width,
                minimum_content_width,
                ..
            } => (
                *item_count,
                *footer_count,
                *pane_open,
                *pane_width,
                *minimum_content_width,
            ),
            _ => unreachable!("navigation layout requires a navigation view node"),
        };
        let layout = zs_navigation_view_layout(
            cx.bounds,
            platform,
            pane_width,
            minimum_content_width,
            pane_open,
            cx.dpi,
            cx.typography_scale(),
        );
        if layout.mode == ZsNavigationViewLayoutMode::Expanded {
            if let ViewNodeKind::NavigationView { pane_open, .. } = &mut self.kind {
                *pane_open = false;
            }
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

        let content_index = item_count.saturating_add(footer_count);
        let split_index = content_index.min(self.children.len());
        let (navigation_children, content_children) = self.children.split_at_mut(split_index);
        let Some(content) = content_children.first_mut() else {
            return LayoutOutput {
                bounds: cx.bounds,
                children,
            };
        };
        let mut content_cx = cx.child(layout.content_bounds);
        children.extend(content.layout(&mut content_cx).children);

        if layout.pane_bounds.is_some() {
            let (items, footer_items) = navigation_children.split_at_mut(item_count);
            let spacing = crate::ZsuiSpacingTokens::for_platform(platform);
            let navigation_profile =
                crate::platform_component_profile::PlatformComponentProfile::for_style(platform)
                    .navigation;
            let item_gap = spacing.xs.to_px(cx.dpi).round_i32().max(0);
            let footer_gap = spacing.xs.to_px(cx.dpi).round_i32().max(0);
            let pane_padding = navigation_profile
                .pane_padding(spacing)
                .to_px(cx.dpi)
                .round_i32()
                .max(0);
            let show_footer =
                layout.mode == ZsNavigationViewLayoutMode::Expanded || layout.overlay_open;
            let footer_heights = if show_footer {
                footer_items
                    .iter()
                    .map(|child| {
                        navigation_intrinsic_height_px(child, cx.dpi, cx.typography_scale())
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let footer_total = footer_heights
                .iter()
                .copied()
                .fold(0i32, i32::saturating_add)
                .saturating_add(
                    footer_gap.saturating_mul(footer_heights.len().saturating_sub(1) as i32),
                );
            let footer_top = layout
                .footer_bounds
                .y
                .saturating_add(layout.footer_bounds.height)
                .saturating_sub(footer_total);
            let item_bottom = if footer_total > 0 {
                footer_top.saturating_sub(pane_padding)
            } else {
                layout
                    .item_bounds
                    .y
                    .saturating_add(layout.item_bounds.height)
            };
            let mut y = layout.item_bounds.y;
            for item in items {
                let height = navigation_intrinsic_height_px(item, cx.dpi, cx.typography_scale());
                if y.saturating_add(height) > item_bottom {
                    break;
                }
                let mut item_cx = cx.child(Rect {
                        x: layout.item_bounds.x,
                        y,
                        width: layout.item_bounds.width,
                        height,
                    });
                children.extend(item.layout(&mut item_cx).children);
                y = y.saturating_add(height).saturating_add(item_gap);
            }
            if show_footer {
                let mut footer_y = footer_top;
                for (item, height) in footer_items.iter_mut().zip(footer_heights) {
                    let mut item_cx = cx.child(Rect {
                            x: layout.footer_bounds.x,
                            y: footer_y,
                            width: layout.footer_bounds.width,
                            height,
                        });
                    children.extend(item.layout(&mut item_cx).children);
                    footer_y = footer_y.saturating_add(height).saturating_add(footer_gap);
                }
            }
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

impl<Msg: Clone> View<Msg> for ViewNode<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        if cx.is_root() {
            self.assign_automatic_ids();
        }
        #[cfg(feature = "menu-flyout")]
        if matches!(self.kind, ViewNodeKind::MenuFlyout { .. }) {
            self.bounds = Some(cx.bounds);
            self.layout_dpi = cx.dpi;
            for child in &mut self.children {
                child.clear_layout_bounds();
            }
            let mut children = Vec::new();
            if let Some(page) = self.children.first_mut() {
                let mut page_cx = cx.child(cx.bounds);
                children.extend(page.layout(&mut page_cx).children);
            }
            return LayoutOutput {
                bounds: cx.bounds,
                children,
            };
        }
        #[cfg(feature = "flyout")]
        if matches!(self.kind, ViewNodeKind::Flyout { .. }) {
            return self.layout_flyout(cx);
        }
        #[cfg(feature = "label")]
        if matches!(self.kind, ViewNodeKind::NavigationView { .. }) {
            return self.layout_navigation_view(cx);
        }
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

        let content_bounds = inset_bounds(cx.bounds, self.style.padding, cx.dpi);
        #[cfg(feature = "list")]
        let content_bounds = horizontal_inset_bounds(
            content_bounds,
            self.list_item_horizontal_inset,
            cx.dpi,
        );
        #[cfg(feature = "scroll")]
        if let ViewNodeKind::Scroll {
            offset_y,
            content_height,
            ..
        } = &mut self.kind
        {
            let max_offset = scroll_max_offset_y(Some(content_bounds), *content_height, cx.dpi);
            let requested = if offset_y.0.is_finite() {
                offset_y.0.max(0.0)
            } else {
                0.0
            };
            *offset_y = Dp::new(requested.min(max_offset.0));
        }
        let child_bounds = split_child_bounds(
            content_bounds,
            &self.kind,
            &self.children,
            self.style.gap,
            cx.dpi,
            cx.typography_scale(),
        );
        for (child, bounds) in self.children.iter_mut().zip(child_bounds) {
            let mut child_cx = cx.child(bounds);
            children.extend(child.layout(&mut child_cx).children);
        }

        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }

    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent) {
        #[cfg(feature = "menu-flyout")]
        if matches!(self.kind, ViewNodeKind::MenuFlyout { .. }) {
            let mut handled = false;
            if let ViewNodeKind::MenuFlyout {
                menu,
                open,
                target: menu_target,
                highlighted,
                open_submenus,
                on_command,
                on_open_change,
                ..
            } = &mut self.kind
            {
                match event {
                    ViewEvent::MenuFlyoutOpenChanged {
                        widget,
                        open: requested,
                    } if self.id == Some(*widget) => {
                        *open = *requested;
                        if *requested {
                            let state = crate::ZsMenuFlyoutState {
                                open: true,
                                target: *menu_target,
                                highlighted: *highlighted,
                                open_submenus: open_submenus.clone(),
                            };
                            *highlighted = state.highlighted.or_else(|| state.first_enabled(menu));
                        } else {
                            *highlighted = None;
                            open_submenus.clear();
                        }
                        if let Some(message) = on_open_change {
                            cx.emit(message.map(*requested));
                        }
                        handled = true;
                    }
                    ViewEvent::MenuFlyoutHighlighted { widget, path }
                        if *open
                            && self.id == Some(*widget)
                            && crate::menu_flyout::menu_flyout_item(menu, *path).is_some() =>
                    {
                        *highlighted = Some(*path);
                        handled = true;
                    }
                    ViewEvent::MenuFlyoutSubmenuChanged { widget, submenu }
                        if *open && self.id == Some(*widget) =>
                    {
                        let previous = open_submenus.clone();
                        let next = submenu
                            .and_then(|path| {
                                crate::menu_flyout::menu_flyout_submenu_stack(menu, path)
                            })
                            .unwrap_or_default();
                        let preserved = highlighted.filter(|path| path.level() == next.len());
                        let closed = (next.len() < previous.len())
                            .then(|| previous.get(next.len()).copied())
                            .flatten();
                        *open_submenus = next;
                        let state = crate::ZsMenuFlyoutState {
                            open: true,
                            target: *menu_target,
                            highlighted: None,
                            open_submenus: open_submenus.clone(),
                        };
                        *highlighted = preserved.or(closed).or_else(|| state.first_enabled(menu));
                        handled = true;
                    }
                    ViewEvent::MenuFlyoutInvoked { widget, path }
                        if *open && self.id == Some(*widget) =>
                    {
                        if let Some(command) = crate::menu_flyout::menu_flyout_command(menu, *path)
                        {
                            *open = false;
                            *highlighted = None;
                            open_submenus.clear();
                            if let Some(message) = on_command {
                                cx.emit(message.map(command));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message.map(false));
                            }
                            handled = true;
                        }
                    }
                    ViewEvent::DismissPopupOverlays { except }
                        if *open && self.id != *except =>
                    {
                        *open = false;
                        *highlighted = None;
                        open_submenus.clear();
                        if let Some(message) = on_open_change {
                            cx.emit(message.map(false));
                        }
                        handled = true;
                    }
                    _ => {}
                }
            }
            if !handled {
                if let Some(page) = self.children.first_mut() {
                    page.event(cx, event);
                }
            }
            return;
        }

        #[cfg(feature = "flyout")]
        if matches!(self.kind, ViewNodeKind::Flyout { .. }) {
            let mut handled = false;
            let mut route_content = false;
            if let ViewNodeKind::Flyout {
                open,
                on_dismiss,
                on_open_change,
                ..
            } = &mut self.kind
            {
                route_content = *open;
                let reason = match event {
                    ViewEvent::FlyoutDismissed { widget, reason }
                        if *open && self.id == Some(*widget) => Some(*reason),
                    ViewEvent::DismissPopupOverlays { except }
                        if *open && self.id != *except =>
                    {
                        Some(crate::ZsFlyoutDismissReason::LightDismiss)
                    }
                    _ => None,
                };
                if let Some(reason) = reason {
                    *open = false;
                    if let Some(message) = on_dismiss {
                        cx.emit(message.map(reason));
                    }
                    if let Some(message) = on_open_change {
                        cx.emit(message.map(false));
                    }
                    handled = true;
                }
            }
            if handled {
                return;
            }
            if let Some(page) = self.children.first_mut() {
                page.event(cx, event);
            }
            if route_content {
                if let Some(content) = self.children.get_mut(1) {
                    content.event(cx, event);
                }
            }
            return;
        }

        #[cfg(feature = "teaching-tip")]
        if matches!(self.kind, ViewNodeKind::TeachingTip { .. }) {
            let mut handled = false;
            if let ViewNodeKind::TeachingTip {
                spec,
                open,
                focused_control,
                on_result,
                on_open_change,
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
                                cx.emit(message.map(crate::ZsTeachingTipResult {
                                    response: *response,
                                }));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message.map(false));
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
                        cx.emit(message.map(*event));
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
                        cx.emit(message.map(*expanded));
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
                            cx.emit(message.map(false));
                        }
                    }
                    if let Some(message) = on_select {
                        cx.emit(message.map(*item));
                    }
                }
                ViewEvent::DismissPopupOverlays { except }
                    if self.id.is_some() && self.id != *except && *overflow_open =>
                {
                    *overflow_open = false;
                    if let Some(message) = on_expanded_change {
                        cx.emit(message.map(false));
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
                on_open_change,
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
                                cx.emit(message.map(result));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message.map(false));
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
                            cx.emit(message.map(*requested));
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
                                    cx.emit(message.map(item));
                                }
                            }
                            if let Some(message) = on_query_change {
                                cx.emit(message.map(value.clone()));
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
                                    cx.emit(message.map(*item));
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
                                cx.emit(message.map(*item));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message.map(false));
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
                on_open_change,
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
                                cx.emit(message.map((*button).into()));
                            }
                            if let Some(message) = on_open_change {
                                cx.emit(message.map(false));
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
                        cx.emit(message.map(*tab));
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
                            cx.emit(message.map(false));
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
                            cx.emit(message.map(false));
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
                            cx.emit(message.map(false));
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
                            cx.emit(message.map(false));
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
                            cx.emit(message.map(false));
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
                    cx.emit(message.map(index));
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
                    cx.emit(message.map(*index));
                }
                if was_expanded {
                    if let Some(message) = on_expanded_change {
                        cx.emit(message.map(false));
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
                on_month_change,
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
            let next_month = next_value.first_day_of_month();
            let month_changed = *visible_month != next_month;
            let was_expanded = *expanded;
            *value = next_value;
            *visible_month = next_month;
            *expanded = false;
            if changed {
                if let Some(message) = on_date_change {
                    cx.emit(message.map(next_value));
                }
            }
            if month_changed {
                if let Some(message) = on_month_change {
                    cx.emit(message.map(next_month));
                }
            }
            if was_expanded {
                if let Some(message) = on_expanded_change {
                    cx.emit(message.map(false));
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
                        cx.emit(message.map(next_value));
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
                            cx.emit(message.map(*expanded));
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
                            cx.emit(message.map(*channel));
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
                            cx.emit(message.map(color));
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
                #[cfg(feature = "label")]
                (
                    ViewNodeKind::NavigationView { pane_open, .. },
                    ViewEvent::Click { .. },
                ) => {
                    *pane_open = !*pane_open;
                }
                #[cfg(feature = "button")]
                (
                    ViewNodeKind::Button {
                        enabled, on_click, ..
                    },
                    ViewEvent::Click { .. },
                ) => {
                    if *enabled {
                        if let Some(message) = on_click.clone() {
                            cx.emit(message);
                        }
                    }
                }
                #[cfg(feature = "canvas")]
                (
                    ViewNodeKind::Canvas { on_click, .. },
                    ViewEvent::Click { .. },
                ) => {
                    if let Some(message) = on_click.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "canvas")]
                (
                    ViewNodeKind::Canvas { on_pointer, .. },
                    ViewEvent::CanvasPointer { event },
                ) => {
                    if let Some(message) = on_pointer {
                        cx.emit(message(*event));
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
                        cx.emit(message.map(*next_checked));
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
                        cx.emit(message.map(next_value.clone()));
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
                        cx.emit(message.map(next_value.clone()));
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
                            cx.emit(message.map(crate::ZsAutoSuggestTextChange::new(
                                String::new(),
                                crate::ZsAutoSuggestTextChangeReason::UserInput,
                            )));
                        }
                    }
                    if was_expanded {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message.map(false));
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
                        cx.emit(message.map(crate::ZsAutoSuggestTextChange::new(
                            next_query.clone(),
                            crate::ZsAutoSuggestTextChangeReason::UserInput,
                        )));
                    }
                    if expanded_changed {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message.map(next_expanded));
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
                            cx.emit(message.map(*next_expanded));
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
                                cx.emit(message.map(crate::ZsAutoSuggestTextChange::new(
                                    query.clone(),
                                    crate::ZsAutoSuggestTextChangeReason::SuggestionChosen,
                                )));
                            }
                            if let Some(message) = on_suggestion_chosen {
                                cx.emit(message.map(*suggestion));
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
                                    cx.emit(message.map(crate::ZsAutoSuggestTextChange::new(
                                        query.clone(),
                                        crate::ZsAutoSuggestTextChangeReason::SuggestionChosen,
                                    )));
                                }
                                if let Some(message) = on_suggestion_chosen {
                                    cx.emit(message.map(chosen));
                                }
                            }
                        }
                    }
                    if let Some(message) = on_query_submit {
                        cx.emit(message.map(crate::ZsAutoSuggestSubmission::new(
                            query.clone(),
                            chosen,
                        )));
                    }
                    let was_expanded = *expanded;
                    *expanded = false;
                    *highlighted = None;
                    if was_expanded {
                        if let Some(message) = on_expanded_change {
                            cx.emit(message.map(false));
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
                                cx.emit(message.map(crate::ZsTreeExpansionChange::new(
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
                            cx.emit(message.map(*node));
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
                            cx.emit(message.map(*node));
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
                            cx.emit(message.map(*item));
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
                            cx.emit(message.map(*item));
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
                        cx.emit(message.map(next_value.clone()));
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
                        cx.emit(message.map(*next_checked));
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
                        cx.emit(message.map(*next_checked));
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
                        cx.emit(message.map(*value));
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
                            cx.emit(message.map(next_value));
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
                            cx.emit(message.map(next_value));
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
                        cx.emit(message.map(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        value,
                        visible_month,
                        expanded,
                        on_month_change,
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
                        let next_month = value.first_day_of_month();
                        if *visible_month != next_month {
                            *visible_month = next_month;
                            if let Some(message) = on_month_change {
                                cx.emit(message.map(next_month));
                            }
                        }
                    }
                    if let Some(message) = on_expanded_change {
                        cx.emit(message.map(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        minimum,
                        maximum,
                        visible_month,
                        on_month_change,
                        ..
                    },
                    ViewEvent::DatePickerMonthChanged { month, .. },
                ) => {
                    let next_month = clamp_visible_month(*month, *minimum, *maximum);
                    if *visible_month != next_month {
                        *visible_month = next_month;
                        if let Some(message) = on_month_change {
                            cx.emit(message.map(next_month));
                        }
                    }
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
                            cx.emit(message.map(*next_expanded));
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
                        cx.emit(message.map(next));
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
            #[cfg(feature = "canvas")]
            ViewNodeKind::Canvas { scene, .. } => {
                for command in crate::zs_canvas_native_draw_plan(bounds, scene, cx.dpi).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "label")]
            ViewNodeKind::NavigationView {
                title,
                subtitle,
                item_count,
                footer_count,
                pane_open,
                pane_width,
                minimum_content_width,
            } => {
                let platform = self.resolved_platform_style();
                let navigation_profile =
                    crate::platform_component_profile::PlatformComponentProfile::for_style(
                        platform,
                    )
                    .navigation;
                let layout = zs_navigation_view_layout(
                    bounds,
                    platform,
                    *pane_width,
                    *minimum_content_width,
                    *pane_open,
                    cx.dpi,
                    cx.plan.typography_scale(),
                );
                cx.draw(NativeDrawCommand::FillRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Surface),
                });
                let content_index = item_count.saturating_add(*footer_count);
                if let Some(content) = self.children.get(content_index) {
                    content.paint(cx);
                }
                if let Some(header) = layout.header_bounds {
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: header,
                        fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
                    });
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: Rect {
                            x: header.x,
                            y: header.y.saturating_add(header.height).saturating_sub(1),
                            width: header.width,
                            height: 1,
                        },
                        fill: NativeDrawFill::Role(ColorRole::Border),
                    });
                }
                if let Some(scrim) = layout.scrim_bounds {
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: scrim,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::PrimaryText,
                            alpha: navigation_profile.scrim_alpha,
                        },
                    });
                }
                if let Some(pane) = layout.pane_bounds {
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: pane,
                        fill: NativeDrawFill::Role(navigation_profile.pane_color),
                    });
                    cx.draw(NativeDrawCommand::FillRect {
                        rect: Rect {
                            x: pane.x.saturating_add(pane.width).saturating_sub(1),
                            y: pane.y,
                            width: 1,
                            height: pane.height,
                        },
                        fill: NativeDrawFill::Role(ColorRole::Border),
                    });
                    if let Some(title_bounds) = layout.title_bounds {
                        let mut style = SemanticTextStyle::body();
                        style.role = navigation_profile.title_role;
                        style.weight = crate::TextWeight::Semibold;
                        style.horizontal_align = crate::HorizontalAlign::Start;
                        style.ellipsis = true;
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            title,
                            title_bounds,
                            style,
                        )));
                    }
                    if let Some(subtitle_bounds) = layout.subtitle_bounds {
                        let mut style = SemanticTextStyle::body();
                        style.role = TextRole::Caption;
                        style.color = ColorRole::SecondaryText;
                        style.horizontal_align = crate::HorizontalAlign::Start;
                        style.ellipsis = true;
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            subtitle,
                            subtitle_bounds,
                            style,
                        )));
                    }
                    for child in self.children.iter().take(content_index) {
                        child.paint(cx);
                    }
                }
                if !layout.overlay_open {
                    if let (Some(header), Some(toggle)) =
                        (layout.header_bounds, layout.toggle_bounds)
                    {
                        let text_x = toggle.x.saturating_add(toggle.width).saturating_add(8);
                        let mut style = SemanticTextStyle::body();
                        style.role = TextRole::Body;
                        style.weight = crate::TextWeight::Semibold;
                        style.horizontal_align = crate::HorizontalAlign::Start;
                        style.ellipsis = true;
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            title,
                            Rect {
                                x: text_x,
                                y: header.y,
                                width: header
                                    .x
                                    .saturating_add(header.width)
                                    .saturating_sub(12)
                                    .saturating_sub(text_x)
                                    .max(0),
                                height: header.height,
                            },
                            style,
                        )));
                    }
                }
                if let Some(toggle) = layout.toggle_bounds {
                    let icon_size = navigation_profile
                        .toggle_icon_size
                        .to_px(cx.dpi)
                        .round_i32()
                        .max(1)
                        .min(toggle.width.min(toggle.height).max(1));
                    cx.draw(NativeDrawCommand::Icon(
                        crate::NativeDrawIconCommand::new(
                            crate::ZsIcon::Sidebar,
                            Rect {
                                x: toggle
                                    .x
                                    .saturating_add(toggle.width.saturating_sub(icon_size) / 2),
                                y: toggle
                                    .y
                                    .saturating_add(toggle.height.saturating_sub(icon_size) / 2),
                                width: icon_size,
                                height: icon_size,
                            },
                            crate::NativeIconColorMode::ThemeAware,
                        )
                        .with_color(ColorRole::PrimaryText),
                    ));
                }
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "label")]
            ViewNodeKind::Text { text, style } => {
                let bounds = padded_bounds(bounds, self.style.padding, cx.dpi);
                #[cfg(feature = "list")]
                let bounds = horizontal_inset_bounds(
                    bounds,
                    self.list_item_horizontal_inset,
                    cx.dpi,
                );
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    text,
                    bounds,
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
                enabled,
                ..
            } => {
                match presentation {
                    ZsButtonPresentation::Standard | ZsButtonPresentation::Icon { .. } => {
                        let platform = self.resolved_platform_style();
                        let component_profile =
                            crate::platform_component_profile::PlatformComponentProfile::for_style(
                                platform,
                            );
                        let metrics = crate::ZsBaseControlMetrics::for_platform(platform);
                        // Standard buttons deliberately keep their platform
                        // bezel grammar: WinUI uses a bordered control,
                        // AppKit uses a clean bezel without an outline, and
                        // Adwaita keeps a raised surface with a subtle edge.
                        let fill = NativeDrawFill::Role(component_profile.button.fill);
                        let stroke =
                            component_profile.button.stroke.map(NativeDrawFill::Role);
                        cx.draw(NativeDrawCommand::RoundRect {
                            rect: bounds,
                            fill,
                            stroke,
                            radius: radius_px(
                                self.style.radius.or(Some(metrics.button_radius)),
                                cx.dpi,
                            ),
                        });
                        match presentation {
                            ZsButtonPresentation::Standard => {
                                let mut text_style = SemanticTextStyle::body();
                                text_style.role = TextRole::Button;
                                text_style.color = if *enabled {
                                    ColorRole::PrimaryText
                                } else {
                                    ColorRole::DisabledText
                                };
                                text_style.horizontal_align = crate::HorizontalAlign::Center;
                                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                                    label,
                                    button_content_bounds(bounds, self.style.padding, cx.dpi),
                                    text_style,
                                )));
                            }
                            ZsButtonPresentation::Icon { icon } => {
                                let icon_size = ZsToolbarMetrics::for_platform(platform)
                                    .icon_size
                                    .to_px(cx.dpi)
                                    .round_i32()
                                    .max(1)
                                    .min(bounds.width.min(bounds.height).max(1));
                                cx.draw(NativeDrawCommand::Icon(
                                    crate::NativeDrawIconCommand::new(
                                        *icon,
                                        Rect {
                                            x: bounds.x
                                                + bounds.width.saturating_sub(icon_size) / 2,
                                            y: bounds.y
                                                + bounds.height.saturating_sub(icon_size) / 2,
                                            width: icon_size,
                                            height: icon_size,
                                        },
                                        crate::NativeIconColorMode::ThemeAware,
                                    )
                                    .with_color(if *enabled {
                                        ColorRole::PrimaryText
                                    } else {
                                        ColorRole::DisabledText
                                    }),
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                    ZsButtonPresentation::Primary => {
                        let platform = self.resolved_platform_style();
                        let metrics = crate::ZsBaseControlMetrics::for_platform(platform);
                        cx.draw(NativeDrawCommand::RoundRect {
                            rect: bounds,
                            fill: NativeDrawFill::Role(if *enabled {
                                ColorRole::Accent
                            } else {
                                ColorRole::Control
                            }),
                            stroke: (!*enabled).then_some(NativeDrawFill::Role(ColorRole::Border)),
                            radius: radius_px(
                                self.style.radius.or(Some(metrics.button_radius)),
                                cx.dpi,
                            ),
                        });
                        let mut text_style = SemanticTextStyle::body();
                        text_style.role = TextRole::Button;
                        text_style.color = if *enabled {
                            ColorRole::AccentText
                        } else {
                            ColorRole::DisabledText
                        };
                        text_style.horizontal_align = crate::HorizontalAlign::Center;
                        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                            label,
                            button_content_bounds(bounds, self.style.padding, cx.dpi),
                            text_style,
                        )));
                    }
                    ZsButtonPresentation::Toolbar { icon, show_label } => {
                        let platform = self.resolved_platform_style();
                        let base = crate::ZsBaseControlMetrics::for_platform(platform);
                        let metrics = ZsToolbarMetrics::for_platform(platform);
                        let icon_size = metrics
                            .icon_size
                            .to_px(cx.dpi)
                            .round_i32()
                            .max(1)
                            .min(bounds.height.max(1));
                        let padding = base
                            .button_padding_left
                            .to_px(cx.dpi)
                            .round_i32()
                            .max(0);
                        let content_gap = metrics
                            .content_gap
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
                            .with_color(if *enabled {
                                ColorRole::PrimaryText
                            } else {
                                ColorRole::DisabledText
                            }),
                        ));
                        if *show_label {
                            let text_x = icon_bounds
                                .x
                                .saturating_add(icon_bounds.width)
                                .saturating_add(content_gap);
                            let mut text_style = SemanticTextStyle::body();
                            text_style.role = metrics.label_role;
                            text_style.color = if *enabled {
                                ColorRole::PrimaryText
                            } else {
                                ColorRole::DisabledText
                            };
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
                let tab_bounds = tab_layout_bounds(bounds, self.style.padding, cx.dpi);
                let plan = crate::zs_tab_view_render_plan_for_tabs(
                    tab_bounds,
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
            #[cfg(feature = "flyout")]
            ViewNodeKind::Flyout { .. } => {
                if let Some(page) = self.children.first() {
                    page.paint(cx);
                }
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "menu-flyout")]
            ViewNodeKind::MenuFlyout { .. } => {
                if let Some(page) = self.children.first() {
                    page.paint(cx);
                }
                cx.finish_node(self);
                return;
            }
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
