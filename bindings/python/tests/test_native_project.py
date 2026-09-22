import gc

from anki_forge import Note, Project


def test_basic_build_owns_temporary_artifact():
    project = Project("Native", stable_id="native-basic")
    project.add_note(Note.basic("Front", "Back", stable_id="note-1"))

    report = project.build()
    report.ensure_success()
    assert report.counts == {"notes": 1, "cards": 1, "media": 0}
    artifact = report.artifact
    path = artifact.path
    assert path.is_file()

    del report
    gc.collect()
    assert path.is_file()
    artifact.close()
    assert not path.exists()


def test_media_change_is_rejected_and_project_can_build_after_restoring_source(tmp_path):
    source = tmp_path / "source.wav"
    source.write_bytes(b"RIFF original source")
    project = Project("Media")
    ref = project.media.add_file(source, export_as="source.wav")
    project.add_note(Note.basic("Front", "Back").sound("back", ref))
    source.write_bytes(b"RIFF changed source")

    report = project.build()

    assert report.status == "invalid"
    assert report.artifact is None
    assert any(d.code == "MEDIA.SOURCE_CHANGED" for d in report.diagnostics)
    source.write_bytes(b"RIFF original source")
    project.build().ensure_success()


def test_explicit_output_is_retained_after_report_release(tmp_path):
    project = Project("Saved", base_dir=tmp_path).add_note(Note.basic("Front", "Back"))
    report = project.write_apkg("saved.apkg")
    report.ensure_success()
    assert report.artifact.path == tmp_path / "saved.apkg"
    del report
    gc.collect()
    assert (tmp_path / "saved.apkg").is_file()
