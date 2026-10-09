"""Seal the completed matrix, all diagnostic controls, and source provenance."""
from pathlib import Path
import hashlib
import json
import shutil
import tarfile

W = Path(__file__).resolve().parent
R = W.parents[2]
DEST = R / "benchmarks/results/20261002-reader-cache-genanki"
assert not DEST.exists()
DEST.mkdir()

files = [
    "report.md", "report.py", "comparison.svg", "comparison.png", "comparison.csv", "delta.csv", "delta.json",
    "summary.json", "verification-summary.json", "source-snapshot.json", "source.patch", "round3.patch",
    "source-and-inputs.tar.gz", "media-inputs.json", "baseline-input-check.json", "identity-after.json", "final-identity.json",
    "plan.json", "completed.json", "host-hardware.json", "pre-matrix-host.json", "prepared-builds.json", "prepared-builds-final.json",
    "oracle-reuse-check.json", "initial-source.json", "initial.patch", "audit.json", "audit.py",
    "run.py", "analyze.py", "setup_probe.py", "probe.patch", "stage-reconstruction.json", "build_probe.py", "build_codec.py",
    "build-controls1.json", "build-controls2.json", "build-codec.json",
    "benchmark-tests.log", "regression-tests-final.log", "regression-codec.log", "verify-fast-final.log", "smoke-final.log",
    "archive.py",
]
for stem in ["baseline2", "stage", "control1", "control2", "codec", "confirmation"]:
    files += [f"{stem}-{suffix}.json" for suffix in ["plan", "results", "summary"]]
for name in ["baseline2-source-before.json", "baseline2-source-after.json", "confirmation-source-before.json", "confirmation-source-after.json"]:
    files.append(name)
for name in files:
    shutil.copy2(W / name, DEST / name)
shutil.copy2(W / "stages.py", DEST / "stages.py")
shutil.copy2(W / "baseline.py", DEST / "baseline.py")
for name in ["controls1.py", "controls2.py", "codec.py", "confirmation.py"]:
    shutil.copy2(W / name, DEST / name)
shutil.copy2(R / "benchmarks/.work/runs/20261002-reader-cache-genanki/manifest.json", DEST / "run-manifest.json")

archives = {}
def write_archive(name, entries):
    members = {}
    with tarfile.open(DEST / name, "w:gz") as archive:
        for path, member in entries:
            assert member not in members, member
            assert path.is_file(), path
            data = path.read_bytes()
            members[member] = hashlib.sha256(data).hexdigest()
            archive.add(path, arcname=member, recursive=False)
    with tarfile.open(DEST / name) as archive:
        assert len(archive.getmembers()) == len(members)
        assert set(archive.getnames()) == set(members)
        for entry in archive.getmembers():
            assert hashlib.sha256(archive.extractfile(entry).read()).hexdigest() == members[entry.name]
    archives[name] = members

entries = []
run = R / "benchmarks/.work/runs/20261002-reader-cache-genanki"
for path in sorted(run.rglob("*")):
    if path.is_file():
        entries.append((path, "run/" + str(path.relative_to(run))))
for stem in ["baseline2", "stage", "control1", "control2", "codec", "confirmation"]:
    for case in sorted(W.glob(stem + "-*")):
        if case.is_dir():
            for path in sorted(case.rglob("*")):
                if path.is_file():
                    entries.append((path, "diagnostic/" + str(path.relative_to(W))))
    for suffix in ["plan", "results", "summary"]:
        path = W / f"{stem}-{suffix}.json"
        entries.append((path, "diagnostic/" + path.name))
for name in ["initial-source.json", "baseline2-source-before.json", "baseline2-source-after.json",
             "confirmation-source-before.json", "confirmation-source-after.json",
             "build-controls1.json", "build-controls2.json", "build-codec.json"]:
    entries.append((W / name, "diagnostic/" + name))
smoke = R / "benchmarks/.work/runs/reader-cache-text-20261002-smoke"
for path in sorted(smoke.rglob("*")):
    if path.is_file() and path.suffix != ".apkg":
        entries.append((path, "smoke/" + str(path.relative_to(smoke))))
write_archive("measurements-and-validation.tar.gz", entries)

entries = []
for directory in ["stage-source", "controls1-source", "controls2-source", "codec-source", "production-before"]:
    for path in sorted((W / directory).rglob("*")):
        if path.is_file():
            entries.append((path, "source/" + str(path.relative_to(W))))
for path in sorted(W.glob("*.py")):
    entries.append((path, "scripts/" + path.name))
for path in sorted(W.glob("*.log")):
    entries.append((path, "logs/" + path.name))
for name in ["stage-reconstruction.json", "baseline-plan.json", "baseline-source-before.json"]:
    path = W / name
    if path.exists():
        entries.append((path, "logs/" + name))
write_archive("source-and-probes.tar.gz", entries)

source_members = {}
with tarfile.open(DEST / "source-and-inputs.tar.gz") as archive:
    for member in archive.getmembers():
        assert member.isfile() and member.name not in source_members
        source_members[member.name] = hashlib.sha256(archive.extractfile(member).read()).hexdigest()
archives["source-and-inputs.tar.gz"] = source_members
(DEST / "archive-manifest.json").write_text(json.dumps({"archives": archives}, indent=2) + "\n")
print(json.dumps({"destination": str(DEST), "members": {name: len(m) for name, m in archives.items()}}, indent=2))
