"""Instrument only an isolated source copy of the optimized production snapshot."""
from pathlib import Path
import json,re,shutil,subprocess,sys
W=Path(__file__).resolve().parent;R=W.parents[2];S=W/'probe-source';assert not S.exists();S.mkdir()
for p in ('Cargo.lock','rust-toolchain.toml'):shutil.copy2(R/p,S/p)
(S/'Cargo.toml').write_text('[workspace]\nmembers=["anki_forge"]\nresolver="2"\n[workspace.package]\nedition="2021"\nrust-version="1.92"\n[workspace.lints.rust]\nunsafe_code="forbid"\n')
for d in ('anki_forge','benchmarks/adapters/rust'):
 shutil.copytree(R/d,S/d,ignore=shutil.ignore_patterns('target','__pycache__','.DS_Store'))
(S/'anki_forge/src/diag.rs').write_text('''use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::Instant};
static STATS: OnceLock<Mutex<BTreeMap<&'static str,(u64,u128)>>> = OnceLock::new();
pub struct Scope(&'static str,Instant);
impl Scope {pub fn new(label:&'static str)->Self{Self(label,Instant::now())}}
impl Drop for Scope {fn drop(&mut self){let ns=self.1.elapsed().as_nanos();let mut s=STATS.get_or_init(Default::default).lock().unwrap();let v=s.entry(self.0).or_default();v.0+=1;v.1+=ns;}}
pub fn count(label:&'static str,n:u64){let mut s=STATS.get_or_init(Default::default).lock().unwrap();s.entry(label).or_default().0+=n;}
pub fn snapshot_budget()->usize{static B:OnceLock<usize>=OnceLock::new();*B.get_or_init(||std::env::var("ANKIFORGE_DIAG_SNAPSHOT_BUDGET").ok().and_then(|s|s.parse().ok()).unwrap_or(4*1024*1024))}
pub fn finish(){let s=STATS.get_or_init(Default::default).lock().unwrap();eprintln!("[DEBUG-residual-20261002] {}",serde_json::to_string(&*s).unwrap());}
''')
p=S/'anki_forge/src/lib.rs';p.write_text(p.read_text()+'\n#[allow(missing_docs)]\npub mod diag;\n')
def scope(file,function,label):
 p=S/'anki_forge/src'/file;s=p.read_text();matches=list(re.finditer(r'\bfn '+function+r'\s*(?:<[^\n]*>)?\s*\(',s));assert len(matches)==1,(file,function,len(matches));a=s.index('{',matches[0].end())+1;p.write_text(s[:a]+f'\n let _diagnostic_scope=crate::diag::Scope::new("{label}");'+s[a:])
for args in [
 ('runtime/defaults.rs','load_embedded_writer_defaults','runtime.defaults'),('build_api/identity/validation.rs','validate_media_history','identity.media_history'),('writer_core/staging.rs','validate_normalized_ir','writer.preflight'),('writer_core/staging.rs','validate_media_invariants','writer.media_invariants'),('media/mod.rs','file_with_limits','media.import'),('media/snapshot.rs','file','snapshot.file'),('media/snapshot.rs','shared','snapshot.shared'),('media/snapshot.rs','spill_memory','snapshot.spill'),('media/snapshot/spool.rs','store','spool.store'),('media/snapshot.rs','reader','snapshot.reader'),
 ('project.rs','add_asset','project.add_asset'),('project.rs','add','project.add_note'),
 ('build_api/pipeline.rs','build','project.build'),('build_api/pipeline.rs','prepare_from_baseline','project.prepare'),
 ('build_api/candidate.rs','generate','candidate.generate'),('build_api/normalize.rs','normalize','normalize.total'),('authoring_core/normalize.rs','normalize_with_prepared_media','normalize.core'),
 ('prepared_media.rs','prepare_all','media.prepare_all'),('prepared_media.rs','prepare_one','media.prepare_one'),
 ('writer_core/staging.rs','materialize_with_prepared_media','writer.staging'),('writer_core/build.rs','build_with_prepared_media','writer.total'),('writer_core/apkg.rs','emit_apkg_with_plans','writer.apkg'),('writer_core/apkg.rs','create_latest_collection_file','writer.collection'),('writer_core/apkg.rs','populate_latest_collection_rows','writer.populate_collection'),
 ('build_api/artifact.rs','temporary_from_candidate','artifact.retain'),('build_api/artifact.rs','persist_file','artifact.publish'),
 ('writer_core/inspect.rs','inspect_native_package','inspect.native'),('writer_core/inspect.rs','read_apkg_facts','inspect.facts'),('writer_core/inspect.rs','read_media_entries','inspect.media'),('writer_core/inspect.rs','read_collection_data','inspect.collection'),('build_api/identity/validation.rs','validate','identity.validate'),('build_api/identity/content.rs','read','identity.note_content'),
 ('build_api/identity/reconcile.rs','prepare','identity.prepare'),('build_api/identity.rs','bind','identity.bind'),('build_api/identity.rs','envelope','identity.envelope')]:scope(*args)
