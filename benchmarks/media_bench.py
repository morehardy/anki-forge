"""Five-scene, four-tier benchmark with separate timing/RSS and exact checks."""
import argparse
import datetime
import json
import os
import platform
from pathlib import Path
import random
import shutil
import statistics
import subprocess
import sys

import bench
import media_workload
import verify
import workload


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def suite_inputs(directory):
    result = []
    for profile in sorted(directory.iterdir()):
        if not profile.is_dir() or not (profile / "inputs").is_dir():
            continue
        for size in workload.SIZES:
            path = profile / "inputs" / f"{size}.json"
            document = json.loads(path.read_text())
            if document["note_count"] != size or len(document["notes"]) != size:
                raise ValueError(f"wrong fixture count: {path}")
            for media in document.get("media", []):
                source = path.parent / media["path"]
                if source.name != media["filename"] or bench.sha256(source) != media["sha256"]:
                    raise ValueError(f"fixture media changed: {source}")
            result.append((profile.name, size, path, document))
    if len(result) != 20:
        raise ValueError("expected five profiles, each with 100/200/500/1000 inputs")
    return result


def summarize(rows):
    result = []
    for profile, size in sorted({(row["profile"], row["size"]) for row in rows}):
        cell = {"profile": profile, "size": size}
        for adapter in ("rust", "genanki"):
            selected = [row for row in rows if (row["profile"], row["size"], row["adapter"]) == (profile, size, adapter)]
            times = [row["measurement"]["elapsed_ns"] / 1e6 for row in selected if row["role"] == "timing"]
            rss = [row["measurement"]["peak_rss_bytes"] / 2**20 for row in selected if row["role"] == "rss"]
            artifact_sizes = [row["artifact_bytes"] for row in selected if row["role"] == "timing" and "artifact_bytes" in row]
            cell[adapter] = {"n": len(times), "median_ms": statistics.median(times),
                             "min_ms": min(times), "max_ms": max(times),
                             "iqr_ms": statistics.quantiles(times, n=4)[2] - statistics.quantiles(times, n=4)[0] if len(times) > 1 else 0,
                             "rss_n": len(rss), "rss_mib": statistics.median(rss) if rss else None,
                             "rss_max_mib": max(rss) if rss else None,
                             "artifact_bytes": statistics.median(artifact_sizes) if artifact_sizes else None}
        cell["reduction"] = 1 - cell["rust"]["median_ms"] / cell["genanki"]["median_ms"]
        cell["target_passed"] = cell["reduction"] >= .30
        result.append(cell)
    return result


