"""Compare independent producers through the same APKG observer."""
import json
import os
from pathlib import Path
import subprocess
import pytest

from anki_forge import Field, GenerationRule, Note, NoteType, Project, Template

ROOT = Path(__file__).resolve().parents[3]
OBSERVER = Path(os.environ.get("ANKI_FORGE_PYTHON_OBSERVER", str(
    ROOT / "target/debug/examples" / ("python_parity.exe" if os.name == "nt" else "python_parity")
)))


def observe(operation, path, *inputs):
    assert OBSERVER.is_file(), "build the python_parity Rust example before running parity tests"
    value = json.loads(subprocess.check_output([str(OBSERVER), operation, str(path), *map(str, inputs)], encoding="utf-8"))
    # The only ignored evidence is the observer's input filename, checked first.
    for note in value["identity"]["notes"]:
        assert note["source_path"] == str(path)
        note["source_path"] = "<observed-apkg>"
    return value


def test_basic_matches_independent_rust_identity_and_observations(tmp_path):
    expected = observe("basic", tmp_path / "rust.apkg")
    report = Project("Native", stable_id="native-basic").add_note(
        Note.basic("Front", "Back", stable_id="note-1")
    ).build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == expected


@pytest.mark.parametrize("scenario,field_name,template_name,key", [
    ("unicode", "中文", "卡片", None),
    ("spaces", " Prompt ", "Card\nOne", None),
    ("punctuation", "C++  Prompt", "Card\tOne", None),
    ("explicit_whitespace", "Prompt", "Card", " key\t "),
    ("explicit_empty", "中文", "Card", ""),
])
def test_exact_names_and_default_keys_match_independent_rust(tmp_path, scenario, field_name, template_name, key):
    field = Field(field_name, key=key, identity=True)
    note_type = (NoteType.custom("names", name="  Names  ").field(field)
        .template(Template(template_name, front="Question", back="Answer", target_deck=" Names:: Cards ",
            generate_when=GenerationRule.all([field.key]))))
    project = Project("Names", stable_id="native-names").add_notetype(note_type)
    project.add_note(Note("names").text(field_name, "内容").identity([field.key]))
    snapshot = project.notetypes["names"]
    assert snapshot.name == "  Names  "
    assert snapshot.fields[0] == field
    assert snapshot.templates[0].name == template_name
    assert snapshot.identity_value.field_keys == (field.key,)
    report = project.build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == observe("names", tmp_path / "rust.apkg", scenario)


def test_custom_templates_and_field_identity_match_rust(tmp_path):
    expected = observe("custom", tmp_path / "rust.apkg")
    note_type = (
        NoteType.custom("vocabulary", name="Vocabulary", css="\n.card {\n\tcolor: navy;\n}\n")
        .field(Field("Prompt", key="prompt", identity=True, sort=True))
        .field(Field("Answer", key="answer", required=True))
        .template(Template(
            "Forward", key="forward", front="\n{{Prompt}}\n", back="{{FrontSide}}<hr>{{Answer}}",
            browser_front="{{Prompt}}", browser_back="{{Answer}}", target_deck="Native::Forward",
            generate_when=GenerationRule.all(["prompt"]),
        ))
        .template(Template("Reverse", key="reverse", front="{{Answer}}", back="{{Prompt}}"))
    )
    project = Project("Native", stable_id="native-custom", default_deck="Native::Custom")
    project.add_notetype(note_type)
    note = Note("vocabulary").text("prompt", "<question>").html("answer", "<b>答案</b>").tag("vocab")
    project.add_note(note)
    # Both inputs remain editable, but the project owns the values at addition.
    note_type.css("changed")
    note.text("answer", "changed")
    report = project.build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == expected


def test_identity_precedence_matches_independent_rust(tmp_path):
    from anki_forge import IdentityRecipe

    expected = observe("identity", tmp_path / "rust.apkg")
    note_type = (NoteType.custom("identity", name="Identity")
        .field(Field("Prompt", key="prompt", identity=True))
        .field(Field("Answer", key="answer"))
        .template(Template("Card", key="card", front="{{Prompt}}", back="{{Answer}}"))
        .identity(IdentityRecipe.fields(["answer"])))
    project = Project("Identity", stable_id="native-identity").add_notetype(note_type)
    project.add_note(Note("identity").text("prompt", "p1").text("answer", "a1"))
    project.add_note(Note("identity").text("Prompt", "p2").text("Answer", "a2").identity(["prompt", "prompt"]))
    project.add_note(Note("identity", stable_id="explicit-3").text("prompt", "p3").text("answer", "a3").identity(["prompt"]))
    report = project.build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == expected


