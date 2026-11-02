"""Bind manually read Japanese result-body findings; never infer semantics from rank pass."""
import hashlib
import json
from pathlib import Path
import sys

directory = Path('docs/plans/evidence/multilingual-ja-de-2026-11-01/ja')
environment = sys.argv[1]
assert environment in ('prefix', 'root')
capture_path = directory / f'search-negative-{environment}/http-observations.json'
manifest_path = directory / f'search-negative-{environment}/origin-manifest.json'
read = lambda filename: json.loads(filename.read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
proof = lambda filename: {'path': str(filename), 'sha256': sha(filename.read_bytes())}
capture, manifest = read(capture_path), read(manifest_path)
assert sha(manifest_path.read_bytes()) == capture['manifestSha256']
assert sha(Path(capture['resultPath']).read_bytes()) == capture['resultSha256']

# These conclusions and exact quotes were authored after reading the actual
# Japanese HTTP result paragraphs and their surrounding capability statements.
findings = {
    'ja-negative-01': {
        'conclusion': 'The exact nonsense input has no result bodies. Empty output is permitted by this frozen boundary; no supported task is invented. Keyboard/input-correction usability is a separate browser check and is not established by this semantic report.',
        'pages': {},
    },
    'ja-negative-02': {
        'conclusion': 'The second result directly requires ankiforge-project-v2 without top-level name and explicitly rejects v1. It distinguishes that tool schema from the still-valid ankiforge-identity-v1 evidence format. The first home result and third API overview describe generic caller-provided CSV/JSON or public authoring APIs; neither promises to accept an unchanged legacy v1 tool recipe. This passes the capability boundary, without claiming the homepage itself gives migration instructions.',
        'pages': {
            'home': ('CSV の表、JSON データ、AI アシスタントが下書きしたカードから始めます。', 'Generic learning input is an authoring starting point; this paragraph does not name or accept a legacy tool schema.'),
            'rust-guide': ('ツール用レシピにはトップレベルの name を持たない ankiforge-project-v2 が必要です。v1 は拒否されます。', 'Direct unsupported-v1 boundary, with the required replacement format; adjacent identity-v1 is explicitly a different package-evidence format.'),
            'api': ('公開 ankiforge パッケージでノートを作成し、メディアを同梱して、新しいデッキを配布する前に更新内容を確認できます。', 'Public note authoring is described without an assurance that an old JSON tool recipe can be submitted unchanged.'),
        },
    },
    'ja-negative-03': {
        'conclusion': 'All three actual result bodies require the original distribution APKG and reject an Anki re-export as the baseline. The update guide also refuses to silently reconstruct missing evidence or waive it with a risk-category allowance. The blog and runnable JSON recipe preserve the same baseline boundary.',
        'pages': {
            'updates': ('Anki からの再エクスポートではこの情報が保持されないため、更新の基準には使えません。', 'Embedded complete identity/revision evidence must remain in the original package; missing evidence is not reconstructed.'),
            'blog:stable-ids-and-deck-updates': ('Anki から再エクスポートしたファイルは、対応する基準パッケージではありません。', 'The historical post explicitly excludes a client re-export rather than implying stable note keys alone recover the baseline.'),
            'json-to-anki': ('Anki から再エクスポートしたファイルは使えません。', 'The recipe requires original v1 at a separate path and retains it for the reviewed v2 publication.'),
        },
    },
    'ja-negative-04': {
        'conclusion': 'Both the media guide and Rust guide explicitly state that renaming does not change retained MIME and identify independent content/extension checks that may reject PNG bytes named wrong.mp3. Typed image acceptance is not audio conversion or playback proof. The JSON recipe uses genuine separate PNG and WAV assets and labels the sound as a generated tone; no returned body presents renaming as conversion.',
        'pages': {
            'media': ('改名によって保持済みの MIME が変わることはありません。', 'The surrounding paragraph identifies wrong.mp3/PNG and MEDIA.DECLARED_MIME_MISMATCH; category validation and independent build checks are separate.'),
            'rust-guide': ('エクスポート名を変更しても MIME は変わりません。', 'The very next sentence discusses PNG renamed wrong.mp3 and the independent build extension/content inspection.'),
            'json-to-anki': ('画像は細胞の模式図です。音声は短い合成音で、単語の読み上げではありません。', 'The actual recipe supplies cell.png and tone.wav and preserves those bytes; it does not claim to synthesize speech or turn image data into audio by a filename change.'),
        },
    },
    'ja-negative-05': {
        'conclusion': 'The update, compatibility and AI-drafting results consistently describe writing an APKG and then testing/importing it in Anki. The update guide disclaims deletion synchronization and recovery of deleted history; compatibility assigns review/import outcomes to client-specific tests; the AI guide exports a package and then asks the user to open it. None provides an API or procedure to directly reset an installed collection or study history. This is non-endorsement of the unsupported capability, not an assertion that every page explicitly says a reset method is unavailable.',
        'pages': {
            'updates': ('APKG は配布用パッケージであり、削除を同期する仕組みではありません。', 'The surrounding client section also refuses to guarantee recovery of deleted history and requires testing local edits and existing schedules in Anki.'),
            'compatibility': ('フィールド・テンプレートの変更、並べ替え用フィールド、ローカルの編集、復習履歴は、クライアントごとのテストが必要です。', 'Export success is distinct from client rendering/import/review behavior; no direct collection modification capability is advertised.'),
            'ai-anki-cards': ('生成した APKG を Anki で開き、カード数、質問、解答、書式を確認します。', 'The described result is a package to import, not an operation on installed Anki state. The same guide says the prompt builder is local text composition without model calls/uploads.'),
        },
    },
}
records = {item['route']: item for item in manifest['pages']}
cases, bindings = [], {}
for case in capture['cases']:
    finding = findings[case['id']]
    assert set(finding['pages']) == {item['pageId'] for item in case['reads']}
    bodies = []
    for response in case['reads']:
        quote, observation = finding['pages'][response['pageId']]
        assert quote in response['mainText']
        assert response['status'] == 200
        assert sha(Path(response['path']).read_bytes()) == response['sha256'] == records[response['url']]['sha256']
        if response['url'] in bindings:
            assert bindings[response['url']] == response['sha256']
        bindings[response['url']] = response['sha256']
        bodies.append({'pageId': response['pageId'], 'url': response['actualUrl'], 'route': response['url'], 'sha256': response['sha256'], 'bodyEvidence': proof(Path(response['path'])), 'quote': quote, 'observation': observation})
    cases.append({'id': case['id'], 'query': case['query'], 'intentId': case['intentId'], 'expectedBoundary': case['expectedBoundary'], 'rankGatePassed': case['passed'], 'semanticBoundaryStatus': 'passed', 'conclusion': finding['conclusion'], 'actualResultBodies': bodies, 'openIssues': []})
assert len(cases) == 5 and sum(len(case['actualResultBodies']) for case in cases) == 12 and len(bindings) == 9
report = {'schemaVersion': 1, 'kind': 'independent-negative-search-boundary-review', 'status': 'passed', 'mode': 'release-candidate', 'reviewerType': 'ai', 'reviewerRun': f'/root/ja_de_infra_audit/ja-negative-{environment}-2026-11-01',
    'scope': 'Five frozen Japanese negative queries only. Twelve actual top-three HTML result rows, nine distinct bodies, independently read for capability boundaries; the nonsense query has no result. This is separate from ranking, all positive-query acceptance and browser interaction checks. The source independent query file was not read.',
    'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'], 'releaseLocales': manifest['releaseLocales'], 'readOrigin': capture['readOrigin'],
    'originManifest': dict(proof(manifest_path), commit=manifest['commit']), 'searchResults': proof(Path(capture['resultPath'])), 'querySetSha256': capture['querySetSha256'], 'rawHttpObservations': proof(capture_path),
    'resourceBindings': [{'route': route, 'sha256': value} for route, value in bindings.items()], 'cases': cases, 'openIssues': []}
if environment == 'root':
    comparison_path = directory / 'search-negative-root/body-comparison.json'
    comparison = read(comparison_path)
    assert comparison['currentCapture'] == proof(capture_path)
    assert len(comparison['rows']) == 12
    report['semanticComparison'] = proof(comparison_path)
    report['semanticReviewMethod'] = comparison['scope']
(directory / f'search-negative-boundary-{environment}.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(f'{environment}: five manually authored capability-boundary conclusions, twelve actual result reads, nine distinct bound HTML resources.')
