"""Verify retained before/after bytes, then import each accepted case into Anki."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO / 'benchmarks'))
import bench
import verify

destination = WORK / 'final-verification'
destination.mkdir(exist_ok=False)
results = {}
for case in json.loads((WORK/'cases.json').read_text()):
    name = case['name']
    input_path = Path(case['path'])
    assert bench.sha256(input_path) == case['input_sha256']
    document = json.loads(input_path.read_text())
    folder = destination / name
    folder.mkdir()
    checks = {}
    for variant in ('current', 'accepted'):
        output = WORK / 'before-after-timing' / f'{name}-{variant}-0' / 'output.apkg'
        checked = verify.verify_artifact(output, document, bench.INSPECTOR)
        (folder/f'{variant}.json').write_text(json.dumps(checked, indent=2)+'\n')
        assert checked['status'] == 'passed', (name, variant, checked)
        checks[variant] = checked
    assert checks['current']['artifact_sha256'] == checks['accepted']['artifact_sha256']
    output = WORK / 'before-after-timing' / f'{name}-accepted-0' / 'output.apkg'
    command = [str(bench.ORACLE), str(input_path), str(output), str(folder/'anki.json')]
    with (folder/'oracle.stdout.log').open('w') as out, (folder/'oracle.stderr.log').open('w') as err:
        subprocess.run(command, stdout=out, stderr=err, timeout=120, check=True)
    assert json.loads((folder/'anki.json').read_text())['status'] == 'passed'
    results[name] = {'checks': checks, 'anki_sha256': bench.sha256(folder/'anki.json'), 'command': command}
    (destination/'verification.json').write_text(json.dumps(results, indent=2)+'\n')
    print('VERIFIED', name, 'both exact artifacts + Anki import/render', flush=True)

bindings = {}
for path in sorted(WORK.glob('*/attempts.jsonl')):
    run = path.parent.name
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    for row in rows:
        assert row['artifact_sha256'] == results[row['case']]['checks']['current']['artifact_sha256']
    bindings[run] = {'verified_artifacts': len(rows), 'byte_identity': 'Every artifact matches the independently verified and Anki-imported artifact for its case.'}
(destination/'attempt-bindings.json').write_text(json.dumps(bindings, indent=2)+'\n')
