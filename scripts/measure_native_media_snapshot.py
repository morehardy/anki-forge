#!/usr/bin/env python3
"""Measure default-public-API Media ownership in isolated native child processes.

The Rust probe is also exercised by media_snapshot_lifecycle_tests. No private
Rust API or pointer identity is used. RSS is the child's OS high-water mark;
filesystem samples observe only the process-specific snapshot directory.
"""
from __future__ import annotations

import argparse
import datetime
import json
import hashlib
import os
import pathlib
import platform
import shutil
import stat
import struct
import subprocess
import sys
import tempfile
import time

REPO = pathlib.Path(__file__).resolve().parents[1]
SOURCE = REPO / "anki_forge/tests/consumers/media_snapshot_lifecycle.rs"


def build_probe(workspace: pathlib.Path) -> pathlib.Path:
    name = f"ankiforge_media_measurement_{os.getpid()}"
    crate = workspace / "consumer"
    (crate / "src").mkdir(parents=True)
    shutil.copyfile(SOURCE, crate / "src/main.rs")
    core_path = json.dumps(str(REPO / "anki_forge"))
    (crate / "Cargo.toml").write_text(
        f'''[package]
name = "{name}"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
ankiforge = {{ path = {core_path}, default-features = false }}
anyhow = "1"
serde_json = "1"
blake3 = "1"
zip = {{ version = "2", default-features = false, features = ["deflate"] }}
zstd = "0.13"
''', encoding="utf-8"
    )
    target = REPO / "target/public-consumer"
    subprocess.run(
        ["cargo", "build", "--offline", "--quiet", "--manifest-path", str(crate / "Cargo.toml")],
        env=dict(os.environ, CARGO_TARGET_DIR=str(target)), check=True,
    )
    return target / "debug" / name


def generate_wave(path: pathlib.Path, size: int) -> None:
    if size < 44 or size >= 1 << 32:
        raise ValueError("probe WAV size must fit the RIFF u32 length")
    header = struct.pack("<4sI4s4sIHHIIHH4sI", b"RIFF", size - 8, b"WAVE",
                         b"fmt ", 16, 1, 1, 8000, 8000, 1, 8, b"data", size - 44)
    chunk = bytes([128]) * (64 * 1024)
    with path.open("wb") as destination:
        destination.write(header)
        remaining = size - len(header)
        while remaining:
            count = min(len(chunk), remaining)
            destination.write(chunk[:count])
            remaining -= count


def disk_usage(directory: pathlib.Path) -> dict[str, int]:
    logical = allocated = files = 0
    for entry in os.scandir(directory):
        try:
            item = entry.stat(follow_symlinks=False)
        except FileNotFoundError:
            continue  # A temporary duplicate may be removed between samples.
        if stat.S_ISREG(item.st_mode):
            files += 1
            logical += item.st_size
            allocated += getattr(item, "st_blocks", 0) * 512
    return {"files": files, "logical_bytes": logical, "allocated_bytes": allocated}


def measure(binary: pathlib.Path, workspace: pathlib.Path, mode: str,
            source: pathlib.Path | None, repeats: int, interval: float) -> dict:
    workspace.mkdir()
    snapshots = workspace / "snapshots"
    snapshots.mkdir()
    if source is not None:
        os.link(source, workspace / "source.wav")
    output_path, error_path = workspace / "stdout.jsonl", workspace / "stderr.txt"
    child_env = dict(os.environ, TMPDIR=str(snapshots), TMP=str(snapshots), TEMP=str(snapshots))
    peaks = {"files": 0, "logical_bytes": 0, "allocated_bytes": 0}
    samples = 0
    started = time.perf_counter()
    with output_path.open("wb") as output, error_path.open("wb") as errors:
        process = subprocess.Popen([str(binary), mode, str(workspace), str(repeats)],
                                   stdout=output, stderr=errors, env=child_env)
        while True:
            current = disk_usage(snapshots)
            for key in peaks:
                peaks[key] = max(peaks[key], current[key])
            samples += 1
            waited, status, usage = os.wait4(process.pid, os.WNOHANG)
            if waited:
                process.returncode = os.waitstatus_to_exitcode(status)
                break
            if time.perf_counter() - started > 180:
                process.kill()
                _, status, _ = os.wait4(process.pid, 0)
                process.returncode = os.waitstatus_to_exitcode(status)
                raise RuntimeError(f"{mode} timed out")
            time.sleep(interval)
    elapsed = time.perf_counter() - started
    if process.returncode:
        raise RuntimeError(f"probe failed ({process.returncode}):\n{output_path.read_text()}\n{error_path.read_text()}")
    observations = [json.loads(line) for line in output_path.read_text().splitlines()]
    final = disk_usage(snapshots)
    if final["files"] != 0:
        raise RuntimeError(f"retained snapshot files after probe exited: {final}")
    rss_unit = 1 if sys.platform == "darwin" else 1024
    return {
        "mode": mode,
        "input_bytes": source.stat().st_size if source else 0,
        "imports": repeats,
        "wall_ms": round(elapsed * 1000, 3),
        "peak_rss_bytes": usage.ru_maxrss * rss_unit,
        "user_cpu_ms": round(usage.ru_utime * 1000, 3),
        "system_cpu_ms": round(usage.ru_stime * 1000, 3),
        "disk": {
            "sample_interval_ms": interval * 1000,
            "samples": samples,
            "sampled_peaks": peaks,
            "checkpoint_peak_retained_logical_bytes": max(
                (row["storage"]["logical_bytes"] for row in observations), default=0),
            "checkpoint_peak_retained_files": max(
                (row["storage"]["files"] for row in observations), default=0),
            "after_exit": final,
        },
        "observations": observations,
    }


