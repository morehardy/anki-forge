"""Private interleaved experiments; immutable executables and frozen fixtures."""
from pathlib import Path
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
import random
import re
import statistics
import subprocess

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
FIXTURES = ROOT / 'benchmarks/.work/readme-comparison/fixtures'
PROFILES = json.loads((ROOT / 'benchmarks/.work/readme-comparison/plan.json').read_text())['profiles']

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for data in iter(lambda: f.read(1024 * 1024), b''):
            h.update(data)
    return h.hexdigest()

def main():
    p = argparse.ArgumentParser()
    p.add_argument('name')
    p.add_argument('variants', nargs='+')
    p.add_argument('--repeats', type=int, default=7)
    p.add_argument('--sizes', type=int, nargs='+', default=[100, 1000])
    p.add_argument('--warmups', type=int, default=3)
    args = p.parse_args()
    run = WORK / args.name
    run.mkdir(exist_ok=False)
    binaries = {v: WORK / 'binaries' / v for v in args.variants}
    identities = {v: sha(path) for v, path in binaries.items()}
    cells = [(profile, size) for profile in PROFILES for size in args.sizes]
    rng = random.Random(20260907)
    records = []
    (run / 'plan.json').write_text(json.dumps({**vars(args), 'binaries': identities, 'started_utc': datetime.now(timezone.utc).isoformat(), 'cells': cells, 'seed': 20260907}, indent=2) + '\n')
    # Save every measurement and first measured artifact per variant/cell.
    for n in range(-args.warmups, args.repeats):
        order = cells.copy()
        rng.shuffle(order)
        for profile, size in order:
            variants = args.variants if (n + cells.index((profile, size))) % 2 else args.variants[::-1]
            for variant in variants:
                key = f'{profile}-{size}-{variant}-{n}'
                folder = run / key
                folder.mkdir()
                tmp = folder / 'tmp'
                tmp.mkdir()
                output = folder / 'output.apkg'
                env = dict(os.environ, TMPDIR=str(tmp), TMP=str(tmp), TEMP=str(tmp))
                command = [str(ROOT / 'benchmarks/.tools/measure'), '120', str(folder / 'stdout.log'), str(folder / 'stderr.log'), str(binaries[variant]), str(FIXTURES / profile / 'inputs' / f'{size}.json'), str(output)]
                result = subprocess.run(command, env=env, cwd=folder, capture_output=True, text=True, timeout=135, check=True)
                measured = json.loads(result.stdout)
                row = dict(profile=profile, size=size, variant=variant, sample=n, measurement=measured)
                stages = re.findall(r'\[DEBUG-scale-opt\] (\S+) (\d+)', (folder / 'stderr.log').read_text())
                row['stages_ns'] = {stage: int(ns) for stage, ns in stages}
                if output.is_file():
                    row.update(artifact_sha256=sha(output), artifact_bytes=output.stat().st_size)
                with (run / 'attempts.jsonl').open('a') as f:
                    f.write(json.dumps(row) + '\n')
                assert measured['reaped'] and not any(measured[k] for k in ('exit_code', 'spawn_error', 'leftover_descendants', 'interrupted_signal')), (key, measured, (folder / 'stderr.log').read_text())
                records.append(row)
                if n != 0:
                    output.unlink()
        print(f'{args.name}: iteration {n + 1}/{args.repeats}', flush=True)
    assert identities == {v: sha(path) for v, path in binaries.items()}
    summary = []
    for profile, size in cells:
        entry = dict(profile=profile, size=size, variants={})
        for variant in args.variants:
            rows = [r for r in records if r['profile'] == profile and r['size'] == size and r['variant'] == variant and r['sample'] >= 0]
            stages = rows[0]['stages_ns']
            entry['variants'][variant] = {'median_ms': statistics.median(r['measurement']['elapsed_ns'] for r in rows) / 1e6, 'median_rss_mib': statistics.median(r['measurement']['peak_rss_bytes'] for r in rows) / 2**20, 'median_stages_ms': {s: statistics.median(r['stages_ns'][s] for r in rows) / 1e6 for s in stages}, 'artifact_hashes': sorted({r['artifact_sha256'] for r in rows})}
        summary.append(entry)
    (run / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    for row in summary:
        print(row['profile'], row['size'], {v: round(data['median_ms'], 3) for v, data in row['variants'].items()}, flush=True)

if __name__ == '__main__':
    main()
