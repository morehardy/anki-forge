"""Serialize /root/spec_review's actual independent readings; never runs or changes builds."""
from pathlib import Path
from urllib.parse import urlparse
import copy
import hashlib
import json

root = Path(__file__).resolve().parents[4]
batch = Path('docs/plans/evidence/multilingual-review-fixes-2026-11-01')
run = '/root/spec_review/independent-usage-review-fixes-2026-11-01'

def read(p):
    return json.loads((root / p).read_text())

def digest(p):
    return hashlib.sha256((root / p).read_bytes()).hexdigest()

def htext(value):
    return hashlib.sha256(value.encode()).hexdigest()

def save(p, data):
    (root / p).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')

def ref(p, description):
    return {'path': str(p), 'sha256': digest(p), 'description': description}

audit = read(batch / 'usage-execution-reuse-audit.json')
for binding in audit['bindings'] + audit['executionFiles']:
    assert digest(binding['path']) == binding['sha256'], binding['path']

tours = {c: read(batch / f'usage-{c}/browser-tour.json') for c in ['prefix', 'root']}
comparisons = []
for locale in tours['prefix']['releaseLocales']:
    for p, r in zip(tours['prefix']['entries'][locale], tours['root']['entries'][locale]):
        assert p['id'] == r['id']
        for key in ['body', 'codeBlocks', 'copiedPrograms', 'h1', 'screenshotSha256']:
            assert p[key] == r[key], (locale, p['id'], key)
        assert [(x['anchorText'], x['sha256'], x['bytes'], x['httpStatus'], x['contentType']) for x in p['downloadedResources']] == [(x['anchorText'], x['sha256'], x['bytes'], x['httpStatus'], x['contentType']) for x in r['downloadedResources']]
        comparisons.append({'locale': locale, 'task': p['id'], 'prefixUrl': p['finalUrl'], 'rootUrl': r['finalUrl'], 'bodySha256': htext(p['body']), 'codeBlocksSha256': htext(json.dumps(p['codeBlocks'], ensure_ascii=False)), 'copiedPrograms': len(p['copiedPrograms']), 'downloadedResources': len(p['downloadedResources']), 'screenshotSha256': p['screenshotSha256'], 'equal': True})
    p, r = tours['prefix']['homeObservations'][locale], tours['root']['homeObservations'][locale]
    for key in ['heading', 'primaryMetric', 'rangeMetric', 'description', 'rows', 'performanceText', 'screenshotSha256']:
        assert p[key] == r[key]
for group in ['japaneseClipboard', 'wordForms']:
    for p, r in zip(tours['prefix']['repairObservations'][group], tours['root']['repairObservations'][group]):
        assert p['screenshotSha256'] == r['screenshotSha256']
comparison_path = batch / 'usage-dual-configuration-comparison.json'
save(comparison_path, {'schemaVersion': 1, 'reviewerRun': run, 'scope': 'Both configurations were independently opened and exercised in Chromium. Compared all captured task main text, code, clipboard programs, downloaded file identity, headings and screenshot bytes. Fresh semantic prose reading of all 36 prefix task pages applies to the byte-equal root captures; this is not a claim of a separate duplicate full reread.', 'status': 'equal', 'entries': comparisons, 'allSixHomeObservationsEqual': True, 'allFiveRepairScreenshotsEqual': True})

