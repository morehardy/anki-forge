"""Assemble this reviewer's authored findings and verifiable input provenance.

This evidence helper is not a product validator or an automatic content approver.
The topic observations below were written after reviewing the actual sources.
"""
import copy
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

BASE = Path('docs/plans/evidence/multilingual-es-integration-2026-10-31')
RUN = '/root/spanish_standards/independent-technical-source-2026-10-31'
MAIN = 'e5d3d66a12e5ab9a2c161b65c7e055a3677f3f79'
SPANISH = '991187c33ac438f10d2b3f7b56586724ab661052'
sha = lambda data: hashlib.sha256(data).hexdigest()
read = lambda path: json.loads(Path(path).read_text())
candidate_path = BASE / 'source-review-candidate.json'
candidate = read(candidate_path)
probe_path = BASE / 'technical-structural-probe.json'
probe = read(probe_path)
assert probe['candidateSha256'] == sha(candidate_path.read_bytes())
assert probe['status'] == 'passed'

prior_paths = {
    'main': Path('docs/plans/evidence/multilingual-common-2026-10-27/technical-source-review.json'),
    'spanish': Path('docs/plans/evidence/multilingual-es-2026-10-28/technical-source-review.json'),
}
prior_reports = {key: read(path) for key, path in prior_paths.items()}
prior_snapshots = {}
prior_references = {}
for key, report in prior_reports.items():
    refs = {}
    for name in ['inputSnapshot', 'taskDescriptionsAudit', 'pythonQaAudit', 'articleAliasesAudit', 'sourceCandidate', 'searchVocabularyAudit']:
        ref = report.get(name)
        if not ref:
            continue
        path = Path(ref['path'])
        assert sha(path.read_bytes()) == ref['sha256'], f'Stale prior evidence: {path}'
        refs[name] = {'path': str(path), 'sha256': ref['sha256']}
    snapshot = read(report['inputSnapshot']['path'])
    previous = {}
    for row in snapshot['files']:
        blob = prior_paths[key].parent / row['snapshotPath']
        assert sha(blob.read_bytes()) == row['sha256'], f'Stale immutable prior input: {blob}'
        previous[row['path']] = row['sha256']
    prior_snapshots[key] = previous
    prior_references[key] = {'path': str(prior_paths[key]), 'sha256': sha(prior_paths[key].read_bytes()), 'reviewerRun': report['reviewerRun'], 'verifiedReferences': refs, 'verifiedImmutableInputBlobs': len(previous)}

