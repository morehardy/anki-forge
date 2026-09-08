"""Update user-facing benchmark summaries only after all evidence is complete."""
from pathlib import Path
import json

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
name = '20260907-bounded-media'
OUT = REPO / 'benchmarks/results' / name
summary = json.loads((OUT / 'summary.json').read_text())
implementation = json.loads((OUT / 'implementation-summary.json').read_text())
assert summary['verified_exports'] == 2520 and summary['anki_import_checks'] == 120
cells = {r['profile']: r for r in summary['cells'] if r['size'] == 1000}
text = cells['basic-mixed-text-v1']
image = cells['basic-image-unique-v2']
spread = image['round_time_saved_range_pct']
rows = []
for profile, label in [('basic-mixed-text-v1', 'Text only'), ('basic-image-unique-v2', 'Unique images'),
                       ('basic-audio-unique-v2', 'Unique audio'), ('basic-mixed-unique-v2', 'Mixed, unique media'),
                       ('basic-mixed-shared-v2', 'Mixed, shared media')]:
    r = cells[profile]
    a, b = r['rust'], r['genanki']
    rows.append(f"| {label} | {a['rss_mib']['median']:.2f} / {b['rss_mib']['median']:.2f} | "
                f"{a['apkg_bytes']['median']/2**20:.2f} / {b['apkg_bytes']['median']/2**20:.2f} |")
extended = {r['case']: r for r in implementation['cells']}
large = extended['image-64x1m']
skewed = extended['image-skewed-128']
variable = sum(r['variable_across_rounds'] for r in summary['cells'])
variation_note = (f'\n{variable} cells exceeded the predeclared 5-percentage-point session-spread diagnostic; all sessions remain in the results.\n'
                  if variable else '')
section = f'''### 2.1 Export benchmarks

The native Rust `Deck` API is compared with genanki **0.13.1 / CPython 3.11.0**
on five synthetic Basic workloads at **100, 200, 500 and 1,000 notes**.
Three complete sessions provide **30 timings and 15 separate peak-RSS samples
per implementation/cell**, excluding warmups. Time includes process startup,
input parsing, media registration and default export checks.

![Export time saved versus genanki, with both absolute median times for every workload and size](benchmarks/results/{name}/time-heatmap.svg)

At 1,000 notes, text-only export uses **{text['time_saved_pct']:.1f}% less time** and unique images use
**{image['time_saved_pct']:.1f}% less time**: {image['rust']['time_ms']['median']:.1f} ms versus {image['genanki']['time_ms']['median']:.1f} ms. The three image sessions range
from {spread[0]:.2f}% to {spread[1]:.2f}% time saved. Memory also depends on the workload:

| Workload, 1,000 notes | Peak RSS MiB, Rust / genanki | APKG MiB, Rust / genanki |
| --- | ---: | ---: |
{chr(10).join(rows)}

Values are pooled medians; RSS is the median of independent process peaks.
Measured on **Apple M1 Pro, 32 GiB, macOS ARM64, battery power**, with Rust
**1.92.0 release, default features and system allocator**, on 2026-09-07.
The measured code is revision `ab7d261` plus a [frozen uncommitted patch](benchmarks/results/{name}/source.patch);
source and binary hashes remained unchanged across all three sessions. Native
power readings were checked before and after every export. PNG/WAV fixtures are
frozen; mixed workloads contain 30% text, 40% images and 30% audio, with 49 distinct
files in the shared case. File cache is uncontrolled. These local measurements
cover the native Rust API; Node/Python bindings are outside this comparison.
Default APKG formats differ: Rust uses modern zstd collections, while genanki
uses legacy stored collections.
{variation_note}
All **2,520 exports** passed content/media checks, and **120 packages** passed
Anki import/content/render checks. The pinned Anki checker includes a recorded
`tokio/io-util` build-feature patch. See the [full report and raw evidence](benchmarks/results/{name}/report.md)
for every cell's absolute values, IQR, session variation and reproduction steps.

The [implementation report](benchmarks/results/{name}/implementation.md) separately compares
the shared media buffer pool with the preceding version across **29 cases**.
In that paired test, 64 × 1 MiB images used **{large['time_saved_pct']:.1f}% less time** with
**{large['rss_change_mib']:.1f} MiB more RSS**; mixed file sizes used **{skewed['time_saved_pct']:.1f}% less time**
with **{skewed['rss_change_mib']:.1f} MiB more RSS**. These synthetic large-file results describe
the buffering change; the standard Rust/genanki matrix above uses smaller files.
The [preceding three-session evidence](benchmarks/results/20260907-streaming-followup/report.md) remains available.

<details>
<summary>Time scaling, resources and the media buffering change</summary>

Points show pooled median times; whiskers show Q1–Q3 sample spread, not confidence
intervals. Faint dots retain the three individual session medians.

![Export time scaling across five workloads, using identical linear axes](benchmarks/results/{name}/time-scaling.svg)

Positive resource savings mean Rust uses less; negative values mean it uses more.

![Peak RSS and APKG size savings across every workload and size](benchmarks/results/{name}/resources.svg)

The separate before/after comparison uses 7 interleaved timings and 5 independent
RSS samples per version/case, with unchanged inputs and byte-identical APKGs.

![Large-media export time and RSS before and after shared buffers](benchmarks/results/{name}/bounded-media.svg)

</details>

'''
readme = REPO / 'README.md'
source = readme.read_text()
start = source.index('### 2.1 Export benchmarks\n')
end = source.index('## 3. Project For Long-Term Decks\n', start)
readme.write_text(source[:start] + section + source[end:])

readme = REPO / 'benchmarks/README.md'
source = readme.read_text()
start = source.index('The [current README evidence]')
end = source.index('The aggregate pools equally sized sessions', start)
source = source[:start] + f'''The [current README evidence](results/{name}/report.md)
uses three sequential full media matrices at revision `ab7d261` plus a frozen
uncommitted source patch. Exact source/binary identities were recorded before
measurement and stayed unchanged. Every exporter/cell has 30 fresh-process
timings and 15 separate RSS samples, plus three warmups before each phase in
each session. All 2,520 exports passed artifact checks and 120 selected packages
passed Anki import/content/render checks. Native battery-power readings were
checked before and after every export. The Anki checker retains a recorded
`tokio/io-util` build-feature patch.

The [shared-buffer implementation report](results/{name}/implementation.md)
compares 29 cases with the preceding version, including large media, long fields
and 10,000 notes. It records the fixed 4.5 MiB encoded-buffer budget, time/RSS
tradeoffs, rejected identity-borrowing candidate, failure tests and source evidence.
The [preceding implementation and three sessions](results/20260907-streaming-followup/implementation.md),
[post-streaming bottleneck audit](results/20260907-post-streaming-audit/README.md)
and [earlier clean baseline](results/20260907-readme-comparison/report.md) remain unchanged.

''' + source[end:]
# Only reproduction commands move to the new snapshot; historical links above
# remain attached to their original measurements.
source = source.replace('benchmarks/media_report.py benchmarks/results/20260907-streaming-followup',
                        f'benchmarks/media_report.py benchmarks/results/{name}')
source = source.replace('cp benchmarks/results/20260907-streaming-followup/{run_three.py,guarded_media_bench.py}',
                        f'cp benchmarks/results/{name}/{{run_three.py,guarded_media_bench.py}}')
source = source.replace('plus separate signed memory/package-size savings. Numeric tables and all',
                        'plus separate signed memory/package-size savings. The implementation chart pairs large-file time and RSS before/after the buffer change. Numeric tables and all')
readme.write_text(source)
print('Updated both READMEs from verified summaries.')
