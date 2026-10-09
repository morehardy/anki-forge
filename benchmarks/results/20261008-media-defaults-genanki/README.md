# Production media defaults vs September 21 — 2026-10-08

Start with [report.md](report.md) and [comparison.png](comparison.png).
One full 20-cell matrix measures the production changes in the frozen patch:
public ordered batch file imports (up to four workers), a 64 MiB process-wide
live-snapshot budget and a 16 MiB encoded-media pool per preparation.
The Rust adapter adopts the public batch API; Node and Python expose it too.
Existing single-file callers need to adopt the batch method for concurrency.

Each implementation/cell has 10 timing samples, 5 independent peak-RSS samples
and 3 warmups before each phase. All 840 exports and 40 actual Anki checks pass.
Inputs are byte-identical to the September 21 archive; the historical Deck API
and current Project API have different work scopes and session conditions.
Differences describe these sessions and do not isolate the three changes.

- `summary.json`, `comparison.csv`: medians, quartiles, ranges, separate RSS,
  output size and freshly measured genanki for every cell.
- `delta.json`, `delta.csv`, `historical-summary.json`, `reference-check.json`:
  the frozen September 21 reference and all current/historical differences.
- `verification-summary.json`: complete schedule, logical content hashes,
  artifact checks, Anki checks and AC-power observations.
- `plan.json`, `source-snapshot.json`, `source.patch`, `identity-after.json`:
  source/build identities, the measured working-tree patch and protocol.
- `source-and-inputs.tar.gz`: frozen core, binding, adapter, checker and harness
  source/locks and 20 input JSON files. Unrelated pre-existing worktree changes
  are recorded; nothing is committed or published by this task.
- `media-inputs.json`, `baseline-input-check.json`: all 2,749 media identities
  and historical fixture equivalence. The frozen workload generator can
  regenerate media bytes for comparison with these hashes.
- `measurements-and-validation.tar.gz`: every main-run attempt, verification,
  Anki, collector and power record, plus smoke and build/regression logs.
- `prepared-builds.json`, `oracle-reuse-check.json`: freshly rebuilt exporter,
  inspector and collector; exact October 4 Anki checker executable reused after
  checking its SHA, recorded source/lock files, upstream revision and patch.
  Its initial rebuild hit a restricted cargo-cache write; the failure is saved.
- `quality.json`: Rust, Node, Python, typing, harness and crate-payload checks. Initial Node
  test-fixture failure and its corrected full rerun are retained in the logs.
  After measurement, the Rust package inventory was amended to list the
  new batch source and the native binding additions were formatted; `post-measurement.patch` and `final-source-check.json`
  retain these packaging/formatting deltas. Core and benchmark adapter sources
  still match; binding changes were verified against rustfmt output. Workspace
  formatting and strict all-target/all-feature clippy checks pass.
- `host-hardware.json`: existing same-host hardware identity; current load,
  power and available thermal readings are recorded in the manifest.
- `archive-manifest.json`, `SHA256SUMS`: archive-member and delivery hashes.

Verify and replay offline using Python 3.11+ (standard library only):

```sh
python3 verify-archives.py
python3 replay.py
shasum -a 256 -c SHA256SUMS
```

Replay validates retained archive members/source/input JSON, reconciles the
records and regenerates statistics/report byte-for-byte. It does not execute
exporters, Anki or builds. Exported APKGs were verified, hashed and deleted;
media bytes, compiled executables and build caches are omitted from the archive.

For a new measurement, copy the scripts to a new work directory and run ID,
prepare the tools, generate/verify fixtures and run `run.py`. Paths retain this
repository's layout and require adaptation elsewhere. Finish all builds/tests
before collection. Do not modify or overwrite the original observations.
