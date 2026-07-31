import base64
import json
import os
from pathlib import Path

import pytest

from anki_forge import Field, GenerationRule, MediaRef, Note, NoteType, Project, Template, ValidationError
from anki_forge.product_json import basic_stock_notetype_json, cloze_stock_notetype_json


def find_repo_root() -> Path:
    env_root = os.environ.get("ANKI_FORGE_REPO_ROOT")
    if env_root:
        root = Path(env_root)
        if (root / "contracts" / "manifest.yaml").is_file():
            return root
    for parent in Path(__file__).resolve().parents:
        if (parent / "contracts" / "manifest.yaml").is_file():
            return parent
    raise RuntimeError("contracts/manifest.yaml not found from test file parents")


REPO_ROOT = find_repo_root()
INLINE_WAV_BYTES = base64.b64decode("UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=")


def test_stock_notetype_helpers_match_rust_fixtures():
    basic_expected = json.loads(
        (REPO_ROOT / "contracts/fixtures/product-v2/basic-stock.json").read_text(encoding="utf-8")
    )["note_types"][0]
    cloze_expected = json.loads(
        (REPO_ROOT / "contracts/fixtures/product-v2/stock-order-cloze-before-basic.json").read_text(encoding="utf-8")
    )["note_types"][0]
    assert basic_stock_notetype_json() == basic_expected
    assert cloze_stock_notetype_json() == cloze_expected


def test_basic_project_matches_rust_fixture():
    project = Project("Demo", stable_id="basic-demo", default_deck="Demo")
    project.add_note(Note.basic("Hello", "World", stable_id="basic:hello").tag("demo"))
    expected = json.loads((REPO_ROOT / "contracts/fixtures/product-v2/basic-stock.json").read_text(encoding="utf-8"))
    assert project.to_product_document() == expected


def test_document_id_defaults_to_project_name_without_slugging():
    project = Project("Japanese::Core")
    project.add_note(Note.basic("Front", "Back"))
    assert project.to_product_document()["document_id"] == "Japanese::Core"


def test_custom_cloze_project_serializes_product_v3_note_type_kind():
    note_type = (
        NoteType.custom_cloze("language-cloze", "text")
        .field(Field("Sentence", key="text", identity=True, sort=True))
        .field(Field("Extra", key="extra"))
        .template(
            Template(
                "Cloze",
                key="cloze",
                front="{{cloze:Sentence}}",
                back="{{cloze:Sentence}}<br>{{Extra}}",
            )
        )
    )
    project = (
        Project("Custom Cloze", stable_id="custom-cloze")
        .add_notetype(note_type)
        .add_note(
            Note("language-cloze", stable_id="custom:1")
            .text("text", "{{c1::Madrid}} is in {{c2::Spain}}")
            .text("extra", "geography")
        )
    )

    document = project.to_product_document()

    assert document["product_document_version"] == "product-v3"
    assert document["note_types"][0]["note_type_kind"] == "cloze"
    assert document["note_types"][0]["cloze_field"] == "text"


def test_custom_template_serializes_browser_templates_and_target_deck():
    note_type = (
        NoteType.custom("portable")
        .field(Field("Front", key="front", identity=True))
        .template(
            Template(
                "Card",
                "{{Front}}",
                "{{FrontSide}}",
                key="card",
                browser_front="{{text:Front}}",
                browser_back="{{Front}}",
                target_deck="Languages::Spanish",
            )
        )
    )
    document = Project("Portable").add_notetype(note_type).to_product_document()
    template = document["note_types"][0]["templates"][0]

    assert template["browser_front"] == "{{text:Front}}"
    assert template["browser_back"] == "{{Front}}"
    assert template["target_deck"] == "Languages::Spanish"


def test_custom_identity_missing_stable_id_fast_fails():
    nt = NoteType.custom("custom").field(Field("Front", key="front"))
    project = Project("Deck").add_notetype(nt).add_note(Note("custom").text("front", "x"))
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_unknown_internal_note_type_fast_fails_before_serialization():
    project = Project("Deck")
    project._notes.append(Note("missing").text("front", "x"))
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_mutated_stock_note_fields_fast_fail_before_serialization():
    note = Note.basic("Front", "Back")
    note.fields["extra"] = note.fields["front"]
    project = Project("Deck").add_note(note)
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_unknown_media_reference_fast_fails_before_serialization():
    nt = (
        NoteType.custom("audio-card")
        .field(Field("Prompt", key="prompt", identity=True))
        .field(Field("Audio", key="audio"))
    )
    note = Note("audio-card").text("prompt", "hello").sound("audio", MediaRef("media:999999", "missing.wav"))
    project = Project("Deck").add_notetype(nt).add_note(note)
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_duplicate_stable_ids_fast_fail_before_serialization():
    project = Project("Deck")
    project.add_note(Note.basic("A", "Back", stable_id="duplicate"))
    project.add_note(Note.basic("B", "Back", stable_id="duplicate"))
    with pytest.raises(ValidationError):
        project.to_product_document()


def test_custom_media_project_serializes_typed_content():
    project = Project("Media", stable_id="custom-media-demo", default_deck="Media")
    ref = project.media.add_bytes(source_label="hello.wav", data=INLINE_WAV_BYTES, export_as="hello.wav")
    nt = (
        NoteType.custom("media-card", name="Media Card")
        .field(Field("Prompt", key="prompt", identity=True, sort=True, required=True))
        .field(Field("Answer", key="answer"))
        .field(Field("Audio", key="audio"))
        .template(
            Template(
                "Card",
                key="card",
                front="{{Prompt}} {{Audio}}",
                back='{{FrontSide}}<hr id="answer">{{Answer}}',
                generate_when=GenerationRule.all(["prompt"]),
            )
        )
    )
    project.add_notetype(nt)
    project.add_note(Note("media-card").text("prompt", "hello").html("answer", "<b>world</b>").sound("audio", ref))
    expected = json.loads((REPO_ROOT / "contracts/fixtures/product-v2/custom-typed-media.json").read_text(encoding="utf-8"))
    assert project.to_product_document() == expected


def test_project_declares_image_occlusion_stock_notetype():
    project = Project("IO", stable_id="io", default_deck="IO")
    image = project.media.add_bytes(source_label="heart.png", data=b"heart", export_as="heart.png")
    project.add_note(
        Note.image_occlusion(image, stable_id="io:1")
        .rect(0, 0, 10, 10)
        .build()
    )

    document = project.to_product_document()

    assert any(
        note_type["kind"] == "stock" and note_type["id"] == "image_occlusion"
        for note_type in document["note_types"]
    )
    note = document["notes"][0]
    assert note["kind"] == "stock"
    assert note["note_type_id"] == "image_occlusion"
    assert note["fields"]["image"] == {
        "kind": "image",
        "media_id": image.media_id,
    }
