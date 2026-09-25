from __future__ import annotations
from dataclasses import dataclass
from collections.abc import Iterable
from . import _native

@dataclass(frozen=True)
class Content:
    """Typed content; strings are text and HTML requires an explicit constructor."""
    _handle: _native.NativeContent

    @staticmethod
    def text(value: str) -> Content:
        return Content(_native.NativeContent.text(value))

    @staticmethod
    def html(value: str) -> Content:
        return Content(_native.NativeContent.html(value))

    @staticmethod
    def sequence(values: Iterable[str | Content]) -> Content:
        return Content(_native.NativeContent.sequence([_content(v)._handle for v in values]))

def _content(value: str | Content) -> Content:
    if isinstance(value, Content):
        return value
    if isinstance(value, str):
        return Content.text(value)
    raise TypeError("content must be a string or Content")
