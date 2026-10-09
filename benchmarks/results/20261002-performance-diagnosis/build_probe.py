from pathlib import Path
import hashlib,json,os,shutil,subprocess,time,sys
W=Path(__file__).resolve().parent; R=W.parents[2]
binary=R/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
name=sys.argv[1] if len(sys.argv)>1 else 'stages'
command=['cargo','build','--release','--offline','--locked','--no-default-features','--manifest-path',str(W/('historical-source' if name=='historical' else 'probe-source')/'benchmarks/adapters/rust/Cargo.toml')]
env=dict(os.environ,CARGO_TARGET_DIR=str(R/'benchmarks/adapters/rust/target'))
start=time.monotonic()
try:
    with (W/f'build-{name}.log').open('w') as stream:
        subprocess.run(command,env=env,stdout=stream,stderr=subprocess.STDOUT,check=True)
    shutil.copy2(binary,W/'binaries'/name)
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    (W/f'build-{name}.json').write_text(json.dumps({'command':command,'elapsed_s':time.monotonic()-start,'binary_sha256':sha(W/'binaries'/name),'source_files':{str(p.relative_to(W/('historical-source' if name=='historical' else 'probe-source'))):sha(p) for p in (W/('historical-source' if name=='historical' else 'probe-source')).rglob('*') if p.is_file()}},indent=2)+'\n')
    print('Diagnostic stages probe built')
finally:
    shutil.copy2(W/'binaries/original',binary)
    assert binary.read_bytes()==(W/'binaries/original').read_bytes()
    print('Original benchmark executable restored')
