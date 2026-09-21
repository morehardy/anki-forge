"""Draw the measured medians and IQRs from validated, frozen raw data."""
import json
import os
from pathlib import Path

WORK = Path(__file__).resolve().parent
os.environ.setdefault("MPLCONFIGDIR", str(WORK / "matplotlib-cache"))
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

result = json.loads((WORK / "summary.json").read_text())
profiles = list(dict.fromkeys(c["profile"] for c in result["cells"]))
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 10,
                     "svg.fonttype": "none", "svg.hashsalt": "round3-genanki-20260909"})
fig, axes = plt.subplots(3, 2, figsize=(11.4, 9.4), layout="constrained")
colors = {"rust": "#146B66", "genanki": "#BC6138"}
markers = {"rust": "o", "genanki": "s"}
for ax, profile in zip(axes.flat, profiles):
    cells = [c for c in result["cells"] if c["profile"] == profile]
    for adapter, offset in (("rust", -0.10), ("genanki", 0.10)):
        measurements = [c[adapter]["time_ms"] for c in cells]
        values = [s["median"] for s in measurements]
        errors = [[s["median"] - s["q1"] for s in measurements],
                  [s["q3"] - s["median"] for s in measurements]]
        ax.errorbar([i + offset for i in range(4)], values, yerr=errors,
                    fmt=markers[adapter], color=colors[adapter], capsize=4,
                    markersize=6, elinewidth=1.5, label=adapter)
    ax.set_title(cells[0]["english_label"], loc="left", fontweight="bold", pad=10)
    ax.set_xticks(range(4), [str(c["notes"]) for c in cells])
    ax.set_xlabel("Basic notes / cards")
    ax.set_ylabel("Elapsed time (ms)")
    ax.set_ylim(bottom=0)
    ax.set_xlim(-0.45, 3.45)
    ax.grid(axis="y", alpha=0.18)
    ax.spines[["top", "right"]].set_visible(False)
legend_ax = axes.flat[-1]
legend_ax.axis("off")
handles = [Line2D([], [], color=colors[a], marker=markers[a], linestyle="none",
                  markersize=7, label=label) for a, label in (
    ("rust", "anki-forge / native Rust / System allocator"),
    ("genanki", "genanki 0.13.1 / CPython 3.11.0 ARM64"))]
legend_ax.legend(handles=handles, loc="upper left", frameon=False, fontsize=10)
legend_ax.text(0.03, 0.71,
    "Medians with Q1-Q3 spread; 10 fresh processes per cell.\n"
    "Each time includes startup, parsing, export and exit.\n"
    "Panel scales differ; all start at zero.\n\n"
    "M1 Pro (10 cores), 32 GiB RAM, macOS 27.0, AC power.\n"
    "Current uncommitted optimization; one descriptive session.\n"
    "Same learning content; default APKG formats differ.\n\n"
    "All 840 exports validated; 40 Anki import/render checks.\n"
    "RSS was measured in 5 separate processes per cell.\n"
    "Raw data and full resource tables accompany this figure.",
    transform=legend_ax.transAxes, va="top", fontsize=9, linespacing=1.65, color="#3F444B")
fig.suptitle("Export time across 20 frozen workloads", fontsize=18, fontweight="bold")
fig.savefig(WORK / "comparison.svg", metadata={"Date": None,
    "Description": "Measured Rust and genanki median export times, with interquartile ranges, across five profiles and four deck sizes."})
fig.savefig(WORK / "comparison.png", dpi=160)
plt.close(fig)
