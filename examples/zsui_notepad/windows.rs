use std::{
    env,
    ffi::c_void,
    mem,
    path::{Path, PathBuf},
    ptr::{null, null_mut},
};

use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, InvalidateRect, SetBkColor,
        SetTextColor, UpdateWindow, HBRUSH, PAINTSTRUCT,
    },
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
            GetClientRect, GetMessageW, GetWindowLongPtrW, LoadCursorW, LoadImageW, MessageBoxW,
            PostMessageW, PostQuitMessage, RegisterClassExW, SendMessageW, SetTimer,
            SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, TranslateMessage,
            CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HICON, IDCANCEL,
            IDC_ARROW, IDNO, IDYES, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MB_ICONERROR,
            MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_YESNOCANCEL, MSG, SWP_NOACTIVATE,
            SWP_NOZORDER, SW_SHOW, WM_CAPTURECHANGED, WM_CLOSE, WM_COMMAND, WM_CREATE,
            WM_CTLCOLOREDIT, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SETFOCUS,
            WM_SETICON, WM_SIZE, WM_TIMER, WNDCLASSEXW, WS_CLIPCHILDREN, WS_OVERLAPPEDWINDOW,
        },
    },
};

use zsui::{
    Color, Dpi, FileDialogService, FileDialogSpec, NativeFileDialogService, Point, Rect,
    SaveFileDialogSpec, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette,
    WindowsWin32OwnedAcceleratorTable, WindowsWin32OwnedTextEditor, ZsAccelerator,
    ZsAcceleratorKey, ZsDocumentShellCommand, ZsDocumentShellInteraction, ZsDocumentShellLayout,
    ZsDocumentShellSpec, ZsTextCursorStatus, ZsTextDocument, ZsuiTheme,
};

const APP_NAME: &str = "ZSUI Notepad";
const CLASS_NAME: &str = "ZSUI_NOTEPAD_WINDOW";

const ID_FILE_NEW: u16 = 1001;
const ID_FILE_OPEN: u16 = 1002;
const ID_FILE_SAVE: u16 = 1003;
const ID_FILE_SAVE_AS: u16 = 1004;
const ID_FILE_EXIT: u16 = 1005;
const ID_EDIT_UNDO: u16 = 1101;
const ID_EDIT_CUT: u16 = 1102;
const ID_EDIT_COPY: u16 = 1103;
const ID_EDIT_PASTE: u16 = 1104;
const ID_EDIT_SELECT_ALL: u16 = 1105;
const ID_FORMAT_WORD_WRAP: u16 = 1201;
const ID_VIEW_STATUS_BAR: u16 = 1301;
const ID_HELP_ABOUT: u16 = 1401;

const STATUS_TIMER: usize = 1;
const AUTO_CLOSE_TIMER: usize = 2;
const WM_MOUSELEAVE: u32 = 0x02A3;

struct EditorState {
    hwnd: HWND,
    editor: Option<WindowsWin32OwnedTextEditor>,
    editor_brush: HBRUSH,
    icon: HICON,
    document: ZsTextDocument,
    suppress_change: bool,
    word_wrap: bool,
    show_status: bool,
    line: usize,
    column: usize,
    character_count: usize,
    shell_interaction: ZsDocumentShellInteraction,
    tracking_mouse: bool,
    auto_close_ms: Option<u32>,
}

impl EditorState {
    fn new(document: ZsTextDocument, auto_close_ms: Option<u32>) -> Self {
        let theme = ZsuiTheme::light();
        Self {
            hwnd: null_mut(),
            editor: None,
            editor_brush: unsafe { CreateSolidBrush(colorref(theme.colors.surface_raised)) },
            icon: null_mut(),
            document,
            suppress_change: true,
            word_wrap: true,
            show_status: true,
            line: 1,
            column: 1,
            character_count: 0,
            shell_interaction: ZsDocumentShellInteraction::default(),
            tracking_mouse: false,
            auto_close_ms,
        }
    }
}

