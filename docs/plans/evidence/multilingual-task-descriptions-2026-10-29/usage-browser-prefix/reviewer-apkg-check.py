"""Independent reviewer APKG check for the reused 2026-10-29 SDK executions (stdlib only).

Uses zipfile, compression.zstd, sqlite3 and hashlib; no Anki Forge code and not the repository
inspector (scripts/packaged_consumer). Decodes each produced APKG and asserts the outcomes the
task pages promise. It records facts and per-check booleans; judgement is the reviewer's.

Usage: python3 -I reviewer-apkg-check.py <repository-root> <output.json>
"""
import hashlib, json, os, sqlite3, sys, tempfile, zipfile
from compression import zstd

root, output = sys.argv[1], sys.argv[2]
E = 'docs/plans/evidence/multilingual-locale-data-2026-10-28/executions'
sha256 = lambda data: hashlib.sha256(data).hexdigest()
sha1 = lambda data: hashlib.sha1(data).hexdigest()


def varint(buf, i):
    shift = value = 0
    while True:
        byte = buf[i]; i += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, i
        shift += 7


def message(buf):
    """Decode one protobuf message into {field: [values]} (wire types 0 and 2 only)."""
    out, i = {}, 0
    while i < len(buf):
        key, i = varint(buf, i)
        field, wire = key >> 3, key & 7
        if wire == 0:
            value, i = varint(buf, i)
        elif wire == 2:
            length, i = varint(buf, i); value = buf[i:i + length]; i += length
        else:
            raise ValueError(f'unsupported wire type {wire}')
        out.setdefault(field, []).append(value)
    return out


def decode(relative):
    path = os.path.join(root, relative)
    raw = open(path, 'rb').read()
    record = {'path': relative, 'sha256': sha256(raw), 'bytes': len(raw)}
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        record['zipEntries'] = sorted(names)
        record['metaVersion'] = message(archive.read('meta')).get(1, [None])[0]
        collection = zstd.decompress(archive.read('collection.anki21b'))
        with tempfile.TemporaryDirectory() as temp:
            db_path = os.path.join(temp, 'collection.sqlite')
            open(db_path, 'wb').write(collection)
            db = sqlite3.connect(db_path)
            record['sqliteIntegrity'] = db.execute('pragma integrity_check').fetchone()[0]
            decks = dict(db.execute('select id, name from decks'))
            notetypes = dict(db.execute('select id, name from notetypes'))
            record['notes'] = [{'guid': guid, 'notetype': notetypes.get(mid), 'fields': fields.split('\x1f')} for guid, mid, fields in db.execute('select guid, mid, flds from notes order by guid')]
            record['cards'] = [{'noteGuid': guid, 'ord': ordinal, 'deck': decks.get(did)} for guid, ordinal, did in db.execute('select n.guid, c.ord, c.did from cards c join notes n on n.id = c.nid order by n.guid, c.ord')]
            db.close()
        media = []
        if 'media' in names:
            entries = message(zstd.decompress(archive.read('media'))).get(1, [])
            for index, entry in enumerate(entries):
                fields = message(entry)
                name = fields[1][0].decode()
                data = zstd.decompress(archive.read(str(index)))
                media.append({'index': index, 'name': name, 'declaredSize': fields.get(2, [None])[0], 'declaredSha1': fields.get(3, [b''])[0].hex(), 'bytes': len(data), 'sha256': sha256(data), 'sha1Matches': sha1(data) == fields.get(3, [b''])[0].hex()})
        record['media'] = media
        identity = json.loads(archive.read('ankiforge-identity.json')) if 'ankiforge-identity.json' in names else None
        record['identity'] = None if identity is None else {
            'format_version': identity.get('format_version'),
            'namespace': identity['identity'].get('namespace'),
            'notes': {key: value['guid'] for key, value in identity['identity'].get('notes', {}).items()},
        }
    return record


def fixture(name):
    return sha256(open(os.path.join(root, 'docs/recipes/json-to-anki', name), 'rb').read())


checks = []
def check(name, condition, detail=None):
    checks.append({'check': name, 'passed': bool(condition), **({'detail': detail} if detail is not None else {})})

result = {'schemaVersion': 1, 'tool': 'reviewer-apkg-check.py', 'python': sys.version.split()[0], 'basic': {}, 'troubleshooting': None, 'publicJson': {}}

# Basic quickstart outputs: one Basic card hola -> hello in Spanish, key es:hola, same GUID across SDKs.
for sdk in ['rust', 'python', 'node']:
    record = decode(f'{E}/basic/{sdk}/spanish.apkg')
    result['basic'][sdk] = record
    check(f'basic/{sdk}: sqlite integrity ok', record['sqliteIntegrity'] == 'ok')
    check(f'basic/{sdk}: 1 note, 1 card, 0 media', (len(record['notes']), len(record['cards']), len(record['media'])) == (1, 1, 0))
    check(f'basic/{sdk}: fields hola/hello, Basic note type, Spanish deck', record['notes'] and record['notes'][0]['fields'] == ['hola', 'hello'] and record['notes'][0]['notetype'] == 'Basic' and record['cards'][0]['deck'] == 'Spanish', record['notes'])
    check(f'basic/{sdk}: identity namespace spanish with key es:hola mapping to the note GUID', record['identity'] and record['identity']['namespace'] == 'spanish' and record['identity']['notes'].get('es:hola') == record['notes'][0]['guid'], record['identity'])