NOTES = {
    'home': 'Actual ES UI says the Rust core exposes Rust/Python/Node APIs, normal export needs no installed Anki, and stable IDs plus an earlier package help recognize existing notes. It does not promise deletion sync or overwrite of newer learner edits. The AI prompt preserves public 0.3.1 install commands, persistent APKG output, stable namespace/keys, a run command, verification and the original distribution baseline. Nine rendered locale/SDK combinations retained the correct localized quickstart; user request substitution remains literal and one-pass.',
    'examples': 'ES copy describes four runnable Rust card examples, the four-card combined download, and an answer-only hello → hello; hi update retaining identity. It explicitly distinguishes preview content/identity from Anki styling and review behavior and notes import options/local edits. The shared website_showcase.rs and examples.ts remain byte-identical to main and the reviewed Spanish baseline; plural-format probes cover 0, 1, 2, 5, 21 and 1000000.',
    'blog': 'ES listing strings preserve the Tutorial/Engineering classification and RSS/article actions. The two actual Spanish article frontmatters keep their translationKey identities and 2026-09-22 historical dates. Article sources and listing renderer remain aligned with stable topic routes; Spanish articles retain all eight explicit semantic section aliases.',
    'docs': 'The ES index directs readers to language selection, public-package quickstarts, CSV/JSON/AI workflows, media/templates and repeatable updates. It keeps note identity and original-baseline concepts separate from merely exporting an APKG, and distinguishes source builds from public-package entry points.',
    'languages': 'ES language selection names the public Rust, Node.js and Python packages with the shared version 0.3.1 commands; the runtime choice is separate from website locale. It points each SDK to its own full quickstart and keeps normal export independent of an installed Anki desktop client.',
    'quickstart': 'The Spanish Rust program creates namespace spanish, stable key es:hola, a Basic hola/hello note and persistent spanish.apkg. The Rust fence exactly equals anki_forge/examples/target_api_basic.rs, which English MDX embeds as CodeBlock; the other install/run fences also match. The prose separates persistent output from temporary artifact ownership and leads to Anki import verification.',
    'json-to-anki': 'The full ES recipe uses three stable source records, three cards and two media assets, writes an initial original APKG, changes one answer, compares policy, prepares and publishes the same next candidate. Rust/Python/TypeScript program fences exactly match current English. It requires a fresh output directory, retains the original baseline, and does not claim omitted notes are removed or learner scheduling is reset.',
    'csv-to-anki': 'The ES CSV recipe specifies UTF-8-sig reading and the exact id,front,back header in that order; it rejects missing/duplicate/blank identifiers, extra columns and empty input rather than silently assigning identity. Its complete Python example matches English and creates three notes/cards with stable keys, persistent output and explicit import checks; namespace and deck are CLI options, and the script avoids overwriting an existing output.',
    'ai-anki-cards': 'The Spanish AI workflow requires human answer review before exporting. It separates AI drafting from Anki Forge packaging and offers the complete CSV path or a coding-assistant brief. The current AI templates constrain public APIs, stable identities, persistent output, run/verification steps and original-baseline updates; the optional request stays in the page until the user copies it.',
    'anki-forge-vs-genanki': 'The Spanish comparison preserves the distinction between APKG authoring choices, native runtime/package constraints, ownership and update-review workflow. It describes migration costs and boundaries instead of claiming arbitrary genanki GUIDs/IDs can be recovered into the new identity protocol. Compatibility and performance statements stay scoped to the documented evidence.',
    'development': 'The source-build guide distinguishes checkout/build commands and examples from the public 0.3.1 SDK installation path. Rust/Node/Python build instructions remain unchanged executable fences, with source-only paths interpreted relative to the checkout and pinned historical revisions not represented as current package contents.',
    'rust-guide': 'The ES Rust guide preserves Project, stable keys, owned immutable NoteType/Media values, validation-before-mutation and build/report/update ownership. Its source examples match English exactly; explicit HTML and typed image/sound content remain distinct from plain Text and raw-HTML asset declarations.',
    'templates': 'Re-read the final intro and metadata: reusable normal/Cloze note types load templates, CSS and media together; Python NoteType.from_bundle and Node NoteType.fromBundle links target actual ES section markers. Full body keeps bundle-v2 validation, field-key binding, owned dependency closure, explicit assets, path containment, 256 KiB manifest / 2 MiB text limits and 255 UTF-8-byte export names. Schema validation alone is not represented as full load validation.',
    'api': 'The ES overview maps authoring, media, diagnostics, build and update operations to Rust/Node/Python references without inventing shared syntax. Links lead to SDK-specific references, and runtime ownership/update-baseline constraints are preserved rather than suggesting APIs are interchangeable source code.',
    'installation': 'The ES installation page keeps the public Rust crate/Cargo dependency and run commands aligned with the shared 0.3.1 quickstart. It distinguishes normal APKG export from importing in an Anki client and directs advanced source builds to their separate guide.',
    'concepts': 'The ES concepts define a Project namespace, stable source-note/model/field/template identities, display names, notes versus generated cards, and owned media/model values. Plain strings are Text; intentional HTML and explicit raw-media dependencies require the appropriate APIs. Stable identity requires an original distribution baseline for repeatable updates.',
    'cards': 'Re-read final intro/metadata: Basic is question-and-answer and Cloze is fill-in-the-blank; the Rust example yields two notes/three cards. New Python/Node Note.basic and Note.cloze links target real ES API section markers. Body preserves explicit stable note keys, Text versus Content::html, back_extra, cloze numbers 1–500, and identity risks of renumbering; the executable fences match current English.',
    'custom-notetypes': 'ES custom model guidance preserves required stable model/field/template keys, immutable validated models, schema consistency on add, field-key references and explicit generation rules. A single sort field is described consistently with the model builder. Display-name translation is not confused with identity migration, and code fences are unchanged.',
    'media': 'Re-read final intro/metadata: supplied media is copied into the APKG; TTS or image creation requires another tool. The body accurately preserves immutable snapshots, explicit MIME, 256 MiB per-asset default, export-name NFC/case collision and 255-byte constraints, explicit assets for raw HTML/CSS, and category validation versus codec support. Batch import preserves order, uses at most four workers and is sequential below 16 files; 64 MiB owned-pool and 16 MiB encoding-pool budgets are not claimed to bound total RSS.',
    'image-occlusion': 'Re-read final intro/metadata: the caller supplies rectangle coordinates; labels are not detected automatically. New Python/Node links resolve to existing ES media/occlusion markers. Body preserves keyed masks, one card per mask, HideAllGuessOne/HideOneGuessOne, post-EXIF pixel coordinates, finite positive in-bounds rectangles, supported image decode formats and separate 256 MiB decoded-image limit. Removed mask ordinals remain reserved; keys 1–500 do not imply learner-card deletion sync.',
    'updates': 'Re-read final intro/metadata: stable IDs aid recognition while import options and newer local edits still affect updates. Full text requires the original distributed APKG with identity-v1, not an Anki re-export; stable namespace and note/model/field/template keys preserve historical mapping. Compare success is distinct from policy allowing publication; High/Critical block by default, allow() accepts an entire risk category, and hard errors cannot be waived. Prepare/review/publish retains the same candidate, protects the archived baseline and does not promise deletion sync or scheduler restoration.',
    'build-guarantees': 'ES guarantee text distinguishes temporary artifacts and their owning handles from persistent target files, inspection/diagnostic limits from proof of client compatibility, and preparing from publishing. Publication consumes the reviewed candidate; destination/path protection and a possible late durability failure are not turned into unconditional rollback or filesystem-atomicity promises.',
    'troubleshooting': 'ES troubleshooting keeps diagnostic codes, installation/runtime boundaries, missing fields, MIME/export-name failures, template validation, identity-baseline errors and output-path failures technically distinct. Recovery guidance uses the original baseline rather than bypassing hard errors, and target-client multimedia/import testing remains required where package validation cannot establish playback or merge behavior.',
    'compatibility': 'The ES compatibility page preserves pinned verification dates, package/runtime environments and conditional Anki client behavior. It does not generalize one tested platform/client or benchmark into universal support; source builds, public 0.3.1 packages and native wheel/package availability remain distinct.',
    'rust-api': 'ES Rust reference preserves names and ownership for authoring/build/compare/prepare/publish, risk-policy scope and diagnostics. The owning build result keeps temporary artifacts alive, and prepared publication consumes the same candidate. All fenced Rust/configuration examples match current English; shared SDK and executable dependency bytes match the verified prior baseline.',
    'node-quickstart': 'ES Node quickstart uses the public npm package and stable namespace/key to write a persistent APKG. Awaited native operations and explicit close of the owning build result preserve lifetime; the complete JavaScript/TypeScript instructions and fences match current English rather than translating API identifiers.',
    'node-api': 'ES Node reference preserves camelCase methods, Promise-returning native operations, snapshots/ownership and close semantics. It distinguishes worker/native asynchronous work from blocking the JS event loop and keeps build/compare policy and prepared same-candidate publication constraints. Performance wording remains comparative within evidence, not a guarantee for every runtime.',
    'python-quickstart': 'ES Python quickstart installs the public PyPI wheel, creates a stable namespace/key and writes a persistent APKG using a full runnable program. Its code exactly matches current English. Source-build instructions and native runtime compatibility remain separate, with import checks instead of claiming Anki is needed merely to export.',
    'python-api': 'ES Python reference preserves snake_case SDK calls, owned models/media and temporary artifact lifetimes, synchronous Python APIs that release the GIL for native work, busy/close behavior and fork constraints. Compare policy is checked separately from a completed comparison; prepare/publish keeps the reviewed candidate and rejects hard baseline/identity errors.',
    'python-diagnostics': 'ES Python diagnostics retain authoring errors versus build reports, native code/path/cause details, versioned protocol evidence and risk-policy semantics. High/Critical defaults, category-wide exceptions and non-waivable identity/inspection errors remain separate; diagnostics are not a promise that a client import preserves every local change.',
    'python-migration': 'The ES Python transition explicitly describes a new owned-value SDK and the boundary with retired Deck/lock-file interfaces. It maps modern Project/NoteType/Media patterns without promising deprecated aliases or automatic recovery of old distribution identity.',
    'genanki-migration': 'Re-read final body: the Anki Forge brand normalization does not alter SDK identifiers. The mapping uses stable business/source keys, immutable models and explicit Text/HTML/media behavior; later releases require a fresh Project with the same namespace/keys and the previous original APKG. It explicitly refuses to promise conversion of arbitrary legacy genanki numeric IDs or GUIDs.',
    'blog:your-first-anki-deck': 'Read the complete Spanish article: its 2026-09-22 date and translationKey remain historical metadata, the complete Rust example/install/run path gives stable namespace/key and persistent output, and it links current quickstart guidance. The four explicit aliases a-complete-program, install-and-run, give-the-note-an-identity and grow-from-one-note immediately precede the corresponding sections.',
    'blog:stable-ids-and-deck-updates': 'Read the complete Spanish article: stable identities are separate from editable content, the next build uses the original distribution baseline, and import review/local edits limit outcomes. Its date remains 2026-09-22; four explicit aliases for identity, baseline, original package and import inspection map to the actual matching sections. It does not imply re-exported APKGs retain full identity evidence.',
}
assert set(NOTES) == set(candidate['entries'])

