import random
import pytest

from anki_forge import Note, Project
from test_native_parity import observe


def test_bytes_output_matches_rust_and_releases_temporary_artifact(tmp_path, monkeypatch):
    temporary = tmp_path / "native temporary"
    temporary.mkdir()
    for name in ("TMPDIR", "TEMP", "TMP"):
        monkeypatch.setenv(name, str(temporary))
    project = Project("Native", stable_id="native-basic").add_note(Note.basic("Front", "Back", stable_id="note-1"))
    with project.build() as control:
        control.ensure_success()
        assert control.artifact.path.parent == temporary
    assert list(temporary.iterdir()) == []
    data = project.to_apkg_bytes()
    assert isinstance(data, bytes)
    assert list(temporary.iterdir()) == []
    output = tmp_path / "bytes.apkg"
    output.write_bytes(data)
    assert observe("inspect", output) == observe("basic", tmp_path / "rust.apkg")


def test_file_object_output_handles_short_writes_in_bounded_chunks_without_project_lock(tmp_path, monkeypatch):
    temporary = tmp_path / "native temporary"
    temporary.mkdir()
    for name in ("TMPDIR", "TEMP", "TMP"):
        monkeypatch.setenv(name, str(temporary))
    source = tmp_path / "large.bin"
    source.write_bytes(random.Random(765).randbytes(512 * 1024))
    expected = observe("large", tmp_path / "rust.apkg", source)
    project = Project("Large", stable_id="native-large")
    media = project.media.add_file(source, export_as="large.bin")
    project.add_note(Note.basic("Question", "Answer", stable_id="one").sound("back", media))

    class ShortSink:
        closed = False
        def __init__(self):
            self.data = bytearray()
            self.requests = []
        def write(self, data):
            if not self.requests:
                project.add_note(Note.basic("Callback addition", "Answer", stable_id="two"))
            self.requests.append(len(data))
            written = min(7001, len(data))
            self.data.extend(data[:written])
            return written
        def close(self):
            self.closed = True

    sink = ShortSink()
    written = project.write_to(sink)
    assert not sink.closed
    assert len(project.notes) == 2
    assert written == len(sink.data) > 512 * 1024
    assert max(sink.requests) == 65536
    assert list(temporary.iterdir()) == []
    output = tmp_path / "sink.apkg"
    output.write_bytes(sink.data)
    assert observe("inspect", output) == expected


@pytest.mark.parametrize("result", [0, None, -1, True, 2**30, "invalid"])
def test_writer_with_no_progress_or_invalid_count_leaves_sink_open_and_cleans_artifact(tmp_path, monkeypatch, result):
    temporary = tmp_path / "native temporary"
    temporary.mkdir()
    for name in ("TMPDIR", "TEMP", "TMP"):
        monkeypatch.setenv(name, str(temporary))
    class Sink:
        closed = False
        def write(self, data):
            return result
        def close(self):
            self.closed = True
    sink = Sink()
    with pytest.raises(OSError):
        Project("Bad sink").add_note(Note.basic("Question", "Answer")).write_to(sink)
    assert not sink.closed
    assert list(temporary.iterdir()) == []


def test_writer_exception_is_preserved_and_cleans_artifact(tmp_path, monkeypatch):
    temporary = tmp_path / "native temporary"
    temporary.mkdir()
    for name in ("TMPDIR", "TEMP", "TMP"):
        monkeypatch.setenv(name, str(temporary))
    failure = RuntimeError("caller sink failed")
    class Sink:
        closed = False
        def write(self, data):
            raise failure
        def close(self):
            self.closed = True
    sink = Sink()
    with pytest.raises(RuntimeError) as caught:
        Project("Failing sink").add_note(Note.basic("Question", "Answer")).write_to(sink)
    assert caught.value is failure
    assert not sink.closed
    assert list(temporary.iterdir()) == []
