"""Capture independent observations from one immutable archive; no passing conclusions."""
import os
from pathlib import Path
import subprocess
import sys

root = Path.cwd()
locale, config = sys.argv[1:3]
assert locale in ('ja', 'de') and config in ('prefix', 'root')
base = root / 'docs/plans/evidence/multilingual-ja-de-2026-11-01'
phase = base / locale
artifact = root / 'target/remaining-locales' / locale / f'{config}-artifact'
env = dict(os.environ, PATH='/Users/hp/.nvm/versions/node/v24.19.0/bin:' + os.environ['PATH'])
steps = {
    'usage': ['node', base / 'usage-observations.mjs', artifact, phase / f'usage-{config}', root, 'release-candidate'],
    'machine': ['node', base / 'read-machine-tasks.mjs', artifact, config, locale],
    'negative': ['node', base / 'read-search-negative-results.mjs', artifact, phase / f'engineering/{config}/search-results.json', config, locale],
}
if config == 'prefix':
    steps['layout'] = ['node', base / 'layout-observations.mjs', artifact, phase / f'layout-{config}', locale]
for name in sys.argv[3:] or steps:
    print(f'{locale}/{config}: observing {name}', flush=True)
    with (phase / f'{name}-{config}.log').open('w') as log:
        result = subprocess.run([str(arg) for arg in steps[name]], cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        print((phase / f'{name}-{config}.log').read_text()[-7000:])
        raise SystemExit(result.returncode)
print(f'{locale}/{config}: observations captured', flush=True)