fresh_es = set()
all_files = sorted({p for translations in candidate['entries'].values() for entry in translations.values() for p in entry['inputFiles']})
for filename in all_files:
    if filename.startswith('docs/es/') or '/blog/es/' in filename or filename in ('website/src/i18n/es.json', 'website/src/content/i18n/es.json'):
        fresh_es.add(filename)
fresh_shared = {
    'website/site-model.mjs', 'docs/ai-quickstart.json', 'website/language-briefs.json',
    'website/release-locales.json', 'website/search-concepts.json', 'website/search-terms.json',
    'website/src/i18n/messages.mjs', 'website/src/lib/ai-quickstart.mjs',
    'website/src/components/LanguageSwitcher.astro', 'website/src/components/AiQuickstart.astro',
    'website/src/lib/examples.ts', 'anki_forge/examples/docs_workflow.rs',
    'anki_forge/examples/docs_publication.rs', 'website/src/content/docs/docs/quickstart.mdx',
}

def git_blob(ref, filename):
    value = subprocess.run(['git', 'show', f'{ref}:{filename}'], capture_output=True)
    return value.stdout if value.returncode == 0 else None

blob_dir = BASE / 'technical-input-snapshot'
blob_dir.mkdir(exist_ok=True)
rows = []
for filename in all_files:
    data = Path(filename).read_bytes()
    digest = sha(data)
    (blob_dir / digest).write_bytes(data)
    main = git_blob(MAIN, filename)
    spanish = git_blob(SPANISH, filename)
    mode = 'fresh complete Spanish prose/catalog read; executable fences compared against current English/canonical source' if filename in fresh_es else 'fresh shared source/data semantic read and comparison' if filename in fresh_shared else 'read bytes and verify unchanged baseline; reuse documented prior semantic review'
    rows.append({'path': filename, 'sha256': digest, 'bytes': len(data), 'snapshotPath': f'technical-input-snapshot/{digest}', 'reviewMode': mode, 'equalsPinnedMain': main is not None and data == main, 'equalsPinnedSpanish': spanish is not None and data == spanish, 'matchesVerifiedPriorSnapshots': [key for key, values in prior_snapshots.items() if values.get(filename) == digest]})

