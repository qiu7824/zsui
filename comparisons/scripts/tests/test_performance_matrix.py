import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch


SPEC = importlib.util.spec_from_file_location(
    "performance_matrix", Path(__file__).resolve().parents[1] / "measure_unix_performance_matrix.py")
MATRIX = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MATRIX
SPEC.loader.exec_module(MATRIX)


def response(stdout="", code=0):
    return subprocess.CompletedProcess([], code, stdout, "probe error" if code else "")


class PerformanceLifecycleTests(unittest.TestCase):
    def test_launch_uses_application_identity_and_absolute_document_argument(self):
        with tempfile.TemporaryDirectory() as root:
            directory = Path(root)
            document = directory / "document.json"
            document.write_text("{}")
            bundle = directory / "app.app"
            app = {"macos_bundle": str(bundle), "working_directory": root}
            with patch.object(MATRIX, "run", return_value=response(json.dumps({
                    "pid": 41, "launch_token": "123.5", "bundle": str(bundle)}))) as run:
                process = MATRIX.launch_macos_application(app, Path("probe"),
                    ["--document", "document.json", "--width", "1000"])
            self.assertEqual(process.pid, 41)
            self.assertEqual(process.launch_token, "123.5")
            self.assertEqual(run.call_args[0][0],
                ["probe", "launch", str(bundle), "--document", str(document.resolve()), "--width", "1000"])

    def test_wrong_bundle_is_rejected_and_owned_application_is_closed(self):
        app = {"macos_bundle": str(Path("expected.app").resolve()), "working_directory": str(Path.cwd())}
        with patch.object(MATRIX, "run", return_value=response(json.dumps({
                "pid": 41, "launch_token": "123.5", "bundle": str(Path("wrong.app").resolve())}))), \
                patch.object(MATRIX.MacosApplicationProcess, "terminate") as terminate:
            with self.assertRaisesRegex(RuntimeError, "different application"):
                MATRIX.launch_macos_application(app, Path("probe"), [])
            terminate.assert_called_once()

    def test_reused_pid_is_not_terminated(self):
        process = MATRIX.MacosApplicationProcess(41, "original-launch", Path("probe"))
        with patch.object(MATRIX, "run", return_value=response('{"alive":false}')) as run, \
                patch.object(MATRIX.os, "kill") as kill:
            MATRIX.stop_process(process, "macos")
            self.assertEqual(run.call_args[0][0], ["probe", "state", "41", "original-launch"])
            self.assertEqual(run.call_count, 1)
            kill.assert_not_called()

    def test_reused_descendant_is_not_killed(self):
        process = MATRIX.MacosApplicationProcess(41, "original-launch", Path("probe"))
        with patch.object(process, "poll", return_value=None), \
                patch.object(process, "terminate"), patch.object(process, "wait"), \
                patch.object(MATRIX, "process_tree", return_value=[41, 42]), \
                patch.object(MATRIX, "run", side_effect=[response("old birth"), response("new birth")]), \
                patch.object(MATRIX.os, "kill") as kill:
            MATRIX.stop_process(process, "macos")
            kill.assert_not_called()

    def test_successful_hide_request_requires_no_visible_windows(self):
        process = Mock(pid=41)
        process.poll.return_value = None
        launch = MATRIX.Launch(process, {"window_id": 4}, 0, 0)
        with patch.object(MATRIX, "run", return_value=response()), \
                patch.object(MATRIX, "process_tree", return_value=[41]), \
                patch.object(MATRIX, "linux_windows", return_value=[{"window_id": 4}]), \
                patch.object(MATRIX.time, "monotonic", side_effect=[0, 1, 6]), \
                patch.object(MATRIX.time, "sleep"):
            with self.assertRaisesRegex(RuntimeError, "remains visible"):
                MATRIX.hide_application(launch, "linux", None)

    def test_process_exit_cannot_count_as_hidden_memory(self):
        process = Mock(pid=41)
        process.poll.side_effect = [None, 0]
        launch = MATRIX.Launch(process, {"window_id": 4}, 0, 0)
        with patch.object(MATRIX, "run", return_value=response()):
            with self.assertRaisesRegex(RuntimeError, "exited during"):
                MATRIX.hide_application(launch, "linux", None)

    def test_verified_hidden_window_is_accepted(self):
        process = Mock(pid=41)
        process.poll.return_value = None
        launch = MATRIX.Launch(process, {"window_id": 4}, 0, 0)
        with patch.object(MATRIX, "run", return_value=response()), \
                patch.object(MATRIX, "process_tree", return_value=[41]), \
                patch.object(MATRIX, "macos_windows", return_value=[]):
            MATRIX.hide_application(launch, "macos", Path("probe"))


if __name__ == "__main__":
    unittest.main()
