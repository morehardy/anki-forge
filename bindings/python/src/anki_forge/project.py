from __future__ import annotations

from dataclasses import asdict, replace
import json
from os import PathLike
from pathlib import Path
from types import MappingProxyType
from typing import Mapping, Protocol

from . import _native
from ._buildable import _Buildable
from .deck import Deck
from .artifact import ApkgArtifact
from ._bridge import invoke
from .media import MediaRegistry
from .note import FieldContent, Note
from .identity import IdentityRecipe
from .notetype import Field, GenerationRule, NoteType, Template, _validate_non_empty, _validate_optional_non_empty
from .report import BuildReport, ProjectDiffReport, ValidationReport, _diagnostics
from .options import BuildOptions, InspectLimits, PathInput, RiskLevelValue, UpdateSafetyValue, absolute_path

_STOCK_NAMES = {
    "basic": {"front": "Front", "back": "Back"},
    "cloze": {"text": "Text", "back_extra": "Back Extra"},
    "image_occlusion": {"occlusion": "Occlusion", "image": "Image", "header": "Header", "back_extra": "Back Extra", "comments": "Comments"},
}


class Project(_Buildable):
    _handle: _native.NativeProject

    """A Rust Project; additions validate and snapshot authoring inputs."""

    def __init__(
        self, name: str, stable_id: str | None = None, default_deck: str | None = None,
        *, base_dir: str | PathLike[str] | None = None,
    ) -> None:
        self._name = _validate_non_empty(name, "project name")
        self._stable_id = _validate_optional_non_empty(stable_id, "stable id")
        self._default_deck = _validate_optional_non_empty(default_deck, "default deck")
        self._base_dir = Path(base_dir or Path.cwd()).expanduser().absolute()
        self._handle = _native.NativeProject(self.name, self.stable_id, self.default_deck)
        self._media = MediaRegistry(_project=self._handle, base_dir=self.base_dir)

    @property
    def name(self) -> str:
        return self._name

    @property
    def stable_id(self) -> str | None:
        return self._stable_id

    @property
    def default_deck(self) -> str | None:
        return self._default_deck

    @property
    def base_dir(self) -> Path:
        return self._base_dir

    @property
    def media(self) -> MediaRegistry:
        return self._media

    @classmethod
    def from_deck(cls, deck: Deck) -> Project:
        """Convert a native Deck snapshot while retaining the original Deck."""
        project = cls.__new__(cls)
        project._name, project._stable_id = deck.name, deck.stable_id
        project._default_deck, project._base_dir = deck.name, deck.base_dir
        project._handle = invoke(deck._handle.to_project)
        project._media = MediaRegistry(_project=project._handle, base_dir=project.base_dir)
        return project

    @property
    def notes(self) -> tuple[Note, ...]:
        values = json.loads(invoke(self._handle.notes))
        notes = []
        for value in values:
            names = {name: key for key, name in _STOCK_NAMES.get(value["note_type_id"], {}).items()}
            note = Note(value["note_type_id"], stable_id=value["stable_id"], deck_name=value["deck_name"])
            note.fields = {names.get(key, key): FieldContent(**content) for key, content in value["fields"].items()}
            note.tag_values = value["tags"]
            if value["identity"] is not None:
                note.identity(value["identity"])
            notes.append(note)
        return tuple(notes)

    @property
    def notetypes(self) -> Mapping[str, NoteType]:
        values = json.loads(invoke(self._handle.notetypes))
        types = {}
        for value in values:
            fields = []
            for field_value in value.pop("fields"):
                if field_value.pop("key_auto_derived"):
                    field_value["key"] = None
                fields.append(Field(**field_value))
            templates = []
            for template_value in value.pop("templates"):
                rule = template_value["generate_when"]
                if "fields" in rule:
                    rule["fields"] = tuple(rule["fields"])
                template_value["generate_when"] = GenerationRule(**rule)
                templates.append(Template(**template_value))
            identity = value.pop("identity")
            note_type = NoteType(**value, fields=fields, templates=templates)
            if identity is not None:
                note_type.identity(IdentityRecipe.fields(identity))
            types[note_type.id] = note_type
        return MappingProxyType(types)

    @property
    def notetype_order(self) -> tuple[str, ...]:
        return tuple(self.notetypes)

    def add_notetype(self, note_type: NoteType) -> Project:
        templates = []
        for template in note_type.templates:
            value = asdict(template)
            rule = template.generate_when
            value["generate_when"] = {"kind": rule.kind if rule is not None else "anki_default"}
            if rule is not None and rule.kind in {"all", "any"}:
                value["generate_when"]["fields"] = list(rule.fields)
            elif rule is not None and rule.kind == "cloze":
                value["generate_when"]["field"] = rule.field
            templates.append(value)
        payload = {
            "id": note_type.id, "name": note_type.name, "cloze_field": note_type.cloze_field,
            "fields": [asdict(field) for field in note_type.fields],
            "templates": templates, "css": note_type.css_value,
            "identity": list(note_type.identity_value.field_keys) if note_type.identity_value is not None else None,
        }
        invoke(self._handle.add_notetype, json.dumps(payload, ensure_ascii=False))
        return self

    def add_note(self, note: Note) -> Project:
        names = _STOCK_NAMES.get(note.note_type_id, {})
        payload = {
            "note_type_id": note.note_type_id,
            "stable_id": note.stable_id,
            "deck_name": note.deck_name,
            "tags": list(note.tag_values),
            "identity": list(note.identity_value.field_keys) if note.identity_value is not None else None,
            "fields": {names.get(key, key): {"kind": content.kind, "value": content.export_as if content.reference is not None else content.value} for key, content in note.fields.items()},
        }
        references = [content.reference._handle for content in note.fields.values() if content.reference is not None]
        invoke(self._handle.add_note, json.dumps(payload, ensure_ascii=False), references)
        return self

    def validate(self) -> ValidationReport:
        return ValidationReport(_diagnostics(json.loads(invoke(self._handle.validate))))

    def import_template_bundle(self, path: PathInput) -> Project:
        """Import a core template bundle atomically, relative to base_dir."""
        invoke(self._handle.import_template_bundle, absolute_path(path, self.base_dir))
        return self

    def diff_against_apkg(self, path: PathInput, *, inspect_limits: InspectLimits | None = None) -> ProjectDiffReport:
        """Compare using a temporary candidate, without publishing an APKG or lockfile."""
        limits = asdict(inspect_limits) if inspect_limits is not None else {}
        return ProjectDiffReport.from_json(json.loads(invoke(
            self._handle.diff_against_apkg, absolute_path(path, self.base_dir), json.dumps(limits),
        )))
