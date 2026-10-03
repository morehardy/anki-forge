"""Audit retained round-four diagnostic evidence without running any exporter.

Only the Python standard library is required. Pass --work-dir to a directory
containing the extracted plans, results, case directories, fixtures and frozen
*-source trees. Executables are optional in an archive; their recorded hashes
are still reconciled with build records. Deleted APKGs and Anki collections are
not regenerated: this checks the retained validation evidence, not a new import.
The script writes nothing and emits its audit result on stdout.
"""

import argparse
from collections import Counter
from datetime import datetime
import hashlib
import json
from pathlib import Path
import random
import re
import statistics


def require(condition, context):
    if not condition:
        raise ValueError(context)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def decode(text):
    return json.loads(text, object_pairs_hook=unique_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(
                          ValueError(f"non-finite JSON number: {value}")))


def read(path):
    return decode(path.read_text(encoding="utf-8"))


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def is_digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def recorded_hash(identity, suffix):
    matches = [value for path, value in identity.items() if path.endswith(suffix)]
    require(len(matches) == 1, f"identity must contain exactly one {suffix}")
    require(is_digest(matches[0]), f"invalid recorded SHA-256: {suffix}")
    return matches[0]


def order_for(plan):
    rng = random.Random(plan["seed"])
    names = list(plan["modes"])
    order = []
    for role, count in [("warmup", plan["warmups"]), ("timing", plan["repeats"])]:
        for repeat in range(count):
            profiles = plan["profiles"].copy()
            rng.shuffle(profiles)
            mode_order = names[repeat % len(names):] + names[:repeat % len(names)]
            for profile in profiles:
                order.extend((role, repeat, profile, mode) for mode in mode_order)
    return order


def expected_renders(fixture):
    notes = fixture["notes"]
    selected = {next(note["id"] for note in notes if note["category"] == category)
                for category in ["english", "mixed", "escaping"]}
    media = {item["id"]: item for item in fixture.get("media", [])}
    for kind in ["image", "audio"]:
        note = next((note for note in notes if any(
            media[key]["kind"] == kind for key in
            note.get("front_media", []) + note.get("back_media", []))), None)
        if note:
            selected.add(note["id"])
    return [{"id": note["id"], "category": note["category"],
             "question": note["front"], "answer_front": note["front"],
             "answer_back": note["back"]}
            for note in sorted(notes, key=lambda note: note["id"]) if note["id"] in selected]


def fixture_facts(work, profile):
    path = work / "fixtures" / profile / "inputs" / "1000.json"
    fixture = read(path)
    require(fixture["profile"] == profile, f"wrong fixture profile: {path}")
    notes = fixture["notes"]
    require(fixture["note_count"] == len(notes) == 1000, f"wrong fixture size: {profile}")
    fields = {note["front"]: note["back"] for note in notes}
    require(len(fields) == len(notes), f"duplicate fixture front: {profile}")
    serialized = json.dumps(sorted(fields.items()), ensure_ascii=False,
                            separators=(",", ":")) + "\n"
    return {"fixture": fixture, "sha256": sha(path),
            "logical_sha256": hashlib.sha256(serialized.encode()).hexdigest(),
            "renders": expected_renders(fixture)}


