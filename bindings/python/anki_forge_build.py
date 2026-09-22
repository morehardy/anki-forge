"""Maturin backend with a locked, standalone source distribution.

Maturin removes unrelated workspace members from an sdist but keeps their lock
entries. Prune that graph offline before shipping, without changing any pins.
"""
from __future__ import annotations

import io
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
from typing import Any, Mapping

import maturin
from maturin import (  # PEP 517 / 660 hooks are otherwise unchanged.
    build_editable,
    build_wheel,
    get_requires_for_build_editable,
    get_requires_for_build_sdist,
    get_requires_for_build_wheel,
    prepare_metadata_for_build_editable,
    prepare_metadata_for_build_wheel,
)


def _pins(data: bytes) -> set[tuple[str, str, str, str]]:
    return {(p["name"], p["version"], p.get("source", ""), p.get("checksum", ""))
            for p in tomllib.loads(data.decode())["package"]}


def build_sdist(sdist_directory: str, config_settings: Mapping[str, Any] | None = None) -> str:
    filename = maturin.build_sdist(sdist_directory, config_settings)
    path = Path(sdist_directory, filename).resolve()
    with tempfile.TemporaryDirectory(prefix="anki-forge-sdist-lock-") as directory:
        root = Path(directory)
        with tarfile.open(path) as archive:
            members = archive.getmembers()
            for member in members:
                name = PurePosixPath(member.name)
                if name.is_absolute() or ".." in name.parts or not (member.isfile() or member.isdir()):
                    raise ValueError(f"unexpected sdist member: {member.name}")
                target = root.joinpath(*name.parts)
                if member.isdir():
                    target.mkdir(parents=True, exist_ok=True)
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with archive.extractfile(member) as source, target.open("wb") as output:
                        shutil.copyfileobj(source, output)
        unpacked, = (p for p in root.iterdir() if p.is_dir())
        lock = unpacked / "Cargo.lock"
        original_pins = _pins(lock.read_bytes())
        subprocess.run(["cargo", "metadata", "--offline", "--format-version", "1"],
                       cwd=unpacked, stdout=subprocess.DEVNULL, check=True)
        if not _pins(lock.read_bytes()).issubset(original_pins):
            raise RuntimeError("sdist lock pruning attempted to change dependency pins")
        subprocess.run(["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
                       cwd=unpacked, stdout=subprocess.DEVNULL, check=True)
        staged = root / filename
        with tarfile.open(staged, "w:gz") as archive:
            for member in members:
                if member.isfile():
                    data = (root / member.name).read_bytes()
                    member.size = len(data)
                    archive.addfile(member, io.BytesIO(data))
                else:
                    archive.addfile(member)
        shutil.copyfile(staged, path)
    return filename
