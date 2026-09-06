#!/usr/bin/env python3
"""Independent benchmark entry point; preparation never runs inside an attempt."""
import argparse
import datetime as dt
import hashlib
import importlib.metadata
import json
import os
import platform
import random
import shutil
import signal
import sqlite3
import subprocess
import sys
import sysconfig
import time
from pathlib import Path

import workload

SUITE = Path(__file__).resolve().parent
REPO = SUITE.parent
COLLECTOR = SUITE / ".tools/measure"
INSPECTOR = REPO / "target/release/contract_tools"
ORACLE = REPO / "scripts/roundtrip_oracle/target/debug/benchmark_oracle"
METRIC = "single_process_peak_rss_os_v1"
SCHEDULE_SEED = 20260906


def utc():
    return dt.datetime.now(dt.timezone.utc).isoformat()


def save(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def append(path, data):
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(data, ensure_ascii=False, separators=(",", ":")) + "\n")


def command(args, *, optional=False, cwd=REPO):
    try:
        return subprocess.check_output(args, cwd=cwd, stderr=subprocess.STDOUT, text=True, timeout=60).strip()
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        if optional:
            return "unavailable: " + str(error)
        raise


def registry():
    entries = json.loads((SUITE / "adapters.json").read_text())["adapters"]
    for item in entries:
        item["command"] = [p.format(suite=str(SUITE)) for p in item["command"]]
    return entries


def schedule(sizes=workload.SIZES, seed=SCHEDULE_SEED):
    rng = random.Random(seed)
    result = []
    for role, rounds in (("timing_warmup", 3), ("timing", 10), ("memory_warmup", 3), ("memory", 5)):
        orders = {}
        for i, size in enumerate(sizes):
            # For odd repeat counts alternate which implementation starts the extra block.
            orders[size] = [bool((r + i) % 2) for r in range(rounds)]
            rng.shuffle(orders[size])
        for round_number in range(rounds):
            blocks = list(sizes)
            rng.shuffle(blocks)
            for size in blocks:
                pair = ["rust", "genanki"] if orders[size][round_number] else ["genanki", "rust"]
                for adapter in pair:
                    result.append({"id": f"{role}-{size}-{adapter}-{round_number + 1:02d}",
                                   "role": role, "size": size, "adapter": adapter, "round": round_number + 1,
                                   "order": len(result), "pair_first": pair[0]})
    return result


def host_state():
    if platform.system() == "Darwin":
        return {"load": list(os.getloadavg()), "power": command(["pmset", "-g", "batt"], optional=True),
                "power_settings": command(["pmset", "-g", "custom"], optional=True),
                "thermal": command(["pmset", "-g", "therm"], optional=True), "utc": utc()}
    return {"load": list(os.getloadavg()), "power": "unavailable", "thermal": "unavailable", "utc": utc()}


def source_paths():
    tracked = command(["git", "ls-files", "-z"]).split("\0")
    paths = {REPO / p for p in tracked if p and not p.startswith("docs/source/")}
    # Include new, untracked runtime sources/assets too: an exploratory build
    # may embed a newly versioned bundle before the working tree is committed.
    for base in (SUITE, REPO / "scripts/roundtrip_oracle", REPO / "anki_forge",
                 REPO / "contract_tools", REPO / "contracts"):
        excluded_dirs = {"target", "__pycache__", "artifacts", ".anki-forge-media"}
        if base == SUITE:
            excluded_dirs |= {".venv", ".work", ".tools", "results", "inputs"}
        for directory, dirs, files in os.walk(base):
            dirs[:] = [d for d in dirs if d not in excluded_dirs]
            paths.update(Path(directory) / name for name in files if not name.endswith(".pyc"))
    paths.add(REPO / "docs/superpowers/specs/2026-09-06-basic-export-benchmark-spec.md")
    return sorted(paths)


