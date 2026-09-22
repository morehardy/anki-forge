import json
from pathlib import Path
import shutil
import subprocess
import sys

import anki_forge


def test_versions_describe_the_binding_core_and_embedded_contract():
    from anki_forge import versions

    metadata = versions()
    assert metadata.binding_version == anki_forge.__version__ == "0.2.0"
    assert metadata.core_version == "0.1.0"
    assert metadata.contract_version


def test_missing_extension_and_mixed_installations_fail_clearly(tmp_path):
    source = Path(anki_forge.__file__).parent
    package = tmp_path / "anki_forge"
    shutil.copytree(source, package, ignore=shutil.ignore_patterns("*.so", "*.pyd", "__pycache__"))
    script = f"import sys; sys.path.insert(0, {str(tmp_path)!r}); import anki_forge"
    missing = subprocess.run([sys.executable, "-I", "-c", script], capture_output=True, text=True)
    assert missing.returncode != 0
    assert "BINDING.EXTENSION_UNAVAILABLE" in missing.stderr
    assert "reinstall" in missing.stderr
    (package / "_native.py").write_text("def binding_metadata():\n    return " + repr(json.dumps({
        "binding_version": "0.1.0", "core_version": "0.1.0", "contract_version": "0.6.3",
    })) + "\n", encoding="utf-8")
    mismatch = subprocess.run([sys.executable, "-I", "-c", script], capture_output=True, text=True)
    assert mismatch.returncode != 0
    assert "BINDING.VERSION_MISMATCH" in mismatch.stderr


def test_all_public_module_imports_use_one_native_implementation():
    from anki_forge.project import Project
    from anki_forge.media import MediaRegistry, MediaRef

    assert Project is anki_forge.Project
    assert MediaRegistry is anki_forge.MediaRegistry
    assert MediaRef is anki_forge.MediaRef
