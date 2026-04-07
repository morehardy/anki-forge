import fs from 'node:fs';
import path from 'node:path';

function readBundleVersion(manifestPath) {
  const raw = fs.readFileSync(manifestPath, 'utf8');
  const match = raw.match(/^bundle_version:\s*"([^"]+)"/m);
  return match ? match[1] : 'unknown';
}

export function resolveRuntime(options = {}) {
  if (options.mode === 'installed') {
    return {
      mode: 'installed',
      manifestPath: path.resolve(options.manifestPath),
      bundleRoot: path.resolve(options.bundleRoot),
      bundleVersion: readBundleVersion(path.resolve(options.manifestPath)),
      launcherExecutable: options.launcherExecutable ?? 'contract_tools',
      launcherPrefix: [...(options.launcherPrefix ?? [])],
    };
  }

  let current = path.resolve(options.cwd ?? process.cwd());
  while (true) {
    const manifestPath = path.join(current, 'contracts', 'manifest.yaml');
    if (fs.existsSync(manifestPath)) {
      return {
        mode: 'workspace',
        manifestPath,
        bundleRoot: path.dirname(manifestPath),
        bundleVersion: readBundleVersion(manifestPath),
        launcherExecutable:
          options.launcherExecutable ??
          process.env.ANKI_FORGE_CONTRACT_TOOLS ??
          'cargo',
        launcherPrefix: [...(options.launcherPrefix ?? ['run', '-q', '-p', 'contract_tools', '--'])],
      };
    }

    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error('failed to discover contracts/manifest.yaml from workspace path');
    }
    current = parent;
  }
}