def identity_snapshot(adapters):
    from verify import sha256
    paths = source_paths()
    source = {str(p.relative_to(REPO)): sha256(p) if p.is_file() else "missing" for p in paths}
    executables = {str(p): sha256(p) if p.is_file() else "unavailable" for p in
                   [COLLECTOR, INSPECTOR, ORACLE] + [Path(a["command"][0]).resolve() for a in adapters]}
    # RECORD content and installed package files are both identities, not merely version strings.
    dependencies = {}
    for name in sorted(d.metadata["Name"] for d in importlib.metadata.distributions()):
        dist = importlib.metadata.distribution(name)
        files = {str(f): sha256(dist.locate_file(f)) for f in dist.files or []
                 if not str(f).endswith(".pyc") and dist.locate_file(f).is_file()}
        dependencies[name] = {"version": dist.version, "files_sha256": hashlib.sha256(workload.serialize(files)).hexdigest()}
    fixtures = {str(size): sha256(SUITE / f"inputs/basic-{size}.json") for size in workload.SIZES}
    return {"source_files": source, "executables": executables, "dependencies": dependencies, "fixtures": fixtures,
            "upstream_revision": command(["git", "rev-parse", "HEAD"], cwd=REPO / "docs/source/anki", optional=True),
            "upstream_dirty": command(["git", "status", "--porcelain"], cwd=REPO / "docs/source/anki", optional=True),
            "upstream_patch": command(["git", "diff", "--binary", "HEAD"], cwd=REPO / "docs/source/anki", optional=True)}


def manifest(adapters, fixture_evidence, mode, attempts, budget_bytes):
    import zstandard
    darwin = platform.system() == "Darwin"
    bundles = sorted((REPO / "anki_forge/assets").rglob("anki-forge-contract-bundle-*.tar.gz"))
    if len(bundles) != 1:
        raise RuntimeError("expected one embedded contract bundle")
    bundle = bundles[0].name.removeprefix("anki-forge-contract-bundle-").removesuffix(".tar.gz")
    metadata = {a["id"]: json.loads(command(a["command"] + ["--metadata"])) for a in adapters}
    if metadata["genanki"]["genanki"] != "0.13.1" or metadata["genanki"]["architecture"] != platform.machine():
        raise RuntimeError("wrong genanki version or cross-architecture comparator")
    feature_tree = command(["cargo", "tree", "--locked", "--offline", "--manifest-path", str(SUITE / "adapters/rust/Cargo.toml"), "-e", "features"])
    if 'anki_forge feature "internal-tools"' in feature_tree:
        raise RuntimeError("measured Rust adapter must not enable internal-tools")
    return {"schema": "basic-benchmark-run-v1", "spec_revision": 3, "created_utc": utc(), "mode": mode,
            "source_commit": command(["git", "rev-parse", "HEAD"]),
            "git_status": command(["git", "status", "--short"]),
            "identity_before": identity_snapshot(adapters), "adapters": adapters,
            "adapter_metadata": metadata,
            "fixture_evidence": fixture_evidence, "schedule": attempts, "schedule_seed": SCHEDULE_SEED,
            "timeout_seconds": 120, "storage_budget_bytes": budget_bytes, "oracle_selected_round": 1,
            "oracle": {"path": str(ORACLE), "available_before_measurement": ORACLE.is_file()},
            "toolchain": {"rustc": command(["rustc", "-vV"]), "cargo": command(["cargo", "--version"]),
                          "cc": command(["cc", "--version"]), "rust_profile": "release",
                          "rust_flags": os.environ.get("RUSTFLAGS", ""),
                          "cargo_encoded_rustflags": os.environ.get("CARGO_ENCODED_RUSTFLAGS", ""),
                          "rust_features": "default; no internal-tools", "feature_tree": feature_tree,
                          "crate_version": "0.1.0", "bundle_version": bundle},
            "runtime": {"python": sys.version, "python_version": platform.python_version(),
                        "executable": sys.executable, "architecture": platform.machine(),
                        "sqlite": sqlite3.sqlite_version, "zstandard": zstandard.__version__,
                        "zstd_library": list(zstandard.ZSTD_VERSION), "python_config_libdir": sysconfig.get_config_var("LIBDIR")},
            "host": {"system": platform.platform(), "machine": platform.machine(), "cpu_count": os.cpu_count(),
                     "cpu": command(["sysctl", "-n", "machdep.cpu.brand_string"], optional=True) if darwin else command(["lscpu"], optional=True),
                     "ram_bytes": command(["sysctl", "-n", "hw.memsize"], optional=True) if darwin else command(["cat", "/proc/meminfo"], optional=True),
                     "filesystem": command(["df", "-h", str(SUITE)]), "pre": host_state()},
            "metrics": {"time": "fresh-process export wall time, including startup and input parsing",
                        "time_unit": "ns", "timer": "CLOCK_MONOTONIC around direct posix_spawn to blocking wait4",
                        "memory": METRIC, "memory_scope": "single_process", "memory_unit": "bytes",
                        "memory_method": "minimal native collector; per-child wait4.ru_maxrss",
                        "quantiles": "Hyndman-Fan type 7: linear interpolation at (n-1)*p",
                        "cache_regime": "fresh processes after warmups; warmed uncontrolled filesystem caches",
                        "storage": "closed APKG; no fsync durability guarantee"},
            "confirmation": "not predeclared; descriptive run only", "interruptions": []}


