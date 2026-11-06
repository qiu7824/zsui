//! Interactive target proof, separate from ordinary unit tests.
use super::*;
use std::{
    os::windows::process::CommandExt,
    sync::atomic::AtomicUsize,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Graphics::Gdi::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

fn output_root() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("ZSUI_INTEGRATION_PROOF_DIR")
            .expect("set ZSUI_INTEGRATION_PROOF_DIR to an operation/artifact directory"),
    );
    assert!(path.is_absolute());
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn pump(windows: &[WindowsWin32OwnedMainWindowHandles], milliseconds: u64) {
    let deadline = Instant::now() + Duration::from_millis(milliseconds);
    while Instant::now() < deadline {
        unsafe {
            let mut message: MSG = zeroed();
            while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if message.message == WM_QUIT {
                    continue;
                }
                if windows.iter().any(|window| {
                    window
                        .services
                        .as_ref()
                        .is_some_and(|services| services.pre_translate(&message))
                        || window.translate_accelerator(&message)
                }) {
                    continue;
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn keys(hwnd: HWND, values: &[(u16, bool)]) {
    assert_eq!(
        unsafe { GetForegroundWindow() },
        hwnd,
        "proof window lost foreground; no input injected"
    );
    let inputs: Vec<INPUT> = values
        .iter()
        .map(|(key, release)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: *key,
                    dwFlags: if *release { KEYEVENTF_KEYUP } else { 0 },
                    ..unsafe { zeroed() }
                },
            },
        })
        .collect();
    assert_eq!(
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        },
        inputs.len() as u32
    );
}

fn text_input(hwnd: HWND, value: &str) {
    assert_eq!(
        unsafe { GetForegroundWindow() },
        hwnd,
        "proof window lost foreground; no text injected"
    );
    let inputs: Vec<INPUT> = value
        .encode_utf16()
        .flat_map(|unit| {
            [false, true].map(move |release| INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: unit,
                        dwFlags: KEYEVENTF_UNICODE | if release { KEYEVENTF_KEYUP } else { 0 },
                        ..unsafe { zeroed() }
                    },
                },
            })
        })
        .collect();
    assert_eq!(
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        },
        inputs.len() as u32
    );
}

fn activate_target(windows: &[WindowsWin32OwnedMainWindowHandles], hwnd: HWND) {
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        SetForegroundWindow(hwnd);
    }
    pump(windows, 40);
    if unsafe { GetForegroundWindow() } != hwnd {
        let mut bounds: RECT = unsafe { zeroed() };
        assert_ne!(unsafe { GetWindowRect(hwnd, &mut bounds) }, 0);
        let point = POINT {
            x: bounds.left + 30,
            y: bounds.top + 20,
        };
        assert_eq!(
            unsafe { WindowFromPoint(point) },
            hwnd,
            "proof title area is covered; no click injected"
        );
        unsafe {
            SetCursorPos(point.x, point.y);
        }
        let inputs = [
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_LEFTDOWN,
                        ..unsafe { zeroed() }
                    },
                },
            },
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_LEFTUP,
                        ..unsafe { zeroed() }
                    },
                },
            },
        ];
        assert_eq!(
            unsafe { SendInput(2, inputs.as_ptr(), size_of::<INPUT>() as i32) },
            2
        );
        pump(windows, 100);
    }
    assert_eq!(
        unsafe { GetForegroundWindow() },
        hwnd,
        "could not focus the proof window"
    );
}

unsafe fn mouse_button(up: bool) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: if up {
                    MOUSEEVENTF_LEFTUP
                } else {
                    MOUSEEVENTF_LEFTDOWN
                },
                ..zeroed()
            },
        },
    };
    SendInput(1, &input, size_of::<INPUT>() as i32);
}

unsafe fn toggle_show_desktop() {
    let values = [
        (VK_LWIN, false),
        (b'D' as u16, false),
        (b'D' as u16, true),
        (VK_LWIN, true),
    ];
    let inputs: Vec<INPUT> = values
        .into_iter()
        .map(|(key, up)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    ..zeroed()
                },
            },
        })
        .collect();
    SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        size_of::<INPUT>() as i32,
    );
}

