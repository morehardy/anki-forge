from __future__ import annotations

from ._loader import Versions, __version__, versions
from .diagnostics import (
    Diagnostic,
    DiagnosticsError,
    BuildError,
    ProtocolError,
    RuntimeInvocationError,
    RuntimeNotFoundError,
    SourceSpan,
    ValidationError,
    ProjectAddError,
    ProductNoteError,
    MediaError,
    TemplateBundleError,
    ProjectDiffError,
    DeckError,
)
from .media import MediaRef, MediaRegistry
from .note import Note
from .notetype import Field, GenerationRule, NoteType, Template
from .project import Project
from .deck import Deck, BasicIdentityOverride, DeckMediaRef, DeckMediaRegistry
from .artifact import ApkgArtifact
from .identity import IdentityRecipe
from .content import Content
from .options import BuildOptions, DiagnosticBehavior, InspectLimits, MediaMode, MediaPolicy, RiskLevel, UpdateSafetyMode
from .report import BuildReport, ProjectDiffReport, ValidationReport

__all__ = [
    "__version__",
    "versions",
    "Versions",
    "Diagnostic",
    "DiagnosticsError",
    "BuildError",
    "ProtocolError",
    "RuntimeInvocationError",
    "RuntimeNotFoundError",
    "SourceSpan",
    "ValidationError",
    "ProjectAddError",
    "ProductNoteError",
    "MediaError",
    "TemplateBundleError",
    "ProjectDiffError",
    "ProjectDiffReport",
    "Deck",
    "DeckError",
    "BasicIdentityOverride",
    "DeckMediaRef",
    "DeckMediaRegistry",
    "MediaRef",
    "MediaRegistry",
    "Note",
    "Field",
    "GenerationRule",
    "NoteType",
    "Template",
    "Project",
    "BuildReport",
    "ValidationReport",
    "ApkgArtifact",
    "IdentityRecipe",
    "Content",
    "BuildOptions",
    "InspectLimits",
    "RiskLevel",
    "UpdateSafetyMode",
    "MediaMode",
    "MediaPolicy",
    "DiagnosticBehavior",
]
