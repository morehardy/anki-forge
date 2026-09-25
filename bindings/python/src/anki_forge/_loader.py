"""Fail at import with a useful error if an installation has mixed binaries."""
from __future__ import annotations

from dataclasses import dataclass
import importlib
import json

__version__ = "0.2.0"
_CORE_API_VERSION = "0.2.0"


@dataclass(frozen=True)
class Versions:
    binding_version: str
    core_version: str
    contract_version: str


def _load() -> Versions:
    try:
        native = importlib.import_module("anki_forge._native")
    except (ImportError, OSError) as error:
        raise ImportError(
            "BINDING.EXTENSION_UNAVAILABLE: cannot load the anki-forge native extension; "
            "reinstall a compatible anki-forge wheel for CPython 3.11+ and your platform. "
            "A source checkout requires `maturin develop` first."
        ) from error
    try:
        value = json.loads(native.binding_metadata())
        metadata = Versions(**value)
        if metadata.binding_version != __version__ or metadata.core_version != _CORE_API_VERSION:
            raise ValueError(f"expected binding {__version__}, core {_CORE_API_VERSION}; found {metadata}")
        if not isinstance(metadata.contract_version, str) or not metadata.contract_version:
            raise ValueError("missing embedded contract version")
    except (AttributeError, TypeError, ValueError) as error:
        raise ImportError(f"BINDING.VERSION_MISMATCH: {error}; reinstall anki-forge to replace mixed package files") from error
    return metadata


_VERSIONS = _load()


def versions() -> Versions:
    """Versions of this Python binding, the linked core API and embedded contract."""
    return _VERSIONS
