"""Run the unchanged full matrix with native power checks outside each timer."""
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
sys.path.insert(0, str(REPO / 'benchmarks'))
import bench
import media_bench

expected = json.loads((HERE / 'plan.json').read_text())['expected_power_source']
original = media_bench.collect_attempt


def power():
    return subprocess.check_output(['pmset', '-g', 'batt'], text=True)


def collect_attempt(case, row, command, env):
    before = power()
    if before.splitlines()[0] != expected:
        bench.append(case.parent / 'power.jsonl', {'id': row['id'], 'before': before, 'status': 'changed_before_launch'})
        raise RuntimeError('Native power source changed before launch')
    try:
        return original(case, row, command, env)
    finally:
        after = power()
        bench.append(case.parent / 'power.jsonl', {'id': row['id'], 'before': before, 'after': after, 'utc': bench.utc()})
        if after.splitlines()[0] != expected:
            raise RuntimeError('Native power source changed during export')


media_bench.collect_attempt = collect_attempt
media_bench.main()
