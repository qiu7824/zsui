#!/usr/bin/env python3
"""Restore the pinned DuckDB linker and runtime files independently of Cargo's cache."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile
import urllib.request
import zipfile

CHUNK_SIZE = 1024 * 1024
PROJECT = Path(__file__).resolve().parents[1] / "zsui_dataviewer"


def stream_digest(stream):
    digest = hashlib.sha256()
    for chunk in iter(lambda: stream.read(CHUNK_SIZE), b""):
        digest.update(chunk)
    return digest.hexdigest()


def file_digest(path):
    with path.open("rb") as source:
        return stream_digest(source)


def atomic_write(destination, source):
    if destination.is_symlink():
        raise ValueError("Refusing a symbolic-link destination: " + str(destination))
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=str(destination.parent), prefix=".duckdb-", delete=False) as output:
        temporary = Path(output.name)
        try:
            shutil.copyfileobj(source, output, CHUNK_SIZE)
        except BaseException:
            output.close()
            temporary.unlink()
            raise
    try:
        os.replace(str(temporary), str(destination))
    finally:
        if temporary.exists():
            temporary.unlink()


def validate_manifest(manifest, lock_text, target):
    if manifest.get("schema") != "zsui.duckdb-runtime/v1":
        raise ValueError("Unsupported DuckDB runtime manifest schema")
    versions = []
    for package in lock_text.split("[[package]]")[1:]:
        if re.search(r'^name\s*=\s*"libduckdb-sys"\s*$', package, re.MULTILINE):
            match = re.search(r'^version\s*=\s*"([^"]+)"\s*$', package, re.MULTILINE)
            if match:
                versions.append(match.group(1))
    if versions != [manifest["crate_version"]]:
        raise ValueError("Cargo.lock libduckdb-sys version does not match duckdb-runtime.json")
    encoded = int(manifest["crate_version"].split(".")[1])
    native_version = "{}.{}.{}".format(encoded // 10000, encoded // 100 % 100, encoded % 100)
    if native_version != manifest["library_version"]:
        raise ValueError("DuckDB library version does not match the crate ABI")
    if target not in manifest["targets"]:
        raise ValueError("No pinned DuckDB runtime for " + target)
    entry = manifest["targets"][target]
    for name in [entry["archive"]] + entry["files"]:
        if not name or name in (".", "..") or "/" in name or "\\" in name:
            raise ValueError("DuckDB manifest filenames must be plain basenames")
    if not re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]) or entry["runtime"] not in entry["files"]:
        raise ValueError("Invalid DuckDB checksum or runtime entry")
    return entry


def prepare_runtime(manifest, lock_text, target, target_directory, opener=urllib.request.urlopen):
    entry = validate_manifest(manifest, lock_text, target)
    target_directory = Path(target_directory).resolve()
    library_directory = target_directory / "duckdb-download" / target / manifest["library_version"]
    library_directory.mkdir(parents=True, exist_ok=True)
    archive_path = library_directory / entry["archive"]
    downloaded = False
    if not archive_path.is_file() or file_digest(archive_path) != entry["sha256"]:
        url = "https://github.com/duckdb/duckdb/releases/download/v{}/{}".format(
            manifest["library_version"], entry["archive"])
        request = urllib.request.Request(url, headers={"User-Agent": "zsui-dataviewer-ci"})
        with opener(request, timeout=120) as response:
            with tempfile.NamedTemporaryFile(dir=str(library_directory), prefix=".duckdb-download-", delete=False) as output:
                download_path = Path(output.name)
                try:
                    shutil.copyfileobj(response, output, CHUNK_SIZE)
                except BaseException:
                    output.close()
                    download_path.unlink()
                    raise
        try:
            if file_digest(download_path) != entry["sha256"]:
                raise ValueError("DuckDB release archive checksum mismatch")
            os.replace(str(download_path), str(archive_path))
        finally:
            if download_path.exists():
                download_path.unlink()
        downloaded = True

    repaired = []
    with zipfile.ZipFile(str(archive_path)) as archive:
        for name in entry["files"]:
            info = archive.getinfo(name)
            if info.is_dir() or stat.S_ISLNK(info.external_attr >> 16) or info.file_size == 0:
                raise ValueError("DuckDB archive contains an invalid library member: " + name)
            with archive.open(info) as source:
                expected = stream_digest(source)
            destination = library_directory / name
            if destination.is_symlink():
                raise ValueError("Refusing a symbolic-link library file: " + str(destination))
            if not destination.is_file() or file_digest(destination) != expected:
                with archive.open(info) as source:
                    atomic_write(destination, source)
                repaired.append(str(destination))

    runtime = library_directory / entry["runtime"]
    runtime_hash = file_digest(runtime)
    for profile in ("debug", "release"):
        destination = target_directory / profile / "deps" / entry["runtime"]
        if not destination.is_file() or file_digest(destination) != runtime_hash:
            with runtime.open("rb") as source:
                atomic_write(destination, source)
            repaired.append(str(destination))
    return {"target": target, "version": manifest["library_version"],
            "archive_sha256": entry["sha256"], "downloaded": downloaded,
            "library_directory": str(library_directory), "repaired_files": repaired}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=PROJECT / "duckdb-runtime.json")
    parser.add_argument("--lock-file", type=Path, default=PROJECT / "Cargo.lock")
    parser.add_argument("--target")
    parser.add_argument("--target-dir", type=Path)
    args = parser.parse_args()
    target = args.target
    if not target:
        version = subprocess.check_output(["rustc", "-vV"], encoding="utf-8")
        target = next(line.split(": ", 1)[1] for line in version.splitlines() if line.startswith("host: "))
    target_dir = args.target_dir or Path(os.environ.get("CARGO_TARGET_DIR", str(PROJECT / "target")))
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    report = prepare_runtime(manifest, args.lock_file.read_text(encoding="utf-8"), target, target_dir)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
