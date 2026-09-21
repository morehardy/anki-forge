"""Rebuild the standard public adapter and verify the sealed round-three source."""
import json
from pathlib import Path
import sys

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO / "benchmarks"))
import bench

sealed = json.loads((REPO / "benchmarks/results/20260908-round3-performance/build-identities.json").read_text())
observed = {name: bench.sha256(REPO / name) for name in sealed["source_files"]}
assert observed == sealed["source_files"], "current product source differs from the sealed optimized source"
bench.save(WORK / "round3-source-match.json", {
    "checked_utc": bench.utc(),
    "head": bench.command(["git", "rev-parse", "HEAD"]),
    "sealed_identity_sha256": bench.sha256(REPO / "benchmarks/results/20260908-round3-performance/build-identities.json"),
    "matched_files": len(observed),
    "source_files": observed,
})
bench.run_build([
    "cargo", "build", "--release", "--locked", "--offline", "--manifest-path",
    str(REPO / "benchmarks/adapters/rust/Cargo.toml"), "--no-default-features",
], [REPO / "benchmarks/adapters/rust/target/release/anki-forge-benchmark"])
bench.run_build([
    "cargo", "build", "--release", "--locked", "--offline", "-p", "contract_tools",
], [bench.INSPECTOR])
bench.workload.generate()
provenance = bench.build_provenance(bench.registry())
assert all(row["status"] == "verified" for row in provenance.values()), provenance
bench.save(WORK / "prepared-builds.json", provenance)
print(json.dumps({"status": "prepared", "matched_source_files": len(observed),
                  "builds": {path: row["status"] for path, row in provenance.items()}}, indent=2), flush=True)
