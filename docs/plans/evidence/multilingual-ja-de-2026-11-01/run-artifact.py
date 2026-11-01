"""Serial real builds and checks; never creates independent review verdicts."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

root = Path.cwd()
locale, config, mode = sys.argv[1:]
assert locale in ('ja', 'de') and config in ('prefix', 'root')
assert mode in ('preview', 'engineering', 'finish')
phase = root / 'docs/plans/evidence/multilingual-ja-de-2026-11-01' / locale
output = phase / ('preview' if mode == 'preview' else 'engineering') / config
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, PATH='/Users/hp/.nvm/versions/node/v24.19.0/bin:' + os.environ['PATH'],
           CHROME_EXECUTABLE='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
           SITE_URL='https://morehardy.github.io' if config == 'prefix' else 'https://ankiforge.dev',
           SITE_BASE='/anki-forge' if config == 'prefix' else '/',
           SITE_CUSTOM_DOMAIN='' if config == 'prefix' else 'ankiforge.dev',
           SITE_DEPLOYMENT='preview', RELEASE_EVIDENCE_DIR=str(output),
           PUBLIC_SURFACE_BASELINE=str(root / 'target/remaining-locales/ja-de-trusted-surfaces/public-surface-baseline.json'),
           MACHINE_SURFACE_BASELINE=str(root / 'target/remaining-locales/ja-de-trusted-surfaces/machine-surface-baseline.json'))
env.pop('CONTENT_REVIEW_MODE', None)
def run(name, args):
    print(f'{locale}/{config}/{mode}: {name}', flush=True)
    with (output / f'{name}.log').open('w') as log:
        result = subprocess.run(args, cwd=root / 'website', env=env, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        print((output / f'{name}.log').read_text()[-10000:])
        raise SystemExit(result.returncode)

artifact = root / 'target/remaining-locales' / locale / f'{config}-artifact'
if mode == 'preview':
    env['CONTENT_REVIEW_MODE'] = 'preview'
    run('prepare', ['npm', 'run', 'prepare:site'])
    run('astro-build', ['node_modules/.bin/astro', 'build'])
    run('search-index', ['npm', 'run', 'generate:search'])
    run('machine-index', ['npm', 'run', 'generate:machine'])
    run('search-check', ['node', 'scripts/check-search.mjs', '--preview'])
elif mode == 'engineering':
    run('build', ['npm', 'run', 'build'])
    run('rebuild', ['npm', 'run', 'check:rebuild'])
    run('engineering', ['node', 'scripts/release-evidence.mjs', 'run', '--engineering-only'])
    assert not artifact.exists(), f'Keep previous frozen artifacts immutable: {artifact}'
    shutil.copytree(root / 'website/dist', artifact)
    shutil.copyfile(artifact / 'generated/site-manifest.json', phase / f'{config}-origin-manifest.json')
else:
    assert artifact.exists()
    shutil.rmtree(root / 'website/dist')
    shutil.copytree(artifact, root / 'website/dist')
    run('finish', ['node', 'scripts/release-evidence.mjs', 'finish'])
    run('release-check', ['npm', 'run', 'check:release'])
print(f'{locale}/{config}/{mode}: completed', flush=True)
