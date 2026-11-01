"""Independent read-only inspection of this batch's executed sources and APKGs."""
import hashlib
import json
import sqlite3
import zipfile
from compression import zstd
from pathlib import Path

BASE = Path('docs/plans/evidence/multilingual-es-integration-2026-10-31')
EX = BASE / 'executions'
sha = lambda data: hashlib.sha256(data).hexdigest()
load = lambda path: json.loads(Path(path).read_text())
binding = lambda path: {'path': str(path), 'sha256': sha(Path(path).read_bytes())}
basic = load(EX / 'basic/execution-inputs.json')
recipe = load(EX / 'public-recipe-inputs.json')
bindings = basic['inputs'] + recipe['sourceBindings'] + recipe['dependencyBindings']
current = {}
for row in bindings:
    assert binding(row['path']) == row, row['path']
    current[row['path']] = row
programs = {
    'rust': ('anki_forge/examples/target_api_basic.rs', 'rust/src/main.rs'),
    'python': ('bindings/python/examples/quickstart.py', 'python/main.py'),
    'node': ('bindings/node/examples/quickstart.mjs', 'node/main.mjs'),
}
copied = []
for sdk, (source, target) in programs.items():
    saved = EX / 'basic' / target
    assert Path(source).read_bytes() == saved.read_bytes()
    copied.append({'sdk': sdk, 'currentSource': binding(source), 'executedCopy': binding(saved)})

def inspect(filename):
    with zipfile.ZipFile(filename) as archive:
        identity = json.loads(archive.read('ankiforge-identity.json'))
        assert identity['format_version'] == 'ankiforge-identity-v1'
        db = sqlite3.connect(':memory:')
        db.deserialize(zstd.decompress(archive.read('collection.anki21b')))
        assert db.execute('pragma integrity_check').fetchone() == ('ok',)
        notes = db.execute('select guid,flds from notes order by guid').fetchall()
        cards = [row[0] for row in db.execute('select ord from cards order by nid,ord')]
        decks = [row[0] for row in db.execute('select name from decks order by name')]
        db.close()
        assets = [zstd.decompress(archive.read(name)) for name in archive.namelist() if name.isdigit()]
        active = {key: value for key, value in identity['identity']['notes'].items() if value['active']}
        assert sorted(note['guid'] for note in active.values()) == [row[0] for row in notes]
        return {'package': binding(filename), 'sqliteIntegrity': 'ok', 'notes': notes, 'cardOrdinals': cards, 'decks': decks, 'mediaSha256': sorted(sha(data) for data in assets), 'identity': identity}

outputs = []
for sdk in programs:
    observed = inspect(EX / 'basic' / sdk / 'spanish.apkg')
    assert len(observed['notes']) == 1 and observed['cardOrdinals'] == [0]
    assert not observed['mediaSha256']
    assert observed['notes'][0][1].split('\x1f')[:2] == ['hola', 'hello']
    assert observed['identity']['identity']['namespace'] == 'spanish'
    assert set(observed['identity']['identity']['notes']) == {'es:hola'}
    assert 'Spanish' in observed['decks']
    observed['task'] = sdk + '-basic'
    outputs.append(observed)

expected_media = sorted(sha(Path('docs/recipes/json-to-anki', name).read_bytes()) for name in ['cell.png', 'tone.wav'])
comparisons = []
for sdk in programs:
    pair = [inspect(EX / 'public-json' / sdk / f'v{v}.apkg') for v in [1, 2]]
    for v, observed in enumerate(pair, start=1):
        assert len(observed['notes']) == 3 and observed['cardOrdinals'] == [0, 0, 0]
        assert observed['mediaSha256'] == expected_media
        expected = load(Path('docs/recipes/json-to-anki') / f'v{v}.json')
        assert observed['identity']['identity']['namespace'] == 'biology-json'
        for row in expected['notes']:
            fields = next(fields.split('\x1f') for _, fields in observed['notes'] if fields.startswith(row['front']))
            assert fields[1] == row['back']
            assert '<img src=' in '\x1f'.join(fields) and '[sound:' in '\x1f'.join(fields)
            assert row['id'] in observed['identity']['identity']['notes']
        observed['task'] = sdk + '-json-v' + str(v)
        outputs.append(observed)
    first, second = pair
    assert [row[0] for row in first['notes']] == [row[0] for row in second['notes']]
    assert sum(a != b for a, b in zip(first['notes'], second['notes'])) == 1
    for key, note in first['identity']['identity']['notes'].items():
        for field in ['guid', 'model', 'active', 'cards', 'masks']:
            assert note[field] == second['identity']['identity']['notes'][key][field]
    for name in ['comparison.json', 'review.json']:
        path = EX / 'public-json' / sdk / name
        report = load(path)
        assert report['policy']['allows_publication'] is True
        assert report['highest_risk'] == 'low'
        assert [f['code'] for f in report['findings']] == ['RISK.NOTE_CHANGED']
        assert report['baseline_counts'] == report['candidate_counts'] == {'notes': 3, 'cards': 3, 'media': 2}
        comparisons.append({**binding(path), 'sdk': sdk, 'highestRisk': 'low', 'policyAllowsPublication': True, 'findings': ['RISK.NOTE_CHANGED']})
