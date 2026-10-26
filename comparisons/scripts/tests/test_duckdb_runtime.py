import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

SPEC = importlib.util.spec_from_file_location("duckdb_runtime", Path(__file__).resolve().parents[1] / "prepare_duckdb_runtime.py")
RUNTIME = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNTIME)
TARGET = "x86_64-pc-windows-msvc"
LOCK = '[[package]]\nname = "libduckdb-sys"\nversion = "1.10505.0"\n'


class DuckDBRuntimeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.target = Path(self.temporary.name) / "target"
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            for name in ("duckdb.h", "duckdb.lib", "duckdb.dll"):
                archive.writestr(name, ("fixture:" + name).encode())
        self.payload = buffer.getvalue()
        self.manifest = {"schema": "zsui.duckdb-runtime/v1", "crate_version": "1.10505.0",
                         "library_version": "1.5.5", "targets": {TARGET: {
                             "archive": "libduckdb-windows-amd64.zip",
                             "sha256": hashlib.sha256(self.payload).hexdigest(),
                             "files": ["duckdb.h", "duckdb.lib", "duckdb.dll"], "runtime": "duckdb.dll"}}}
        self.downloads = 0

    def open_archive(self, request, timeout):
        self.assertTrue(request.full_url.startswith("https://github.com/duckdb/duckdb/releases/download/v1.5.5/"))
        self.downloads += 1
        return io.BytesIO(self.payload)

    def prepare(self):
        return RUNTIME.prepare_runtime(self.manifest, LOCK, TARGET, self.target, self.open_archive)

    def test_restored_rust_build_without_native_files_is_repaired(self):
        cached_build = self.target / "debug/build/libduckdb-sys-cached/output"
        cached_build.parent.mkdir(parents=True)
        cached_build.write_text("cached linker directives")
        report = self.prepare()
        library = Path(report["library_directory"])
        self.assertTrue(report["downloaded"])
        self.assertEqual((library / "duckdb.lib").read_bytes(), b"fixture:duckdb.lib")
        for profile in ("debug", "release"):
            self.assertEqual((self.target / profile / "deps/duckdb.dll").read_bytes(), b"fixture:duckdb.dll")
        self.assertEqual(cached_build.read_text(), "cached linker directives")

    def test_missing_import_library_is_repaired_when_dll_exists(self):
        first = self.prepare()
        missing = Path(first["library_directory"]) / "duckdb.lib"
        missing.unlink()
        report = self.prepare()
        self.assertFalse(report["downloaded"])
        self.assertEqual(report["repaired_files"], [str(missing)])
        self.assertEqual(self.downloads, 1)

    def test_missing_runtime_copy_is_restored_from_verified_cache(self):
        self.prepare()
        missing = self.target / "release/deps/duckdb.dll"
        missing.unlink()
        report = self.prepare()
        self.assertEqual(report["repaired_files"], [str(missing)])
        self.assertEqual(self.downloads, 1)

    def test_complete_warm_cache_does_no_network_or_rewrites(self):
        self.prepare()
        report = self.prepare()
        self.assertFalse(report["downloaded"])
        self.assertEqual(report["repaired_files"], [])
        self.assertEqual(self.downloads, 1)

    def test_corrupt_library_is_repaired_from_archive(self):
        first = self.prepare()
        corrupt = Path(first["library_directory"]) / "duckdb.lib"
        corrupt.write_bytes(b"invalid")
        self.assertEqual(self.prepare()["repaired_files"], [str(corrupt)])
        self.assertEqual(corrupt.read_bytes(), b"fixture:duckdb.lib")

    def test_bad_download_hash_rejects_before_extracting(self):
        self.manifest["targets"][TARGET]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            self.prepare()
        self.assertFalse(any(self.target.rglob("duckdb.lib")))
        self.assertFalse(any(self.target.rglob(".duckdb-download-*")))

    def test_lock_version_mismatch_fails_before_download(self):
        with self.assertRaisesRegex(ValueError, "Cargo.lock"):
            RUNTIME.prepare_runtime(self.manifest, LOCK.replace("1.10505.0", "1.10504.0"), TARGET, self.target, self.open_archive)
        self.assertEqual(self.downloads, 0)

    def test_checked_in_manifest_matches_lock_and_supported_targets(self):
        manifest = json.loads((RUNTIME.PROJECT / "duckdb-runtime.json").read_text())
        lock = (RUNTIME.PROJECT / "Cargo.lock").read_text()
        for target in (TARGET, "aarch64-apple-darwin", "x86_64-unknown-linux-gnu"):
            self.assertIn("runtime", RUNTIME.validate_manifest(manifest, lock, target))


if __name__ == "__main__":
    unittest.main()
