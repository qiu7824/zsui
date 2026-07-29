use std::{env, fs, path::PathBuf};

use serde::Serialize;
use zsui::{ClipboardData, ClipboardService, NativeClipboardService, ZsuiError, ZsuiResult};

const WIDTH: usize = 2;
const HEIGHT: usize = 2;
const RGBA: [u8; WIDTH * HEIGHT * 4] = [
    0xE8, 0x3B, 0x35, 0xFF, 0x2D, 0x7D, 0xD2, 0xFF, 0x31, 0xA3, 0x54, 0xFF, 0xF2, 0xC9, 0x4C, 0xFF,
];

#[derive(Serialize)]
struct ClipboardSmokeProof {
    schema: &'static str,
    platform: &'static str,
    architecture: &'static str,
    width: usize,
    height: usize,
    rgba_bytes: usize,
    image_write_succeeded: bool,
    image_read_succeeded: bool,
    original_clipboard_restored: bool,
}

fn output_path() -> ZsuiResult<PathBuf> {
    let args = env::args().collect::<Vec<_>>();
    args.windows(2)
        .find(|pair| pair[0] == "--output")
        .map(|pair| PathBuf::from(&pair[1]))
        .ok_or_else(|| {
            ZsuiError::invalid_spec(
                "native_clipboard_smoke.output",
                "usage: native_clipboard_smoke --output <proof.json>",
            )
        })
}

fn main() -> ZsuiResult<()> {
    let output = output_path()?;
    let mut clipboard = NativeClipboardService::new();
    let original = clipboard.read_clipboard()?;
    let image = ClipboardData::image_rgba(WIDTH, HEIGHT, RGBA)?;

    let proof_result = (|| {
        clipboard.write_clipboard(&image)?;
        let read = clipboard.read_clipboard()?;
        if read.as_ref() != Some(&image) {
            return Err(ZsuiError::host(
                "native_clipboard_smoke",
                format!("native RGBA clipboard round trip returned {read:?}"),
            ));
        }
        Ok(())
    })();

    let restore = clipboard.write_clipboard(original.as_ref().unwrap_or(&ClipboardData::Empty));
    proof_result?;
    restore?;

    let proof = ClipboardSmokeProof {
        schema: "zsui.native-clipboard-proof/v1",
        platform: env::consts::OS,
        architecture: env::consts::ARCH,
        width: WIDTH,
        height: HEIGHT,
        rgba_bytes: RGBA.len(),
        image_write_succeeded: true,
        image_read_succeeded: true,
        original_clipboard_restored: true,
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ZsuiError::host(
                "native_clipboard_smoke_output",
                format!("cannot create {}: {error}", parent.display()),
            )
        })?;
    }
    fs::write(
        &output,
        serde_json::to_vec_pretty(&proof)
            .map_err(|error| ZsuiError::host("native_clipboard_smoke_json", error.to_string()))?,
    )
    .map_err(|error| {
        ZsuiError::host(
            "native_clipboard_smoke_output",
            format!("cannot write {}: {error}", output.display()),
        )
    })
}
