from __future__ import annotations

from collections.abc import Iterator, Mapping
from os import PathLike
from pathlib import Path
from types import TracebackType
from typing import Any

from . import _native
from .options import absolute_path


class ApkgArtifact(Mapping[str, str]):
    """An owning handle. A saved pathname alone does not retain temporary files."""

    def __init__(self, handle: _native.NativeArtifact, *, base_dir: Path | None = None) -> None:
        if not isinstance(handle, _native.NativeArtifact):
            raise TypeError("ApkgArtifact requires an owning native handle")
        self._handle = handle
        self._base_dir = base_dir if base_dir is not None else Path.cwd()

    @property
    def path(self) -> Path:
        return Path(self._handle.path())

    def close(self) -> None:
        self._handle.close()

    def persist_to(self, path: str | PathLike[str]) -> ApkgArtifact:
        handle = self._handle.persist_to(absolute_path(path, self._base_dir))
        return ApkgArtifact(handle, base_dir=self._base_dir)

    def __copy__(self) -> ApkgArtifact:
        return ApkgArtifact(self._handle.clone_handle(), base_dir=self._base_dir)

    def __deepcopy__(self, memo: dict[int, Any]) -> ApkgArtifact:
        result = self.__copy__()
        memo[id(self)] = result
        return result

    def __enter__(self) -> ApkgArtifact:
        _ = self.path
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
