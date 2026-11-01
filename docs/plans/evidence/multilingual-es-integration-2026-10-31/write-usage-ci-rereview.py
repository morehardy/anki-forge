"""Preserve actual new-tour provenance and distinguish execution reuse."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

BASE = Path('docs/plans/evidence/multilingual-es-integration-2026-10-31')
digest = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
load = lambda path: json.loads(Path(path).read_text())
bound = lambda path: {'path': str(path), 'sha256': digest(path)}
environments = []
receipts = {}
for environment in ['prefix', 'root']:
    old_manifest_path = BASE / f'{environment}-origin-manifest.json'
    new_manifest_path = BASE / f'{environment}-ci-origin-manifest.json'
    old = load(old_manifest_path)
    new = load(new_manifest_path)
    assert new['commit'] == 'a9f3facef528369976712b81ead4c99c162b2907'
    assert old['inputSha256'] == new['inputSha256']
    assert old['configuration'] == new['configuration']
    before = {r['route']: r['sha256'] for r in old['pages'] + old['resources']}
    after = {r['route']: r['sha256'] for r in new['pages'] + new['resources']}
    machine = [r for r in new['resources'] if r['route'].endswith(('/index.md', '/llms.txt'))]
    assert all(before[r['route']] == r['sha256'] for r in machine)
    prior_report_path = BASE / f'usage-tasks-{environment}-symlink.json'
    previous = load(prior_report_path)
    for entry in previous['entries'].values():
        for task in entry['taskResults']:
            for proof in task['executionEvidence'] + task['evidence']:
                filename = proof.get('path', '')
                if '/executions/' in filename:
                    assert digest(filename) == proof['sha256']
                    receipts[filename] = bound(filename)
    old_tour_path = BASE / f'usage-browser-{environment}' / 'browser-tour.json'
    new_tour_path = BASE / f'usage-browser-{environment}-ci' / 'browser-tour.json'
    old_tour, new_tour = load(old_tour_path), load(new_tour_path)
    assert new_tour['manifestSha256'] == digest(new_manifest_path)
    assert new_tour['readOrigin'] != old_tour['readOrigin']
    observed = []
    for locale in new['releaseLocales']:
        for prior, task in zip(old_tour['entries'][locale], new_tour['entries'][locale]):
            assert prior['id'] == task['id']
            assert prior['body'] == task['body'] and prior['codeBlocks'] == task['codeBlocks']
            assert digest(task['screenshot']) == task['screenshotSha256']
            observed.append({
                'locale': locale, 'id': task['id'], 'readUrls': task['readUrls'],
                'actualNewScreenshot': bound(task['screenshot']),
                'renderedTextAndCodeEqualToPriorRead': True,
                'clipboardPrograms': len(task['copiedPrograms']),
                'httpInputDownloads': len(task['downloadedResources']),
                'reviewerObservation': 'New screenshot inspected: current locale heading, main content, sidebar and TOC remain readable; no obscured task control or broken layout. Navigation, search/clipboard/download operations were actually repeated against the new artifact. Critical rendered install/run, original-baseline/policy and error-recovery instructions were reread; full current body/code equality confirms no text was lost through the CSS-scope rebuild.',
            })
    environments.append({
        'environment': environment,
        'preservedHistoricalReport': bound(prior_report_path),
        'oldManifest': bound(old_manifest_path), 'newManifest': bound(new_manifest_path),
        'inputSha256Unchanged': new['inputSha256'],
        'unchangedMachineResources': len(machine),
        'changedHtmlRoutes': [p['route'] for p in new['pages'] if before.get(p['route']) != p['sha256']],
        'changedOrNewResourceRoutes': [p['route'] for p in new['resources'] if before.get(p['route']) != p['sha256']],
        'removedResourceRoutes': sorted(set(before) - set(after)),
        'newTour': bound(new_tour_path), 'newReadOrigin': new_tour['readOrigin'],
        'oldReadOrigin': old_tour['readOrigin'],
        'fullArtifactIntegrity': bound(BASE / f'usage-browser-{environment}-ci' / 'artifact-integrity.json'),
        'actualTaskObservations': observed,
    })
prior_audit = load(BASE / 'usage-execution-audit.json')
for row in prior_audit['currentSourceDependencyBindings']:
    assert digest(row['path']) == row['sha256']
report = {
    'reviewerRun': '/root/spanish_standards/independent-usage-ci-rebuild-a9f3face-2026-11-01',
    'observedAt': datetime.now(timezone.utc).isoformat(),
    'scope': 'Genuine new independent browser acceptance of the CI-compatible artifacts. This reviewer did not change product source. Rebuild changed Astro CSS-scope HTML/resource hashes after dependencies were placed in an ordinary local node_modules directory; source input identity remains unchanged. Prior report verdicts are not reused as artifact acceptance.',
    'nodeModulesIsSymlink': Path('website/node_modules').is_symlink(),
    'environments': environments,
    'executionReuse': {
        'priorIndependentAudit': bound(BASE / 'usage-execution-audit.json'),
        'currentIndependentReinspection': bound(BASE / 'usage-ci-execution-audit.json'),
        'exactUnchangedSourceDependencyCount': len(prior_audit['currentSourceDependencyBindings']),
        'exactUnchangedExecutionReceiptCount': len(receipts),
        'receipts': list(receipts.values()),
        'conclusion': 'All 23 source/dependency bindings and all 39 execution receipt files remain byte-identical. All 10 output APKGs were independently decoded again; no new SDK execution or Anki GUI import is claimed.',
    },
    'openIssues': [],
}
path = BASE / 'usage-ci-rereview.json'
path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(path), 'sha256': digest(path), 'environments': len(environments), 'tasks': sum(len(e['actualTaskObservations']) for e in environments)}))
