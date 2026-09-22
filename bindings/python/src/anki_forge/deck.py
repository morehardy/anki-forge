from __future__ import annotations

from dataclasses import asdict, dataclass
from collections.abc import Iterable
import json
from pathlib import Path
from typing import Literal

from . import _native
from ._bridge import invoke
from ._buildable import _Buildable
from .diagnostics import ValidationError
from .options import PathInput, absolute_path
from .report import ValidationReport, _diagnostics

BasicIdentityField = Literal["front", "back"]
IoMode = Literal["hide_all_guess_one", "hide_one_guess_one"]


@dataclass(frozen=True)
class BasicIdentityOverride:
    fields: tuple[BasicIdentityField, ...]
    reason_code: str

    def __init__(self, fields: Iterable[BasicIdentityField], reason_code: str) -> None:
        object.__setattr__(self, "fields", tuple(fields))
        object.__setattr__(self, "reason_code", reason_code)


class DeckMediaRef:
    """A Deck media filename, distinct from Project's MediaRef."""

    def __init__(self, handle: _native.NativeDeckMediaRef) -> None:
        if not isinstance(handle, _native.NativeDeckMediaRef):
            raise TypeError("DeckMediaRef requires a registered Deck media handle")
        self._handle = handle

    @property
    def name(self) -> str:
        return self._handle.filename

    def __eq__(self, other: object) -> bool:
        return isinstance(other, DeckMediaRef) and self.name == other.name

    def __hash__(self) -> int:
        return hash(self.name)


class DeckMediaRegistry:
    def __init__(self, handle: _native.NativeDeck, base_dir: Path) -> None:
        self._handle = handle
        self._base_dir = base_dir

    def add_file(self, path: PathInput) -> DeckMediaRef:
        """Register a file under its basename, following Rust Deck semantics."""
        return DeckMediaRef(invoke(self._handle.add_media_file, absolute_path(path, self._base_dir)))

    def add_bytes(self, name: str, data: bytes | bytearray) -> DeckMediaRef:
        if not isinstance(data, (bytes, bytearray)):
            raise ValidationError("media data must be bytes or bytearray")
        return DeckMediaRef(invoke(self._handle.add_media_bytes, name, bytes(data)))

    def get(self, name: str) -> DeckMediaRef | None:
        handle = invoke(self._handle.get_media, name)
        return DeckMediaRef(handle) if handle is not None else None


class Deck(_Buildable):
    """The Rust stock-note Deck, with inferred identity and image geometry checks.

    Basic/Cloze strings follow Rust Deck's HTML semantics. Use Project for
    typed Content inputs and custom note types.
    """

    _handle: _native.NativeDeck

    def __init__(self, name: str, stable_id: str | None = None, *, basic_identity: Iterable[BasicIdentityField] | None = None, base_dir: PathInput | None = None) -> None:
        self._name = name
        self._stable_id = stable_id
        self._base_dir = Path(base_dir or Path.cwd()).expanduser().absolute()
        self._handle = invoke(_native.NativeDeck, name, json.dumps({"stable_id": stable_id, "basic_identity": list(basic_identity) if basic_identity is not None else None}))
        self._media = DeckMediaRegistry(self._handle, self.base_dir)

    @property
    def name(self) -> str:
        return self._name

    @property
    def stable_id(self) -> str | None:
        return self._stable_id

    @property
    def base_dir(self) -> Path:
        return self._base_dir

    @property
    def media(self) -> DeckMediaRegistry:
        return self._media

    def add_basic(self, front: str, back: str, *, stable_id: str | None = None, tags: Iterable[str] = (), identity_override: BasicIdentityOverride | None = None) -> Deck:
        invoke(self._handle.add_basic, front, back, json.dumps({
            "stable_id": stable_id, "tags": list(tags),
            "identity_override": asdict(identity_override) if identity_override is not None else None,
        }, ensure_ascii=False))
        return self

    def add_cloze(self, text: str, *, stable_id: str | None = None, extra: str = "", tags: Iterable[str] = ()) -> Deck:
        invoke(self._handle.add_cloze, text, json.dumps({"stable_id": stable_id, "extra": extra, "tags": list(tags)}, ensure_ascii=False))
        return self

    def add_image_occlusion(self, image: DeckMediaRef, *, rects: Iterable[tuple[int, int, int, int]], stable_id: str | None = None, mode: IoMode = "hide_all_guess_one", header: str = "", back_extra: str = "", comments: str = "", tags: Iterable[str] = ()) -> Deck:
        rectangles = [tuple(rect) for rect in rects]
        if any(len(rect) != 4 or not all(type(v) is int and 0 <= v <= 2**32 - 1 for v in rect) for rect in rectangles):
            raise ValidationError("image occlusion rect values must be u32 integers")
        invoke(self._handle.add_image_occlusion, image._handle, json.dumps({
            "stable_id": stable_id, "mode": mode, "rects": rectangles, "header": header,
            "back_extra": back_extra, "comments": comments, "tags": list(tags),
        }, ensure_ascii=False))
        return self

    def validate(self) -> ValidationReport:
        return ValidationReport(_diagnostics(json.loads(invoke(self._handle.validate))))
