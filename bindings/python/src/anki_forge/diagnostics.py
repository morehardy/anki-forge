"""Structured failures from the Rust public API."""
from __future__ import annotations
from copy import deepcopy
from typing import Any

class ForgeError(Exception):
    def __init__(self, code: str, message: str, *, kind: str, details: dict[str, Any]) -> None:
        super().__init__(message)
        self.code = code
        self.kind = kind
        self.details = details
        self.causes = tuple(details.get("causes", ()))
        self.source_details = tuple(deepcopy(details.get("source_details", ())))

class SchemaError(ForgeError): pass
class AddError(ForgeError): pass
class MediaError(ForgeError): pass
class ImageOcclusionError(ForgeError): pass
class TemplateBundleError(ForgeError): pass
class CompareError(ForgeError):
    @property
    def report(self) -> Any:
        from .report import BuildReport
        return BuildReport(self.details["report"])

class PolicyError(ForgeError): pass
class PersistError(ForgeError): pass
class BuildError(ForgeError):
    def snapshot(self) -> dict[str, Any]:
        return deepcopy(self.details["snapshot"])

    @property
    def report(self) -> Any:
        from .report import BuildReport
        return BuildReport(self.details["snapshot"]["report"])
