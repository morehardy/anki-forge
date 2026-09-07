# Media export optimization: measured evidence

The current result uses ordinary `deck.media().add(...)` calls and default exports. The unreleased `add_many` interface was removed. Parallel media preparation is private to the exporter; registration still checks files immediately.

[Current four-tier results](report.md) · [Machine-readable summary](summary.json) · [Implementation report](../../../docs/superpowers/specs/2026-09-07-internal-media-registration-report.md)

Five profiles cover 100/200/500/1000 notes, with 10 fresh-process timing samples and 5 separate RSS samples per exporter/cell, plus 3 warmups before each phase. Every cell is faster than its same-run genanki control; 17/20 meet the proposed 30% elapsed-time reduction. At 1000 notes, images take 236.810 ms versus genanki's 299.155 ms (20.8% less time). Audio and mixed-unique are also below the 30% target. This is a local exploratory result, not a release or cross-platform guarantee.

All 840 outputs passed original SQLite row/template and exact media checks; 40 outputs passed pinned Anki import/content/render checks. All 420 Rust APKG hashes match the earlier batch exporter for the same inputs. `compatibility.json` retains those comparisons.

`attempts.jsonl`, `verified.jsonl` and `manifest.json` retain original timing, RSS and source/executable/dependency/build identities. `verification.tar.gz` contains each original verification and Anki record; `checks.json` pins verification bytes and `anki.json` aggregates import evidence. `inputs-json.tar.gz` contains all workload documents; media regenerates from the checked-in v2 recipe and golden hashes.

`historical/` preserves the earlier batch v1/v2 matrices, scalar comparison, image confirmation and intermediate measurements. Those results describe withdrawn or intermediate implementations and are not current-interface performance claims. Raw recorded paths identify local provenance; omitted logs, executables, APKGs and media files are not portable download links. Diagnostic scripts and source snapshots remain local.

[Publication map](publication.json) records source and published hashes. [SHA256SUMS](SHA256SUMS) covers published evidence. Measurement records are copied without rewriting their source identities; documentation and publication were prepared after measurement. Runtime source is unchanged since the current matrix. Validation is recorded in the implementation report and PR; build logs remain local.
