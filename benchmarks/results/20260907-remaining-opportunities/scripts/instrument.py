"""Isolated diagnostic build. No production probes or changed build settings."""
from pathlib import Path
import hashlib
import json
import re
import shutil

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
SRC = WORK / 'profile-source'
SRC.mkdir(exist_ok=False)
(WORK / 'binaries').mkdir(exist_ok=True)
binary = ROOT / 'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
records = json.loads((ROOT / 'benchmarks/.tools/build-records.json').read_text())
assert hashlib.sha256(binary.read_bytes()).hexdigest() == records[str(binary)]['executable_sha256']
shutil.copy2(binary, WORK / 'binaries/before')
for name in ('Cargo.lock', 'rust-toolchain.toml'):
    shutil.copy2(ROOT / name, SRC / name)
(SRC / 'Cargo.toml').write_text('''[workspace]
members = ["anki_forge"]
resolver = "2"
[workspace.package]
edition = "2021"
rust-version = "1.92"
[workspace.lints.rust]
unsafe_code = "forbid"
''')
for name in ('anki_forge', 'benchmarks/adapters/rust'):
    shutil.copytree(ROOT / name, SRC / name, ignore=shutil.ignore_patterns('target'))
files = [p for d in ('anki_forge', 'benchmarks/adapters/rust') for p in (SRC / d).rglob('*') if p.is_file()]
(WORK / 'before-source-sha256.json').write_text(json.dumps({str(p.relative_to(SRC)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}, indent=2) + '\n')
shutil.copy2(ROOT / 'benchmarks/.tools/build-records.json', WORK / 'before-build-records.json')

def edit(file, old, new):
    p = SRC / file
    s = p.read_text()
    assert s.count(old) == 1, (file, old, s.count(old))
    p.write_text(s.replace(old, new))

def scope(file, function, label):
    p = SRC / file
    s = p.read_text()
    matches = list(re.finditer(r'\bfn ' + function + r'\s*(?:<[^\n]*>)?\s*\(', s))
    assert len(matches) == 1, (file, function, len(matches))
    i = s.index('{', matches[0].end()) + 1
    p.write_text(s[:i] + f'\n    let _scale_scope = crate::diag::Scope::new("{label}");' + s[i:])

(SRC / 'anki_forge/src/diag.rs').write_text('''//! Private coarse wall-time probes, excluded from production.
use std::time::Instant;
pub struct Scope(&'static str, Instant);
impl Scope { pub fn new(label: &'static str) -> Self { Self(label, Instant::now()) } }
impl Drop for Scope { fn drop(&mut self) {
    eprintln!("[DEBUG-scale-opt] {} {}", self.0, self.1.elapsed().as_nanos());
} }
''')
p = SRC / 'anki_forge/src/lib.rs'
p.write_text(p.read_text() + '\n#[allow(missing_docs)]\npub mod diag;\n')
for file, function, label in [
    ('product/project/pipeline.rs', 'execute', 'pipeline.total'),
    ('product/project/pipeline.rs', 'prepare', 'pipeline.prepare'),
    ('product/project/pipeline.rs', 'generate', 'pipeline.generate'),
    ('product/project/pipeline.rs', 'inspect', 'pipeline.inspect'),
    ('product/project/pipeline.rs', 'publish', 'pipeline.publish'),
    ('product/project/pipeline/reconcile.rs', 'reconcile', 'pipeline.reconcile'),
    ('product/project/input.rs', 'normalize_with_prepared_media', 'normalize.total'),
    ('product/project/input.rs', 'lower_with_project_error', 'normalize.lowering'),
    ('authoring_core/normalize.rs', 'normalize_with_prepared_media', 'normalize.core'),
    ('prepared_media.rs', 'prepare_all', 'media.prepare'),
    ('writer_core/apkg.rs', 'emit_apkg_with_plans', 'apkg.total'),
    ('writer_core/apkg.rs', 'create_latest_collection_file', 'sqlite.total'),
    ('writer_core/apkg.rs', 'populate_latest_collection', 'sqlite.populate'),
    ('writer_core/inspect.rs', 'read_apkg_facts', 'inspect.facts'),
    ('writer_core/inspect.rs', 'read_collection_data', 'inspect.sqlite'),
    ('writer_core/inspect.rs', 'read_media_entries', 'inspect.media'),
]:
    scope('anki_forge/src/' + file, function, label)
f = 'anki_forge/src/writer_core/apkg.rs'
edit(f, '        let transaction = conn.unchecked_transaction()?;', '        let _schema = crate::diag::Scope::new("sqlite.schema");\n        let transaction = conn.unchecked_transaction()?;')
edit(f, '    let compacted =\n', '    let _compact = crate::diag::Scope::new("sqlite.compact");\n    let compacted =\n')
edit(f, '        zstd::stream::copy_encode(File::open(collection.path())?, &mut measured, 0)?;', '        let _compress = crate::diag::Scope::new("apkg.collection_zstd");\n        zstd::stream::copy_encode(File::open(collection.path())?, &mut measured, 0)?;')
f = 'benchmarks/adapters/rust/src/main.rs'
edit(f, '    let workload: Workload =', '    let _total = anki_forge::diag::Scope::new("adapter.total");\n    let parse = anki_forge::diag::Scope::new("adapter.parse");\n    let workload: Workload =')
edit(f, '    let mut deck = Deck::new(workload.deck_name);', '    drop(parse);\n    let register = anki_forge::diag::Scope::new("adapter.registration");\n    let mut deck = Deck::new(workload.deck_name);')
edit(f, '    for note in workload.notes {', '    drop(register);\n    let notes = anki_forge::diag::Scope::new("adapter.notes");\n    for note in workload.notes {')
edit(f, '    deck.write_apkg(output)?.ensure_success()?;', '    drop(notes);\n    deck.write_apkg(output)?.ensure_success()?;')
print(SRC)
