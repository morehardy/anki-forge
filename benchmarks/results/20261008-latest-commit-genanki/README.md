# Latest committed code versus September 21

Start with [report.md](report.md) and [comparison.png](comparison.png).
The measured runtime is commit `11991964b06896b2e07ba09cbb3e6465f7585099`, including
build-time validated defaults and immutable memory-snapshot digest reuse.
The existing four-worker media imports, 16 MiB encoded pool and shared 64 MiB
snapshot budget are active. No production code was changed for this run.

One complete 20-cell matrix covers five profiles at 100/200/500/1,000 notes.
Every Rust/genanki cell has 10 timing samples, 5 separate peak-RSS samples and
3 warmups before each phase. Input JSON and media bytes match the September 21
archive. Timing covers native process spawn through exit; verification follows
each adjacent pair outside the timer. OS caches/background activity are not
isolated. Historical Deck and current Project APIs perform different work.

- `summary.json`, `comparison.csv`: all medians, quartiles, ranges, independent
  RSS, package sizes and freshly measured genanki.
- `delta.json`, `delta.csv`, `historical-summary.json`, `reference-check.json`:
  every comparison with the frozen September 21 reference.
- `verification-summary.json`: every original artifact check, selected actual
  Anki import, logical content hash, measurement order and power record.
- `plan.json`, `source-snapshot.json`, `source.patch`, `identity-after.json`:
  source/build identities and the fixed protocol. `measured-source-check.json`
  confirms the production/harness inputs match the commit. Existing unrelated
  documentation changes are recorded rather than labelled a clean worktree.
- `source-and-inputs.tar.gz`: frozen source/locks and 20 input JSON files.
  `media-inputs.json` and `baseline-input-check.json` preserve all 2,749 media
  identities and the exact historical fixture comparison.
- `measurements-and-validation.tar.gz`: all main-run attempts, verification,
  Anki, collector and power records, plus smoke, build and harness-test logs.
- `prepared-builds.json`, `oracle-reuse-check.json`: freshly rebuilt exporter,
  inspector and collector; reused Anki executable after exact binary SHA,
  checker source/locks and upstream revision/patch verification. Anki was not
  rebuilt in this session; its recorded provenance remains explicit.
- `host-hardware.json`: prior identity of this same host, with its original
  query date. Current power/load/available thermal readings are in the manifest.
- `quality.json`: this session's 46 harness tests and 200-note smoke, plus a
  reference to the unchanged commit's prior Rust/Node/Python/package checks.
- `archive-manifest.json`, `SHA256SUMS`: archive-member and delivery hashes.

Verify and replay offline with Python 3.11+ (standard library only):

```sh
python3 verify-archives.py
python3 replay.py
shasum -a 256 -c SHA256SUMS
```

Replay verifies all archived source/input JSON and records, then regenerates
statistics and the report byte-for-byte. It does not run exporters, Anki or
builds. Successful APKGs were checked and hashed before deletion. Executables,
media payloads and build caches are omitted; their identities remain recorded.

For a fresh measurement, copy the runner/preparation/analysis scripts to a new
work directory and change the run name. Prepare the tools and complete all
builds/tests/smoke before running `run.py`. It regenerates and hash-checks the
shared fixtures and refuses to replace an existing run. Environment paths
reflect this repository layout and may require adaptation on another host.
