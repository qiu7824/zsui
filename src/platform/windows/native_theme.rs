// Appearance propagation for native child HWNDs and native menus.

struct NativeControlBrushes {
    surface: windows_sys::Win32::Graphics::Gdi::HBRUSH,
    control: windows_sys::Win32::Graphics::Gdi::HBRUSH,
}
impl NativeControlBrushes {
    fn new(palette: crate::PlatformPalette) -> Self {
        let rgb = |color: Color| {
            u32::from(color.r) | (u32::from(color.g) << 8) | (u32::from(color.b) << 16)
        };
        unsafe {
            Self {
                surface: windows_sys::Win32::Graphics::Gdi::CreateSolidBrush(rgb(
                    palette.resolve(crate::ColorRole::Surface)
                )),
                control: windows_sys::Win32::Graphics::Gdi::CreateSolidBrush(rgb(
                    palette.resolve(crate::ColorRole::Control)
                )),
            }
        }
    }
}
impl Drop for NativeControlBrushes {
    fn drop(&mut self) {
        unsafe {
            if !self.surface.is_null() {
                windows_sys::Win32::Graphics::Gdi::DeleteObject(self.surface);
            }
            if !self.control.is_null() {
                windows_sys::Win32::Graphics::Gdi::DeleteObject(self.control);
            }
        }
    }
}

fn native_control_color(
    hwnd: HWND,
    message: u32,
    dc: windows_sys::Win32::Graphics::Gdi::HDC,
) -> Option<LRESULT> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC,
    };
    if !matches!(
        message,
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX | WM_CTLCOLORSTATIC
    ) {
        return None;
    }
    let services = window_services(hwnd)?;
    let palette = services.palette.get()?;
    let brushes = services.control_brushes.borrow();
    let brushes = brushes.as_ref()?;
    let edit = message != WM_CTLCOLORSTATIC;
    let brush = if edit {
        brushes.control
    } else {
        brushes.surface
    };
    if brush.is_null() {
        return None;
    }
    let rgb =
        |color: Color| u32::from(color.r) | (u32::from(color.g) << 8) | (u32::from(color.b) << 16);
    unsafe {
        windows_sys::Win32::Graphics::Gdi::SetTextColor(
            dc,
            rgb(palette.resolve(crate::ColorRole::PrimaryText)),
        );
        windows_sys::Win32::Graphics::Gdi::SetBkColor(
            dc,
            rgb(palette.resolve(if edit {
                crate::ColorRole::Control
            } else {
                crate::ColorRole::Surface
            })),
        );
    }
    Some(brush as LRESULT)
}

#[cfg(feature = "dark-mode")]
mod dark_mode_compatibility {
    use super::*;
    type Preferred = unsafe extern "system" fn(i32) -> i32;
    type Allow = unsafe extern "system" fn(HWND, bool) -> bool;
    #[derive(Default)]
    struct State {
        references: usize,
        module: isize,
        previous: i32,
        preferred: Option<Preferred>,
        allow: Option<Allow>,
    }
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();

    // These optional uxtheme exports are version-gated compatibility helpers.
    // Documented SetWindowTheme and semantic control colors remain the fallback.
    fn supported_build() -> bool {
        #[repr(C)]
        struct Version {
            size: u32,
            major: u32,
            minor: u32,
            build: u32,
            platform: u32,
            service_pack: [u16; 128],
        }
        unsafe {
            let module = GetModuleHandleW(wide_null("ntdll.dll").as_ptr());
            let Some(address) = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                module,
                b"RtlGetVersion\0".as_ptr(),
            ) else {
                return false;
            };
            let query: unsafe extern "system" fn(*mut Version) -> i32 =
                std::mem::transmute(address);
            let mut version: Version = zeroed();
            version.size = size_of::<Version>() as u32;
            query(&mut version) == 0 && version.major == 10 && version.build >= 18362
        }
    }

    pub(super) struct Lease(bool);
    impl Lease {
        pub(super) fn new() -> Self {
            let mut state = STATE
                .get_or_init(|| Mutex::new(State::default()))
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if state.references == 0 {
                if !supported_build() {
                    return Self(false);
                }
                unsafe {
                    let module = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(
                        wide_null("uxtheme.dll").as_ptr(),
                    );
                    if module.is_null() {
                        return Self(false);
                    }
                    let preferred = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                        module,
                        135usize as *const u8,
                    );
                    let allow = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                        module,
                        133usize as *const u8,
                    );
                    let (Some(preferred), Some(allow)) = (preferred, allow) else {
                        windows_sys::Win32::Foundation::FreeLibrary(module);
                        return Self(false);
                    };
                    let preferred: Preferred = std::mem::transmute(preferred);
                    state.previous = preferred(1); // AllowDark, never ForceDark.
                    state.preferred = Some(preferred);
                    state.allow = Some(std::mem::transmute::<
                        unsafe extern "system" fn() -> isize,
                        Allow,
                    >(allow));
                    state.module = module as isize;
                }
            }
            state.references += 1;
            Self(true)
        }
        pub(super) fn apply(&self, hwnd: HWND, dark: bool) {
            if !self.0 {
                return;
            }
            let state = STATE
                .get()
                .expect("lease initialized")
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let allow = state.allow;
            drop(state);
            if let Some(allow) = allow {
                unsafe {
                    allow(hwnd, dark);
                }
            }
        }
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            if !self.0 {
                return;
            }
            let mut state = STATE
                .get()
                .expect("lease initialized")
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.references -= 1;
            if state.references == 0 {
                unsafe {
                    if let Some(preferred) = state.preferred {
                        preferred(state.previous);
                    }
                    windows_sys::Win32::Foundation::FreeLibrary(state.module as _);
                }
                *state = State::default();
            }
        }
    }

    pub(super) struct AppearanceScope {
        preferred: Option<Preferred>,
        previous: i32,
        _lease: Lease,
    }
    impl AppearanceScope {
        pub(super) fn new(mode: crate::ZsuiThemeMode) -> Self {
            let lease = Lease::new();
            let preferred = if lease.0 {
                STATE
                    .get()
                    .expect("theme lease")
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .preferred
            } else {
                None
            };
            let requested = match mode {
                crate::ZsuiThemeMode::Dark => 2,
                crate::ZsuiThemeMode::Light => 3,
                _ => 0,
            };
            let previous = preferred.map_or(0, |function| unsafe { function(requested) });
            Self {
                preferred,
                previous,
                _lease: lease,
            }
        }
    }
    impl Drop for AppearanceScope {
        fn drop(&mut self) {
            if let Some(preferred) = self.preferred {
                unsafe {
                    preferred(self.previous);
                }
            }
        }
    }
}
#[cfg(feature = "native-host")]
fn apply_native_child_theme(hwnd: HWND, mode: crate::ZsuiThemeMode) {
    unsafe extern "system" fn apply(hwnd: HWND, parameter: LPARAM) -> i32 {
        let mode = match parameter {
            1 => crate::ZsuiThemeMode::Dark,
            2 => crate::ZsuiThemeMode::HighContrast,
            _ => crate::ZsuiThemeMode::Light,
        };
        apply_native_control_theme(hwnd, mode);
        1
    }
    unsafe {
        apply_native_control_theme(hwnd, mode);
        windows_sys::Win32::UI::WindowsAndMessaging::EnumChildWindows(
            hwnd,
            Some(apply),
            match mode {
                crate::ZsuiThemeMode::Dark => 1,
                crate::ZsuiThemeMode::HighContrast => 2,
                _ => 0,
            },
        );
    }
}

