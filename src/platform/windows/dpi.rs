/// The native host owns its UI thread's DPI context, including applications
/// without an executable manifest and hosts embedded in another process.
/// Restore the caller's context on return; never change process-wide policy.
pub(crate) struct WindowsDpiScope {
    previous: windows_sys::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl WindowsDpiScope {
    pub(crate) fn enter() -> ZsuiResult<Self> {
        use windows_sys::Win32::UI::HiDpi::{
            SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            return Err(ZsuiError::host(
                "windows_dpi_awareness",
                format!("SetThreadDpiAwarenessContext failed: {}", unsafe {
                    GetLastError()
                }),
            ));
        }
        Ok(Self {
            previous,
            _thread: std::marker::PhantomData,
        })
    }
}

impl Drop for WindowsDpiScope {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(self.previous);
        }
    }
}

fn windows_logical_dimension_to_pixels(value: i32, dpi: u32) -> i32 {
    ((i64::from(value.max(1)) * i64::from(dpi.max(96)) + 48) / 96).clamp(1, i64::from(i32::MAX))
        as i32
}

#[cfg(test)]
mod dpi_tests {
    use super::*;
    use windows_sys::Win32::UI::HiDpi::{
        AreDpiAwarenessContextsEqual, GetThreadDpiAwarenessContext, SetThreadDpiAwarenessContext,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_CONTEXT_UNAWARE,
    };

    #[test]
    fn ui_dpi_scope_upgrades_unaware_callers_and_restores_nested_contexts() {
        std::thread::spawn(|| unsafe {
            let original = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_UNAWARE);
            assert!(!original.is_null());
            {
                let _outer = WindowsDpiScope::enter().unwrap();
                assert_ne!(
                    AreDpiAwarenessContextsEqual(
                        GetThreadDpiAwarenessContext(),
                        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
                    ),
                    0
                );
                {
                    let _inner = WindowsDpiScope::enter().unwrap();
                }
                assert_ne!(
                    AreDpiAwarenessContextsEqual(
                        GetThreadDpiAwarenessContext(),
                        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
                    ),
                    0
                );
            }
            assert_ne!(
                AreDpiAwarenessContextsEqual(
                    GetThreadDpiAwarenessContext(),
                    DPI_AWARENESS_CONTEXT_UNAWARE
                ),
                0
            );
            SetThreadDpiAwarenessContext(original);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn client_and_minimum_sizes_scale_once_at_each_supported_dpi() {
        for (dpi, width, height) in [
            (96, 1280, 800),
            (120, 1600, 1000),
            (144, 1920, 1200),
            (192, 2560, 1600),
        ] {
            assert_eq!(
                unsafe {
                    windows_win32_outer_size_for_client_at_dpi(1280, 800, WS_POPUP, 0, false, dpi)
                },
                (width, height)
            );
            assert_eq!(
                windows_logical_dimension_to_pixels(1100, dpi),
                1100 * dpi as i32 / 96
            );
        }
        assert_eq!(windows_logical_dimension_to_pixels(i32::MAX, 192), i32::MAX);
    }

    #[test]
    fn hidden_native_windows_are_per_monitor_v2_without_a_process_manifest() {
        let _lock = windows_win32_view_input_route_test_lock();
        std::thread::spawn(|| unsafe {
            use windows_sys::Win32::UI::HiDpi::GetWindowDpiAwarenessContext;
            let original = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_UNAWARE);
            let windows =
                create_owned_windows_for_specs(&[WindowSpec::new("DPI creation regression")
                    .size(320, 200)
                    .visible(false)])
                .unwrap();
            assert_ne!(
                AreDpiAwarenessContextsEqual(
                    GetWindowDpiAwarenessContext(windows[0].main()),
                    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
                ),
                0
            );
            assert_ne!(
                AreDpiAwarenessContextsEqual(
                    GetThreadDpiAwarenessContext(),
                    DPI_AWARENESS_CONTEXT_UNAWARE
                ),
                0
            );
            drop(windows);
            SetThreadDpiAwarenessContext(original);
        })
        .join()
        .unwrap();
    }
}

fn windows_palette_for_draw_plan(draw_plan: Option<&NativeDrawPlan>) -> WindowsGdiPalette {
    match resolved_windows_theme_mode(
        draw_plan
            .map(|plan| plan.theme_mode)
            .unwrap_or(crate::ZsuiThemeMode::System),
    ) {
        crate::ZsuiThemeMode::HighContrast => windows_high_contrast_palette(),
        crate::ZsuiThemeMode::Dark => WindowsGdiPalette::from_theme(&crate::ZsuiTheme::dark()),
        _ => WindowsGdiPalette::default(),
    }
}

pub(crate) fn windows_platform_palette(mode: crate::ZsuiThemeMode) -> crate::PlatformPalette {
    let mode = resolved_windows_theme_mode(mode);
    let palette = match mode {
        crate::ZsuiThemeMode::HighContrast => windows_high_contrast_palette(),
        crate::ZsuiThemeMode::Dark => WindowsGdiPalette::from_theme(&crate::ZsuiTheme::dark()),
        _ => WindowsGdiPalette::default(),
    };
    crate::PlatformPalette {
        mode,
        colors: [
            palette.primary_text,
            palette.secondary_text,
            palette.disabled_text,
            palette.accent,
            palette.accent_text,
            palette.surface,
            palette.surface_raised,
            palette.control,
            palette.border,
            palette.strong_stroke,
            palette.success,
            palette.warning,
            palette.danger,
        ],
    }
}

fn resolved_windows_theme_mode(theme_mode: crate::ZsuiThemeMode) -> crate::ZsuiThemeMode {
    resolved_windows_theme_mode_for_system(theme_mode, windows_system_theme_mode())
}

fn resolved_windows_theme_mode_for_system(
    theme_mode: crate::ZsuiThemeMode,
    system_mode: crate::ZsuiThemeMode,
) -> crate::ZsuiThemeMode {
    if system_mode == crate::ZsuiThemeMode::HighContrast {
        crate::ZsuiThemeMode::HighContrast
    } else if theme_mode == crate::ZsuiThemeMode::System {
        system_mode
    } else {
        theme_mode
    }
}

pub fn windows_system_theme_mode() -> crate::ZsuiThemeMode {
    if windows_system_high_contrast() {
        return crate::ZsuiThemeMode::HighContrast;
    }
    let subkey = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = wide_null("AppsUseLightTheme");
    let mut value = 1u32;
    let mut value_size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as _,
            &mut value_size,
        )
    };
    if status == 0 && value == 0 {
        crate::ZsuiThemeMode::Dark
    } else {
        crate::ZsuiThemeMode::Light
    }
}

