#!/usr/bin/env python3
"""Generate README charts and screenshot pages from Anki-rendered card HTML.

Requires matplotlib (already pinned by benchmarks/requirements.lock).
Screenshot the generated pages at their declared width; see the asset README.
"""
from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import html
import json
import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "docs/assets/readme"
PREVIEW = ROOT / "target/readme/preview"
CSV = ROOT / "benchmarks/results/20260921-readme-genanki/comparison.csv"
PROFILES = [
    ("basic-mixed-text-v1", "Text only"),
    ("basic-image-unique-v2", "Unique images"),
    ("basic-audio-unique-v2", "Unique audio"),
    ("basic-mixed-unique-v2", "Mixed · unique media"),
    ("basic-mixed-shared-v2", "Mixed · shared media"),
]
PALETTES = {
    "light": dict(bg="#fbf9f5", paper="#ffffff", ink="#272d33", muted="#606974",
                  line="#deded9", accent="#a84725", other="#929ca6", code="#f0eee9"),
    "dark": dict(bg="#151a21", paper="#202731", ink="#edf0f3", muted="#b1bac6",
                 line="#39424e", accent="#efa77f", other="#758394", code="#202731"),
}
CSS = """
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--ink);
       font: 18px/1.55 -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
body.desktop { width:1000px; }
body.mobile { width:440px; }
.art { padding: 30px; }
.kicker { color: var(--muted); font-size: 14px; letter-spacing: 1.5px; }
.topline { display:flex; justify-content:space-between; gap:20px; margin-bottom:22px; }
.topline strong { color:var(--accent); font-weight:600; }
.hero { display:grid; grid-template-columns:1.55fr 1fr; gap:26px; align-items:stretch; }
.code { background:var(--code); border-radius:12px; padding:24px; }
pre { font:19px/1.8 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      white-space:pre-wrap; margin:16px 0 20px; }
.string { color:var(--accent); }
.output { border-top:1px solid var(--line); padding-top:14px; font-size:15px; }
.card-panel { min-width:0; background:var(--paper); border:1px solid var(--line);
              border-radius:12px; overflow:hidden; }
.card-label { padding:15px 20px; color:var(--muted); font-size:14px;
              border-bottom:1px solid var(--line); display:flex; justify-content:space-between; }
iframe { border:0; display:block; width:100%; background:var(--paper); }
.hero iframe { height:260px; }
.foot { margin-top:20px; font-size:14px; color:var(--muted); }
.gallery { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:18px; }
.gallery iframe { height:355px; }
.gallery .cloze-question { height:150px; }
.gallery .cloze-answer { height:220px; }
.divider { margin:0 20px; padding-top:7px; border-top:1px solid var(--line);
           color:var(--muted); font-size:12px; letter-spacing:1px; }
.mobile .art { padding:18px; }
.mobile .hero, .mobile .gallery { grid-template-columns:1fr; gap:14px; }
.mobile .topline { display:block; margin-bottom:18px; }
.mobile .topline strong { display:block; }
.mobile .code { padding:20px; }
.mobile pre { font-size:17px; }
.mobile .hero iframe { height:170px; }
.mobile .gallery iframe { height:310px; }
.mobile .gallery .basic-answer { height:145px; }
.mobile .gallery .cloze-question { height:110px; }
.mobile .gallery .cloze-answer { height:165px; }
.mobile .foot { font-size:13px; margin-top:14px; }
"""


def data_url(path: Path) -> str:
    mime = "image/svg+xml" if path.suffix == ".svg" else "audio/wav"
    return f"data:{mime};base64,{base64.b64encode(path.read_bytes()).decode()}"


def card_frame(card: dict, side: str, theme: str, media: Path, extra_class="") -> str:
    # Anki Desktop supplies its own replay button. This browser preview uses
    # native audio controls for the same imported sound file.
    content = card[side].replace("[sound:concert-a.wav]",
                                '<audio controls preload="metadata" src="concert-a.wav"></audio>')
    for filename in ("waveform.svg", "concert-a.wav"):
        content = content.replace(filename, data_url(media / filename))
    night = " nightMode" if theme == "dark" else ""
    # A neutral preview base supplies typography for stock types with no card CSS.
    # Anki-rendered content and exported note-type CSS are retained unchanged.
    colors = PALETTES[theme]
    base_css = (f"body {{ font:22px/1.4 Arial,sans-serif; text-align:center; background:{colors['paper']}; "
                f"color:{colors['ink']}; }} hr {{ border:0; border-top:1px solid {colors['line']}; "
                "margin:22px 0; }")
    source = (
        f"<!doctype html><html><head><meta charset='utf-8'><style>{base_css}{card['css']}"
        "body { margin:0; padding:18px; }</style></head>"
        f"<body class='card{night}'>{content}</body></html>"
    )
    return (f'<iframe class="{extra_class}" title="{html.escape(card["notetype"])} {side}" '
            f'srcdoc="{html.escape(source, quote=True)}"></iframe>')


