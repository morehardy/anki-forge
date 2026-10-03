from pathlib import Path
import shutil

W = Path(__file__).resolve().parent
R = W.parents[2]
S = W / 'probe-source'

def edit(name, before, after):
    path = S / name
    text = path.read_text()
    assert text.count(before) == 1, (name, before[:100], text.count(before))
    path.write_text(text.replace(before, after))

diag = '''use std::sync::{atomic::{AtomicU64, Ordering}, LazyLock};
use std::time::Instant;
static VALUES: LazyLock<Vec<(AtomicU64, AtomicU64)>> = LazyLock::new(|| (0..10).map(|_| (AtomicU64::new(0), AtomicU64::new(0))).collect());
pub struct Span(usize, Instant);
pub fn span(index: usize) -> Span { Span(index, Instant::now()) }
impl Drop for Span { fn drop(&mut self) { VALUES[self.0].0.fetch_add(1, Ordering::Relaxed); VALUES[self.0].1.fetch_add(self.1.elapsed().as_nanos() as u64, Ordering::Relaxed); } }
pub fn report() { let names = ["snapshot_file", "preflight", "open_source", "source_read", "share", "spool_store", "descriptor_stat", "get_flags", "set_flags", "source_open"];
let values: std::collections::BTreeMap<_, _> = names.into_iter().enumerate().map(|(i,n)| (n, [VALUES[i].0.load(Ordering::Relaxed), VALUES[i].1.load(Ordering::Relaxed)])).collect();
eprintln!("[DEBUG-import-round4] {}", serde_json::to_string(&values).unwrap()); }
pub fn writer() -> bool { static VALUE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("ANKIFORGE_ROUND4_WRITER").is_some()); *VALUE }
pub fn nonblock() -> bool { static VALUE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("ANKIFORGE_ROUND4_NONBLOCK").is_some()); *VALUE }
'''
(S/'anki_forge/src/round4_diag.rs').write_text(diag)
with (S/'anki_forge/src/lib.rs').open('a') as stream: stream.write('\n#[doc(hidden)] pub mod round4_diag;\n')
edit('benchmarks/adapters/rust/src/main.rs','    project.build(BuildOptions::to(output))?;','    project.build(BuildOptions::to(output))?;\n    ankiforge::round4_diag::report();')

name='anki_forge/src/media/snapshot/spool.rs'
edit(name, 'sync::{Arc, Weak}', 'sync::{Arc, Mutex, Weak}')
edit(name, 'struct Block {', 'struct Block {\n    writer: Mutex<Option<File>>,')
edit(name, 'path: Some(path),', 'writer: Mutex::new(None),\n                        path: Some(path),')
edit(name, '    pub(super) fn store(&mut self, bytes: &[u8]) -> io::Result<Segment> {',
'''    pub(super) fn store(&mut self, bytes: &[u8]) -> io::Result<Segment> {
        let _timer = crate::round4_diag::span(5);
        if crate::round4_diag::writer() { return self.store_active(bytes); }''')
old=(R/'benchmarks/.work/spool-20261002/spool-active-writer.rs').read_text()
active=old[old.index('    pub(super) fn store('):old.index('\nimpl Segment {')].removesuffix('}\n')
active=active.replace('pub(super) fn store(', 'fn store_active(')
edit(name, '\nimpl Segment {\n    pub(super) fn reader', '\nimpl Pool {\n'+active+'}\n\nimpl Segment {\n    pub(super) fn reader')

name='anki_forge/src/media/snapshot.rs'
edit(name, '    fn shared(mut self) -> Result<Arc<Self>, MediaError> {', '    fn shared(mut self) -> Result<Arc<Self>, MediaError> {\n        let _timer = crate::round4_diag::span(4);')
edit(name, '        // Reject invalid inputs early.', '        let _timer = crate::round4_diag::span(0);\n        // Reject invalid inputs early.')
edit(name, 'std::fs::metadata(path).map_err(|e| MediaError::io("inspect media source", e))?;', '{ let _timer = crate::round4_diag::span(1); std::fs::metadata(path).map_err(|e| MediaError::io("inspect media source", e))? };')
edit(name, 'let mut source = open_source(path, limits)?;', 'let mut source = { let _timer = crate::round4_diag::span(2); open_source(path, limits)? };')
edit(name, 'match source.read(&mut buffer)', 'match { let _timer = crate::round4_diag::span(3); source.read(&mut buffer) }')
edit(name, 'crate::regular_file::open(path, limits.max_bytes)', 'crate::regular_file::open_snapshot(path, limits.max_bytes)')

name='anki_forge/src/regular_file.rs'
edit(name, 'pub(crate) fn open(path: &Path, max_bytes: u64) -> Result<File, OpenError> {', '''pub(crate) fn open(path: &Path, max_bytes: u64) -> Result<File, OpenError> { open_impl(path, max_bytes, false) }
pub(crate) fn open_snapshot(path: &Path, max_bytes: u64) -> Result<File, OpenError> { open_impl(path, max_bytes, crate::round4_diag::nonblock()) }
fn open_impl(path: &Path, max_bytes: u64, keep_nonblock: bool) -> Result<File, OpenError> {''')
edit(name, '    #[cfg(unix)]\n    let source', '    let open_timer = crate::round4_diag::span(9);\n    #[cfg(unix)]\n    let source')
edit(name, 'let metadata = source.metadata()?;', 'drop(open_timer);\n    let metadata = { let _timer = crate::round4_diag::span(6); source.metadata()? };')
edit(name, '    {\n        // Restore blocking reads', '    if !keep_nonblock {\n        // Restore blocking reads')
edit(name, 'let flags = rustix::fs::fcntl_getfl(&source).map_err(io::Error::from)?;', 'let flags = { let _timer = crate::round4_diag::span(7); rustix::fs::fcntl_getfl(&source).map_err(io::Error::from)? };')
edit(name, '        rustix::fs::fcntl_setfl(&source, flags & !rustix::fs::OFlags::NONBLOCK)', '        let _timer = crate::round4_diag::span(8);\n        rustix::fs::fcntl_setfl(&source, flags & !rustix::fs::OFlags::NONBLOCK)')
shutil.copytree(S,W/'import-probe-source')
print('Probe source prepared; production unaffected')
