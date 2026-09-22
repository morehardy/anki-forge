import base64
import pytest
from anki_forge import Note, Project
from test_native_parity import observe

# A complete 1x1 PNG; no image decoding is implemented by Python.
PNG = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==")


def make_deck(tmp_path):
    from anki_forge import BasicIdentityOverride, Deck

    deck = Deck("Native Deck", stable_id="native-deck", basic_identity=["front"], base_dir=tmp_path)
    deck.add_basic("<b>Front</b>", "Answer", tags=["basic"])
    deck.add_basic("Second", "<i>Identity answer</i>", identity_override=BasicIdentityOverride(["back"], "test-answer-key"))
    deck.add_cloze("{{c1::one}} and {{c2::two}}", extra="Extra", tags=["cloze"])
    image = deck.media.add_bytes("image.png", PNG)
    deck.add_image_occlusion(image, rects=[(0, 0, 1, 1)], header="Header", back_extra="Extra", comments="Comments", tags=["io"])
    return deck


def test_deck_stock_notes_and_inferred_io_identity_match_rust(tmp_path):
    deck = make_deck(tmp_path)
    image = tmp_path / "image.png"
    image.write_bytes(PNG)
    report = deck.build()
    report.ensure_success()
    assert report.counts == {"notes": 4, "cards": 5, "media": 1}
    assert observe("inspect", report.artifact.path) == observe("deck", tmp_path / "rust.apkg", image)
    assert any(d.code == "DECK.NOTE_LEVEL_IDENTITY_OVERRIDE_USED" for d in deck.validate().diagnostics)


def test_from_deck_is_an_editable_snapshot_preserving_identities_and_media(tmp_path):
    from anki_forge import Field, NoteType, Template

    deck = make_deck(tmp_path)
    original = deck.build()
    original.ensure_success()
    project = Project.from_deck(deck)
    assert project.base_dir == tmp_path
    converted = project.build()
    converted.ensure_success()
    assert observe("inspect", converted.artifact.path) == observe("inspect", original.artifact.path)
    project.add_notetype(NoteType.custom("extra")
        .field(Field("Question", key="q", identity=True))
        .template(Template("Card", key="card", front="{{Question}}", back="{{FrontSide}}")))
    project.add_note(Note("extra").text("q", "custom"))
    deck.add_basic("later", "deck only", stable_id="later")
    appended = project.build()
    appended.ensure_success()
    assert appended.counts["notes"] == 5
    image = tmp_path / "image.png"
    image.write_bytes(PNG)
    assert observe("inspect", appended.artifact.path) == observe("deck_project", tmp_path / "rust-converted.apkg", image)
    assert deck.build().counts["notes"] == 5
    assert len(project.notes) == 5
    assert all(note.stable_id != "later" for note in project.notes)


def test_deck_errors_are_atomic_and_io_bounds_come_from_core(tmp_path):
    from anki_forge import BasicIdentityOverride, Deck, DeckError

    deck = Deck("Errors", stable_id="errors", base_dir=tmp_path)
    image = deck.media.add_bytes("image.png", PNG)
    with pytest.raises(DeckError) as caught:
        deck.add_image_occlusion(image, rects=[(0, 0, 2, 1)])
    assert caught.value.code == "AFID.IO_RECT_OUT_OF_BOUNDS"
    deck.add_image_occlusion(image, rects=[(0, 0, 1, 1)])
    with pytest.raises(DeckError) as caught:
        deck.add_basic("Front", "Back", identity_override=BasicIdentityOverride(["front"], " "))
    assert "REASON" in caught.value.code
    deck.add_basic("Front", "Back", stable_id="one")
    with pytest.raises(DeckError) as caught:
        deck.add_basic("Other", "Back", stable_id="one")
    assert "DUPLICATE" in caught.value.code
    result = deck.build()
    result.ensure_success()
    assert result.counts["notes"] == 2


def test_deck_file_evidence_survives_project_conversion(tmp_path):
    from anki_forge import Deck

    source = tmp_path / "image.png"
    source.write_bytes(PNG)
    deck = Deck("File", stable_id="file", base_dir=tmp_path)
    reference = deck.media.add_file("image.png")
    assert deck.media.get("image.png") == reference
    assert deck.media.get("unknown") is None
    deck.add_image_occlusion(reference, rects=[(0, 0, 1, 1)])
    project = Project.from_deck(deck)
    source.write_bytes(b"changed")
    for value in (deck, project):
        assert any(d.code == "MEDIA.SOURCE_CHANGED" for d in value.build().diagnostics)
    source.write_bytes(PNG)
    deck.build().ensure_success()
    project.build().ensure_success()


def test_deck_grouped_io_preserves_shared_core_limitation(tmp_path):
    from anki_forge import Deck

    deck = Deck("Grouped", stable_id="grouped", base_dir=tmp_path)
    image = deck.media.add_bytes("image.png", PNG)
    deck.add_image_occlusion(image, rects=[(0, 0, 1, 1)], mode="hide_one_guess_one")
    report = deck.build()
    assert report.status == "invalid"
    assert any(d.code == "PRODUCT.CLOZE_MARKER_MALFORMED" for d in report.diagnostics)
