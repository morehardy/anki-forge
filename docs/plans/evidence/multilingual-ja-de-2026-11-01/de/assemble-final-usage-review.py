"""Bind six-locale browser evidence to separately recorded independent judgments.

Preparation alone emits no review. Each environment requires its own actual browser
tour, immutable origin manifest, execution audit and independently read conclusions.
"""
import hashlib
import json
import sys
from pathlib import Path
from urllib.parse import urlparse

root = Path(__file__).resolve().parents[5]
directory = Path(__file__).resolve().parent
assert sys.argv[1] in ('prefix', 'root')
environment = 'final-' + sys.argv[1]

def digest(data):
    return hashlib.sha256(data).hexdigest()

def proof(path, description):
    path = Path(path)
    return {'path': str(path.relative_to(root)), 'sha256': digest(path.read_bytes()), 'description': description}

def read(path):
    return json.loads(Path(path).read_text())

def check_binding(binding):
    assert digest((root / binding['path']).read_bytes()) == binding['sha256'], binding['path']

def check_issues(issues):
    assert isinstance(issues, list)
    assert all(issue['severity'] in ('minor', 'preference') for issue in issues), issues

tour_path = directory / f'usage-{environment}/browser-tour.json'
tour = read(tour_path)
manifest_path = directory / f'{environment}-origin-manifest.json'
manifest_bytes = manifest_path.read_bytes()
manifest = json.loads(manifest_bytes)
assert (directory / f'usage-{environment}/origin-manifest.json').read_bytes() == manifest_bytes
assert tour['manifestSha256'] == digest(manifest_bytes)
assert tour['mode'] == 'release-candidate'
assert tour['commit'] == '6f5f860117f17859d8cbe1fbe0073590e33564b4'
assert set(manifest['releaseLocales']) == {'en', 'zh-CN', 'es', 'pt-BR', 'ja', 'de'}
assert not tour['pageErrors'] and not tour['responseReadErrors'] and 'failure' not in tour
for key in ('inputSha256', 'configuration', 'releaseLocales', 'commit'):
    assert tour[key] == manifest[key]
check_binding(tour['helper'])
conclusions_path = directory / f'usage-task-conclusions-{environment}.json'
conclusions = read(conclusions_path)
assert tour['commit'] == conclusions['sourceCommit']
assert {Path(item['path']).name for item in conclusions['sourceReviews']} == {
    'linguistic-source-review-final.json', 'linguistic-source-review-mime-fixed.json'
}
review = conclusions['observations'][environment]
assert review['status'] == 'passed'
check_issues(review['openIssues'])
check_issues(conclusions.get('openIssues', []))
for source_review in conclusions['sourceReviews']:
    check_binding(source_review)
check_binding(review['browserTour'])
assert review['browserTour']['path'] == str(tour_path.relative_to(root))
audit_path = directory / 'execution-reuse-audit.json'
audit = read(audit_path)
assert audit['status'] == 'passed' and not audit['openIssues']
check_binding(conclusions['executionAudit'])
assert conclusions['executionAudit']['path'] == str(audit_path.relative_to(root))
for binding in audit['currentBindings']:
    check_binding(binding)
for executions in audit['executionEvidenceByTask'].values():
    for execution in executions:
        for binding in [execution] + execution['sourceBindings'] + execution['dependencyBindings']:
            check_binding(binding)

records = {record['route']: record for record in manifest['pages'] + manifest['resources']}
bindings = {}
def bind(route, expected=None):
    record = records[route]
    assert expected is None or record['sha256'] == expected
    bindings[route] = record['sha256']

machine = [record for record in manifest['resources'] if record['route'].endswith(('/index.md', '/llms.txt'))]
assert len(machine) == 32 * len(manifest['releaseLocales'])
for record in machine:
    bind(record['route'], record['sha256'])
for record in tour['loadedResources']:
    assert record['status'] == 200
    assert records[record['route']]['sha256'] == record['sha256']

