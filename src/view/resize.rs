impl<Msg> ViewNode<Msg> {
    fn normalize_resize_constraints(&mut self) {
        if self.on_pane_resize.is_some() {
            let limits = self.pane_resize_limits.unwrap_or_default();
            #[cfg(any(feature = "split-view", feature = "label"))]
            let profile = crate::platform_component_profile::PlatformComponentProfile::for_style(
                self.resolved_platform_style(),
            );
            match &mut self.kind {
                #[cfg(feature = "split-view")]
                ViewNodeKind::SplitView { spec, .. } => {
                    *spec = spec.pane_extent(
                        limits.clamp(
                            spec.requested_pane_width()
                                .unwrap_or(profile.split_view.preferred_pane_width),
                        ),
                    );
                }
                #[cfg(feature = "label")]
                ViewNodeKind::NavigationView { pane_width, .. } => {
                    *pane_width = Some(
                        limits.clamp(pane_width.unwrap_or(profile.navigation.preferred_pane_width)),
                    );
                }
                _ => {
                    let _ = limits;
                }
            }
        }
        #[cfg(feature = "table")]
        if self.on_table_column_resize.is_some() || self.on_table_columns_change.is_some() {
            if let ViewNodeKind::DataGrid { columns, .. } = &mut self.kind {
                for column in columns.iter_mut().filter(|column| column.is_resizable()) {
                    if let crate::ZsTableColumnWidth::Fixed(width) = column.width() {
                        column.resize_to(width);
                    }
                }
            }
        }
    }
    /// Emits changes to a navigation pane's controlled open state.
    #[cfg(feature = "label")]
    pub fn on_navigation_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        self.on_navigation_open_change = Some(ViewMessageMapper::from_function(message));
        self
    }

    #[cfg(feature = "label")]
    pub fn on_navigation_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        self.on_navigation_open_change = Some(ViewMessageMapper::from_shared(message));
        self
    }
    #[cfg(feature = "accessibility")]
    fn resize_handle_label(&self, handle: crate::ZsResizeHandle) -> Option<String> {
        if self.id == Some(handle.owner()) {
            #[cfg(feature = "table")]
            if let (
                ViewNodeKind::DataGrid { columns, .. },
                crate::ZsResizeTarget::TableColumn(column),
            ) = (&self.kind, handle.target())
            {
                return columns
                    .iter()
                    .find(|candidate| candidate.id() == column)
                    .map(|column| column.header().to_owned());
            }
            return self
                .accessibility
                .as_ref()
                .and_then(|spec| spec.label.clone());
        }
        self.children
            .iter()
            .find_map(|child| child.resize_handle_label(handle))
    }
    /// Sets logical limits for a controlled SplitView or NavigationView pane.
    pub fn pane_resize_limits(mut self, limits: crate::ZsResizeLimits) -> Self {
        self.pane_resize_limits = Some(limits);
        self
    }

    /// Enables divider dragging and emits the next logical pane extent.
    ///
    /// Store the value in application state and feed it back through the pane
    /// width/extent builder so it survives unrelated View rebuilds.
    pub fn on_pane_resize(mut self, message: fn(Dp) -> Msg) -> Self {
        self.on_pane_resize = Some(ViewMessageMapper::from_function(message));
        self
    }

    pub fn on_pane_resize_with(
        mut self,
        message: impl Fn(Dp) -> Msg + Send + Sync + 'static,
    ) -> Self {
        self.on_pane_resize = Some(ViewMessageMapper::from_shared(message));
        self
    }

    #[cfg(feature = "table")]
    pub fn on_table_column_resize(
        mut self,
        message: fn(crate::ZsTableColumnResize) -> Msg,
    ) -> Self {
        self.on_table_column_resize = Some(ViewMessageMapper::from_function(message));
        self
    }

    #[cfg(feature = "table")]
    pub fn on_table_column_resize_with(
        mut self,
        message: impl Fn(crate::ZsTableColumnResize) -> Msg + Send + Sync + 'static,
    ) -> Self {
        self.on_table_column_resize = Some(ViewMessageMapper::from_shared(message));
        self
    }

    #[cfg(feature = "table")]
    pub(crate) fn on_table_columns_change_with(
        mut self,
        message: impl Fn(Vec<crate::ZsTableColumn>) -> Msg + Send + Sync + 'static,
    ) -> Self {
        self.on_table_columns_change = Some(ViewMessageMapper::from_shared(message));
        self
    }

    fn apply_resize(
        &mut self,
        target: crate::ZsResizeTarget,
        extent: Dp,
        cx: &mut ViewEventCx<Msg>,
    ) {
        if !extent.0.is_finite() || extent.0 < 0.0 {
            return;
        }
        match target {
            crate::ZsResizeTarget::Pane => {
                let Some(callback) = self.on_pane_resize.as_ref() else {
                    return;
                };
                let extent = self.pane_resize_limits.unwrap_or_default().clamp(extent);
                let supported = match &mut self.kind {
                    #[cfg(feature = "split-view")]
                    ViewNodeKind::SplitView { spec, .. } => {
                        *spec = spec.pane_width(extent);
                        true
                    }
                    #[cfg(feature = "label")]
                    ViewNodeKind::NavigationView { pane_width, .. } => {
                        *pane_width = Some(extent);
                        true
                    }
                    _ => false,
                };
                if supported {
                    cx.emit(callback.map(extent));
                }
            }
            #[cfg(feature = "table")]
            crate::ZsResizeTarget::TableColumn(column) => {
                if self.on_table_column_resize.is_none() && self.on_table_columns_change.is_none() {
                    return;
                }
                if let ViewNodeKind::DataGrid { columns, .. } = &mut self.kind {
                    if let Some(column) = columns
                        .iter_mut()
                        .find(|candidate| candidate.id() == column && candidate.is_resizable())
                    {
                        let width = column.resize_to(extent);
                        if let Some(callback) = self.on_table_column_resize.as_ref() {
                            cx.emit(callback.map(crate::ZsTableColumnResize {
                                column: column.id(),
                                width,
                            }));
                        }
                        if let Some(callback) = self.on_table_columns_change.as_ref() {
                            cx.emit(callback.map(columns.clone()));
                        }
                    }
                }
            }
        }
    }

    fn push_pane_resize_target(
        &self,
        hit_targets: &mut Vec<ViewHitTarget>,
        owner: Rect,
        divider: Rect,
        pane: Rect,
        axis: crate::ZsResizeAxis,
        available: i32,
        reversed: bool,
        clip: Option<Rect>,
    ) {
        let Some(widget) = self.id else {
            return;
        };
        if self.on_pane_resize.is_none() {
            return;
        }
        let handle = crate::ZsResizeHandle::new(
            widget,
            crate::ZsResizeTarget::Pane,
            axis,
            axis.extent(pane),
            self.pane_resize_limits.unwrap_or_default(),
            available,
            reversed,
            self.layout_dpi,
        );
        if !handle.is_adjustable() {
            return;
        }
        let bounds = crate::resize::resize_hit_bounds(divider, owner, axis, self.layout_dpi);
        if let Some(bounds) = clipped_rect(bounds, clip) {
            hit_targets.push(ViewHitTarget::with_kind(
                handle.widget_id(),
                bounds,
                ViewHitTargetKind::ResizeHandle(handle),
            ));
        }
    }
}
