from __future__ import annotations

import subprocess
from dataclasses import dataclass

from .errors import RuntimeInvocationError
from .runtime import ResolvedRuntime, resolve_runtime


@dataclass(frozen=True)
class RawCommandResult:
    command: str
    argv: tuple[str, ...]
    exit_status: int
    stdout: str
    stderr: str
    resolved_runtime: ResolvedRuntime


def _build_args(command: str, request: dict, runtime: ResolvedRuntime) -> list[str]:
    if command == "normalize":
        return [
            *runtime.launcher_prefix,
            "normalize",
            "--manifest",
            str(runtime.manifest_path),
            "--input",
            request["input_path"],
            "--output",
            request.get("output", "contract-json"),
        ]
    if command == "build":
        return [
            *runtime.launcher_prefix,
            "build",
            "--manifest",
            str(runtime.manifest_path),
            "--input",
            request["input_path"],
            "--writer-policy",
            request.get("writer_policy", "default"),
            "--build-context",
            request.get("build_context", "default"),
            "--artifacts-dir",
            request["artifacts_dir"],
            "--output",
            request.get("output", "contract-json"),
        ]
    if command == "inspect":
        if "staging_path" in request:
            return [
                *runtime.launcher_prefix,
                "inspect",
                "--staging",
                request["staging_path"],
                "--output",
                request.get("output", "contract-json"),
            ]
        return [
            *runtime.launcher_prefix,
            "inspect",
            "--apkg",
            request["apkg_path"],
            "--output",
            request.get("output", "contract-json"),
        ]
    if command == "diff":
        return [
            *runtime.launcher_prefix,
            "diff",
            "--left",
            request["left_path"],
            "--right",
            request["right_path"],
            "--output",
            request.get("output", "contract-json"),
        ]
    raise ValueError(f"unsupported command: {command}")


def run_raw(command: str, request: dict, **runtime_kwargs) -> RawCommandResult:
    try:
        runtime = resolve_runtime(**runtime_kwargs)
    except Exception as error:
        raise RuntimeInvocationError(
            str(error),
            command=command,
            stdout="",
            stderr="",
            resolved_runtime=None,
            failure_phase="runtime-resolution",
        ) from error

    argv = [runtime.launcher_executable, *_build_args(command, request, runtime)]
    try:
        completed = subprocess.run(
            argv,
            cwd=str(runtime.manifest_path.parent.parent)
            if runtime.mode == "workspace"
            else None,
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        raise RuntimeInvocationError(
            str(error),
            command=command,
            stdout="",
            stderr="",
            resolved_runtime=runtime,
            failure_phase="spawn",
        ) from error

    return RawCommandResult(
        command=command,
        argv=tuple(argv),
        exit_status=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        resolved_runtime=runtime,
    )