def run(args):
    if sys.version_info[:2] != (3, 11) or Path(sys.prefix).resolve() != (bench.SUITE / ".venv").resolve():
        raise RuntimeError("run with benchmarks/.venv/bin/python")
    if not args.name or Path(args.name).name != args.name or args.name in (".", ".."):
        raise ValueError("run name must be a single directory name")
    destination = bench.SUITE / ".work/runs" / args.name
    destination.mkdir(parents=True, exist_ok=False)
    inputs = args.inputs.resolve() if args.inputs else destination / "fixtures"
    if args.inputs is None:
        media_workload.generate(inputs)
    cases = suite_inputs(inputs)
    workload.generate()
    adapters = bench.registry()
    identity = bench.identity_snapshot(adapters)
    inputs_identity = {str(path): bench.sha256(path) for _, _, path, _ in cases}
    metadata = {adapter["id"]: json.loads(subprocess.check_output(adapter["command"] + ["--metadata"], text=True)) for adapter in adapters}
    rust_configuration = bench.rust_adapter_configuration(metadata["rust"])
    if metadata["genanki"].get("genanki") != "0.13.1" or metadata["genanki"].get("architecture") != platform.machine():
        raise RuntimeError("wrong genanki version or cross-architecture comparator")
    feature_tree = bench.command(["cargo", "tree", "--locked", "--offline", "--manifest-path", str(bench.SUITE / "adapters/rust/Cargo.toml"), "-e", "features"] + bench.rust_feature_args(rust_configuration["adapter_features"]))
    if 'anki_forge feature "internal-tools"' in feature_tree:
        raise RuntimeError("measured Rust adapter must use public default features")
    provenance = bench.build_provenance(adapters)
    required = [bench.COLLECTOR, bench.INSPECTOR, Path(next(adapter for adapter in adapters if adapter["id"] == "rust")["command"][0])]
    if not args.skip_oracle:
        required.append(bench.ORACLE)
    if any(provenance[str(path.resolve())]["status"] != "verified" for path in required):
        raise RuntimeError("missing or stale build provenance; prepare binaries with bench.run_build first")
    manifest = {"schema": "media-benchmark-v1", "created_utc": bench.utc(), "sizes": list(workload.SIZES),
                "profiles": sorted({profile for profile, _, _, _ in cases}), "seed": 20260907,
                "timing_repeats": args.repeats, "rss_repeats": args.rss_repeats, "warmups_per_phase": 3,
                "adapter_metadata": metadata, "identity_before": identity,
                "inputs": inputs_identity, "host_before": bench.host_state(),
                "build_provenance": provenance, "rust_configuration": rust_configuration,
                "rust_feature_tree": feature_tree, "rustc": bench.command(["rustc", "--version", "--verbose"]),
                "platform": platform.platform(), "machine": platform.machine(),
                "source_commit": bench.command(["git", "rev-parse", "HEAD"]),
                "git_status": bench.command(["git", "status", "--short"]),
                "workloads": [{"profile": profile, "notes": size, "media_files": len(document.get("media", [])),
                               "media_bytes": sum(item["bytes"] for item in document.get("media", []))}
                              for profile, size, _, document in cases],
                "timing_scope": "native collector: exporter process creation through exit; RSS collected in a separate pass",
                "cold_cache_controlled": False, "verification_schedule": "after each adjacent adapter pair, outside collector", "oracle_required": not args.skip_oracle,
                "fixture_protocol": "frozen external suite" if args.inputs else "portable synthetic PNG/WAV v2"}
    save(destination / "manifest.json", manifest)
    if not args.skip_oracle and not bench.ORACLE.is_file():
        raise RuntimeError("Anki oracle is required; prepare it first or explicitly use --skip-oracle for exploratory runs")
    rows = []
    rng = random.Random(20260907)
    first_adapter = {(profile, size): rng.randrange(2) for profile, size, _, _ in cases}
    phases = [("timing_warmup", 3), ("timing", args.repeats), ("rss_warmup", 3), ("rss", args.rss_repeats)]
    try:
        for role, repeats in phases:
            for repeat in range(repeats):
                cells = list(cases)
                rng.shuffle(cells)
                for profile, size, input_path, document in cells:
                    order = list(adapters)
                    if (repeat + first_adapter[(profile, size)]) % 2:
                        order.reverse()
                    pending = []
                    for adapter in order:
                        if shutil.disk_usage(destination).free < 512 * 1024**2:
                            raise RuntimeError("benchmark stopped below the 512 MiB free-space floor")
                        name = f"{role}-{profile}-{size}-{repeat}-{adapter['id']}"
                        case = destination / name
                        case.mkdir()
                        output = case / "output.apkg"
                        env = os.environ.copy()
                        env.update(TMPDIR=str(case), TMP=str(case), TEMP=str(case), PYTHONHASHSEED=str(workload.SEED))
                        process = subprocess.run([str(bench.COLLECTOR), "60", str(case / "stdout.log"), str(case / "stderr.log"),
                                                  *adapter["command"], str(input_path), str(output)],
                                                 capture_output=True, text=True, env=env, timeout=75)
                        measurement = json.loads(process.stdout)
                        row = {"id": name, "role": role, "profile": profile, "size": size, "repeat": repeat,
                               "adapter": adapter["id"], "measurement": measurement}
                        with (destination / "attempts.jsonl").open("a") as stream:
                            stream.write(json.dumps(row) + "\n")
                        if process.returncode or measurement.get("exit_code") != 0 or measurement.get("signal") or measurement.get("leftover_descendants"):
                            raise RuntimeError(f"export failed: {name}; inspect retained logs")
                        pending.append((row, case, output))
                    # Verify after both adjacent timed exporters have exited.
                    for row, case, output in pending:
                        name = row["id"]
                        checked = verify.verify_artifact(output, document, bench.INSPECTOR)
                        save(case / "verification.json", checked)
                        if checked["status"] != "passed":
                            raise RuntimeError(f"artifact validation failed: {name}: {checked}")
                        row["artifact_sha256"] = checked["artifact_sha256"]
                        row["artifact_bytes"] = checked["artifact_bytes"]
                        if role == "timing" and repeat == 0 and not args.skip_oracle:
                            with (case / "oracle.stdout.log").open("w") as stdout, (case / "oracle.stderr.log").open("w") as stderr:
                                subprocess.run([str(bench.ORACLE), str(input_path), str(output), str(case / "anki.json")],
                                               check=True, stdout=stdout, stderr=stderr, timeout=120)
                            if json.loads((case / "anki.json").read_text()).get("status") != "passed":
                                raise RuntimeError(f"Anki oracle did not pass: {name}")
                            row["oracle_sha256"] = bench.sha256(case / "anki.json")
                        rows.append(row)
                        with (destination / "verified.jsonl").open("a") as stream:
                            stream.write(json.dumps(row) + "\n")
                        output.unlink()
                print(f"{role} repeat {repeat + 1}/{repeats}: all 20 cells verified", flush=True)
        identity_after = bench.identity_snapshot(adapters)
        if identity_after != identity:
            raise RuntimeError("source, executable, dependencies or fixture identity changed during measurement")
        suite_inputs(inputs)
        if inputs_identity != {str(path): bench.sha256(path) for _, _, path, _ in cases}:
            raise RuntimeError("inputs changed during measurement")
        manifest.update(completed_utc=bench.utc(), identity_unchanged=True, host_after=bench.host_state(), status="completed")
    except BaseException as error:
        manifest.update(status="failed", error=str(error), completed_utc=bench.utc())
        raise
    finally:
        save(destination / "manifest.json", manifest)
    summary = summarize([row for row in rows if row["role"] in ("timing", "rss")])
    save(destination / "summary.json", summary)
    lines = ["# Media export benchmark", "", f"Timing: {args.repeats} independent processes per cell; RSS: {args.rss_repeats} separate processes.", "",
             "| Profile | Notes | Rust ms | genanki ms | Time reduction | Rust RSS MiB | genanki RSS MiB |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for row in summary:
        rust, genanki = row["rust"], row["genanki"]
        lines.append(f"| {row['profile']} | {row['size']} | {rust['median_ms']:.3f} | {genanki['median_ms']:.3f} | {100*row['reduction']:.1f}% | {rust['rss_mib'] or 0:.2f} | {genanki['rss_mib'] or 0:.2f} |")
    lines += ["", f"Cells meeting the proposed 30% reduction: {sum(row['target_passed'] for row in summary)}/20.",
              "", "All successful samples passed original SQLite row/template and exact media payload checks. Anki evidence, when enabled, covers one package per cell/adapter. No GUI or audible playback is checked."]
    (destination / "report.md").write_text("\n".join(lines) + "\n")
    print(destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default=datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ-media"))
    parser.add_argument("--inputs", type=Path, help="frozen suite containing PROFILE/inputs/{100,200,500,1000}.json")
    parser.add_argument("--repeats", type=int, default=10)
    parser.add_argument("--rss-repeats", type=int, default=5)
    parser.add_argument("--skip-oracle", action="store_true", help="exploratory runs only")
    args = parser.parse_args()
    if args.repeats < 1 or args.rss_repeats < 1:
        parser.error("repeat counts must be positive")
    run(args)


if __name__ == "__main__":
    main()