def version(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sizes-mib", nargs="+", type=int, default=[64, 256])
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--bytes-size-mib", type=int, default=64,
                        help="also measure caller-owned byte imports; zero disables")
    parser.add_argument("--sample-ms", type=float, default=2)
    parser.add_argument("--binary", type=pathlib.Path, help="reuse an already-built probe")
    parser.add_argument("--output", type=pathlib.Path,
                        default=REPO / "docs/plans/evidence/rust-api-clean-slate-2026-09-24/media-snapshot.json")
    args = parser.parse_args()
    if sys.platform not in ("darwin", "linux") or not hasattr(os, "wait4"):
        parser.error("this RSS observer requires macOS/Linux wait4; no cross-platform memory claim is made")
    if args.repeats < 2 or args.sample_ms <= 0:
        parser.error("repeats must be >=2 and sampling interval must be positive")
    sizes = set(args.sizes_mib)
    if args.bytes_size_mib:
        sizes.add(args.bytes_size_mib)
    if any(size < 8 or size > 256 for size in sizes):
        parser.error("this disk-spill probe requires 8..256 MiB inputs within the default media budget")
    results = []
    with tempfile.TemporaryDirectory(prefix="ankiforge-media-measure-") as directory:
        workspace = pathlib.Path(directory)
        binary = args.binary.resolve() if args.binary else build_probe(workspace)
        sources = {}
        for size in sorted(sizes):
            source = workspace / f"source-{size}.wav"
            generate_wave(source, size << 20)
            sources[size] = source
        cases = [("measure-baseline", None, 0)]
        for size in args.sizes_mib:
            cases.extend(("measure-file", sources[size], count) for count in (1, args.repeats))
        if args.bytes_size_mib:
            cases.extend(("measure-bytes", sources[args.bytes_size_mib], count) for count in (1, args.repeats))
        for index, (mode, source, repeats) in enumerate(cases):
            result = measure(binary, workspace / f"case-{index}", mode, source, repeats, args.sample_ms / 1000)
            results.append(result)
            print(f"{mode}: {result['input_bytes'] >> 20} MiB x{repeats}, "
                  f"RSS {result['peak_rss_bytes'] / (1 << 20):.2f} MiB, "
                  f"wall {result['wall_ms']:.1f} ms, "
                  f"retained files {result['disk']['checkpoint_peak_retained_files']}", flush=True)
    evidence = {
        "schema_version": "ankiforge-media-snapshot-measurement-v1",
        "measured_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "environment": {
            "platform": platform.platform(), "machine": platform.machine(),
            "python": platform.python_version(), "rustc": version(["rustc", "--version"]),
            "cargo": version(["cargo", "--version"]), "build_profile": "debug",
            "probe_sha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
            "media_snapshot_source_sha256": hashlib.sha256((REPO / "anki_forge/src/media/snapshot.rs").read_bytes()).hexdigest(),
            "consumer": "standalone crate; ankiforge default-features=false; no internal-tools",
        },
        "method": {
            "input": "exact-size 8-bit mono PCM WAV; generated before child startup; warm filesystem cache; no cache eviction",
            "rss": "per-child wait4 ru_maxrss, converted from bytes on macOS / KiB on Linux; includes loader, allocator and caller Vec for bytes mode",
            "disk": f"{args.sample_ms:g} ms nominal polling of isolated snapshot directory plus synchronous post-import checkpoints; source inputs and build outputs excluded",
            "limitations": [
                "One sample per scenario; this is not a statistical performance bound.",
                "RSS includes all child allocations and does not include all filesystem page-cache memory or system-wide memory.",
                "Whole-process wall_ms includes startup and observer polling; import_ms separately times each API call plus checkpoint bookkeeping. Do not subtract baseline latency or RSS to infer a precise incremental cost.",
                "Sampled disk peaks can miss short-lived transients; checkpoint values prove retained storage only.",
                "Allocated bytes use filesystem st_blocks*512 and are not a portable physical-device accounting guarantee.",
                "Debug build, cache state, filesystem, hardware and concurrent workload affect latency and RSS.",
                "Repeated imports still read/hash/spool the input before sharing retained storage; a temporary second spool is allowed.",
                "Lifecycle and failure correctness are asserted by the separate default-consumer test, not inferred from RSS thresholds.",
            ],
        },
        "results": results,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(f"Evidence: {args.output}")


if __name__ == "__main__":
    main()