required = {'rust-basic', 'python-basic', 'node-basic', 'json-media', 'original-baseline-update', 'troubleshooting'}
entries = {}
assert set(review['entries']) == set(manifest['releaseLocales'])
for locale in manifest['releaseLocales']:
    observed = tour['entries'][locale]
    assert {task['id'] for task in observed} == required
    assert set(review['entries'][locale]) == required
    results = []
    for task in observed:
        task_id = task['id']
        judgment = review['entries'][locale][task_id]
        assert judgment['status'] == 'passed'
        check_issues(judgment['openIssues'])
        assert judgment['bodySha256'] == digest(task['body'].encode())
        assert judgment['conclusion'] and judgment['method']
        read_urls = list(task['readUrls'])
        evidence = [
            proof(tour_path, f'Actual final-artifact browser actions, rendered {locale} body, clipboard and downloads for {task_id}.'),
            proof(root / task['screenshot'], f'Current actual {locale} {task_id} screenshot.'),
            proof(conclusions_path, 'Independent judgments over the actual rendered bodies and exact previous-body reuse checks.'),
            proof(audit_path, 'Independent current-input audit of retained real SDK runs and decoded outputs; no new SDK run claimed.')
        ]
        assert evidence[1]['sha256'] == task['screenshotSha256']
        if 'reusedBody' in judgment:
            reuse = judgment['reusedBody']
            check_binding(reuse['tour'])
            prior_tour = read(root / reuse['tour']['path'])
            previous = next(item for item in prior_tour['entries'][locale] if item['id'] == task_id)
            assert previous['body'] == task['body']
            check_binding(reuse['review'])
            evidence.extend([reuse['tour'], reuse['review']])
        else:
            assert judgment['method'] == 'independent-full-rendered-body-read'
        for url in read_urls:
            assert urlparse(url).scheme + '://' + urlparse(url).netloc == tour['readOrigin']
            bind(urlparse(url).path)
        for copied in task['copiedPrograms']:
            source = (root / copied['source']).read_bytes()
            assert digest(source) == copied['sourceSha256']
            assert source.decode().strip() == copied['copiedCode'].strip()
            assert digest(copied['copiedCode'].strip().encode()) == copied['clipboardSha256']
        for download in task['downloadedResources']:
            assert download['httpStatus'] == 200
            bind(urlparse(download['url']).path, download['sha256'])
            read_urls.append(download['url'])
            item = proof(root / download['path'], f'Actual HTTP download from the {locale} input anchor: {download["anchorText"]}.')
            assert item['sha256'] == download['sha256']
            evidence.append(item)
        if task_id == 'troubleshooting':
            assert conclusions['localizedDuplicateKeyRows'][locale] in task['body']
        final_route = urlparse(task['finalUrl']).path
        evidence.append({'route': final_route, 'sha256': records[final_route]['sha256'], 'observation': f'Read localized heading “{task["h1"]}”. ' + judgment['conclusion']})
        steps = task['steps'] + [judgment['conclusion'], judgment['method']]
        results.append({'id': task_id, 'status': 'passed', 'openIssues': judgment['openIssues'], 'readUrls': read_urls, 'steps': steps, 'executionEvidence': audit['executionEvidenceByTask'][task_id], 'evidence': evidence})
    entries[locale] = {'taskResults': results, 'openIssues': []}

report = {
    'schemaVersion': 1, 'kind': 'usage-tasks', 'status': 'passed', 'mode': 'release-candidate',
    'reviewerType': 'ai', 'reviewerRun': conclusions['reviewerRun'] + '/' + environment,
    'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'],
    'releaseLocales': manifest['releaseLocales'], 'readOrigin': tour['readOrigin'],
    'originManifest': {'path': str(manifest_path.relative_to(root)), 'sha256': digest(manifest_bytes), 'commit': manifest['commit']},
    'resourceBindings': [{'route': route, 'sha256': sha} for route, sha in sorted(bindings.items())],
    'resourceScope': f'All {len(machine)} required Markdown/llms resources plus actual task readUrls and downloaded recipe inputs are bound. The current observer separately verifies every archived manifest byte and records actual implicit network resources. Unvisited showcase packages and implicit search/JS/CSS resources are not claimed as independently read semantic task content.',
    'executionScope': conclusions['scope'], 'sourceReviews': conclusions['sourceReviews'],
    'openIssues': conclusions.get('openIssues', []) + review['openIssues'], 'entries': entries
}
if 'visualReview' in conclusions:
    report['visualReview'] = conclusions['visualReview']
    for binding in report['visualReview']['evidence']:
        check_binding(binding)
assert sum(len(entry['taskResults']) for entry in entries.values()) == 6 * len(manifest['releaseLocales'])
output = directory / f'usage-tasks-{environment}.json'
assert not output.exists(), f'Refuse overwriting a sealed report: {output}'
output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(output.relative_to(root)), 'sha256': digest(output.read_bytes()), 'tasks': sum(len(e['taskResults']) for e in entries.values()), 'bindings': len(bindings), 'machineScope': len(machine), 'origin': tour['readOrigin']}))
