from __future__ import annotations

from dataclasses import dataclass, field
import json
from typing import Any, Iterable, Mapping

from . import _native
from ._bridge import invoke
from .diagnostics import ValidationError
from .content import Content
from .content import Content as FieldContent
from .media import MediaRef
from .identity import IdentityRecipe
from .notetype import _validate_id, _validate_optional_non_empty, _validate_source, _validate_tag

_STOCK_NAMES = {
    "basic": {"front": "Front", "back": "Back"},
    "cloze": {"text": "Text", "back_extra": "Back Extra"},
    "image_occlusion": {"occlusion": "Occlusion", "image": "Image", "header": "Header", "back_extra": "Back Extra", "comments": "Comments"},
}
_STOCK_KEYS = {kind: {name: key for key, name in names.items()} for kind, names in _STOCK_NAMES.items()}


@dataclass
class Note:
    note_type_id: str
    stable_id: str | None = None
    deck_name: str | None = None
    fields: dict[str, FieldContent] = field(default_factory=dict)
    tag_values: list[str] = field(default_factory=list)
    identity_value: IdentityRecipe | None = None

    def __post_init__(self) -> None:
        self.note_type_id = _validate_id(self.note_type_id, "note type id")
        self.stable_id = _validate_optional_non_empty(self.stable_id, "stable id")
        self.deck_name = _validate_optional_non_empty(self.deck_name, "deck name")
        values, self.fields = self.fields, {}
        for key, content in values.items():
            self.field(key, content)

    @classmethod
    def _from_native(cls, value: Mapping[str, Any]) -> Note:
        note = cls(value["note_type_id"], stable_id=value["stable_id"], deck_name=value["deck_name"])
        for key, content in value["fields"].items():
            note.field(key, FieldContent(**content))
        note.tag_values = value["tags"]
        if value["identity"] is not None:
            note.identity(value["identity"])
        return note

    @classmethod
    def basic(
        cls,
        front: str,
        back: str,
        *,
        stable_id: str | None = None,
        deck_name: str | None = None,
    ) -> Note:
        return cls("basic", stable_id=stable_id, deck_name=deck_name).text("front", front).text("back", back)

    @classmethod
    def cloze(
        cls,
        text: str,
        back_extra: str | None = None,
        *,
        stable_id: str | None = None,
        deck_name: str | None = None,
    ) -> Note:
        return cls("cloze", stable_id=stable_id, deck_name=deck_name).html("text", text).text("back_extra", back_extra or "")

    @classmethod
    def image_occlusion(
        cls,
        image: MediaRef,
        *,
        stable_id: str | None = None,
        deck_name: str | None = None,
    ) -> ImageOcclusionNoteBuilder:
        return ImageOcclusionNoteBuilder(image, stable_id=stable_id, deck_name=deck_name)

    def text(self, key: str, value: str) -> Note:
        if not isinstance(value, str):
            raise ValidationError("text field value must be a string")
        return self._set_field(key, "text", value)

    def field(self, key: str, content: Content) -> Note:
        if not isinstance(content, Content):
            raise ValidationError("field content must be Content")
        key = _validate_source(key, "field key")
        key = _STOCK_KEYS.get(self.note_type_id, {}).get(key, key)
        name = _STOCK_NAMES.get(self.note_type_id, {}).get(key)
        if name is not None:
            self.fields.pop(name, None)
        self.fields[key] = content
        return self

    def html(self, key: str, value: str) -> Note:
        if not isinstance(value, str):
            raise ValidationError("html field value must be a string")
        return self._set_field(key, "html", value)

    def sound(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "sound", None, reference=ref)

    def image(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "image", None, reference=ref)

    def tag(self, tag: str) -> Note:
        normalized = _validate_tag(tag)
        if normalized not in self.tag_values:
            self.tag_values.append(normalized)
        return self

    def tags(self, tags: Iterable[str]) -> Note:
        for tag in tags:
            self.tag(tag)
        return self

    def deck(self, deck_name: str | None) -> Note:
        self.deck_name = _validate_optional_non_empty(deck_name, "deck name")
        return self

    def identity(self, keys: Iterable[str]) -> Note:
        self.identity_value = IdentityRecipe.fields(keys)
        return self

    def _set_field(
        self, key: str, kind: str, value: str | None, *,
        reference: MediaRef | None = None,
    ) -> Note:
        return self.field(key, FieldContent(
            kind=kind, value=value, reference=reference,
            media_id=reference.media_id if reference is not None else None,
            export_as=reference.export_as if reference is not None else None,
        ))


class ImageOcclusionNoteBuilder:
    def __init__(self, image: MediaRef, *, stable_id: str | None = None, deck_name: str | None = None) -> None:
        self._image = image
        self._stable_id = _validate_optional_non_empty(stable_id, "stable id")
        self._deck_name = _validate_optional_non_empty(deck_name, "deck name")
        self._mode = "hide_all_guess_one"
        self._rects: list[tuple[int, int, int, int]] = []
        self._header = ""
        self._back_extra = ""
        self._comments = ""
        self._tags: list[str] = []

    def mode(self, mode: str) -> ImageOcclusionNoteBuilder:
        if mode not in {"hide_all_guess_one", "hide_one_guess_one"}:
            raise ValidationError(f"unknown image occlusion mode: {mode}")
        self._mode = mode
        return self

    def rect(self, x: int, y: int, width: int, height: int) -> ImageOcclusionNoteBuilder:
        values = (x, y, width, height)
        if not all(type(value) is int and 0 <= value <= 2**32 - 1 for value in values):
            raise ValidationError("image occlusion rect values must be u32 integers")
        self._rects.append(values)
        return self

    def rects(self, rects: Iterable[tuple[int, int, int, int]]) -> ImageOcclusionNoteBuilder:
        for x, y, width, height in rects:
            self.rect(x, y, width, height)
        return self

    def header(self, value: str) -> ImageOcclusionNoteBuilder:
        self._header = value
        return self

    def back_extra(self, value: str) -> ImageOcclusionNoteBuilder:
        self._back_extra = value
        return self

    def comments(self, value: str) -> ImageOcclusionNoteBuilder:
        self._comments = value
        return self

    def tag(self, tag: str) -> ImageOcclusionNoteBuilder:
        normalized = _validate_tag(tag)
        if normalized not in self._tags:
            self._tags.append(normalized)
        return self

    def tags(self, tags: Iterable[str]) -> ImageOcclusionNoteBuilder:
        for tag in tags:
            self.tag(tag)
        return self

    def build(self) -> Note:
        payload = {
            "stable_id": self._stable_id, "deck_name": self._deck_name, "mode": self._mode,
            "rects": self._rects, "header": self._header, "back_extra": self._back_extra,
            "comments": self._comments, "tags": self._tags,
        }
        value = json.loads(invoke(_native.build_image_occlusion, json.dumps(payload, ensure_ascii=False), self._image._handle))
        return Note._from_native(value)