impl Drop for EditorState {
    fn drop(&mut self) {
        drop(self.editor.take());
        unsafe {
            if !self.editor_brush.is_null() {
                DeleteObject(self.editor_brush as _);
            }
            if !self.icon.is_null() {
                DestroyIcon(self.icon);
            }
        }
    }
}

pub fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let auto_close_ms = benchmark_timeout(&arguments);
    let document = initial_document(&arguments)?;
    let state = Box::new(EditorState::new(document, auto_close_ms));

    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err("GetModuleHandleW failed".to_string());
    }
    register_window_class(instance)?;
    let title = window_title(&state.document);
    let class_name = wide(CLASS_NAME);
    let title = wide(&title);
    let state_ptr = Box::into_raw(state);
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            900,
            620,
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

    let accelerators = create_accelerators()?;
    let mut message: MSG = unsafe { mem::zeroed() };
    while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {
        if !accelerators.translate(hwnd, &message) {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
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
        let state = create.lpCreateParams.cast::<EditorState>();
        (*state).hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut EditorState;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &mut *state_ptr;

    match message {
        WM_CREATE => match create_children(state) {
            Ok(()) => 0,
            Err(error) => {
                show_error(hwnd, &error);
                -1
            }
        },
        WM_SIZE => {
            layout_children(state);
            invalidate_shell(state.hwnd);
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
            let dpi = ((wparam >> 16) as u32).max(96);
            if let Some(editor) = state.editor.as_mut() {
                if let Err(error) = editor.apply_dpi(Dpi::new(dpi as f32)) {
                    show_error(hwnd, &error.to_string());
                }
            }
            layout_children(state);
            invalidate_shell(hwnd);
            0
        }
        WM_SETFOCUS => {
            editor(state).focus();
            0
        }
        WM_COMMAND => {
            handle_command(state, wparam, lparam);
            0
        }
        WM_TIMER => {
            if wparam == STATUS_TIMER {
                update_status(state);
            } else if wparam == AUTO_CLOSE_TIMER {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_CTLCOLOREDIT => {
            let theme = ZsuiTheme::light();
            SetTextColor(wparam as _, colorref(theme.colors.text_primary));
            SetBkColor(wparam as _, colorref(theme.colors.surface_raised));
            state.editor_brush as LRESULT
        }
        WM_PAINT => {
            paint_shell(state);
            0
        }
        WM_ERASEBKGND => 1,
        WM_MOUSEMOVE => {
            let point = point_from_lparam(lparam);
            let hovered = shell_layout(state).command_at(point);
            if state.shell_interaction.hovered != hovered {
                state.shell_interaction.hovered = hovered;
                invalidate_shell(hwnd);
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
            state.shell_interaction.hovered = None;
            invalidate_shell(hwnd);
            0
        }
        WM_LBUTTONDOWN => {
            let command = shell_layout(state).command_at(point_from_lparam(lparam));
            if command.is_some() {
                state.shell_interaction.pressed = command;
                SetCapture(hwnd);
                invalidate_shell(hwnd);
            }
            0
        }
        WM_LBUTTONUP => {
            let command = shell_layout(state).command_at(point_from_lparam(lparam));
            let pressed = state.shell_interaction.pressed.take();
            ReleaseCapture();
            invalidate_shell(hwnd);
            if command.is_some() && command == pressed {
                handle_shell_command(state, command.expect("command checked above"));
            }
            0
        }
        WM_CAPTURECHANGED => {
            state.shell_interaction.pressed = None;
            invalidate_shell(hwnd);
            0
        }
        WM_CLOSE => {
            if confirm_discard_or_save(state) {
                DestroyWindow(hwnd);
            }
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
        return Err("RegisterClassExW failed".to_string());
    }
    Ok(())
}

unsafe fn create_children(state: &mut EditorState) -> Result<(), String> {
    state.editor = Some(
        WindowsWin32OwnedTextEditor::create(
            state.hwnd,
            state.document.text(),
            state.word_wrap,
            Dpi::new(GetDpiForWindow(state.hwnd).max(96) as f32),
        )
        .map_err(|error| error.to_string())?,
    );

    let icon_path = wide(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "\\assets\\notepad\\notepad.ico"
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
        SendMessageW(state.hwnd, WM_SETICON, 0, state.icon as LPARAM);
        SendMessageW(state.hwnd, WM_SETICON, 1, state.icon as LPARAM);
    }

    state.suppress_change = false;
    SetTimer(state.hwnd, STATUS_TIMER, 200, None);
    if let Some(delay) = state.auto_close_ms {
        SetTimer(state.hwnd, AUTO_CLOSE_TIMER, delay, None);
    }
    layout_children(state);
    update_title(state);
    update_status(state);
    Ok(())
}

unsafe fn layout_children(state: &EditorState) {
    let bounds = shell_layout(state).editor_content;
    let _ = editor(state).set_bounds(bounds);
}

unsafe fn handle_command(state: &mut EditorState, wparam: WPARAM, lparam: LPARAM) {
    let id = (wparam & 0xffff) as u16;
    if editor(state).is_change_notification(wparam, lparam) {
        if !state.suppress_change {
            state.document.replace_text(editor(state).text());
            update_title(state);
            update_status(state);
        }
        return;
    }

    match id {
        ID_FILE_NEW => new_document(state),
        ID_FILE_OPEN => open_document(state),
        ID_FILE_SAVE => {
            save_document(state, false);
        }
        ID_FILE_SAVE_AS => {
            save_document(state, true);
        }
        ID_FILE_EXIT => {
            PostMessageW(state.hwnd, WM_CLOSE, 0, 0);
        }
        ID_EDIT_UNDO => {
            editor(state).undo();
        }
        ID_EDIT_CUT => {
            editor(state).cut();
        }
        ID_EDIT_COPY => {
            editor(state).copy();
        }
        ID_EDIT_PASTE => {
            editor(state).paste();
        }
        ID_EDIT_SELECT_ALL => {
            editor(state).select_all();
        }
        ID_FORMAT_WORD_WRAP => toggle_word_wrap(state),
        ID_VIEW_STATUS_BAR => toggle_status_bar(state),
        ID_HELP_ABOUT => {
            MessageBoxW(
                state.hwnd,
                wide("A native text editor benchmark built with ZSUI host contracts and Windows text services.").as_ptr(),
                wide("About ZSUI Notepad").as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        _ => {}
    }
}

unsafe fn handle_shell_command(state: &mut EditorState, command: ZsDocumentShellCommand) {
    match command {
        ZsDocumentShellCommand::New => new_document(state),
        ZsDocumentShellCommand::Close => {
            PostMessageW(state.hwnd, WM_CLOSE, 0, 0);
        }
        ZsDocumentShellCommand::Open => open_document(state),
        ZsDocumentShellCommand::Save => {
            save_document(state, false);
        }
        ZsDocumentShellCommand::SaveAs => {
            save_document(state, true);
        }
        ZsDocumentShellCommand::Undo => {
            editor(state).focus();
            editor(state).undo();
        }
        ZsDocumentShellCommand::Cut => {
            editor(state).focus();
            editor(state).cut();
        }
        ZsDocumentShellCommand::Copy => {
            editor(state).focus();
            editor(state).copy();
        }
        ZsDocumentShellCommand::Paste => {
            editor(state).focus();
            editor(state).paste();
        }
        ZsDocumentShellCommand::ToggleWrap => toggle_word_wrap(state),
        ZsDocumentShellCommand::ToggleStatus => toggle_status_bar(state),
        ZsDocumentShellCommand::About => {
            MessageBoxW(
                state.hwnd,
                wide("ZSUI Notepad combines a buffered Fluent document shell with the native Windows text service.").as_ptr(),
                wide("About ZSUI Notepad").as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
    invalidate_shell(state.hwnd);
}

unsafe fn new_document(state: &mut EditorState) {
    if !confirm_discard_or_save(state) {
        return;
    }
    state.document = ZsTextDocument::untitled("");
    replace_editor_text(state);
}

unsafe fn open_document(state: &mut EditorState) {
    if !confirm_discard_or_save(state) {
        return;
    }
    let Some(path) = choose_file(state.hwnd, false, state.document.path()) else {
        return;
    };
    match ZsTextDocument::open(path) {
        Ok(document) => {
            state.document = document;
            replace_editor_text(state);
        }
        Err(error) => show_error(state.hwnd, &error.to_string()),
    }
}

unsafe fn save_document(state: &mut EditorState, force_picker: bool) -> bool {
    state.document.replace_text(editor(state).text());
    let result = if force_picker || state.document.path().is_none() {
        let Some(path) = choose_file(state.hwnd, true, state.document.path()) else {
            return false;
        };
        state.document.save_as(path)
    } else {
        state.document.save()
    };
    match result {
        Ok(()) => {
            update_title(state);
            update_status(state);
            true
        }
        Err(error) => {
            show_error(state.hwnd, &error.to_string());
            false
        }
    }
}

unsafe fn confirm_discard_or_save(state: &mut EditorState) -> bool {
    if !state.document.is_dirty() {
        return true;
    }
    let message = format!("Save changes to {}?", state.document.display_name());
    match MessageBoxW(
        state.hwnd,
        wide(&message).as_ptr(),
        wide(APP_NAME).as_ptr(),
        MB_YESNOCANCEL | MB_ICONWARNING,
    ) {
        IDYES => save_document(state, false),
        IDNO => true,
        IDCANCEL => false,
        _ => false,
    }
}

unsafe fn replace_editor_text(state: &mut EditorState) {
    state.suppress_change = true;
    if let Err(error) = editor(state).replace_text(state.document.text()) {
        show_error(state.hwnd, &error.to_string());
    }
    state.suppress_change = false;
    update_title(state);
    update_status(state);
}

unsafe fn toggle_word_wrap(state: &mut EditorState) {
    let word_wrap = !state.word_wrap;
    if let Err(error) = editor(state).set_word_wrap(word_wrap) {
        show_error(state.hwnd, &error.to_string());
        return;
    }
    state.word_wrap = word_wrap;
    update_status(state);
    invalidate_shell(state.hwnd);
}

unsafe fn toggle_status_bar(state: &mut EditorState) {
    state.show_status = !state.show_status;
    layout_children(state);
    invalidate_shell(state.hwnd);
}

unsafe fn update_title(state: &EditorState) {
    SetWindowTextW(state.hwnd, wide(&window_title(&state.document)).as_ptr());
    invalidate_shell(state.hwnd);
}

unsafe fn update_status(state: &mut EditorState) {
    let editor = editor(state);
    let text = editor.text();
    let status = ZsTextCursorStatus::from_utf16_caret(&text, editor.selection_utf16().0);
    if (state.line, state.column, state.character_count)
        != (status.line, status.column, status.character_count)
    {
        state.line = status.line;
        state.column = status.column;
        state.character_count = status.character_count;
        invalidate_shell(state.hwnd);
    }
}

fn shell_spec(state: &EditorState) -> ZsDocumentShellSpec {
    ZsDocumentShellSpec::new(APP_NAME, state.document.display_name())
        .dirty(state.document.is_dirty())
        .word_wrap(state.word_wrap)
        .show_status(state.show_status)
        .status(state.line, state.column, state.character_count)
        .encoding(state.document.encoding().label())
}

unsafe fn shell_layout(state: &EditorState) -> ZsDocumentShellLayout {
    let mut client: RECT = mem::zeroed();
    GetClientRect(state.hwnd, &mut client);
    shell_spec(state).layout(
        Rect {
            x: 0,
            y: 0,
            width: (client.right - client.left).max(0),
            height: (client.bottom - client.top).max(0),
        },
        Dpi::new(GetDpiForWindow(state.hwnd).max(96) as f32),
    )
}

unsafe fn paint_shell(state: &EditorState) {
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
    let plan = shell_spec(state).native_draw_plan(surface, dpi, state.shell_interaction);
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

unsafe fn invalidate_shell(hwnd: HWND) {
    InvalidateRect(hwnd, null(), 0);
}

fn editor(state: &EditorState) -> &WindowsWin32OwnedTextEditor {
    state
        .editor
        .as_ref()
        .expect("native text editor is created during WM_CREATE")
}

fn point_from_lparam(lparam: LPARAM) -> Point {
    let packed = lparam as u32;
    Point {
        x: (packed as u16 as i16) as i32,
        y: ((packed >> 16) as u16 as i16) as i32,
    }
}

fn create_accelerators() -> Result<WindowsWin32OwnedAcceleratorTable, String> {
    let bindings = [
        (ID_FILE_NEW, ZsAccelerator::primary_character('N')),
        (ID_FILE_OPEN, ZsAccelerator::primary_character('O')),
        (ID_FILE_SAVE, ZsAccelerator::primary_character('S')),
        (
            ID_FILE_SAVE_AS,
            ZsAccelerator::primary_character('S').shifted(),
        ),
        (ID_EDIT_SELECT_ALL, ZsAccelerator::primary_character('A')),
        (
            ID_HELP_ABOUT,
            ZsAccelerator::new(ZsAcceleratorKey::Function(1)),
        ),
    ];
    WindowsWin32OwnedAcceleratorTable::from_bindings(&bindings).map_err(|error| error.to_string())
}

unsafe fn choose_file(hwnd: HWND, save: bool, current: Option<&Path>) -> Option<PathBuf> {
    let mut dialogs = NativeFileDialogService::new();
    let result = if save {
        let mut spec = SaveFileDialogSpec::new("Save text document")
            .filter("Text files", ["*.txt", "*.md", "*.log"])
            .filter("All files", ["*.*"]);
        if let Some(current) = current {
            spec = spec.current_path(current);
        }
        dialogs.save_file_dialog(&spec)
    } else {
        let mut spec = FileDialogSpec::new("Open text document")
            .filter("Text files", ["*.txt", "*.md", "*.log"])
            .filter("All files", ["*.*"]);
        if let Some(current) = current {
            spec = spec.current_path(current);
        }
        dialogs
            .open_file_dialog(&spec)
            .map(|paths| paths.and_then(|paths| paths.into_iter().next()))
    };
    match result {
        Ok(path) => path,
        Err(error) => {
            show_error(hwnd, &error.to_string());
            None
        }
    }
}

fn initial_document(arguments: &[String]) -> Result<ZsTextDocument, String> {
    let path = arguments
        .windows(2)
        .find(|pair| pair[0] == "--open")
        .map(|pair| PathBuf::from(&pair[1]));
    match path {
        Some(path) => ZsTextDocument::open(path).map_err(|error| error.to_string()),
        None => Ok(ZsTextDocument::untitled(
            "ZSUI Notepad\r\n\r\nA complete native text editing benchmark.\r\n",
        )),
    }
}

fn benchmark_timeout(arguments: &[String]) -> Option<u32> {
    if arguments.iter().any(|argument| argument == "--smoke") {
        return Some(1200);
    }
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u32>().ok())
        .map(|seconds| seconds.saturating_mul(1000))
}

fn window_title(document: &ZsTextDocument) -> String {
    format!(
        "{}{} - {APP_NAME}",
        if document.is_dirty() { "*" } else { "" },
        document.display_name()
    )
}

unsafe fn show_error(hwnd: HWND, error: &str) {
    MessageBoxW(
        hwnd,
        wide(error).as_ptr(),
        wide(APP_NAME).as_ptr(),
        MB_OK | MB_ICONERROR,
    );
}

const fn colorref(color: Color) -> u32 {
    color.r as u32 | ((color.g as u32) << 8) | ((color.b as u32) << 16)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}
