"""Bind separately authored negative-search semantic findings to actual HTTP."""
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path.cwd()
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/de')
ENV = sys.argv[1]
assert ENV in ('prefix', 'root')
load = lambda filename: json.loads((ROOT / filename).read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()


def proof(filename):
    return {'path': str(filename), 'sha256': sha((ROOT / filename).read_bytes())}


capture_path = AREA / f'search-negative-{ENV}/http-observations.json'
manifest_path = AREA / f'{ENV}-origin-manifest.json'
findings_path = AREA / 'negative-boundary-findings.json'
capture = load(capture_path)
manifest = load(manifest_path)
findings = load(findings_path)
assert capture['manifestSha256'] == proof(manifest_path)['sha256']
assert capture['configuration'] == manifest['configuration']
assert proof(capture['resultPath'])['sha256'] == capture['resultSha256']
records = {record['route']: record for record in manifest['pages']}
report = {
    'schemaVersion': 1, 'kind': 'independent-negative-search-boundary-review',
    'status': 'passed', 'mode': 'release-candidate', 'reviewerType': 'ai',
    'reviewerRun': f'/root/de_technical_review/de-negative-{ENV}-2026-11-01',
    'scope': 'Five frozen German negative queries. Actual top-three result-body prose and tables were independently read for supported-capability boundaries; complete task-related code was separately read in the actual machine responses. This is not ranking or browser-interaction approval. Source withheld queries were not consulted during source/vocabulary review.',
    'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'],
    'releaseLocales': manifest['releaseLocales'], 'readOrigin': capture['readOrigin'],
    'originManifest': {**proof(manifest_path), 'commit': manifest['commit']},
    'searchResults': proof(capture['resultPath']), 'querySetSha256': capture['querySetSha256'],
    'rawHttpObservations': proof(capture_path), 'semanticFindings': proof(findings_path),
    'resourceBindings': [], 'cases': [], 'openIssues': [],
}
assert len(capture['cases']) == 5
seen = set()
for case in capture['cases']:
    finding = findings['cases'][case['id']]
    assert len(case['reads']) == len(case['top3']) == len(finding['hits'])
    results = []
    for hit, meaning in zip(case['reads'], finding['hits']):
        assert hit['pageId'] == meaning['pageId']
        assert meaning['quote'] in hit['mainText']
        assert hit['status'] == 200 and 'text/html' in hit['headers']['content-type']
        assert proof(hit['path'])['sha256'] == hit['sha256'] == records[hit['url']]['sha256']
        results.append({
            'pageId': hit['pageId'], 'url': hit['actualUrl'], 'route': hit['url'],
            'sha256': hit['sha256'], 'bodyEvidence': proof(hit['path']),
            'quote': meaning['quote'], 'observation': meaning['observation'],
        })
        if hit['url'] not in seen:
            report['resourceBindings'].append({'route': hit['url'], 'sha256': hit['sha256']})
            seen.add(hit['url'])
    report['cases'].append({
        'id': case['id'], 'query': case['query'], 'intentId': case['intentId'],
        'expectedBoundary': case['expectedBoundary'], 'rankGatePassed': case['passed'],
        'semanticBoundaryStatus': 'passed', 'conclusion': finding['conclusion'],
        'actualResultBodies': results, 'openIssues': [],
    })
if ENV == 'root':
    prefix_path = AREA / 'search-negative-prefix/http-observations.json'
    prefix = load(prefix_path)
    comparisons = []
    for now, old in zip(capture['cases'], prefix['cases']):
        for field in ['id', 'query', 'intentId', 'expectedBoundary', 'passed']:
            assert now[field] == old[field], (field, now['id'])
        assert len(now['reads']) == len(old['reads'])
        for current, former in zip(now['reads'], old['reads']):
            assert current['pageId'] == former['pageId']
            assert current['mainText'] == former['mainText']
            assert current['paragraphs'] == former['paragraphs']
            comparisons.append({'case': now['id'], 'pageId': current['pageId'], 'rootBody': proof(current['path']), 'prefixBody': proof(former['path']), 'actualMainTextAndParagraphsIdentical': True})
    report['deploymentReview'] = {
        'method': 'Compared all five frozen inputs/boundaries and every actual root result body to the independently read prefix result. Main prose and paragraph text is exactly identical; root HTTP URLs and complete HTML resource hashes are independently bound.',
        'prefixCapture': proof(prefix_path),
        'prefixReview': proof(AREA / 'search-negative-boundary-prefix.json'),
        'comparisons': comparisons,
    }
output = AREA / f'search-negative-boundary-{ENV}.json'
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({**proof(output), 'cases': len(report['cases']), 'readResults': sum(len(case['actualResultBodies']) for case in report['cases']), 'distinctBodies': len(seen)}, ensure_ascii=False))
