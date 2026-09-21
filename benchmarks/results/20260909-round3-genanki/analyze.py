"""Validate the complete recorded run and derive statistics without rerunning it."""
from collections import Counter
import csv
import json
from pathlib import Path
import sys

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
RUN = REPO / "benchmarks/.work/runs/20260909-round3-genanki"
sys.path.insert(0, str(REPO / "benchmarks"))
import bench
from media_report import validate_power
from report import stats

PROFILES = [
    ("basic-mixed-text-v1", "纯文本", "Text only"),
    ("basic-image-unique-v2", "独立图片", "Unique images"),
    ("basic-audio-unique-v2", "独立音频", "Unique audio"),
    ("basic-mixed-unique-v2", "混合独立媒体", "Mixed, unique media"),
    ("basic-mixed-shared-v2", "混合共享媒体", "Mixed, shared media"),
]
SIZES = (100, 200, 500, 1000)
ROLES = {"timing_warmup": 3, "timing": 10, "rss_warmup": 3, "rss": 5}

def read(path):
    return json.loads(path.read_text())

def read_rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]

def main():
    manifest = read(RUN / "manifest.json")
    plan = read(WORK / "plan.json")
    completed = read(WORK / "completed.json")
    before = read(WORK / "source-snapshot.json")
    assert manifest["status"] == completed["status"] == "completed"
    assert manifest["identity_unchanged"] and completed["identity_unchanged"]
    assert before == manifest["identity_before"] == read(WORK / "identity-after.json")
    assert plan["source_commit"] == manifest["source_commit"]
    assert plan["git_status"] == manifest["git_status"]
    for name, key in (("run.py", "runner_sha256"), ("source-snapshot.json", "source_snapshot_sha256"),
                      ("source.patch", "source_patch_sha256"), ("source-and-inputs.tar.gz", "source_and_inputs_sha256"),
                      ("media-inputs.json", "media_inputs_sha256")):
        assert bench.sha256(WORK / name) == plan[key], name
    assert manifest["adapter_metadata"]["genanki"]["python"] == "3.11.0"
    assert manifest["adapter_metadata"]["genanki"]["genanki"] == "0.13.1"
    assert manifest["rust_configuration"] == {"allocator": "system", "allocator_version": None, "adapter_features": []}
    attempts = read_rows(RUN / "attempts.jsonl")
    verified = read_rows(RUN / "verified.jsonl")
    assert len(attempts) == len(verified) == plan["expected_attempts"] == 840
    assert len({r["id"] for r in attempts}) == 840
    assert [r["id"] for r in attempts] == [r["id"] for r in verified]
    expected = Counter({(p, n, a, role): count for p, _, _ in PROFILES for n in SIZES
                        for a in ("rust", "genanki") for role, count in ROLES.items()})
    assert Counter((r["profile"], r["size"], r["adapter"], r["role"]) for r in attempts) == expected
    first_counts = Counter()
    for i in range(0, len(attempts), 2):
        first, second = attempts[i:i + 2]
        assert all(first[k] == second[k] for k in ("profile", "size", "role", "repeat"))
        assert {first["adapter"], second["adapter"]} == {"rust", "genanki"}
        if first["role"] == "timing":
            first_counts[(first["profile"], first["size"], first["adapter"])] += 1
    assert all(first_counts[(p, n, a)] == 5 for p, _, _ in PROFILES for n in SIZES for a in ("rust", "genanki"))
    validate_power(read_rows(RUN / "power.jsonl"), attempts, "Now drawing from 'AC Power'")
    audits, selected_oracles, digests = [], [], {}
    previous_end = 0
    for attempt, row in zip(attempts, verified):
        assert row["status"] == attempt["status"] == "success"
        assert all(row[key] == value for key, value in attempt.items())
        measurement = row["measurement"]
        assert measurement["reaped"] and not any(measurement[key] for key in (
            "spawn_error", "exit_code", "signal", "interrupted_signal", "leftover_descendants"))
        assert measurement["elapsed_ns"] > 0 and measurement["peak_rss_bytes"] > 0
        assert measurement["peak_rss_raw_unit"] == "bytes"
        assert measurement["peak_rss_bytes"] == measurement["peak_rss_raw"]
        assert measurement["start_monotonic_ns"] >= previous_end
        previous_end = measurement["end_monotonic_ns"]
        folder = RUN / row["id"]
        assert read(folder / "attempt.json") == attempt
        check = read(folder / "verification.json")
        assert check["status"] == check["physical"]["status"] == check["semantic"]["status"] == "passed"
        assert check["artifact_sha256"] == row["artifact_sha256"]
        assert check["artifact_bytes"] == row["artifact_bytes"]
        assert check["physical"]["notes"] == check["physical"]["cards"] == row["size"]
        digest_key = (row["profile"], row["size"])
        digests.setdefault(digest_key, check["physical"]["logical_sha256"])
        assert digests[digest_key] == check["physical"]["logical_sha256"]
        audit = {"id": row["id"], "artifact_sha256": row["artifact_sha256"],
                 "verification_sha256": bench.sha256(folder / "verification.json"), "status": "passed"}
        if row["role"] == "timing" and row["repeat"] == 0:
            oracle = read(folder / "anki.json")
            assert oracle["status"] == "passed"
            assert oracle["notes"] == oracle["cards"] == row["size"]
            assert oracle["all_fields_checked"] and oracle["conflicting"] == 0
            assert bench.sha256(folder / "anki.json") == row["oracle_sha256"]
            audit["oracle_sha256"] = row["oracle_sha256"]
            selected_oracles.append(audit)
        else:
            assert "oracle_sha256" not in row
        audits.append(audit)
    assert len(selected_oracles) == plan["expected_anki_imports"] == 40
    resources = {(w["profile"], w["notes"]): w for w in manifest["workloads"]}
    cells = []
    for profile, label, english in PROFILES:
        for size in SIZES:
            cell = {"profile": profile, "label": label, "english_label": english, "notes": size,
                    "resources": resources[(profile, size)], "status": "verified_exploratory"}
            for adapter in ("rust", "genanki"):
                rows = [r for r in verified if (r["profile"], r["size"], r["adapter"]) == (profile, size, adapter)]
                cell[adapter] = {
                    "time_ms": stats([r["measurement"]["elapsed_ns"] / 1e6 for r in rows if r["role"] == "timing"]),
                    "rss_mib": stats([r["measurement"]["peak_rss_bytes"] / 2**20 for r in rows if r["role"] == "rss"]),
                    "apkg_bytes": stats([r["artifact_bytes"] for r in rows if r["role"] == "timing"]),
                }
            rust, genanki = cell["rust"], cell["genanki"]
            cell["time_saved_pct"] = 100 * (1 - rust["time_ms"]["median"] / genanki["time_ms"]["median"])
            cell["time_saved_ms"] = genanki["time_ms"]["median"] - rust["time_ms"]["median"]
            cell["genanki_over_rust"] = genanki["time_ms"]["median"] / rust["time_ms"]["median"]
            for metric in ("rss_mib", "apkg_bytes"):
                cell[metric + "_saved_pct"] = 100 * (1 - rust[metric]["median"] / genanki[metric]["median"])
            cell["time_target_passed"] = cell["time_saved_pct"] >= 30
            cells.append(cell)
    image_1000 = next(c for c in cells if c["profile"] == "basic-image-unique-v2" and c["notes"] == 1000)
    result = {"schema": "round3-genanki-comparison-v1", "run_id": plan["name"],
              "source_commit": plan["source_commit"], "exploratory_dirty_source": True,
              "started_utc": manifest["created_utc"], "completed_utc": manifest["completed_utc"],
              "host": read(WORK / "host-hardware.json"), "adapter_metadata": manifest["adapter_metadata"],
              "quantiles": plan["report_quantiles"], "cells": cells,
              "time_target_passed_cells": sum(c["time_target_passed"] for c in cells),
              "image_1000_max_rss_mib": image_1000["rust"]["rss_mib"]["max"],
              "image_1000_rss_target_passed": image_1000["rust"]["rss_mib"]["max"] <= 64}
    bench.save(WORK / "summary.json", result)
    bench.save(WORK / "verification-summary.json", {
        "status": "passed", "attempts": 840, "timing_samples": 400, "rss_samples": 200,
        "warmup_samples": 240, "raw_semantic_media_checks": 840, "anki_checks": 40,
        "power_records": 840, "all_power_records_ac": True, "identity_unchanged": True,
        "adjacent_pairs": 420, "timing_first_order": "5 Rust-first / 5 genanki-first for each cell",
        "media_files_unchanged": completed["media_files_unchanged"],
        "clock_resolution_ns": sorted({r["measurement"]["clock_resolution_ns"] for r in attempts}),
        "checks": audits,
    })
    flat = []
    for c in cells:
        row = {"profile": c["profile"], "notes": c["notes"], "media_files": c["resources"]["media_files"],
               "media_bytes": c["resources"]["media_bytes"], "time_saved_pct": c["time_saved_pct"],
               "time_saved_ms": c["time_saved_ms"], "genanki_over_rust": c["genanki_over_rust"]}
        for adapter in ("rust", "genanki"):
            for metric in ("time_ms", "rss_mib", "apkg_bytes"):
                row.update({f"{adapter}_{metric}_{key}": value for key, value in c[adapter][metric].items()})
        flat.append(row)
    with (WORK / "comparison.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(flat[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(flat)
    print(json.dumps({"status": "verified", "attempts": 840, "anki_checks": 40,
                      "target_passed_cells": result["time_target_passed_cells"],
                      "image_1000_max_rss_mib": result["image_1000_max_rss_mib"]}, indent=2))

if __name__ == "__main__":
    main()
