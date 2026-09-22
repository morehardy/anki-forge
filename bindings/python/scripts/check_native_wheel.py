"""Install a real wheel outside the checkout and run the native trial."""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import venv
import zipfile


SMOKE = '''
import gc
import importlib.util
from importlib.metadata import version
from pathlib import Path
from anki_forge import Note, Project
assert version("anki-forge") == "0.2.0"
assert importlib.util.find_spec("anki_forge_python") is None
root = Path.cwd()
project = Project("Installed", stable_id="installed")
project.add_note(Note.basic("<front>", "answer", stable_id="note-1"))
report = project.build()
report.ensure_success()
assert report.counts == {"notes": 1, "cards": 1, "media": 0}
artifact = report.artifact
path = artifact.path
del report
gc.collect()
assert path.is_file()
artifact.close()
assert not path.exists()
source = root / "音频 file.wav"
source.write_bytes(b"RIFF first")
ref = project.media.add_file(source, export_as="audio.wav")
project.add_note(Note.basic("Media", "").sound("back", ref))
source.write_bytes(b"RIFF changed")
report = project.build()
assert any(d.code == "MEDIA.SOURCE_CHANGED" for d in report.diagnostics)
assert report.artifact is None
print("Installed native wheel: Basic, media evidence and Artifact lifetime passed")
'''


def main() -> None:
    wheel = Path(sys.argv[1]).resolve()
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        assert any(name.endswith((".so", ".pyd")) for name in names), "missing native extension"
        assert not any("anki_forge_python/" in name or "anki_forge/_runtime/" in name for name in names)
    with tempfile.TemporaryDirectory(prefix="anki-forge-installed-") as directory:
        root = Path(directory)
        venv.EnvBuilder(with_pip=True).create(root / "venv")
        python = root / "venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        subprocess.run([str(python), "-m", "pip", "install", "--no-index", "--no-deps", str(wheel)], check=True)
        work = root / "安装 test"
        work.mkdir()
        environment = {key: value for key, value in os.environ.items() if not key.startswith(("PYTHON", "ANKI_FORGE"))}
        # Native operation must not find a repository CLI or compiler on PATH.
        environment["PATH"] = str(python.parent)
        subprocess.run([str(python), "-I", "-c", SMOKE], cwd=work, env=environment, check=True)


if __name__ == "__main__":
    main()
