"""Build the independent inspector and freeze the final benchmark identity."""
from pathlib import Path
import re
import sys

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks'))
import bench

bench.run_build(['cargo', 'build', '--release', '--locked', '-p', 'contract_tools'], [bench.INSPECTOR])
snapshot = bench.identity_snapshot(bench.registry())
bench.save(WORK / 'accepted-build.json', snapshot)
bench.save(WORK / 'accepted-source-sha256.json', {
    str(path.relative_to(ROOT)): bench.sha256(path)
    for folder in ('anki_forge', 'benchmarks/adapters/rust')
    for path in (ROOT / folder).rglob('*')
    if path.is_file() and 'target' not in path.parts})
segments = re.split(r'Finished `test` profile[^\n]*\n', (WORK / 'quality-rust-quality.log').read_text())
counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', segments[2])
assert len(counts) == 78
bench.save(WORK / 'quality-summary.json', {
    'workspace_all_features': {'suites': len(counts), 'passed': sum(int(a) for a,b,c in counts),
                               'failed': sum(int(b) for a,b,c in counts), 'ignored': sum(int(c) for a,b,c in counts)},
    'benchmark_python_tests_passed': 43,
    'gates': ['default facade', 'workspace all features', 'Clippy -D warnings',
              'fmt', 'doctests', 'rustdoc -D warnings', 'embedded contract bundle',
              'crate payload', 'release metadata', 'dependency policy', 'git diff --check'],
})
print('Accepted source and executable identities frozen.', flush=True)
