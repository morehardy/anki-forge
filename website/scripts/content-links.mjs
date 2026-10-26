import path from 'node:path';
import { translatedDocs } from './core-pages.mjs';
import { repository, withBase } from '../site.config.mjs';

export { importedDocs } from '../site-model.mjs';
import { importedDocs, pages } from '../site-model.mjs';

export function rewriteTarget(target, source, base) {
  const locale = source.startsWith('docs/zh-cn/') ? 'zh-CN' : 'en';
  if (target.startsWith('https://ankiforge.dev/')) target = target.slice('https://ankiforge.dev'.length);
  if (target.startsWith('/') && !target.startsWith('//')) {
    const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
    const page = pages.find(page => page.en === pathname || page.zh === pathname);
    return `${withBase(page ? (locale === 'zh-CN' ? page.zh : page.en) : pathname, base)}${suffix}`;
  }
  if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(target)) return target;
  const [, pathname, suffix] = target.match(/^([^?#]*)(.*)$/);
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(source), pathname));
  if (resolved.startsWith('docs/recipes/json-to-anki/')) return `${withBase('/generated/recipes/json-to-anki/', base)}${path.posix.basename(resolved)}${suffix}`;
  const doc = [...importedDocs, ...translatedDocs].find(entry => entry.source === resolved);
  if (doc) {
    const page = pages.find(page => Object.values(page.sources).includes(doc.source));
    const route = source.startsWith('docs/zh-cn/') && page ? page.zh : doc.route;
    return `${withBase(route, base)}${suffix}`;
  }
  return `${repository}/blob/main/${resolved.split('/').map(encodeURIComponent).join('/')}${suffix}`;
}

export function rewriteMarkdown(markdown, source, base) {
  // Source examples remain verbatim. Only prose links are made site-aware.
  return markdown.split(/(```[\s\S]*?```|~~~[\s\S]*?~~~)/g).map((part, i) => i % 2 ? part : part
    .replace(/(!?\[[^\]\n]*\]\()([^\s)]+)(\))/g, (_, start, target, end) => `${start}${rewriteTarget(target, source, base)}${end}`)
    .replace(/^(\[[^\]\n]+\]:\s*)(\S+)/gm, (_, start, target) => `${start}${rewriteTarget(target, source, base)}`)
  ).join('');
}

// Starlight supplies the page title; the crate README uses an HTML title on GitHub.
export function stripDocumentTitle(markdown) {
  const content = markdown.replace(/^# .+\r?\n+/, '');
  const boundary = content.search(/^```|^~~~|^## /m);
  const end = boundary < 0 ? content.length : boundary;
  return content.slice(0, end).replace(/<h1\b[^>]*>[\s\S]*?<\/h1>\s*/i, '') + content.slice(end);
}
