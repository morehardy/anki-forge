import fs from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { root, targets } from './platforms.mjs';

const all = process.argv.includes('--all');
const check = spawnSync(process.execPath, ['scripts/check-package.mjs', ...(all ? ['--all'] : [])], { cwd: root, stdio: 'inherit' });
if (check.error) throw check.error;
if (check.status !== 0) process.exit(check.status ?? 1);
const manifest = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
const npmCli = process.env.npm_execpath ?? path.resolve(path.dirname(process.execPath), process.platform === 'win32' ? 'node_modules/npm/bin/npm-cli.js' : '../lib/node_modules/npm/bin/npm-cli.js');
const selected = all ? targets : targets.filter(target => target.os === process.platform && target.cpu === process.arch);
const output = path.join(root, 'artifacts', manifest.version, all ? 'all' : selected[0].suffix);
await fs.mkdir(output, { recursive: true });
const packed = [];
for (const directory of [...selected.map(target => path.join(root, 'npm', target.suffix)), root]) {
  const result = spawnSync(process.execPath, [npmCli, 'pack', '--json', '--ignore-scripts', '--cache', path.join(root, 'artifacts/cache'), '--pack-destination', output], { cwd: directory, encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || result.stdout);
  packed.push(...JSON.parse(result.stdout));
}
await fs.writeFile(path.join(output, 'manifest.json'), JSON.stringify({ version: manifest.version, targets: selected.map(target => target.target), packages: packed }, null, 2) + '\n');
console.log(`Reviewable tarballs and integrity manifest: ${output}`);
