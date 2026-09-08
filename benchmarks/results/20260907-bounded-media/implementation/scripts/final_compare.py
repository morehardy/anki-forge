"""Predeclare paired before/after timing and independent RSS confirmation."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
plan = WORK / 'final-comparison-plan.json'
assert not plan.exists()
hashes = {name: hashlib.sha256((WORK / 'binaries' / name).read_bytes()).hexdigest()
          for name in ('current', 'accepted')}
plan.write_text(json.dumps({
    'cases': json.loads((WORK / 'cases.json').read_text()),
    'binaries': hashes, 'timing_repeats': 7, 'rss_repeats': 5,
    'warmups_each_phase': 2, 'schedule_seed': 20260907,
    'power': subprocess.check_output(['pmset', '-g', 'batt'], text=True),
    'policy': 'Fresh interleaved processes; keep all attempts; exact APKG equality; independent RSS phase; no best-run selection.',
}, indent=2)+'\n')
for name, repeats in (('before-after-timing', 7), ('before-after-rss', 5)):
    subprocess.run([sys.executable, str(WORK / 'run.py'), name, 'current', 'accepted',
                    '--warmups', '2', '--repeats', str(repeats)], cwd=ROOT, check=True)
assert hashes == {name: hashlib.sha256((WORK / 'binaries' / name).read_bytes()).hexdigest()
                  for name in hashes}
