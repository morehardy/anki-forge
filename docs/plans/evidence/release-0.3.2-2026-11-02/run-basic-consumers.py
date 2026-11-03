# Independent reviewer: fresh public-registry basic consumers (Rust/Python/Node) and the
# duplicate-key troubleshooting reproduction for the release-0.3.2 website artifact.
# The website teaches the published 0.3.1 packages, so every consumer installs exactly that
# public version from crates.io / PyPI / npm (fresh resolution, no copied lockfiles, no caches).
from pathlib import Path
import subprocess, os, json, hashlib, shutil, datetime
root = Path('/Users/hp/Desktop/2026/anki-forge')
work = root/'target/release-0.3.2-review/basic-execution'
evidence = root/'docs/plans/evidence/release-0.3.2-2026-11-02/executions/basic'
prior = root/'docs/plans/evidence/bing-seo-2026-11-02/executions/basic'
quickstart = json.loads((root/'docs/ai-quickstart.json').read_text())
version = quickstart['publicVersion']
assert version == '0.3.1', version
env = os.environ.copy()
env['PATH'] = '/Users/hp/.nvm/versions/node/v24.19.0/bin:/Users/hp/.cargo/bin:' + env['PATH']
env.pop('CARGO_TARGET_DIR', None)
if work.exists(): shutil.rmtree(work)
if evidence.exists(): shutil.rmtree(evidence)
for lang in ['rust', 'python', 'node']:
    (work/lang).mkdir(parents=True, exist_ok=True); (evidence/lang).mkdir(parents=True, exist_ok=True)
def copy(src, dst): dst.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(src, dst)
def run(args, lang, log, cwd=None):
    out = subprocess.run(args, cwd=cwd or work/lang, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    with open(evidence/lang/log, 'a') as f: f.write('$ ' + ' '.join(map(str, args)) + '\n' + out.stdout + '\nexit=' + str(out.returncode) + '\n')
    if out.returncode: raise RuntimeError(f'{lang}: {args}: {out.stdout[-3000:]}')
    return out.stdout

# Rust: the documented `cargo add ankiforge@0.3.1` into a fresh binary crate, plus the
# public consumer inspector (reads ZIP/protobuf/SQLite directly, no SDK internals).
rust = work/'rust'
(rust/'Cargo.toml').write_text('''[package]
name = "locale_data_basic_task"
version = "0.0.0"
edition = "2021"
[workspace]
[[bin]]
name = "locale_data_basic_task"
path = "src/main.rs"
[dependencies]
anyhow = "1"
serde_json = "1"
tempfile = "3"
prost = "0.13"
rusqlite = { version = "0.32", features = ["bundled"] }
zip = { version = "2", default-features = false, features = ["deflate"] }
zstd = "0.13"
''')
copy(root/'anki_forge/examples/target_api_basic.rs', rust/'src/main.rs')
copy(root/'scripts/packaged_consumer/src/inspection.rs', rust/'src/inspection.rs')
copy(root/'scripts/packaged_consumer/src/bin/verify_apkg.rs', rust/'src/bin/verify_apkg.rs')
run(['cargo', 'add', f'ankiforge@{version}'], 'rust', 'install.log')
run(['cargo', 'generate-lockfile'], 'rust', 'install.log')
lock = (rust/'Cargo.lock').read_text()
record = next(r for r in lock.split('[[package]]') if '\nname = "ankiforge"\n' in r)
assert f'version = "{version}"' in record and 'source = "registry+https://github.com/rust-lang/crates.io-index"' in record, record
run(['cargo', 'run', '--locked', '--quiet', '--bin', 'locale_data_basic_task'], 'rust', 'basic.log')

# Python: documented venv + `python -m pip install ankiforge==0.3.1` (public wheel, no cache).
run(['python3', '-m', 'venv', '.venv'], 'python', 'venv.log')
py = str(work/'python/.venv/bin/python')
run([py, '-m', 'pip', 'install', '--no-cache-dir', '--index-url', 'https://pypi.org/simple', '--only-binary=:all:', '--report', 'pip-report.json', f'ankiforge=={version}'], 'python', 'install.log')
installed = next(e for e in json.loads((work/'python/pip-report.json').read_text())['install'] if e['metadata']['name'] == 'ankiforge')
assert installed['metadata']['version'] == version and installed['download_info']['url'].startswith('https://files.pythonhosted.org/') and installed['download_info']['url'].endswith('.whl')
copy(root/'bindings/python/examples/quickstart.py', work/'python/main.py')
copy(prior/'python/duplicate-key.py', work/'python/duplicate-key.py')
run([py, 'main.py'], 'python', 'basic.log')
troubleshooting = run([py, 'duplicate-key.py'], 'python', 'troubleshooting.log')
assert 'NOTE.KEY_DUPLICATE' in troubleshooting

# Node: documented `npm install --include=optional ankiforge@0.3.1` (fresh resolution from npmjs).
(work/'node/package.json').write_text('{"name":"locale-data-basic-task","private":true,"type":"module"}\n')
run(['npm', 'install', '--prefer-online', '--registry=https://registry.npmjs.org', '--include=optional', '--ignore-scripts', f'ankiforge@{version}'], 'node', 'install.log')
nlock = json.loads((work/'node/package-lock.json').read_text())
for name, pkg in nlock['packages'].items():
    if name.startswith('node_modules/ankiforge'):
        assert pkg['version'] == version and pkg['resolved'].startswith('https://registry.npmjs.org/'), (name, pkg)
        assert not pkg.get('hasInstallScript'), f'{name} declares install scripts; --ignore-scripts would change behaviour'
copy(root/'bindings/node/examples/quickstart.mjs', work/'node/main.mjs')
run(['node', 'main.mjs'], 'node', 'basic.log')

for lang in ['rust', 'python', 'node']:
    run(['cargo', 'run', '--locked', '--quiet', '--manifest-path', str(rust/'Cargo.toml'), '--bin', 'verify_apkg', '--', str(work/lang)], 'rust', f'inspection-{lang}.log')
    for file in (work/lang).iterdir():
        if file.is_file() and file.suffix in ['.apkg', '.json', '.mjs', '.py', '.toml', '.lock']: copy(file, evidence/lang/file.name)
copy(rust/'src/main.rs', evidence/'rust/src/main.rs')
priorAudit = json.loads((root/'docs/plans/evidence/bing-seo-2026-11-02/usage-execution-audit.json').read_text())
out = {'schemaVersion': 1, 'reviewerRun': '/root/usage_release_review/release-0.3.2-2026-11-02', 'executedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
       'publicVersion': version, 'pythonWheel': installed['download_info']['url'],
       'description': 'Fresh public-registry basic programs (documented cargo add / pip install / npm install of 0.3.1) and duplicate-key recovery executed by this independent reviewer; every generated APKG decoded by the public consumer inspector.'}
for kind in ['sourceBindings', 'dependencyBindings']:
    out[kind] = [{'path': b['path'], 'sha256': hashlib.sha256((root/b['path']).read_bytes()).hexdigest()} for b in priorAudit[kind]]
(evidence/'execution-inputs.json').write_text(json.dumps(out, indent=2) + '\n')
print('Fresh basic Rust, Python, Node and duplicate-key recovery passed; APKG content independently decoded.')
