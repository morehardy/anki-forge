from __future__ import annotations
from dataclasses import dataclass
from . import _native
from ._bridge import invoke
from .content import Content
from .options import PathInput, absolute_path

@dataclass(frozen=True)
class MediaLimits:
    max_bytes: int = 256 << 20

@dataclass(frozen=True)
class Media:
    """An immutable owned snapshot, reusable across notes and projects."""
    _handle: _native.NativeMedia

    @staticmethod
    def file(path: PathInput, *, limits: MediaLimits = MediaLimits()) -> Media:
        return Media(invoke(_native.NativeMedia.file, absolute_path(path), limits.max_bytes))

    @staticmethod
    def bytes(data: bytes, media_type: str, *, limits: MediaLimits = MediaLimits()) -> Media:
        return Media(invoke(_native.NativeMedia.bytes, data, media_type, limits.max_bytes))

    def with_export_name(self, name: str) -> Media:
        return Media(invoke(self._handle.with_export_name, name))

    @property
    def filename(self) -> str:
        return self._handle.filename()

    @property
    def media_type(self) -> str:
        return self._handle.media_type()

    def __len__(self) -> int:
        return self._handle.size()

    def image(self) -> Content:
        return Content(self._handle.image())

    def sound(self) -> Content:
        return Content(self._handle.sound())
