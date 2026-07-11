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
        BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, InvalidateRect,
        SetBkColor, SetTextColor, UpdateWindow, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
        DEFAULT_PITCH, FF_DONTCARE, FW_NORMAL, HBRUSH, HFONT, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Controls::{
            Dialogs::{
                GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST,
                OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
            },
            EM_GETSEL, EM_SETLIMITTEXT, EM_SETMARGINS, EM_SETSEL,
        },
        HiDpi::{
            GetDpiForWindow, SetProcessDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        Input::KeyboardAndMouse::{
            ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
            VK_F1,
        },
        WindowsAndMessaging::{
            CreateAcceleratorTableW, CreateWindowExW, DefWindowProcW, DestroyAcceleratorTable,
            DestroyIcon, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
            GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, LoadCursorW, LoadImageW,
            MessageBoxW, MoveWindow, PostMessageW, PostQuitMessage, RegisterClassExW, SendMessageW,
            SetTimer, SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow,
            TranslateAcceleratorW, TranslateMessage, ACCEL, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, EN_CHANGE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE,
            ES_NOHIDESEL, ES_WANTRETURN, FCONTROL, FSHIFT, FVIRTKEY, GWLP_USERDATA, GWL_STYLE,
            HACCEL, HICON, IDCANCEL, IDC_ARROW, IDNO, IDYES, IMAGE_ICON, LR_DEFAULTSIZE,
            LR_LOADFROMFILE, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK,
            MB_YESNOCANCEL, MSG, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SW_SHOW, WM_CAPTURECHANGED, WM_CLOSE, WM_COMMAND, WM_COPY, WM_CREATE,
            WM_CTLCOLOREDIT, WM_CUT, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_PASTE, WM_SETFOCUS,
            WM_SETFONT, WM_SETICON, WM_SIZE, WM_TIMER, WM_UNDO, WNDCLASSEXW, WS_CHILD,
            WS_CLIPCHILDREN, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
        },
    },
};

use zsui::{
    Color, Dpi, Point, Rect, WindowsBufferedPaint, WindowsGdiDrawSink, WindowsGdiPalette,
    ZsDocumentShellCommand, ZsDocumentShellInteraction, ZsDocumentShellLayout, ZsDocumentShellSpec,
    ZsuiTheme,
};

