#!/usr/bin/env python3
"""Additional full controls and real SDK review/publication workflows."""
import argparse
from pathlib import Path
import shutil

import publication_performance as measurement


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["native", "node", "python"], required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--binaries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--python", type=Path)
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    binaries = args.binaries.resolve()
    args.dependencies = [Path(__file__)]
    if args.kind == "native":
        args.scope = "collector spawn-to-exit; staging-control includes in-child manifest verification, APKG copy and cleanup in elapsed/RSS, but operation_ms excludes them; external APKG verification and Anki import are outside timing"
        measurement.GROUPS = {
            "reuse-compare-control": ("prepared", "compare", "compare", ["text1k", "image1k", "wide1k", "text10k"], 1),
            "staging-control": ("manifest-control", "staging", "staging", ["text1k", "wide1k"], 1),
            "concurrent-miss": ("media-control", "bytes-concurrent", "bytes-concurrent", ["bytes2m", "bytes8m"], 8),
        }
        def command(case, input_path, output, mode, baseline):
            variant = measurement.GROUPS[case["group"]][0]
            if case["mode"] == "baseline":
                variant = "baseline" if mode == "compare" else "baseline-control"
            return ([str(binaries / variant), str(input_path), str(output), mode, str(baseline)], {})
    else:
        measurement.GROUPS = {
            f"sdk-{args.kind}": ("prepared", "compare-build", "prepare-publish", ["text1k", "wide1k", "text10k"], 1)
        }
        sdk = repository / "bindings/node/dist/index.mjs"
        script = Path(__file__).with_name(f"publication_sdk_workflow.{ 'mjs' if args.kind == 'node' else 'py' }")
        native = binaries / "node-before.node"
        args.dependencies += [script, sdk, native]
        if args.kind == "python":
            if args.python is None: parser.error("--python is required for Python SDK controls")
            args.dependencies += list((repository / "bindings/python/src/ankiforge").glob("*"))
        def command(case, input_path, output, mode, baseline):
            trailing = [str(input_path), str(output), mode, str(baseline)]
            if args.kind == "node":
                return ([shutil.which("node"), str(script), str(sdk), *trailing], {"ANKI_FORGE_NATIVE_PATH": str(native)})
            return ([str(args.python.absolute()), str(script), *trailing], {"PYTHONPATH": str(repository / "bindings/python/src")})
    args.command = command
    args.groups = ",".join(measurement.GROUPS)
    measurement.run(args)


if __name__ == "__main__":
    main()
