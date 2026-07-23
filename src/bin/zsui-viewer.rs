#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use zsui::{
    native_window,
    ui_viewer::{
        ui_viewer_update, UiViewerSource, UiViewerSourceSnapshot, UiViewerState,
        ZSUI_UI_VIEWER_PROOF_SCHEMA, ZSUI_UI_VIEWER_PROOF_SCHEMA_VERSION,
    },
    NativeWindowSmokeRunOptions, NativeWindowSmokeRunReport, Point, ZsuiError,
};

#[derive(Debug)]
struct Arguments {
    document: PathBuf,
    bindings: Option<PathBuf>,
    values: Option<PathBuf>,
    width: u32,
    height: u32,
    poll_ms: u64,
    smoke_output: Option<PathBuf>,
    smoke_clicks: Vec<Point>,
    smoke_scroll: Option<(Point, i32)>,
    benchmark_empty: bool,
    benchmark_seconds: Option<u64>,
}

#[derive(Serialize)]
struct ViewerProof {
    schema: &'static str,
    schema_version: u32,
    platform: &'static str,
    capture_backend: &'static str,
    display_server: Option<&'static str>,
    window: ViewerProofWindow,
    source: UiViewerSourceSnapshot,
    runtime: NativeWindowSmokeRunReport,
}

#[derive(Serialize)]
struct ViewerProofWindow {
    logical_width: u32,
    logical_height: u32,
    pixel_width: u32,
    pixel_height: u32,
    scale_factor: f64,
    typography_scale: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments(env::args().skip(1))?;
    let bindings = arguments
        .bindings
        .or_else(|| inferred_binding_path(&arguments.document));
    let source = UiViewerSource::open(&arguments.document, bindings.as_ref())?
        .poll_interval_ms(arguments.poll_ms);
    let state = load_values(arguments.values.as_deref())?;
    source.validate_properties(&state.properties)?;
    let live_source = source.clone();
    let builder = if arguments.benchmark_empty {
        native_window("ZSUI UI Viewer")
            .size(arguments.width, arguments.height)
            .min_size(320, 240)
            .release_view_when_hidden()
    } else {
        native_window("ZSUI UI Viewer")
            .size(arguments.width, arguments.height)
            .min_size(320, 240)
            .release_view_when_hidden()
            .stateful_view(
                state,
                move |state| live_source.view(state),
                ui_viewer_update,
            )
    };

    if let Some(seconds) = arguments.benchmark_seconds {
        builder.run_smoke(NativeWindowSmokeRunOptions::new(
            seconds.saturating_mul(1_000).max(250),
        ))?;
    } else if let Some(output_directory) = arguments.smoke_output {
        run_smoke(
            builder,
            &source,
            &output_directory,
            &arguments.smoke_clicks,
            arguments.smoke_scroll,
        )?;
    } else {
        builder.run()?;
    }
    Ok(())
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut document = None;
    let mut bindings = None;
    let mut values = None;
    let mut width = 720_u32;
    let mut height = 520_u32;
    let mut poll_ms = 250_u64;
    let mut smoke_output = None;
    let mut smoke_clicks = Vec::new();
    let mut smoke_scroll = None;
    let mut benchmark_empty = false;
    let mut benchmark_seconds = None;
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--bindings" => bindings = Some(path_argument(&mut arguments, "--bindings")?),
            "--values" => values = Some(path_argument(&mut arguments, "--values")?),
            "--width" => width = number_argument(&mut arguments, "--width")?,
            "--height" => height = number_argument(&mut arguments, "--height")?,
            "--poll-ms" => poll_ms = number_argument(&mut arguments, "--poll-ms")?,
            "--smoke" => smoke_output = Some(path_argument(&mut arguments, "--smoke")?),
            "--benchmark-empty" => benchmark_empty = true,
            "--benchmark-seconds" => {
                benchmark_seconds = Some(number_argument(&mut arguments, "--benchmark-seconds")?)
            }
            "--smoke-click" => {
                let x = number_argument(&mut arguments, "--smoke-click x")?;
                let y = number_argument(&mut arguments, "--smoke-click y")?;
                smoke_clicks.push(Point { x, y });
            }
            "--smoke-scroll" => {
                let x = number_argument(&mut arguments, "--smoke-scroll x")?;
                let y = number_argument(&mut arguments, "--smoke-scroll y")?;
                let delta_y = number_argument(&mut arguments, "--smoke-scroll delta-y")?;
                smoke_scroll = Some((Point { x, y }, delta_y));
            }
            "--help" | "-h" => return Err(usage().to_owned()),
            value if value.starts_with('-') => {
                return Err(format!("unknown option `{value}`\n{}", usage()));
            }
            value if document.is_none() => document = Some(PathBuf::from(value)),
            value => return Err(format!("unexpected argument `{value}`\n{}", usage())),
        }
    }

    let document = document.ok_or_else(|| usage().to_owned())?;
    if width == 0 || height == 0 {
        return Err("--width and --height must be greater than zero".to_owned());
    }

    Ok(Arguments {
        document,
        bindings,
        values,
        width,
        height,
        poll_ms: poll_ms.max(16),
        smoke_output,
        smoke_clicks,
        smoke_scroll,
        benchmark_empty,
        benchmark_seconds,
    })
}

fn path_argument(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} requires a path"))
}

