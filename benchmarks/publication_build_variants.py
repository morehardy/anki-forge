#!/usr/bin/env python3
"""Build isolated variants from the frozen pre-implementation source archive.

Only the four selected changes are copied; no experimental production switches.
The archive is the invocation-time tracked source plus the prerequisite test file.
Each binary has a full source-digest inventory and its exact command saved before
the next source mutation. Must finish before measurement starts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("work", type=Path)
    args = parser.parse_args()
    work = args.work.resolve()
    (work / "binaries").mkdir(parents=True, exist_ok=True)
    source = work / "variant-source"
    source.mkdir(parents=True, exist_ok=True)
    with tarfile.open(args.archive) as archive: archive.extractall(source, filter="data")
    baseline_core = {str(p.relative_to(source)): p.read_bytes() for p in (source / "anki_forge").rglob("*") if p.is_file()}
    records = []
    for variant in ["baseline", "prepared", "manifest", "media", "combined"]:
        shutil.rmtree(source / "anki_forge")
        for name, data in baseline_core.items():
            target = source / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        files = []
        if variant in ["prepared", "combined"]:
            files += ["anki_forge/src/build_api/mod.rs", "anki_forge/src/build_api/pipeline.rs", "anki_forge/src/build_api/prepared.rs"]
        if variant in ["manifest", "combined"]:
            files += ["anki_forge/src/writer_core/staging.rs", "anki_forge/src/writer_core/build.rs", "anki_forge/src/build_api/candidate.rs"]
        if variant in ["media", "combined"]: files += ["anki_forge/src/media/snapshot.rs"]
        if variant == "combined":
            # Same final embedded assets as the delivered crate; baseline and
            # isolated variants keep the frozen original bundle.
            shutil.rmtree(source / "anki_forge/assets")
            shutil.copytree(ROOT / "anki_forge/assets", source / "anki_forge/assets")
        files += ["benchmarks/adapters/rust/src/main.rs", "benchmarks/adapters/rust/src/control.rs", "benchmarks/adapters/rust/Cargo.toml", "benchmarks/adapters/rust/Cargo.lock"]
        for name in files: shutil.copy2(ROOT / name, source / name)
        inventory = {str(p.relative_to(source)): digest(p) for base in [source / "anki_forge", source / "benchmarks/adapters/rust"]
            for p in base.rglob("*") if p.is_file() and "target" not in p.parts}
        (work / f"{variant}-source.json").write_text(json.dumps(inventory, indent=2) + "\n")
        command = ["cargo", "build", "--release", "--offline", "--locked", "--manifest-path", str(source / "benchmarks/adapters/rust/Cargo.toml")]
        if variant in ["prepared", "combined"]: command += ["--features", "prepared-publication"]
        target = ROOT / "benchmarks/adapters/rust/target"
        started = time.time()
        with (work / f"{variant}-build.log").open("w") as log:
            proc = subprocess.run(command, env={**os.environ, "CARGO_TARGET_DIR": str(target)}, stdout=log, stderr=subprocess.STDOUT)
        record = dict(variant=variant, argv=command, source_sha256=digest(work / f"{variant}-source.json"), returncode=proc.returncode, started_unix=started, seconds=time.time() - started)
        records.append(record)
        (work / "variant-builds.json").write_text(json.dumps(records, indent=2) + "\n")
        if proc.returncode: raise RuntimeError(f"{variant} build failed")
        destination = work / "binaries" / variant
        shutil.copy2(target / "release/anki-forge-benchmark", destination)
        record["binary_sha256"] = digest(destination)
        (work / "variant-builds.json").write_text(json.dumps(records, indent=2) + "\n")
        print(variant, "built", flush=True)


if __name__ == "__main__": main()
