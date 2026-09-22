import pytest
from anki_forge import BuildOptions, Field, GenerationRule, Note, NoteType, Project, ProjectAddError, Template, ValidationError


def custom(identity=True):
    return NoteType.custom("x").field(Field("Front", key="front", identity=identity)).template(Template("Card", key="card", front="{{Front}}", back="{{FrontSide}}"))


def test_missing_custom_identity_fails_at_addition():
    project = Project("Deck").add_notetype(custom(False))
    with pytest.raises(ProjectAddError) as caught:
        project.add_note(Note("x").text("front", "value"))
    assert caught.value.code == "PRODUCT.IDENTITY_MISSING"
    assert not project.notes


def test_duplicate_stable_ids_are_project_wide():
    project = Project("Deck").add_note(Note.basic("a", "b", stable_id="same")).add_notetype(custom())
    with pytest.raises(ProjectAddError) as caught:
        project.add_note(Note("x", stable_id="same").text("front", "value"))
    assert caught.value.code == "AFID.STABLE_ID_DUPLICATE"
    assert len(project.notes) == 1


def test_generation_rule_reference_fails_at_addition():
    nt = custom().template(Template("Broken", key="broken", front="{{Front}}", back="{{Front}}", generate_when=GenerationRule.all(["missing"])))
    with pytest.raises(ProjectAddError):
        Project("Deck").add_notetype(nt)


def test_unknown_fields_fail_and_later_input_mutation_is_detached():
    project = Project("Deck").add_notetype(custom())
    with pytest.raises(ProjectAddError):
        project.add_note(Note("x").text("missing", "value"))
    note = Note("x").text("front", "value")
    project.add_note(note)
    note.text("missing", "late mutation")
    project.build().ensure_success()
    assert "missing" not in project.notes[0].fields


def test_reserved_stock_type_is_rejected():
    with pytest.raises(ProjectAddError):
        Project("Deck").add_notetype(NoteType.custom("basic"))


@pytest.mark.parametrize("mode,status,severity", [("strict", "invalid", "error"), ("report_only", "success", "warning"), ("report-only", "success", "warning"), ("disabled", "success", None)])
def test_core_decides_missing_project_identity_by_safety_mode(tmp_path, mode, status, severity):
    project = Project("Deck").add_note(Note.basic("front", "back"))
    report = project.build(BuildOptions(identity_lockfile=tmp_path / "missing.lock.json", update_safety=mode))
    assert report.status == status
    matching = [d for d in report.diagnostics if d.code == "UPDATE.PROJECT_STABLE_ID_MISSING"]
    assert ([d.severity for d in matching] == [severity]) if severity else not matching


def test_invalid_mode_fails_before_build():
    with pytest.raises(ValidationError):
        BuildOptions(update_safety="unknown")


def test_writing_lockfile_requires_a_path_in_core_report():
    report = Project("Deck", stable_id="deck").add_note(Note.basic("front", "back")).build(BuildOptions(write_identity_lockfile=True))
    assert report.status == "invalid"
    assert any(d.code == "UPDATE.LOCKFILE_PATH_REQUIRED" for d in report.diagnostics)


@pytest.mark.parametrize("alias", ["output", "report"])
def test_core_rejects_lockfile_publication_aliases(tmp_path, alias):
    shared = tmp_path / "same"
    shared.write_bytes(b"preserve")
    project = Project("Deck", stable_id="deck").add_note(Note.basic("front", "back"))
    report = project.build(BuildOptions(output=shared if alias == "output" else tmp_path / "out.apkg", report_json=shared if alias == "report" else None, identity_lockfile=shared))
    assert report.status == "invalid"
    assert any(d.code == "PROJECT.PATH_COLLISION" for d in report.diagnostics)
    assert shared.read_bytes() == b"preserve"
