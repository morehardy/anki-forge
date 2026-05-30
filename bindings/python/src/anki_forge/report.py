from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from .diagnostics import Diagnostic, DiagnosticsError, ProtocolError

BUILD_REPORT_KIND = "anki-forge-build-report"
BUILD_REPORT_SCHEMA_VERSION = "phase4-build-report-v2"
ALLOWED_STATUSES = {"success", "blocked", "invalid", "error"}
ALLOWED_COMPARISONS = {"not_requested", "complete", "partial", "unavailable"}
COUNT_FIELDS = ("notes", "cards", "media")
MEDIA_FIELDS = (
    "objects",
    "bindings",
    "references",
    "missing_references",
    "unsafe_references",
    "unused_bindings",
    "unique_bytes",
)
POLICY_FIELDS = {"status", "threshold", "highest_risk", "blocking_findings"}
POLICY_STATUSES = {"passed", "blocked", "not_evaluated"}
RISK_LEVELS = {"info", "low", "medium", "high", "critical"}


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
        if status not in ALLOWED_STATUSES:
            raise ProtocolError(f"build report has unsupported status: {status}")
        comparison = _required_non_empty_string(payload, "comparison")
        if comparison not in ALLOWED_COMPARISONS:
            raise ProtocolError(f"build report has unsupported comparison: {comparison}")

        artifact = _artifact(payload.get("artifact"))
        counts = _required_int_map(payload, "counts", COUNT_FIELDS)
        media = _required_int_map(payload, "media", MEDIA_FIELDS)
        diagnostics = _diagnostics(payload.get("diagnostics"))

        _metrics(payload.get("metrics"))
        _policy(payload.get("policy"))

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
    if not set(fields).issubset(value):
        raise ProtocolError(f"build report {key} is missing required fields")
    parsed: dict[str, int] = {}
    for field in fields:
        parsed[field] = _non_negative_int(value.get(field), f"{key}.{field}")
    return parsed


def _artifact(value: object) -> Mapping[str, Any] | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise ProtocolError("build report artifact must be an object or null")
    if "path" not in value:
        raise ProtocolError("build report artifact must contain path")
    if not isinstance(value["path"], str) or not value["path"]:
        raise ProtocolError("build report artifact.path must be a non-empty string")
    return value


def _metrics(value: object) -> None:
    if not isinstance(value, dict):
        raise ProtocolError("build report metrics must be an object")
    if "duration_ms" not in value:
        raise ProtocolError("build report metrics must contain duration_ms")
    _non_negative_int(value.get("duration_ms"), "metrics.duration_ms")


def _policy(value: object) -> None:
    if not isinstance(value, dict):
        raise ProtocolError("build report policy must be an object")
    if not POLICY_FIELDS.issubset(value):
        raise ProtocolError("build report policy is missing required fields")
    status = value["status"]
    if not isinstance(status, str) or status not in POLICY_STATUSES:
        raise ProtocolError("build report policy.status is unsupported")
    for key in ("threshold", "highest_risk"):
        risk_value = value[key]
        if risk_value is not None and (not isinstance(risk_value, str) or risk_value not in RISK_LEVELS):
            raise ProtocolError(f"build report policy.{key} is unsupported")
    blocking_findings = value["blocking_findings"]
    if not isinstance(blocking_findings, list) or not all(isinstance(item, str) for item in blocking_findings):
        raise ProtocolError("build report policy.blocking_findings must be an array of strings")


def _non_negative_int(value: object, label: str) -> int:
    if type(value) is not int or value < 0:
        raise ProtocolError(f"build report {label} must be a non-negative integer")
    return value


def _diagnostics(value: object) -> tuple[Diagnostic, ...]:
    if not isinstance(value, list):
        raise ProtocolError("build report diagnostics must be an array")
    diagnostics: list[Diagnostic] = []
    for item in value:
        if not isinstance(item, dict):
            raise ProtocolError("build report diagnostic must be an object")
        severity = _required_non_empty_string(item, "severity")
        if severity not in {"error", "warning", "info"}:
            raise ProtocolError("build report diagnostic severity is unsupported")
        diagnostics.append(
            Diagnostic(
                code=_required_non_empty_string(item, "code"),
                severity=severity,
                message=_required_non_empty_string(item, "message"),
                domain=_optional_string(item, "domain"),
                stage=_optional_string(item, "stage"),
                path=_optional_string(item, "path") or _optional_string(item, "source"),
                suggested_fix=_optional_string(item, "suggested_fix")
                or _optional_string(item, "help"),
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
