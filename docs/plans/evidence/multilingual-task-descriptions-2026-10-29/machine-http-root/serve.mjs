// Independent machine-read reviewer: serve one frozen artifact with the repository's
// own serveBuiltSite (base read from SITE_* env at import time) and record the session.
// Usage: SITE_URL=... SITE_BASE=... SITE_CUSTOM_DOMAIN=... node serve.mjs <artifactDir> <outDir> <originManifest>
// Before serving it re-hashes every manifest record in the artifact and checks the embedded manifest.
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const repo = '/Users/hp/Desktop/2026/anki-forge';
const [artifact, outDir, originManifest] = process.argv.slice(2).map(value => path.resolve(value));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const serverModule = `${repo}/website/scripts/serve-built-site.mjs`;
const { serveBuiltSite } = await import(serverModule);
const { siteConfig } = await import(`${repo}/website/site.config.mjs`);
const manifestBytes = await readFile(originManifest);
const manifest = JSON.parse(manifestBytes);
if (JSON.stringify(siteConfig) !== JSON.stringify(manifest.configuration)) throw new Error(`Server configuration ${JSON.stringify(siteConfig)} differs from manifest ${JSON.stringify(manifest.configuration)}`);
const embedded = sha(await readFile(path.join(artifact, 'generated/site-manifest.json')));
if (embedded !== sha(manifestBytes)) throw new Error('Artifact embedded manifest differs from archived origin manifest');
const prefix = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const mismatches = [];
let checked = 0;
for (const entry of [...manifest.pages, ...manifest.resources]) {
  let relative = entry.route.slice(prefix.length);
  if (relative.endsWith('/')) relative += 'index.html';
  const bytes = await readFile(path.join(artifact, '.' + relative)).catch(() => null);
  checked += 1;
  if (!bytes || sha(bytes) !== entry.sha256 || bytes.length !== entry.bytes) mismatches.push(entry.route);
}
if (mismatches.length) throw new Error(`Artifact differs from manifest: ${mismatches.join(', ')}`);
const site = await serveBuiltSite({ directory: artifact });
await mkdir(outDir, { recursive: true });
const sessionFile = path.join(outDir, 'session.json');
const session = {
  schemaVersion: 1,
  reviewer: 'independent AI machine-read reviewer (Claude Code subagent)',
  readOrigin: site.origin,
  configuration: siteConfig,
  environment: { SITE_URL: process.env.SITE_URL, SITE_BASE: process.env.SITE_BASE, SITE_CUSTOM_DOMAIN: process.env.SITE_CUSTOM_DOMAIN ?? null },
  artifact,
  artifactVerification: { manifestRecordsChecked: checked, mismatches: mismatches.length },
  originManifest: { path: path.relative(repo, originManifest), sha256: sha(manifestBytes), commit: manifest.commit, inputSha256: manifest.inputSha256, releaseLocales: manifest.releaseLocales },
  artifactEmbeddedManifestSha256: embedded,
  server: { module: path.relative(repo, serverModule), moduleSha256: sha(await readFile(serverModule)), wrapper: 'serve.mjs', wrapperSha256: sha(await readFile(fileURLToPath(import.meta.url))), node: process.version, pid: process.pid },
  startedAt: new Date().toISOString(),
  fetches: [],
};
await writeFile(sessionFile, JSON.stringify(session, null, 2) + '\n');
console.log(`READ_ORIGIN ${site.origin} checked=${checked}`);
const stop = async () => {
  const current = JSON.parse(await readFile(sessionFile));
  current.stoppedAt = new Date().toISOString();
  await writeFile(sessionFile, JSON.stringify(current, null, 2) + '\n');
  await site.close();
  process.exit(0);
};
process.on('SIGTERM', stop);
process.on('SIGINT', stop);
setInterval(() => {}, 1 << 30);
