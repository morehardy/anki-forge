"""Record the actual root HTTP delta after inspecting its exact changed text."""
import hashlib
import json
from pathlib import Path

directory = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja')
read = lambda path: json.loads(path.read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
proof = lambda path: {'path': str(path), 'sha256': sha(path.read_bytes())}
prefix_path = directory / 'machine-prefix/http-observations.json'
root_path = directory / 'machine-root/http-observations.json'
prefix, current = read(prefix_path), read(root_path)
old_base = prefix['configuration']['site'] + prefix['configuration']['base']
new_base = current['configuration']['site']
assert current['configuration']['base'] == '/'
assert prefix['inputSha256'] == current['inputSha256']
assert prefix['releaseLocales'] == current['releaseLocales']
comparisons = []
for locale, tasks in current['entries'].items():
    for task in tasks:
        before = next(item for item in prefix['entries'][locale] if item['id'] == task['id'])
        for left, right in zip([before['index']] + before['followed'], [task['index']] + task['followed'], strict=True):
            assert left.get('topic') == right.get('topic')
            assert left['body'].replace(old_base, new_base) == right['body']
            for item in [left, right]:
                assert sha(Path(item['path']).read_bytes()) == item['sha256']
                assert Path(item['path']).read_text() == item['body']
            comparisons.append({'locale': locale, 'task': task['id'], 'topic': right.get('topic', 'llms'), 'previousUrl': left['url'], 'previousSha256': left['sha256'], 'currentUrl': right['url'], 'currentSha256': right['sha256'], 'equalAfterExactDeploymentUrlReplacement': True})
assert len(comparisons) == 70
comparison_path = directory / 'machine-root/content-comparison.json'
comparison = {'schemaVersion': 1, 'previousCapture': proof(prefix_path), 'currentCapture': proof(root_path),
    'scope': 'Compared all seventy actual HTTP responses for twenty root tasks against this reviewer’s prefix responses. Only the exact configured https://morehardy.github.io/anki-forge -> https://ankiforge.dev replacement is permitted; all prose, code, headings, conditions and locale destinations otherwise match. The root manifest and response bodies are independently byte-bound; this is an exact deployment-delta review, not a second full Japanese prose reading.',
    'from': old_base, 'to': new_base, 'comparisons': comparisons}
comparison_path.write_text(json.dumps(comparison, ensure_ascii=False, indent=2) + '\n')
prefix_conclusions_path = directory / 'machine-conclusions-prefix.json'
conclusions = read(prefix_conclusions_path)
conclusions['reviewerRun'] = '/root/ja_de_infra_audit/ja-machine-root-2026-11-01'
conclusions['reviewedCaptures'] = {'root': proof(root_path)}
conclusions['method'] = 'Twenty new root HTTP tasks start at same-locale llms and follow root-localized Markdown. Every actual response was compared to the prefix artifact this reviewer assessed; the only body difference is the exact configured deployment URL. The prefix conclusions supply four independently authored Japanese answers and sixteen explicitly retained old-language answers with verified prior response lineage. The deployment delta was inspected, not treated as permission to refresh arbitrary evidence. No second full Japanese reading or fresh full reading of unchanged older translations is claimed.'
conclusions['supportingEvidence'] += [{'path': str(comparison_path), 'description': 'All seventy actual prefix/root response bodies differ only by the exact configured deployment URL.'}, {'path': str(prefix_conclusions_path), 'description': 'Preserved answers and full versus incremental semantic-review lineage from this same candidate’s prefix artifact.'}]
(directory / 'machine-conclusions-root.json').write_text(json.dumps(conclusions, ensure_ascii=False, indent=2) + '\n')

prefix_path = directory / 'search-negative-prefix/http-observations.json'
root_path = directory / 'search-negative-root/http-observations.json'
prefix, current = read(prefix_path), read(root_path)
rows = []
for before, after in zip(prefix['cases'], current['cases'], strict=True):
    assert before['id'] == after['id']
    assert before['query'] == after['query']
    assert before['expectedBoundary'] == after['expectedBoundary']
    for left, right in zip(before['reads'], after['reads'], strict=True):
        assert left['pageId'] == right['pageId']
        assert left['paragraphs'] == right['paragraphs']
        original = left['mainText']
        if left['pageId'] == 'home':
            assert original.count('href="/anki-forge/ja/examples/"') == 1
            original = original.replace('href="/anki-forge/ja/examples/"', 'href="/ja/examples/"')
        assert original == right['mainText']
        for item in [left, right]:
            assert sha(Path(item['path']).read_bytes()) == item['sha256']
        rows.append({'case': after['id'], 'pageId': right['pageId'], 'previousUrl': left['actualUrl'], 'previousSha256': left['sha256'], 'currentUrl': right['actualUrl'], 'currentSha256': right['sha256'], 'allParagraphsExactlyEqual': True, 'mainTextDelta': 'One noscript examples href changes from /anki-forge/ja/examples/ to /ja/examples/; inspected and correct.' if left['pageId'] == 'home' else 'None.'})
assert len(rows) == 12
comparison = {'schemaVersion': 1, 'previousCapture': proof(prefix_path), 'currentCapture': proof(root_path),
    'scope': 'Fresh root requests retain the same five queries and twelve actual result rows. All paragraphs are exactly equal to the independently read prefix HTML. All mainText is also equal except one homepage noscript href, whose exact /anki-forge/ja/examples/ -> /ja/examples/ change was inspected and is correct for root deployment. No technical statement, capability boundary, code or result identity changed. This actual delta review retains the manually authored prefix semantic conclusions.', 'rows': rows}
(directory / 'search-negative-root/body-comparison.json').write_text(json.dumps(comparison, ensure_ascii=False, indent=2) + '\n')
print('Reviewed root deltas: seventy machine responses with only exact deployment-URL changes; twelve negative-result bodies with identical paragraphs and one correct homepage noscript href.')