use super::document::Document;

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
    edit: HWND,
    font: HFONT,
    editor_brush: HBRUSH,
    icon: HICON,
    document: Document,
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
    fn new(document: Document, auto_close_ms: Option<u32>) -> Self {
        let theme = ZsuiTheme::light();
        Self {
            hwnd: null_mut(),
            edit: null_mut(),
            font: null_mut(),
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
        unsafe {
            if !self.font.is_null() {
                DeleteObject(self.font as _);
            }
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
        if unsafe { TranslateAcceleratorW(hwnd, accelerators, &message) } == 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
    unsafe {
        DestroyAcceleratorTable(accelerators);
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
            apply_editor_font(state, dpi);
            layout_children(state);
            invalidate_shell(hwnd);
            0
        }
        WM_SETFOCUS => {
            SetFocus(state.edit);
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
                state.document.dirty = false;
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
    let edit_class = wide("EDIT");
    let initial = wide(&state.document.text);
    state.edit = CreateWindowExW(
        0,
        edit_class.as_ptr(),
        initial.as_ptr(),
        WS_CHILD
            | WS_VISIBLE
            | WS_TABSTOP
            | WS_VSCROLL
            | ES_LEFT as u32
            | ES_MULTILINE as u32
            | ES_AUTOVSCROLL as u32
            | ES_WANTRETURN as u32
            | ES_NOHIDESEL as u32,
        0,
        0,
        0,
        0,
        state.hwnd,
        null_mut(),
        GetModuleHandleW(null()),
        null_mut(),
    );
    if state.edit.is_null() {
        return Err("failed to create the multiline editor".to_string());
    }

    apply_editor_font(state, GetDpiForWindow(state.hwnd).max(96));
    SendMessageW(state.edit, EM_SETLIMITTEXT, 0x7fff_ffff, 0);

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
    let editor = shell_layout(state).editor_content;
    MoveWindow(
        state.edit,
        editor.x,
        editor.y,
        editor.width,
        editor.height,
        1,
    );
}

unsafe fn handle_command(state: &mut EditorState, wparam: WPARAM, lparam: LPARAM) {
    let id = (wparam & 0xffff) as u16;
    let notification = ((wparam >> 16) & 0xffff) as u16;
    if lparam as HWND == state.edit && notification as u32 == EN_CHANGE {
        if !state.suppress_change {
            state.document.text = editor_text(state.edit);
            state.document.dirty = true;
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
            SendMessageW(state.edit, WM_UNDO, 0, 0);
        }
        ID_EDIT_CUT => {
            SendMessageW(state.edit, WM_CUT, 0, 0);
        }
        ID_EDIT_COPY => {
            SendMessageW(state.edit, WM_COPY, 0, 0);
        }
        ID_EDIT_PASTE => {
            SendMessageW(state.edit, WM_PASTE, 0, 0);
        }
        ID_EDIT_SELECT_ALL => {
            SendMessageW(state.edit, EM_SETSEL, 0, -1);
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
            SetFocus(state.edit);
            SendMessageW(state.edit, WM_UNDO, 0, 0);
        }
        ZsDocumentShellCommand::Cut => {
            SetFocus(state.edit);
            SendMessageW(state.edit, WM_CUT, 0, 0);
        }
        ZsDocumentShellCommand::Copy => {
            SetFocus(state.edit);
            SendMessageW(state.edit, WM_COPY, 0, 0);
        }
        ZsDocumentShellCommand::Paste => {
            SetFocus(state.edit);
            SendMessageW(state.edit, WM_PASTE, 0, 0);
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
    state.document = Document::untitled("");
    replace_editor_text(state);
}

unsafe fn open_document(state: &mut EditorState) {
    if !confirm_discard_or_save(state) {
        return;
    }
    let Some(path) = choose_file(state.hwnd, false, state.document.path.as_deref()) else {
        return;
    };
    match Document::open(path) {
        Ok(document) => {
            state.document = document;
            replace_editor_text(state);
        }
        Err(error) => show_error(state.hwnd, &error),
    }
}

unsafe fn save_document(state: &mut EditorState, force_picker: bool) -> bool {
    state.document.text = editor_text(state.edit);
    let result = if force_picker || state.document.path.is_none() {
        let Some(path) = choose_file(state.hwnd, true, state.document.path.as_deref()) else {
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
            show_error(state.hwnd, &error);
            false
        }
    }
}

unsafe fn confirm_discard_or_save(state: &mut EditorState) -> bool {
    if !state.document.dirty {
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
    SetWindowTextW(state.edit, wide(&state.document.text).as_ptr());
    SendMessageW(state.edit, EM_SETSEL, 0, 0);
    state.suppress_change = false;
    state.document.dirty = false;
    update_title(state);
    update_status(state);
}

unsafe fn toggle_word_wrap(state: &mut EditorState) {
    state.word_wrap = !state.word_wrap;
    let mut style = GetWindowLongPtrW(state.edit, GWL_STYLE) as u32;
    if state.word_wrap {
        style &= !(ES_AUTOHSCROLL as u32);
        style &= !WS_HSCROLL;
    } else {
        style |= ES_AUTOHSCROLL as u32;
        style |= WS_HSCROLL;
    }
    SetWindowLongPtrW(state.edit, GWL_STYLE, style as isize);
    SetWindowPos(
        state.edit,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
    );
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
    let text = editor_text(state.edit);
    let mut selection_start = 0u32;
    let mut selection_end = 0u32;
    SendMessageW(
        state.edit,
        EM_GETSEL,
        (&mut selection_start as *mut u32) as usize,
        (&mut selection_end as *mut u32) as isize,
    );
    let wide_text = text.encode_utf16().collect::<Vec<_>>();
    let caret = (selection_start as usize).min(wide_text.len());
    let prefix = String::from_utf16_lossy(&wide_text[..caret]);
    let line = prefix.chars().filter(|ch| *ch == '\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map(|(_, tail)| tail.chars().count() + 1)
        .unwrap_or_else(|| prefix.chars().count() + 1);
    let character_count = text.chars().count();
    if (state.line, state.column, state.character_count) != (line, column, character_count) {
        state.line = line;
        state.column = column;
        state.character_count = character_count;
        invalidate_shell(state.hwnd);
    }
}

unsafe fn editor_text(edit: HWND) -> String {
    let length = GetWindowTextLengthW(edit).max(0) as usize;
    let mut buffer = vec![0u16; length + 1];
    let copied = GetWindowTextW(edit, buffer.as_mut_ptr(), buffer.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&buffer[..copied])
}

fn shell_spec(state: &EditorState) -> ZsDocumentShellSpec {
    ZsDocumentShellSpec::new(APP_NAME, state.document.display_name())
        .dirty(state.document.dirty)
        .word_wrap(state.word_wrap)
        .show_status(state.show_status)
        .status(state.line, state.column, state.character_count)
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

unsafe fn apply_editor_font(state: &mut EditorState, dpi: u32) {
    let font_name = wide("Segoe UI Variable Text");
    let height = -((((17 * dpi.max(96)) + 48) / 96) as i32);
    let font = CreateFontW(
        height,
        0,
        0,
        0,
        FW_NORMAL as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET.into(),
        OUT_DEFAULT_PRECIS.into(),
        CLIP_DEFAULT_PRECIS.into(),
        5,
        (DEFAULT_PITCH | FF_DONTCARE).into(),
        font_name.as_ptr(),
    );
    if !font.is_null() {
        SendMessageW(state.edit, WM_SETFONT, font as usize, 1);
        let previous = mem::replace(&mut state.font, font);
        if !previous.is_null() {
            DeleteObject(previous as _);
        }
    }
    let margin = (((10 * dpi.max(96)) + 48) / 96).min(u16::MAX as u32);
    SendMessageW(
        state.edit,
        EM_SETMARGINS,
        3,
        (margin | (margin << 16)) as LPARAM,
    );
}

unsafe fn invalidate_shell(hwnd: HWND) {
    InvalidateRect(hwnd, null(), 0);
}

fn point_from_lparam(lparam: LPARAM) -> Point {
    let packed = lparam as u32;
    Point {
        x: (packed as u16 as i16) as i32,
        y: ((packed >> 16) as u16 as i16) as i32,
    }
}

fn create_accelerators() -> Result<HACCEL, String> {
    let mut accelerators = [
        accelerator(FCONTROL | FVIRTKEY, b'N', ID_FILE_NEW),
        accelerator(FCONTROL | FVIRTKEY, b'O', ID_FILE_OPEN),
        accelerator(FCONTROL | FVIRTKEY, b'S', ID_FILE_SAVE),
        accelerator(FCONTROL | FSHIFT | FVIRTKEY, b'S', ID_FILE_SAVE_AS),
        accelerator(FCONTROL | FVIRTKEY, b'A', ID_EDIT_SELECT_ALL),
        accelerator(FVIRTKEY, VK_F1 as u8, ID_HELP_ABOUT),
    ];
    let handle =
        unsafe { CreateAcceleratorTableW(accelerators.as_mut_ptr(), accelerators.len() as i32) };
    if handle.is_null() {
        Err("CreateAcceleratorTableW failed".to_string())
    } else {
        Ok(handle)
    }
}

const fn accelerator(flags: u8, key: u8, command: u16) -> ACCEL {
    ACCEL {
        fVirt: flags,
        key: key as u16,
        cmd: command,
    }
}

fn choose_file(hwnd: HWND, save: bool, current: Option<&Path>) -> Option<PathBuf> {
    let mut path_buffer = [0u16; 32768];
    if let Some(path) = current {
        let value = wide(&path.to_string_lossy());
        let count = value.len().saturating_sub(1).min(path_buffer.len() - 1);
        path_buffer[..count].copy_from_slice(&value[..count]);
    }
    let filter = wide_with_embedded_nuls("Text files\0*.txt;*.md;*.log\0All files\0*.*\0\0");
    let extension = wide("txt");
    let mut dialog: OPENFILENAMEW = unsafe { mem::zeroed() };
    dialog.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = hwnd;
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = path_buffer.as_mut_ptr();
    dialog.nMaxFile = path_buffer.len() as u32;
    dialog.lpstrDefExt = extension.as_ptr();
    dialog.Flags = OFN_EXPLORER
        | OFN_PATHMUSTEXIST
        | if save {
            OFN_OVERWRITEPROMPT
        } else {
            OFN_FILEMUSTEXIST
        };
    let accepted = unsafe {
        if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        }
    };
    if accepted == 0 {
        return None;
    }
    let length = path_buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(path_buffer.len());
    Some(PathBuf::from(String::from_utf16_lossy(
        &path_buffer[..length],
    )))
}

fn initial_document(arguments: &[String]) -> Result<Document, String> {
    let path = arguments
        .windows(2)
        .find(|pair| pair[0] == "--open")
        .map(|pair| PathBuf::from(&pair[1]));
    match path {
        Some(path) => Document::open(path),
        None => Ok(Document::untitled(
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

fn window_title(document: &Document) -> String {
    format!(
        "{}{} - {APP_NAME}",
        if document.dirty { "*" } else { "" },
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

fn wide_with_embedded_nuls(value: &str) -> Vec<u16> {
    value.chars().map(|character| character as u16).collect()
}
