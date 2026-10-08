from __future__ import annotations
import json
from pathlib import Path
from . import _native
from ._bridge import invoke
from .artifact import ApkgArtifact
from .media import Media
from .note import Note
from .options import BuildOptions, CompareOptions
from .prepared import PreparedPublication
from .report import BuildOutput, ComparisonReport

class Project:
    """The sole authoring container, identified by an explicit stable namespace."""
    def __init__(self, namespace: str, *, default_deck: str | None = None) -> None:
        self._handle = invoke(_native.NativeProject, namespace, default_deck)
        self._namespace = namespace

    @property
    def namespace(self) -> str:
        return self._namespace

    def __len__(self) -> int:
        return invoke(self._handle.len)

    def add(self, key: str, note: Note) -> Project:
        invoke(self._handle.add, key, note._handle)
        return self

    def add_asset(self, media: Media) -> Project:
        invoke(self._handle.add_asset, media._handle)
        return self

    def build(self, options: BuildOptions) -> BuildOutput:
        base_dir = Path.cwd()
        snapshot, artifact = invoke(self._handle.build, json.dumps(options._payload()))
        return BuildOutput(ApkgArtifact(artifact, base_dir=base_dir), json.loads(snapshot))

    def prepare_publication(self, options: BuildOptions) -> PreparedPublication:
        base_dir = Path.cwd()
        handle = invoke(self._handle.prepare_publication, json.dumps(options._payload(prepare=True)))
        return PreparedPublication._from_native(handle, base_dir)

    def compare(self, options: CompareOptions) -> ComparisonReport:
        return ComparisonReport(json.loads(invoke(self._handle.compare, json.dumps(options._payload()))))
