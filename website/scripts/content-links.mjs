import path from 'node:path';
import { repository, withBase } from '../site.config.mjs';

export { importedDocs } from '../site-model.mjs';
import { pages, pagePath, resolvePage, localeIds, configuredLocales } from '../site-model.mjs';

function sourceLocale(source) {
  const entry = Object.entries(configuredLocales).find(([, locale]) => locale.prefix && source.startsWith(`docs${locale.prefix}/`));
  return entry?.[0] ?? 'en';
}

export function rewriteTarget(target, source, base, locale = sourceLocale(source)) {
  if (!localeIds.includes(locale)) throw new Error(`Unreleased content locale: ${locale}`);
  if (target.startsWith('https://ankiforge.dev/')) target = target.slice('https://ankiforge.dev'.length);
  if (target.startsWith('/') && !target.startsWith('//')) {
    const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
    const page = resolvePage(pathname)?.page;
    return `${withBase(page ? pagePath(page.id, locale) : pathname, base)}${suffix}`;
  }
  if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(target)) return target;
  const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(source), pathname));
  if (resolved.startsWith('docs/recipes/json-to-anki/')) return `${withBase('/generated/recipes/json-to-anki/', base)}${path.posix.basename(resolved)}${suffix}`;
  const page = pages.find(page => Object.values(page.sources).includes(resolved));
  if (page) {
    return `${withBase(pagePath(page.id, locale), base)}${suffix}`;
  }
  return `${repository}/blob/main/${resolved.split('/').map(encodeURIComponent).join('/')}${suffix}`;
}

export function rewriteMarkdown(markdown, source, base, locale = sourceLocale(source)) {
  // Source examples remain verbatim. Only prose links are made site-aware.
  return markdown.split(/(```[\s\S]*?```|~~~[\s\S]*?~~~)/g).map((part, i) => i % 2 ? part : part
    .replace(/(!?\[[^\]\n]*\]\()([^\s)]+)(\))/g, (_, start, target, end) => `${start}${rewriteTarget(target, source, base, locale)}${end}`)
    .replace(/^(\[[^\]\n]+\]:\s*)(\S+)/gm, (_, start, target) => `${start}${rewriteTarget(target, source, base, locale)}`)
  ).join('');
}

// Starlight supplies the page title; the crate README uses an HTML title on GitHub.
export function stripDocumentTitle(markdown) {
  const content = markdown.replace(/^# .+\r?\n+/, '');
  const boundary = content.search(/^```|^~~~|^## /m);
  const end = boundary < 0 ? content.length : boundary;
  return content.slice(0, end).replace(/<h1\b[^>]*>[\s\S]*?<\/h1>\s*/i, '') + content.slice(end);
}
