#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "label")]
    #[test]
    fn styled_text_keeps_semantic_role_and_matching_line_box() {
        let node: ViewNode<()> = styled_text(
            "Section heading",
            SemanticTextStyle::for_role(crate::TextRole::Subtitle),
        );
        assert_eq!(
            node.style.height,
            Some(Dp::new(
                crate::TextRole::Subtitle
                    .metrics_for(crate::ZsTypographyPlatformStyle::current())
                    .line_height
            ))
        );
        let ViewNodeKind::Text { style, .. } = node.kind else {
            panic!("styled_text should create a text node");
        };
        assert_eq!(style.role, crate::TextRole::Subtitle);
        assert_eq!(style.weight, crate::TextWeight::Automatic);
    }

    #[cfg(any(
        feature = "button",
        feature = "toggle-button",
        feature = "textbox",
        feature = "password-box",
        feature = "checkbox",
        feature = "toggle",
        feature = "slider",
        feature = "number-box",
        feature = "radio",
        feature = "breadcrumb",
        feature = "combo",
        feature = "date-picker",
        feature = "dialog",
        feature = "command-palette",
        feature = "info-bar",
        feature = "teaching-tip",
        feature = "toast",
        feature = "time-picker",
        feature = "tabs",
        feature = "list",
        feature = "grid-view",
        feature = "table",
        feature = "tree"
    ))]
    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        #[cfg(feature = "button")]
        SaveClicked,
        #[cfg(feature = "textbox")]
        NameChanged(String),
        #[cfg(feature = "textbox")]
        TextSelectionChanged(ZsTextSelection),
        #[cfg(feature = "password-box")]
        PasswordChanged(crate::ZsPassword),
        #[cfg(any(feature = "checkbox", feature = "toggle", feature = "toggle-button"))]
        DarkModeChanged(bool),
        #[cfg(feature = "slider")]
        VolumeChanged(f32),
        #[cfg(feature = "number-box")]
        NumberChanged(Option<f64>),
        #[cfg(feature = "radio")]
        ChoiceSelected(&'static str),
        #[cfg(feature = "combo")]
        ComboSelected(usize),
        #[cfg(feature = "combo")]
        ComboExpanded(bool),
        #[cfg(feature = "date-picker")]
        DateChanged(ZsDate),
        #[cfg(feature = "date-picker")]
        DateExpanded(bool),
        #[cfg(feature = "time-picker")]
        TimeChanged(ZsTime),
        #[cfg(feature = "time-picker")]
        TimeExpanded(bool),
        #[cfg(feature = "tabs")]
        TabSelected(ZsTabId),
        #[cfg(feature = "list")]
        RowSelected(usize),
        #[cfg(feature = "tree")]
        TreeSelected(crate::ZsTreeNodeId),
        #[cfg(feature = "tree")]
        TreeExpanded(crate::ZsTreeExpansionChange),
        #[cfg(feature = "tree")]
        TreeInvoked(crate::ZsTreeNodeId),
        #[cfg(feature = "grid-view")]
        GridViewSelected(crate::ZsGridViewItemId),
        #[cfg(feature = "grid-view")]
        GridViewInvoked(crate::ZsGridViewItemId),
        #[cfg(feature = "table")]
        TableSelected(crate::ZsTableRowId),
        #[cfg(feature = "table")]
        TableSorted(crate::ZsTableSort),
        #[cfg(feature = "table")]
        TableInvoked(crate::ZsTableRowId),
        #[cfg(feature = "dialog")]
        DialogResult(crate::ZsContentDialogResult),
        #[cfg(feature = "command-palette")]
        CommandQuery(String),
        #[cfg(feature = "command-palette")]
        CommandHighlight(crate::ZsCommandPaletteItemId),
        #[cfg(feature = "command-palette")]
        CommandInvoke(crate::ZsCommandPaletteItemId),
        #[cfg(feature = "command-palette")]
        CommandOpen(bool),
        #[cfg(feature = "toast")]
        ToastResult(crate::ZsToastResult),
        #[cfg(feature = "teaching-tip")]
        TeachingTipResult(crate::ZsTeachingTipResult),
        #[cfg(feature = "info-bar")]
        InfoBarEvent(crate::ZsInfoBarEvent),
        #[cfg(feature = "breadcrumb")]
        BreadcrumbSelected(crate::ZsBreadcrumbId),
        #[cfg(feature = "breadcrumb")]
        BreadcrumbExpanded(bool),
        #[cfg(feature = "scroll")]
        ScrollChanged(Dp),
        #[cfg(feature = "virtual-list")]
        ViewportChanged(VirtualListViewport),
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_uses_typed_messages_without_string_events() {
        let save_id = WidgetId::new(1);
        let mut view = column(vec![
            text("Clipboard history"),
            button("Save")
                .id(save_id)
                .padding(Dp::new(12.0))
                .radius(Dp::new(8.0))
                .on_click(Msg::SaveClicked),
        ]);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: save_id });

        assert_eq!(events.into_messages(), vec![Msg::SaveClicked]);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_layout_and_paint_emit_native_draw_plan() {
        let mut view: ViewNode<Msg> =
            column(vec![text("Title"), button("Copy").radius(Dp::new(8.0))]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert_eq!(output.bounds.width, 240);
        assert_eq!(paint.plan().text_count(), 2);
        assert!(paint.plan().command_count() >= 3);
    }

    #[test]
    #[cfg(feature = "button")]
    fn toolbar_button_keeps_semantic_icon_and_flat_resting_chrome() {
        let mut view: ViewNode<Msg> = toolbar_button_for_style(
            crate::ZsBaseControlPlatformStyle::Windows,
            "Save",
            crate::ZsIcon::Save,
        )
        .on_click(Msg::SaveClicked);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(command)
                if command.icon == crate::ZsIcon::Save && command.bounds.width == 20
        )));
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command)
                if command.text == "Save" && command.style.role == crate::TextRole::Body
        )));
        assert!(!paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::RoundRect { .. })));
    }

    #[test]
    #[cfg(all(windows, feature = "button"))]
    fn navigation_item_keeps_button_activation_with_navigation_chrome() {
        let item_id = WidgetId::new(19);
        let mut view: ViewNode<Msg> =
            navigation_item("Navigation", crate::ZsIcon::Sidebar, true)
            .id(item_id)
            .width(Dp::new(184.0))
            .on_click(Msg::SaveClicked);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 184,
                height: 36,
            },
            Dpi::standard(),
        );
        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(&mut events, &ViewEvent::Click { widget: item_id });

        assert_eq!(output.bounds, Rect { x: 0, y: 0, width: 184, height: 36 });
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundFill {
                rect: Rect { width: 3, height: 16, .. },
                ..
            }
        )));
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::Sidebar
        )));
        assert_eq!(events.into_messages(), vec![Msg::SaveClicked]);
    }

    #[test]
    #[cfg(all(windows, feature = "button", feature = "label"))]
    fn windows_button_uses_winui_metrics_and_does_not_stretch_by_default() {
        let button_id = WidgetId::new(2);
        let mut view: ViewNode<Msg> = column(vec![
            text("Title"),
            button("Copy")
                .id(button_id)
                .on_click(Msg::SaveClicked),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let button_bounds = output
            .children
            .iter()
            .find(|node| node.component == button_id.into())
            .expect("button should expose layout bounds")
            .bounds;
        assert_eq!(
            button_bounds,
            Rect {
                x: 0,
                y: 20,
                width: 120,
                height: 32,
            }
        );

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect {
                rect,
                stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
                radius: 4,
                ..
            } if *rect == button_bounds
        )));
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text)
                if text.text == "Copy"
                    && text.bounds == Rect {
                        x: 11,
                        y: 25,
                        width: 98,
                        height: 21,
                    }
                    && text.style.horizontal_align == crate::HorizontalAlign::Center
        )));
    }

    #[test]
    fn stack_layout_honors_fixed_size_flex_and_gap() {
        let navigation = WidgetId::new(70);
        let content = WidgetId::new(71);
        let mut view: ViewNode<()> = row(vec![
            spacer().id(navigation).width(Dp::new(240.0)),
            spacer().id(content).flex(1.0),
        ])
        .gap(Dp::new(12.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 960,
                height: 640,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let navigation_bounds = output
            .children
            .iter()
            .find(|node| node.component == navigation.into())
            .unwrap()
            .bounds;
        let content_bounds = output
            .children
            .iter()
            .find(|node| node.component == content.into())
            .unwrap()
            .bounds;

        assert_eq!(navigation_bounds.width, 240);
        assert_eq!(content_bounds.x, 252);
        assert_eq!(content_bounds.width, 708);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn row_uses_intrinsic_child_height_inside_a_column() {
        let heading_id = WidgetId::new(710);
        let row_id = WidgetId::new(711);
        let button_id = WidgetId::new(712);
        let mut view: ViewNode<()> = column([
            text("Heading").id(heading_id),
            row([text("Action"), button("Save").id(button_id)])
                .id(row_id)
                .gap(Dp::new(8.0)),
            text("Status"),
        ])
        .gap(Dp::new(12.0));

        let output = view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 480,
                height: 320,
            },
            Dpi::standard(),
        ));
        let bounds_for = |widget: WidgetId| {
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("nested row child should expose layout bounds")
                .bounds
        };

        let metrics = crate::ZsBaseControlMetrics::for_platform(
            crate::ZsBaseControlPlatformStyle::current(),
        );
        let button_height = metrics
            .button_height
            .to_px(Dpi::standard())
            .round_i32();
        // A platform may add a small baseline/intrinsic row allowance around
        // a compact button (AppKit currently resolves to 32dp for this mixed
        // text/button row while the button itself remains 28dp). The row and
        // button must agree, and neither may undersize the platform metric.
        assert_eq!(bounds_for(row_id).height, bounds_for(button_id).height);
        assert!(bounds_for(button_id).height >= button_height);
        // The shared stack contract guarantees that the row starts after the
        // heading. Native text backends may retain a small baseline allowance
        // around the requested gap, so do not require one exact y coordinate.
        let heading = bounds_for(heading_id);
        let row = bounds_for(row_id);
        assert!(row.y >= heading.y + heading.height);
    }

    #[test]
    #[cfg(feature = "grid")]
    fn grid_layout_honors_typed_tracks_spans_and_independent_gaps() {
        let header = WidgetId::new(72);
        let content = WidgetId::new(73);
        let action = WidgetId::new(74);
        let mut view: ViewNode<()> = grid(
            [
                ZsGridTrack::fixed(Dp::new(120.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fraction(ZsGridFraction::TWO),
            ],
            [
                ZsGridTrack::fixed(Dp::new(40.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fixed(Dp::new(60.0)),
            ],
            [
                ZsGridCell::new(0, 0, spacer().id(header)).column_span(ZsGridSpan::THREE),
                ZsGridCell::new(1, 1, spacer().id(content)).column_span(ZsGridSpan::TWO),
                ZsGridCell::new(2, 2, spacer().id(action)),
            ],
        )
        .column_gap(Dp::new(10.0))
        .row_gap(Dp::new(8.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 600,
                height: 300,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let bounds_for = |widget: WidgetId| {
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("grid child should be laid out")
                .bounds
        };

        assert_eq!(
            bounds_for(header),
            Rect {
                x: 0,
                y: 0,
                width: 600,
                height: 40,
            }
        );
        assert_eq!(
            bounds_for(content),
            Rect {
                x: 130,
                y: 48,
                width: 470,
                height: 184,
            }
        );
        assert_eq!(
            bounds_for(action),
            Rect {
                x: 293,
                y: 240,
                width: 307,
                height: 60,
            }
        );
    }

    #[test]
    #[cfg(feature = "grid")]
    fn grid_explicit_placement_scales_fixed_tracks_with_dpi_and_bounds_invalid_cells() {
        let first = WidgetId::new(75);
        let second = WidgetId::new(76);
        let explicit = WidgetId::new(77);
        let invalid = WidgetId::new(79);
        let mut view: ViewNode<()> = grid(
            [ZsGridTrack::fixed(Dp::new(40.0)), ZsGridTrack::FLEX],
            [ZsGridTrack::FLEX, ZsGridTrack::FLEX],
            [
                ZsGridCell::new(1, 0, spacer().id(first)),
                ZsGridCell::new(1, 1, spacer().id(second)),
                ZsGridCell::new(0, 0, spacer().id(explicit)).column_span(ZsGridSpan::TWO),
                ZsGridCell::new(4, 4, spacer().id(invalid)),
            ],
        )
        .gap(Dp::new(8.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 180,
            },
            Dpi::new(144.0),
        );

        let output = view.layout(&mut layout);
        let bounds_for = |widget: WidgetId| {
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("grid child should be laid out")
                .bounds
        };

        assert_eq!(
            bounds_for(explicit),
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 84,
            }
        );
        assert_eq!(
            bounds_for(first),
            Rect {
                x: 10,
                y: 116,
                width: 60,
                height: 84,
            }
        );
        assert_eq!(bounds_for(second).width, 228);
        assert_eq!(bounds_for(second).x, 82);
        assert_eq!(bounds_for(invalid).width, 0);
        assert_eq!(bounds_for(invalid).height, 0);
        assert!(ZsGridFraction::new(0).is_err());
        assert!(ZsGridSpan::new(0).is_err());
    }

    #[test]
    #[cfg(all(feature = "grid", feature = "button"))]
    fn grid_layout_drives_shared_paint_and_typed_hit_geometry() {
        let behind = WidgetId::new(80);
        let action = WidgetId::new(78);
        let mut view: ViewNode<Msg> = grid(
            [
                ZsGridTrack::FLEX,
                ZsGridTrack::fraction(ZsGridFraction::TWO),
            ],
            [ZsGridTrack::FLEX, ZsGridTrack::FLEX],
            [
                ZsGridCell::new(0, 0, spacer()),
                ZsGridCell::new(1, 1, button("Behind").id(behind).on_click(Msg::SaveClicked)),
                ZsGridCell::new(1, 1, button("Apply").id(action).on_click(Msg::SaveClicked)),
            ],
        )
        .gap(Dp::new(6.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 306,
                height: 206,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let interaction = view.interaction_plan();
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert_eq!(
            interaction.click_event_at(Point { x: 205, y: 155 }),
            Some(ViewEvent::Click { widget: action })
        );
        assert_eq!(interaction.hit_target_count(), 2);
        assert!(paint.plan().text_count() >= 2);
    }

    #[test]
    fn square_background_uses_full_rect_fill() {
        let mut view: ViewNode<()> = spacer().bg(ThemeColorToken::Surface);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 80,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert!(matches!(
            paint.plan().commands.first(),
            Some(NativeDrawCommand::FillRect { .. })
        ));
    }

    #[test]
    #[cfg(all(feature = "tooltip", feature = "button"))]
    fn tooltip_attachment_adds_overlay_metadata_without_an_extra_hit_target() {
        let widget = WidgetId::new(501);
        let mut view: ViewNode<()> = button("Save")
            .id(widget)
            .tooltip_spec(crate::ZsTooltipSpec::new("Save document"));
        let surface = Rect {
            x: 0,
            y: 0,
            width: 240,
            height: 120,
        };
        view.layout(&mut ViewLayoutCx::new(surface, Dpi::standard()));

        let interaction = view.interaction_plan();

        assert_eq!(interaction.hit_target_count(), 1);
        assert_eq!(interaction.tooltip_targets.len(), 1);
        assert_eq!(interaction.tooltip_targets[0].widget, widget);
        assert_eq!(interaction.tooltip_targets[0].spec.text, "Save document");
        assert_eq!(interaction.tooltip_targets[0].bounds, surface);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_interaction_plan_maps_points_to_typed_click_events() {
        let save_id = WidgetId::new(42);
        let mut view: ViewNode<Msg> = column(vec![
            text("Title"),
            button("Save").id(save_id).on_click(Msg::SaveClicked),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 120,
            },
            Dpi::standard(),
        );
        let _output = view.layout(&mut layout);
        let plan = view.interaction_plan();

        assert_eq!(plan.hit_target_count(), 1);
        assert_eq!(
            plan.target_kind_at(Point { x: 60, y: 36 }),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.hit_target_for_widget(save_id)
                .map(|target| target.kind),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.click_event_at(Point { x: 60, y: 36 }),
            Some(ViewEvent::Click { widget: save_id })
        );
        assert_eq!(
            plan.first_focus_target().map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(
            plan.next_focus_target(None, 1).map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(plan.click_event_at(Point { x: 150, y: 20 }), None);
    }

    #[test]
    #[cfg(all(feature = "textbox", feature = "checkbox"))]
    fn input_views_map_runtime_values_into_typed_messages() {
        let name_id = WidgetId::new(2);
        let dark_id = WidgetId::new(3);
        let mut view = column(vec![
            textbox("").id(name_id).on_change(Msg::NameChanged),
            checkbox("Dark mode", false)
                .id(dark_id)
                .on_toggle(Msg::DarkModeChanged),
        ]);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: name_id,
                value: "ZSUI".to_string(),
            },
        );
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: dark_id,
                checked: true,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::NameChanged("ZSUI".to_string()),
                Msg::DarkModeChanged(true)
            ]
        );
    }

    #[test]
    #[cfg(all(feature = "textbox", not(feature = "checkbox")))]
    fn textbox_maps_runtime_value_without_other_input_features() {
        let name_id = WidgetId::new(2);
        let mut view = textbox("").id(name_id).on_change(Msg::NameChanged);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: name_id,
                value: "ZSUI".to_string(),
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![Msg::NameChanged("ZSUI".to_string())]
        );
    }

    #[test]
    #[cfg(feature = "password-box")]
    fn password_box_routes_redacted_value_and_exposes_a_separate_reveal_target() {
        let widget = WidgetId::new(3);
        let secret = "vault🙂";
        let next_secret = "next中";
        let mut view = password_box(secret)
            .id(widget)
            .height(Dp::new(36.0))
            .reveal_mode(crate::ZsPasswordRevealMode::Peek)
            .on_password_change(Msg::PasswordChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 220,
                height: 36,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let interaction = view.interaction_plan();

        assert_eq!(
            view.widget_password_value(widget)
                .map(crate::ZsPassword::as_str),
            Some(secret)
        );
        assert!(interaction.hit_targets.iter().any(|target| {
            target.widget == widget && target.kind == ViewHitTargetKind::PasswordBox
        }));
        assert!(interaction.hit_targets.iter().any(|target| {
            target.widget == widget && target.kind == ViewHitTargetKind::PasswordBoxReveal
        }));
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "••••••"
        )));
        assert!(!format!("{:?}", paint.plan()).contains(secret));
        assert!(!serde_json::to_string(paint.plan())
            .expect("password draw plan should serialize redacted")
            .contains(secret));

        let event = ViewEvent::PasswordChanged {
            widget,
            value: crate::ZsPassword::from(next_secret),
        };
        assert!(!format!("{event:?}").contains(next_secret));
        assert!(!serde_json::to_string(&event)
            .expect("password event should serialize redacted")
            .contains(next_secret));
        let mut events = ViewEventCx::new();
        view.event(&mut events, &event);

        assert_eq!(
            events.into_messages(),
            vec![Msg::PasswordChanged(crate::ZsPassword::from(next_secret))]
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn text_editor_is_a_multiline_focus_target_with_wrapped_text() {
        let editor_id = WidgetId::new(5);
        let mut view = text_editor::<Msg>("first\nsecond")
            .id(editor_id)
            .on_change(Msg::NameChanged)
            .on_text_selection_change(Msg::TextSelectionChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 180,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::TextEditor);
        assert_eq!(
            view.widget_text_wrap(editor_id),
            Some(crate::TextWrap::Word)
        );
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text)
                if text.style.wrap == crate::TextWrap::Word
                    && text.style.vertical_align == crate::VerticalAlign::Start
                    && !text.style.ellipsis
        )));

        let selection = ZsTextSelection {
            anchor: 2,
            caret: 7,
        };
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::TextEdited {
                widget: editor_id,
                value: "first\nchanged".to_string(),
                selection,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::NameChanged("first\nchanged".to_string()),
                Msg::TextSelectionChanged(selection),
            ]
        );
        assert_eq!(selection.ordered(), (2, 7));
        assert!(!selection.is_collapsed());

        let mut no_wrap = text_editor::<Msg>("first\nsecond")
            .id(editor_id)
            .text_wrap(crate::TextWrap::NoWrap);
        no_wrap.layout(&mut layout);
        let mut no_wrap_paint = ViewPaintCx::new(Dpi::standard());
        no_wrap.paint(&mut no_wrap_paint);
        let lines = no_wrap_paint
            .plan()
            .commands
            .iter()
            .filter_map(|command| match command {
                NativeDrawCommand::Text(text) => Some((text.text.as_str(), text.style.wrap)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            lines,
            vec![
                ("first", crate::TextWrap::NoWrap),
                ("second", crate::TextWrap::NoWrap),
            ]
        );
        assert_eq!(
            no_wrap.widget_text_wrap(editor_id),
            Some(crate::TextWrap::NoWrap)
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn app_context_queues_focused_and_strongly_targeted_text_edit_commands() {
        let widget = WidgetId::new(6);
        let mut cx = AppCx::new();

        cx.text_edit_command(ZsTextEditCommand::Copy);
        cx.text_edit_command_for(widget, ZsTextEditCommand::Undo);

        assert_eq!(
            cx.text_edit_commands(),
            [
                ZsTextEditCommandRequest::focused(ZsTextEditCommand::Copy),
                ZsTextEditCommandRequest::for_widget(widget, ZsTextEditCommand::Undo),
            ]
        );
    }

    #[test]
    #[cfg(feature = "toggle")]
    fn toggle_routes_typed_state_and_paints_shared_geometry() {
        let toggle_id = WidgetId::new(4);
        let mut view = toggle(false).id(toggle_id).on_toggle(Msg::DarkModeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: toggle_id,
                checked: true,
            },
        );

        assert_eq!(view.widget_checked_value(toggle_id), Some(true));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::Toggle);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(events.into_messages(), vec![Msg::DarkModeChanged(true)]);
    }

    #[test]
    #[cfg(feature = "toggle-button")]
    fn toggle_button_routes_explicit_state_and_paints_platform_profile() {
        let toggle_id = WidgetId::new(41);
        let mut view = toggle_button("Pin", false)
            .id(toggle_id)
            .on_toggle(Msg::DarkModeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 36,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: toggle_id,
                checked: true,
            },
        );

        assert_eq!(view.widget_checked_value(toggle_id), Some(true));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::ToggleButton);
        assert_eq!(view.interaction_plan().hit_target_count(), 1);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(events.into_messages(), vec![Msg::DarkModeChanged(true)]);
    }

    #[test]
    #[cfg(feature = "slider")]
    fn slider_clamps_snaps_routes_typed_value_and_paints_shared_geometry() {
        let slider_id = WidgetId::new(6);
        let range = SliderRange::new(0.0, 10.0).step(0.5);
        let mut view = slider(12.0, range)
            .id(slider_id)
            .on_slide(Msg::VolumeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::SliderChanged {
                widget: slider_id,
                value: 4.74,
            },
        );

        assert_eq!(view.widget_slider_state(slider_id), Some((4.5, range)));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::Slider);
        assert_eq!(paint.plan().command_count(), 3);
        assert_eq!(events.into_messages(), vec![Msg::VolumeChanged(4.5)]);
        assert_eq!(range.value_at_fraction(0.26), 2.5);
        assert_eq!(range.offset_steps(4.5, 1), 5.0);

        let uneven = SliderRange::new(0.0, 1.0).step(0.3);
        assert_eq!(uneven.value_at_fraction(1.0), 1.0);
        assert_eq!(uneven.offset_steps(0.9, 1), 1.0);
    }

    #[test]
    #[cfg(feature = "number-box")]
    fn number_box_preserves_invalid_draft_and_routes_typed_steps() {
        let number_id = WidgetId::new(61);
        let range = ZsNumberRange::new(-10.0, 10.0).step(0.5).large_step(5.0);
        let mut view = number_box(Some(2.5), range)
            .id(number_id)
            .height(Dp::new(36.0))
            .fraction_digits(1)
            .on_number_change(Msg::NumberChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 36,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: number_id,
                value: "-".to_string(),
            },
        );
        assert_eq!(
            view.widget_number_box_state(number_id),
            Some(ZsNumberBoxState {
                value: Some(2.5),
                draft: "-".to_string(),
                valid: false,
            })
        );

        view.event(
            &mut events,
            &ViewEvent::NumberBoxStep {
                widget: number_id,
                steps: 1,
                large: false,
            },
        );
        assert_eq!(
            view.widget_number_box_state(number_id),
            Some(ZsNumberBoxState {
                value: Some(3.0),
                draft: "3.0".to_string(),
                valid: true,
            })
        );
        assert_eq!(events.into_messages(), vec![Msg::NumberChanged(Some(3.0))]);
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::NumberBox);
        assert_eq!(view.interaction_plan().hit_target_count(), 3);

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert_eq!(paint.plan().command_count(), 6);
    }

    #[test]
    #[cfg(feature = "radio")]
    fn radio_button_routes_typed_choice_and_paints_selected_state() {
        let radio_id = WidgetId::new(7);
        let mut view = radio_button("Balanced", false)
            .id(radio_id)
            .on_choose(Msg::ChoiceSelected("balanced"));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();
        view.event(&mut events, &ViewEvent::RadioSelected { widget: radio_id });
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::RadioButton);
        assert_eq!(paint.plan().command_count(), 3);
        assert_eq!(
            events.into_messages(),
            vec![Msg::ChoiceSelected("balanced")]
        );
        assert!(matches!(
            view.kind,
            ViewNodeKind::RadioButton { selected: true, .. }
        ));
    }

    #[test]
    #[cfg(feature = "radio")]
    fn radio_groups_enforce_single_selection_and_non_wrapping_directional_navigation() {
        let first = WidgetId::new(71);
        let second = WidgetId::new(72);
        let third = WidgetId::new(73);
        let mut vertical = column([
            radio_button("Balanced", true)
                .id(first)
                .on_choose(Msg::ChoiceSelected("balanced")),
            radio_button("Performance", false)
                .id(second)
                .on_choose(Msg::ChoiceSelected("performance")),
            radio_button("Quiet", false)
                .id(third)
                .on_choose(Msg::ChoiceSelected("quiet")),
        ]);

        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Column, -1),
            Some(first)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Column, 1),
            Some(second)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Row, 1),
            Some(first)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(third, ViewStackDirection::Column, 1),
            Some(third)
        );
        assert_eq!(vertical.widget_radio_is_tab_stop(first), Some(true));
        assert_eq!(vertical.widget_radio_is_tab_stop(second), Some(false));

        let mut events = ViewEventCx::new();
        vertical.event(&mut events, &ViewEvent::RadioSelected { widget: second });
        assert_eq!(vertical.widget_checked_value(first), Some(false));
        assert_eq!(vertical.widget_checked_value(second), Some(true));
        assert_eq!(vertical.widget_checked_value(third), Some(false));
        assert_eq!(vertical.widget_radio_is_tab_stop(first), Some(false));
        assert_eq!(vertical.widget_radio_is_tab_stop(second), Some(true));
        assert_eq!(
            events.into_messages(),
            vec![Msg::ChoiceSelected("performance")]
        );

        let horizontal = row([
            radio_button::<()>("One", true).id(first),
            radio_button::<()>("Two", false).id(second),
            radio_button::<()>("Three", false).id(third),
        ]);
        assert_eq!(
            horizontal.widget_radio_relative_widget(second, ViewStackDirection::Row, -1),
            Some(first)
        );
        assert_eq!(
            horizontal.widget_radio_relative_widget(second, ViewStackDirection::Column, 1),
            Some(third)
        );
    }

    #[test]
    #[cfg(feature = "progress")]
    fn progress_bar_normalizes_range_clamps_state_and_paints_fraction() {
        let range = crate::ProgressRange::new(100.0, 0.0);
        let mut view = progress_bar::<()>(125.0, range).id(WidgetId::new(8));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(range.min(), 0.0);
        assert_eq!(range.max(), 100.0);
        assert_eq!(range.fraction(25.0), 0.25);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(view.interaction_plan().hit_target_count(), 0);
        assert!(matches!(
            view.kind,
            ViewNodeKind::ProgressBar { value: 100.0, .. }
        ));
    }

    #[test]
    #[cfg(feature = "progress-ring")]
    fn progress_ring_animates_without_becoming_a_hit_target() {
        let spec = crate::ZsProgressRingSpec::indeterminate();
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        };
        let mut view = progress_ring::<()>(spec).id(WidgetId::new(81));
        view.layout(&mut ViewLayoutCx::new(
            bounds,
            Dpi::standard(),
        ));
        let mut first =
            ViewPaintCx::with_animation_elapsed(Dpi::standard(), std::time::Duration::ZERO);
        view.paint(&mut first);
        let mut half = ViewPaintCx::with_animation_elapsed(
            Dpi::standard(),
            std::time::Duration::from_millis(500),
        );
        view.paint(&mut half);

        assert_eq!(view.background_poll_interval_ms(), Some(16));
        assert_eq!(view.interaction_plan().hit_target_count(), 0);
        let platform = crate::ZsProgressRingPlatformStyle::current();
        let expected_first = crate::zs_progress_ring_native_draw_plan(
            &crate::zs_progress_ring_render_plan(spec, bounds, platform, Dpi::standard(), 0),
        );
        let expected_half = crate::zs_progress_ring_native_draw_plan(
            &crate::zs_progress_ring_render_plan(spec, bounds, platform, Dpi::standard(), 500),
        );
        assert_eq!(first.plan(), &expected_first);
        assert_eq!(half.plan(), &expected_half);
    }

    #[test]
    #[cfg(feature = "auto-suggest")]
    fn auto_suggest_routes_strong_id_overlay_keyboard_state_and_submission() {
        let widget = WidgetId::new(91);
        let chosen = crate::ZsAutoSuggestionId::new(102);
        let mut view: ViewNode<()> = auto_suggest_box(
            "Ch",
            [
                crate::ZsAutoSuggestion::new(101_u64, "Chicago"),
                crate::ZsAutoSuggestion::new(chosen, "China"),
                crate::ZsAutoSuggestion::new(103_u64, "Chile"),
            ],
        )
        .id(widget)
        .placeholder("Search")
        .expanded(true)
        .highlighted_suggestion(Some(chosen));
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 20,
                y: 20,
                width: 280,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert!(interaction.hit_targets.iter().any(|target| {
            target.widget == widget
                && target.kind == ViewHitTargetKind::AutoSuggestSuggestion { suggestion: chosen }
        }));
        assert_eq!(
            view.widget_auto_suggest_state(widget),
            Some(crate::ZsAutoSuggestState {
                query: "Ch".into(),
                suggestion_ids: vec![101_u64.into(), chosen, 103_u64.into()],
                highlighted: Some(chosen),
                expanded: true,
            })
        );

        let mut event_cx = ViewEventCx::new();
        view.event(
            &mut event_cx,
            &ViewEvent::AutoSuggestSubmitted {
                widget,
                suggestion: Some(chosen),
            },
        );
        assert_eq!(event_cx.into_messages(), Vec::<()>::new());
        let state = view
            .widget_auto_suggest_state(widget)
            .expect("auto-suggest state");
        assert_eq!(state.query, "China");
        assert!(!state.expanded);
        assert_eq!(state.highlighted, None);
    }

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn auto_suggest_user_text_and_clear_keep_explicit_popup_state() {
        let widget = WidgetId::new(92);
        let mut view: ViewNode<()> =
            auto_suggest_box("", [crate::ZsAutoSuggestion::new(1_u64, "Alpha")])
                .id(widget)
                .no_results_text("No results");
        let mut cx = ViewEventCx::new();
        view.event(
            &mut cx,
            &ViewEvent::TextChanged {
                widget,
                value: "a".into(),
            },
        );
        assert_eq!(view.widget_text_value(widget), Some("a"));
        assert!(view
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.expanded));

        view.event(&mut cx, &ViewEvent::AutoSuggestCleared { widget });
        assert_eq!(view.widget_text_value(widget), Some(""));
        assert!(view
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| !state.expanded));
    }

    #[cfg(feature = "tree")]
    #[test]
    fn tree_view_routes_strong_id_expansion_selection_invocation_and_hit_geometry() {
        let widget = WidgetId::new(93);
        let root = crate::ZsTreeNodeId::new(1);
        let folder = crate::ZsTreeNodeId::new(2);
        let leaf = crate::ZsTreeNodeId::new(3);
        let mut view = tree_view([crate::ZsTreeNode::new(root, "Workspace")
            .icon(crate::ZsIcon::Folder)
            .children([
                crate::ZsTreeNode::new(folder, "src")
                    .icon(crate::ZsIcon::Folder)
                    .children([crate::ZsTreeNode::new(leaf, "lib.rs")]),
                crate::ZsTreeNode::new(4, "Cargo.toml"),
            ])])
        .id(widget)
        .expanded_tree_nodes([root])
        .selected_tree_node(Some(folder))
        .on_tree_select(Msg::TreeSelected)
        .on_tree_expansion_change(Msg::TreeExpanded)
        .on_tree_invoke(Msg::TreeInvoked);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 10,
                y: 20,
                width: 260,
                height: 160,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::TreeNodeExpander { node: folder } }));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::TreeNode { node: folder } }));

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::TreeNodeExpandedChanged {
                widget,
                node: folder,
                expanded: true,
            },
        );
        view.event(
            &mut events,
            &ViewEvent::TreeNodeSelected { widget, node: leaf },
        );
        view.event(
            &mut events,
            &ViewEvent::TreeNodeInvoked { widget, node: leaf },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::TreeExpanded(crate::ZsTreeExpansionChange::new(folder, true)),
                Msg::TreeSelected(leaf),
                Msg::TreeInvoked(leaf),
            ]
        );
        let state = view.widget_tree_view_state(widget).expect("tree state");
        assert_eq!(state.selected, Some(leaf));
        assert_eq!(
            state.rows.iter().map(|row| row.node).collect::<Vec<_>>(),
            vec![root, folder, leaf, 4_u64.into()]
        );
    }

    #[cfg(feature = "grid-view")]
    #[test]
    fn grid_view_has_one_tab_stop_and_routes_typed_item_events() {
        let widget = WidgetId::new(109);
        let first = crate::ZsGridViewItemId::new(1);
        let selected = crate::ZsGridViewItemId::new(2);
        let invoked = crate::ZsGridViewItemId::new(5);
        let items = [
            crate::ZsGridViewItem::new(first, "Desktop"),
            crate::ZsGridViewItem::new(selected, "Documents"),
            crate::ZsGridViewItem::new(3, "Photos"),
            crate::ZsGridViewItem::new(invoked, "src"),
            crate::ZsGridViewItem::new(selected, "Duplicate"),
        ];
        let mut view = grid_view(items.clone())
        .id(widget)
        .selected_grid_view_item(Some(selected))
        .on_grid_view_select(Msg::GridViewSelected)
        .on_grid_view_invoke(Msg::GridViewInvoked);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 10,
                y: 20,
                width: 420,
                height: 240,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| target.accepts_focus())
                .count(),
            1
        );
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| matches!(target.kind, ViewHitTargetKind::GridViewItem { .. }))
                .count(),
            4
        );

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::GridViewItemSelected {
                widget,
                item: invoked,
            },
        );
        view.event(
            &mut events,
            &ViewEvent::GridViewItemInvoked {
                widget,
                item: invoked,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::GridViewSelected(invoked),
                Msg::GridViewInvoked(invoked)
            ]
        );
        let expected_column_count = crate::zs_grid_view_render_plan(
            Rect {
                x: 10,
                y: 20,
                width: 420,
                height: 240,
            },
            &items,
            Some(invoked),
            crate::ZsGridViewPlatformStyle::current(),
            Dpi::standard(),
        )
        .column_count;
        assert_eq!(
            view.widget_grid_view_state(widget),
            Some(crate::ZsGridViewState {
                selected: Some(invoked),
                items: vec![first, selected, 3_u64.into(), invoked],
                column_count: expected_column_count,
            })
        );
    }

    #[cfg(feature = "table")]
    #[test]
    fn table_data_grid_routes_strong_id_selection_sort_invocation_and_hit_geometry() {
        let widget = WidgetId::new(94);
        let name = crate::ZsTableColumnId::new(1);
        let first = crate::ZsTableRowId::new(10);
        let second = crate::ZsTableRowId::new(11);
        let mut view = data_grid(
            [
                crate::ZsTableColumn::new(name, "Name").sortable(true),
                crate::ZsTableColumn::new(2, "Size")
                    .fixed_width(Dp::new(80.0))
                    .alignment(crate::HorizontalAlign::End),
            ],
            [
                crate::ZsTableRow::new(first, ["Cargo.toml", "4 KB"]),
                crate::ZsTableRow::new(second, ["src", "—"]),
            ],
        )
        .id(widget)
        .selected_table_row(Some(first))
        .on_table_select(Msg::TableSelected)
        .on_table_sort(Msg::TableSorted)
        .on_table_invoke(Msg::TableInvoked);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 160,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::TableHeader { column: name } }));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::TableRow { row: second } }));

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::TableSorted {
                widget,
                column: name,
            },
        );
        view.event(
            &mut events,
            &ViewEvent::TableRowSelected {
                widget,
                row: second,
            },
        );
        view.event(
            &mut events,
            &ViewEvent::TableRowInvoked {
                widget,
                row: second,
            },
        );

        let ascending = crate::ZsTableSort::new(name, crate::ZsTableSortDirection::Ascending);
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::TableSorted(ascending),
                Msg::TableSelected(second),
                Msg::TableInvoked(second),
            ]
        );
        assert_eq!(
            view.widget_table_state(widget),
            Some(crate::ZsTableViewState {
                selected: Some(second),
                sort: Some(ascending),
                rows: vec![first, second],
            })
        );
    }

    #[cfg(feature = "dialog")]
    #[test]
    fn content_dialog_is_modal_self_drawn_and_routes_one_typed_result() {
        let dialog = WidgetId::new(95);
        let background = WidgetId::new(96);
        let spec =
            crate::ZsContentDialogSpec::new("The unsaved changes will be discarded.", "Cancel")
                .title("Discard changes?")
                .primary_button("Discard")
                .secondary_button("Save")
                .default_button(crate::ZsContentDialogButton::Secondary)
                .destructive_button(crate::ZsContentDialogButton::Primary);
        let mut view = content_dialog(
            dialog,
            true,
            spec,
            spacer::<Msg>().id(background).bg(ThemeColorToken::Surface),
        )
        .on_dialog_result(Msg::DialogResult);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 400,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let interaction = view.interaction_plan();
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::ContentDialog)
        );
        assert!(interaction.focus_target_for_widget(background).is_none());
        assert_eq!(
            interaction
                .focus_target_for_widget(dialog)
                .map(|target| target.kind),
            Some(ViewHitTargetKind::ContentDialog)
        );
        assert_eq!(
            interaction.target_kind_at(Point { x: 4, y: 4 }),
            Some(ViewHitTargetKind::ContentDialogScrim)
        );
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| matches!(
                    target.kind,
                    ViewHitTargetKind::ContentDialogButton { .. }
                ))
                .count(),
            3
        );

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let scrim = paint
            .plan()
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    NativeDrawCommand::FillRect {
                        rect,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::PrimaryText,
                            ..
                        },
                    } if *rect == viewport
                )
            })
            .expect("dialog scrim should be drawn");
        let page = paint
            .plan()
            .commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    NativeDrawCommand::FillRect {
                        rect,
                        fill: NativeDrawFill::Role(ColorRole::Surface),
                    } if *rect == viewport
                )
            })
            .expect("page should be drawn beneath the dialog");
        assert!(scrim > page);

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ContentDialogFocused {
                widget: dialog,
                button: crate::ZsContentDialogButton::Primary,
            },
        );
        assert_eq!(
            view.widget_content_dialog_state(dialog)
                .map(|(state, _)| state.focused_button),
            Some(crate::ZsContentDialogButton::Primary)
        );
        view.event(
            &mut events,
            &ViewEvent::ContentDialogResponded {
                widget: dialog,
                button: crate::ZsContentDialogButton::Primary,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::DialogResult(crate::ZsContentDialogResult::Primary)]
        );
        assert!(view
            .widget_content_dialog_state(dialog)
            .is_some_and(|(state, _)| !state.open));
        assert_eq!(
            view.interaction_plan()
                .first_focus_target()
                .map(|target| target.widget),
            Some(background)
        );
    }

    #[cfg(feature = "command-palette")]
    #[test]
    fn command_palette_is_filtered_modal_self_drawn_and_routes_strong_ids() {
        let palette = WidgetId::new(197);
        let page = WidgetId::new(198);
        let settings = crate::ZsCommandPaletteItemId::new(2);
        let file = crate::ZsCommandPaletteItemId::new(3);
        let mut view = command_palette(
            palette,
            true,
            "open",
            [
                crate::ZsCommandPaletteItem::new(1_u64, "New window").icon(crate::ZsIcon::Add),
                crate::ZsCommandPaletteItem::new(settings, "Open settings")
                    .keywords(["preferences"])
                    .shortcut("Ctrl+,"),
                crate::ZsCommandPaletteItem::new(file, "Open file")
                    .subtitle("Choose from disk")
                    .icon(crate::ZsIcon::File),
                crate::ZsCommandPaletteItem::new(4_u64, "Open recent").enabled(false),
            ],
            spacer::<Msg>().id(page).bg(ThemeColorToken::Surface),
        )
        .highlighted_command(Some(file))
        .on_command_palette_query_change(Msg::CommandQuery)
        .on_command_palette_highlight_change(Msg::CommandHighlight)
        .on_command_palette_invoke(Msg::CommandInvoke)
        .on_command_palette_open_change(Msg::CommandOpen);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 900,
            height: 620,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let state = view
            .widget_command_palette_state(palette)
            .expect("command palette state");
        assert_eq!(state.visible_items, vec![settings, file, 4_u64.into()]);
        assert_eq!(state.enabled_items, vec![settings, file]);
        assert_eq!(state.highlighted, Some(file));
        let interaction = view.interaction_plan();
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::CommandPalette)
        );
        assert_eq!(
            interaction.target_kind_at(Point { x: 4, y: 4 }),
            Some(ViewHitTargetKind::CommandPaletteScrim)
        );
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::CommandPaletteItem { item: file } }));
        assert!(!interaction.hit_targets.iter().any(|target| {
            target.kind == ViewHitTargetKind::CommandPaletteItem { item: 4_u64.into() }
        }));

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::Search
        )));

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::CommandPaletteInvoked {
                widget: palette,
                item: file,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::CommandInvoke(file), Msg::CommandOpen(false)]
        );
        assert!(view
            .widget_command_palette_state(palette)
            .is_some_and(|state| !state.open));
        assert_eq!(
            view.interaction_plan()
                .first_focus_target()
                .map(|target| target.widget),
            Some(page)
        );

        let mut reopen_events = ViewEventCx::new();
        view.event(
            &mut reopen_events,
            &ViewEvent::CommandPaletteOpenChanged {
                widget: palette,
                open: true,
            },
        );
        assert_eq!(reopen_events.into_messages(), vec![Msg::CommandOpen(true)]);
        assert!(view
            .widget_command_palette_state(palette)
            .is_some_and(|state| state.open && state.highlighted == Some(file)));
        assert_eq!(
            view.interaction_plan()
                .first_focus_target()
                .map(|target| target.widget),
            Some(palette)
        );
    }

    #[cfg(feature = "toast")]
    #[test]
    fn toast_presenter_overlays_page_and_routes_typed_action_without_blocking_page() {
        let presenter = WidgetId::new(105);
        let page = WidgetId::new(106);
        let toast_id = crate::ZsToastId::new(9);
        let mut view = toast_presenter(
            presenter,
            Some(crate::ZsToastSpec::new(toast_id, "File deleted").action("Undo")),
            spacer::<Msg>().id(page).bg(ThemeColorToken::Surface),
        )
        .on_toast_result(Msg::ToastResult);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 400,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let interaction = view.interaction_plan();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.widget == page));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::ToastAction));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::ToastClose));

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let page_index = paint
            .plan()
            .commands
            .iter()
            .position(|command| matches!(command, NativeDrawCommand::FillRect { rect, .. } if *rect == viewport))
            .expect("page background");
        let toast_index = paint
            .plan()
            .commands
            .iter()
            .rposition(|command| matches!(command, NativeDrawCommand::Icon(_)))
            .expect("toast close icon");
        assert!(toast_index > page_index);

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ToastResponded {
                widget: presenter,
                toast: toast_id,
                response: crate::ZsToastResponse::Action,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::ToastResult(crate::ZsToastResult {
                id: toast_id,
                response: crate::ZsToastResponse::Action,
            })]
        );
        assert!(view.widget_toast_state(presenter).is_none());
        assert_eq!(
            view.interaction_plan()
                .first_focus_target()
                .map(|target| target.widget),
            Some(page)
        );
    }

    #[cfg(feature = "teaching-tip")]
    #[test]
    fn teaching_tip_targets_stable_widget_and_routes_typed_action() {
        let presenter = WidgetId::new(115);
        let target = WidgetId::new(116);
        let mut view = teaching_tip(
            presenter,
            true,
            target,
            crate::ZsTeachingTipSpec::new(
                "Save automatically",
                "Your changes are saved as you work.",
            )
            .action("Review settings"),
            spacer::<Msg>().id(target),
        )
        .on_teaching_tip_result(Msg::TeachingTipResult);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 420,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let interaction = view.interaction_plan();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|candidate| candidate.kind == ViewHitTargetKind::TeachingTipAction));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|candidate| candidate.kind == ViewHitTargetKind::TeachingTipClose));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::FillTriangle { .. })));

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::TeachingTipFocused {
                widget: presenter,
                control: crate::ZsTeachingTipControl::Close,
            },
        );
        assert_eq!(
            view.widget_teaching_tip_state(presenter)
                .map(|(state, _)| state.focused_control),
            Some(crate::ZsTeachingTipControl::Close)
        );
        view.event(
            &mut events,
            &ViewEvent::TeachingTipResponded {
                widget: presenter,
                response: crate::ZsTeachingTipResponse::Action,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::TeachingTipResult(crate::ZsTeachingTipResult {
                response: crate::ZsTeachingTipResponse::Action,
            })]
        );
        assert!(view
            .widget_teaching_tip_state(presenter)
            .is_some_and(|(state, _)| !state.open));
    }

    #[cfg(feature = "info-bar")]
    #[test]
    fn info_bar_is_inline_and_routes_semantic_focus_and_typed_events() {
        let widget = WidgetId::new(107);
        let mut view = column([
            info_bar(
                widget,
                crate::ZsInfoBarSpec::new("Renew to keep all functionality.")
                    .title("Subscription expires soon")
                    .severity(crate::ZsInfoBarSeverity::Warning)
                    .action("Renew"),
            )
            .on_info_bar_event(Msg::InfoBarEvent),
            spacer::<Msg>().height(Dp::new(40.0)),
        ]);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 640,
            height: 160,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let interaction = view.interaction_plan();
        assert_eq!(
            interaction.first_focus_target().map(|target| target.widget),
            Some(widget)
        );
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::InfoBarAction));
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::InfoBarClose));

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::Warning
        )));

        let mut focus = ViewEventCx::new();
        view.event(
            &mut focus,
            &ViewEvent::InfoBarFocused {
                widget,
                control: crate::ZsInfoBarControl::Close,
            },
        );
        assert!(focus.messages().is_empty());
        assert_eq!(
            view.widget_info_bar_state(widget)
                .map(|(state, _)| state.focused_control),
            Some(Some(crate::ZsInfoBarControl::Close))
        );

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::InfoBarInvoked {
                widget,
                event: crate::ZsInfoBarEvent::Action,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::InfoBarEvent(crate::ZsInfoBarEvent::Action)]
        );
    }

    #[cfg(feature = "breadcrumb")]
    #[test]
    fn breadcrumb_routes_one_tab_stop_overflow_focus_and_typed_selection() {
        let widget = WidgetId::new(118);
        let first = crate::ZsBreadcrumbId::new(1);
        let selected = crate::ZsBreadcrumbId::new(2);
        let current = crate::ZsBreadcrumbId::new(5);
        let mut view = column([
            breadcrumb_bar([
                crate::ZsBreadcrumbItem::new(first, "Home"),
                crate::ZsBreadcrumbItem::new(selected, "Projects"),
                crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(3), "ZSUI Framework"),
                crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(4), "Documentation"),
                crate::ZsBreadcrumbItem::new(current, "BreadcrumbBar"),
            ])
            .id(widget)
            .width(Dp::new(240.0))
            .expanded(true)
            .on_expanded_change(Msg::BreadcrumbExpanded)
            .on_breadcrumb_select(Msg::BreadcrumbSelected),
            spacer::<Msg>(),
        ]);
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 220,
        };
        view.layout(&mut ViewLayoutCx::new(viewport, Dpi::standard()));

        let interaction = view.interaction_plan();
        assert_eq!(
            interaction.first_focus_target().map(|target| target.widget),
            Some(widget)
        );
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| target.accepts_focus())
                .filter(|target| target.widget == widget)
                .count(),
            1
        );
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::BreadcrumbOverflow));
        assert!(interaction.hit_targets.iter().any(|target| matches!(
            target.kind,
            ViewHitTargetKind::BreadcrumbOverflowItem { .. }
        )));

        let mut focus = ViewEventCx::new();
        view.event(
            &mut focus,
            &ViewEvent::BreadcrumbFocused {
                widget,
                target: crate::ZsBreadcrumbFocusTarget::Overflow,
            },
        );
        assert!(focus.messages().is_empty());
        assert_eq!(
            view.widget_breadcrumb_state(widget)
                .and_then(|state| state.focused),
            Some(crate::ZsBreadcrumbFocusTarget::Overflow)
        );

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::BreadcrumbSelected {
                widget,
                item: selected,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::BreadcrumbExpanded(false),
                Msg::BreadcrumbSelected(selected),
            ]
        );
        assert!(view
            .widget_breadcrumb_state(widget)
            .is_some_and(|state| !state.overflow_open
                && state.focused == Some(crate::ZsBreadcrumbFocusTarget::Item(selected))));

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::More
        )));
    }

    #[cfg(feature = "combo")]
    #[test]
    fn combo_box_routes_overlay_selection_and_paints_above_later_siblings() {
        let combo_id = WidgetId::new(9);
        let mut view = column([
            combo_box(["Balanced", "Fast", "Quiet"], Some(0))
                .id(combo_id)
                .height(Dp::new(36.0))
                .expanded(true)
                .on_select(Msg::ComboSelected)
                .on_expanded_change(Msg::ComboExpanded),
            spacer().bg(ThemeColorToken::Control),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 160,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let interaction = view.interaction_plan();
        let option = interaction
            .hit_targets
            .iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .copied()
            .expect("expanded option should be in the overlay hit plan");
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::ComboBox)
        );
        assert_eq!(
            interaction.target_kind_at(Point {
                x: option.bounds.x + 8,
                y: option.bounds.y + option.bounds.height / 2,
            }),
            Some(ViewHitTargetKind::ComboBoxOption { index: 1 })
        );

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ComboBoxSelected {
                widget: combo_id,
                index: 1,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::ComboSelected(1), Msg::ComboExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo_id), Some((Some(1), 3, false)));

        let mut expanded = combo_box::<_, ()>(["One", "Two"], Some(0))
            .id(combo_id)
            .expanded(true);
        expanded.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 36,
            },
            Dpi::standard(),
        ));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        expanded.paint(&mut paint);
        assert!(matches!(
            paint.plan().commands.last(),
            Some(NativeDrawCommand::Text(text)) if text.text == "Two"
        ));
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_rejects_out_of_range_initial_selection() {
        let view = combo_box::<_, ()>(["One"], Some(7)).id(WidgetId::new(10));
        assert_eq!(
            view.widget_combo_state(WidgetId::new(10)),
            Some((None, 1, false))
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_scrolls_a_bounded_popup_with_global_option_indices() {
        let combo_id = WidgetId::new(91);
        let options = (0..30)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let mut view = column([
            combo_box::<_, ()>(options, Some(0))
                .id(combo_id)
                .height(Dp::new(36.0))
                .expanded(true),
            spacer(),
        ]);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 200,
            },
            Dpi::standard(),
        ));

        let initial_plan = view.interaction_plan();
        let initial_range = initial_plan
            .combo_visible_option_range(combo_id)
            .expect("expanded long combo should expose visible options");
        assert_eq!(initial_range.start, 0);
        assert!(initial_range.len() < 30);

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ComboBoxScrolled {
                widget: combo_id,
                first_visible_index: 1,
            },
        );
        assert!(events.into_messages().is_empty());

        let scrolled_plan = view.interaction_plan();
        let scrolled_range = scrolled_plan
            .combo_visible_option_range(combo_id)
            .expect("scrolled combo should retain visible options");
        assert_eq!(scrolled_range.start, 1);
        assert_eq!(scrolled_range.len(), initial_range.len());
        let first_row = scrolled_plan
            .hit_targets
            .iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .expect("first scrolled row should keep its global option index");
        assert_eq!(
            scrolled_plan.target_kind_at(Point {
                x: first_row.bounds.x + 8,
                y: first_row.bounds.y + first_row.bounds.height / 2,
            }),
            Some(ViewHitTargetKind::ComboBoxOption { index: 1 })
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_type_ahead_match_wraps_after_selection() {
        let widget = WidgetId::new(12);
        let view = combo_box::<_, ()>(["Quartz", "Quiet", "Balanced"], Some(2)).id(widget);

        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "Q", Some(2)),
            Some(0)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "qu", Some(2)),
            Some(0)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "qui", Some(2)),
            Some(1)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "b", Some(1)),
            Some(2)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "missing", Some(2)),
            None
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_overlay_paint_and_hits_share_viewport_flipped_geometry() {
        let widget = WidgetId::new(11);
        let mut view = column([
            spacer(),
            combo_box::<_, ()>(["One", "Two", "Three"], None)
                .id(widget)
                .height(Dp::new(32.0))
                .expanded(true),
        ])
        .gap(Dp::new(4.0));
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 240,
            },
            Dpi::standard(),
        ));

        let option = view
            .interaction_plan()
            .hit_targets
            .into_iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .expect("second option should be hittable in the flipped popup");
        assert_eq!(option.bounds.y, 140);

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect { rect, .. }
                if *rect == Rect { x: 0, y: 108, width: 300, height: 96 }
        )));
    }

    #[test]
    #[cfg(feature = "date-picker")]
    fn date_picker_routes_typed_range_month_and_overlay_selection() {
        let widget = WidgetId::new(12);
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let minimum = ZsDate::new(2026, 7, 10).unwrap();
        let maximum = ZsDate::new(2026, 8, 20).unwrap();
        let mut view = date_picker(value)
            .id(widget)
            .height(Dp::new(32.0))
            .date_range(minimum, maximum)
            .today(ZsDate::new(2026, 7, 14).unwrap())
            .expanded(true)
            .on_date_change(Msg::DateChanged)
            .on_expanded_change(Msg::DateExpanded);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 24,
                y: 64,
                width: 472,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        let next_day = ZsDate::new(2026, 7, 14).unwrap();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::DatePickerDay { date: next_day } }));
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::DatePicker)
        );
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert_eq!(
            paint
                .plan()
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    NativeDrawCommand::RoundRect {
                        fill: NativeDrawFill::Role(ColorRole::Accent),
                        ..
                    }
                ))
                .count(),
            2
        );

        let mut month_events = ViewEventCx::new();
        view.event(
            &mut month_events,
            &ViewEvent::DatePickerMonthChanged {
                widget,
                month: ZsDate::new(2026, 8, 1).unwrap(),
            },
        );
        assert!(month_events.messages().is_empty());
        assert_eq!(
            view.widget_date_picker_state(widget)
                .expect("date picker state")
                .visible_month,
            ZsDate::new(2026, 8, 1).unwrap()
        );

        let mut selection_events = ViewEventCx::new();
        view.event(
            &mut selection_events,
            &ViewEvent::DateChanged {
                widget,
                value: ZsDate::new(2026, 8, 31).unwrap(),
            },
        );
        assert_eq!(
            selection_events.into_messages(),
            vec![Msg::DateChanged(maximum), Msg::DateExpanded(false)]
        );
        assert_eq!(
            view.widget_date_picker_state(widget),
            Some(ZsDatePickerState {
                value: maximum,
                minimum,
                maximum,
                visible_month: maximum.first_day_of_month(),
                expanded: false,
            })
        );
    }

    #[test]
    #[cfg(feature = "time-picker")]
    fn time_picker_routes_typed_increment_popup_and_selection() {
        let widget = WidgetId::new(13);
        let initial = ZsTime::new(18, 15).unwrap();
        let selected = ZsTime::new(18, 30).unwrap();
        let mut view = time_picker(initial)
            .id(widget)
            .height(Dp::new(32.0))
            .minute_increment(ZsMinuteIncrement::FIFTEEN)
            .clock_format(ZsClockFormat::TwentyFourHour)
            .expanded(true)
            .on_time_change(Msg::TimeChanged)
            .on_expanded_change(Msg::TimeExpanded);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 24,
                y: 180,
                width: 240,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert!(interaction.hit_targets.iter().any(|target| {
            target.kind == ViewHitTargetKind::TimePickerChoice { value: selected }
        }));
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::TimePicker)
        );
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(crate::NativeDrawIconCommand {
                icon: crate::ZsIcon::ChevronDown,
                ..
            })
        )));

        let mut selection_events = ViewEventCx::new();
        view.event(
            &mut selection_events,
            &ViewEvent::TimeChanged {
                widget,
                value: selected,
            },
        );
        assert_eq!(
            selection_events.into_messages(),
            vec![Msg::TimeChanged(selected)]
        );
        assert_eq!(
            view.widget_time_picker_state(widget),
            Some(ZsTimePickerState {
                value: selected,
                minute_increment: ZsMinuteIncrement::FIFTEEN,
                clock: ZsClockFormat::TwentyFourHour,
                expanded: true,
            })
        );

        let mut expanded_events = ViewEventCx::new();
        view.event(
            &mut expanded_events,
            &ViewEvent::TimePickerExpandedChanged {
                widget,
                expanded: false,
            },
        );
        assert_eq!(
            expanded_events.into_messages(),
            vec![Msg::TimeExpanded(false)]
        );
        assert_eq!(
            view.widget_time_picker_state(widget)
                .map(|state| state.expanded),
            Some(false)
        );
    }

    #[test]
    #[cfg(all(feature = "combo", feature = "date-picker"))]
    fn dismiss_popup_overlays_closes_every_expanded_control_except_the_owner() {
        let combo = WidgetId::new(90);
        let date = WidgetId::new(91);
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let mut view = column([
            combo_box(["One", "Two"], Some(0))
                .id(combo)
                .expanded(true)
                .on_expanded_change(Msg::ComboExpanded),
            date_picker(value)
                .id(date)
                .expanded(true)
                .on_expanded_change(Msg::DateExpanded),
        ]);

        let mut date_dismissed = ViewEventCx::new();
        view.event(
            &mut date_dismissed,
            &ViewEvent::DismissPopupOverlays {
                except: Some(combo),
            },
        );
        assert_eq!(
            date_dismissed.into_messages(),
            vec![Msg::DateExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo), Some((Some(0), 2, true)));
        assert_eq!(
            view.widget_date_picker_state(date)
                .map(|state| state.expanded),
            Some(false)
        );

        let mut all_dismissed = ViewEventCx::new();
        view.event(
            &mut all_dismissed,
            &ViewEvent::DismissPopupOverlays { except: None },
        );
        assert_eq!(
            all_dismissed.into_messages(),
            vec![Msg::ComboExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo), Some((Some(0), 2, false)));
    }

    #[test]
    #[cfg(all(feature = "list", feature = "label"))]
    fn list_view_routes_child_clicks_to_typed_selection_messages() {
        let first = WidgetId::new(10);
        let second = WidgetId::new(11);
        let mut view = list([(first, "One"), (second, "Two")], |(id, label)| {
            text(label).id(id)
        })
        .selected_index(Some(0))
        .on_select(Msg::RowSelected);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: second });

        assert_eq!(events.into_messages(), vec![Msg::RowSelected(1)]);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::RoundFill { .. })));
        assert_eq!(view.widget_list_index(second), Some(1));
        assert_eq!(
            view.widget_list_relative_widget(second, -1),
            Some((first, 0))
        );
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn scroll_view_offsets_children_and_clips_hit_targets() {
        let top = WidgetId::new(20);
        let bottom = WidgetId::new(21);
        let scroll_id = WidgetId::new(22);
        let mut view: ViewNode<Msg> = scroll(column([
            text("Top row").id(top).height(Dp::new(60.0)),
            text("Bottom row").id(bottom).height(Dp::new(60.0)),
        ]))
        .id(scroll_id)
        .content_height(Dp::new(120.0))
        .scroll_y(Dp::new(60.0))
        .on_scroll(Msg::ScrollChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 60,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let plan = view.interaction_plan();
        let mut events = ViewEventCx::new();
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: scroll_id,
                delta_y: Dp::new(-20.0),
            },
        );
        view.paint(&mut paint);

        assert_eq!(
            events.into_messages(),
            vec![Msg::ScrollChanged(Dp::new(40.0))]
        );
        assert_eq!(plan.target_at(Point { x: 20, y: 20 }), Some(bottom));
        assert_eq!(plan.hit_target_for_widget(top), None);
        assert_eq!(view.widget_scroll_target(bottom), Some(scroll_id));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PushClip { .. })));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PopClip)));
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn scroll_boundary_converts_viewport_pixels_at_high_dpi() {
        let scroll_id = WidgetId::new(23);
        let mut view: ViewNode<Msg> = scroll(text("High DPI content"))
            .id(scroll_id)
            .content_height(Dp::new(240.0))
            .scroll_y(Dp::new(170.0))
            .on_scroll(Msg::ScrollChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 120,
            },
            Dpi::new(192.0),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: scroll_id,
                delta_y: Dp::new(20.0),
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![Msg::ScrollChanged(Dp::new(180.0))]
        );
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn virtual_list_layout_and_paint_only_touch_the_materialized_window() {
        let list_id = WidgetId::new(600);
        let mut view = virtual_list(
            100_000,
            (490..520).map(|index| (index, format!("Row {index}"))),
            |index, label| text(label).id(WidgetId::new(1_000 + index as u64)),
        )
        .id(list_id)
        .height(Dp::new(100.0))
        .item_height(Dp::new(20.0))
        .overscan_rows(2)
        .scroll_y(Dp::new(10_000.0))
        .on_viewport_changed(Msg::ViewportChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 100,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(output.children.len(), 10);
        assert_eq!(view.interaction_plan().hit_target_count(), 6);
        assert_eq!(
            paint
                .plan()
                .commands
                .iter()
                .filter(|command| matches!(command, NativeDrawCommand::Text(_)))
                .count(),
            9
        );
        assert!(view.children[0].bounds().is_none());
        assert!(view.children[8].bounds().is_some());
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn virtual_list_scroll_emits_global_range_and_global_selection() {
        let list_id = WidgetId::new(700);
        let row_id = WidgetId::new(711);
        let mut view = virtual_list(100, [(11, "Eleven"), (12, "Twelve")], |index, label| {
            text(label).id(if index == 11 {
                row_id
            } else {
                WidgetId::new(712)
            })
        })
        .id(list_id)
        .item_height(Dp::new(20.0))
        .overscan_rows(1)
        .scroll_y(Dp::new(200.0))
        .on_select(Msg::RowSelected)
        .on_viewport_changed(Msg::ViewportChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 60,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: row_id });
        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: list_id,
                delta_y: Dp::new(20.0),
            },
        );

        assert_eq!(events.messages()[0], Msg::RowSelected(11));
        assert!(matches!(
            events.messages()[1],
            Msg::ViewportChanged(VirtualListViewport {
                visible_range: VirtualListRange { start: 11, end: 14 },
                materialized_range: VirtualListRange { start: 10, end: 15 },
                direction: VirtualListScrollDirection::Forward,
                ..
            })
        ));
        assert_eq!(view.widget_list_index(row_id), Some(11));
    }

    #[test]
    #[cfg(feature = "virtual-list")]
    fn virtual_list_viewport_clamps_large_offsets_without_iterating_items() {
        let viewport = virtual_list_viewport(
            100_000,
            Dp::new(24.0),
            Dp::new(f32::MAX),
            Dp::new(240.0),
            4,
            VirtualListScrollDirection::Forward,
        );

        assert_eq!(
            viewport.visible_range,
            VirtualListRange::new(99_990, 100_000)
        );
        assert_eq!(
            viewport.materialized_range,
            VirtualListRange::new(99_986, 100_000)
        );
        assert_eq!(viewport.offset_y, Dp::new(2_399_760.0));
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn live_view_background_poll_stops_after_loaded_state_is_refreshed() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };

        let loading = Arc::new(AtomicBool::new(true));
        let view_loading = Arc::clone(&loading);
        let runtime = live_view_runtime(
            (),
            move |_| {
                virtual_list(1, [(0, "Loaded")], |_, value| text(value))
                    .loading(view_loading.load(Ordering::SeqCst))
            },
            |_, _: (), _| {},
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 80,
            },
            Dpi::standard(),
        );

        assert_eq!(runtime.background_poll_interval_ms(), Some(33));
        loading.store(false, Ordering::SeqCst);
        let update = runtime.refresh();
        assert!(update.redraw);
        assert_eq!(update.revision, 1);
        assert_eq!(runtime.background_poll_interval_ms(), None);
    }

    #[test]
    #[cfg(all(feature = "tabs", feature = "label"))]
    fn tab_view_keeps_one_active_page_and_routes_strongly_typed_selection() {
        let tab_view_id = WidgetId::new(200);
        let general = ZsTabId::new(201);
        let advanced = ZsTabId::new(202);
        let about = ZsTabId::new(203);
        let general_content = WidgetId::new(211);
        let advanced_content = WidgetId::new(212);
        let mut view = tab_view(
            [
                ZsTabItem::new(
                    general,
                    "General",
                    text("General content").id(general_content),
                )
                .icon(crate::ZsIcon::Settings),
                ZsTabItem::new(
                    advanced,
                    "Advanced",
                    text("Advanced content").id(advanced_content),
                ),
                ZsTabItem::new(about, "About", text("About content")),
            ],
            Some(general),
        )
        .id(tab_view_id)
        .on_tab_select(Msg::TabSelected);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let interactions = view.interaction_plan();
        assert_eq!(
            interactions
                .hit_targets
                .iter()
                .filter(|target| matches!(target.kind, ViewHitTargetKind::Tab { .. }))
                .count(),
            3
        );
        assert!(interactions
            .hit_target_for_widget(general_content)
            .is_some());
        assert!(interactions
            .hit_target_for_widget(advanced_content)
            .is_none());
        assert!(view
            .widget_tab_header_state(WidgetId(general.0))
            .is_some_and(|state| state.selected));
        assert!(view
            .widget_tab_header_state(WidgetId(advanced.0))
            .is_some_and(|state| !state.selected));
        assert_eq!(
            view.widget_tab_cycle_target(general_content, 1),
            Some((tab_view_id, advanced))
        );

        let mut event_cx = ViewEventCx::new();
        view.event(
            &mut event_cx,
            &ViewEvent::TabSelected {
                widget: tab_view_id,
                tab: advanced,
            },
        );
        assert_eq!(event_cx.messages(), &[Msg::TabSelected(advanced)]);
        assert_eq!(
            view.widget_tab_view_state(tab_view_id),
            Some(ZsTabViewState {
                selected: Some(advanced),
                tab_count: 3,
            })
        );

        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Advanced content"
        )));
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(icon) if icon.icon == crate::ZsIcon::Settings
        )));
        assert!(!paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "General content"
        )));
    }

    #[test]
    fn app_context_keeps_commands_explicit() {
        let mut cx = AppCx::new();

        cx.command(Command::OpenSettings);
        cx.ui_command(crate::UiCommand::app(crate::CommandId("view.save")));
        cx.quit();

        assert_eq!(cx.commands(), &[Command::OpenSettings]);
        assert_eq!(cx.ui_commands()[0].id, crate::CommandId("view.save"));
        assert!(cx.quit_requested());
    }

    #[cfg(feature = "color-picker")]
    #[test]
    fn color_picker_keeps_rgba_state_typed_and_uses_one_tab_stop_with_overlay_rows() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Msg {
            Color(crate::Color),
            Expanded(bool),
            Channel(ZsColorChannel),
        }

        let widget = WidgetId::new(218);
        let initial =
            ZsColorPickerState::new(crate::Color::rgba(24, 80, 160, 200)).with_expanded(true);
        let mut view = color_picker(initial)
            .id(widget)
            .height(Dp::new(32.0))
            .on_color_change(Msg::Color)
            .on_expanded_change(Msg::Expanded)
            .on_color_channel_change(Msg::Channel);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 24,
                y: 20,
                width: 220,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::ColorPicker)
        );
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| target.accepts_focus())
                .count(),
            1
        );
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::ColorPickerPopup));
        let metrics = crate::ZsColorPickerMetrics::for_platform(
            crate::ZsColorPickerPlatformStyle::current(),
        );
        let has_spectrum = interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::ColorPickerSpectrum);
        let has_hue = interaction
            .hit_targets
            .iter()
            .any(|target| target.kind == ViewHitTargetKind::ColorPickerHue);
        assert_eq!(has_spectrum, metrics.spectrum_height.0 > 0.0);
        assert_eq!(has_hue, metrics.hue_track_height.0 > 0.0);
        assert_eq!(
            interaction
                .hit_targets
                .iter()
                .filter(|target| matches!(
                    target.kind,
                    ViewHitTargetKind::ColorPickerChannel { .. }
                ))
                .count(),
            4
        );

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "#1850A0C8"
        )));

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ColorPickerChannelChanged {
                widget,
                channel: ZsColorChannel::Green,
            },
        );
        view.event(
            &mut events,
            &ViewEvent::ColorChanged {
                widget,
                color: crate::Color::rgba(24, 192, 160, 200),
            },
        );
        view.event(
            &mut events,
            &ViewEvent::ColorPickerExpandedChanged {
                widget,
                expanded: false,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::Channel(ZsColorChannel::Green),
                Msg::Color(crate::Color::rgba(24, 192, 160, 200)),
                Msg::Expanded(false),
            ]
        );
        assert_eq!(
            view.widget_color_picker_state(widget),
            Some(ZsColorPickerState {
                color: crate::Color::rgba(24, 192, 160, 200),
                expanded: false,
                active_channel: ZsColorChannel::Green,
                alpha_enabled: true,
            })
        );
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn live_view_runtime_rebuilds_from_state_after_typed_message() {
        #[derive(Clone)]
        enum CounterMsg {
            Increment,
        }

        struct CounterState {
            value: u32,
        }

        let button_id = WidgetId::new(90);
        let runtime = live_view_runtime(
            CounterState { value: 0 },
            move |state| {
                column([
                    text(format!("Count: {}", state.value)),
                    button("Increment")
                        .id(button_id)
                        .on_click(CounterMsg::Increment),
                ])
            },
            |state, message, cx| match message {
                CounterMsg::Increment => {
                    state.value += 1;
                    cx.ui_command(UiCommand::app(crate::CommandId("counter.incremented")));
                }
            },
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 160,
            },
            Dpi::standard(),
        );

        let before = runtime.draw_plan();
        assert!(before.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 0"
        )));

        let update = runtime.dispatch_event(&ViewEvent::Click { widget: button_id });

        assert!(update.redraw);
        assert_eq!(update.message_count, 1);
        assert_eq!(update.revision, 1);
        assert_eq!(
            update.ui_commands[0].id,
            crate::CommandId("counter.incremented")
        );
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 1"
        )));
    }

    #[test]
    #[cfg(feature = "image-preview")]
    fn image_preview_paints_one_complete_frame_and_polls_only_while_loading() {
        let frame = crate::ZsImageFrame::from_rgba8(
            crate::ZsImageFrameId::new(9),
            2,
            1,
            vec![255, 0, 0, 255, 0, 255, 0, 255],
        )
        .unwrap();
        let snapshot = crate::ZsImagePreviewSnapshot {
            generation: 1,
            frame: Some(frame),
            loading: false,
            last_error: None,
        };
        let mut view: ViewNode<()> = image_preview(&snapshot).image_fit(crate::ZsImageFit::Cover);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            Dpi::standard(),
        ));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let plan = paint.into_plan();
        assert_eq!(plan.image_count(), 1);
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Image(image)
                if image.frame.id() == crate::ZsImageFrameId::new(9)
                    && image.source.width == 1
        )));
        assert_eq!(view.background_poll_interval_ms(), None);

        let loading: ViewNode<()> = image_preview(&crate::ZsImagePreviewSnapshot {
            generation: 2,
            frame: None,
            loading: true,
            last_error: None,
        });
        assert_eq!(loading.background_poll_interval_ms(), Some(16));
    }
}