def check_verification(verified, row, facts, context):
    fixture = facts["fixture"]
    require(verified["status"] == "passed", f"verification failed: {context}")
    for name in ["artifact_sha256", "artifact_bytes"]:
        require(row[name] == verified[name], f"verification {name} mismatch: {context}")
    require(is_digest(verified["artifact_sha256"]) and verified["artifact_bytes"] > 0,
            f"invalid artifact identity: {context}")
    expected_physical = {"status": "passed", "notes": fixture["note_count"],
                         "cards": fixture["note_count"], "schema": 18,
                         "used_note_types": 1, "populated_decks": 1,
                         "logical_sha256": facts["logical_sha256"]}
    require(verified["physical"] == expected_physical, f"physical facts mismatch: {context}")
    require(row["logical_sha256"] == facts["logical_sha256"],
            f"learning content mismatch: {context}")
    semantic = verified["semantic"]
    require(semantic["status"] == "passed"
            and semantic["reader"] == "repository-inspector-modern-v1"
            and semantic["observation_model_version"] == "phase3-inspect-v2"
            and semantic["stock_model_name"] == "Basic" and semantic["css"] == ""
            and is_digest(semantic["inspection_sha256"]), f"semantic facts mismatch: {context}")
    package = verified["package"]
    count = len(fixture.get("media", []))
    require(package["version"] == 3 and package["canonical_entry"] == "collection.anki21b"
            and package["nested_compression"] == "zstd" and package["media_count"] == count,
            f"package facts mismatch: {context}")
    payloads = package["payloads"]
    names = [payload["name"] for payload in payloads]
    required = {"meta", "collection.anki21b", "collection.anki2", "media", "ankiforge-identity.json"}
    require(len(names) == len(set(names)) and set(names) == required | {str(n) for n in range(count)},
            f"package payload set mismatch: {context}")
    for payload in payloads:
        require(payload["zip_method"] == 0
                and payload["stored_bytes"] == payload["decoded_zip_bytes"] >= 0,
                f"invalid stored payload: {context}/{payload['name']}")
        role = ("canonical" if payload["name"] == "collection.anki21b" else
                "compatibility_placeholder" if payload["name"] == "collection.anki2" else
                "native_identity_metadata" if payload["name"] == "ankiforge-identity.json" else "metadata")
        require(payload["role"] == role, f"payload role mismatch: {context}")
    identity = package["native_identity"]
    identity_payload = next(item for item in payloads if item["name"] == "ankiforge-identity.json")
    require(identity["entry"] == "ankiforge-identity.json"
            and identity["format_version"] == "ankiforge-identity-v1"
            and is_digest(identity["sha256"])
            and identity["bytes"] == identity_payload["stored_bytes"],
            f"native identity record mismatch: {context}")


def check_case(work, stem, row, facts, traced):
    key = tuple(row[name] for name in ["role", "repeat", "profile", "mode"])
    case = work / (stem + "-" + "-".join(map(str, key)))
    measurement = read(case / "measurement.json")
    require(measurement["reaped"] is True and not any(measurement[name] for name in
            ["spawn_error", "exit_code", "signal", "interrupted_signal", "leftover_descendants"]),
            f"unsuccessful measured process: {case.name}")
    require(measurement["elapsed_ns"] == measurement["end_monotonic_ns"] - measurement["start_monotonic_ns"] > 0,
            f"clock arithmetic mismatch: {case.name}")
    require(measurement["clock_resolution_ns"] > 0 and measurement["pid"] > 0,
            f"invalid collector metadata: {case.name}")
    require(measurement["peak_rss_raw_unit"] == "bytes"
            and measurement["peak_rss_raw"] == measurement["peak_rss_bytes"] > 0,
            f"invalid RSS metadata: {case.name}")
    require(row["elapsed_ms"] == measurement["elapsed_ns"] / 1e6
            and row["rss_mib"] == measurement["peak_rss_bytes"] / 2**20,
            f"raw measurement mismatch: {case.name}")
    for stream in ["stdout", "stderr"]:
        require((case / f"{stream}.log").stat().st_size == measurement[f"{stream}_bytes"],
                f"{stream} byte count mismatch: {case.name}")
    check_verification(read(case / "verification.json"), row, facts, case.name)
    require(row["power_before"].splitlines()[0] == row["power_after"].splitlines()[0]
            == "Now drawing from 'AC Power'", f"not continuously AC: {case.name}")
    oracle = row["role"] == "timing" and row["repeat"] == 0
    require(row["oracle_passed"] is oracle and (case / "anki.json").exists() is oracle,
            f"Anki selection mismatch: {case.name}")
    expected_files = {"measurement.json", "verification.json", "stdout.log", "stderr.log"}
    if oracle:
        expected_files |= {"anki.json", "oracle.stdout.log", "oracle.stderr.log"}
        fixture = facts["fixture"]
        expected = {"status": "passed", "oracle": "upstream-anki-media-artifact-v2",
                    "notes": fixture["note_count"], "cards": fixture["note_count"],
                    "new_notes": fixture["note_count"], "conflicting": 0,
                    "used_models": 1, "populated_decks": 1, "all_fields_checked": True,
                    "media_files": len(fixture.get("media", [])), "renders": facts["renders"]}
        require(read(case / "anki.json") == expected, f"Anki fields/renders mismatch: {case.name}")
    require({path.name for path in case.iterdir()} == expected_files,
            f"missing or unexpected case files: {case.name}")
    prefix = "[DEBUG-import-round4] "
    lines = (case / "stderr.log").read_text(encoding="utf-8").splitlines()
    trace = [line[len(prefix):] for line in lines if line.startswith(prefix)]
    require(len(trace) == int(traced), f"trace count mismatch: {case.name}")
    stages = {}
    if traced:
        for name, value in decode(trace[0]).items():
            require(isinstance(value, list) and len(value) == 2
                    and all(type(item) is int and item >= 0 for item in value),
                    f"invalid stage counters: {case.name}/{name}")
            stages[name] = {"count": value[0], "ms": value[1] / 1e6}
    require(row["stages"] == stages, f"stage trace mismatch: {case.name}")
    return measurement, oracle


