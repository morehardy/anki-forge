"""Predeclared same-session before/after timing, followed by separate RSS."""
import json
from pathlib import Path
import subprocess
import sys
import time
import run

work = Path(__file__).resolve().parent
plan = work / 'final-comparison-plan.json'
assert not plan.exists(), 'Do not overwrite an experiment'
power = subprocess.check_output(['pmset', '-g', 'batt'], text=True)
plan.write_text(json.dumps({
    'timing': {'warmups': 3, 'samples': 10}, 'rss': {'warmups': 3, 'samples': 5},
    'variants': ['current', 'accepted'], 'cases': json.loads((work/'cases.json').read_text()),
    'binaries': {v: run.sha(work/'binaries'/v) for v in ('current', 'accepted')},
    'power': power, 'policy': 'Keep every attempt. Stop on any failure or power source change. No outlier removal or automatic extra runs.',
}, indent=2)+'\n')
print('Cooling down for 30 seconds before the frozen comparison.', flush=True)
time.sleep(30)
for name, count in [('before-after-timing', 10), ('before-after-rss', 5)]:
    assert subprocess.check_output(['pmset', '-g', 'batt'], text=True).splitlines()[0] == power.splitlines()[0]
    sys.argv = ['run.py', name, 'current', 'accepted', '--warmups', '3', '--repeats', str(count)]
    run.main()
