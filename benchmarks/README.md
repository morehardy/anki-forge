# Basic export benchmark

An independent, unpublished suite in this repository. Phase one compares the native Rust public `Deck` API with genanki 0.13.1 on **200 / 500 / 1,000 / 10,000 Basic notes**, one card per note. No anki-forge Node or Python binding is measured. See the [reviewed specification](../docs/superpowers/specs/2026-09-06-basic-export-benchmark-spec.md).

## Run

Requirements: the repository's Rust 1.92.0 toolchain, `uv`, a native CPython 3.11 interpreter, a C compiler with pthreads, and a local filesystem with sufficient free space. The initial local profile is **CPython 3.11.0 ARM64**; the exact patch is captured in every manifest. Another patch is a separate runtime profile, not a silently interchangeable baseline. Linux x86_64 and macOS ARM64 are supported. Python dependencies, including reporting tools, are hash locked; the private Rust adapter has its own Cargo.lock and workspace.

```sh
python3 benchmarks/bench.py prepare --python python3.11
benchmarks/.venv/bin/python -m unittest discover -s benchmarks/tests -v
benchmarks/.venv/bin/python benchmarks/bench.py smoke
benchmarks/.venv/bin/python benchmarks/bench.py run
```

`prepare` installs dependencies and builds the adapters/collector/inspector. It does not measure performance. Its Rust allocator defaults to `system`; an optional configuration is described below. `smoke` exports and checks 200 notes per implementation; it has no benchmark score. Full measurement runs sequentially and prints its output directory under `benchmarks/.work/runs/`. Use `--name <unique-name>` to name a run and `--budget-gib 8` to set its storage budget. Existing run names are refused. Do not compile, run tests, import Anki files or perform other heavy work during a full run.

For independent import/render evidence, the existing isolated upstream source checkout must be available at `docs/source/anki`, with `protoc` on PATH:

```sh
python3 benchmarks/bench.py prepare --python python3.11 --with-anki
benchmarks/.venv/bin/python benchmarks/bench.py run
```

This builds `scripts/roundtrip_oracle/src/bin/benchmark_oracle.rs`. The upstream Git revision, dirty state, lockfile and executable hash are recorded **before** measurement. Without that pinned executable the report remains unverified, and first timed artifacts are retained. Completing evidence later requires the same pinned executable and exact retained APKG bytes; rebuilding different historical artifacts does not validate the old run. The oracle is optional for CI smoke and stays outside the public crate.

```sh
benchmarks/.venv/bin/python benchmarks/bench.py oracle <run-directory>
benchmarks/.venv/bin/python benchmarks/bench.py report <run-directory>
benchmarks/.venv/bin/python benchmarks/bench.py cleanup <run-directory>
```

Reporting is offline and generates `report.md`, `timing.svg` and `summary.json`. It never edits the project README. Raw manifests, attempt records, verification results, phase events and Anki evidence are retained. `cleanup` removes only this run's verified outputs and private temporary files, preserving selected APKGs until their Anki evidence passes. Interrupted or failed attempts remain recorded. Copy a reviewed run's compact evidence/report files (excluding `artifacts/`) into `benchmarks/results/<run-id>/` to retain a reviewable snapshot.

## Optional Rust allocator

The public library does not install a global allocator. The private Rust benchmark adapter defaults to the system allocator and can explicitly enable **mimalloc 0.1.52**. This is a host-application configuration: reports, chart legends and summaries identify it separately from the default result. The adapter metadata records the allocator, its version and adapter features; the manifest captures the matching Cargo feature tree, executable hash, build command and build-environment overrides. The public crate keeps its default features in both configurations, without `internal-tools`.

Run each configuration to completion before preparing the next, because `prepare` replaces the same adapter executable. With the upstream oracle requirements above satisfied, the complete commands are:

```sh
python3 benchmarks/bench.py prepare --python python3.11 --with-anki --rust-allocator system
benchmarks/.venv/bin/python benchmarks/bench.py run --name basic-system

python3 benchmarks/bench.py prepare --python python3.11 --with-anki --rust-allocator mimalloc
benchmarks/.venv/bin/python benchmarks/bench.py run --name basic-mimalloc
```

Choose unused run names when repeating the comparison. Omitting `--with-anki` is supported, but leaves import/render evidence unverified unless an existing pinned oracle is available. Rebuild with `--rust-allocator system` to restore the default adapter.

An application using anki-forge can make the same opt-in choice in its **binary crate**, without changing the library:

```toml
[features]
default = []
mimalloc = ["dep:mimalloc"]

[dependencies]
# Keep the application's existing anki_forge dependency.
mimalloc = { version = "=0.1.52", optional = true, default-features = false }
```

