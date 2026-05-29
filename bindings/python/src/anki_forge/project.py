from __future__ import annotations

from dataclasses import dataclass, field

from .diagnostics import ValidationError
from .media import MediaRegistry
from .note import Note
from .notetype import NoteType, _validate_non_empty, _validate_optional_non_empty

_RESERVED_NOTE_TYPE_IDS = {"basic", "cloze"}


@dataclass
class Project:
    name: str
    stable_id: str | None = None
    default_deck: str | None = None
    media: MediaRegistry = field(default_factory=MediaRegistry)
    notetypes: dict[str, NoteType] = field(default_factory=dict)
    notetype_order: list[str] = field(default_factory=list)
    notes: list[Note] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.name = _validate_non_empty(self.name, "project name")
        self.stable_id = _validate_optional_non_empty(self.stable_id, "stable id")
        self.default_deck = _validate_optional_non_empty(self.default_deck, "default deck")

    def add_notetype(self, note_type: NoteType) -> Project:
        if note_type.id in _RESERVED_NOTE_TYPE_IDS:
            raise ValidationError(f"custom note type id is reserved: {note_type.id}")
        if note_type.id in self.notetypes:
            raise ValidationError(f"duplicate note type id: {note_type.id}")
        self.notetypes[note_type.id] = note_type
        self.notetype_order.append(note_type.id)
        return self

    def add_note(self, note: Note) -> Project:
        if note.note_type_id not in _RESERVED_NOTE_TYPE_IDS and note.note_type_id not in self.notetypes:
            raise ValidationError(f"unknown note type id: {note.note_type_id}")
        if note.note_type_id in self.notetypes:
            self._validate_custom_note_field_keys(note, self.notetypes[note.note_type_id])
        self.notes.append(note)
        return self

    def _validate_custom_note_field_keys(self, note: Note, note_type: NoteType) -> None:
        allowed = {field.key for field in note_type.fields}
        for field_key in note.fields:
            if field_key not in allowed:
                raise ValidationError(f"unknown field key for {note_type.id}: {field_key}")
