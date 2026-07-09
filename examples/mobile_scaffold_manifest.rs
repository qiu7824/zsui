use std::{env, process::ExitCode};

use zsui::{
    mobile_runtime_bridge_contract_json, mobile_runtime_bridge_contracts_json,
    mobile_runtime_device_smoke_plan_json, mobile_runtime_device_smoke_plans_json,
    mobile_runtime_host_scaffold_json, mobile_runtime_host_scaffolds_json,
    review_mobile_runtime_device_smoke_artifacts, review_mobile_runtime_device_smoke_artifacts_at,
    NativeUiPlatform,
};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    match mobile_manifest_json(&args) {
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

fn mobile_manifest_json(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("--bridge") => mobile_bridge_json(args.get(1).map(String::as_str)),
        Some("--smoke") => mobile_device_smoke_json(args.get(1).map(String::as_str)),
        Some("--review") => mobile_device_smoke_review_json(
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
        ),
        platform => mobile_scaffold_json(platform),
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

fn mobile_bridge_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_bridge_contracts_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_bridge_contract_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn mobile_device_smoke_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_device_smoke_plans_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_device_smoke_plan_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn mobile_device_smoke_review_json(
    platform: Option<&str>,
    artifact_root: Option<&str>,
) -> Result<String, String> {
    let platform = parse_mobile_platform(platform.unwrap_or("android"))?;
    let report = match artifact_root {
        Some(root) => review_mobile_runtime_device_smoke_artifacts_at(platform, root),
        None => review_mobile_runtime_device_smoke_artifacts(platform),
    }
    .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

fn parse_mobile_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    match platform {
        "android" => Ok(NativeUiPlatform::Android),
        "harmony" => Ok(NativeUiPlatform::Harmony),
        _ => Err(format!("unknown ZSUI mobile platform `{platform}`")),
    }
}
