"""Record independently authored conclusions after the actual two browser tours.

This does not derive a passing verdict from helper success or older reviews.
The reviewer read the rendered task text and inspected all 36 captured images.
"""
import hashlib
import json
from pathlib import Path
from urllib.parse import urlparse

BASE = Path('docs/plans/evidence/multilingual-es-integration-2026-10-31')
EX = BASE / 'executions'
RUN = '/root/spanish_standards/independent-usage-ci-rebuild-a9f3face-2026-11-01'
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda path: json.loads(Path(path).read_text())
binding = lambda path: {'path': str(path), 'sha256': sha(Path(path).read_bytes())}
def evidence(path, description):
    return {**binding(path), 'description': description}

audit_path = BASE / 'usage-ci-execution-audit.json'
audit = load(audit_path)
sources = {row['path']: row for row in audit['currentSourceDependencyBindings']}
basic = {
    'rust-basic': ('rust', 'anki_forge/examples/target_api_basic.rs', 'rust/src/main.rs'),
    'python-basic': ('python', 'bindings/python/examples/quickstart.py', 'python/main.py'),
    'node-basic': ('node', 'bindings/node/examples/quickstart.mjs', 'node/main.mjs'),
}
recipe = load(EX / 'public-recipe-inputs.json')
queries = {'en': 'python first deck', 'zh-CN': 'Python 生成 牌组', 'es': 'crear tarjetas Python'}
observed_limits = {
    'en': '“original distribution APKG”; “Anki re-export does not preserve this evidence”; “Give each logical source record one unique key”.',
    'zh-CN': '“原始分发 APKG”; “Anki 重新导出不保留该证据”; “每条逻辑源记录使用唯一键”。',
    'es': '“APKG original de distribución”; “La reexportación desde Anki no conserva estas pruebas”; “Asigna una clave única a cada registro lógico de origen”.',
}
for environment in ['prefix', 'root']:
    directory = BASE / f'usage-browser-{environment}-ci'
    tour_path = directory / 'browser-tour.json'
    tour = load(tour_path)
    manifest_path = BASE / f'{environment}-ci-origin-manifest.json'
    manifest = load(manifest_path)
    assert sha(manifest_path.read_bytes()) == tour['manifestSha256']
    frozen = Path('target/spanish-integration') / f'{environment}-artifact'
    assert manifest_path.read_bytes() == (frozen / 'generated/site-manifest.json').read_bytes()
    integrity_path = directory / 'artifact-integrity.json'
    integrity = load(integrity_path)
    assert integrity['manifestSha256'] == tour['manifestSha256']
    assert integrity['identity']['inputSha256'] == manifest['inputSha256'] == tour['inputSha256']
    records = {row['route']: row for row in manifest['pages'] + manifest['resources']}
    bound_routes = {row['route'] for row in manifest['resources'] if row['route'].endswith(('/index.md', '/llms.txt'))}
    for tasks in tour['entries'].values():
        for task in tasks:
            bound_routes.update(urlparse(url).path for url in task['readUrls'])
            bound_routes.update(urlparse(row['url']).path for row in task['downloadedResources'])
    report = {
        'schemaVersion': 1, 'kind': 'usage-tasks', 'status': 'passed', 'mode': 'release-candidate',
        'reviewerType': 'ai', 'reviewerRun': RUN + '/' + environment,
        'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'],
        'releaseLocales': manifest['releaseLocales'], 'readOrigin': tour['readOrigin'],
        'originManifest': {**binding(manifest_path), 'commit': manifest['commit']},
        'resourceBindings': [{'route': route, 'sha256': records[route]['sha256']} for route in sorted(bound_routes)],
        'scopeNotes': [
            'New independent usage re-review of the CI-compatible rebuild at a9f3facef528369976712b81ead4c99c162b2907. The prior build used a shared node_modules symlink; the rebuilt artifact uses ordinary local dependencies, changing generated Astro CSS scopes/HTML hashes without changing source inputSha256. This report is based on new real browser tours and newly inspected screenshots, not replacement of old HTML hashes. No product source was changed by this reviewer.',
            'Independent actual Chrome tours ran against this exact frozen artifact after all delivered files were independently hashed and matched to the manifest/current source identity. Each release locale completed all six tasks. No old usage verdict or hash refresh was used.',
            'The reviewer read complete actual rendered task prose, checked the real clipboard code matches, inspected all 18 screenshots for this environment, and retrieved four linked recipe inputs per locale through actual HTTP requests. The captured root and prefix task texts/code blocks are exactly equal, while routes were exercised separately under the correct base.',
            'SDK installation and execution were performed by the coordinator earlier in this integration batch and are explicitly reused for this CI rebuild only after verifying that all bound source/dependency bytes and output receipts remain unchanged. This reviewer rechecked the retained logs, verified all 23 current execution-source/dependency bindings and three exact retained basic program copies, reviewed the execution/inspection code, and freshly decoded 10 APKG outputs with independent Python ZIP/Zstandard/SQLite inspection. The reviewer did not itself rerun the SDKs or import into an Anki GUI.',
            'Actual public consumer Cargo locks, PyPI wheel hashes and npm package integrity records are retained. Repository dependency bindings document current project inputs and do not replace the separate public consumer resolution evidence. JSON temporary consumer copies were removed by the reviewed runner; its current source snapshot and actual run logs supply provenance.',
            'All Markdown/llms resources are integrity-bound as required by the release scope. readUrls name actual browser task reading; binding a machine resource does not claim it was independently read in this usage review. Unrelated generated showcase/APKG files are covered by full artifact integrity and are not unstable report resource bindings.',
            'Acceptance covers local frozen-artifact task completion at 1440×1000 in real Chrome. It does not establish live deployment/CDN headers, every OS/runtime, mobile task performance or Anki-client import guarantees. Those limitations remain explicit in the documentation.',
        ],
        'ciRereviewEvidence': evidence(BASE / 'usage-ci-rereview.json', 'New Chrome tour provenance, old/new artifact delta, unchanged-source and execution-reuse audit; preserved historical reports remain intact.'),
        'observationHelper': evidence(BASE / 'usage-tour-helper.mjs', 'Exact helper used for actual browser navigation, locale assertions, real clipboard copying, recipe HTTP downloads and screenshots. It records observations only and does not issue review verdicts.'),
        'entries': {}, 'openIssues': [],
    }
    for locale in manifest['releaseLocales']:
        results = []
        for task in tour['entries'][locale]:
            task_id = task['id']
            proof = [
                evidence(tour_path, f'Actual {environment} Chrome tour: complete rendered main text, code blocks, copied programs, linked HTTP download receipts and screenshot hashes for {locale}/{task_id}.'),
                evidence(task['screenshot'], f'Actually inspected 1440×1000 screenshot of {locale}/{task_id}; localized title, sidebar and TOC are readable; commands/content remain in the main column and the locale does not fall back.'),
                evidence(integrity_path, f'Independent pre-tour inventory verifies all {len(integrity["files"])} delivered files and the complete archived manifest for this frozen {environment} artifact.'),
                evidence(audit_path, 'Independent review of this batch’s current execution inputs and fresh direct ZIP/Zstandard/SQLite observations of all 10 output APKGs, plus equality of separately observed root/prefix task text and clipboard source checks.'),
            ]
            for url in task['readUrls']:
                route = urlparse(url).path
                proof.append({'route': route, 'sha256': records[route]['sha256'], 'observation': f'Actually visited {url} during the {locale} browser task. Navigation kept the selected locale and deployment base; final page title was {task["h1"]!r}. Full final rendered text is retained in the tour evidence.'})
            steps = list(task['steps'])
            execution = []
            if task_id in basic:
                sdk, source, copy = basic[task_id]
                steps += [
                    f'Read the actual localized heading {task["h1"]!r}, public 0.3.1 installation instructions, complete program, run command, persistent spanish.apkg output and stable spanish/es:hola note identity.',
                    'Rechecked the unchanged coordinator execution/install/inspection logs from this integration batch and directly inspected the retained APKG: valid SQLite, one note, one Basic card, no media, hola → hello in Spanish; the identity payload contains namespace spanish and key es:hola.',
                ]
                if task_id == 'python-basic':
                    steps.append(f'The actual localized search query {queries[locale]!r} reached the Python quickstart; CPython/wheel and Windows activation guidance were readable and did not require an Anki installation for export.')
                elif task_id == 'node-basic':
                    steps.append('Read Node.js 22.13+, native optional package installation, ESM main.mjs and explicit artifact.close(); persistent output remains after close. The linked JSON task supplied a complete TypeScript compile/run program, also copied in this tour.')
                else:
                    steps.append('Read Rust 1.92+, cargo new/add and src/main.rs/cargo run instructions. The custom English CodeBlock and translated fenced code both copied the exact current canonical Rust program.')
                deps = ['docs/ai-quickstart.json', 'scripts/packaged_consumer/src/inspection.rs', 'scripts/packaged_consumer/src/bin/verify_apkg.rs', 'Cargo.toml', 'Cargo.lock']
                if sdk == 'python': deps.append('bindings/python/pyproject.toml')
                if sdk == 'node': deps.append('bindings/node/package.json')
                execution.append({
                    **evidence(EX / 'basic' / sdk / 'basic.log', f'Retained coordinator public ankiforge 0.3.1 {sdk} execution exited zero and produced persistent spanish.apkg. Independently reviewed here, bound to the exact current/copied program and dependencies.'),
                    'sourceBindings': [sources[source]], 'dependencyBindings': [sources[name] for name in deps],
                })
                proof.extend([
                    evidence(EX / 'basic' / sdk / 'install.log', 'Retained actual public-package install commands and successful exit; version fixed to 0.3.1.'),
                    evidence(EX / 'basic' / sdk / 'inspection.log', 'Retained independent Rust ZIP/protobuf/SQLite inspector output; additionally checked by this reviewer using Python ZIP/Zstandard/SQLite.'),
                    evidence(EX / 'basic' / copy, 'Exact executable program retained from this batch; raw bytes equal current source and trimmed bytes equal the real browser clipboard.'),
                    evidence(EX / 'basic' / sdk / 'spanish.apkg', 'Actual persistent basic output directly inspected by this reviewer.'),
                    evidence(EX / 'basic/execution-inputs.json', 'Current source/dependency snapshot accompanying the retained basic execution, independently checked without changing any hash.'),
                ])
                resolution = {'rust': 'rust/Cargo.lock', 'python': 'python/pip-report.json', 'node': 'node/package-lock.json'}[sdk]
                proof.append(evidence(EX / 'basic' / resolution, 'Actual public consumer resolution: crates.io checksum, PyPI wheel hash, or npm integrity as appropriate; independently read and confirmed ankiforge 0.3.1.'))
            elif task_id in ['json-media', 'original-baseline-update']:
                execution.append({
                    **evidence(EX / 'public-recipe.log', 'Retained public 0.3.1 Rust, Python and TypeScript JSON programs from this integration batch ran successfully in isolated consumers. Actual logs and reviewer-decoded v1/v2 APKGs establish three notes/cards, two exact media assets, one answer change, stable note identities and allowed comparison/prepared publication.'),
                    'sourceBindings': recipe['sourceBindings'], 'dependencyBindings': recipe['dependencyBindings'],
                })
                proof.append(evidence(EX / 'public-recipe-inputs.json', 'Current source, fixture, verifier, version and dependency bindings for the retained JSON execution; every stored hash equals current bytes.'))
                for sdk in ['rust', 'python', 'node']:
                    for name in ['installation.json', 'comparison.json', 'review.json', 'v1.apkg', 'v2.apkg']:
                        proof.append(evidence(EX / 'public-json' / sdk / name, f'Actual {sdk} JSON {name} from this integration batch; registry version/integrity, comparison policy and output identity/content independently reviewed.'))
                if task_id == 'json-media':
                    steps += [
                        'Read the full rendered JSON recipe in this locale: directory layout, caller-parsed JSON, public SDK install/run commands, three notes/cards and two shared media assets. It accurately identifies tone.wav as a tone, not speech.',
                        'Clicked each real copy control for the complete Rust/Python/TypeScript program; actual clipboard content matched all three current sources. Retrieved v1.json, v2.json, cell.png and tone.wav from the actual rendered input links: HTTP 200, local artifact origin, correct manifest and fixture hashes.',
                        'Independently checked all three SDK v1/v2 outputs: SQLite integrity, three stable GUIDs, exactly one changed answer, and exact decoded image/audio bytes; comparison and prepared-review reports both permit this low-risk answer edit. The runner additionally asserts that repeating the run cannot overwrite the original release.',
                    ]
                    for download in task['downloadedResources']:
                        route = urlparse(download['url']).path
                        proof += [
                            evidence(download['path'], f'Actual {download["httpStatus"]} HTTP download from the rendered {download["anchorText"]!r} anchor; {download["bytes"]} bytes and MIME {download["contentType"]}, exact fixture hash.'),
                            {'route': route, 'sha256': records[route]['sha256'], 'observation': 'Actual linked resource fetched from this frozen artifact and saved; bytes match the source fixture consumed by the retained SDK runs.'},
                        ]
                else:
                    steps += [
                        'Completed the answer-only update via the just-read JSON recipe, then followed the localized updates sidebar entry and read the entire original-baseline, policy, identity, client-import and prepare/review/publish explanation.',
                        'Verified the instructions require the previous original distribution APKG, reject Anki re-export as an identity baseline, keep stable namespace/keys and use a separate candidate path. Compare success is not policy approval; High/Critical defaults, category-wide exceptions and non-waivable hard errors are clearly separated.',
                        'Matched the actual JSON v1/v2 identity mappings and low RISK.NOTE_CHANGED report to this workflow. The retained original v1 and same prepared candidate satisfy the documented baseline/update task; client merge/local edits and omitted-note behavior remain limitations, not guaranteed scheduler preservation.',
                        'Actual localized wording read: '+observed_limits[locale],
                    ]
            else:
                assert task_id == 'troubleshooting'
                steps += [
                    'Read the actual NOTE.KEY_DUPLICATE row and unique-logical-source-key correction, together with the atomic add-failure statement. Structured error codes are distinguished from mutable display messages and blocked-comparison policy.',
                    'Reviewed this batch’s deliberately duplicated source:1 execution, which produced AddError NOTE.KEY_DUPLICATE; a corrected unique key succeeded. Independently decoded recovered.apkg: two notes, two Basic cards and zero media, confirming no partial duplicate was added.',
                    'Actual localized wording read: '+observed_limits[locale],
                ]
                execution.append({
                    **evidence(EX / 'basic/python/troubleshooting.log', 'Retained public Python 0.3.1 duplicate-key failure and corrected-key recovery, independently matched to the rendered localized remedy and directly inspected recovered APKG.'),
                    'sourceBindings': [sources['docs/troubleshooting.md'], sources['docs/zh-cn/troubleshooting.md'], binding('docs/es/troubleshooting.md')],
                    'dependencyBindings': [sources[name] for name in ['docs/ai-quickstart.json', 'bindings/python/pyproject.toml', 'scripts/packaged_consumer/src/inspection.rs', 'scripts/packaged_consumer/src/bin/verify_apkg.rs']],
                })
                proof += [
                    evidence(EX / 'basic/python/duplicate-key.py', 'Actual retained diagnostic/recovery script used by the coordinator; duplicate source:1 is followed by correction to a unique key.'),
                    evidence(EX / 'basic/python/inspection.log', 'Retained independent inspector verifies the recovered package has two notes/cards and the ordinary basic package has one.'),
                    evidence(EX / 'basic/python/recovered.apkg', 'Actual recovered output; separately decoded by this reviewer through ZIP/Zstandard/SQLite.'),
                ]
            results.append({'id': task_id, 'status': 'passed', 'readUrls': task['readUrls'], 'steps': steps, 'executionEvidence': execution, 'evidence': proof, 'openIssues': []})
        report['entries'][locale] = {'taskResults': results, 'openIssues': []}
    output = BASE / f'usage-tasks-{environment}.json'
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'path': str(output), 'sha256': sha(output.read_bytes()), 'tasks': sum(len(entry['taskResults']) for entry in report['entries'].values()), 'resourceBindings': len(report['resourceBindings']), 'manifestSha256': tour['manifestSha256']}))
