from __future__ import annotations

from dataclasses import dataclass
from typing import Any, TYPE_CHECKING

if TYPE_CHECKING:
    from .report import BuildReport, ProjectDiffReport


class AnkiForgeError(Exception):
    """Base exception for anki-forge Python API errors."""


class ValidationError(AnkiForgeError, ValueError):
    """Raised when public API input fails validation."""


class AuthoringError(ValidationError):
    def __init__(self, code: str, message: str, *, diagnostic: Diagnostic | None = None, details: dict[str, Any] | None = None) -> None:
        super().__init__(message if message.startswith(code) else f"{code}: {message}")
        self.code = code
        self.message = message
        self.diagnostic = diagnostic
        self.diagnostics = (diagnostic,) if diagnostic is not None else ()
        self.details = details or {}
        self.path = diagnostic.path if diagnostic is not None else self.details.get("path")
        self.span = diagnostic.span if diagnostic is not None else None


class ProjectAddError(AuthoringError):
    """The core rejected an addition without changing the Project."""


class ProductNoteError(AuthoringError):
    """The core rejected a Product note builder."""


class MediaError(AuthoringError):
    """The core rejected a media registration."""


class DeckError(AuthoringError):
    """The Rust Deck rejected a stock-note or identity operation."""


class TemplateBundleError(AuthoringError):
    """A failed, atomic template bundle import; offsets are UTF-8 bytes."""

    @property
    def byte_offset(self) -> int | None:
        value = self.details.get("byte_offset")
        return value if isinstance(value, int) else None


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


class BuildError(DiagnosticsError):
    """A failed build, retaining its complete report and any recoverable artifact."""

    def __init__(self, report: BuildReport) -> None:
        super().__init__("anki-forge build failed", report=report)
        self.failure_cause = report.failure_cause
        self.code = report.failure_code or next(
            (diagnostic.code for diagnostic in report.diagnostics if diagnostic.severity == "error"),
            "PROJECT.BUILD_DIAGNOSTICS",
        )


class ProjectDiffError(DiagnosticsError):
    """A failed comparison, retaining diagnostics and partial evidence."""

    def __init__(self, report: ProjectDiffReport) -> None:
        super().__init__("anki-forge comparison failed", report=report)
        self.failure_cause = report.failure_cause
        self.code = next((d.code for d in report.diagnostics if d.severity == "error"), "PROJECT.DIFF_FAILED")


@dataclass(frozen=True)
class SourceSpan:
    byte_start: int
    byte_end: int


@dataclass(frozen=True)
class Diagnostic:
    code: str
    severity: str
    message: str
    domain: str | None = None
    stage: str | None = None
    path: str | None = None
    span: SourceSpan | None = None
    suggested_fix: str | None = None

    @property
    def source(self) -> str | None:
        return self.path

    @property
    def help(self) -> str | None:
        return self.suggested_fix