def screenshot_pages(cards: list[dict], media: Path) -> None:
    basic = next(card for card in cards if card["fields"][:2] == ["hola", "hello"])
    cloze = next(card for card in cards if "{{c1::frequency}}" in card["fields"][0])
    custom = next(card for card in cards if card["notetype"] == "Ear Training")
    assert "[...]</span>" in cloze["question"] and ">frequency</span>" in cloze["answer"]
    code = (ROOT / "anki_forge/examples/target_api_basic.rs").read_text()
    excerpt = code.split('    let mut deck', 1)[1].split('    Ok(())', 1)[0]
    excerpt = 'let mut deck' + excerpt
    excerpt = '\n'.join(line[4:] if line.startswith('    ') else line for line in excerpt.rstrip().splitlines())
    excerpt = excerpt.replace('?.ensure_success()?;', '?\n    .ensure_success()?;')
    excerpt = html.escape(excerpt)
    for text in ("Spanish", "hola", "hello", "es:hola", "spanish.apkg"):
        token = f'&quot;{text}&quot;'
        excerpt = excerpt.replace(token, f'<span class="string">{token}</span>')
    for theme, colors in PALETTES.items():
        variables = ";".join(f"--{key}:{value}" for key, value in colors.items())
        hero = f"""
<div class="topline"><strong>FROM CODE TO CARDS</strong><span class="kicker">ONE APKG. READY TO IMPORT.</span></div>
<div class="hero">
  <div class="code"><div class="kicker">RUST · AUTHORING EXCERPT</div><pre>{excerpt}</pre><div class="output">Output → <strong>spanish.apkg</strong></div></div>
  <div class="card-panel"><div class="card-label"><span>Spanish · Basic</span><span>Answer</span></div>{card_frame(basic, 'answer', theme, media)}</div>
</div>
<div class="foot">Built-in Basic template · card HTML rendered by Anki</div>"""
        gallery = f"""
<div class="topline"><strong>THREE WAYS TO REMEMBER</strong><span class="kicker">REAL EXPORTED CARDS</span></div>
<div class="gallery">
  <div class="card-panel"><div class="card-label"><span>01 · Basic</span><span>Recall</span></div>{card_frame(basic, 'answer', theme, media, 'basic-answer')}</div>
  <div class="card-panel"><div class="card-label"><span>02 · Cloze</span><span>In context</span></div>{card_frame(cloze, 'question', theme, media, 'cloze-question')}<div class="divider">ANSWER</div>{card_frame(cloze, 'answer', theme, media, 'cloze-answer')}</div>
  <div class="card-panel"><div class="card-label"><span>03 · Custom + media</span></div>{card_frame(custom, 'answer', theme, media)}</div>
</div>
<div class="foot">Basic + Cloze: built-in templates · Ear training: example CSS, image + audio</div>"""
        for mobile in (False, True):
            suffix = f"{theme}{'-mobile' if mobile else ''}"
            for stem, body in (("code-to-card", hero), ("card-showcase", gallery)):
                document = (
                    '<!doctype html><html lang="en"><meta charset="utf-8">'
                    '<meta name="viewport" content="width=device-width,initial-scale=1">'
                    f'<title>{stem}-{suffix}</title><style>:root {{{variables}}}{CSS}</style>'
                    f'<body class="{"mobile" if mobile else "desktop"}"><main class="art">{body}</main></body></html>'
                )
                (PREVIEW / f"{stem}-{suffix}.html").write_text(document)


