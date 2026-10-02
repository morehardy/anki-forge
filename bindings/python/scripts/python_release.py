"""Validate, preserve and verify one immutable PyPI release candidate.

Uses only the standard library so the upload job never needs build tools or
third-party Python dependencies. Distributions are built in separate jobs.
"""
from __future__ import annotations

import argparse
import ast
from email import message_from_bytes
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
from typing import Any
from urllib.error import HTTPError
from urllib.request import urlopen
import zipfile


PROJECT = "ankiforge"
REPOSITORY = Path(__file__).resolve().parents[3]
VERSION_RE = re.compile(r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)")
TARGETS = {"linux-x64", "windows-x64", "macos-x64", "macos-arm64"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def normalized_name(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def source_metadata(root: Path, tag: str | None = None) -> dict[str, str]:
    configuration = tomllib.loads((root / "bindings/python/pyproject.toml").read_text())
    project = configuration["project"]
    require(configuration["tool"]["maturin"]["module-name"] == f"{PROJECT}._native" and
            configuration["tool"]["maturin"]["python-packages"] == [PROJECT],
            "Python distribution and import package names must both be ankiforge")
    native = tomllib.loads((root / "bindings/python/native/Cargo.toml").read_text())
    core = tomllib.loads((root / "anki_forge/Cargo.toml").read_text())["package"]
    loader: dict[str, str] = {}
    for node in ast.parse((root / "bindings/python/src" / PROJECT / "_loader.py").read_text()).body:
        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Constant):
            for target in node.targets:
                if isinstance(target, ast.Name) and isinstance(node.value.value, str):
                    loader[target.id] = node.value.value
    version = project["version"]
    require(project["name"] == PROJECT, f"expected distribution name {PROJECT}")
    require(VERSION_RE.fullmatch(version) is not None, "release version must be stable X.Y.Z")
    require(native["package"]["version"] == version == loader.get("__version__"),
            "pyproject, native crate and Python loader binding versions disagree")
    require(native["dependencies"]["ankiforge"]["version"] == core["version"] == loader.get("_CORE_API_VERSION"),
            "native dependency, core crate and Python loader core versions disagree")
    bundle = list((root / "anki_forge/assets/contracts").glob("anki-forge-contract-bundle-*.tar.gz"))
    require(len(bundle) == 1, "expected exactly one embedded contract bundle")
    contract = bundle[0].name.removeprefix("anki-forge-contract-bundle-").removesuffix(".tar.gz")
    require(loader.get("_CONTRACT_VERSION") == contract, "Python loader contract version disagrees with bundle")
    if tag is not None:
        require(tag == f"python-v{version}", f"expected tag python-v{version}, got {tag}")
    return {"version": version, "core_version": core["version"], "contract_version": contract,
            "requires_python": project["requires-python"]}


def wheel_target(platforms: str) -> str:
    tags = set(platforms.split("."))
    if tags and tags <= {"manylinux_2_17_x86_64", "manylinux2014_x86_64"}:
        return "linux-x64"
    if tags == {"win_amd64"}:
        return "windows-x64"
    for architecture, target in (("x86_64", "macos-x64"), ("arm64", "macos-arm64")):
        if len(tags) == 1 and re.fullmatch(rf"macosx_\d+_\d+_{architecture}", platforms):
            return target
    raise ValueError(f"unsupported wheel platform tags: {platforms}")


def check_metadata(data: bytes, version: str, requires_python: str) -> None:
    metadata = message_from_bytes(data)
    require(normalized_name(metadata.get("Name", "")) == PROJECT, "distribution metadata has wrong project name")
    require(metadata.get("Version") == version, "distribution metadata has wrong version")
    require(metadata.get("Requires-Python") == requires_python, "distribution Requires-Python disagrees with source")
    require(metadata.get("License-Expression") == "MIT", "distribution metadata must declare MIT")


def distribution_info(path: Path, version: str, requires_python: str) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"distribution must be a regular file: {path.name}")
    if path.name.endswith(".whl"):
        parts = path.name.removesuffix(".whl").split("-", 2)
        require(len(parts) == 3 and parts[:2] == [PROJECT, version], "unexpected wheel name or version")
        python_tag, abi, platforms = parts[2].split("-")
        require(python_tag == "cp311" and abi == "abi3", "expected a cp311-abi3 wheel")
        target = wheel_target(platforms)
        dist_info = f"{PROJECT}-{version}.dist-info"
        with zipfile.ZipFile(path) as archive:
            check_metadata(archive.read(f"{dist_info}/METADATA"), version, requires_python)
            wheel = message_from_bytes(archive.read(f"{dist_info}/WHEEL"))
            require(set(wheel.get_all("Tag", [])) == {f"cp311-abi3-{tag}" for tag in platforms.split(".")},
                    "WHEEL tags disagree with filename")
    else:
        require(path.name == f"{PROJECT}-{version}.tar.gz", "unexpected source distribution name or version")
        target = "source"
        with tarfile.open(path) as archive:
            member = archive.getmember(f"{PROJECT}-{version}/PKG-INFO")
            require(member.isfile(), "sdist metadata must be a regular file")
            content = archive.extractfile(member)
            require(content is not None, "sdist metadata is missing")
            with content:
                check_metadata(content.read(), version, requires_python)
    with path.open("rb") as content:
        checksum = hashlib.file_digest(content, "sha256").hexdigest()
    return {"filename": path.name, "target": target, "size": path.stat().st_size, "sha256": checksum}


