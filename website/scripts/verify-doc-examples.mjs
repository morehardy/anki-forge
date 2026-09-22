import { execFileSync } from 'node:child_process';
import { mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { importedDocs } from './content-links.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-docs-'));
try {
  // Run examples outside the checkout, so relative outputs and bundled resources
  // are exercised without overwriting the user's files.
  for (const example of ['target_api_basic', 'target_api_custom_notetype']) {
    execFileSync('cargo', ['run', '--locked', '--quiet', '--manifest-path', path.join(root, 'Cargo.toml'), '-p', 'anki_forge', '--example', example], { cwd: directory, stdio: 'inherit' });
  }
  for (const name of ['spanish.apkg', 'jp-core.apkg']) {
    if ((await stat(path.join(directory, name))).size === 0) throw new Error(`Empty example output: ${name}`);
  }
  // Exercise the documented path-dependency installation in an independent app.
  execFileSync('cargo', ['new', '--quiet', '--vcs', 'none', '--name', 'docs-consumer', 'consumer'], { cwd: directory });
  const app = path.join(directory, 'consumer');
  execFileSync('cargo', ['add', '--quiet', '--offline', 'anki_forge', '--path', path.join(root, 'anki_forge')], { cwd: app, stdio: 'inherit' });
  execFileSync('cargo', ['add', '--quiet', 'anyhow', '--offline'], { cwd: app, stdio: 'inherit' });
  const programs = new Map();
  for (const entry of importedDocs) {
    const markdown = await readFile(path.join(root, entry.source), 'utf8');
    for (const [, source] of markdown.matchAll(/```rust(?:,[^\n]+)?\n([\s\S]*?)\n```/g)) {
      if (source.includes('fn main()')) programs.set(source, entry.source);
    }
  }
  for (const [source, filename] of programs) {
    await writeFile(path.join(app, 'src/main.rs'), source);
    execFileSync('cargo', ['run', '--quiet', '--offline'], {
      cwd: app, stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: path.join(root, 'target') },
    });
    console.log(`Executed complete Rust program from ${filename}`);
  }
  await stat(path.join(app, 'spanish.apkg'));
  console.log('Verified complete Rust examples and a separate path-dependency application.');
} finally {
  await rm(directory, { recursive: true, force: true });
}
