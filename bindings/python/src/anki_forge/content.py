from __future__ import annotations

from dataclasses import dataclass
import json
from typing import TYPE_CHECKING

from . import _native
from ._bridge import invoke
from .diagnostics import ValidationError

if TYPE_CHECKING:
    from .media import MediaRef


@dataclass(frozen=True)
class Content:
    """Typed input; escaping and media markup are rendered by the core."""

    kind: str
    value: str | None = None
    media_id: str | None = None
    export_as: str | None = None
    reference: MediaRef | None = None

    @classmethod
    def text(cls, value: str) -> Content:
        if not isinstance(value, str):
            raise ValidationError("text content must be a string")
        return cls("text", value=value)

    @classmethod
    def html(cls, value: str) -> Content:
        if not isinstance(value, str):
            raise ValidationError("html content must be a string")
        return cls("html", value=value)

    def render(self) -> str:
        reference = self.reference
        payload = {"kind": self.kind, "value": reference.export_as if reference is not None else self.value}
        return invoke(_native.render_content, json.dumps(payload, ensure_ascii=False),
                      [reference._handle] if reference is not None else [])
