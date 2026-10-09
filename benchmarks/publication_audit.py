#!/usr/bin/env python3
"""Recompute saved publication measurements without exporting or importing.

Pass an extracted evidence directory. This validates the sample schedule,
recomputes all paired statistics and checks complete comparison observations.
It does not replace actual APKG inspection or the archived Anki oracle results.
"""
import argparse
from collections import Counter
import json
from pathlib import Path

import publication_performance as measurement
from publication_collector import validate


def audit(root):
    results = {}
    for completed in sorted(root.glob("*/complete.json")):
        directory = completed.parent
        samples = directory / "samples.jsonl"
        if not samples.exists():
            continue
        for path in directory.glob("*/collector.json"):
            collector = json.loads(path.read_text())
            assert collector["returncode"] == 0
            validate(json.loads(collector["stdout"]))
        rows = [json.loads(line) for line in samples.read_text().splitlines()]
        assert len(rows) == json.loads(completed.read_text())["samples"]
        for index, row in enumerate(rows):
            kind = row.get("operation", row.get("group"))
            attempt = directory / f"{index:04}-{kind}-{row['profile']}-{row['mode']}"
            collector = json.loads(json.loads((attempt / "collector.json").read_text())["stdout"])
            assert row["elapsed_ms"] == collector["elapsed_ns"] / 1e6, attempt
            assert row["rss_mib"] == collector["peak_rss_bytes"] / 2**20, attempt
            facts = json.loads((attempt / "stdout.log").read_text())
            assert all(row[key] == value for key, value in facts.items()), attempt
            for filename in ["verification.json", "anki.json"]:
                result = attempt / filename
                if result.exists():
                    assert json.loads(result.read_text())["status"] == "passed", result
            oracle = attempt / "oracle-process.json"
            if oracle.exists():
                assert json.loads(oracle.read_text())["returncode"] == 0, oracle
        plan = json.loads((directory / "plan.json").read_text())
        keys = ["profile", "role", "repeat", "mode"]
        keys.append("operation" if "operation" in rows[0] else "group")
        expected = Counter(tuple(row[key] for key in keys) for row in plan["schedule"])
        observed = Counter(tuple(row[key] for key in keys) for row in rows)
        assert expected == observed, directory
        if "operation" in rows[0]:
            measurement.validate_node_reports(rows)
            computed = measurement.node_summary(rows)
        else:
            for group, profile in {(r["group"], r["profile"]) for r in rows}:
                selected = [r for r in rows if (r["group"], r["profile"]) == (group, profile)]
                assert len({json.dumps(r["comparison"], sort_keys=True) for r in selected}) == 1
                assert len({r["logical_sha256"] for r in selected}) == 1
                if "manifest_fingerprint" in selected[0]:
                    assert all(r["manifest_verified"] for r in selected)
                    assert len({r["manifest_fingerprint"] for r in selected}) == 1
            computed = measurement.summary(rows)
        saved_path = directory / "summary-reviewed.json"
        if not saved_path.exists():
            saved_path = directory / "summary.json"
        saved = json.loads(saved_path.read_text())
        if not any(row["role"] == "timing" for row in rows):
            saved = {key: value for key, value in saved.items() if value.get("cells")}
        assert computed == saved, f"statistics differ: {directory}"
        results[directory.name] = {"samples": len(rows), "status": "verified",
            "regression_flags": [key for key, value in computed.items() if value.get("requires_confirmation")]}
    return {"scope": "offline schedule, raw observations, saved verification status, statistics and report equality; no APKG export or Anki import", "batches": results}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.evidence), indent=2))
