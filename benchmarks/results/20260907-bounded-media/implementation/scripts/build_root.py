"""Build the real workspace adapter with normal benchmark provenance."""
from pathlib import Path
import json
import shutil
import sys

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks'))
import bench

name = sys.argv[1]
adapter = ROOT / 'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
bench.run_build(['cargo', 'build', '--release', '--locked', '--manifest-path',
                 str(ROOT / 'benchmarks/adapters/rust/Cargo.toml')], [adapter])
shutil.copy2(adapter, WORK / 'binaries' / name)
bench.save(WORK / f'build-{name}.json', {
    'binary_sha256': bench.sha256(adapter),
    'provenance': json.loads(bench.BUILD_RECORDS.read_text())[str(adapter)],
    'source_sha256': {str(path.relative_to(ROOT)): bench.sha256(path)
        for folder in ('anki_forge', 'benchmarks/adapters/rust')
        for path in (ROOT / folder).rglob('*')
        if path.is_file() and 'target' not in path.parts},
})
print(name, bench.sha256(adapter), flush=True)
