use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::image_diff;
use crate::report::ScenarioReport;

#[derive(Debug, Clone)]
pub struct WidgetEvidence {
    pub id: String,
    pub role: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub enabled: bool,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct WindowEvidence {
    pub width: u64,
    pub height: u64,
    pub logical_width: u64,
    pub logical_height: u64,
    pub pixel_width: u64,
    pub pixel_height: u64,
}

#[derive(Debug, Clone)]
pub struct ProofDocument {
    pub application: String,
    pub scenario: String,
    pub theme: String,
    pub schema: String,
    pub schema_version: u64,
    pub platform: String,
    pub backend: String,
    pub os_family: String,
    pub architecture: String,
    pub scale_factor: f64,
    pub typography_scale: f64,
    pub window: WindowEvidence,
    pub focused_widget: Option<String>,
    pub widgets: Vec<WidgetEvidence>,
    pub messages: Vec<String>,
    pub unhandled_commands: Vec<String>,
    pub errors: Vec<String>,
}

impl ProofDocument {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "proof JSON root is not an object".to_string())?;

        let string = |key: &str| -> Result<String, String> {
            object
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("missing string field `{key}`"))
        };
        let u64_field = |key: &str| -> Result<u64, String> {
            object
                .get(key)
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("missing unsigned integer field `{key}`"))
        };
        let f64_field = |key: &str| -> Result<f64, String> {
            object
                .get(key)
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("missing number field `{key}`"))
        };
        let string_list = |key: &str| -> Result<Vec<String>, String> {
            object
                .get(key)
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .ok_or_else(|| format!("missing string array field `{key}`"))
        };

        let window = object
            .get("window")
            .and_then(Value::as_object)
            .ok_or_else(|| "missing `window` object".to_string())?;
        let window_u64 = |key: &str| -> Result<u64, String> {
            window
                .get(key)
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("missing window field `{key}`"))
        };

        let widgets = object
            .get("widgets")
            .and_then(Value::as_array)
            .ok_or_else(|| "missing `widgets` array".to_string())?
            .iter()
            .enumerate()
            .map(|(index, widget)| {
                let widget = widget
                    .as_object()
                    .ok_or_else(|| format!("widget {index} is not an object"))?;
                let get = |key: &str| -> Result<Value, String> {
                    widget
                        .get(key)
                        .cloned()
                        .ok_or_else(|| format!("widget {index} is missing `{key}`"))
                };
                let as_i64 = |value: Value, key: &str| -> Result<i64, String> {
                    value
                        .as_i64()
                        .ok_or_else(|| format!("widget {index} field `{key}` is not an integer"))
                };
                let as_bool = |value: Value, key: &str| -> Result<bool, String> {
                    value
                        .as_bool()
                        .ok_or_else(|| format!("widget {index} field `{key}` is not a boolean"))
                };
                let id = get("id")?;
                let role = get("role")?;
                let x = get("x")?;
                let y = get("y")?;
                let width = get("width")?;
                let height = get("height")?;
                let enabled = get("enabled")?;
                let focused = get("focused")?;
                Ok(WidgetEvidence {
                    id: id.as_str().unwrap_or_default().to_string(),
                    role: role.as_str().unwrap_or_default().to_string(),
                    x: as_i64(x, "x")?,
                    y: as_i64(y, "y")?,
                    width: as_i64(width, "width")?,
                    height: as_i64(height, "height")?,
                    enabled: as_bool(enabled, "enabled")?,
                    focused: as_bool(focused, "focused")?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            application: string("application")?,
            scenario: string("scenario")?,
            theme: string("theme")?,
            schema: string("schema")?,
            schema_version: u64_field("schema_version")?,
            platform: string("platform")?,
            backend: string("backend")?,
            os_family: string("os_family")?,
            architecture: string("architecture")?,
            scale_factor: f64_field("scale_factor")?,
            typography_scale: f64_field("typography_scale")?,
            window: WindowEvidence {
                width: window_u64("width")?,
                height: window_u64("height")?,
                logical_width: window_u64("logical_width")?,
                logical_height: window_u64("logical_height")?,
                pixel_width: window_u64("pixel_width")?,
                pixel_height: window_u64("pixel_height")?,
            },
            focused_widget: object
                .get("focused_widget")
                .and_then(Value::as_str)
                .map(str::to_string),
            widgets,
            messages: string_list("messages")?,
            unhandled_commands: string_list("unhandled_commands")?,
            errors: string_list("errors")?,
        })
    }

    pub fn semantic_differences(&self, actual: &Self) -> Vec<String> {
        let mut differences = Vec::new();

        compare_string(&mut differences, "schema", &self.schema, &actual.schema);
        compare_u64(
            &mut differences,
            "schema_version",
            self.schema_version,
            actual.schema_version,
        );
        compare_string(
            &mut differences,
            "application",
            &self.application,
            &actual.application,
        );
        compare_string(
            &mut differences,
            "scenario",
            &self.scenario,
            &actual.scenario,
        );
        compare_string(&mut differences, "theme", &self.theme, &actual.theme);
        compare_string(
            &mut differences,
            "platform",
            &self.platform,
            &actual.platform,
        );
        compare_string(&mut differences, "backend", &self.backend, &actual.backend);
        compare_string(
            &mut differences,
            "os_family",
            &self.os_family,
            &actual.os_family,
        );
        compare_string(
            &mut differences,
            "architecture",
            &self.architecture,
            &actual.architecture,
        );
        compare_f64(
            &mut differences,
            "scale_factor",
            self.scale_factor,
            actual.scale_factor,
        );
        compare_f64(
            &mut differences,
            "typography_scale",
            self.typography_scale,
            actual.typography_scale,
        );
        compare_u64(
            &mut differences,
            "window.width",
            self.window.width,
            actual.window.width,
        );
        compare_u64(
            &mut differences,
            "window.height",
            self.window.height,
            actual.window.height,
        );
        compare_u64(
            &mut differences,
            "window.logical_width",
            self.window.logical_width,
            actual.window.logical_width,
        );
        compare_u64(
            &mut differences,
            "window.logical_height",
            self.window.logical_height,
            actual.window.logical_height,
        );
        compare_u64(
            &mut differences,
            "window.pixel_width",
            self.window.pixel_width,
            actual.window.pixel_width,
        );
        compare_u64(
            &mut differences,
            "window.pixel_height",
            self.window.pixel_height,
            actual.window.pixel_height,
        );
        compare_option_string(
            &mut differences,
            "focused_widget",
            self.focused_widget.as_deref(),
            actual.focused_widget.as_deref(),
        );

        if self.widgets.len() != actual.widgets.len() {
            differences.push(format!(
                "widgets: expected {} widgets, actual {}",
                self.widgets.len(),
                actual.widgets.len()
            ));
        } else {
            for (index, (baseline, actual)) in self.widgets.iter().zip(&actual.widgets).enumerate()
            {
                let prefix = format!("widgets[{index}]");
                compare_string(
                    &mut differences,
                    &format!("{prefix}.id"),
                    &baseline.id,
                    &actual.id,
                );
                compare_string(
                    &mut differences,
                    &format!("{prefix}.role"),
                    &baseline.role,
                    &actual.role,
                );
                compare_i64(
                    &mut differences,
                    &format!("{prefix}.x"),
                    baseline.x,
                    actual.x,
                );
                compare_i64(
                    &mut differences,
                    &format!("{prefix}.y"),
                    baseline.y,
                    actual.y,
                );
                compare_i64(
                    &mut differences,
                    &format!("{prefix}.width"),
                    baseline.width,
                    actual.width,
                );
                compare_i64(
                    &mut differences,
                    &format!("{prefix}.height"),
                    baseline.height,
                    actual.height,
                );
                compare_bool(
                    &mut differences,
                    &format!("{prefix}.enabled"),
                    baseline.enabled,
                    actual.enabled,
                );
                compare_bool(
                    &mut differences,
                    &format!("{prefix}.focused"),
                    baseline.focused,
                    actual.focused,
                );
            }
        }

        compare_string_list(
            &mut differences,
            "messages",
            &self.messages,
            &actual.messages,
        );
        compare_string_list(
            &mut differences,
            "unhandled_commands",
            &self.unhandled_commands,
            &actual.unhandled_commands,
        );
        compare_string_list(&mut differences, "errors", &self.errors, &actual.errors);

        if !actual.unhandled_commands.is_empty() {
            differences.push(format!(
                "actual unhandled_commands must be empty, got {:?}",
                actual.unhandled_commands
            ));
        }
        if !actual.errors.is_empty() {
            differences.push(format!(
                "actual errors must be empty, got {:?}",
                actual.errors
            ));
        }

        differences
    }
}

