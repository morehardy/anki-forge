"""Seal compact, reproducible evidence for the post-streaming audit."""
from pathlib import Path
import difflib
import gzip
import hashlib
import json
import re
import shutil
import statistics

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
DEST = ROOT / 'benchmarks/results/20260907-post-streaming-audit'
DEST.mkdir(exist_ok=False)
RUNS = ['coarse-pass', 'fine-pass', 'memory-reproduce', 'memory-trace', 'scratch-sweep',
        'deferred-index-pass', 'larger-spool-pass', 'larger-spool-rss']
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


rows_by_run = {}
for run in RUNS:
    folder = DEST / 'runs' / run
    folder.mkdir(parents=True)
    for name in ['plan.json', 'attempts.jsonl', 'summary.json']:
        shutil.copy2(WORK / run / name, folder / name)
    rows = [json.loads(s) for s in (WORK / run / 'attempts.jsonl').read_text().splitlines()]
    rows_by_run[run] = rows
    with gzip.open(folder / 'logs.jsonl.gz', 'wt', encoding='utf-8') as logs:
        for row in rows:
            source = WORK / run / f'{row["case"]}-{row["variant"]}-{row["sample"]}'
            logs.write(json.dumps({key: row[key] for key in ['case', 'variant', 'sample']} |
                                  {name: (source / (name + '.log')).read_text() for name in ['stdout', 'stderr']},
                                  ensure_ascii=False) + '\n')

evidence = DEST / 'evidence'
evidence.mkdir()
for pattern in ['*.json', 'build-*.log']:
    for path in WORK.glob(pattern):
        shutil.copy2(path, evidence / path.name)
shutil.copytree(WORK / 'verification', DEST / 'verification')
scripts = DEST / 'scripts'
scripts.mkdir()
for path in WORK.glob('*.py'):
    shutil.copy2(path, scripts / path.name)

patches = DEST / 'probes'
patches.mkdir()
baseline = json.loads((WORK / 'before-source-sha256.json').read_text())


def source_patch(name, source):
    output = []
    paths = set(baseline)
    paths.update(str(p.relative_to(source)) for directory in ['anki_forge', 'benchmarks/adapters/rust']
                 for p in (source / directory).rglob('*') if p.is_file())
    for relative in sorted(paths):
        before, after = ROOT / relative, source / relative
        if before.exists() and after.exists() and before.read_bytes() == after.read_bytes():
            continue
        a = before.read_text().splitlines(True) if before.exists() else []
        b = after.read_text().splitlines(True) if after.exists() else []
        output.extend(difflib.unified_diff(a, b, fromfile='a/' + relative if before.exists() else '/dev/null',
                                         tofile='b/' + relative if after.exists() else '/dev/null'))
    (patches / (name + '.patch')).write_text(''.join(output))


for name, directory in [('coarse', 'coarse-source'), ('fine', 'fine-source'), ('memory', 'memory-source'),
                        ('larger-spool', 'profile-source')]:
    source_patch(name, WORK / directory)
relative = 'anki_forge/src/writer_core/apkg.rs'
original = (ROOT / relative).read_text()
anchor = '        .with_context(|| format!("open collection database {}", path.display()))?;'
for name, statement in [('scratch-sync-off', '    conn.pragma_update(None, "synchronous", "OFF")?;'),
                        ('scratch-journal-memory', '    conn.pragma_update(None, "journal_mode", "MEMORY")?;')]:
    changed = original.replace(anchor, anchor + '\n' + statement)
    (patches / (name + '.patch')).write_text(''.join(difflib.unified_diff(original.splitlines(True), changed.splitlines(True),
                                                                      fromfile='a/' + relative, tofile='b/' + relative)))
(patches / 'scratch-defer-indexes.patch').write_text(''.join(difflib.unified_diff(
    original.splitlines(True), (WORK / 'deferred-index-apkg.rs').read_text().splitlines(True),
    fromfile='a/' + relative, tofile='b/' + relative)))


def compare(run, base='current', variants=None):
    records = [r for r in rows_by_run[run] if r['sample'] >= 0]
    result = []
    for case in dict.fromkeys(r['case'] for r in records):
        controls = {r['sample']: r for r in records if r['case'] == case and r['variant'] == base}
        for variant in dict.fromkeys(r['variant'] for r in records if r['variant'] != base):
            if variants and variant not in variants:
                continue
            candidates = [r for r in records if r['case'] == case and r['variant'] == variant]
            before = statistics.median(r['measurement']['elapsed_ns'] for r in controls.values()) / 1e6
            after = statistics.median(r['measurement']['elapsed_ns'] for r in candidates) / 1e6
            before_rss = statistics.median(r['measurement']['peak_rss_bytes'] for r in controls.values()) / 2**20
            after_rss = statistics.median(r['measurement']['peak_rss_bytes'] for r in candidates) / 2**20
            result.append({'case': case, 'variant': variant, 'before_ms': before, 'after_ms': after,
                           'time_saved_percent': (1-after/before)*100, 'before_rss_mib': before_rss,
                           'after_rss_mib': after_rss, 'rss_delta_mib': after_rss-before_rss,
                           'samples': len(candidates), 'pairs_faster': sum(r['measurement']['elapsed_ns'] < controls[r['sample']]['measurement']['elapsed_ns'] for r in candidates)})
    return result


memory = {}
for row in rows_by_run['memory-trace']:
    if row['sample'] < 0:
        continue
    log = (WORK / 'memory-trace' / f'{row["case"]}-{row["variant"]}-{row["sample"]}/stderr.log').read_text()
    for raw in re.findall(r'\[DEBUG-post-audit-memory\] (.*)', log):
        event = json.loads(raw)
        key = '|'.join([row['case'], row['variant'], event['label'], 'begin' if event['begin'] else 'end'])
        memory.setdefault(key, []).append(event)
memory = {key: {metric: statistics.median(event[metric] for event in events)
                for metric in ['live_rust_bytes', 'peak_rust_bytes', 'current_rss_bytes', 'cumulative_rust_bytes', 'allocation_calls']}
          for key, events in memory.items()}
summary = {'comparisons': {name: compare(name, 'previous' if name == 'memory-reproduce' else 'current')
                           for name in RUNS if name != 'memory-trace'},
           'memory_boundary_medians': memory, 'exports': sum(len(rows) for rows in rows_by_run.values()),
           'production_binary_sha256': sha(WORK / 'binaries/current'),
           'production_source_evidence': '../20260907-streaming-followup/source.patch',
           'production_source_patch_sha256': sha(ROOT / 'benchmarks/results/20260907-streaming-followup/source.patch'),
           'all_power_sources': sorted({r['power_before'].splitlines()[0] for rows in rows_by_run.values() for r in rows}),
           'limitations': ['Diagnostic phase medians overlap; worker times sum across threads and are not serialized wall time.',
                          'Timing probes perturb execution. Memory-only allocator/ps probes are excluded from timing claims.',
                          'Candidate timings are separate same-session comparisons, never pooled across sessions.',
                          'Larger-spool RSS conclusion uses larger-spool-rss only; timing uses larger-spool-pass only.',
                          '13 earlier cases plus two fixed-note-count long-field variants; no custom/Cloze/persistent-build performance claims.',
                          'Deferred indexes failed full-table equality because sqlite_stat1 changed; that failure is retained.']}
write_json(DEST / 'summary.json', summary)
print('Archived', summary['exports'], 'exports in', DEST)
