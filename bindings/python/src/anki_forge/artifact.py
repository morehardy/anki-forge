from __future__ import annotations

from collections.abc import Iterator, Mapping
from pathlib import Path
from types import TracebackType

from . import _native


class ApkgArtifact(Mapping[str, str]):
    """An owning handle. A saved pathname alone does not retain temporary files."""

    def __init__(self, handle: _native.NativeArtifact) -> None:
        self._handle = handle

    @property
    def path(self) -> Path:
        return Path(self._handle.path())

    def close(self) -> None:
        self._handle.close()

    def __enter__(self) -> ApkgArtifact:
        return self

    def __exit__(self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: TracebackType | None) -> None:
        self.close()

    def __getitem__(self, key: str) -> str:
        if key != "path":
            raise KeyError(key)
        return str(self.path)

    def __iter__(self) -> Iterator[str]:
        return iter(("path",))

    def __len__(self) -> int:
        return 1
