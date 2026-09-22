"""Regenerate the migration fixture with the actual pre-native Python source."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile

BASELINE_COMMIT = "51a44ad"


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    output = root / "bindings/python/tests/fixtures/python01"
    output.mkdir(parents=True, exist_ok=True)
    commit = subprocess.check_output(["git", "rev-parse", BASELINE_COMMIT], cwd=root, text=True).strip()
    paths = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", commit, "bindings/python/src"], cwd=root, text=True,
    ).splitlines()
    with tempfile.TemporaryDirectory(prefix="anki-forge-python01-") as directory:
        source = Path(directory)
        for path in paths:
            relative = Path(path).relative_to("bindings/python/src")
            destination = source / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=root))
        subprocess.run([sys.executable, "-I", "-c", CAPTURE, str(source), str(root), str(output)], check=True)
    (output / "provenance.json").write_text(json.dumps({
        "python_source_commit": commit,
        "python_version": "0.1.0",
        "core_source_commit": commit,
        "command": "python bindings/python/scripts/capture_01_baseline.py",
        "migration": "Preserve the implicit 0.1 field key back_extra and template key c_card explicitly in 0.2.",
    }, indent=2) + "\n", encoding="utf-8")


CAPTURE = '''
import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
from anki_forge import Field, Note, NoteType, Project, Template
from anki_forge.runtime import RuntimeOverride
root, output = Path(sys.argv[2]), Path(sys.argv[3])
runtime = RuntimeOverride(manifest=root / "contracts/manifest.yaml", executable=root / "target/release/contract_tools")
project = Project("Python migration", stable_id="python01-migration")
project.add_note(Note.basic("Basic front", "Basic answer", stable_id="basic-1").tag("baseline"))
note_type = (NoteType.custom("custom")
    .field(Field("Prompt", identity=True))
    .field(Field("Back  Extra"))
    .template(Template("C++ Card", front="{{Prompt}}", back="{{Back  Extra}}"))
    .template(Template("Reverse", front="{{Back  Extra}}", back="{{Prompt}}")))
project.add_notetype(note_type)
project.add_note(Note("custom").text("prompt", "Custom front").text("back_extra", "Custom answer").tag("baseline"))
project.write_apkg(output / "baseline.apkg", identity_lockfile=output / "baseline.lock.json",
                   write_identity_lockfile=True, runtime=runtime).ensure_success()
'''


if __name__ == "__main__":
    main()
