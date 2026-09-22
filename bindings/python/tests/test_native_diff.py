from pathlib import Path
import json
import subprocess
import tempfile

import pytest

from anki_forge import BuildOptions, InspectLimits, Note, Project
from test_native_parity import OBSERVER


def make_project(answer="Back"):
    return Project("Diff", stable_id="native-diff").add_note(
        Note.basic("Front", answer, stable_id="one").tag("diff")
    )


def test_diff_returns_complete_report_without_publication_or_lock_changes(tmp_path, monkeypatch):
    baseline = tmp_path / "baseline.apkg"
    lock = tmp_path / "identity.lock.json"
    make_project().build(BuildOptions(output=baseline, identity_lockfile=lock, write_identity_lockfile=True)).ensure_success()
    before = {path.name: path.read_bytes() for path in tmp_path.iterdir()}
    candidate_dirs = set(Path(tempfile.gettempdir()).glob("anki-forge-project-diff-*"))
    project = make_project("changed")
    monkeypatch.chdir(tmp_path)
    report = project.diff_against_apkg(baseline)
    report.ensure_success()
    assert report.comparison == "complete"
    assert report.current_inspect is not None
    assert report.previous_inspect is not None
    assert report.diff["summary_counts"]["modified"] > 0
    assert report.risk is not None
    assert report.metrics["duration_ms"] >= 0
    expected = json.loads(subprocess.check_output([str(OBSERVER), "diff", str(baseline)], encoding="utf-8"))
    actual = report.raw
    # Only wall-clock duration differs between the independent Rust/Python runs.
    assert expected["metrics"]["duration_ms"] >= 0
    expected["metrics"]["duration_ms"] = actual["metrics"]["duration_ms"]
    assert actual == expected
    assert {path.name: path.read_bytes() for path in tmp_path.iterdir()} == before
    assert set(Path(tempfile.gettempdir()).glob("anki-forge-project-diff-*")) == candidate_dirs
    project.add_note(Note.basic("usable", "after comparison", stable_id="two"))
    project.build().ensure_success()


def test_failed_diff_retains_structured_report_and_limits(tmp_path):
    from anki_forge import ProjectDiffError

    project = make_project()
    missing = project.diff_against_apkg(tmp_path / "missing.apkg")
    assert missing.status != "success"
    with pytest.raises(ProjectDiffError) as caught:
        missing.ensure_success()
    assert caught.value.report is missing
    assert caught.value.failure_cause is not None
    assert caught.value.diagnostics
    baseline = tmp_path / "baseline.apkg"
    project.write_apkg(baseline).ensure_success()
    limited = project.diff_against_apkg(baseline, inspect_limits=InspectLimits(max_archive_bytes=0))
    assert limited.status != "success"
    assert any(d.code == "INSPECT.RESOURCE_LIMIT_EXCEEDED" for d in limited.diagnostics)
    project.diff_against_apkg(baseline).ensure_success()


def test_comparison_json_preserves_extensions_and_integer_precision(tmp_path):
    from anki_forge import ProjectDiffReport

    baseline = tmp_path / "baseline.apkg"
    make_project().write_apkg(baseline).ensure_success()
    raw = make_project("changed").diff_against_apkg(baseline).raw
    raw["extension"] = {"counter": 2**100}
    raw["metrics"]["duration_ms"] = 2**80
    parsed = ProjectDiffReport.from_json(raw)
    assert parsed.to_json() == raw
    raw["extension"]["counter"] = 0
    assert parsed.raw["extension"]["counter"] == 2**100
