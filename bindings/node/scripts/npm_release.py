"""Audit, freeze and publish the five npm tarballs. Python 3.11+, stdlib only.

Publication is only allowed from the original protected-tag GitHub Actions run.
All retries verify registry bytes; an existing name/version is never overwritten.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import time
import tomllib
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = "https://registry.npmjs.org/"
REPOSITORY = "morehardy/anki-forge"
PLATFORMS = {
    "darwin-arm64": ("darwin", "arm64", None),
    "darwin-x64": ("darwin", "x64", None),
    "linux-x64-gnu": ("linux", "x64", "glibc"),
    "win32-x64-msvc": ("win32", "x64", None),
}
NAMES = [f"ankiforge-{suffix}" for suffix in PLATFORMS] + ["ankiforge"]
HOOKS = {"preinstall", "install", "postinstall", "prepare", "prepack", "postpack",
         "prepublish", "prepublishOnly", "publish", "postpublish"}
PUBLISH_CONFIG = {"access": "public", "registry": REGISTRY, "tag": "next"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def stable_version(version: str) -> tuple[int, ...]:
    require(bool(re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version)),
            f"Expected a stable X.Y.Z version, got {version!r}")
    return tuple(map(int, version.split(".")))


def audit_manifest(manifest: dict, name: str, version: str) -> None:
    require(manifest.get("name") == name and manifest.get("version") == version,
            f"Package identity mismatch: {name}@{version}")
    require(not manifest.get("private"), f"Private package: {name}")
    require(manifest.get("license") == "MIT", f"Unexpected license: {name}")
    require(manifest.get("publishConfig") == PUBLISH_CONFIG, f"Unsafe publishConfig: {name}")
    require(not HOOKS.intersection(manifest.get("scripts", {})), f"Lifecycle hook: {name}")
    require(manifest.get("engines") == {"node": ">=22.13.0"}, f"Node support drift: {name}")
    require(manifest.get("repository", {}).get("url") in (
        f"https://github.com/{REPOSITORY}.git", f"git+https://github.com/{REPOSITORY}.git"),
        f"Repository must match the trusted publisher: {name}")
    require(not manifest.get("dependencies"), f"Unexpected runtime dependency: {name}")
    if name == "ankiforge":
        require(manifest.get("optionalDependencies") == {n: version for n in NAMES[:-1]},
                "All four native dependencies must use the exact SDK version")
        require(manifest.get("exports", {}).get(".") == {
            "import": {"types": "./dist/index.d.mts", "default": "./dist/index.mjs"},
            "require": {"types": "./dist/cjs/index.d.ts", "default": "./dist/cjs/index.js"}},
            "Missing ESM/CJS/TypeScript entry points")
    else:
        system, cpu, libc = PLATFORMS[name.removeprefix("ankiforge-")]
        for field, expected in (("os", [system]), ("cpu", [cpu]), ("libc", [libc] if libc else None)):
            require(manifest.get(field) == expected, f"Incorrect {field}: {name}")
        require(manifest.get("main") == "anki-forge.node", f"Native entry point: {name}")
        require(not manifest.get("scripts") and not manifest.get("optionalDependencies"),
                f"Unexpected native package dependencies/scripts: {name}")


def metadata(root: Path = ROOT, tag: str | None = None) -> str:
    main = read_json(root / "package.json")
    version = main["version"]
    stable_version(version)
    if tag is not None:
        require(tag == f"npm-v{version}", "Release tag must match package.json: npm-vX.Y.Z")
    audit_manifest(main, "ankiforge", version)
    native = tomllib.loads((root / "native/Cargo.toml").read_text(encoding="utf-8"))
    require(native["package"]["version"] == version, "Native Cargo package version differs")
    loader = (root / "src/internal/native.ts").read_text(encoding="utf-8")
    require(f'export const VERSION = "{version}";' in loader, "Loader version differs")
    require('const packageName = `ankiforge-${suffix}`;' in loader, "Loader uses the old package name")
    for suffix in PLATFORMS:
        directory = root / "npm" / suffix
        audit_manifest(read_json(directory / "package.json"), f"ankiforge-{suffix}", version)
        for filename in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
            require((directory / filename).read_bytes() == (root / filename).read_bytes(),
                    f"Stale {filename} in {suffix}")
    return version


def inspect_tarball(path: Path, name: str, version: str) -> dict:
    payload = {}
    with tarfile.open(path, "r:gz") as archive:
        for member in archive:
            parts = PurePosixPath(member.name).parts
            require(parts and parts[0] == "package" and ".." not in parts
                    and "\\" not in member.name, f"Unsafe archive path: {member.name}")
            if member.isdir():
                continue
            require(member.isfile(), f"Archive links are forbidden: {member.name}")
            filename = "/".join(parts[1:])
            require(filename not in payload, f"Duplicate archive entry: {filename}")
            payload[filename] = archive.extractfile(member).read()
    manifest = json.loads(payload["package.json"])
    audit_manifest(manifest, name, version)
    required = {"package.json", "README.md", "LICENSE", "THIRD_PARTY_NOTICES.md"}
    if name == "ankiforge":
        required |= {"dist/index.mjs", "dist/index.d.mts", "dist/cjs/index.js",
                     "dist/cjs/index.d.ts", "dist/cjs/package.json", "RELEASING.md", "COVERAGE.md"}
        for filename in payload:
            require(filename in required or (filename.startswith("dist/cjs/") and filename.endswith((".js", ".d.ts"))),
                    f"Unexpected main package payload: {filename}")
        require(json.loads(payload["dist/cjs/package.json"]) == {"type": "commonjs"}, "CJS boundary missing")
    else:
        required.add("anki-forge.node")
        require(set(payload) == required, f"Unexpected native payload: {name}")
    require(required <= payload.keys(), f"Missing required files: {name}: {required - payload.keys()}")
    require(all(payload[f] for f in required), f"Empty required file: {name}")
    raw = path.read_bytes()
    return {"name": name, "version": version, "filename": path.name,
            "sha256": hashlib.sha256(raw).hexdigest(),
            "integrity": "sha512-" + base64.b64encode(hashlib.sha512(raw).digest()).decode(),
            "size": len(raw), "manifest": manifest,
            "files": {key: hashlib.sha256(value).hexdigest() for key, value in sorted(payload.items())}}


def candidate(directory: Path, commit: str, ref: str, run_id: str, root: Path = ROOT) -> dict:
    version = metadata(root)
    require(bool(re.fullmatch(r"[0-9a-f]{40}", commit)), "Candidate needs a full Git commit")
    expected = {f"{name}-{version}.tgz" for name in NAMES}
    require({p.name for p in directory.glob("*.tgz")} == expected, "Expected exactly five named tarballs")
    packages = [inspect_tarball(directory / f"{name}-{version}.tgz", name, version) for name in NAMES]
    for filename in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
        digest = hashlib.sha256((root / filename).read_bytes()).hexdigest()
        require(all(p["files"][filename] == digest for p in packages), f"Packed {filename} differs from source")
    record = {"schema": 1, "version": version, "commit": commit, "ref": ref,
              "run_id": run_id, "repository": REPOSITORY, "packages": packages}
    destination = directory / "release.json"
    require(not destination.exists() or read_json(destination) == record, "Never replace a frozen candidate")
    destination.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return record


def verify_local(directory: Path) -> dict:
    record = read_json(directory / "release.json")
    require(record.get("schema") == 1 and record.get("repository") == REPOSITORY, "Invalid release record")
    stable_version(record["version"])
    require([p["name"] for p in record["packages"]] == NAMES, "Release must contain four runtimes then main")
    expected = {f"{name}-{record['version']}.tgz" for name in NAMES}
    require({p.name for p in directory.glob("*.tgz")} == expected, "Tarball set differs from candidate")
    for item in record["packages"]:
        require(item["filename"] == f"{item['name']}-{record['version']}.tgz", "Invalid candidate filename")
        actual = inspect_tarball(directory / item["filename"], item["name"], record["version"])
        require(actual == item, f"Candidate changed: {item['filename']}")
    return record


def release_context(record: dict) -> None:
    for key, expected in {
        "GITHUB_REPOSITORY": REPOSITORY, "GITHUB_EVENT_NAME": "push", "GITHUB_REF_TYPE": "tag",
        "GITHUB_REF_NAME": f"npm-v{record['version']}", "GITHUB_REF_PROTECTED": "true",
        "GITHUB_SHA": record["commit"], "GITHUB_RUN_ID": record["run_id"],
    }.items():
        require(bool(expected) and os.environ.get(key) == expected, f"Publication context mismatch: {key}")
    require(record["ref"] == os.environ["GITHUB_REF_NAME"], "Candidate is from a different tag")


def fetch(url: str) -> bytes | None:
    require(url.startswith(REGISTRY), "Only the public npm registry is permitted")
    try:
        with urlopen(Request(url, headers={"User-Agent": "ankiforge-release-ci"}), timeout=45) as response:
            require(response.url.startswith(REGISTRY), "Registry redirected outside npm")
            return response.read()
    except HTTPError as error:
        if error.code == 404:
            return None
        raise


def registry_matches(item: dict) -> bool:
    raw = fetch(f"{REGISTRY}{item['name']}/{item['version']}")
    if raw is None:
        return False
    published = json.loads(raw)
    audit_manifest(published, item["name"], item["version"])
    require(published["dist"].get("integrity") == item["integrity"],
            f"Existing registry version has different integrity: {item['name']}")
    tarball = fetch(published["dist"]["tarball"])
    if tarball is None:
        return False
    require(hashlib.sha256(tarball).hexdigest() == item["sha256"],
            f"Registry bytes differ: {item['name']}")
    return True


def wait_for_registry(packages: list[dict], seconds: int) -> None:
    deadline = time.monotonic() + seconds
    while True:
        try:
            missing = [p["name"] for p in packages if not registry_matches(p)]
            if not missing:
                return
            problem = "Not public yet: " + ", ".join(missing)
        except (URLError, TimeoutError) as error:
            if isinstance(error, HTTPError) and error.code not in (429, 500, 502, 503, 504):
                raise
            problem = str(error)
        if time.monotonic() >= deadline:
            raise TimeoutError(problem)
        print(f"{problem}; retrying in 15 seconds", flush=True)
        time.sleep(min(15, max(0, deadline - time.monotonic())))


def npm(*args: str) -> None:
    subprocess.run(["npm", *args, "--registry", REGISTRY], check=True)


def publish(directory: Path, record: dict, seconds: int) -> None:
    release_context(record)
    # Check every collision before making any changes. A retry only skips identical bytes.
    existing = {p["name"]: registry_matches(p) for p in record["packages"]}
    for item in record["packages"]:
        if item["name"] == "ankiforge":
            wait_for_registry(record["packages"][:-1], seconds)
        if existing[item["name"]]:
            print(f"Already published with identical bytes: {item['name']}", flush=True)
        else:
            npm("publish", str((directory / item["filename"]).resolve()), "--ignore-scripts", "--access", "public", "--tag", "next")
    wait_for_registry(record["packages"], seconds)


def latest_versions(packages: list[dict]) -> dict[str, str | None]:
    result = {}
    for item in packages:
        raw = fetch(f"{REGISTRY}-/package/{item['name']}/dist-tags")
        require(raw is not None, f"Package is missing: {item['name']}")
        latest = json.loads(raw).get("latest")
        # Refuse to roll a newer stable release back after retrying an old run.
        # A staged bootstrap may have latest=0.0.0-stage. For a stable candidate,
        # an equal-core prerelease is always older; build metadata does not order.
        latest_core = latest.split("-", 1)[0].split("+", 1)[0] if latest else None
        require(latest is None or stable_version(latest_core) <= stable_version(item["version"]),
                f"Refusing to downgrade latest for {item['name']}: {latest}")
        result[item["name"]] = latest
    return result


def promote(record: dict, seconds: int) -> None:
    release_context(record)
    wait_for_registry(record["packages"], seconds)
    latest = latest_versions(record["packages"])
    for item in record["packages"]:
        if latest[item["name"]] != item["version"]:
            npm("dist-tag", "add", f"{item['name']}@{item['version']}", "latest")
    # npm can acknowledge a mutation before the read endpoint reflects it.
    deadline = time.monotonic() + seconds
    while True:
        if all(v == record["version"] for v in latest_versions(record["packages"]).values()):
            return
        require(time.monotonic() < deadline, "latest tags did not become visible")
        time.sleep(15)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("metadata").add_argument("--tag")
    freeze = commands.add_parser("candidate")
    freeze.add_argument("--directory", type=Path, required=True)
    freeze.add_argument("--commit", default=os.environ.get("GITHUB_SHA", ""))
    freeze.add_argument("--ref", default=os.environ.get("GITHUB_REF_NAME", "local"))
    freeze.add_argument("--run-id", default=os.environ.get("GITHUB_RUN_ID", ""))
    for action in ("verify", "verify-published", "publish", "promote"):
        command = commands.add_parser(action)
        command.add_argument("--directory", type=Path, required=True)
        command.add_argument("--wait-seconds", type=int, default=900)
    args = parser.parse_args()
    if args.command == "metadata":
        print(f"version={metadata(tag=args.tag)}")
    elif args.command == "candidate":
        record = candidate(args.directory, args.commit, args.ref, args.run_id)
        print(f"Frozen ankiforge@{record['version']}: five tarballs, commit {record['commit']}")
        for item in record["packages"]:
            print(f"- {item['filename']}: SHA-256 {item['sha256']}")
    else:
        record = verify_local(args.directory)
        if args.command == "verify-published":
            wait_for_registry(record["packages"], args.wait_seconds)
        elif args.command == "publish":
            publish(args.directory, record, args.wait_seconds)
        elif args.command == "promote":
            promote(record, args.wait_seconds)
        print(f"{args.command}: ankiforge@{record['version']} passed")


if __name__ == "__main__":
    main()
