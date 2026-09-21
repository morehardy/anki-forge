# 2026-09-21 README performance comparison

[Full report](report.md) · [Time chart](time-heatmap.svg) ·
[Timing spread](time-scaling.svg) · [Resources](resources.svg)

One fresh session compares the current Rust working tree with genanki 0.13.1:
five workloads × 100 / 200 / 500 / 1,000 Basic notes. Per implementation/cell:
10 timings, 5 independent peak-RSS samples, and three warmups before each phase.
All 840 exports and 40 selected Anki import/content/render checks passed.

Measurements ran on 2026-09-21, 16:18–16:22 Asia/Shanghai, on Apple M1 Pro,
32 GiB, macOS 27.0 (26A428), native ARM64, AC power. Rust uses 1.92.0,
release/default product features and System allocator; genanki uses native
CPython 3.11.0. The working tree includes uncommitted optimizations relative to
`e2d69b8025fd6dcf817ccbe56208bd7022c5671c`. Older measurements are preserved in
their own directories and are not pooled into this session.

## Evidence

| File | Contents |
| --- | --- |
| [summary.json](summary.json), [comparison.csv](comparison.csv) | Every cell's median, Q1/Q3, min/max, sample count, time, RSS, and package size |
| [verification-summary.json](verification-summary.json) | Completeness, ordering, timing/RSS separation, artifact hashes, and 40 Anki checks |
| [run-manifest.json](run-manifest.json), [plan.json](plan.json) | Run protocol, actual environment, dates, source/build identities, host load, and power |
| [measurements-and-validation.tar.gz](measurements-and-validation.tar.gz) | All main-run attempts, collector logs, raw artifact checks, Anki results, power records, and separate smoke evidence |
| [archive-manifest.json](archive-manifest.json) | SHA-256 for every member of that archive |
| [source-snapshot.json](source-snapshot.json), [source.patch](source.patch), [source-and-inputs.tar.gz](source-and-inputs.tar.gz) | Pre-run hashes, tracked changes, runtime/adapter/verifier sources and locks, and all 20 input JSON files |
| [media-inputs.json](media-inputs.json), [identity-after.json](identity-after.json) | All 2,749 media file hashes and the unchanged final source/build identity |
| [prepared-builds.json](prepared-builds.json), [oracle-reuse-check.json](oracle-reuse-check.json) | Build commands/environments/executable hashes and verification of the reused pinned Anki oracle |
| [prepare.log](prepare.log), [benchmark-tests.log](benchmark-tests.log), [smoke.log](smoke.log), [run.log](run.log) | Preparation, 43 harness tests, smoke, and the complete run progress |
| [run.py](run.py), [analyze.py](analyze.py), [render.py](render.py) | Measurement protocol, strict offline validation/statistics, and chart/report generation |
| [SHA256SUMS](SHA256SUMS) | Hashes for every published evidence file except this checksum list itself |

Exported APKG bytes were checked, hashed, and removed after validation by the
runner to bound disk use. They are not included in the archive. A later
regenerated APKG is not evidence for the original historical bytes. Media can be
regenerated with the archived deterministic v2 generator and checked against
`media-inputs.json`.

## Verify or regenerate this report

Check the published files from this directory:

```sh
shasum -a 256 -c SHA256SUMS
```

Verify every archive member without running an exporter:

```sh
python3 - <<'PY'
import hashlib, json, tarfile
expected = json.load(open('archive-manifest.json'))['members']
with tarfile.open('measurements-and-validation.tar.gz') as archive:
    assert set(archive.getnames()) == set(expected)
    for member in archive.getmembers():
        assert hashlib.sha256(archive.extractfile(member).read()).hexdigest() == expected[member.name]
print('All archived records match their hashes.')
PY
```

From the repository root, extract into a new directory and audit the original
measurements. The analyzer rejects missing/duplicate samples, unbalanced timing
order, failures, missing Anki evidence, and changed frozen files:

```sh
mkdir -p benchmarks/.work/evidence-20260921
tar -xzf benchmarks/results/20260921-readme-genanki/measurements-and-validation.tar.gz -C benchmarks/.work/evidence-20260921
benchmarks/.venv/bin/python benchmarks/results/20260921-readme-genanki/analyze.py --run-dir benchmarks/.work/evidence-20260921/run
MPLCONFIGDIR=benchmarks/.work/matplotlib benchmarks/.venv/bin/python benchmarks/results/20260921-readme-genanki/render.py
```

This uses the existing hash-locked benchmark Python environment. Quantiles use
`report.stats` (Hyndman–Fan type 7). The original runner's `iqr_ms` uses a different
quartile convention and remains a raw record; the figures use the validated
Q1/Q3 values in `summary.json`.

## Run a new comparison

Follow the [benchmark preparation guide](../../README.md#run). The
`source-and-inputs.tar.gz` archive and `source.patch` permit reconstruction of
the measured source; the snapshot records the required Anki upstream revision
and local build patch. All compilation, dependency installation, harness tests,
fixture generation and smoke checks must finish before timing begins.

Use a fresh directory under `benchmarks/.work/`. `run.py` expects that layout,
an unused `NAME`, current `host-hardware.json`, and an `oracle-reuse-check.json`
that verifies the actual oracle binary/source against the pinned reference.
Record the new machine and build evidence rather than copying historical host
metadata as a new observation. The wrapper generates the fixtures and source
snapshot, declares sampling in `plan.json`, checks AC power around every export,
and stops on failures or changed identities. A power or source change requires
a separately named full session; retain the failed run instead of selectively
replacing samples.

These results include startup and default export checks; they do not measure
only the writer or a long-lived process. Default APKG formats differ. Desktop
load and filesystem cache are uncontrolled, thermal queries were unavailable,
and larger media workloads used more Rust RSS than genanki. See the full report
for the observed spread and conditions.
