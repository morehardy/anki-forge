"""Bind every attempt to verified bytes; validate all newly generated container shapes."""
from pathlib import Path
import hashlib
import json
import sqlite3
import subprocess
import sys
import tempfile
import zipfile

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks'))
import bench
import verify

DEST = WORK / 'verification'
DEST.mkdir(exist_ok=True)
cases = {c['name']: c for c in json.loads((WORK / 'cases.json').read_text())}
runs = ['coarse-pass', 'fine-pass', 'memory-reproduce', 'memory-trace', 'scratch-sweep', 'deferred-index-pass', 'larger-spool-pass', 'larger-spool-rss']
attempts = {name: [json.loads(s) for s in (WORK / name / 'attempts.jsonl').read_text().splitlines()] for name in runs}
paths = {}
for name, rows in attempts.items():
    for row in rows:
        if row['sample'] == 0:
            path = WORK / name / f'{row["case"]}-{row["variant"]}-0' / 'output.apkg'
            assert bench.sha256(path) == row['artifact_sha256']
            paths.setdefault(row['artifact_sha256'], (row['case'], path))

# These exact accepted containers were independently checked and imported in the prior sealed archive.
prior = ROOT / 'benchmarks/results/20260907-streaming-followup/implementation/verification/verification.json'
prior_results = json.loads(prior.read_text())
known = {item['checks']['accepted']['artifact_sha256']: {'case': case, 'evidence': str(prior.relative_to(ROOT)),
                                                       'status': item['checks']['accepted']['status']}
         for case, item in prior_results.items()}
assert all(item['status'] == 'passed' for item in known.values())
checked = {}


def verify_new(digest, case, path):
    document = json.loads(Path(cases[case]['path']).read_text())
    folder = DEST / (case + '-' + digest[:12])
    if (folder / 'artifact.json').exists() and (folder / 'anki.json').exists():
        result = json.loads((folder / 'artifact.json').read_text())
        assert result['status'] == 'passed' and result['artifact_sha256'] == bench.sha256(path) == digest
        assert json.loads((folder / 'anki.json').read_text())['status'] == 'passed'
        return {'case': case, 'artifact': result, 'anki': 'passed', 'reused_completed_check': True,
                'anki_sha256': bench.sha256(folder / 'anki.json')}
    folder.mkdir(exist_ok=True)
    result = verify.verify_artifact(path, document, bench.INSPECTOR)
    (folder / 'artifact.json').write_text(json.dumps(result, indent=2) + '\n')
    assert result['status'] == 'passed', result
    command = [str(bench.ORACLE), cases[case]['path'], str(path), str(folder / 'anki.json')]
    with (folder / 'oracle.stdout.log').open('w') as out, (folder / 'oracle.stderr.log').open('w') as err:
        subprocess.run(command, stdout=out, stderr=err, check=True, timeout=120)
    assert json.loads((folder / 'anki.json').read_text())['status'] == 'passed'
    return {'case': case, 'artifact': result, 'anki': 'passed', 'command': command,
            'anki_sha256': bench.sha256(folder / 'anki.json')}


for digest, (case, path) in paths.items():
    if digest in known:
        assert known[digest]['case'] == case
        checked[digest] = dict(known[digest], validation='byte-identical to prior independently verified and Anki-imported artifact')
    else:
        checked[digest] = verify_new(digest, case, path)
        print('New container independently checked and Anki imported:', case, flush=True)


def logical_database(path, document):
    raw, package = verify.read_package(path, document)
    with tempfile.TemporaryDirectory(prefix='audit-sqlite-') as folder:
        db_path = Path(folder) / 'collection.sqlite'
        db_path.write_bytes(raw)
        db = sqlite3.connect(db_path.as_uri() + '?mode=ro&immutable=1', uri=True)
        try:
            assert db.execute('pragma integrity_check').fetchall() == [('ok',)]
            schema = db.execute('select type, name, tbl_name, sql from sqlite_master order by type, name').fetchall()
            tables = {}
            for (table,) in db.execute("select name from sqlite_master where type='table' order by name"):
                identifier = '"' + table.replace('"', '""') + '"'
                columns = db.execute('pragma table_info(' + identifier + ')').fetchall()
                order = ','.join(str(i+1) for i in range(len(columns)))
                content = hashlib.sha256()
                count = 0
                for row in db.execute('select * from ' + identifier + ' order by ' + order):
                    content.update(repr(tuple(row)).encode())
                    content.update(b'\n')
                    count += 1
                tables[table] = {'columns': columns, 'rows': count, 'row_sha256': content.hexdigest()}
        finally:
            db.close()
    with zipfile.ZipFile(path) as archive:
        other_entries = {name: hashlib.sha256(archive.read(name)).hexdigest()
                         for name in archive.namelist() if name != package['canonical_entry']}
    return {'schema': schema, 'tables': tables, 'non_collection_zip_entries': other_entries}


equivalence = {}
for case in ['text-1000', 'text-10000', 'long-back-1000']:
    document = json.loads(Path(cases[case]['path']).read_text())
    evidence = {v: logical_database(WORK / 'deferred-index-pass' / f'{case}-{v}-0' / 'output.apkg', document)
                for v in ['current', 'scratch-defer-indexes']}
    left, right = evidence['current'], evidence['scratch-defer-indexes']
    differing_tables = [name for name in sorted(set(left['tables']) | set(right['tables']))
                        if left['tables'].get(name) != right['tables'].get(name)]
    equal = left == right
    equivalence[case] = {'full_table_equivalence': 'passed' if equal else 'failed',
                         'schema_equal': left['schema'] == right['schema'],
                         'other_zip_entries_equal': left['non_collection_zip_entries'] == right['non_collection_zip_entries'],
                         'differing_tables': differing_tables, 'evidence': evidence}
    # Preserve the failed gate. The candidate is rejected, not repaired or silently
    # made equivalent by excluding planner statistics from the comparison.
    print('Full table equivalence:', case, equivalence[case]['full_table_equivalence'], differing_tables, flush=True)

bindings = {}
for name, rows in attempts.items():
    for row in rows:
        assert row['artifact_sha256'] in checked, row
        assert checked[row['artifact_sha256']]['case'] == row['case']
    bindings[name] = {'exports': len(rows), 'unique_artifacts': len({r['artifact_sha256'] for r in rows}),
                      'all_content_and_anki_bindings_verified': True,
                      'strict_candidate_equivalence': 'failed: sqlite_stat1 changed' if name == 'deferred-index-pass' else 'passed: identical bytes'}

baseline = json.loads((WORK / 'before-source-sha256.json').read_text())
assert all(bench.sha256(ROOT / path) == value for path, value in baseline.items())
build_records = json.loads((ROOT / 'benchmarks/.tools/build-records.json').read_text())
adapter = ROOT / 'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
assert bench.sha256(adapter) == build_records[str(adapter)]['executable_sha256'] == bench.sha256(WORK / 'binaries/current')
result = {'status': 'audit_complete_with_rejected_index_candidate', 'production_changes': False,
          'attempt_bindings': bindings, 'artifacts': checked, 'new_independent_artifact_and_anki_checks': sum('anki' in x for x in checked.values()),
          'source_files_unchanged': len(baseline), 'main_adapter_restored_sha256': bench.sha256(adapter),
          'deferred_index_equivalence': equivalence}
(DEST / 'validation.json').write_text(json.dumps(result, indent=2) + '\n')
print('All content bindings checked; index candidate failed full-table equivalence. Production source and measured adapter unchanged.', flush=True)
