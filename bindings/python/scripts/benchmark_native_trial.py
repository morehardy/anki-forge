"""Measure the CLI baseline and release native slice in separate processes."""
from __future__ import annotations

import argparse
import gc
import json
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time


def worker(mode: str, source: Path, count: int) -> dict[str, object]:
    sys.path.insert(0, str(source))
    started = time.perf_counter()
    import anki_forge
    from anki_forge import Note, Project
    import_ms = (time.perf_counter() - started) * 1000
    assert Path(anki_forge.__file__).is_relative_to(source)
    with tempfile.TemporaryDirectory(prefix="anki-forge-benchmark-") as directory:
        root = Path(directory)
        started = time.perf_counter()
        project = Project("Benchmark", stable_id="benchmark")
        for index in range(count):
            project.add_note(Note.basic(f"Front {index}", "Answer", stable_id=f"note-{index}"))
        authoring_ms = (time.perf_counter() - started) * 1000
        builds = []
        for _ in range(3):
            started = time.perf_counter()
            report = project.write_apkg(root / "deck.apkg")
            report.ensure_success()
            assert report.counts["notes"] == count
            builds.append((time.perf_counter() - started) * 1000)
        source_file = root / "media.wav"
        source_file.write_bytes(b"RIFF" + bytes(512 * 1024 - 4))
        started = time.perf_counter()
        for index in range(10):
            project.media.add_file(source_file, export_as=f"media-{index}.wav")
        media_ms = (time.perf_counter() - started) * 1000
        leaked = None
        if mode == "native":
            cleanup = Project("Cleanup").add_note(Note.basic("Front", "Back"))
            paths = []
            for _ in range(10):
                report = cleanup.build()
                report.ensure_success()
                paths.append(report.artifact.path)
                del report
            gc.collect()
            leaked = sum(path.exists() for path in paths)
            assert leaked == 0
        return {"mode": mode, "notes": count, "import_ms": import_ms,
                "authoring_ms": authoring_ms, "first_build_ms": builds[0],
                "repeated_build_ms": builds[1:], "ten_file_registrations_ms": media_ms,
                "temporary_artifacts_leaked": leaked}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--worker", choices=["cli", "native"])
    parser.add_argument("--source", type=Path)
    parser.add_argument("--notes", type=int)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.worker:
        print(json.dumps(worker(args.worker, args.source.resolve(), args.notes)))
        return
    root = Path(__file__).resolve().parents[3]
    script = Path(__file__).resolve()
    baseline = subprocess.check_output(["git", "rev-parse", "51a44ad"], cwd=root, text=True).strip()
    result = {"platform": platform.platform(), "python": sys.version, "cli_source_commit": baseline,
              "native_profile": "release", "scope": "Basic notes; local measurements, not a speed guarantee", "samples": []}
    with tempfile.TemporaryDirectory(prefix="anki-forge-benchmark-source-") as directory:
        old_source = Path(directory)
        paths = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", baseline, "bindings/python/src"], cwd=root, text=True).splitlines()
        for path in paths:
            target = old_source / Path(path).relative_to("bindings/python/src")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(subprocess.check_output(["git", "show", f"{baseline}:{path}"], cwd=root))
        for mode, source in [("cli", old_source), ("native", root / "bindings/python/src")]:
            for count in (1000, 10000):
                completed = subprocess.check_output([
                    sys.executable, str(script), "--worker", mode, "--source", str(source), "--notes", str(count),
                ], cwd=root, text=True)
                result["samples"].append(json.loads(completed))
    if args.output is None:
        parser.error("--output is required")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
