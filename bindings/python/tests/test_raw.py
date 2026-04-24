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


def bundled_contract_version() -> str:
    for line in (REPO_ROOT / "contracts/manifest.yaml").read_text(
        encoding="utf-8"
    ).splitlines():
        stripped = line.strip()
        if stripped.startswith("bundle_version:"):
            return stripped.split(":", 1)[1].strip().strip("\"'")
    raise AssertionError("bundled manifest must declare bundle_version")


class RawBindingsTests(unittest.TestCase):
    def test_resolve_runtime_discovers_workspace_metadata(self) -> None:
        runtime = resolve_runtime(cwd=pathlib.Path(__file__).resolve().parents[1])

        self.assertEqual(runtime.mode, "workspace")
        self.assertTrue(str(runtime.manifest_path).endswith("contracts/manifest.yaml"))
        self.assertTrue(str(runtime.bundle_root).endswith("contracts"))
        self.assertEqual(runtime.bundle_version, bundled_contract_version())
        self.assertIsInstance(WRAPPER_API_VERSION, str)

    def test_resolve_runtime_installed_mode_tolerates_indented_single_quoted_bundle_version(
        self,
    ) -> None:
        temp_root = pathlib.Path(
            tempfile.mkdtemp(prefix="anki-forge-python-manifest-")
        ).resolve()
        manifest_path = temp_root / "manifest.yaml"
        manifest_path.write_text("  bundle_version: '9.9.9'\n", encoding="utf-8")

        runtime = resolve_runtime(
            mode="installed",
            manifest_path=str(manifest_path),
            bundle_root=str(temp_root),
        )

        self.assertEqual(runtime.bundle_version, "9.9.9")

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
