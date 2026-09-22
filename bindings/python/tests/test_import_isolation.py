import subprocess
import sys
import venv
from pathlib import Path

import pytest

PYTHON_ROOT = Path(__file__).resolve().parents[1]


class CleanVenv:
    def __init__(self, root: Path) -> None:
        self.root = root
        venv.EnvBuilder(with_pip=True).create(root)
        self.python = root / ("Scripts/python.exe" if sys.platform == "win32" else "bin/python")

    def run_python(self, code: str, *, check: bool = True) -> subprocess.CompletedProcess[str]:
        return subprocess.run([str(self.python), "-c", code], text=True, capture_output=True, check=check)

    def pip_install_wheel(self) -> None:
        wheels = sorted((PYTHON_ROOT / "dist").glob("anki_forge-*.whl"))
        assert wheels, "no wheel matched bindings/python/dist/anki_forge-*.whl"
        subprocess.run([str(self.python), "-m", "pip", "install", str(wheels[0])], check=True)


@pytest.fixture
def clean_venv(tmp_path: Path) -> CleanVenv:
    return CleanVenv(tmp_path / "venv")


def test_installed_public_wheel_does_not_expose_low_level_wrapper(clean_venv: CleanVenv):
    clean_venv.pip_install_wheel()
    clean_venv.run_python("import anki_forge")
    clean_venv.run_python(
        "import importlib.util; raise SystemExit(0 if importlib.util.find_spec('anki_forge_python') is None else 1)"
    )


def test_installed_wheel_embeds_native_core_without_runtime_files(clean_venv: CleanVenv):
    clean_venv.pip_install_wheel()
    clean_venv.run_python(
        """
from pathlib import Path
import anki_forge

root = Path(anki_forge.__file__).resolve().parent
assert not (root / "_runtime").exists()
assert not (root / "runtime.py").exists()
assert (root / "py.typed").is_file()
assert anki_forge.versions().binding_version == "0.2.0"
anki_forge.Project("Installed").add_note(anki_forge.Note.basic("Front", "Back")).build().ensure_success()
"""
    )