```rust
#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

Enable it with `cargo build --release --features mimalloc`; a program can define only one global allocator. This replaces allocations routed through Rust's global allocator, including those in Rust dependencies. Native libraries that allocate directly through their own C allocators are not switched by this declaration. Python and Node host allocations are outside this choice, and their bindings are not measured here.

The [optimization implementation report](results/20260907-rust-export-pr/README.md) publishes the final complete System and mimalloc runs, the last incremental same-allocator comparison, selection evidence and correctness records. It preserves the slower 10K result and the exploratory provenance. The 3.9% last-round time reduction is measured against the frozen preceding worktree, not the full PR's Git base. Use each allocator's own complete run when comparing with genanki.

## Data and API boundary

`workload.py` owns the synthetic phrase pools and deterministic SHA-256 selection recipe. `workload-golden.json` freezes the four byte digests. Smaller cases are ordered prefixes of 10K: 50% English, 40% Chinese/English, 10% literal HTML-sensitive text. Fronts contain unique fixture markers. Lengths are 40–100 / 120–300 code points for Front / Back, with real UTF-8 distributions and repetition diagnostics in the manifest. This is a named synthetic workload, not a representative sample of user decks.

Both libraries accept HTML field strings. Each adapter escapes the shared plain-text input **inside** its measured process, then calls normal public APIs with default GUID/identity and export behavior. Rust retains all default validation, report and copy costs. genanki reuses its stock Basic model. Future adapters can use the same `basic-apkg-v1` input/output protocol and registry; adding a process-tree adapter requires a new validated memory metric and a fresh common baseline.

The outputs have matching learning content, not identical format or styling. Rust uses a modern zstd collection plus a legacy compatibility placeholder; genanki uses an uncompressed legacy collection. The reported size includes these default differences. No artifact is normalized or recompressed for scoring.

## Measurement and verification

- Three timing warmups per cell, ten interleaved timing rounds, three memory warmups per cell, then five interleaved memory rounds. Each size has exactly five Rust-first and five genanki-first timing blocks. Sizes are shuffled with a recorded seed. One exporter runs at a time.
- Time covers direct process launch through exit, including imports, parsing, conversion, export, writes and shutdown. A minimal native collector uses `CLOCK_MONOTONIC`, `posix_spawn` and blocking `wait4`; no polling tick or setup launcher enters the timing boundary. Logs are drained while retaining at most 64 KiB per stream. Descendant work left after exit invalidates the attempt.
- Memory uses each completed single process's OS high-water RSS, collected during separate invocations. Linux KiB and macOS bytes are normalized. The native collector prevents a large Python supervisor's inherited memory from inflating the child; behavioral tests check this and cumulative-child counterexamples.
- All artifact decoding, hashing, SQL checks, semantic inspection, Anki imports and cleanup occur after **both** measurement passes. Physical checks use original canonical SQLite rows before any import/upgrade can repair them. Modern semantic checks use the repository inspector; legacy schema 11 has a benchmark-local read-only reader because that inspector requires a modern `decks` table.
- First timed artifact per cell is imported by pinned upstream Anki into a fresh collection. Every field/card/deck association is checked, and fixed English/mixed/escaping examples are rendered. All other successful artifacts, including warmups, must pass full raw/semantic checks.
- Timeout is 120 seconds for both implementations. The affected pass stops that cell; unaffected cells continue. User cancellation stops further launches. Storage is estimated with headroom and monitored without scanning/deleting artifacts during measurement. Missing, invalid, timed-out and unsupported evidence never becomes zero or a survivor-only median.

Reports show all four scales, absolute median/IQR, signed differences, ratios, separate RSS and timed-artifact sizes. Ten samples are descriptive; no confidence intervals, significance claims, p95 or averaged cross-scale score are produced. Dirty or changed provenance stays an exploratory draft. A slower Rust result is valid evidence. This implementation emits no automatic promotional wording: a predeclared complete confirmation is required before any later 10K advantage headline can be considered.

## Isolation

The Rust adapter is not a root-workspace member and is `publish = false`; it uses only default public-crate features. Its optional mimalloc dependency stays in the private adapter workspace. Fixtures, environments, temporary tools/APKGs and raw working runs are ignored. `results/` is also ignored by default: deliberately add only reviewed compact evidence when publishing a snapshot, excluding profiling scripts, source copies, build logs and generated databases. No benchmark or Anki dependencies enter the public Rust distribution. CI runs behavior checks and the real 200-note smoke, with no performance thresholds and no Anki dependency.
