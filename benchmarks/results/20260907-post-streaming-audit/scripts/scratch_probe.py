"""Uninstrumented, single-variable scratch-SQLite probes; not production changes."""
from pathlib import Path
import json
import shutil
import sys

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
SRC = WORK / 'profile-source'
kind = sys.argv[1]
assert kind in ('scratch-sync-off', 'scratch-journal-memory', 'scratch-defer-indexes')
if (SRC / 'benchmarks/adapters/rust/src/memory.rs').exists():
    shutil.move(SRC, WORK / 'memory-source')
    SRC.mkdir()
    for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'):
        shutil.copy2(WORK / 'coarse-source' / name, SRC / name)
    for directory in ('anki_forge', 'benchmarks/adapters/rust'):
        shutil.copytree(ROOT / directory, SRC / directory, ignore=shutil.ignore_patterns('target'))

p = SRC / 'anki_forge/src/writer_core/apkg.rs'
s = (ROOT / 'anki_forge/src/writer_core/apkg.rs').read_text()
anchor = '        .with_context(|| format!("open collection database {}", path.display()))?;'
assert s.count(anchor) == 1
if kind == 'scratch-defer-indexes':
    indexes = [
        'CREATE INDEX ix_notes_usn ON notes (usn);',
        'CREATE INDEX ix_cards_usn ON cards (usn);',
        'CREATE INDEX ix_cards_nid ON cards (nid);',
        'CREATE INDEX ix_cards_sched ON cards (did, queue, due);',
        'CREATE INDEX ix_notes_csum ON notes (csum);',
        'CREATE INDEX idx_notes_mid ON notes (mid);',
        'CREATE INDEX idx_cards_odid ON cards (odid) WHERE odid != 0;',
    ]
    statement = 'Defer seven non-unique note/card indexes until after row insertion, inside the same transaction.'
    s += '\n// Private diagnostic candidate only.\nconst AUDIT_DEFERRED_INDEXES: &[&str] = &[\n' + ''.join('    '+json.dumps(i)+',\n' for i in indexes) + '];\n'
    old = '    let sql = sql.replace("COLLATE unicase", "");'
    assert s.count(old) == 1
    s = s.replace(old, old + '\n    let sql = AUDIT_DEFERRED_INDEXES.iter().fold(sql, |sql, index| sql.replace(index, ""));')
    old = '    populate_latest_collection_rows(&transaction, normalized_ir, guid_assignments, notetype_ids)?;\n    transaction.commit()?;'
    assert s.count(old) == 1
    s = s.replace(old, old.replace('    transaction.commit()?;', '    for index in AUDIT_DEFERRED_INDEXES { transaction.execute_batch(index)?; }\n    transaction.commit()?;'))
    p.write_text(s)
else:
    statement = ('    conn.pragma_update(None, "synchronous", "OFF")?;' if kind == 'scratch-sync-off'
                 else '    conn.pragma_update(None, "journal_mode", "MEMORY")?;')
    p.write_text(s.replace(anchor, anchor + '\n' + statement))
(WORK / (kind + '-plan.json')).write_text(json.dumps({
    'candidate': kind,
    'variable': statement.strip(),
    'scope': 'Only create_latest_collection_file() and its unpublished disposable SQLite file',
    'unchanged': ['Public API', 'registration fingerprints', 'media/source integrity checks',
                  'collection rows and schema', 'VACUUM INTO', 'mandatory inspection',
                  'final APKG flush/sync/atomic publication'],
    'gate': ('Physical container may differ; require deterministic bytes within each variant, complete raw schema/row equivalence, input/content validation and Anki import/render.' if kind == 'scratch-defer-indexes' else 'Every final APKG must match current production bytes exactly.') + ' No production adoption in this audit.',
}, indent=2) + '\n')
print(kind, 'prepared as a single-variable private experiment')
