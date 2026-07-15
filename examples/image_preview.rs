#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::sync::Arc;

use zsui::{
    button, column, image_preview, native_window, row, text, AppCx, Dp,
    NativeWindowSmokeRunOptions, ThemeColorToken, ViewNode, WidgetId, ZsIcon, ZsImageFit,
    ZsImageFrameId, ZsImagePreviewConfig, ZsImagePreviewState, ZsuiError, ZsuiResult,
};

const PREVIEW: WidgetId = WidgetId::new(1);

struct State {
    preview: ZsImagePreviewState,
    alternate: bool,
}

impl State {
    fn new() -> ZsuiResult<Self> {
        let mut preview = ZsImagePreviewState::new(ZsImagePreviewConfig::default())?;
        preview.set_png(ZsImageFrameId::new(1), icon_png(ZsIcon::Image)?);
        Ok(Self {
            preview,
            alternate: false,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    SwitchImage,
}

fn view(state: &State) -> ViewNode<Msg> {
    let snapshot = state.preview.snapshot();
    let status = if snapshot.loading {
        "Decoding image"
    } else if snapshot.last_error.is_some() {
        "Image decode failed"
    } else {
        "Image ready"
    };
    let preview = image_preview(&snapshot)
        .id(PREVIEW)
        .image_fit(ZsImageFit::Contain)
        .height(Dp::new(360.0))
        .bg(ThemeColorToken::SurfaceRaised);
    column([
        row([
            text(status).flex(1.0),
            button("Switch image").on_click(Msg::SwitchImage),
        ])
        .height(Dp::new(40.0))
        .gap(Dp::new(8.0)),
        preview,
    ])
    .padding(Dp::new(16.0))
    .gap(Dp::new(12.0))
    .bg(ThemeColorToken::Surface)
}

fn update(state: &mut State, message: Msg, _cx: &mut AppCx) {
    match message {
        Msg::SwitchImage => {
            state.alternate = !state.alternate;
            let icon = if state.alternate {
                ZsIcon::Search
            } else {
                ZsIcon::Image
            };
            if let Ok(bytes) = icon_png(icon) {
                state.preview.set_png(
                    ZsImageFrameId::new(if state.alternate { 2 } else { 1 }),
                    bytes,
                );
            }
        }
    }
}

fn icon_png(icon: ZsIcon) -> ZsuiResult<Arc<[u8]>> {
    icon.png_24_bytes().map(Arc::from).ok_or_else(|| {
        ZsuiError::invalid_spec("image_preview.icon", "icon has no PNG fallback asset")
    })
}

fn main() -> ZsuiResult<()> {
    let builder = native_window("ZSUI Retained Image Preview")
        .size(720, 500)
        .min_size(480, 360)
        .stateful_view(State::new()?, view, update);
    if std::env::args().any(|argument| argument == "--smoke") {
        let report = builder.run_smoke(NativeWindowSmokeRunOptions::new(700))?;
        if !report.visible_window_was_created() {
            return Err(ZsuiError::host(
                "image_preview_smoke",
                "the preview window was not visible",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}
