#!/usr/bin/env python3
"""Measure the fixed ZSUI UI matrix on real macOS or Linux windows."""

from __future__ import annotations

import argparse
import json
import math
import os
import pathlib
import re
import shutil
import signal
import statistics
import struct
import subprocess
import tempfile
import time
from dataclasses import dataclass
from typing import Any


PROFILES = {
    "minimal": {
        "name": "Minimal",
        "workload": "Window + Text + Button",
        "comparison_contract": "one 1000x700 window, bilingual title/body text and one button",
    },
    "common": {
        "name": "Common",
        "workload": "Navigation + Form + List + Dialog",
        "comparison_contract": "invoice assistant with navigation, editable form, two-row list and confirmation surface",
    },
    "full": {
        "name": "Full Native App",
        "workload": "20-30 common component instances",
        "comparison_contract": "24 visible control instances across navigation, input, selection, collection, progress and action families",
    },
    "viewer": {
        "name": "Viewer",
        "workload": "UiDocument + hot reload + all document components",
        "comparison_contract": "one document surface, 250 ms source polling and all current document component kinds",
    },
}

FRAMEWORKS = {
    "zsui": "ZSUI",
    "egui": "eframe/egui",
    "iced": "Iced",
    "slint": "Slint",
    "tauri": "Tauri 2 / system WebView",
}


