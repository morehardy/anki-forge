// Fresh public-package executions for the basic, troubleshooting and inspection tasks.
// Usage: node run-basic-tasks.mjs <repository root>
// Programs are exact copies of repository sources; ankiforge comes from crates.io, PyPI and npm.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(process.argv[2] ?? '.');
const evidence = path.dirname(fileURLToPath(import.meta.url));
const work = '/private/tmp/anki-forge-locale-data-execution';
const data = JSON.parse(await readFile(path.join(root, 'docs/ai-quickstart.json'), 'utf8'));
const version = data.publicVersion;
const digest = async file => createHash('sha256').update(await readFile(file)).digest('hex');

async function run(log, command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', env: { ...process.env, CARGO_TERM_COLOR: 'never', NO_COLOR: '1' } });
  const text = `$ ${[command, ...args].join(' ')}\n(cwd ${cwd}; exit ${result.status})\n${result.stdout}${result.stderr}`;
  await writeFile(log, text, { flag: 'a' });
  assert.equal(result.status, 0, `${command} ${args.join(' ')} failed; see ${log}`);
  return result.stdout;
}

await rm(work, { recursive: true, force: true });
for (const language of ['rust', 'python', 'node']) {
  await mkdir(path.join(work, language), { recursive: true });
  await rm(path.join(evidence, 'basic', language), { recursive: true, force: true });
  await mkdir(path.join(evidence, 'basic', language), { recursive: true });
}
const logs = language => ({ install: path.join(evidence, 'basic', language, 'install.log'), basic: path.join(evidence, 'basic', language, 'basic.log'), inspection: path.join(evidence, 'basic', language, 'inspection.log') });

// Rust: the documented `cargo add` install, the repository quickstart as main, and the repository APKG inspector.
const rust = path.join(work, 'rust');
await writeFile(path.join(rust, 'Cargo.toml'), `[package]\nname = "locale_data_basic_task"\nversion = "0.0.0"\nedition = "2021"\n[workspace]\n[[bin]]\nname = "locale_data_basic_task"\npath = "src/main.rs"\n[dependencies]\nanyhow = "1"\nserde_json = "1"\ntempfile = "3"\nprost = "0.13"\nrusqlite = { version = "0.32", features = ["bundled"] }\nzip = { version = "2", default-features = false, features = ["deflate"] }\nzstd = "0.13"\n`);
await mkdir(path.join(rust, 'src/bin'), { recursive: true });
await cp(path.join(root, 'anki_forge/examples/target_api_basic.rs'), path.join(rust, 'src/main.rs'));
await cp(path.join(root, 'scripts/packaged_consumer/src/inspection.rs'), path.join(rust, 'src/inspection.rs'));
await cp(path.join(root, 'scripts/packaged_consumer/src/bin/verify_apkg.rs'), path.join(rust, 'src/bin/verify_apkg.rs'));
assert.equal(data.languages.rust.install, `cargo add ankiforge@${version}`);
await run(logs('rust').install, 'cargo', ['add', `ankiforge@${version}`], rust);
await run(logs('rust').install, 'cargo', ['generate-lockfile'], rust);
assert.match(await readFile(path.join(rust, 'Cargo.lock'), 'utf8'), new RegExp(`name = "ankiforge"\\nversion = "${version.replaceAll('.', '\\.')}"\\n`), 'cargo add must resolve the published version');
await run(logs('rust').basic, 'cargo', ['run', '--quiet', '--bin', 'locale_data_basic_task'], rust);
// The inspector verifies every .apkg in the directory it is given.
const inspect = async language => run(logs(language).inspection, 'cargo', ['run', '--quiet', '--manifest-path', path.join(rust, 'Cargo.toml'), '--bin', 'verify_apkg', '--', path.join(work, language)], rust);
await inspect('rust');

// Python: an isolated virtual environment and the documented pip command.
const python = path.join(work, 'python');
assert.equal(data.languages.python.install, `python -m pip install ankiforge==${version}`);
await run(logs('python').install, 'python3', ['-m', 'venv', 'venv'], python);
await run(logs('python').install, path.join(python, 'venv/bin/python'), ['-m', 'pip', 'install', '--report', path.join(evidence, 'basic/python/pip-report.json'), `ankiforge==${version}`], python);
await cp(path.join(root, 'bindings/python/examples/quickstart.py'), path.join(python, 'main.py'));
await cp(path.join(evidence, 'duplicate-key.py'), path.join(python, 'duplicate-key.py'));
await run(logs('python').basic, path.join(python, 'venv/bin/python'), ['main.py'], python);
await run(path.join(evidence, 'basic/python/troubleshooting.log'), path.join(python, 'venv/bin/python'), ['duplicate-key.py'], python);
await inspect('python');

// Node: the documented npm command, including optional native packages.
const node = path.join(work, 'node');
assert.equal(data.languages.node.install, `npm install --include=optional ankiforge@${version}`);
await writeFile(path.join(node, 'package.json'), '{"name":"locale-data-basic-task","private":true,"type":"module"}\n');
await run(logs('node').install, 'npm', ['install', '--include=optional', `ankiforge@${version}`], node);
await cp(path.join(root, 'bindings/node/examples/quickstart.mjs'), path.join(node, 'main.mjs'));
await run(logs('node').basic, 'node', ['main.mjs'], node);
await inspect('node');

// Keep exact programs, lockfiles and outputs next to their logs.
const kept = {
  rust: ['Cargo.toml', 'Cargo.lock', 'src/main.rs', 'spanish.apkg'],
  python: ['main.py', 'duplicate-key.py', 'spanish.apkg', 'recovered.apkg'],
  node: ['package.json', 'package-lock.json', 'main.mjs', 'spanish.apkg'],
};
const files = [];
for (const [language, names] of Object.entries(kept)) for (const name of names) {
  const destination = path.join(evidence, 'basic', language, name);
  await mkdir(path.dirname(destination), { recursive: true });
  await cp(path.join(work, language, name), destination);
}
const sources = ['anki_forge/examples/target_api_basic.rs', 'bindings/python/examples/quickstart.py', 'bindings/node/examples/quickstart.mjs', 'scripts/packaged_consumer/src/inspection.rs', 'scripts/packaged_consumer/src/bin/verify_apkg.rs', 'docs/ai-quickstart.json', 'docs/troubleshooting.md', 'docs/zh-cn/troubleshooting.md', 'bindings/python/pyproject.toml', 'bindings/node/package.json', 'Cargo.toml', 'Cargo.lock'];
for (const source of sources) files.push({ path: source, sha256: await digest(path.join(root, source)) });
await writeFile(path.join(evidence, 'basic/execution-inputs.json'), JSON.stringify({ schemaVersion: 1, executedAt: new Date().toISOString(), publicVersion: version, workDirectory: work, runtime: { node: process.version, platform: process.platform, arch: process.arch }, inputs: files }, null, 2) + '\n');
console.log(`Executed public ${version} Rust, Python and Node basic tasks plus duplicate-key recovery; evidence in ${path.relative(root, evidence)}/basic`);
