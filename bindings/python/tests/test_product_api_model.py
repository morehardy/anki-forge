import pytest

from anki_forge import Field, GenerationRule, MediaRef, Note, NoteType, Project, Template, ValidationError


def test_notetype_custom_templates_and_duplicate_validation():
    nt = NoteType.custom("jp")
    assert nt.css(None).css_value is None
    nt.field(Field("Expression", key="expr", identity=True, sort=True, required=True))
    nt.field(Field("Meaning", key="meaning"))
    nt.template(Template("Recognition", front="{{Expression}}", back="{{Meaning}}", generate_when=GenerationRule.all(["expr"])))
    with pytest.raises(ValidationError):
        nt.field(Field("Duplicate", key="expr"))
    with pytest.raises(ValidationError):
        nt.field(Field("Expression", key="expr_2"))
    with pytest.raises(ValidationError):
        nt.field(Field("Other Sort", key="sort2", sort=True))
    with pytest.raises(ValidationError):
        nt.template(Template("Recognition", front="x", back="y"))
    with pytest.raises(ValidationError):
        nt.template(Template("Broken", front="x", back="y", generate_when=GenerationRule.all(["missing"])))


def test_generation_rules_validate_empty_inputs():
    assert GenerationRule.anki_default().kind == "anki_default"
    assert GenerationRule.cloze("text").field == "text"
    with pytest.raises(ValidationError):
        GenerationRule.all([])
    with pytest.raises(ValidationError):
        GenerationRule.any([""])


def test_note_mutators_validate_tags_and_deck_clearing():
    note = Note.basic("front", "back", stable_id="n1", deck_name="Deck").tag("demo").deck(None)
    assert note.note_type_id == "basic"
    assert note.fields["front"].kind == "text"
    assert note.fields["back"].value == "back"
    assert note.deck_name is None
    with pytest.raises(ValidationError):
        note.tag("bad tag")
    with pytest.raises(ValidationError):
        Note("basic", stable_id=" ")


def test_note_tags_mutator_validates_and_deduplicates():
    note = Note.basic("front", "back").tags(["demo", "demo", "review"]).tag("review")
    assert hasattr(Note, "tags")
    assert note.tag_values == ["demo", "review"]
    with pytest.raises(ValidationError):
        note.tags(["ok", "bad tag"])


def test_stock_notes_reject_unknown_field_keys():
    ref = MediaRef("media:000001", "x.png")
    with pytest.raises(ValidationError):
        Note("basic").text("missing", "value")
    with pytest.raises(ValidationError):
        Note("basic").sound("text", ref)
    with pytest.raises(ValidationError):
        Note("cloze").html("front", "value")
    with pytest.raises(ValidationError):
        Note("cloze").image("back", ref)


def test_media_registry_validates_export_names_and_duplicates(tmp_path):
    registry = Project("Deck").media
    ref = registry.add_bytes(source_label="hello.wav", data=b"abc", export_as="hello.wav")
    assert registry.add_bytes(source_label="again.wav", data=bytearray(b"abc"), export_as="hello.wav") == ref
    with pytest.raises(ValidationError):
        registry.add_bytes(source_label="bad", data=b"different", export_as="hello.wav")
    with pytest.raises(ValidationError):
        registry.add_bytes(source_label="bad", data=b"x", export_as="../bad.wav")
    media_file = tmp_path / "sound.wav"
    media_file.write_bytes(b"RIFF")
    file_ref = registry.add_file(media_file, export_as="sound.wav")
    assert file_ref.export_as == "sound.wav"


def test_project_validates_ids_and_custom_note_registration():
    with pytest.raises(ValidationError):
        Project("Deck", stable_id="")
    project = Project("Deck")
    with pytest.raises(ValidationError):
        project.add_note(Note("custom").text("front", "x"))
    project.add_notetype(NoteType.custom("custom").field(Field("Front", key="front", identity=True)))
    with pytest.raises(ValidationError):
        project.add_note(Note("custom").text("missing", "x"))
    project.add_note(Note("custom").text("front", "x"))
