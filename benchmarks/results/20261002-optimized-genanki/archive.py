"""Seal the optimized measurement and validation evidence without overwriting baselines."""
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
W=Path(__file__).resolve().parent
R=W.parents[2]
D=R/'benchmarks/results/20261002-optimized-genanki'
D.mkdir(exist_ok=False)
name='20261002-optimized-genanki'
run=R/'benchmarks/.work/runs'/name
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
files=[p for p in W.iterdir() if p.is_file() and p.name not in {'archive.py','archive.log'}]
for p in files:shutil.copy2(p,D/p.name)
shutil.copy2(run/'manifest.json',D/'run-manifest.json')
for p in (R/'benchmarks/.work').glob('optimization-*.log'):shutil.copy2(p,D/p.name)
members={}
archive=D/'measurements-and-validation.tar.gz'
with tarfile.open(archive,'w:gz') as a:
    for sub,folder in [('run',run),('smoke-initial',R/'benchmarks/.work/runs/20261002-optimized-smoke'),('smoke-final',R/'benchmarks/.work/runs/20261002-optimized-final-smoke')]:
        for p in sorted(folder.rglob('*')):
            if p.is_file():
                assert p.suffix not in {'.apkg','.sqlite','.db'},p
                key=sub+'/'+str(p.relative_to(folder))
                a.add(p,arcname=key,recursive=False)
                members[key]={'sha256':sha(p),'bytes':p.stat().st_size}
with tarfile.open(archive) as a:
    assert len(a.getmembers())==len(members)
    for member in a:
        assert member.isfile()
        assert hashlib.sha256(a.extractfile(member).read()).hexdigest()==members[member.name]['sha256']
(D/'archive-manifest.json').write_text(json.dumps({'archive':archive.name,'archive_sha256':sha(archive),'archive_bytes':archive.stat().st_size,'members':members,'members_verified':True,'cleanup_note':'Original APKG bytes were deleted by the runner only after their required successful verification; archived reports are observations of those originals.'},indent=2)+'\n')
(D/'README.md').write_text('''# 2026-10-02 optimized benchmark

[中文完整报告](report.md) · [1,000-note chart](comparison.svg) · [20-cell comparison CSV](delta.csv)

One complete 20-cell matrix measured 2026-10-02 17:45:36–17:49:46 Asia/Taipei. The working tree at HEAD `4265fc4751daa18164cbf85bda425f2120e0f57d` includes the production optimizations saved in source.patch. All 20 time medians improved from the pre-optimization 2026-10-02 matrix. At 1,000 notes, unique images/audio/mixed media improved by about 96%, and image RSS fell from 98.28 to 44.39 MiB. Shared-media RSS increased by about 4 MiB. Unique-media time remains above the 2026-09-21 baseline and slower than the newly measured genanki at 1,000 notes; the report preserves these results.

All 840 exports passed original SQLite/content/media/semantic checks; 40 selected actual outputs passed pinned upstream Anki import/content/render checks. All inputs, sources, binaries and dependencies were unchanged before/after. All launches remained on AC Power. The 46 benchmark behavior tests and a separate final 200-note smoke passed. These are exploratory descriptive results from separate desktop sessions; power, background load and cache are not controlled.

Production changes use immutable owned snapshots directly in bounded encoded-media preparation, a shared 4 MiB snapshot memory budget with descriptor-free temporary ownership, reusable identity queries/configuration, and atomic move publication for exclusive build artifacts. Public shared-artifact persistence retains copy semantics; persistent CAS and final artifact/parent-directory durability checks are retained. Regression tests cover the original copy-path failure, bounded small snapshots, low file-descriptor limits, partial spill failure, shared ownership, cleanup and publication facts.

## Evidence

- summary.json / comparison.csv: all 20 cells, both freshly measured implementations, time/RSS/package sample counts, medians, Q1/Q3 and extrema.
- delta.json / delta.csv: changes against both immutable archives, including the remeasured genanki and Rust/genanki ratio changes.
- verification-summary.json: all 840 checks, 40 Anki checks and complete scheduling/power evidence.
- source-snapshot.json / source.patch / source-and-inputs.tar.gz / identity-after.json: exact dirty source and before/after identities. Other README/website/doc changes were already present at freeze time; full Git state is in plan.json.
- run-manifest.json / plan.json / completed.json / host-hardware.json: actual environment, matrix protocol, source/build metadata, host power/load and completion.
- prepared-builds.json / oracle-reuse-check.json / prepare-final.log: current release exporter/inspector/collector and checker provenance. Oracle dependencies are built locked and offline with the same pinned upstream Anki revision and checker source.
- measurements-and-validation.tar.gz / archive-manifest.json: every official raw record plus both preparation smoke runs, hashed and verified member by member. Exported APKG bytes were removed after successful validation and are not archived.
- optimization-*.log: retained regression failures and successful final clippy/core/lifecycle checks, plus the passing workspace verify-fast log. The first memory-test log also retains discovery of constructors that were test-only; the final build promotes the streaming constructors to production. The initial benchmark-tests.log discovered zero tests; benchmark-tests-final.log is the corrected test-directory run with all 46 tests passing.
- run.py / analyze.py / compare.py: predeclared measurement wrapper, strict offline validation and both-baseline comparison/chart generation. archive.py: evidence packaging logic.

## Offline audit

```sh
shasum -a 256 -c SHA256SUMS
```

From the repository root:

```sh
mkdir -p benchmarks/.work/evidence-optimized-20261002
tar -xzf benchmarks/results/20261002-optimized-genanki/measurements-and-validation.tar.gz -C benchmarks/.work/evidence-optimized-20261002
benchmarks/.venv/bin/python benchmarks/results/20261002-optimized-genanki/analyze.py --run-dir benchmarks/.work/evidence-optimized-20261002/run
benchmarks/.venv/bin/python benchmarks/results/20261002-optimized-genanki/compare.py
```

The analyzer rejects incomplete, duplicate, failed or unbalanced samples, missing Anki checks, inconsistent power records and mismatched source/input identities. It computes type-7 quartiles from all raw samples rather than the runner's alternate raw IQR convention. The original source archive and all baselines must remain available for verification.

For a fresh measurement, use a new run name and directory under benchmarks/.work, restore the frozen source files or check out HEAD and apply the relevant source.patch, then rebuild all recorded tools and prepare dependencies before timing. Requery the hardware and checker provenance, and regenerate media fixtures with the frozen generator; compare all 20 JSON hashes and the complete 2,749-file media inventory. Keep compilation, testing and reporting outside measurement. Do not overwrite or selectively replace archived attempts.
''')
shutil.copy2(W/'archive.py',D/'archive.py')
(D/'SHA256SUMS').write_text(''.join(f'{sha(p)}  {p.name}\n' for p in sorted(D.iterdir()) if p.is_file() and p.name!='SHA256SUMS'))
print(json.dumps({'destination':str(D),'archive_members':len(members),'archive_bytes':archive.stat().st_size,'all_members_verified':True}))