supporting = [
    'website/scripts/record-content-review.mjs', 'website/scripts/content-review-inputs.mjs',
    'website/scripts/ai-quickstart.mjs', 'website/scripts/third-locale.test.mjs',
    'website/search-queries.json', 'anki_forge/examples/target_api_basic.rs',
    'contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml',
    str(candidate_path), str(probe_path), str(BASE / 'technical-source-probe.mjs'),
]
supporting.extend(str(path) for path in prior_paths.values())
for ref in prior_references.values():
    supporting.extend(row['path'] for row in ref['verifiedReferences'].values())
supporting_rows = []
for filename in sorted(set(supporting)):
    data = Path(filename).read_bytes()
    supporting_rows.append({'path': filename, 'sha256': sha(data), 'bytes': len(data)})

snapshot = {
    'reviewerRun': RUN, 'createdAt': datetime.now(timezone.utc).isoformat(),
    'candidate': {'path': str(candidate_path), 'sha256': sha(candidate_path.read_bytes())},
    'pinnedMain': MAIN, 'pinnedSpanish': SPANISH,
    'scope': [
        'Independent technical source review. Fresh semantic read of all 29 ES documents, two ES articles, entire ES UI/framework dictionaries, AI prompt/constraint data, locale model migration, search additions and briefs. Re-read six changed ES bodies and five metadata descriptions after the final integration correction.',
        'EN/zh-CN longforms, render/runtime code and SDK dependencies are assessed by explicit unchanged-input reuse of the prior independent main review, with byte comparisons against pinned main and verified immutable prior blobs. EN/zh-CN metadata/search/AI fields are semantically equal to main; no claim to have newly re-read every unchanged EN/zh-CN paragraph.',
        'Current main runtime/validators retained. ES model metadata lives in existing locale maps; AI constraints live in promptContracts.es. Read-only probe checks all 102 current triples/input lists, 29 metadata records, 64 code fences/canonical program instances and 9 prompt cases. This is not an SDK execution, browser, artifact-build or live deployment review.',
    ],
    'files': rows, 'supportingFiles': supporting_rows, 'priorEvidence': prior_references,
    'delta': {
        'candidateFileCount': len(rows),
        'byteIdenticalToPinnedMain': sum(r['equalsPinnedMain'] for r in rows),
        'byteIdenticalToPinnedSpanish': sum(r['equalsPinnedSpanish'] for r in rows),
        'verifiedPriorMainMatches': sum('main' in r['matchesVerifiedPriorSnapshots'] for r in rows),
        'verifiedPriorSpanishMatches': sum('spanish' in r['matchesVerifiedPriorSnapshots'] for r in rows),
        'differentFromBothPinnedTrees': [r['path'] for r in rows if not r['equalsPinnedMain'] and not r['equalsPinnedSpanish']],
        'reReviewedFinalSpanishBodies': ['docs/es/'+name+'.md' for name in ['cards','image-occlusion','media','templates','updates','genanki-migration']],
        'reason': 'Three-language menu/search/AI/metadata inputs intentionally invalidate older composite hashes. New hashes are supported by independent source observations and current closure checks; old reviewed statuses were not copied by hash replacement alone.',
    },
}
snapshot_path = BASE / 'technical-input-snapshot.json'
snapshot_path.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2)+'\n')

