"""Build the accepted public defaults and refresh exact benchmark provenance."""
import json
from pathlib import Path
import shutil
import sys

WORK = Path(__file__).resolve().parent
REPO = WORK.parents[2]
sys.path.insert(0, str(REPO/'benchmarks'))
import bench

adapter = REPO/'benchmarks/adapters/rust/target/release/anki-forge-benchmark'
bench.run_build(['cargo', 'build', '--release', '--locked', '--manifest-path', str(REPO/'benchmarks/adapters/rust/Cargo.toml')], [adapter])
bench.run_build(['cargo', 'build', '--release', '--locked', '-p', 'contract_tools'], [bench.INSPECTOR])
shutil.copy2(adapter, WORK/'binaries/accepted')
(WORK/'accepted-build.json').write_text(json.dumps({
    'provenance': bench.build_provenance(bench.registry()),
    'identity': bench.identity_snapshot(bench.registry()),
    'adapter_sha256': bench.sha256(adapter),
}, indent=2)+'\n')
assert all(row['status']=='verified' for row in bench.build_provenance(bench.registry()).values())
print('Accepted default adapter, inspector and existing oracle/collector provenance verified')