def run(command: list[str], *, timeout: float = 20, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        check=check,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def linux_process_table() -> dict[int, int]:
    table: dict[int, int] = {}
    for entry in pathlib.Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            value = (entry / "stat").read_text(encoding="utf-8")
            close = value.rfind(")")
            fields = value[close + 2 :].split()
            table[int(entry.name)] = int(fields[1])
        except (OSError, ValueError, IndexError):
            pass
    return table


def macos_process_table() -> dict[int, int]:
    table: dict[int, int] = {}
    for line in run(["ps", "-axo", "pid=,ppid="], timeout=10).stdout.splitlines():
        fields = line.split()
        if len(fields) == 2:
            table[int(fields[0])] = int(fields[1])
    return table


def process_tree(root: int, platform: str) -> list[int]:
    table = linux_process_table() if platform == "linux" else macos_process_table()
    result = {root}
    changed = True
    while changed:
        changed = False
        for process_id, parent_id in table.items():
            if parent_id in result and process_id not in result:
                result.add(process_id)
                changed = True
    return sorted(result)


def linux_process_name(process_id: int) -> str:
    try:
        return pathlib.Path(f"/proc/{process_id}/comm").read_text(encoding="utf-8").strip()
    except OSError:
        return str(process_id)


def macos_process_name(process_id: int) -> str:
    result = run(["ps", "-o", "comm=", "-p", str(process_id)], timeout=5, check=False)
    return pathlib.Path(result.stdout.strip()).name or str(process_id)


def parse_kib_fields(path: pathlib.Path) -> dict[str, int]:
    values: dict[str, int] = {}
    try:
        for line in path.read_text(encoding="utf-8").splitlines():
            match = re.match(r"^([A-Za-z_]+):\s+(\d+)\s+kB$", line)
            if match:
                values[match.group(1)] = int(match.group(2)) * 1024
    except OSError:
        pass
    return values


def parse_vmmap_size(value: str) -> int | None:
    match = re.search(r"([0-9]+(?:\.[0-9]+)?)\s*([KMG])", value, re.IGNORECASE)
    if not match:
        return None
    scale = {"K": 1024, "M": 1024**2, "G": 1024**3}[match.group(2).upper()]
    return int(float(match.group(1)) * scale)


def macos_physical_footprint(process_id: int) -> tuple[int | None, int | None]:
    result = run(["vmmap", "-summary", str(process_id)], timeout=15, check=False)
    if result.returncode != 0:
        return None, None
    footprint = None
    peak = None
    for line in result.stdout.splitlines():
        if line.startswith("Physical footprint:"):
            footprint = parse_vmmap_size(line)
        elif line.startswith("Physical footprint (peak):"):
            peak = parse_vmmap_size(line)
    return footprint, peak


def memory_snapshot(root: int, platform: str) -> dict[str, Any]:
    pids = process_tree(root, platform)
    rss = pss = private = virtual = peak = 0
    physical = peak_physical = 0
    physical_available = True
    names: list[str] = []
    alive = 0
    for process_id in pids:
        if platform == "linux":
            rollup = parse_kib_fields(pathlib.Path(f"/proc/{process_id}/smaps_rollup"))
            status = parse_kib_fields(pathlib.Path(f"/proc/{process_id}/status"))
            if not rollup and not status:
                continue
            alive += 1
            names.append(linux_process_name(process_id))
            rss += rollup.get("Rss", status.get("VmRSS", 0))
            pss += rollup.get("Pss", 0)
            private += rollup.get("Private_Clean", 0) + rollup.get("Private_Dirty", 0)
            virtual += status.get("VmSize", 0)
            peak += status.get("VmHWM", rollup.get("Rss", 0))
        else:
            result = run(
                ["ps", "-o", "rss=,vsz=", "-p", str(process_id)],
                timeout=5,
                check=False,
            )
            fields = result.stdout.split()
            if len(fields) < 2:
                continue
            alive += 1
            names.append(macos_process_name(process_id))
            resident = int(fields[0]) * 1024
            rss += resident
            virtual += int(fields[1]) * 1024
            peak += resident
            current_footprint, process_peak = macos_physical_footprint(process_id)
            if current_footprint is None:
                physical_available = False
            else:
                physical += current_footprint
            if process_peak is not None:
                peak_physical += process_peak
    return {
        "source": "proc_smaps_rollup" if platform == "linux" else "ps_and_vmmap",
        "process_count": alive,
        "process_names": sorted(set(names)),
        "rss_bytes": rss,
        "peak_rss_bytes": max(peak, rss),
        "private_rss_bytes": private if platform == "linux" else None,
        "proportional_set_size_bytes": pss if platform == "linux" else None,
        "physical_footprint_bytes": physical if physical_available else None,
        "peak_physical_footprint_bytes": peak_physical or None,
        "virtual_bytes": virtual,
    }


def average_memory(root: int, platform: str, samples: int, interval: float) -> dict[str, Any]:
    snapshots = []
    for index in range(samples):
        snapshot = memory_snapshot(root, platform)
        if snapshot["process_count"]:
            snapshots.append(snapshot)
        if index + 1 < samples:
            time.sleep(interval)
    if not snapshots:
        raise RuntimeError(f"process tree {root} disappeared before memory sampling")
    result = dict(snapshots[-1])
    for field in (
        "rss_bytes",
        "private_rss_bytes",
        "proportional_set_size_bytes",
        "physical_footprint_bytes",
        "virtual_bytes",
    ):
        values = [item[field] for item in snapshots if item[field] is not None]
        result[field] = int(statistics.mean(values)) if values else None
    result["peak_rss_bytes"] = max(item["peak_rss_bytes"] for item in snapshots)
    physical_peaks = [
        item["peak_physical_footprint_bytes"]
        for item in snapshots
        if item["peak_physical_footprint_bytes"] is not None
    ]
    result["peak_physical_footprint_bytes"] = max(physical_peaks) if physical_peaks else None
    return result


def parse_cpu_time(value: str) -> float:
    value = value.strip()
    days = 0
    if "-" in value:
        day, value = value.split("-", 1)
        days = int(day)
    fields = [float(part) for part in value.split(":")]
    if len(fields) == 3:
        hours, minutes, seconds = fields
    elif len(fields) == 2:
        hours, (minutes, seconds) = 0, fields
    else:
        hours, minutes, seconds = 0, 0, fields[0]
    return days * 86400 + hours * 3600 + minutes * 60 + seconds


def cpu_total(root: int, platform: str) -> float:
    total = 0.0
    if platform == "linux":
        ticks = os.sysconf(os.sysconf_names["SC_CLK_TCK"])
        for process_id in process_tree(root, platform):
            try:
                value = pathlib.Path(f"/proc/{process_id}/stat").read_text(encoding="utf-8")
                close = value.rfind(")")
                fields = value[close + 2 :].split()
                total += (int(fields[11]) + int(fields[12])) / ticks
            except (OSError, ValueError, IndexError):
                pass
    else:
        for process_id in process_tree(root, platform):
            result = run(["ps", "-o", "time=", "-p", str(process_id)], timeout=5, check=False)
            if result.stdout.strip():
                total += parse_cpu_time(result.stdout)
    return total


def macos_windows(probe: pathlib.Path, process_ids: list[int]) -> list[dict[str, Any]]:
    result = run([str(probe), "windows", ",".join(map(str, process_ids))], timeout=5)
    return json.loads(result.stdout or "[]")


def linux_windows(process_ids: list[int]) -> list[dict[str, Any]]:
    windows: list[dict[str, Any]] = []
    for process_id in process_ids:
        result = run(
            ["xdotool", "search", "--onlyvisible", "--pid", str(process_id)],
            timeout=3,
            check=False,
        )
        for value in result.stdout.split():
            geometry = run(
                ["xdotool", "getwindowgeometry", "--shell", value],
                timeout=3,
                check=False,
            )
            fields = dict(
                line.split("=", 1) for line in geometry.stdout.splitlines() if "=" in line
            )
            width = int(fields.get("WIDTH", "0"))
            height = int(fields.get("HEIGHT", "0"))
            if width >= 320 and height >= 240:
                windows.append(
                    {
                        "pid": process_id,
                        "window_id": int(value),
                        "x": int(fields.get("X", "0")),
                        "y": int(fields.get("Y", "0")),
                        "width": width,
                        "height": height,
                    }
                )
    return windows


def wait_for_window(
    process: subprocess.Popen[bytes],
    platform: str,
    probe: pathlib.Path | None,
    started: float,
    timeout: float = 25,
) -> tuple[dict[str, Any], float]:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"process {process.pid} exited before presenting a window")
        process_ids = process_tree(process.pid, platform)
        windows = (
            linux_windows(process_ids)
            if platform == "linux"
            else macos_windows(probe or pathlib.Path(), process_ids)
        )
        if windows:
            return max(windows, key=lambda item: item["width"] * item["height"]), (
                time.monotonic() - started
            ) * 1000
        time.sleep(0.025)
    raise RuntimeError(f"process {process.pid} did not present a non-empty native window")


