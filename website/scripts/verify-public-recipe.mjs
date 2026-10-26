import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadAiQuickstart } from './ai-quickstart.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const data = await loadAiQuickstart();
const requested = process.argv[2] ?? 'all';
assert.ok(['all', 'rust', 'python', 'node'].includes(requested), 'Use all, rust, python or node');
const languages = requested === 'all' ? ['rust', 'python', 'node'] : [requested];
const evidence = path.resolve(process.argv[3] ?? path.join(root, 'target/geo-public-consumers'));
const work = await mkdtemp(path.join(os.tmpdir(), 'ankiforge-public-recipe-'));
const fixture = path.join(root, 'docs/recipes/json-to-anki');
const run = (command, args, cwd = work) => execFileSync(command, args, { cwd, stdio: 'inherit' });
const readJSON = async file => JSON.parse(await readFile(file, 'utf8'));
const digest = async file => createHash('sha256').update(await readFile(file)).digest('hex');

try {
  // The inspector reads ZIP/protobuf/SQLite directly; it uses no SDK internals.
  const rust = path.join(work, 'rust');
  await mkdir(path.join(rust, 'src/bin'), { recursive: true });
  await writeFile(path.join(rust, 'Cargo.toml'), `[package]
name = "geo_recipe_consumer"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
ankiforge = "=${data.publicVersion}"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
prost = "0.13"
rusqlite = { version = "0.32", features = ["bundled"] }
zip = { version = "2", default-features = false, features = ["deflate"] }
zstd = "0.13"
tempfile = "3"
`);
  await cp(path.join(root, 'anki_forge/examples/json_to_anki.rs'), path.join(rust, 'src/main.rs'));
  await cp(path.join(root, 'scripts/packaged_consumer/src/inspection.rs'), path.join(rust, 'src/inspection.rs'));
  await cp(path.join(root, 'scripts/packaged_consumer/src/bin/verify_json_recipe.rs'), path.join(rust, 'src/bin/verify_json_recipe.rs'));
  run('cargo', ['generate-lockfile'], rust);
  const lock = await readFile(path.join(rust, 'Cargo.lock'), 'utf8');
  const resolved = lock.split('[[package]]').find(record => /\nname = "ankiforge"\n/.test(record));
  assert.ok(resolved?.includes(`version = "${data.publicVersion}"`) && /source = "registry\+https:\/\/github.com\/rust-lang\/crates.io-index"/.test(resolved), 'Rust must resolve the exact crates.io package');
  run('cargo', ['build', '--locked', '--quiet', '--bin', 'verify_json_recipe'], rust);
  const inspector = path.join(rust, 'target/debug', `verify_json_recipe${process.platform === 'win32' ? '.exe' : ''}`);
  for (const language of languages) {
    const app = language === 'rust' ? rust : path.join(work, language);
    const output = path.join(app, 'output', language);
    await mkdir(app, { recursive: true });
    await cp(fixture, path.join(app, 'recipe'), { recursive: true });
    let command, args, source;
    if (language === 'rust') {
      command = 'cargo'; args = ['run', '--locked', '--quiet', '--bin', 'geo_recipe_consumer', '--', 'recipe', `output/${language}`];
      source = { registry: 'crates.io', package: 'ankiforge', version: data.publicVersion, resolution: resolved.trim() };
    } else if (language === 'python') {
      run('python3', ['-m', 'venv', '.venv'], app);
      command = path.join(app, '.venv', process.platform === 'win32' ? 'Scripts/python.exe' : 'bin/python');
      run(command, ['-m', 'pip', 'install', '--index-url', 'https://pypi.org/simple', '--only-binary=:all:', '--report', 'install-report.json', `ankiforge==${data.publicVersion}`], app);
      const installed = (await readJSON(path.join(app, 'install-report.json'))).install.find(entry => entry.metadata.name === 'ankiforge');
      assert.equal(installed.metadata.version, data.publicVersion);
      assert.ok(new URL(installed.download_info.url).hostname === 'files.pythonhosted.org' && installed.download_info.url.endsWith('.whl'), 'Use a public PyPI wheel, never a local build');
      run(command, ['-c', `import importlib.metadata; assert importlib.metadata.version('ankiforge') == '${data.publicVersion}'`], app);
      await cp(path.join(root, 'bindings/python/examples/json_to_anki.py'), path.join(app, 'main.py'));
      args = ['main.py', 'recipe', `output/${language}`];
      source = installed;
    } else {
      await writeFile(path.join(app, 'package.json'), '{"name":"geo-recipe-consumer","private":true,"type":"module"}\n');
      run('npm', ['install', '--registry=https://registry.npmjs.org', '--include=optional', '--ignore-scripts', `ankiforge@${data.publicVersion}`, 'typescript@6.0.3', '@types/node@24.13.6'], app);
      const lock = await readJSON(path.join(app, 'package-lock.json'));
      const installed = lock.packages['node_modules/ankiforge'];
      assert.equal(installed.version, data.publicVersion);
      assert.equal(new URL(installed.resolved).hostname, 'registry.npmjs.org');
      run('npm', ['ls', 'ankiforge', '--depth=0'], app);
      await cp(path.join(root, 'bindings/node/examples/json_to_anki.mts'), path.join(app, 'main.mts'));
      run(process.execPath, ['node_modules/typescript/bin/tsc', 'main.mts', '--target', 'ES2022', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--strict', '--outDir', 'dist', '--types', 'node'], app);
      command = process.execPath; args = ['dist/main.mjs', 'recipe', `output/${language}`];
      const native = [];
      for (const [name, record] of Object.entries(lock.packages)) {
        if (!name.startsWith('node_modules/ankiforge-')) continue;
        let metadata;
        try { metadata = await readJSON(path.join(app, name, 'package.json')); }
        catch (error) { if (error.code === 'ENOENT') continue; throw error; }
        assert.equal(metadata.name, path.basename(name));
        assert.equal(metadata.version, data.publicVersion, 'Installed native package version mismatch');
        assert.equal(record.version, data.publicVersion);
        assert.equal(new URL(record.resolved).origin, 'https://registry.npmjs.org');
        assert.ok(metadata.os.includes(process.platform) && metadata.cpu.includes(process.arch), 'Native platform mismatch');
        native.push([name, record]);
      }
      assert.equal(native.length, 1, 'Expected exactly one installed native platform package');
      source = { ...installed, native };
    }
    run(command, args, app);
    run(inspector, [output, path.join(app, 'recipe')], app);
    // A repeated invocation must refuse to overwrite the original release.
    const before = await digest(path.join(output, 'v1.apkg'));
    const repeated = spawnSync(command, args, { cwd: app, encoding: 'utf8' });
    assert.notEqual(repeated.status, 0, 'A repeat must fail instead of overwriting releases');
    assert.match(repeated.stderr, /Choose a fresh output directory/);
    assert.equal(await digest(path.join(output, 'v1.apkg')), before, 'Original v1 was overwritten');
    const destination = path.join(evidence, language);
    await mkdir(destination, { recursive: true });
    await cp(output, destination, { recursive: true });
    await writeFile(path.join(destination, 'installation.json'), JSON.stringify({
      checkedAt: new Date().toISOString(), platform: process.platform, arch: process.arch,
      node: process.version, language, source, contract: '3 notes / 3 cards / 2 exact media; stable identity; one answer changed; compare + review policy allowed; v1 preserved',
    }, null, 2) + '\n');
    if (language === 'rust') await cp(path.join(rust, 'Cargo.lock'), path.join(destination, 'Cargo.lock'));
    console.log(`Verified public ${language} ${data.publicVersion}; evidence: ${destination}`);
  }
} finally {
  await rm(work, { recursive: true, force: true });
}
