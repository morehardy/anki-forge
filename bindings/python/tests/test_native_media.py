import pytest

from anki_forge import MediaError, MediaRegistry, Note, Project


def test_inline_media_limit_and_snapshot_are_enforced_by_registration():
    project = Project("Inline media")
    assert MediaRegistry.inline_limit_bytes() == 65536
    for data, code in [(b"", "MEDIA.EMPTY_SOURCE"), (b"x" * 65537, "MEDIA.INLINE_TOO_LARGE")]:
        with pytest.raises(MediaError) as caught:
            project.media.add_bytes(source_label="inline", data=data, export_as="sound.wav")
        assert caught.value.code == code
    original = b"RIFF" + b"x" * 65532
    mutable = bytearray(original)
    reference = project.media.add_bytes(source_label="inline", data=mutable, export_as="sound.wav")
    mutable[:] = b"changed"
    duplicate = project.media.add_bytes(source_label="another label", data=original, export_as="sound.wav")
    assert duplicate == reference
    project.add_note(Note.basic("Question", "Answer").sound("back", reference))
    report = project.build()
    report.ensure_success()
    assert report.counts["media"] == 1
    assert report.media["unique_bytes"] == 65536


def test_file_registration_errors_preserve_paths_and_do_not_reserve_names(tmp_path):
    project = Project("Registration", base_dir=tmp_path)
    empty = tmp_path / "empty.wav"
    empty.touch()
    for source, code in [(tmp_path / "missing.wav", "MEDIA.SOURCE_MISSING"),
                         (tmp_path, "MEDIA.SOURCE_NOT_REGULAR_FILE"),
                         (empty, "MEDIA.EMPTY_SOURCE")]:
        with pytest.raises(MediaError) as caught:
            project.media.add_file(source, export_as="sound.wav")
        assert caught.value.code == code
        assert caught.value.path == str(source)
    empty.write_bytes(b"RIFF valid")
    reference = project.media.add_file(empty, export_as="sound.wav")
    project.add_note(Note.basic("Question", "Answer").sound("back", reference))
    project.build().ensure_success()


@pytest.mark.parametrize("inline_first", [True, False])
def test_deduplication_preserves_the_first_registration_evidence(tmp_path, inline_first):
    project = Project("First registration")
    payload = b"RIFF same payload"
    path = tmp_path / "original.wav"
    path.write_bytes(payload)
    def inline():
        return project.media.add_bytes(source_label="inline", data=payload, export_as="sound.wav")
    def file():
        return project.media.add_file(path, export_as="sound.wav")
    first = inline() if inline_first else file()
    second = file() if inline_first else inline()
    assert first == second and hash(first) == hash(second)
    with pytest.raises(MediaError) as conflict:
        project.media.add_bytes(source_label="conflicting", data=b"different", export_as="sound.wav")
    assert conflict.value.code == "MEDIA.DUPLICATE_FILENAME_CONFLICT"
    path.unlink()
    project.add_note(Note.basic("Question", "Answer").sound("back", second))
    report = project.build()
    if inline_first:
        report.ensure_success()
        assert report.media["unique_bytes"] == len(payload)
    else:
        assert report.status == "invalid"
        assert any(d.code == "MEDIA.SOURCE_MISSING" for d in report.diagnostics)
