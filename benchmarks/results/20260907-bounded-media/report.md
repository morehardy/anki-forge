# Three-session export comparison

Measured package: base revision `ab7d261238619e99da9a6b8e5ffff8d952e6153b` plus the frozen uncommitted [source patch](source.patch). The exact [source hashes](source-snapshot.json) and patch hashes were recorded before measurement and remained unchanged across all three sessions.

Five synthetic Basic scenes × 100/200/500/1,000 notes. Each cell/exporter has 30 timing samples and 15 separate RSS samples; each session has 3 warmups before each pass. All 2,520 exports passed original SQLite/template/media checks; 120 first-timed packages passed pinned Anki import/content/render checks.

Host: Apple M1 Pro; 32 GiB RAM; 10 logical CPUs; macOS-27.0-arm64-arm-64bit. Rust 1.92.0 release/default features/system allocator; genanki 0.13.1 on native CPython 3.11.0.

Time covers fresh-process launch through exit, including imports, input parsing, media registration and default export checks. Media setup, output verification and Anki imports are outside each timed exporter. One exporter runs at a time, with adjacent interleaved controls; the same balanced seed/schedule is repeated in three sequential sessions. No controlled cold-cache or CPU-work claim is made.

PNG images are 64,152 bytes; WAV audio files are 32,044 bytes. Mixed scenes use 30% text, 40% images and 30% audio; shared media uses 49 distinct files. These fixtures do not measure Cloze, Image Occlusion, video, large individual media, bindings, long-lived processes or other hosts.

Anki verification uses the pinned upstream revision with the recorded local `tokio/io-util` build-feature patch in [plan.json](plan.json). This is local descriptive evidence with a patched oracle build, not an unmodified-upstream or cross-platform release benchmark. The patch and executable hashes are preserved; the measured package source and binaries did not change. No GUI or audible playback is exercised.

Power: Battery power. Native readings were recorded before and after all 2,520 exports and checked against the predeclared source; see each session's `power.jsonl`. Absolute times are not compared against earlier sessions with different power conditions.

## Timing

Values pool all equally sized sessions. Q1/Q3 use linear interpolation at `(n - 1) × p`; the original per-session runner reports retain its exclusive convention. IQR describes sample spread, not confidence. Savings = `100 × (1 - Rust median / genanki median)`. No cross-scene/size average, best-run selection or p95 is used.

| Scene | Notes | Rust ms [Q1, Q3] | genanki ms [Q1, Q3] | Time saved | genanki / Rust | ms saved |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Text only | 100 | 27.62 [26.37, 28.52] | 103.53 [102.51, 104.22] | 73.3% | 3.75× | 75.91 |
| Text only | 200 | 31.60 [30.81, 33.14] | 105.23 [104.55, 107.02] | 70.0% | 3.33× | 73.63 |
| Text only | 500 | 45.11 [44.39, 47.48] | 113.90 [111.23, 115.87] | 60.4% | 2.52× | 68.79 |
| Text only | 1000 | 68.32 [67.12, 70.09] | 121.37 [119.91, 123.33] | 43.7% | 1.78× | 53.04 |
| Unique images | 100 | 45.22 [44.07, 46.36] | 119.41 [118.29, 124.15] | 62.1% | 2.64× | 74.19 |
| Unique images | 200 | 67.16 [65.88, 68.20] | 138.53 [136.43, 140.05] | 51.5% | 2.06× | 71.37 |
| Unique images | 500 | 128.26 [126.80, 131.35] | 188.49 [187.24, 190.20] | 32.0% | 1.47× | 60.23 |
| Unique images | 1000 | 232.03 [229.68, 235.41] | 275.58 [273.76, 278.12] | 15.8% | 1.19× | 43.56 |
| Unique audio | 100 | 43.08 [41.26, 45.19] | 115.28 [114.02, 117.19] | 62.6% | 2.68× | 72.20 |
| Unique audio | 200 | 59.43 [57.64, 61.50] | 128.45 [127.67, 130.47] | 53.7% | 2.16× | 69.02 |
| Unique audio | 500 | 110.21 [107.96, 112.00] | 167.81 [165.96, 170.64] | 34.3% | 1.52× | 57.60 |
| Unique audio | 1000 | 195.64 [192.93, 202.17] | 231.93 [230.43, 234.00] | 15.6% | 1.19× | 36.29 |
| Mixed, unique media | 100 | 40.08 [38.87, 41.43] | 113.24 [112.51, 114.83] | 64.6% | 2.83× | 73.16 |
| Mixed, unique media | 200 | 54.32 [52.83, 55.44] | 125.78 [124.34, 127.07] | 56.8% | 2.32× | 71.45 |
| Mixed, unique media | 500 | 98.75 [96.46, 100.59] | 159.34 [158.22, 163.10] | 38.0% | 1.61× | 60.59 |
| Mixed, unique media | 1000 | 172.29 [170.59, 175.34] | 216.84 [215.24, 217.96] | 20.5% | 1.26× | 44.55 |
| Mixed, shared media | 100 | 37.16 [35.71, 38.75] | 111.66 [110.11, 114.15] | 66.7% | 3.00× | 74.50 |
| Mixed, shared media | 200 | 40.49 [39.80, 42.30] | 112.25 [111.43, 113.39] | 63.9% | 2.77× | 71.75 |
| Mixed, shared media | 500 | 57.27 [56.54, 58.77] | 118.63 [117.56, 121.44] | 51.7% | 2.07× | 61.35 |
| Mixed, shared media | 1000 | 81.51 [79.86, 83.31] | 129.51 [127.95, 131.23] | 37.1% | 1.59× | 48.00 |

