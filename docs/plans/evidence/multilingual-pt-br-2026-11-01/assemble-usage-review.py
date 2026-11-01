"""Bind manually reviewed current browser observations; no browser/SDK run is invented."""
import hashlib
import json
import sys
from pathlib import Path
from urllib.parse import urlparse

root = Path(__file__).resolve().parents[4]
directory = Path(__file__).resolve().parent
environment = sys.argv[1]
assert environment in ('prefix', 'root')

def digest(data):
    return hashlib.sha256(data).hexdigest()

def proof(path, description):
    path = Path(path)
    return {'path': str(path.relative_to(root)), 'sha256': digest(path.read_bytes()), 'description': description}

def read(path):
    return json.loads(Path(path).read_text())

tour_path = directory / f'usage-final-{environment}/browser-tour.json'
tour = read(tour_path)
manifest_path = directory / f'final-{environment}-origin-manifest.json'
manifest_bytes = manifest_path.read_bytes()
manifest = json.loads(manifest_bytes)
assert (directory / f'usage-final-{environment}/origin-manifest.json').read_bytes() == manifest_bytes
assert tour['manifestSha256'] == digest(manifest_bytes)
assert not tour['pageErrors'] and not tour['responseReadErrors'] and 'failure' not in tour
for key in ('inputSha256', 'configuration', 'releaseLocales', 'commit'):
    assert tour[key] == manifest[key]
conclusions_path = directory / 'usage-task-conclusions.json'
conclusions = read(conclusions_path)
audit_path = directory / 'execution-reuse-audit.json'
audit = read(audit_path)
assert audit['status'] == 'passed' and not audit['openIssues']
assert digest(audit_path.read_bytes()) == 'b0282d29e2de33f624f8b484ffb18d03dfecf2471f7dfa7577933439fcbfb2f8'
for binding in audit['currentBindings']:
    assert digest((root / binding['path']).read_bytes()) == binding['sha256']
for executions in audit['executionEvidenceByTask'].values():
    for execution in executions:
        for binding in [execution] + execution['sourceBindings'] + execution['dependencyBindings']:
            assert digest((root / binding['path']).read_bytes()) == binding['sha256']

records = {record['route']: record for record in manifest['pages'] + manifest['resources']}
bindings = {}
def bind(route, expected=None):
    record = records[route]
    assert expected is None or record['sha256'] == expected
    bindings[route] = record['sha256']

machine = [record for record in manifest['resources'] if record['route'].endswith(('/index.md', '/llms.txt'))]
assert len(machine) == 128
for record in machine:
    bind(record['route'], record['sha256'])
for record in tour['loadedResources']:
    assert record['status'] == 200
    assert records[record['route']]['sha256'] == record['sha256']

entries = {}
for locale in manifest['releaseLocales']:
    observed = tour['entries'][locale]
    assert set(task['id'] for task in observed) == set(conclusions['tasks'])
    results = []
    for task in observed:
        task_id = task['id']
        read_urls = list(task['readUrls'])
        evidence = [proof(tour_path, f'Actual final-artifact browser actions, rendered {locale} body, clipboard contents and downloads for {task_id}.'), proof(root / task['screenshot'], f'Current actual {locale} {task_id} page screenshot.'), proof(conclusions_path, 'Independent interpretation of the actual rendered tasks and explicit execution reuse boundary.'), proof(audit_path, 'Independent refreshed audit of current executable inputs, retained original execution logs and decoded APKG outputs; no SDK rerun claimed.')]
        assert evidence[1]['sha256'] == task['screenshotSha256']
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
            download_proof = proof(root / download['path'], f'Actual HTTP download from the {locale} recipe input anchor: {download["anchorText"]}.')
            assert download_proof['sha256'] == download['sha256']
            evidence.append(download_proof)
        if task_id == 'troubleshooting':
            assert conclusions['localizedDuplicateKeyRows'][locale] in task['body']
        final_route = urlparse(task['finalUrl']).path
        evidence.append({'route': final_route, 'sha256': records[final_route]['sha256'], 'observation': f'Read localized heading “{task["h1"]}”. ' + conclusions['tasks'][task_id]})
        steps = task['steps'] + [conclusions['tasks'][task_id]]
        if environment == 'root':
            prefix = read(directory / 'usage-final-prefix/browser-tour.json')
            previous = next(item for item in prefix['entries'][locale] if item['id'] == task_id)
            assert previous['body'] == task['body']
            steps.append('Reran the task against the root-deployment artifact and its own manifest, including actual clipboard and linked downloads. Compared this rendered main text with the independently read final prefix task: byte-identical prose and code, so its semantic conclusion applies unchanged; routes and resource bytes are separately bound here.')
        results.append({'id': task_id, 'status': 'passed', 'openIssues': [], 'readUrls': read_urls, 'steps': steps, 'executionEvidence': audit['executionEvidenceByTask'][task_id], 'evidence': evidence})
    entries[locale] = {'taskResults': results, 'openIssues': []}

report = {'schemaVersion': 1, 'kind': 'usage-tasks', 'status': 'passed', 'mode': 'release-candidate', 'reviewerType': 'ai', 'reviewerRun': conclusions['reviewerRun'] + '/' + environment, 'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'], 'releaseLocales': manifest['releaseLocales'], 'readOrigin': tour['readOrigin'], 'originManifest': {'path': str(manifest_path.relative_to(root)), 'sha256': digest(manifest_bytes), 'commit': manifest['commit']}, 'resourceBindings': [{'route': route, 'sha256': sha} for route, sha in sorted(bindings.items())], 'resourceScope': 'The reusable semantic scope binds all 128 required Markdown/llms resources plus actual task readUrls: localized HTML and four downloaded recipe inputs. Separately, artifact-integrity.json checked every manifest byte and browser-tour.json retains actual implicit network resource hashes from this run. Implicit search/JS/CSS resources and unvisited generated showcase APKG/JSON are not included in semantic reuse bindings; current source/configuration identity and the independently rerun engineering browser/search receipts cover those surfaces.', 'executionScope': conclusions['scope'], 'openIssues': [], 'entries': entries}
if environment == 'prefix':
    report['layoutReview'] = {'conclusion': conclusions['layoutConclusion'], 'evidence': [proof(directory / 'layout-final-prefix/accessibility-observations.json', 'Actual final prefix pt-BR layout, keyboard, short viewport and JavaScript-disabled observations.'), proof(directory / 'eight-language-final-prefix/fixture-observations.json', 'Actual final prefix native-details seven-target fixture focus/hit testing at both narrow widths and text scales.')]}
output = directory / f'usage-tasks-{environment}.json'
output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(output.relative_to(root)), 'sha256': digest(output.read_bytes()), 'tasks': sum(len(e['taskResults']) for e in entries.values()), 'bindings': len(bindings), 'machineScope': len(machine), 'origin': tour['readOrigin']}))
