from pathlib import Path
import json,shutil,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'));import bench
for command,output in [
 (['cc','-O2','-Wall','-Wextra','-Werror','-pthread',str(R/'benchmarks/native/measure.c'),'-o',str(bench.COLLECTOR)],bench.COLLECTOR),
 (['cargo','build','--release','--locked','--offline','--no-default-features','--manifest-path',str(R/'benchmarks/adapters/rust/Cargo.toml')],R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'),
 (['cargo','build','--release','--locked','--offline','-p','contract_tools'],bench.INSPECTOR)]:
 bench.run_build(command,[output])
provenance=bench.build_provenance(bench.registry());assert all(v['status']=='verified' for v in provenance.values())
bench.save(W/'prepared-builds.json',provenance)
shutil.copy2(R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark',W/'binaries/after')
oracle=json.loads((R/'benchmarks/results/20261004-publication-genanki/oracle-reuse-check.json').read_text())
assert bench.sha256(R/'scripts/roundtrip_oracle/src/bin/benchmark_oracle.rs')==oracle['checker_source_sha256']
upstream=R/'docs/source/anki'
assert bench.command(['git','rev-parse','HEAD'],cwd=upstream)==oracle['upstream_revision']
assert bench.command(['git','diff','--binary','HEAD'],cwd=upstream)==oracle['upstream_patch']
assert bench.sha256(bench.ORACLE)==oracle['oracle_sha256']
reference=json.loads((R/'benchmarks/results/20261004-publication-genanki/source-snapshot.json').read_text())
checker_files={name:digest for name,digest in reference['source_files'].items() if name.startswith('scripts/roundtrip_oracle/')}
for name,digest in checker_files.items(): assert bench.sha256(R/name)==digest,name
oracle['checker_files_verified']=checker_files
oracle['rebuilt_from_current_source']=False
oracle['reused_verified_executable']=True
oracle['observation_note']='Reused the exact Oct 04 Anki oracle executable after checking SHA, every recorded checker source/lock file and upstream revision/patch. Initial offline rebuild could not write a missing git dependency checkout in the sandbox; failure retained in prepare-initial-failure.log. Other tools freshly rebuilt; existing oracle build provenance verified.'
bench.save(W/'oracle-reuse-check.json',oracle)
shutil.copy2(R/'benchmarks/results/20261002-active-writer-genanki/host-hardware.json',W/'host-hardware.json')
bench.save(W/'prepared-identity.json',bench.identity_snapshot(bench.registry()))
print('Production exporter/inspector rebuilt; exact archived Anki oracle independently verified and reused',flush=True)
