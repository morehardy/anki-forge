from copy import copy
import gc
import json
import os

import pytest

from anki_forge import Note, Project


def test_report_release_and_copied_handles_have_independent_lifetimes(tmp_path):
    project = Project("Artifacts", base_dir=tmp_path).add_note(Note.basic("Question", "Answer"))
    report = project.build()
    report.ensure_success()
    artifact = report.artifact
    clone = copy(artifact)
    path = artifact.path
    original = path.read_bytes()
    report.close()
    assert report.artifact is None
    assert path.is_file()
    alias = artifact
    artifact.close()
    with pytest.raises(RuntimeError, match="ARTIFACT_CLOSED"):
        _ = alias.path
    assert clone.path.is_file()
    saved = clone.persist_to("保存 files/saved.apkg")
    assert saved.path == tmp_path / "保存 files/saved.apkg"
    assert saved.path.read_bytes() == original
    clone.close()
    assert not path.exists()
    saved_path = saved.path
    saved.close()
    assert saved_path.is_file()


def test_failed_persistence_and_temporary_aliases_preserve_original(tmp_path):
    report = Project("Persistence").add_note(Note.basic("Question", "Answer")).build()
    artifact = report.artifact
    path = artifact.path
    original = path.read_bytes()
    hardlink = tmp_path / "alias.apkg"
    os.link(path, hardlink)
    for target in [path, hardlink, tmp_path]:
        with pytest.raises(OSError):
            artifact.persist_to(target)
        assert path.read_bytes() == original
    destination = tmp_path / "saved.apkg"
    destination.write_bytes(b"previous saved output")
    with artifact.persist_to(destination) as saved:
        assert saved.path.read_bytes() == original
    assert destination.read_bytes() == original
    artifact.close()
    report.close()
    assert not path.exists()
    assert hardlink.read_bytes() == original


def test_report_context_and_copy_keep_only_live_owners():
    with Project("Context").add_note(Note.basic("Question", "Answer")).build() as report:
        report.ensure_success()
        path = report.artifact.path
        cloned_report = copy(report)
        report.artifact.close()
        assert cloned_report.artifact.path.is_file()
    assert report.artifact is None
    assert path.is_file()
    cloned_report.close()
    assert not path.exists()


def test_json_snapshot_does_not_acquire_temporary_artifact_ownership():
    from anki_forge import BuildReport

    report = Project("JSON ownership").add_note(Note.basic("Question", "Answer")).build()
    path = report.artifact.path
    serialized = BuildReport.from_json(report.to_json())
    assert serialized.artifact["path"] == str(path)
    report.close()
    gc.collect()
    assert not path.exists()


def test_late_failure_exception_retains_recoverable_artifact(tmp_path):
    from anki_forge import BuildError, BuildOptions

    project = Project("Late failure", stable_id="late-failure").add_note(Note.basic("Question", "Answer"))
    report = project.build(BuildOptions(identity_lockfile=tmp_path, write_identity_lockfile=True, update_safety="disabled"))
    assert any(d.code == "UPDATE.LOCKFILE_WRITE_FAILED" for d in report.diagnostics)
    path = report.artifact.path
    with pytest.raises(BuildError) as caught:
        report.ensure_success()
    del report
    gc.collect()
    assert path.is_file()
    saved = caught.value.report.artifact.persist_to(tmp_path / "recovered.apkg")
    assert saved.path.is_file()
    caught.value.report.close()
    assert not path.exists()


def test_report_json_requires_persistent_artifact_destination(tmp_path):
    from anki_forge import BuildOptions

    target = tmp_path / "report.json"
    report = Project("Temporary JSON").add_note(Note.basic("Question", "Answer")).build(BuildOptions(report_json=target))
    assert report.artifact is None
    assert any(d.code == "PROJECT.REPORT_JSON_WRITE_FAILED" for d in report.diagnostics)
    assert json.loads(target.read_text())["artifact"] is None
