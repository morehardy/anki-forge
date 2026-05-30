from copy import deepcopy
import json
import os
import subprocess
from pathlib import Path

import pytest

from anki_forge import DiagnosticsError, Note, Project, ProtocolError, RuntimeInvocationError, ValidationError
import anki_forge.runtime as runtime_module
from anki_forge.report import BuildReport
from anki_forge.runtime import RuntimeOverride, build_product_build_argv, parse_completed_process

try:
    from anki_forge_python.raw import _build_args
except ImportError as exc:
    raise AssertionError("low-level _build_args drifted; update this parity test with the wrapper change") from exc
from anki_forge_python.runtime import ResolvedRuntime as LowLevelRuntime


def build_report_payload(**overrides):
    payload = {
        "kind": "anki-forge-build-report",
        "schema_version": "phase4-build-report-v2",
        "tool_version": "test",
        "status": "success",
        "comparison": "not_requested",
        "artifact": {"path": "deck.apkg"},
        "counts": {"notes": 1, "cards": 1, "media": 0},
        "media": {
            "objects": 0,
            "bindings": 0,
            "references": 0,
            "missing_references": 0,
            "unsafe_references": 0,
            "unused_bindings": 0,
            "unique_bytes": 0,
        },
        "diagnostics": [],
        "metrics": {"duration_ms": 1},
        "policy": {
            "status": "not_evaluated",
            "threshold": None,
            "highest_risk": None,
            "blocking_findings": [],
        },
    }
    payload.update(deepcopy(overrides))
    return payload


def test_report_success_warning_does_not_raise():
    report = BuildReport.from_json(build_report_payload(
        diagnostics=[{"code": "W", "severity": "warning", "message": "warn"}],
    ))
    report.ensure_success()


def test_report_ensure_success_raises_for_invalid_report():
    payload = build_report_payload(
        status="invalid",
        artifact=None,
        counts={"notes": 0, "cards": 0, "media": 0},
        diagnostics=[{"code": "E", "severity": "error", "message": "bad"}],
    )
    with pytest.raises(DiagnosticsError):
        BuildReport.from_json(payload).ensure_success()


def test_report_rejects_unknown_comparison():
    payload = build_report_payload(comparison="garbage")
    with pytest.raises(ProtocolError):
        BuildReport.from_json(payload)


def test_report_rejects_wrong_kind_and_schema_version():
    base = build_report_payload()
    with pytest.raises(ProtocolError):
        BuildReport.from_json({**base, "kind": "wrong"})
    with pytest.raises(ProtocolError):
        BuildReport.from_json({**base, "schema_version": "future"})


def test_report_parses_schema_media_summary_unique_bytes():
    report = BuildReport.from_json(build_report_payload(
        media={
            "objects": 2,
            "bindings": 3,
            "references": 4,
            "missing_references": 0,
            "unsafe_references": 1,
            "unused_bindings": 5,
            "unique_bytes": 987,
        },
    ))

    assert report.media["unique_bytes"] == 987


def test_report_rejects_legacy_media_bytes_payload():
    payload = build_report_payload(media={"objects": 0, "bindings": 0, "bytes": 0})

    with pytest.raises(ProtocolError):
        BuildReport.from_json(payload)


def test_report_rejects_unknown_status():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(status="warning"))


def test_report_rejects_missing_or_invalid_artifact_path():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(artifact={}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(artifact={"path": ""}))

    report = BuildReport.from_json(build_report_payload(artifact={"path": "deck.apkg", "extra": True}))
    assert report.artifact == {"path": "deck.apkg", "extra": True}


def test_report_rejects_bool_integer_field():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(counts={"notes": True, "cards": 1, "media": 0}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(metrics={"duration_ms": True}))


def test_report_rejects_invalid_policy_shape_and_status():
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(policy={"status": "not_evaluated"}))
    with pytest.raises(ProtocolError):
        BuildReport.from_json(build_report_payload(policy={
            "status": "not_applicable",
            "threshold": None,
            "highest_risk": None,
            "blocking_findings": [],
        }))

    BuildReport.from_json(build_report_payload(policy={
        "status": "not_evaluated",
        "threshold": None,
        "highest_risk": None,
        "blocking_findings": [],
        "extra": "ok",
    }))


def test_report_accepts_forward_compatible_summary_fields():
    report = BuildReport.from_json(build_report_payload(
        counts={"notes": 1, "cards": 1, "media": 0, "future": 99},
        media={
            "objects": 0,
            "bindings": 0,
            "references": 0,
            "missing_references": 0,
            "unsafe_references": 0,
            "unused_bindings": 0,
            "unique_bytes": 0,
            "future": 99,
        },
        metrics={"duration_ms": 1, "future": True},
    ))

    assert report.counts == {"notes": 1, "cards": 1, "media": 0}
    assert report.media == {
        "objects": 0,
        "bindings": 0,
        "references": 0,
        "missing_references": 0,
        "unsafe_references": 0,
        "unused_bindings": 0,
        "unique_bytes": 0,
    }