fn compare_string(differences: &mut Vec<String>, name: &str, baseline: &str, actual: &str) {
    if baseline != actual {
        differences.push(format!(
            "{name}: baseline `{baseline}` != actual `{actual}`"
        ));
    }
}

fn compare_option_string(
    differences: &mut Vec<String>,
    name: &str,
    baseline: Option<&str>,
    actual: Option<&str>,
) {
    if baseline != actual {
        differences.push(format!(
            "{name}: baseline `{}` != actual `{}`",
            baseline.unwrap_or("<none>"),
            actual.unwrap_or("<none>")
        ));
    }
}

fn compare_u64(differences: &mut Vec<String>, name: &str, baseline: u64, actual: u64) {
    if baseline != actual {
        differences.push(format!("{name}: baseline {baseline} != actual {actual}"));
    }
}

fn compare_i64(differences: &mut Vec<String>, name: &str, baseline: i64, actual: i64) {
    if baseline != actual {
        differences.push(format!("{name}: baseline {baseline} != actual {actual}"));
    }
}

fn compare_f64(differences: &mut Vec<String>, name: &str, baseline: f64, actual: f64) {
    if baseline.to_bits() != actual.to_bits() {
        differences.push(format!("{name}: baseline {baseline} != actual {actual}"));
    }
}

