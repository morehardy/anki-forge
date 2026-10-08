"""Offline reconciliation using only frozen records/source and the Python stdlib."""
from pathlib import Path
import hashlib
import importlib.util
import json
import shutil
import subprocess
import sys
import tarfile
import tempfile

D = Path(__file__).resolve().parent
def read(p):
    return json.loads(p.read_text())
spec = importlib.util.spec_from_file_location('archive_check', D / 'verify-archives.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
archives = module.verify(D)
with tempfile.TemporaryDirectory(prefix='ankiforge-publication-replay-') as temporary:
    root = Path(temporary)
    analysis = root / 'analysis'
    analysis.mkdir()
    for p in D.iterdir():
        if p.is_file() and p.suffix not in {'.png', '.svg'}:
            shutil.copy2(p, analysis / p.name)
    for name in read(D / 'archive-manifest.json')['archives']:
        with tarfile.open(D / name) as archive:
            for member in archive.getmembers():
                # verify() rejected links, traversal and duplicate members.
                p = root / member.name
                assert not p.exists(), member.name
                p.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source, p.open('wb') as target:
                    shutil.copyfileobj(source, target)
    snapshot = read(analysis / 'source-snapshot.json')
    for name, record in read(analysis / 'reference-check.json')['references'].items():
        assert hashlib.sha256((analysis / name).read_bytes()).hexdigest() == record['sha256']
        assert record['sha256'] == snapshot['source_files'][record['source']]
    assert read(analysis / 'identity-after.json') == snapshot
    source_count = 0
    for p in (root / 'source').rglob('*'):
        if p.is_file():
            name = str(p.relative_to(root / 'source'))
            assert hashlib.sha256(p.read_bytes()).hexdigest() == snapshot['source_files'][name], name
            source_count += 1
    for name, expected in read(analysis / 'baseline-input-check.json')['sha256'].items():
        assert hashlib.sha256((root / 'fixtures' / name).read_bytes()).hexdigest() == expected
    result = subprocess.run([sys.executable, str(analysis / 'analyze.py'), '--run-dir', str(root / 'run'),
                             '--source-root', str(root / 'source')], check=True, capture_output=True, text=True)
    assert json.loads(result.stdout) == {'status': 'verified', 'attempts': 840, 'anki_checks': 40}
    for name in ['summary.json', 'verification-summary.json', 'comparison.csv']:
        assert (analysis / name).read_bytes() == (D / name).read_bytes(), name
    # report.py reads the archived reference summaries, not another checkout.
    subprocess.run([sys.executable, str(analysis / 'report.py')], check=True, capture_output=True, text=True)
    for name in ['delta.json', 'delta.csv', 'report.md']:
        assert (analysis / name).read_bytes() == (D / name).read_bytes(), name
    print(json.dumps({'status': 'passed', 'exports_reconciled': 840, 'anki_records_reconciled': 40,
                      'archived_source_files_verified': source_count, 'fixture_json_verified': 20,
                      'summaries_and_comparisons_byte_identical': True, 'archives': archives,
                      'scope': 'Retained-record reconciliation only. No exporter, Anki, build or original absolute path executed. '
                               'APKG/binary/media bytes omitted and not reverified.'}, indent=2))
