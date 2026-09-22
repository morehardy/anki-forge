"""Collect locked native dependencies' license/notice texts, including vendored code.

Includes build dependencies conservatively. Run after cargo fetch for all targets.
The result contains no absolute build-machine paths.
"""
from pathlib import Path
import json
import subprocess


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--manifest-path", str(root / "bindings/python/native/Cargo.toml"),
        "--format-version", "1", "--locked", "--offline",
    ]))
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    pending = [p["id"] for p in packages.values() if p["name"] == "anki_forge_python_native"]
    included = set()
    while pending:
        current = pending.pop()
        if current in included:
            continue
        included.add(current)
        pending.extend(d["pkg"] for d in nodes[current]["deps"] if any(k["kind"] != "dev" for k in d["dep_kinds"]))
    output = ["# Third-party notices\n\nGenerated from Cargo.lock with scripts/generate_notices.py. Includes normal and build dependencies across targets, and bundled native-library notices where present.\n"]
    missing = []
    for package in sorted((packages[key] for key in included if packages[key]["source"]), key=lambda p: (p["name"], p["version"])):
        directory = Path(package["manifest_path"]).parent
        output.append(f"\n## {package['name']} {package['version']}\n\nLicense expression: {package['license'] or 'see files below'}\n\nSource: https://crates.io/crates/{package['name']}/{package['version']}\n")
        files = [p for p in directory.rglob("*") if p.is_file() and p.name.lower().startswith(("license", "licence", "copying", "notice", "copyright"))]
        if package.get("license_file"):
            files.append(directory / package["license_file"])
        if not files and (directory / "AUTHORS").is_file():
            files.append(directory / "AUTHORS")
        seen = set()
        for path in sorted(set(files)):
            try:
                content = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            if content in seen:
                continue
            seen.add(content)
            output.append(f"\n### {path.relative_to(directory).as_posix()}\n\n```text\n{content.rstrip()}\n```\n")
        if not seen:
            fallback = root / "bindings/python/licenses" / f"{package['name']}.txt"
            if fallback.is_file():
                output.append("\n```text\n" + fallback.read_text(encoding="utf-8").rstrip() + "\n```\n")
            else:
                missing.append(package["name"])
    destination = root / "bindings/python/THIRD_PARTY_NOTICES.md"
    destination.write_text("".join(output), encoding="utf-8")
    print(f"Wrote {destination.name}: {destination.stat().st_size} bytes; missing local license texts: {missing}")
    if missing:
        raise SystemExit("resolve missing notices before distributing")


if __name__ == "__main__":
    main()
