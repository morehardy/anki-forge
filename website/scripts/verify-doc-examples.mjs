import { execFileSync } from 'node:child_process';
import { cp, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { importedDocs } from './content-links.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-docs-'));
const app = path.join(directory, 'consumer');
const target = path.join(root, 'target', 'docs-consumer');
// Share dependency artifacts, but never replace another running verifier's binary.
const consumerName = `docs_consumer_${process.pid}`;
const inspectorName = `verify_apkg_${process.pid}`;
const environment = { ...process.env, CARGO_TARGET_DIR: target };
const executable = name => path.join(target, 'debug', `${name}${process.platform === 'win32' ? '.exe' : ''}`);
const fixtures = path.join(root, 'scripts', 'packaged_consumer', 'fixtures');
let executions = 0;
let packages = 0;

async function workspace(name) {
  const work = path.join(directory, name);
  await mkdir(work, { recursive: true });
  await cp(fixtures, path.join(work, 'fixtures'), { recursive: true });
  // The crate README uses the shorter source-relative spelling.
  await cp(path.join(fixtures, 'pixel.png'), path.join(work, 'cell.png'));
  return work;
}
async function inspect(work) {
  execFileSync(executable(inspectorName), [work], { cwd: work, stdio: 'inherit' });
  packages += (await readdir(work)).filter(name => name.endsWith('.apkg')).length;
}
function cargo(args, cwd = app) {
  execFileSync('cargo', args, { cwd, stdio: 'inherit', env: environment });
}

try {
  await mkdir(path.join(app, 'src', 'bin'), { recursive: true });
  await writeFile(path.join(app, 'Cargo.toml'), `[package]
name = "${consumerName}"
version = "0.0.0"
edition = "2021"
rust-version = "1.92"
[workspace]
[dependencies]
ankiforge = { path = ${JSON.stringify(path.join(root, 'anki_forge'))}, default-features = false }
anyhow = "1"
serde_json = "1"
prost = "0.13"
rusqlite = { version = "0.32", features = ["bundled"] }
zip = { version = "2", default-features = false, features = ["deflate"] }
zstd = "0.13"
tempfile = "3"
`);
  await cp(path.join(root, 'scripts/packaged_consumer/src/inspection.rs'), path.join(app, 'src/inspection.rs'));
  await cp(path.join(root, 'scripts/packaged_consumer/src/bin/verify_apkg.rs'), path.join(app, `src/bin/${inspectorName}.rs`));
  await writeFile(path.join(app, 'src/main.rs'), 'fn main() {}\n');
  cargo(['generate-lockfile', '--offline']);
  cargo(['build', '--offline', '--locked', '--quiet', '--bin', inspectorName]);

  for (const example of ['target_api_basic', 'target_api_custom_notetype', 'target_api_media', 'docs_workflow']) {
    const work = await workspace(example);
    const args = ['run', '--offline', '--locked', '--quiet', '--manifest-path', path.join(root, 'Cargo.toml'), '-p', 'ankiforge', '--no-default-features', '--example', example];
    if (example === 'docs_workflow') args.push('--', work);
    cargo(args, work);
    await inspect(work);
    executions += 1;
  }

  const programs = new Map();
  const sources = [...new Set(['README.md', 'README.zh-CN.md', ...importedDocs.map(entry => entry.source)])];
  for (const filename of sources) {
    const markdown = await readFile(path.join(root, filename), 'utf8');
    for (const [, source] of markdown.matchAll(/```rust(?:,[^\n]+)?\n([\s\S]*?)\n```/g)) {
      if (/\bfn\s+main\s*\(/.test(source)) {
        const origins = programs.get(source) ?? [];
        origins.push(filename);
        programs.set(source, origins);
      }
    }
  }
  for (const [source, origins] of programs) {
    console.log(`Verifying complete Rust main from ${origins.join(', ')}`);
    await writeFile(path.join(app, 'src/main.rs'), source);
    cargo(['build', '--quiet', '--offline', '--locked', '--bin', consumerName]);
    const work = await workspace(`program-${executions}`);
    execFileSync(executable(consumerName), [], { cwd: work, stdio: 'inherit' });
    await inspect(work);
    console.log(`Executed complete Rust main from ${origins.join(', ')}`);
    executions += 1;
  }
  if (programs.size === 0 || packages === 0) throw new Error('No complete programs or APKG outputs were verified');
  console.log(`Verified ${executions} complete executions and ${packages} APKG outputs with real fixtures and independent SQLite/media observations.`);
} finally {
  await rm(directory, { recursive: true, force: true });
}
