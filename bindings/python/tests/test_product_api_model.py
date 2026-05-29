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


def test_template_rejects_invalid_generate_when_type():
    with pytest.raises(ValidationError):
        Template("Broken", front="x", back="y", generate_when="bad")


def test_generation_rules_validate_empty_inputs():
    assert GenerationRule.anki_default().kind == "anki_default"
    assert GenerationRule.cloze("text").field == "text"
    with pytest.raises(ValidationError):
        GenerationRule.all([])
    with pytest.raises(ValidationError):
        GenerationRule.any([""])


def test_generation_rules_validate_direct_construction():
    direct = GenerationRule("all", fields=["front"])
    assert direct.fields == ("front",)
    assert direct.field is None
    with pytest.raises(ValidationError):
        GenerationRule("invalid")
    with pytest.raises(ValidationError):
        GenerationRule("all")
    with pytest.raises(ValidationError):
        GenerationRule("all", fields=("front",), field="front")
    with pytest.raises(ValidationError):
        GenerationRule("cloze", fields=("front",), field="text")
    with pytest.raises(ValidationError):
        GenerationRule("anki_default", fields=("front",))


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


def test_note_media_content_and_cloze_defaults_validate_shape():
    ref = MediaRef("media:000001", "x.png")
    note = Note("custom").sound("audio", ref).image("picture", ref)
    assert ref.media_id == "media:000001"
    assert note.fields["audio"].media_id == "media:000001"
    assert note.fields["audio"].value is None
    assert note.fields["picture"].media_id == "media:000001"
    cloze = Note.cloze("<b>{{c1::text}}</b>")
    assert cloze.fields["text"].kind == "html"
    assert cloze.fields["text"].value == "<b>{{c1::text}}</b>"
    assert cloze.fields["back_extra"].kind == "text"
    assert cloze.fields["back_extra"].value == ""
    with pytest.raises(ValidationError):
        Note("custom").text("front", 123)
    with pytest.raises(ValidationError):
        Note("custom").html("front", object())


def test_media_registry_validates_export_names_and_duplicates(tmp_path):
    registry = Project("Deck").media
    payload = b"abc"
    ref = registry.add_bytes(source_label="hello.wav", data=payload, export_as="hello.wav")
    assert registry.items[0].data == payload
    assert registry.items[0].source_kind == "bytes"
    assert registry.add_bytes(source_label="again.wav", data=bytearray(b"abc"), export_as="hello.wav") == ref
    with pytest.raises(ValidationError):
        registry.add_bytes(source_label="bad", data=b"different", export_as="hello.wav")
    with pytest.raises(ValidationError):
        registry.add_bytes(source_label="bad", data=b"x", export_as="../bad.wav")
    with pytest.raises(ValidationError):
        registry.add_bytes(source_label="bad", data=b"x", export_as="bad:name.wav")
    safe_ref = registry.add_bytes(source_label="safe", data=b"safe", export_as="hello-world_1.wav")
    assert safe_ref.export_as == "hello-world_1.wav"
    media_file = tmp_path / "sound.wav"
    media_file.write_bytes(b"RIFF")
    file_ref = registry.add_file(media_file, export_as="sound.wav")
    assert file_ref.export_as == "sound.wav"
    assert registry.add_file(media_file, export_as="sound.wav") == file_ref
    file_item = next(item for item in registry.items if item.ref == file_ref)
    assert file_item.source_kind == "file"
    assert file_item.data is None
    assert isinstance(registry.items, tuple)


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


def test_project_accepts_hyphenated_custom_ids_and_validates_mutated_notetypes():
    nt = NoteType.custom("media-card", css=".card {}").field(Field("Front", key="front"))
    assert nt.css_value == ".card {}"
    project = Project("Deck").add_notetype(nt)
    assert project._note_type_order == ["media-card"]
    project.add_note(Note("media-card").text("front", "x"))
    nt.fields.append(Field("Other Front", key="front"))
    with pytest.raises(ValidationError):
        nt.validate()
    with pytest.raises(ValidationError):
        Project("Other").add_notetype(nt)


def test_notetype_defaults_name_to_id_and_keeps_id_stable():
    nt = NoteType.custom("custom")
    assert nt.name == "custom"
    with pytest.raises(AttributeError):
        nt.id = "other"
    with pytest.raises(AttributeError):
        del nt.id
    assert nt.id == "custom"
    nt.field(Field("Front", key="front"))
    assert nt.fields[0].key == "front"


def test_project_notetypes_is_read_only_but_add_notetype_still_mutates():
    project = Project("Deck")
    with pytest.raises(TypeError):
        project.notetypes["custom"] = NoteType.custom("custom").field(Field("Front", key="front"))
    assert project.notetypes == {}
    project.add_notetype(NoteType.custom("custom").field(Field("Front", key="front")))
    assert "custom" in project.notetypes
    assert project._note_type_order == ["custom"]
