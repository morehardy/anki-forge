import base64
import json
import os
from pathlib import Path

import pytest

from anki_forge import Field, GenerationRule, Note, NoteType, Project, Template, ValidationError
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