guids = {sdk: result['basic'][sdk]['notes'][0]['guid'] for sdk in result['basic']}
check('basic: es:hola GUID identical across Rust, Python and Node', len(set(guids.values())) == 1, guids)

# Troubleshooting recovery: duplicate key corrected to unique keys.
record = decode(f'{E}/basic/python/recovered.apkg')
result['troubleshooting'] = record
check('troubleshooting: 2 notes, 2 cards, 0 media in deck Recovery', (len(record['notes']), len(record['cards']), len(record['media'])) == (2, 2, 0) and {card['deck'] for card in record['cards']} == {'Recovery'})
check('troubleshooting: identity keys source:1 and source:2', record['identity'] and sorted(record['identity']['notes']) == ['source:1', 'source:2'], record['identity'])
by_guid = {note['guid']: note['fields'] for note in record['notes']}
check('troubleshooting: fields one/first and two/second', record['identity'] and by_guid.get(record['identity']['notes'].get('source:1')) == ['one', 'first'] and by_guid.get(record['identity']['notes'].get('source:2')) == ['two', 'second'], by_guid)

# JSON + media recipe: v1 and v2 per SDK.
expected_media = {'cell.png': fixture('cell.png'), 'tone.wav': fixture('tone.wav')}
v1_input = json.load(open(os.path.join(root, 'docs/recipes/json-to-anki/v1.json')))
v2_input = json.load(open(os.path.join(root, 'docs/recipes/json-to-anki/v2.json')))
for sdk in ['rust', 'python', 'node']:
    versions = {version: decode(f'{E}/public-json/{sdk}/{version}.apkg') for version in ['v1', 'v2']}
    result['publicJson'][sdk] = versions
    for version, record in versions.items():
        source = v1_input if version == 'v1' else v2_input
        check(f'json/{sdk}/{version}: sqlite integrity ok', record['sqliteIntegrity'] == 'ok')
        check(f'json/{sdk}/{version}: 3 notes, 3 cards, 2 media', (len(record['notes']), len(record['cards']), len(record['media'])) == (3, 3, 2))
        check(f'json/{sdk}/{version}: media bytes equal cell.png and tone.wav and declared SHA-1s match', sorted(item['sha256'] for item in record['media']) == sorted(expected_media.values()) and all(item['sha1Matches'] for item in record['media']), [(item['name'], item['sha256']) for item in record['media']])
        check(f'json/{sdk}/{version}: media names keep their extensions', sorted(os.path.splitext(item['name'])[1] for item in record['media']) == ['.png', '.wav'])
        names = [item['name'] for item in record['media']]
        check(f'json/{sdk}/{version}: every front references both media files', all(all(name in note['fields'][0] for name in names) for note in record['notes']))
        check(f'json/{sdk}/{version}: identity ankiforge-identity-v1, namespace {source["namespace"]}, keys equal the JSON ids', record['identity'] and record['identity']['format_version'] == 'ankiforge-identity-v1' and record['identity']['namespace'] == source['namespace'] and sorted(record['identity']['notes']) == sorted(note['id'] for note in source['notes']), record['identity'])
        backs = {key: next(note['fields'][1] for note in record['notes'] if note['guid'] == guid) for key, guid in record['identity']['notes'].items()}
        check(f'json/{sdk}/{version}: backs equal the JSON answers', backs == {note['id']: note['back'] for note in source['notes']}, backs)
        check(f'json/{sdk}/{version}: all cards in the JSON deck', {card['deck'] for card in record['cards']} == {source['deck']})
    check(f'json/{sdk}: each key keeps its GUID from v1 to v2', versions['v1']['identity']['notes'] == versions['v2']['identity']['notes'])
    v1_fields = {key: next(n['fields'] for n in versions['v1']['notes'] if n['guid'] == guid) for key, guid in versions['v1']['identity']['notes'].items()}
    v2_fields = {key: next(n['fields'] for n in versions['v2']['notes'] if n['guid'] == guid) for key, guid in versions['v2']['identity']['notes'].items()}
    changed = sorted(key for key in v1_fields if v1_fields[key] != v2_fields[key])
    check(f'json/{sdk}: only bio:cell changes, and only its back', changed == ['bio:cell'] and v1_fields['bio:cell'][0] == v2_fields['bio:cell'][0], changed)
identity_maps = {sdk: result['publicJson'][sdk]['v1']['identity']['notes'] for sdk in result['publicJson']}
check('json: key -> GUID maps identical across Rust, Python and Node', len({json.dumps(value, sort_keys=True) for value in identity_maps.values()}) == 1, identity_maps)

result['checks'] = checks
result['summary'] = {'passed': sum(item['passed'] for item in checks), 'total': len(checks)}
result['status'] = 'passed' if all(item['passed'] for item in checks) else 'failed'
json.dump(result, open(output, 'w'), indent=2, ensure_ascii=False)
open(output, 'a').write('\n')
print(json.dumps(result['summary']), result['status'])
for item in checks:
    if not item['passed']:
        print('FAILED', item['check'], item.get('detail'))
