from __future__ import annotations

from types import MappingProxyType
from dataclasses import dataclass, field
from typing import Mapping, Sequence

from .diagnostics import ValidationError
from .media import MediaRegistry
from .note import Note
from .notetype import NoteType, _validate_non_empty, _validate_optional_non_empty

STOCK_NOTE_TYPE_IDS = {"basic", "cloze"}


@dataclass
class Project:
    name: str
    stable_id: str | None = None
    default_deck: str | None = None
    media: MediaRegistry = field(default_factory=MediaRegistry)
    _note_types: dict[str, NoteType] = field(default_factory=dict)
    _note_type_order: list[str] = field(default_factory=list)
    _notes: list[Note] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.name = _validate_non_empty(self.name, "project name")
        self.stable_id = _validate_optional_non_empty(self.stable_id, "stable id")
        self.default_deck = _validate_optional_non_empty(self.default_deck, "default deck")

    @property
    def notetypes(self) -> Mapping[str, NoteType]:
        return MappingProxyType(self._note_types)

    @property
    def notetype_order(self) -> Sequence[str]:
        return tuple(self._note_type_order)

    @property
    def notes(self) -> Sequence[Note]:
        return tuple(self._notes)

    def add_notetype(self, note_type: NoteType) -> Project:
        note_type.validate()
        if note_type.id in STOCK_NOTE_TYPE_IDS:
            raise ValidationError(f"custom note type id is reserved: {note_type.id}")
        if note_type.id in self._note_types:
            raise ValidationError(f"duplicate note type id: {note_type.id}")
        self._note_types[note_type.id] = note_type
        self._note_type_order.append(note_type.id)
        return self

    def add_note(self, note: Note) -> Project:
        if note.note_type_id not in STOCK_NOTE_TYPE_IDS and note.note_type_id not in self._note_types:
            raise ValidationError(f"unknown note type id: {note.note_type_id}")
        if note.note_type_id in self._note_types:
            self._validate_custom_note_field_keys(note, self._note_types[note.note_type_id])
        self._notes.append(note)
        return self

    def to_product_document(self) -> dict[str, object]:
        from .product_json import (
            basic_stock_notetype_json,
            cloze_stock_notetype_json,
            custom_notetype_json,
            media_to_json,
            note_to_json,
        )

        self._validate_notes_for_serialization()
        note_types: list[dict[str, object]] = []
        for note_type_id in self._stock_note_types():
            if note_type_id == "basic":
                note_types.append(basic_stock_notetype_json())
            elif note_type_id == "cloze":
                note_types.append(cloze_stock_notetype_json())
        note_types.extend(custom_notetype_json(self._note_types[note_type_id]) for note_type_id in self._note_type_order)

        return {
            "product_document_version": "product-v2",
            "document_id": self.stable_id or self.name,
            "default_deck_name": self.default_deck,
            "note_types": note_types,
            "notes": [note_to_json(note, index, self._resolve_deck(note)) for index, note in enumerate(self._notes)],
            "media": [media_to_json(item) for item in self.media.items],
        }

    def _stock_note_types(self) -> list[str]:
        used = {note.note_type_id for note in self._notes}
        return [note_type_id for note_type_id in ("basic", "cloze") if note_type_id in used]

    def _resolve_deck(self, note: Note) -> str:
        return note.deck_name or self.default_deck or self.name

    def _validate_notes_for_serialization(self) -> None:
        for note_type_id in self._note_type_order:
            self._note_types[note_type_id].validate()

        seen_stable_ids: set[str] = set()
        for note in self._notes:
            if note.stable_id is not None:
                if note.stable_id in seen_stable_ids:
                    raise ValidationError(f"duplicate note stable_id: {note.stable_id}")
                seen_stable_ids.add(note.stable_id)

            if note.note_type_id in self._note_types:
                note_type = self._note_types[note.note_type_id]
                self._validate_custom_note_field_keys(note, note_type)
                has_identity_fields = any(field.identity for field in note_type.fields)
                if note.stable_id is None and not has_identity_fields:
                    raise ValidationError(f"custom note type {note_type.id} needs identity fields or stable_id")

    def _validate_custom_note_field_keys(self, note: Note, note_type: NoteType) -> None:
        allowed = {field.key for field in note_type.fields}
        for field_key in note.fields:
            if field_key not in allowed:
                raise ValidationError(f"unknown field key for {note_type.id}: {field_key}")
