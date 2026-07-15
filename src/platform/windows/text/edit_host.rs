use std::ptr::{null, null_mut};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::{
        CreateFontW, DeleteObject, GetDC, ReleaseDC, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
        DEFAULT_PITCH, FF_DONTCARE, FW_NORMAL, HFONT, OUT_DEFAULT_PRECIS,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Controls::{EM_GETSEL, EM_SETLIMITTEXT, EM_SETMARGINS, EM_SETSEL},
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetWindowLongPtrW, GetWindowTextLengthW,
            GetWindowTextW, IsWindow, MoveWindow, SendMessageW, SetWindowLongPtrW, SetWindowPos,
            SetWindowTextW, EN_CHANGE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE,
            ES_NOHIDESEL, ES_WANTRETURN, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
            WM_COPY, WM_CUT, WM_PASTE, WM_SETFONT, WM_UNDO, WS_CHILD, WS_HSCROLL, WS_TABSTOP,
            WS_VISIBLE, WS_VSCROLL,
        },
    },
};

use crate::{Dpi, Rect, ZsuiError, ZsuiResult};

/// Owns one multiline Win32 EDIT child and its DPI-scaled font resource.
#[derive(Debug)]
pub struct WindowsWin32OwnedTextEditor {
    handle: HWND,
    font: HFONT,
}

impl WindowsWin32OwnedTextEditor {
    pub fn create(owner: HWND, initial_text: &str, word_wrap: bool, dpi: Dpi) -> ZsuiResult<Self> {
        if owner.is_null() {
            return Err(ZsuiError::invalid_spec(
                "text_editor.owner",
                "Win32 text editor owner cannot be null",
            ));
        }
        let instance = unsafe { GetModuleHandleW(null()) };
        if instance.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_text_editor.module",
                "GetModuleHandleW failed",
            ));
        }
        let class = wide_null("EDIT");
        let text = wide_null(initial_text);
        let handle = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                text.as_ptr(),
                text_editor_style(word_wrap),
                0,
                0,
                0,
                0,
                owner,
                null_mut(),
                instance,
                null_mut(),
            )
        };
        if handle.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_text_editor.create",
                "CreateWindowExW failed for the multiline EDIT control",
            ));
        }

        let mut editor = Self {
            handle,
            font: null_mut(),
        };
        unsafe {
            SendMessageW(editor.handle, EM_SETLIMITTEXT, 0x7fff_ffff, 0);
        }
        editor.apply_dpi(dpi)?;
        Ok(editor)
    }

    pub fn is_change_notification(&self, wparam: WPARAM, lparam: LPARAM) -> bool {
        lparam as HWND == self.handle && ((wparam >> 16) & 0xffff) as u32 == EN_CHANGE
    }

    pub fn focus(&self) {
        unsafe {
            SetFocus(self.handle);
        }
    }

    pub fn set_bounds(&self, bounds: Rect) -> ZsuiResult<()> {
        if unsafe {
            MoveWindow(
                self.handle,
                bounds.x,
                bounds.y,
                bounds.width.max(0),
                bounds.height.max(0),
                1,
            )
        } == 0
        {
            Err(ZsuiError::host(
                "windows_win32_text_editor.bounds",
                "MoveWindow failed for the multiline EDIT control",
            ))
        } else {
            Ok(())
        }
    }

    pub fn text(&self) -> String {
        let length = unsafe { GetWindowTextLengthW(self.handle) }.max(0) as usize;
        let mut buffer = vec![0u16; length + 1];
        let copied =
            unsafe { GetWindowTextW(self.handle, buffer.as_mut_ptr(), buffer.len() as i32) }.max(0)
                as usize;
        String::from_utf16_lossy(&buffer[..copied])
    }

    pub fn replace_text(&self, text: &str) -> ZsuiResult<()> {
        let text = wide_null(text);
        if unsafe { SetWindowTextW(self.handle, text.as_ptr()) } == 0 {
            return Err(ZsuiError::host(
                "windows_win32_text_editor.text",
                "SetWindowTextW failed for the multiline EDIT control",
            ));
        }
        unsafe {
            SendMessageW(self.handle, EM_SETSEL, 0, 0);
        }
        Ok(())
    }

    pub fn selection_utf16(&self) -> (usize, usize) {
        let mut start = 0u32;
        let mut end = 0u32;
        unsafe {
            SendMessageW(
                self.handle,
                EM_GETSEL,
                (&mut start as *mut u32) as usize,
                (&mut end as *mut u32) as isize,
            );
        }
        (start as usize, end as usize)
    }

    pub fn set_word_wrap(&self, word_wrap: bool) -> ZsuiResult<()> {
        let mut style = unsafe { GetWindowLongPtrW(self.handle, GWL_STYLE) } as u32;
        if word_wrap {
            style &= !(ES_AUTOHSCROLL as u32);
            style &= !WS_HSCROLL;
        } else {
            style |= ES_AUTOHSCROLL as u32;
            style |= WS_HSCROLL;
        }
        unsafe {
            SetWindowLongPtrW(self.handle, GWL_STYLE, style as isize);
        }
        if unsafe {
            SetWindowPos(
                self.handle,
                null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
            )
        } == 0
        {
            Err(ZsuiError::host(
                "windows_win32_text_editor.word_wrap",
                "SetWindowPos failed while updating word-wrap style",
            ))
        } else {
            Ok(())
        }
    }

    pub fn apply_dpi(&mut self, dpi: Dpi) -> ZsuiResult<()> {
        let dpi = dpi.0.round().max(96.0) as u32;
        let dc = unsafe { GetDC(self.handle) };
        let font_family = crate::windows_gdi_renderer::windows_ui_text_font_family(dc);
        if !dc.is_null() {
            unsafe {
                ReleaseDC(self.handle, dc);
            }
        }
        let font_name = wide_null(font_family);
        let height = -((((14 * dpi) + 48) / 96) as i32);
        let font = unsafe {
            CreateFontW(
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
            )
        };
        if font.is_null() {
            return Err(ZsuiError::host(
                "windows_win32_text_editor.font",
                "CreateFontW failed for the native text editor",
            ));
        }
        unsafe {
            SendMessageW(self.handle, WM_SETFONT, font as usize, 1);
        }
        let previous = std::mem::replace(&mut self.font, font);
        if !previous.is_null() {
            unsafe {
                DeleteObject(previous as _);
            }
        }

        let margin = (((10 * dpi) + 48) / 96).min(u16::MAX as u32);
        unsafe {
            SendMessageW(
                self.handle,
                EM_SETMARGINS,
                3,
                (margin | (margin << 16)) as LPARAM,
            );
        }
        Ok(())
    }

    pub fn undo(&self) {
        self.send_edit_message(WM_UNDO);
    }

    pub fn cut(&self) {
        self.send_edit_message(WM_CUT);
    }

    pub fn copy(&self) {
        self.send_edit_message(WM_COPY);
    }

    pub fn paste(&self) {
        self.send_edit_message(WM_PASTE);
    }

    pub fn select_all(&self) {
        unsafe {
            SendMessageW(self.handle, EM_SETSEL, 0, -1);
        }
    }

    fn send_edit_message(&self, message: u32) {
        unsafe {
            SendMessageW(self.handle, message, 0, 0);
        }
    }
}

