from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable

from .notetype import _validate_source


@dataclass(frozen=True)
class IdentityRecipe:
    """An identity recipe whose resolution and hashing are performed by Rust."""

    field_keys: tuple[str, ...]

    @classmethod
    def fields(cls, keys: Iterable[str]) -> IdentityRecipe:
        return cls(tuple(_validate_source(key, "identity field key") for key in keys))
