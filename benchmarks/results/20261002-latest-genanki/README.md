# 2026-10-02 current-code benchmark

[Full comparison](report.md) · [20-cell timing chart](time-delta.svg) · [CSV deltas](delta.csv)

One standard five-scene × four-size matrix at commit `4265fc4751daa18164cbf85bda425f2120e0f57d`, measured 2026-10-02 15:31:03–15:51:59 Asia/Taipei. The current Rust `Project` API is compared with the archived 2026-09-21 `Deck` API; genanki 0.13.1 was independently remeasured. All 20 input JSON SHA-256 hashes match the historical baseline. The frozen v2 PNG/WAV generator reproduces the 2,749 media files.

Each implementation/cell has 10 fresh-process timings, 5 independent peak-RSS samples, and 3 warmups before each phase. All 840 exports and 40 selected Anki import/content/render checks passed. Source, executable, dependency and fixture identities remained unchanged; every launch was checked for AC power before and after. The 46 benchmark tests and separate 200-note smoke passed before measurement.

All 20 Rust elapsed-time medians increased. At 1,000 notes, unique images and audio took 41.3× and 53.5× the historical time, with higher RSS. Text and shared-media RSS decreased. The report preserves the full matrix, quartiles, package sizes, remeasured genanki and power/load details. Differences include API paths and uncontrolled desktop conditions; this is a single-session descriptive comparison, not an isolated root-cause diagnosis.

## Evidence

- `summary.json`, `comparison.csv`: validated current medians, Q1/Q3, extrema, sample counts, RSS and APKG sizes.
- `delta.json`, `delta.csv`: signed changes from the immutable 2026-09-21 baseline, plus the change in Rust/genanki time ratio.
- `verification-summary.json`: 840 original-artifact checks, 40 Anki checks, sample completeness and ordering.
- `run-manifest.json`, `plan.json`, `completed.json`, `host-hardware.json`: measured environment, commit, complete Git state, scheduling, power/load, identities and completion.
- `baseline-input-check.json`, `media-inputs.json`: matching historical inputs and frozen media hashes.
- `source-snapshot.json`, `source.patch`, `source-and-inputs.tar.gz`, `identity-after.json`: exact before/after source and build evidence. Tracked product source was clean; the initial Git status contained three unrelated untracked documents, so the run retains exploratory labeling.
- `prepared-builds.json`, `oracle-reuse-check.json`, `oracle-build.log`: current exporter/inspector/collector and rebuilt oracle provenance. The benchmark oracle source and pinned upstream Anki revision/patch match the historical baseline; its Cargo manifest/lock and sibling oracle changed, so this run rebuilt the checker instead of claiming unchanged source provenance.
- `measurements-and-validation.tar.gz`, `archive-manifest.json`: all run/smoke attempts, collector/exporter logs, original-artifact checks, Anki results and per-export power records, with every archive member hashed. Exported APKG bytes were removed only after their required validation; they are not archived.
- Preparation, test, smoke, run and analysis logs: includes the initial preparation failure caused by the restricted uv cache and the successful workspace-cache retry. Those preparation commands were outside measurement.
- `run.py`, `analyze.py`, `compare.py`: frozen measurement, strict validation and offline comparison/chart scripts.
- `SHA256SUMS`: integrity hashes for all evidence files except this checksum list itself.

## Verify and regenerate offline

From this evidence directory:

```sh
shasum -a 256 -c SHA256SUMS
```

From the repository root, with the hash-locked benchmark Python environment:

```sh
mkdir -p benchmarks/.work/evidence-20261002
# This extracts validation records, not exporter outputs or executable code.
tar -xzf benchmarks/results/20261002-latest-genanki/measurements-and-validation.tar.gz -C benchmarks/.work/evidence-20261002
benchmarks/.venv/bin/python benchmarks/results/20261002-latest-genanki/analyze.py --run-dir benchmarks/.work/evidence-20261002/run
benchmarks/.venv/bin/python benchmarks/results/20261002-latest-genanki/compare.py
```

The analyzer rejects incomplete/duplicate samples, failed attempts, unbalanced timing order, missing Anki evidence, power-source changes and inconsistent frozen file hashes. Quartiles use Hyndman–Fan type 7. The runner's native `iqr_ms` summary uses a different convention and is retained only as a raw record.

For a new session, follow [the benchmark preparation guide](../../README.md#run), use a fresh run name and finish compilation/tests/smoke before measuring. Record the actual hardware/build/oracle observations; historical metadata must not be reused as a new observation. Copy the wrapper to a fresh directory under `benchmarks/.work/`, update its `NAME`, and prepare its current `host-hardware.json` and `oracle-reuse-check.json`. A failed or interrupted session must remain separate; do not selectively replace samples.