fn screen_capture(hwnd: HWND, path: &Path) -> serde_json::Value {
    assert_ne!(unsafe { IsWindowVisible(hwnd) }, 0);
    assert_eq!(unsafe { IsIconic(hwnd) }, 0);
    unsafe {
        let _ = windows_sys::Win32::Graphics::Dwm::DwmFlush();
        let mut rect: RECT = zeroed();
        assert_ne!(GetWindowRect(hwnd, &mut rect), 0);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        assert!(width > 0 && height > 0);
        struct Capture {
            screen: HDC,
            memory: HDC,
            bitmap: HBITMAP,
            previous: HGDIOBJ,
        }
        impl Drop for Capture {
            fn drop(&mut self) {
                unsafe {
                    SelectObject(self.memory, self.previous);
                    DeleteObject(self.bitmap);
                    DeleteDC(self.memory);
                    ReleaseDC(null_mut(), self.screen);
                }
            }
        }
        let screen = GetDC(null_mut());
        assert!(!screen.is_null());
        let memory = CreateCompatibleDC(screen);
        assert!(!memory.is_null());
        let bitmap = CreateCompatibleBitmap(screen, width, height);
        assert!(!bitmap.is_null());
        let _capture = Capture {
            screen,
            memory,
            bitmap,
            previous: SelectObject(memory, bitmap),
        };
        assert_ne!(
            BitBlt(
                memory,
                0,
                0,
                width,
                height,
                screen,
                rect.left,
                rect.top,
                SRCCOPY | CAPTUREBLT
            ),
            0
        );
        SelectObject(memory, _capture.previous);
        let mut info: BITMAPINFO = zeroed();
        info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width;
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        assert_eq!(
            GetDIBits(
                memory,
                bitmap,
                0,
                height as u32,
                rgba.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS
            ),
            height
        );
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        serde_json::json!({"capture":"desktop_dc","width":width,"height":height,"screen_x":rect.left,"screen_y":rect.top,"dpi":GetDpiForWindow(hwnd),"file":path.file_name().unwrap().to_string_lossy()})
    }
}

#[derive(Clone)]
enum Message {
    Native(crate::NativeWindowEvent),
    Files(crate::FilesDropped),
}
struct State {
    reserved: f32,
    events: Arc<Mutex<Vec<crate::NativeWindowEvent>>>,
    drops: Arc<Mutex<Vec<crate::FilesDropped>>>,
}

fn main_view(state: &State) -> crate::ViewNode<Message> {
    use crate::{native_host, Dp, NativeHostId, ZsIcon};
    crate::column([
        crate::row([
            crate::text("Native window integration").flex(1.0),
            crate::spacer().width(Dp::new(state.reserved)),
        ])
        .height(Dp::new(40.0))
        .window_drag_region(),
        native_host(NativeHostId::new(810)).height(Dp::new(34.0)),
        crate::row([
            native_host(NativeHostId::new(811)).width(Dp::new(210.0)),
            native_host(NativeHostId::new(812)).flex(1.0),
        ])
        .gap(Dp::new(12.0))
        .flex(1.0),
        crate::row(
            [
                ZsIcon::Back,
                ZsIcon::Forward,
                ZsIcon::Up,
                ZsIcon::Sort,
                ZsIcon::View,
                ZsIcon::Rename,
                ZsIcon::Share,
                ZsIcon::Maximize,
                ZsIcon::Restore,
                ZsIcon::Preview,
                ZsIcon::Properties,
                ZsIcon::Star,
                ZsIcon::StarFilled,
            ]
            .map(|icon| crate::icon(icon).width(Dp::new(36.0))),
        )
        .height(Dp::new(36.0)),
        crate::text("File drop surface").height(Dp::new(26.0)),
    ])
    .id(crate::WidgetId(800))
    .gap(Dp::new(10.0))
    .padding_sides(Dp::new(12.0), Dp::new(0.0), Dp::new(12.0), Dp::new(12.0))
    .theme_mode(crate::ZsuiThemeMode::Dark)
    .on_native_window_event(Message::Native)
    .on_drop_files(Message::Files)
}

