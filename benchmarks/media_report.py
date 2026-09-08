"""Offline three-session media report; reads compact evidence, never runs exporters."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import tarfile

from report import stats

SIZES = (100, 200, 500, 1000)
PROFILES = (
    ('basic-mixed-text-v1', 'Text only'),
    ('basic-image-unique-v2', 'Unique images'),
    ('basic-audio-unique-v2', 'Unique audio'),
    ('basic-mixed-unique-v2', 'Mixed, unique media'),
    ('basic-mixed-shared-v2', 'Mixed, shared media'),
)
ADAPTERS = ('rust', 'genanki')
ROLES = {'timing_warmup': 3, 'timing': 10, 'rss_warmup': 3, 'rss': 5}


def read_json(path):
    return json.loads(path.read_text())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_source(root, plan, manifest):
    require(manifest['source_commit'] == plan['source_commit'], 'package source revision changed')
    frozen = plan.get('source_snapshot')
    if frozen is None:
        require(not manifest['git_status'] and not plan.get('git_status'),
                'package source must match the clean predeclared revision')
        return
    for name in ('source-snapshot.json', 'source.patch'):
        require(hashlib.sha256((root/name).read_bytes()).hexdigest() == frozen[name],
                f'frozen source changed: {name}')
    snapshot = read_json(root/'source-snapshot.json')
    require(snapshot['source_commit'] == plan['source_commit']
            and snapshot['git_status'] == manifest['git_status'] == plan['git_status']
            and snapshot['source_files'] == manifest['identity_before']['source_files'],
            'package source must match the predeclared worktree snapshot')


def validate_power(records, attempts, expected):
    require(Counter(r['id'] for r in records) == Counter(r['id'] for r in attempts),
            'missing or duplicate per-export power records')
    require(all(r.get(side, '').splitlines()[:1] == [expected]
                for r in records for side in ('before', 'after')),
            'native power source changed during measurement')


def aggregate(rounds):
    """Equal-size sessions: pool samples, retain every same-session comparison."""
    require(len(rounds) == 3, 'expected exactly three predeclared sessions')
    expected = Counter({(p, n, a, role): count for p, _ in PROFILES for n in SIZES
                        for a in ADAPTERS for role, count in ROLES.items()})
    for rows in rounds:
        require(Counter((r['profile'], r['size'], r['adapter'], r['role']) for r in rows) == expected,
                'incomplete or duplicate session cells')
        require(len({r['id'] for r in rows}) == len(rows), 'duplicate attempt IDs')
        require(all(r['status'] == 'success' and r['measurement']['exit_code'] == 0
                    and not r['measurement']['signal'] and not r['measurement']['leftover_descendants']
                    for r in rows), 'failed attempt: no survivor-only statistics')
    cells = []
    for profile, label in PROFILES:
        for size in SIZES:
            cell = {'profile': profile, 'label': label, 'size': size, 'rounds': []}
            selections = [[r for r in rows if (r['profile'], r['size']) == (profile, size)]
                          for rows in rounds]
            for adapter in ADAPTERS:
                rows = [r for selection in selections for r in selection if r['adapter'] == adapter]
                cell[adapter] = {
                    'time_ms': stats([r['measurement']['elapsed_ns']/1e6 for r in rows if r['role'] == 'timing']),
                    'rss_mib': stats([r['measurement']['peak_rss_bytes']/2**20 for r in rows if r['role'] == 'rss']),
                    'apkg_bytes': stats([r['artifact_bytes'] for r in rows if r['role'] == 'timing']),
                }
            for index, selection in enumerate(selections, 1):
                medians = {a: stats([r['measurement']['elapsed_ns']/1e6 for r in selection
                                    if r['adapter'] == a and r['role'] == 'timing'])['median'] for a in ADAPTERS}
                cell['rounds'].append({'round': index, **medians, 'time_saved_pct': 100*(1-medians['rust']/medians['genanki'])})
            rust, genanki = cell['rust'], cell['genanki']
            cell['time_saved_pct'] = 100*(1-rust['time_ms']['median']/genanki['time_ms']['median'])
            cell['genanki_over_rust'] = genanki['time_ms']['median']/rust['time_ms']['median']
            cell['time_saved_ms'] = genanki['time_ms']['median']-rust['time_ms']['median']
            for metric in ('rss_mib', 'apkg_bytes'):
                cell[metric+'_saved_pct'] = 100*(1-rust[metric]['median']/genanki[metric]['median'])
            reductions = [r['time_saved_pct'] for r in cell['rounds']]
            cell['round_time_saved_range_pct'] = [min(reductions), max(reductions)]
            cell['round_spread_pp'] = max(reductions)-min(reductions)
            cell['variable_across_rounds'] = cell['round_spread_pp'] > 5
            cells.append(cell)
    return cells


def load_evidence(root):
    for name, digest in read_json(root/'evidence-sha256.json').items():
        require(hashlib.sha256((root/name).read_bytes()).hexdigest() == digest, f'evidence changed: {name}')
    plan = read_json(root/'plan.json')
    require(read_json(root/'experiment.json')['status'] == 'completed', 'experiment incomplete')
    require(plan['round_count'] == 3 and len(set(plan['rounds'])) == 3, 'wrong predeclared sessions')
    manifests, rounds = [], []
    for index, name in enumerate(plan['rounds'], 1):
        folder = root/f'round-{index}'
        manifest = read_json(folder/'manifest.json')
        require(manifest['status'] == 'completed' and manifest['identity_unchanged']
                and manifest['oracle_required'], f'incomplete or unverified round {index}')
        validate_source(root, plan, manifest)
        require(list(manifest['sizes']) == list(SIZES) and manifest['timing_repeats'] == 10
                and manifest['rss_repeats'] == 5, 'wrong measurement schedule')
        require(manifest['rust_configuration']['allocator'] == 'system'
                and manifest['adapter_metadata']['rust']['media_registration'] == 'individual',
                'this comparison requires the default allocator and individual media API')
        rows = [json.loads(line) for line in (folder/'verified.jsonl').read_text().splitlines()]
        attempts = [json.loads(line) for line in (folder/'attempts.jsonl').read_text().splitlines()]
        if plan.get('expected_power_source'):
            power = [json.loads(line) for line in (folder/'power.jsonl').read_text().splitlines()]
            validate_power(power, attempts, plan['expected_power_source'])
        require(len(attempts) == len(rows) and len({r['id'] for r in attempts}) == len(rows), 'unverified attempts')
        by_id = {r['id']: r for r in attempts}
        with tarfile.open(folder/'verification.tar.gz') as archive:
            for row in rows:
                original = by_id[row['id']]
                require(original['status'] == 'success' and original['measurement'] == row['measurement'], 'attempt mismatch')
                check = json.loads(archive.extractfile(row['id']+'/verification.json').read())
                require(check['status'] == 'passed' and check['artifact_sha256'] == row['artifact_sha256']
                        and check['artifact_bytes'] == row['artifact_bytes'], 'artifact evidence mismatch')
                if row['role'] == 'timing' and row['repeat'] == 0:
                    raw = archive.extractfile(row['id']+'/anki.json').read()
                    require(hashlib.sha256(raw).hexdigest() == row['oracle_sha256']
                            and json.loads(raw)['status'] == 'passed', 'Anki evidence mismatch')
        if manifests:
            require(manifests[-1]['completed_utc'] < manifest['created_utc'], 'duplicate or overlapping sessions')
            for key in ('source_commit', 'identity_before', 'adapter_metadata', 'rust_configuration', 'rust_feature_tree',
                        'rustc', 'platform', 'machine', 'inputs', 'workloads', 'seed', 'fixture_protocol'):
                require(manifest[key] == manifests[0][key], f'sessions differ: {key}')
        manifests.append(manifest)
        rounds.append(rows)
    return plan, manifests, aggregate(rounds)


def plots(root, cells, plan, manifest):
    os.environ.setdefault('MPLCONFIGDIR', str(Path(__file__).resolve().parent/'.work/matplotlib'))
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from matplotlib.colors import LinearSegmentedColormap, Normalize, TwoSlopeNorm
    import numpy as np
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10, 'svg.fonttype': 'none',
                         'svg.hashsalt': 'readme-media-three-sessions-v1', 'text.color': '#172b35',
                         'figure.facecolor': 'white', 'axes.facecolor': 'white'})
    metadata = manifest['adapter_metadata']
    common = (f"Native Rust, {manifest['rust_configuration']['allocator']} allocator · "
              f"genanki {metadata['genanki']['genanki']} / CPython {metadata['genanki']['python']} · "
              f"{plan['environment']['cpu']} · {manifest['machine']}")
    if plan.get('power_label'):
        common += f" · {plan['power_label']}"
    lookup = {(c['profile'], c['size']): c for c in cells}
    saved_cmap = LinearSegmentedColormap.from_list('saved', ['#f3f7f7', '#a6d7ce', '#126959'])
    signed_cmap = LinearSegmentedColormap.from_list('signed', ['#b66339', '#fafafa', '#126959'])

    def save(fig, name, description):
        fig.savefig(root/f'{name}.svg', metadata={'Date': None, 'Title': name.replace('-', ' '),
                    'Description': description+' '+common+'. Full numeric and per-session tables: report.md.'})
        # PNGs are local QA previews; the README embeds resolution-independent SVG.
        preview = Path(__file__).resolve().parent/'.work/readme-chart-preview'
        preview.mkdir(parents=True, exist_ok=True)
        fig.savefig(preview/f'{name}.png', dpi=160)
        plt.close(fig)

    def heatmap(ax, metric, cmap, norm, row_labels=True, times=False):
        values = np.array([[lookup[p,n][metric] for n in SIZES] for p, _ in PROFILES])
        im = ax.imshow(values, cmap=cmap, norm=norm, aspect='auto')
        ax.set_xticks(range(4), ['100', '200', '500', '1,000'])
        ax.xaxis.tick_top()
        ax.set_yticks(range(5), [label for _, label in PROFILES] if row_labels else ['']*5)
        ax.tick_params(axis='both', which='both', length=0, pad=10)
        ax.set_xticks(np.arange(-.5,4,1), minor=True)
        ax.set_yticks(np.arange(-.5,5,1), minor=True)
        ax.grid(which='minor', color='white', linewidth=4)
        ax.spines[:].set_visible(False)
        for i, (profile, _) in enumerate(PROFILES):
            for j, size in enumerate(SIZES):
                value = values[i,j]
                r,g,b,_ = im.cmap(im.norm(value))
                ink = 'white' if .2126*r+.7152*g+.0722*b < .52 else '#172b35'
                ax.text(j, i-.11 if times else i, f'{value:.1f}%' if times else f'{value:+.1f}%',
                        ha='center', va='center', fontsize=16 if times else 12, weight='bold', color=ink)
                if times:
                    c = lookup[profile,size]
                    ax.text(j, i+.23, f"{c['rust']['time_ms']['median']:.1f} / {c['genanki']['time_ms']['median']:.1f} ms",
                            ha='center', va='center', fontsize=9.4, color=ink)
        return im

    fig = plt.figure(figsize=(10.8,5.7))
    fig.text(.025,.952,'Export time saved vs genanki',fontsize=21,weight='bold')
    fig.text(.025,.905,'3 complete sessions · 30 fresh-process timings per cell/exporter · columns: notes',color='#536b76')
    # A signed scale is retained if a future complete comparison has slower Rust cells.
    has_negative = any(c['time_saved_pct'] < 0 for c in cells)
    im = heatmap(fig.add_axes([.22,.22,.70,.60]), 'time_saved_pct',
                 signed_cmap if has_negative else saved_cmap,
                 TwoSlopeNorm(vmin=-100,vcenter=0,vmax=100) if has_negative else Normalize(0,100), times=True)
    cb = fig.colorbar(im,cax=fig.add_axes([.94,.22,.014,.60]))
    cb.outline.set_visible(False)
    fig.text(.22,.16,'Cell: time saved (%) · pooled median anki-forge / genanki (ms)',fontsize=10)
    fig.text(.025,.103,'Saved = 100 × (1 − Rust median / genanki median). Startup, parsing and default export checks are included.',fontsize=9)
    fig.text(.025,.067,common,fontsize=9,color='#536b76')
    fig.text(.025,.031,'Synthetic Basic workloads; file cache uncontrolled. Full report retains sample spread and every session separately.',fontsize=9,color='#536b76')
    save(fig,'time-heatmap','Time saved and absolute medians for all twenty workload/size cells. No cross-cell average.')

    fig, axes = plt.subplots(2,3,figsize=(11.6,7),sharex=True,sharey=True)
    fig.subplots_adjust(left=.075,right=.97,top=.79,bottom=.17,hspace=.40,wspace=.18)
    fig.text(.035,.95,'How export time grows with deck size',fontsize=21,weight='bold')
    fig.text(.035,.908,'30 timings per point · whiskers: pooled Q1–Q3 · faint dots: the 3 session medians',color='#536b76')
    upper = max([c[a]['time_ms']['q3'] for c in cells for a in ADAPTERS]
                + [r[a] for c in cells for r in c['rounds'] for a in ADAPTERS])
    for ax, (profile,label) in zip(axes.flat,PROFILES):
        for adapter,color,marker in [('rust','#087e8b','o'),('genanki','#c16b26','s')]:
            selected = [lookup[profile,n] for n in SIZES]
            ss = [c[adapter]['time_ms'] for c in selected]
            ax.errorbar(SIZES,[s['median'] for s in ss],
                        yerr=[[s['median']-s['q1'] for s in ss],[s['q3']-s['median'] for s in ss]],
                        color=color,marker=marker,markersize=5,linewidth=1.3,capsize=3,
                        label='anki-forge / Rust [system]' if adapter == 'rust' else 'genanki / Python')
            for c in selected:
                ax.scatter([c['size']]*3,[r[adapter] for r in c['rounds']],s=12,color=color,alpha=.4,zorder=2)
        ax.set_title(label,loc='left',fontsize=11,weight='bold',pad=10)
        ax.set_xlim(0,1050)
        ax.set_ylim(0,upper*1.14)
        ax.set_xticks(SIZES,['100','200','500','1K'])
        ax.tick_params(labelsize=8,labelbottom=True)
        ax.set_xlabel('Notes',fontsize=9)
        ax.grid(axis='y',color='#e1e9ed')
        ax.spines[['top','right']].set_visible(False)
    axes[0,0].set_ylabel('Elapsed time (ms)')
    axes[1,0].set_ylabel('Elapsed time (ms)')
    axes[1,2].set_axis_off()
    axes[1,2].text(0,.9,'Three sessions, all retained',fontsize=11,weight='bold',transform=axes[1,2].transAxes)
    axes[1,2].text(0,.73,'Each session: 3 warmups + 10 timings\nRSS: 3 warmups + 5 separate samples\n\nSame linear axes in every panel\nLines connect measured sizes only\nIQR is spread, not confidence',va='top',fontsize=10,linespacing=1.55,transform=axes[1,2].transAxes)
    fig.legend(*axes[0,0].get_legend_handles_labels(),loc='upper left',bbox_to_anchor=(.03,.883),frameon=False,ncol=2)
    fig.text(.035,.083,common,fontsize=9,color='#536b76')
    fig.text(.035,.046,'Startup and parsing included. Quartiles use linear interpolation at (n − 1) × p; no statistical significance claim.',fontsize=9,color='#536b76')
    save(fig,'time-scaling','Identical linear axes, pooled median/Q1/Q3 and three same-session medians. Connecting lines are visual guides.')

    fig = plt.figure(figsize=(12.8,5.7))
    fig.text(.025,.95,'Memory and package size',fontsize=21,weight='bold')
    fig.text(.025,.904,'Signed savings vs genanki · negative values mean Rust uses more · columns: notes',color='#536b76')
    for bounds,metric,title,detail,labels in [
        ([.18,.22,.34,.50],'rss_mib_saved_pct','Peak RSS saved (%)','Median of 15 separate process peaks',True),
        ([.58,.22,.34,.50],'apkg_bytes_saved_pct','APKG bytes saved (%)','Median of 30 timed output sizes',False)]:
        fig.text(bounds[0],.837,title,fontsize=13,weight='bold')
        fig.text(bounds[0],.799,detail,fontsize=9,color='#536b76')
        im = heatmap(fig.add_axes(bounds),metric,signed_cmap,TwoSlopeNorm(vmin=-100,vcenter=0,vmax=100),labels)
    cb = fig.colorbar(im,cax=fig.add_axes([.944,.22,.012,.50]),ticks=[-100,-50,0,50,100])
    cb.outline.set_visible(False)
    fig.text(.18,.16,'Saved = 100 × (1 − Rust / genanki). RSS compares process high-water marks, not allocation totals.',fontsize=9)
    fig.text(.025,.103,'Default output formats differ: Rust uses modern zstd; genanki uses legacy stored collections. Learning content is equivalent.',fontsize=9)
    fig.text(.025,.065,common,fontsize=9,color='#536b76')
    fig.text(.025,.029,'Full report includes both absolute values, RSS ranges and APKG bytes at every size.',fontsize=9,color='#536b76')
    save(fig,'resources','Signed savings in median process peak RSS and default APKG bytes. Negative cells remain visible.')


def render(root):
    plan, manifests, cells = load_evidence(root)
    summary = {'schema':'media-three-session-summary-v1','source_commit':plan['source_commit'],
               'rounds':plan['rounds'],'quartiles':'linear interpolation at (n - 1) * p',
               'aggregation':plan['aggregate'],'verified_exports':2520,'anki_import_checks':120,
               'oracle_patch':plan['oracle_patch'],'cells':cells,
               'source_snapshot':plan.get('source_snapshot'),
               'renderer_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    (root/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
    source_description = (f"Measured package: base revision `{plan['source_commit']}` plus the frozen uncommitted [source patch](source.patch). The exact [source hashes](source-snapshot.json) and patch hashes were recorded before measurement and remained unchanged across all three sessions."
                          if plan.get('source_snapshot') else
                          f"Measured package revision: `{plan['source_commit']}` (clean for all three sessions).")
    lines = ['# Three-session export comparison','', source_description,'',
        'Five synthetic Basic scenes × 100/200/500/1,000 notes. Each cell/exporter has 30 timing samples and 15 separate RSS samples; each session has 3 warmups before each pass. All 2,520 exports passed original SQLite/template/media checks; 120 first-timed packages passed pinned Anki import/content/render checks.', '',
        f"Host: {plan['environment']['cpu']}; {int(plan['environment']['memory_bytes'])/2**30:g} GiB RAM; {plan['environment']['logical_cpus']} logical CPUs; {manifests[0]['platform']}. Rust 1.92.0 release/default features/system allocator; genanki 0.13.1 on native CPython 3.11.0.", '',
        'Time covers fresh-process launch through exit, including imports, input parsing, media registration and default export checks. Media setup, output verification and Anki imports are outside each timed exporter. One exporter runs at a time, with adjacent interleaved controls; the same balanced seed/schedule is repeated in three sequential sessions. No controlled cold-cache or CPU-work claim is made.', '',
        'PNG images are 64,152 bytes; WAV audio files are 32,044 bytes. Mixed scenes use 30% text, 40% images and 30% audio; shared media uses 49 distinct files. These fixtures do not measure Cloze, Image Occlusion, video, large individual media, bindings, long-lived processes or other hosts.', '',
        'Anki verification uses the pinned upstream revision with the recorded local `tokio/io-util` build-feature patch in [plan.json](plan.json). This is local descriptive evidence with a patched oracle build, not an unmodified-upstream or cross-platform release benchmark. The patch and executable hashes are preserved; the measured package source and binaries did not change. No GUI or audible playback is exercised.', '',
        *([f"Power: {plan['power_label']}. Native readings were recorded before and after all 2,520 exports and checked against the predeclared source; see each session's `power.jsonl`. Absolute times are not compared against earlier sessions with different power conditions.", ''] if plan.get('expected_power_source') else []),
        '## Timing','',
        'Values pool all equally sized sessions. Q1/Q3 use linear interpolation at `(n - 1) × p`; the original per-session runner reports retain its exclusive convention. IQR describes sample spread, not confidence. Savings = `100 × (1 - Rust median / genanki median)`. No cross-scene/size average, best-run selection or p95 is used.', '',
        '| Scene | Notes | Rust ms [Q1, Q3] | genanki ms [Q1, Q3] | Time saved | genanki / Rust | ms saved |',
        '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
    def timing(s):
        return f"{s['median']:.2f} [{s['q1']:.2f}, {s['q3']:.2f}]"
    for c in cells:
        lines.append(f"| {c['label']} | {c['size']} | {timing(c['rust']['time_ms'])} | {timing(c['genanki']['time_ms'])} | {c['time_saved_pct']:.1f}% | {c['genanki_over_rust']:.2f}× | {c['time_saved_ms']:.2f} |")
    lines += ['', '![Time saved for every scene and size](time-heatmap.svg)', '',
              '![Absolute time scaling with sample spread](time-scaling.svg)', '',
              '## Across-session variation', '',
              'Each entry below is Rust/genanki median milliseconds, followed by the same-session time saving. The predeclared diagnostic flags a range exceeding 5 percentage points. All three sessions stay in the aggregate regardless of the flag; it is not a significance test.', '',
              '| Scene | Notes | Session 1 | Session 2 | Session 3 | Saving spread (pp) |',
              '| --- | ---: | ---: | ---: | ---: | ---: |']
    for c in cells:
        entries = ' | '.join(f"{r['rust']:.2f}/{r['genanki']:.2f} ({r['time_saved_pct']:.1f}%)" for r in c['rounds'])
        lines.append(f"| {c['label']} | {c['size']} | {entries} | {c['round_spread_pp']:.2f}{' *' if c['variable_across_rounds'] else ''} |")
    lines += ['', '## Memory and output size', '',
              'RSS is the median [minimum, maximum] of 15 independent process high-water marks. APKG size uses only the 30 timed outputs. Rust writes a modern zstd collection plus a legacy compatibility placeholder; genanki writes a legacy stored collection. Size savings include those default format/compression choices. Matching learning content does not imply byte-identical packages or styles.', '',
              '| Scene | Notes | Rust RSS MiB [min, max] | genanki RSS MiB [min, max] | Rust APKG KiB | genanki APKG KiB |',
              '| --- | ---: | ---: | ---: | ---: | ---: |']
    def rss(s):
        return f"{s['median']:.2f} [{s['min']:.2f}, {s['max']:.2f}]"
    for c in cells:
        lines.append(f"| {c['label']} | {c['size']} | {rss(c['rust']['rss_mib'])} | {rss(c['genanki']['rss_mib'])} | {c['rust']['apkg_bytes']['median']/1024:.2f} | {c['genanki']['apkg_bytes']['median']/1024:.2f} |")
    lines += ['', '![Memory and package size savings](resources.svg)', '', '## Reproduce', '',
              'See the [suite instructions](../../README.md) for locked preparation and measurement. [plan.json](plan.json) was saved before fixture generation or measurement. Original manifests, attempts, verified samples and compressed check records live in `round-1/`, `round-2/`, `round-3/`. Source paths inside these immutable records describe the measurement host; fixtures regenerate from the frozen repository recipe. APKG/media files and executables are excluded from this compact snapshot.', '',
              'Regenerate this report offline from the archived evidence:', '', '```sh',
              f'benchmarks/.venv/bin/python benchmarks/media_report.py benchmarks/results/{root.name}', '```', '',
              '[summary.json](summary.json) contains all statistics; [evidence-sha256.json](evidence-sha256.json) pins the original evidence bytes. The renderer rejects changed evidence, differing experiment identities, incomplete cells, failed attempts or missing Anki checks before calculating scores.', '']
    (root/'report.md').write_text('\n'.join(lines))
    plots(root,cells,plan,manifests[0])
    print(root/'report.md')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory',type=Path)
    render(parser.parse_args().directory.resolve())
