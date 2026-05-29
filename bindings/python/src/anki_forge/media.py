from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path, PurePath
from typing import Sequence

from .diagnostics import ValidationError
from .notetype import _ASCII_CONTROL, _validate_non_empty


@dataclass(frozen=True)
class MediaRef:
    media_id: str
    export_as: str

    def __post_init__(self) -> None:
        object.__setattr__(self, "media_id", _validate_non_empty(self.media_id, "media id"))
        object.__setattr__(self, "export_as", _validate_export_as(self.export_as))


@dataclass(frozen=True)
class MediaItem:
    ref: MediaRef
    source_kind: str
    source_label: str
    data: bytes | None
    path: Path | None
    length: int | None
    sha256: str | None


class MediaRegistry:
    def __init__(self) -> None:
        self._items: list[MediaItem] = []
        self._items_by_export: dict[str, MediaItem] = {}
        self._items_by_path_export: dict[tuple[Path, str], MediaItem] = {}
        self._next_id = 1

    @property
    def items(self) -> Sequence[MediaItem]:
        return tuple(self._items)

    def add_bytes(self, *, source_label: str, data: bytes | bytearray, export_as: str) -> MediaRef:
        label = _validate_non_empty(source_label, "source label")
        payload = bytes(data)
        export_name = _validate_export_as(export_as)
        digest = hashlib.sha256(payload).hexdigest()
        existing = self._items_by_export.get(export_name)
        if existing is not None:
            if existing.length == len(payload) and existing.sha256 == digest:
                return existing.ref
            raise ValidationError(f"media export name already exists with different content: {export_name}")
        ref = self._new_ref(export_name)
        item = MediaItem(
            ref=ref,
            source_kind="bytes",
            source_label=label,
            data=payload,
            path=None,
            length=len(payload),
            sha256=digest,
        )
        self._items.append(item)
        self._items_by_export[export_name] = item
        return ref

    def add_file(self, path: str | Path, *, export_as: str | None = None) -> MediaRef:
        media_path = Path(path).expanduser().resolve()
        export_name = _validate_export_as(export_as or media_path.name)
        key = (media_path, export_name)
        existing_for_path = self._items_by_path_export.get(key)
        if existing_for_path is not None:
            return existing_for_path.ref
        existing = self._items_by_export.get(export_name)
        if existing is not None:
            raise ValidationError(f"media export name already exists: {export_name}")
        ref = self._new_ref(export_name)
        item = MediaItem(
            ref=ref,
            source_kind="file",
            source_label=str(media_path),
            data=None,
            path=media_path,
            length=None,
            sha256=None,
        )
        self._items.append(item)
        self._items_by_export[export_name] = item
        self._items_by_path_export[key] = item
        return ref

    def _new_ref(self, export_as: str) -> MediaRef:
        ref = MediaRef(f"media:{self._next_id:06d}", export_as)
        self._next_id += 1
        return ref


def _validate_export_as(value: str) -> str:
    export_as = _validate_non_empty(value, "export name")
    if any(char in _ASCII_CONTROL for char in export_as):
        raise ValidationError("export name must not contain ASCII control characters")
    if "%" in export_as:
        raise ValidationError("export name must not contain percent signs")
    if export_as in {".", ".."}:
        raise ValidationError("export name must be a filename")
    if "/" in export_as or "\\" in export_as:
        raise ValidationError("export name must be a bare filename")
    path = PurePath(export_as)
    if path.is_absolute() or any(part == ".." for part in path.parts) or len(path.parts) != 1:
        raise ValidationError("export name must be a bare filename")
    return export_as
