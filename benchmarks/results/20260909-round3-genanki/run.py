"""One predeclared standard matrix, with frozen source and power checks."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tarfile

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO / "benchmarks"))
import bench
import media_bench

NAME = "20260909-round3-genanki"
INPUTS = REPO / "benchmarks/.work/readme-comparison/fixtures"
assert sys.version_info[:3] == (3, 11, 0) and platform.machine() == "arm64"
assert not any(os.environ.get(key) for key in (
    "ANKI_AUDIT_MODE", "ANKI_AUDIT_STABLE_ID", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
)), "unexpected benchmark or compiler override"
match = json.loads((WORK / "round3-source-match.json").read_text())
assert match["source_files"] == {name: bench.sha256(REPO / name) for name in match["source_files"]}
cases = media_bench.suite_inputs(INPUTS)
bench.workload.generate()
identity = bench.identity_snapshot(bench.registry())
bench.save(WORK / "source-snapshot.json", identity)
(WORK / "source.patch").write_bytes(subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=REPO))

# Keep source and input JSON compact; media bytes are reproducible from the
# frozen v2 generator and must match the complete before/after hash inventory.
with tarfile.open(WORK / "source-and-inputs.tar.gz", "w:gz") as archive:
    for name, digest in identity["source_files"].items():
        selected = name.startswith(("anki_forge/", "contract_tools/", "contracts/", "scripts/roundtrip_oracle/", ".cargo/"))
        selected |= name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")
        selected |= name.startswith("benchmarks/") and not name.startswith("benchmarks/results/")
        if selected and digest != "missing":
            archive.add(REPO / name, arcname="source/" + name, recursive=False)
    for _, _, path, _ in cases:
        archive.add(path, arcname="fixtures/" + str(path.relative_to(INPUTS)), recursive=False)

media_identity = {}
for _, _, path, document in cases:
    for media in document.get("media", []):
        source = (path.parent / media["path"]).resolve()
        key = str(source.relative_to(INPUTS))
        media_identity[key] = {"sha256": bench.sha256(source), "bytes": source.stat().st_size}
bench.save(WORK / "media-inputs.json", media_identity)

def power():
    return subprocess.check_output(["/usr/bin/pmset", "-g", "batt"], text=True).strip()

initial_power = power()
expected_power = initial_power.splitlines()[0]
assert expected_power == "Now drawing from 'AC Power'", initial_power
bench.save(WORK / "plan.json", {
    "created_utc": bench.utc(), "name": NAME,
    "source_commit": bench.command(["git", "rev-parse", "HEAD"]),
    "git_status": bench.command(["git", "status", "--short"]),
    "exploratory_dirty_source": True,
    "runner_sha256": bench.sha256(Path(__file__)),
    "source_snapshot_sha256": bench.sha256(WORK / "source-snapshot.json"),
    "source_patch_sha256": bench.sha256(WORK / "source.patch"),
    "source_and_inputs_sha256": bench.sha256(WORK / "source-and-inputs.tar.gz"),
    "media_inputs_sha256": bench.sha256(WORK / "media-inputs.json"),
    "python": sys.version, "python_executable": sys.executable,
    "power_before": initial_power,
    "matrix": [{"profile": profile, "notes": size} for profile, size, _, _ in cases],
    "timing_repeats": 10, "rss_repeats": 5, "warmups_before_each_phase": 3,
    "expected_attempts": 840, "expected_anki_imports": 40,
    "schedule_seed": 20260907,
    "headline_metric": "median elapsed time from native process spawn to exit; includes startup, parsing, export and shutdown",
    "rss_metric": "single_process_peak_rss_os_v1, 5 independent invocations",
    "report_quantiles": "Hyndman-Fan type 7; Q1/Q3 interpolated at (n-1)*p from raw timing samples",
    "verification_schedule": "after both adjacent exporters exit, outside timing, following current media_bench.py",
    "targets": {"each_cell_time_reduction": 0.30, "unique_image_1000_rss_mib": 64},
    "no_selective_retries": True, "cold_cache_controlled": False,
})

original = media_bench.collect_attempt
def guarded(case, row, command, env):
    before = power()
    if before.splitlines()[0] != expected_power:
        raise RuntimeError("power source changed before " + row["id"])
    try:
        return original(case, row, command, env)
    finally:
        after = power()
        bench.append(case.parent / "power.jsonl", {
            "id": row["id"], "before": before, "after": after, "utc": bench.utc(),
        })
        if after.splitlines()[0] != expected_power:
            raise RuntimeError("power source changed after " + row["id"])

media_bench.collect_attempt = guarded
print("Frozen standard matrix: 20 cells; 840 exporter launches; 40 Anki imports; AC Power", flush=True)
media_bench.run(argparse.Namespace(name=NAME, inputs=INPUTS, repeats=10, rss_repeats=5, skip_oracle=False))
after = bench.identity_snapshot(bench.registry())
bench.save(WORK / "identity-after.json", after)
assert after == identity, "frozen source/build identity changed"
for name, record in media_identity.items():
    assert bench.sha256(INPUTS / name) == record["sha256"], name
bench.save(WORK / "completed.json", {
    "status": "completed", "utc": bench.utc(), "identity_unchanged": True,
    "media_files_unchanged": len(media_identity), "power_after": power(),
})
