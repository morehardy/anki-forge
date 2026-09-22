from concurrent.futures import ThreadPoolExecutor
import os
import gc
import time

import pytest

from anki_forge import Deck, Note, Project


@pytest.mark.parametrize("kind", ["project", "deck"])
def test_busy_object_rejects_concurrent_edits_but_independent_objects_work(kind):
    value = Project("Concurrent", stable_id="concurrent") if kind == "project" else Deck("Concurrent", stable_id="concurrent")
    for index in range(5000):
        if kind == "project":
            value.add_note(Note.basic(str(index), "Answer", stable_id=str(index)))
        else:
            value.add_basic(str(index), "Answer", stable_id=str(index))
    busy = f"BINDING.{kind.upper()}_BUSY"
    with ThreadPoolExecutor(max_workers=1) as pool:
        future = pool.submit(value.build)
        deadline = time.monotonic() + 20
        while True:
            try:
                value.validate()
            except RuntimeError as error:
                assert busy in str(error)
                break
            assert not future.done(), "build did not expose a GIL-free busy operation"
            assert time.monotonic() < deadline
        with pytest.raises(RuntimeError, match=busy):
            if kind == "project":
                value.add_note(Note.basic("concurrent", "rejected"))
            else:
                value.add_basic("concurrent", "rejected")
        Project("Independent").add_note(Note.basic("other", "state")).build().ensure_success()
        with future.result(timeout=30) as report:
            report.ensure_success()
            assert report.counts["notes"] == 5000
    value.validate().ensure_success()


@pytest.mark.skipif(not hasattr(os, "fork"), reason="os.fork is unavailable on this platform")
def test_inherited_project_and_deck_reject_use_before_acquiring_mutex():
    project = Project("Parent")
    deck = Deck("Parent")
    read_fd, write_fd = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(read_fd)
        try:
            for value in (project, deck):
                try:
                    value.validate()
                except RuntimeError as error:
                    assert "BINDING.FORKED_OBJECT" in str(error)
                else:
                    raise AssertionError("inherited object accepted")
            os.write(write_fd, b"ok")
        except BaseException as error:
            os.write(write_fd, repr(error).encode())
        finally:
            os._exit(0)
    os.close(write_fd)
    try:
        assert os.read(read_fd, 1024) == b"ok"
        assert os.waitpid(pid, 0)[1] == 0
    finally:
        os.close(read_fd)
    project.add_note(Note.basic("still", "usable")).build().ensure_success()
    deck.add_basic("still", "usable").build().ensure_success()


@pytest.mark.skipif(not hasattr(os, "fork"), reason="os.fork is unavailable on this platform")
def test_child_cannot_close_or_collect_parent_artifact():
    report = Project("Parent").add_note(Note.basic("temporary", "owner")).build()
    report.ensure_success()
    artifact = report.artifact
    path = artifact.path
    report.close()
    read_fd, write_fd = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(read_fd)
        try:
            try:
                artifact.close()
            except RuntimeError as error:
                assert "BINDING.FORKED_OBJECT" in str(error)
            else:
                raise AssertionError("inherited artifact accepted close")
            del artifact
            gc.collect()
            assert path.is_file()
            os.write(write_fd, b"ok")
        except BaseException as error:
            os.write(write_fd, repr(error).encode())
        finally:
            os._exit(0)
    os.close(write_fd)
    try:
        message = os.read(read_fd, 1024)
        assert os.waitpid(pid, 0)[1] == 0
        assert message == b"ok"
        assert path.is_file()
    finally:
        os.close(read_fd)
        artifact.close()
    assert not path.exists()
