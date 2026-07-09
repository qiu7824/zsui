use std::{env, fs, path::PathBuf, process::ExitCode};

use serde_json::json;
#[cfg(all(feature = "button", feature = "label"))]
use zsui::{button, column, text, WidgetId};
use zsui::{
    native_ui_platform_for_current_target, native_window,
    write_native_host_smoke_artifacts_with_interaction_to, Command,
    NativeHostSmokeInteractionReport, NativeUiPlatform, NativeWindowBuilder,
    NativeWindowSmokeRunOptions, TraySpec,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let non_flag_args = args
        .iter()
        .filter(|arg| !arg.starts_with("--"))
        .collect::<Vec<_>>();
    match run_smoke(
        non_flag_args.first().map(|arg| arg.as_str()),
        non_flag_args.get(1).map(|arg| arg.as_str()),
        args.iter()
            .any(|arg| arg == "--tray" || arg == "--status-item"),
        args.iter().any(|arg| arg == "--view"),
    ) {
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

fn run_smoke(
    platform: Option<&str>,
    artifact_root: Option<&str>,
    include_status_item: bool,
    include_typed_view: bool,
) -> Result<String, String> {
    let platform = parse_platform(platform.unwrap_or("current"))?;
    let current = native_ui_platform_for_current_target()
        .ok_or_else(|| "current target is not a supported ZSUI platform".to_string())?;
    if platform != current {
        return Err(format!(
            "cannot run `{}` native smoke on current `{}` target",
            platform.platform_name(),
            current.platform_name()
        ));
    }

    let artifact_root = artifact_root.unwrap_or("target/native-host-smoke");
    let artifact_dir = PathBuf::from(artifact_root).join(platform.platform_name());
    fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;
    let screenshot_file = artifact_dir
        .join("window.png")
        .to_string_lossy()
        .replace('\\', "/");
    let mut smoke_options = NativeWindowSmokeRunOptions::quick()
        .screenshot_file(screenshot_file)
        .require_screenshot(platform == NativeUiPlatform::Windows);
    if include_status_item {
        smoke_options = smoke_options
            .status_item(
                TraySpec::new()
                    .tooltip("ZSUI Smoke")
                    .item("Open", Command::ShowMainWindow)
                    .separator()
                    .item("Quit", Command::Quit),
            )
            .require_status_item(platform == NativeUiPlatform::Windows);
    }

    let builder = native_window("ZSUI Smoke").size(520, 320);
    let builder = if include_typed_view {
        attach_typed_view(builder)
    } else {
        builder
    };
    let run_report = builder
        .run_smoke(smoke_options)
        .map_err(|err| err.to_string())?;
    let interaction = NativeHostSmokeInteractionReport::from_native_window_smoke(
        platform.platform_name(),
        "real_native_host",
        &run_report,
    );
    let write_report =
        write_native_host_smoke_artifacts_with_interaction_to(platform, artifact_root, interaction)
            .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&json!({
        "run": run_report,
        "artifacts": write_report,
    }))
    .map_err(|err| err.to_string())
}

#[cfg(all(feature = "button", feature = "label"))]
#[derive(Clone)]
enum SmokeMsg {
    Save,
}

#[cfg(all(feature = "button", feature = "label"))]
fn attach_typed_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.view(column(vec![
        text::<SmokeMsg>("ZSUI Native View Smoke"),
        button("Save").id(WidgetId::new(1)).on_click(SmokeMsg::Save),
    ]))
}

#[cfg(not(all(feature = "button", feature = "label")))]
fn attach_typed_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

fn parse_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    if platform == "current" {
        return native_ui_platform_for_current_target()
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
