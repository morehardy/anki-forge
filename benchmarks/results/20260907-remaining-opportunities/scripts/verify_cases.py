"""Verify each unique artifact, then bind every diagnostic attempt to it by SHA-256."""
from pathlib import Path
import hashlib
import json
import sys

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks'))
from verify import verify_artifact

results = []
for case in json.loads((WORK / 'cases.json').read_text()):
    artifact = WORK / 'coarse' / f'{case["name"]}-current-0/output.apkg'
    result = verify_artifact(artifact, json.loads(Path(case['path']).read_text()), ROOT / 'target/release/contract_tools')
    result['case'] = case['name']
    results.append(result)
    print(case['name'], result['status'], flush=True)
    if result['status'] != 'passed':
        print(json.dumps(result, indent=2), flush=True)
        raise SystemExit(1)

hashes = {row['case']: row['artifact_sha256'] for row in results}
counts = {}
for name in ('coarse', 'detail', 'candidates'):
    rows = [json.loads(line) for line in (WORK / name / 'attempts.jsonl').read_text().splitlines()]
    assert all(row['artifact_sha256'] == hashes[row['case']] for row in rows)
    counts[name] = len(rows)

source_hashes = json.loads((WORK / 'before-source-sha256.json').read_text())
assert all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in source_hashes.items())
binary = ROOT / 'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
assert binary_sha == hashlib.sha256((WORK / 'binaries/current').read_bytes()).hexdigest()
records = json.loads((ROOT / 'benchmarks/.tools/build-records.json').read_text())
assert binary_sha == records[str(binary)]['executable_sha256']
(WORK / 'verification.json').write_text(json.dumps(dict(
    results=results, attempt_counts=counts, all_artifacts_equal_verified_current=True,
    production_source_unchanged=True, production_binary_sha256=binary_sha,
    unchanged_source_files=len(source_hashes), anki_imports_run_this_assessment=0), indent=2) + '\n')
print('Verified', sum(counts.values()), 'exports against', len(results), 'unique artifacts; production source and binary unchanged')
