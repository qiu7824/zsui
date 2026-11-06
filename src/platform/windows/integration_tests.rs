use super::*;
use std::sync::atomic::{AtomicIsize, AtomicUsize};

pub(super) fn on_ui(test: impl FnOnce() + Send + 'static) {
    std::thread::spawn(move || {
        let _guard = windows_win32_view_input_route_test_lock();
        let _paint = crate::windows_gdi_renderer::WindowsBufferedPaintSession::new();
        test();
    })
    .join()
    .expect("native integration test UI thread");
}

fn placement(id: crate::NativeHostId) -> crate::NativeHostPlacement {
    crate::NativeHostPlacement {
        id,
        widget: crate::WidgetId(id.get()),
        bounds: crate::Rect {
            x: 12,
            y: 18,
            width: 220,
            height: 80,
        },
        clip: crate::Rect {
            x: 32,
            y: 18,
            width: 180,
            height: 80,
        },
        focusable: true,
    }
}

#[test]
fn native_child_identity_clipping_overlay_and_destruction_follow_view_lifetime() {
    on_ui(|| {
        struct Child {
            window: interop::NativeChildWindow,
            destroyed: Arc<AtomicUsize>,
        }
        impl interop::WindowsNativeChild for Child {
            fn window(&self) -> &interop::NativeChildWindow {
                &self.window
            }
        }
        impl Drop for Child {
            fn drop(&mut self) {
                self.destroyed.fetch_add(1, Ordering::SeqCst);
            }
        }
        let created = Arc::new(AtomicUsize::new(0));
        let destroyed = Arc::new(AtomicUsize::new(0));
        let handle = Arc::new(AtomicIsize::new(0));
        let id = crate::NativeHostId::new(801);
        let hooks = interop::WindowsWindowHooks::new()
            .child(id, {
                let created = created.clone();
                let destroyed = destroyed.clone();
                let handle = handle.clone();
                move |parent| {
                    let window = parent.edit("native", false)?;
                    handle.store(window.hwnd() as isize, Ordering::SeqCst);
                    created.fetch_add(1, Ordering::SeqCst);
                    Ok(Box::new(Child {
                        window,
                        destroyed: destroyed.clone(),
                    }))
                }
            })
            .unwrap();
        let mut spec = WindowSpec::new("Native slot lifetime")
            .size(320, 200)
            .visible(false);
        spec.windows_hooks = Some(hooks);
        let mut plan = NativeDrawPlan::default();
        plan.native_hosts.push(placement(id));
        let windows =
            create_owned_windows_for_specs_with_draw_plans(&[spec], &[Some(plan.clone())]).unwrap();
        let child = handle.load(Ordering::SeqCst) as HWND;
        assert_eq!(created.load(Ordering::SeqCst), 1);
        assert_ne!(unsafe { IsWindow(child) }, 0);
        plan.native_occlusions
            .push(crate::native_host::NativeSurfaceOcclusion {
                bounds: crate::Rect {
                    x: 60,
                    y: 20,
                    width: 30,
                    height: 30,
                },
                hosts_before: 1,
                drops_before: 0,
            });
        assert!(set_windows_win32_window_draw_plan(windows[0].main(), plan));
        assert_eq!(created.load(Ordering::SeqCst), 1);
        unsafe {
            use windows_sys::Win32::Graphics::Gdi::{
                CreateRectRgn, DeleteObject, GetWindowRgn, PtInRegion,
            };
            let region = CreateRectRgn(0, 0, 0, 0);
            assert!(!region.is_null());
            assert_ne!(GetWindowRgn(child, region), 0);
            assert_eq!(PtInRegion(region, 0, 5), 0);
            assert_ne!(PtInRegion(region, 28, 7), 0);
            assert_eq!(PtInRegion(region, 50, 7), 0);
            DeleteObject(region);
        }
        set_windows_win32_window_draw_plan(windows[0].main(), NativeDrawPlan::default());
        assert_eq!(destroyed.load(Ordering::SeqCst), 1);
        assert_eq!(unsafe { IsWindow(child) }, 0);
        let mut plan = NativeDrawPlan::default();
        plan.native_hosts.push(placement(id));
        set_windows_win32_window_draw_plan(windows[0].main(), plan);
        assert_eq!(created.load(Ordering::SeqCst), 2);
        drop(windows);
        assert_eq!(destroyed.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn message_filters_are_ordered_scoped_and_released_after_reentrant_destroy() {
    on_ui(|| {
        struct Filter {
            log: Arc<Mutex<Vec<u8>>>,
            id: u8,
            consume: bool,
            destroy: bool,
            drops: Arc<AtomicUsize>,
        }
        impl interop::NativeMessageFilter for Filter {
            fn pre_translate(&mut self, message: &MSG) -> bool {
                self.log.lock().unwrap().push(self.id);
                if self.destroy {
                    unsafe {
                        DestroyWindow(message.hwnd);
                    }
                }
                self.consume
            }
        }
        impl Drop for Filter {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }
        let log = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(AtomicUsize::new(0));
        let mut hooks = interop::WindowsWindowHooks::new();
        for id in 1..=3 {
            let log = log.clone();
            let drops = drops.clone();
            hooks = hooks.message_filter(move || {
                Box::new(Filter {
                    log: log.clone(),
                    id,
                    consume: id == 2,
                    destroy: id == 2,
                    drops: drops.clone(),
                })
            });
        }
        let mut spec = WindowSpec::new("Scoped message filters").visible(false);
        spec.windows_hooks = Some(hooks);
        let windows = create_owned_windows_for_specs(&[spec]).unwrap();
        let services = windows[0].services.as_ref().unwrap().clone();
        let message = MSG {
            hwnd: windows[0].main(),
            message: WM_KEYDOWN,
            wParam: 0x41,
            ..unsafe { zeroed() }
        };
        assert!(services.pre_translate(&message));
        assert_eq!(*log.lock().unwrap(), vec![1, 2]);
        assert_eq!(drops.load(Ordering::SeqCst), 3);
        assert!(!services.pre_translate(&message));
        drop(windows);
    });
}

#[test]
fn missing_native_provider_is_an_error_before_showing_the_window() {
    on_ui(|| {
        let mut plan = NativeDrawPlan::default();
        plan.native_hosts
            .push(placement(crate::NativeHostId::new(99)));
        let result = create_owned_windows_for_specs_with_draw_plans(
            &[WindowSpec::new("Missing provider").visible(false)],
            &[Some(plan)],
        );
        assert!(
            matches!(result, Err(ZsuiError::InvalidSpec { field, .. }) if field == "native_host.provider")
        );
    });
}

#[cfg(feature = "window-chrome")]
#[test]
fn integrated_title_bar_reserves_real_system_buttons_and_native_hit_codes() {
    on_ui(|| {
        let mut spec = WindowSpec::new("Integrated caption")
            .size(480, 240)
            .visible(false);
        spec.custom_title_bar = true;
        spec.integration.title_bar = Some(crate::IntegratedTitleBar::default());
        let windows = create_owned_windows_for_specs(&[spec]).unwrap();
        let hwnd = windows[0].main();
        let metrics = windows[0]
            .services
            .as_ref()
            .unwrap()
            .caption_metrics
            .get()
            .unwrap();
        assert!(metrics.buttons.width > 0 && metrics.dpi >= 96);
        let mut point = POINT {
            x: metrics.buttons.x + metrics.buttons.width / 2,
            y: metrics.buttons.y + metrics.buttons.height / 2,
        };
        unsafe {
            windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut point);
            let packed = ((point.y as u16 as u32) << 16) | point.x as u16 as u32;
            assert_eq!(
                SendMessageW(hwnd, WM_NCHITTEST, 0, packed as isize),
                windows_sys::Win32::UI::WindowsAndMessaging::HTMAXBUTTON as isize
            );
        }
    });
}

#[cfg(feature = "hotkey")]
#[test]
fn global_hotkey_conflicts_are_reported_and_ownership_is_released() {
    on_ui(|| {
        let windows = create_owned_windows_for_specs(&[
            WindowSpec::new("Hotkey owner").visible(false),
            WindowSpec::new("Hotkey contender").visible(false),
        ])
        .unwrap();
        let binding =
            crate::HotkeySpec::new("Ctrl+Alt+Shift+F24", Command::custom("native.hotkey"));
        let first =
            WindowsGlobalHotkeys::register(windows[0].main(), std::slice::from_ref(&binding))
                .unwrap();
        assert!(
            WindowsGlobalHotkeys::register(windows[1].main(), std::slice::from_ref(&binding))
                .is_err()
        );
        drop(first);
        let second = WindowsGlobalHotkeys::register(windows[1].main(), &[binding]).unwrap();
        assert_eq!(second.commands.len(), 1);
        drop(second);
    });
}

#[test]
fn native_palette_queries_match_window_paint_in_dark_and_high_contrast_modes() {
    on_ui(|| {
        for mode in [
            crate::ZsuiThemeMode::Light,
            crate::ZsuiThemeMode::Dark,
            crate::ZsuiThemeMode::HighContrast,
        ] {
            let palette = crate::PlatformPalette::current(mode).unwrap();
            let plan = NativeDrawPlan::default().theme_mode(mode);
            let renderer = windows_palette_for_draw_plan(Some(&plan));
            for role in [
                crate::ColorRole::Surface,
                crate::ColorRole::PrimaryText,
                crate::ColorRole::Accent,
                crate::ColorRole::StrongStroke,
            ] {
                assert_eq!(palette.resolve(role), renderer.resolve(role));
            }
        }
    });
}

#[cfg(feature = "window-surface")]
#[test]
fn layered_surface_is_prepared_before_show_without_a_paint_message() {
    on_ui(|| {
        let mut spec = WindowSpec::new("Prepared alpha surface")
            .size(80, 60)
            .decorations(false)
            .visible(false);
        spec.transparent = true;
        spec.integration.surface = crate::WindowSurface::PerPixelAlpha {
            hit_test: crate::AlphaHitTest::TransparentPixels,
        };
        let plan = NativeDrawPlan::new([crate::NativeDrawCommand::FillRect {
            rect: crate::Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 60,
            },
            fill: crate::NativeDrawFill::Color(Color::rgba(180, 60, 20, 128)),
        }]);
        let windows =
            create_owned_windows_for_specs_with_draw_plans(&[spec], &[Some(plan)]).unwrap();
        assert_eq!(
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(windows[0].main())
            },
            0
        );
        assert!(windows[0]
            .services
            .as_ref()
            .unwrap()
            .direct2d
            .borrow()
            .is_some());
    });
}

#[cfg(feature = "desktop-layer")]
#[test]
fn explicit_desktop_hiding_survives_shell_recovery() {
    on_ui(|| {
        let mut spec = WindowSpec::new("Desktop visibility policy")
            .size(80, 60)
            .decorations(false)
            .visible(false);
        spec.integration.layer = crate::WindowLayer::Desktop {
            follow_show_desktop: true,
        };
        let windows = create_owned_windows_for_specs(&[spec]).unwrap();
        let hwnd = windows[0].main();
        assert!(set_desktop_window_visibility(hwnd, true));
        assert!(set_desktop_window_visibility(hwnd, false));
        unsafe {
            SendMessageW(hwnd, windows_win32_taskbar_created_message(), 0, 0);
        }
        assert_eq!(
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd) },
            0
        );
        assert!(!windows[0]
            .services
            .as_ref()
            .unwrap()
            .desktop_layer()
            .unwrap()
            .visible
            .get());
    });
}
