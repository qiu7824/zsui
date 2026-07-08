use std::{env, process::ExitCode};

use zsui::{
    write_native_host_smoke_artifacts, write_native_host_smoke_artifacts_to, NativeUiPlatform,
};

fn main() -> ExitCode {
    match write_report_json(env::args().nth(1).as_deref(), env::args().nth(2).as_deref()) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(2)
        }
    }
}

fn write_report_json(
    platform: Option<&str>,
    artifact_root: Option<&str>,
) -> Result<String, String> {
    let platform = parse_platform(platform.unwrap_or("current"))?;
    let report = match artifact_root {
        Some(root) => write_native_host_smoke_artifacts_to(platform, root),
        None => write_native_host_smoke_artifacts(platform),
    }
    .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

fn parse_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    if platform == "current" {
        return zsui::native_ui_platform_for_current_target()
            .ok_or_else(|| "current target is not a supported ZSUI platform".to_string());
    }

    match platform {
        "windows" => Ok(NativeUiPlatform::Windows),
        "macos" => Ok(NativeUiPlatform::Macos),
        "linux" => Ok(NativeUiPlatform::Linux),
        "android" => Ok(NativeUiPlatform::Android),
        "harmony" => Ok(NativeUiPlatform::Harmony),
        _ => Err(format!("unknown ZSUI platform `{platform}`")),
    }
}
