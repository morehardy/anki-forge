from __future__ import annotations
from dataclasses import dataclass, replace
from enum import StrEnum
from collections.abc import Iterable
from . import _native
from ._bridge import invoke
from .content import Content, _content
from .media import Media
from .notetype import NoteType

class OcclusionMode(StrEnum):
    HIDE_ALL_GUESS_ONE = "hide_all_guess_one"
    HIDE_ONE_GUESS_ONE = "hide_one_guess_one"

@dataclass(frozen=True)
class Mask:
    key: str
    x: float
    y: float
    width: float
    height: float

    @staticmethod
    def rect(key: str, x: float, y: float, width: float, height: float) -> Mask:
        return Mask(key, x, y, width, height)

@dataclass(frozen=True)
class Note:
    """A note owning its model and content; stable identity is supplied to Project.add."""
    _handle: _native.NativeNote

    @staticmethod
    def basic(front: str | Content, back: str | Content) -> Note:
        return Note(_native.NativeNote.basic(_content(front)._handle, _content(back)._handle))

    @staticmethod
    def cloze(text: str | Content) -> Note:
        return Note(_native.NativeNote.cloze(_content(text)._handle))

    @staticmethod
    def image_occlusion(image: Media) -> ImageOcclusionBuilder:
        return ImageOcclusionBuilder(image)

    def field(self, key: str, value: str | Content) -> Note:
        return Note(self._handle.field(key, _content(value)._handle))

    def deck(self, name: str) -> Note:
        return Note(self._handle.deck(name))

    def tag(self, tag: str) -> Note:
        return self.tags([tag])

    def tags(self, tags: Iterable[str]) -> Note:
        return Note(self._handle.tags(list(tags)))

    @property
    def note_type(self) -> NoteType:
        return NoteType(self._handle.note_type())

@dataclass(frozen=True)
class ImageOcclusionBuilder:
    _image: Media
    _masks: tuple[Mask, ...] = ()
    _mode: OcclusionMode = OcclusionMode.HIDE_ALL_GUESS_ONE

    def mask(self, mask: Mask) -> ImageOcclusionBuilder:
        return replace(self, _masks=(*self._masks, mask))

    def mode(self, mode: OcclusionMode) -> ImageOcclusionBuilder:
        return replace(self, _mode=mode)

    def build(self) -> Note:
        masks = [(m.key, m.x, m.y, m.width, m.height) for m in self._masks]
        return Note(invoke(_native.NativeNote.image_occlusion, self._image._handle, masks, self._mode))