# These judgments were authored after actually reading all six localized current
# task pages per locale, the six performance panels and the repair observations.
semantics = {
    'rust-basic': 'The localized instructions give Rust 1.92+, public ankiforge 0.3.1, complete src/main.rs, cargo run and a persistent spanish.apkg containing hola → hello in Spanish. They distinguish exporting without Anki from opening/importing the resulting package. Namespace spanish and logical key es:hola remain stable. The real clipboard matched the current shared Rust program. The earlier public-package execution is reused after independent current-byte verification; it was not rerun.',
    'python-basic': 'The localized instructions create/activate a virtual environment, give the Windows activation path, install public ankiforge==0.3.1, and require ordinary CPython 3.11+ with a matching wheel (recorded checks cover 3.11–3.14). main.py generates persistent spanish.apkg with stable namespace/key. Localized search actually reached this page and the real clipboard matched the current Python program. Earlier execution is explicitly reused, not rerun.',
    'node-basic': 'The localized page requires Node 22.13+, includes optional native dependencies in the public 0.3.1 installation, saves main.mjs, uses ESM with an explicit CommonJS alternative, and explains that closing the persistent artifact leaves its file. It links the complete TypeScript path in the JSON recipe. Actual sidebar navigation and clipboard matched the shared Node program. Earlier execution is explicitly reused, not rerun.',
    'json-media': 'The localized recipe identifies all four files under recipe/, 3 notes/3 cards/2 media, a cell image and generated tone rather than spoken vocabulary; it leaves schema validation to the application. It provides complete public-0.3.1 Rust/Python/TypeScript programs and launch commands. Actual HTTP reads downloaded the four source-identical inputs, and all three complete programs were copied correctly. The prose keeps the original v1 APKG, reviews an answer-only v2 with unchanged stable IDs, checks policy before publishing the prepared candidate, avoids overwriting a previous output directory, and describes Anki import/merge limitations and deletion non-synchronization. Prior three-SDK recipe execution is bound and reused; no new SDK execution or native Anki import was performed.',
    'original-baseline-update': 'The localized update page distinguishes persistent namespace/note/model/field/template keys from display names, requires the original distributed APKG identity baseline, and rejects Anki re-export as that baseline. Successful comparison is distinct from publication permission; High/Critical deny by default, category allowances preserve diagnostics, and missing/corrupt/namespace-invalid/oversize identity stays an operation error. The candidate reviewed must be the candidate published, the prior baseline is archived separately, and removal is not deletion synchronization. The complete source example is identified as pinned source 1199196 rather than falsely bundled in public 0.3.1. User edits/history are not guaranteed. Prior answer-only JSON update execution is explicitly reused under unchanged bindings.',
    'troubleshooting': 'The localized page distinguishes structured error kind/code/cause from display text and an operation error from a successful but blocked comparison. NOTE.KEY_DUPLICATE directs a unique key per logical source record. It describes atomic add failure and correction/retry, direct media filename/MIME constraints, the original baseline requirement, retained temporary artifact ownership and persistence errors after publication. The prior duplicate-key/correction execution log was read and byte-verified for reuse; no new runtime correction was executed.',
}
needle = {'rust-basic': '1.92', 'python-basic': '3.11', 'node-basic': '22.13', 'json-media': 'tone.wav', 'original-baseline-update': 'spanish-v1.apkg', 'troubleshooting': 'NOTE.KEY_DUPLICATE'}
semantic_entries = {}
for locale, tasks in tours['prefix']['entries'].items():
    semantic_entries[locale] = {}
    for task in tasks:
        if task['id'] == 'troubleshooting':
            quote = next(x for x in task['body'].split('\n') if x.startswith('NOTE.KEY_DUPLICATE'))
        else:
            quote = next(x for x in task['body'].split('\n\n') if needle[task['id']] in x)
        semantic_entries[locale][task['id']] = {'headingActuallyRead': task['h1'], 'representativeCurrentTextActuallyRead': quote, 'conclusion': semantics[task['id']], 'bodySha256': htext(task['body']), 'scope': 'All task prose in current prefix HTML read; code/clipboard/downloads examined. Root independently navigated and captured, with entire main text/code equal under the dual-configuration comparison.'}

viewed = []
for locale in tours['prefix']['releaseLocales']:
    s = tours['prefix']['homeObservations'][locale]
    viewed.append(ref(s['screenshot'], f'Actually visually inspected the {locale} performance panel: decimal values, description, all five bar rows and labels are legible. The independently captured root screenshot is byte-identical.'))
for s in tours['prefix']['repairObservations']['japaneseClipboard']:
    viewed.append(ref(s['screenshot'], 'Actually visually inspected Japanese action and manual-selection feedback with selected Rust code; selection matches shared source. The root screenshot is byte-identical.'))
for s in tours['prefix']['repairObservations']['wordForms']:
    viewed.append(ref(s['screenshot'], f'Actually visually inspected query {s["query"]}: expected result is visible with a relevant inflected excerpt and localized message. The root screenshot is byte-identical.'))
