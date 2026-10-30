// Independent machine-read reviewer: plain HTTP GET of artifact routes, saving exact bytes.
// Usage: node fetch.mjs <outDir> <purpose> <route> [<route>...]
// Each body is saved byte-exactly as NNN-<route>.body with NNN-<route>.http.json metadata,
// and the invocation is appended to <outDir>/session.json.
import { createHash } from 'node:crypto';
import { readFile, writeFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const [outDir, purpose, ...routes] = process.argv.slice(2);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const sessionFile = path.join(outDir, 'session.json');
const session = JSON.parse(await readFile(sessionFile));
const manifest = JSON.parse(await readFile(path.join('/Users/hp/Desktop/2026/anki-forge', session.originManifest.path)));
const records = new Map([...manifest.pages, ...manifest.resources].map(entry => [entry.route, entry]));
// Build-volatile or local-only resources are never read or bound by this review.
const excluded = /\/(?:generated\/[^/]+\.apkg|generated\/showcase\.json|pagefind\/pagefind-entry\.json|generated\/cell-diagram\.png)$/;
let next = (await readdir(outDir)).filter(name => /^\d{3}-.*\.body$/.test(name)).length + 1;
const invocation = { purpose, argv: process.argv.slice(2), fetchScript: 'fetch.mjs', fetchScriptSha256: sha(await readFile(fileURLToPath(import.meta.url))), startedAt: new Date().toISOString(), results: [] };
for (const route of routes) {
  if (excluded.test(route)) throw new Error(`Refusing excluded route ${route}`);
  const url = session.readOrigin + route;
  const response = await fetch(url, { redirect: 'manual', headers: { accept: 'text/markdown, text/plain;q=0.9, */*;q=0.1' } });
  const bytes = Buffer.from(await response.arrayBuffer());
  const observedAt = new Date().toISOString();
  const stem = `${String(next).padStart(3, '0')}-${route.replace(/^\//, '').replaceAll('/', '_')}`;
  next += 1;
  await writeFile(path.join(outDir, `${stem}.body`), bytes);
  const headers = Object.fromEntries([...response.headers.entries()].sort(([a], [b]) => a.localeCompare(b)));
  const type = headers['content-type'] || '';
  const disposition = headers['content-disposition'] || '';
  let utf8Decodes = true;
  try { new TextDecoder('utf-8', { fatal: true }).decode(bytes); } catch { utf8Decodes = false; }
  const record = records.get(route);
  const metadata = {
    url, finalUrl: response.url || url, method: 'GET', redirected: response.redirected, status: response.status, headers,
    sha256: sha(bytes), bytes: bytes.length, bodyFile: `${stem}.body`, observedAt,
    checks: {
      machineContentType: /^text\/(?:markdown|plain)\s*;/i.test(type) || /^text\/(?:markdown|plain)$/i.test(type),
      utf8Charset: /charset\s*=\s*utf-8/i.test(type),
      attachmentDisposition: /attachment/i.test(disposition),
      utf8Decodes,
      startsWithBom: bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf,
      manifestRoute: Boolean(record),
      manifestSha256: record?.sha256 ?? null,
      matchesManifest: record ? record.sha256 === sha(bytes) && record.bytes === bytes.length : false,
    },
  };
  await writeFile(path.join(outDir, `${stem}.http.json`), JSON.stringify(metadata, null, 2) + '\n');
  invocation.results.push({ route, url, status: metadata.status, contentType: type, sha256: metadata.sha256, bytes: metadata.bytes, bodyFile: metadata.bodyFile, metadataFile: `${stem}.http.json`, checks: metadata.checks });
  console.log(`${metadata.status} ${type} ${metadata.sha256} ${bytes.length}B match=${metadata.checks.matchesManifest} ${stem}`);
}
invocation.finishedAt = new Date().toISOString();
const current = JSON.parse(await readFile(sessionFile));
current.fetches.push(invocation);
await writeFile(sessionFile, JSON.stringify(current, null, 2) + '\n');