fn update(state: &mut State, message: Message, _cx: &mut crate::AppCx) {
    match message {
        Message::Native(event) => {
            if let crate::NativeWindowEvent::CaptionChanged(metrics) = event {
                state.reserved = metrics.reserved_width().0;
            }
            state.events.lock().unwrap().push(event);
        }
        Message::Files(files) => state.drops.lock().unwrap().push(files),
    }
}

#[test]
#[ignore = "interactive Windows target proof; requires ZSUI_INTEGRATION_PROOF_DIR"]
fn native_windows_integration_target_proof() {
    let output = output_root();
    let fixture = output.join("fixture");
    std::fs::create_dir_all(&fixture).unwrap();
    let paths = [fixture.join("文件 示例.txt"), fixture.join("second.txt")];
    for path in &paths {
        if path.exists() {
            assert_eq!(std::fs::read(path).unwrap(), b"ZSUI native fixture\n");
        } else {
            std::fs::write(path, b"ZSUI native fixture\n").unwrap();
        }
    }
    let before = unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() };
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "windows_win32_host::integration_proof::isolated_file_clipboard_target_proof",
            "--ignored",
            "--nocapture",
        ])
        .env("ZSUI_CLIPBOARD_ISOLATED_CHILD", "1")
        .env("ZSUI_INTEGRATION_PROOF_DIR", &output)
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "isolated clipboard proof: {} {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let clipboard_unchanged =
        unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() } == before;
    let output_ui = output.clone();
    let fixture_ui = fixture.clone();
    let paths_ui = paths.clone();
    integration_tests::on_ui(move || {
        let fixture = fixture_ui;
        let paths = paths_ui;
        struct Restore {
            foreground: HWND,
            cursor: POINT,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    mouse_button(true);
                    SetCursorPos(self.cursor.x, self.cursor.y);
                    if IsWindow(self.foreground) != 0 && IsWindowVisible(self.foreground) != 0 {
                        SetForegroundWindow(self.foreground);
                    }
                }
            }
        }
        let mut cursor: POINT = unsafe { zeroed() };
        unsafe {
            GetCursorPos(&mut cursor);
        }
        let _restore = Restore {
            foreground: unsafe { GetForegroundWindow() },
            cursor,
        };
        let events = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(Mutex::new(Vec::new()));
        let filters = Arc::new(AtomicUsize::new(0));
        struct Filter(Arc<AtomicUsize>);
        impl interop::NativeMessageFilter for Filter {
            fn pre_translate(&mut self, message: &MSG) -> bool {
                if message.message == WM_KEYDOWN || message.message == WM_CHAR {
                    self.0.fetch_add(1, Ordering::SeqCst);
                }
                false
            }
        }
        let hooks = interop::WindowsWindowHooks::new().message_filter({
            let filters = filters.clone();
            move || Box::new(Filter(filters.clone()))
        });
        let providers = crate::NativeHosts::new()
            .edit(crate::NativeHostId::new(810), "Native edit", false)
            .unwrap()
            .namespace_tree(crate::NativeHostId::new(811), &fixture)
            .unwrap()
            .explorer(crate::NativeHostId::new(812), &fixture)
            .unwrap();
        let main = crate::native_window("ZSUI Native Integration")
            .size(860, 560)
            .native_hosts(providers)
            .windows_hooks(hooks)
            .integrated_title_bar(
                crate::IntegratedTitleBar::default().backdrop(crate::WindowBackdrop::Mica),
            )
            .global_hotkey(crate::HotkeySpec::new(
                "Ctrl+Alt+Shift+F24",
                Command::custom("integration.hotkey"),
            ))
            .stateful_view(
                State {
                    reserved: 138.0,
                    events: events.clone(),
                    drops: drops.clone(),
                },
                main_view,
                update,
            );
        let desktop = crate::native_window("ZSUI Desktop Surface")
            .size(360, 180)
            .decorations(false)
            .surface(crate::WindowSurface::PerPixelAlpha {
                hit_test: crate::AlphaHitTest::TransparentPixels,
            })
            .layer(crate::WindowLayer::Desktop {
                follow_show_desktop: true,
            })
            .stateful_view(
                State {
                    reserved: 0.0,
                    events: events.clone(),
                    drops: drops.clone(),
                },
                |_| {
                    crate::column([
                        crate::text("Desktop panel"),
                        crate::text("Native alpha surface"),
                    ])
                    .padding(crate::Dp::new(24.0))
                    .gap(crate::Dp::new(12.0))
                    .bg(crate::ThemeColorToken::SurfaceRaised)
                    .radius(crate::Dp::new(18.0))
                    .theme_mode(crate::ZsuiThemeMode::Dark)
                    .on_native_window_event(Message::Native)
                },
                update,
            );
        let builders = [main, desktop];
        let specs: Vec<_> = builders.iter().map(|b| b.window_spec().clone()).collect();
        let plans: Vec<_> = builders
            .iter()
            .map(|b| b.native_draw_plan().cloned())
            .collect();
        let routes: Vec<_> = builders
            .iter()
            .map(|b| windows_win32_view_input_route(&b.native_view_input_runtime()))
            .collect();
        let windows =
            create_owned_windows_for_specs_with_routes(&specs, &plans, &routes, &[None, None])
                .unwrap();
        let hwnd = windows[0].main();
        let desktop = windows[1].main();
        unsafe {
            SetWindowPos(hwnd, null_mut(), 60, 60, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
            SetWindowPos(
                desktop,
                null_mut(),
                100,
                100,
                0,
                0,
                SWP_NOSIZE | SWP_NOACTIVATE,
            );
            SetForegroundWindow(hwnd);
            SetFocus(hwnd);
        }
        pump(&windows, 1200);
        let edit = unsafe { FindWindowExW(hwnd, null_mut(), wide_null("EDIT").as_ptr(), null()) };
        assert!(!edit.is_null());
        activate_target(&windows, hwnd);
        unsafe {
            SetFocus(hwnd);
        }
        keys(hwnd, &[(VK_TAB, false), (VK_TAB, true)]);
        pump(&windows, 120);
        if unsafe { GetFocus() } != edit {
            let focus = unsafe { GetFocus() };
            let mut class = [0u16; 128];
            let length = unsafe { GetClassNameW(focus, class.as_mut_ptr(), 128) };
            println!("focus diagnostic: main={hwnd:?}, edit={edit:?}, focus={focus:?}, class={}, semantic={:?}",String::from_utf16_lossy(&class[..length.max(0) as usize]),windows_win32_window_focused_target(hwnd));
            screen_capture(hwnd, &output_ui.join("focus-probe.png"));
        }
        assert_eq!(unsafe { GetFocus() }, edit);
        text_input(hwnd, "测试");
        pump(&windows, 120);
        let mut text = vec![0u16; 256];
        let length = unsafe { GetWindowTextW(edit, text.as_mut_ptr(), text.len() as i32) };
        assert!(String::from_utf16_lossy(&text[..length as usize]).contains("测试"));
        keys(hwnd, &[(VK_F6, false), (VK_F6, true)]);
        pump(&windows, 120);
        assert_ne!(unsafe { GetFocus() }, edit);
        assert_ne!(unsafe { IsChild(hwnd, GetFocus()) }, 0);
        keys(hwnd, &[(VK_F6, false), (VK_F6, true)]);
        pump(&windows, 150);
        assert_ne!(unsafe { IsChild(hwnd, GetFocus()) }, 0);
        keys(hwnd, &[(VK_F5, false), (VK_F5, true)]);
        pump(&windows, 300);
        keys(
            hwnd,
            &[
                (VK_CONTROL, false),
                (VK_MENU, false),
                (VK_SHIFT, false),
                (VK_F24, false),
                (VK_F24, true),
                (VK_SHIFT, true),
                (VK_MENU, true),
                (VK_CONTROL, true),
            ],
        );
        pump(&windows, 150);
        assert!(events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event, crate::NativeWindowEvent::GlobalHotkey(_))));
        assert!(filters.load(Ordering::SeqCst) > 0);
        let main_capture = screen_capture(hwnd, &output_ui.join("native-controls.png"));
        // A real OLE drag ends on the View-owned footer, outside native children.
        let mut drop_point = POINT { x: 650, y: 530 };
        unsafe {
            ClientToScreen(hwnd, &mut drop_point);
            SetCursorPos(drop_point.x - 80, drop_point.y);
            mouse_button(false);
        }
        pump(&windows, 30);
        let owner = hwnd as usize;
        let worker = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            unsafe {
                if GetForegroundWindow() == owner as HWND {
                    SetCursorPos(drop_point.x, drop_point.y);
                }
                std::thread::sleep(Duration::from_millis(150));
                mouse_button(true);
            }
        });
        let outcome = shell::start_drag(
            hwnd,
            &crate::DragPayload::ShellItems(paths.to_vec()),
            crate::DragEffects::COPY,
        )
        .unwrap();
        worker.join().unwrap();
        pump(&windows, 150);
        assert_eq!(
            outcome,
            crate::DragOutcome::Performed(crate::DropEffect::Copy)
        );
        assert_eq!(drops.lock().unwrap().last().unwrap().files, paths);
        let owner = hwnd as usize;
        let dismiss = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(350));
            unsafe {
                PostMessageW(owner as HWND, WM_CANCELMODE, 0, 0);
            }
        });
        let menu = shell::context_menu(
            hwnd,
            &crate::ShellMenuSpec {
                items: paths.to_vec(),
                application: MenuSpec::new()
                    .item("Application command", Command::custom("integration.menu")),
                position: crate::Point { x: 300, y: 460 },
            },
        )
        .unwrap();
        dismiss.join().unwrap();
        assert_eq!(menu, crate::ShellMenuOutcome::Cancelled);
        let menu_messages = windows[0]
            .services
            .as_ref()
            .unwrap()
            .menu_message_count
            .get();
        assert!(menu_messages > 0);
        // Win+D is paired even on panic; no Explorer process is terminated.
        struct RestoreDesktop(bool);
        impl Drop for RestoreDesktop {
            fn drop(&mut self) {
                if self.0 {
                    unsafe {
                        toggle_show_desktop();
                    }
                }
            }
        }
        let mut restore_desktop = RestoreDesktop(false);
        activate_target(&windows, hwnd);
        keys(
            hwnd,
            &[
                (VK_LWIN, false),
                (b'D' as u16, false),
                (b'D' as u16, true),
                (VK_LWIN, true),
            ],
        );
        restore_desktop.0 = true;
        pump(&windows, 450);
        let desktop_visible = unsafe { IsWindowVisible(desktop) != 0 && IsIconic(desktop) == 0 };
        assert!(desktop_visible);
        let desktop_capture = screen_capture(desktop, &output_ui.join("desktop-alpha.png"));
        let services = windows[1].services.as_ref().unwrap();
        let color = services
            .palette
            .get()
            .unwrap()
            .resolve(crate::ColorRole::SurfaceRaised);
        let mut rectangle: RECT = unsafe { zeroed() };
        unsafe {
            GetWindowRect(desktop, &mut rectangle);
        }
        let expected = u32::from(color.r) | (u32::from(color.g) << 8) | (u32::from(color.b) << 16);
        let (actual, under, cloaked) = unsafe {
            let dc = GetDC(null_mut());
            let point = POINT {
                x: rectangle.right - 30,
                y: rectangle.bottom - 30,
            };
            let actual = GetPixel(dc, point.x, point.y);
            ReleaseDC(null_mut(), dc);
            let under = WindowFromPoint(point);
            let mut cloaked = 0u32;
            windows_sys::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                desktop,
                windows_sys::Win32::Graphics::Dwm::DWMWA_CLOAKED as u32,
                (&mut cloaked as *mut u32).cast(),
                4,
            );
            (actual, under, cloaked)
        };
        let pixel = services
            .direct2d
            .borrow()
            .as_ref()
            .and_then(|painter| painter.proof_pixel(330, 150));
        let visible_pixel = (0..3).all(|channel| {
            (((actual >> (channel * 8)) & 255) as i32 - ((expected >> (channel * 8)) & 255) as i32)
                .abs()
                <= 3
        });
        std::fs::write(output_ui.join("desktop-check.json"),serde_json::to_vec_pretty(&serde_json::json!({"expected":expected,"actual":actual,"cloaked":cloaked,"under_pointer":format!("{under:?}"),"desktop":format!("{desktop:?}"),"rendered_pixel_bgra":pixel,"commands":window_draw_plan(desktop).map(|plan|plan.commands.len()),"visible_pixel":visible_pixel})).unwrap()).unwrap();
        assert!(
            visible_pixel,
            "desktop body pixel was not presented; inspect desktop-check.json"
        );
        unsafe {
            toggle_show_desktop();
        }
        restore_desktop.0 = false;
        pump(&windows, 300);
        unsafe {
            SendMessageW(desktop, windows_win32_taskbar_created_message(), 0, 0);
        }
        pump(&windows, 100);
        assert!(events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event, crate::NativeWindowEvent::DesktopRecreated)));
        let errors: Vec<_> = windows
            .iter()
            .filter_map(|window| {
                window
                    .services
                    .as_ref()
                    .and_then(|services| services.error.borrow().clone())
            })
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
        let report = serde_json::json!({"schema":"zsui.windows-integration-proof/v1","platform":"windows","arch":std::env::consts::ARCH,"captures":[main_capture,desktop_capture],"native_controls":["EDIT","INameSpaceTreeControl","IExplorerBrowser"],"unicode_input":true,"focus_traversal":"SendInput Tab/F6","global_hotkey":"RegisterHotKey and SendInput Ctrl+Alt+Shift+F24","filter_messages":filters.load(Ordering::SeqCst),"drop_files":2,"drop_effect":"copy","menu_messages_forwarded":menu_messages,"menu_result":"cancelled","show_desktop":desktop_visible,"shell_recovery_test":"TaskbarCreated delivered to target HWND; Explorer was not restarted","clipboard_isolation":"separate noninteractive window station in child process","interactive_clipboard_sequence_unchanged":clipboard_unchanged,"events":*events.lock().unwrap(),"errors":errors});
        std::fs::write(
            output_ui.join("report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        drop(windows);
    });
    for path in paths {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(fixture).unwrap();
}

