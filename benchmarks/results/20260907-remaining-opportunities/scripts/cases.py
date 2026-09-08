"""Diagnostic extensions only; never mutate the frozen README corpus."""
from pathlib import Path
import copy
import hashlib
import json
import struct
import sys
import zlib

WORK = Path(__file__).resolve().parent
ROOT = WORK.parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks'))
import workload

FROZEN = ROOT / 'benchmarks/.work/readme-comparison/fixtures'
CASES = []


def describe(name, path):
    data = json.loads(path.read_text())
    CASES.append(dict(name=name, path=str(path), input_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                      notes=len(data['notes']), media=len(data.get('media', [])),
                      media_bytes=sum(m['bytes'] for m in data.get('media', [])),
                      field_utf8_bytes=sum(len(n[f].encode()) for n in data['notes'] for f in ('front', 'back'))))


def save(name, data):
    path = WORK / 'fixtures' / name / 'input.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(workload.serialize(data))
    describe(name, path)


def image_case(name, count, sizes):
    source = FROZEN / 'basic-image-unique-v2/inputs/1000.json'
    data = json.loads(source.read_text())
    data.update(profile=name, note_count=count, notes=data['notes'][:count], media=data['media'][:count])
    assets = WORK / 'fixtures' / name / 'media'
    assets.mkdir(parents=True, exist_ok=False)
    for i, item in enumerate(data['media']):
        original = (source.parent / item['path']).read_bytes()
        target = sizes[i % len(sizes)]
        if target == len(original):
            payload = original
        else:
            padding = hashlib.shake_256(f'anki-forge-diagnostic-padding:{i}'.encode()).digest(target - len(original) - 12)
            kind = b'afGp'  # Private ancillary PNG chunk, valid length and CRC.
            chunk = struct.pack('>I', len(padding)) + kind + padding + struct.pack('>I', zlib.crc32(kind + padding))
            payload = original[:-12] + chunk + original[-12:]
        assert len(payload) == target
        (assets / item['filename']).write_bytes(payload)
        item.update(bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest())
    save(name, data)


if __name__ == '__main__':
    for label, profile in [('text', 'basic-mixed-text-v1'), ('image', 'basic-image-unique-v2'),
                           ('audio', 'basic-audio-unique-v2'), ('mixed', 'basic-mixed-unique-v2'),
                           ('shared', 'basic-mixed-shared-v2')]:
        describe(f'{label}-1000', FROZEN / profile / 'inputs/1000.json')
    describe('text-100', FROZEN / 'basic-mixed-text-v1/inputs/100.json')
    notes = []
    for i in range(10000):
        category = 'english' if i % 20 < 10 else 'mixed' if i % 20 < 18 else 'escaping'
        notes.append(dict(id=f'BF-{i + 1:05d}', category=category,
                          front=workload.field_text(i, 'front', category),
                          back=workload.field_text(i, 'back', category)))
    save('text-10000', workload.document(notes, len(notes)))
    data = copy.deepcopy(workload.document(notes, 1000))
    for note in data['notes']:
        note['back'] = (note['back'] * 120)[:4096]
    save('long-back-1000', data)
    image_case('boundary-65536', 1000, [65536])
    image_case('boundary-65537', 1000, [65537])
    image_case('image-256x256k', 256, [262144])
    image_case('image-64x1m', 64, [1048576])
    image_case('image-skewed-128', 128, [1048576, 64152, 64152, 64152])
    (WORK / 'cases.json').write_text(json.dumps(CASES, indent=2) + '\n')
    print(json.dumps(CASES, indent=2))
