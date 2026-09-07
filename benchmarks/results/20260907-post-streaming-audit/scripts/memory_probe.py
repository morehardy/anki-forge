"""Separate memory-only probe; allocator atomics and ps make it unsuitable for timing claims."""
from pathlib import Path
import shutil

WORK = Path(__file__).resolve().parent
SRC = WORK / 'profile-source'
shutil.move(SRC, WORK / 'fine-source')
shutil.copytree(WORK / 'coarse-source', SRC)
diag = SRC / 'anki_forge/src/diag.rs'
diag.write_text('''//! Private memory-boundary probes, excluded from production.
use std::{sync::OnceLock, time::Instant};
static OBSERVER: OnceLock<fn(&str, bool)> = OnceLock::new();
pub fn install_observer(observer: fn(&str, bool)) { let _ = OBSERVER.set(observer); }
pub struct Scope(&'static str, Instant);
impl Scope { pub fn new(label: &'static str) -> Self {
    if let Some(observer) = OBSERVER.get() { observer(label, true); }
    Self(label, Instant::now())
} }
impl Drop for Scope { fn drop(&mut self) {
    if let Some(observer) = OBSERVER.get() { observer(self.0, false); }
    eprintln!("[DEBUG-scale-opt] {} {}", self.0, self.1.elapsed().as_nanos());
} }
''')
adapter = SRC / 'benchmarks/adapters/rust/src'
(adapter / 'memory.rs').write_text('''//! Diagnostic-only allocation accounting; delegates every allocation to System.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);
struct Counted;
fn add(size: usize) {
    let live = LIVE.fetch_add(size, Relaxed) + size;
    PEAK.fetch_max(live, Relaxed);
    TOTAL.fetch_add(size, Relaxed);
    COUNT.fetch_add(1, Relaxed);
}
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() { add(layout.size()); }
        ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() { add(layout.size()); }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        LIVE.fetch_sub(layout.size(), Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let new = System.realloc(ptr, layout, size);
        if !new.is_null() { LIVE.fetch_sub(layout.size(), Relaxed); add(size); }
        new
    }
}
#[global_allocator]
static ALLOCATOR: Counted = Counted;
pub fn observe(label: &str, begin: bool) {
    let live = LIVE.load(Relaxed);
    let peak = PEAK.load(Relaxed);
    let total = TOTAL.load(Relaxed);
    let count = COUNT.load(Relaxed);
    let result = std::process::Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().expect("ps RSS");
    assert!(result.status.success());
    let rss_kib: usize = std::str::from_utf8(&result.stdout).expect("RSS UTF-8").trim().parse().expect("RSS number");
    eprintln!("[DEBUG-post-audit-memory] {}", serde_json::json!({
        "label": label, "begin": begin, "live_rust_bytes": live,
        "peak_rust_bytes": peak, "cumulative_rust_bytes": total,
        "allocation_calls": count, "current_rss_bytes": rss_kib * 1024,
    }));
}
''')
p = adapter / 'main.rs'
s = p.read_text().replace('use anki_forge::prelude::*;', 'mod memory;\nuse anki_forge::prelude::*;')
s = s.replace('    let _total = anki_forge::diag::Scope::new("adapter.total");',
              '    anki_forge::diag::install_observer(memory::observe);\n    let _total = anki_forge::diag::Scope::new("adapter.total");')
p.write_text(s)
p = SRC / 'anki_forge/src/writer_core/pipelined_sha1.rs'
s = p.read_text()
assert s.count('attempted_worker: false,') == 1
p.write_text(s.replace('attempted_worker: false,', 'attempted_worker: std::env::var_os("ANKI_FORGE_AUDIT_SERIAL_SHA1").is_some(),'))
print('Memory-only probe prepared. Rust accounting excludes native SQLite/zstd allocations; ps records total resident memory.')