def create_candidate(dist: Path, evidence: Path, metadata: dict[str, str], *, commit: str,
                     ref: str, run_id: str, run_url: str) -> dict[str, Any]:
    require(re.fullmatch(r"[0-9a-f]{40}", commit) is not None, "candidate must record a full commit SHA")
    files = sorted(dist.iterdir())
    require(len(files) == 5, "candidate must contain exactly one sdist and four wheels")
    entries = [distribution_info(path, metadata["version"], metadata["requires_python"]) for path in files]
    require({entry["target"] for entry in entries} == TARGETS | {"source"}, "missing or duplicated platform wheel")
    record = {"schema": 1, "project": PROJECT, **metadata, "commit": commit,
              "ref": ref, "run_id": run_id, "run_url": run_url, "files": entries}
    evidence.mkdir(parents=True, exist_ok=True)
    (evidence / "release.json").write_text(json.dumps(record, indent=2) + "\n")
    (evidence / "SHA256SUMS").write_text("".join(f"{entry['sha256']}  {entry['filename']}\n" for entry in entries))
    return record


def read_record(path: Path) -> dict[str, Any]:
    record = json.loads(path.read_text())
    require(record["schema"] == 1 and record["project"] == PROJECT, "unrecognized release record")
    require(VERSION_RE.fullmatch(record["version"]) is not None, "invalid recorded version")
    entries = record["files"]
    require(len(entries) == 5 and {entry["target"] for entry in entries} == TARGETS | {"source"},
            "release record must list one source package and four target wheels")
    require(len({entry["filename"] for entry in entries}) == 5, "duplicate recorded filename")
    for entry in entries:
        require(Path(entry["filename"]).name == entry["filename"] and "\\" not in entry["filename"],
                "unsafe recorded filename")
        require(re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]) is not None, "invalid recorded checksum")
    return record


def verify_local(record: dict[str, Any], dist: Path) -> None:
    expected = {entry["filename"]: entry for entry in record["files"]}
    require({path.name for path in dist.iterdir()} == set(expected), "local candidate file list changed")
    for name, entry in expected.items():
        actual = distribution_info(dist / name, record["version"], record["requires_python"])
        require(actual == entry, f"local candidate changed: {name}")


def registry_release(version: str) -> dict[str, Any] | None:
    require(VERSION_RE.fullmatch(version) is not None, "invalid registry version")
    try:
        with urlopen(f"https://pypi.org/pypi/{PROJECT}/{version}/json", timeout=30) as response:
            return json.load(response)
    except HTTPError as error:
        if error.code == 404:
            return None
        raise


def missing_files(record: dict[str, Any], published: dict[str, Any] | None) -> list[str]:
    expected = {entry["filename"]: entry for entry in record["files"]}
    if published is None:
        return sorted(expected)
    require(normalized_name(published["info"]["name"]) == PROJECT and
            published["info"]["version"] == record["version"], "registry release identity disagrees")
    seen: set[str] = set()
    for actual in published["urls"]:
        name = actual["filename"]
        require(name in expected and name not in seen, f"unexpected registry file: {name}")
        entry = expected[name]
        require(actual["digests"]["sha256"] == entry["sha256"] and actual["size"] == entry["size"],
                f"registry file differs from original candidate: {name}")
        require(not actual.get("yanked", False), f"registry file has been yanked: {name}")
        seen.add(name)
    return sorted(set(expected) - seen)


