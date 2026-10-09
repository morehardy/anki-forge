from pathlib import Path
import argparse,json,os,shutil,subprocess,sys,time
W=Path(__file__).resolve().parent;R=W.parents[2]
parser=argparse.ArgumentParser();parser.add_argument('name');parser.add_argument('source');args=parser.parse_args()
source=W/args.source;binary=R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
sys.path.insert(0,str(R/'benchmarks'));import bench
original=binary.read_bytes(); original_hash=bench.sha256(binary)
command=['cargo','build','--release','--offline','--locked','--no-default-features','--manifest-path',str(source/'benchmarks/adapters/rust/Cargo.toml')]
started=time.monotonic()
try:
 with (W/f'build-{args.name}.log').open('w') as log:
  subprocess.run(command,env=dict(os.environ,CARGO_TARGET_DIR=str(R/'benchmarks/adapters/rust/target')),check=True,stdout=log,stderr=subprocess.STDOUT)
 shutil.copy2(binary,W/'binaries'/args.name)
 bench.save(W/f'build-{args.name}.json',dict(command=command,elapsed_s=time.monotonic()-started,binary_sha256=bench.sha256(W/'binaries'/args.name),source_files={str(p.relative_to(source)):bench.sha256(p) for p in source.rglob('*') if p.is_file()}))
finally:
 binary.write_bytes(original);assert bench.sha256(binary)==original_hash
 print('Official executable restored',flush=True)
