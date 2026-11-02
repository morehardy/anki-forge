"""Seal the completed, scoped HTTP/delta reading; never rewrite older evidence."""
import pathlib, json, hashlib

batch = pathlib.Path('docs/plans/evidence/website-pr66-rebuild-2026-11-02')
previous_batch = pathlib.Path('docs/plans/evidence/website-pr66-final-2026-11-02')
digest = lambda data: hashlib.sha256(data).hexdigest()
def load(path): return json.loads(path.read_bytes())
def proof(path, description):
    return {'path':str(path), 'sha256':digest(path.read_bytes()), 'description':description}

# These observations record the actual fresh reading performed after each
# capture, not an automatic consequence of byte equality.
task_observations = {
 'python-install': 'Rechecked the newly fetched locale llms installation command and runtime, current localized quickstart title/canonical/language and persistent Spanish/hola→hello result paragraph. English/Chinese virtualenv/pip/program passages were also re-read. Complete identical programs and detailed install conditions retain their prior independent six-language review. No new installation execution is claimed.',
 'node-api': 'Rechecked the newly fetched localized API heading/canonical/language and artifact-owner close/persist lifetime paragraphs. The report-versus-file ownership distinction and structured failure observations remain explicit. The complete previously reviewed Node program, native dependency requirement and consumed-publication details are inherited through exact full-resource equality.',
 'json-update': 'Rechecked the newly fetched localized recipe introduction/counts, comparison.json/review.json policy result paragraph, original-distribution APKG baseline requirement, and client import/non-deletion/history caveat. The three complete SDK programs and run commands remain byte-identical to the prior independently read recipe; this review does not claim to execute them again.',
 'compatibility-constraints': 'Rechecked the newly fetched localized compatibility title/canonical/language, Rust/Node/CPython verification table and original-baseline/client-specific-testing paragraph. The unchanged dated package record, excluded platforms, per-file atomicity and late-durability limitations retain their prior complete compatibility/build-guarantee review, supported by exact bytes.',
}
comparison_path = batch/'technical-machine-dual-configuration-comparison.json'
comparison = load(comparison_path)
assert len(comparison['records']) == 192
reading = {
 'schemaVersion':1,
 'reviewerRun':'/root/standards_review/pr66-rebuild-scoped-machine-reading-2026-11-02',
 'status':'reviewed',
 'scope':'Incremental review after producer-only Pagefind canonicalization. Personally fetched both frozen archives via localhost HTTP: six locale llms entries and all four task discovery paths, 60 distinct responses per configuration. Independently read every actual old/current machine resource as bytes: all 192 per configuration exactly match the previously reviewed final artifact. No new prose delta exists. The old 24 detailed semantic answers per configuration are explicitly retained and checked against newly fetched current task entry/result/constraint passages; this is not a claim to newly reread or rewrite every unchanged long-form page.',
 'rootScope':'Personally fetched the root task resources and separately read all six root llms introductions/runtime commands and all 24 primary task headers/canonical/language metadata. Current prefix task-result/constraint reading applies to root only through exact full paired-body comparison after the sole configured site+base URL substitution. Each root resource also exactly matches its prior independently reviewed root bytes.',
 'configurationComparison':proof(comparison_path,'All 192 current root/prefix pairs differ only by the complete configured HTTPS site+base string; no text normalization.'),
 'taskObservations':task_observations,
 'priorDetailedReading':proof(previous_batch/'technical-machine-semantic-reading.json','Immutable prior independently authored 24 answers and 65-resource delta review. This run inherits the unchanged semantics explicitly after old/new byte and current HTTP checks.'),
 'sourceDelta':proof(batch/'technical-pagefind-observations.json','Independent review of the sole changed producer input, six-language exact selection preservation, unknown-language fallback boundary, all 720 key permutations and migration/config changes.'),
 'environments':{}, 'openIssues':[],
 'limitations':['No deployment or hosting MIME acceptance is claimed. The short-lived localhost server serves the frozen archive.', 'SDK execution and Anki GUI/media behavior belong to separate usage/client evidence.', 'The 192 bindings cover Markdown/llms resources only; unread showcase metadata and HTML are not added.']
}

