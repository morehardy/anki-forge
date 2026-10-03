# Shared snapshot blocks: full benchmark vs 2026-09-21

[中文完整报告](report.md) · [正式矩阵图](comparison.svg) · [20 格差异 CSV](delta.csv) · [同机确认](confirmation-summary.json)

Production changes coalesce small snapshot spills into bounded 4 MiB files with owned immutable segments and independent bounded readers, and use the existing UniCase ASCII path while preserving Unicode/NFC identity rules. The global 4 MiB snapshot residency budget, large-object streaming, content/identity validation and final publication synchronization remain intact. Caches keep weak owners and no persistent descriptors. Source is uncommitted; the new spool module is included in the frozen source archive and round2.patch.

The full 20-cell matrix measured 2026-10-02 19:13:37–19:18:38 Asia/Taipei. All 840 exports and 40 selected Anki import/content/render checks passed. Every Rust time median is slower than both the archived 2026-09-21 Deck matrix and the preceding optimized Project matrix. At 1,000 notes, images are 483.938 vs 238.638 ms (+102.8%), audio +103.7%, and mixed unique media +70.3%. Image RSS is 43.59 vs 40.25 MiB. These slower results are retained and are the values used in the chart and primary comparison tables.

Desktop load rose from 19.51 to 26.02; all launches stayed on AC Power, but the battery discharged from 95% to 93%. Current genanki time medians are about 27%–37% above the preceding complete run at 1,000 notes. API scope, background load, caches and writeback differ between sessions. Neither this observation nor the subsequent confirmation precisely removes environment costs from the historical comparison.

A separate predeclared same-host confirmation follows the full matrix: five 1,000-note profiles, three warmups and ten alternating before/after timings per configuration, with five preceding-version-first and five final-version-first pairs per profile. All 130 outputs and ten selected Anki checks passed. Unique images improve 505.668 → 365.624 ms (−27.7%), audio −31.6%, mixed unique media −21.7%; text/shared-media controls are largely unchanged. These samples compare two uninstrumented Project binaries and are never substituted into the full Rust/genanki matrix. Earlier short pilots and the discarded active-writer candidate are also retained.

## Evidence

- summary.json / comparison.csv: the complete formal matrix, medians/type-7 quartiles/extrema, independent RSS and default package sizes for both newly measured implementations.
- delta.json / delta.csv / compare.py: all changes against 2026-09-21 and the preceding optimized Project matrix, including comparator changes and ratio changes. report.md and comparison.{svg,png} use only this complete matrix.
- verification-summary.json / analyze.py: strict offline verification of 840 raw records, 400 timings, 200 independent RSS samples, 240 warmups, paired/balanced scheduling, AC records, unchanged source/tool/dependency/input identities and 40 actual-output Anki checks.
- historical-content-check.json: the first Rust timing output in every cell has the same learning-content digest as the corresponding raw 2026-09-21 output; 20 input JSON hashes also match. Media fixture hashes stayed fixed for all 2,749 files.
- run-manifest.json / plan.json / completed.json / host-hardware.json: source commit, current system/hardware, background load, power state, protocol and completion. Hardware was queried again during preparation.
- source-snapshot.json / source.patch / source-and-inputs.tar.gz / identity-after.json / identity-final.json: exact frozen source, before/after/final identities, fixture descriptions and checksum inventory. The Git diff alone omits the untracked new module; restore the complete source archive or use round2.patch against the frozen preceding archive.
- round2.patch: this round's implementation and regression-test changes relative to the preceding optimized source archive. No previous result archive was changed.
- prepared-builds.json / oracle-reuse-check.json / prepare-selected.log: locked offline final exporter/inspector/collector/checker builds and exact binary hashes; the checker source and pinned upstream revision/patch match the historical archive.
- verify-fast-selected.log: final selected source passes repository format/governance/clippy/workspace/script/whitespace gates. spool-red.log retains the pre-fix 192-file failure; lifecycle/core and other verification logs retain implementation iterations. benchmark-tests.log contains 46 passing behavior tests; smoke.log identifies the passing independent 200-note run.
- measurements-and-validation.tar.gz: all formal and smoke raw measurements, logs, artifact validation and selected Anki observations. Original APKGs were deleted after successful required validation and are not archived.
- confirmation-{plan,results,summary,source-before,source-after}.json / confirmation.py / confirmation-audit.json / audit-confirmation.py: the separate balanced ten-round confirmation. Diagnostic RSS was collected during timing and is not the formal independent RSS metric.
- pilot*.json / pilot*.py: shorter exploratory comparisons. confirmation-and-probes.tar.gz retains all 194 diagnostic outputs' raw evidence, 26 Anki records, prototype source variants and exact source identities. The active-writer candidate showed no reliable extra benefit and is excluded from production.
- archive-manifest.json / verify-archives.py: every member of all three archives is individually hashed and checked. SHA256SUMS covers the compact evidence files.
- archive-first-attempt.log: packaging caught a duplicate diagnostic manifest entry; no measurement was rerun. The corrected archive script verifies unique names before sealing.

## Offline verification

From this evidence directory:

```sh
shasum -a 256 -c SHA256SUMS
python3 verify-archives.py
```

From the repository root, extract into a new unused ignored directory:

```sh
mkdir -p benchmarks/.work/replay-spool-20261002
tar -xzf benchmarks/results/20261002-spool-genanki/measurements-and-validation.tar.gz -C benchmarks/.work/replay-spool-20261002
tar -xzf benchmarks/results/20261002-spool-genanki/confirmation-and-probes.tar.gz -C benchmarks/.work/replay-spool-20261002
```

The analyzer writes its derived outputs beside its script. Copy the compact plans and analysis inputs into a fresh workspace directory before running it, to preserve the sealed report:

```sh
mkdir -p benchmarks/.work/replay-spool-analysis
cp benchmarks/results/20261002-spool-genanki/*.json benchmarks/.work/replay-spool-analysis/
cp benchmarks/results/20261002-spool-genanki/*.py benchmarks/.work/replay-spool-analysis/
cp benchmarks/results/20261002-spool-genanki/*.patch benchmarks/.work/replay-spool-analysis/
cp benchmarks/results/20261002-spool-genanki/source-and-inputs.tar.gz benchmarks/.work/replay-spool-analysis/
benchmarks/.venv/bin/python benchmarks/.work/replay-spool-analysis/analyze.py --run-dir benchmarks/.work/replay-spool-20261002/run
python3 benchmarks/results/20261002-spool-genanki/audit-confirmation.py --work-dir benchmarks/.work/replay-spool-20261002/diagnostic
```

No exporter is launched by these audits. Rebuilt medians/quartiles and the complete validation summary must match the published files exactly. Baseline archives and the repository's frozen benchmark helper source remain required for comparison. A fresh live rerun must use a new run name, restore all frozen runtime files including spool.rs, prepare tools locked/offline, verify identical inputs/media inventory, keep builds/tests/analysis outside measurement and retain every repeat, slow cell and failure. Do not overwrite this sealed matrix.
