# Basic export benchmark

Run `20260907-basic-rust-genanki-optimized-04` · 2026-09-06T23:45:25.406116+00:00

**Rust allocator: system.** Adapter features: `none`. Default Rust allocator configuration.

**Exploratory draft.** dirty checkout: exploratory measurements; upstream oracle checkout dirty/unavailable. Not a release/README advantage claim.

Host: unavailable: Command '['sysctl', '-n', 'machdep.cpu.brand_string']' returned non-zero exit status 1.; macOS-27.0-arm64-arm-64bit; arm64. CPython 3.11.0, genanki 0.13.1; anki-forge 0.1.0, Rust release; public-crate features: default.

Synthetic `basic-mixed-text-v1` (seed 20260906), one Basic card per note, no media. Each field is escaped inside its adapter. The time includes process startup, imports, JSON input parsing, default identity/checks, export and shutdown. Filesystem caches are warmed and uncontrolled; writes are closed without an fsync guarantee.

Medians [Q1, Q3] are milliseconds from 10 attempts per cell; IQR is sample spread, not confidence. Percentiles use linear interpolation (Hyndman–Fan type 7). No significance or language-only throughput claim.

| Notes/cards | anki-forge Rust, ms [IQR] | genanki, ms [IQR] | genanki / Rust | genanki − Rust, ms | Evidence |
| ---: | ---: | ---: | ---: | ---: | --- |
| 200 | 46.678 [44.238, 48.488] | 118.471 [117.999, 119.494] | 2.5381× | +71.793 | draft; Rust verified; genanki verified |
| 500 | 59.511 [58.727, 61.168] | 124.338 [122.579, 125.636] | 2.0893× | +64.826 | draft; Rust verified; genanki verified |
| 1,000 | 86.460 [85.288, 88.472] | 134.623 [133.765, 135.255] | 1.5571× | +48.163 | draft; Rust verified; genanki verified |
| 10,000 | 633.059 [623.668, 635.010] | 334.524 [332.585, 335.505] | 0.5284× | -298.535 | draft; Rust verified; genanki verified |

A ratio below 1 means Rust took longer. A negative difference means additional time in Rust.

![All four sizes, linear time axis; medians and IQR](timing.svg)

## Separate memory and output measurements

RSS is the OS high-water mark of one completed exporter process (`single_process_peak_rss_os_v1`), from five separate memory attempts. It includes the runtime and is not a heap-allocation or whole-tree metric. Size uses only the ten timed APKGs. Values below are median [min, max].

| Notes/cards | Implementation | OS peak RSS, MiB (n=5) | APKG, KiB (n=10) | Memory status |
| ---: | --- | ---: | ---: | --- |
| 200 | rust | 15.70 [15.61, 15.78] | 87.91 [87.91, 87.91] | complete |
| 200 | genanki | 28.06 [27.36, 28.30] | 204.21 [204.21, 204.21] | complete |
| 500 | rust | 21.27 [21.22, 21.56] | 133.13 [133.13, 133.13] | complete |
| 500 | genanki | 29.69 [29.19, 30.02] | 464.21 [464.21, 464.21] | complete |
| 1,000 | rust | 30.34 [30.11, 30.53] | 204.17 [204.17, 204.17] | complete |
| 1,000 | genanki | 32.27 [32.06, 32.67] | 848.21 [848.21, 848.21] | complete |
| 10,000 | rust | 147.72 [146.92, 149.80] | 1500.37 [1500.37, 1500.37] | complete |
| 10,000 | genanki | 69.36 [69.11, 69.75] | 8004.21 [8004.21, 8004.21] | complete |

The preselected showcase size is **10,000 notes / 10,000 cards**, retained regardless of the winner. Rust writes modern `collection.anki21b` with nested zstd and a legacy compatibility placeholder; genanki writes legacy `collection.anki2` with ZIP_STORED. Stock CSS, metadata, IDs and default validation work differ. These are default-output comparisons; no package is converted or recompressed for scoring.

## Frozen synthetic data

| Notes | Input, KiB | English / mixed / escaping | Front code points, min–median–max | Back code points, min–median–max | zlib-9 compressed / raw |
| ---: | ---: | --- | --- | --- | ---: |
| 200 | 103.38 | 100 / 80 / 20 | 40–70.5–100 | 122–216–300 | 6.11% |
| 500 | 254.06 | 250 / 200 / 50 | 40–69–100 | 121–214–300 | 5.15% |
| 1,000 | 503.31 | 500 / 400 / 100 | 40–70–100 | 120–213–300 | 4.82% |
| 10,000 | 5037.22 | 5000 / 4000 / 1000 | 40–70–100 | 120–209–300 | 4.45% |

The phrase pools make this corpus repetitive. The compressor ratio is a characterization diagnostic, not a performance score or evidence that real decks have this compressibility. Exact UTF-8 byte distributions, compressor version and golden fixture hashes are in the manifest.

## Correctness and provenance

All successful attempts, including both sets of warmups, are checked after both measurement passes: physical rows, GUID/field multiplicity, card associations, templates and complete literal text. Modern semantics use the repository inspector; legacy schema-11 semantics use the benchmark-local read-only reader because the repository inspector requires a modern decks table. Independent verification requires the first timed artifact per cell to be imported into a fresh pinned Anki collection, every imported field/card to be checked and deterministic English, mixed and escaping examples to be rendered.

Anki checks passed for 8 of 8 cells. Missing or failed oracle evidence remains explicitly unverified.

Anki revision: `2d44d4d6bc486803f9236033ad840df203c87036`. Artifact and executable SHA-256 identities bind the evidence. Successful artifacts are cleaned only after their required checks; selected artifacts awaiting Anki are retained.

No advantage headline is generated: this run has no predeclared full confirmation. The complete matrix is retained even where Rust is slower. No outliers are removed or unsuccessful attempts retried.

[Manifest and frozen inputs](manifest.json) · [Every attempt](attempts.jsonl) · [Physical/semantic checks](verification.json) · [Anki evidence](anki.json) · [Phase events](events.jsonl) · [Machine-readable summary](summary.json) · [Retention](retention.json) · [Report revision and renderer identity](render-provenance.json)

Regenerate offline: `benchmarks/.venv/bin/python benchmarks/bench.py report <run-directory>`.
