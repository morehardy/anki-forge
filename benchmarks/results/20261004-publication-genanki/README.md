# Current publication implementation vs Sep 21 — 2026-10-04

Start with [report.md](report.md) and [comparison.png](comparison.png). One full
20-cell session measures committed runtime `c61188e` with byte-identical Sep 21
inputs, default features and System allocator. Historical API and session
conditions differ; differences are descriptive, not isolated causal effects.

- `summary.json`, `comparison.csv`: all cells, median/Q1/Q3/min/max, independent
  RSS, package sizes and newly measured genanki.
- `delta.json`, `delta.csv`, `historical-summary.json`: differences from the frozen
  Sep 21 reference; `reference-check.json` binds it to the source inventory.
- `verification-summary.json`: all 840 exports and 40 real Anki checks, complete
  sample schedule, order, AC-power checks, content hashes and unchanged identity.
- `plan.json`, `source-snapshot.json`, `source.patch`, `identity-after.json`:
  source/build identities and protocol. Existing unrelated uncommitted changes
  are recorded. Runtime source was not modified for this benchmark.
- `source-and-inputs.tar.gz`: frozen source, locks and 20 input JSON files.
- `media-inputs.json`, `baseline-input-check.json`: 2,749 media file identities
  and exact historical fixture equivalence. Media bytes can be regenerated using
  the archived deterministic workload generator and checked against these hashes.
- `measurements-and-validation.tar.gz`: every original main-run attempt,
  verification, Anki, collector and power record; separate smoke and build logs.
- `prepared-builds.json`, `oracle-reuse-check.json`: fresh locked/offline builds
  and independently pinned Anki checker source/upstream revision/patch/executable.
- `host-hardware.json`: reused same-machine hardware record. Current observed
  host load/power/thermal information is saved separately in the run manifest.
- `archive-manifest.json`, `SHA256SUMS`: archive-member and delivery checksums.

The benchmark harness's 46 tests and 200-note smoke passed before measurement.
Every cell contains 10 timings and 5 separate RSS samples per exporter, with
3 warmups before each phase. All samples are retained; no selective retry.

Verify and replay offline using Python 3.11+ (standard library only):

```sh
python3 verify-archives.py
python3 replay.py
shasum -a 256 -c SHA256SUMS
```

Replay checks all archive members and regenerates the exact statistics and report
from frozen records/source. It does not build, export, import Anki, or use original
absolute paths. Exported APKGs were hashed and deleted after successful validation;
executables and regenerated media are not included or reverified by offline replay.

For a fresh measurement, use a new work directory and run ID, rebuild with
`prepare.py`, regenerate and verify fixtures, then run `run.py`. Those scripts
retain the original repository/work layout and require path adaptation in another
checkout. Run all builds/tests/smoke before measurement, with no concurrent heavy
work. Original observations must remain unchanged. This standard matrix does not
resolve publication spec wide-field RSS or supported-platform CI acceptance.
