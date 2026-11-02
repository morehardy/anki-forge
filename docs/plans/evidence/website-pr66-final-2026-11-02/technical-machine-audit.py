import hashlib
import json
import pathlib
import difflib
import sys

environment = sys.argv[1]
assert environment in ('prefix', 'root')
root = pathlib.Path.cwd()
batch = pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
old_batch = pathlib.Path('docs/plans/evidence/multilingual-review-fixes-2026-11-01')
archive = pathlib.Path(f'target/website-pr66-final/{environment}-dist')
old_archive = pathlib.Path(f'/private/tmp/anki-forge-multilingual-fixes-20261101/target/multilingual-review-fixes/{environment}-dist')
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest_bytes = (batch / f'{environment}-site-manifest.json').read_bytes()
manifest = json.loads(manifest_bytes)
observations_path = batch / f'machine-{environment}/observations.json'
observations_bytes = observations_path.read_bytes()
observations = json.loads(observations_bytes)
assert sha(manifest_bytes) == observations['manifest']['sha256']
assert manifest_bytes == (archive / 'generated/site-manifest.json').read_bytes()
assert manifest['inputSha256'] == 'fd6716207f6f5557895553e95f710a27cc66cee6d4bbd1cacf0ddc56b7aceebf'
old_report_path = old_batch / f'machine-read-{environment}.json'
old_report_bytes = old_report_path.read_bytes()
old_report = json.loads(old_report_bytes)
old_manifest_bytes = pathlib.Path(old_report['originManifest']['path']).read_bytes()
assert sha(old_manifest_bytes) == old_report['originManifest']['sha256']
assert old_manifest_bytes == (old_archive / 'generated/site-manifest.json').read_bytes()
old_manifest = json.loads(old_manifest_bytes)
old_resources = {x['route']: x for x in old_manifest['resources']}
responses = {x['route']: x for x in observations['responses']}
prefix = '' if manifest['configuration']['base'] == '/' else manifest['configuration']['base']
records = []
for resource in observations['inventory']:
    route = resource['route']
    name = route[len(prefix):].lstrip('/')
    before_bytes = (old_archive / name).read_bytes()
    after_bytes = (archive / name).read_bytes()
    assert sha(before_bytes) == resource['previousSha256'] == old_resources[route]['sha256']
    assert sha(after_bytes) == resource['sha256']
    before = before_bytes.decode('utf-8')
    after = after_bytes.decode('utf-8')
    changed = before_bytes != after_bytes
    assert changed == resource['changed']
    if changed:
        assert route in responses
    captured = responses.get(route)
    if captured:
        assert pathlib.Path(captured['bodyPath']).read_bytes() == after_bytes
        assert captured['status'] == 200
        assert captured['sha256'] == sha(after_bytes)
    records.append({
        'route': route, 'sha256': sha(after_bytes), 'previousSha256': sha(before_bytes),
        'changed': changed, 'capture': captured,
        'delta': ''.join(difflib.unified_diff(before.splitlines(True), after.splitlines(True), fromfile=route+' (previous reviewed)', tofile=route+' (current captured)', n=2)) if changed else '',
    })
assert len(records) == 192
report = {
    'schemaVersion': 1, 'reviewerRun': f'/root/standards_review/pr66-machine-{environment}-2026-11-02',
    'status': 'verified-bytes-awaiting-semantic-review',
    'manifest': observations['manifest'],
    'capture': {'path': str(observations_path), 'sha256': sha(observations_bytes), 'captureRun': observations['captureRun'], 'readOrigin': observations['readOrigin']},
    'previousReview': {'path': str(old_report_path), 'sha256': sha(old_report_bytes), 'originManifest': old_report['originManifest']},
    'method': 'Reviewer independently re-read all 192 new archive files and the actual previously reviewed archive files as bytes. Current response bodies were captured over HTTP by the coordinator; each saved response was matched to its actual new archive bytes. This audit only verifies byte identity and retains exact changed-text deltas, without asserting semantic pass.',
    'records': records,
}
output = batch / f'technical-machine-{environment}-input-audit.json'
output.write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
print(json.dumps({'path': str(output), 'sha256': sha(output.read_bytes()), 'resources': len(records), 'changed': sum(x['changed'] for x in records), 'capturedResponses': len(responses)}))
