"""Single-use ownership of a fully inspected, unpublished candidate."""
from __future__ import annotations
import json
from pathlib import Path
from . import _native
from ._bridge import invoke
from .artifact import ApkgArtifact
from .report import BuildOutput, BuildReport

_TOKEN = object()

class PreparedPublication:
    def __init__(self, token: object, handle: _native.NativePreparedPublication, base_dir: Path) -> None:
        if token is not _TOKEN:
            raise TypeError("Use Project.prepare_publication")
        self.__handle = handle
        self.__base_dir = base_dir
        self.__report = BuildReport(json.loads(invoke(handle.report)))

    @classmethod
    def _from_native(cls, handle: _native.NativePreparedPublication, base_dir: Path) -> PreparedPublication:
        return cls(_TOKEN, handle, base_dir)

    @property
    def report(self) -> BuildReport:
        return self.__report

    def publish(self) -> BuildOutput:
        snapshot, artifact = invoke(self.__handle.publish)
        return BuildOutput(ApkgArtifact(artifact, base_dir=self.__base_dir), json.loads(snapshot))

    def close(self) -> None:
        invoke(self.__handle.close)
