from __future__ import annotations

from os import PathLike
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .content import Content

from . import _native
from ._bridge import invoke
from .diagnostics import ValidationError


class MediaRef:
    """An opaque Rust reference identified by its export filename."""

    def __init__(self, handle: _native.NativeMediaRef) -> None:
        if not isinstance(handle, _native.NativeMediaRef):
            raise TypeError("MediaRef requires a registered media handle")
        self._handle = handle

    @property
    def export_as(self) -> str:
        return self._handle.filename

    @property
    def media_id(self) -> str:
        return f"media:{self.export_as}"

    def __eq__(self, other: object) -> bool:
        return isinstance(other, MediaRef) and self.export_as == other.export_as

    def __hash__(self) -> int:
        return hash(self.export_as)

    def image(self) -> Content:
        from .content import Content

        return Content("image", media_id=self.media_id, export_as=self.export_as, reference=self)

    def sound(self) -> Content:
        from .content import Content

        return Content("sound", media_id=self.media_id, export_as=self.export_as, reference=self)


class MediaRegistry:
    def __init__(self, *, _project: _native.NativeProject | None = None, base_dir: str | PathLike[str] | None = None) -> None:
        self._handle = _project if _project is not None else _native.NativeProject("Media")
        self._base_dir = Path(base_dir or Path.cwd()).expanduser().absolute()

    @staticmethod
    def inline_limit_bytes() -> int:
        return _native.inline_media_limit_bytes()

    def add_bytes(self, *, source_label: str, data: bytes | bytearray, export_as: str) -> MediaRef:
        if not isinstance(data, (bytes, bytearray)):
            raise ValidationError("media data must be bytes or bytearray")
        return MediaRef(invoke(self._handle.add_media_bytes, source_label, bytes(data), export_as))

    def add_file(self, path: str | PathLike[str], *, export_as: str | None = None) -> MediaRef:
        source = Path(path).expanduser()
        if not source.is_absolute():
            source = self._base_dir / source
        return MediaRef(invoke(self._handle.add_media_file, source, export_as if export_as is not None else source.name))
