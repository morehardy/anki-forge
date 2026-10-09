"""Audit retained measurements in a fresh workspace without rerunning benchmarks."""
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
module.verify()

destination = Path(tempfile.mkdtemp(prefix="reader-cache-replay-", dir=REPO / "benchmarks/.work"))
# The analyzer locates this repo from its own directory. Keep it at the same
# depth as the original .work directory, independently of extracted records.
analysis_workspace = Path(tempfile.mkdtemp(prefix="reader-cache-analysis-", dir=REPO / "benchmarks/.work"))
for path in ROOT.iterdir():
    if path.is_file() and path.suffix not in {".gz", ".png", ".svg"}:
        shutil.copy2(path, analysis_workspace / path.name)
shutil.copy2(ROOT / "source-and-inputs.tar.gz", analysis_workspace / "source-and-inputs.tar.gz")

for name in ("measurements-and-validation.tar.gz", "source-and-probes.tar.gz"):
    with tarfile.open(ROOT / name) as archive:
        for entry in archive.getmembers():
            # verify() already restricted all entries to safe regular files.
            path = destination / entry.name
            path.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(entry) as source, path.open("wb") as target:
                shutil.copyfileobj(source, target)
for name in ("controls1-source", "controls2-source", "codec-source"):
    source = destination / "source" / name
    target = destination / "diagnostic" / name
    if target.exists():
        for path in source.rglob("*"):
            if path.is_file():
                assert path.read_bytes() == (target / path.relative_to(source)).read_bytes()
    else:
        shutil.copytree(source, target)

diagnostic = subprocess.run(
    [sys.executable, str(ROOT / "audit.py"), "--work-dir", str(destination / "diagnostic")],
    check=True, capture_output=True, text=True,
)
assert json.loads(diagnostic.stdout) == json.loads((ROOT / "audit.json").read_text())
formal = subprocess.run(
    [sys.executable, str(analysis_workspace / "analyze.py"), "--run-dir", str(destination / "run")],
    check=True, capture_output=True, text=True,
)
for name in ("summary.json", "verification-summary.json", "comparison.csv"):
    assert (analysis_workspace / name).read_bytes() == (ROOT / name).read_bytes(), name
result = {"status": "passed", "diagnostic_exports": 640, "formal_exports": 840,
          "anki_check_records": 130, "summaries_byte_identical": True,
          "records_workspace": str(destination), "analysis_workspace": str(analysis_workspace)}
(destination / "replay-result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
