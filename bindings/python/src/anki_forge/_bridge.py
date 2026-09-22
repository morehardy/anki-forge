"""Translate private extension errors at the Python public boundary."""
from __future__ import annotations

from collections.abc import Callable
import json
from typing import ParamSpec, TypeVar

from . import _native
from .diagnostics import AuthoringError, DeckError, MediaError, ProductNoteError, ProjectAddError, TemplateBundleError
from .report import _diagnostics

P = ParamSpec("P")
T = TypeVar("T")


def invoke(operation: Callable[P, T], *args: P.args, **kwargs: P.kwargs) -> T:
    try:
        return operation(*args, **kwargs)
    except _native.OperationError as error:
        payload = json.loads(str(error))
        details = payload.get("details") or {}
        diagnostic = _diagnostics([details["diagnostic"]])[0] if "diagnostic" in details else None
        cls = {"add": ProjectAddError, "media": MediaError, "note": ProductNoteError, "bundle": TemplateBundleError, "deck": DeckError}.get(payload["kind"], AuthoringError)
        raise cls(payload["code"], payload["message"], diagnostic=diagnostic, details=details) from error
