"""Bind explicit old-answer lineage and independently authored German answers.

Run only after the reviewer has read corrected actual HTTP MIME paragraphs.
Required separately written closure evidence prevents premature assembly.
"""
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path.cwd()
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01')
DE = AREA / 'de'
ENV = sys.argv[1]
assert ENV in ('prefix', 'root')
load = lambda filename: json.loads((ROOT / filename).read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
proof = lambda filename: {'path': str(filename), 'sha256': sha((ROOT / filename).read_bytes())}
capture_path = DE / f'machine-final-{ENV}/http-observations.json'
capture = load(capture_path)
manifest_path = DE / f'final-{ENV}-origin-manifest.json'
assert load(manifest_path)['commit'] == '7b5f539ec2b8c34a0a9ac052531cda9571e97725'
assert capture['manifestSha256'] == proof(manifest_path)['sha256']
closure_path = DE / f'artifact-mime-closure-final-{ENV}.json'
closure = load(closure_path)
assert closure['status'] == 'closed'
assert closure['reviewedCapture'] == proof(capture_path)
assert closure['reviewedMimePages'] == ['python-api', 'node-api', 'rust-api', 'media', 'rust-guide']
assert closure['mimeHttpResponsesRead'] == 10
comparison_path = DE / f'machine-final-{ENV}/prior-response-comparison.json'
comparison = load(comparison_path)
assert comparison['currentCapture'] == proof(capture_path)
prior_conclusions_path = AREA / f'ja/machine-conclusions-{ENV}.json'
prior_report_path = AREA / f'ja/machine-read-{ENV}.json'
prior = load(prior_conclusions_path)
assert comparison['priorConclusions'] == proof(prior_conclusions_path)
assert comparison['priorReport'] == proof(prior_report_path)
answers = copy.deepcopy(prior['answers'])
answers['de'] = copy.deepcopy(load(DE / 'machine-german-answers.json')['answers'])
assert set(answers) == set(capture['releaseLocales'])
supporting = [
    {'path': str(comparison_path), 'description': 'Actual old-five-language response comparison: 50 followed Markdown responses unchanged and 20 llms starts with the exact read Deutsch entry addition.'},
    {'path': str(prior_conclusions_path), 'description': 'Original independently authored twenty old-five-language task answers and their complete prior review-method lineage, retained verbatim.'},
    {'path': str(prior_report_path), 'description': 'Accepted prior-five-language report binds those retained answers to their actual original HTTP source artifacts.'},
    {'path': str(DE / 'machine-german-answers.json'), 'description': 'Four independently authored German answers after full actual German HTTP body/code reading; no SDK rerun claimed.'},
    {'path': str(DE / 'mime-final-prefix-http-delta.json'), 'description': 'Exact original-to-corrected prefix response differences, preserving the earlier full-reading evidence and limiting changes to the two MIME paragraphs.'},
    {'path': str(closure_path), 'description': 'Independent actual corrected MIME paragraph reading and source/artifact finding closure.'},
    {'path': str(DE / 'technical-source-review-mime-fixed.json'), 'description': 'Independent Node/Python two-entry MIME source correction preserving the original full 204-entry technical review.'},
    {'path': str(DE / 'technical-source-review-mime-rust.json'), 'description': 'Independent Rust API single-entry correction and whole German MIME-page scope check, preserving both earlier source-review layers.'},
    {'path': str(DE / f'mime-green-final-{ENV}.json'), 'description': 'Actual extra HTTP captures of five German MIME pages in HTML and Markdown, including Rust API outside the four standard tasks.'},
]
if ENV == 'prefix':
    method = (
        'Twenty-four current HTTP tasks start at their same-locale llms index. '
        'Four German answers were independently authored after reading all actual '
        'German task responses and complete code from the original 8f prefix '
        'artifact. This reviewer found the German MIME literal rendering defect. '
        'After the narrowly reviewed source correction, every one of the 84 new '
        'prefix responses was compared to that original capture; only two German '
        'API paragraphs change, and both corrected actual paragraphs were read '
        'again. Separately read ten actual HTTP responses covering all five '
        'German MIME-bearing pages, including the Rust API correction beyond '
        'the four task routes; all fifteen literals survive in HTML code and '
        'Markdown code. The twenty prior-five-language answers are retained verbatim after '
        'reading their original conclusions and proving all fifty followed '
        'Markdown responses unchanged from the accepted Japanese release. Twenty '
        'llms starts only add the exact localized Deutsch index line, which was '
        'inspected in each language. This is a real incremental reading, not '
        'automatic inheritance of a previous pass or a claim of repeated full '
        'older-language readings.'
    )
else:
    prefix_conclusions_path = DE / 'machine-conclusions-final-prefix.json'
    prefix = load(prefix_conclusions_path)
    assert prefix['answers'] == answers
    assert len(comparison['deploymentComparison']['responses']) == 84
    supporting.append({'path': str(prefix_conclusions_path), 'description': 'Explicit final-prefix full-reading and MIME-correction answer lineage for all twenty-four answers.'})
    method = (
        'Twenty-four fresh root HTTP tasks start at their same-locale llms index '
        'and follow root-localized Markdown. All 84 actual root response bodies '
        'were compared to the independently reviewed corrected prefix capture; '
        'the only body difference is the exact configured deployment URL prefix. '
        'The actual root German API MIME paragraphs were independently inspected '
        'again through ten extra HTML/Markdown HTTP responses for all five '
        'MIME-bearing pages, including Rust API; all fifteen literal wildcard '
        'code spans survive. The twenty '
        'older-language answers and four German answers retain the explicit '
        'final-prefix reading lineage; no second full German prose reading or '
        'fresh full old-five-language reading is claimed. Independent same-'
        'environment comparison to the prior Japanese release also confirms '
        'that old-five-language Markdown is unchanged and llms adds only Deutsch.'
    )
report = {
    'schemaVersion': 1, 'reviewerRun': f'/root/de_technical_review/de-machine-final-{ENV}-2026-11-01',
    'reviewedCaptures': {ENV: proof(capture_path)}, 'method': method,
    'supportingEvidence': supporting, 'answers': answers,
    'resolvedFindings': [{'id': 'DE-ARTIFACT-001', 'closure': proof(closure_path)}],
    'openIssues': [],
}
output = DE / f'machine-conclusions-final-{ENV}.json'
assert not (ROOT / output).exists(), 'Preserve any already bound final conclusions'
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({**proof(output), 'answers': 24}))
