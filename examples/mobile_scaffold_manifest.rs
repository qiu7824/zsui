use std::{env, process::ExitCode};

use zsui::{
    mobile_runtime_bridge_contract_json, mobile_runtime_bridge_contract_smoke_report_json,
    mobile_runtime_bridge_contract_smoke_reports_json, mobile_runtime_bridge_contracts_json,
    mobile_runtime_bridge_dispatch_report_json, mobile_runtime_bridge_dispatch_reports_json,
    mobile_runtime_bridge_parity_report_json, mobile_runtime_bridge_parity_reports_json,
    mobile_runtime_device_smoke_plan_json, mobile_runtime_device_smoke_plans_json,
    mobile_runtime_device_smoke_trace_template_json,
    mobile_runtime_device_smoke_trace_templates_json, mobile_runtime_host_scaffold_json,
    mobile_runtime_host_scaffolds_json, review_mobile_runtime_bridge_contract_artifacts,
    review_mobile_runtime_bridge_contract_artifacts_at,
    review_mobile_runtime_bridge_contract_artifacts_for_all,
    review_mobile_runtime_bridge_contract_artifacts_for_all_at,
    review_mobile_runtime_device_smoke_artifacts, review_mobile_runtime_device_smoke_artifacts_at,
    write_mobile_runtime_bridge_contract_artifacts,
    write_mobile_runtime_bridge_contract_artifacts_for_all,
    write_mobile_runtime_bridge_contract_artifacts_for_all_to,
    write_mobile_runtime_bridge_contract_artifacts_to, MobileRuntimeDeviceSmokeTraceKind,
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
        Some("--parity") => mobile_bridge_parity_json(args.get(1).map(String::as_str)),
        Some("--dispatch") => mobile_bridge_dispatch_json(args.get(1).map(String::as_str)),
        Some("--dispatch-smoke") => {
            mobile_bridge_dispatch_smoke_json(args.get(1).map(String::as_str))
        }
        Some("--write-contract") => mobile_bridge_contract_write_json(
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
        ),
        Some("--review-contract") => mobile_bridge_contract_review_json(
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
        ),
        Some("--smoke") => mobile_device_smoke_json(args.get(1).map(String::as_str)),
        Some("--trace-template") => mobile_device_smoke_trace_template_json(
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
        ),
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

fn mobile_bridge_parity_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_bridge_parity_reports_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_bridge_parity_report_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn mobile_bridge_dispatch_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_bridge_dispatch_reports_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_bridge_dispatch_report_json(platform).map_err(|err| err.to_string())
        }
    }
}

fn mobile_bridge_dispatch_smoke_json(platform: Option<&str>) -> Result<String, String> {
    match platform.unwrap_or("all") {
        "all" => mobile_runtime_bridge_contract_smoke_reports_json().map_err(|err| err.to_string()),
        platform => {
            let platform = parse_mobile_platform(platform)?;
            mobile_runtime_bridge_contract_smoke_report_json(platform)
                .map_err(|err| err.to_string())
        }
    }
}

fn mobile_bridge_contract_write_json(
    platform: Option<&str>,
    artifact_root: Option<&str>,
) -> Result<String, String> {
    let report = match platform.unwrap_or("android") {
        "all" => match artifact_root {
            Some(root) => serde_json::to_value(
                write_mobile_runtime_bridge_contract_artifacts_for_all_to(root)
                    .map_err(|err| err.to_string())?,
            ),
            None => serde_json::to_value(
                write_mobile_runtime_bridge_contract_artifacts_for_all()
                    .map_err(|err| err.to_string())?,
            ),
        },
        platform => {
            let platform = parse_mobile_platform(platform)?;
            match artifact_root {
                Some(root) => serde_json::to_value(
                    write_mobile_runtime_bridge_contract_artifacts_to(platform, root)
                        .map_err(|err| err.to_string())?,
                ),
                None => serde_json::to_value(
                    write_mobile_runtime_bridge_contract_artifacts(platform)
                        .map_err(|err| err.to_string())?,
                ),
            }
        }
    }
    .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

fn mobile_bridge_contract_review_json(
    platform: Option<&str>,
    artifact_root: Option<&str>,
) -> Result<String, String> {
    let report = match platform.unwrap_or("android") {
        "all" => match artifact_root {
            Some(root) => serde_json::to_value(
                review_mobile_runtime_bridge_contract_artifacts_for_all_at(root)
                    .map_err(|err| err.to_string())?,
            ),
            None => serde_json::to_value(
                review_mobile_runtime_bridge_contract_artifacts_for_all()
                    .map_err(|err| err.to_string())?,
            ),
        },
        platform => {
            let platform = parse_mobile_platform(platform)?;
            match artifact_root {
                Some(root) => serde_json::to_value(
                    review_mobile_runtime_bridge_contract_artifacts_at(platform, root)
                        .map_err(|err| err.to_string())?,
                ),
                None => serde_json::to_value(
                    review_mobile_runtime_bridge_contract_artifacts(platform)
                        .map_err(|err| err.to_string())?,
                ),
            }
        }
    }
    .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
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

fn mobile_device_smoke_trace_template_json(
    platform: Option<&str>,
    trace_kind: Option<&str>,
) -> Result<String, String> {
    let platform = parse_mobile_platform(platform.unwrap_or("android"))?;
    match trace_kind.unwrap_or("all") {
        "all" => mobile_runtime_device_smoke_trace_templates_json(platform)
            .map_err(|err| err.to_string()),
        trace_kind => {
            let trace_kind = parse_mobile_trace_kind(trace_kind)?;
            mobile_runtime_device_smoke_trace_template_json(platform, trace_kind)
                .map_err(|err| err.to_string())
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

fn parse_mobile_trace_kind(trace_kind: &str) -> Result<MobileRuntimeDeviceSmokeTraceKind, String> {
    match trace_kind {
        "lifecycle" => Ok(MobileRuntimeDeviceSmokeTraceKind::Lifecycle),
        "surface" => Ok(MobileRuntimeDeviceSmokeTraceKind::Surface),
        "input" => Ok(MobileRuntimeDeviceSmokeTraceKind::Input),
        "clipboard" => Ok(MobileRuntimeDeviceSmokeTraceKind::Clipboard),
        _ => Err(format!("unknown ZSUI mobile trace kind `{trace_kind}`")),
    }
}

fn parse_mobile_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    match platform {
        "android" => Ok(NativeUiPlatform::Android),
        _ => Err(format!("unknown ZSUI mobile platform `{platform}`")),
    }
}
