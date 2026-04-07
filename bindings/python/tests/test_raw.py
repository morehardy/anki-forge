import pathlib
import tempfile
import unittest

from anki_forge_python import (
    WRAPPER_API_VERSION,
    RuntimeInvocationError,
    resolve_runtime,
    run_raw,
)


REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
VALID_AUTHORING_INPUT = REPO_ROOT / "contracts/fixtures/valid/minimal-authoring-ir.json"


class RawBindingsTests(unittest.TestCase):
    def test_resolve_runtime_discovers_workspace_metadata(self) -> None:
        runtime = resolve_runtime(cwd=pathlib.Path(__file__).resolve().parents[1])

        self.assertEqual(runtime.mode, "workspace")
        self.assertTrue(str(runtime.manifest_path).endswith("contracts/manifest.yaml"))
        self.assertTrue(str(runtime.bundle_root).endswith("contracts"))
        self.assertEqual(runtime.bundle_version, "0.1.0")
        self.assertIsInstance(WRAPPER_API_VERSION, str)

    def test_run_raw_normalize_preserves_process_result(self) -> None:
        result = run_raw(
            "normalize",
            {"input_path": str(VALID_AUTHORING_INPUT)},
            cwd=pathlib.Path(__file__).resolve().parents[1],
        )

        self.assertEqual(result.command, "normalize")
        self.assertEqual(result.exit_status, 0)
        self.assertIsInstance(result.stdout, str)
        self.assertIsInstance(result.stderr, str)
        self.assertGreaterEqual(len(result.argv), 1)

    def test_run_raw_raises_runtime_invocation_error_for_missing_launcher(self) -> None:
        with self.assertRaises(RuntimeInvocationError) as context:
            run_raw(
                "normalize",
                {"input_path": str(VALID_AUTHORING_INPUT)},
                cwd=pathlib.Path(__file__).resolve().parents[1],
                launcher_executable="/definitely-missing-anki-forge-python-binary",
            )

        self.assertEqual(context.exception.command, "normalize")
        self.assertEqual(context.exception.resolved_runtime.mode, "workspace")

    def test_run_raw_wraps_runtime_discovery_failure(self) -> None:
        detached_dir = pathlib.Path(tempfile.mkdtemp(prefix="anki-forge-python-detached-"))

        with self.assertRaises(RuntimeInvocationError) as context:
            run_raw("normalize", {"input_path": str(VALID_AUTHORING_INPUT)}, cwd=detached_dir)

        self.assertEqual(context.exception.command, "normalize")
        self.assertEqual(context.exception.failure_phase, "runtime-resolution")
        self.assertIsNone(context.exception.resolved_runtime)


if __name__ == "__main__":
    unittest.main()
