#!/usr/bin/env python3
"""Restore deterministic standard media beside archived publication inputs.

Extract the fixture-input and baseline archives first. Large random WAV controls
are archived directly; standard PNG/WAV assets use the existing frozen generator.
Every restored payload must match the input document's SHA-256.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil

from media_workload import png, wav


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    payloads = {}
    checked = 0
    for input_path in sorted(args.root.rglob("input.json")):
        document = json.loads(input_path.read_text())
        for media in document.get("media", []):
            target = input_path.parent / media["path"]
            if not target.exists():
                match = re.fullmatch(r"(image|audio)-(\d+)\.(png|wav)", media["filename"])
                if not match:
                    raise ValueError(f"required archived payload is missing: {target}")
                identity = media["filename"]
                if identity not in payloads:
                    generator = png if match[1] == "image" else wav
                    payloads[identity] = generator(int(match[2]))
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(payloads[identity])
            actual = hashlib.sha256(target.read_bytes()).hexdigest()
            assert actual == media["sha256"], target
            checked += 1
    sdk = args.root / "sdk-fixtures"
    if sdk.exists():
        destination = args.root / "node-fixtures"
        destination.mkdir(exist_ok=True)
        for profile in ["text1k", "wide1k", "text10k"]:
            shutil.copy2(sdk / profile / "input.json", destination / f"{profile}.json")
            shutil.copy2(sdk / profile / "baseline.apkg", destination / f"{profile}.apkg")
    print(json.dumps({"status": "verified", "media_references": checked}))


if __name__ == "__main__":
    main()
