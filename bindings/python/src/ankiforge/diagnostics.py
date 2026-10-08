"""Structured failures from the Rust public API."""
from __future__ import annotations
from copy import deepcopy
from collections.abc import Mapping
from typing import Any, Generic, Literal, TypeAlias, TypeVar, TypedDict

class ByteRange(TypedDict):
    start: int
    end: int

class SimpleAddTarget(TypedDict):
    type: Literal["note_key", "note", "model", "unknown"]

class FieldAddTarget(TypedDict):
    type: Literal["field"]
    field_key: str
    content_path: list[int] | None
    byte_range: ByteRange | None

class TagAddTarget(TypedDict):
    type: Literal["tag"]
    index: int
    value: str

class DeckAddTarget(TypedDict):
    type: Literal["deck"]
    name: str
    inherited: bool

class AssetAddTarget(TypedDict):
    type: Literal["model_asset", "occlusion_image", "explicit_asset"]
    media_name: str

class DefaultDeckAddTarget(TypedDict):
    type: Literal["project_default_deck"]
    name: str

AddTarget: TypeAlias = SimpleAddTarget | FieldAddTarget | TagAddTarget | DeckAddTarget | AssetAddTarget | DefaultDeckAddTarget

class AddContext(TypedDict):
    note_key: str | None
    model_key: str | None
    target: AddTarget

MediaUsage: TypeAlias = Literal["image", "sound", "unknown"]
MediaConflictKind: TypeAlias = Literal["portable_name_collision", "different_content", "unknown"]

class SimpleAddDetail(TypedDict):
    type: Literal["model_definition_conflict", "unknown"]

class ModelNameAddDetail(TypedDict):
    type: Literal["model_name_conflict"]
    existing_model_key: str
    conflicting_name: str

class MediaConflictAddDetail(TypedDict):
    type: Literal["media_conflict"]
    kind: MediaConflictKind
    existing_name: str
    incoming_name: str

class MediaUsageAddDetail(TypedDict):
    type: Literal["media_usage"]
    requested: MediaUsage
    media_name: str
    media_type: str

AddDetail: TypeAlias = SimpleAddDetail | ModelNameAddDetail | MediaConflictAddDetail | MediaUsageAddDetail

class AddErrorDetails(TypedDict):
    error_kind: str
    causes: list[str]
    source_details: list[dict[str, Any]]
    context: AddContext
    detail: AddDetail | None

DetailsT = TypeVar("DetailsT", bound=Mapping[str, Any])

class ForgeError(Exception, Generic[DetailsT]):
    def __init__(self, code: str, message: str, *, kind: str, details: DetailsT) -> None:
        super().__init__(message)
        self.code = code
        self.kind = kind
        self.details = details
        self.causes = tuple(details.get("causes", ()))
        self.source_details = tuple(deepcopy(details.get("source_details", ())))

class SchemaError(ForgeError[dict[str, Any]]): pass
class AddError(ForgeError[AddErrorDetails]): pass
class MediaError(ForgeError[dict[str, Any]]): pass
class ImageOcclusionError(ForgeError[dict[str, Any]]): pass
class TemplateBundleError(ForgeError[dict[str, Any]]): pass
class CompareError(ForgeError[dict[str, Any]]):
    @property
    def report(self) -> Any:
        from .report import BuildReport
        return BuildReport(self.details["report"])

class PolicyError(ForgeError[dict[str, Any]]): pass
class PersistError(ForgeError[dict[str, Any]]): pass
class BuildError(ForgeError[dict[str, Any]]):
    def snapshot(self) -> dict[str, Any]:
        return deepcopy(self.details["snapshot"])

    @property
    def report(self) -> Any:
        from .report import BuildReport
        return BuildReport(self.details["snapshot"]["report"])

class PreparedPublicationStateError(ForgeError[dict[str, Any]]):
    @property
    def reason(self) -> Literal["closed", "consumed"]:
        return "consumed" if self.details["reason"] == "consumed" else "closed"
