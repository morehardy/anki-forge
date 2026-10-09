from pathlib import Path
import json,shutil,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'));import bench
for command,output in [
 (['cc','-O2','-Wall','-Wextra','-Werror','-pthread',str(R/'benchmarks/native/measure.c'),'-o',str(bench.COLLECTOR)],bench.COLLECTOR),
 (['cargo','build','--release','--locked','--offline','--no-default-features','--manifest-path',str(R/'benchmarks/adapters/rust/Cargo.toml')],R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'),
 (['cargo','build','--release','--locked','--offline','-p','contract_tools'],bench.INSPECTOR),
 (['cargo','build','--locked','--offline','--manifest-path',str(R/'scripts/roundtrip_oracle/Cargo.toml'),'--bin','benchmark_oracle'],bench.ORACLE)]:
 bench.run_build(command,[output])
provenance=bench.build_provenance(bench.registry());assert all(v['status']=='verified' for v in provenance.values())
bench.save(W/'prepared-builds.json',provenance)
shutil.copy2(R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark',W/'binaries/after')
oracle=json.loads((R/'benchmarks/results/20261002-active-writer-genanki/oracle-reuse-check.json').read_text())
assert bench.sha256(R/'scripts/roundtrip_oracle/src/bin/benchmark_oracle.rs')==oracle['checker_source_sha256']
upstream=R/'docs/source/anki'
assert bench.command(['git','rev-parse','HEAD'],cwd=upstream)==oracle['upstream_revision']
assert bench.command(['git','diff','--binary','HEAD'],cwd=upstream)==oracle['upstream_patch']
oracle['preceding_oracle_sha256']=oracle['oracle_sha256'];oracle['oracle_sha256']=bench.sha256(bench.ORACLE)
oracle['observation_note']='Fresh locked/offline build for current publication implementation benchmark. Checker source and upstream revision/patch match the historical reference; build provenance verified. No claim of identical historical executable.'
bench.save(W/'oracle-reuse-check.json',oracle)
shutil.copy2(R/'benchmarks/results/20261002-active-writer-genanki/host-hardware.json',W/'host-hardware.json')
bench.save(W/'prepared-identity.json',bench.identity_snapshot(bench.registry()))
print('Current production exporter, inspector and pinned Anki oracle rebuilt and verified',flush=True)
