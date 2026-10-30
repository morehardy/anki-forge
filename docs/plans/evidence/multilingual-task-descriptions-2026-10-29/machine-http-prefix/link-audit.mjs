// Independent machine-read reviewer: audit every link inside the Markdown/llms bodies
// actually received over HTTP (saved *.body files), against the archived origin manifest
// and the frozen artifact's HTML heading ids. Reads only local saved bytes; no network.
// Also records, per fragment link, whether the fragment's words appear as a heading in the
// target's Markdown (which carries no anchors), so an agent reading only Markdown can map it.
// Usage: node link-audit.mjs <outDir>
import { createHash } from 'node:crypto';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const [outDir] = process.argv.slice(2);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const session = JSON.parse(await readFile(path.join(outDir, 'session.json')));
const manifest = JSON.parse(await readFile(path.join('/Users/hp/Desktop/2026/anki-forge', session.originManifest.path)));
const { site, base } = manifest.configuration;
const prefix = base === '/' ? '' : base;
const pages = new Map(manifest.pages.map(page => [page.route, page]));
const resources = new Map(manifest.resources.map(resource => [resource.route, resource]));
const localeOf = route => route.startsWith(`${prefix}/zh-cn/`) ? 'zh-CN' : 'en';
const idCache = new Map();
async function ids(route) {
  if (!idCache.has(route)) {
    const html = await readFile(path.join(session.artifact, route.slice(prefix.length) + 'index.html'), 'utf8');
    idCache.set(route, new Set([...html.matchAll(/\sid="([^"]+)"/g)].map(match => match[1])));
  }
  return idCache.get(route);
}
const slug = text => text.toLowerCase().replace(/[`*_]/g, '').replace(/[^\p{L}\p{N}\s-]/gu, '').trim().replace(/\s+/g, '-');
async function markdownHeadingSlugs(route) {
  const file = path.join(session.artifact, route.slice(prefix.length) + 'index.md');
  const text = await readFile(file, 'utf8').catch(() => null);
  if (text === null) return null;
  return new Set([...text.matchAll(/^#{2,6}\s+(.+)$/gm)].map(match => slug(match[1])));
}
const report = { schemaVersion: 1, auditScript: 'link-audit.mjs', auditScriptSha256: sha(await readFile(fileURLToPath(import.meta.url))), site, base, generatedAt: new Date().toISOString(), documents: [] };
const seenRoutes = new Set();
for (const name of (await readdir(outDir)).filter(file => /^\d{3}-.*\.(?:md|txt)\.body$/.test(file)).sort()) {
  const meta = JSON.parse(await readFile(path.join(outDir, name.replace(/\.body$/, '.http.json'))));
  const route = new URL(meta.url).pathname;
  if (seenRoutes.has(route)) continue;
  seenRoutes.add(route);
  const text = await readFile(path.join(outDir, name), 'utf8');
  const ownLocale = localeOf(route);
  const links = [...text.matchAll(/\]\((https?:\/\/[^)\s]+)\)/g)].map(match => match[1]);
  const canonical = text.match(/^Canonical page: (\S+)$/m)?.[1] ?? null;
  const declaredLanguage = text.match(/^Language: (\S+)$/m)?.[1] ?? null;
  const document = { route, bodyFile: name, sha256: meta.sha256, declaredLanguage, canonical, canonicalOk: null, siteLinks: 0, externalLinks: 0, markdownTargets: 0, htmlTargets: 0, fragmentLinks: [], problems: [], crossLocale: [] };
  if (canonical) {
    const url = new URL(canonical);
    document.canonicalOk = url.origin === site && pages.has(url.pathname) && route === url.pathname + 'index.md';
    if (!document.canonicalOk) document.problems.push(`canonical ${canonical} does not match ${route}`);
  }
  for (const link of links) {
    const url = new URL(link);
    if (url.origin !== site) { document.externalLinks += 1; continue; }
    document.siteLinks += 1;
    if (!url.pathname.startsWith(prefix + '/')) { document.problems.push(`site link outside base: ${link}`); continue; }
    const target = url.pathname;
    let ok = false;
    if (target.endsWith('/')) { ok = pages.has(target); document.htmlTargets += 1; }
    else { ok = resources.has(target); if (/\/(?:index\.md|llms\.txt)$/.test(target)) document.markdownTargets += 1; }
    if (!ok) document.problems.push(`unresolved site link: ${link}`);
    if (ok && url.hash && target.endsWith('/')) {
      const fragment = decodeURIComponent(url.hash.slice(1));
      const htmlId = (await ids(target)).has(fragment);
      if (!htmlId) document.problems.push(`missing fragment: ${link}`);
      const headings = await markdownHeadingSlugs(target);
      document.fragmentLinks.push({ link, htmlId, markdownHeading: headings ? headings.has(fragment) : null });
    }
    if (!target.startsWith(`${prefix}/generated/`) && localeOf(target) !== ownLocale) document.crossLocale.push(link);
  }
  report.documents.push(document);
}
report.summary = {
  documents: report.documents.length,
  siteLinks: report.documents.reduce((sum, doc) => sum + doc.siteLinks, 0),
  markdownTargetsInsideDocuments: report.documents.filter(doc => !doc.route.endsWith('llms.txt')).reduce((sum, doc) => sum + doc.markdownTargets, 0),
  problems: report.documents.flatMap(doc => doc.problems.map(problem => `${doc.route}: ${problem}`)),
  crossLocale: report.documents.flatMap(doc => doc.crossLocale.map(link => `${doc.route}: ${link}`)),
  fragmentsWithoutMarkdownHeading: report.documents.flatMap(doc => doc.fragmentLinks.filter(item => item.markdownHeading === false).map(item => `${doc.route}: ${item.link}`)),
};
await writeFile(path.join(outDir, 'link-audit.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report.summary, null, 1));
for (const doc of report.documents) console.log(doc.route, doc.declaredLanguage, 'canonicalOk', doc.canonicalOk, 'site', doc.siteLinks, 'md', doc.markdownTargets, 'html', doc.htmlTargets, 'ext', doc.externalLinks);
