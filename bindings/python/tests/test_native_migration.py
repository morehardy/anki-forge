from pathlib import Path
import json
import shutil
import pytest

from anki_forge import Field, Note, NoteType, Project, Template
from test_native_parity import observe

FIXTURES = Path(__file__).parent / "fixtures/python01"
CUSTOM_GUID = "afid:v1:975df42793b505aad2611005f54be3e7a95df8acfbca0387392ddcd3b569e2f3"


def migrated_project(answer="Custom answer", tag="baseline"):
    project = Project("Python migration", stable_id="python01-migration")
    project.add_note(Note.basic("Basic front", "Basic answer", stable_id="basic-1").tag("baseline"))
    project.add_notetype(NoteType.custom("custom")
        .field(Field("Prompt", key="prompt", identity=True))
        .field(Field("Back  Extra", key="back_extra"))
        .template(Template("C++ Card", key="c_card", front="{{Prompt}}", back="{{Back  Extra}}"))
        .template(Template("Reverse", key="reverse", front="{{Back  Extra}}", back="{{Prompt}}")))
    project.add_note(Note("custom").text("prompt", "Custom front").text("back_extra", answer).tag(tag))
    return project


def test_actual_python01_apkg_and_lockfile_migrate_without_identity_or_revision_changes(tmp_path):
    baseline = FIXTURES / "baseline.apkg"
    original = baseline.read_bytes()
    lockfile = tmp_path / "migration.lock.json"
    shutil.copyfile(FIXTURES / "baseline.lock.json", lockfile)
    output = tmp_path / "python02.apkg"
    report = migrated_project().write_apkg(output, compare_to=baseline, identity_lockfile=lockfile,
        write_identity_lockfile=True, update_safety="strict", fail_on="high")
    report.ensure_success()
    assert report.update_safety["notes_preserved"] == 2
    assert report.update_safety["lockfile_written"] is True
    assert report.policy["status"] == "passed"
    assert report.comparison == "complete"
    expected = observe("inspect", baseline)
    # Core Project persists inferred generation requirements that the 0.1
    # ProductDocument path did not serialize. Assert these specific additions;
    # identity, revision, model IDs and all other observations remain compared.
    custom_templates = [item for item in expected["observations"]["templates"] if item["notetype_id"] == "custom"]
    assert [item["name"] for item in custom_templates] == ["C++ Card", "Reverse"]
    for template, field in zip(custom_templates, ["Prompt", "Back  Extra"]):
        assert "generation_requirement" not in template
        template["generation_requirement"] = {"kind": "all", "field_names": [field]}
    identity_metadata = [item for item in expected["observations"]["metadata"] if "guid_source" in item]
    assert len(identity_metadata) == 2
    for item in identity_metadata:
        assert item["guid_source"] == "current_derivation"
        item["guid_source"] = "previous_apkg"
    assert observe("inspect", output) == expected
    assert baseline.read_bytes() == original


@pytest.mark.parametrize("answer,tag", [("Changed answer", "baseline"), ("Custom answer", "changed-tag")])
def test_python01_answer_and_tag_updates_keep_guid_and_advance_only_changed_revision(tmp_path, answer, tag):
    baseline = FIXTURES / "baseline.apkg"
    original = observe("inspect", baseline)["identity"]
    lockfile = tmp_path / "migration.lock.json"
    shutil.copyfile(FIXTURES / "baseline.lock.json", lockfile)
    output = tmp_path / "changed.apkg"
    report = migrated_project(answer, tag).write_apkg(output, compare_to=baseline,
        identity_lockfile=lockfile, write_identity_lockfile=True, update_safety="strict")
    report.ensure_success()
    changed = observe("inspect", output)["identity"]
    assert changed["notetypes"] == original["notetypes"]
    before = {note["anki_guid"]: note for note in original["notes"]}
    after = {note["anki_guid"]: note for note in changed["notes"]}
    assert set(before) == set(after) == {"basic-1", CUSTOM_GUID}
    assert after["basic-1"] == before["basic-1"]
    assert after[CUSTOM_GUID]["provenance"] == before[CUSTOM_GUID]["provenance"]
    assert after[CUSTOM_GUID]["revision"]["mtime_secs"] > before[CUSTOM_GUID]["revision"]["mtime_secs"]
    assert after[CUSTOM_GUID]["revision"]["content_hash"] != before[CUSTOM_GUID]["revision"]["content_hash"]
    repeated = tmp_path / "repeated.apkg"
    migrated_project(answer, tag).write_apkg(repeated, compare_to=output, identity_lockfile=lockfile,
        write_identity_lockfile=True, update_safety="strict").ensure_success()
    assert observe("inspect", repeated)["identity"] == changed
    reverted = tmp_path / "reverted.apkg"
    migrated_project().write_apkg(reverted, compare_to=repeated, identity_lockfile=lockfile,
        write_identity_lockfile=True, update_safety="strict").ensure_success()
    restored = {note["anki_guid"]: note for note in observe("inspect", reverted)["identity"]["notes"]}
    assert set(restored) == {"basic-1", CUSTOM_GUID}
    assert restored[CUSTOM_GUID]["revision"]["content_hash"] == before[CUSTOM_GUID]["revision"]["content_hash"]
    assert restored[CUSTOM_GUID]["revision"]["mtime_secs"] > after[CUSTOM_GUID]["revision"]["mtime_secs"]


def test_legacy_lockfile_requires_apkg_evidence_before_safe_migration(tmp_path):
    legacy = json.loads((FIXTURES / "baseline.lock.json").read_text())
    for note in legacy["identity_index"]["notes"]:
        note.pop("revision")
    lockfile = tmp_path / "legacy.lock.json"
    lockfile.write_text(json.dumps(legacy))
    original = lockfile.read_bytes()
    output = tmp_path / "output.apkg"
    output.write_bytes(b"previous output")
    project = migrated_project()
    report = project.write_apkg(output, identity_lockfile=lockfile, write_identity_lockfile=True, update_safety="strict")
    assert report.status != "success"
    assert lockfile.read_bytes() == original
    assert output.read_bytes() == b"previous output"
    assert any(d.code == "UPDATE.NOTE_REVISION_MISSING" for d in report.diagnostics)
    recovered = project.write_apkg(output, compare_to=FIXTURES / "baseline.apkg",
        identity_lockfile=lockfile, write_identity_lockfile=True, update_safety="strict")
    recovered.ensure_success()
    assert recovered.update_safety["notes_preserved"] == 2
    migrated = json.loads(lockfile.read_text())
    assert all(note["revision"]["mtime_secs"] == 1 for note in migrated["identity_index"]["notes"])
