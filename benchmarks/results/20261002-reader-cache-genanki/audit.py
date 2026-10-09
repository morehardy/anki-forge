"""Audit every retained diagnostic result and recompute summaries offline."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import statistics

parser = argparse.ArgumentParser()
parser.add_argument("--work-dir", type=Path, default=Path(__file__).resolve().parent)
work = parser.parse_args().work_dir.resolve()

def read(path):
    return json.loads(path.read_text())

groups = {}
content = {}
for stem in ["baseline2", "stage", "control1", "control2", "codec", "confirmation"]:
    plan = read(work / f"{stem}-plan.json")
    results = read(work / f"{stem}-results.json")
    assert results["status"] == "completed"
    rows = results["rows"]
    assert len(rows) == plan["expected_exports"]
    expected = Counter({(p, m, role): count for p in plan["profiles"] for m in plan["modes"]
                        for role, count in [("warmup", plan["warmup_repeats"]),
                                            ("timing", plan["timing_repeats"])]})
    assert Counter((r["profile"], r["mode"], r["role"]) for r in rows) == expected
    seen = set()
    oracle_count = 0
    for row in rows:
        key = (row["role"], row["repeat"], row["profile"], row["mode"])
        assert key not in seen
        seen.add(key)
        case = work / (stem + "-" + "-".join(map(str, key)))
        measurement = read(case / "measurement.json")
        verified = read(case / "verification.json")
        assert measurement["reaped"] and not any(measurement[k] for k in
            ["spawn_error", "exit_code", "signal", "interrupted_signal", "leftover_descendants"])
        assert row["elapsed_ms"] == measurement["elapsed_ns"] / 1e6
        assert row["rss_mib"] == measurement["peak_rss_bytes"] / 2**20
        assert verified["status"] == verified["physical"]["status"] == verified["semantic"]["status"] == "passed"
        assert row["logical_sha256"] == verified["physical"]["logical_sha256"]
        assert row["artifact_sha256"] == verified["artifact_sha256"]
        content.setdefault(row["profile"], set()).add(row["logical_sha256"])
        assert row["power_before"].splitlines()[0] == row["power_after"].splitlines()[0] == "Now drawing from 'AC Power'"
        assert row["oracle_passed"] == (case / "anki.json").exists()
        if row["oracle_passed"]:
            assert row["role"] == "timing" and row["repeat"] == 0
            assert read(case / "anki.json")["status"] == "passed"
            oracle_count += 1
        if row["stages"]:
            line = next(l for l in (case / "stderr.log").read_text().splitlines()
                        if l.startswith("[DEBUG-residual-20261002] "))
            parsed = {k: {"count": v[0], "ms": v[1] / 1e6}
                      for k, v in json.loads(line.split(" ", 1)[1]).items()}
            assert row["stages"] == parsed
    assert oracle_count == plan["expected_anki_checks"]
    summary = read(work / f"{stem}-summary.json")
    assert {c["profile"] for c in summary} == set(plan["profiles"])
    for cell in summary:
        for mode in plan["modes"]:
            selected = [r for r in rows if r["role"] == "timing" and
                        r["profile"] == cell["profile"] and r["mode"] == mode]
            result = cell[mode]
            assert result["n"] == plan["timing_repeats"]
            assert result["samples_ms"] == [r["elapsed_ms"] for r in selected]
            assert result["median_ms"] == statistics.median(r["elapsed_ms"] for r in selected)
            assert result["rss_mib"] == statistics.median(r["rss_mib"] for r in selected)
            for label, stage in result["stages"].items():
                assert stage["ms"] == statistics.median(r["stages"].get(label, {"ms": 0})["ms"] for r in selected)
                assert stage["count"] == statistics.median(r["stages"].get(label, {"count": 0})["count"] for r in selected)
    if stem == "confirmation":
        first = Counter()
        for a, b in zip(rows[::2], rows[1::2]):
            assert all(a[k] == b[k] for k in ["role", "repeat", "profile"])
            assert {a["mode"], b["mode"]} == {"before", "after"}
            if a["role"] == "timing":
                first[a["profile"], a["mode"]] += 1
        assert len(first) == 10 and set(first.values()) == {5}
    before = work / f"{stem}-source-before.json"
    if before.exists():
        assert read(before) == read(work / f"{stem}-source-after.json")
    groups[stem] = {"exports": len(rows), "anki_checks": oracle_count, "status": "passed"}

for label in ["controls1", "controls2", "codec"]:
    build = read(work / f"build-{label}.json")
    source = work / f"{label}-source"
    for path, expected in build["source_files"].items():
        assert hashlib.sha256((source / path).read_bytes()).hexdigest() == expected, (label, path)

assert all(len(digests) == 1 for digests in content.values())
output = {"status": "passed", "exports": sum(g["exports"] for g in groups.values()),
          "anki_checks": sum(g["anki_checks"] for g in groups.values()),
          "learning_content_equal": True, "summaries_recomputed": True,
          "probe_source_hashes_verified": True, "groups": groups}
print(json.dumps(output, indent=2))
