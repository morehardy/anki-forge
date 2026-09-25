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
from anki_forge import Note, Project, Media, BuildOptions
assert version("anki-forge") == "0.2.0"
assert importlib.util.find_spec("anki_forge_python") is None
project = Project("installed").add("note-1", Note.basic("<front>", "answer"))
output = project.build(BuildOptions.temporary())
assert output.report.counts.notes == 1
artifact, snapshot = output.artifact, output.snapshot()
path = artifact.path
del output
gc.collect()
assert path.is_file()
artifact.close()
assert not path.exists()
source = Path("theme.css")
source.write_bytes(b".card { color: navy; }")
media = Media.file(source).with_export_name("theme.css")
source.unlink()
project.add_asset(media)
assert project.build(BuildOptions.temporary()).report.counts.media == 1
print("Installed native wheel: explicit keys, media snapshot and artifact lifetime passed")
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
        # Isolated mode ignores PYTHONUTF8; request UTF-8 explicitly so printing
        # the Unicode installation path also works with Windows redirected stdout.
        isolated_python = [str(python), "-I", "-X", "utf8"]
        subprocess.run([*isolated_python, "-c", SMOKE], cwd=work, env=environment, check=True)
        example = work / "native_workflow.py"
        shutil.copyfile(source_root / "examples/native_workflow.py", example)
        subprocess.run([*isolated_python, str(example), str(work / "example output")], cwd=work, env=environment, check=True)
        for filename in ("positive.py", "negative.py"):
            shutil.copyfile(source_root / "tests/typing" / filename, work / filename)
        command = [sys.executable, "-m", "mypy", "--strict", "--no-incremental", "--python-executable", str(python)]
        subprocess.run([*command, "positive.py"], cwd=work, env=environment, check=True)
        negative = subprocess.run([*command, "negative.py"], cwd=work, env=environment, text=True, capture_output=True)
        assert negative.returncode == 1, negative.stdout + negative.stderr
        assert negative.stdout.count("error:") == 7, negative.stdout
        assert "[attr-defined]" in negative.stdout, negative.stdout
        print("Installed wheel positive/negative consumer typing passed")
        if not args.observer:
            probe = work / "media_budget_probe.py"
            shutil.copyfile(source_root / "tests/media_budget_probe.py", probe)
            for budget in ("small", "default"):
                subprocess.run([*isolated_python, str(probe), budget], cwd=work, env=environment, check=True)
        if args.observer:
            observer = args.observer.resolve()
            assert observer.is_file()
            subprocess.run([str(python), "-m", "pip", "install", "pytest==9.1.1"], check=True)
            tests = work / "tests"
            tests.mkdir()
            shutil.copyfile(source_root / "tests/media_budget_probe.py", tests / "media_budget_probe.py")
            for source in (source_root / "tests").glob("test_*.py"):
                if source.name in {"test_public_api.py", "test_fork_ownership.py"}:
                    shutil.copyfile(source, tests / source.name)
            environment["ANKI_FORGE_PYTHON_OBSERVER"] = str(observer)
            subprocess.run([*isolated_python, "-m", "pytest", str(tests), "-q"], cwd=work, env=environment, check=True)


if __name__ == "__main__":
    main()
