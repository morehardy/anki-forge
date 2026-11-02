"""Bind separately authored, actually read task answers to frozen HTTP artifacts.

This assembler refuses missing conclusions. It does not create answers, inspect
semantics, inherit previous passing reports, or execute an SDK.
"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path.cwd()
directory = pathlib.Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/de')
environment = sys.argv[1]
assert environment in ('prefix', 'root')
capture_path = directory / f'machine-final-{environment}/http-observations.json'
manifest_path = directory / f'final-{environment}-origin-manifest.json'
conclusions_path = directory / f'machine-conclusions-final-{environment}.json'
load = lambda filename: json.loads((root / filename).read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()

def proof(filename, description):
    return {'path': str(filename), 'sha256': sha((root / filename).read_bytes()), 'description': description}

capture = load(capture_path)
manifest = load(manifest_path)
conclusions = load(conclusions_path)
assert capture['manifestSha256'] == sha((root / manifest_path).read_bytes())
assert capture['inputSha256'] == manifest['inputSha256']
assert capture['configuration'] == manifest['configuration']
assert capture['releaseLocales'] == manifest['releaseLocales']
assert set(capture['entries']) == set(manifest['releaseLocales'])
assert conclusions['reviewedCaptures'][environment]['sha256'] == sha((root / capture_path).read_bytes())
assert conclusions['reviewedCaptures'][environment]['path'] == str(capture_path)
assert set(conclusions['answers']) == set(manifest['releaseLocales'])
records = {record['route']: record for record in manifest['pages'] + manifest['resources']}
bindings = [{'route': record['route'], 'sha256': record['sha256']} for record in manifest['resources'] if record['route'].endswith(('/index.md', '/llms.txt'))]
assert len(bindings) == 192
assert manifest['releaseLocales'] == ['en', 'zh-CN', 'es', 'pt-BR', 'ja', 'de']
expected_ids = {'python-install', 'node-api', 'json-update', 'compatibility-constraints'}
report = {
    'schemaVersion': 1, 'kind': 'machine-read', 'status': 'passed', 'mode': 'release-candidate',
    'reviewerType': 'ai', 'reviewerRun': f'/root/de_technical_review/de-machine-final-{environment}-2026-11-01',
    'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'], 'releaseLocales': manifest['releaseLocales'],
    'readOrigin': capture['readOrigin'],
    'originManifest': dict(proof(manifest_path, 'Original manifest bytes of the actually captured frozen candidate.'), commit=manifest['commit']),
    'resourceBindings': bindings, 'entries': {}, 'openIssues': [],
    'method': conclusions['method'] + ' Header/status/body/hash checks establish artifact identity rather than semantic success. Every machine resource is hash-bound, but unvisited resources are not claimed to have been manually read. Published SDK programs were not rerun for this machine-reading review.',
}
for locale, tasks in capture['entries'].items():
    assert {task['id'] for task in tasks} == expected_ids and len(tasks) == 4
    assert set(conclusions['answers'][locale]) == expected_ids
    results = []
    for task in tasks:
        response_proofs = []
        reads = [task['index']] + task['followed']
        assert task['index']['route'].endswith('/llms.txt')
        assert task['readUrls'] == [item['url'] for item in reads]
        for item in reads:
            raw = (root / item['path']).read_bytes()
            assert item['status'] == 200
            assert 'utf-8' in item['headers']['content-type'].lower()
            assert 'content-disposition' not in item['headers']
            assert raw.decode('utf-8') == item['body']
            assert len(raw) == item['bytes']
            assert sha(raw) == item['sha256'] == records[item['route']]['sha256']
            response_proofs.append(proof(pathlib.Path(item['path']), f"Actual UTF-8 HTTP 200 body read from {item['url']}; bytes match frozen manifest."))
        answer = conclusions['answers'][locale][task['id']]
        assert isinstance(answer, str) and answer.strip()
        results.append({
            'id': task['id'], 'status': 'passed', 'readUrls': task['readUrls'],
            'answer': answer + ' Sources: ' + ', '.join(item['url'] for item in task['followed']),
            'evidence': [proof(capture_path, 'Current HTTP URLs, response headers, bodies, byte sizes and artifact hashes.'), proof(conclusions_path, 'Explicit full-reading versus incremental-review answer lineage and exact current capture bindings.')] + [proof(pathlib.Path(item['path']), item['description']) for item in conclusions['supportingEvidence']] + response_proofs,
            'openIssues': [],
        })
    report['entries'][locale] = {'aiMachineReadReview': results, 'openIssues': []}
target = directory / f'machine-read-final-{environment}.json'
(root / target).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(f'{environment}: {sum(len(value["aiMachineReadReview"]) for value in report["entries"].values())} actual HTTP reading answers, {len(bindings)} exact machine-resource bindings; no SDK rerun claimed.')
