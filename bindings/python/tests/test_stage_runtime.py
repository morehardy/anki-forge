from __future__ import annotations

import importlib.util
from pathlib import Path


def load_stage_runtime():
    script = Path(__file__).resolve().parents[1] / "scripts" / "stage_runtime.py"
    spec = importlib.util.spec_from_file_location("stage_runtime_for_test", script)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def touch_runtime_binary(root: Path, *parts: str) -> Path:
    executable = root.joinpath("target", *parts, "release", "contract_tools")
    executable.parent.mkdir(parents=True)
    executable.touch()
    return executable


def test_runtime_executable_defaults_to_workspace_release_binary(monkeypatch, tmp_path):
    module = load_stage_runtime()
    executable = touch_runtime_binary(tmp_path)
    monkeypatch.setattr(module, "REPO_ROOT", tmp_path)
    monkeypatch.delenv("CARGO_BUILD_TARGET", raising=False)

    assert module.runtime_executable() == executable


def test_runtime_executable_uses_cargo_build_target(monkeypatch, tmp_path):
    module = load_stage_runtime()
    executable = touch_runtime_binary(tmp_path, "x86_64-unknown-linux-gnu")
    monkeypatch.setattr(module, "REPO_ROOT", tmp_path)
    monkeypatch.setenv("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu")

    assert module.runtime_executable() == executable
