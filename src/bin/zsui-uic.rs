use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde_json::Value;
use zsui::ui_document::{
    validate_ui_binding_values, UiAiHandoffPackage, UiBindingSchema, UiDocument, UiFeatureSet,
    UiValidationReport,
};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), String> {
    match parse_command(arguments)? {
        Command::Check {
            document_path,
            bindings_path,
            json,
        } => check(document_path, bindings_path, json),
        Command::Handoff {
            document_path,
            bindings_path,
            values_path,
            output_path,
            preview_path,
            force,
        } => handoff(
            document_path,
            bindings_path,
            values_path,
            output_path,
            preview_path,
            force,
        ),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Check {
        document_path: PathBuf,
        bindings_path: Option<PathBuf>,
        json: bool,
    },
    Handoff {
        document_path: PathBuf,
        bindings_path: Option<PathBuf>,
        values_path: Option<PathBuf>,
        output_path: PathBuf,
        preview_path: Option<PathBuf>,
        force: bool,
    },
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut arguments = arguments.into_iter();
    let command = arguments.next().ok_or_else(usage)?;
    let document_path = arguments.next().map(PathBuf::from).ok_or_else(usage)?;
    match command.as_str() {
        "check" => {
            let mut bindings_path = None;
            let mut json = false;
            while let Some(argument) = arguments.next() {
                match argument.as_str() {
                    "--bindings" => {
                        bindings_path = Some(required_path(&mut arguments, "--bindings")?);
                    }
                    "--json" => json = true,
                    _ => return Err(format!("unknown argument {argument:?}\n{}", usage())),
                }
            }
            Ok(Command::Check {
                document_path,
                bindings_path,
                json,
            })
        }
        "handoff" => {
            let mut bindings_path = None;
            let mut values_path = None;
            let mut output_path = None;
            let mut preview_path = None;
            let mut force = false;
            while let Some(argument) = arguments.next() {
                match argument.as_str() {
                    "--bindings" => {
                        bindings_path = Some(required_path(&mut arguments, "--bindings")?);
                    }
                    "--values" => {
                        values_path = Some(required_path(&mut arguments, "--values")?);
                    }
                    "--output" => {
                        output_path = Some(required_path(&mut arguments, "--output")?);
                    }
                    "--preview" | "--screenshot" => {
                        preview_path = Some(required_path(&mut arguments, &argument)?);
                    }
                    "--force" => force = true,
                    _ => return Err(format!("unknown argument {argument:?}\n{}", usage())),
                }
            }
            Ok(Command::Handoff {
                document_path,
                bindings_path,
                values_path,
                output_path: output_path
                    .ok_or_else(|| format!("--output is required\n{}", usage()))?,
                preview_path,
                force,
            })
        }
        _ => Err(usage()),
    }
}

fn required_path(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} requires a path"))
}

fn check(document_path: PathBuf, bindings_path: Option<PathBuf>, json: bool) -> Result<(), String> {
    let document = read_document(&document_path)?;
    let bindings = read_bindings(bindings_path.as_deref())?;
    let features = UiFeatureSet::compiled();
    let report = document.validate(&features, &bindings);
    print_report(&document_path, &report, json)?;
    if report.is_valid() {
        Ok(())
    } else {
        Err(format!(
            "UI document check failed with {} diagnostic(s)",
            report.diagnostics.len()
        ))
    }
}

