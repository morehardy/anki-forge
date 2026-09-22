import base64
import json
import os
from pathlib import Path

import pytest

from anki_forge import DiagnosticsError, Field, Note, NoteType, Project, Template, ValidationError
from anki_forge.report import BuildReport


def minimal_wav_bytes() -> bytes:
    return base64.b64decode("UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=")


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\t"])
def test_multiline_templates_and_css_preserve_source_and_build(tmp_path, newline):
    front = f" {newline}<section>{{{{Front}}}}</section>{newline} "
    back = f"{newline}{{{{FrontSide}}}}{newline} "
    css = f"{newline}.card {{{newline}  color: navy;{newline}}}{newline}"
    note_type = (
        NoteType.custom("source", css=css)
        .field(Field("Front", identity=True))
        .template(Template("Card", front=front, back=back, browser_front=front, browser_back=back))
    )
    project = Project("Source").add_notetype(note_type).add_note(Note("source").text("front", "hello"))

    exported_type = project.notetypes["source"]
    assert exported_type.css_value == css
    template = exported_type.templates[0]
    assert (template.front, template.back, template.browser_front, template.browser_back) == (front, back, front, back)
    project.write_apkg(tmp_path / "source.apkg").ensure_success()


def test_python_basic_project_writes_apkg(tmp_path):
    project = Project("Deck")
    project.add_note(Note.basic("Front", "Back"))
    report = project.write_apkg(tmp_path / "deck.apkg")
    report.ensure_success()
    assert (tmp_path / "deck.apkg").is_file()


def test_python_hard_link_baseline_alias_is_rejected_by_rust_runtime(tmp_path):
    project = Project("Deck", stable_id="baseline-guard")
    project.add_note(Note.basic("Front", "Back", stable_id="note-1"))
    baseline = tmp_path / "previous.apkg"
    project.write_apkg(baseline).ensure_success()
    original = baseline.read_bytes()
    alias = tmp_path / "alias.apkg"
    os.link(baseline, alias)

    report = project.write_apkg(alias, compare_to=baseline)

    assert report.status == "invalid"
    assert report.artifact is None
    assert any(diagnostic.code == "PROJECT.PATH_COLLISION" for diagnostic in report.diagnostics)
    assert baseline.read_bytes() == original
    assert alias.read_bytes() == original


def test_python_policy_block_preserves_output_and_lockfile(tmp_path):
    baseline = tmp_path / "previous.apkg"
    lockfile = tmp_path / "identity.json"
    output = tmp_path / "output.apkg"
    report_path = tmp_path / "report.json"
    project = Project("Deck", stable_id="baseline-guard")
    project.add_note(Note.basic("Front", "Back", stable_id="note-1"))
    project.write_apkg(
        baseline, identity_lockfile=lockfile, write_identity_lockfile=True
    ).ensure_success()
    original = baseline.read_bytes()
    original_lockfile = lockfile.read_bytes()
    output.write_bytes(b"previous publication")
    changed = Project("Deck", stable_id="baseline-guard")
    changed.add_note(Note.basic("Front", "Changed", stable_id="note-1"))

    report = changed.write_apkg(
        output,
        compare_to=baseline,
        fail_on="low",
        identity_lockfile=lockfile,
        write_identity_lockfile=True,
        report_json=report_path,
    )

    assert report.status == "blocked"
    assert report.artifact is None
    assert report.diff["artifact_diff"]["changes"]
    assert report.risk["findings"]
    assert report.update_safety["lockfile_written"] is False
    with pytest.raises(DiagnosticsError):
        report.ensure_success()
    assert baseline.read_bytes() == original
    assert output.read_bytes() == b"previous publication"
    assert lockfile.read_bytes() == original_lockfile
    persisted = json.loads(report_path.read_text(encoding="utf-8"))
    assert persisted["status"] == "blocked"
    assert persisted["artifact"] is None


@pytest.mark.parametrize("unreadable", [False, True])
def test_lockfile_only_risk_threshold_is_decided_by_core(tmp_path, unreadable):
    lockfile = tmp_path / "identity.json"
    project = Project("Deck", stable_id="lock-only").add_note(Note.basic("Front", "Back", stable_id="note-1"))
    project.write_apkg(tmp_path / "first.apkg", identity_lockfile=lockfile, write_identity_lockfile=True).ensure_success()
    if unreadable:
        lockfile.write_text("broken", encoding="utf-8")
    original = lockfile.read_bytes()
    output = tmp_path / "next.apkg"
    output.write_bytes(b"previous publication")

    report = project.write_apkg(output, identity_lockfile=lockfile, fail_on="high", update_safety="report_only")

    assert report.status == ("blocked" if unreadable else "success")
    assert report.policy["status"] == ("blocked" if unreadable else "passed")
    assert report.policy["threshold"] == "high"
    if unreadable:
        assert "RISK.BASELINE_UNAVAILABLE" in report.policy["blocking_findings"]
        assert output.read_bytes() == b"previous publication"
    else:
        report.ensure_success()
        assert output.read_bytes() != b"previous publication"
    assert lockfile.read_bytes() == original


def test_python_image_occlusion_runtime_build(tmp_path):
    project = Project("IO")
    image = project.media.add_bytes(source_label="heart.png", data=b"heart", export_as="heart.png")
    project.add_note(
        Note.image_occlusion(image, stable_id="io:1")
        .rect(0, 0, 10, 10)
        .header("Heart")
        .back_extra("Identify it")
        .build()
    )

    report = project.write_apkg(tmp_path / "io.apkg")

    report.ensure_success()
    assert report.counts["notes"] == 1
    assert report.counts["cards"] == 1
    assert report.counts["media"] == 1
    assert (tmp_path / "io.apkg").is_file()


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
    from anki_forge import MediaError
    project = Project("Deck")
    with pytest.raises(MediaError) as caught:
        project.media.add_file(tmp_path / "missing.wav")
    assert caught.value.code == "MEDIA.SOURCE_MISSING"
    assert caught.value.path == str(tmp_path / "missing.wav")


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
    assert isinstance(payload["tool_version"], str)
    assert payload["tool_version"]
    assert any(diagnostic["code"] == "PRODUCT.CLOZE_MARKER_MISSING" for diagnostic in payload["diagnostics"])
    parsed = BuildReport.from_json(payload)
    assert parsed.status == "invalid"