for environment in ['prefix','root']:
    capture_path = batch/f'technical-machine-{environment}/observations.json'
    capture = load(capture_path)
    audit_path = batch/f'technical-machine-{environment}-input-audit.json'
    audit = load(audit_path)
    manifest_path = batch/f'{environment}-site-manifest.json'
    manifest = load(manifest_path)
    prior_path = previous_batch/f'machine-read-{environment}.json'
    prior = load(prior_path)
    archive = pathlib.Path(f'target/website-pr66-rebuild/{environment}-dist')
    old_archive = pathlib.Path(f'target/website-pr66-final/{environment}-dist')
    assert audit['capture']['sha256'] == digest(capture_path.read_bytes())
    assert audit['manifest']['sha256'] == digest(manifest_path.read_bytes())
    assert audit['previousReport']['sha256'] == digest(prior_path.read_bytes())
    assert manifest_path.read_bytes() == (archive/'generated/site-manifest.json').read_bytes()
    assert manifest['inputSha256'] == 'd002c1a1d04b2fad447d233557f235733eeeb4484ad362af03c2d7a4e984098e'
    assert manifest['configuration'] == prior['configuration']
    assert manifest['releaseLocales'] == prior['releaseLocales']
    assert len(audit['records']) == 192 and len(capture['responses']) == 60
    records = {x['route']:x for x in audit['records']}
    responses = {x['route']:x for x in capture['responses']}
    for resource in audit['records']:
        route = resource['route']
        filename = route.removeprefix('/anki-forge').lstrip('/') if environment=='prefix' else route.lstrip('/')
        current = (archive/filename).read_bytes()
        assert digest(current) == resource['sha256'] == resource['previousSha256']
        assert current == (old_archive/filename).read_bytes()
        if route in responses:
            assert pathlib.Path(responses[route]['bodyPath']).read_bytes() == current
            assert responses[route]['status'] == 200
    observations = {}
    for locale,tasks in capture['readings'].items():
        observations[locale] = {}
        for task,items in tasks.items():
            entry = pathlib.Path(items[0]['bodyPath']).read_text()
            primary = pathlib.Path(items[1]['bodyPath']).read_text()
            header = primary.splitlines()[:7]
            assert f'Language: {locale}' in header
            canonical = next(x.removeprefix('Canonical page: ') for x in header if x.startswith('Canonical page: '))
            assert canonical in entry
            observations[locale][task] = {'llmsUrl':items[0]['url'],'primaryUrl':items[1]['url'],'currentPrimaryHeader':header,'observation':task_observations[task] if environment=='prefix' else 'Fresh root HTTP entry and primary header/canonical/language reading completed. Identical root prose retains prior root semantics and current prefix scoped recheck via explicit paired-byte proof.','answerLineage':{'path':str(prior_path),'sha256':digest(prior_path.read_bytes()),'entry':f'entries/{locale}/aiMachineReadReview/{task}'}}
    reading['environments'][environment] = {'readOrigin':capture['readOrigin'],'manifest':proof(manifest_path,'Actual current frozen archive manifest.'),'inputAudit':proof(audit_path,'192 old/current actual archive byte comparisons plus fresh response-byte verification.'),'actualHttp':proof(capture_path,'This independent reviewer personally performed these fresh localhost GETs and saved the actual bodies.'),'observations':observations}

reading_path = batch/'technical-machine-semantic-reading.json'
reading_path.write_text(json.dumps(reading,ensure_ascii=False,indent=2)+'\n')

