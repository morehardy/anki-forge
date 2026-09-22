from __future__ import annotations

from dataclasses import asdict, dataclass, field, fields, replace
from enum import StrEnum
import json
from os import PathLike
from pathlib import Path
from typing import Any, Literal, TypeAlias

from .diagnostics import ValidationError
from . import _native

PathInput = str | PathLike[str]


class RiskLevel(StrEnum):
    INFO = "info"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class UpdateSafetyMode(StrEnum):
    DISABLED = "disabled"
    REPORT_ONLY = "report_only"
    STRICT = "strict"


RiskLevelValue: TypeAlias = RiskLevel | Literal["info", "low", "medium", "high", "critical"]
UpdateSafetyValue: TypeAlias = UpdateSafetyMode | Literal["disabled", "report_only", "report-only", "strict"]


class MediaMode(StrEnum):
    PATH_BACKED = "path_backed"
    SELF_CONTAINED = "self_contained"


class DiagnosticBehavior(StrEnum):
    IGNORE = "ignore"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


MediaModeValue: TypeAlias = MediaMode | Literal["path_backed", "path-backed", "self_contained", "self-contained"]
DiagnosticBehaviorValue: TypeAlias = DiagnosticBehavior | Literal["ignore", "info", "warning", "error"]


@dataclass(frozen=True)
class MediaPolicy:
    """Named SDK access to the core's advanced media diagnostics policy."""

    unused_binding: DiagnosticBehaviorValue | None = None
    unknown_mime: DiagnosticBehaviorValue | None = None
    declared_mime_mismatch: Literal["warning", "error"] | None = None

    def __post_init__(self) -> None:
        try:
            for name in ("unused_binding", "unknown_mime"):
                value = getattr(self, name)
                if value is not None:
                    object.__setattr__(self, name, DiagnosticBehavior(value))
            if self.declared_mime_mismatch not in {None, "warning", "error"}:
                raise ValueError("declared_mime_mismatch must be warning or error")
        except ValueError as error:
            raise ValidationError(str(error)) from error


def _limit_default(name: str) -> int:
    defaults: dict[str, int] = json.loads(_native.default_inspect_limits())
    return defaults[name]


@dataclass(frozen=True)
class InspectLimits:
    """Finite u64 budgets; defaults are read from the linked Rust core."""

    max_archive_bytes: int = field(default_factory=lambda: _limit_default("max_archive_bytes"))
    max_entries: int = field(default_factory=lambda: _limit_default("max_entries"))
    max_central_directory_bytes: int = field(default_factory=lambda: _limit_default("max_central_directory_bytes"))
    max_zip_entry_bytes: int = field(default_factory=lambda: _limit_default("max_zip_entry_bytes"))
    max_zip_total_bytes: int = field(default_factory=lambda: _limit_default("max_zip_total_bytes"))
    max_meta_bytes: int = field(default_factory=lambda: _limit_default("max_meta_bytes"))
    max_media_map_bytes: int = field(default_factory=lambda: _limit_default("max_media_map_bytes"))
    max_collection_bytes: int = field(default_factory=lambda: _limit_default("max_collection_bytes"))
    max_media_bytes: int = field(default_factory=lambda: _limit_default("max_media_bytes"))
    max_decoded_total_bytes: int = field(default_factory=lambda: _limit_default("max_decoded_total_bytes"))
    max_zstd_window_bytes: int = field(default_factory=lambda: _limit_default("max_zstd_window_bytes"))

    def __post_init__(self) -> None:
        for item in fields(self):
            value = getattr(self, item.name)
            if type(value) is not int or not 0 <= value <= 2**64 - 1:
                raise ValidationError(f"{item.name} must be an integer from 0 to 2**64 - 1")


def absolute_path(path: PathInput, base_dir: Path) -> Path:
    value = Path(path).expanduser()
    return value if value.is_absolute() else base_dir / value


