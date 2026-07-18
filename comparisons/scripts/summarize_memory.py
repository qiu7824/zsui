#!/usr/bin/env python3
import argparse
import json
import statistics
from pathlib import Path


SCHEMA = "zsui.ui-memory-comparison/v1"
FRAMEWORKS = ("slint", "iced")


def median_value(reports, field):
    values = [report[field] for report in reports if report.get(field) is not None]
    return int(statistics.median(values)) if values else None


def mib(value):
    return "—" if value is None else f"{value / 1024 / 1024:.2f}"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--runner", required=True)
    parser.add_argument("--slint-binary", type=Path, required=True)
    parser.add_argument("--iced-binary", type=Path, required=True)
    parser.add_argument("--github-summary", type=Path)
    args = parser.parse_args()

    binaries = {
        "slint": args.slint_binary,
        "iced": args.iced_binary,
    }
    result = {
        "schema": SCHEMA,
        "runner": args.runner,
        "scenario": "notepad-900x620-first-frame-idle",
        "sample_count": 5,
        "frameworks": {},
    }
    if not args.slint_binary.is_file() or not args.iced_binary.is_file():
        raise SystemExit("comparison release binaries are missing")
    markdown = [
        f"## {args.runner} UI memory comparison",
        "",
        "| Framework | Median RSS MiB | Median peak RSS MiB | Median private MiB | Median PSS MiB | Median physical footprint MiB | Binary MiB |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]

    for framework in FRAMEWORKS:
        reports = []
        for path in sorted(args.input.glob(f"{framework}-notepad-run*.json")):
            report = json.loads(path.read_text(encoding="utf-8"))
            if report.get("schema") != SCHEMA:
                raise SystemExit(f"unexpected schema in {path}")
            if report.get("framework") != framework:
                raise SystemExit(f"unexpected framework in {path}")
            if report.get("scenario") != "notepad":
                raise SystemExit(f"unexpected scenario in {path}")
            if report.get("sample_point") != "first_frame_idle":
                raise SystemExit(f"unexpected sample point in {path}")
            if report.get("resident_bytes", 0) <= 0:
                raise SystemExit(f"missing resident memory in {path}")
            reports.append(report)
        if len(reports) != 5:
            raise SystemExit(f"expected five {framework} reports, got {len(reports)}")

        platform = reports[0]["platform"]
        if any(report["platform"] != platform for report in reports):
            raise SystemExit(f"mixed target platforms in {framework} reports")
        if platform == "macos" and any(
            report.get("physical_footprint_bytes") is None for report in reports
        ):
            raise SystemExit(f"missing macOS physical footprint for {framework}")
        if platform == "linux" and any(
            report.get("private_resident_bytes") is None
            or report.get("proportional_set_size_bytes") is None
            for report in reports
        ):
            raise SystemExit(f"missing Linux private/PSS memory for {framework}")

        summary = {
            "platform": platform,
            "architecture": reports[0]["architecture"],
            "binary_bytes": binaries[framework].stat().st_size,
            "median_resident_bytes": median_value(reports, "resident_bytes"),
            "minimum_resident_bytes": min(report["resident_bytes"] for report in reports),
            "maximum_resident_bytes": max(report["resident_bytes"] for report in reports),
            "median_peak_resident_bytes": median_value(reports, "peak_resident_bytes"),
            "median_private_resident_bytes": median_value(
                reports, "private_resident_bytes"
            ),
            "median_proportional_set_size_bytes": median_value(
                reports, "proportional_set_size_bytes"
            ),
            "median_physical_footprint_bytes": median_value(
                reports, "physical_footprint_bytes"
            ),
            "median_peak_physical_footprint_bytes": median_value(
                reports, "peak_physical_footprint_bytes"
            ),
            "runs": reports,
        }
        result["frameworks"][framework] = summary
        markdown.append(
            "| {framework} | {rss} | {peak} | {private} | {pss} | {footprint} | {binary} |".format(
                framework=framework.capitalize(),
                rss=mib(summary["median_resident_bytes"]),
                peak=mib(summary["median_peak_resident_bytes"]),
                private=mib(summary["median_private_resident_bytes"]),
                pss=mib(summary["median_proportional_set_size_bytes"]),
                footprint=mib(summary["median_physical_footprint_bytes"]),
                binary=mib(summary["binary_bytes"]),
            )
        )

    summary_path = args.input / "summary.json"
    markdown_path = args.input / "summary.md"
    summary_path.write_text(
        json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    markdown_path.write_text("\n".join(markdown) + "\n", encoding="utf-8")
    if args.github_summary:
        with args.github_summary.open("a", encoding="utf-8") as summary_file:
            summary_file.write(markdown_path.read_text(encoding="utf-8"))
    print(markdown_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
