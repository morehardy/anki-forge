from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
from typing import Protocol, cast

from . import _native
from ._bridge import invoke
from .artifact import ApkgArtifact
from .options import BuildOptions, PathInput, RiskLevelValue, UpdateSafetyValue
from .report import BuildReport


class _BinaryWriter(Protocol):
    def write(self, data: bytes, /) -> int | None: ...


class _Buildable:
    _handle: _native.NativeProject | _native.NativeDeck

    @property
    def base_dir(self) -> Path:
        raise NotImplementedError

    def build(self, options: BuildOptions | None = None) -> BuildReport[ApkgArtifact]:
        payload, artifact = invoke(self._handle.build, json.dumps((options or BuildOptions())._payload(self.base_dir)))
        report = BuildReport.from_json(json.loads(payload))
        return cast(BuildReport[ApkgArtifact], replace(report, artifact=ApkgArtifact(artifact, base_dir=self.base_dir) if artifact is not None else None))

    def to_apkg_bytes(self, options: BuildOptions | None = None) -> bytes:
        report = self.build(options)
        report.ensure_success()
        artifact = report.artifact
        assert isinstance(artifact, ApkgArtifact)
        try:
            return artifact.path.read_bytes()
        finally:
            artifact.close()
            report.close()

    def write_to(self, destination: _BinaryWriter, options: BuildOptions | None = None) -> int:
        """Build an APKG, then copy it in chunks of at most 64 KiB; keep the sink open."""
        report = self.build(options)
        report.ensure_success()
        artifact = report.artifact
        assert isinstance(artifact, ApkgArtifact)
        total = 0
        try:
            with artifact.path.open("rb") as source:
                while chunk := source.read(64 * 1024):
                    offset = 0
                    while offset < len(chunk):
                        written = destination.write(chunk[offset:])
                        if written is None or written == 0:
                            raise BlockingIOError("binary writer made no progress")
                        if type(written) is not int or written < 0 or written > len(chunk) - offset:
                            raise OSError("binary writer returned an invalid byte count")
                        offset += written
                    total += len(chunk)
            return total
        finally:
            artifact.close()
            report.close()

    def write_apkg(
        self, path: PathInput, *, options: BuildOptions | None = None,
        compare_to: PathInput | None = None, fail_on: RiskLevelValue | None = None,
        report_json: PathInput | None = None, identity_lockfile: PathInput | None = None,
        write_identity_lockfile: bool | None = None, update_safety: UpdateSafetyValue | None = None,
    ) -> BuildReport[ApkgArtifact]:
        selected = (options or BuildOptions())._with_overrides(
            self.base_dir, output=path, compare_to=compare_to, fail_on=fail_on,
            report_json=report_json, identity_lockfile=identity_lockfile,
            write_identity_lockfile=write_identity_lockfile, update_safety=update_safety,
        )
        return self.build(selected)
