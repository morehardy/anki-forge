# Publication implementation evidence — 2026-10-03

This directory records the implementation of the
[publication performance specification](../../../docs/superpowers/specs/2026-10-03-publication-performance-spec.md).
The source implementation is committed, but the specification's acceptance gate
is **not passed**: the confirmed wide-field RSS regression remains unresolved and
the supported-platform CI matrix has not run.
A completed measurement batch means its scheduled attempts and content checks
completed; it does not mean all performance or release gates passed.

The report distinguishes actual measurements, functional verification and pending
cross-platform completion and release gates. The implementation has not been published to package
registries. Existing historical benchmark evidence was not modified.

- [Report](report.md): timing targets, controls, costs, regressions and limitations.
- [Verification and review](verification.md): F1–F11 coverage and the two review axes.
- `raw-evidence.tar.gz`: all recorded attempts, full reports/comparisons, collector
  records, verification/oracle results, build/test logs and source inventories.
- `fixture-inputs.tar.gz`: original JSON inputs and random-byte control payloads.
- `baseline-packages.tar.gz`: the actual fixed APKG baselines used for review.
- `SHA256SUMS`: archive and report integrity.
- `archive-verification.json`: all 26,873 archived members matched their manifests.
- `fixture-restoration.json`: 7,109 restored media references passed SHA-256 checks.
- `restored-offline-audit.json`: statistics recomputed from the extracted archive.
- `offline-audit.json`: independent recomputation of saved schedules/statistics;
  this does not export new APKGs or import anything into Anki.

## Recompute statistics

Use the repository benchmark environment, with the locked benchmark dependencies:

```sh
mkdir -p tmp/publication-evidence
cd tmp/publication-evidence
tar -xzf ../../benchmarks/results/20261003-publication-implementation/raw-evidence.tar.gz
cd ../..
benchmarks/.venv/bin/python benchmarks/publication_audit.py tmp/publication-evidence
```

All timing samples, quartiles, paired differences and the fixed-seed descriptive
bootstrap ranges are present in each batch's `summary.json` (or
`summary-reviewed.json` for the original Node batches). Failed smoke attempts
remain separate from complete batches. No individual slow score was replaced.

## Restore inputs and rerun

Extract the two fixture archives into the same evidence directory. Run
`benchmarks/publication_restore_fixtures.py` with the benchmark Python environment
to recreate standard PNG/WAV data using the existing deterministic generator.
The script checks every media SHA-256. The archived baseline APKGs are used
unchanged; rebuilding a baseline may change nonsemantic package timestamps.

The fixed source baseline is `7c4c8ed`, which commits the exact prerequisite core
changes that were present in the invocation-time worktree. The preceding HEAD
alone (`eb4b463`) is not the complete implementation baseline. The original
worktree patch, inventory and their hashes are retained in the evidence.

The four implementation commits are `a2bf805`, `9b152ea`, `a175697`, and `3a9e5da`.
Build recipes and per-variant source inventories are archived; source variants
include only the selected production changes. The original benchmark adapter and
runners are retained under `harness-v1/` because later changes expanded validation
and controls. Node's preserved pre-COW and COW build sources are included. Their
initial inventory was recorded after measurement from the preserved source copy;
the binaries themselves were hash-frozen in the original plans.

For a fresh source archive, `git archive 7c4c8ed` supplies the prerequisite tree.
To replay the archived control recipe, write that gzip archive to
`tmp/publication-implementation-baseline/source.tar.gz`, run the variant builder
with `tmp/publication-performance` as its work directory, and invoke the archived
`build-controls.py` from the repository root.
The source-archive builder uses the safe tar extraction filter and requires
Python 3.12 or later. Measurement and audit use the locked benchmark environment.
Use the archived original adapter for exact original-harness reconstruction, then
`publication_build_variants.py` for isolated core variants. Control binaries are
built by the archived `build-controls.py`; it uses the same prerequisite core,
adding only the manifest or media change as appropriate. It records exact build
commands, binary hashes and inventories. Run builds and smoke before timing;
never overlap compilation, tests or two measurement batches.

`publication_performance.py` runs the core and Node matrices;
`publication_controls.py` runs standalone compare, full staging, concurrent miss
and real Node/Python review workflows. `publication_standard.py` runs all five
existing profiles at 100/200/500/1000 notes. The archived `run-controls.sh` records
the complete sequential control schedule. Paths in original execution records
are historical absolute paths; current scripts accept explicit fixture, binary
and output locations for replay.
