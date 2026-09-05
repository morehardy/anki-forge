import json
import os
import pathlib
import sys
import tempfile
import textwrap
import unittest

from anki_forge_python import (
    ProtocolParseError,
    RuntimeInvocationError,
    build,
    diff,
    inspect,
    normalize,
)


REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
VALID_AUTHORING_INPUT = REPO_ROOT / "contracts/fixtures/valid/minimal-authoring-ir.json"
VALID_NORMALIZED_INPUT = REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"


def fake_launcher_script(source: str) -> str:
    fake_dir = pathlib.Path(tempfile.mkdtemp(prefix="anki-forge-python-fake-"))
    fake_script = fake_dir / "fake_launcher.py"
    fake_script.write_text(textwrap.dedent(source), encoding="utf-8")
    return str(fake_script)


def installed_runtime_options(script_path: str) -> dict:
    return {
        "mode": "installed",
        "manifest_path": str(REPO_ROOT / "contracts/manifest.yaml"),
        "bundle_root": str(REPO_ROOT / "contracts"),
        "launcher_executable": sys.executable,
        "launcher_prefix": [script_path],
    }


class StructuredBindingsTests(unittest.TestCase):
    def test_structured_normalize_returns_invalid_result_without_throwing(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({
                "kind": "normalization-result",
                "result_status": "invalid",
                "tool_contract_version": "phase2-v1",
                "policy_refs": {"identity_policy": "identity-policy.default@1.0.0"},
                "comparison_context": {"kind": "comparison-context", "identity_mode": "document-id"},
                "diagnostics": {"status": "invalid", "items": []},
            }))
            """
        )

        result = normalize(
            {"input_path": str(VALID_AUTHORING_INPUT)},
            **installed_runtime_options(script_path),
        )

        self.assertEqual(result["kind"], "normalization-result")
        self.assertEqual(result["result_status"], "invalid")
        self.assertTrue(result["helper"]["is_invalid"])
        self.assertGreaterEqual(result["helper"]["warning_count"], 0)

    def test_structured_build_derives_artifact_paths(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({
                "kind": "package-build-result",
                "result_status": "success",
                "tool_contract_version": "phase3-v1",
                "writer_policy_ref": "writer-policy.default@1.0.0",
                "build_context_ref": "build-context.default@1.0.0",
                "staging_ref": "artifacts/alt/staging/manifest.json",
                "artifact_fingerprint": "artifact:demo",
                "apkg_ref": "artifacts/alt/package.apkg",
                "diagnostics": {"kind": "build-diagnostics", "items": []},
            }))
            """
        )
        artifacts_dir = tempfile.mkdtemp(prefix="anki-forge-python-build-")

        result = build(
            {
                "input_path": str(VALID_NORMALIZED_INPUT),
                "artifacts_dir": artifacts_dir,
            },
            **installed_runtime_options(script_path),
        )

        self.assertEqual(result["kind"], "package-build-result")
        self.assertEqual(result["result_status"], "success")
        self.assertIsInstance(result["resolved_runtime"].bundle_version, str)
        self.assertTrue(
            result["helper"]["artifact_paths"]["staging_manifest"].endswith(
                os.path.join("alt", "staging", "manifest.json")
            )
        )
        self.assertTrue(
            result["helper"]["artifact_paths"]["apkg"].endswith(
                os.path.join("alt", "package.apkg")
            )
        )

    def test_structured_normalize_raises_protocol_parse_error_for_invalid_json(self) -> None:
        script_path = fake_launcher_script("print('{broken')")

        with self.assertRaises(ProtocolParseError) as context:
            normalize(
                {"input_path": str(VALID_AUTHORING_INPUT)},
                **installed_runtime_options(script_path),
            )

        self.assertEqual(context.exception.parse_phase, "json")

    def test_structured_normalize_raises_protocol_parse_error_for_contract_shape(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({"kind": "normalization-result"}))
            """
        )

        with self.assertRaises(ProtocolParseError) as context:
            normalize(
                {"input_path": str(VALID_AUTHORING_INPUT)},
                **installed_runtime_options(script_path),
            )

        self.assertEqual(context.exception.parse_phase, "contract-shape")

    def test_structured_build_raises_protocol_parse_error_for_contract_version(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({
                "kind": "package-build-result",
                "result_status": "success",
                "tool_contract_version": "phase3-v999",
                "writer_policy_ref": "writer-policy.default@1.0.0",
                "build_context_ref": "build-context.default@1.0.0",
                "staging_ref": "artifacts/staging/manifest.json",
                "artifact_fingerprint": "artifact:demo",
                "diagnostics": {"kind": "build-diagnostics", "items": []},
            }))
            """
        )

        with self.assertRaises(ProtocolParseError) as context:
            build(
                {
                    "input_path": str(VALID_NORMALIZED_INPUT),
                    "artifacts_dir": tempfile.mkdtemp(prefix="anki-forge-python-version-"),
                },
                **installed_runtime_options(script_path),
            )

        self.assertEqual(context.exception.parse_phase, "contract-version")

    def test_structured_inspect_returns_degraded_result_without_throwing(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({
                "kind": "inspect-report",
                "observation_model_version": "phase3-inspect-v1",
                "source_kind": "apkg",
                "source_ref": "artifacts/package-no-media.apkg",
                "artifact_fingerprint": "artifact:demo",
                "observation_status": "degraded",
                "missing_domains": ["media"],
                "degradation_reasons": ["media map unavailable"],
                "observations": {
                    "notetypes": [],
                    "templates": [],
                    "fields": [],
                    "media": [],
                    "metadata": [],
                    "references": [],
                },
            }))
            """
        )

        result = inspect(
            {"apkg_path": str(REPO_ROOT / "tmp/fake.apkg")},
            **installed_runtime_options(script_path),
        )

        self.assertEqual(result["observation_status"], "degraded")
        self.assertTrue(result["helper"]["is_degraded"])

    def test_structured_diff_returns_partial_result_without_throwing(self) -> None:
        script_path = fake_launcher_script(
            """
            import json

            print(json.dumps({
                "kind": "diff-report",
                "comparison_status": "partial",
                "left_fingerprint": "artifact:left",
                "right_fingerprint": "artifact:right",
                "left_observation_model_version": "phase3-inspect-v1",
                "right_observation_model_version": "phase3-inspect-v1",
                "summary": "reference coverage reduced",
                "uncompared_domains": ["references"],
                "comparison_limitations": ["right report is degraded"],
                "changes": [],
            }))
            """
        )

        result = diff(
            {
                "left_path": str(REPO_ROOT / "tmp/left.inspect.json"),
                "right_path": str(REPO_ROOT / "tmp/right.inspect.json"),
            },
            **installed_runtime_options(script_path),
        )

        self.assertEqual(result["comparison_status"], "partial")
        self.assertTrue(result["helper"]["is_partial"])

    def test_structured_normalize_raises_runtime_invocation_error_for_nonzero_exit(self) -> None:
        script_path = fake_launcher_script(
            """
            import sys

            sys.stderr.write("normalize failed\\n")
            raise SystemExit(3)
            """
        )

        with self.assertRaises(RuntimeInvocationError) as context:
            normalize(
                {"input_path": str(VALID_AUTHORING_INPUT)},
                **installed_runtime_options(script_path),
            )

        self.assertEqual(context.exception.failure_phase, "process-exit")
        self.assertEqual(context.exception.exit_status, 3)
        self.assertIn("normalize failed", context.exception.stderr)

    def test_observation_versions_support_legacy_current_and_mixed_reports(self) -> None:
        fixtures = REPO_ROOT / "contracts/fixtures/phase3/expected"

        def runtime_for(payload: dict) -> dict:
            return installed_runtime_options(
                fake_launcher_script(f"print({json.dumps(payload)!r})")
            )

        for version in ("phase3-inspect-v1", "phase3-inspect-v2"):
            with self.subTest(version=version):
                payload = json.loads((fixtures / "basic.inspect.json").read_text())
                payload["observation_model_version"] = version
                result = inspect({"apkg_path": "unused.apkg"}, **runtime_for(payload))
                self.assertEqual(result["observation_model_version"], version)

        for left, right in (
            ("phase3-inspect-v1", "phase3-inspect-v2"),
            ("phase3-inspect-v2", "phase3-inspect-v1"),
            ("phase3-inspect-v2", "phase3-inspect-v2"),
        ):
            with self.subTest(left=left, right=right):
                payload = json.loads((fixtures / "basic.diff.json").read_text())
                payload["left_observation_model_version"] = left
                payload["right_observation_model_version"] = right
                payload["comparison_status"] = "complete" if left == right else "partial"
                payload["comparison_limitations"] = (
                    [] if left == right else ["observation model versions differ"]
                )
                result = diff(
                    {"left_path": "left.json", "right_path": "right.json"},
                    **runtime_for(payload),
                )
                self.assertEqual(result["comparison_status"], payload["comparison_status"])

        unknown = json.loads((fixtures / "basic.inspect.json").read_text())
        unknown["observation_model_version"] = "phase3-inspect-v999"
        with self.assertRaises(ProtocolParseError) as context:
            inspect({"apkg_path": "unused.apkg"}, **runtime_for(unknown))
        self.assertEqual(context.exception.parse_phase, "contract-version")


if __name__ == "__main__":
    unittest.main()
