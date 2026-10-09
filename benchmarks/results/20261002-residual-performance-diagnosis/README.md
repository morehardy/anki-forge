# 2026-10-02 residual performance diagnosis

[中文诊断报告](report.md) · [控制矩阵](control-summary.json) · [身份细分](detail-summary.json) · [审计结果](audit.json)

This is a diagnosis of the already optimized Project runtime, not a new full Rust/genanki comparison. It retains the slower controls and the memory cost of increasing snapshot residency. Production code was not changed during this investigation. The official optimized exporter was restored after each private probe build.

The main finding is the per-object temporary snapshot lifecycle under the shared 4 MiB budget. For 1,000 unique images, 935 files spill; a diagnostic 128 MiB residency budget reduces process median time from 414.679 to 244.814 ms while increasing diagnostic peak RSS from 43.27 to 102.05 MiB. Single-worker media preparation is slower. Native identity validation costs about 7.6 ms for text and 14.4 ms for images. Skipping the initial candidate sync transfers some cost to final synchronization and does not demonstrate a consistent end-to-end benefit.

The same-host baseline, randomized control matrix and detailed identity traces contain 216 completed exports: 36 warmups and 180 measured runs. All original artifact checks and all 36 selected upstream Anki import/content/render checks passed. There are five timed observations per configuration and no discarded samples. Diagnostic RSS is collected during these timing invocations, unlike the separate RSS protocol of the archived full benchmark. Background desktop load remained uncontrolled; see the report's negative controls and environment caveats.

## Evidence

- baseline-{plan,results,summary}.json and baseline.log: interleaved current official and previously source-verified historical Deck binary.
- control-{plan,results,summary}.json and control.log: five 1,000-note profiles and six modes. Every private mode changes one variable from default; official measures probe overhead. Snapshot budget 0/128 MiB, serial preparation and skipped initial sync are diagnostic controls only.
- detail-{plan,results,summary}.json and detail.log: text/image repeated traces of identity checksum allocation, collection hashing, media history and note/card validation. The original canonical checksum helper is inlined without changing serialization rules.
- source-before.json / source-after.json / runtime-match.json / identity-check.json: all 2,110 registered source files, inputs, tools and dependencies stayed unchanged during measurement. The 253 relevant runtime files and all tools match the optimized full benchmark despite an unrelated brand commit changing HEAD to 937d938.
- source.patch: pre-existing uncommitted production optimization against that HEAD, frozen before this report's documentation changes.
- build-{probe,detail,historical}.json / probe.patch / detail.patch: exact private source and binary hashes. Historical source reuse matches the earlier diagnosis build record.
- host-hardware.json: the hardware query retained from the optimized benchmark; identity-check.json records this session's system, ending load and power. Each raw row also contains before/after AC records.
- measurements-and-validation.tar.gz: every completed process measurement, stdout/stderr, raw/semantic validation and selected Anki result, together with plans, summaries and source identities. APKG bytes were removed after successful checking and are not archived.
- source-and-probes.tar.gz: both instrumented source copies, the verified historical source copy, preparation/build/run scripts, build logs and the five exact 1,000-note input descriptions. Original media bytes and official uninstrumented source/checker provenance are retained in the referenced [optimized benchmark evidence](../20261002-optimized-genanki/README.md); that archive remains required for fresh measurement. No executable binaries are distributed here.
- archive-manifest.json: all 1,848 archive members individually hashed and checked after writing. SHA256SUMS covers every compact evidence file.
- audit.py / audit.json: offline verification of complete sample counts, unique scheduling keys, successful process/verification/oracle records, unchanged production identities, identical learning content, AC power, raw trace parsing and recomputed summaries. No exporter is launched.

## Offline verification

From this evidence directory:

```sh
shasum -a 256 -c SHA256SUMS
python3 verify-archives.py
```

From the repository root, extract into a new unused directory:

```sh
mkdir -p benchmarks/.work/replay-residual-20261002
tar -xzf benchmarks/results/20261002-residual-performance-diagnosis/measurements-and-validation.tar.gz -C benchmarks/.work/replay-residual-20261002
python3 benchmarks/results/20261002-residual-performance-diagnosis/audit.py --work-dir benchmarks/.work/replay-residual-20261002/raw
```

The archive verifier checks every member's recorded hash without extracting it. The offline audit recomputes every summary from complete raw records. Use a fresh work directory and update the private scripts' recorded paths for a new live measurement; never overwrite these measurements. Rebuild source copies offline and locked, confirm frozen source/dependency/input hashes, preserve all warmups and repeats, keep compilation and analysis outside the timed matrix, and restore the official tool executable afterward. Applying private controls to production would be a separate implementation decision requiring its own semantic and ownership regression checks.
