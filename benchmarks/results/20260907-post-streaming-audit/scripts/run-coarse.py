"""All attempts retained. Paired fresh processes; diagnostics, not formal claims."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import os
import random
import re
import statistics
import subprocess

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('name')
    parser.add_argument('variants', nargs='+')
    parser.add_argument('--cases', nargs='+')
    parser.add_argument('--warmups', type=int, default=3)
    parser.add_argument('--repeats', type=int, default=7)
    args = parser.parse_args()
    run = WORK / args.name
    run.mkdir(exist_ok=False)
    cases = [c for c in json.loads((WORK / 'cases.json').read_text()) if not args.cases or c['name'] in args.cases]
    binaries = {v: WORK / 'binaries' / v for v in args.variants}
    identities = {v: sha(p) for v, p in binaries.items()}
    initial_power = subprocess.run(['pmset', '-g', 'batt'], capture_output=True, text=True, check=True).stdout
    power_source = initial_power.splitlines()[0]
    (run / 'plan.json').write_text(json.dumps(dict(args=vars(args), cases=cases, binaries=identities,
                                                   utc=datetime.now(timezone.utc).isoformat(), initial_power=initial_power, seed=20260907), indent=2) + '\n')
    rng = random.Random(20260907)
    records = []
    artifact_hashes = {}
    for sample in range(-args.warmups, args.repeats):
        order = cases.copy()
        rng.shuffle(order)
        for case in order:
            path = Path(case['path'])
            assert sha(path) == case['input_sha256']
            variants = args.variants if (sample + cases.index(case)) % 2 else args.variants[::-1]
            for variant in variants:
                folder = run / f'{case["name"]}-{variant}-{sample}'
                folder.mkdir()
                temporary = folder / 'tmp'
                temporary.mkdir()
                output = folder / 'output.apkg'
                env = dict(os.environ, TMPDIR=str(temporary), TMP=str(temporary), TEMP=str(temporary))
                command = [str(ROOT / 'benchmarks/.tools/measure'), '120', str(folder / 'stdout.log'),
                           str(folder / 'stderr.log'), str(binaries[variant]), str(path), str(output)]
                power_before = subprocess.run(['pmset', '-g', 'batt'], capture_output=True, text=True, check=True).stdout
                utc_start = datetime.now(timezone.utc).isoformat()
                result = subprocess.run(command, env=env, cwd=folder, text=True, capture_output=True, timeout=135, check=True)
                measured = json.loads(result.stdout)
                power_after = subprocess.run(['pmset', '-g', 'batt'], capture_output=True, text=True, check=True).stdout
                stages = re.findall(r'\[DEBUG-scale-opt\] (\S+) (\d+)', (folder / 'stderr.log').read_text())
                row = dict(case=case['name'], variant=variant, sample=sample, measurement=measured, utc_start=utc_start, power_before=power_before, power_after=power_after,
                           stages_ns={k: int(v) for k, v in stages})
                if output.is_file():
                    row.update(artifact_sha256=sha(output), artifact_bytes=output.stat().st_size)
                with (run / 'attempts.jsonl').open('a') as handle:
                    handle.write(json.dumps(row) + '\n')
                assert measured['reaped'] and not any(measured[k] for k in ('exit_code', 'spawn_error', 'leftover_descendants', 'interrupted_signal')), (row, (folder / 'stderr.log').read_text())
                previous = artifact_hashes.setdefault(case['name'], row['artifact_sha256'])
                assert row['artifact_sha256'] == previous, row
                assert all(p.splitlines()[0] == power_source for p in (power_before, power_after)), 'Power source changed; all attempts retained'
                records.append(row)
                if sample != 0:
                    output.unlink()
        print(f'{args.name}: iteration {sample + 1}/{args.repeats}', flush=True)
    assert identities == {v: sha(p) for v, p in binaries.items()}
    summary = []
    for case in cases:
        entry = dict(case=case['name'], variants={})
        for variant in args.variants:
            rows = [r for r in records if r['case'] == case['name'] and r['variant'] == variant and r['sample'] >= 0]
            entry['variants'][variant] = dict(
                median_ms=statistics.median(r['measurement']['elapsed_ns'] for r in rows) / 1e6,
                median_rss_mib=statistics.median(r['measurement']['peak_rss_bytes'] for r in rows) / 2**20,
                stages_ms={k: statistics.median(r['stages_ns'][k] for r in rows) / 1e6 for k in rows[0]['stages_ns']},
                artifact_sha256=artifact_hashes[case['name']])
        summary.append(entry)
    (run / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    for row in summary:
        print(row['case'], {v: round(d['median_ms'], 3) for v, d in row['variants'].items()}, flush=True)


if __name__ == '__main__':
    main()
