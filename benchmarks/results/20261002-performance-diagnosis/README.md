# 2026-10-02 performance regression diagnosis

[Findings and repair direction](report.md) · [Sync ablation medians](control-summary.json) · [Same-host historical comparison](history-summary.json)

Diagnostic source copies isolate the default Project media path, two intermediate synchronization points, and native identity-validation work. No product sources were modified. The original measured adapter was restored after every build; `completion.json` verifies runtime sources against the preceding benchmark snapshot, executable SHA-256 and all four build-provenance records.

All **117 diagnostic exports** passed original physical/semantic/media checks; **14 selected outputs** passed actual Anki import/content/render checks:

| Experiment | Exports | Sampling | Anki checks |
| --- | ---: | --- | ---: |
| Minimal fixed-note-count reproduction | 12 | 0/1/10/100 images × 3 repeats | 0 |
| First stage probe | 6 | Text/images, 1,000 notes × 3 repeats | 0 |
| Same-binary CAS/staging sync ablation | 80 | 5 scenes × 4 modes × (1 warmup + 3 timings) | 10 |
| Frozen historical versus current source | 16 | Text/images × 2 versions × (1 warmup + 3 timings) | 4 |
| Identity substage probe | 3 | 1,000 text notes × 3 repeats | 0 |

The 80-export ablation and 16-export historical comparison both verify equal logical field digests within each scene. Every output's exact original media bytes were checked separately before removal. These are diagnostic experiments, not additional full benchmark scores. Same-process RSS observations are retained without treating them as the benchmark's independent memory pass.

`diagnostic.patch` applies only to a private source copy. Its environment controls independently bypass CAS and staging sync for the ablation; they are absent from product sources. Final package sync/publication and all package checks remained enabled. Byte/content checks establish output validity for these successful experiments; they do not establish crash durability or justify applying the bypass to shared persistent CAS/staging implementations.

## Evidence

- `reproduction.json`, `stages-results.json`, `control-results.json`, `history-results.json`, `identity-results.json`: all raw samples, stage counters and statuses, including predeclared warmups and first-launch startup overhead.
- `control-plan.json`, `history-plan.json`: sample schedules, profiles, modes and executable/source identities.
- `control-summary.json`, `history-summary.json`: medians and every formal elapsed-time sample.
- `historical-source-check.json`: historical source files checked against the immutable 2026-09-21 snapshot before instrumenting.
- `build-*.json`, `build-*.log`: exact commands, private-source file hashes and diagnostic executable identities.
- `completion.json`: original adapter restored, product source identities unchanged, build provenance verified, final host/power/load observation.
- `measurements-and-validation.tar.gz`, `archive-manifest.json`: the 117 cases' native collector measurements, exporter logs, validation records and selected Anki results, with a SHA-256 inventory checked against every archive member.
- `*.py`, `diagnostic.patch`: reconstruction/measurement/report scripts and final private instrumentation. `SHA256SUMS` covers every retained evidence file except the checksum list itself.

Per-call worker timings overlap across threads and include nested scopes. Do not sum them into process wall time. Reports use whole-stage wall time and same-binary ablations for causal conclusions. New probe executables' earliest starts had extra uninstrumented startup overhead; all records remain intact. The control/history medians use their predeclared timing roles. Identity substage medians include all three samples, while first-probe total elapsed time is not a headline result.

## Verify

From this directory:

```sh
shasum -a 256 -c SHA256SUMS
```

The raw archive's `archive-manifest.json` maps each member name to its SHA-256. Archive outputs contain validation evidence, not APKG binaries; packages were removed only after their required exact-output checks. Recreating an APKG does not validate historical output bytes.

## Reproduce locally

Use the repository's prepared hash-locked Python 3.11 environment, Rust 1.92.0, native collector, inspector and pinned Anki oracle. See [benchmark setup](../../README.md#run). Finish all compilation before measuring. Use a fresh directory under `benchmarks/.work/` and copy these scripts into it; do not execute setup/report scripts in this archived evidence directory.

The scripts refer to the preceding run's frozen fixtures at `benchmarks/.work/latest-20261002/fixtures`. If absent, regenerate using the current `media_workload.generate` recipe and verify the hashes against `../20261002-latest-genanki/baseline-input-check.json` and `media-inputs.json` before proceeding. This guarantees the same workload while avoiding use of historical host observations as new observations.

From the repo root, substitute a fresh directory for `WORK`:

```sh
benchmarks/.venv/bin/python WORK/reproduce.py
benchmarks/.venv/bin/python WORK/setup_probe.py
benchmarks/.venv/bin/python WORK/build_probe.py stages
benchmarks/.venv/bin/python WORK/run_probe.py stages basic-mixed-text-v1 basic-image-unique-v2
benchmarks/.venv/bin/python WORK/setup_controls.py
benchmarks/.venv/bin/python WORK/build_probe.py controls
benchmarks/.venv/bin/python WORK/controls.py
benchmarks/.venv/bin/python WORK/extract_historical.py
benchmarks/.venv/bin/python WORK/setup_historical.py
benchmarks/.venv/bin/python WORK/build_probe.py historical
benchmarks/.venv/bin/python WORK/history_compare.py
```

The builder temporarily shares dependency artifacts with the benchmark target directory and always restores the saved original exporter in a `finally` block. Do not run other builds/benchmarks against that directory concurrently. Independent source trees and original/diagnostic binary copies remain in the explicitly named debug work directory.

The final `identity` binary adds three scopes (`IdentityEnvelope::validate`, `note_from_collection`, `config_rows`) to the controls source as shown in `diagnostic.patch`; rebuild it using `build_probe.py identity`, then run `run_probe.py identity basic-mixed-text-v1`. Instrumentation source hashes are preserved in `build-identity.json`.
