from __future__ import annotations

from dataclasses import replace
import json
from os import PathLike
from pathlib import Path

from . import _native
from .artifact import ApkgArtifact
from ._bridge import invoke
from .native_media import MediaRegistry
from .note import Note
from .notetype import _validate_non_empty, _validate_optional_non_empty
from .report import BuildReport

_STOCK_NAMES = {
    "basic": {"front": "Front", "back": "Back"},
    "cloze": {"text": "Text", "back_extra": "Back Extra"},
    "image_occlusion": {"occlusion": "Occlusion", "image": "Image", "header": "Header", "back_extra": "Back Extra", "comments": "Comments"},
}


class Project:
    """A Rust Project; additions validate and snapshot authoring inputs."""

    def __init__(
        self, name: str, stable_id: str | None = None, default_deck: str | None = None,
        *, base_dir: str | PathLike[str] | None = None,
    ) -> None:
        self.name = _validate_non_empty(name, "project name")
        self.stable_id = _validate_optional_non_empty(stable_id, "stable id")
        self.default_deck = _validate_optional_non_empty(default_deck, "default deck")
        self.base_dir = Path(base_dir or Path.cwd()).expanduser().absolute()
        self._handle = _native.NativeProject(self.name, self.stable_id, self.default_deck)
        self.media = MediaRegistry(_project=self._handle, base_dir=self.base_dir)

    def add_note(self, note: Note) -> Project:
        names = _STOCK_NAMES.get(note.note_type_id, {})
        payload = {
            "note_type_id": note.note_type_id,
            "stable_id": note.stable_id,
            "deck_name": note.deck_name,
            "tags": list(note.tag_values),
            "fields": {names.get(key, key): {"kind": content.kind, "value": content.export_as if content.reference is not None else content.value} for key, content in note.fields.items()},
        }
        references = [content.reference._handle for content in note.fields.values() if content.reference is not None]
        invoke(self._handle.add_note, json.dumps(payload, ensure_ascii=False), references)
        return self

    def build(self) -> BuildReport:
        return self._build(None)

    def write_apkg(self, path: str | PathLike[str]) -> BuildReport:
        target = Path(path).expanduser()
        if not target.is_absolute():
            target = self.base_dir / target
        return self._build(target)

    def _build(self, output: Path | None) -> BuildReport:
        payload, artifact = self._handle.build(output)
        report = BuildReport.from_json(json.loads(payload))
        return replace(report, artifact=ApkgArtifact(artifact) if artifact is not None else None)
