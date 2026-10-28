import { pages } from '../../site-model.mjs';
import { configuredLocales, localeIds, localePath } from '../../locale-model.mjs';
import { withBase } from '../../site-paths.mjs';

export function canonicalUrl(id, locale, config, buildLocales = localeIds) {
  const page = pages.find(page => page.id === id);
  if (!page || !buildLocales.includes(locale)) throw new Error(`Unknown or inactive page locale: ${id}/${locale}`);
  return new URL(withBase(localePath(page.route, locale), config.base), config.site).href;
}

export function markdownUrl(page) {
  if (!['docs', 'article'].includes(page.type)) return null;
  const url = new URL(page.canonical);
  url.search = '';
  url.hash = '';
  if (!url.pathname.endsWith('/')) throw new Error(`Canonical page must end with a slash: ${url.href}`);
  return new URL('index.md', url).href;
}

export function llmsPath(locale) {
  return localePath('/llms.txt', locale);
}

export function llmsUrl(locale, config) {
  return new URL(withBase(llmsPath(locale), config.base), config.site).href;
}

// These namespaces belong to the exporter, including configured languages that
// were generated for a candidate and then removed from the release selection.
export function machineResourceKind(relative, registry = configuredLocales) {
  const prefixes = Object.values(registry).map(locale => locale.prefix.replace(/^\//, '')).sort((a, b) => b.length - a.length);
  for (const prefix of prefixes) {
    if (prefix && !relative.startsWith(`${prefix}/`)) continue;
    const local = prefix ? relative.slice(prefix.length + 1) : relative;
    if (local === 'llms.txt') return 'index';
    if (/^(?:docs|blog)\/(?:[^/]+\/)*index\.md$/.test(local)) return 'markdown';
  }
  return null;
}

export function entityIds(page, config) {
  const root = new URL(withBase('/', config.base), config.site).href;
  const canonical = new URL(page.canonical);
  canonical.search = '';
  canonical.hash = '';
  return { website: `${root}#website`, software: `${root}#software`, source: `${root}#source`, webpage: `${canonical.href}#webpage`, article: `${canonical.href}#article` };
}