#[test]
#[ignore = "run only in the isolated clipboard child process"]
fn isolated_file_clipboard_target_proof() {
    assert_eq!(
        std::env::var("ZSUI_CLIPBOARD_ISOLATED_CHILD").as_deref(),
        Ok("1")
    );
    use windows_sys::Win32::System::StationsAndDesktops::*;
    let output = output_root();
    let fixture = output.join("fixture");
    let paths = [fixture.join("文件 示例.txt"), fixture.join("second.txt")];
    unsafe {
        let original = GetProcessWindowStation();
        let original_desktop =
            GetThreadDesktop(windows_sys::Win32::System::Threading::GetCurrentThreadId());
        let name = wide_null(&format!("ZSUIClipboardProof{}", std::process::id()));
        let station = CreateWindowStationW(name.as_ptr(), 0, 0x10000000, null());
        assert!(
            !station.is_null(),
            "CreateWindowStationW {}",
            GetLastError()
        );
        assert_ne!(SetProcessWindowStation(station), 0);
        let desktop = CreateDesktopW(
            wide_null("Proof").as_ptr(),
            null(),
            null(),
            0,
            DESKTOP_CREATEWINDOW | DESKTOP_CREATEMENU | DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS,
            null(),
        );
        assert!(!desktop.is_null(), "CreateDesktopW {}", GetLastError());
        assert_ne!(SetThreadDesktop(desktop), 0);
        assert_eq!(GetProcessWindowStation(), station);
        {
            let _apartment = shell::OleApartment::new().unwrap();
            let values: Vec<String> = paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            windows_write_clipboard(&crate::ClipboardData::Files(values.clone())).unwrap();
            assert_eq!(
                windows_read_clipboard().unwrap(),
                Some(crate::ClipboardData::Files(values))
            );
            windows::Win32::System::Ole::OleSetClipboard(None).unwrap();
        }
        let desktop_restored = SetThreadDesktop(original_desktop) != 0;
        let station_restored = SetProcessWindowStation(original) != 0;
        let desktop_closed = CloseDesktop(desktop) != 0;
        let station_closed = CloseWindowStation(station) != 0;
        std::fs::write(output.join("clipboard.json"),serde_json::to_vec_pretty(&serde_json::json!({"round_trip":true,"files":2,"isolation":"noninteractive window station","desktop_restored":desktop_restored,"station_restored":station_restored,"desktop_handle_closed":desktop_closed,"station_handle_closed":station_closed,"remaining_process_resources_released_on_exit":true})).unwrap()).unwrap();
    }
}
