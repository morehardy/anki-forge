# Independent second inspection of the fresh public JSON recipe outputs (Rust/Python/Node v1/v2).
# 1) Runs the public consumer inspector (scripts/packaged_consumer verify_json_recipe) on each SDK output.
# 2) Decodes every APKG with this reviewer's own Python reader (zip + zstd + SQLite), independent of the
#    repository's Rust inspection code: notes/cards counts, GUID stability, exactly one changed field,
#    exact media bytes against the recipe inputs, and comparison/review policy JSON.
import subprocess, json, hashlib, zipfile, sqlite3, os, shutil
from pathlib import Path
root = Path('/Users/hp/Desktop/2026/anki-forge')
ev = root/'docs/plans/evidence/release-0.3.2-2026-11-02'
outputs = ev/'executions/public-json'
recipe = root/'docs/recipes/json-to-anki'
crate = root/'target/release-0.3.2-review/basic-execution/rust'
scratch = root/'target/release-0.3.2-review/json-decode'
env = os.environ.copy(); env['PATH'] = '/Users/hp/.cargo/bin:' + env['PATH']; env.pop('CARGO_TARGET_DIR', None)
sha = lambda b: hashlib.sha256(b).hexdigest()
log = []
shutil.copyfile(root/'scripts/packaged_consumer/src/bin/verify_json_recipe.rs', crate/'src/bin/verify_json_recipe.rs')
subprocess.run(['cargo', 'build', '--locked', '--quiet', '--bin', 'verify_json_recipe'], cwd=crate, env=env, check=True)
inspector = crate/'target/debug/verify_json_recipe'
for lang in ['rust', 'python', 'node']:
    out = subprocess.run([str(inspector), str(outputs/lang), str(recipe)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log.append(f'$ {inspector} {outputs/lang} {recipe}\n{out.stdout}exit={out.returncode}\n')
    assert out.returncode == 0, out.stdout

def unz(data):
    if data[:4] == b'\x28\xb5\x2f\xfd':
        return subprocess.run(['zstd', '-q', '-d', '-c'], input=data, stdout=subprocess.PIPE, check=True).stdout
    return data

def decode(path, tag):
    z = zipfile.ZipFile(path)
    scratch.mkdir(parents=True, exist_ok=True)
    db = scratch/f'{tag}.sqlite'; db.write_bytes(unz(z.read('collection.anki21b')))
    con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    notes = {g: f for g, f in con.execute('select guid, flds from notes')}
    cards = con.execute('select count(*) from cards').fetchone()[0]
    con.close()
    media = sorted(sha(unz(z.read(n))) for n in z.namelist() if n.isdigit())
    return notes, cards, media

expected_media = sorted({sha((recipe/'cell.png').read_bytes()), sha((recipe/'tone.wav').read_bytes())})
summary = {}
for lang in ['rust', 'python', 'node']:
    n1, c1, m1 = decode(outputs/lang/'v1.apkg', f'{lang}-v1'); n2, c2, m2 = decode(outputs/lang/'v2.apkg', f'{lang}-v2')
    changed = [g for g in n1 if n1[g] != n2.get(g)]
    comparison = json.loads((outputs/lang/'comparison.json').read_text()); review = json.loads((outputs/lang/'review.json').read_text())
    install = json.loads((outputs/lang/'installation.json').read_text())
    version = (install['source'].get('version') or install['source'].get('metadata', {}).get('version'))
    checks = {'publicVersion': version, 'v1Notes': len(n1), 'v1Cards': c1, 'v2Notes': len(n2), 'v2Cards': c2,
              'guidsStable': set(n1) == set(n2), 'changedNotes': len(changed),
              'mediaExact': m1 == expected_media and m2 == expected_media, 'mediaCount': len(m2),
              'comparisonSha256': sha((outputs/lang/'comparison.json').read_bytes()), 'reviewSha256': sha((outputs/lang/'review.json').read_bytes())}
    assert version == '0.3.1' and len(n1) == len(n2) == c1 == c2 == 3 and checks['guidsStable'] and len(changed) == 1 and checks['mediaExact'], checks
    summary[lang] = checks
    log.append(f'independent-python-decode {lang}: ' + json.dumps(checks, sort_keys=True) + '\n')
(ev/'usage-json-inspection.log').write_text('\n'.join(log))
print(''.join(log))
