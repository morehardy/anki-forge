from pathlib import Path
import json,os,shutil,subprocess,sys,time
W=Path(__file__).resolve().parent;R=W.parents[2];binary=R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
command=['cargo','build','--release','--offline','--locked','--no-default-features','--manifest-path',str(W/'probe-source/benchmarks/adapters/rust/Cargo.toml')]
started=time.monotonic();sys.path.insert(0,str(R/'benchmarks'));import bench
original_hash=bench.sha256(binary);shutil.copy2(binary,W/'binaries/final-reader-cache')
try:
 with (W/'build-probe.log').open('w') as s:subprocess.run(command,env=dict(os.environ,CARGO_TARGET_DIR=str(R/'benchmarks/adapters/rust/target')),check=True,stdout=s,stderr=subprocess.STDOUT)
 shutil.copy2(binary,W/'binaries/probe')
 bench.save(W/'build-probe.json',dict(command=command,elapsed_s=time.monotonic()-started,binary_sha256=bench.sha256(W/'binaries/probe'),source_files={str(p.relative_to(W/'probe-source')):bench.sha256(p) for p in (W/'probe-source').rglob('*') if p.is_file()}))
finally:
 shutil.copy2(W/'binaries/final-reader-cache',binary);assert bench.sha256(binary)==original_hash
 print('Official optimized executable restored',flush=True)
