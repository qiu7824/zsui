#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use qrcode::QrCode;
use zsui::{
    column, image, native_window, text, textbox, AppCx, Dp, NativeImageInterpolation,
    NativeWindowSmokeRunOptions, ThemeColorToken, ViewAlign, ViewJustify, ViewNode, WidgetId,
    ZsImageFit, ZsImageFrame, ZsImageFrameId, ZsuiError, ZsuiResult,
};

const INPUT: WidgetId = WidgetId::new(1);
const INSTRUCTION: WidgetId = WidgetId::new(2);
const PREVIEW: WidgetId = WidgetId::new(3);
const PREVIEW_SIZE: f32 = 200.0;

#[derive(Debug, Clone)]
enum Message {
    TextChanged(String),
}

struct State {
    text: String,
    qr_code: Option<ZsImageFrame>,
    error: Option<String>,
    revision: u64,
}

impl State {
    fn new() -> Self {
        Self {
            text: String::new(),
            qr_code: None,
            error: None,
            revision: 0,
        }
    }

    fn set_text(&mut self, text: String) {
        self.revision = self.revision.saturating_add(1);
        match qr_frame(&text, ZsImageFrameId::new(self.revision)) {
            Ok(qr_code) => {
                self.text = text;
                self.qr_code = qr_code;
                self.error = None;
            }
            Err(error) => {
                self.text = text;
                self.qr_code = None;
                self.error = Some(error);
            }
        }
    }
}

fn qr_frame(text: &str, frame_id: ZsImageFrameId) -> Result<Option<ZsImageFrame>, String> {
    if text.is_empty() {
        return Ok(None);
    }

    let pixels = QrCode::new(text.as_bytes())
        .map_err(|error| error.to_string())?
        .render::<image::Rgba<u8>>()
        .min_dimensions(PREVIEW_SIZE as u32, PREVIEW_SIZE as u32)
        .build();
    let width = pixels.width();
    let height = pixels.height();
    ZsImageFrame::from_rgba8(frame_id, width, height, pixels.into_raw())
        .map(Some)
        .map_err(|error| error.to_string())
}

fn view(state: &State) -> ViewNode<Message> {
    let preview = if let Some(frame) = &state.qr_code {
        image(frame.clone())
            .image_fit(ZsImageFit::Contain)
            .image_interpolation(NativeImageInterpolation::Nearest)
            .accessibility_label("Generated QR code")
            .width(Dp::new(PREVIEW_SIZE))
            .height(Dp::new(PREVIEW_SIZE))
    } else if let Some(error) = &state.error {
        text(format!("Unable to generate QR code: {error}"))
    } else {
        text("Enter some text to generate a QR code")
    }
    .id(PREVIEW);

    column([
        text("Enter text to generate QR code:").id(INSTRUCTION),
        textbox(&state.text)
            .id(INPUT)
            .placeholder("https://example.com")
            .accessibility_label("Text to encode")
            .on_change(Message::TextChanged)
            .width(Dp::new(300.0)),
        column([preview])
            .width(Dp::new(PREVIEW_SIZE))
            .height(Dp::new(PREVIEW_SIZE))
            .justify(ViewJustify::Center)
            .align(ViewAlign::Center)
            .bg(ThemeColorToken::SurfaceRaised),
    ])
    .gap(Dp::new(16.0))
    .padding(Dp::new(20.0))
    .justify(ViewJustify::Center)
    .align(ViewAlign::Center)
    .bg(ThemeColorToken::Surface)
}

fn update(state: &mut State, message: Message, _cx: &mut AppCx) {
    match message {
        Message::TextChanged(text) => state.set_text(text),
    }
}

fn argument_value(name: &str) -> Option<String> {
    let arguments = std::env::args().collect::<Vec<_>>();
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn main() -> ZsuiResult<()> {
    let builder = native_window("QR Code Generator")
        .size(400, 400)
        .min_size(320, 360)
        .stateful_view(State::new(), view, update);

    if std::env::args().any(|argument| argument == "--smoke") {
        let screenshot = argument_value("--screenshot")
            .unwrap_or_else(|| "target/qr-code-generator.png".to_owned());
        if let Some(parent) = std::path::Path::new(&screenshot).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ZsuiError::host("create_qr_smoke_dir", error.to_string()))?;
        }
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1_600)
                .native_view_click_widget(INPUT)
                .native_view_ime_preedit("中", Some((1, 1)))
                .native_view_ime_commit("中文")
                .native_view_text_input(" https://example.com")
                .screenshot_file(screenshot)
                .require_screenshot(true),
        )?;

        if let Some(report_path) = argument_value("--report") {
            if let Some(parent) = std::path::Path::new(&report_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| ZsuiError::host("create_qr_report_dir", error.to_string()))?;
            }
            let bytes = serde_json::to_vec_pretty(&report)
                .map_err(|error| ZsuiError::host("serialize_qr_smoke_report", error.to_string()))?;
            std::fs::write(report_path, bytes)
                .map_err(|error| ZsuiError::host("write_qr_smoke_report", error.to_string()))?;
        }

        let committed_cjk = report
            .native_view_ime_commit_script_evidence
            .iter()
            .any(|evidence| evidence.contains_cjk);
        if !report.visible_window_was_created()
            || report.native_view_message_count < 2
            || report.native_view_ime_preedit_update_count != 1
            || report.native_view_ime_commit_count != 1
            || report.native_view_ime_caret_rect_observation_count < 2
            || report.native_view_ime_preedit_active
            || !committed_cjk
            || report.native_accessibility_backend.is_none()
        {
            return Err(ZsuiError::host(
                "qr_code_generator_smoke",
                "native text update, CJK IME, accessibility tree or final screenshot was not verified",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_has_no_preview() {
        assert!(qr_frame("", ZsImageFrameId::new(1)).unwrap().is_none());
    }

    #[test]
    fn unicode_input_produces_an_rgba_frame() {
        let frame = qr_frame("中文 input", ZsImageFrameId::new(7))
            .unwrap()
            .unwrap();
        assert!(frame.width() >= PREVIEW_SIZE as u32);
        assert_eq!(frame.width(), frame.height());
        assert_eq!(
            frame.decoded_bytes(),
            frame.width() as usize * frame.height() as usize * 4
        );
        assert_eq!(frame.id(), ZsImageFrameId::new(7));
    }
}
