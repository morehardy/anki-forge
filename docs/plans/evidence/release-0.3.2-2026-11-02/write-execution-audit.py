# Records this reviewer's fresh executions: binds the current repository sources/dependencies
# (same binding list as the previous independent cycle), the registry state at execution time,
# and every produced evidence file.
import json, hashlib, datetime, urllib.request
from pathlib import Path
root = Path('/Users/hp/Desktop/2026/anki-forge')
ev = 'docs/plans/evidence/release-0.3.2-2026-11-02'
sha = lambda p: hashlib.sha256((root/p).read_bytes()).hexdigest()
inputs = json.loads((root/ev/'executions/basic/execution-inputs.json').read_text())
prior = json.loads((root/'docs/plans/evidence/bing-seo-2026-11-02/usage-execution-audit.json').read_text())
assert [b['path'] for b in inputs['sourceBindings']] == [b['path'] for b in prior['sourceBindings']]
assert [b['path'] for b in inputs['dependencyBindings']] == [b['path'] for b in prior['dependencyBindings']]
for b in inputs['sourceBindings'] + inputs['dependencyBindings']:
    assert sha(b['path']) == b['sha256'], f'source changed since execution: {b["path"]}'
def get(url):
    req = urllib.request.Request(url, headers={'User-Agent': 'anki-forge-release-review (independent reviewer)'})
    return json.load(urllib.request.urlopen(req, timeout=30))
registry = {
    'crates.io': [v['num'] for v in get('https://crates.io/api/v1/crates/ankiforge')['versions']],
    'pypi': sorted(get('https://pypi.org/pypi/ankiforge/json')['releases']),
    'npm': get('https://registry.npmjs.org/ankiforge')['dist-tags'],
}
changed = [b['path'] for b in inputs['dependencyBindings'] if b['sha256'] != next(p['sha256'] for p in prior['dependencyBindings'] if p['path'] == b['path'])]
changed += [b['path'] for b in inputs['sourceBindings'] if b['sha256'] != next(p['sha256'] for p in prior['sourceBindings'] if p['path'] == b['path'])]
files = sorted(str(p.relative_to(root)) for d in ['executions'] for p in (root/ev/d).rglob('*') if p.is_file())
files += [f'{ev}/usage-json-inspection.log', f'{ev}/generated-download-inspection.json']
audit = {
    'schemaVersion': 1, 'reviewerRun': '/root/usage_release_review/release-0.3.2-2026-11-02',
    'executedAt': inputs['executedAt'], 'auditedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'description': 'Fresh executions by this independent reviewer for the release-0.3.2 website artifact (commit 75a8748c). Nothing from the previous cycle is rebound: basic Rust/Python/Node public 0.3.1 consumers, duplicate-key troubleshooting reproduction, public JSON recipe v1/v2 in all three SDKs (website/scripts/verify-public-recipe.mjs), the consumer inspector plus this reviewer\'s own Python APKG decoder.',
    'sourceBindings': inputs['sourceBindings'], 'dependencyBindings': inputs['dependencyBindings'],
    'bindingsChangedSincePreviousCycle': changed,
    'bindingChangeAssessment': 'bindings/python/pyproject.toml, bindings/node/package.json and Cargo.lock changed only by the 0.3.1->0.3.2 package version bump of the unreleased workspace packages. The website still teaches, and these executions installed, the published 0.3.1 packages (docs/ai-quickstart.json publicVersion 0.3.1, unchanged). All other bound sources/examples/recipe inputs are byte-identical to the previous cycle.',
    'registryStateAtAudit': registry,
    'publicJsonCompleted': True,
    'readConclusions': {
        'basic': 'Fresh documented cargo add / pip install / npm install of public 0.3.1 (crates.io, files.pythonhosted.org wheel, registry.npmjs.org) each yielded a decoded Spanish hola/hello Basic package: 1 note, 1 card, 0 media.',
        'json': 'Fresh public 0.3.1 Rust/Python/Node recipe runs: six new APKGs decoded by the consumer inspector and independently by this reviewer\'s Python decoder: 3 notes/3 cards/2 media byte-identical to cell.png and tone.wav; repeated invocation refused to overwrite v1.',
        'update': 'All three comparison.json/review.json pairs: RISK.NOTE_CHANGED only (low), identical baseline/candidate counts, publication allowed; GUIDs stable and exactly one note changed between v1 and v2.',
        'troubleshooting': 'Fresh duplicate-key program reproduced NOTE.KEY_DUPLICATE (note key "source:1" already exists); a distinct second key recovered a decoded package containing 2 notes/2 cards.',
        'generatedDownloads': 'The website /generated/*.apkg downloads changed only in build-time modification timestamps; showcase.json only in crateVersion 0.3.1->0.3.2, sourceCommit and dependent digests (see generated-download-inspection.json).'},
    'evidenceFiles': [{'path': f, 'sha256': sha(f)} for f in files],
}
(root/ev/'usage-execution-audit.json').write_text(json.dumps(audit, indent=2) + '\n')
print(json.dumps({'changedBindings': changed, 'registry': registry, 'files': len(files)}, indent=1))
