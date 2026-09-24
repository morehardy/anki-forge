from __future__ import annotations
import json
from . import _native
from ._bridge import invoke
from .artifact import ApkgArtifact
from .media import Media
from .note import Note
from .options import BuildOptions, CompareOptions
from .report import BuildOutput, ComparisonReport

class Project:
    """The sole authoring container, identified by an explicit stable namespace."""
    def __init__(self, namespace: str, *, name: str | None = None, default_deck: str | None = None) -> None:
        self._handle = invoke(_native.NativeProject, namespace, name, default_deck)
        self._namespace = namespace
        self._name = namespace if name is None else name

    @property
    def namespace(self) -> str:
        return self._namespace

    @property
    def name(self) -> str:
        return self._name

    def __len__(self) -> int:
        return invoke(self._handle.len)

    def add(self, key: str, note: Note) -> Project:
        invoke(self._handle.add, key, note._handle)
        return self

    def add_asset(self, media: Media) -> Project:
        invoke(self._handle.add_asset, media._handle)
        return self

    def build(self, options: BuildOptions) -> BuildOutput:
        snapshot, artifact = invoke(self._handle.build, json.dumps(options._payload()))
        return BuildOutput(ApkgArtifact(artifact), json.loads(snapshot))

    def compare(self, options: CompareOptions) -> ComparisonReport:
        return ComparisonReport(json.loads(invoke(self._handle.compare, json.dumps(options._payload()))))
