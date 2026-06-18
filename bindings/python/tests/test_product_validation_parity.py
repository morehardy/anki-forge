import json
import subprocess

import pytest

from anki_forge import Field, GenerationRule, Note, NoteType, Project, Template, ValidationError
from anki_forge.runtime import RuntimeOverride


def test_missing_custom_identity_with_unstabilized_note_fast_fails():
    nt = NoteType.custom("x").field(Field("Front", key="front"))
    project = Project("Deck").add_notetype(nt).add_note(Note("x").text("front", "value"))
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_duplicate_stable_ids_are_project_wide():
    project = Project("Deck")
    project.add_note(Note.basic("a", "b", stable_id="same"))
    nt = NoteType.custom("x").field(Field("Front", key="front", identity=True))
    project.add_notetype(nt)
    project.add_note(Note("x", stable_id="same").text("front", "value"))
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_generation_rule_reference_fast_fails():
    nt = NoteType.custom("x").field(Field("Front", key="front"))
    with pytest.raises(ValidationError):
        nt.template(Template("Broken", front="{{Front}}", back="{{Front}}", generate_when=GenerationRule.all(["missing"])))


def test_custom_note_unknown_field_key_fast_fails_and_rechecks():
    nt = NoteType.custom("x").field(Field("Front", key="front", identity=True))
    project = Project("Deck").add_notetype(nt)
    with pytest.raises(ValidationError):
        project.add_note(Note("x").text("missing", "value"))
    note = Note("x").text("front", "value")
    project.add_note(note)
    note.text("missing", "late mutation")
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_reserved_stock_id_fast_fails_for_custom_notetype():
    with pytest.raises(ValidationError):
        Project("Deck").add_notetype(NoteType.custom("basic"))


def test_fail_on_requires_compare_to_fast_fails_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")
    with pytest.raises(ValidationError):
        Project("Deck").write_apkg(tmp_path / "out.apkg", fail_on="medium", runtime=runtime)


def test_update_safe_requires_project_stable_id_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    with pytest.raises(ValidationError, match="stable_id"):
        Project("Deck").write_apkg(
            tmp_path / "out.apkg",
            identity_lockfile=tmp_path / "anki-forge.lock.json",
            runtime=runtime,
        )


def test_update_safety_strict_requires_project_stable_id_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    with pytest.raises(ValidationError, match="stable_id"):
        Project("Deck").write_apkg(
            tmp_path / "out.apkg",
            update_safety="strict",
            runtime=runtime,
        )


def test_update_safety_disabled_allows_identity_lockfile_without_project_stable_id(monkeypatch, tmp_path):
    def fake_run(args, **kwargs):
        return subprocess.CompletedProcess(
            args,
            0,
            stdout=json.dumps({
                "kind": "anki-forge-build-report",
                "schema_version": "phase4-build-report-v2",
                "tool_version": "test",
                "status": "success",
                "comparison": "not_requested",
                "artifact": {"path": str(tmp_path / "out.apkg")},
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
                "update_safety": {"mode": "disabled", "baseline_sources": [], "notes_preserved": 0, "notes_derived": 1, "notes_failed": 0, "baseline_conflicts": 0, "blocking_diagnostics": [], "lockfile_written": False},
            }),
            stderr="",
        )

    monkeypatch.setattr(subprocess, "run", fake_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    report = Project("Deck").add_note(Note.basic("front", "back")).write_apkg(
        tmp_path / "out.apkg",
        identity_lockfile=tmp_path / "ignored.lock.json",
        update_safety="disabled",
        runtime=runtime,
    )

    assert report.status == "success"


@pytest.mark.parametrize("mode", ["report_only", "report-only"])
def test_update_safety_report_only_allows_missing_project_stable_id_before_runtime(monkeypatch, tmp_path, mode):
    captured = {}

    def fake_run(args, **kwargs):
        captured["args"] = list(args)
        return subprocess.CompletedProcess(
            args,
            0,
            stdout=json.dumps({
                "kind": "anki-forge-build-report",
                "schema_version": "phase4-build-report-v2",
                "tool_version": "test",
                "status": "success",
                "comparison": "not_requested",
                "artifact": {"path": str(tmp_path / "out.apkg")},
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
                "diagnostics": [
                    {
                        "code": "UPDATE_SAFE.PROJECT_STABLE_ID_MISSING",
                        "severity": "warning",
                        "domain": "update_safety",
                        "stage": "validate",
                        "path": "project.stable_id",
                        "message": "project stable_id is recommended for update-safe builds",
                        "suggested_fix": "set Project.stable_id",
                    }
                ],
                "metrics": {"duration_ms": 1},
                "policy": {
                    "status": "not_evaluated",
                    "threshold": None,
                    "highest_risk": None,
                    "blocking_findings": [],
                },
                "update_safety": {"mode": mode, "baseline_sources": [], "notes_preserved": 0, "notes_derived": 1, "notes_failed": 0, "baseline_conflicts": 0, "blocking_diagnostics": [], "lockfile_written": False},
            }),
            stderr="",
        )

    monkeypatch.setattr(subprocess, "run", fake_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    report = Project("Deck").add_note(Note.basic("front", "back")).write_apkg(
        tmp_path / "out.apkg",
        identity_lockfile=tmp_path / "baseline.lock.json",
        update_safety=mode,
        runtime=runtime,
    )

    assert "--update-safety" in captured["args"]
    assert captured["args"][captured["args"].index("--update-safety") + 1] == mode
    assert report.diagnostics[0].severity == "warning"


def test_unknown_update_safety_fast_fails_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    with pytest.raises(ValidationError, match="update_safety"):
        Project("Deck").write_apkg(
            tmp_path / "out.apkg",
            update_safety="unknown",
            runtime=runtime,
        )


def test_write_identity_lockfile_requires_identity_lockfile_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")

    with pytest.raises(ValidationError, match="identity_lockfile"):
        Project("Deck", stable_id="deck").write_apkg(
            tmp_path / "out.apkg",
            write_identity_lockfile=True,
            runtime=runtime,
        )


def test_identity_lockfile_must_differ_from_apkg_output_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")
    shared_path = tmp_path / "same-path"

    with pytest.raises(ValidationError, match="identity_lockfile"):
        Project("Deck", stable_id="deck").write_apkg(
            shared_path,
            identity_lockfile=shared_path,
            runtime=runtime,
        )


def test_identity_lockfile_must_differ_from_report_json_before_runtime(monkeypatch, tmp_path):
    def fail_run(*args, **kwargs):
        raise AssertionError("subprocess must not run")

    monkeypatch.setattr(subprocess, "run", fail_run)
    runtime = RuntimeOverride(manifest=tmp_path / "manifest.yaml", executable=tmp_path / "contract_tools")
    shared_path = tmp_path / "same-path"

    with pytest.raises(ValidationError, match="identity_lockfile"):
        Project("Deck", stable_id="deck").write_apkg(
            tmp_path / "out.apkg",
            report_json=shared_path,
            identity_lockfile=shared_path,
            runtime=runtime,
        )
