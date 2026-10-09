"""Private causality probe: independently bypass only the two intermediate syncs."""
from pathlib import Path
import re
W=Path(__file__).resolve().parent;S=W/'probe-source'
p=S/'anki_forge/src/authoring_core/media_io.rs';s=p.read_text();old='    temp.as_file()\n        .sync_all()\n        .map_err';new='    (if std::env::var_os("ANKIFORGE_DIAG_SKIP_CAS_SYNC").is_some() { Ok(()) } else { temp.as_file().sync_all() })\n        .map_err';assert s.count(old)==1;s=s.replace(old,new);p.write_text(s)
p=S/'anki_forge/src/writer_core/media.rs';s=p.read_text();old='    let sync_result = output.sync_all();';assert s.count(old)==1;s=s.replace(old,'    let sync_result = if std::env::var_os("ANKIFORGE_DIAG_SKIP_STAGE_SYNC").is_some() { Ok(()) } else { output.sync_all() };');p.write_text(s)
for function,label in [('temporary_from_candidate','artifact.retain'),('publish_owned_candidate','artifact.owned_publish'),('persist_copy','artifact.persist_copy')]:
    p=S/'anki_forge/src/build_api/artifact.rs';s=p.read_text();matches=list(re.finditer(r'\bfn '+function+r'\s*(?:<[^\n]*>)?\s*\(',s));assert len(matches)==1;offset=s.index('{',matches[0].end())+1;p.write_text(s[:offset]+f'\n    let _diagnostic_scope = crate::diag::Scope::new("{label}");'+s[offset:])
p=S/'anki_forge/src/build_api/candidate.rs';s=p.read_text();start='    let (policy, context) = crate::runtime::defaults::load_embedded_writer_defaults()';assert s.count(start)==1;s=s.replace(start,'    let defaults_scope = crate::diag::Scope::new("runtime.writer_defaults");\n'+start);s=s.replace('    let mut target =','    drop(defaults_scope);\n    let mut target =',1);p.write_text(s)
p=W/'build_probe.py';s=p.read_text().replace('subprocess,time','subprocess,time,sys').replace("name='stages'","name=sys.argv[1] if len(sys.argv)>1 else 'stages'");p.write_text(s)
print('Two independent diagnostic controls and remaining boundary timers prepared')
