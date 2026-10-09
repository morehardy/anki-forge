"""Reconcile archived measurements without rerunning exports or Anki imports."""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
spec = importlib.util.spec_from_file_location("archive_check", ROOT / "verify-archives.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
archives = module.verify()

records = Path(tempfile.mkdtemp(prefix="active-writer-records-", dir=REPO / "benchmarks/.work"))
# analyze.py discovers the repository relative to its own directory.
analysis = Path(tempfile.mkdtemp(prefix="active-writer-analysis-", dir=REPO / "benchmarks/.work"))
for path in ROOT.iterdir():
    if path.is_file() and path.suffix not in {".gz", ".png", ".svg"}:
        shutil.copy2(path, analysis / path.name)
shutil.copy2(ROOT / "source-and-inputs.tar.gz", analysis / "source-and-inputs.tar.gz")

seen = set()
for name in ("measurements-and-validation.tar.gz", "source-and-probes.tar.gz"):
    with tarfile.open(ROOT / name) as archive:
        for entry in archive.getmembers():
            # verify() already rejected non-regular files and unsafe paths.
            assert entry.name not in seen, entry.name
            seen.add(entry.name)
            target = records / entry.name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(entry) as source, target.open("wb") as stream:
                shutil.copyfileobj(source, stream)

diagnostic = subprocess.run(
    [sys.executable, str(ROOT / "audit.py"), "--work-dir", str(records / "diagnostic")],
    check=True, capture_output=True, text=True,
)
actual = json.loads(diagnostic.stdout)
expected = json.loads((ROOT / "audit.json").read_text())
binary_names = expected["retained_binary_hashes_verified"]
assert len(binary_names) == 7 and expected["binaries_not_archived"] == []
assert actual["retained_binary_hashes_verified"] == []
assert actual["binaries_not_archived"] == binary_names
for key in ("retained_binary_hashes_verified", "binaries_not_archived"):
    actual.pop(key)
    expected.pop(key)
assert actual == expected
formal = subprocess.run(
    [sys.executable, str(analysis / "analyze.py"), "--run-dir", str(records / "run")],
    check=True, capture_output=True, text=True,
)
assert json.loads(formal.stdout) == {"status": "verified", "attempts": 840, "anki_checks": 40}
for name in ("summary.json", "verification-summary.json", "comparison.csv"):
    assert (analysis / name).read_bytes() == (ROOT / name).read_bytes(), name
result = {
    "status": "passed",
    "diagnostic_exports": 570,
    "formal_exports": 840,
    "anki_check_records": 105,
    "summaries_byte_identical": True,
    "binaries_not_archived": binary_names,
    "archives": archives,
    "records_workspace": str(records),
    "analysis_workspace": str(analysis),
    "scope": "Offline record reconciliation; no exports or Anki imports rerun.",
}
(records / "replay-result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
