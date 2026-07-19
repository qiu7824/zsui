#!/usr/bin/env python3
"""Capture process-owned Linux smaps totals and the largest resident mappings."""

from __future__ import annotations

import argparse
import json
import os
import re
from collections import defaultdict
from pathlib import Path


HEADER = re.compile(
    r"^[0-9a-f]+-[0-9a-f]+\s+\S+\s+\S+\s+\S+\s+\S+\s*(?P<path>.*)$"
)
METRIC = re.compile(r"^(?P<name>[A-Za-z_]+):\s+(?P<value>\d+)\s+kB$")
TRACKED = (
    "Size",
    "Rss",
    "Pss",
    "Shared_Clean",
    "Shared_Dirty",
    "Private_Clean",
    "Private_Dirty",
    "Anonymous",
    "AnonHugePages",
    "Swap",
)


def mapping_category(path: str, executable: str) -> str:
    if path == executable:
        return "application"
    if path == "[heap]":
        return "heap"
    if path.startswith("[stack"):
        return "stack"
    if path in {"", "[anonymous]"} or path.startswith("[anon:"):
        return "anonymous"
    if path.startswith(("/dev/shm/", "/memfd:", "/SYSV")):
        return "shared-memory"
    if path.startswith(("/usr/share/fonts/", "/var/cache/fontconfig/")):
        return "font-data"
    if "/lib" in path and (".so" in path or "/ld-linux" in path):
        return "system-library"
    if path.startswith("["):
        return "kernel"
    return "other-file"


def parse_smaps(text: str, executable: str) -> dict[str, object]:
    mappings: list[dict[str, object]] = []
    current: dict[str, object] | None = None
    for line in text.splitlines():
        header = HEADER.match(line)
        if header:
            path = header.group("path").strip() or "[anonymous]"
            current = {"path": path, **{name: 0 for name in TRACKED}}
            mappings.append(current)
            continue
        metric = METRIC.match(line)
        if current is not None and metric and metric.group("name") in TRACKED:
            current[metric.group("name")] = int(metric.group("value")) * 1024

    totals = {name: sum(int(item[name]) for item in mappings) for name in TRACKED}
    by_path: dict[str, dict[str, int]] = defaultdict(
        lambda: {name: 0 for name in TRACKED}
    )
    by_category: dict[str, dict[str, int]] = defaultdict(
        lambda: {name: 0 for name in TRACKED}
    )
    for mapping in mappings:
        path = str(mapping["path"])
        category = mapping_category(path, executable)
        for name in TRACKED:
            value = int(mapping[name])
            by_path[path][name] += value
            by_category[category][name] += value

    def rows(values: dict[str, dict[str, int]], label: str) -> list[dict[str, object]]:
        return sorted(
            ({label: key, **metrics} for key, metrics in values.items()),
            key=lambda item: (int(item["Rss"]), int(item["Pss"])),
            reverse=True,
        )

    return {
        "mapping_count": len(mappings),
        "totals": totals,
        "categories": rows(by_category, "category"),
        "largest_mappings": rows(by_path, "path")[:40],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pid", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    proc = Path("/proc") / str(args.pid)
    executable = os.readlink(proc / "exe")
    report = {
        "schema": "zsui.linux-smaps/v1",
        "pid": args.pid,
        "executable": executable,
        **parse_smaps((proc / "smaps").read_text(encoding="utf-8"), executable),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
