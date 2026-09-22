"""Build an unpacked sdist offline, then exercise its installed wheel."""
from __future__ import annotations

import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import tarfile
import tempfile


def main() -> None:
    source = Path(sys.argv[1]).resolve()
    smoke = Path(__file__).with_name("check_native_wheel.py").resolve()
    with tempfile.TemporaryDirectory(prefix="anki-forge-sdist-") as directory:
        root = Path(directory)
        with tarfile.open(source) as archive:
            # Early Python 3.11 versions do not yet expose tarfile's data filter.
            for member in archive.getmembers():
                name = PurePosixPath(member.name)
                if name.is_absolute() or ".." in name.parts or not (member.isfile() or member.isdir()):
                    raise ValueError(f"unexpected source-package member: {member.name}")
                target = root.joinpath(*name.parts)
                if member.isdir():
                    target.mkdir(parents=True, exist_ok=True)
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with archive.extractfile(member) as content, target.open("wb") as output:
                        shutil.copyfileobj(content, output)
        unpacked, = (path for path in root.iterdir() if path.is_dir())
        assert (unpacked / "Cargo.lock").is_file()
        assert (unpacked / "anki_forge/build.rs").is_file()
        environment = os.environ.copy()
        environment["PATH"] = str(Path(sys.executable).parent) + os.pathsep + environment.get("PATH", "")
        # Share dependency compilation only; package source must be self-contained.
        environment["CARGO_TARGET_DIR"] = str(Path.cwd() / "target/python-sdist-check")
        subprocess.run([
            sys.executable, "-m", "build", "--wheel", "--no-isolation",
            "--config-setting", "maturin.build-args=--locked --offline",
            "--outdir", str(root / "wheels"),
        ], cwd=unpacked, env=environment, check=True)
        wheel, = (root / "wheels").glob("*.whl")
        subprocess.run([sys.executable, str(smoke), str(wheel)], check=True)


if __name__ == "__main__":
    main()
