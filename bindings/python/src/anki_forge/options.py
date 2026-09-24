from __future__ import annotations
from dataclasses import dataclass, replace, asdict
from enum import StrEnum
from os import PathLike
from pathlib import Path
from typing import Any
from . import _native
from ._bridge import invoke
PathInput = str | PathLike[str]

def absolute_path(path: PathInput, base_dir: Path | None = None) -> str:
    value = Path(path).expanduser()
    return str(value.absolute() if value.is_absolute() else ((base_dir or Path.cwd()) / value).absolute())

@dataclass(frozen=True)
class InspectLimits:
    max_archive_bytes: int = 2 << 30
    max_entries: int = 100_000
    max_central_directory_bytes: int = 16 << 20
    max_zip_entry_bytes: int = 1 << 30
    max_zip_total_bytes: int = 4 << 30
    max_meta_bytes: int = 64 << 10
    max_media_map_bytes: int = 16 << 20
    max_identity_bytes: int = 64 << 20
    max_collection_bytes: int = 512 << 20
    max_media_bytes: int = 256 << 20
    max_decoded_total_bytes: int = 4 << 30
    max_zstd_window_bytes: int = 64 << 20

class RiskLevel(StrEnum):
    INFO = "info"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

@dataclass(frozen=True)
class UpdatePolicy:
    _threshold: RiskLevel = RiskLevel.HIGH
    _allowances: tuple[str, ...] = ()

    def fail_on(self, level: RiskLevel) -> UpdatePolicy:
        return replace(self, _threshold=level)

    def allow(self, code: str) -> UpdatePolicy:
        invoke(_native.validate_risk_code, code)
        return replace(self, _allowances=(*self._allowances, code))

    def _payload(self) -> dict[str, Any]:
        return dict(threshold=self._threshold, allowances=self._allowances)

@dataclass(frozen=True)
class BuildOptions:
    _output: str | None
    _baseline: str | None = None
    _limits: InspectLimits | None = None
    _policy: UpdatePolicy | None = None

    @staticmethod
    def to(path: PathInput) -> BuildOptions:
        return BuildOptions(absolute_path(path))

    @staticmethod
    def temporary() -> BuildOptions:
        return BuildOptions(None)

    def update_from(self, path: PathInput) -> BuildOptions:
        return replace(self, _baseline=absolute_path(path))

    def inspect_limits(self, limits: InspectLimits) -> BuildOptions:
        return replace(self, _limits=limits)

    def update_policy(self, policy: UpdatePolicy) -> BuildOptions:
        return replace(self, _policy=policy)

    def _payload(self) -> dict[str, Any]:
        return dict(output=self._output, baseline=self._baseline,
                    limits=asdict(self._limits) if self._limits else None,
                    policy=self._policy._payload() if self._policy else None)

@dataclass(frozen=True)
class CompareOptions:
    _baseline: str
    _limits: InspectLimits | None = None
    _policy: UpdatePolicy | None = None

    @staticmethod
    def against(path: PathInput) -> CompareOptions:
        return CompareOptions(absolute_path(path))

    def inspect_limits(self, limits: InspectLimits) -> CompareOptions:
        return replace(self, _limits=limits)

    def update_policy(self, policy: UpdatePolicy) -> CompareOptions:
        return replace(self, _policy=policy)

    def _payload(self) -> dict[str, Any]:
        return dict(baseline=self._baseline, limits=asdict(self._limits) if self._limits else None,
                    policy=self._policy._payload() if self._policy else None)
