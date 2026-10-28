import { pagePath } from '../../site-model.mjs';
import { withBase } from '../../site-paths.mjs';

export function canonicalUrl(id, locale, config) {
  return new URL(withBase(pagePath(id, locale), config.base), config.site).href;
}

export function markdownUrl(page) {
  if (!['docs', 'article'].includes(page.type)) return null;
  const url = new URL(page.canonical);
  url.search = '';
  url.hash = '';
  if (!url.pathname.endsWith('/')) throw new Error(`Canonical page must end with a slash: ${url.href}`);
  return new URL('index.md', url).href;
}

export function llmsUrl(locale, config) {
  if (!['en', 'zh-CN'].includes(locale)) throw new Error(`Unknown locale: ${locale}`);
  return new URL(withBase(locale === 'en' ? '/llms.txt' : '/zh-cn/llms.txt', config.base), config.site).href;
}

export function entityIds(page, config) {
  const root = new URL(withBase('/', config.base), config.site).href;
  const canonical = new URL(page.canonical);
  canonical.search = '';
  canonical.hash = '';
  return { website: `${root}#website`, software: `${root}#software`, source: `${root}#source`, webpage: `${canonical.href}#webpage`, article: `${canonical.href}#article` };
}
