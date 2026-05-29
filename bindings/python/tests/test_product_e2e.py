import base64
import json
import os
from pathlib import Path

import pytest

from anki_forge import DiagnosticsError, Field, Note, NoteType, Project, Template, ValidationError


def find_repo_root() -> Path:
    for parent in Path(__file__).resolve().parents:
        if (parent / "contracts" / "manifest.yaml").is_file():
            return parent
    raise RuntimeError("contracts/manifest.yaml not found from test file parents")


def release_contract_tools_exists() -> bool:
    try:
        root = find_repo_root()
    except RuntimeError:
        return False
    executable = "contract_tools.exe" if os.name == "nt" else "contract_tools"
    return (root / "target" / "release" / executable).is_file()


@pytest.fixture(autouse=True)
def require_release_contract_tools_binary():
    if not release_contract_tools_exists():
        pytest.fail(
            "release contract_tools binary missing; run `cargo build -p contract_tools --release` before product E2E tests"
        )


def minimal_wav_bytes() -> bytes:
    return base64.b64decode("UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=")


def test_python_basic_project_writes_apkg(tmp_path):
    project = Project("Deck")
    project.add_note(Note.basic("Front", "Back"))
    report = project.write_apkg(tmp_path / "deck.apkg")
    report.ensure_success()
    assert (tmp_path / "deck.apkg").is_file()


def test_python_custom_media_project_writes_apkg(tmp_path):
    audio = tmp_path / "hello.wav"
    audio.write_bytes(minimal_wav_bytes())
    project = Project("Deck")
    ref = project.media.add_file(audio, export_as="hello.wav")
    nt = (
        NoteType.custom("audio")
        .field(Field("Audio", key="audio", required=True))
        .template(Template("Card", front="{{Audio}}", back="{{Audio}}"))
    )
    project.add_notetype(nt)
    project.add_note(Note("audio", stable_id="audio:hello").sound("audio", ref))
    report = project.write_apkg(tmp_path / "media.apkg")
    report.ensure_success()
    assert (tmp_path / "media.apkg").is_file()


def test_missing_media_file_returns_structured_diagnostics(tmp_path):
    missing = tmp_path / "missing.wav"
    project = Project("Deck")
    ref = project.media.add_file(missing, export_as="missing.wav")
    nt = (
        NoteType.custom("audio")
        .field(Field("Audio", key="audio", required=True))
        .template(Template("Card", front="{{Audio}}", back="{{Audio}}"))
    )
    project.add_notetype(nt)
    project.add_note(Note("audio", stable_id="audio:missing").sound("audio", ref))
    report = project.write_apkg(tmp_path / "missing-media.apkg")
    with pytest.raises(DiagnosticsError):
        report.ensure_success()
    assert any(diagnostic.code == "MEDIA.SOURCE_MISSING" for diagnostic in report.diagnostics)


def test_cloze_note_without_cloze_marker_returns_structured_diagnostics(tmp_path):
    project = Project("Deck")
    project.add_note(Note.cloze("plain text without cloze", stable_id="cloze:plain"))
    report = project.write_apkg(tmp_path / "no-cloze.apkg")
    with pytest.raises(DiagnosticsError):
        report.ensure_success()
    assert any(diagnostic.code == "PRODUCT.CLOZE_MARKER_MISSING" for diagnostic in report.diagnostics)


def test_cloze_note_without_cloze_marker_writes_report_json(tmp_path):
    report_json = tmp_path / "report.json"
    project = Project("Deck")
    project.add_note(Note.cloze("plain text without cloze", stable_id="cloze:plain"))
    report = project.write_apkg(tmp_path / "no-cloze.apkg", report_json=report_json)

    assert report.status == "invalid"
    assert report_json.is_file()
    payload = json.loads(report_json.read_text(encoding="utf-8"))
    assert payload["status"] == "invalid"
    assert any(diagnostic["code"] == "PRODUCT.CLOZE_MARKER_MISSING" for diagnostic in payload["diagnostics"])
