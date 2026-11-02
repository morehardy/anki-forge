import pathlib, json, hashlib, sys

environment = sys.argv[1]
assert environment in ['prefix', 'root']
batch = pathlib.Path('docs/plans/evidence/website-pr66-portable-search-2026-11-02')
old_batch = pathlib.Path('docs/plans/evidence/website-pr66-rebuild-2026-11-02')
archive = pathlib.Path(f'target/website-pr66-portable-search/{environment}-dist')
old_archive = pathlib.Path(f'target/website-pr66-rebuild/{environment}-dist')
digest = lambda data: hashlib.sha256(data).hexdigest()
def proof(path):
    return {'path':str(path), 'sha256':digest(path.read_bytes())}
capture_path = batch/f'technical-machine-{environment}/observations.json'
capture = json.loads(capture_path.read_bytes())
manifest_path = batch/f'{environment}-site-manifest.json'
manifest = json.loads(manifest_path.read_bytes())
previous_report_path = old_batch/f'machine-read-{environment}.json'
previous_report = json.loads(previous_report_path.read_bytes())
old_manifest_path = pathlib.Path(previous_report['originManifest']['path'])
old_manifest = json.loads(old_manifest_path.read_bytes())
assert manifest_path.read_bytes() == (archive/'generated/site-manifest.json').read_bytes()
assert old_manifest_path.read_bytes() == (old_archive/'generated/site-manifest.json').read_bytes()
assert digest(old_manifest_path.read_bytes()) == previous_report['originManifest']['sha256']
assert digest(previous_report_path.read_bytes()) == capture['previousReport']['sha256']
assert digest(manifest_path.read_bytes()) == capture['manifest']['sha256']
assert previous_report['status'] == 'passed'
assert manifest['configuration'] == previous_report['configuration'] == old_manifest['configuration']
assert manifest['releaseLocales'] == previous_report['releaseLocales'] == old_manifest['releaseLocales']
old_bindings = {x['route']: x['sha256'] for x in previous_report['resourceBindings']}
responses = {x['route']:x for x in capture['responses']}
records = []
for resource in capture['inventory']:
    route = resource['route']
    filename = route.removeprefix('/anki-forge').lstrip('/') if environment == 'prefix' else route.lstrip('/')
    before = (old_archive/filename).read_bytes()
    current = (archive/filename).read_bytes()
    assert digest(before) == resource['previousSha256'] == old_bindings[route]
    assert digest(current) == resource['sha256']
    assert before == current, route
    assert not resource['changed']
    capture_record = responses.get(route)
    if capture_record:
        assert capture_record['status'] == 200
        assert pathlib.Path(capture_record['bodyPath']).read_bytes() == current
        assert capture_record['sha256'] == digest(current)
    records.append({'route': route, 'previousSha256': digest(before), 'sha256': digest(current), 'byteIdentical': True, 'capture': capture_record})
assert len(records) == 192 and len(responses) == 60
output = {
 'schemaVersion':1, 'reviewerRun':f'/root/standards_review/pr66-portable-search-machine-{environment}-2026-11-02',
 'status':'bytes-verified-awaiting-scoped-reading',
 'method':'The reviewer personally performed fresh localhost HTTP GETs for all six locale llms entries and four task paths (60 distinct resources), and independently read all 192 actual old/current archive resource byte arrays. Each previous byte digest matches the immutable prior independent report; every current digest matches the new manifest and every captured response matches current archive bytes. All 192 are exactly unchanged, with no normalization. This audit alone is not a semantic pass.',
 'manifest':proof(manifest_path), 'capture':proof(capture_path), 'previousReport':proof(previous_report_path), 'previousManifest':previous_report['originManifest'],
 'inputSha256':manifest['inputSha256'], 'configuration':manifest['configuration'], 'releaseLocales':manifest['releaseLocales'],
 'records':records,
}
target = batch/f'technical-machine-{environment}-input-audit.json'
target.write_text(json.dumps(output,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'path':str(target),'sha256':digest(target.read_bytes()),'resources':len(records),'unchanged':len(records),'actualHttpResponses':len(responses),'inputSha256':manifest['inputSha256']}))