def benchmark_charts() -> None:
    with CSV.open() as source:
        rows = {row["profile"]: row for row in csv.DictReader(source) if row["notes"] == "1000"}
    plt.rcParams.update({"font.family": "DejaVu Sans", "svg.fonttype": "path",
                         "svg.hashsalt": "anki-forge-readme-20260921"})
    for theme, colors in PALETTES.items():
        for mobile in (False, True):
            fig, ax = plt.subplots(figsize=(4.4, 8.1) if mobile else (10, 6.2))
            fig.patch.set_facecolor(colors["bg"])
            ax.set_facecolor(colors["bg"])
            fig.subplots_adjust(left=.08 if mobile else .27, right=.97,
                                top=.79 if mobile else .74, bottom=.13 if mobile else .17)
            height = .22
            for i, (profile, label) in enumerate(PROFILES):
                row = rows[profile]
                if mobile:
                    ax.text(0, i - .31, label, color=colors["ink"], fontsize=14, weight="medium")
                for offset, implementation, color in ((0, "rust", colors["accent"]), (.26, "genanki", colors["other"])):
                    value = float(row[f"{implementation}_time_ms_median"])
                    ax.barh(i + offset, value, height=height, color=color, zorder=3)
                    ax.text(value + 7, i + offset, f"{value:.1f}", va="center", fontsize=13 if mobile else 16,
                            color=colors["ink"])
            max_value = max(float(row[f"{impl}_time_ms_median"]) for row in rows.values() for impl in ("rust", "genanki"))
            ax.set_xlim(0, math.ceil(max_value / 100) * 100 + 50)
            ax.set_ylim(4.65, -.57 if mobile else -.5)
            ax.set_xticks([0, 100, 200, 300, 400])
            ax.set_yticks([] if mobile else [i + .13 for i in range(5)],
                          [] if mobile else [label for _, label in PROFILES])
            ax.tick_params(length=0, labelsize=13 if mobile else 16, colors=colors["muted"], pad=12)
            ax.grid(axis="x", color=colors["line"], linewidth=.6, zorder=0)
            for spine in ax.spines.values():
                spine.set_visible(False)
            ax.set_xlabel("Median export time (ms) · lower is better", color=colors["muted"],
                          fontsize=12 if mobile else 15, labelpad=15)
            fig.text(.07 if mobile else .035, .94 if mobile else .92, "Less time exporting.", fontsize=21 if mobile else 25,
                     weight="bold", color=colors["ink"])
            fig.text(.07 if mobile else .035, .9 if mobile else .86, "1,000 notes per workload · 10 timings each", fontsize=12 if mobile else 16,
                     color=colors["muted"])
            fig.legend([plt.Rectangle((0, 0), 1, 1, color=colors["accent"]),
                        plt.Rectangle((0, 0), 1, 1, color=colors["other"])],
                       ["anki-forge · Rust Deck", "genanki"], loc="upper left", bbox_to_anchor=(.05 if mobile else .02, .865 if mobile else .83),
                       ncol=2, frameon=False, fontsize=11 if mobile else 16, labelcolor=colors["ink"], handlelength=1)
            suffix = f"{theme}{'-mobile' if mobile else ''}"
            svg_path = ASSETS / f"export-times-{suffix}.svg"
            fig.savefig(svg_path, metadata={"Date": None,
                        "Title": "Rust Deck and genanki: five 1,000-note export workloads",
                        "Description": "M1 Pro, 2026-09-21 source snapshot. Startup included. Default APKG formats differ."})
            plt.close(fig)
            # Matplotlib emits trailing spaces in multiline SVG path data.
            svg_path.write_text("\n".join(line.rstrip() for line in svg_path.read_text().splitlines()) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cards", type=Path, default=ASSETS / "source/cards.json")
    parser.add_argument("--package", type=Path, default=ASSETS / "showcase.apkg")
    args = parser.parse_args()
    ASSETS.mkdir(parents=True, exist_ok=True)
    PREVIEW.mkdir(parents=True, exist_ok=True)
    source = ASSETS / "source"
    source.mkdir(exist_ok=True)
    cards = json.loads(args.cards.read_text())["cards"]
    assert len(cards) == 3, "showcase must contain exactly three cards"
    for filename in ("waveform.svg", "concert-a.wav"):
        (source / filename).write_bytes((args.cards.parent / filename).read_bytes())
    package = args.package.read_bytes()
    (ASSETS / "showcase.apkg").write_bytes(package)
    (source / "cards.json").write_text(json.dumps({
        "package_sha256": hashlib.sha256(package).hexdigest(),
        "renderer": "Anki rslib via scripts/roundtrip_oracle/src/bin/readme_render.rs",
        "cards": cards,
    }, indent=2, ensure_ascii=False) + "\n")
    screenshot_pages(cards, source)
    benchmark_charts()
    print(f"Screenshot pages: {PREVIEW} (desktop: 1000px; mobile: 440px)")
    print(f"Charts and source evidence: {ASSETS}")


if __name__ == "__main__":
    main()
