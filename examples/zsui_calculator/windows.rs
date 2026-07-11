use std::{
    env,
    ffi::c_void,
    mem,
    ptr::{null, null_mut},
};

use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, UpdateWindow, PAINTSTRUCT},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::{
            GetDpiForWindow, SetProcessDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        Input::KeyboardAndMouse::{
            ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow, DispatchMessageW,
            GetClientRect, GetMessageW, GetWindowLongPtrW, LoadCursorW, LoadImageW,
            PostQuitMessage, RegisterClassExW, SetTimer, SetWindowLongPtrW, SetWindowPos,
            ShowWindow, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
            GWLP_USERDATA, HICON, IDC_ARROW, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
            MINMAXINFO, MSG, SWP_NOACTIVATE, SWP_NOZORDER, SW_SHOW, WM_CAPTURECHANGED, WM_CHAR,
            WM_CLOSE, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_GETMINMAXINFO,
            WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY,
            WM_PAINT, WM_SETICON, WM_SIZE, WM_TIMER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use zsui::{
    Dpi, Point, Rect, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette,
    ZsCalculatorAction, ZsCalculatorBinaryOperator, ZsCalculatorEngine, ZsCalculatorInteraction,
    ZsCalculatorLayout, ZsCalculatorShellSpec, ZsuiTheme,
};

const APP_NAME: &str = "ZSUI Calculator";
const CLASS_NAME: &str = "ZSUI_CALCULATOR_WINDOW";
const AUTO_CLOSE_TIMER: usize = 1;
const WM_MOUSELEAVE: u32 = 0x02A3;
const VK_ESCAPE: usize = 0x1b;
const VK_DELETE: usize = 0x2e;
const VK_F9: usize = 0x78;

struct CalculatorState {
    hwnd: HWND,
    engine: ZsCalculatorEngine,
    history_visible: bool,
    interaction: ZsCalculatorInteraction,
    tracking_mouse: bool,
    icon: HICON,
    auto_close_ms: Option<u32>,
}

impl CalculatorState {
    fn new(auto_close_ms: Option<u32>) -> Self {
        Self {
            hwnd: null_mut(),
            engine: ZsCalculatorEngine::new(),
            history_visible: false,
            interaction: ZsCalculatorInteraction::default(),
            tracking_mouse: false,
            icon: null_mut(),
            auto_close_ms,
        }
    }

    fn shell_spec(&self) -> ZsCalculatorShellSpec {
        ZsCalculatorShellSpec::from_engine(&self.engine).history_visible(self.history_visible)
    }
}

impl Drop for CalculatorState {
    fn drop(&mut self) {
        if !self.icon.is_null() {
            unsafe {
                DestroyIcon(self.icon);
            }
        }
    }
}

pub fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let auto_close_ms = benchmark_timeout(&arguments);
    let state = Box::new(CalculatorState::new(auto_close_ms));

    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err("GetModuleHandleW failed".to_string());
    }
    register_window_class(instance)?;

    let state_ptr = Box::into_raw(state);
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            wide(CLASS_NAME).as_ptr(),
            wide(APP_NAME).as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            420,
            680,
            null_mut(),
            null_mut(),
            instance,
            state_ptr.cast::<c_void>(),
        )
    };
    if hwnd.is_null() {
        unsafe {
            drop(Box::from_raw(state_ptr));
        }
        return Err("CreateWindowExW failed".to_string());
    }
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut message: MSG = unsafe { mem::zeroed() };
    while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam as *const CREATESTRUCTW);
        let state = create.lpCreateParams.cast::<CalculatorState>();
        (*state).hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut CalculatorState;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &mut *state_ptr;

    match message {
        WM_CREATE => {
            load_icon(state);
            if let Some(delay) = state.auto_close_ms {
                SetTimer(hwnd, AUTO_CLOSE_TIMER, delay, None);
            }
            0
        }
        WM_GETMINMAXINFO => {
            let limits = &mut *(lparam as *mut MINMAXINFO);
            limits.ptMinTrackSize.x = 360;
            limits.ptMinTrackSize.y = 560;
            0
        }
        WM_SIZE => {
            invalidate(hwnd);
            0
        }
        WM_DPICHANGED => {
            let suggested = &*(lparam as *const RECT);
            SetWindowPos(
                hwnd,
                null_mut(),
                suggested.left,
                suggested.top,
                suggested.right - suggested.left,
                suggested.bottom - suggested.top,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            invalidate(hwnd);
            0
        }
        WM_PAINT => {
            paint(state);
            0
        }
        WM_ERASEBKGND => 1,
        WM_MOUSEMOVE => {
            let hovered = layout(state).action_at(point_from_lparam(lparam));
            if state.interaction.hovered != hovered {
                state.interaction.hovered = hovered;
                invalidate(hwnd);
            }
            if !state.tracking_mouse {
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                if TrackMouseEvent(&mut tracking) != 0 {
                    state.tracking_mouse = true;
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            state.tracking_mouse = false;
            state.interaction.hovered = None;
            invalidate(hwnd);
            0
        }
        WM_LBUTTONDOWN => {
            let action = layout(state).action_at(point_from_lparam(lparam));
            if action.is_some() {
                state.interaction.pressed = action;
                SetCapture(hwnd);
                invalidate(hwnd);
            }
            0
        }
        WM_LBUTTONUP => {
            let action = layout(state).action_at(point_from_lparam(lparam));
            let pressed = state.interaction.pressed.take();
            ReleaseCapture();
            if let Some(action) = action {
                if Some(action) == pressed {
                    apply_action(state, action);
                }
            }
            invalidate(hwnd);
            0
        }
        WM_CAPTURECHANGED => {
            state.interaction.pressed = None;
            invalidate(hwnd);
            0
        }
        WM_CHAR => {
            if let Some(action) = action_for_character(char::from_u32(wparam as u32)) {
                apply_action(state, action);
            }
            0
        }
        WM_KEYDOWN => {
            let action = match wparam {
                VK_ESCAPE => Some(ZsCalculatorAction::ClearAll),
                VK_DELETE => Some(ZsCalculatorAction::ClearEntry),
                VK_F9 => Some(ZsCalculatorAction::ToggleSign),
                _ => None,
            };
            if let Some(action) = action {
                apply_action(state, action);
            }
            0
        }
        WM_TIMER if wparam == AUTO_CLOSE_TIMER => {
            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(state_ptr));
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn register_window_class(instance: HINSTANCE) -> Result<(), String> {
    let class_name = wide(CLASS_NAME);
    let class = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };
    if unsafe { RegisterClassExW(&class) } == 0 {
        Err("RegisterClassExW failed".to_string())
    } else {
        Ok(())
    }
}

unsafe fn apply_action(state: &mut CalculatorState, action: ZsCalculatorAction) {
    match action {
        ZsCalculatorAction::ToggleHistory => {
            state.history_visible = !state.history_visible;
        }
        ZsCalculatorAction::ClearHistory => {
            state.engine.apply(action);
        }
        _ => {
            state.history_visible = false;
            state.engine.apply(action);
        }
    }
    state.interaction.hovered = None;
    invalidate(state.hwnd);
}

fn action_for_character(character: Option<char>) -> Option<ZsCalculatorAction> {
    match character? {
        '0'..='9' => Some(ZsCalculatorAction::Digit(
            character?.to_digit(10).expect("ASCII digit") as u8,
        )),
        '.' | ',' => Some(ZsCalculatorAction::DecimalPoint),
        '+' => Some(ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add)),
        '-' => Some(ZsCalculatorAction::Binary(
            ZsCalculatorBinaryOperator::Subtract,
        )),
        '*' => Some(ZsCalculatorAction::Binary(
            ZsCalculatorBinaryOperator::Multiply,
        )),
        '/' => Some(ZsCalculatorAction::Binary(
            ZsCalculatorBinaryOperator::Divide,
        )),
        '%' => Some(ZsCalculatorAction::Percent),
        '=' | '\r' => Some(ZsCalculatorAction::Equals),
        '\u{8}' => Some(ZsCalculatorAction::Backspace),
        _ => None,
    }
}

unsafe fn layout(state: &CalculatorState) -> ZsCalculatorLayout {
    let (surface, dpi) = surface_and_dpi(state.hwnd);
    state.shell_spec().layout(surface, dpi)
}

unsafe fn paint(state: &CalculatorState) {
    let mut paint: PAINTSTRUCT = mem::zeroed();
    let target = BeginPaint(state.hwnd, &mut paint);
    if target.is_null() {
        return;
    }
    let mut client: RECT = mem::zeroed();
    GetClientRect(state.hwnd, &mut client);
    let surface = Rect {
        x: 0,
        y: 0,
        width: (client.right - client.left).max(0),
        height: (client.bottom - client.top).max(0),
    };
    let dpi = Dpi::new(GetDpiForWindow(state.hwnd).max(96) as f32);
    let plan = state
        .shell_spec()
        .native_draw_plan(surface, dpi, state.interaction);
    let palette = WindowsGdiPalette::from_theme(&ZsuiTheme::light());
    if let Some(buffer) = WindowsBufferedPaint::begin(target, &client) {
        let mut sink = WindowsGdiDrawSink::with_palette(buffer.hdc(), palette);
        sink.draw_native_plan(&plan);
    } else {
        let mut sink = WindowsGdiDrawSink::with_palette(target, palette);
        sink.draw_native_plan(&plan);
    }
    EndPaint(state.hwnd, &paint);
}

unsafe fn surface_and_dpi(hwnd: HWND) -> (Rect, Dpi) {
    let mut client: RECT = mem::zeroed();
    GetClientRect(hwnd, &mut client);
    (
        Rect {
            x: 0,
            y: 0,
            width: (client.right - client.left).max(0),
            height: (client.bottom - client.top).max(0),
        },
        Dpi::new(GetDpiForWindow(hwnd).max(96) as f32),
    )
}

unsafe fn load_icon(state: &mut CalculatorState) {
    let icon_path = wide(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "\\assets\\calculator\\calculator.ico"
    ));
    state.icon = LoadImageW(
        null_mut(),
        icon_path.as_ptr(),
        IMAGE_ICON,
        0,
        0,
        LR_LOADFROMFILE | LR_DEFAULTSIZE,
    ) as HICON;
    if !state.icon.is_null() {
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
            state.hwnd,
            WM_SETICON,
            0,
            state.icon as LPARAM,
        );
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
            state.hwnd,
            WM_SETICON,
            1,
            state.icon as LPARAM,
        );
    }
}

unsafe fn invalidate(hwnd: HWND) {
    InvalidateRect(hwnd, null(), 0);
}

fn point_from_lparam(lparam: LPARAM) -> Point {
    let packed = lparam as u32;
    Point {
        x: (packed as u16 as i16) as i32,
        y: ((packed >> 16) as u16 as i16) as i32,
    }
}

fn benchmark_timeout(arguments: &[String]) -> Option<u32> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u32>().ok())
        .map(|seconds| seconds.saturating_mul(1000))
        .or_else(|| {
            arguments
                .iter()
                .any(|argument| argument == "--smoke")
                .then_some(1400)
        })
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
