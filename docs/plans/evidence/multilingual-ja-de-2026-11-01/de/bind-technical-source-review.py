"""Bind this reviewer's completed semantic reading to frozen source contracts.

Not a general approval generator: findings are the separately authored reading
records and delta notes, and older evidence is preserved rather than rewritten.
"""
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
AREA = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01')
DE = AREA / 'de'
RUN = '/root/de_technical_review/de-source-final-2026-11-01'


def load(filename):
    return json.loads((ROOT / filename).read_text())


def binding(filename):
    return {'path': str(filename), 'sha256': hashlib.sha256((ROOT / filename).read_bytes()).hexdigest()}


candidate_path = DE / 'source-candidates-final.json'
prior_path = AREA / 'ja/technical-source-review-final.json'
audit_path = DE / 'source-input-audit-final.json'
audit = load(audit_path)
assert audit['sourceCandidate'] == {**binding(candidate_path), 'all204EntriesCurrent': True}
prior = load(prior_path)
assert audit['priorReview']['sha256'] == binding(prior_path)['sha256']
assert len(audit['priorEntryChecks']) == 170
assert all(entry['priorHashesReproducedAtBaseline'] for entry in audit['priorEntryChecks'])
for item in audit['currentInputBindings']:
    assert binding(item['path'])['sha256'] == item['sha256'], item['path']
readings = load(DE / 'technical-reading-records.json')['entries']
report = load(candidate_path)
report['reviewerRun'] = RUN
execution_path = DE / 'execution-reuse-audit.json'
assert load(execution_path)['status'] == 'passed'
common_delta = (
    'Independently inspected the exact six-file DE delta from accepted commit '
    '270f26aa5653a0f56297f46fd35e586da444622b. Reconstructed all 226 prior input files '
    'and reproduced all prior 170 source/target/technical hashes. The prior five '
    'locales, render/search implementations, schemas and executable programs are '
    'unchanged. Only German model/catalog/brief/prompt/search values and the German '
    'release-menu entry are added; removing 66 DE model literals recovers exact '
    'baseline model bytes. See source-input-audit-final.json and technical-source-delta-notes.md.'
)
execution_evidence = (
    'Actual public SDK execution reuse was independently re-audited, not rerun: '
    'execution-reuse-audit.json verifies 23 current inputs, ten retained APKG '
    'ZIP/Zstd/SQLite/identity/media contents, six compare/review outputs and 14 '
    'logs/install resolutions. Public 0.3.1 install/runtime and executable programs '
    'are unchanged; AI JSON adds only three German prompt maps. The separately '
    'read doc-executions.log records the parent\'s current 14 source-checkout Rust '
    'documentation executions/23 APKG observations; it is not a public SDK rerun.'
)
vocabulary_evidence = (
    'Independently re-read frozen DE vocabulary deltas and complete formal '
    'concepts/compatibility/updates bodies: relationship predicates retain domain '
    'contexts; stable-key terms target concepts/updates without new intent; '
    'package-content wording still needs package context; learning-history terms '
    'target client import limitations and do not promise reset or recovery. '
    'Existing search context/role precedence implementation was read and is '
    'unchanged. Withheld query files were not read for this source approval; '
    'actual negative-result semantics remains a separate artifact review.'
)
count = 0
for topic, translations in report['entries'].items():
    for locale, entry in translations.items():
        count += 1
        if locale == 'de':
            key = 'blog-list' if topic == 'blog' else topic
            evidence = [
                readings[key]['finding'],
                'Independent complete German reading against the actual English '
                'source and relevant three-SDK implementation contracts is recorded '
                'in technical-reading-records.json and the 139-file '
                'technical-input-hashes.json. All 33 formal German files exactly '
                'match the reviewed staging snapshot; all source fences/code '
                'blocks and 210 UI/11 framework key-placeholder contracts passed '
                'independent corroboration. Metadata/brief/prompt values match '
                'those read. The internal-tools consumer API boundary in media '
                'was corrected and re-read before this binding.',
                common_delta, vocabulary_evidence, execution_evidence,
            ]
        else:
            previous = prior['entries'][topic][locale]['technical']
            assert previous['status'] == 'reviewed' and not previous['openIssues']
            evidence = copy.deepcopy(previous['evidence'])
            evidence.extend([
                f'Existing {topic}/{locale} full independent technical review '
                f'and all earlier evidence above are preserved verbatim from '
                f'{prior_path}, sha256 {binding(prior_path)["sha256"]}. '
                'This round reviewed actual DE/shared deltas and unchanged-input '
                'proof; it does not claim a fresh full reading of this unchanged '
                'older-language body.',
                common_delta, execution_evidence,
                'Current hashes are bound only after reproducing the prior '
                'accepted contracts and inspecting why they change: the added '
                'Deutsch menu affects rendered model hashes, and the added DE '
                'assistant maps affect global technical input hashes. Existing '
                'locale-specific prose, vocabulary and brief remain identical.',
            ])
        entry['technical'] = {'status': 'reviewed', 'evidence': evidence, 'openIssues': []}
        entry['linguistic'] = {'status': 'pending', 'score': {}, 'evidence': [], 'openIssues': []}
assert count == 204
report['reviewScope'] = {
    'axis': 'technical', 'newLocale': 'de', 'fullyReadNewLocaleTopics': 34,
    'incrementallyReviewedExistingEntries': 170,
    'independentOfContentAuthor': True, 'independentSearchQueriesRead': False,
    'immutableBaseline': audit['immutableBaseline'],
    'sourceCandidate': binding(candidate_path), 'priorReview': binding(prior_path),
    'priorReviewScopePreserved': prior['reviewScope'],
    'inputAudit': binding(audit_path),
    'independentFullReadInputs': binding(DE / 'technical-input-hashes.json'),
    'independentTopicReadings': binding(DE / 'technical-reading-records.json'),
    'semanticDeltaNotes': binding(DE / 'technical-source-delta-notes.md'),
    'executionReuseAudit': binding(execution_path),
    'separateCurrentDocumentationExecutionLog': binding(DE / 'doc-executions.log'),
    'closedFindings': [{'id': 'DE-TECH-001', 'severity': 'minor', 'resolution': 'Explicit internal-tools feature and unsupported consumer API boundary for tools::load_project on German media page; re-read against actual lib.rs.'}],
    'openIssues': [],
    'limits': 'Source-only technical approval. No product/staging/shared/index/dist edit, build, commit or new public SDK execution by this reviewer. Browser/ranking/machine/negative-query acceptance requires separate actual-output review.',
}
output = DE / 'technical-source-review-final.json'
(ROOT / output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({**binding(output), 'technicalEntries': count, 'newGerman': 34, 'preservedIncremental': 170}, ensure_ascii=False))
