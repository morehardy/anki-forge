import pytest

from anki_forge import Field, Note, NoteType, Project, ProjectAddError, Template


def test_validation_preserves_auto_key_warning_and_does_not_read_media(tmp_path):
    field = Field("Back  Extra")
    template = Template("C++ Card", front="{{Back  Extra}}", back="{{Back  Extra}}")
    assert field.key == "back__extra"
    assert template.key == "c++_card"
    project = Project("Validate", base_dir=tmp_path).add_notetype(
        NoteType.custom("custom").field(field).template(template)
    )
    source = tmp_path / "registered.wav"
    source.write_bytes(b"RIFF registered")
    project.media.add_file(source, export_as="registered.wav")
    source.unlink()
    validation = project.validate()
    assert not validation.has_errors
    validation.ensure_success()
    assert [(d.code, d.severity, d.path) for d in validation.diagnostics] == [
        ("NOTETYPE.IDENTITY_RECIPE_MISSING", "warning", 'project.note_types["custom"]'),
        ("NOTETYPE.FIELD_KEY_AUTO_DERIVED", "warning", 'project.note_types["custom"].fields["Back  Extra"]'),
    ]
    assert list(tmp_path.iterdir()) == []


def test_project_observations_are_detached_and_failed_addition_is_atomic():
    note_type = (NoteType.custom("custom").field(Field("Prompt", key="prompt", identity=True))
        .template(Template("Card", key="card", front="{{Prompt}}", back="{{Prompt}}")))
    project = Project("Snapshots").add_notetype(note_type)
    project.add_note(Note("custom", stable_id="one").text("prompt", "original"))
    with pytest.raises(ProjectAddError) as caught:
        project.add_note(Note("custom", stable_id="one").text("prompt", "duplicate"))
    assert caught.value.code == "AFID.STABLE_ID_DUPLICATE"
    assert caught.value.path == "project.notes[1]"
    assert caught.value.diagnostic.severity == "error"
    assert len(project.notes) == 1
    project.notes[0].text("prompt", "snapshot changed")
    project.notetypes["custom"].css("snapshot changed")
    assert project.notes[0].fields["prompt"].value == "original"
    assert project.notetypes["custom"].css_value is None
    assert project.notetype_order == ("custom",)
    with pytest.raises(TypeError):
        project.notetypes["other"] = note_type
    with pytest.raises(AttributeError):
        project.name = "silently ignored change"
    project.add_note(Note("custom", stable_id="two").text("Prompt", "second"))
    report = project.build()
    report.ensure_success()
    assert report.counts["notes"] == 2


def test_optional_field_declaration_round_trips_without_becoming_required():
    from anki_forge import ValidationError

    note_type = (NoteType.custom("optional")
        .field(Field("Prompt", key="prompt", identity=True))
        .field(Field("Extra", key="extra", optional=True))
        .template(Template("Card", key="card", front="{{Prompt}}", back="{{Extra}}")))
    project = Project("Optional").add_notetype(note_type)
    extra = project.notetypes["optional"].fields[1]
    assert extra.optional and not extra.required
    project.add_note(Note("optional").text("prompt", "question"))
    project.build().ensure_success()
    with pytest.raises(ValidationError, match="required.*optional"):
        Field("Conflicting", required=True, optional=True)


def test_invalid_authoring_uses_structured_core_errors_and_allows_retry():
    invalid = (NoteType.custom("retry").field(Field("Prompt", key="prompt"))
        .field(Field("Duplicate", key="prompt")))
    project = Project("Retry")
    with pytest.raises(ProjectAddError) as duplicate:
        project.add_notetype(invalid)
    assert duplicate.value.code == "NOTETYPE.FIELD_KEY_DUPLICATE"
    assert project.notetype_order == ()
    with pytest.raises(ProjectAddError) as unknown:
        project.add_note(Note("basic").text("unexpected", "value"))
    assert unknown.value.code == "PRODUCT.FIELD_UNKNOWN"
    assert unknown.value.path == 'project.notes[0].fields["unexpected"]'
    assert project.notes == ()
    repaired = (NoteType.custom("retry").field(Field("Prompt", key="prompt"))
        .template(Template("Card", key="card", front="{{Prompt}}", back="{{Prompt}}")))
    project.add_notetype(repaired)
    with pytest.raises(ProjectAddError) as missing:
        project.add_note(Note("retry").text("prompt", "value"))
    assert missing.value.code == "PRODUCT.IDENTITY_MISSING"
    project.add_note(Note("retry").text("Prompt", "value").identity(["prompt"]))
    project.build().ensure_success()


def test_template_error_preserves_core_byte_span_and_failed_type_is_not_added():
    from anki_forge import SourceSpan

    project = Project("Template diagnostics")
    invalid = (NoteType.custom("bad").field(Field("Prompt", key="prompt"))
        .template(Template("Card", front="中{{Missing}}", back="{{Prompt}}")))
    with pytest.raises(ProjectAddError) as caught:
        project.add_notetype(invalid)
    assert caught.value.code == "TEMPLATE.RENDER_FIELD_UNKNOWN"
    assert caught.value.path == 'project.note_types[0].templates["Card"].front'
    assert caught.value.span == SourceSpan(3, 5)
    assert project.notetypes == {}


def test_typed_content_renders_through_rust_and_can_be_added_to_a_note(tmp_path):
    from anki_forge import Content

    project = Project("Content")
    source = tmp_path / "sound.wav"
    source.write_bytes(b"RIFF example")
    media = project.media.add_file(source, export_as="sound.wav")
    assert Content.text('<b>"中" &amp;</b>').render() == '&lt;b&gt;&quot;中&quot; &amp;amp;&lt;/b&gt;'
    assert Content.html("<b>中</b>").render() == "<b>中</b>"
    assert media.sound().render() == "[sound:sound.wav]"
    assert media.image().render() == '<img src="sound.wav">'
    project.add_note(Note("basic").field("Front", Content.text("question")).field("Back", media.sound()))
    project.build().ensure_success()


def test_image_occlusion_builder_uses_core_validation_and_can_be_retried(tmp_path):
    from anki_forge import ProductNoteError, ValidationError

    project = Project("Occlusion")
    image_path = tmp_path / "image.svg"
    image_path.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"></svg>')
    image = project.media.add_file(image_path, export_as="image.svg")
    with pytest.raises(ProductNoteError) as missing:
        Note.image_occlusion(image).rect(0, 0, 10, 10).build()
    assert missing.value.code == "DECK.MISSING_STABLE_ID"
    builder = Note.image_occlusion(image, stable_id="io-1").header("Header").tag("io")
    with pytest.raises(ProductNoteError) as empty:
        builder.build()
    assert empty.value.code == "DECK.EMPTY_IO_MASKS"
    builder.rect(0, 0, 10, 10)
    project.add_note(builder.build())
    project.build().ensure_success()
    for rect in [(False, 0, 10, 10), (0, 0, 2**32, 10)]:
        with pytest.raises(ValidationError):
            builder.rect(*rect)
