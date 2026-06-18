from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from .diagnostics import ProtocolError, RuntimeInvocationError, RuntimeNotFoundError
from .report import BuildReport


@dataclass(frozen=True)
class ResolvedRuntime:
    manifest: Path
    executable: Path
    mode: str


@dataclass(frozen=True)
class RuntimeOverride:
    manifest: Path
    executable: Path

    def resolve(self) -> ResolvedRuntime:
        return ResolvedRuntime(
            manifest=self.manifest.resolve(),
            executable=self.executable.resolve(),
            mode="explicit",
        )


def resolve_runtime(explicit: RuntimeOverride | ResolvedRuntime | None = None, cwd: Path | str | None = None) -> ResolvedRuntime:
    if isinstance(explicit, ResolvedRuntime):
        return explicit
    if explicit is not None:
        return explicit.resolve()

    bundled = _bundled_runtime()
    if bundled is not None:
        return bundled
    return _workspace_runtime(cwd)


def _workspace_runtime(cwd: Path | str | None) -> ResolvedRuntime:
    workspace_root = _find_workspace_root(cwd)
    manifest = workspace_root / "contracts" / "manifest.yaml"
    executable_name = "contract_tools.exe" if os.name == "nt" else "contract_tools"
    cargo_target = os.environ.get("CARGO_BUILD_TARGET")
    for profile in ("release", "debug"):
        if cargo_target:
            executable = workspace_root / "target" / cargo_target / profile / executable_name
            if executable.is_file():
                return ResolvedRuntime(manifest=manifest, executable=executable, mode="workspace")
        executable = workspace_root / "target" / profile / executable_name
        if executable.is_file():
            return ResolvedRuntime(manifest=manifest, executable=executable, mode="workspace")
    raise RuntimeNotFoundError(
        "found contracts/manifest.yaml but no contract_tools executable; "
        "build it with cargo or pass RuntimeOverride(manifest=..., executable=...)"
    )


def _find_workspace_root(cwd: Path | str | None) -> Path:
    current = Path(cwd or Path.cwd()).resolve()
    while True:
        if (current / "contracts" / "manifest.yaml").is_file():
            return current
        if current.parent == current:
            raise RuntimeNotFoundError(
                "failed to discover anki-forge runtime; build contract_tools in the workspace "
                "or pass RuntimeOverride(manifest=..., executable=...)"
            )
        current = current.parent


def _bundled_runtime() -> ResolvedRuntime | None:
    package_dir = Path(__file__).resolve().parent
    executable_name = "contract_tools.exe" if os.name == "nt" else "contract_tools"
    executable = package_dir / "_runtime" / "bin" / executable_name
    manifest = package_dir / "_runtime" / "contracts" / "manifest.yaml"
    if executable.is_file() and manifest.is_file():
        return ResolvedRuntime(manifest=manifest, executable=executable, mode="bundled")
    return None


def build_product_build_argv(
    *,
    executable: Path,
    manifest: Path,
    product_input: Path,
    apkg_out: Path,
    compare_to: Path | None,
    fail_on: str | None,
    report_json: Path | None,
    identity_lockfile: Path | None,
    write_identity_lockfile: bool,
    update_safety: str | None,
) -> list[str]:
    argv = [
        str(executable),
        "product-build",
        "--manifest",
        str(manifest),
        "--product-input",
        str(product_input),
        "--apkg-out",
        str(apkg_out),
        "--output",
        "contract-json",
    ]
    if compare_to is not None:
        argv.extend(["--compare-to", str(compare_to)])
    if fail_on is not None:
        argv.extend(["--fail-on", fail_on])
    if report_json is not None:
        argv.extend(["--report-json", str(report_json)])
    if identity_lockfile is not None:
        argv.extend(["--identity-lockfile", str(identity_lockfile)])
    if write_identity_lockfile:
        argv.append("--write-identity-lockfile")
    if update_safety is not None:
        argv.extend(["--update-safety", update_safety])
    return argv


def run_product_build(
    *,
    runtime: ResolvedRuntime,
    product_input: Path,
    apkg_out: Path,
    compare_to: Path | None = None,
    fail_on: str | None = None,
    report_json: Path | None = None,
    identity_lockfile: Path | None = None,
    write_identity_lockfile: bool = False,
    update_safety: str | None = None,
) -> BuildReport:
    argv = build_product_build_argv(
        executable=runtime.executable,
        manifest=runtime.manifest,
        product_input=product_input,
        apkg_out=apkg_out,
        compare_to=compare_to,
        fail_on=fail_on,
        report_json=report_json,
        identity_lockfile=identity_lockfile,
        write_identity_lockfile=write_identity_lockfile,
        update_safety=update_safety,
    )
    try:
        completed = subprocess.run(
            argv,
            shell=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
    except OSError as error:
        raise RuntimeInvocationError(str(error), kind="spawn_failed", argv=argv) from error
    except UnicodeDecodeError as error:
        raise RuntimeInvocationError(str(error), kind="decode_failed", argv=argv) from error
    return parse_completed_process(completed)


def parse_completed_process(completed: subprocess.CompletedProcess[str]) -> BuildReport:
    stdout = completed.stdout.strip()
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as error:
        if completed.returncode < 0:
            raise RuntimeInvocationError(
                "runtime process was interrupted before producing a build report",
                kind="interrupted",
                argv=list(completed.args) if isinstance(completed.args, (list, tuple)) else [str(completed.args)],
                exit_code=completed.returncode,
                stdout=completed.stdout,
                stderr=completed.stderr,
            ) from error
        if completed.returncode == 0:
            raise ProtocolError("runtime exited successfully without valid build report JSON") from error
        raise RuntimeInvocationError(
            "runtime exited without a build report",
            kind="exit_without_report",
            argv=list(completed.args) if isinstance(completed.args, (list, tuple)) else [str(completed.args)],
            exit_code=completed.returncode,
            stdout=completed.stdout,
            stderr=completed.stderr,
        ) from error
    return BuildReport.from_json(payload)
