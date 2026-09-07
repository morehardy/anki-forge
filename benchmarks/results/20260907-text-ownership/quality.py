"""Run repository quality gates, preserving each command and complete output."""
from pathlib import Path
import json
import os
import subprocess
import time

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
checks = [
    ('rust-quality', ['bash', 'scripts/check_rust_crate_quality.sh']),
    ('benchmark-tests', ['benchmarks/.venv/bin/python', '-m', 'unittest', 'discover', '-s', 'benchmarks/tests', '-v']),
    ('diff-whitespace', ['git', 'diff', '--check']),
]
results = []
for name, command in checks:
    begin = time.monotonic()
    print('START', name, flush=True)
    with (WORK / f'quality-{name}.log').open('w') as log:
        proc = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                              env=dict(os.environ, ANKI_FORGE_ALLOW_DIRTY_PACKAGE='1', CARGO_NET_OFFLINE='true',
                                       PYTHONPATH=str(ROOT / 'benchmarks')))
    row = dict(name=name, command=command, exit_code=proc.returncode, elapsed_s=time.monotonic()-begin)
    results.append(row)
    (WORK / 'quality.json').write_text(json.dumps(results, indent=2)+'\n')
    print('END', row, flush=True)
    if proc.returncode:
        raise SystemExit(proc.returncode)
