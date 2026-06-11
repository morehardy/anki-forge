from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterable

from .diagnostics import ValidationError
from .media import MediaRef
from .notetype import _validate_id, _validate_optional_non_empty, _validate_tag

_STOCK_FIELD_KEYS = {
    "basic": {"front", "back"},
    "cloze": {"text", "back_extra"},
    "image_occlusion": {"occlusion", "image", "header", "back_extra", "comments"},
}


@dataclass(frozen=True)
class FieldContent:
    kind: str
    value: str | None = None
    media_id: str | None = None


@dataclass
class Note:
    note_type_id: str
    stable_id: str | None = None
    deck_name: str | None = None
    fields: dict[str, FieldContent] = field(default_factory=dict)
    tag_values: list[str] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.note_type_id = _validate_id(self.note_type_id, "note type id")
        self.stable_id = _validate_optional_non_empty(self.stable_id, "stable id")
        self.deck_name = _validate_optional_non_empty(self.deck_name, "deck name")

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

    def html(self, key: str, value: str) -> Note:
        if not isinstance(value, str):
            raise ValidationError("html field value must be a string")
        return self._set_field(key, "html", value)

    def sound(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "sound", None, media_id=ref.media_id)

    def image(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "image", None, media_id=ref.media_id)

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

    def _set_field(self, key: str, kind: str, value: str | None, *, media_id: str | None = None) -> Note:
        field_key = _validate_id(key, "field key")
        allowed_keys = _STOCK_FIELD_KEYS.get(self.note_type_id)
        if allowed_keys is not None and field_key not in allowed_keys:
            raise ValidationError(f"unknown field key for {self.note_type_id}: {field_key}")
        self.fields[field_key] = FieldContent(kind=kind, value=value, media_id=media_id)
        return self


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
        if not all(isinstance(value, int) for value in values):
            raise ValidationError("image occlusion rect values must be integers")
        if x < 0 or y < 0:
            raise ValidationError("image occlusion rect coordinates must be non-negative")
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
        if self._stable_id is None:
            raise ValidationError("image occlusion stable id is required")
        if not self._rects:
            raise ValidationError("image occlusion note requires at least one rect")
        seen: set[tuple[int, int, int, int]] = set()
        for rect in self._rects:
            _x, _y, width, height = rect
            if width <= 0 or height <= 0:
                raise ValidationError("image occlusion rect width and height must be positive")
            if rect in seen:
                raise ValidationError("duplicate image occlusion rect")
            seen.add(rect)
        prefix = "c1" if self._mode == "hide_all_guess_one" else "c1,2"
        occlusion = "".join(
            f"{{{{{prefix}::image-occlusion:rect:left={x}:top={y}:width={width}:height={height}}}}}<br>"
            for x, y, width, height in self._rects
        )
        note = (
            Note("image_occlusion", stable_id=self._stable_id, deck_name=self._deck_name)
            .html("occlusion", occlusion)
            .image("image", self._image)
            .text("header", self._header)
            .text("back_extra", self._back_extra)
            .text("comments", self._comments)
        )
        note.tags(self._tags)
        return note
