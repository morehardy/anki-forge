"""Render the validated single-session results; no exporter is invoked here."""
import datetime
import json
import os
from pathlib import Path
from zoneinfo import ZoneInfo

WORK = Path(__file__).resolve().parent
os.environ.setdefault('MPLCONFIGDIR', str(WORK / 'matplotlib-cache'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap, TwoSlopeNorm

result = json.loads((WORK / 'summary.json').read_text())
checks = json.loads((WORK / 'verification-summary.json').read_text())
manifest = json.loads((WORK / 'run-manifest.json').read_text())
assert checks['status'] == 'passed' and checks['attempts'] == 840 and checks['anki_checks'] == 40
cells = result['cells']
profiles = list(dict.fromkeys(c['profile'] for c in cells))
sizes = [100, 200, 500, 1000]
lookup = {(c['profile'], c['notes']): c for c in cells}
assert len(lookup) == 20
labels = [lookup[(p, 100)]['english_label'] for p in profiles]
start = datetime.datetime.fromisoformat(result['started_utc'])
end = datetime.datetime.fromisoformat(result['completed_utc'])
date = start.astimezone(ZoneInfo('Asia/Shanghai')).strftime('%Y-%m-%d')
short_commit = result['source_commit'][:7]
plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                     'svg.fonttype': 'none', 'svg.hashsalt': result['run_id']})
cmap = LinearSegmentedColormap.from_list('savings', ['#c06a51', '#f8f8f3', '#287a6b'])
norm = TwoSlopeNorm(vmin=-100, vcenter=0, vmax=100)

def save(fig, stem):
    fig.savefig(WORK / f'{stem}.svg', metadata={'Date': None})
    plt.close(fig)

fig = plt.figure(figsize=(11.8, 6.6), facecolor='white')
ax = fig.add_axes([.23, .27, .73, .56])
values = [[lookup[(p, n)]['time_saved_pct'] for n in sizes] for p in profiles]
im = ax.imshow(values, cmap=cmap, norm=norm, aspect='auto')
ax.set_xticks(range(4), [f'{n:,} notes' for n in sizes])
ax.xaxis.tick_top()
ax.set_yticks(range(5), labels)
ax.tick_params(length=0, pad=10)
for i, p in enumerate(profiles):
    for j, n in enumerate(sizes):
        c = lookup[(p, n)]
        ax.text(j, i-.10, f"{c['time_saved_pct']:.1f}%", ha='center', va='center', fontsize=17, weight='bold', color='#132d29')
        ax.text(j, i+.21, f"{c['rust']['time_ms']['median']:.1f} / {c['genanki']['time_ms']['median']:.1f} ms", ha='center', va='center', fontsize=10, color='#132d29')
ax.set_xticks([x-.5 for x in range(5)], minor=True)
ax.set_yticks([x-.5 for x in range(6)], minor=True)
ax.grid(which='minor', color='white', linewidth=3)
ax.tick_params(which='minor', length=0)
for spine in ax.spines.values():
    spine.set_visible(False)
fig.text(.03, .96, 'anki-forge vs genanki: export time', fontsize=20, weight='bold', va='top')
fig.text(.03, .895, f'{date} · Each cell: time saved, then Rust / genanki median milliseconds', fontsize=11, color='#4d5955')
cax = fig.add_axes([.32, .19, .55, .026])
fig.colorbar(im, cax=cax, orientation='horizontal', ticks=[-100, -50, 0, 50, 100]).set_label('Positive = Rust takes less time (%)', fontsize=9)
fig.text(.03, .11, 'Rust 1.92.0 release / System allocator · genanki 0.13.1 / CPython 3.11.0 ARM64', fontsize=9, color='#4d5955')
fig.text(.03, .075, f'M1 Pro · 32 GiB · macOS 27.0 · AC power · current working tree at {short_commit} + recorded patch', fontsize=9, color='#4d5955')
fig.text(.03, .035, 'One session; 10 timings per implementation/cell. Startup through exit. Default APKG formats differ. 840 exports / 40 Anki checks passed.', fontsize=9, color='#4d5955')
save(fig, 'time-heatmap')

fig, axes = plt.subplots(3, 2, figsize=(11.6, 9), layout='constrained')
colors = {'rust': '#287a6b', 'genanki': '#b96748'}
max_time = max(c[a]['time_ms']['q3'] for c in cells for a in colors) * 1.12
for ax, p in zip(axes.flat, profiles):
    group = [lookup[(p, n)] for n in sizes]
    for adapter, offset in [('rust', -8), ('genanki', 8)]:
        data = [c[adapter]['time_ms'] for c in group]
        medians = [d['median'] for d in data]
        ax.errorbar([n+offset for n in sizes], medians,
                    yerr=[[d['median']-d['q1'] for d in data], [d['q3']-d['median'] for d in data]],
                    fmt='o' if adapter == 'rust' else 's', color=colors[adapter], capsize=4, label=adapter)
    ax.set_title(group[0]['english_label'], loc='left', weight='bold')
    ax.set_xticks(sizes, [str(n) for n in sizes])
    ax.set_xlim(50, 1060)
    ax.set_xlabel('Basic notes / cards')
    ax.set_ylabel('Elapsed time (ms)')
    ax.set_ylim(0, max_time)
    ax.grid(axis='y', alpha=.18)
    ax.spines[['top', 'right']].set_visible(False)