semantic_path = batch / 'usage-independent-conclusions.json'
save(semantic_path, {'schemaVersion': 1, 'reviewerType': 'ai', 'reviewerRun': run, 'scope': 'Independent local usage acceptance of isolated commit 6d4625639fc1477222b41880d29724c162baa3be plus the seven files frozen in implementation-inputs.json. Does not cover concurrent edits in the original workspace or deployment.', 'taskReadings': semantic_entries, 'visualReview': viewed, 'repairs': {'numbers': 'Actually read all six rendered panels: es/pt-BR/de use 52,7, 45,7–53,9 and comma decimals in all five chart rows; en/zh-CN/ja use periods. Percent reductions, units, workload labels and scope disclaimer agree with the source data. Bar geometry remains plausible and unchanged across both builds.', 'japaneseCopy': 'Both actual article CodeBlocks expose コードをコピー and rust のコード; success/recovery say コードをコピーしました。; denied clipboard access selects precisely the source and says コードを選択しました。キーボードでコピーしてください。. Explicit browser-context permissions before each article prevent CDP permission state from contaminating the next navigation. Initial harness timeout is retained in usage-prefix-initial and is not a product failure.', 'wordForms': 'Both builds show 11 results for es reutilización with templates first, 1 for de Nebenläufig with Python-API, and 11 for de Zeichenfolge with Python-API third. All result URLs stay in the query locale. Screenshots show the relevant Reutiliza/Nebenläufigkeit/Zeichenfolgen excerpts. This verifies the reported native Pagefind word-form regression; it is not a new exhaustive query evaluation.'}, 'executionReuseAudit': ref(batch / 'usage-execution-reuse-audit.json', 'Every one of 23 source/dependency files and 8 retained execution artifacts was independently hash-checked again in the isolated root. Five logs were actually read. No SDK rerun or fresh native Anki import claimed.'), 'crossConfigurationComparison': ref(comparison_path, 'All 36 actual task body/code/clipboard/screenshot sets and downloaded identities match; six home and five repair screenshots also match.'), 'openIssues': [], 'limitations': ['Local candidate only; no deployed-page claim.', 'SDK execution and APKG inspection conclusions are inherited under explicit unchanged-source/dependency bindings; the browser actions and downloads are new.', 'No new 320px/200% layout run; historical minor/preference observations are retained in the final reports.', 'All 703 artifact records are byte-bound, including 192 machine resources. Machine-content semantic reading is a separate independent review.']})

