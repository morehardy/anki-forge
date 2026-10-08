#!/usr/bin/env python3
"""Serial paired publication experiments; binaries/fixtures must already be frozen.

Run with the benchmark venv (zstandard dependency). Verification is outside the
collector's spawn-to-exit interval. Raw failures are saved before aborting; never
retry a single score. Independent complete batches have distinct output roots.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import statistics
import shutil
import subprocess
import sys

import bench
import verify
import publication_collector

SEED = 20261003
GROUPS = {
    "reuse": ("prepared", "compare-build", "prepare-publish", ["text1k", "image1k", "wide1k", "text10k"], 1),
    "reuse-changed": ("prepared", "compare-build", "prepare-publish", ["text1k-changed"], 1),
    "reuse-blocked": ("prepared", "compare-blocked", "prepare-blocked", ["text1k-blocked"], 1),
    "combined-reuse": ("combined", "compare-build", "prepare-publish", ["text1k", "image1k", "wide1k", "text10k"], 1),
    "combined-bytes": ("combined", "bytes", "bytes", ["bytes2m", "bytes8m"], 8),
    "prepared-control": ("prepared", "build", "build", ["text1k", "image1k", "wide1k", "text10k"], 1),
    "manifest": ("manifest", "build", "build", ["text1k", "image1k", "wide1k", "text10k"], 1),
    "bytes": ("media", "bytes", "bytes", ["bytes2m", "bytes8m"], 8),
    "bytes-single": ("media", "bytes", "bytes", ["bytes2m", "bytes8m", "bytes-small"], 1),
    "combined": ("combined", "build", "build", ["text1k", "image1k", "audio1k", "shared1k", "wide1k", "text10k"], 1),
}


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def frozen_identities(fixtures, dependencies):
    paths = [p for p in fixtures.rglob("*") if p.is_file()]
    paths += [Path(__file__), Path(bench.__file__), Path(verify.__file__),
              bench.COLLECTOR, bench.INSPECTOR, bench.ORACLE,
              Path(publication_collector.__file__), *dependencies]
    return {str(p.resolve()): sha(p) for p in paths if p.is_file()}


def measure_command(command, environment, directory):
    proc = subprocess.run(command, env=environment, text=True, capture_output=True, timeout=150)
    save(directory / "collector.json", {"returncode": proc.returncode, "stdout": proc.stdout, "stderr": proc.stderr})
    if proc.returncode:
        raise RuntimeError(f"collector failed, retained {directory}")
    measurement = json.loads(proc.stdout)
    try:
        publication_collector.validate(measurement)
    except AssertionError as cause:
        raise RuntimeError(f"invalid collector attempt, retained {directory}: {cause}") from cause
    return measurement


def validate_node_reports(rows):
    # Keep the entire report/comparison; duration alone is an observed runtime.
    for operation, profile in {(r["operation"], r["profile"]) for r in rows}:
        reports = []
        for row in rows:
            if (row["operation"], row["profile"]) != (operation, profile):
                continue
            report = dict(row["report"])
            report.pop("duration_ms", None)
            reports.append(json.dumps(report, sort_keys=True))
        assert len(set(reports)) == 1, (operation, profile, "Node report evidence changed")


def summary(rows):
    result = {}
    for group, profile in sorted({(r["group"], r["profile"]) for r in rows}):
        data = [r for r in rows if (r["group"], r["profile"]) == (group, profile)]
        cells = {}
        for mode in ["baseline", "candidate"]:
            timings = [r for r in data if r["mode"] == mode and r["role"] == "timing"]
            memory = [r["rss_mib"] for r in data if r["mode"] == mode and r["role"] == "rss"]
            cells[mode] = {}
            for metric in ["elapsed_ms", "input_ms", "operation_ms"]:
                values = [r[metric] for r in timings]
                if values:
                    cells[mode][metric] = {"median": statistics.median(values), "quartiles": statistics.quantiles(values, n=4), "samples": values}
            cells[mode]["rss_mib"] = {"median": statistics.median(memory) if memory else None, "samples": memory}
        pairs = []
        for repeat in sorted({r["repeat"] for r in data if r["role"] == "timing"}):
            pair = {r["mode"]: r["elapsed_ms"] for r in data if r["role"] == "timing" and r["repeat"] == repeat}
            if len(pair) == 2:
                pairs.append(pair["baseline"] - pair["candidate"])
        if pairs:
            rng = random.Random(SEED)
            resampled = sorted(statistics.median(rng.choices(pairs, k=len(pairs))) for _ in range(10000))
            before, after = (cells[m]["elapsed_ms"]["median"] for m in ["baseline", "candidate"])
            bmem, amem = (cells[m]["rss_mib"]["median"] for m in ["baseline", "candidate"])
            result[f"{group}/{profile}"] = {"cells": cells, "paired_savings_ms": pairs,
                "median_paired_saving_ms": statistics.median(pairs),
                "paired_bootstrap_95_range_ms": [resampled[249], resampled[9749]],
                "median_reduction_percent": 100 * (before - after) / before,
                "requires_confirmation": after > before * 1.05 or (amem is not None and amem > bmem + max(bmem * .1, 8))}
    return result


def run(args):
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    fixtures = args.fixtures.resolve()
    binaries = args.binaries.resolve()
    groups = args.groups.split(",")
    phases = [("smoke", 1)] if args.smoke else [("warmup", 2), ("timing", 10), ("rss", 3)]
    schedule = []
    for group in groups:
        config = GROUPS[group]
        for role, count in phases:
            for repeat in range(count):
                for profile in config[3]:
                    for mode in (["baseline", "candidate"] if repeat % 2 == 0 else ["candidate", "baseline"]):
                        schedule.append(dict(group=group, role=role, repeat=repeat, profile=profile, mode=mode))
    identities = frozen_identities(fixtures, [*binaries.iterdir(), *getattr(args, "dependencies", [])])
    save(root / "plan.json", {"schedule": schedule, "sha256": identities,
        "environment": {"platform": platform.platform(), "machine": platform.machine(), "python": sys.version,
            "host_state": bench.host_state()}, "seed": SEED,
        "scope": getattr(args, "scope", "collector spawn-to-exit; input and operation spans are nested; verification and Anki import are outside timing"),
        "rss_scope": "3 independent processes per mode/workload, not timing-process RSS",
        "cache": "OS page cache uncontrolled; media owners retained within each process; fresh process each sample"})
    rows = []
    for index, case in enumerate(schedule):
        candidate, old_mode, new_mode, _, repeats = GROUPS[case["group"]]
        mode = old_mode if case["mode"] == "baseline" else new_mode
        binary = binaries / ("baseline" if case["mode"] == "baseline" else candidate)
        directory = root / f"{index:04}-{case['group']}-{case['profile']}-{case['mode']}"
        directory.mkdir()
        temp = directory / "tmp"
        temp.mkdir()
        output = directory / "output.apkg"
        input_path = fixtures / case["profile"] / "input.json"
        document = json.loads(input_path.read_text())
        baseline = fixtures / case["profile"] / "baseline.apkg"
        command = [str(bench.COLLECTOR), "120", str(directory / "stdout.log"), str(directory / "stderr.log"),
            str(binary), str(input_path), str(output), mode,
            str(baseline) if (case["group"].startswith("reuse") or case["group"] == "combined-reuse") else "", str(repeats)]
        environment = {k: v for k, v in os.environ.items() if not k.startswith(("ANKIFORGE_EXP_", "ANKIFORGE_DIAG_"))}
        environment.update(TMPDIR=str(temp), TMP=str(temp), TEMP=str(temp))
        if hasattr(args, "command"):
            child, additions = args.command(case, input_path, output, mode, baseline)
            command = command[:4] + child
            environment.update(additions)
        save(directory / "execution.json", {"argv": command, "temp": str(temp), "host_before": bench.host_state()})
        measurement = measure_command(command, environment, directory)
        facts = json.loads((directory / "stdout.log").read_text())
        blocked = case["group"] == "reuse-blocked"
        comparison_only = mode == "compare"
        oracle = ((case["role"] == "timing" and case["repeat"] == 0) or args.smoke) and not (blocked or comparison_only)
        if comparison_only:
            assert facts["comparison"] and not output.exists()
            logical, artifact_hash, artifact_bytes = None, None, 0
        elif blocked:
            assert facts["failure"]["result"]["code"] == "UPDATE.POLICY_BLOCKED", facts
            assert not output.exists(), "blocked workflow published an artifact"
            logical, artifact_hash, artifact_bytes = None, None, 0
        else:
            checked = verify.verify_artifact(output, document, bench.INSPECTOR)
            save(directory / "verification.json", checked)
            if checked["status"] != "passed": raise RuntimeError(f"content failure: {directory}")
            logical, artifact_hash, artifact_bytes = checked["physical"]["logical_sha256"], sha(output), output.stat().st_size
        if oracle:
            proc = subprocess.run([str(bench.ORACLE), str(input_path), str(output), str(directory / "anki.json")], text=True, capture_output=True, timeout=120)
            save(directory / "oracle-process.json", {"returncode": proc.returncode, "stdout": proc.stdout, "stderr": proc.stderr})
            if proc.returncode or json.loads((directory / "anki.json").read_text())["status"] != "passed":
                raise RuntimeError(f"Anki failure: {directory}")
        if list(temp.iterdir()): raise RuntimeError(f"temporary ownership leak: {directory}")
        row = {**case, **facts, "elapsed_ms": measurement["elapsed_ns"] / 1e6,
            "rss_mib": measurement["peak_rss_bytes"] / 2**20, "artifact_sha256": artifact_hash,
            "artifact_bytes": artifact_bytes, "logical_sha256": logical, "oracle": oracle}
        rows.append(row)
        with (root / "samples.jsonl").open("a") as stream: stream.write(json.dumps(row, ensure_ascii=False) + "\n")
        if output.exists(): output.unlink()
        if index % 20 == 0: print(index + 1, "/", len(schedule), flush=True)
    for p, digest in identities.items():
        assert sha(Path(p)) == digest, p
    for group in groups:
        for profile in GROUPS[group][3]:
            selected = [r for r in rows if r["group"] == group and r["profile"] == profile]
            assert len({r["logical_sha256"] for r in selected}) == 1
            assert len({json.dumps(r["comparison"], sort_keys=True) for r in selected}) == 1
            if "manifest_fingerprint" in selected[0]:
                assert all(r["manifest_verified"] for r in selected)
                assert len({r["manifest_fingerprint"] for r in selected}) == 1
    save(root / "summary.json", summary(rows))
    save(root / "complete.json", {"samples": len(rows), "status": "passed", "host_after": bench.host_state()})


def run_node(args):
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    fixtures, binaries = args.fixtures.resolve(), args.binaries.resolve()
    sdk = bench.REPO / "bindings/node/dist/index.mjs"
    script = bench.SUITE / "publication_node.mjs"
    phases = [("smoke", 1)] if args.smoke else [("warmup", 2), ("timing", 10), ("rss", 3)]
    profiles = ["text1k", "wide1k", "text10k"]
    schedule = [dict(profile=p, operation=o, role=role, repeat=i, mode=m)
        for o in ["build", "compare", "prepare"] for role, n in phases for i in range(n)
        for p in profiles for m in (["before", "cow"] if i % 2 == 0 else ["cow", "before"])]
    identities = frozen_identities(fixtures, [script, sdk, binaries / "node-before.node", binaries / "node-cow.node"])
    save(root / "plan.json", {"schedule": schedule, "sha256": identities, "host": bench.host_state(),
        "timer_scope": "per-run 1ms timer delays, not population p99", "rss_scope": "independent RSS-phase processes",
        "first_edit_scope": "add with task and clone snapshots live; excludes exclusive defaultDeck and clone measurements"})
    rows = []
    for index, case in enumerate(schedule):
        directory = root / f"{index:04}-{case['operation']}-{case['profile']}-{case['mode']}"
        directory.mkdir()
        temp = directory / "tmp"
        temp.mkdir()
        input_path, baseline = fixtures / f"{case['profile']}.json", fixtures / f"{case['profile']}.apkg"
        artifact = directory / "output.apkg"
        environment = {**os.environ, "ANKI_FORGE_NATIVE_PATH": str(binaries / f"node-{case['mode']}.node"), "TMPDIR": str(temp)}
        command = [str(bench.COLLECTOR), "120", str(directory / "stdout.log"), str(directory / "stderr.log"), shutil.which("node"), "--expose-gc", str(script), str(sdk), str(input_path), case["operation"], str(baseline), str(artifact)]
        if getattr(args, "operation_only", False): command.append("--operation-only")
        save(directory / "execution.json", {"argv": command, "native": environment["ANKI_FORGE_NATIVE_PATH"]})
        measurement = measure_command(command, environment, directory)
        facts = json.loads((directory / "stdout.log").read_text())
        if case["operation"] != "compare":
            checked = verify.verify_artifact(artifact, json.loads(input_path.read_text()), bench.INSPECTOR)
            save(directory / "verification.json", checked)
            if checked["status"] != "passed": raise RuntimeError(f"Node content mismatch: {directory}")
            if args.smoke or (case["role"] == "timing" and case["repeat"] == 0):
                proc = subprocess.run([str(bench.ORACLE), str(input_path), str(artifact), str(directory / "anki.json")], text=True, capture_output=True, timeout=120)
                save(directory / "oracle-process.json", {"returncode": proc.returncode, "stdout": proc.stdout, "stderr": proc.stderr})
                if proc.returncode or json.loads((directory / "anki.json").read_text())["status"] != "passed": raise RuntimeError(f"Node Anki mismatch: {directory}")
            artifact.unlink()
        if list(temp.iterdir()): raise RuntimeError(f"Node temporary owner leak: {directory}")
        rows.append({**case, **facts, "elapsed_ms": measurement["elapsed_ns"] / 1e6, "rss_mib": measurement["peak_rss_bytes"] / 2**20})
        with (root / "samples.jsonl").open("a") as stream: stream.write(json.dumps(rows[-1]) + "\n")
        if index % 20 == 0: print("node", index + 1, "/", len(schedule), flush=True)
    for p, digest in identities.items(): assert sha(Path(p)) == digest, p
    validate_node_reports(rows)
    save(root / "summary.json", node_summary(rows))
    save(root / "complete.json", {"status": "passed", "samples": len(rows)})


def node_summary(rows):
    result = {}
    metrics = ["submit_ms", "completion_ms", "clone_ms", "exclusive_edit_ms", "first_shared_edit_ms", "subsequent_edit_ms", "failed_add_ms", "timer_max_ms", "elapsed_ms"]
    for operation, profile in sorted({(r["operation"], r["profile"]) for r in rows}):
        selected = [r for r in rows if (r["operation"], r["profile"]) == (operation, profile)]
        cells = {}
        for mode in ["before", "cow"]:
            timing = [r for r in selected if r["role"] == "timing" and r["mode"] == mode]
            if not timing: continue
            cells[mode] = {metric: {"median": statistics.median(r[metric] for r in timing),
                "quartiles": statistics.quantiles([r[metric] for r in timing], n=4),
                "samples": [r[metric] for r in timing]} for metric in metrics}
            memory = [r["rss_mib"] for r in selected if r["role"] == "rss" and r["mode"] == mode]
            cells[mode]["rss_mib"] = {"median": statistics.median(memory) if memory else None, "samples": memory}
        paired = {}
        for metric in metrics:
            pairs = [{r["mode"]: r[metric] for r in selected if r["role"] == "timing" and r["repeat"] == i} for i in range(10)]
            deltas = [p["before"] - p["cow"] for p in pairs if len(p) == 2]
            if deltas:
                rng = random.Random(SEED)
                boot = sorted(statistics.median(rng.choices(deltas, k=len(deltas))) for _ in range(10000))
                paired[metric] = {"before_minus_cow": deltas, "median": statistics.median(deltas), "bootstrap_95_range": [boot[249], boot[9749]]}
        if not cells: continue
        before, after = (cells[m] for m in ["before", "cow"])
        bmem, amem = (c["rss_mib"]["median"] for c in [before, after])
        result[f"{operation}/{profile}"] = {"cells": cells, "paired": paired,
            "requires_confirmation": after["elapsed_ms"]["median"] > before["elapsed_ms"]["median"] * 1.05
                or (amem is not None and amem > bmem + max(bmem * .1, 8))}
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", type=Path)
    parser.add_argument("--binaries", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--groups", default=",".join(GROUPS))
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--node", action="store_true")
    parser.add_argument("--operation-only", action="store_true", help="Node operations without concurrent edits or clone")
    parser.add_argument("--analyze", type=Path, help="offline recomputation only; does not export or import APKGs")
    args = parser.parse_args()
    if args.analyze:
        rows = [json.loads(line) for line in args.analyze.read_text().splitlines()]
        print(json.dumps((node_summary if args.node else summary)(rows), indent=2))
    elif args.node:
        run_node(args)
    else:
        run(args)
