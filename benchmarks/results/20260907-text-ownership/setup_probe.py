"""Private allocation/stage probes. Never change production or exporter settings."""
from pathlib import Path
import hashlib,json,re,shutil,sys
W=Path(__file__).resolve().parent; R=W.parents[2]
variant,kind=sys.argv[1:]; name=f'{variant}-{kind}'
source=W/'baseline-source' if variant=='current' else R
S=W/'profile-source'
if S.exists(): shutil.rmtree(S)
S.mkdir()
for p in ('Cargo.lock','rust-toolchain.toml'): shutil.copy2(R/p,S/p)
(S/'Cargo.toml').write_text('[workspace]\nmembers = ["anki_forge"]\nresolver = "2"\n[workspace.package]\nedition = "2021"\nrust-version = "1.92"\n[workspace.lints.rust]\nunsafe_code = "forbid"\n')
for d in ('anki_forge','benchmarks/adapters/rust'):
    shutil.copytree(source/d,S/d,ignore=shutil.ignore_patterns('target'))
if kind=='memory':
    old=R/'benchmarks/.work/bounded-media-20260907'
    shutil.copy2(old/'memory-main.rs',S/'benchmarks/adapters/rust/src/main.rs')
    shutil.copy2(old/'profile-source/benchmarks/adapters/rust/src/memory.rs',S/'benchmarks/adapters/rust/src/memory.rs')
else:
    (S/'anki_forge/src/diag.rs').write_text('''//! Private diagnostic stage timers.
use std::time::Instant;
pub struct Scope(&'static str, Instant);
impl Scope { pub fn new(label: &'static str) -> Self { Self(label, Instant::now()) } }
impl Drop for Scope { fn drop(&mut self) {
    eprintln!("[DEBUG-scale-opt] {} {}", self.0, self.1.elapsed().as_nanos());
} }
''')
    p=S/'anki_forge/src/lib.rs'; p.write_text(p.read_text()+'\n#[allow(missing_docs)]\npub mod diag;\n')
    scopes=[('product/project/pipeline.rs','prepare','pipeline.prepare'),('product/project/pipeline/reconcile.rs','reconcile','pipeline.reconcile'),('authoring_core/normalize.rs','normalize_with_prepared_media','normalize.core')]
    if variant=='current':
        scopes += [('product/project/input.rs','normalize_with_prepared_media','normalize.total'),('product/project/input.rs','lower_with_project_error','normalize.lowering'),('product/project/input.rs','resolved_note_identities','identity.after_normalize')]
    else:
        scopes += [('product/project/input.rs','normalize_for_build','normalize.with_identity'),('product/project.rs','into_build_lowering','normalize.lowering')]
    for file,function,label in scopes:
        p=S/'anki_forge/src'/file; text=p.read_text()
        matches=list(re.finditer(r'\bfn '+function+r'\s*(?:<[^\n]*>)?\s*\(',text)); assert len(matches)==1,(file,function)
        offset=text.index('{',matches[0].end())+1
        p.write_text(text[:offset]+f'\n    let _text_scope = crate::diag::Scope::new("{label}");'+text[offset:])
shutil.copytree(S,W/'probe-sources'/name)
print(name,'source captured')
