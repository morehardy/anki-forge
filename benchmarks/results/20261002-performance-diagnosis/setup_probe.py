"""Build diagnostic-only source copies; never edit product sources."""
from pathlib import Path
import json,re,shutil,sys
W=Path(__file__).resolve().parent; R=W.parents[2]; S=W/'probe-source'
assert not S.exists()
S.mkdir()
for p in ('Cargo.lock','rust-toolchain.toml'): shutil.copy2(R/p,S/p)
(S/'Cargo.toml').write_text('[workspace]\nmembers = ["anki_forge"]\nresolver = "2"\n[workspace.package]\nedition = "2021"\nrust-version = "1.92"\n[workspace.lints.rust]\nunsafe_code = "forbid"\n')
for d in ('anki_forge','benchmarks/adapters/rust'):
    shutil.copytree(R/d,S/d,ignore=shutil.ignore_patterns('target','__pycache__','.DS_Store'))
(S/'anki_forge/src/diag.rs').write_text('''use std::{collections::BTreeMap, sync::{Mutex,OnceLock},time::Instant};
static STATS: OnceLock<Mutex<BTreeMap<&'static str,(u64,u128)>>> = OnceLock::new();
pub struct Scope(&'static str,Instant);
impl Scope { pub fn new(label: &'static str)->Self { Self(label,Instant::now()) } }
impl Drop for Scope { fn drop(&mut self) { let ns=self.1.elapsed().as_nanos();let mut stats=STATS.get_or_init(Default::default).lock().unwrap();let value=stats.entry(self.0).or_default();value.0+=1;value.1+=ns; } }
pub fn finish(){let stats=STATS.get_or_init(Default::default).lock().unwrap();eprintln!("[DEBUG-perf-20261002] {}",serde_json::to_string(&*stats).unwrap());}
''')
p=S/'anki_forge/src/lib.rs'; p.write_text(p.read_text()+'\n#[allow(missing_docs)]\npub mod diag;\n')
scopes=[('build_api/pipeline.rs','build','project.build'),('build_api/pipeline.rs','prepare_from_baseline','project.prepare'),('build_api/candidate.rs','generate','candidate.generate'),('build_api/normalize.rs','normalize','normalize.total'),('authoring_core/normalize.rs','normalize_with_prepared_media','normalize.core'),('authoring_core/media_io.rs','ingest_media_read_source_to_cas','media.cas_ingest'),('writer_core/build.rs','build_with_prepared_media','writer.total'),('writer_core/media.rs','prepare_verified_cas_copy','media.staging_copy'),('writer_core/staging.rs','materialize_with_prepared_media','writer.staging'),('writer_core/apkg.rs','emit_apkg_with_plans','writer.apkg'),('writer_core/inspect.rs','inspect_native_package','inspect.native'),('media/mod.rs','file_with_limits','media.import'),('build_api/identity/reconcile.rs','prepare','identity.prepare'),('build_api/identity.rs','bind','identity.bind')]
for file,function,label in scopes:
    p=S/'anki_forge/src'/file; s=p.read_text()
    matches=list(re.finditer(r'\bfn '+function+r'\s*(?:<[^\n]*>)?\s*\(',s));assert len(matches)==1,(file,function,len(matches))
    offset=s.index('{',matches[0].end())+1
    p.write_text(s[:offset]+f'\n    let _diagnostic_scope = crate::diag::Scope::new("{label}");'+s[offset:])
p=S/'anki_forge/src/authoring_core/media_io.rs';s=p.read_text();s=s.replace('    temp.as_file()\n        .sync_all()','    let sync_scope = crate::diag::Scope::new("media.cas_sync");\n    temp.as_file()\n        .sync_all()',1);s=s.replace('    let blake3 = blake3_hasher.finalize()', '    drop(sync_scope);\n    let blake3 = blake3_hasher.finalize()',1);p.write_text(s)
p=S/'anki_forge/src/writer_core/media.rs';s=p.read_text().replace('    if let Err(err) = output.sync_all() {', '    let sync_scope = crate::diag::Scope::new("media.staging_sync");\n    let sync_result = output.sync_all();\n    drop(sync_scope);\n    if let Err(err) = sync_result {',1);p.write_text(s)
p=S/'anki_forge/src/build_api/pipeline.rs';s=p.read_text().replace('        let mut media = Vec::new();','        let stage_scope = crate::diag::Scope::new("project.asset_staging");\n        let mut media = Vec::new();',1).replace('        let note_types = self','        drop(stage_scope);\n        let bridge_scope = crate::diag::Scope::new("project.document_bridge");\n        let note_types = self',1).replace('        let (candidate, mut report) =','        drop(bridge_scope);\n        let (candidate, mut report) =',1);p.write_text(s)
p=S/'benchmarks/adapters/rust/src/main.rs';s=p.read_text().replace('    project.build(BuildOptions::to(output))?;','    project.build(BuildOptions::to(output))?;\n    ankiforge::diag::finish();');p.write_text(s)
(W/'binaries').mkdir();binary=R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark';shutil.copy2(binary,W/'binaries/original')
print('Private probe prepared')
