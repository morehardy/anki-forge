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


def test_installed_wheel_runtime_contracts_match_manifest_assets(clean_venv: CleanVenv):
    clean_venv.pip_install_wheel()
    clean_venv.run_python(
        """
from pathlib import Path
import os
import subprocess
import json
import anki_forge

runtime_root = Path(anki_forge.__file__).resolve().parent / "_runtime"
manifest = runtime_root / "contracts" / "manifest.yaml"
executable = runtime_root / "bin" / ("contract_tools.exe" if os.name == "nt" else "contract_tools")
assert manifest.is_file(), manifest
assert executable.is_file(), executable
expected = {f"contracts/{relative}" for relative in json.loads(subprocess.check_output([str(executable), "package-runtime-assets", "--manifest", str(manifest)], text=True, encoding="utf-8"))}
actual = {path.relative_to(runtime_root).as_posix() for path in (runtime_root / "contracts").rglob("*") if path.is_file()}
missing = expected - actual
unexpected = {path for path in actual if path.startswith("contracts/")} - expected
assert not missing, sorted(missing)
assert not unexpected, sorted(unexpected)
"""
    )
