"""Recreate the two fixed-note-count memory fixtures, or verify their frozen bytes."""
from pathlib import Path
import argparse
import hashlib
import json

WORK = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--verify-existing', action='store_true')
args = parser.parse_args()
cases = json.loads((WORK / 'cases.json').read_text())
base = next(c for c in cases if c['name'] == 'long-back-1000')
for name, multiplier in [('long-back-quarter-1000', 0.25), ('long-back-quadruple-1000', 4)]:
    document = json.loads(Path(base['path']).read_text())
    for note in document['notes']:
        note['back'] = note['back'][:len(note['back'])//4] if multiplier < 1 else note['back']*4
    data = (json.dumps(document, ensure_ascii=False, separators=(',', ':')) + '\n').encode()
    path = WORK / 'fixtures' / name / 'input.json'
    digest = hashlib.sha256(data).hexdigest()
    if args.verify_existing:
        expected = next(c for c in cases if c['name'] == name)
        assert data == path.read_bytes() and digest == expected['input_sha256']
    else:
        assert all(c['name'] != name for c in cases)
        path.parent.mkdir(parents=True, exist_ok=False)
        path.write_bytes(data)
        cases.append({'name': name, 'path': str(path), 'input_sha256': digest, 'notes': 1000,
                      'media': 0, 'media_bytes': 0,
                      'field_utf8_bytes': sum(len((n['front']+n['back']).encode()) for n in document['notes'])})
    print(name, digest)
if not args.verify_existing:
    (WORK / 'cases.json').write_text(json.dumps(cases, indent=2) + '\n')
