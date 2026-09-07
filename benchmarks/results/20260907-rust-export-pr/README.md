# Rust export optimization: implementation and measured results

This PR consolidates the export optimizations developed after `72802e55e80892a97ef1feff610b865bf27d6e42`. The public library keeps its System allocator, existing toolchain, contract and output behavior. The private benchmark adapter can opt into mimalloc 0.1.52; this does not configure a Python or Node host allocator.

The implementation removes repeated authoring/model copies, reuses reconciliation data, reads validated embedded defaults in memory, streams canonical staging serialization, writes identity/revision JSON from borrowed fields, avoids unnecessary summary-inspection allocations, and streams the compacted SQLite collection into the APKG. Existing identity information, runtime validation, database compaction, post-write inspection and atomic publication remain enabled. Regression coverage includes malformed inputs, custom fields, error behavior, canonical JSON and `serde_json/preserve_order`, historical GUID retention and actual Anki roundtrips.

## Final complete benchmark

Synthetic Basic 200 / 500 / 1K / 10K, one card per note, no media. Each cell has three warmups and ten fresh-process timings, followed by a separate three warmups and five RSS measurements. Times include startup, parsing, normal export and shutdown. Values below are medians in milliseconds. Each allocator run has its own interleaved genanki control; columns from different runs are not a paired allocator comparison.

| Notes/cards | Rust System | Same-run genanki | Rust mimalloc | Same-run genanki |
| ---: | ---: | ---: | ---: | ---: |
| 200 | 46.678 | 118.471 | 43.227 | 117.398 |
| 500 | 59.511 | 124.338 | 55.221 | 124.324 |
| 1,000 | 86.460 | 134.623 | 79.587 | 135.130 |
| 10,000 | 633.059 | 334.524 | 558.809 | 334.003 |

At 10K, System peak RSS is 147.719 MiB and optional mimalloc peak RSS is 114.938 MiB; genanki in the mimalloc comparison is 69.266 MiB. The 450 ms time target is unmet with either allocator; both meet the 180 MiB RSS target. Rust is faster on the three smaller cases and slower at 10K. Rust's compressed modern package is about 1,500 KiB versus genanki's default legacy package at about 8,004 KiB; format, validation, metadata and styling differ. This compares complete default APIs, not language-only speed.

[System report and IQR](system/report.md) · [mimalloc report and IQR](mimalloc/report.md)

![System allocator: all four cases](system/timing.svg)

![Optional mimalloc: all four cases](mimalloc/timing.svg)

## Last incremental round, same allocator

This comparison is specifically the frozen third-round worktree versus the final fourth-round implementation, both using mimalloc 0.1.52. It is **not** the improvement from the PR's Git base. Each scale has five before-first and five after-first timing blocks; RSS is measured separately. Earlier cross-session absolute scores are not used to calculate these reductions.

| Notes | Before, ms | After, ms | Time reduction | Before RSS, MiB | After RSS, MiB |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 200 | 42.203 | 40.706 | 3.5% | 16.016 | 15.203 |
| 500 | 53.577 | 53.221 | 0.7% | 21.953 | 20.188 |
| 1000 | 79.409 | 78.930 | 0.6% | 31.875 | 28.094 |
| 10000 | 572.711 | 550.193 | 3.9% | 151.188 | 114.984 |

The main incremental benefit is memory: 10K peak RSS falls 23.9%, while median time falls 3.9%. The 500/1K timing changes are below 1%; these descriptive medians do not establish statistical significance. All 168 paired APKGs are byte-identical to the corresponding frozen older output on these four Basic inputs.

[Paired records](paired/attempts.jsonl) · [Summary](paired/summary.json) · [Frozen baseline identity](before.json) · [Selected build identity](selected-build.json)

## Selection and remaining costs

The final round adopts owned canonical lowering, movement/restoration of GUID assignments, borrowed fresh identity JSON and direct collection streaming. Its individual System candidates and buffer alternatives retain all samples; 128 KiB buffering changed the 10K median by only 0.14% and 1 MiB was not faster, so the direct streaming path was retained. Single-component improvements cannot be added together.

[Selection decisions](selection-decisions.json) · [Code candidates](candidates/code-selection/summary.json) · [Buffer candidates](candidates/buffer-selection/summary.json)

Separate 10K mimalloc stage instrumentation measures candidate-package generation at about 261 ms, preparation at 90 ms and post-write inspection at 57 ms. Generation includes SQLite row filling (116 ms), compaction (54 ms) and compression/write (32 ms). Child timers overlap parent timers and cannot be added to them or divided by formal-run totals. The note-insert timer includes Rust parameter conversion and tag joining. Reading moved from the old compaction timer into the new streaming-compression timer; the common compaction/read/compression boundary is recorded separately.

System allocation counters record 2,906,627 to 2,386,598 cumulative requests (17.9% fewer) and 387.002 to 321.818 MiB cumulative requested bytes (16.8% fewer) at the adapter's final checkpoint. These are instrumented Rust allocation requests, including realloc and probe overhead, not RSS, allocator metadata, native C allocations or formal timing gains.

[Checked diagnostic values](implementation-diagnostics.json) · [Stage records](candidates/final-profile/attempts.jsonl) · [Allocation records](candidates/final-allocation/attempts.jsonl)

## Correctness, provenance and publication

The two complete runs and paired run passed 508 artifact checks and 24 selected Anki import/content/render checks. The final runtime source passed all eight recorded validation groups: crate quality, contract governance, user capabilities, native Anki roundtrips, benchmark behavior tests, adapter Clippy, packaged consumer and preserve-order regression.

[Verification summary](implementation-verification.json) · [Validation commands and outcomes](validation-results.json)

These measurements remain exploratory: one macOS ARM64 host, a repetitive synthetic corpus, an uncommitted runtime worktree during measurement and a recorded upstream-oracle patch. No Node/Python binding, media or Cloze performance claim is made, and no README advantage headline is introduced. The baseline identifiers refer to the frozen development worktree, not a clean release commit.

This directory is a curated publication of compact evidence. Copied measurement records retain their original bytes; generated SVGs only have line-ending whitespace trimmed. [publication.json](publication.json) maps each to the local frozen source, source SHA-256, published SHA-256 and any presentation transformation. [SHA256SUMS](SHA256SUMS) covers the published files. APKGs, temporary instrumentation scripts, duplicate source snapshots, executables and build logs stay local. Captured local paths and omitted log/artifact paths in raw records are provenance, not portable download links. The original collector/verifier/renderer identities remain captured rather than being relabelled after commit. Documentation and this publication were prepared after measurement; runtime code is unchanged. A new run against a committed revision is required for a release or promotional claim.

Regenerate each formal report offline with `benchmarks/.venv/bin/python benchmarks/bench.py report <this-directory>/system` (or `mimalloc`). Re-running the benchmarks uses the commands in [the suite README](../../README.md), on a fresh named run. Regenerating reports changes presentation hashes; preserve the original publication when auditing these measurements.