fn number_argument<T>(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<T, String>
where
    T: std::str::FromStr,
{
    let value = arguments
        .next()
        .ok_or_else(|| format!("{option} requires a number"))?;
    value
        .parse()
        .map_err(|_| format!("invalid number `{value}` for {option}"))
}

fn usage() -> &'static str {
    "usage: zsui-viewer <document.json> [--bindings path] [--values path] \
     [--width pixels] [--height pixels] [--poll-ms milliseconds] [--smoke output-directory] \
     [--smoke-click x y]... [--smoke-scroll x y delta-y] \
     [--benchmark-empty] [--benchmark-seconds seconds]"
}

fn inferred_binding_path(document: &Path) -> Option<PathBuf> {
    let file_stem = document.file_stem()?.to_str()?;
    let candidate = document.with_file_name(format!("{file_stem}.bindings.json"));
    candidate.is_file().then_some(candidate)
}

fn load_values(path: Option<&Path>) -> Result<UiViewerState, Box<dyn Error>> {
    let Some(path) = path else {
        return Ok(UiViewerState::default());
    };
    let source = fs::read_to_string(path)?;
    let properties = serde_json::from_str::<BTreeMap<String, Value>>(&source)?;
    Ok(UiViewerState::with_properties(properties))
}

fn run_smoke(
    builder: zsui::NativeWindowBuilder,
    source: &UiViewerSource,
    output_directory: &Path,
    smoke_clicks: &[Point],
    smoke_scroll: Option<(Point, i32)>,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output_directory)?;
    let screenshot = output_directory.join("window.png");
    let mut options = NativeWindowSmokeRunOptions::new(900)
        .screenshot_file(screenshot.to_string_lossy())
        .require_screenshot(true);
    if !smoke_clicks.is_empty() {
        options = options.native_view_clicks(smoke_clicks.iter().copied());
    }
    if let Some((point, delta_y)) = smoke_scroll {
        options = options.native_view_scroll(point, delta_y);
    }
    let runtime = builder.run_smoke(options)?;
    if !runtime.visible_window_was_created() || !runtime.screenshot_captured {
        return Err(Box::new(ZsuiError::host(
            "ui_viewer_smoke",
            "the native Viewer did not create and capture a visible window",
        )));
    }
    if smoke_scroll.is_some() && runtime.native_view_scroll_count == 0 {
        return Err(Box::new(ZsuiError::host(
            "ui_viewer_smoke",
            "the native Viewer did not route the requested scroll input",
        )));
    }
    if !smoke_clicks.is_empty()
        && (runtime.native_view_click_count < smoke_clicks.len()
            || runtime.native_view_message_count < smoke_clicks.len())
    {
        return Err(Box::new(ZsuiError::host(
            "ui_viewer_smoke",
            "the native Viewer did not route every requested click through a typed message",
        )));
    }
    let capture = runtime.native_view_capture.as_ref().ok_or_else(|| {
        ZsuiError::host(
            "ui_viewer_smoke",
            "the native Viewer did not report its final platform-surface capture",
        )
    })?;
    let proof = ViewerProof {
        schema: ZSUI_UI_VIEWER_PROOF_SCHEMA,
        schema_version: ZSUI_UI_VIEWER_PROOF_SCHEMA_VERSION,
        platform: capture.platform,
        capture_backend: capture.backend,
        display_server: capture.display_server,
        window: ViewerProofWindow {
            logical_width: capture.logical_width,
            logical_height: capture.logical_height,
            pixel_width: capture.pixel_width,
            pixel_height: capture.pixel_height,
            scale_factor: capture.scale_factor,
            typography_scale: capture.typography_scale,
        },
        source: source.snapshot(),
        runtime,
    };
    fs::write(
        output_directory.join("proof.json"),
        serde_json::to_vec_pretty(&proof)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_document_authoring_options() {
        let arguments = parse_arguments([
            "ui.json".to_owned(),
            "--bindings".to_owned(),
            "ui.bindings.json".to_owned(),
            "--poll-ms".to_owned(),
            "40".to_owned(),
        ])
        .unwrap();

        assert_eq!(arguments.document, PathBuf::from("ui.json"));
        assert_eq!(arguments.bindings, Some(PathBuf::from("ui.bindings.json")));
        assert_eq!(arguments.poll_ms, 40);
    }

    #[test]
    fn parser_accepts_native_scroll_smoke_input() {
        let arguments = parse_arguments([
            "ui.json".to_owned(),
            "--smoke".to_owned(),
            "proof".to_owned(),
            "--smoke-scroll".to_owned(),
            "120".to_owned(),
            "240".to_owned(),
            "96".to_owned(),
        ])
        .unwrap();

        assert_eq!(arguments.smoke_scroll, Some((Point { x: 120, y: 240 }, 96)));
    }

    #[test]
    fn parser_accepts_repeated_native_click_smoke_input() {
        let arguments = parse_arguments([
            "ui.json".to_owned(),
            "--smoke".to_owned(),
            "proof".to_owned(),
            "--smoke-click".to_owned(),
            "120".to_owned(),
            "240".to_owned(),
            "--smoke-click".to_owned(),
            "300".to_owned(),
            "160".to_owned(),
        ])
        .unwrap();

        assert_eq!(
            arguments.smoke_clicks,
            vec![Point { x: 120, y: 240 }, Point { x: 300, y: 160 }]
        );
    }
}
