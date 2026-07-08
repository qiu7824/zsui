use std::{env, process::ExitCode};

use zsui::{
    mobile_runtime_host_scaffold_json, mobile_runtime_host_scaffolds_json, NativeUiPlatform,
};

fn main() -> ExitCode {
    match mobile_scaffold_json(env::args().nth(1).as_deref()) {
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

fn mobile_scaffold_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_host_scaffolds_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_host_scaffold_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn parse_mobile_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    match platform {
        "android" => Ok(NativeUiPlatform::Android),
        "harmony" => Ok(NativeUiPlatform::Harmony),
        _ => Err(format!("unknown ZSUI mobile platform `{platform}`")),
    }
}
