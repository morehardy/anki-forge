"""Compare independent producers through the same APKG observer."""
import json
import os
from pathlib import Path
import subprocess

from anki_forge import Note, Project

ROOT = Path(__file__).resolve().parents[3]
OBSERVER = Path(os.environ.get("ANKI_FORGE_PYTHON_OBSERVER", str(
    ROOT / "target/debug/examples" / ("python_parity.exe" if os.name == "nt" else "python_parity")
)))


def observe(operation, path):
    assert OBSERVER.is_file(), "build the python_parity Rust example before running parity tests"
    value = json.loads(subprocess.check_output([str(OBSERVER), operation, str(path)], text=True))
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