impl Drop for WindowsWin32OwnedTextEditor {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() && IsWindow(self.handle) != 0 {
                DestroyWindow(self.handle);
            }
            if !self.font.is_null() {
                DeleteObject(self.font as _);
            }
        }
    }
}

fn text_editor_style(word_wrap: bool) -> u32 {
    let mut style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_VSCROLL
        | ES_LEFT as u32
        | ES_MULTILINE as u32
        | ES_AUTOVSCROLL as u32
        | ES_WANTRETURN as u32
        | ES_NOHIDESEL as u32;
    if !word_wrap {
        style |= WS_HSCROLL | ES_AUTOHSCROLL as u32;
    }
    style
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_wrap_controls_horizontal_scroll_styles() {
        assert_eq!(text_editor_style(true) & WS_HSCROLL, 0);
        assert_eq!(text_editor_style(true) & ES_AUTOHSCROLL as u32, 0);
        assert_ne!(text_editor_style(false) & WS_HSCROLL, 0);
        assert_ne!(text_editor_style(false) & ES_AUTOHSCROLL as u32, 0);
    }

    #[test]
    fn null_owner_is_rejected_before_native_creation() {
        let error = WindowsWin32OwnedTextEditor::create(null_mut(), "", true, Dpi::standard())
            .expect_err("null owner must be invalid");

        assert!(matches!(
            error,
            ZsuiError::InvalidSpec { field, .. } if field == "text_editor.owner"
        ));
    }
}