def run_attempt(item, run, adapters, timeout=120):
    folder = run / "artifacts" / item["id"]
    folder.mkdir(parents=True, exist_ok=False)
    temp = folder / "tmp"
    temp.mkdir()
    output = folder / "output.apkg"
    adapter = next(a for a in adapters if a["id"] == item["adapter"])
    env = os.environ.copy()
    env.update(TMPDIR=str(temp), TMP=str(temp), TEMP=str(temp), PYTHONHASHSEED=str(workload.SEED),
               PYTHONNOUSERSITE="1", PYTHONDONTWRITEBYTECODE="1")
    argv = [str(COLLECTOR), str(timeout), str(folder / "stdout.log"), str(folder / "stderr.log")]
    argv += adapter["command"] + [str(SUITE / f"inputs/basic-{item['size']}.json"), str(output)]
    record = {**item, "started_utc": utc(), "artifact": str(output.relative_to(run))}
    process = None
    try:
        process = subprocess.Popen(argv, cwd=folder, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        out, err = process.communicate(timeout=timeout + 15)
        if process.returncode:
            record.update(status="collector_failure", reason=err.decode(errors="replace")[:1500])
        else:
            measured = json.loads(out)
            record["measurement"] = measured
            if measured["interrupted_signal"] in (signal.SIGINT, signal.SIGTERM):
                record["status"] = "cancelled"
            elif measured["interrupted_signal"]:
                record["status"] = "timeout"
            elif measured["spawn_error"] or not measured["reaped"] or measured["exit_code"] or measured["leftover_descendants"]:
                record["status"] = "adapter_failure"
            elif not output.is_file() or output.stat().st_size == 0:
                record["status"] = "missing_output"
            else:
                record["status"] = "success"
            record["memory"] = {"metric": METRIC, "scope": "single_process", "unit": "bytes",
                                "status": "available" if measured["reaped"] and measured["peak_rss_bytes"] > 0 else "unavailable"}
    except (KeyboardInterrupt, subprocess.TimeoutExpired) as error:
        if process is not None:
            process.send_signal(signal.SIGTERM)
            try:
                process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate()
        record.update(status="cancelled" if isinstance(error, KeyboardInterrupt) else "collector_failure", reason=type(error).__name__)
    except (OSError, ValueError) as error:
        record.update(status="collector_failure", reason=str(error))
    record["finished_utc"] = utc()
    return record


def artifact_size(root):
    return sum(p.stat().st_size for p in root.rglob("*") if p.is_file())


def verify_records(run, records):
    from verify import verify_artifact
    expected = {size: json.loads((SUITE / f"inputs/basic-{size}.json").read_text()) for size in workload.SIZES}
    results = {}
    for i, record in enumerate(records):
        if record["status"] == "success":
            results[record["id"]] = verify_artifact(run / record["artifact"], expected[record["size"]], INSPECTOR)
            if results[record["id"]]["status"] != "passed":
                print(f"VERIFY {record['id']}: {results[record['id']]}", flush=True)
        if (i + 1) % 16 == 0:
            print(f"Verified {i + 1}/{len(records)} artifacts", flush=True)
    return results


def complete_oracle(run):
    from verify import sha256
    m = json.loads((run / "manifest.json").read_text())
    verification = json.loads((run / "verification.json").read_text())
    raw = [json.loads(line) for line in (run / "attempts.jsonl").read_text().splitlines()]
    selected = [r for r in raw if r["role"] == "timing" and r["round"] == 1]
    evidence_path = run / "anki.json"
    evidence = json.loads(evidence_path.read_text()) if evidence_path.exists() else {}
    frozen_sha = m["identity_before"]["executables"].get(str(ORACLE))
    for record in selected:
        key = record["id"]
        if evidence.get(key, {}).get("status") == "passed":
            continue
        path = run / record.get("artifact", "missing")
        check = verification.get(key, {})
        if not m["oracle"]["available_before_measurement"] or not ORACLE.is_file() or sha256(ORACLE) != frozen_sha:
            evidence[key] = {"status": "missing_oracle", "reason": "same pinned oracle must be identified before measurement"}
        elif check.get("status") != "passed":
            evidence[key] = {"status": "unverified_artifact"}
        elif not path.is_file() or sha256(path) != check["artifact_sha256"]:
            evidence[key] = {"status": "missing_or_changed_artifact"}
        else:
            destination = run / "oracle-evidence" / f"{key}.json"
            destination.parent.mkdir(exist_ok=True)
            p = subprocess.run([str(ORACLE), str(SUITE / f"inputs/basic-{record['size']}.json"), str(path), str(destination)],
                               capture_output=True, timeout=120)
            if p.returncode:
                evidence[key] = {"status": "oracle_failed", "reason": p.stderr.decode(errors="replace")[:3000]}
            else:
                outcome = json.loads(destination.read_text())
                evidence[key] = {"status": outcome["status"], "artifact_sha256": check["artifact_sha256"],
                                 "oracle_sha256": frozen_sha, "upstream_revision": m["identity_before"]["upstream_revision"],
                                 "logical_sha256": check["physical"]["logical_sha256"],
                                 "evidence_file": str(destination.relative_to(run)), "evidence_sha256": sha256(destination),
                                 "notes": outcome["notes"], "cards": outcome["cards"], "rendered_categories": 3}
                if sha256(path) != check["artifact_sha256"]:
                    evidence[key] = {"status": "changed_during_oracle"}
        save(evidence_path, evidence)
        print(f"Anki {key}: {evidence[key]['status']}", flush=True)
    save(evidence_path, evidence)
    return evidence


def cleanup(run):
    verification = json.loads((run / "verification.json").read_text())
    anki = json.loads((run / "anki.json").read_text())
    records = [json.loads(line) for line in (run / "attempts.jsonl").read_text().splitlines()]
    retention = {}
    for r in records:
        key = r["id"]
        if verification.get(key, {}).get("status") != "passed":
            continue
        if r["role"] == "timing" and r["round"] == 1 and anki.get(key, {}).get("status") != "passed":
            retention[key] = "retained: selected Anki evidence incomplete"
            continue
        # Only this suite's attempt directories; logs and immutable raw records survive.
        folder = run / "artifacts" / key
        for p in folder.iterdir():
            if p.name in {"stdout.log", "stderr.log"}:
                continue
            if p.is_dir() and not p.is_symlink():
                shutil.rmtree(p)
            else:
                p.unlink()
        retention[key] = "cleaned after required verification"
    save(run / "retention.json", retention)


def execute(mode, name=None, budget_gib=8):
    from report import render
    if sys.version_info[:2] != (3, 11) or Path(sys.prefix).resolve() != (SUITE / ".venv").resolve():
        raise RuntimeError("run with benchmarks/.venv/bin/python (pinned CPython 3.11 environment)")
    adapters = registry()
    fixture_evidence = workload.generate()
    sizes = (200,) if mode == "smoke" else workload.SIZES
    attempts = [] if mode == "smoke" else schedule()
    run_name = name or dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + mode
    run = SUITE / ".work" / "runs" / run_name
    run.mkdir(parents=True, exist_ok=False)
    budget_bytes = int(budget_gib * 1024 ** 3)
    m = manifest(adapters, fixture_evidence, mode, attempts, budget_bytes)
    m["run_id"] = run_name
    save(run / "manifest.json", m)
    print(f"Run: {run}", flush=True)
    append(run / "events.jsonl", {"phase": "preflight", "utc": utc()})
    preflight = []
    for adapter in adapters:
        r = run_attempt({"id": f"preflight-200-{adapter['id']}", "role": "preflight", "adapter": adapter["id"],
                         "size": 200, "round": 0, "order": -1}, run, adapters)
        preflight.append(r)
        append(run / "attempts.jsonl", r)
    verified = verify_records(run, preflight)
    records = list(preflight)
    preflight_ok = all(r["status"] == "success" and verified.get(r["id"], {}).get("status") == "passed" for r in records)
    projected = sum(artifact_size(run / "artifacts" / r["id"]) for r in records) * (sum(sizes) / 200) * 21 * 4
    available = shutil.disk_usage(run).free
    m["storage_preflight"] = {"estimated_with_4x_headroom_bytes": int(projected), "available_bytes": available}
    save(run / "manifest.json", m)
    storage_floor = max(int(available * .2), available - budget_bytes)
    stopped = "preflight_failed" if not preflight_ok else "storage_exhaustion" if projected > min(budget_bytes, available * 0.8) else None
    failed_cells = set()
    current_role = None
    # Between samples: launch/reap, stat the output, append a small record. No artifact hashing,
    # decoding, imports, reporting, directory scanning, or deletion until both passes end.
    for item in attempts:
        # statvfs only: no scanning/hashing/deleting outputs during measurement.
        if not stopped and shutil.disk_usage(run).free < storage_floor:
            stopped = "storage_exhaustion"
        cell = (item["size"], item["adapter"], "memory" if item["role"].startswith("memory") else "timing")
        if stopped or cell in failed_cells:
            record = {**item, "status": "not_run", "reason": stopped or "earlier_attempt_failed"}
        else:
            if item["role"] != current_role:
                current_role = item["role"]
                append(run / "events.jsonl", {"phase": current_role, "utc": utc()})
                print(f"Phase: {current_role}", flush=True)
            record = run_attempt(item, run, adapters)
            print(f"{item['order'] + 1}/{len(attempts)} {item['id']}: {record['status']}", flush=True)
            if record["status"] == "cancelled":
                stopped = "cancelled"
                m["interruptions"].append({"attempt": item["id"], "utc": utc()})
            elif record["status"] != "success":
                failed_cells.add(cell)
        records.append(record)
        append(run / "attempts.jsonl", record)
    append(run / "events.jsonl", {"phase": "measurement_complete", "utc": utc()})
    m["host"]["post"] = host_state()
    append(run / "events.jsonl", {"phase": "deferred_verification", "utc": utc()})
    verified.update(verify_records(run, records[len(preflight):]))
    save(run / "verification.json", verified)
    save(run / "manifest.json", m)
    complete_oracle(run)
    m["identity_after"] = identity_snapshot(adapters)
    m["identity_unchanged"] = m["identity_before"] == m["identity_after"]
    m["completed_utc"] = utc()
    save(run / "manifest.json", m)
    cleanup(run)
    append(run / "events.jsonl", {"phase": "cleanup_complete", "utc": utc()})
    render(run)
    print(f"Report: {run / 'report.md'}", flush=True)
    return 0 if preflight_ok and all(r["status"] == "success" for r in records) and all(v["status"] == "passed" for v in verified.values()) else 1


def prepare(python, with_anki):
    setup = [] if (SUITE / ".venv/bin/python").is_file() else [["uv", "venv", str(SUITE / ".venv"), "--python", python]]
    for args in setup + [
                 ["uv", "pip", "sync", str(SUITE / "requirements.lock"), "--python", str(SUITE / ".venv/bin/python"), "--require-hashes"],
                 ["cargo", "build", "--release", "--locked", "--manifest-path", str(SUITE / "adapters/rust/Cargo.toml")],
                 ["cargo", "build", "--release", "--locked", "-p", "contract_tools"]]:
        subprocess.run(args, cwd=REPO, check=True)
    (SUITE / ".tools").mkdir(exist_ok=True)
    subprocess.run(["cc", "-O2", "-Wall", "-Wextra", "-Werror", "-pthread", str(SUITE / "native/measure.c"), "-o", str(COLLECTOR)], check=True)
    workload.generate()
    if with_anki:
        subprocess.run(["cargo", "build", "--locked", "--manifest-path", str(REPO / "scripts/roundtrip_oracle/Cargo.toml"), "--bin", "benchmark_oracle"], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    prep.add_argument("--python", default="python3.11")
    prep.add_argument("--with-anki", action="store_true")
    for kind in ("smoke", "run"):
        p = sub.add_parser(kind)
        p.add_argument("--name")
        p.add_argument("--budget-gib", type=float, default=8)
    for kind in ("report", "oracle", "cleanup"):
        sub.add_parser(kind).add_argument("run_dir", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.python, args.with_anki)
    elif args.command in ("smoke", "run"):
        return execute("full" if args.command == "run" else "smoke", args.name, args.budget_gib)
    elif args.command == "report":
        from report import render
        render(args.run_dir.resolve())
    elif args.command == "oracle":
        complete_oracle(args.run_dir.resolve())
    else:
        cleanup(args.run_dir.resolve())
    return 0


if __name__ == "__main__":
    sys.exit(main())