pub fn windows_system_high_contrast() -> bool {
    let mut high_contrast = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        dwFlags: 0,
        lpszDefaultScheme: null_mut(),
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            high_contrast.cbSize,
            &mut high_contrast as *mut HIGHCONTRASTW as _,
            0,
        ) != 0
            && high_contrast.dwFlags & HCF_HIGHCONTRASTON != 0
    }
}

fn windows_high_contrast_palette() -> WindowsGdiPalette {
    let surface = windows_system_color(COLOR_WINDOW);
    let primary_text = windows_system_color(COLOR_WINDOWTEXT);
    WindowsGdiPalette {
        primary_text,
        secondary_text: primary_text,
        disabled_text: primary_text,
        accent: windows_system_color(COLOR_HIGHLIGHT),
        accent_text: windows_system_color(COLOR_HIGHLIGHTTEXT),
        surface,
        surface_raised: surface,
        control: surface,
        border: primary_text,
        strong_stroke: primary_text,
        success: primary_text,
        warning: primary_text,
        danger: primary_text,
    }
}

fn windows_system_color(index: i32) -> Color {
    let color = unsafe { GetSysColor(index) };
    Color::rgb(
        (color & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        ((color >> 16) & 0xff) as u8,
    )
}

fn apply_windows_win32_window_theme(hwnd: HWND, theme_mode: crate::ZsuiThemeMode) {
    if hwnd.is_null() {
        return;
    }
    let dark = i32::from(matches!(
        resolved_windows_theme_mode(theme_mode),
        crate::ZsuiThemeMode::Dark
    ));
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &dark as *const i32 as _,
            size_of::<i32>() as u32,
        );
    }
}
