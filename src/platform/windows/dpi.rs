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