axes.flat[-1].axis('off')
handles, names = axes.flat[0].get_legend_handles_labels()
axes.flat[-1].legend(handles, ['anki-forge / Rust 1.92 release / System', 'genanki 0.13.1 / CPython 3.11.0'], loc='upper left', frameon=False)
axes.flat[-1].text(.02, .68, 'Medians with Q1–Q3 sample spread.\n10 fresh processes per implementation/cell.\nAll panels share linear axes; time starts at zero.\n\nTime includes process startup through exit.\nOne session on M1 Pro / macOS / AC power.\nRSS uses 5 separate samples per cell.\nDefault APKG formats differ.', va='top', transform=axes.flat[-1].transAxes, linespacing=1.7, fontsize=10)
fig.suptitle(f'Export time across 20 workloads · {date}', fontsize=18, weight='bold')
save(fig, 'time-scaling')

fig, axes = plt.subplots(1, 2, figsize=(14, 6.5))
fig.subplots_adjust(left=.16, right=.97, bottom=.20, top=.79, wspace=.12)
for ax, metric, title in zip(axes, ['rss_mib', 'apkg_bytes'], ['Peak RSS (MiB)', 'APKG size (MiB)']):
    data = [[lookup[(p, n)][metric+'_saved_pct'] for n in sizes] for p in profiles]
    im = ax.imshow(data, cmap=cmap, norm=norm, aspect='auto')
    ax.set_title(title, weight='bold', pad=35)
    ax.set_xticks(range(4), [str(n) for n in sizes])
    ax.xaxis.tick_top()
    ax.set_yticks(range(5), labels if metric == 'rss_mib' else ['']*5)
    ax.tick_params(length=0, pad=10)
    for i, p in enumerate(profiles):
        for j, n in enumerate(sizes):
            c = lookup[(p, n)]
            divisor = 1 if metric == 'rss_mib' else 2**20
            text = f"{c[metric+'_saved_pct']:.1f}%\n{c['rust'][metric]['median']/divisor:.2f} / {c['genanki'][metric]['median']/divisor:.2f}"
            ax.text(j, i, text, ha='center', va='center', color='#132d29', fontsize=9, linespacing=1.5)
    for spine in ax.spines.values():
        spine.set_visible(False)
fig.suptitle(f'Resources · {date} · each cell shows savings, then Rust / genanki', fontsize=17, weight='bold')
cax = fig.add_axes([.35, .115, .45, .025])
fig.colorbar(im, cax=cax, orientation='horizontal', ticks=[-100, -50, 0, 50, 100]).set_label('Positive = Rust uses less; negative = Rust uses more (%)', fontsize=9)
fig.text(.16, .025, 'Median of 5 independent OS peak-RSS samples; package size from 10 timed exports. Package formats and compression differ.', fontsize=10)
save(fig, 'resources')

host = result['host']
period = f"{start.astimezone(ZoneInfo('Asia/Shanghai')):%Y-%m-%d %H:%M:%S}–{end.astimezone(ZoneInfo('Asia/Shanghai')):%H:%M:%S} Asia/Shanghai"
lines = ['# Current Rust / genanki export comparison', '', f'Measured **{period}** in one complete session. This report uses the current working tree, including uncommitted Rust optimizations.', '',
         f"The measured base commit is `{result['source_commit']}` plus the frozen [source patch](source.patch). [Exact source and executable hashes](source-snapshot.json) were recorded before measurement and checked again afterward. These results describe this source snapshot, not a published release.", '',
         '## Results at 1,000 notes', '',
         '| Workload | Rust ms | genanki ms | Time saved | Rust / genanki peak RSS MiB |',
         '| --- | ---: | ---: | ---: | ---: |']
for c in cells:
    if c['notes'] == 1000:
        lines.append(f"| {c['english_label']} | {c['rust']['time_ms']['median']:.3f} | {c['genanki']['time_ms']['median']:.3f} | {c['time_saved_pct']:.1f}% | {c['rust']['rss_mib']['median']:.2f} / {c['genanki']['rss_mib']['median']:.2f} |")
