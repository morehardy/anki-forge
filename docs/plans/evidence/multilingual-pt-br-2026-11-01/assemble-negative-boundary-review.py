"""Bind this reviewer's explicit negative-result findings to fresh HTTP captures."""
import hashlib
import json
from pathlib import Path
import sys

root = Path.cwd()
p = Path('docs/plans/evidence/multilingual-pt-br-2026-11-01')
environment, label = sys.argv[1:]
capture = p / f'search-negative-{label}/http-observations.json'
origin = p / f'search-negative-{label}/origin-manifest.json'
read = lambda f: json.loads((root / f).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
proof = lambda f: {'path': str(f), 'sha256': sha((root / f).read_bytes())}
observations, manifest = read(capture), read(origin)
assert observations['manifestSha256'] == sha((root / origin).read_bytes())
assert observations['resultSha256'] == sha((root / observations['resultPath']).read_bytes())

# Written after actual result-body review. These are distinct conclusions, not
# a conversion of the search engine's ranking boolean into semantic approval.
findings = {
    'pt-BR-negative-01': {
        'conclusion': 'The exact nonsense query returned no results. This is allowed by the frozen boundary; there is no body from which a false supported capability could be inferred. This semantic check does not independently assert keyboard or search-form usability.',
        'pages': {},
    },
    'pt-BR-negative-02': {
        'conclusion': 'The first result explicitly rejects v1 for tool recipes and requires v2 without top-level name. The other two results describe APKG updates and data-to-APKG authoring without asserting that a tool v1 schema is accepted. Grouping v1/v2 as a retrieval concept therefore does not advertise schema compatibility or confuse caller-parsed recipe JSON with the tool schema.',
        'pages': {
            'rust-guide': ('As receitas das ferramentas agora exigem ankiforge-project-v2, sem name no nível superior; v1 é rejeitado.', 'Directly states the unsupported-v1 boundary and the required tool format.'),
            'updates': ('O pacote inclui ankiforge-identity.json no formato ankiforge-identity-v1.', 'The v1 identifier here describes embedded package identity evidence, not the rejected ankiforge-project-v1 tool input. The body requires an original distribution APKG and contains no promise to accept an old project schema unchanged.'),
            'home': ('Comece com uma planilha CSV, dados JSON ou rascunhos criados com um assistente de IA.', 'Generic data inputs are described without claiming a specific legacy tool schema works unchanged. The explicit correction is supplied by the first result.'),
        },
    },
    'pt-BR-negative-03': {
        'conclusion': 'All three returned pages require the original distributed package and explain why an Anki re-export is not a usable baseline. No result proposes weakening policy to recover missing identity evidence.',
        'pages': {
            'updates': ('Uma reexportação pelo Anki não preserva essas evidências e não serve como base de atualização.', 'Direct rejection; missing identity evidence is not silently reconstructed.'),
            'blog:stable-ids-and-deck-updates': ('Preserve o pacote original que você distribuiu; uma reexportação do Anki não é uma base aceita.', 'The blog distinguishes version history and stable note identity from a client re-export.'),
            'troubleshooting': ('Use como base um APKG original distribuído com evidências completas de identidade. Pacotes reexportados pelo Anki não preservam essas evidências.', 'The recovery advice preserves the original-package requirement; corrupted evidence and namespace mismatches cannot be allowed by an update policy.'),
        },
    },
    'pt-BR-negative-04': {
        'conclusion': 'The first result explicitly says renaming never changes retained MIME and explains that a PNG named wrong.mp3 can fail the separate build extension/content inspection. It distinguishes copying supplied media from creating media with another tool. The second result supplies actual image/audio assets and never teaches renaming as conversion.',
        'pages': {
            'media': ('Renomear um arquivo nunca altera o MIME preservado.', 'The same paragraph names MEDIA.DECLARED_MIME_MISMATCH for a PNG exported as wrong.mp3 and distinguishes typed category acceptance from independent build inspection.'),
            'json-to-anki': ('A imagem é um diagrama de célula; o áudio é um tom curto gerado, não uma gravação de vocabulário falado.', 'The complete recipe uses separate genuine PNG and WAV resources; this is an example of valid media input rather than a claim that renaming changes its type.'),
        },
    },
    'pt-BR-negative-05': {
        'conclusion': 'The returned bodies consistently describe producing an APKG and importing it in Anki. The comparison refuses guarantees about preserved review history; compatibility assigns history/import outcomes to client-specific checks; the first-deck tutorial explicitly requires an import after export. None offers an API or procedure for directly resetting an installed collection or review history. This is a non-endorsement finding, not a claim that every result directly states an unsupported-operation error.',
        'pages': {
            'anki-forge-vs-genanki': ('Nem esta comparação nem a geração bem-sucedida de um APKG prometem uma migração sem perdas ou a preservação de todo o histórico de revisão.', 'Package creation and migration are distinguished; no direct history-edit capability is advertised.'),
            'compatibility': ('Alterações de campos, modelos e campos de ordenação, edições locais e histórico de revisão exigem testes específicos para cada cliente.', 'The preceding paragraph separates ordinary export from Anki installation, import, rendering and codecs. Client behavior remains outside an APKG-generation success claim.'),
            'blog:your-first-anki-deck': ('Após a exportação, importe spanish.apkg no Anki e revele a resposta do seu primeiro cartão.', 'The example writes a package and then instructs the user to import it; it never calls into an installed collection or resets study progress.'),
        },
    },
}

records = {x['route']: x for x in manifest['pages']}
cases = []
bindings = {}
for case in observations['cases']:
    finding = findings[case['id']]
    assert set(finding['pages']) == {r['pageId'] for r in case['reads']}
    actual = []
    for response in case['reads']:
        quote, observation = finding['pages'][response['pageId']]
        assert quote in response['mainText']
        assert response['status'] == 200
        assert sha((root / response['path']).read_bytes()) == response['sha256'] == records[response['url']]['sha256']
        bindings[response['url']] = response['sha256']
        actual.append({
            'pageId': response['pageId'], 'url': response['actualUrl'],
            'route': response['url'], 'sha256': response['sha256'],
            'bodyEvidence': proof(response['path']), 'quote': quote,
            'observation': observation,
        })
    cases.append({
        'id': case['id'], 'query': case['query'], 'intentId': case['intentId'],
        'expectedBoundary': case['expectedBoundary'],
        'rankGatePassed': case['passed'], 'semanticBoundaryStatus': 'passed',
        'conclusion': finding['conclusion'], 'actualResultBodies': actual,
        'openIssues': [],
    })
assert len(cases) == 5 and len(bindings) == 10
report = {
    'schemaVersion': 1, 'kind': 'independent-negative-search-boundary-review',
    'status': 'passed', 'mode': 'release-candidate', 'reviewerType': 'ai',
    'reviewerRun': f'/root/pt_br_technical/search-boundary-{environment}-2026-11-01',
    'scope': 'Five frozen Portuguese negative queries; all eleven nonempty top-three result rows (ten distinct pages) fetched over real HTTP and their technical claims reviewed independently of ranking. The empty query has no body to read. No positive-query acceptance or broad browser-usability conclusion is asserted.',
    'inputSha256': manifest['inputSha256'], 'configuration': manifest['configuration'],
    'releaseLocales': manifest['releaseLocales'], 'readOrigin': observations['readOrigin'],
    'originManifest': dict(proof(origin), commit=manifest['commit']),
    'searchResults': proof(observations['resultPath']), 'querySetSha256': observations['querySetSha256'],
    'rawHttpObservations': proof(capture),
    'resourceBindings': [{'route': r, 'sha256': h} for r, h in bindings.items()],
    'cases': cases, 'openIssues': [],
}
if environment == 'root':
    comparison = p / f'search-negative-{label}/body-comparison.json'
    assert len(read(comparison)['rows']) == 11
    report['semanticComparison'] = proof(comparison)
    report['semanticReviewMethod'] = 'Fresh root HTTP reads and exact text/paragraph comparison to the final prefix bodies independently read by this reviewer. The sole delta is one homepage noscript examples href losing the deployment prefix; that actual delta was inspected. The recorded per-page findings apply to the unchanged technical text.'
(root / p / f'search-negative-boundary-{environment}.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(f'{environment}: five explicit semantic conclusions bound to eleven actual result-row reads and ten distinct HTML resources.')
