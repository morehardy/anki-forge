from __future__ import annotations
from copy import deepcopy
from dataclasses import dataclass
from typing import Any
from .artifact import ApkgArtifact

@dataclass(frozen=True)
class BuildCounts:
    notes: int
    cards: int
    media: int

class ComparisonReport:
    def __init__(self, snapshot: dict[str, Any]) -> None:
        self._snapshot = deepcopy(snapshot)

    def snapshot(self) -> dict[str, Any]:
        return deepcopy(self._snapshot)

    @property
    def findings(self) -> tuple[dict[str, Any], ...]:
        return tuple(deepcopy(self._snapshot["findings"]))

    @property
    def allows_publication(self) -> bool:
        return bool(self._snapshot["policy"]["allows_publication"])

    @property
    def policy(self) -> dict[str, Any]:
        return deepcopy(self._snapshot["policy"])

class BuildReport:
    """Observations only: this report neither owns a file nor represents success."""
    def __init__(self, snapshot: dict[str, Any]) -> None:
        self._snapshot = deepcopy(snapshot)

    def snapshot(self) -> dict[str, Any]:
        return deepcopy(self._snapshot)

    @property
    def counts(self) -> BuildCounts:
        return BuildCounts(**self._snapshot["counts"])

    @property
    def diagnostics(self) -> tuple[dict[str, Any], ...]:
        return tuple(deepcopy(self._snapshot["diagnostics"]))

    @property
    def comparison(self) -> ComparisonReport | None:
        value = self._snapshot["comparison"]
        return ComparisonReport(value) if value is not None else None

class BuildOutput:
    """A successful build with an owning artifact and independent observations."""
    def __init__(self, artifact: ApkgArtifact, snapshot: dict[str, Any]) -> None:
        self.artifact = artifact
        self._snapshot = deepcopy(snapshot)
        self.report = BuildReport(snapshot["report"])

    def snapshot(self) -> dict[str, Any]:
        return deepcopy(self._snapshot)
