"""Fail at import with a useful error if an installation has mixed binaries."""
from __future__ import annotations

from dataclasses import dataclass
import importlib
import json

__version__ = "0.2.0"
_CORE_API_VERSION = "0.2.0"
_CONTRACT_VERSION = "2.1.0"


@dataclass(frozen=True)
class Versions:
    binding_version: str
    core_version: str
    contract_version: str


def _load() -> Versions:
    try:
        native = importlib.import_module("ankiforge._native")
    except (ImportError, OSError) as error:
        raise ImportError(
            "BINDING.EXTENSION_UNAVAILABLE: cannot load the ankiforge native extension; "
            "reinstall a compatible ankiforge wheel for CPython 3.11+ and your platform. "
            "A source checkout requires `maturin develop` first."
        ) from error
    try:
        value = json.loads(native.binding_metadata())
        if value.pop("binding_protocol_version", None) != 1:
            raise ValueError("expected native binding protocol 1 with prepared publication support")
        metadata = Versions(**value)
        if metadata.binding_version != __version__ or metadata.core_version != _CORE_API_VERSION:
            raise ValueError(f"expected binding {__version__}, core {_CORE_API_VERSION}; found {metadata}")
        if metadata.contract_version != _CONTRACT_VERSION:
            raise ValueError(f"expected contract {_CONTRACT_VERSION}; found {metadata.contract_version}")
    except (AttributeError, TypeError, ValueError) as error:
        raise ImportError(f"BINDING.VERSION_MISMATCH: {error}; reinstall ankiforge to replace mixed package files") from error
    return metadata


_VERSIONS = _load()


def versions() -> Versions:
    """Versions of this Python binding, the linked core API and embedded contract."""
    return _VERSIONS