for config, tour in tours.items():
    manifest_path = batch / f'{config}-site-manifest.json'
    manifest = read(manifest_path)
    assert tour['manifestSha256'] == digest(manifest_path)
    assert not tour['pageErrors'] and not tour['responseReadErrors']
    records = {x['route']: x for x in manifest['pages'] + manifest['resources']}
    old_path = Path(f'docs/plans/evidence/multilingual-ja-de-2026-11-01/de/usage-tasks-final-{config}.json')
    old = read(old_path)
    entries = {}
    for locale, tasks in tour['entries'].items():
        results = []
        for task in tasks:
            previous = next(x for x in old['entries'][locale]['taskResults'] if x['id'] == task['id'])
            execution = copy.deepcopy(previous.get('executionEvidence', []))
            for e in execution:
                assert digest(e['path']) == e['sha256']
                for binding in e['sourceBindings'] + e['dependencyBindings']:
                    assert digest(binding['path']) == binding['sha256']
                e['description'] = 'Explicit reuse of this retained public-package execution artifact after independent current SHA-256 verification of every sourceBinding and dependencyBinding; see usage-execution-reuse-audit.json. Existing logs were read; SDK execution and APKG decoding were not rerun in this review.'
            read_urls = list(task['readUrls']) + [d['url'] for d in task['downloadedResources']]
            steps = list(task['steps']) + [semantics[task['id']], 'Read the current localized HTML prose and examined captured code/clipboard/downloads; see usage-independent-conclusions.json for task-specific actual text and the exact root/prefix comparison.']
            if config == 'root':
                steps.append('Independently performed this navigation/copy/download task against root-origin HTML. Compared the entire actual main text, code, copied programs and downloaded identity with the freshly read prefix task: equal, so the same semantic reading applies.')
            route = urlparse(task['finalUrl']).path
            evidence = [
                {'route': route, 'sha256': records[route]['sha256'], 'observation': 'Actual localized task HTML was opened and read; ' + semantics[task['id']]},
                ref(batch / f'usage-{config}/browser-tour.json', f'New {config} actual browser steps, localized main text/code, clipboard bytes, downloads and read URLs; see entries[{locale}] task {task["id"]}.'),
                ref(semantic_path, f'Independent current semantic conclusion and actual localized quote for {locale}/{task["id"]}; explicit reuse limits.'),
                ref(batch / 'usage-execution-reuse-audit.json', 'Fresh independent byte audit supports only explicit retained SDK execution reuse.'),
                ref(comparison_path, 'New independent browser runs produce equal task main text/code/clipboard/downloads across both configurations.'),
                ref(task['screenshot'], 'New actual browser screenshot captured for this task. Task prose was read as text; this screenshot is retained for reproducibility, not claimed as separately visually inspected.'),
            ]
            if task['id'] == 'rust-basic':
                home = tour['homeObservations'][locale]
                steps.append(f'Actually read the current {locale} performance panel: {home["primaryMetric"]}, {home["rangeMetric"]}; all five timing rows and explanatory percent values use the current locale decimal format.')
                evidence.append(ref(home['screenshot'], 'Current rendered performance panel. Prefix image actually visually inspected; root image independently captured and verified byte-identical.'))
            if locale == 'ja' and task['id'] in ['rust-basic', 'original-baseline-update']:
                probe = tour['repairObservations']['japaneseClipboard'][0 if task['id'] == 'rust-basic' else 1]
                read_urls.append(probe['url'])
                steps.append('Also opened the corresponding Japanese article and exercised real clipboard success, denied/manual selection, and restored permission. Japanese action/accessible label/feedback are correct; selected and copied code equals the shared source.')
                evidence.append(ref(probe['screenshot'], 'Actual denied clipboard state with Japanese manual-copy instruction. Prefix screenshot visually inspected; independently captured root image byte-identical.'))
            if task['id'] == 'python-basic' and locale in ['es', 'de']:
                for probe in tour['repairObservations']['wordForms']:
                    if probe['locale'] != locale:
                        continue
                    read_urls.append(probe['pageUrl'])
                    steps.append(f'Also entered {probe["query"]!r} in the actual localized search UI: {probe["message"]}; expected {probe["expected"]} is present and every result remains in this locale. Result snippets were read; result destinations were not all opened.')
                    evidence.append(ref(probe['screenshot'], 'Current word-form search UI and relevant excerpt. Prefix screenshot visually inspected; root capture byte-identical.'))
            results.append({'id': task['id'], 'status': 'passed', 'openIssues': copy.deepcopy(previous['openIssues']), 'readUrls': list(dict.fromkeys(read_urls)), 'steps': steps, 'executionEvidence': execution, 'evidence': evidence})
        entries[locale] = {'taskResults': results, 'openIssues': copy.deepcopy(old['entries'][locale]['openIssues'])}
    issues = copy.deepcopy(old['openIssues'])
    for issue in issues:
        issue['historicalScope'] = 'Preserved from prior independent usage review; not introduced or silently cleared by this delta. The narrow 320px/200% view was not rerun. Current word-form results also retain the known Spanish home-title br-boundary concatenation, while the task result remains usable.'
        if issue.get('evidence', {}).get('path'):
            assert digest(issue['evidence']['path']) == issue['evidence']['sha256']
    result = {
        'schemaVersion': 1, 'kind': 'usage-tasks', 'status': 'passed', 'mode': 'release-candidate', 'reviewerType': 'ai', 'reviewerRun': run + '/' + config,
        'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'], 'releaseLocales': manifest['releaseLocales'], 'readOrigin': tour['readOrigin'],
        'originManifest': {'path': str(manifest_path), 'sha256': digest(manifest_path), 'commit': manifest['commit']},
        'resourceBindings': [{'route': r, 'sha256': v['sha256']} for r,v in records.items()],
        'resourceScope': 'Every 703-record artifact byte was independently compared with the actual frozen manifest before browsing, including all 192 Markdown/llms resources. Semantic usage scope: six tasks in each of six locales, all six homepage performance panels, two Japanese article CodeBlocks and three word-form search probes. 189 unique actual loaded browser resources were response-hash checked. Machine semantic conclusions belong to the separate machine-read reviewer.',
        'executionScope': '36 actual localized browser tasks, 36 complete program clipboard copies and 24 input-file HTTP downloads per configuration, plus repair probes. Every retained SDK execution source/dependency binding was independently rehashed (23 unique bindings, 8 execution files, 5 logs read). Reused execution is explicitly marked; no SDK rerun, fresh APKG decode, native Anki import or deployed-site check claimed.',
        'candidateScope': 'Isolated baseline 6d4625639fc1477222b41880d29724c162baa3be plus the seven frozen implementation-inputs.json files. Excludes concurrent edits in the original workspace.',
        'sourceReviews': [ref(batch / 'linguistic-source-review.json', 'Independent delta language/source assessment; unchanged previous nonblocking scores/observations retained.')],
        'independentConclusions': ref(semantic_path, 'Actual semantic readings, visual observations, repair results and explicit limitations.'),
        'crossConfigurationComparison': ref(comparison_path, 'Entire captured task text/code/clipboard/downloads equal after independent browser runs.'),
        'previousReview': ref(old_path, 'Retained SDK execution provenance and historical minor/preference issues only; current reading/navigation evidence is newly recorded.'),
        'openIssues': issues, 'entries': entries,
        'visualReview': {'scope': 'All six performance panels, two denied Japanese clipboard states and three word-form search screenshots actually viewed from prefix; independent root captures match their bytes. Other task screenshots captured, not claimed as individually visually inspected.', 'evidence': viewed},
    }
    save(batch / f'usage-tasks-{config}.json', result)
    print(config, len(entries), sum(len(e['taskResults']) for e in entries.values()), 'tasks;', len(result['resourceBindings']), 'bindings;', digest(batch / f'usage-tasks-{config}.json'))
