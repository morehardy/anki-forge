"""Publish compact assessment evidence without large generated media/artifacts."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import platform
import shutil
import subprocess
import tarfile

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
DESTINATION = ROOT / 'benchmarks/results/20260907-remaining-opportunities'

for name in ('coarse', 'detail', 'candidates'):
    target = DESTINATION / name
    target.mkdir(exist_ok=True)
    for filename in ('plan.json', 'summary.json', 'attempts.jsonl'):
        shutil.copy2(WORK / name / filename, target / filename)

for name in ('cases.json', 'verification.json', 'profile-detail.patch', 'hash256k.patch', 'spool-buffer.patch',
             'build-profile.json', 'build-profile.log', 'build-profile-detail.json', 'build-profile-detail.log',
             'build-hash256k.json', 'build-hash256k.log', 'build-spool-buffer.json', 'build-spool-buffer.log'):
    shutil.copy2(WORK / name, DESTINATION / name)
shutil.copy2(WORK / 'before-source-sha256.json', DESTINATION / 'production-source-sha256.json')
shutil.copy2(WORK / 'before-build-records.json', DESTINATION / 'production-build-records.json')
scripts = DESTINATION / 'scripts'
scripts.mkdir(exist_ok=True)
for name in ('instrument.py', 'setup_coarse.py', 'detail.py', 'build_probe.py', 'cases.py', 'run.py', 'verify_cases.py', 'archive.py'):
    shutil.copy2(WORK / name, scripts / name)

with tarfile.open(DESTINATION / 'process-logs.tar.gz', 'w:gz') as archive:
    for name in ('coarse', 'detail', 'candidates'):
        for path in sorted((WORK / name).glob('*/*.log')):
            archive.add(path, arcname=str(path.relative_to(WORK)), recursive=False)

def command(args):
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout.strip()

environment = dict(captured_utc=datetime.now(timezone.utc).isoformat(), captured_after_measurement=True,
    platform=platform.platform(), rustc=command(['rustc', '-vV']), cargo=command(['cargo', '--version']),
    cpu=command(['sysctl', '-n', 'machdep.cpu.brand_string']), memory_bytes=command(['sysctl', '-n', 'hw.memsize']),
    logical_cpus=command(['sysctl', '-n', 'hw.logicalcpu']), power=command(['pmset', '-g', 'batt']),
    collector_sha256=hashlib.sha256((ROOT / 'benchmarks/.tools/measure').read_bytes()).hexdigest(),
    inspector_sha256=hashlib.sha256((ROOT / 'target/release/contract_tools').read_bytes()).hexdigest())
(DESTINATION / 'environment.json').write_text(json.dumps(environment, indent=2) + '\n')

files = sorted(path for path in DESTINATION.rglob('*') if path.is_file() and path.name != 'SHA256SUMS')
(DESTINATION / 'SHA256SUMS').write_text(''.join(
    hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + str(path.relative_to(DESTINATION)) + '\n' for path in files))
print('Archived', len(files) + 1, 'files;', sum(p.stat().st_size for p in DESTINATION.rglob('*') if p.is_file()), 'bytes')