![Time saved for every scene and size](time-heatmap.svg)

![Absolute time scaling with sample spread](time-scaling.svg)

## Across-session variation

Each entry below is Rust/genanki median milliseconds, followed by the same-session time saving. The predeclared diagnostic flags a range exceeding 5 percentage points. All three sessions stay in the aggregate regardless of the flag; it is not a significance test.

| Scene | Notes | Session 1 | Session 2 | Session 3 | Saving spread (pp) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Text only | 100 | 27.48/103.87 (73.5%) | 27.64/103.19 (73.2%) | 27.82/103.41 (73.1%) | 0.45 |
| Text only | 200 | 31.56/104.82 (69.9%) | 31.05/105.01 (70.4%) | 32.89/105.95 (69.0%) | 1.47 |
| Text only | 500 | 45.30/111.99 (59.5%) | 44.77/114.66 (61.0%) | 45.73/114.24 (60.0%) | 1.41 |
| Text only | 1000 | 67.12/120.21 (44.2%) | 69.11/121.71 (43.2%) | 68.63/122.40 (43.9%) | 0.95 |
| Unique images | 100 | 45.02/118.30 (61.9%) | 44.72/119.17 (62.5%) | 45.73/124.14 (63.2%) | 1.22 |
| Unique images | 200 | 67.18/136.47 (50.8%) | 67.43/137.63 (51.0%) | 66.73/140.07 (52.4%) | 1.59 |
| Unique images | 500 | 128.52/188.10 (31.7%) | 127.24/187.97 (32.3%) | 129.14/189.12 (31.7%) | 0.63 |
| Unique images | 1000 | 229.91/276.15 (16.7%) | 235.15/276.64 (15.0%) | 231.70/274.95 (15.7%) | 1.75 |
| Unique audio | 100 | 41.77/114.67 (63.6%) | 43.71/115.24 (62.1%) | 42.83/115.49 (62.9%) | 1.50 |
| Unique audio | 200 | 57.86/128.03 (54.8%) | 60.42/128.10 (52.8%) | 59.98/130.19 (53.9%) | 1.98 |
| Unique audio | 500 | 108.16/168.10 (35.7%) | 109.84/167.77 (34.5%) | 111.07/166.95 (33.5%) | 2.19 |
| Unique audio | 1000 | 194.39/231.63 (16.1%) | 195.84/231.95 (15.6%) | 198.16/232.25 (14.7%) | 1.40 |
| Mixed, unique media | 100 | 39.15/113.80 (65.6%) | 40.29/113.15 (64.4%) | 40.76/112.96 (63.9%) | 1.69 |
| Mixed, unique media | 200 | 54.70/125.06 (56.3%) | 53.79/126.11 (57.3%) | 54.26/126.50 (57.1%) | 1.08 |
| Mixed, unique media | 500 | 97.91/158.19 (38.1%) | 99.37/159.34 (37.6%) | 98.71/162.35 (39.2%) | 1.56 |
| Mixed, unique media | 1000 | 171.60/215.72 (20.5%) | 173.04/217.67 (20.5%) | 173.48/216.84 (20.0%) | 0.51 |
| Mixed, shared media | 100 | 36.12/111.87 (67.7%) | 37.48/111.74 (66.5%) | 37.95/111.66 (66.0%) | 1.70 |
| Mixed, shared media | 200 | 40.48/111.75 (63.8%) | 42.45/113.00 (62.4%) | 39.95/112.43 (64.5%) | 2.03 |
| Mixed, shared media | 500 | 57.39/118.48 (51.6%) | 57.46/119.41 (51.9%) | 57.14/118.88 (51.9%) | 0.38 |
| Mixed, shared media | 1000 | 81.51/129.00 (36.8%) | 80.11/128.71 (37.8%) | 82.07/132.43 (38.0%) | 1.21 |

