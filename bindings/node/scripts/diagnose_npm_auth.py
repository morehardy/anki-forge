"""Diagnose npm OIDC against a frozen candidate without uploading packages."""

import argparse
import os
from pathlib import Path
import subprocess

from npm_release import REGISTRY, require, verify_local


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    args = parser.parse_args()
    record = verify_local(args.directory)
    require(record["run_id"] == args.run_id, "Candidate is from a different original run")
    require(record["ref"] == f"npm-v{record['version']}", "Candidate must come from a release tag")
    print(f"Original candidate: {record['ref']}, commit {record['commit']}, run {record['run_id']}", flush=True)
    for name in ("GITHUB_ACTIONS", "ACTIONS_ID_TOKEN_REQUEST_URL", "ACTIONS_ID_TOKEN_REQUEST_TOKEN"):
        available = bool(os.environ.get(name))
        print(f"{name} available: {available}", flush=True)
        require(available, f"Missing GitHub OIDC environment: {name}")
    subprocess.run(["node", "--version"], check=True)
    subprocess.run(["npm", "--version"], check=True)
    failed = []
    for item in record["packages"]:
        result = subprocess.run([
            "npm", "publish", str((args.directory / item["filename"]).resolve()),
            "--dry-run", "--ignore-scripts", "--access", "public", "--tag", "next",
            "--registry", REGISTRY, "--loglevel", "verbose",
        ], capture_output=True, text=True)
        # Emit only the authentication diagnostics; never print credentials or full HTTP responses.
        diagnostics = [line for line in result.stderr.splitlines()
                       if "oidc" in line.lower() or "npm warn publish" in line or "npm error code" in line]
        print(f"Authentication dry-run: {item['name']}", flush=True)
        for line in diagnostics:
            print(line, flush=True)
        if result.returncode or not any("Successfully retrieved and set token" in line for line in diagnostics):
            failed.append(item["name"])
    require(not failed, "OIDC authentication failed: " + ", ".join(failed))
    print("All five authentication dry-runs passed; no packages or dist-tags were changed.")


if __name__ == "__main__":
    main()
