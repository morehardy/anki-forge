"""Install a real wheel outside the checkout and run the native trial."""
from __future__ import annotations

import os
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile
import venv
import zipfile
import shutil


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
    parser = argparse.ArgumentParser()
    parser.add_argument("wheel", type=Path)
    parser.add_argument("--observer", type=Path, help="run the full public suite with this independent Rust producer")
    args = parser.parse_args()
    wheel = args.wheel.resolve()
    source_root = Path(__file__).resolve().parents[1]
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        assert any(name.endswith((".so", ".pyd")) for name in names), "missing native extension"
        assert not any("anki_forge_python/" in name or "anki_forge/_runtime/" in name for name in names)
        assert "anki_forge/py.typed" in names
        assert "anki_forge/_native.pyi" in names
        assert not any(name.endswith(("/runtime.py", "/product_json.py", "/native_project.py", "/native_media.py")) for name in names)
        assert any(name.endswith("/LICENSE") for name in names)
        assert any(name.endswith("/THIRD_PARTY_NOTICES.md") for name in names)
    with tempfile.TemporaryDirectory(prefix="anki-forge-installed-") as directory:
        root = Path(directory)
        venv.EnvBuilder(with_pip=True).create(root / "venv")
        python = root / "venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        subprocess.run([str(python), "-m", "pip", "install", "--no-index", "--no-deps", str(wheel)], check=True)
        work = root / "安装 test"
        work.mkdir()
        environment = {key: value for key, value in os.environ.items() if not key.startswith(("PYTHON", "ANKI_FORGE", "MYPY"))}
        # Native operation must not find a repository CLI or compiler on PATH.
        environment["PATH"] = str(python.parent)
        subprocess.run([str(python), "-I", "-c", SMOKE], cwd=work, env=environment, check=True)
        example = work / "native_workflow.py"
        shutil.copyfile(source_root / "examples/native_workflow.py", example)
        subprocess.run([str(python), "-I", str(example), str(work / "example output")], cwd=work, env=environment, check=True)
        for filename in ("positive.py", "negative.py"):
            shutil.copyfile(source_root / "tests/typing" / filename, work / filename)
        command = [sys.executable, "-m", "mypy", "--strict", "--no-incremental", "--python-executable", str(python)]
        subprocess.run([*command, "positive.py"], cwd=work, env=environment, check=True)
        negative = subprocess.run([*command, "negative.py"], cwd=work, env=environment, text=True, capture_output=True)
        assert negative.returncode == 1, negative.stdout + negative.stderr
        assert negative.stdout.count("error:") == 6, negative.stdout
        assert "[attr-defined]" in negative.stdout, negative.stdout
        print("Installed wheel positive/negative consumer typing passed")
        if args.observer:
            observer = args.observer.resolve()
            assert observer.is_file()
            subprocess.run([str(python), "-m", "pip", "install", "pytest==9.1.1"], check=True)
            tests = work / "tests"
            tests.mkdir()
            for source in (source_root / "tests").glob("test_*.py"):
                if source.name.startswith("test_native_") or source.name in {"test_report.py", "test_product_api_model.py", "test_product_e2e.py", "test_product_validation_parity.py"}:
                    shutil.copyfile(source, tests / source.name)
            shutil.copytree(source_root / "tests/fixtures", tests / "fixtures")
            environment["ANKI_FORGE_PYTHON_OBSERVER"] = str(observer)
            subprocess.run([str(python), "-I", "-m", "pytest", str(tests), "-q"], cwd=work, env=environment, check=True)


if __name__ == "__main__":
    main()
