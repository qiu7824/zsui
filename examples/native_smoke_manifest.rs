use std::{env, process::ExitCode};

use zsui::{
    native_host_smoke_plan_for_current_target, native_host_smoke_plan_json,
    native_host_smoke_plans_json, NativeUiPlatform,
};

fn main() -> ExitCode {
    match smoke_manifest_json(env::args().nth(1).as_deref()) {
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

fn smoke_manifest_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => native_host_smoke_plans_json().map_err(|err| err.to_string()),
        "current" => serde_json::to_string_pretty(&native_host_smoke_plan_for_current_target())
            .map_err(|err| err.to_string()),
        platform => {
            let platform = parse_platform(platform)
                .ok_or_else(|| format!("unknown ZSUI platform `{platform}`"))?;
            native_host_smoke_plan_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn parse_platform(platform: &str) -> Option<NativeUiPlatform> {
    match platform {
        "windows" => Some(NativeUiPlatform::Windows),
        "macos" => Some(NativeUiPlatform::Macos),
        "linux" => Some(NativeUiPlatform::Linux),
        "android" => Some(NativeUiPlatform::Android),
        "harmony" => Some(NativeUiPlatform::Harmony),
        _ => None,
    }
}
