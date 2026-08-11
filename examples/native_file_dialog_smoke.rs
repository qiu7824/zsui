#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Serialize;
#[cfg(not(target_os = "macos"))]
use zsui::{FileDialogService, NativeFileDialogService};
use zsui::{FileDialogSpec, SaveFileDialogSpec, ZsuiError, ZsuiResult};

const OPEN_TITLE: &str = "ZSUI 打开文件验证 / Open File Proof";
const SAVE_TITLE: &str = "ZSUI 保存文件验证 / Save File Proof";

#[derive(Debug)]
struct SmokeArgs {
    output: PathBuf,
    open_screenshot: PathBuf,
    save_screenshot: PathBuf,
}

#[derive(Serialize)]
struct NativeFileDialogSmokeProof {
    schema: &'static str,
    platform: &'static str,
    architecture: &'static str,
    native_open_surface: &'static str,
    native_save_surface: &'static str,
    owner_window_supplied: bool,
    open_title: &'static str,
    save_title: &'static str,
    open_allows_multiple: bool,
    filters: Vec<&'static str>,
    suggested_name: &'static str,
    open_cancelled: bool,
    save_cancelled: bool,
    open_screenshot_file: String,
    save_screenshot_file: String,
    open_screenshot_captured: bool,
    save_screenshot_captured: bool,
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
    let usage = "usage: native_file_dialog_smoke --output <proof.json> \
                 --open-screenshot <open.png> --save-screenshot <save.png>";
    let output = value_after("--output")
        .ok_or_else(|| ZsuiError::invalid_spec("native_file_dialog_smoke.output", usage))?;
    let open_screenshot = value_after("--open-screenshot").ok_or_else(|| {
        ZsuiError::invalid_spec("native_file_dialog_smoke.open_screenshot", usage)
    })?;
    let save_screenshot = value_after("--save-screenshot").ok_or_else(|| {
        ZsuiError::invalid_spec("native_file_dialog_smoke.save_screenshot", usage)
    })?;
    Ok(SmokeArgs {
        output,
        open_screenshot,
        save_screenshot,
    })
}

fn native_surfaces() -> (&'static str, &'static str) {
    match env::consts::OS {
        "windows" => ("win32_get_open_file_name", "win32_get_save_file_name"),
        "macos" => ("appkit_nsopenpanel", "appkit_nssavepanel"),
        "linux" => (
            "xdg_desktop_portal_open_file",
            "xdg_desktop_portal_save_file",
        ),
        _ => ("unsupported", "unsupported"),
    }
}

fn screenshot_is_evidence(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() >= 1_024)
}

fn write_report(path: &Path, proof: &NativeFileDialogSmokeProof) -> ZsuiResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ZsuiError::host(
                "native_file_dialog_smoke_output",
                format!("cannot create {}: {error}", parent.display()),
            )
        })?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(proof)
            .map_err(|error| ZsuiError::host("native_file_dialog_smoke_json", error.to_string()))?,
    )
    .map_err(|error| {
        ZsuiError::host(
            "native_file_dialog_smoke_output",
            format!("cannot write {}: {error}", path.display()),
        )
    })
}

fn main() -> ZsuiResult<()> {
    let args = parse_args()?;
    let initial_directory = env::current_dir().map_err(|error| {
        ZsuiError::host(
            "native_file_dialog_smoke_current_dir",
            format!("cannot resolve the proof directory: {error}"),
        )
    })?;
    let open_spec = FileDialogSpec::new(OPEN_TITLE)
        .current_path(&initial_directory)
        .filter("Text documents", ["*.txt", "*.md"])
        .filter("All files", ["*.*"])
        .allow_multiple(true);
    let save_spec = SaveFileDialogSpec::new(SAVE_TITLE)
        .current_path(&initial_directory)
        .suggested_name("zsui-native-proof.txt")
        .filter("Text documents", ["*.txt", "*.md"])
        .filter("All files", ["*.*"]);

    let started = Instant::now();
    #[cfg(target_os = "macos")]
    let open_result = zsui::macos_appkit_services::macos_appkit_open_file_dialog_cancel_proof(
        &open_spec,
        &args.open_screenshot,
    );
    #[cfg(not(target_os = "macos"))]
    let open_result = NativeFileDialogService::new().open_file_dialog(&open_spec);

    #[cfg(target_os = "macos")]
    let save_result = zsui::macos_appkit_services::macos_appkit_save_file_dialog_cancel_proof(
        &save_spec,
        &args.save_screenshot,
    );
    #[cfg(not(target_os = "macos"))]
    let save_result = NativeFileDialogService::new().save_file_dialog(&save_spec);

    let mut errors = Vec::new();
    let open_cancelled = match open_result {
        Ok(None) => true,
        Ok(Some(paths)) => {
            errors.push(format!(
                "open panel was expected to cancel but returned {} path(s)",
                paths.len()
            ));
            false
        }
        Err(error) => {
            errors.push(format!("open panel failed: {error}"));
            false
        }
    };
    let save_cancelled = match save_result {
        Ok(None) => true,
        Ok(Some(path)) => {
            errors.push(format!(
                "save panel was expected to cancel but returned {}",
                path.display()
            ));
            false
        }
        Err(error) => {
            errors.push(format!("save panel failed: {error}"));
            false
        }
    };
    let open_screenshot_captured = screenshot_is_evidence(&args.open_screenshot);
    let save_screenshot_captured = screenshot_is_evidence(&args.save_screenshot);
    if !open_screenshot_captured {
        errors.push(format!(
            "native open-panel screenshot is missing or too small: {}",
            args.open_screenshot.display()
        ));
    }
    if !save_screenshot_captured {
        errors.push(format!(
            "native save-panel screenshot is missing or too small: {}",
            args.save_screenshot.display()
        ));
    }
    let (native_open_surface, native_save_surface) = native_surfaces();
    let proof = NativeFileDialogSmokeProof {
        schema: "zsui.native-file-dialog-proof/v1",
        platform: env::consts::OS,
        architecture: env::consts::ARCH,
        native_open_surface,
        native_save_surface,
        owner_window_supplied: false,
        open_title: OPEN_TITLE,
        save_title: SAVE_TITLE,
        open_allows_multiple: true,
        filters: vec!["Text documents:*.txt,*.md", "All files:*.*"],
        suggested_name: "zsui-native-proof.txt",
        open_cancelled,
        save_cancelled,
        open_screenshot_file: args.open_screenshot.to_string_lossy().into_owned(),
        save_screenshot_file: args.save_screenshot.to_string_lossy().into_owned(),
        open_screenshot_captured,
        save_screenshot_captured,
        elapsed_ms: started.elapsed().as_millis(),
        errors,
    };
    write_report(&args.output, &proof)?;
    if proof.errors.is_empty() {
        Ok(())
    } else {
        Err(ZsuiError::host(
            "native_file_dialog_smoke",
            proof.errors.join("; "),
        ))
    }
}
