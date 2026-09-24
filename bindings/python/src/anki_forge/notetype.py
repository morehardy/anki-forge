from __future__ import annotations
from dataclasses import dataclass, replace, asdict
from collections.abc import Iterable
import json
from . import _native
from ._bridge import invoke
from .media import Media, MediaLimits
from .options import PathInput, absolute_path

@dataclass(frozen=True)
class Field:
    key: str
    name: str | None = None
    required: bool = False
    sort: bool = False

@dataclass(frozen=True)
class GenerationRule:
    kind: str = "anki_default"
    fields: tuple[str, ...] = ()

    @staticmethod
    def all(fields: Iterable[str]) -> GenerationRule:
        return GenerationRule("all", tuple(fields))

    @staticmethod
    def any(fields: Iterable[str]) -> GenerationRule:
        return GenerationRule("any", tuple(fields))

@dataclass(frozen=True)
class Template:
    key: str
    front: str
    back: str
    name: str | None = None
    browser_front: str | None = None
    browser_back: str | None = None
    target_deck: str | None = None
    generation: GenerationRule = GenerationRule()

@dataclass(frozen=True)
class NoteType:
    """A completed model validated by Rust; its definition cannot be mutated."""
    _handle: _native.NativeNoteType

    @staticmethod
    def builder(key: str) -> NoteTypeBuilder:
        return NoteTypeBuilder(key)

    @staticmethod
    def from_bundle(path: PathInput, *, limits: MediaLimits = MediaLimits()) -> NoteType:
        return NoteType(invoke(_native.NativeNoteType.from_bundle, absolute_path(path), limits.max_bytes))

    @property
    def key(self) -> str:
        return self._handle.key()

    @property
    def name(self) -> str:
        return self._handle.display_name()

    def note(self) -> Note:
        from .note import Note
        return Note(self._handle.note())

@dataclass(frozen=True)
class NoteTypeBuilder:
    _key: str
    _name: str | None = None
    _fields: tuple[Field, ...] = ()
    _templates: tuple[Template, ...] = ()
    _css: str = ""
    _cloze_field: str | None = None
    _assets: tuple[Media, ...] = ()

    def name(self, value: str) -> NoteTypeBuilder:
        return replace(self, _name=value)

    def field(self, value: Field) -> NoteTypeBuilder:
        return replace(self, _fields=(*self._fields, value))

    def template(self, value: Template) -> NoteTypeBuilder:
        return replace(self, _templates=(*self._templates, value))

    def css(self, value: str) -> NoteTypeBuilder:
        return replace(self, _css=value)

    def cloze_field(self, key: str) -> NoteTypeBuilder:
        return replace(self, _cloze_field=key)

    def asset(self, value: Media) -> NoteTypeBuilder:
        return replace(self, _assets=(*self._assets, value))

    def build(self) -> NoteType:
        templates = [asdict(t) for t in self._templates]
        for t in templates:
            if t["generation"]["kind"] == "anki_default":
                t["generation"].pop("fields")
        value = dict(key=self._key, name=self._name, fields=[asdict(f) for f in self._fields],
                     templates=templates, css=self._css, cloze_field=self._cloze_field)
        return NoteType(invoke(_native.NativeNoteType.build, json.dumps(value),
                               [m._handle for m in self._assets]))

from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from .note import Note
