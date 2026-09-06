"""Offline descriptive reporting. No measurement, fixture generation or README writes."""
import hashlib
import json
import os
from pathlib import Path

SIZES = (200, 500, 1000, 10000)
ADAPTERS = ("rust", "genanki")
METRIC = "single_process_peak_rss_os_v1"


def quantile(values, p):
    values = sorted(values)
    position = (len(values) - 1) * p
    low = int(position)
    high = min(low + 1, len(values) - 1)
    return values[low] + (values[high] - values[low]) * (position - low)


def stats(values):
    return {"n": len(values), "median": quantile(values, .5), "q1": quantile(values, .25),
            "q3": quantile(values, .75), "min": min(values), "max": max(values)}


def cell_summary(records, verification, anki, size, adapter):
    rows = [r for r in records if r["size"] == size and r["adapter"] == adapter]
    timed = [r for r in rows if r["role"] == "timing"]
    memory = [r for r in rows if r["role"] == "memory"]
    successful = [r for r in rows if r["status"] == "success"]
    outcomes = [verification.get(r["id"], {}).get("status", "missing_verification") for r in successful]
    artifact = "invalid_artifact" if "invalid_artifact" in outcomes else "passed" if outcomes and all(s == "passed" for s in outcomes) else "unverified"
    failed = [r for r in rows if r["status"] not in ("success", "not_run") and not
              (r["role"].startswith("memory") and r["status"] == "collector_failure")]
    complete = len(timed) == 10 and all(r["status"] == "success" for r in timed)
    time_stat = stats([r["measurement"]["elapsed_ns"] for r in timed]) if complete and artifact != "invalid_artifact" else None
    size_stat = stats([verification[r["id"]]["artifact_bytes"] for r in timed]) if time_stat and all(
        verification.get(r["id"], {}).get("artifact_bytes") is not None for r in timed) else None
    rss_complete = len(memory) == 5 and all(r["status"] == "success" and
        r.get("memory", {}).get("metric") == METRIC and r["memory"].get("scope") == "single_process" and
        r["memory"].get("status") == "available" for r in memory)
    rss_stat = stats([r["measurement"]["peak_rss_bytes"] for r in memory]) if rss_complete and artifact != "invalid_artifact" else None
    selected = f"timing-{size}-{adapter}-01"
    oracle_status = anki.get(selected, {}).get("status", "missing_anki")
    if oracle_status == "passed" and (not verification.get(selected, {}).get("artifact_sha256") or
            anki[selected].get("artifact_sha256") != verification[selected]["artifact_sha256"]):
        oracle_status = "artifact_identity_mismatch"
    status = "verified" if time_stat and artifact == "passed" and oracle_status == "passed" and not failed else \
             "invalid_artifact" if artifact == "invalid_artifact" else "incomplete" if not complete else "unverified"
    first_check = next((verification[r["id"]] for r in timed if r["id"] in verification), {})
    return {"size": size, "adapter": adapter, "status": status, "artifact_status": artifact,
            "anki_status": oracle_status, "time_ns": time_stat, "peak_rss_bytes": rss_stat,
            "rss_status": "complete" if rss_stat else "unavailable_or_incomplete", "apkg_bytes": size_stat,
            "timing_succeeded": sum(r["status"] == "success" for r in timed), "timing_scheduled": len(timed),
            "timing_attempted": sum(r["status"] != "not_run" for r in timed),
            "memory_succeeded": sum(r["status"] == "success" for r in memory),
            "failures": [{"id": r["id"], "status": r["status"], "reason": r.get("reason")} for r in failed],
            "package": first_check.get("package"), "semantic_reader": first_check.get("semantic", {}).get("reader")}


def summarize(manifest, records, verification, anki):
    reasons = []
    if manifest.get("git_status"):
        reasons.append("dirty checkout: exploratory measurements")
    if not manifest.get("identity_unchanged"):
        reasons.append("source, fixture, executable or dependency identity changed/unconfirmed")
    if manifest["identity_before"].get("upstream_dirty"):
        reasons.append("upstream oracle checkout dirty/unavailable")
    cells = [cell_summary(records, verification, anki, size, adapter) for size in SIZES for adapter in ADAPTERS]
    pairs = []
    for size in SIZES:
        rust, genanki = [c for c in cells if c["size"] == size]
        ratio = delta = reduction = None
        if rust["time_ns"] and genanki["time_ns"]:
            r, g = rust["time_ns"]["median"], genanki["time_ns"]["median"]
            ratio, delta, reduction = g / r, (g - r) / 1e6, (g - r) / g
        pairs.append({"size": size, "genanki_over_rust": ratio, "genanki_minus_rust_ms": delta,
                      "rust_time_reduction": reduction,
                      "status": "draft" if reasons else "verified" if rust["status"] == genanki["status"] == "verified" else "unverified"})
    return {"schema": "basic-benchmark-summary-v1", "run_id": manifest["run_id"], "draft_reasons": reasons,
            "cells": cells, "comparisons": pairs, "headline_eligible": False,
            "headline_reason": "no predeclared matching full confirmation; this report emits descriptive values only"}


