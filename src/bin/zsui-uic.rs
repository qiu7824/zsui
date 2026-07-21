use std::{env, fs, path::PathBuf, process::ExitCode};

use zsui::ui_document::{UiBindingSchema, UiDocument, UiFeatureSet, UiValidationReport};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    if arguments.next().as_deref() != Some("check") {
        return Err(usage());
    }
    let document_path = arguments.next().map(PathBuf::from).ok_or_else(usage)?;
    let mut bindings_path = None;
    let mut json = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--bindings" => {
                bindings_path = Some(
                    arguments
                        .next()
                        .map(PathBuf::from)
                        .ok_or_else(|| "--bindings requires a JSON file path".to_owned())?,
                );
            }
            "--json" => json = true,
            _ => return Err(format!("unknown argument {argument:?}\n{}", usage())),
        }
    }

    let document_source = fs::read_to_string(&document_path)
        .map_err(|error| format!("cannot read {}: {error}", document_path.display()))?;
    let document = UiDocument::from_json(&document_source)
        .map_err(|error| format!("cannot parse {}: {error}", document_path.display()))?;
    let bindings = match bindings_path {
        Some(path) => {
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            serde_json::from_str::<UiBindingSchema>(&source)
                .map_err(|error| format!("cannot parse {}: {error}", path.display()))?
        }
        None => UiBindingSchema::default(),
    };

    let report = document.validate(&UiFeatureSet::compiled(), &bindings);
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

fn print_report(path: &PathBuf, report: &UiValidationReport, json: bool) -> Result<(), String> {
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
    "usage: zsui-uic check <document.json> [--bindings <bindings.json>] [--json]".to_owned()
}