def test_exit_zero_non_json_is_protocol_error(monkeypatch):
    def fake_run(*args, **kwargs):
        return subprocess.CompletedProcess(["contract_tools"], 0, stdout="not json", stderr="")
    monkeypatch.setattr(subprocess, "run", fake_run)
    with pytest.raises(ProtocolError):
        Project("Deck").write_apkg("out.apkg", runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")))


def test_fail_on_rejects_unknown_level_before_subprocess(monkeypatch):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")
    monkeypatch.setattr(subprocess, "run", fail_run)
    with pytest.raises(ValidationError):
        Project("Deck").write_apkg("out.apkg", compare_to="old.apkg", fail_on="severe", runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")))


def test_fail_on_requires_compare_to_before_subprocess(monkeypatch):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")
    monkeypatch.setattr(subprocess, "run", fail_run)
    with pytest.raises(ValidationError):
        Project("Deck").write_apkg("out.apkg", fail_on="medium", runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")))


def test_write_apkg_returns_invalid_report_without_raising(monkeypatch):
    def fake_run(*args, **kwargs):
        return subprocess.CompletedProcess(["contract_tools"], 1, stdout=json.dumps(build_report_payload(
            status="invalid",
            artifact=None,
            counts={"notes": 0, "cards": 0, "media": 0},
            diagnostics=[{"code": "E", "severity": "error", "message": "bad"}],
        )), stderr="")
    monkeypatch.setattr(subprocess, "run", fake_run)

    report = Project("Deck").write_apkg(
        "out.apkg",
        runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")),
    )

    assert report.status == "invalid"
    with pytest.raises(DiagnosticsError):
        report.ensure_success()


def test_find_workspace_root_discovers_parent_manifest(tmp_path):
    workspace = tmp_path / "workspace"
    nested = workspace / "bindings" / "python"
    manifest = workspace / "contracts" / "manifest.yaml"
    manifest.parent.mkdir(parents=True)
    nested.mkdir(parents=True)
    manifest.write_text("bundle_version: test\n", encoding="utf-8")

    find_workspace_root = getattr(runtime_module, "_find_workspace_root", None)

    assert callable(find_workspace_root)
    assert find_workspace_root(nested) == workspace


def test_workspace_runtime_uses_cargo_build_target(monkeypatch, tmp_path):
    workspace = tmp_path / "workspace"
    manifest = workspace / "contracts" / "manifest.yaml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text("bundle_version: test\n", encoding="utf-8")
    executable_name = "contract_tools.exe" if os.name == "nt" else "contract_tools"
    executable = workspace / "target" / "x86_64-unknown-linux-gnu" / "release" / executable_name
    executable.parent.mkdir(parents=True)
    executable.touch()
    monkeypatch.setenv("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu")

    workspace_runtime = getattr(runtime_module, "_workspace_runtime")

    resolved = workspace_runtime(workspace)

    assert resolved.executable == executable


def test_cloze_marker_detection_accepts_uppercase_c(monkeypatch):
    def fake_run(*args, **kwargs):
        return subprocess.CompletedProcess(["contract_tools"], 0, stdout=json.dumps(build_report_payload()), stderr="")

    monkeypatch.setattr(subprocess, "run", fake_run)

    report = Project("Deck").add_note(Note.cloze("A {{C1::valid}} marker")).write_apkg(
        "out.apkg",
        runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")),
    )

    assert report.status == "success"


def test_cloze_report_json_creates_parent_directories(tmp_path):
    report_json = tmp_path / "nested" / "reports" / "report.json"
    project = Project("Deck")
    project.add_note(Note.cloze("plain text", stable_id="cloze:plain"))

    report = project.write_apkg(
        tmp_path / "no-cloze.apkg",
        report_json=report_json,
        runtime=RuntimeOverride(manifest=Path("contracts/manifest.yaml"), executable=Path("contract_tools")),
    )

    assert report.status == "invalid"
    assert report_json.is_file()


def test_negative_returncode_is_interrupted():
    completed = subprocess.CompletedProcess(["contract_tools"], -2, stdout="", stderr="signal")
    with pytest.raises(RuntimeInvocationError) as err:
        parse_completed_process(completed)
    assert err.value.kind == "interrupted"
    assert err.value.exit_code == -2


def test_negative_returncode_with_report_json_returns_report():
    completed = subprocess.CompletedProcess(["contract_tools"], -2, stdout=json.dumps(build_report_payload(
        status="error",
        artifact=None,
        counts={"notes": 0, "cards": 0, "media": 0},
        diagnostics=[{"code": "E", "severity": "error", "message": "interrupted after report"}],
    )), stderr="signal")
    report = parse_completed_process(completed)
    assert report.status == "error"


def test_runtime_argv_helpers_use_list_args_without_shell(tmp_path):
    """Guard against argv drift while public and low-level runtime wrappers coexist."""
    manifest = tmp_path / "contracts" / "manifest.yaml"
    manifest.parent.mkdir()
    manifest.write_text("bundle_version: test\n", encoding="utf-8")
    product_argv = build_product_build_argv(
        executable=tmp_path / "contract_tools",
        manifest=manifest,
        product_input=tmp_path / "输入.json",
        apkg_out=tmp_path / "out deck.apkg",
        compare_to=None,
        fail_on=None,
        report_json=None,
    )
    low_level = _build_args(
        "normalize",
        {"input_path": str(tmp_path / "输入.json"), "output": "contract-json"},
        LowLevelRuntime("workspace", manifest, manifest.parent, "test", "cargo", ("run", "-q", "-p", "contract_tools", "--")),
    )
    assert isinstance(product_argv, list)
    assert isinstance(low_level, list)
    assert "--product-input" in product_argv
    assert "--input" in low_level