def test_cloze_and_image_occlusion_match_independent_rust(tmp_path):
    expected = observe("cloze_io", tmp_path / "rust.apkg")
    project = Project("Cloze and IO", stable_id="native-cloze-io")
    project.add_note(Note.cloze("中 {{c1::one}} and {{c2::two}}", back_extra="More & less").tag("stock"))
    project.add_notetype(NoteType.custom_cloze("custom-cloze", "body", name="Custom Cloze")
        .field(Field("Body", key="body", identity=True))
        .field(Field("Extra", key="extra", optional=True))
        .template(Template("Cloze Card", key="cloze-card", front="{{cloze:Body}}", back="{{cloze:Body}}<hr>{{Extra}}",
            browser_front="{{Body}}", generate_when=GenerationRule.cloze("body"))))
    project.add_note(Note("custom-cloze").html("Body", "{{c1::custom::hint}} and {{c3::third}}").text("extra", "extra"))
    source = tmp_path / "image.svg"
    source.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"></svg>')
    media = project.media.add_file(source, export_as="image.svg")
    project.add_note(Note.image_occlusion(media, stable_id="io-1").rect(0, 0, 10, 10)
        .rect(10, 10, 5, 5).header("Header").back_extra("Extra").comments("Comments").tag("io").build())
    report = project.build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == expected


def test_cross_project_media_uses_destination_filename_and_payload(tmp_path):
    expected = observe("media", tmp_path / "rust.apkg")
    source = Project("Source")
    reference = source.media.add_bytes(source_label="source", data=b"RIFF source", export_as="voice.wav")
    destination = Project("Destination", stable_id="native-media")
    destination.media.add_bytes(source_label="noise", data=b"RIFF noise", export_as="noise.wav")
    destination.add_note(Note.basic("Question", "Answer", stable_id="media-note").sound("back", reference))
    missing = destination.build()
    assert missing.status == "invalid"
    assert any(d.code == "MEDIA.MISSING_REFERENCE" for d in missing.diagnostics)
    local = destination.media.add_bytes(source_label="destination", data=b"RIFF destination", export_as="voice.wav")
    assert local == reference
    report = destination.build()
    report.ensure_success()
    assert observe("inspect", report.artifact.path) == expected


@pytest.mark.parametrize("mode", ["rename", "explicit", "reorder"])
def test_field_rename_and_reorder_follow_core_identity_rules(tmp_path, mode):
    def make_project(changed):
        prompt = Field("Renamed Prompt" if changed and mode != "reorder" else "Prompt", key="prompt", identity=True, sort=True)
        answer = Field("Answer", key="answer")
        name = prompt.name
        note_type = NoteType.custom("evolution", name="Evolution")
        for field in ([answer, prompt] if changed else [prompt, answer]):
            note_type.field(field)
        note_type.template(Template("Card", key="card", front="{{" + name + "}}", back="{{Answer}}"))
        return Project("Evolution", stable_id="native-evolution").add_notetype(note_type).add_note(
            Note("evolution", stable_id="existing" if mode == "explicit" else None).text("prompt", "Question").text("answer", "Answer")
        )

    before = make_project(False).write_apkg(tmp_path / "before.apkg")
    before.ensure_success()
    after = make_project(True).write_apkg(tmp_path / "after.apkg", compare_to=tmp_path / "before.apkg")
    after.ensure_success()
    actual = observe("inspect", after.artifact.path)
    assert actual == observe("evolution_" + mode, tmp_path / "rust-after.apkg")
    baseline = observe("inspect", before.artifact.path)
    same_guids = [n["anki_guid"] for n in actual["identity"]["notes"]] == [n["anki_guid"] for n in baseline["identity"]["notes"]]
    assert same_guids is (mode != "rename"), "core-derived identity includes the selected field display name"