def prepare_upload(record: dict[str, Any], dist: Path, output: Path,
                   published: dict[str, Any] | None, *, attempt: int, run_id: str) -> list[str]:
    require(attempt >= 1, "invalid workflow attempt")
    require(record["ref"] == f"python-v{record['version']}" and record["run_id"] == run_id,
            "upload must use the original tag run's candidate")
    require(published is None or attempt > 1, "version already exists; only a retry of the original run may recover it")
    verify_local(record, dist)
    missing = missing_files(record, published)
    require(not output.exists(), "upload staging directory must be new")
    output.mkdir(parents=True)
    for name in missing:
        shutil.copyfile(dist / name, output / name)
        original = next(entry for entry in record["files"] if entry["filename"] == name)
        require(distribution_info(output / name, record["version"], record["requires_python"]) == original,
                f"staged upload differs from original candidate: {name}")
    return missing


def verify_published(record: dict[str, Any], attempts: int, delay: float) -> None:
    for attempt in range(attempts):
        missing = missing_files(record, registry_release(record["version"]))
        if not missing:
            print("All five PyPI files match the original candidate")
            return
        if attempt + 1 < attempts:
            print(f"Waiting for PyPI files: {', '.join(missing)}", flush=True)
            time.sleep(delay)
    raise ValueError(f"PyPI release remains incomplete: {', '.join(missing)}")


def install_published(record: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="ankiforge-pypi-") as directory:
        root = Path(directory)
        command = [sys.executable, "-m", "pip", "--isolated", "download", "--index-url", "https://pypi.org/simple",
                   "--only-binary=:all:", "--no-deps", "--no-cache-dir", "--dest", str(root),
                   f"{PROJECT}=={record['version']}"]
        for attempt in range(6):
            result = subprocess.run(command)
            if result.returncode == 0:
                break
            if attempt == 5:
                raise RuntimeError("published wheel is not installable from public PyPI")
            time.sleep(10)
        wheel, = root.glob("*.whl")
        actual = distribution_info(wheel, record["version"], record["requires_python"])
        require(actual in record["files"], "downloaded PyPI wheel does not match original candidate")
        subprocess.run([sys.executable, str(Path(__file__).with_name("check_native_wheel.py")), str(wheel),
                        "--expected-version", record["version"]], check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    metadata_parser = commands.add_parser("metadata")
    metadata_parser.add_argument("--tag")
    candidate = commands.add_parser("candidate")
    candidate.add_argument("--dist", type=Path, required=True)
    candidate.add_argument("--evidence", type=Path, required=True)
    candidate.add_argument("--commit", required=True)
    candidate.add_argument("--ref", required=True)
    candidate.add_argument("--run-id", required=True)
    candidate.add_argument("--run-url", required=True)
    unpublished = commands.add_parser("unpublished")
    unpublished.add_argument("--version", required=True)
    for name in ("prepare-upload", "verify-published", "install-published"):
        command = commands.add_parser(name)
        command.add_argument("--record", type=Path, required=True)
        if name == "prepare-upload":
            command.add_argument("--dist", type=Path, required=True)
            command.add_argument("--output", type=Path, required=True)
            command.add_argument("--attempt", type=int, required=True)
            command.add_argument("--run-id", required=True)
        elif name == "verify-published":
            command.add_argument("--attempts", type=int, default=20)
            command.add_argument("--delay", type=float, default=15)
    args = parser.parse_args()
    if args.command == "metadata":
        for key, value in source_metadata(REPOSITORY, args.tag).items():
            print(f"{key}={value}")
    elif args.command == "candidate":
        record = create_candidate(args.dist, args.evidence, source_metadata(REPOSITORY), commit=args.commit,
                                  ref=args.ref, run_id=args.run_id, run_url=args.run_url)
        print(f"Candidate {PROJECT} {record['version']} at {record['commit']}\n")
        print("| File | SHA-256 |\n| --- | --- |")
        for entry in record["files"]:
            print(f"| {entry['filename']} | {entry['sha256']} |")
    elif args.command == "unpublished":
        require(registry_release(args.version) is None, "PyPI version already exists; do not rebuild or replace it")
    else:
        record = read_record(args.record)
        if args.command == "prepare-upload":
            missing = prepare_upload(record, args.dist, args.output, registry_release(record["version"]),
                                     attempt=args.attempt, run_id=args.run_id)
            print(f"upload_needed={'true' if missing else 'false'}")
        elif args.command == "verify-published":
            require(args.attempts > 0 and args.delay >= 0, "invalid retry policy")
            verify_published(record, args.attempts, args.delay)
        else:
            install_published(record)


if __name__ == "__main__":
    main()
