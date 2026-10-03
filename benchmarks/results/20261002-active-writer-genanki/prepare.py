from pathlib import Path
import json,shutil,sys
W=Path(__file__).resolve().parent;R=W.parents[2]
sys.path.insert(0,str(R/'benchmarks'));import bench
previous=json.loads((R/'benchmarks/results/20261002-reader-cache-genanki/prepared-builds-final.json').read_text())
if '--record-only' not in sys.argv:
 for output in [R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark',bench.INSPECTOR,bench.ORACLE]:
  command=previous[str(output)]['record']['command']
  bench.run_build(command,[output])
provenance=bench.build_provenance(bench.registry());assert all(v['status']=='verified' for v in provenance.values())
bench.save(W/'prepared-builds.json',provenance)
shutil.copy2(R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark',W/'binaries/after')
oracle=json.loads((R/'benchmarks/results/20261002-reader-cache-genanki/oracle-reuse-check.json').read_text())
assert bench.sha256(R/'scripts/roundtrip_oracle/src/bin/benchmark_oracle.rs')==oracle['checker_source_sha256']
upstream=R/'docs/source/anki'
assert bench.command(['git','rev-parse','HEAD'],cwd=upstream)==oracle['upstream_revision']
assert bench.command(['git','diff','--binary','HEAD'],cwd=upstream)==oracle['upstream_patch']
oracle['preceding_oracle_sha256']=oracle['oracle_sha256']
oracle['oracle_sha256']=bench.sha256(bench.ORACLE)
oracle['observation_note']='Fresh locked/offline build for round4. Binary differs from preceding run; checker source, upstream revision and patch match the frozen reference. Fresh build provenance verified; no claim of identical preceding binary.'
bench.save(W/'oracle-reuse-check.json',oracle)
shutil.copy2(R/'benchmarks/results/20261002-reader-cache-genanki/host-hardware.json',W/'host-hardware.json')
bench.save(W/'prepared-identity.json',bench.identity_snapshot(bench.registry()))
config=json.loads((W/'import-config.json').read_text());config['purpose']='Final uninstrumented comparison: frozen round3 versus active writer + private synced candidate publication + streaming collection digest; 4 MiB resident budget; other experimental switches excluded'
config['modes']={name:{'binary':name} for name in ['before','after']}
(W/'confirmation-config.json').write_text(json.dumps(config,indent=2)+'\n')
print('Final runtime, inspector and oracle prepared and verified',flush=True)
