# Local acceptance record

Date: 2026-09-24. Host: macOS ARM64. Production implementation:
`399bc19ca816f44d441e1139dc0acb196bf79fc0`. Repository-only consumer isolation
followed in `289ccfc` and the accompanying final harness changes. No production
source changed after the implementation commit. Commands, source tree identity,
and local log checksums are recorded in [verification.json](verification.json).

| Gate | Observed result |
| --- | --- |
| Rust quality | Passed: formatting, default tests, all-target/all-feature Clippy with warnings denied, complete all-feature workspace suite, five doctests, rustdoc, embedded bundle reproducibility, exact payload, release metadata and dependency exception policy |
| Rust library | Default 121 tests and all-feature 353 tests passed; integration suites passed separately |
| Default public contracts | 21/21; positive and negative public-boundary probes each 3/3; no `unnameable_types`; warning-free default rustdoc |
| Full local CI | `verify-ci.sh --ci` passed: governance, workspace checks, 23/23 capability scenarios, rebuilt SDKs, examples and contract packaging |
| Rust distribution | Offline package and external consumer passed on Rust 1.92.0 and installed 1.98.1; actual APKG content, updates, assets, IO modes, budgets and lifetime inspected independently |
| Node | 17/17; isolated ESM/CJS installation, four README examples, TypeScript declarations, read-only installation, missing-runtime and version/protocol rejection passed |
| Python | 47/47; typing and examples passed; final sdist rebuilt offline outside the repository, wheel independently installed, all 47 tests passed again including five real fork owner classes; see [python-sdist.json](python-sdist.json) |
| Website | Documentation/source links passed; 16 complete executions and 22 actual APKG checks per run; build, typecheck, six script tests and 30-page site audit passed |
| Dependencies | Fresh RustSec database, cargo-deny 0.20.2: advisories/licenses/bans/sources passed; exact revision in [dependency-audit.txt](dependency-audit.txt) |

## Requirement coverage

| Requirement | Executed evidence |
| --- | --- |
| A1: snapshots and publication facts | Default `build_create`/`build_failures` consumers, report and artifact lifecycle suites; late fsync failures retain publication/durability facts and the original output |
| A2: errors and machine-readable causes | Public error propagation/downcast consumers; original read/write/sync I/O causes survive normalization, candidate failure and prior baseline observations; parallel-media regression passed |
| A3: policies and stage-specific budgets | `update_policy` and `update_evidence_errors`: blocked/allowed risks, warnings, hard errors, setter order, baseline/candidate limits and retry |
| A4: owned and borrowed keys | `schema_consumer_contract` and complete key-entry conversion checks, including rejected wrong key types |
| A5: must-use values | Positive controls plus exact `unused_must_use` compile failures and explicit drop behavior |
| A6: supported public surface | Default external crate, allowed root/domain paths, negative legacy/internal/lowering probes, tools callers, external packaged consumer |
| A7: asset closure | Native/bundle/media-reference consumers and independent ZIP/SQLite/media checks in packaged and documentation consumers |

## Concurrency regression discovered during final acceptance

Concurrent quality/CI runs exposed a shared executable race in the public
consumer harness: Cargo released its target lock before launching a binary that
another probe could replace. Unique process/consumer names now isolate runnable
artifacts while retaining the shared dependency cache. Two concurrent public
consumer suites each passed all 21 tests.

The same pattern was removed from website and media probes. Two concurrent
website runs each completed all 16 programs and 22 APKG checks; media lifecycle
4/4 and targeted Clippy passed. The updated measurement launcher completed its
8 MiB file/bytes smoke; [its record](media-measurement-harness-smoke.json) is a
launcher check, separate from the earlier 64/256 MiB measurement. Focused
independent review confirmed that all executable names and launch paths match.

## Limits of this evidence

- Other Tier 1 platforms and the current stable channel still require the retained
  hosted four-platform × two-toolchain matrix. Installed Rust 1.98.1 is named
  explicitly and is not evidence that the stable channel was refreshed.
- The Anki oracle covers 28 scenarios and 96 native imports. Its 11 asserted
  client limitations and untriggered same-second Always case remain documented
  in `scripts/roundtrip_oracle/RESULTS.md`; this is not an interactive all-client
  test.
- The 64/256 MiB measurements are single debug-build samples. Their source hashes
  identify the measured revision; later fork/cache ownership fixes are verified
  functionally, not claimed as a repeat of those performance measurements.
- No package was published, no release tag created, and no changes pushed.
