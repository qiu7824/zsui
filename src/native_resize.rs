use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct NativeResizeDrag {
    handle: crate::ZsResizeHandle,
    start: Point,
    dpi: Dpi,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(
        runtime: &NativeViewInputRuntime,
        target: crate::ZsResizeTarget,
    ) -> crate::ViewHitTarget {
        runtime.current_interaction_plan().unwrap().hit_targets.into_iter().find(|hit| {
            matches!(hit.kind, crate::ViewHitTargetKind::ResizeHandle(value) if value.target() == target)
        }).expect("enabled resize handle")
    }

    fn center(bounds: Rect) -> Point {
        Point {
            x: bounds.x + bounds.width / 2,
            y: bounds.y + bounds.height / 2,
        }
    }

    fn extent(hit: crate::ViewHitTarget) -> i32 {
        match hit.kind {
            crate::ViewHitTargetKind::ResizeHandle(value) => value.extent_px,
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "split-view")]
    #[test]
    fn resize_panels_keep_controlled_state_capture_direction_and_escape() {
        for axis in [
            crate::ZsResizeAxis::Horizontal,
            crate::ZsResizeAxis::Vertical,
        ] {
            for trailing in [false, true] {
                let builder = native_window("Panels").size(900, 720).stateful_view(
                    Dp::new(180.0),
                    move |width| {
                        crate::split_view(
                            crate::WidgetId::new(700),
                            crate::ZsSplitViewSpec::new(true)
                                .axis(axis)
                                .display_mode(crate::ZsSplitViewDisplayMode::Inline)
                                .pane_placement(if trailing {
                                    crate::ZsSplitViewPanePlacement::Trailing
                                } else {
                                    crate::ZsSplitViewPanePlacement::Leading
                                })
                                .pane_extent(*width)
                                .minimum_content_extent(Dp::new(80.0)),
                            crate::column([]),
                            crate::column([]),
                        )
                        .pane_resize_limits(
                            crate::ZsResizeLimits::new(Dp::new(100.0), Dp::new(300.0)).unwrap(),
                        )
                        .on_pane_resize(std::convert::identity)
                    },
                    |width, next, _cx| *width = next,
                );
                let mut runtime = builder.native_view_input_runtime();
                let target = handle(&runtime, crate::ZsResizeTarget::Pane);
                let start = center(target.bounds);
                let mut end = start;
                let delta = if trailing { -40 } else { 40 };
                match axis {
                    crate::ZsResizeAxis::Horizontal => end.x += delta,
                    crate::ZsResizeAxis::Vertical => end.y += delta,
                }
                assert!(
                    runtime
                        .dispatch_pointer_down(start, false)
                        .resize_drag_active
                );
                assert!(runtime.dispatch_pointer_move(end).resize_extent_changed);
                assert_eq!(extent(handle(&runtime, crate::ZsResizeTarget::Pane)), 220);
                assert!(!runtime.dispatch_pointer_up(end).resize_drag_active);
                let key = match (axis, trailing) {
                    (crate::ZsResizeAxis::Horizontal, false) => NativeViewKey::Right,
                    (crate::ZsResizeAxis::Horizontal, true) => NativeViewKey::Left,
                    (crate::ZsResizeAxis::Vertical, false) => NativeViewKey::Down,
                    (crate::ZsResizeAxis::Vertical, true) => NativeViewKey::Up,
                };
                assert!(runtime.dispatch_key(key).resize_extent_changed);
                assert_eq!(extent(handle(&runtime, crate::ZsResizeTarget::Pane)), 228);
                let start = center(handle(&runtime, crate::ZsResizeTarget::Pane).bounds);
                runtime.dispatch_pointer_down(start, false);
                runtime.dispatch_pointer_move(end);
                assert!(runtime.dispatch_key(NativeViewKey::Escape).handled);
                assert!(runtime.resize_drag.is_none());
                assert_eq!(extent(handle(&runtime, crate::ZsResizeTarget::Pane)), 228);
            }
        }
    }

    #[cfg(feature = "label")]
    #[test]
    fn resize_navigation_pane_is_controlled_across_live_rebuilds() {
        for width in [640, 1200] {
            let builder = native_window("Navigation").size(width, 720).stateful_view(
                Dp::new(240.0),
                |width| {
                    crate::navigation_view(
                        crate::ZsNavigationViewSpec::new("Files", "")
                            .pane_width(*width)
                            .minimum_content_width(Dp::new(320.0))
                            .content(crate::WidgetId::new(701), crate::column([])),
                    )
                    .on_pane_resize(std::convert::identity)
                },
                |width, next, _cx| *width = next,
            );
            let mut runtime = builder.native_view_input_runtime();
            if !runtime
                .current_interaction_plan()
                .unwrap()
                .hit_targets
                .iter()
                .any(|target| matches!(target.kind, crate::ViewHitTargetKind::ResizeHandle(_)))
            {
                let toggle = runtime
                    .current_interaction_plan()
                    .unwrap()
                    .hit_targets
                    .into_iter()
                    .find(|target| target.kind == crate::ViewHitTargetKind::NavigationViewToggle)
                    .unwrap();
                assert!(
                    runtime
                        .dispatch_pointer_click(center(toggle.bounds))
                        .handled
                );
            }
            let target = handle(&runtime, crate::ZsResizeTarget::Pane);
            let start = center(target.bounds);
            runtime.dispatch_pointer_down(start, false);
            runtime.dispatch_pointer_move(Point {
                x: start.x + 64,
                y: start.y,
            });
            runtime.dispatch_pointer_up(Point {
                x: start.x + 64,
                y: start.y,
            });
            assert_eq!(extent(handle(&runtime, crate::ZsResizeTarget::Pane)), 304);
        }
    }

    #[cfg(feature = "table")]
    #[test]
    fn resize_table_columns_use_ids_and_do_not_sort_on_divider_release() {
        #[derive(Clone)]
        enum Msg {
            Resize(crate::ZsTableColumnResize),
            Sort(crate::ZsTableSort),
        }
        let sorts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = sorts.clone();
        let column = crate::ZsTableColumnId::new(2);
        let builder = native_window("Table").size(500, 300).stateful_view(
            vec![
                crate::ZsTableColumn::new(1, "Name").fixed_width(Dp::new(180.0)),
                crate::ZsTableColumn::new(column, "Size")
                    .fixed_width(Dp::new(140.0))
                    .sortable(true),
            ],
            |columns| {
                crate::data_grid(columns.clone(), [crate::ZsTableRow::new(1, ["A", "12"])])
                    .id(crate::WidgetId::new(702))
                    .on_table_column_resize(Msg::Resize)
                    .on_table_sort(Msg::Sort)
            },
            move |columns, message, _cx| match message {
                Msg::Resize(change) => {
                    columns
                        .iter_mut()
                        .find(|column| column.id() == change.column)
                        .unwrap()
                        .resize_to(change.width);
                }
                Msg::Sort(_sort) => {
                    observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            },
        );
        let mut runtime = builder.native_view_input_runtime();
        let target = handle(&runtime, crate::ZsResizeTarget::TableColumn(column));
        assert_eq!(extent(target), 140);
        let start = center(target.bounds);
        let end = Point {
            x: start.x - 32,
            y: start.y,
        };
        runtime.dispatch_pointer_down(start, false);
        runtime.dispatch_pointer_move(end);
        runtime.dispatch_pointer_up(end);
        assert_eq!(
            extent(handle(&runtime, crate::ZsResizeTarget::TableColumn(column))),
            108
        );
        assert_eq!(sorts.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}

impl NativeViewInputRuntime {
    #[cfg(feature = "accessibility")]
    pub(super) fn resize_handle_for_widget(
        &self,
        widget: crate::WidgetId,
    ) -> Option<crate::ZsResizeHandle> {
        self.current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(widget))
            .and_then(|target| {
                if let crate::ViewHitTargetKind::ResizeHandle(handle) = target.kind {
                    Some(handle)
                } else {
                    None
                }
            })
    }
    pub(crate) fn resize_cursor_at(&self, point: Point) -> Option<crate::ZsResizeAxis> {
        if let Some(drag) = self.resize_drag {
            return Some(drag.handle.axis());
        }
        self.current_interaction_plan()
            .and_then(|plan| plan.hit_target_at(point))
            .and_then(|target| {
                if let crate::ViewHitTargetKind::ResizeHandle(handle) = target.kind {
                    Some(handle.axis())
                } else {
                    None
                }
            })
    }

    pub(super) fn begin_resize_drag(
        &mut self,
        target: crate::ViewHitTarget,
        point: Point,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let crate::ViewHitTargetKind::ResizeHandle(handle) = target.kind else {
            return report;
        };
        if !handle.is_adjustable() {
            return report;
        }
        self.text_drag = None;
        self.focus_target(target, &mut report);
        self.resize_drag = Some(NativeResizeDrag {
            handle,
            start: point,
            dpi: self.dpi,
        });
        report.handled = true;
        report.resize_drag_active = true;
        report
    }

    pub(super) fn move_resize_drag(
        &mut self,
        point: Point,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let Some(drag) = self.resize_drag else {
            return report;
        };
        report.handled = true;
        report.hit_target_count = self.hit_target_count();
        report.focused_widget = self.focused_widget.map(|widget| widget.0);
        let current = self
            .current_interaction_plan()
            .and_then(|plan| plan.hit_target_for_widget(drag.handle.widget_id()));
        let current = current.and_then(|target| match target.kind {
            crate::ViewHitTargetKind::ResizeHandle(handle)
                if handle.target() == drag.handle.target()
                    && handle.axis() == drag.handle.axis()
                    && handle.reversed == drag.handle.reversed =>
            {
                Some(handle)
            }
            _ => None,
        });
        let Some(current) = current.filter(|_| self.dpi == drag.dpi) else {
            self.resize_drag = None;
            return report;
        };
        report.resize_drag_active = true;
        let captured = crate::ZsResizeHandle {
            extent_px: drag.handle.extent_px,
            ..current
        };
        let extent = captured.extent_at(drag.start, point, self.dpi);
        if extent.to_px(self.dpi).round_i32() == current.extent_px {
            return report;
        }
        report.resize_extent_changed = true;
        self.dispatch_view_event(
            ViewEvent::Resized {
                widget: current.owner(),
                target: current.target(),
                extent,
            },
            report,
        )
    }

    pub(super) fn resize_key(
        &mut self,
        target: crate::ViewHitTarget,
        key: NativeViewKey,
        shift: bool,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let crate::ViewHitTargetKind::ResizeHandle(handle) = target.kind else {
            return report;
        };
        let amount = if shift { 32.0 } else { 8.0 };
        let direction = match (handle.axis(), key) {
            (crate::ZsResizeAxis::Horizontal, NativeViewKey::Left)
            | (crate::ZsResizeAxis::Vertical, NativeViewKey::Up) => Some(-1.0),
            (crate::ZsResizeAxis::Horizontal, NativeViewKey::Right)
            | (crate::ZsResizeAxis::Vertical, NativeViewKey::Down) => Some(1.0),
            _ => None,
        };
        let extent = match key {
            NativeViewKey::Home => Dp::new(handle.minimum_px as f32 / self.dpi.scale_factor()),
            NativeViewKey::End => Dp::new(handle.maximum_px as f32 / self.dpi.scale_factor()),
            _ => {
                let Some(direction) = direction else {
                    return report;
                };
                handle.step(
                    Dp::new(amount * direction * if handle.reversed { -1.0 } else { 1.0 }),
                    self.dpi,
                )
            }
        };
        report.handled = true;
        report.resize_extent_changed = extent.to_px(self.dpi).round_i32() != handle.extent_px;
        if !report.resize_extent_changed {
            return report;
        }
        self.dispatch_view_event(
            ViewEvent::Resized {
                widget: handle.owner(),
                target: handle.target(),
                extent,
            },
            report,
        )
    }

    pub(super) fn cancel_resize_with_restore(
        &mut self,
        mut report: NativeViewInputDispatchReport,
    ) -> NativeViewInputDispatchReport {
        let Some(drag) = self.resize_drag.take() else {
            return report;
        };
        report.handled = true;
        let extent = Dp::new(drag.handle.extent_px as f32 / drag.dpi.scale_factor());
        self.dispatch_view_event(
            ViewEvent::Resized {
                widget: drag.handle.owner(),
                target: drag.handle.target(),
                extent,
            },
            report,
        )
    }
}
