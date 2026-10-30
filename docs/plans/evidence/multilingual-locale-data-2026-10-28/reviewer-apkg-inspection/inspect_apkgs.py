"""Independent reviewer APKG decoder (stdlib only: zipfile, compression.zstd, sqlite3, hashlib).

Does not use the repository inspector (scripts/packaged_consumer) or any Anki Forge code.
Usage: python3 -I inspect_apkgs.py <repository-root> <output.json>
"""
import hashlib, json, sqlite3, sys, tempfile, zipfile, os
from compression import zstd

root, output = sys.argv[1], sys.argv[2]
N = 'docs/plans/evidence/multilingual-locale-data-2026-10-28/executions'
sha256 = lambda b: hashlib.sha256(b).hexdigest()

def varint(buf, i):
    shift = value = 0
    while True:
        byte = buf[i]; i += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80: return value, i
        shift += 7

def protobuf(buf):
    """Decode one protobuf message into {field: [values]} (wire types 0 and 2 only)."""
    out, i = {}, 0
    while i < len(buf):
        key, i = varint(buf, i)
        field, wire = key >> 3, key & 7
        if wire == 0: value, i = varint(buf, i)
        elif wire == 2:
            length, i = varint(buf, i); value = buf[i:i + length]; i += length
        else: raise ValueError(f'unsupported wire type {wire}')
        out.setdefault(field, []).append(value)
    return out

def inspect(relative):
    path = os.path.join(root, relative)
    raw = open(path, 'rb').read()
    record = {'path': relative, 'sha256': sha256(raw), 'bytes': len(raw)}
    with zipfile.ZipFile(path) as archive:
        record['zipEntries'] = sorted(archive.namelist())
        collection = zstd.decompress(archive.read('collection.anki21b'))
        with tempfile.TemporaryDirectory() as temp:
            db_path = os.path.join(temp, 'c.db'); open(db_path, 'wb').write(collection)
            db = sqlite3.connect(db_path)
            record['sqliteIntegrity'] = db.execute('pragma integrity_check').fetchone()[0]
            decks = dict(db.execute('select id, name from decks'))
            notetypes = dict(db.execute('select id, name from notetypes'))
            record['notes'] = [{'guid': g, 'notetype': notetypes.get(m), 'fields': f.split('\x1f'), 'tags': t.strip()} for g, m, f, t in db.execute('select guid, mid, flds, tags from notes order by guid')]
            record['cards'] = [{'noteGuid': g, 'ord': o, 'deck': decks.get(d)} for g, o, d in db.execute('select n.guid, c.ord, c.did from cards c join notes n on n.id = c.nid order by n.guid, c.ord')]
            db.close()
        media_map = protobuf(zstd.decompress(archive.read('media'))) if 'media' in archive.namelist() else {}
        media = []
        for ordinal, entry in enumerate(media_map.get(1, [])):
            fields = protobuf(entry)
            name = fields[1][0].decode(); size = fields.get(2, [0])[0]; sha1 = fields.get(3, [b''])[0].hex()
            data = zstd.decompress(archive.read(str(ordinal)))
            media.append({'name': name, 'declaredSize': size, 'actualSize': len(data), 'sha1Matches': hashlib.sha1(data).hexdigest() == sha1, 'sha256': sha256(data)})
        record['media'] = media
        if 'ankiforge-identity.json' in archive.namelist():
            identity = json.loads(archive.read('ankiforge-identity.json'))
            notes = identity.get('identity', {}).get('notes', {})
            record['identity'] = {'format_version': identity.get('format_version'), 'namespace': identity.get('identity', {}).get('namespace', identity.get('namespace')), 'notes': {key: {'guid': value.get('guid'), 'active': value.get('active')} for key, value in sorted(notes.items())}}
    record['counts'] = {'notes': len(record['notes']), 'cards': len(record['cards']), 'media': len(record['media'])}
    return record

fixtures = {name: sha256(open(os.path.join(root, 'docs/recipes/json-to-anki', name), 'rb').read()) for name in ['cell.png', 'tone.wav']}
targets = [f'{N}/basic/{language}/spanish.apkg' for language in ['rust', 'python', 'node']] + [f'{N}/basic/python/recovered.apkg'] + [f'{N}/public-json/{language}/{version}.apkg' for language in ['rust', 'python', 'node'] for version in ['v1', 'v2']]
records = {target: inspect(target) for target in targets}
checks = []
def check(name, ok, detail=None):
    checks.append({'check': name, 'passed': bool(ok), **({'detail': detail} if detail is not None else {})})

for language in ['rust', 'python', 'node']:
    r = records[f'{N}/basic/{language}/spanish.apkg']
    check(f'{language} basic: sqlite ok, 1 note/1 card/0 media', r['sqliteIntegrity'] == 'ok' and r['counts'] == {'notes': 1, 'cards': 1, 'media': 0}, r['counts'])
    check(f'{language} basic: fields hola/hello, Basic notetype, Spanish deck', r['notes'][0]['fields'] == ['hola', 'hello'] and r['cards'][0]['deck'] == 'Spanish', {'fields': r['notes'][0]['fields'], 'notetype': r['notes'][0]['notetype'], 'deck': r['cards'][0]['deck']})
    check(f'{language} basic: identity key es:hola maps to the note GUID', r.get('identity', {}).get('notes', {}).get('es:hola', {}).get('guid') == r['notes'][0]['guid'], r.get('identity'))
