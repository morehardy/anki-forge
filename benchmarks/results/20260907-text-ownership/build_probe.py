"""Build one private probe, then restore the hash-pinned benchmark adapter."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
name = sys.argv[1]
binary = ROOT / 'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
command = ['cargo', 'build', '--release', '--locked', '--manifest-path', str(WORK / 'profile-source/benchmarks/adapters/rust/Cargo.toml')]
environment = dict(os.environ, CARGO_NET_OFFLINE='true', CARGO_TARGET_DIR=str(ROOT / 'benchmarks/adapters/rust/target'))
started = time.monotonic()
try:
    # copy2 preserves mtimes and can make Cargo reuse a stale crate after a
    # variant switch. Mark the exact private source tree fresh before building.
    for directory in ('anki_forge', 'benchmarks/adapters/rust'):
        for path in (WORK / 'profile-source' / directory).rglob('*'):
            if path.is_file():
                os.utime(path, None)
    with (WORK / f'build-{name}.log').open('w') as log:
        subprocess.run(command, env=environment, stdout=log, stderr=subprocess.STDOUT, check=True)
    shutil.copy2(binary, WORK / 'binaries' / name)
    (WORK / f'build-{name}.json').write_text(json.dumps(dict(command=command, elapsed_s=time.monotonic() - started,
        binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), source_sha256={
            str(path.relative_to(WORK / 'profile-source')): hashlib.sha256(path.read_bytes()).hexdigest()
            for directory in ('anki_forge', 'benchmarks/adapters/rust')
            for path in (WORK / 'profile-source' / directory).rglob('*') if path.is_file()}), indent=2) + '\n')
    print(name, 'built in', round(time.monotonic() - started, 1), 'seconds')
finally:
    shutil.copy2(WORK / 'binaries/accepted', binary)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == hashlib.sha256((WORK / 'binaries/accepted').read_bytes()).hexdigest()
    print('Original measured adapter restored')
