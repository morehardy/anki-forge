from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterable

from .diagnostics import ValidationError
from .media import MediaRef
from .notetype import _validate_id, _validate_optional_non_empty, _validate_tag

_STOCK_FIELD_KEYS = {
    "basic": {"front", "back"},
    "cloze": {"text", "back_extra"},
}


@dataclass(frozen=True)
class FieldContent:
    kind: str
    value: str | MediaRef


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
        note = cls("cloze", stable_id=stable_id, deck_name=deck_name).text("text", text)
        if back_extra is not None:
            note.html("back_extra", back_extra)
        return note

    def text(self, key: str, value: str) -> Note:
        return self._set_field(key, "text", value)

    def html(self, key: str, value: str) -> Note:
        return self._set_field(key, "html", value)

    def sound(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "sound", ref)

    def image(self, key: str, ref: MediaRef) -> Note:
        return self._set_field(key, "image", ref)

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

    def _set_field(self, key: str, kind: str, value: str | MediaRef) -> Note:
        field_key = _validate_id(key, "field key")
        allowed_keys = _STOCK_FIELD_KEYS.get(self.note_type_id)
        if allowed_keys is not None and field_key not in allowed_keys:
            raise ValidationError(f"unknown field key for {self.note_type_id}: {field_key}")
        self.fields[field_key] = FieldContent(kind=kind, value=value)
        return self