def recompute_summary(plan, rows):
    summary = []
    for profile in plan["profiles"]:
        cell = {"profile": profile}
        for mode in plan["modes"]:
            selected = [row for row in rows if row["role"] == "timing"
                        and row["profile"] == profile and row["mode"] == mode]
            keys = set().union(*(row["stages"] for row in selected))
            require(all(set(row["stages"]) == keys for row in selected),
                    f"stage set changes within a cell: {profile}/{mode}")
            cell[mode] = {"n": len(selected), "samples_ms": [row["elapsed_ms"] for row in selected],
                          "median_ms": statistics.median(row["elapsed_ms"] for row in selected),
                          "rss_mib": statistics.median(row["rss_mib"] for row in selected),
                          "artifact_bytes": statistics.median(row["artifact_bytes"] for row in selected),
                          "stages": {name: {field: statistics.median(row["stages"][name][field]
                                      for row in selected) for field in ["ms", "count"]} for name in keys}}
        summary.append(cell)
    return summary


def audit(work):
    groups, skipped, fixtures, used_binaries = {}, {}, {}, {}
    content = {}
    for plan_path in sorted(work.glob("*-plan.json")):
        stem = plan_path.name[:-len("-plan.json")]
        result_path = work / f"{stem}-results.json"
        if not result_path.exists():
            skipped[stem] = "results not yet present"
            continue
        results = read(result_path)
        if results.get("status") != "completed":
            skipped[stem] = results.get("status", "missing status")
            continue
        plan = read(plan_path)
        require(plan["group"] == stem, f"group mismatch: {stem}")
        require(all(type(plan[key]) is int and plan[key] > 0 for key in ["warmups", "repeats"]),
                f"invalid repeat counts: {stem}")
        require(len(set(plan["profiles"])) == len(plan["profiles"]) and plan["modes"],
                f"duplicate profiles or empty modes: {stem}")
        require(results["identity_unchanged"] is True, f"changed runtime identity: {stem}")
        require(datetime.fromisoformat(results["completed_utc"]) >= datetime.fromisoformat(plan["created_utc"]),
                f"invalid completion date: {stem}")
        config = read(work / f"{stem}-config.json")
        require(all(plan[key] == value for key, value in config.items()), f"config/plan mismatch: {stem}")
        for mode in plan["modes"].values():
            binary = mode["binary"]
            digest = recorded_hash(plan["identity_before"], f"/binaries/{binary}")
            require(binary not in used_binaries or used_binaries[binary] == digest,
                    f"binary changed between groups: {binary}")
            used_binaries[binary] = digest
        for profile in plan["profiles"]:
            facts = fixtures.setdefault(profile, fixture_facts(work, profile))
            require(facts["sha256"] == recorded_hash(plan["identity_before"], f"/fixtures/{profile}/inputs/1000.json"),
                    f"fixture bytes differ from plan: {stem}/{profile}")
            for media in facts["fixture"].get("media", []):
                require(media["sha256"] == recorded_hash(plan["identity_before"],
                        f"/fixtures/{profile}/inputs/{media['path']}"),
                        f"media manifest identity differs from plan: {stem}/{profile}/{media['id']}")
        rows = results["rows"]
        expected_order = order_for(plan)
        actual_order = [tuple(row[key] for key in ["role", "repeat", "profile", "mode"]) for row in rows]
        require(actual_order == expected_order, f"predeclared sample order differs: {stem}")
        require(len(rows) == plan["expected_exports"] == len(expected_order), f"export count mismatch: {stem}")
        expected_cases = {stem + "-" + "-".join(map(str, key)) for key in expected_order}
        actual_cases = {path.name for role in ["timing", "warmup"]
                        for path in work.glob(f"{stem}-{role}-*") if path.is_dir()}
        require(actual_cases == expected_cases, f"missing/extra case directories: {stem}")
        previous_end, oracle_count = 0, 0
        for row in rows:
            measurement, oracle = check_case(work, stem, row, fixtures[row["profile"]],
                                             stem in {"import", "budget", "hash"})
            require(measurement["start_monotonic_ns"] >= previous_end, f"overlapping/out-of-order samples: {stem}")
            previous_end = measurement["end_monotonic_ns"]
            oracle_count += oracle
            content.setdefault(row["profile"], set()).add(row["logical_sha256"])
        require(oracle_count == plan["expected_anki_checks"] == len(plan["profiles"]) * len(plan["modes"]),
                f"Anki count mismatch: {stem}")
        require(read(work / f"{stem}-summary.json") == recompute_summary(plan, rows),
                f"recomputed summary differs: {stem}")
        groups[stem] = {"status": "passed", "exports": len(rows), "anki_checks": oracle_count,
                        "role_counts": dict(Counter(row["role"] for row in rows))}
    require(groups, "no completed diagnostic groups found")
    controls = {}
    for binary in sorted(used_binaries):
        build_path = work / f"build-{binary}.json"
        if not build_path.exists():
            require(binary in {"before", "after"}, f"missing control build record: {binary}")
            continue
        build = read(build_path)
        source = work / f"{binary}-source"
        files = build["source_files"]
        require(build["binary_sha256"] == used_binaries[binary], f"build binary hash mismatch: {binary}")
        actual = {str(path.relative_to(source)) for path in source.rglob("*") if path.is_file()}
        require(actual == set(files), f"frozen source file set mismatch: {binary}")
        for relative, expected in files.items():
            require(not Path(relative).is_absolute() and ".." not in Path(relative).parts,
                    f"unsafe source manifest path: {relative}")
            require(is_digest(expected) and sha(source / relative) == expected,
                    f"frozen source hash mismatch: {binary}/{relative}")
        command = build["command"]
        require(all(flag in command for flag in ["--release", "--offline", "--locked", "--no-default-features"]),
                f"unexpected control build options: {binary}")
        controls[binary] = len(files)
    available, absent = [], []
    for binary, expected in used_binaries.items():
        path = work / "binaries" / binary
        if path.exists():
            require(sha(path) == expected, f"retained executable hash mismatch: {binary}")
            available.append(binary)
        else:
            absent.append(binary)
    require(all(len(values) == 1 for values in content.values()), "learning content changed between groups")
    return {"status": "passed", "exports": sum(group["exports"] for group in groups.values()),
            "anki_checks": sum(group["anki_checks"] for group in groups.values()),
            "learning_content_equal": True, "summaries_recomputed": True,
            "predeclared_order_verified": True, "raw_measurements_and_traces_verified": True,
            "probe_source_hashes_verified": controls,
            "retained_binary_hashes_verified": sorted(available), "binaries_not_archived": sorted(absent),
            "groups": groups, "incomplete_groups_not_audited": skipped,
            "scope": "Offline reconciliation of retained measurements and validation records; does not rerun deleted APKG or Anki imports."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, default=Path(__file__).resolve().parent)
    print(json.dumps(audit(parser.parse_args().work_dir.resolve()), indent=2))
