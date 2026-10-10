mod compare;
mod image_diff;
mod report;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use compare::CompareOptions;

const USAGE: &str = "\
ZSUI native proof tool

USAGE:
    zsui-native-proof compare --baseline <DIR> --actual <DIR> --diff <DIR> [OPTIONS]
    zsui-native-proof init --actual <DIR> --baseline <DIR>

OPTIONS:
    --color-tolerance <0-255>              channel delta that counts as a difference (default: 32)
    --critical-threshold <PERCENT>         maximum critical-region difference (default: 0.5)
    --full-threshold <PERCENT>             maximum full-window difference (default: 1.0)
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!();
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("compare") => run_compare(&args[1..]),
        Some("init") => run_init(&args[1..]),
        _ => {
            eprintln!("{USAGE}");
            Ok(2)
        }
    }
}

fn run_compare(args: &[String]) -> Result<u8, String> {
    let mut baseline: Option<PathBuf> = None;
    let mut actual: Option<PathBuf> = None;
    let mut diff: Option<PathBuf> = None;
    let mut color_tolerance = 32u8;
    let mut critical_threshold_percent = 0.5f64;
    let mut full_threshold_percent = 1.0f64;

    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--baseline" => {
                index += 1;
                baseline = Some(required_value(args, index, "--baseline")?);
            }
            "--actual" => {
                index += 1;
                actual = Some(required_value(args, index, "--actual")?);
            }
            "--diff" => {
                index += 1;
                diff = Some(required_value(args, index, "--diff")?);
            }
            "--color-tolerance" => {
                index += 1;
                color_tolerance = parse_value(args, index, "--color-tolerance")?;
            }
            "--critical-threshold" => {
                index += 1;
                critical_threshold_percent = parse_value(args, index, "--critical-threshold")?;
            }
            "--full-threshold" => {
                index += 1;
                full_threshold_percent = parse_value(args, index, "--full-threshold")?;
            }
            other => return Err(format!("unknown compare option `{other}`")),
        }
        index += 1;
    }

    let baseline = required_option(baseline, "--baseline")?;
    let actual = required_option(actual, "--actual")?;
    let diff = required_option(diff, "--diff")?;

    let reports = compare::run_compare(CompareOptions {
        baseline_dir: baseline,
        actual_dir: actual,
        diff_dir: diff.clone(),
        color_tolerance,
        critical_threshold_percent,
        full_threshold_percent,
    })?;

    let summary_path = diff.join("summary.md");
    report::write_summary(&summary_path, &reports)?;
    report::print_summary(&reports);

    if reports.iter().any(|report| !report.passed) {
        println!("summary written to {}", summary_path.display());
        Ok(1)
    } else {
        println!("summary written to {}", summary_path.display());
        Ok(0)
    }
}

fn run_init(args: &[String]) -> Result<u8, String> {
    let mut actual: Option<PathBuf> = None;
    let mut baseline: Option<PathBuf> = None;

    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--actual" => {
                index += 1;
                actual = Some(required_value(args, index, "--actual")?);
            }
            "--baseline" => {
                index += 1;
                baseline = Some(required_value(args, index, "--baseline")?);
            }
            other => return Err(format!("unknown init option `{other}`")),
        }
        index += 1;
    }

    let actual = required_option(actual, "--actual")?;
    let baseline = required_option(baseline, "--baseline")?;
    let copied = compare::init_baseline(&actual, &baseline)?;
    println!("copied {copied} proof artifacts to {}", baseline.display());
    Ok(0)
}

fn required_value(args: &[String], index: usize, option: &str) -> Result<PathBuf, String> {
    args.get(index)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {option}"))
}

fn parse_value<T>(args: &[String], index: usize, option: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    args.get(index)
        .ok_or_else(|| format!("missing value for {option}"))?
        .parse::<T>()
        .map_err(|_| format!("invalid value for {option}"))
}

fn required_option(value: Option<PathBuf>, option: &str) -> Result<PathBuf, String> {
    value.ok_or_else(|| format!("missing required option {option}"))
}
