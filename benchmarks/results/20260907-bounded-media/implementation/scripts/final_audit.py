"""Verify sources, replay frozen patches, and seal the completed evidence."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import tarfile

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
OUT = REPO / 'benchmarks/results/20260907-bounded-media'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
read = lambda path: json.loads(path.read_text())
results = {}

for relative, expected in read(WORK / 'accepted-source-sha256.json').items():
    assert sha(REPO / relative) == expected, relative
results['accepted_crate_and_adapter_sources'] = 'unchanged'
for relative, record in read(WORK / 'prior-archives.json').items():
    directory = REPO / relative
    assert sha(directory / 'SHA256SUMS') == record['seal_sha256']
    for line in (directory / 'SHA256SUMS').read_text().splitlines():
        digest, name = line.split('  ', 1)
        assert sha(directory / name) == digest, (relative, name)
results['previous_archives_unchanged'] = len(read(WORK / 'prior-archives.json'))

baseline = read(WORK / 'before-source-sha256.json')
cache = WORK / 'replay-content'
cache.mkdir(exist_ok=False)
with tarfile.open(OUT / 'implementation/probes/source-content.tar.gz') as archive:
    archive.extractall(cache, filter='data')
for path in cache.iterdir():
    assert sha(path) == path.name
replayed = []
for patch in sorted((OUT / 'implementation/probes').glob('*.patch')):
    variant = patch.stem
    root = WORK / 'replay-probes' / variant
    root.mkdir(parents=True, exist_ok=False)
    for relative, digest in baseline.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((cache / digest).read_bytes())
    if patch.stat().st_size:
        subprocess.run(['git', 'apply', str(patch)], cwd=root, check=True,
                        env=dict(os.environ, GIT_CEILING_DIRECTORIES=str(root.parent)))
    expected = read(OUT / 'implementation' / f'build-{variant}.json')['source_sha256']
    actual = {str(p.relative_to(root)): sha(p) for p in root.rglob('*') if p.is_file()}
    assert actual == expected, variant
    replayed.append(variant)
results['candidate_patch_replays'] = replayed

# Check the full frozen patch against its actual Git base, including the new
# untracked runtime files that a plain git diff would otherwise omit.
root = WORK / 'replay-frozen-patch'
root.mkdir(exist_ok=False)
base_archive = WORK / 'git-base.tar'
base = read(OUT / 'plan.json')['source_commit']
with base_archive.open('wb') as out:
    subprocess.run(['git', 'archive', base], cwd=REPO, stdout=out, check=True)
with tarfile.open(base_archive) as archive:
    archive.extractall(root, filter='data')
subprocess.run(['git', 'apply', str(OUT / 'source.patch')], cwd=root, check=True,
                env=dict(os.environ, GIT_CEILING_DIRECTORIES=str(root.parent)))
snapshot = read(OUT / 'source-snapshot.json')['source_files']
for relative, digest in snapshot.items():
    if digest == 'missing':
        assert not (root / relative).exists(), relative
    else:
        assert sha(root / relative) == digest, relative
results['full_frozen_patch_replay_files'] = len(snapshot)
results['status'] = 'passed'
(OUT / 'final-audit.json').write_text(json.dumps(results, indent=2)+'\n')
files = sorted(p for p in OUT.rglob('*') if p.is_file() and p.name != 'SHA256SUMS')
(OUT / 'SHA256SUMS').write_text(''.join(f'{sha(p)}  {p.relative_to(OUT)}\n' for p in files))
print(json.dumps(results, indent=2))
print('Sealed', len(files), 'files;', round(sum(p.stat().st_size for p in files)/2**20, 2), 'MiB')