#[cfg(feature = "native-host")]
unsafe fn apply_native_control_theme(hwnd: HWND, mode: crate::ZsuiThemeMode) {
    #[cfg(feature = "dark-mode")]
    let _appearance = dark_mode_compatibility::AppearanceScope::new(mode);
    windows_sys::Win32::UI::Shell::SetWindowSubclass(
        hwnd,
        Some(native_theme_subclass),
        0x5a535549,
        match mode {
            crate::ZsuiThemeMode::Dark => 1,
            crate::ZsuiThemeMode::HighContrast => 2,
            _ => 0,
        },
    );
    #[cfg(feature = "dark-mode")]
    {
        let lease = dark_mode_compatibility::Lease::new();
        lease.apply(hwnd, mode == crate::ZsuiThemeMode::Dark);
    }
    let theme = wide_null(if mode == crate::ZsuiThemeMode::Dark {
        "DarkMode_Explorer"
    } else {
        "Explorer"
    });
    // High contrast uses the user's default theme rather than a forced Explorer skin.
    windows_sys::Win32::UI::Controls::SetWindowTheme(
        hwnd,
        if mode == crate::ZsuiThemeMode::HighContrast {
            null()
        } else {
            theme.as_ptr()
        },
        null(),
    );
    let palette = windows_platform_palette(mode);
    let rgb =
        |value: Color| u32::from(value.r) | (u32::from(value.g) << 8) | (u32::from(value.b) << 16);
    let foreground = rgb(palette.resolve(crate::ColorRole::PrimaryText));
    let background = rgb(palette.resolve(crate::ColorRole::Surface));
    let mut class = [0u16; 64];
    let length = windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(
        hwnd,
        class.as_mut_ptr(),
        class.len() as i32,
    );
    match String::from_utf16_lossy(&class[..length.max(0) as usize]).as_str() {
        "SysTreeView32" => {
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::Controls::TVM_SETBKCOLOR,
                0,
                background as isize,
            );
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::Controls::TVM_SETTEXTCOLOR,
                0,
                foreground as isize,
            );
        }
        "SysListView32" => {
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::Controls::LVM_SETBKCOLOR,
                0,
                background as isize,
            );
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::Controls::LVM_SETTEXTBKCOLOR,
                0,
                background as isize,
            );
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::Controls::LVM_SETTEXTCOLOR,
                0,
                foreground as isize,
            );
        }
        _ => {}
    }
    InvalidateRect(hwnd, null(), 0);
}

#[cfg(feature = "native-host")]
unsafe extern "system" fn native_theme_subclass(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    mode: usize,
) -> LRESULT {
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GA_ROOT, WM_CREATE, WM_PARENTNOTIFY,
    };
    #[cfg(feature = "dark-mode")]
    let _appearance = dark_mode_compatibility::AppearanceScope::new(match mode {
        1 => crate::ZsuiThemeMode::Dark,
        2 => crate::ZsuiThemeMode::HighContrast,
        _ => crate::ZsuiThemeMode::Light,
    });
    if message == WM_NCDESTROY {
        RemoveWindowSubclass(hwnd, Some(native_theme_subclass), id);
    }
    if message == WM_PARENTNOTIFY && wparam as u16 == WM_CREATE as u16 {
        let mode = match mode {
            1 => crate::ZsuiThemeMode::Dark,
            2 => crate::ZsuiThemeMode::HighContrast,
            _ => crate::ZsuiThemeMode::Light,
        };
        apply_native_child_theme(lparam as HWND, mode);
    }
    if let Some(result) = native_control_color(GetAncestor(hwnd, GA_ROOT), message, wparam as _) {
        return result;
    }
    DefSubclassProc(hwnd, message, wparam, lparam)
}
