"""Compare actual captured machine responses, without generating task answers."""
import difflib
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path.cwd()
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01')
ENV = sys.argv[1]
assert ENV in ('prefix', 'root')


def load(path):
    return json.loads((ROOT / path).read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def proof(path):
    return {'path': str(path), 'sha256': sha((ROOT / path).read_bytes())}


current_path = AREA / f'de/machine-{ENV}/http-observations.json'
prior_path = AREA / f'ja/machine-{ENV}/http-observations.json'
current = load(current_path)
prior = load(prior_path)
assert current['configuration'] == prior['configuration']
assert current['releaseLocales'] == prior['releaseLocales'] + ['de']
records = []
for locale in prior['releaseLocales']:
    previous_tasks = {task['id']: task for task in prior['entries'][locale]}
    for task in current['entries'][locale]:
        previous = previous_tasks[task['id']]
        assert [item['topic'] for item in task['followed']] == [item['topic'] for item in previous['followed']]
        for now, old in zip([task['index']] + task['followed'], [previous['index']] + previous['followed']):
            assert now['route'] == old['route']
            for item in (now, old):
                data = (ROOT / item['path']).read_bytes()
                assert sha(data) == item['sha256']
                assert data.decode('utf-8') == item['body']
                assert item['status'] == 200
            difference = list(difflib.unified_diff(old['body'].splitlines(), now['body'].splitlines(), lineterm=''))
            if now['route'].endswith('/llms.txt'):
                additions = [line for line in difference if line.startswith('+') and not line.startswith('+++')]
                removals = [line for line in difference if line.startswith('-') and not line.startswith('---')]
                assert len(additions) == 1 and not removals, (locale, difference)
                assert additions[0].startswith('+- [Deutsch](') and '/de/llms.txt)' in additions[0]
                judgment = 'Only the exact added Deutsch index line changes; its text/target must be semantically read by the reviewer.'
            else:
                assert not difference, (locale, now['route'], difference)
                judgment = 'Exact current response bytes equal the independently reviewed accepted Japanese-release response.'
            records.append({'locale': locale, 'task': task['id'], 'route': now['route'], 'current': proof(Path(now['path'])), 'prior': proof(Path(old['path'])), 'diff': difference, 'comparison': judgment})
assert len(records) == 70
report = {
    'schemaVersion': 1, 'reviewerRun': f'/root/de_technical_review/de-machine-{ENV}-2026-11-01',
    'currentCapture': proof(current_path), 'priorCapture': proof(prior_path),
    'priorConclusions': proof(AREA / f'ja/machine-conclusions-{ENV}.json'),
    'priorReport': proof(AREA / f'ja/machine-read-{ENV}.json'),
    'scope': 'Exact actual old-five-language HTTP response comparison: 50 followed Markdown responses and 20 llms starts. This records byte deltas; it does not author answers or assert a semantic pass.',
    'comparisons': records,
}
if ENV == 'root':
    prefix_path = AREA / 'de/machine-prefix/http-observations.json'
    prefix = load(prefix_path)
    before_url = prefix['configuration']['site'].rstrip('/') + prefix['configuration']['base'].rstrip('/') + '/'
    after_url = current['configuration']['site'].rstrip('/') + current['configuration']['base'].rstrip('/') + '/'
    changes = []
    for locale, tasks in current['entries'].items():
        prior_tasks = {task['id']: task for task in prefix['entries'][locale]}
        for task in tasks:
            former = prior_tasks[task['id']]
            for now, old in zip([task['index']] + task['followed'], [former['index']] + former['followed']):
                assert now['body'] == old['body'].replace(before_url, after_url), now['route']
                changes.append({'locale': locale, 'task': task['id'], 'current': proof(Path(now['path'])), 'prefix': proof(Path(old['path'])), 'onlyConfiguredUrlPrefixChanged': True})
    assert len(changes) == 84
    report['deploymentComparison'] = {'prefixCapture': proof(prefix_path), 'beforeUrlPrefix': before_url, 'afterUrlPrefix': after_url, 'responses': changes}
output = AREA / f'de/machine-{ENV}/prior-response-comparison.json'
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(output), 'oldLocaleResponses': len(records), 'followedMarkdownIdentical': 50, 'llmsAdditions': 20, 'rootPrefixCompared': 84 if ENV == 'root' else 0}))
