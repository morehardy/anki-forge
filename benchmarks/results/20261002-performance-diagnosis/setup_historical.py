from pathlib import Path
import re,shutil
W=Path(__file__).resolve().parent;S=W/'historical-source'
(S/'Cargo.toml').write_text('[workspace]\nmembers = ["anki_forge"]\nresolver = "2"\n[workspace.package]\nedition = "2021"\nrust-version = "1.92"\n[workspace.lints.rust]\nunsafe_code = "forbid"\n')
shutil.copy2(W/'probe-source/anki_forge/src/diag.rs',S/'anki_forge/src/diag.rs')
p=S/'anki_forge/src/lib.rs';p.write_text(p.read_text()+'\n#[allow(missing_docs)]\npub mod diag;\n')
scopes=[('product/project/pipeline.rs','prepare','old.prepare'),('product/project/input.rs','normalize_with_prepared_media','normalize.total'),('authoring_core/normalize.rs','normalize_with_prepared_media','normalize.core'),('writer_core/build.rs','build_with_prepared_media','writer.total'),('writer_core/staging.rs','materialize_with_prepared_media','writer.staging'),('writer_core/apkg.rs','emit_apkg_with_plans','writer.apkg'),('writer_core/inspect.rs','inspect_apkg_summary_with_limits','inspect.summary')]
for file,function,label in scopes:
    p=S/'anki_forge/src'/file;s=p.read_text();matches=list(re.finditer(r'\bfn '+function+r'\s*(?:<[^\n]*>)?\s*\(',s));assert len(matches)==1,(file,function,len(matches));offset=s.index('{',matches[0].end())+1;p.write_text(s[:offset]+f'\n    let _diagnostic_scope = crate::diag::Scope::new("{label}");'+s[offset:])
p=S/'benchmarks/adapters/rust/src/main.rs';s=p.read_text().replace('    deck.write_apkg(output)?.ensure_success()?;','    deck.write_apkg(output)?.ensure_success()?;\n    anki_forge::diag::finish();');p.write_text(s)
p=W/'build_probe.py';s=p.read_text().replace("str(W/'probe-source/benchmarks/adapters/rust/Cargo.toml')","str(W/('historical-source' if name=='historical' else 'probe-source')/'benchmarks/adapters/rust/Cargo.toml')");s=s.replace("(W/'probe-source').rglob('*')","(W/('historical-source' if name=='historical' else 'probe-source')).rglob('*')").replace("p.relative_to(W/'probe-source')","p.relative_to(W/('historical-source' if name=='historical' else 'probe-source'))");p.write_text(s)
print('Historical frozen source instrumented')
