import fs from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { root, repo, targets, preparePlatforms } from './platforms.mjs';

const targetArg = process.argv.indexOf('--target');
const requested = targetArg < 0 ? undefined : process.argv[targetArg + 1];
const platform = requested
  ? targets.find((item) => item.target === requested)
  : targets.find((item) => item.os === process.platform && item.cpu === process.arch);
if (!platform) throw new Error('Unsupported target; pass --target with a documented Rust target');
const release = process.argv.includes('--release');
function run(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run(
  'cargo',
  [
    'build',
    '-p',
    'anki_forge_node_native',
    '--locked',
    ...(requested ? ['--target', requested] : []),
    ...(release ? ['--release'] : []),
  ],
  repo,
);
await preparePlatforms();
const library =
  platform.os === 'win32'
    ? 'anki_forge_node_native.dll'
    : `libanki_forge_node_native.${platform.os === 'darwin' ? 'dylib' : 'so'}`;
const targetDir = process.env.CARGO_TARGET_DIR
  ? path.resolve(repo, process.env.CARGO_TARGET_DIR)
  : path.join(repo, 'target');
await fs.copyFile(
  path.join(targetDir, ...(requested ? [requested] : []), release ? 'release' : 'debug', library),
  path.join(root, 'npm', platform.suffix, 'anki-forge.node'),
);
run(process.execPath, ['toolchain/node_modules/typescript/bin/tsc', '-p', 'tsconfig.json']);
run(process.execPath, ['scripts/entries.mjs']);
