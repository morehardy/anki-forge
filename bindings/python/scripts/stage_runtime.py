from __future__ import annotations

import fnmatch
import json
import os
import shutil
import stat
import subprocess
import sys
import tomllib
from pathlib import Path

if sys.version_info < (3, 11):
    raise SystemExit("stage_runtime.py requires Python 3.11+")

PYTHON_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = PYTHON_ROOT.parents[1]
CONTRACTS_ROOT = REPO_ROOT / "contracts"
RUNTIME_ROOT = PYTHON_ROOT / "src" / "anki_forge" / "_runtime"


def runtime_executable() -> Path:
    executable_name = "contract_tools.exe" if os.name == "nt" else "contract_tools"
    cargo_target = os.environ.get("CARGO_BUILD_TARGET")
    target_dir = REPO_ROOT / "target"
    if cargo_target:
        target_dir = target_dir / cargo_target
    executable = target_dir / "release" / executable_name
    if not executable.is_file():
        raise SystemExit(f"release contract_tools binary missing: {executable}")
    return executable


def load_runtime_asset_list(executable: Path) -> list[str]:
    manifest = CONTRACTS_ROOT / "manifest.yaml"
    completed = subprocess.run(
        [str(executable), "package-runtime-assets", "--manifest", str(manifest)],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    try:
        assets = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"contract_tools emitted invalid runtime asset JSON: {error}") from error
    if not isinstance(assets, list) or not all(isinstance(asset, str) for asset in assets):
        raise SystemExit("contract_tools runtime asset list must be a JSON string array")
    return assets


def copy_runtime(executable: Path, assets: list[str]) -> None:
    if RUNTIME_ROOT.exists():
        shutil.rmtree(RUNTIME_ROOT)

    bin_dir = RUNTIME_ROOT / "bin"
    contracts_dir = RUNTIME_ROOT / "contracts"
    bin_dir.mkdir(parents=True)
    contracts_dir.mkdir(parents=True)

    staged_executable = bin_dir / executable.name
    shutil.copy2(executable, staged_executable)
    if os.name != "nt":
        staged_executable.chmod(staged_executable.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    for relative in assets:
        source = CONTRACTS_ROOT / relative
        if not source.is_file():
            raise SystemExit(f"runtime asset missing from contracts/: {relative}")
        destination = contracts_dir / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)


def validate_staged_assets(assets: list[str], executable: Path) -> None:
    expected_contracts = {f"contracts/{asset}" for asset in assets}
    actual_contracts = {
        path.relative_to(RUNTIME_ROOT).as_posix()
        for path in (RUNTIME_ROOT / "contracts").rglob("*")
        if path.is_file()
    }
    if actual_contracts != expected_contracts:
        missing = sorted(expected_contracts - actual_contracts)
        unexpected = sorted(actual_contracts - expected_contracts)
        raise SystemExit(f"staged runtime contract assets mismatch; missing={missing}, unexpected={unexpected}")

    staged_executable = RUNTIME_ROOT / "bin" / executable.name
    if not staged_executable.is_file():
        raise SystemExit(f"staged runtime executable missing: {staged_executable}")


def validate_package_data_globs() -> None:
    pyproject = tomllib.loads((PYTHON_ROOT / "pyproject.toml").read_text(encoding="utf-8"))
    package_data = (
        pyproject.get("tool", {})
        .get("setuptools", {})
        .get("package-data", {})
        .get("anki_forge", [])
    )
    if not isinstance(package_data, list) or not all(isinstance(glob, str) for glob in package_data):
        raise SystemExit("pyproject package-data for anki_forge must be a string array")

    package_root = PYTHON_ROOT / "src" / "anki_forge"
    staged_files = [
        path.relative_to(package_root).as_posix()
        for path in RUNTIME_ROOT.rglob("*")
        if path.is_file()
    ]
    uncovered = [
        path
        for path in staged_files
        if not any(fnmatch.fnmatchcase(path, glob) for glob in package_data)
    ]
    if uncovered:
        raise SystemExit(f"pyproject package-data globs do not cover staged runtime files: {uncovered}")


def main() -> None:
    executable = runtime_executable()
    assets = load_runtime_asset_list(executable)
    copy_runtime(executable, assets)
    validate_staged_assets(assets, executable)
    validate_package_data_globs()
    print(f"staged {len(assets)} contract assets and {executable.name} into {RUNTIME_ROOT}")


if __name__ == "__main__":
    main()
