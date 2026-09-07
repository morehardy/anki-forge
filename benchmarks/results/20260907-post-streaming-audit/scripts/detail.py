"""Fine wall-time probes in a private source copy; no shared hot-path mutex."""
from pathlib import Path
import re
import shutil

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


shutil.copytree(SRC, WORK / 'coarse-source')
diag = SRC / 'anki_forge/src/diag.rs'
diag.write_text(diag.read_text() + '''
use std::{cell::RefCell, collections::BTreeMap};
thread_local! {
    static TOTALS: RefCell<BTreeMap<&'static str, (u128, usize)>> = const { RefCell::new(BTreeMap::new()) };
}
pub struct Accum(&'static str, Instant);
impl Accum { pub fn new(label: &'static str) -> Self { Self(label, Instant::now()) } }
impl Drop for Accum { fn drop(&mut self) {
    let elapsed = self.1.elapsed().as_nanos();
    TOTALS.with(|totals| { let mut totals = totals.borrow_mut(); let entry = totals.entry(self.0).or_default(); entry.0 += elapsed; entry.1 += 1; });
} }
pub fn report() { TOTALS.with(|totals| {
    for (key, (ns, count)) in std::mem::take(&mut *totals.borrow_mut()) {
        eprintln!("[DEBUG-scale-opt] {} {}", key, ns);
        eprintln!("[DEBUG-post-audit-count] {} {}", key, count);
    }
}); }
pub struct ThreadReport;
impl Drop for ThreadReport { fn drop(&mut self) { report(); } }
''')
edit('benchmarks/adapters/rust/src/main.rs', '    deck.write_apkg(output)?.ensure_success()?;',
     '    deck.write_apkg(output)?.ensure_success()?;\n    anki_forge::diag::report();')

for file, name, label in [
    ('deck/identity.rs', 'normalize_field_text_for_identity', 'note.identity_normalize'),
    ('deck/identity.rs', 'hash_payload', 'note.identity_serialize_hash'),
    ('writer_core/apkg.rs', 'note_storage_values', 'sqlite.field_storage'),
    ('writer_core/note_data.rs', 'fresh_identity_note_data', 'sqlite.identity_data'),
    ('writer_core/note_revision.rs', 'from_note', 'note.revision'),
    ('deck/media.rs', 'validate_source_file', 'registration.stat'),
    ('deck/media.rs', 'raster_image_metadata_from_bytes', 'registration.dimensions'),
    ('prepared_media.rs', 'prepare_one', 'media.worker_compute'),
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

f = 'anki_forge/src/writer_core/stream_zip.rs'
edit(f, '        let written = self.output.write(bytes)?;',
     '        let written = { let _t = crate::diag::Accum::new("zip.write"); self.output.write(bytes)? };')
edit(f, '        self.hash.update(&bytes[..written])?;',
     '        { let _t = crate::diag::Accum::new("zip.hash_enqueue_or_serial"); self.hash.update(&bytes[..written])?; }')

f = 'anki_forge/src/prepared_media.rs'
edit(f, '                            .spawn_scoped(scope, move || {',
     '                            .spawn_scoped(scope, move || {\n                                let _report = crate::diag::ThreadReport;')
edit(f, '                        let result = receiver.recv().unwrap_or_else(|_| Err(stopped(item)));',
     '                        let result = { let _t = crate::diag::Accum::new("media.consumer_wait"); receiver.recv().unwrap_or_else(|_| Err(stopped(item))) };')
edit(f, '                ingested[index] = Some(result.and_then(|(metadata, mut payload)| {',
     '                let _accept = crate::diag::Accum::new("media.accept");\n                ingested[index] = Some(result.and_then(|(metadata, mut payload)| {')
edit(f, '        let written = self.data.write(bytes)?;',
     '        let written = { let _t = crate::diag::Accum::new("media.encoded_payload_write"); self.data.write(bytes)? };')
edit(f, '            let count = reader\n                .read(&mut buffer)',
     '            let count = { let _t = crate::diag::Accum::new("media.worker_read"); reader.read(&mut buffer) }')
edit(f, '            sha1.update(bytes);\n            blake3.update(bytes);',
     '            { let _t = crate::diag::Accum::new("media.worker_hash"); sha1.update(bytes); blake3.update(bytes); }')
edit(f, '                encoder.write_all(bytes).map_err(encode_error)?;',
     '                { let _t = crate::diag::Accum::new("media.worker_encode_write"); encoder.write_all(bytes).map_err(encode_error)?; }')

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
edit(f, '    let mut hash = sha1::Sha1::new();\n    while let Ok(chunk) = jobs.recv() {\n        hash.update(&chunk.bytes);',
     '    let _report = crate::diag::ThreadReport;\n    let mut hash = sha1::Sha1::new();\n    while let Ok(chunk) = jobs.recv() {\n        { let _t = crate::diag::Accum::new("inspect.worker_hash"); hash.update(&chunk.bytes); }')
edit(f, '    fn send_block(&mut self, last: bool) -> std::io::Result<()> {',
     '    fn send_block(&mut self, last: bool) -> std::io::Result<()> {\n        let _t = crate::diag::Accum::new("inspect.hash_send_and_alloc");')
edit(f, '            let size = archive\n                .copy(',
     '            let copy_timer = crate::diag::Accum::new("inspect.media_copy");\n            let size = archive\n                .copy(')
edit(f, '            let sha1_hex = hash.finish()?.unwrap_or_default();',
     '            drop(copy_timer);\n            let sha1_hex = hash.finish()?.unwrap_or_default();')
edit(f, '        let collection = read_collection_data(&collection_path, projection)?;',
     '        let collection = read_collection_data(&collection_path, projection)?;')

f = 'anki_forge/src/writer_core/pipelined_sha1.rs'
edit(f, '            .spawn(move || {', '            .spawn(move || {\n                let _report = crate::diag::ThreadReport;')
edit(f, '                    hash.update(&bytes);',
     '                    { let _t = crate::diag::Accum::new("zip.worker_hash"); hash.update(&bytes); }')
edit(f, '    fn send_pending(&mut self) -> io::Result<()> {',
     '    fn send_pending(&mut self) -> io::Result<()> {\n        let _t = crate::diag::Accum::new("zip.hash_send");')
edit(f, '    fn finish(mut self) -> io::Result<Sha1> {',
     '    fn finish(mut self) -> io::Result<Sha1> {\n        let _t = crate::diag::Accum::new("zip.hash_finish");')
print('Fine probes ready; counters are local to each thread and flushed once.')