## Memory and output size

RSS is the median [minimum, maximum] of 15 independent process high-water marks. APKG size uses only the 30 timed outputs. Rust writes a modern zstd collection plus a legacy compatibility placeholder; genanki writes a legacy stored collection. Size savings include those default format/compression choices. Matching learning content does not imply byte-identical packages or styles.

| Scene | Notes | Rust RSS MiB [min, max] | genanki RSS MiB [min, max] | Rust APKG KiB | genanki APKG KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Text only | 100 | 14.06 [14.00, 14.17] | 26.58 [26.44, 27.53] | 72.64 | 132.21 |
| Text only | 200 | 15.84 [15.80, 16.06] | 27.36 [26.94, 28.28] | 87.91 | 204.21 |
| Text only | 500 | 21.27 [21.19, 21.45] | 28.97 [28.55, 29.47] | 133.13 | 464.21 |
| Text only | 1000 | 30.19 [30.02, 30.27] | 31.47 [31.09, 32.19] | 204.17 | 848.21 |
| Unique images | 100 | 19.78 [19.45, 20.05] | 26.89 [26.42, 28.19] | 6349.31 | 6411.18 |
| Unique images | 200 | 22.17 [21.95, 22.44] | 27.98 [27.41, 28.59] | 12641.49 | 12762.47 |
| Unique images | 500 | 28.94 [28.53, 29.17] | 30.56 [30.17, 31.38] | 31517.22 | 31868.35 |
| Unique images | 1000 | 40.11 [39.75, 40.53] | 34.73 [34.42, 35.30] | 62972.33 | 63664.82 |
| Unique audio | 100 | 18.55 [18.28, 18.95] | 27.23 [26.56, 27.72] | 3184.97 | 3275.63 |
| Unique audio | 200 | 21.16 [20.61, 21.44] | 27.75 [27.58, 29.03] | 6331.77 | 6487.38 |
| Unique audio | 500 | 28.22 [27.78, 28.73] | 30.59 [30.30, 31.50] | 15754.95 | 16174.62 |
| Unique audio | 1000 | 39.75 [39.30, 40.38] | 35.12 [34.72, 35.72] | 31452.66 | 32269.35 |
| Mixed, unique media | 100 | 19.47 [18.81, 19.73] | 26.80 [26.59, 28.11] | 3518.16 | 3588.02 |
| Mixed, unique media | 200 | 21.52 [21.14, 21.89] | 28.05 [27.33, 28.70] | 6983.60 | 7111.97 |
| Mixed, unique media | 500 | 28.34 [27.72, 28.77] | 29.81 [29.55, 30.95] | 17373.83 | 17736.09 |
| Mixed, unique media | 1000 | 38.52 [38.31, 38.97] | 33.86 [33.56, 34.81] | 34684.56 | 35400.30 |
| Mixed, shared media | 100 | 19.16 [18.64, 19.53] | 26.98 [26.53, 27.95] | 2482.42 | 2552.47 |
| Mixed, shared media | 200 | 21.06 [20.52, 21.55] | 27.75 [27.16, 28.39] | 2498.15 | 2624.47 |
| Mixed, shared media | 500 | 26.34 [26.11, 26.59] | 29.20 [28.47, 30.31] | 2544.99 | 2892.47 |
| Mixed, shared media | 1000 | 35.39 [34.77, 36.12] | 32.20 [31.66, 32.77] | 2618.29 | 3296.47 |

![Memory and package size savings](resources.svg)

## Reproduce

See the [suite instructions](../../README.md) for locked preparation and measurement. [plan.json](plan.json) was saved before fixture generation or measurement. Original manifests, attempts, verified samples and compressed check records live in `round-1/`, `round-2/`, `round-3/`. Source paths inside these immutable records describe the measurement host; fixtures regenerate from the frozen repository recipe. APKG/media files and executables are excluded from this compact snapshot.

Regenerate this report offline from the archived evidence:

```sh
benchmarks/.venv/bin/python benchmarks/media_report.py benchmarks/results/20260907-bounded-media
```

[summary.json](summary.json) contains all statistics; [evidence-sha256.json](evidence-sha256.json) pins the original evidence bytes. The renderer rejects changed evidence, differing experiment identities, incomplete cells, failed attempts or missing Anki checks before calculating scores.