def duration(value):
    return "—" if value is None else f"{value / 1e6:.3f}"


def time_cell(cell):
    s = cell["time_ns"]
    return f"{duration(s['median'])} [{duration(s['q1'])}, {duration(s['q3'])}]" if s else "— (" + cell["status"] + ")"


def plot(summary, destination):
    os.environ.setdefault("MPLCONFIGDIR", str(Path(__file__).resolve().parent / ".work/matplotlib"))
    os.environ.setdefault("XDG_CACHE_HOME", str(Path(__file__).resolve().parent / ".work/cache"))
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    matplotlib.rcParams.update({"svg.hashsalt": "basic-benchmark-v1", "svg.fonttype": "none", "font.size": 10})
    fig, ax = plt.subplots(figsize=(9, 4.8), layout="constrained")
    colors, markers = ("#175A9C", "#B45119"), ("o", "s")
    for a, adapter in enumerate(ADAPTERS):
        for i, size in enumerate(SIZES):
            cell = next(c for c in summary["cells"] if c["size"] == size and c["adapter"] == adapter)
            s = cell["time_ns"]
            y = i + (-.13 if a == 0 else .13)
            if s:
                median = s["median"] / 1e6
                ax.errorbar(median, y, xerr=[[median - s["q1"] / 1e6], [s["q3"] / 1e6 - median]],
                            fmt=markers[a], color=colors[a], capsize=4, markersize=6,
                            label=("anki-forge / Rust" if a == 0 else "genanki / Python") if i == 0 else None)
                ax.annotate(f"{median:.3f} ms", (median, y), xytext=(7, -4 if a == 0 else 6), textcoords="offset points", fontsize=8)
            else:
                ax.text(0, y, f"{adapter}: {cell['status']}", fontsize=8)
    ax.set_yticks(range(4), ["Basic 200", "Basic 500", "Basic 1K", "Basic 10K"])
    ax.invert_yaxis()
    upper = max((c["time_ns"]["q3"] / 1e6 for c in summary["cells"] if c["time_ns"]), default=1)
    ax.set_xlim(0, upper * 1.24)
    ax.set_xlabel("Fresh-process export wall time (ms), including startup and input parsing")
    ax.set_title(("Exploratory draft — " if summary["draft_reasons"] else "") + "Basic export: median and IQR")
    ax.grid(axis="x", alpha=.2)
    ax.spines[["top", "right"]].set_visible(False)
    if any(c["time_ns"] for c in summary["cells"]):
        ax.legend(loc="upper right")
    fig.savefig(destination, format="svg", metadata={"Date": None, "Title": "Basic export benchmark: all four sizes",
        "Description": "Linear axis. Circles: Rust; squares: genanki. Points are medians of 10 fresh-process exports; whiskers show IQR spread, not confidence. Full numeric table is in report.md."})
    plt.close(fig)


