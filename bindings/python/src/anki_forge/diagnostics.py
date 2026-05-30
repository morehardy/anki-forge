from __future__ import annotations

from dataclasses import dataclass
from typing import Any


class AnkiForgeError(Exception):
    """Base exception for anki-forge Python API errors."""


class ValidationError(AnkiForgeError, ValueError):
    """Raised when public API input fails validation."""


class RuntimeNotFoundError(AnkiForgeError):
    """Raised when the anki-forge runtime cannot be found."""


class ProtocolError(AnkiForgeError):
    """Raised when runtime protocol data is invalid."""


class RuntimeInvocationError(AnkiForgeError):
    """Raised when invoking the runtime fails."""

    def __init__(
        self,
        message: str,
        *,
        kind: str,
        argv: list[str] | None = None,
        exit_code: int | None = None,
        stdout: str | None = None,
        stderr: str | None = None,
    ) -> None:
        super().__init__(message)
        self.kind = kind
        self.argv = argv or []
        self.exit_code = exit_code
        self.stdout = stdout
        self.stderr = stderr


class DiagnosticsError(AnkiForgeError):
    """Raised when a diagnostics report contains errors."""

    def __init__(
        self,
        message: str,
        report: Any,
        *,
        exit_status: int | None = None,
        stdout: str | None = None,
        stderr: str | None = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.report = report
        self.diagnostics = getattr(report, "diagnostics", ())
        self.status = getattr(report, "status", None)
        self.exit_status = exit_status
        self.stdout = stdout
        self.stderr = stderr


@dataclass(frozen=True)
class Diagnostic:
    code: str
    severity: str
    message: str
    domain: str | None = None
    stage: str | None = None
    path: str | None = None
    suggested_fix: str | None = None

    @property
    def source(self) -> str | None:
        return self.path

    @property
    def help(self) -> str | None:
        return self.suggested_fix
