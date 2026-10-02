"""Translate native failures while retaining machine identifiers and source chains."""
from __future__ import annotations
import json
from collections.abc import Callable
from typing import ParamSpec, TypeVar
from . import _native
from .diagnostics import (ForgeError, SchemaError, AddError, MediaError, ImageOcclusionError,
                          TemplateBundleError, CompareError, PolicyError, PersistError, BuildError)
P = ParamSpec("P")
T = TypeVar("T")
def invoke(operation: Callable[P, T], *args: P.args, **kwargs: P.kwargs) -> T:
    try:
        return operation(*args, **kwargs)
    except _native.OperationError as error:
        value = json.loads(str(error))
        details = value["details"]
        cls = {"schema": SchemaError, "add": AddError, "media": MediaError,
               "note": ImageOcclusionError, "bundle": TemplateBundleError,
               "compare": CompareError, "policy": PolicyError, "persist": PersistError,
               "build": BuildError}.get(value["kind"], ForgeError)
        raise cls(value["code"], value["message"],
                  kind=details.get("error_kind", value["kind"]), details=details) from error