def render(run):
    run = Path(run)
    manifest = json.loads((run / "manifest.json").read_text())
    records = [json.loads(line) for line in (run / "attempts.jsonl").read_text().splitlines()]
    verification = json.loads((run / "verification.json").read_text())
    anki = json.loads((run / "anki.json").read_text())
    provenance = {
        "report_format": "basic-benchmark-report-v2",
        "renderer_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "captured_run_renderer_sha256": manifest["identity_before"]["source_files"].get("benchmarks/report.py"),
        "inputs_sha256": {name: hashlib.sha256((run / name).read_bytes()).hexdigest()
                          for name in ("manifest.json", "attempts.jsonl", "verification.json", "anki.json")},
        "revision_note": "Presentation v2 moves the legend away from 10K and reserves label space. Measurement definitions and numeric aggregation are unchanged.",
    }
    (run / "render-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    summary = summarize(manifest, records, verification, anki)
    (run / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    lines = ["# Basic export benchmark", "", f"Run `{manifest['run_id']}` · {manifest['created_utc']}", ""]
    if summary["draft_reasons"]:
        lines += ["**Exploratory draft.** " + "; ".join(summary["draft_reasons"]) + ". Not a release/README advantage claim.", ""]
    lines += [f"Host: {manifest['host']['cpu']}; {manifest['host']['system']}; {manifest['host']['machine']}. "
              f"CPython {manifest['runtime']['python_version']}, genanki 0.13.1; "
              f"anki-forge {manifest['toolchain']['crate_version']}, Rust release/default features.", "",
              "Synthetic `basic-mixed-text-v1` (seed 20260906), one Basic card per note, no media. "
              "Each field is escaped inside its adapter. The time includes process startup, imports, JSON input parsing, "
              "default identity/checks, export and shutdown. Filesystem caches are warmed and uncontrolled; writes are closed without an fsync guarantee.", "",
              "Medians [Q1, Q3] are milliseconds from 10 attempts per cell; IQR is sample spread, not confidence. "
              "Percentiles use linear interpolation (Hyndman–Fan type 7). No significance or language-only throughput claim.", "",
              "| Notes/cards | anki-forge Rust, ms [IQR] | genanki, ms [IQR] | genanki / Rust | genanki − Rust, ms | Evidence |",
              "| ---: | ---: | ---: | ---: | ---: | --- |"]
    for pair in summary["comparisons"]:
        rust, genanki = [c for c in summary["cells"] if c["size"] == pair["size"]]
        ratio = "—" if pair["genanki_over_rust"] is None else f"{pair['genanki_over_rust']:.4f}×"
        delta = "—" if pair["genanki_minus_rust_ms"] is None else f"{pair['genanki_minus_rust_ms']:+.3f}"
        status = f"{pair['status']}; Rust {rust['status']}; genanki {genanki['status']}"
        lines.append(f"| {pair['size']:,} | {time_cell(rust)} | {time_cell(genanki)} | {ratio} | {delta} | {status} |")
    lines += ["", "A ratio below 1 means Rust took longer. A negative difference means additional time in Rust.", "",
              "![All four sizes, linear time axis; medians and IQR](timing.svg)", "", "## Separate memory and output measurements", "",
              "RSS is the OS high-water mark of one completed exporter process (`single_process_peak_rss_os_v1`), "
              "from five separate memory attempts. It includes the runtime and is not a heap-allocation or whole-tree metric. "
              "Size uses only the ten timed APKGs. Values below are median [min, max].", "",
              "| Notes/cards | Implementation | OS peak RSS, MiB (n=5) | APKG, KiB (n=10) | Memory status |",
              "| ---: | --- | ---: | ---: | --- |"]
    for c in summary["cells"]:
        formatted = []
        for key, divisor in (("peak_rss_bytes", 1024 ** 2), ("apkg_bytes", 1024)):
            s = c[key]
            formatted.append("—" if not s else f"{s['median']/divisor:.2f} [{s['min']/divisor:.2f}, {s['max']/divisor:.2f}]")
        lines.append(f"| {c['size']:,} | {c['adapter']} | {formatted[0]} | {formatted[1]} | {c['rss_status']} |")
    lines += ["", "The preselected showcase size is **10,000 notes / 10,000 cards**, retained regardless of the winner. "
              "Rust writes modern `collection.anki21b` with nested zstd and a legacy compatibility placeholder; genanki writes "
              "legacy `collection.anki2` with ZIP_STORED. Stock CSS, metadata, IDs and default validation work differ. "
              "These are default-output comparisons; no package is converted or recompressed for scoring.", "",
              "## Frozen synthetic data", "",
              "| Notes | Input, KiB | English / mixed / escaping | Front code points, min–median–max | Back code points, min–median–max | zlib-9 compressed / raw |",
              "| ---: | ---: | --- | --- | --- | ---: |"]
    for size in SIZES:
        e = manifest["fixture_evidence"]["cases"][str(size)]
        f, b = e["fields"]["front"]["codepoints"], e["fields"]["back"]["codepoints"]
        categories = e["categories"]
        lines.append(f"| {size:,} | {e['input_bytes']/1024:.2f} | {categories['english']} / {categories['mixed']} / {categories['escaping']} | "
                     f"{f['min']}–{f['median']:g}–{f['max']} | {b['min']}–{b['median']:g}–{b['max']} | {e['compression_diagnostic']['compressed_raw_ratio']:.2%} |")
    lines += ["", "The phrase pools make this corpus repetitive. The compressor ratio is a characterization diagnostic, "
              "not a performance score or evidence that real decks have this compressibility. Exact UTF-8 byte distributions, "
              "compressor version and golden fixture hashes are in the manifest.", "",
              "## Correctness and provenance", "",
              "All successful attempts, including both sets of warmups, are checked after both measurement passes: physical rows, "
              "GUID/field multiplicity, card associations, templates and complete literal text. Modern semantics use the repository inspector; "
              "legacy schema-11 semantics use the benchmark-local read-only reader because the repository inspector requires a modern decks table. "
              "Independent verification requires the first timed artifact per cell to be imported into a fresh pinned Anki collection, "
              "every imported field/card to be checked and deterministic English, mixed and escaping examples to be rendered.", "",
              f"Anki checks passed for {sum(c['anki_status'] == 'passed' for c in summary['cells'])} of 8 cells. "
              "Missing or failed oracle evidence remains explicitly unverified.", "",
              f"Anki revision: `{manifest['identity_before']['upstream_revision']}`. Artifact and executable SHA-256 identities bind the evidence. "
              "Successful artifacts are cleaned only after their required checks; selected artifacts awaiting Anki are retained.", "",
              "No advantage headline is generated: this run has no predeclared full confirmation. "
              "The complete matrix is retained even where Rust is slower. No outliers are removed or unsuccessful attempts retried.", "",
              "[Manifest and frozen inputs](manifest.json) · [Every attempt](attempts.jsonl) · [Physical/semantic checks](verification.json) · "
              "[Anki evidence](anki.json) · [Phase events](events.jsonl) · [Machine-readable summary](summary.json) · [Retention](retention.json) · "
              "[Report revision and renderer identity](render-provenance.json)", "",
              "Regenerate offline: `benchmarks/.venv/bin/python benchmarks/bench.py report <run-directory>`.", ""]
    (run / "report.md").write_text("\n".join(lines), encoding="utf-8")
    plot(summary, run / "timing.svg")
    return summary