fn handoff(
    document_path: PathBuf,
    bindings_path: Option<PathBuf>,
    values_path: Option<PathBuf>,
    output_path: PathBuf,
    preview_path: Option<PathBuf>,
    force: bool,
) -> Result<(), String> {
    let document = read_document(&document_path)?;
    let bindings = read_bindings(bindings_path.as_deref())?;
    let values = values_path.as_deref().map(read_values).transpose()?;
    let preview = preview_path
        .as_deref()
        .map(|path| read_bytes(path, "preview PNG"))
        .transpose()?;

    let features = UiFeatureSet::compiled();
    let report = document.validate(&features, &bindings);
    if !report.is_valid() {
        print_report(&document_path, &report, false)?;
        return Err(format!(
            "UI document handoff failed with {} diagnostic(s)",
            report.diagnostics.len()
        ));
    }
    if let (Some(values), Some(values_path)) = (&values, values_path.as_deref()) {
        let report = validate_ui_binding_values(&bindings, values);
        if !report.is_valid() {
            print_report(values_path, &report, false)?;
            return Err(format!(
                "UI handoff values failed with {} diagnostic(s)",
                report.diagnostics.len()
            ));
        }
    }

    let package = UiAiHandoffPackage::build(
        &document,
        &features,
        &bindings,
        values.as_ref(),
        preview.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    prepare_output_directory(&output_path, force)?;
    write_package(&output_path, &package)?;
    println!("{}: AI handoff ready", output_path.display());
    Ok(())
}

fn read_document(path: &Path) -> Result<UiDocument, String> {
    let source = read_text(path, "UI document")?;
    UiDocument::from_json(&source)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

fn read_bindings(path: Option<&Path>) -> Result<UiBindingSchema, String> {
    match path {
        Some(path) => {
            let source = read_text(path, "binding schema")?;
            serde_json::from_str(&source)
                .map_err(|error| format!("cannot parse {}: {error}", path.display()))
        }
        None => Ok(UiBindingSchema::default()),
    }
}

fn read_values(path: &Path) -> Result<BTreeMap<String, Value>, String> {
    let source = read_text(path, "binding values")?;
    serde_json::from_str(&source)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

fn read_text(path: &Path, description: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("cannot read {description} {}: {error}", path.display()))
}

fn read_bytes(path: &Path, description: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("cannot read {description} {}: {error}", path.display()))
}

fn prepare_output_directory(path: &Path, force: bool) -> Result<(), String> {
    if path.exists() {
        if !path.is_dir() {
            return Err(format!("output {} is not a directory", path.display()));
        }
        let mut entries = fs::read_dir(path)
            .map_err(|error| format!("cannot inspect output {}: {error}", path.display()))?;
        if entries.next().is_some() && !force {
            return Err(format!(
                "output {} is not empty; pass --force to replace handoff files",
                path.display()
            ));
        }
    } else {
        fs::create_dir_all(path)
            .map_err(|error| format!("cannot create output {}: {error}", path.display()))?;
    }
    Ok(())
}

fn write_package(path: &Path, package: &UiAiHandoffPackage) -> Result<(), String> {
    write_output(path, "document.json", package.document_json.as_bytes())?;
    write_output(path, "bindings.json", package.bindings_json.as_bytes())?;
    write_optional_output(
        path,
        "values.json",
        package.values_json.as_ref().map(String::as_bytes),
    )?;
    write_optional_output(path, "preview.png", package.preview_png.as_deref())?;
    write_output(path, "handoff.json", package.handoff_json.as_bytes())
}

fn write_optional_output(path: &Path, name: &str, bytes: Option<&[u8]>) -> Result<(), String> {
    match bytes {
        Some(bytes) => write_output(path, name, bytes),
        None => {
            let stale_path = path.join(name);
            if stale_path.exists() {
                fs::remove_file(&stale_path).map_err(|error| {
                    format!("cannot remove stale {}: {error}", stale_path.display())
                })?;
            }
            Ok(())
        }
    }
}

fn write_output(path: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let output = path.join(name);
    fs::write(&output, bytes).map_err(|error| format!("cannot write {}: {error}", output.display()))
}

fn print_report(path: &Path, report: &UiValidationReport, json: bool) -> Result<(), String> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(report)
                .map_err(|error| format!("cannot serialize validation report: {error}"))?
        );
    } else if report.is_valid() {
        println!("{}: valid", path.display());
    } else {
        for diagnostic in &report.diagnostics {
            println!(
                "error[{:?}] {}: {}",
                diagnostic.code, diagnostic.path, diagnostic.message
            );
        }
    }
    Ok(())
}

fn usage() -> String {
    concat!(
        "usage:\n",
        "  zsui-uic check <document.json> [--bindings <bindings.json>] [--json]\n",
        "  zsui-uic handoff <document.json> [--bindings <bindings.json>] ",
        "[--values <values.json>] --output <directory> ",
        "[--preview <preview.png>] [--force]"
    )
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_handoff_arguments_and_screenshot_alias() {
        let command = parse_command(args(&[
            "handoff",
            "view.json",
            "--bindings",
            "bindings.json",
            "--values",
            "values.json",
            "--output",
            "handoff",
            "--screenshot",
            "native.png",
            "--force",
        ]))
        .unwrap();

        assert_eq!(
            command,
            Command::Handoff {
                document_path: PathBuf::from("view.json"),
                bindings_path: Some(PathBuf::from("bindings.json")),
                values_path: Some(PathBuf::from("values.json")),
                output_path: PathBuf::from("handoff"),
                preview_path: Some(PathBuf::from("native.png")),
                force: true,
            }
        );
    }

    #[test]
    fn handoff_requires_output_path() {
        let error = parse_command(args(&["handoff", "view.json"])).unwrap_err();
        assert!(error.contains("--output is required"));
    }
}
