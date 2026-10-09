"""Static exportable plot derived from the audited matrix."""
from pathlib import Path
import json
import os

W = Path(__file__).resolve().parent
os.environ.setdefault('MPLCONFIGDIR', str(W / 'matplotlib'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np

old = json.loads((W / 'historical-summary.json').read_text())
current = json.loads((W / 'summary.json').read_text())
old = {(c['profile'], c['notes']): c for c in old['cells']}
large = [c for c in current['cells'] if c['notes'] == 1000]
plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10, 'svg.fonttype': 'none'})
fig, axes = plt.subplots(1, 2, figsize=(13, 5.6), gridspec_kw={'width_ratios': [1.5, 1]})
y = np.arange(len(large))
for offset, label, color, source in [(-.18, 'Sep 21', '#8d969f', old),
                                     (.18, 'Oct 08 · 1199196', '#246c65', {(c['profile'], c['notes']): c for c in large})]:
    for ax, metric in zip(axes, ['time_ms', 'rss_mib']):
        cells = [source[c['profile'], 1000]['rust'][metric] for c in large]
        values = np.array([c['median'] for c in cells])
        errors = np.array([[c['median'] - c['q1'] for c in cells], [c['q3'] - c['median'] for c in cells]])
        ax.barh(y + offset, values, height=.31, color=color, label=label,
                xerr=errors if metric == 'time_ms' else None,
                error_kw={'elinewidth': 1, 'capsize': 2, 'ecolor': '#39414a'})
        for pos, value, cell in zip(y + offset, values, cells):
            ax.annotate(f'{value:.1f}', (max(value, cell['q3']) if metric == 'time_ms' else value, pos),
                        xytext=(5, 0), textcoords='offset points', va='center', fontsize=8)
for ax, title in zip(axes, ['Elapsed time (ms)', 'Independent peak RSS (MiB)']):
    ax.set_yticks(y, [c['english_label'] for c in large] if ax is axes[0] else [])
    ax.invert_yaxis()
    ax.set_title(title, loc='left', fontweight='bold')
    ax.spines[['top', 'right', 'left']].set_visible(False)
    ax.grid(axis='x', alpha=.15)
    ax.set_axisbelow(True)
    ax.set_xlim(0, ax.get_xlim()[1] * 1.13)
axes[0].legend(frameon=False, loc='lower right', fontsize=9)
fig.suptitle('1,000 notes · latest commit vs September 21', x=.16, y=.98,
             ha='left', fontweight='bold', fontsize=16)
fig.text(.16, .025, 'Median: 10 timings and 5 independent RSS samples per cell. Whiskers: Q1–Q3.\n'
                   'Separate sessions/API generations; background load/cache uncontrolled. No samples replaced.',
         color='#555555', fontsize=9)
fig.subplots_adjust(left=.16, right=.98, top=.87, bottom=.16, wspace=.1)
fig.savefig(W / 'comparison.svg')
fig.savefig(W / 'comparison.png', dpi=160)
plt.close(fig)
