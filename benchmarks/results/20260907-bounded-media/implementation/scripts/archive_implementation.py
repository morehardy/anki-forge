"""Archive complete compact evidence and replayable private candidate sources."""
from pathlib import Path
import difflib
import hashlib
import io
import json
import shutil
import tarfile

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
OUT = REPO / 'benchmarks/results/20260907-bounded-media'
DETAIL = OUT / 'implementation'
assert (OUT / 'experiment.json').is_file()
DETAIL.mkdir(exist_ok=False)
RUNS = ('pool-screen', 'pool-rss', 'rebuild-control', 'root-and-identity',
        'allocation-check', 'before-after-timing', 'before-after-rss')
for name in RUNS:
    dest = DETAIL / name
    dest.mkdir()
    for filename in ('plan.json', 'attempts.jsonl', 'summary.json'):
        shutil.copyfile(WORK / name / filename, dest / filename)
    with tarfile.open(dest / 'process-logs.tar.gz', 'w:gz') as archive:
        for path in sorted((WORK / name).glob('*/*.log')):
            archive.add(path, arcname=str(path.relative_to(WORK / name)))

for pattern in ('build-*.json', 'build-*.log', 'quality*.json', 'quality*.log', '*tests.log', '*failure.log'):
    for path in WORK.glob(pattern):
        shutil.copyfile(path, DETAIL / path.name)
for name in ('plan.json', 'before-source-sha256.json', 'before-build-records.json',
             'accepted-source-sha256.json', 'accepted-build.json', 'cases.json',
             'candidate-cases.json', 'final-comparison-plan.json', 'prior-archives.json',
             'allocation-details.json', 'invalid-builds.json', 'red-payload.log',
             'freeze-accepted.log', 'final-compare.log', 'verify-final.log', 'decision.json', 'review.md'):
    shutil.copyfile(WORK / name, DETAIL / name)
shutil.copytree(WORK / 'final-verification', DETAIL / 'verification')
scripts = DETAIL / 'scripts'
scripts.mkdir()
for name in ('build_probe.py', 'build_root.py', 'freeze_accepted.py', 'run.py',
             'quality.py', 'final_compare.py', 'verify_final.py',
             'archive_implementation.py', 'write_implementation.py',
             'update_readmes.py', 'final_audit.py'):
    shutil.copyfile(WORK / name, scripts / name)

# Each valid build must have compiled the intended crate. Two stale diagnostic
# builds were detected before measurement and are retained as invalid evidence.
builds = {}
for path in WORK.glob('build-*.json'):
    if 'invalid-' in path.name:
        continue
    record = json.loads(path.read_text())
    if 'source_sha256' in record:
        builds[path.stem.removeprefix('build-')] = record
        log = (WORK / (path.stem + '.log')).read_text()
        assert 'Compiling anki_forge ' in log, path

baseline = json.loads((WORK / 'before-source-sha256.json').read_text())
needed = set(baseline.values())
for record in builds.values():
    needed.update(record['source_sha256'].values())
contents = {}
paths = [REPO / name for name in baseline]
paths += [p for p in (REPO / 'anki_forge/src/prepared_media').rglob('*') if p.is_file()]
paths += [p for p in WORK.glob('*.rs') if p.is_file()]
paths += [WORK / 'current-PACKAGE_FILES.txt']
paths += [p for p in (WORK / 'profile-source').rglob('*') if p.is_file()]
for path in paths:
    if not path.is_file():
        continue
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest in needed:
        contents[digest] = data
assert needed == contents.keys(), ('missing source content', needed - contents.keys())
probes = DETAIL / 'probes'
probes.mkdir()
with tarfile.open(probes / 'source-content.tar.gz', 'w:gz') as archive:
    for digest, data in sorted(contents.items()):
        entry = tarfile.TarInfo(digest)
        entry.size = len(data)
        archive.addfile(entry, io.BytesIO(data))
shutil.copyfile(WORK / 'profile-source/Cargo.toml', probes / 'workspace.Cargo.toml')
for name, record in builds.items():
    snapshot = record['source_sha256']
    patch = []
    for relative in sorted(set(baseline) | set(snapshot)):
        old = contents[baseline[relative]] if relative in baseline else b''
        new = contents[snapshot[relative]] if relative in snapshot else b''
        if old == new:
            continue
        patch.extend(difflib.unified_diff(old.decode().splitlines(True), new.decode().splitlines(True),
            fromfile='a/' + relative if old else '/dev/null',
            tofile='b/' + relative if new else '/dev/null'))
    (probes / f'{name}.patch').write_text(''.join(patch))

(probes / 'README.md').write_text('''# Candidate source evidence

Each valid `build-*.json` records the exact crate/adapter source hashes and binary
hash. `source-content.tar.gz` contains each unique file once, named by SHA-256.
Reconstruct a variant by writing each referenced blob to its recorded relative
path; the private workspace root is `workspace.Cargo.toml`. Use the adapter's
locked Cargo manifest to rebuild. Patches compare each variant with the frozen
pre-change crate/adapter, including tests and unused source files where present.

`current` is the prior accepted production binary, pinned in each run plan.
`current-rebuilt` rebuilds its runtime source in the private path. `pool` and
`pool-root` test shared storage; `pool-borrowed` adds a rejected identity-borrowing
experiment. `accepted` is the real workspace build. The `*-memory` builds add
diagnostic allocation accounting only; their timings are not performance claims.

Two diagnostic builds with `invalid-copy-timestamps` in their names reused a stale
crate because copied source mtimes were older. They were detected by identical
binary hashes before any measurement, excluded, and rebuilt after forcing private
source mtimes fresh. All measured candidate logs show the intended crate compiled.
The original invalid build records remain in the archive.
''')
print(DETAIL, flush=True)