p=S/'anki_forge/src/media/snapshot.rs';s=p.read_text().replace('const MEMORY_BUDGET: usize = 4 * MEMORY_THRESHOLD;\n','').replace('> MEMORY_BUDGET','> crate::diag::snapshot_budget()').replace('                self.storage = Storage::File(file.into_temp_path());','                crate::diag::count("snapshot.spilled_files",1);\n                crate::diag::count("snapshot.spilled_bytes",bytes.len() as u64);\n                self.storage = Storage::File(file.into_temp_path());');p.write_text(s)
p=S/'anki_forge/src/prepared_media.rs';s=p.read_text().replace('        let workers = if jobs.len() < 16 {','        let workers = if std::env::var_os("ANKIFORGE_DIAG_SERIAL_MEDIA").is_some() || jobs.len() < 16 {');s=s.replace('        let mut archive = self','        crate::diag::count("media.preparation_workers",workers as u64);\n        let mut archive = self',1)
a=s.index('                ingested[index] = Some(result.and_then(|(metadata, payload)| {')+len('                ingested[index] = Some(result.and_then(|(metadata, payload)| {');s=s[:a]+'\n let _append_scope=crate::diag::Scope::new("media.archive_append");'+s[a:];p.write_text(s)
p=S/'anki_forge/src/build_api/artifact.rs';s=p.read_text();old='''    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(candidate)?
        .sync_all()?;''';assert old in s;s=s.replace(old,'''    if std::env::var_os("ANKIFORGE_DIAG_SKIP_RETAIN_SYNC").is_none() {
        let _sync_scope=crate::diag::Scope::new("artifact.retain_sync");
'''+old+'''\n    }''',1)
s=s.replace('''            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(source)?
                .sync_all()?;''','''            {
                let _sync_scope=crate::diag::Scope::new("artifact.final_file_sync");
                std::fs::OpenOptions::new().read(true).write(true).open(source)?.sync_all()?;
            }''',1);s=s.replace('                std::fs::File::open(directory)?.sync_all()?;','                let _sync_scope=crate::diag::Scope::new("artifact.final_directory_sync");\n                std::fs::File::open(directory)?.sync_all()?;',1);p.write_text(s)
p=S/'anki_forge/src/build_api/pipeline.rs';s=p.read_text().replace('        let mut media = Vec::new();','        let _bridge_scope=crate::diag::Scope::new("project.document_bridge");\n        let mut media = Vec::new();',1).replace('        let (candidate, mut report) =','        drop(_bridge_scope);\n        let (candidate, mut report) =',1);p.write_text(s)
p=S/'benchmarks/adapters/rust/src/main.rs';s=p.read_text().replace('    let workload: Workload =','    let _parse_scope=ankiforge::diag::Scope::new("adapter.parse");\n    let workload: Workload =',1).replace('    let mut project = Project::new','    drop(_parse_scope);\n    let mut project = Project::new',1).replace('    for media in workload.media {','    let _assets_scope=ankiforge::diag::Scope::new("adapter.assets");\n    for media in workload.media {',1).replace('    for (index, note) in workload.notes.into_iter().enumerate() {','    drop(_assets_scope);\n    let _notes_scope=ankiforge::diag::Scope::new("adapter.notes");\n    for (index, note) in workload.notes.into_iter().enumerate() {',1).replace('    project.build(BuildOptions::to(output))?;','''    drop(_notes_scope);
    project.build(BuildOptions::to(output))?;
    let _cleanup_scope=ankiforge::diag::Scope::new("adapter.cleanup");
    drop(project);
    drop(media_by_id);
    drop(_cleanup_scope);
    ankiforge::diag::finish();''',1);p.write_text(s)
# Preserve the exact diagnostic changes independently from the original dirty production patch.
subprocess.run(['diff','-ru',str(R/'anki_forge/src'),str(S/'anki_forge/src')],stdout=(W/'probe.patch').open('w'),check=False)
print('Isolated optimized probe prepared with stage timers and three single-variable controls')
