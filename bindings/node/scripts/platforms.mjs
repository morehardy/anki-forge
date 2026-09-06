import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const repo = path.resolve(root, '../..');
export const targets = [
  { suffix: 'darwin-arm64', target: 'aarch64-apple-darwin', os: 'darwin', cpu: 'arm64' },
  { suffix: 'darwin-x64', target: 'x86_64-apple-darwin', os: 'darwin', cpu: 'x64' },
  {
    suffix: 'linux-x64-gnu',
    target: 'x86_64-unknown-linux-gnu',
    os: 'linux',
    cpu: 'x64',
    libc: 'glibc',
  },
  { suffix: 'win32-x64-msvc', target: 'x86_64-pc-windows-msvc', os: 'win32', cpu: 'x64' },
];

export async function preparePlatforms() {
  const main = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
  for (const item of targets) {
    const directory = path.join(root, 'npm', item.suffix);
    await fs.mkdir(directory, { recursive: true });
    await fs.writeFile(
      path.join(directory, 'package.json'),
      JSON.stringify(
        {
          name: `${main.name}-${item.suffix}`,
          version: main.version,
          description: `Native runtime for ${main.name} (${item.target})`,
          license: main.license,
          repository: main.repository,
          engines: main.engines,
          os: [item.os],
          cpu: [item.cpu],
          ...(item.libc ? { libc: [item.libc] } : {}),
          main: 'anki-forge.node',
          files: ['anki-forge.node', 'README.md', 'LICENSE'],
        },
        null,
        2,
      ) + '\n',
    );
    await fs.writeFile(
      path.join(directory, 'README.md'),
      `# ${main.name}-${item.suffix}\n\nPlatform runtime installed automatically by \`${main.name}\`.\n`,
    );
    await fs.copyFile(path.join(repo, 'LICENSE'), path.join(directory, 'LICENSE'));
  }
  await fs.copyFile(path.join(repo, 'LICENSE'), path.join(root, 'LICENSE'));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await preparePlatforms();