lines += ['', '![Export time saved and absolute medians](time-heatmap.svg)', '', '## Method', '',
          f"- Host: {host['cpu']}, {host['physical_cores']} physical / {host['logical_cores']} logical cores, {host['ram_bytes']/2**30:.0f} GiB RAM, {host['system']} ({host['system_build']}), native ARM64, AC power.",
          '- Rust: 1.92.0, release build, public `Deck` API, default product features, System allocator. No batch-only or benchmark-only product API.',
          '- Comparator: genanki 0.13.1 on native CPython 3.11.0; dependencies use the repository hash-locked environment. Both implementations were measured again in this session.',
          '- Matrix: five synthetic workloads × 100 / 200 / 500 / 1,000 Basic notes, one card per note. Text uses v1 fixtures; PNG/WAV media uses the frozen portable v2 recipe.',
          '- Each implementation/cell: 3 timing warmups + 10 timing samples, then 3 RSS warmups + 5 independent RSS samples. Total: 400 timings, 200 RSS samples, 240 warmups.',
          '- Adjacent Rust/genanki pairs alternate order; each timing cell has 5 Rust-first and 5 genanki-first pairs. Scene order is shuffled with seed 20260907. No outlier removal or selective retries.',
          '- Timing spans native process launch through exit, including imports, input parsing, escaping, authoring, media registration, default export checks and file writes. Compilation, external validation and Anki imports are outside the timed interval.',
          '- Verification and cleanup occur after each adjacent exporter pair. No other compilation, tests or plotting from this task ran during measurement. Filesystem cache and unrelated desktop background work are uncontrolled.',
          '- Medians and Q1/Q3 use Hyndman–Fan type 7 interpolation. IQR shows sample spread, not confidence intervals. RSS is the median of five OS process high-water marks, not mean memory or total process-tree memory.', '',
          '## Complete timing results', '',
          '| Workload | Notes | Rust median [Q1, Q3] ms | genanki median [Q1, Q3] ms | Time saved | genanki / Rust |',
          '| --- | ---: | ---: | ---: | ---: | ---: |']
for c in cells:
    r, g = c['rust']['time_ms'], c['genanki']['time_ms']
    lines.append(f"| {c['english_label']} | {c['notes']} | {r['median']:.3f} [{r['q1']:.3f}, {r['q3']:.3f}] | {g['median']:.3f} [{g['q1']:.3f}, {g['q3']:.3f}] | {c['time_saved_pct']:.1f}% | {c['genanki_over_rust']:.2f}× |")
lines += ['', '![Medians and interquartile ranges on shared axes](time-scaling.svg)', '', '## Memory and package size', '',
          '| Workload | Notes | Rust RSS MiB, median [min, max] | genanki RSS MiB, median [min, max] | Rust / genanki APKG MiB |',
          '| --- | ---: | ---: | ---: | ---: |']
for c in cells:
    r, g = c['rust']['rss_mib'], c['genanki']['rss_mib']
    lines.append(f"| {c['english_label']} | {c['notes']} | {r['median']:.2f} [{r['min']:.2f}, {r['max']:.2f}] | {g['median']:.2f} [{g['min']:.2f}, {g['max']:.2f}] | {c['rust']['apkg_bytes']['median']/2**20:.3f} / {c['genanki']['apkg_bytes']['median']/2**20:.3f} |")
lines += ['', '![Separate peak memory and package-size savings](resources.svg)', '',
          'Rust uses a modern zstd-compressed collection and a legacy compatibility placeholder; genanki uses a legacy stored collection. Learning content is checked for equivalence, but output formats, styling, identity metadata and compression differ. Package-size savings include those default differences.', '',
          '## Verification and limits', '',
          'All **840 exports** passed original SQLite row/field/template checks and exact media name/content/reference checks. The first timed artifact for each implementation/cell passed the pinned Anki import, field and representative-render checks: **40/40**. All warmups are retained in the evidence. No GUI interaction or audible playback is tested.', '',
          'The independently pinned Anki oracle was reused after verifying its executable, six source/lock files, upstream revision and local patch against the earlier archived evidence. The oracle uses upstream `2d44d4d6bc486803f9236033ad840df203c87036` with the recorded `tokio/io-util` build-feature patch. See [oracle provenance](oracle-reuse-check.json) and [build records](prepared-builds.json). The Rust exporter, inspector and native collector were prepared before this run.', '',
          'The benchmark harness passed 43 tests and the separate 200-note smoke passed both exporters before measurement. Source, executable, Python package, fixture and media hashes stayed unchanged; every recorded export began and ended on AC power. The raw power and host-state records remain archived.', '',
          f"The 1-minute system load was {manifest['host_before']['load'][0]:.2f} before and {manifest['host_after']['load'][0]:.2f} after the run. Thermal status queries were unavailable; their raw errors are preserved in [the run manifest](run-manifest.json). Battery remained charged at 100% on AC power, with low-power mode disabled at both endpoints. This is a desktop-session comparison, not a controlled idle machine or cross-platform performance claim.", '',
          'One session provides descriptive evidence for this machine and these workloads. Startup costs are included, so these ratios do not describe only the writer or a long-lived process. Memory savings depend on workload and size. No samples from earlier dates or implementations are pooled into these results.', '',
          'Machine-readable results: [JSON](summary.json), [CSV](comparison.csv), and [verification summary](verification-summary.json). See the [evidence index and reproduction instructions](README.md).', '']
(WORK / 'report.md').write_text('\n'.join(lines))
print('Rendered three figures and the complete report from validated data.')
