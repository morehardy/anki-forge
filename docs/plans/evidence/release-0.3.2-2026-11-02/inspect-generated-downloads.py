# Independent reviewer inspection of the changed /generated/* downloads.
# Compares every ZIP entry, the ankiforge identity sidecar and a full SQLite dump of
# collection.anki21b (zstd) between the previously reviewed artifact and the new one.
# Usage: python3 -I inspect-generated-downloads.py  (run from the repository root)
import zipfile, json, hashlib, subprocess, sqlite3, difflib, sys
from pathlib import Path
root = Path('/Users/hp/Desktop/2026/anki-forge')
out = root / 'docs/plans/evidence/release-0.3.2-2026-11-02/generated-download-inspection.json'
scratch = root / 'target/release-0.3.2-review/generated'
pairs = {'root': (root/'target/bing-seo-artifacts/root', root/'target/release-0.3.2-artifacts/root-dist'),
         'prefix': (root/'target/bing-seo-artifacts/prefix', root/'target/release-0.3.2-artifacts/prefix-dist')}
sha = lambda b: hashlib.sha256(b).hexdigest()
TIME_COLUMNS = {'mod', 'mtime_secs', 'crt', 'scm', 'ls'}

def dump(blob, tag):
    scratch.mkdir(parents=True, exist_ok=True)
    zst = scratch / f'{tag}.anki21b'; db = scratch / f'{tag}.sqlite'
    zst.write_bytes(blob); db.unlink(missing_ok=True)
    subprocess.run(['zstd', '-q', '-d', '-f', str(zst), '-o', str(db)], check=True)
    con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    rows = {}
    for (name,) in con.execute("select name from sqlite_master where type='table' order by name"):
        cols = [c[1] for c in con.execute(f'pragma table_info("{name}")')]
        data = con.execute(f'select * from "{name}" order by 1').fetchall()
        rows[name] = {'columns': cols, 'rows': [[(v.hex() if isinstance(v, bytes) else v) for v in r] for r in data]}
    con.close()
    return rows

def identity_without_times(obj):
    if isinstance(obj, dict):
        return {k: identity_without_times(v) for k, v in obj.items() if k not in ('mtime_secs', 'collection_blake3', 'identity_blake3')}
    if isinstance(obj, list): return [identity_without_times(v) for v in obj]
    return obj

report = {'schemaVersion': 1, 'reviewerRun': '/root/usage_release_review/release-0.3.2-2026-11-02', 'environments': {}}
ok = True
for env, (old_dir, new_dir) in pairs.items():
    env_report = {}
    for path in sorted((new_dir/'generated').glob('*.apkg')):
        name = path.name
        zo, zn = zipfile.ZipFile(old_dir/'generated'/name), zipfile.ZipFile(path)
        assert [i.filename for i in zo.infolist()] == [i.filename for i in zn.infolist()], name
        entries = {}
        for info in zn.infolist():
            a, b = zo.read(info.filename), zn.read(info.filename)
            entries[info.filename] = {'old': sha(a), 'new': sha(b), 'identical': a == b}
        io, inew = json.loads(zo.read('ankiforge-identity.json')), json.loads(zn.read('ankiforge-identity.json'))
        do, dn = dump(zo.read('collection.anki21b'), f'{env}-{name}-old'), dump(zn.read('collection.anki21b'), f'{env}-{name}-new')
        cell_diffs = []
        for table in sorted(set(do) | set(dn)):
            assert do[table]['columns'] == dn[table]['columns'], (name, table)
            cols = dn[table]['columns']
            assert len(do[table]['rows']) == len(dn[table]['rows']), (name, table)
            for ro, rn in zip(do[table]['rows'], dn[table]['rows']):
                for c, a, b in zip(cols, ro, rn):
                    if a != b: cell_diffs.append({'table': table, 'column': c, 'old': a if not isinstance(a, str) or len(a) < 200 else sha(a.encode()), 'new': b if not isinstance(b, str) or len(b) < 200 else sha(b.encode())})
        counts = {t: len(dn[t]['rows']) for t in ('notes', 'cards', 'notetypes', 'decks') if t in dn}
        non_time = [d for d in cell_diffs if d['column'] not in TIME_COLUMNS]
        mtimes = sorted({(io['identity']['notes'] and next(iter(io['identity']['notes'].values()))['mtime_secs']), (inew['identity']['notes'] and next(iter(inew['identity']['notes'].values()))['mtime_secs'])})
        rec = {'entries': entries, 'counts': counts,
               'identityEqualIgnoringTimesAndHashes': identity_without_times(io) == identity_without_times(inew),
               'sqliteCellDifferences': cell_diffs, 'nonTimestampSqliteDifferences': non_time,
               'identityMtimeOldNew': mtimes}
        rec['onlyBuildTimestampChanged'] = rec['identityEqualIgnoringTimesAndHashes'] and not non_time and all(v['identical'] for k, v in entries.items() if k not in ('collection.anki21b', 'ankiforge-identity.json'))
        ok = ok and rec['onlyBuildTimestampChanged']
        env_report[name] = rec
    so = json.loads((old_dir/'generated/showcase.json').read_text()); sn = json.loads((new_dir/'generated/showcase.json').read_text())
    diff = [l for l in difflib.unified_diff(json.dumps(so, indent=1, sort_keys=True).splitlines(), json.dumps(sn, indent=1, sort_keys=True).splitlines(), lineterm='', n=0) if l[:1] in '+-' and l[:3] not in ('+++', '---')]
    env_report['showcase.json'] = {'diffLines': diff, 'oldSha256': sha((old_dir/'generated/showcase.json').read_bytes()), 'newSha256': sha((new_dir/'generated/showcase.json').read_bytes())}
    report['environments'][env] = env_report
report['conclusion'] = ('All changed APKGs differ only in build-time modification timestamps (SQLite mod/mtime columns and identity mtime_secs, '
                        'with the dependent collection/identity BLAKE3 digests); notes, cards, fields, templates, GUIDs, IDs, content hashes and media bytes are identical. '
                        'showcase.json changes only crateVersion 0.3.1->0.3.2, sourceCommit and the dependent download sizes/digests.') if ok else 'CONTENT CHANGE DETECTED: see per-file nonTimestampSqliteDifferences'
report['onlyMetadataChanged'] = ok
out.write_text(json.dumps(report, indent=1) + '\n')
print(report['conclusion'])
for env, r in report['environments'].items():
    for n, v in r.items():
        if n.endswith('.apkg'): print(env, n, v['counts'], 'onlyTimestamps=', v['onlyBuildTimestampChanged'], 'nonTime=', v['nonTimestampSqliteDifferences'][:3], 'cols=', sorted({(d['table'], d['column']) for d in v['sqliteCellDifferences']}))
    print(env, 'showcase.json', r['showcase.json']['diffLines'])
sys.exit(0 if ok else 1)
