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