def valid_png_window_capture(output: pathlib.Path, minimum_width: int, minimum_height: int) -> bool:
    try:
        data = output.read_bytes()
    except OSError:
        return False
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        return False
    width, height = struct.unpack(">II", data[16:24])
    return width >= minimum_width and height >= minimum_height


def capture_window(window: dict[str, Any], platform: str, output: pathlib.Path) -> bool:
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        output.unlink()
    if platform == "linux":
        result = run(
            ["import", "-window", str(window["window_id"]), str(output)],
            timeout=15,
            check=False,
        )
    else:
        result = run(
            ["screencapture", "-x", "-l", str(window["window_id"]), str(output)],
            timeout=15,
            check=False,
        )
    return result.returncode == 0 and valid_png_window_capture(
        output,
        max(320, int(window["width"]) - 16),
        max(240, int(window["height"]) - 48),
    )


def stop_process(process: subprocess.Popen[bytes], platform: str) -> None:
    pids = process_tree(process.pid, platform)
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=1.5)
        except subprocess.TimeoutExpired:
            pass
    for process_id in reversed(pids):
        try:
            os.kill(process_id, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        pass


@dataclass
class Launch:
    process: subprocess.Popen[bytes]
    window: dict[str, Any]
    startup_ms: float
    first_frame_ms: float


def launch_application(
    application: dict[str, Any],
    platform: str,
    probe: pathlib.Path | None,
    extra_arguments: list[str] | None = None,
    frame_probe: pathlib.Path | None = None,
) -> Launch:
    arguments = list(application.get("arguments", [])) + list(extra_arguments or [])
    started = time.monotonic()
    process = subprocess.Popen(
        [application["executable"], *arguments],
        cwd=application["working_directory"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    try:
        window, startup_ms = wait_for_window(process, platform, probe, started)
        capture = frame_probe or pathlib.Path(tempfile.gettempdir()) / f"zsui-frame-{process.pid}.png"
        if not capture_window(window, platform, capture):
            raise RuntimeError(
                f"{application['framework']} {application['profile']} window capture failed"
            )
        first_frame_ms = (time.monotonic() - started) * 1000
        if frame_probe is None:
            capture.unlink(missing_ok=True)
        return Launch(process, window, startup_ms, first_frame_ms)
    except Exception:
        stop_process(process, platform)
        raise


def hide_application(launch: Launch, platform: str, probe: pathlib.Path | None) -> None:
    if platform == "linux":
        result = run(
            ["xdotool", "windowunmap", str(launch.window["window_id"])],
            timeout=5,
            check=False,
        )
    else:
        result = run([str(probe), "hide", str(launch.process.pid)], timeout=5, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"could not hide process {launch.process.pid}: {result.stderr}")


def drive_linux_repaint(window: dict[str, Any], seconds: float, hz: int) -> None:
    base_x = int(window["x"])
    base_y = int(window["y"])
    deadline = time.monotonic() + seconds
    frame = 0
    interval = 1.0 / hz
    while time.monotonic() < deadline:
        target = time.monotonic() + interval
        run(
            [
                "xdotool",
                "windowmove",
                str(window["window_id"]),
                str(base_x + (frame & 1)),
                str(base_y),
            ],
            timeout=2,
            check=False,
        )
        frame += 1
        time.sleep(max(0.0, target - time.monotonic()))


def measure_cpu(
    launch: Launch,
    platform: str,
    seconds: float,
    repaint: bool,
    external_repaint: bool,
    probe: pathlib.Path | None,
) -> dict[str, Any]:
    before = cpu_total(launch.process.pid, platform)
    started = time.monotonic()
    if repaint and external_repaint:
        if platform == "linux":
            drive_linux_repaint(launch.window, seconds, 60)
            driver = "x11_configure_expose_60hz"
        else:
            run(
                [str(probe), "churn", str(launch.process.pid), str(seconds), "60"],
                timeout=seconds + 10,
            )
            driver = "nsrunningapplication_visibility_churn_60hz"
    else:
        time.sleep(seconds)
        driver = "application_render_loop" if repaint else "stationary_window"
    elapsed = max(time.monotonic() - started, 0.001)
    after = cpu_total(launch.process.pid, platform)
    machine_percent = max(0.0, after - before) / elapsed / (os.cpu_count() or 1) * 100
    return {
        "sample_seconds": elapsed,
        "machine_percent": round(machine_percent, 3),
        "requested_hz": 60 if repaint else None,
        "driver": driver,
    }


def measure_application(
    application: dict[str, Any],
    platform: str,
    probe: pathlib.Path | None,
    output: pathlib.Path,
    startup_runs: int,
    memory_samples: int,
    warmup_seconds: float,
    cpu_seconds: float,
) -> dict[str, Any]:
    label = f"{application['framework']} / {application['profile']}"
    print(f"measure: {label}", flush=True)
    screenshot = output / f"{application['framework']}-{application['profile']}.png"
    startups: list[float] = []
    frames: list[float] = []
    page_memory = hidden_memory = idle_cpu = None
    first = None
    for run_index in range(startup_runs):
        launch = launch_application(
            application,
            platform,
            probe,
            frame_probe=screenshot if run_index == 0 else None,
        )
        try:
            startups.append(round(launch.startup_ms, 2))
            frames.append(round(launch.first_frame_ms, 2))
            if run_index == 0:
                first = launch
                time.sleep(warmup_seconds)
                page_memory = average_memory(
                    launch.process.pid, platform, memory_samples, 0.1
                )
                idle_cpu = measure_cpu(
                    launch, platform, cpu_seconds, False, False, probe
                )
                hide_application(launch, platform, probe)
                time.sleep(warmup_seconds)
                hidden_memory = average_memory(
                    launch.process.pid, platform, memory_samples, 0.1
                )
        finally:
            stop_process(launch.process, platform)

    empty_launch = launch_application(
        application, platform, probe, extra_arguments=["--benchmark-empty"]
    )
    try:
        time.sleep(warmup_seconds)
        empty_memory = average_memory(
            empty_launch.process.pid, platform, memory_samples, 0.1
        )
    finally:
        stop_process(empty_launch.process, platform)

    repaint_arguments = [] if application["framework"] == "zsui" else ["--benchmark-repaint"]
    repaint_launch = launch_application(
        application, platform, probe, extra_arguments=repaint_arguments
    )
    try:
        time.sleep(warmup_seconds)
        repaint_cpu = measure_cpu(
            repaint_launch,
            platform,
            cpu_seconds,
            True,
            application["framework"] == "zsui",
            probe,
        )
        repaint_memory = memory_snapshot(repaint_launch.process.pid, platform)
    finally:
        stop_process(repaint_launch.process, platform)

    assert first is not None and page_memory is not None and hidden_memory is not None
    assert idle_cpu is not None
    peak_rss = max(
        empty_memory["peak_rss_bytes"],
        page_memory["peak_rss_bytes"],
        hidden_memory["peak_rss_bytes"],
        repaint_memory["peak_rss_bytes"],
    )
    page_memory["peak_rss_bytes"] = peak_rss
    warm_startups = startups[1:]
    warm_frames = frames[1:]
    return {
        "framework": FRAMEWORKS[application["framework"]],
        "profile": PROFILES[application["profile"]]["name"],
        "executable": application["executable"],
        "arguments": application.get("arguments", []),
        "executable_bytes": pathlib.Path(application["executable"]).stat().st_size,
        "cold_start": {
            "method": "best_effort_first_launch_after_release_build",
            "file_cache_purged": False,
            "startup_to_window_ms": startups[0],
            "first_presented_frame_ms": frames[0],
        },
        "warm_start": {
            "runs": len(warm_startups),
            "startup_to_window_median_ms": round(statistics.median(warm_startups), 2),
            "first_presented_frame_median_ms": round(statistics.median(warm_frames), 2),
            "startup_samples_ms": warm_startups,
            "first_frame_samples_ms": warm_frames,
        },
        "empty_window": {
            "startup_to_window_ms": round(empty_launch.startup_ms, 2),
            "first_presented_frame_ms": round(empty_launch.first_frame_ms, 2),
            "memory": empty_memory,
        },
        "full_page_memory": page_memory,
        "hidden_memory": hidden_memory,
        "idle_cpu": idle_cpu,
        "repaint_cpu": repaint_cpu,
        "screenshot_captured": screenshot.exists() and screenshot.stat().st_size >= 1024,
        "screenshot": str(screenshot),
    }


def mib(value: int | None) -> str:
    return "n/a" if value is None else f"{value / 1048576:.2f} MiB"


def write_markdown(report: dict[str, Any], path: pathlib.Path) -> None:
    lines = [
        f"# UI performance matrix — {report['machine']['runner']}",
        "",
        "Equal-complexity rows only. ZSUI applications and the Viewer are separate binaries.",
    ]
    for profile, profile_spec in PROFILES.items():
        lines += [
            "",
            f"## {profile_spec['name']}",
            "",
            "| Framework | Binary | Cold frame | Warm frame | Empty RSS | Page RSS | Hidden RSS | Peak RSS | Private/PSS | Idle CPU | Repaint CPU |",
            "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
        ]
        for framework, framework_name in FRAMEWORKS.items():
            item = report["implementations"][framework][profile]
            memory = item["full_page_memory"]
            private = memory["private_rss_bytes"]
            pss = memory["proportional_set_size_bytes"]
            private_pss = f"{mib(private)} / {mib(pss)}"
            lines.append(
                f"| {framework_name} | {mib(item['executable_bytes'])} | "
                f"{item['cold_start']['first_presented_frame_ms']} ms | "
                f"{item['warm_start']['first_presented_frame_median_ms']} ms | "
                f"{mib(item['empty_window']['memory']['rss_bytes'])} | "
                f"{mib(memory['rss_bytes'])} | {mib(item['hidden_memory']['rss_bytes'])} | "
                f"{mib(memory['peak_rss_bytes'])} | {private_pss} | "
                f"{item['idle_cpu']['machine_percent']}% | "
                f"{item['repaint_cpu']['machine_percent']}% |"
            )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--applications", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--platform", choices=("linux", "macos"), required=True)
    parser.add_argument("--runner", required=True)
    parser.add_argument("--macos-probe", type=pathlib.Path)
    parser.add_argument("--startup-runs", type=int, default=3)
    parser.add_argument("--memory-samples", type=int, default=3)
    parser.add_argument("--warmup-seconds", type=float, default=2)
    parser.add_argument("--cpu-seconds", type=float, default=2)
    arguments = parser.parse_args()
    if arguments.startup_runs < 2 or arguments.memory_samples < 1:
        parser.error("startup-runs must be at least 2 and memory-samples at least 1")
    if arguments.platform == "macos" and not arguments.macos_probe:
        parser.error("--macos-probe is required on macOS")

    configuration = json.loads(arguments.applications.read_text(encoding="utf-8"))
    applications = configuration["applications"]
    cells = {(item["framework"], item["profile"]) for item in applications}
    expected = {(framework, profile) for framework in FRAMEWORKS for profile in PROFILES}
    if cells != expected or len(applications) != len(expected):
        raise SystemExit("the applications manifest must contain exactly 20 unique matrix cells")
    for application in applications:
        executable = pathlib.Path(application["executable"]).resolve()
        if not executable.is_file():
            raise SystemExit(f"matrix executable is missing: {executable}")
        application["executable"] = str(executable)
        application["working_directory"] = str(
            pathlib.Path(application.get("working_directory", ".")).resolve()
        )

    arguments.output.mkdir(parents=True, exist_ok=True)
    implementations = {framework: {} for framework in FRAMEWORKS}
    for application in sorted(
        applications,
        key=lambda item: (
            list(PROFILES).index(item["profile"]),
            list(FRAMEWORKS).index(item["framework"]),
        ),
    ):
        implementations[application["framework"]][application["profile"]] = measure_application(
            application,
            arguments.platform,
            arguments.macos_probe.resolve() if arguments.macos_probe else None,
            arguments.output,
            arguments.startup_runs,
            arguments.memory_samples,
            arguments.warmup_seconds,
            arguments.cpu_seconds,
        )

    report = {
        "schema": "zsui.ui-performance-matrix/v1",
        "measured_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "machine": {
            "platform": arguments.platform,
            "runner": arguments.runner,
            "architecture": os.uname().machine,
            "logical_processors": os.cpu_count(),
            "startup_runs": arguments.startup_runs,
            "memory_samples": arguments.memory_samples,
            "warmup_seconds": arguments.warmup_seconds,
            "cpu_sample_seconds": arguments.cpu_seconds,
        },
        "profiles": PROFILES,
        "implementations": implementations,
        "methodology": {
            "fairness": "compare frameworks only within the same fixed profile",
            "viewer_boundary": "formal ZSUI applications and zsui-viewer are separate release binaries",
            "cold_start": "best-effort first launch after release build; filesystem caches are not purged",
            "first_frame": "elapsed process start to a visible nonzero native window and successful final-window capture",
            "memory": "recursive process-tree samples; Linux reports RSS/PSS/private RSS from smaps_rollup, macOS reports RSS and VM physical footprint while unsupported PSS/private RSS remain null",
            "hidden": "the same full-page process after native unmap or NSRunningApplication.hide",
            "idle_cpu": "recursive process-tree CPU delta while the page is stationary",
            "repaint_cpu": "comparison applications use their 60 Hz benchmark render loop; ZSUI uses an external target-native invalidation driver",
            "webview": "Tauri memory includes child processes; binary size excludes the system WebView runtime",
        },
    }
    (arguments.output / "report.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    write_markdown(report, arguments.output / "report.md")
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as summary:
            summary.write((arguments.output / "report.md").read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
