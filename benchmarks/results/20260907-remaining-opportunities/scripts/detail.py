"""Fine probes in the isolated copy; all accumulators are diagnostic only."""
from pathlib import Path
import re

WORK = Path(__file__).resolve().parent
SRC = WORK / 'profile-source'


def edit(file, old, new):
    path = SRC / file
    text = path.read_text()
    assert text.count(old) == 1, (file, old, text.count(old))
    path.write_text(text.replace(old, new))


def function(file, name, label, timer='Accum'):
    path = SRC / 'anki_forge/src' / file
    text = path.read_text()
    matches = list(re.finditer(r'\bfn ' + name + r'\s*(?:<[^\n]*>)?\s*\(', text))
    assert len(matches) == 1, (file, name, len(matches))
    offset = text.index('{', matches[0].end()) + 1
    path.write_text(text[:offset] + f'\n    let _detail = crate::diag::{timer}::new("{label}");' + text[offset:])


diag = SRC / 'anki_forge/src/diag.rs'
diag.write_text(diag.read_text() + '''
use std::{collections::BTreeMap, sync::{Mutex, OnceLock}};
static TOTALS: OnceLock<Mutex<BTreeMap<&'static str, u128>>> = OnceLock::new();
pub struct Accum(&'static str, Instant);
impl Accum { pub fn new(label: &'static str) -> Self { Self(label, Instant::now()) } }
impl Drop for Accum { fn drop(&mut self) {
    let elapsed = self.1.elapsed().as_nanos();
    *TOTALS.get_or_init(Default::default).lock().unwrap().entry(self.0).or_default() += elapsed;
} }
pub fn report() { if let Some(totals) = TOTALS.get() {
    for (key, ns) in totals.lock().unwrap().iter() { eprintln!("[DEBUG-scale-opt] {} {}", key, ns); }
} }
''')
edit('benchmarks/adapters/rust/src/main.rs', '    deck.write_apkg(output)?.ensure_success()?;',
     '    deck.write_apkg(output)?.ensure_success()?;\n    anki_forge::diag::report();')

for file, name, label in [
    ('deck/identity.rs', 'normalize_field_text_for_identity', 'note.identity_normalize'),
    ('deck/identity.rs', 'hash_payload', 'note.identity_serialize_hash'),
    ('writer_core/apkg.rs', 'note_storage_values', 'sqlite.field_storage'),
    ('writer_core/note_data.rs', 'fresh_identity_note_data', 'sqlite.identity_data'),
    ('writer_core/note_revision.rs', 'from_note', 'note.revision'),
]:
    function(file, name, label)
function('product/project/deck_import.rs', 'from_deck', 'deck.to_project', 'Scope')

f = 'anki_forge/src/deck/media.rs'
edit(f, '    let mut file = std::fs::File::open(path).map_err(|err| match err.kind() {',
     '    let mut file = { let _t = crate::diag::Accum::new("registration.open"); std::fs::File::open(path) }.map_err(|err| match err.kind() {')
edit(f, '        let read = file\n            .read(&mut buffer)',
     '        let read = { let _t = crate::diag::Accum::new("registration.read"); file.read(&mut buffer) }')
edit(f, '        hasher.update(&buffer[..read]);',
     '        { let _t = crate::diag::Accum::new("registration.hash"); hasher.update(&buffer[..read]); }')
function('deck/media.rs', 'validate_source_file', 'registration.stat')

f = 'anki_forge/src/writer_core/stream_zip.rs'
edit(f, '        let written = self.output.write(bytes)?;',
     '        let written = { let _t = crate::diag::Accum::new("zip.write"); self.output.write(bytes)? };')
edit(f, '        self.hash.update(&bytes[..written]);',
     '        { let _t = crate::diag::Accum::new("zip.hash"); self.hash.update(&bytes[..written]); }')

f = 'anki_forge/src/prepared_media.rs'
edit(f, '                                    if sender\n                                        .send(prepared.prepare_one(item, options, &mut context))',
     '''                                    let result = {
                                        let _t = crate::diag::Accum::new(["media.worker0.compute", "media.worker1.compute", "media.worker2.compute", "media.worker3.compute"][worker]);
                                        prepared.prepare_one(item, options, &mut context)
                                    };
                                    if sender
                                        .send(result)''')
edit(f, '                    let result = receivers[job_index % workers].recv().unwrap_or_else(|_| {',
     '                    let wait = crate::diag::Accum::new("media.consumer_wait");\n                    let result = receivers[job_index % workers].recv().unwrap_or_else(|_| {')
edit(f, '                    accept(index, result);', '                    drop(wait);\n                    accept(index, result);')
edit(f, '                ingested[index] = Some(result.and_then(|(metadata, mut payload)| {',
     '                let _accept = crate::diag::Accum::new("media.accept");\n                ingested[index] = Some(result.and_then(|(metadata, mut payload)| {')
edit(f, '        let written = self.data.write(bytes)?;',
     '        let written = { let _t = crate::diag::Accum::new("media.encoded_payload_write"); self.data.write(bytes)? };')

f = 'anki_forge/src/writer_core/apkg.rs'
edit(f, '    populate_latest_collection_rows(&transaction, normalized_ir, guid_assignments, notetype_ids)?;\n    transaction.commit()?;',
     '    { let _t = crate::diag::Scope::new("sqlite.rows");\n    populate_latest_collection_rows(&transaction, normalized_ir, guid_assignments, notetype_ids)?; }\n    let _t = crate::diag::Scope::new("sqlite.commit");\n    transaction.commit()?;')
edit(f, '        insert_note.execute(rusqlite::params![',
     '        let insert_timer = crate::diag::Accum::new("sqlite.note_insert_including_data");\n        insert_note.execute(rusqlite::params![')
edit(f, '        for tag in &note.tags {', '        drop(insert_timer);\n        for tag in &note.tags {')
edit(f, '            insert_card.execute(rusqlite::params![',
     '            let insert_timer = crate::diag::Accum::new("sqlite.card_insert");\n            insert_card.execute(rusqlite::params![')
edit(f, '            card_row_id += 1;', '            drop(insert_timer);\n            card_row_id += 1;')

f = 'anki_forge/src/writer_core/inspect.rs'
edit(f, '            if let Some(pending) = self.pending.take() {\n                self.hash.update(&pending);\n            }\n            self.hash.update(bytes);',
     '            let _t = crate::diag::Accum::new("inspect.serial_hash");\n            if let Some(pending) = self.pending.take() {\n                self.hash.update(&pending);\n            }\n            self.hash.update(bytes);')
print('Fine probes ready; production source untouched')