fn compare_bool(differences: &mut Vec<String>, name: &str, baseline: bool, actual: bool) {
    if baseline != actual {
        differences.push(format!("{name}: baseline {baseline} != actual {actual}"));
    }
}

fn compare_string_list(
    differences: &mut Vec<String>,
    name: &str,
    baseline: &[String],
    actual: &[String],
) {
    if baseline != actual {
        differences.push(format!(
            "{name}: baseline {baseline:?} != actual {actual:?}"
        ));
    }
}

pub fn read_proof(path: &Path) -> Result<ProofDocument, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    ProofDocument::from_value(&value)
        .map_err(|error| format!("invalid proof {}: {error}", path.display()))
}

fn discover_actual_proofs(actual_dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut files = Vec::new();
    visit_json_files(actual_dir, actual_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut seen = BTreeMap::new();
    for (relative, absolute) in files {
        if relative
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "manifest.json" || name == "summary.md")
        {
            continue;
        }
        seen.insert(relative, absolute);
    }

    Ok(seen.into_iter().collect())
}

fn visit_json_files(
    root: &Path,
    directory: &Path,
    output: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to list {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            visit_json_files(root, &path, output)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("failed to relativize {}: {error}", path.display()))?
                .to_path_buf();
            output.push((relative, path));
        }
    }
    Ok(())
}

pub struct CompareOptions {
    pub baseline_dir: PathBuf,
    pub actual_dir: PathBuf,
    pub diff_dir: PathBuf,
    pub color_tolerance: u8,
    pub critical_threshold_percent: f64,
    pub full_threshold_percent: f64,
}

pub fn run_compare(options: CompareOptions) -> Result<Vec<ScenarioReport>, String> {
    let actual_files = discover_actual_proofs(&options.actual_dir)?;
    if actual_files.is_empty() {
        return Err(format!(
            "no proof JSON files found under {}",
            options.actual_dir.display()
        ));
    }

    fs::create_dir_all(&options.diff_dir)
        .map_err(|error| format!("failed to create {}: {error}", options.diff_dir.display()))?;

    let mut reports = Vec::new();
    for (relative, actual_json) in actual_files {
        let name = relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        let baseline_json = options.baseline_dir.join(&relative);
        if !baseline_json.is_file() {
            reports.push(ScenarioReport::missing_baseline(
                name,
                baseline_json.display().to_string(),
            ));
            continue;
        }

        let baseline = read_proof(&baseline_json);
        let actual = read_proof(&actual_json);
        let (baseline, actual) = match (baseline, actual) {
            (Ok(baseline), Ok(actual)) => (baseline, actual),
            (Err(error), _) => {
                reports.push(ScenarioReport::invalid_json(name, error));
                continue;
            }
            (_, Err(error)) => {
                reports.push(ScenarioReport::invalid_json(name, error));
                continue;
            }
        };

        let semantic_differences = baseline.semantic_differences(&actual);

        let baseline_png = baseline_json.with_extension("png");
        let actual_png = actual_json.with_extension("png");
        let diff_png = options.diff_dir.join(relative.with_extension("png"));
        let image_result = if baseline_png.is_file() && actual_png.is_file() {
            image_diff::compare_image_pair(
                &baseline_png,
                &actual_png,
                &diff_png,
                &baseline,
                &actual,
                options.color_tolerance,
                options.critical_threshold_percent,
                options.full_threshold_percent,
            )
            .map_err(|error| format!("image comparison failed for {name}: {error}"))
        } else {
            Err(format!(
                "missing image pair for {name}: baseline={} actual={}",
                baseline_png.display(),
                actual_png.display()
            ))
        };

        reports.push(ScenarioReport::evaluated(
            name,
            semantic_differences,
            image_result,
        ));
    }

    Ok(reports)
}

pub fn init_baseline(actual_dir: &Path, baseline_dir: &Path) -> Result<usize, String> {
    let actual_files = discover_actual_proofs(actual_dir)?;
    if actual_files.is_empty() {
        return Err(format!(
            "no proof JSON files found under {}",
            actual_dir.display()
        ));
    }

    let mut copied = 0usize;
    for (relative, actual_json) in actual_files {
        for (source, relative) in [
            (actual_json.clone(), relative.clone()),
            (
                actual_json.with_extension("png"),
                relative.with_extension("png"),
            ),
        ] {
            if !source.is_file() {
                continue;
            }
            let destination = baseline_dir.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
            }
            fs::copy(&source, &destination).map_err(|error| {
                format!(
                    "failed to copy {} -> {}: {error}",
                    source.display(),
                    destination.display()
                )
            })?;
            copied += 1;
        }
    }

    Ok(copied)
}