report = copy.deepcopy(candidate)
report['reviewerRun'] = RUN
report['completedAt'] = datetime.now(timezone.utc).isoformat()
report['reviewMethod'] = snapshot['scope']
report['reviewerAuthorshipDisclosure'] = 'This independent reviewer did not implement the locale/runtime migration or modify product sources/content-review records. It reported five missing latest-main ES introduction/metadata clarifications, re-read the integrator corrections and wrote only evidence in this batch directory.'
report['inputSnapshot'] = {'path': str(snapshot_path), 'sha256': sha(snapshot_path.read_bytes())}
report['sourceCandidate'] = snapshot['candidate']
report['priorEvidence'] = prior_references
report['structuralProbe'] = {'path': str(probe_path), 'sha256': sha(probe_path.read_bytes()), 'topics': 34, 'entries': 102, 'codeFences': 64, 'promptCases': 9}
report['closedFindings'] = [{
    'id': 'ES-INTEGRATION-TECH-001', 'status': 'closed',
    'finding': 'Older Spanish introductions omitted latest-main task context/limitations and SDK crosslinks for cards, image occlusion, media, templates and updates.',
    'resolution': 'Integrator added faithful ES task descriptions, caller-provided rectangles/no label autodetection, external TTS/image creation, reusable template SDK links and client import/local edit caveat, with matching five metadata descriptions. Final six-body read also verified the brand-only migration change. Existing executable code and semantic sections were preserved.',
}]
metadata = {entry['id']: entry for entry in probe['metadata']}
codes = {entry['id']: entry for entry in probe['codeChecks']}
checks = {(entry['id'], entry['locale']): entry for entry in probe['sourceChecks']}
for topic, translations in report['entries'].items():
    for locale, entry in translations.items():
        check = checks[topic, locale]
        for field in ['sourceHash', 'targetHash', 'technicalDependencyHash']:
            assert check[field] == entry[field]
        prior_key = 'spanish' if locale == 'es' else 'main'
        old = prior_reports[prior_key]['entries'][topic][locale]['technical']
        assert old['status'] == 'reviewed'
        assert not any(issue.get('severity') in ['major', 'critical'] for issue in old['openIssues'])
        ref = prior_references[prior_key]
        if locale == 'es':
            evidence = [NOTES[topic], f"Fresh ES content read plus verified prior technical dependencies: {ref['path']} (SHA-256 {ref['sha256']}), entry {topic}/es/technical. The immutable-input comparisons and explicit current data/runtime deltas are recorded in {snapshot_path}; prior execution claims remain prior evidence, not new executions by this reviewer."]
        else:
            evidence = [f"Unchanged-input review reuse for {topic}/{locale}: {ref['path']} (SHA-256 {ref['sha256']}), entry {topic}/{locale}/technical, including its verified task-description and Python QA follow-up evidence. Current source body/template and catalog bytes equal pinned main; unchanged dependency snapshot hashes are verified individually. Current {locale} metadata, search fields and AI data preserve main semantics. No full new linguistic/content rewrite review is claimed for the unchanged paragraph text.", 'Topic-specific semantic check, independently applied to the Spanish counterpart and unchanged shared executable/runtime contract: '+NOTES[topic]]
        if topic in metadata:
            item = metadata[topic]
            evidence.append(f"Actual current {locale} model metadata read: title {item['title'][locale]!r}; description {item['description'][locale]!r}. These describe the page's actual supported task and limitations. ES descriptions for the five task pages were independently re-read after the latest-main parity correction; EN/zh-CN values remain exactly main.")
        if topic in codes:
            check = codes[topic]
            evidence.append(f"Current {topic} source comparison passed for {check['fences']} fenced examples/commands ({check['mode']}); source-owned programs and SDK dependency bytes retain the baseline reviewed behavior. API identifiers and commands were not translated.")
        evidence.append('Shared integration checked: main locale-map implementation and release-locale-driven message loading remain; Spanish catalog/metadata/search/brief/prompt data are present without special-case runtime branches. ES API anchors and eight article aliases correspond to actual sections. Candidate source/target/dependency hashes and exact inputFiles were recomputed for this entry by the independent probe; no artifact acceptance is inferred from source approval.')
        entry['technical'] = {'status': 'reviewed', 'evidence': evidence, 'openIssues': []}
        entry['linguistic'] = {'status': 'pending', 'score': {}, 'evidence': [], 'openIssues': []}
report['summary'] = {'topics': len(report['entries']), 'technicalReviewed': 102, 'linguisticPending': 102, 'openMajorOrCriticalIssues': 0, 'limits': 'No fresh SDK compilation/execution, Anki-client import, browser, built-artifact or live-deployment verification by this reviewer. Separate integration checks must provide that evidence.'}
out = BASE / 'technical-source-review.json'
out.write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
for row in rows:
    assert sha(Path(row['path']).read_bytes()) == row['sha256'], f"Input changed during evidence assembly: {row['path']}"
print(json.dumps({'report': str(out), 'reportSha256': sha(out.read_bytes()), 'snapshot': str(snapshot_path), 'snapshotSha256': sha(snapshot_path.read_bytes()), 'delta': snapshot['delta'], 'summary': report['summary']}, ensure_ascii=False, indent=2))
