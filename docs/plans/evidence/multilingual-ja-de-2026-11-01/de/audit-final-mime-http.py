"""Record exact actual HTTP changes from original German read to corrected build."""
import difflib
import hashlib
import json
from pathlib import Path

ROOT = Path.cwd()
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/de')
load = lambda name: json.loads((ROOT / name).read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
proof = lambda name: {'path': str(name), 'sha256': sha((ROOT / name).read_bytes())}
old_path = AREA / 'machine-prefix/http-observations.json'
new_path = AREA / 'machine-final-prefix/http-observations.json'
old = load(old_path)
new = load(new_path)
assert old['configuration'] == new['configuration']
assert old['releaseLocales'] == new['releaseLocales']
comparisons = []
changed = []
for locale, tasks in new['entries'].items():
    originals = {task['id']: task for task in old['entries'][locale]}
    for task in tasks:
        original = originals[task['id']]
        current_reads = [task['index']] + task['followed']
        previous_reads = [original['index']] + original['followed']
        assert len(current_reads) == len(previous_reads)
        for now, before in zip(current_reads, previous_reads):
            assert now['route'] == before['route']
            for response in (now, before):
                assert proof(response['path'])['sha256'] == response['sha256']
                assert (ROOT / response['path']).read_text() == response['body']
            difference = list(difflib.unified_diff(before['body'].splitlines(), now['body'].splitlines(), lineterm=''))
            targeted = locale == 'de' and now.get('topic') in ('python-api', 'node-api')
            if targeted:
                expected = before['body'].replace('image/_', '`image/*`').replace('audio/_', '`audio/*`').replace('video/\\*', '`video/*`')
                assert now['body'] == expected and now['body'] != before['body'], now['route']
                paragraph = next(line for line in now['body'].splitlines() if 'NOTE.MEDIA_USAGE_INVALID' in line)
                assert all(f'`{mime}`' in paragraph for mime in ['image/*', 'audio/*', 'video/*'])
                changed.append({'route': now['route'], 'topic': now['topic'], 'correctedParagraph': paragraph})
            else:
                assert not difference, (locale, now['route'], difference)
            comparisons.append({'locale': locale, 'task': task['id'], 'route': now['route'], 'before': proof(before['path']), 'after': proof(now['path']), 'diff': difference, 'onlyReviewedMimeCodeLiteralChanged': targeted})
assert len(comparisons) == 84 and len(changed) == 2
negative_old_path = AREA / 'search-negative-prefix/http-observations.json'
negative_new_path = AREA / 'search-negative-final-prefix/http-observations.json'
negative_old, negative_new = load(negative_old_path), load(negative_new_path)
assert negative_old['querySetSha256'] == negative_new['querySetSha256']
negative_changes = []
for now_case, old_case in zip(negative_new['cases'], negative_old['cases']):
    for field in ['id', 'query', 'intentId', 'expectedBoundary', 'passed']:
        assert now_case[field] == old_case[field]
    assert [hit['pageId'] for hit in now_case['reads']] == [hit['pageId'] for hit in old_case['reads']]
    for now, before in zip(now_case['reads'], old_case['reads']):
        actual_diff = list(difflib.unified_diff(before['paragraphs'], now['paragraphs'], lineterm=''))
        targeted = now['pageId'] == 'python-api'
        if targeted:
            expected = [paragraph.replace('image/.', 'image/*.').replace('audio/ oder', 'audio/* oder') for paragraph in before['paragraphs']]
            assert now['paragraphs'] == expected and actual_diff
            assert now['mainText'] == before['mainText'].replace('image/.', 'image/*.').replace('audio/ oder', 'audio/* oder')
        else:
            assert now['paragraphs'] == before['paragraphs']
            assert now['mainText'] == before['mainText']
        negative_changes.append({'case': now_case['id'], 'pageId': now['pageId'], 'before': proof(before['path']), 'after': proof(now['path']), 'paragraphDiff': actual_diff, 'onlyReviewedMimeLiteralChanged': targeted})
assert len(negative_changes) == 12
report = {
    'schemaVersion': 1, 'reviewerRun': '/root/de_technical_review/de-mime-http-final-prefix-2026-11-01',
    'scope': 'Exact actual HTTP delta between the independently fully read original German prefix and corrected final prefix. This proof does not replace the semantic reading of its exact displayed corrected paragraphs.',
    'originalCapture': proof(old_path), 'correctedCapture': proof(new_path),
    'machineComparisons': comparisons, 'correctedParagraphs': changed,
    'originalNegativeCapture': proof(negative_old_path), 'correctedNegativeCapture': proof(negative_new_path),
    'negativeComparisons': negative_changes,
}
output = AREA / 'mime-final-prefix-http-delta.json'
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'path': str(output), 'machineResponses': len(comparisons), 'changedMachineBodies': len(changed), 'negativeResults': len(negative_changes)}))
