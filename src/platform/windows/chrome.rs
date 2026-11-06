use windows_sys::Win32::Graphics::Dwm::{
    DwmDefWindowProc, DwmExtendFrameIntoClientArea, DwmGetWindowAttribute, DwmIsCompositionEnabled,
    DWMWA_CAPTION_BUTTON_BOUNDS, DWMWA_SYSTEMBACKDROP_TYPE,
};
use windows_sys::Win32::UI::Controls::MARGINS;

fn update_integrated_caption(hwnd: HWND) {
    let Some(services) = window_services(hwnd) else {
        return;
    };
    let Some(title) = services.integration.borrow().title_bar else {
        return;
    };
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    let height = title
        .height()
        .to_px(crate::Dpi::new(dpi as f32))
        .round_i32();
    let margins = MARGINS {
        cxLeftWidth: 0,
        cxRightWidth: 0,
        cyTopHeight: height,
        cyBottomHeight: 0,
    };
    let mut enabled = 0;
    let composition = unsafe { DwmIsCompositionEnabled(&mut enabled) } >= 0 && enabled != 0;
    let extended = composition && unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) } >= 0;
    let mode = window_draw_plan(hwnd)
        .map(|plan| plan.theme_mode)
        .unwrap_or_default();
    let requested = if title.backdrop == crate::WindowBackdrop::Mica
        && resolved_windows_theme_mode(mode) != crate::ZsuiThemeMode::HighContrast
    {
        2u32
    } else {
        1u32
    };
    let backdrop = extended
        && unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                (&requested as *const u32).cast(),
                4,
            )
        } >= 0
        && requested == 2;
    services.caption_composited.set(extended);
    let metrics = integrated_caption_metrics(hwnd, dpi, height, backdrop);
    if services.caption_metrics.replace(Some(metrics)) != Some(metrics) {
        services.queue_event(crate::NativeWindowEvent::CaptionChanged(metrics));
    }
}

fn integrated_caption_metrics(
    hwnd: HWND,
    dpi: u32,
    height: i32,
    backdrop_active: bool,
) -> crate::CaptionMetrics {
    let mut client: RECT = unsafe { zeroed() };
    unsafe {
        GetClientRect(hwnd, &mut client);
    }
    let mut buttons: RECT = unsafe { zeroed() };
    let available = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_BUTTON_BOUNDS as u32,
            (&mut buttons as *mut RECT).cast(),
            size_of::<RECT>() as u32,
        )
    } >= 0
        && buttons.right > buttons.left;
    let width = if available {
        buttons.right - buttons.left
    } else {
        (138.0 * dpi as f32 / 96.0).round() as i32
    };
    crate::CaptionMetrics {
        buttons: crate::Rect {
            x: (client.right - width).max(0),
            y: 0,
            width: width.min(client.right).max(0),
            height: if available {
                (buttons.bottom - buttons.top).max(1).min(height)
            } else {
                height
            },
        },
        dpi,
        backdrop_active,
    }
}

unsafe fn integrated_caption_message(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HTCLOSE, HTMAXBUTTON, HTMINBUTTON, WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP,
        WM_NCMOUSELEAVE, WM_NCMOUSEMOVE,
    };
    let services = window_services(hwnd)?;
    services.integration.borrow().title_bar?;
    if matches!(
        msg,
        WM_NCHITTEST
            | WM_NCMOUSEMOVE
            | WM_NCMOUSELEAVE
            | WM_NCLBUTTONDOWN
            | WM_NCLBUTTONUP
            | WM_NCLBUTTONDBLCLK
    ) {
        let mut result = 0;
        if DwmDefWindowProc(hwnd, msg, wparam, lparam, &mut result) != 0 {
            return Some(result);
        }
    }
    if msg == WM_NCHITTEST {
        let metrics = services.caption_metrics.get()?;
        let screen = point_from_lparam(lparam);
        let mut point = POINT {
            x: screen.x,
            y: screen.y,
        };
        ScreenToClient(hwnd, &mut point);
        if metrics.buttons.contains(crate::Point {
            x: point.x,
            y: point.y,
        }) {
            let index =
                ((point.x - metrics.buttons.x) * 3 / metrics.buttons.width.max(1)).clamp(0, 2);
            return Some(match index {
                0 => HTMINBUTTON,
                1 => HTMAXBUTTON,
                _ => HTCLOSE,
            } as LRESULT);
        }
    }
    None
}

unsafe fn paint_integrated_caption(hwnd: HWND, paint: &mut WindowsBufferedPaint) {
    let Some(services) = window_services(hwnd) else {
        return;
    };
    let Some(metrics) = services.caption_metrics.get() else {
        return;
    };
    if metrics.backdrop_active {
        let Some(title) = services.integration.borrow().title_bar else {
            return;
        };
        let mut rect: RECT = zeroed();
        GetClientRect(hwnd, &mut rect);
        rect.bottom = title
            .height()
            .to_px(crate::Dpi::new(metrics.dpi as f32))
            .round_i32();
        let palette = windows_palette_for_draw_plan(window_draw_plan(hwnd).as_deref());
        paint.clear_background_alpha(rect, palette.surface);
    }
    if !services.caption_composited.get() {
        use windows_sys::Win32::Graphics::Gdi::{
            DrawFrameControl, DFCS_CAPTIONCLOSE, DFCS_CAPTIONMAX, DFCS_CAPTIONMIN,
            DFCS_CAPTIONRESTORE, DFC_CAPTION,
        };
        let b = metrics.buttons;
        for (index, state) in [
            DFCS_CAPTIONMIN,
            if IsZoomed(hwnd) != 0 {
                DFCS_CAPTIONRESTORE
            } else {
                DFCS_CAPTIONMAX
            },
            DFCS_CAPTIONCLOSE,
        ]
        .into_iter()
        .enumerate()
        {
            let mut rect = RECT {
                left: b.x + b.width * index as i32 / 3,
                top: b.y,
                right: b.x + b.width * (index as i32 + 1) / 3,
                bottom: b.y + b.height,
            };
            DrawFrameControl(paint.hdc(), &mut rect, DFC_CAPTION, state);
        }
    }
}