recovery = inspect(EX / 'basic/python/recovered.apkg')
assert len(recovery['notes']) == 2 and recovery['cardOrdinals'] == [0, 0]
assert 'NOTE.KEY_DUPLICATE' in (EX / 'basic/python/troubleshooting.log').read_text()
recovery['task'] = 'troubleshooting-recovery'
outputs.append(recovery)
records = []
for filename in ['basic/rust/Cargo.lock', 'public-json/rust/Cargo.lock']:
    data = (EX / filename).read_text()
    block = next(part for part in data.split('[[package]]') if '\nname = "ankiforge"\n' in part)
    assert 'version = "0.3.1"' in block and 'registry+https://github.com/rust-lang/crates.io-index' in block
    records.append({**binding(EX / filename), 'registry': 'crates.io', 'version': '0.3.1'})
pip = load(EX / 'basic/python/pip-report.json')['install']
package = next(p for p in pip if p['metadata']['name'] == 'ankiforge')
assert package['metadata']['version'] == '0.3.1'
assert package['download_info']['url'].startswith('https://files.pythonhosted.org/')
records.append({**binding(EX / 'basic/python/pip-report.json'), 'registry': 'PyPI', 'version': '0.3.1', 'wheel': package['download_info']})
node = load(EX / 'basic/node/package-lock.json')['packages']['node_modules/ankiforge']
assert node['version'] == '0.3.1' and node['resolved'].startswith('https://registry.npmjs.org/')
records.append({**binding(EX / 'basic/node/package-lock.json'), 'registry': 'npm', 'version': '0.3.1', 'integrity': node['integrity']})
for sdk in programs:
    records.append({**binding(EX / 'public-json' / sdk / 'installation.json'), 'sdk': sdk, 'installation': load(EX / 'public-json' / sdk / 'installation.json')})

tours = {environment: load(BASE / f'usage-browser-{environment}' / 'browser-tour.json') for environment in ['prefix', 'root']}
browser_pairs = []
for locale in ['en', 'zh-CN', 'es']:
    for prefix, root in zip(tours['prefix']['entries'][locale], tours['root']['entries'][locale]):
        assert prefix['id'] == root['id']
        assert prefix['body'] == root['body'] and prefix['codeBlocks'] == root['codeBlocks']
        for task in [prefix, root]:
            for program in task['copiedPrograms']:
                data = Path(program['source']).read_bytes()
                assert sha(data) == program['sourceSha256']
                assert data.decode().strip() == program['copiedCode'].strip()
            for resource in task['downloadedResources']:
                assert sha(Path(resource['path']).read_bytes()) == resource['sha256']
        browser_pairs.append({'locale': locale, 'id': prefix['id'], 'renderedBodyEqualAcrossActualTours': True, 'codeBlocksEqual': True, 'bodySha256': sha(prefix['body'].encode())})

result = {
    'reviewerRun': '/root/spanish_standards/independent-usage-2026-10-31',
    'scope': 'Coordinator freshly installed and executed the public 0.3.1 SDKs in this batch. This independent reviewer read their logs, inspected the runner and independent verifier sources, verified exact current source/dependency hashes and preserved basic copies, and independently decoded all 10 output APKGs through Python ZIP/Zstandard/SQLite. This is a fresh output inspection, not a claim of independently rerunning SDK installs or Anki GUI import.',
    'inputSnapshots': [binding(EX / 'basic/execution-inputs.json'), binding(EX / 'public-recipe-inputs.json')],
    'currentSourceDependencyBindings': list(current.values()), 'basicExecutedProgramCopies': copied,
    'outputObservations': outputs, 'comparisonReports': comparisons, 'publicInstallations': records,
    'browserTaskTextPairs': browser_pairs,
    'limitations': ['JSON runner copies the source into its temporary consumer and deletes that temporary directory; its source snapshot and reviewed runner/logs provide provenance. Retained exact executed basic copies provide direct byte equality for the three basic tasks.', 'Repository Cargo/package bindings document current project inputs; actual public consumer locks, PyPI wheel hashes and npm integrity records identify the packages that ran.', 'No Anki GUI import, live CDN/header verification, or universal operating-system/runtime claim.'],
}
output = BASE / 'usage-execution-audit.json'
output.write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
print(json.dumps({'path': str(output), 'sha256': sha(output.read_bytes()), 'currentBindings': len(current), 'inspectedApkgs': len(outputs), 'browserTaskPairs': len(browser_pairs)}))
