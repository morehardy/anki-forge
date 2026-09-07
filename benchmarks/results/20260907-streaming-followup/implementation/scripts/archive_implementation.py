"""Archive every experiment and final verification without generated payloads."""
import hashlib
import json
from pathlib import Path
import shutil

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
OUT = REPO / 'benchmarks/results/20260907-streaming-followup'
assert (OUT/'experiment.json').is_file()
DETAIL = OUT / 'implementation'
DETAIL.mkdir(exist_ok=False)
for name in ('media-stream-stable', 'zip-pipeline', 'remaining-candidates', 'before-after-timing', 'before-after-rss', 'media-stream'):
    dest = DETAIL/name
    dest.mkdir()
    for file in ('plan.json', 'attempts.jsonl', 'summary.json'):
        if (WORK/name/file).is_file():
            shutil.copy2(WORK/name/file, dest/file)
for pattern in ('build-*.json', 'build-*.log', '*.patch', '*tests.log', '*failure.log'):
    for path in WORK.glob(pattern):
        shutil.copy2(path, DETAIL/path.name)
for name in ('before-source-sha256.json', 'before-build-records.json', 'cases.json', 'candidate-cases.json', 'final-comparison-plan.json', 'media-hash-unit.log', 'media-stream-limits.log', 'quality.log', 'quality-summary.json', 'accepted-build.log', 'accepted-build.json'):
    shutil.copy2(WORK/name, DETAIL/name)
scripts = DETAIL/'scripts'
scripts.mkdir()
for name in ('build_probe.py', 'build_accepted.py', 'run.py', 'identity_candidate.py', 'scheduling_candidate.py', 'final_compare.py', 'verify_final.py', 'archive_implementation.py', 'write_implementation.py'):
    shutil.copy2(WORK/name, scripts/name)
shutil.copytree(WORK/'final-verification', DETAIL/'verification')
print(DETAIL)
