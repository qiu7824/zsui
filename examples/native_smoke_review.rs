use std::{env, process::ExitCode};

use zsui::{
    review_native_host_smoke_artifacts, review_native_host_smoke_artifacts_at, NativeUiPlatform,
};

fn main() -> ExitCode {
    match review_report_json(env::args().nth(1).as_deref(), env::args().nth(2).as_deref()) {
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

fn review_report_json(
    platform: Option<&str>,
    artifact_root: Option<&str>,
) -> Result<String, String> {
    let platform = parse_platform(platform.unwrap_or("current"))?;
    let report = match artifact_root {
        Some(root) => review_native_host_smoke_artifacts_at(platform, root),
        None => review_native_host_smoke_artifacts(platform),
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
        _ => Err(format!("unknown ZSUI platform `{platform}`")),
    }
}
