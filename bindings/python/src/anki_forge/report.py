from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from .diagnostics import Diagnostic, DiagnosticsError, ProtocolError

BUILD_REPORT_KIND = "anki-forge-build-report"
BUILD_REPORT_SCHEMA_VERSION = "phase4-build-report-v1"
ALLOWED_COMPARISONS = {"not_requested", "complete", "partial", "unavailable"}


@dataclass(frozen=True)
class BuildReport:
    status: str
    comparison: str
    artifact: Mapping[str, Any] | None
    counts: Mapping[str, int]
    media: Mapping[str, int]
    diagnostics: tuple[Diagnostic, ...]
    inspect: Mapping[str, Any] | None = None
    previous_inspect: Mapping[str, Any] | None = None
    update_safety: Mapping[str, Any] | None = None
    diff: Mapping[str, Any] | None = None
    risk: Mapping[str, Any] | None = None

    @classmethod
    def from_json(cls, payload: object) -> BuildReport:
        if not isinstance(payload, dict):
            raise ProtocolError("build report must be a JSON object")
        if payload.get("kind") != BUILD_REPORT_KIND:
            raise ProtocolError("build report has unexpected kind")
        if payload.get("schema_version") != BUILD_REPORT_SCHEMA_VERSION:
            raise ProtocolError("build report has unexpected schema_version")

        status = _required_non_empty_string(payload, "status")
        comparison = _required_non_empty_string(payload, "comparison")
        if comparison not in ALLOWED_COMPARISONS:
            raise ProtocolError(f"build report has unsupported comparison: {comparison}")

        artifact = payload.get("artifact")
        if artifact is not None and not isinstance(artifact, dict):
            raise ProtocolError("build report artifact must be an object or null")

        counts = _required_int_map(payload, "counts", ("notes", "cards", "media"))
        media = _required_int_map(payload, "media", ("objects", "bindings", "bytes"))
        diagnostics = _diagnostics(payload.get("diagnostics"))

        metrics = payload.get("metrics")
        if not isinstance(metrics, dict) or not isinstance(metrics.get("duration_ms"), int):
            raise ProtocolError("build report metrics.duration_ms must be an integer")

        policy = payload.get("policy")
        if not isinstance(policy, dict) or not isinstance(policy.get("status"), str) or not policy["status"]:
            raise ProtocolError("build report policy.status must be a non-empty string")

        return cls(
            status=status,
            comparison=comparison,
            artifact=artifact,
            counts=counts,
            media=media,
            diagnostics=diagnostics,
            inspect=_optional_object(payload, "inspect"),
            previous_inspect=_optional_object(payload, "previous_inspect"),
            update_safety=_optional_object(payload, "update_safety"),
            diff=_optional_object(payload, "diff"),
            risk=_optional_object(payload, "risk"),
        )

    def ensure_success(self) -> None:
        has_error = any(diagnostic.severity in {"error", "critical"} for diagnostic in self.diagnostics)
        if self.status != "success" or self.artifact is None or has_error:
            raise DiagnosticsError("anki-forge build failed", report=self)


def _required_non_empty_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise ProtocolError(f"build report {key} must be a non-empty string")
    return value


def _required_int_map(payload: dict[str, Any], key: str, fields: tuple[str, ...]) -> dict[str, int]:
    value = payload.get(key)
    if not isinstance(value, dict):
        raise ProtocolError(f"build report {key} must be an object")
    parsed: dict[str, int] = {}
    for field in fields:
        field_value = value.get(field)
        if not isinstance(field_value, int):
            raise ProtocolError(f"build report {key}.{field} must be an integer")
        parsed[field] = field_value
    return parsed


def _diagnostics(value: object) -> tuple[Diagnostic, ...]:
    if not isinstance(value, list):
        raise ProtocolError("build report diagnostics must be an array")
    diagnostics: list[Diagnostic] = []
    for item in value:
        if not isinstance(item, dict):
            raise ProtocolError("build report diagnostic must be an object")
        diagnostics.append(
            Diagnostic(
                code=_required_non_empty_string(item, "code"),
                severity=_required_non_empty_string(item, "severity"),
                message=_required_non_empty_string(item, "message"),
                source=_optional_string(item, "source"),
                help=_optional_string(item, "help"),
            )
        )
    return tuple(diagnostics)


def _optional_string(payload: dict[str, Any], key: str) -> str | None:
    value = payload.get(key)
    if value is None:
        return None
    if not isinstance(value, str):
        raise ProtocolError(f"build report diagnostic {key} must be a string")
    return value


def _optional_object(payload: dict[str, Any], key: str) -> Mapping[str, Any] | None:
    value = payload.get(key)
    if value is None:
        return None
    if not isinstance(value, dict):
        raise ProtocolError(f"build report {key} must be an object")
    return value
