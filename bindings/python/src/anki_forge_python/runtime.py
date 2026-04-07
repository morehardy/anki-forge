from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ResolvedRuntime:
    mode: str
    manifest_path: Path
    bundle_root: Path
    bundle_version: str
    launcher_executable: str
    launcher_prefix: tuple[str, ...]


def _read_bundle_version(manifest_path: Path) -> str:
    for line in manifest_path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("bundle_version:"):
            return stripped.split(":", 1)[1].strip().strip("'\"")
    return "unknown"


def resolve_runtime(
    *,
    cwd: Path | None = None,
    mode: str | None = None,
    manifest_path: str | None = None,
    bundle_root: str | None = None,
    launcher_executable: str | None = None,
    launcher_prefix: list[str] | None = None,
) -> ResolvedRuntime:
    if mode == "installed":
        manifest = Path(manifest_path).resolve()
        bundle = Path(bundle_root).resolve()
        return ResolvedRuntime(
            mode="installed",
            manifest_path=manifest,
            bundle_root=bundle,
            bundle_version=_read_bundle_version(manifest),
            launcher_executable=launcher_executable or "contract_tools",
            launcher_prefix=tuple(launcher_prefix or []),
        )

    current = Path(cwd or Path.cwd()).resolve()
    while True:
        manifest = current / "contracts" / "manifest.yaml"
        if manifest.is_file():
            return ResolvedRuntime(
                mode="workspace",
                manifest_path=manifest,
                bundle_root=manifest.parent,
                bundle_version=_read_bundle_version(manifest),
                launcher_executable=launcher_executable or "cargo",
                launcher_prefix=tuple(
                    launcher_prefix or ["run", "-q", "-p", "contract_tools", "--"]
                ),
            )
        if current.parent == current:
            raise RuntimeError("failed to discover contracts/manifest.yaml from workspace path")
        current = current.parent
