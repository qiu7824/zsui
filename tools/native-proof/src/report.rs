use std::fs;
use std::path::Path;

use crate::image_diff::{ImageComparison, ImageDiffMetrics};

#[derive(Debug, Clone)]
pub struct ScenarioReport {
    pub name: String,
    pub semantic_differences: Vec<String>,
    pub image: Option<ImageComparison>,
    pub missing_baseline: Option<String>,
    pub invalid_json: Option<String>,
    pub passed: bool,
}

impl ScenarioReport {
    pub fn evaluated(
        name: String,
        semantic_differences: Vec<String>,
        image: Result<ImageComparison, String>,
    ) -> Self {
        let image = match image {
            Ok(image) => Some(image),
            Err(error) => {
                return Self {
                    name,
                    semantic_differences,
                    image: None,
                    missing_baseline: None,
                    invalid_json: Some(error),
                    passed: false,
                }
            }
        };
        let passed =
            semantic_differences.is_empty() && image.as_ref().is_some_and(|item| item.passed);
        Self {
            name,
            semantic_differences,
            image,
            missing_baseline: None,
            invalid_json: None,
            passed,
        }
    }

    pub fn missing_baseline(name: String, path: String) -> Self {
        Self {
            name,
            semantic_differences: Vec::new(),
            image: None,
            missing_baseline: Some(path),
            invalid_json: None,
            passed: false,
        }
    }

    pub fn invalid_json(name: String, error: String) -> Self {
        Self {
            name,
            semantic_differences: Vec::new(),
            image: None,
            missing_baseline: None,
            invalid_json: Some(error),
            passed: false,
        }
    }
}

pub fn write_summary(path: &Path, reports: &[ScenarioReport]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let mut markdown = String::new();
    markdown.push_str("# ZSUI Native Proof Comparison\n\n");
    markdown
        .push_str("| Scenario | Semantic | Full window diff | Critical region diff | Status |\n");
    markdown.push_str("| --- | --- | --- | --- | --- |\n");

    for report in reports {
        let semantic = if report.semantic_differences.is_empty() {
            "PASS".to_string()
        } else {
            format!("FAIL ({})", report.semantic_differences.len())
        };
        let (full, critical, status) = if let Some(error) = &report.invalid_json {
            (format!("ERROR: {error}"), String::new(), "FAIL")
        } else if let Some(path) = &report.missing_baseline {
            (format!("missing baseline: {path}"), String::new(), "FAIL")
        } else if let Some(image) = &report.image {
            (
                format_metrics(&image.full),
                format_metrics(&image.critical),
                if report.passed { "PASS" } else { "FAIL" },
            )
        } else {
            (String::new(), String::new(), "FAIL")
        };
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            report.name, semantic, full, critical, status
        ));
    }

    markdown.push_str("\n## Details\n\n");
    for report in reports {
        if report.passed {
            continue;
        }
        markdown.push_str(&format!("### {}\n", report.name));
        if let Some(error) = &report.invalid_json {
            markdown.push_str(&format!("- {error}\n"));
        }
        if let Some(path) = &report.missing_baseline {
            markdown.push_str(&format!("- missing baseline file: {path}\n"));
        }
        for difference in &report.semantic_differences {
            markdown.push_str(&format!("- {difference}\n"));
        }
        if let Some(image) = report.image.as_ref().filter(|image| !image.passed) {
            markdown.push_str(&format!(
                "- pixels over threshold (critical regions checked: {}), diff image: {}\n",
                image.critical_region_count, image.diff_path
            ));
        }
        markdown.push('\n');
    }

    fs::write(path, markdown)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn format_metrics(metrics: &ImageDiffMetrics) -> String {
    format!(
        "{:.3}% ({}/{}, max Δ {})",
        metrics.difference_percent,
        metrics.different_pixels,
        metrics.total_pixels,
        metrics.max_channel_delta
    )
}

pub fn print_summary(reports: &[ScenarioReport]) {
    let failed = reports.iter().filter(|report| !report.passed).count();
    println!(
        "compared {} scenarios, {} passed, {} failed",
        reports.len(),
        reports.len() - failed,
        failed
    );
    for report in reports {
        println!(
            "- {}: {}",
            report.name,
            if report.passed { "PASS" } else { "FAIL" }
        );
    }
}
