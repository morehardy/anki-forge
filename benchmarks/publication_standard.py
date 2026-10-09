#!/usr/bin/env python3
"""Run the existing 5-profile × 4-size matrix with frozen publication binaries.

Use publication_performance.py --analyze for offline statistics. Fixture input
bytes and relative media names are copied unchanged from the standard generator.
"""
import argparse
import json
from pathlib import Path
import shutil

import publication_performance as measurement


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--binaries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    profiles = []
    for profile in ["basic-mixed-text-v1", "basic-image-unique-v2",
                    "basic-audio-unique-v2", "basic-mixed-shared-v2",
                    "basic-mixed-unique-v2"]:
        source = args.source / profile / "inputs"
        for count in [100, 200, 500, 1000]:
            name = f"{profile}-{count}"
            profiles.append(name)
            destination = args.fixtures / name
            if not destination.exists():
                destination.mkdir(parents=True)
                shutil.copy2(source / f"{count}.json", destination / "input.json")
                document = json.loads((destination / "input.json").read_text())
                for media in document.get("media", []):
                    target = destination / media["path"]
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source / media["path"], target)
    if args.prepare_only: return
    args.dependencies = [Path(__file__)]
    measurement.GROUPS = {"standard": ("combined", "build", "build", profiles, 1)}
    args.groups = "standard"
    measurement.run(args)
    measurement.save(args.output / "wrapper-sha256.json", {
        str(Path(__file__).resolve()): measurement.sha(Path(__file__))})


if __name__ == "__main__":
    main()