guids = {records[f'{N}/basic/{language}/spanish.apkg']['notes'][0]['guid'] for language in ['rust', 'python', 'node']}
check('basic: same stable GUID across the three SDKs for spanish/es:hola', len(guids) == 1, sorted(guids))
rec = records[f'{N}/basic/python/recovered.apkg']
check('troubleshooting recovered: 2 notes/2 cards/0 media, keys source:1 and source:2, Recovery deck', rec['counts'] == {'notes': 2, 'cards': 2, 'media': 0} and sorted(rec.get('identity', {}).get('notes', {})) == ['source:1', 'source:2'] and {c['deck'] for c in rec['cards']} == {'Recovery'}, {'counts': rec['counts'], 'fields': sorted(n['fields'] for n in rec['notes']), 'identity': rec.get('identity')})
check('troubleshooting recovered: the rejected duplicate ("two","second" under source:1) was not stored under source:1', sorted(n['fields'] for n in rec['notes']) == [['one', 'first'], ['two', 'second']])
for language in ['rust', 'python', 'node']:
    v1, v2 = records[f'{N}/public-json/{language}/v1.apkg'], records[f'{N}/public-json/{language}/v2.apkg']
    for label, r in [('v1', v1), ('v2', v2)]:
        check(f'{language} {label}: sqlite ok, 3 notes/3 cards/2 media, Biology deck', r['sqliteIntegrity'] == 'ok' and r['counts'] == {'notes': 3, 'cards': 3, 'media': 2} and {c['deck'] for c in r['cards']} == {'Biology'}, r['counts'])
        # The SDK stores media under content-derived names that keep the extension; compare bytes, not names.
        by_ext = {m['name'].rsplit('.', 1)[-1]: m for m in r['media']}
        exact = sorted(by_ext) == ['png', 'wav'] and by_ext['png']['sha256'] == fixtures['cell.png'] and by_ext['wav']['sha256'] == fixtures['tone.wav'] and all(m['sha1Matches'] and m['declaredSize'] == m['actualSize'] for m in r['media'])
        check(f'{language} {label}: the two stored media are byte-exact cell.png/tone.wav fixtures (content-derived names, extension kept)', exact, [{k: m[k] for k in ('name', 'sha256', 'actualSize')} for m in r['media']])
        referenced = all(f'<img src="{by_ext["png"]["name"]}">' in n['fields'][0] and f'[sound:{by_ext["wav"]["name"]}]' in n['fields'][0] for n in r['notes']) if exact else False
        check(f'{language} {label}: every note front references the stored image and sound', referenced)
    k1, k2 = v1['identity']['notes'], v2['identity']['notes']
    check(f'{language}: identity keys bio:cell/bio:membrane/bio:nucleus with identical GUIDs in v1 and v2', sorted(k1) == sorted(k2) == ['bio:cell', 'bio:membrane', 'bio:nucleus'] and all(k1[k]['guid'] == k2[k]['guid'] for k in k1), {k: k1[k]['guid'] for k in k1})
    by_guid = lambda r: {n['guid']: n['fields'] for n in r['notes']}
    f1, f2 = by_guid(v1), by_guid(v2)
    changed = {key: [f1[k1[key]['guid']], f2[k2[key]['guid']]] for key in k1 if f1[k1[key]['guid']] != f2[k2[key]['guid']]}
    check(f'{language}: only bio:cell changed between v1 and v2', list(changed) == ['bio:cell'], changed)
    cell1, cell2 = f1[k1['bio:cell']['guid']], f2[k2['bio:cell']['guid']]
    check(f'{language}: bio:cell back text changed A unit of life -> The basic structural and functional unit of life; front and media references unchanged', 'A unit of life' in ' '.join(cell1) and 'The basic structural and functional unit of life' in ' '.join(cell2) and cell1[0] == cell2[0], {'v1': cell1, 'v2': cell2})
v1guids = {language: {k: v['guid'] for k, v in records[f'{N}/public-json/{language}/v1.apkg']['identity']['notes'].items()} for language in ['rust', 'python', 'node']}
check('JSON recipe: the three SDKs assign the same GUID to each stable key', v1guids['rust'] == v1guids['python'] == v1guids['node'], v1guids)
result = {'schemaVersion': 1, 'tool': 'reviewer-apkg-inspection/inspect_apkgs.py (stdlib zipfile/compression.zstd/sqlite3; no Anki Forge code)', 'python': sys.version.split()[0], 'fixtureSha256': fixtures, 'status': 'passed' if all(c['passed'] for c in checks) else 'failed', 'checks': checks, 'packages': records}
open(output, 'w').write(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
print(result['status'], sum(c['passed'] for c in checks), '/', len(checks))
for c in checks:
    if not c['passed']: print('FAIL', c['check'], json.dumps(c.get('detail'), ensure_ascii=False)[:800])
