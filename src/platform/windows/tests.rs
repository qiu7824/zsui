#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_plan_theme_mode_selects_shared_dark_palette() {
        let plan = NativeDrawPlan::default().theme_mode(crate::ZsuiThemeMode::Dark);
        let palette = windows_palette_for_draw_plan(Some(&plan));

        assert_eq!(palette.surface, crate::ZsuiTheme::dark().colors.surface);
        assert_eq!(
            palette.primary_text,
            crate::ZsuiTheme::dark().colors.text_primary
        );
    }

    #[test]
    fn high_contrast_system_mode_overrides_explicit_light_or_dark_preferences() {
        assert_eq!(
            resolved_windows_theme_mode_for_system(
                crate::ZsuiThemeMode::Light,
                crate::ZsuiThemeMode::HighContrast,
            ),
            crate::ZsuiThemeMode::HighContrast
        );
        assert_eq!(
            resolved_windows_theme_mode_for_system(
                crate::ZsuiThemeMode::System,
                crate::ZsuiThemeMode::Dark,
            ),
            crate::ZsuiThemeMode::Dark
        );
    }

    #[test]
    fn high_contrast_palette_uses_user_selected_system_color_pairs() {
        let palette = windows_high_contrast_palette();
        assert_eq!(palette.surface, windows_system_color(COLOR_WINDOW));
        assert_eq!(palette.primary_text, windows_system_color(COLOR_WINDOWTEXT));
        assert_eq!(palette.accent, windows_system_color(COLOR_HIGHLIGHT));
        assert_eq!(
            palette.accent_text,
            windows_system_color(COLOR_HIGHLIGHTTEXT)
        );
        assert_eq!(palette.border, palette.primary_text);
    }

    #[test]
    fn file_dialog_filter_and_multi_select_buffer_are_structured_utf16() {
        let filters = vec![
            crate::FileDialogFilter::new("Text", ["*.txt", "*.md"]),
            crate::FileDialogFilter::new("All", ["*.*"]),
        ];
        let filter_buffer = windows_file_dialog_filter(&filters);
        let filter_parts = parse_windows_utf16_segments(&filter_buffer);

        assert_eq!(
            filter_parts,
            vec![
                OsString::from("Text"),
                OsString::from("*.txt;*.md"),
                OsString::from("All"),
                OsString::from("*.*"),
            ]
        );
        assert_eq!(
            windows_file_dialog_default_extension(&filters),
            Some(vec!['t' as u16, 'x' as u16, 't' as u16, 0])
        );

        let mut selection = Vec::new();
        for part in ["C:\\docs", "one.txt", "two.md"] {
            selection.extend(part.encode_utf16());
            selection.push(0);
        }
        selection.push(0);
        assert_eq!(
            parse_windows_open_file_buffer(&selection),
            vec![
                PathBuf::from("C:\\docs\\one.txt"),
                PathBuf::from("C:\\docs\\two.md")
            ]
        );
    }

    fn view_input_route_test_lock() -> std::sync::MutexGuard<'static, ()> {
        windows_win32_view_input_route_test_lock()
    }

    #[test]
    fn main_window_styles_map_to_win32_flags() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::standard(),
        );

        assert_eq!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_eq!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.style & WS_CAPTION, 0);
        assert_ne!(plan.style & WS_SYSMENU, 0);
        assert_ne!(plan.style & WS_THICKFRAME, 0);
        assert_ne!(plan.style & WS_MAXIMIZEBOX, 0);
    }

    #[test]
    fn tool_window_shape_maps_to_popup_topmost_flags() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::tool_window(),
        );

        assert_ne!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_ne!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.style & WS_POPUP, 0);
        assert_eq!(plan.style & WS_CAPTION, 0);
        assert_eq!(plan.style & WS_THICKFRAME, 0);
    }

    #[test]
    fn decorated_window_converts_requested_client_size_to_outer_size() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Main,
            &NativeWindowOptions::standard(),
        );
        let (width, height) =
            unsafe { windows_win32_outer_size_for_client(1280, 800, plan.style, plan.ex_style) };

        assert!(width >= 1280);
        assert!(height > 800);
    }

    #[test]
    fn quick_window_forces_no_activate_topmost_tool_window() {
        let plan = windows_win32_main_window_style_plan(
            WindowsWindowRole::Quick,
            &NativeWindowOptions::standard(),
        );

        assert_ne!(plan.ex_style & WS_EX_TOPMOST, 0);
        assert_ne!(plan.ex_style & WS_EX_TOOLWINDOW, 0);
        assert_ne!(plan.ex_style & WS_EX_NOACTIVATE, 0);
    }

    #[test]
    fn window_create_params_preserve_role_and_min_size_for_win32_create() {
        let params = WindowsWindowCreateParams::new(
            WindowsWindowRole::Main,
            Some(Size {
                width: 640,
                height: 420,
            }),
        );

        let decoded = WindowsWindowCreateParams::from_create_param(&params as *const _ as isize);
        assert_eq!(decoded, params);
        assert_eq!(
            WindowsWindowCreateParams::from_create_param(WindowsWindowRole::Quick as isize),
            WindowsWindowCreateParams::new(WindowsWindowRole::Quick, None)
        );
    }

    #[test]
    fn window_draw_plan_registry_tracks_native_paint_content() {
        let _guard = view_input_route_test_lock();
        let hwnd = 1isize as HWND;
        let plan = NativeDrawPlan::new([crate::NativeDrawCommand::FillRect {
            rect: crate::Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
            fill: crate::NativeDrawFill::Role(crate::ColorRole::Accent),
        }]);

        assert!(set_windows_win32_window_draw_plan(hwnd, plan.clone()));
        assert_eq!(window_draw_plan(hwnd), Some(plan));

        clear_windows_win32_window_draw_plan(hwnd);
        assert_eq!(window_draw_plan(hwnd), None);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_click_to_ui_command() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 77isize as HWND;
        let widget = crate::WidgetId::new(9);
        let executor = crate::SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![crate::AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 48,
                },
                crate::ViewHitTargetKind::Button,
            )]),
            crate::button("Save")
                .id(widget)
                .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.save"))),
        )
        .ui_command_executor(executor.clone());

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let dispatched =
            dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
                .expect("registered route should dispatch click");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(dispatched.hit_target_count, 1);
        assert_eq!(dispatched.click_count, 1);
        assert_eq!(dispatched.event_count, 1);
        assert_eq!(dispatched.ui_command_count, 1);
        assert_eq!(dispatched.ui_command_executed_count, 1);
        assert_eq!(dispatched.ui_command_event_count, 1);
        assert_eq!(executor.report().executed_count, 1);
        assert_eq!(dispatched.ui_command_ids, vec!["zsui.test.win32.save"]);
        assert_eq!(dispatched.focus_count, 1);
        assert_eq!(dispatched.focused_widget, Some(widget.0));
        assert_eq!(aggregate.click_count, 1);
        assert_eq!(aggregate.focus_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
        assert!(windows_win32_window_view_input_report(hwnd).is_none());
    }

    #[test]
    fn window_shell_route_updates_navigation_and_draw_plan() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_shell_input_routes();
        let hwnd = 0x5252isize as HWND;
        let spec = crate::ZsShellLayoutSpec::new("gallery", "Gallery")
            .selected_nav("general")
            .nav_item(crate::ZsShellNavItemSpec::new("general", "General"))
            .nav_item(crate::ZsShellNavItemSpec::new("controls", "Controls"));
        let runtime = crate::ZsShellRuntime::new(
            spec,
            crate::Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            crate::Dpi::standard(),
        );

        assert!(set_windows_win32_window_shell_input_route(
            hwnd,
            WindowsWin32ShellInputRoute::new(runtime),
        ));
        let update =
            dispatch_windows_win32_window_shell_pointer_down(hwnd, crate::Point { x: 40, y: 140 })
                .expect("shell route should handle pointer input");

        assert_eq!(
            update.events,
            vec![crate::ZsShellInteractionEvent::NavigationSelected {
                id: "controls".to_string(),
            }]
        );
        assert!(window_draw_plan(hwnd).is_some());
        assert_eq!(
            windows_win32_window_shell_input_events(hwnd).unwrap(),
            update.events
        );

        clear_windows_win32_window_shell_input_route(hwnd);
        clear_windows_win32_window_draw_plan(hwnd);
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn window_live_view_route_updates_state_and_repaints_draw_plan() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 0x5353isize as HWND;
        let button_id = crate::WidgetId::new(501);

        #[derive(Clone)]
        enum Msg {
            Increment,
        }
        struct State {
            count: u32,
        }

        let runtime = crate::live_view_runtime(
            State { count: 0 },
            move |state| {
                crate::column([
                    crate::text(format!("Count: {}", state.count)),
                    crate::button("Increment")
                        .id(button_id)
                        .on_click(Msg::Increment),
                ])
            },
            |state, message, cx| match message {
                Msg::Increment => {
                    state.count += 1;
                    cx.command(crate::Command::custom("counter.incremented"));
                    cx.ui_command(UiCommand::app(crate::CommandId("counter.persist")));
                }
            },
            crate::Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 120,
            },
            crate::Dpi::standard(),
        );
        let executor = crate::SharedAppCommandExecutor::new(|command| {
            Ok(vec![crate::AppEvent::MenuCommand { command }])
        });
        let ui_executor = crate::SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![crate::AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });
        assert!(set_windows_win32_window_view_input_route(
            hwnd,
            WindowsWin32ViewInputRoute::from_live_view(runtime)
                .app_command_executor(executor.clone())
                .ui_command_executor(ui_executor.clone()),
        ));

        let report = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 150, y: 90 })
            .expect("live view route should handle click");

        assert_eq!(report.message_count, 1);
        assert_eq!(report.app_command_count, 1);
        assert_eq!(report.app_command_executed_count, 1);
        assert_eq!(report.app_command_event_count, 1);
        assert_eq!(executor.report().executed_count, 1);
        assert_eq!(report.ui_command_count, 1);
        assert_eq!(report.ui_command_executed_count, 1);
        assert_eq!(report.ui_command_event_count, 1);
        assert_eq!(ui_executor.report().executed_count, 1);
        assert_eq!(report.live_view_revision, 2);
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_live_view_repaint:1"));
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_live_view_app_effect_refresh:2"));
        assert!(window_draw_plan(hwnd)
            .unwrap()
            .commands
            .iter()
            .any(|command| matches!(
                command,
                crate::NativeDrawCommand::Text(text) if text.text == "Count: 1"
            )));

        clear_windows_win32_window_view_input_route(hwnd);
        clear_windows_win32_window_draw_plan(hwnd);
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn window_menu_command_updates_typed_live_view_and_repaints() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_draw_plans();
        clear_windows_win32_window_view_input_routes();
        clear_windows_win32_window_menu_command_tables();
        let hwnd = 0x5454isize as HWND;

        #[derive(Clone)]
        enum Msg {
            Open,
        }
        struct State {
            status: &'static str,
        }

        let builder = crate::native_window("Menu State").stateful_view_with_app_commands(
            State { status: "Ready" },
            |state| crate::text::<Msg>(state.status),
            |state, message, _cx| match message {
                Msg::Open => state.status = "Opened from native menu",
            },
            |command| match command {
                Command::Custom { id, .. } if id == "document.open" => Some(Msg::Open),
                _ => None,
            },
        );
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful view should keep a live runtime")
            .clone();
        assert!(set_windows_win32_window_view_input_route(
            hwnd,
            WindowsWin32ViewInputRoute::from_live_view(runtime.clone()),
        ));
        let table = WindowsWin32StatusMenuCommandTable::from_menu(
            &MenuSpec::new().item("Open", Command::custom("document.open")),
        );
        let native_id = table
            .first_native_id()
            .expect("menu should allocate a native command id");
        set_windows_win32_window_menu_command_table(hwnd, table);

        assert!(matches!(
            dispatch_windows_win32_window_menu_command(hwnd, native_id),
            Some(NativeStatusMenuCommandResult::Dispatched(Command::Custom { id, .. }))
                if id == "document.open"
        ));
        assert_eq!(runtime.revision(), 1);
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Opened from native menu"
        )));
        assert!(window_draw_plan(hwnd).is_some());

        clear_windows_win32_window_menu_command_table(hwnd);
        clear_windows_win32_window_view_input_route(hwnd);
        clear_windows_win32_window_draw_plan(hwnd);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_keyboard_activation_to_ui_command() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 80isize as HWND;
        let widget = crate::WidgetId::new(12);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 48,
                },
                crate::ViewHitTargetKind::Button,
            )]),
            crate::button("Save")
                .id(widget)
                .on_click(UiCommand::app(crate::CommandId(
                    "zsui.test.win32.keyboard_save",
                ))),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should focus button");
        let key = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_RETURN)
            .expect("focused button should dispatch keyboard activation");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(key.key_down_count, 1);
        assert_eq!(key.keyboard_activation_count, 1);
        assert_eq!(key.event_count, 1);
        assert_eq!(key.ui_command_count, 1);
        assert_eq!(key.ui_command_ids, vec!["zsui.test.win32.keyboard_save"]);
        assert_eq!(aggregate.key_down_count, 1);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "button")]
    fn window_view_input_route_dispatches_tab_focus_traversal() {
        let _guard = view_input_route_test_lock();
        clear_windows_win32_window_view_input_routes();
        let hwnd = 82isize as HWND;
        let first = crate::WidgetId::new(15);
        let second = crate::WidgetId::new(16);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([
                crate::ViewHitTarget::with_kind(
                    first,
                    crate::Rect {
                        x: 0,
                        y: 0,
                        width: 120,
                        height: 48,
                    },
                    crate::ViewHitTargetKind::Button,
                ),
                crate::ViewHitTarget::with_kind(
                    second,
                    crate::Rect {
                        x: 0,
                        y: 48,
                        width: 120,
                        height: 48,
                    },
                    crate::ViewHitTargetKind::Button,
                ),
            ]),
            crate::column([
                crate::button("First")
                    .id(first)
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.first"))),
                crate::button("Second")
                    .id(second)
                    .on_click(UiCommand::app(crate::CommandId("zsui.test.win32.second"))),
            ]),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let first_focus = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_TAB)
            .expect("registered route should focus first target from Tab");
        let second_focus = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_TAB)
            .expect("registered route should focus next target from Tab");
        let key = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_RETURN)
            .expect("focused second button should dispatch keyboard activation");
        let focused_plan = window_draw_plan(hwnd).expect("focus should publish a draw plan");
        let blur = dispatch_windows_win32_window_view_blur(hwnd)
            .expect("registered route should clear focus visuals");
        let blurred_plan = window_draw_plan(hwnd).expect("blur should publish a clean draw plan");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(first_focus.focus_traversal_count, 1);
        assert_eq!(first_focus.focus_visual_count, 1);
        assert_eq!(first_focus.focused_widget, Some(first.0));
        assert_eq!(second_focus.focus_traversal_count, 1);
        assert_eq!(second_focus.focus_visual_count, 1);
        assert_eq!(second_focus.focused_widget, Some(second.0));
        assert!(focused_plan.commands.iter().any(|command| {
            matches!(command, crate::NativeDrawCommand::StrokeRect { rect, width: 2, .. }
                if rect.x == 1 && rect.y == 49 && rect.width == 118 && rect.height == 46)
        }));
        assert_eq!(key.ui_command_ids, vec!["zsui.test.win32.second"]);
        assert_eq!(blur.focus_visual_count, 1);
        assert!(!blurred_plan
            .commands
            .iter()
            .any(|command| { matches!(command, crate::NativeDrawCommand::StrokeRect { .. }) }));
        assert_eq!(aggregate.focus_traversal_count, 2);
        assert_eq!(aggregate.key_down_count, 3);
        assert_eq!(aggregate.focus_count, 2);
        assert_eq!(aggregate.focus_visual_count, 3);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(all(feature = "list", feature = "label"))]
    fn window_view_input_route_dispatches_list_selection_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn selected(_: usize) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.list_selected"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 81isize as HWND;
        let first = crate::WidgetId::new(13);
        let second = crate::WidgetId::new(14);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([
                crate::ViewHitTarget::new(
                    first,
                    crate::Rect {
                        x: 0,
                        y: 0,
                        width: 180,
                        height: 40,
                    },
                ),
                crate::ViewHitTarget::new(
                    second,
                    crate::Rect {
                        x: 0,
                        y: 40,
                        width: 180,
                        height: 40,
                    },
                ),
            ]),
            crate::list([(first, "One"), (second, "Two")], |(id, label)| {
                crate::text(label).id(id)
            })
            .on_select(selected),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let selection =
            dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 60 })
                .expect("registered route should select list row");
        let keyboard_selection =
            dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_UP)
                .expect("registered route should move list selection from keyboard");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(selection.click_count, 1);
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.selection_count, 1);
        assert_eq!(selection.ui_command_count, 1);
        assert_eq!(
            selection.ui_command_ids,
            vec!["zsui.test.win32.list_selected"]
        );
        assert_eq!(keyboard_selection.key_down_count, 1);
        assert_eq!(keyboard_selection.selection_count, 1);
        assert_eq!(keyboard_selection.keyboard_selection_count, 1);
        assert_eq!(keyboard_selection.ui_command_count, 1);
        assert_eq!(
            keyboard_selection.ui_command_ids,
            vec!["zsui.test.win32.list_selected"]
        );
        assert_eq!(aggregate.selection_count, 2);
        assert_eq!(aggregate.keyboard_selection_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn window_view_input_route_dispatches_scroll_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn scrolled(_: crate::Dp) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.scrolled"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 83isize as HWND;
        let scroll_id = crate::WidgetId::new(17);
        let row = crate::WidgetId::new(18);
        let mut view = crate::scroll(crate::text("Row").id(row))
            .id(scroll_id)
            .content_height(crate::Dp::new(120.0))
            .on_scroll(scrolled);
        let mut layout = crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 40,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let route = WindowsWin32ViewInputRoute::new(view.interaction_plan(), view);

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let scroll = dispatch_windows_win32_window_view_scroll(
            hwnd,
            crate::Point { x: 20, y: 20 },
            crate::Dp::new(48.0),
        )
        .expect("registered route should dispatch scroll");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(scroll.scroll_count, 1);
        assert_eq!(scroll.unhandled_scroll_count, 0);
        assert_eq!(scroll.event_count, 1);
        assert_eq!(scroll.ui_command_count, 1);
        assert_eq!(scroll.ui_command_ids, vec!["zsui.test.win32.scrolled"]);
        assert_eq!(aggregate.scroll_count, 1);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_dispatches_text_input_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn text_changed(_: String) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.text_changed"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 78isize as HWND;
        let widget = crate::WidgetId::new(10);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("").id(widget).on_change(text_changed),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let focus = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should focus textbox");
        let text = dispatch_windows_win32_window_view_text_input(hwnd, "ZS")
            .expect("focused textbox should dispatch text");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(focus.focus_count, 1);
        assert_eq!(focus.focused_widget, Some(widget.0));
        assert_eq!(text.text_input_count, 2);
        assert_eq!(text.event_count, 1);
        assert_eq!(text.ui_command_count, 1);
        assert_eq!(text.ui_command_ids, vec!["zsui.test.win32.text_changed"]);
        assert_eq!(aggregate.focus_count, 1);
        assert_eq!(aggregate.text_input_count, 2);
        assert_eq!(aggregate.ui_command_count, 1);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(all(feature = "tooltip", feature = "button"))]
    fn window_view_input_route_ticks_delayed_tooltip_into_buffered_draw_plan() {
        let widget = crate::WidgetId::new(1009);
        let mut view: crate::ViewNode<UiCommand> = crate::button("Save")
            .id(widget)
            .tooltip_spec(crate::ZsTooltipSpec::new("Save document").open_delay_ms(100));
        let surface = crate::Rect {
            x: 0,
            y: 0,
            width: 240,
            height: 120,
        };
        view.layout(&mut crate::ViewLayoutCx::new(
            surface,
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let target = interaction
            .tooltip_for_widget(widget)
            .expect("tooltip target should be collected");
        let point = crate::Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        };
        let start = std::time::Instant::now();
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        route.dispatch_pointer_move_at(point, start);
        let tick = route.refresh_background_view_at(start + std::time::Duration::from_millis(100));
        let draw = route
            .take_pending_draw_plan()
            .expect("tooltip tick should rebuild the buffered draw plan");

        assert!(tick.handled);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "Save document"
        )));
        assert_eq!(route.hit_target_count(), 1);
    }

    #[test]
    #[cfg(feature = "password-box")]
    fn window_view_input_route_keeps_password_text_and_peek_plans_redacted() {
        fn password_changed(_: crate::ZsPassword) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.password_changed"))
        }

        let widget = crate::WidgetId::new(1010);
        let initial_secret = "A🙂";
        let mut view = crate::password_box(initial_secret)
            .id(widget)
            .height(crate::Dp::new(36.0))
            .reveal_mode(crate::ZsPasswordRevealMode::Peek)
            .on_password_change(password_changed);
        let mut layout = crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 220,
                height: 36,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let interaction = view.interaction_plan();
        let input = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBox)
            .expect("password box should expose a Win32 input target");
        let reveal = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::PasswordBoxReveal)
            .expect("password box should expose a Win32 reveal target");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);
        route.dispatch_click(crate::Point {
            x: input.bounds.x + 2,
            y: input.bounds.y + input.bounds.height / 2,
        });

        let typed = route.dispatch_text_input("中");
        let current_secret = "A🙂中";
        assert_eq!(typed.text_input_count, 1);
        assert_eq!(
            typed.ui_command_ids,
            vec!["zsui.test.win32.password_changed"]
        );
        assert_eq!(
            route
                .widget_password_value(widget)
                .map(|value| value.as_str().to_owned())
                .as_deref(),
            Some(current_secret)
        );
        assert!(!format!("{typed:?}").contains(current_secret));
        let _ = route.take_pending_draw_plan();

        let reveal_point = crate::Point {
            x: reveal.bounds.x + reveal.bounds.width / 2,
            y: reveal.bounds.y + reveal.bounds.height / 2,
        };
        let pressed = route.dispatch_pointer_down(reveal_point, false);
        let pressed_plan = route
            .take_pending_draw_plan()
            .expect("Win32 reveal press should rebuild the draw plan");
        assert!(pressed.handled);
        assert!(pressed_plan.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::SecureText(command) if command.character_count() == 3
        )));
        assert!(!format!("{pressed_plan:?}").contains(current_secret));
        assert!(!serde_json::to_string(&pressed_plan)
            .expect("Win32 peek plan should serialize redacted")
            .contains(current_secret));

        let released = route.dispatch_pointer_up(reveal_point);
        let released_plan = route
            .take_pending_draw_plan()
            .expect("Win32 reveal release should restore the mask");
        assert!(released.handled);
        assert!(released_plan.commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::Text(text) if text.text == "•••"
        )));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_replaces_unicode_keyboard_selection() {
        fn selection_changed(_: crate::ZsTextSelection) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.text_selection"))
        }

        let widget = crate::WidgetId::new(32);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("A中文Z")
                .id(widget)
                .on_text_selection_change(selection_changed),
        );
        route.dispatch_click(crate::Point { x: 20, y: 20 });
        route.dispatch_key_down(u32::from(VK_HOME));
        route.dispatch_key_down(u32::from(VK_RIGHT));
        route.dispatch_key_down_with_shift(u32::from(VK_RIGHT), true);
        let selected = route.dispatch_key_down_with_shift(u32::from(VK_RIGHT), true);
        let selection_plan = route
            .take_pending_draw_plan()
            .expect("selection navigation should rebuild the draw plan");

        let replaced = route.dispatch_text_input("🙂");

        assert_eq!(selected.text_navigation_count, 1);
        assert_eq!(selected.text_selection_change_count, 1);
        assert_eq!(selected.text_caret, Some(3));
        assert_eq!(
            selected.ui_command_ids,
            vec!["zsui.test.win32.text_selection"]
        );
        assert!(selection_plan.commands.iter().any(|command| {
            matches!(
                command,
                crate::NativeDrawCommand::FillRect {
                    fill: crate::NativeDrawFill::RoleWithAlpha {
                        role: crate::ColorRole::Accent,
                        alpha: 64,
                    },
                    ..
                }
            )
        }));
        assert_eq!(replaced.text_caret, Some(2));
        assert_eq!(
            replaced.ui_command_ids,
            vec!["zsui.test.win32.text_selection"]
        );
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A🙂Z"));
    }

    #[test]
    #[cfg(all(feature = "textbox", feature = "text-input-core"))]
    fn window_view_input_route_moves_bidirectional_caret_in_visual_order() {
        let widget = crate::WidgetId::new(325);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox::<UiCommand>("abאב").id(widget),
        );
        route.dispatch_click(crate::Point { x: 10, y: 20 });
        route.dispatch_key_down(u32::from(VK_HOME));

        for expected in [1, 4, 3, 2] {
            let moved = route.dispatch_key_down(u32::from(VK_RIGHT));
            assert_eq!(moved.text_caret, Some(expected));
            assert_eq!(moved.text_navigation_count, 1);
        }
        for expected in [3, 4, 1, 0] {
            let moved = route.dispatch_key_down(u32::from(VK_LEFT));
            assert_eq!(moved.text_caret, Some(expected));
            assert_eq!(moved.text_navigation_count, 1);
        }
    }

    #[test]
    #[cfg(all(feature = "textbox", feature = "text-input-core"))]
    fn window_view_input_route_navigates_and_deletes_extended_graphemes() {
        let widget = crate::WidgetId::new(319);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 220,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox::<UiCommand>("A\u{65}\u{301}👩🏽‍💻Z").id(widget),
        );
        route.dispatch_click(crate::Point { x: 10, y: 20 });
        route.dispatch_key_down(u32::from(VK_END));

        let before_z = route.dispatch_key_down(u32::from(VK_LEFT));
        let before_emoji = route.dispatch_key_down(u32::from(VK_LEFT));
        let deleted = route.dispatch_text_input("\u{8}");

        assert_eq!(before_z.text_caret, Some(7));
        assert_eq!(before_emoji.text_caret, Some(3));
        assert_eq!(deleted.text_caret, Some(1));
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A👩🏽‍💻Z"));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_moves_wrapped_editor_caret_by_visual_row() {
        #[derive(Clone)]
        enum Msg {
            Selection(crate::ZsTextSelection),
        }

        let widget = crate::WidgetId::new(320);
        let builder = crate::native_window("Win32 wrapped navigation")
            .size(48, 140)
            .stateful_view(
                crate::ZsTextSelection::default(),
                move |_selection| {
                    crate::text_editor("abcdef\nx\nuvwxyz")
                        .id(widget)
                        .width(crate::Dp::new(48.0))
                        .height(crate::Dp::new(120.0))
                        .on_text_selection_change(Msg::Selection)
                },
                |selection, message, _cx| match message {
                    Msg::Selection(next) => *selection = next,
                },
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("wrapped editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("wrapped editor should expose Win32 geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        let point = crate::Point {
            x: target.bounds.x + 24,
            y: target.bounds.y + 10,
        };
        route.dispatch_pointer_down(point, false);
        route.dispatch_pointer_up(point);

        let second_visual_row = route.dispatch_key_down(u32::from(VK_DOWN));
        let short_hard_line = route.dispatch_key_down(u32::from(VK_DOWN));
        let next_wrapped_line = route.dispatch_key_down(u32::from(VK_DOWN));
        let extended = route.dispatch_key_down_with_shift(u32::from(VK_UP), true);

        assert_eq!(second_visual_row.text_caret, Some(6));
        assert_eq!(short_hard_line.text_caret, Some(8));
        assert_eq!(next_wrapped_line.text_caret, Some(11));
        assert_eq!(extended.text_caret, Some(8));
        assert_eq!(extended.text_selection_change_count, 1);
        assert_eq!(extended.message_count, 1);
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_scrolls_editor_and_reveals_keyboard_caret() {
        let widget = crate::WidgetId::new(321);
        let value = "row0\nrow1\nrow2\nrow3\nrow4\nrow5";
        let builder = crate::native_window("Win32 editor viewport")
            .size(160, 80)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>(value)
                        .id(widget)
                        .width(crate::Dp::new(120.0))
                        .height(crate::Dp::new(52.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("editor should expose Win32 viewport geometry");
        let point = crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(point, false);
        route.dispatch_pointer_up(point);

        let scrolled = route.dispatch_scroll(point, crate::Dp::new(48.0));
        let scrolled_plan = route
            .take_pending_draw_plan()
            .expect("editor scroll should rebuild the Win32 draw plan");
        route.dispatch_key_down(u32::from(VK_RIGHT));
        let revealed_plan = route
            .take_pending_draw_plan()
            .expect("keyboard navigation should reveal the caret row");

        assert!(scrolled.handled);
        assert_eq!(scrolled.scroll_count, 1);
        assert_eq!(scrolled.unhandled_scroll_count, 0);
        assert!(scrolled_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row3"),
        ));
        assert!(!scrolled_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row0"),
        ));
        assert!(revealed_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "row0"),
        ));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_reveals_no_wrap_columns_for_pointer_hits() {
        let widget = crate::WidgetId::new(322);
        let builder = crate::native_window("Win32 horizontal editor viewport")
            .size(48, 70)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>("0123456789")
                        .id(widget)
                        .text_wrap(crate::TextWrap::NoWrap)
                        .width(crate::Dp::new(48.0))
                        .height(crate::Dp::new(52.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("no-wrap editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("no-wrap editor should expose Win32 viewport geometry");
        let left = crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 10,
        };
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(left, false);
        route.dispatch_pointer_up(left);

        let revealed = route.dispatch_key_down(u32::from(VK_END));
        let revealed_plan = route
            .take_pending_draw_plan()
            .expect("End should reveal the no-wrap caret column");
        let clicked = route.dispatch_pointer_down(left, false);

        assert_eq!(revealed.text_caret, Some(10));
        assert!(revealed_plan.commands.iter().any(|command| {
            matches!(command, crate::NativeDrawCommand::Text(text)
                if text.text == "0123456789" && text.bounds.x < target.bounds.x + 8)
        }));
        assert_eq!(clicked.text_caret, Some(6));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_pages_editor_by_visible_rows() {
        let widget = crate::WidgetId::new(323);
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let builder = crate::native_window("Win32 paged editor viewport")
            .size(160, 70)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>(value)
                        .id(widget)
                        .text_wrap(crate::TextWrap::NoWrap)
                        .width(crate::Dp::new(160.0))
                        .height(crate::Dp::new(70.0))
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("paged editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("paged editor should expose Win32 viewport geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(
            crate::Point {
                x: target.bounds.x + 16,
                y: target.bounds.y + 10,
            },
            false,
        );
        route.dispatch_pointer_up(crate::Point {
            x: target.bounds.x + 16,
            y: target.bounds.y + 10,
        });

        let page_down = route.dispatch_key_down(u32::from(VK_NEXT));
        let page_plan = route
            .take_pending_draw_plan()
            .expect("PageDown should rebuild the paged editor viewport");
        let shift_page_down = route.dispatch_key_down_with_shift(u32::from(VK_NEXT), true);
        let page_up = route.dispatch_key_down(u32::from(VK_PRIOR));

        assert_eq!(page_down.text_caret, Some(10));
        assert!(page_plan.commands.iter().any(
            |command| matches!(command, crate::NativeDrawCommand::Text(text) if text.text == "d3"),
        ));
        assert_eq!(shift_page_down.text_caret, Some(19));
        assert_eq!(page_up.text_caret, Some(10));
        assert_eq!(
            route.text_edit.map(|state| state.selection),
            Some(crate::native_text_edit::NativeTextSelection::collapsed(10))
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_assembles_utf16_surrogate_pairs_before_dispatch() {
        let widget = crate::WidgetId::new(318);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox::<UiCommand>("").id(widget),
        );
        route.dispatch_click(crate::Point { x: 10, y: 20 });
        let units = "👩".encode_utf16().collect::<Vec<_>>();

        let high = route.dispatch_utf16_input_unit(units[0]);
        let low = route.dispatch_utf16_input_unit(units[1]);

        assert!(high.handled);
        assert_eq!(high.text_input_count, 0);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("👩"));
        assert_eq!(low.text_input_count, 1);
        assert_eq!(low.text_caret, Some(1));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_live_view_routes_typed_undo_command_to_focused_editor() {
        #[derive(Clone)]
        enum Msg {
            Changed(String),
            Undo,
        }

        let widget = crate::WidgetId::new(34);
        let builder = crate::native_window("Win32 typed editor command")
            .size(320, 160)
            .stateful_view_with_app_commands(
                String::new(),
                move |value| crate::text_editor(value).id(widget).on_change(Msg::Changed),
                |value, message, cx| match message {
                    Msg::Changed(next) => *value = next,
                    Msg::Undo => cx.text_edit_command(crate::ZsTextEditCommand::Undo),
                },
                |command| (command == &Command::custom("edit.undo")).then_some(Msg::Undo),
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("stateful editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("editor should expose Win32 focus geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_click(crate::Point {
            x: target.bounds.x + 8,
            y: target.bounds.y + 8,
        });
        route.dispatch_text_input("A");
        route.dispatch_text_input("中");

        let undone = route.dispatch_app_command(Command::custom("edit.undo"));

        assert_eq!(undone.text_edit_command_count, 1);
        assert_eq!(undone.text_undo_count, 1);
        assert!(undone.text_edit_command_errors.is_empty());
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A"));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_captures_unicode_pointer_drag_selection() {
        let widget = crate::WidgetId::new(33);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("A中文Z").id(widget),
        );

        let pressed = route.dispatch_pointer_down(crate::Point { x: 16, y: 12 }, false);
        let shaped = crate::windows_gdi_renderer::shape_windows_gdi_text_line("A中文Z")
            .expect("Windows text geometry should use the same shaped line as drawing");
        let after_chinese = shaped
            .carets
            .iter()
            .find(|caret| caret.index == 3)
            .expect("the shaped line should expose the grapheme boundary after 中文")
            .primary_x;
        let drag_point = crate::Point {
            x: 8 + after_chinese,
            y: 12,
        };
        let dragged = route.dispatch_pointer_move(drag_point);
        let released = route.dispatch_pointer_up(drag_point);

        assert!(pressed.handled);
        assert_eq!(pressed.pointer_down_count, 1);
        assert_eq!(pressed.text_caret, Some(1));
        assert!(pressed.text_drag_active);
        assert_eq!(dragged.pointer_move_count, 1);
        assert_eq!(dragged.text_caret, Some(3));
        assert_eq!(dragged.text_selection_change_count, 1);
        assert!(dragged.text_drag_active);
        assert_eq!(released.pointer_up_count, 1);
        assert_eq!(released.text_drag_count, 1);
        assert!(!released.text_drag_active);

        let replaced = route.dispatch_text_input("🙂");

        assert_eq!(replaced.text_caret, Some(2));
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("A🙂Z"));
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_scrolls_editor_during_captured_edge_drag() {
        let widget = crate::WidgetId::new(324);
        let value = "a0\nb1\nc2\nd3\ne4\nf5\ng6";
        let builder = crate::native_window("Win32 editor edge drag")
            .size(160, 70)
            .stateful_view(
                (),
                move |_| {
                    crate::text_editor::<UiCommand>(value)
                        .id(widget)
                        .text_wrap(crate::TextWrap::NoWrap)
                },
                |_, _, _| {},
            );
        let runtime = builder
            .native_live_view_runtime()
            .expect("edge-drag editor should own a live runtime")
            .clone();
        let target = runtime
            .interaction_plan()
            .hit_target_for_widget(widget)
            .expect("edge-drag editor should expose Win32 geometry");
        let mut route = WindowsWin32ViewInputRoute::from_live_view(runtime);
        route.dispatch_pointer_down(
            crate::Point {
                x: target.bounds.x + 16,
                y: target.bounds.y + 10,
            },
            false,
        );
        let outside = crate::Point {
            x: target.bounds.x + 16,
            y: target.bounds.y + target.bounds.height + 40,
        };

        let first = route.dispatch_pointer_move(outside);
        let second = route.dispatch_pointer_move(outside);

        assert_eq!(first.text_drag_scroll_count, 1);
        assert_eq!(first.text_caret, Some(10));
        assert_eq!(second.text_drag_scroll_count, 1);
        assert_eq!(second.text_caret, Some(13));
        assert!(second
            .events
            .iter()
            .any(|event| event.starts_with("win32_view_text_drag_scroll:324:")));
    }

    #[test]
    #[cfg(feature = "slider")]
    fn window_view_input_route_drags_and_steps_slider() {
        fn changed(_: f32) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.slider_changed"))
        }

        let widget = crate::WidgetId::new(34);
        let range = crate::SliderRange::new(0.0, 100.0).step(5.0);
        let target = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            crate::ViewHitTargetKind::Slider,
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([target]),
            crate::slider(0.0, range).id(widget).on_slide(changed),
        );
        let track = crate::zs_slider_render_plan(target.bounds, 0.0, crate::Dpi::standard()).track;

        let pressed = route.dispatch_pointer_down(
            crate::Point {
                x: track.x + track.width / 4,
                y: 16,
            },
            false,
        );
        let dragged = route.dispatch_pointer_move(crate::Point {
            x: track.x + track.width * 3 / 4,
            y: 16,
        });
        let released = route.dispatch_pointer_up(crate::Point {
            x: track.x + track.width * 3 / 4,
            y: 16,
        });
        let stepped = route.dispatch_key_down(u32::from(VK_LEFT));

        assert!(pressed.handled);
        assert_eq!(pressed.slider_value_change_count, 1);
        assert!(pressed.slider_drag_active);
        assert_eq!(dragged.slider_value_change_count, 1);
        assert_eq!(dragged.pointer_move_count, 1);
        assert_eq!(released.slider_drag_count, 1);
        assert!(!released.slider_drag_active);
        assert_eq!(stepped.slider_keyboard_change_count, 1);
        assert_eq!(stepped.slider_value_change_count, 1);
        assert_eq!(route.widget_slider_state(widget), Some((70.0, range)));
        assert_eq!(route.pending_ui_commands.len(), 3);
    }

    #[test]
    #[cfg(feature = "color-picker")]
    fn window_view_input_route_drags_and_keys_color_picker_channels() {
        fn color_changed(_: crate::Color) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_changed"))
        }
        fn channel_changed(_: crate::ZsColorChannel) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_channel"))
        }
        fn expanded_changed(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.color_picker_expanded"))
        }

        let widget = crate::WidgetId::new(341);
        let viewport = crate::Rect {
            x: 0,
            y: 0,
            width: 480,
            height: 680,
        };
        let state = crate::ZsColorPickerState::new(crate::Color::rgba(32, 96, 160, 224))
            .with_expanded(true);
        let mut view = crate::column([
            crate::color_picker(state)
                .id(widget)
                .height(crate::Dp::new(32.0))
                .on_color_change(color_changed)
                .on_color_channel_change(channel_changed)
                .on_expanded_change(expanded_changed),
            crate::spacer(),
        ])
        .padding(crate::Dp::new(24.0))
        .gap(crate::Dp::new(12.0));
        view.layout(&mut crate::ViewLayoutCx::new(
            viewport,
            crate::Dpi::standard(),
        ));
        let plan = view.interaction_plan();
        let root = plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.widget == widget && target.kind == crate::ViewHitTargetKind::ColorPicker
            })
            .expect("color picker root");
        let render = crate::zs_color_picker_render_plan_in_viewport(
            root.bounds,
            state,
            crate::ZsColorPickerPlatformStyle::Windows,
            crate::Dpi::standard(),
            viewport,
        );
        let red = render
            .channels
            .iter()
            .find(|row| row.channel == crate::ZsColorChannel::Red)
            .expect("red row");
        let mut route = WindowsWin32ViewInputRoute::new(plan, view);

        let pressed = route.dispatch_pointer_down(
            crate::Point {
                x: red.track.x + red.track.width / 4,
                y: red.track.y + red.track.height / 2,
            },
            false,
        );
        let dragged = route.dispatch_pointer_move(crate::Point {
            x: red.track.x + red.track.width * 9 / 10,
            y: red.track.y + red.track.height / 2,
        });
        let released = route.dispatch_pointer_up(crate::Point {
            x: red.track.x + red.track.width * 9 / 10,
            y: red.track.y + red.track.height / 2,
        });

        assert!(pressed.handled);
        assert!(pressed.color_picker_drag_active);
        assert_eq!(pressed.color_picker_value_change_count, 1);
        assert!(dragged.handled);
        assert!(dragged.color_picker_drag_active);
        assert_eq!(dragged.color_picker_value_change_count, 1);
        assert_eq!(released.color_picker_drag_count, 1);
        assert!(!released.color_picker_drag_active);
        assert!(route
            .widget_color_picker_state(widget)
            .is_some_and(|state| state.color.r > 220));

        let channel = route.dispatch_key_down(u32::from(VK_DOWN));
        let maximum = route.dispatch_key_down(u32::from(VK_END));
        let closed = route.dispatch_key_down(u32::from(VK_ESCAPE));
        let reopened = route.dispatch_key_down(ZSUI_WIN32_VK_SPACE);

        assert_eq!(channel.color_picker_channel_change_count, 1);
        assert_eq!(maximum.color_picker_value_change_count, 1);
        assert_eq!(closed.color_picker_expanded_change_count, 1);
        assert_eq!(reopened.color_picker_expanded_change_count, 1);
        assert!(route
            .widget_color_picker_state(widget)
            .is_some_and(|state| {
                state.active_channel == crate::ZsColorChannel::Green
                    && state.color.g == 255
                    && state.expanded
            }));
        assert_eq!(route.pending_ui_commands.len(), 6);
    }

    #[test]
    #[cfg(feature = "number-box")]
    fn window_view_input_route_edits_commits_and_steps_number_box() {
        fn changed(_: Option<f64>) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.number_box_changed"))
        }

        let widget = crate::WidgetId::new(340);
        let bounds = crate::Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 36,
        };
        let render = crate::zs_number_box_render_plan(
            bounds,
            crate::ZsNumberBoxPlatformStyle::Windows,
            crate::Dpi::standard(),
        );
        let plan = crate::ViewInteractionPlan::new([
            crate::ViewHitTarget::with_kind(widget, bounds, crate::ViewHitTargetKind::NumberBox),
            crate::ViewHitTarget::with_kind(
                widget,
                render.decrement_button,
                crate::ViewHitTargetKind::NumberBoxDecrement,
            ),
            crate::ViewHitTarget::with_kind(
                widget,
                render.increment_button,
                crate::ViewHitTargetKind::NumberBoxIncrement,
            ),
        ]);
        let range = crate::ZsNumberRange::new(0.0, 10.0)
            .step(0.5)
            .large_step(5.0);
        let mut route = WindowsWin32ViewInputRoute::new(
            plan,
            crate::number_box(Some(2.5), range)
                .id(widget)
                .fraction_digits(1)
                .on_number_change(changed),
        );

        let incremented = route.dispatch_click(crate::Point {
            x: render.increment_button.x + render.increment_button.width / 2,
            y: render.increment_button.y + render.increment_button.height / 2,
        });
        let stepped = route.dispatch_key_down(u32::from(VK_UP));
        let cleared = route.dispatch_text_input("\u{8}\u{8}\u{8}");
        let typed = route.dispatch_text_input("9.5");
        let committed = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);

        assert!(incremented.handled);
        assert!(stepped.handled);
        assert_eq!(cleared.text_input_count, 3);
        assert_eq!(typed.text_input_count, 3);
        assert!(committed.handled);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("9.5"));
        assert_eq!(route.pending_ui_commands.len(), 3);
    }

    #[test]
    #[cfg(feature = "radio")]
    fn window_view_input_route_selects_radio_from_pointer_and_space() {
        let first = crate::WidgetId::new(35);
        let second = crate::WidgetId::new(36);
        let selected = UiCommand::app(crate::CommandId("zsui.test.win32.radio_selected"));
        let mut view = crate::column([
            crate::radio_button("Balanced", true)
                .id(first)
                .height(crate::Dp::new(36.0))
                .on_choose(selected.clone()),
            crate::radio_button("Performance", false)
                .id(second)
                .height(crate::Dp::new(36.0))
                .on_choose(selected),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 72,
            },
            crate::Dpi::standard(),
        ));
        let interaction_plan = view.interaction_plan();
        let second_bounds = interaction_plan
            .hit_target_for_widget(second)
            .expect("second radio should have hit geometry")
            .bounds;
        let mut route = WindowsWin32ViewInputRoute::new(interaction_plan, view);

        let pointer = route.dispatch_click(crate::Point {
            x: second_bounds.x + 10,
            y: second_bounds.y + second_bounds.height / 2,
        });
        let keyboard = route.dispatch_key_down(u32::from(VK_SPACE));
        let arrow = route.dispatch_key_down(u32::from(VK_UP));
        let focus_only = route.dispatch_key_down_with_modifiers(u32::from(VK_DOWN), false, true);
        let tabbed = route.dispatch_key_down(u32::from(VK_TAB));

        assert_eq!(pointer.radio_selection_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(keyboard.radio_selection_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
        assert_eq!(arrow.radio_selection_count, 1);
        assert_eq!(arrow.radio_keyboard_selection_count, 1);
        assert_eq!(arrow.focused_widget, Some(first.0));
        assert!(arrow
            .events
            .iter()
            .any(|event| event == "win32_view_radio_key_select:36:35"));
        assert_eq!(focus_only.radio_keyboard_focus_only_count, 1);
        assert_eq!(focus_only.radio_selection_count, 0);
        assert_eq!(focus_only.focused_widget, Some(second.0));
        assert!(focus_only
            .events
            .iter()
            .any(|event| event == "win32_view_radio_key_focus_only:35:36"));
        assert_eq!(tabbed.focus_traversal_count, 1);
        assert_eq!(tabbed.focused_widget, Some(first.0));
        assert_eq!(route.widget_checked_value(first), Some(true));
        assert_eq!(route.widget_checked_value(second), Some(false));
    }

    #[test]
    #[cfg(all(feature = "tabs", feature = "label"))]
    fn window_view_input_route_routes_tab_pointer_focus_activation_and_ctrl_tab() {
        fn selected(_: crate::ZsTabId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.tabs.selected"))
        }

        let tab_view_id = crate::WidgetId::new(340);
        let general = crate::ZsTabId::new(341);
        let advanced = crate::ZsTabId::new(342);
        let mut view = crate::tab_view(
            [
                crate::ZsTabItem::new(general, "General", crate::text("General content")),
                crate::ZsTabItem::new(advanced, "Advanced", crate::text("Advanced content")),
            ],
            Some(general),
        )
        .id(tab_view_id)
        .on_tab_select(selected);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let second = interaction
            .hit_target_for_widget(crate::WidgetId(advanced.0))
            .expect("second tab should expose a hit target");
        let second_point = crate::Point {
            x: second.bounds.x + second.bounds.width / 2,
            y: second.bounds.y + second.bounds.height / 2,
        };
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let hovered = route.dispatch_pointer_move(second_point);
        let pressed = route.dispatch_pointer_down(second_point, false);
        let pointer = route.dispatch_pointer_up(second_point);

        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert_eq!(pressed.pointer_visual_change_count, 1);
        assert_eq!(pointer.tab_selection_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(pointer.focused_widget, Some(advanced.0));
        assert_eq!(
            route
                .ui_command_view
                .as_ref()
                .and_then(|view| view.widget_tab_view_state(tab_view_id))
                .and_then(|state| state.selected),
            Some(advanced)
        );

        let focus_only = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(focus_only.tab_keyboard_focus_only_count, 1);
        assert_eq!(focus_only.tab_selection_count, 0);
        assert_eq!(focus_only.focused_widget, Some(general.0));

        let keyboard = route.dispatch_key_down(u32::from(VK_SPACE));
        assert_eq!(keyboard.tab_selection_count, 1);
        assert_eq!(keyboard.tab_keyboard_selection_count, 1);
        assert_eq!(
            route
                .ui_command_view
                .as_ref()
                .and_then(|view| view.widget_tab_view_state(tab_view_id))
                .and_then(|state| state.selected),
            Some(general)
        );

        let cycled = route.dispatch_key_down_with_modifiers(u32::from(VK_TAB), false, true);
        assert_eq!(cycled.tab_selection_count, 1);
        assert_eq!(cycled.tab_keyboard_selection_count, 1);
        assert_eq!(cycled.focused_widget, Some(advanced.0));
        assert!(route.take_pending_draw_plan().is_some());
    }

    #[test]
    #[cfg(feature = "auto-suggest")]
    fn window_view_input_route_closes_auto_suggest_with_pointer_and_keyboard_submission() {
        fn text(_change: crate::ZsAutoSuggestTextChange) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_text"))
        }
        fn chosen(_suggestion: crate::ZsAutoSuggestionId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_chosen"))
        }
        fn submitted(_submission: crate::ZsAutoSuggestSubmission) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_submitted"))
        }
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.auto_suggest_expanded"))
        }

        let widget = crate::WidgetId::new(136);
        let beta = crate::ZsAutoSuggestionId::new(2);
        let mut view = crate::column([
            crate::auto_suggest_box(
                "B",
                [
                    crate::ZsAutoSuggestion::new(1_u64, "Alpha"),
                    crate::ZsAutoSuggestion::new(beta, "Beta"),
                    crate::ZsAutoSuggestion::new(3_u64, "Bravo"),
                ],
            )
            .id(widget)
            .expanded(true)
            .on_auto_suggest_text_change(text)
            .on_suggestion_chosen(chosen)
            .on_query_submit(submitted)
            .on_expanded_change(expanded),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let suggestion = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind == crate::ViewHitTargetKind::AutoSuggestSuggestion { suggestion: beta }
            })
            .expect("expanded auto-suggest should expose Beta row");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let pointer = route.dispatch_click(crate::Point {
            x: suggestion.bounds.x + 8,
            y: suggestion.bounds.y + suggestion.bounds.height / 2,
        });
        assert_eq!(pointer.auto_suggest_submit_count, 1);
        assert_eq!(pointer.auto_suggest_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 4);
        assert!(route
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.query == "Beta" && !state.expanded));

        let typed = route.dispatch_text_input("x");
        assert_eq!(typed.text_input_count, 1);
        assert_eq!(typed.auto_suggest_expanded_change_count, 1);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some("Betax"));
        let highlighted = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(highlighted.auto_suggest_highlight_change_count, 1);
        assert_eq!(
            route
                .widget_auto_suggest_state(widget)
                .and_then(|state| state.highlighted),
            Some(1_u64.into())
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.auto_suggest_submit_count, 1);
        assert_eq!(keyboard.auto_suggest_expanded_change_count, 1);
        assert!(route
            .widget_auto_suggest_state(widget)
            .is_some_and(|state| state.query == "Alpha" && !state.expanded));

        let clear = route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::AutoSuggestClear)
            .expect("non-empty query should expose clear button");
        let cleared = route.dispatch_click(crate::Point {
            x: clear.bounds.x + clear.bounds.width / 2,
            y: clear.bounds.y + clear.bounds.height / 2,
        });
        assert_eq!(cleared.auto_suggest_clear_count, 1);
        assert_eq!(route.widget_text_value(widget).as_deref(), Some(""));
    }

    #[test]
    #[cfg(feature = "command-palette")]
    fn window_view_input_route_filters_navigates_and_invokes_command_palette() {
        fn query(_query: String) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_query"))
        }
        fn highlight(_item: crate::ZsCommandPaletteItemId) -> UiCommand {
            UiCommand::app(crate::CommandId(
                "zsui.test.win32.command_palette_highlight",
            ))
        }
        fn invoke(_item: crate::ZsCommandPaletteItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_invoke"))
        }
        fn open(_open: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.command_palette_open"))
        }

        let widget = crate::WidgetId::new(341);
        let first = crate::ZsCommandPaletteItemId::new(1);
        let settings = crate::ZsCommandPaletteItemId::new(2);
        let mut view = crate::command_palette(
            widget,
            true,
            "",
            [
                crate::ZsCommandPaletteItem::new(first, "Open file"),
                crate::ZsCommandPaletteItem::new(settings, "Open settings")
                    .keywords(["preferences"]),
                crate::ZsCommandPaletteItem::new(3_u64, "Unavailable").enabled(false),
            ],
            crate::spacer(),
        )
        .highlighted_command(Some(first))
        .on_command_palette_query_change(query)
        .on_command_palette_highlight_change(highlight)
        .on_command_palette_invoke(invoke)
        .on_command_palette_open_change(open);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 900,
                height: 620,
            },
            crate::Dpi::standard(),
        ));
        let mut route = WindowsWin32ViewInputRoute::new(view.interaction_plan(), view);

        let moved = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(moved.command_palette_highlight_change_count, 1);
        assert_eq!(
            route
                .widget_command_palette_state(widget)
                .and_then(|state| state.highlighted),
            Some(settings)
        );

        let typed = route.dispatch_text_input("settings");
        assert_eq!(typed.command_palette_query_change_count, 1);
        assert!(route
            .widget_command_palette_state(widget)
            .is_some_and(
                |state| state.query == "settings" && state.visible_items == vec![settings]
            ));

        let invoked = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(invoked.command_palette_invoke_count, 1);
        assert_eq!(invoked.command_palette_open_change_count, 1);
        assert!(route
            .widget_command_palette_state(widget)
            .is_some_and(|state| !state.open));
        assert!(invoked
            .events
            .iter()
            .any(|event| event == "win32_view_command_palette_invoke:341:2"));
    }

    #[test]
    #[cfg(feature = "tree")]
    fn window_view_input_route_handles_tree_disclosure_rows_and_keyboard_hierarchy() {
        fn selected(_node: crate::ZsTreeNodeId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_selected"))
        }
        fn expanded(_change: crate::ZsTreeExpansionChange) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_expanded"))
        }
        fn invoked(_node: crate::ZsTreeNodeId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.tree_invoked"))
        }

        let widget = crate::WidgetId::new(137);
        let root = crate::ZsTreeNodeId::new(1);
        let folder = crate::ZsTreeNodeId::new(2);
        let leaf = crate::ZsTreeNodeId::new(3);
        let mut view = crate::tree_view([crate::ZsTreeNode::new(root, "Workspace")
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
        .on_tree_select(selected)
        .on_tree_expansion_change(expanded)
        .on_tree_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let disclosure = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind == crate::ViewHitTargetKind::TreeNodeExpander { node: folder }
            })
            .expect("folder should expose a disclosure target");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let opened = route.dispatch_click(crate::Point {
            x: disclosure.bounds.x + disclosure.bounds.width / 2,
            y: disclosure.bounds.y + disclosure.bounds.height / 2,
        });
        assert_eq!(opened.tree_expansion_change_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        let leaf_row = route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TreeNode { node: leaf })
            .expect("expanded folder should expose leaf row");
        let pointer = route.dispatch_click(crate::Point {
            x: leaf_row.bounds.x + leaf_row.bounds.width / 2,
            y: leaf_row.bounds.y + leaf_row.bounds.height / 2,
        });
        assert_eq!(pointer.tree_selection_count, 1);
        assert_eq!(pointer.tree_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let parent = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(parent.tree_selection_count, 1);
        assert_eq!(
            route
                .widget_tree_view_state(widget)
                .and_then(|state| state.selected),
            Some(folder)
        );
        let collapsed = route.dispatch_key_down(u32::from(VK_LEFT));
        assert_eq!(collapsed.tree_expansion_change_count, 1);
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.tree_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "grid-view")]
    fn window_view_input_route_handles_grid_view_tiles_and_two_axis_keyboard_navigation() {
        fn selected(_item: crate::ZsGridViewItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.grid_view_selected"))
        }
        fn invoked(_item: crate::ZsGridViewItemId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.grid_view_invoked"))
        }

        let widget = crate::WidgetId::new(151);
        let first = crate::ZsGridViewItemId::new(1);
        let fifth = crate::ZsGridViewItemId::new(5);
        let mut view = crate::grid_view([
            crate::ZsGridViewItem::new(1, "One"),
            crate::ZsGridViewItem::new(2, "Two"),
            crate::ZsGridViewItem::new(3, "Three"),
            crate::ZsGridViewItem::new(4, "Four"),
            crate::ZsGridViewItem::new(5, "Five"),
            crate::ZsGridViewItem::new(6, "Six"),
        ])
        .id(widget)
        .selected_grid_view_item(Some(first))
        .on_grid_view_select(selected)
        .on_grid_view_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let fifth_tile = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::GridViewItem { item: fifth })
            .expect("fifth grid-view tile");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let pointer = route.dispatch_click(crate::Point {
            x: fifth_tile.bounds.x + fifth_tile.bounds.width / 2,
            y: fifth_tile.bounds.y + fifth_tile.bounds.height / 2,
        });
        assert_eq!(pointer.grid_view_selection_count, 1);
        assert_eq!(pointer.grid_view_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_HOME))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_RIGHT))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .dispatch_key_down(u32::from(VK_DOWN))
                .grid_view_selection_count,
            1
        );
        assert_eq!(
            route
                .widget_grid_view_state(widget)
                .and_then(|state| state.selected),
            Some(fifth)
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.grid_view_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "table")]
    fn window_view_input_route_handles_table_sort_rows_and_keyboard_navigation() {
        fn selected(_row: crate::ZsTableRowId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_selected"))
        }
        fn sorted(_sort: crate::ZsTableSort) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_sorted"))
        }
        fn invoked(_row: crate::ZsTableRowId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.table_invoked"))
        }

        let widget = crate::WidgetId::new(138);
        let name = crate::ZsTableColumnId::new(1);
        let first = crate::ZsTableRowId::new(10);
        let second = crate::ZsTableRowId::new(11);
        let mut view = crate::data_grid(
            [
                crate::ZsTableColumn::new(name, "Name").sortable(true),
                crate::ZsTableColumn::new(2, "Size").fixed_width(crate::Dp::new(80.0)),
            ],
            [
                crate::ZsTableRow::new(first, ["Cargo.toml", "4 KB"]),
                crate::ZsTableRow::new(second, ["src", "—"]),
            ],
        )
        .id(widget)
        .selected_table_row(Some(first))
        .on_table_select(selected)
        .on_table_sort(sorted)
        .on_table_invoke(invoked);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let header = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableHeader { column: name })
            .expect("sortable table header");
        let second_row = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TableRow { row: second })
            .expect("second table row");
        let mut route = WindowsWin32ViewInputRoute::new(interaction, view);

        let sort = route.dispatch_click(crate::Point {
            x: header.bounds.x + header.bounds.width / 2,
            y: header.bounds.y + header.bounds.height / 2,
        });
        assert_eq!(sort.table_sort_count, 1);
        assert_eq!(sort.ui_command_count, 1);
        assert_eq!(
            route
                .widget_table_state(widget)
                .and_then(|state| state.sort),
            Some(crate::ZsTableSort::new(
                name,
                crate::ZsTableSortDirection::Ascending
            ))
        );

        let pointer = route.dispatch_click(crate::Point {
            x: second_row.bounds.x + second_row.bounds.width / 2,
            y: second_row.bounds.y + second_row.bounds.height / 2,
        });
        assert_eq!(pointer.table_selection_count, 1);
        assert_eq!(pointer.table_invoke_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let moved = route.dispatch_key_down(u32::from(VK_UP));
        assert_eq!(moved.table_selection_count, 1);
        assert_eq!(
            route
                .widget_table_state(widget)
                .and_then(|state| state.selected),
            Some(first)
        );
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.table_invoke_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
    }

    #[test]
    #[cfg(feature = "dialog")]
    fn window_view_input_route_traps_modal_focus_and_routes_dialog_buttons() {
        fn responded(_result: crate::ZsContentDialogResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.dialog_responded"))
        }

        let widget = crate::WidgetId::new(139);
        let spec = crate::ZsContentDialogSpec::new("Choose a response.", "Cancel")
            .title("Continue?")
            .primary_button("Continue")
            .secondary_button("Review")
            .default_button(crate::ZsContentDialogButton::Primary);
        let mut view =
            crate::content_dialog(widget, true, spec, crate::spacer()).on_dialog_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 400,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let primary = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                target.kind
                    == crate::ViewHitTargetKind::ContentDialogButton {
                        button: crate::ZsContentDialogButton::Primary,
                    }
            })
            .expect("primary dialog button");
        let scrim = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ContentDialogScrim)
            .expect("dialog scrim");

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let caught = keyboard_route.dispatch_click(crate::Point {
            x: scrim.bounds.x + 2,
            y: scrim.bounds.y + 2,
        });
        assert!(caught.handled);
        assert_eq!(caught.content_dialog_response_count, 0);
        let suppressed = keyboard_route.dispatch_text_input("x");
        assert!(suppressed.handled);
        assert_eq!(suppressed.ui_command_count, 0);
        let focused = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focused.content_dialog_focus_change_count, 1);
        assert_eq!(focused.focused_widget, Some(widget.0));
        assert_eq!(
            keyboard_route
                .widget_content_dialog_state(widget)
                .map(|(state, _)| state.focused_button),
            Some(crate::ZsContentDialogButton::Secondary)
        );
        let keyboard = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(keyboard.content_dialog_response_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert!(keyboard_route
            .widget_content_dialog_state(widget)
            .is_some_and(|(state, _)| !state.open));

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: primary.bounds.x + primary.bounds.width / 2,
            y: primary.bounds.y + primary.bounds.height / 2,
        });
        assert_eq!(pointer.content_dialog_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "toast")]
    fn window_view_input_route_routes_toast_action_and_owned_timeout() {
        fn responded(_result: crate::ZsToastResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.toast_responded"))
        }

        let widget = crate::WidgetId::new(149);
        let mut view = crate::toast_presenter(
            widget,
            Some(crate::ZsToastSpec::new(51, "File deleted").action("Undo")),
            crate::spacer(),
        )
        .on_toast_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 400,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::ToastAction)
            .expect("toast action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.toast_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert!(pointer_route.widget_toast_state(widget).is_none());

        let start = std::time::Instant::now();
        let mut timeout_route = WindowsWin32ViewInputRoute::new(interaction, view);
        assert!(timeout_route.background_poll_interval_ms().is_some());
        let timeout =
            timeout_route.refresh_background_view_at(start + std::time::Duration::from_secs(6));
        assert_eq!(timeout.toast_response_count, 1);
        assert_eq!(timeout.toast_timeout_count, 1);
        assert_eq!(timeout.ui_command_count, 1);
        assert!(timeout_route.widget_toast_state(widget).is_none());
    }

    #[test]
    #[cfg(feature = "info-bar")]
    fn window_view_input_route_routes_info_bar_action_and_keyboard_close() {
        fn invoked(event: crate::ZsInfoBarEvent) -> UiCommand {
            UiCommand::app(crate::CommandId(match event {
                crate::ZsInfoBarEvent::Action => "zsui.test.win32.info_bar_action",
                crate::ZsInfoBarEvent::Close => "zsui.test.win32.info_bar_close",
            }))
        }

        let widget = crate::WidgetId::new(150);
        let mut view = crate::column([
            crate::info_bar(
                widget,
                crate::ZsInfoBarSpec::new("Renew to keep all functionality.")
                    .title("Subscription expires soon")
                    .severity(crate::ZsInfoBarSeverity::Warning)
                    .action("Renew"),
            )
            .on_info_bar_event(invoked),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 240,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::InfoBarAction)
            .expect("info-bar action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.info_bar_event_count, 1);
        assert_eq!(pointer.ui_command_count, 1);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let next = keyboard_route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(next.info_bar_focus_change_count, 1);
        assert_eq!(
            keyboard_route
                .widget_info_bar_state(widget)
                .map(|(state, _)| state.focused_control),
            Some(Some(crate::ZsInfoBarControl::Close))
        );
        let close = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(close.info_bar_event_count, 1);
        assert_eq!(close.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "teaching-tip")]
    fn window_view_input_route_routes_teaching_tip_action_and_keyboard_close() {
        fn responded(_result: crate::ZsTeachingTipResult) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.teaching_tip_responded"))
        }

        let widget = crate::WidgetId::new(151);
        let target = crate::WidgetId::new(152);
        let mut view = crate::teaching_tip(
            widget,
            true,
            target,
            crate::ZsTeachingTipSpec::new(
                "Save automatically",
                "Your changes are saved as you work.",
            )
            .action("Review settings"),
            crate::spacer().id(target),
        )
        .on_teaching_tip_result(responded);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 420,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let action = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::TeachingTipAction)
            .expect("teaching-tip action");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: action.bounds.x + action.bounds.width / 2,
            y: action.bounds.y + action.bounds.height / 2,
        });
        assert_eq!(pointer.teaching_tip_response_count, 1);
        assert_eq!(pointer.ui_command_count, 1);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(target.0));
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let next = keyboard_route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(next.teaching_tip_focus_change_count, 1);
        assert_eq!(
            keyboard_route
                .widget_teaching_tip_state(widget)
                .map(|(state, _)| state.focused_control),
            Some(crate::ZsTeachingTipControl::Close)
        );
        let close = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(close.teaching_tip_response_count, 1);
        assert_eq!(close.ui_command_count, 1);
    }

    #[test]
    #[cfg(feature = "breadcrumb")]
    fn window_view_input_route_routes_breadcrumb_overflow_focus_and_selection() {
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.breadcrumb_expanded"))
        }
        fn selected(_item: crate::ZsBreadcrumbId) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.breadcrumb_selected"))
        }

        let widget = crate::WidgetId::new(153);
        let mut view = crate::breadcrumb_bar([
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(1), "Home"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(2), "Projects"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(3), "ZSUI Framework"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(4), "Documentation"),
            crate::ZsBreadcrumbItem::new(crate::ZsBreadcrumbId::new(5), "BreadcrumbBar"),
        ])
        .id(widget)
        .width(crate::Dp::new(240.0))
        .expanded(false)
        .on_expanded_change(expanded)
        .on_breadcrumb_select(selected);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let overflow = interaction
            .hit_targets
            .iter()
            .copied()
            .find(|target| target.kind == crate::ViewHitTargetKind::BreadcrumbOverflow)
            .expect("narrow breadcrumb overflow");

        let mut pointer_route = WindowsWin32ViewInputRoute::new(interaction.clone(), view.clone());
        let open = pointer_route.dispatch_click(crate::Point {
            x: overflow.bounds.x + overflow.bounds.width / 2,
            y: overflow.bounds.y + overflow.bounds.height / 2,
        });
        assert_eq!(open.breadcrumb_expanded_change_count, 1);
        assert_eq!(open.ui_command_count, 1);
        let row = pointer_route
            .interaction_plan
            .hit_targets
            .iter()
            .copied()
            .find(|target| {
                matches!(
                    target.kind,
                    crate::ViewHitTargetKind::BreadcrumbOverflowItem { .. }
                )
            })
            .expect("open overflow row");
        let pointer = pointer_route.dispatch_click(crate::Point {
            x: row.bounds.x + row.bounds.width / 2,
            y: row.bounds.y + row.bounds.height / 2,
        });
        assert_eq!(pointer.breadcrumb_selection_count, 1);
        assert_eq!(pointer.breadcrumb_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 2);

        let mut keyboard_route = WindowsWin32ViewInputRoute::new(interaction, view);
        let focus = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_TAB);
        assert_eq!(focus.focused_widget, Some(widget.0));
        let home = keyboard_route.dispatch_key_down(u32::from(VK_HOME));
        assert_eq!(home.breadcrumb_focus_change_count, 1);
        let open = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(open.breadcrumb_expanded_change_count, 1);
        let down = keyboard_route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(down.breadcrumb_focus_change_count, 1);
        let selection = keyboard_route.dispatch_key_down(ZSUI_WIN32_VK_RETURN);
        assert_eq!(selection.breadcrumb_selection_count, 1);
        assert_eq!(selection.breadcrumb_expanded_change_count, 1);
        assert_eq!(selection.ui_command_count, 2);
    }

    #[test]
    #[cfg(feature = "combo")]
    fn window_view_input_route_selects_combo_overlay_and_keyboard_option() {
        fn selected(_index: usize) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.combo_selected"))
        }
        fn expanded(_expanded: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.combo_expanded"))
        }

        let widget = crate::WidgetId::new(36);
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 36,
            },
            crate::ViewHitTargetKind::ComboBox,
        );
        let option = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 76,
                width: 200,
                height: 36,
            },
            crate::ViewHitTargetKind::ComboBoxOption { index: 1 },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, option]),
            crate::combo_box(["Balanced", "Fast", "Quiet"], Some(0))
                .id(widget)
                .expanded(true)
                .on_select(selected)
                .on_expanded_change(expanded),
        );

        let pointer = route.dispatch_click(crate::Point { x: 10, y: 90 });
        assert_eq!(pointer.combo_selection_count, 1);
        assert_eq!(pointer.combo_expanded_change_count, 1);
        assert_eq!(pointer.ui_command_count, 2);
        assert_eq!(route.widget_combo_state(widget), Some((Some(1), 3, false)));

        let opened = route.dispatch_key_down(u32::from(VK_SPACE));
        assert_eq!(opened.combo_expanded_change_count, 1);
        assert_eq!(opened.keyboard_activation_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(1), 3, true)));

        let keyboard = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(keyboard.combo_selection_count, 1);
        assert_eq!(keyboard.combo_keyboard_selection_count, 1);
        assert_eq!(keyboard.combo_expanded_change_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(2), 3, false)));

        let typed = route.dispatch_text_input("B");
        assert!(typed.handled);
        assert_eq!(typed.combo_type_ahead_match_count, 1);
        assert_eq!(typed.combo_selection_count, 1);
        assert_eq!(typed.combo_keyboard_selection_count, 1);
        assert_eq!(typed.ui_command_count, 1);
        assert!(typed
            .events
            .iter()
            .any(|event| event == "win32_view_combo_type_ahead_match:36:b:0"));
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));

        route.dispatch_key_down(u32::from(VK_SPACE));
        let outside = route.dispatch_pointer_down(crate::Point { x: 260, y: 200 }, false);
        assert!(outside.handled);
        assert_eq!(outside.event_count, 1);
        assert_eq!(outside.ui_command_count, 1);
        assert_eq!(outside.combo_expanded_change_count, 1);
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));

        route.dispatch_click(crate::Point { x: 10, y: 18 });
        let blurred = route.dispatch_blur();
        assert!(blurred.handled);
        assert_eq!(route.widget_combo_state(widget), Some((Some(0), 3, false)));
    }

    #[test]
    #[cfg(feature = "combo")]
    fn window_view_input_route_scrolls_long_combo_popup() {
        let widget = crate::WidgetId::new(93);
        let options = (0..30)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let mut view = crate::column([
            crate::combo_box::<_, UiCommand>(options, Some(0))
                .id(widget)
                .height(crate::Dp::new(36.0))
                .expanded(true),
            crate::spacer(),
        ]);
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 220,
            },
            crate::Dpi::standard(),
        ));
        let interaction_plan = view.interaction_plan();
        let option = interaction_plan
            .hit_targets
            .iter()
            .find(|target| target.kind == crate::ViewHitTargetKind::ComboBoxOption { index: 0 })
            .copied()
            .expect("long combo should expose a visible option");
        let mut route = WindowsWin32ViewInputRoute::new(interaction_plan, view);

        let report = route.dispatch_scroll(
            crate::Point {
                x: option.bounds.x + 8,
                y: option.bounds.y + option.bounds.height / 2,
            },
            crate::Dp::new(48.0),
        );

        assert!(report.handled);
        assert_eq!(report.combo_scroll_count, 1);
        assert_eq!(report.scroll_count, 1);
        assert_eq!(report.unhandled_scroll_count, 0);
        assert!(report
            .events
            .iter()
            .any(|event| event == "win32_view_combo_scroll:93:1"));
        assert_eq!(
            route
                .interaction_plan
                .combo_visible_option_range(widget)
                .map(|range| range.start),
            Some(1)
        );
    }

    #[test]
    #[cfg(feature = "date-picker")]
    fn window_view_input_route_selects_and_navigates_date_picker() {
        fn changed(_: crate::ZsDate) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.date_changed"))
        }
        fn expanded(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.date_expanded"))
        }

        let widget = crate::WidgetId::new(37);
        let initial = crate::ZsDate::new(2026, 7, 13).unwrap();
        let selected = crate::ZsDate::new(2026, 7, 14).unwrap();
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            crate::ViewHitTargetKind::DatePicker,
        );
        let previous = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 160,
                y: 40,
                width: 40,
                height: 48,
            },
            crate::ViewHitTargetKind::DatePickerPreviousMonth,
        );
        let day = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 80,
                y: 120,
                width: 40,
                height: 40,
            },
            crate::ViewHitTargetKind::DatePickerDay { date: selected },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, previous, day]),
            crate::date_picker(initial)
                .id(widget)
                .on_date_change(changed)
                .on_expanded_change(expanded),
        );

        let header_point = crate::Point { x: 20, y: 16 };
        let hovered = route.dispatch_pointer_move(header_point);
        assert!(hovered.handled);
        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert!(route.take_pending_draw_plan().is_some());
        let pressed = route.dispatch_pointer_down(header_point, false);
        assert_eq!(pressed.pointer_visual_change_count, 1);
        assert!(route.take_pending_draw_plan().is_some());
        let opened = route.dispatch_pointer_up(header_point);
        assert_eq!(opened.event_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        assert_eq!(opened.pointer_visual_change_count, 1);
        assert!(
            route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
        let left = route.dispatch_pointer_leave();
        assert!(left.handled);
        assert_eq!(left.pointer_visual_change_count, 1);

        let previous_month = route.dispatch_click(crate::Point { x: 180, y: 64 });
        assert_eq!(previous_month.event_count, 1);
        assert_eq!(previous_month.ui_command_count, 0);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .visible_month,
            crate::ZsDate::new(2026, 6, 1).unwrap()
        );

        let selection = route.dispatch_click(crate::Point { x: 100, y: 140 });
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.ui_command_count, 2);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(selected)
        );

        let keyboard = route.dispatch_key_down(u32::from(VK_RIGHT));
        assert_eq!(keyboard.event_count, 1);
        assert_eq!(
            route
                .widget_date_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsDate::new(2026, 7, 15).unwrap())
        );

        route.dispatch_click(crate::Point { x: 20, y: 16 });
        let blurred = route.dispatch_blur();
        assert!(blurred.handled);
        assert!(
            !route
                .widget_date_picker_state(widget)
                .expect("date picker state")
                .expanded
        );
    }

    #[test]
    #[cfg(feature = "time-picker")]
    fn window_view_input_route_selects_and_navigates_time_picker() {
        fn changed(_: crate::ZsTime) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.time_changed"))
        }
        fn expanded(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.time_expanded"))
        }

        let widget = crate::WidgetId::new(38);
        let initial = crate::ZsTime::new(9, 30).unwrap();
        let selected = crate::ZsTime::new(9, 45).unwrap();
        let header = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            crate::ViewHitTargetKind::TimePicker,
        );
        let choice = crate::ViewHitTarget::with_kind(
            widget,
            crate::Rect {
                x: 80,
                y: 120,
                width: 80,
                height: 40,
            },
            crate::ViewHitTargetKind::TimePickerChoice { value: selected },
        );
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([header, choice]),
            crate::time_picker(initial)
                .id(widget)
                .minute_increment(crate::ZsMinuteIncrement::FIFTEEN)
                .clock_format(crate::ZsClockFormat::TwentyFourHour)
                .on_time_change(changed)
                .on_expanded_change(expanded),
        );

        let header_point = crate::Point { x: 20, y: 16 };
        let hovered = route.dispatch_pointer_move(header_point);
        assert!(hovered.handled);
        assert_eq!(hovered.pointer_visual_change_count, 1);
        let opened = route.dispatch_click(header_point);
        assert_eq!(opened.event_count, 1);
        assert_eq!(opened.ui_command_count, 1);
        assert!(
            route
                .widget_time_picker_state(widget)
                .expect("time picker state")
                .expanded
        );

        let selection = route.dispatch_click(crate::Point { x: 100, y: 140 });
        assert_eq!(selection.event_count, 1);
        assert_eq!(selection.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| (state.value, state.expanded)),
            Some((selected, true))
        );

        let closed = route.dispatch_key_down(u32::from(VK_ESCAPE));
        assert_eq!(closed.event_count, 1);
        assert_eq!(closed.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| state.expanded),
            Some(false)
        );
        let keyboard = route.dispatch_key_down(u32::from(VK_DOWN));
        assert_eq!(keyboard.event_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert_eq!(
            route
                .widget_time_picker_state(widget)
                .map(|state| state.value),
            Some(crate::ZsTime::new(10, 0).unwrap())
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn window_view_input_route_normalizes_multiline_text_and_ignores_single_line_enter() {
        let editor = crate::WidgetId::new(30);
        let mut editor_route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                editor,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 120,
                },
                crate::ViewHitTargetKind::TextEditor,
            )]),
            crate::text_editor("").id(editor),
        );
        editor_route.dispatch_click(crate::Point { x: 20, y: 20 });

        let editor_report = editor_route.dispatch_text_input("A\r\nB\n\nC");

        assert_eq!(editor_report.text_input_count, 6);
        assert_eq!(
            editor_route.widget_text_value(editor).as_deref(),
            Some("A\nB\n\nC")
        );

        let input = crate::WidgetId::new(31);
        let mut input_route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                input,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox("").id(input),
        );
        input_route.dispatch_click(crate::Point { x: 20, y: 20 });

        let input_report = input_route.dispatch_text_input("\r");

        assert_eq!(input_report.text_input_count, 0);
        assert_eq!(input_report.event_count, 0);
        assert_eq!(input_route.widget_text_value(input).as_deref(), Some(""));
        assert_eq!(text_from_char_wparam('\r' as usize).as_deref(), Some("\r"));
    }

    #[test]
    #[cfg(feature = "checkbox")]
    fn window_view_input_route_dispatches_checkbox_toggle_to_ui_command() {
        let _guard = view_input_route_test_lock();
        fn toggled(_: bool) -> UiCommand {
            UiCommand::app(crate::CommandId("zsui.test.win32.toggle_changed"))
        }

        clear_windows_win32_window_view_input_routes();
        let hwnd = 79isize as HWND;
        let widget = crate::WidgetId::new(11);
        let route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Checkbox,
            )]),
            crate::checkbox("Dark mode", false)
                .id(widget)
                .on_toggle(toggled),
        );

        assert!(set_windows_win32_window_view_input_route(hwnd, route));
        let toggle = dispatch_windows_win32_window_view_click(hwnd, crate::Point { x: 20, y: 20 })
            .expect("registered route should toggle checkbox");
        let key_toggle = dispatch_windows_win32_window_view_key_down(hwnd, ZSUI_WIN32_VK_SPACE)
            .expect("focused checkbox should toggle from keyboard");
        let aggregate = windows_win32_window_view_input_report(hwnd)
            .expect("registered route should keep aggregate report");

        assert_eq!(toggle.toggle_count, 1);
        assert_eq!(toggle.event_count, 1);
        assert_eq!(toggle.ui_command_count, 1);
        assert_eq!(
            toggle.ui_command_ids,
            vec!["zsui.test.win32.toggle_changed"]
        );
        assert_eq!(key_toggle.key_down_count, 1);
        assert_eq!(key_toggle.keyboard_activation_count, 1);
        assert_eq!(key_toggle.toggle_count, 1);
        assert_eq!(key_toggle.ui_command_count, 1);
        assert_eq!(aggregate.toggle_count, 2);
        assert_eq!(aggregate.key_down_count, 1);
        assert_eq!(aggregate.keyboard_activation_count, 1);
        assert_eq!(aggregate.ui_command_count, 2);
        clear_windows_win32_window_view_input_route(hwnd);
    }

    #[test]
    #[cfg(feature = "toggle-button")]
    fn window_view_input_route_dispatches_toggle_button_pointer_and_keyboard() {
        let widget = crate::WidgetId::new(15);
        let mut route = WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 36,
                },
                crate::ViewHitTargetKind::ToggleButton,
            )]),
            crate::toggle_button("Pin", false)
                .id(widget)
                .on_toggle(|_| UiCommand::app(crate::CommandId("zsui.test.pin_changed"))),
        );

        let hovered = route.dispatch_pointer_move(crate::Point { x: 20, y: 18 });
        let pointer = route.dispatch_click(crate::Point { x: 20, y: 18 });
        let keyboard = route.dispatch_key_down(ZSUI_WIN32_VK_SPACE);

        assert_eq!(hovered.pointer_visual_change_count, 1);
        assert_eq!(pointer.toggle_count, 1);
        assert_eq!(pointer.ui_command_count, 1);
        assert_eq!(keyboard.keyboard_activation_count, 1);
        assert_eq!(keyboard.toggle_count, 1);
        assert_eq!(keyboard.ui_command_count, 1);
        assert_eq!(route.widget_checked_value(widget), Some(false));
    }

    #[test]
    fn owned_hwnd_wrapper_is_drop_backed_and_can_release_legacy_raw_handles() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedMainWindowHandles>());

        let handles = NativeMainWindowHandles {
            main: 1isize as HWND,
            quick: 2isize as HWND,
        };
        let owned = WindowsWin32OwnedMainWindowHandles::new(handles);

        assert_eq!(owned.handles(), handles);
        assert_eq!(owned.main(), handles.main);
        assert_eq!(owned.quick(), handles.quick);
        assert_eq!(owned.app_icon_count(), 0);
        assert_eq!(owned.into_handles(), handles);
    }

    #[test]
    fn owned_hicon_wrappers_model_raii_without_double_destroying_shared_sizes() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedIcon>());
        assert!(std::mem::needs_drop::<WindowsWin32OwnedAppIconResource>());
        assert!(WindowsWin32OwnedIcon::from_raw(null_mut()).is_none());
        assert!(WindowsWin32OwnedAppIconResource::from_raw(null_mut(), null_mut()).is_none());
        assert!(matches!(
            WindowsWin32OwnedIcon::from_icon_path("", 16, 16),
            Err(ZsuiError::InvalidSpec { field, .. }) if field == "window.icon_path"
        ));

        let icon = WindowsWin32OwnedIcon::from_raw(1isize as HICON)
            .expect("non-null HICON should be accepted");
        assert_eq!(icon.into_raw(), 1isize as HICON);

        let resource = WindowsWin32OwnedAppIconResource::from_raw(2isize as HICON, 2isize as HICON)
            .expect("shared small/big HICON should be accepted");
        assert_eq!(
            resource.as_native_resource(),
            NativeAppIconResource { small: 2, big: 2 }
        );
        assert_eq!(resource.into_raw_pair(), (2isize as HICON, 2isize as HICON));
    }

    #[test]
    fn owned_tray_icon_data_keeps_win32_notify_contract_and_raii_shape() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedTrayIcon>());

        let hwnd = 7isize as HWND;
        let data = tray_notify_data(
            hwnd,
            42,
            Some("ZSUI"),
            Some(9isize as HICON),
            ZSUI_WIN32_TRAY_CALLBACK_MESSAGE,
        );

        assert_eq!(data.hWnd, hwnd);
        assert_eq!(data.uID, 42);
        assert_eq!(data.uCallbackMessage, ZSUI_WIN32_TRAY_CALLBACK_MESSAGE);
        assert_ne!(data.uFlags & NIF_MESSAGE, 0);
        assert_ne!(data.uFlags & NIF_TIP, 0);
        assert_ne!(data.uFlags & NIF_ICON, 0);
        assert_eq!(data.szTip[0], 'Z' as u16);
    }

    #[test]
    fn status_menu_command_table_maps_nested_menu_to_native_ids() {
        let menu = MenuSpec::new()
            .item("Open", Command::ShowMainWindow)
            .submenu(
                "More",
                MenuSpec::new()
                    .item("Refresh", Command::custom("example.refresh"))
                    .separator()
                    .item("Quit", Command::Quit),
            );
        let table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);

        assert_eq!(table.entry_count(), 3);
        assert_eq!(
            table.first_native_id(),
            Some(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID)
        );
        assert_eq!(
            table.resolve_native_command_id(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID + 1),
            NativeStatusMenuCommandResult::Dispatched(Command::custom("example.refresh"))
        );
        assert_eq!(
            table.resolve_native_command_id(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID + 99),
            NativeStatusMenuCommandResult::NotFound
        );
    }

    #[test]
    fn owned_accelerator_table_uses_typed_menu_commands_and_raii() {
        let mut menu = MenuSpec::new();
        menu.items.push(
            MenuItemSpec::command("Open", Command::custom("file.open"))
                .accelerator(ZsAccelerator::primary_character('O')),
        );
        menu.items.push(
            MenuItemSpec::command("Save As", Command::custom("file.save_as"))
                .accelerator(ZsAccelerator::primary_character('S').shifted()),
        );
        let table = WindowsWin32StatusMenuCommandTable::from_menu(&menu);
        let accelerators = WindowsWin32OwnedAcceleratorTable::from_command_table(&table)
            .expect("valid Win32 accelerators")
            .expect("accelerator table should be created");

        assert!(std::mem::needs_drop::<WindowsWin32OwnedAcceleratorTable>());
        assert_eq!(accelerators.entry_count(), 2);
        assert_eq!(
            table.entries()[0].accelerator,
            Some(ZsAccelerator::primary_character('O'))
        );

        let duplicate = WindowsWin32OwnedAcceleratorTable::from_bindings(&[
            (1, ZsAccelerator::primary_character('O')),
            (2, ZsAccelerator::primary_character('O')),
        ])
        .expect_err("duplicate native bindings must be rejected");
        assert!(matches!(
            duplicate,
            ZsuiError::InvalidSpec { field, .. } if field == "accelerator.bindings"
        ));
    }

    #[test]
    fn owned_status_popup_menu_creates_native_menu_and_cleans_up() {
        assert!(std::mem::needs_drop::<WindowsWin32OwnedPopupMenu>());
        let menu = MenuSpec::new()
            .item("Open", Command::ShowMainWindow)
            .separator()
            .item("Quit", Command::Quit);
        let popup = WindowsWin32OwnedPopupMenu::from_menu(&menu)
            .expect("Win32 popup menu should be created from a status menu spec");

        assert!(!popup.handle().is_null());
        assert_eq!(popup.command_entry_count(), 2);
        assert_eq!(
            popup.dispatch_native_menu_command(ZSUI_WIN32_STATUS_MENU_FIRST_COMMAND_ID),
            NativeStatusMenuCommandResult::Dispatched(Command::ShowMainWindow)
        );
        assert!(popup.destroy());
    }

    #[test]
    fn status_popup_menu_selection_uses_return_command_flags() {
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_RETURNCMD, 0);
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_NONOTIFY, 0);
        assert_ne!(ZSUI_WIN32_STATUS_MENU_TRACK_FLAGS & TPM_RIGHTBUTTON, 0);

        let popup = WindowsWin32OwnedPopupMenu::from_menu(
            &MenuSpec::new().item("Open", Command::ShowMainWindow),
        )
        .expect("Win32 popup menu should be created");
        assert!(matches!(
            popup.present_at(null_mut(), 0, 0),
            Err(ZsuiError::InvalidSpec { field, .. }) if field == "status_item.owner"
        ));
    }

    #[test]
    fn win32_status_item_host_rejects_null_owner_without_leaking_tray_handle() {
        let mut host = WindowsWin32StatusItemHost::new(null_mut());
        let presentation = host.create_status_item(NativeStatusItemRequest::from_tray_spec(
            &crate::TraySpec::new()
                .tooltip("ZSUI")
                .item("Quit", crate::Command::Quit),
        ));

        assert!(matches!(presentation, NativeStatusItemPresentation::Failed));
        assert_eq!(host.item_count(), 0);
        assert!(host
            .last_error()
            .expect("failed status item should retain host error")
            .contains("status_item.owner"));
        assert_eq!(
            host.operation_log(),
            &[NativeStatusItemHostOperation::CreateStatusItem]
        );
    }

    #[test]
    fn win32_host_records_native_main_window_host_operations() {
        let mut host = WindowsWin32MainWindowHost::new();

        host.hide_main_window(null_mut());
        assert_eq!(
            host.operation_log(),
            &[NativeMainWindowHostOperation::HideMainWindow]
        );
    }

    #[test]
    fn transient_host_preserves_topmost_noactivate_window_shape() {
        let mut host = WindowsWin32TransientWindowHost::new();

        host.present_transient_window(
            null_mut(),
            UiRect {
                left: 10,
                top: 20,
                right: 110,
                bottom: 70,
            },
        );
        host.hide_transient_window(null_mut());
        host.destroy_transient_window(null_mut());

        assert_eq!(host.class_name(), "ZsuiTransientWindow");
        assert_eq!(
            host.operation_log(),
            &[
                NativeTransientWindowHostOperation::PresentTransientWindow,
                NativeTransientWindowHostOperation::HideTransientWindow,
                NativeTransientWindowHostOperation::DestroyTransientWindow,
            ]
        );
    }
}