@dataclass(frozen=True)
class BuildOptions:
    """Immutable build configuration. None keeps the corresponding Rust default."""

    def first_update_safe_build(self, lockfile: PathInput) -> BuildOptions:
        """Return strict options that also publish initial identity evidence."""
        return replace(self, identity_lockfile=lockfile, write_identity_lockfile=True, update_safety=UpdateSafetyMode.STRICT)

    def update_safe(self, lockfile: PathInput) -> BuildOptions:
        """Return strict options using an existing identity lockfile."""
        return replace(self, identity_lockfile=lockfile, update_safety=UpdateSafetyMode.STRICT)

    output: PathInput | None = None
    artifacts_dir: PathInput | None = None
    report_json: PathInput | None = None
    inspect: bool | None = None
    inspect_limits: InspectLimits | None = None
    compare_to: PathInput | None = None
    fail_on: RiskLevelValue | None = None
    identity_lockfile: PathInput | None = None
    write_identity_lockfile: bool | None = None
    update_safety: UpdateSafetyValue | None = None
    self_contained: bool | None = None
    media_mode: MediaModeValue | None = None
    media_policy: MediaPolicy | None = None
    media_store_dir: PathInput | None = None

    def __post_init__(self) -> None:
        if self.inspect is not None and type(self.inspect) is not bool:
            raise ValidationError("inspect must be bool")
        if self.inspect_limits is not None and not isinstance(self.inspect_limits, InspectLimits):
            raise ValidationError("inspect_limits must be InspectLimits")
        if self.write_identity_lockfile is not None and type(self.write_identity_lockfile) is not bool:
            raise ValidationError("write_identity_lockfile must be bool")
        if self.self_contained is not None and type(self.self_contained) is not bool:
            raise ValidationError("self_contained must be bool")
        if self.media_policy is not None and not isinstance(self.media_policy, MediaPolicy):
            raise ValidationError("media_policy must be MediaPolicy")
        try:
            if self.fail_on is not None:
                object.__setattr__(self, "fail_on", RiskLevel(self.fail_on))
            if self.update_safety is not None:
                object.__setattr__(self, "update_safety", UpdateSafetyMode(self.update_safety.replace("-", "_")))
            if self.media_mode is not None:
                object.__setattr__(self, "media_mode", MediaMode(self.media_mode.replace("-", "_")))
        except (ValueError, AttributeError) as error:
            raise ValidationError(str(error)) from error
        if self.self_contained is not None and self.media_mode is not None:
            if self.self_contained != (self.media_mode == MediaMode.SELF_CONTAINED):
                raise ValidationError("conflicting self_contained and media_mode")

    def _with_overrides(self, base_dir: Path, **overrides: Any) -> BuildOptions:
        values = {key: value for key, value in overrides.items() if value is not None}
        candidate = replace(self, **values)
        for key in values:
            old = getattr(self, key)
            new = getattr(candidate, key)
            if old is None:
                continue
            if key in {"output", "artifacts_dir", "report_json", "compare_to", "identity_lockfile"}:
                old, new = absolute_path(old, base_dir), absolute_path(new, base_dir)
            if old != new:
                raise ValidationError(f"conflicting {key} in BuildOptions and write_apkg arguments")
        return candidate

    def _payload(self, base_dir: Path) -> dict[str, Any]:
        return {
            "output": str(absolute_path(self.output, base_dir)) if self.output is not None else None,
            "artifacts_dir": str(absolute_path(self.artifacts_dir, base_dir)) if self.artifacts_dir is not None else None,
            "report_json": str(absolute_path(self.report_json, base_dir)) if self.report_json is not None else None,
            "inspect": self.inspect,
            "inspect_limits": asdict(self.inspect_limits) if self.inspect_limits is not None else None,
            "compare_to": str(absolute_path(self.compare_to, base_dir)) if self.compare_to is not None else None,
            "identity_lockfile": str(absolute_path(self.identity_lockfile, base_dir)) if self.identity_lockfile is not None else None,
            "fail_on": self.fail_on,
            "write_identity_lockfile": self.write_identity_lockfile,
            "update_safety": self.update_safety,
            "self_contained": self.self_contained,
            "media_mode": self.media_mode,
            "media_policy": asdict(self.media_policy) if self.media_policy is not None else None,
            "media_store_dir": str(absolute_path(self.media_store_dir, base_dir)) if self.media_store_dir is not None else None,
        }
