#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Serialize;
use zsui::{
    DialogButtonLabels, DialogButtons, DialogLevel, DialogResponse, NativeDesktopDialogService,
    NativeDialogService, NativeDialogSpec, ZsuiError, ZsuiResult,
};

const DIALOG_TITLE: &str = "ZSUI 原生对话框验证 / Native Dialog Proof";

#[derive(Debug)]
struct SmokeArgs {
    output: PathBuf,
    screenshot: PathBuf,
}

#[derive(Serialize)]
struct NativeDialogSmokeProof {
    schema: &'static str,
    platform: &'static str,
    architecture: &'static str,
    native_surface: &'static str,
    title: &'static str,
    owner_window_supplied: bool,
    localized_button_labels: Vec<&'static str>,
    expected_response: &'static str,
    actual_response: &'static str,
    response_matched: bool,
    screenshot_file: String,
    screenshot_captured: bool,
    elapsed_ms: u128,
    errors: Vec<String>,
}

fn parse_args() -> ZsuiResult<SmokeArgs> {
    let args = env::args().collect::<Vec<_>>();
    let value_after = |flag: &str| {
        args.windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| PathBuf::from(&pair[1]))
    };
    let output = value_after("--output").ok_or_else(|| {
        ZsuiError::invalid_spec(
            "native_dialog_smoke.output",
            "usage: native_dialog_smoke --output <proof.json> --screenshot <dialog.png>",
        )
    })?;
    let screenshot = value_after("--screenshot").ok_or_else(|| {
        ZsuiError::invalid_spec(
            "native_dialog_smoke.screenshot",
            "usage: native_dialog_smoke --output <proof.json> --screenshot <dialog.png>",
        )
    })?;
    Ok(SmokeArgs { output, screenshot })
}

fn native_surface() -> &'static str {
    match env::consts::OS {
        "windows" => "win32_message_box",
        "macos" => "appkit_nsalert",
        "linux" => "linux_desktop_zenity",
        _ => "unsupported",
    }
}

fn response_name(response: DialogResponse) -> &'static str {
    match response {
        DialogResponse::Ok => "ok",
        DialogResponse::Cancel => "cancel",
        DialogResponse::Yes => "yes",
        DialogResponse::No => "no",
    }
}

fn screenshot_is_evidence(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() >= 1_024)
}

fn write_report(path: &Path, proof: &NativeDialogSmokeProof) -> ZsuiResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ZsuiError::host(
                "native_dialog_smoke_output",
                format!("cannot create {}: {error}", parent.display()),
            )
        })?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(proof)
            .map_err(|error| ZsuiError::host("native_dialog_smoke_json", error.to_string()))?,
    )
    .map_err(|error| {
        ZsuiError::host(
            "native_dialog_smoke_output",
            format!("cannot write {}: {error}", path.display()),
        )
    })
}

fn main() -> ZsuiResult<()> {
    let args = parse_args()?;
    let labels = DialogButtonLabels::new("确定", "取消", "是", "否");
    let spec = NativeDialogSpec::message(
        DIALOG_TITLE,
        "三平台使用同一 Rust 规格，并保留各自系统的原生操作顺序。\n\
         One Rust specification retains each platform's native action order.",
    )
    .level(DialogLevel::Question)
    .buttons(DialogButtons::YesNoCancel)
    .button_labels(labels);

    let started = Instant::now();
    let response = NativeDesktopDialogService::new().show_native_dialog(&spec);
    let (actual_response, response_matched, mut errors) = match response {
        Ok(actual) => {
            let matched = actual == DialogResponse::Yes;
            let mut errors = Vec::new();
            if !matched {
                errors.push(format!(
                    "expected semantic response `yes`, received `{}`",
                    response_name(actual)
                ));
            }
            (response_name(actual), matched, errors)
        }
        Err(error) => (
            "error",
            false,
            vec![format!("native dialog service failed: {error}")],
        ),
    };
    let screenshot_captured = screenshot_is_evidence(&args.screenshot);
    if !screenshot_captured {
        errors.push(format!(
            "native dialog screenshot is missing or too small: {}",
            args.screenshot.display()
        ));
    }

    let proof = NativeDialogSmokeProof {
        schema: "zsui.native-system-dialog-proof/v1",
        platform: env::consts::OS,
        architecture: env::consts::ARCH,
        native_surface: native_surface(),
        title: DIALOG_TITLE,
        owner_window_supplied: false,
        localized_button_labels: vec!["确定", "取消", "是", "否"],
        expected_response: "yes",
        actual_response,
        response_matched,
        screenshot_file: args.screenshot.to_string_lossy().into_owned(),
        screenshot_captured,
        elapsed_ms: started.elapsed().as_millis(),
        errors,
    };
    write_report(&args.output, &proof)?;
    if proof.errors.is_empty() {
        Ok(())
    } else {
        Err(ZsuiError::host(
            "native_dialog_smoke",
            proof.errors.join("; "),
        ))
    }
}