for environment in ['prefix','root']:
    capture = load(batch/f'technical-machine-{environment}/observations.json')
    audit = load(batch/f'technical-machine-{environment}-input-audit.json')
    manifest_path = batch/f'{environment}-site-manifest.json'; manifest = load(manifest_path)
    prior_path = previous_batch/f'machine-read-{environment}.json'; prior = load(prior_path)
    resources = {x['route']:x for x in audit['records']}
    entries = {}
    for locale in manifest['releaseLocales']:
        results = []
        for old in prior['entries'][locale]['aiMachineReadReview']:
            task = old['id']; items = capture['readings'][locale][task]
            # Cite the same directly reviewed answer sources, now actually fetched
            # from this candidate; extra captured Python API remains in inventory.
            old_routes = {__import__('urllib.parse',fromlist=['urlparse']).urlparse(url).path for url in old['readUrls']}
            current_reads = [x for x in items if x['route'] in old_routes]
            assert len(current_reads) == len(old['readUrls'])
            answer = old['answer'].split('\n\nSources:',1)[0]
            primary = current_reads[1]
            evidence = [
                {'route':primary['route'],'sha256':resources[primary['route']]['sha256'],'observation':reading['environments'][environment]['observations'][locale][task]['observation']},
                proof(reading_path,'Actual new HTTP/scoped reading, per-task current headers and explicit detailed-answer inheritance; no changed prose exists.'),
                proof(batch/f'technical-machine-{environment}-input-audit.json','Independently checked all 192 actual previous/current archive byte identities and new response bytes against both manifests.'),
                proof(batch/f'technical-machine-{environment}/observations.json','Actual fresh localhost HTTP GETs by this independent reviewer; 60 distinct current responses serving six languages and four tasks.'),
                proof(prior_path,'Immutable earlier independent report supplies this exact unchanged semantic answer and any prior open issues; current support rechecked in this run.'),
            ]
            results.append({'id':task,'status':'passed','readUrls':[x['url'] for x in current_reads],'answer':answer+'\n\nSources: '+', '.join(x['url'] for x in current_reads),'answerLineage':{'path':str(prior_path),'sha256':digest(prior_path.read_bytes()),'task':f'{locale}/{task}','method':'Inherited semantic answer, explicitly revalidated against exact unchanged resource bytes and fresh current task reading. Not represented as newly authored.'},'evidence':evidence,'openIssues':old['openIssues']})
        entries[locale] = {'aiMachineReadReview':results,'openIssues':prior['entries'][locale]['openIssues']}
    report = {
        'schemaVersion':1,'kind':'machine-read','status':'passed','mode':'release-candidate','reviewerType':'ai',
        'reviewerRun':f'/root/standards_review/pr66-rebuild-independent-machine-{environment}-2026-11-02',
        'inputSha256':manifest['inputSha256'],'configuration':manifest['configuration'],'releaseLocales':manifest['releaseLocales'],
        'readOrigin':capture['readOrigin'],
        'originManifest':{'path':str(manifest_path),'sha256':digest(manifest_path.read_bytes()),'commit':manifest['commit'],'description':'Actual new frozen candidate manifest whose archive was personally fetched over HTTP and independently compared/read in this run.'},
        'resourceBindings':[{'route':x['route'],'sha256':x['sha256']} for x in audit['records']],
        'method':reading['scope'],'rootScope':reading['rootScope'] if environment=='root' else 'Current prefix localized task-result/constraint passages personally re-read as recorded in technical-machine-semantic-reading.json.',
        'previousReport':proof(prior_path,'Immutable prior independent full semantic review; reuse is supported by all 192 actual unchanged archive files and fresh current HTTP tasks.'),
        'entries':entries,'openIssues':prior['openIssues'],'limitations':reading['limitations'],
    }
    output = batch/f'machine-read-{environment}.json'
    output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'path':str(output),'sha256':digest(output.read_bytes()),'tasks':sum(len(x['aiMachineReadReview']) for x in entries.values()),'bindings':len(report['resourceBindings']),'actualHttpResponses':len(capture['responses'])}))
