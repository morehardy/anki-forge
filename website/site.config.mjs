import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const localEnvironment = new URL('./.env', import.meta.url);
if (existsSync(localEnvironment)) process.loadEnvFile(fileURLToPath(localEnvironment));

export const repository = 'https://github.com/morehardy/anki-forge';

/** Keep public links, Astro routing, feeds, and deployment on the same base. */
export function resolveSiteConfig(environment = process.env) {
  const site = new URL(environment.SITE_URL || 'https://morehardy.github.io');
  if (!['http:', 'https:'].includes(site.protocol) || site.username || site.password || site.search || site.hash || site.pathname !== '/') {
    throw new Error('SITE_URL must be an HTTP(S) origin, without a path, query, or credentials. Set the path with SITE_BASE.');
  }
  const rawBase = environment.SITE_BASE ?? '/anki-forge';
  if (!/^\/?(?:[A-Za-z0-9_-]+\/)*[A-Za-z0-9_-]*\/?$/.test(rawBase)) {
    throw new Error('SITE_BASE must be a local path such as /anki-forge or /.');
  }
  const base = `/${rawBase.split('/').filter(Boolean).join('/')}`;
  const customDomain = environment.SITE_CUSTOM_DOMAIN || '';
  if (customDomain && (customDomain !== site.hostname || base !== '/')) {
    throw new Error('SITE_CUSTOM_DOMAIN must match SITE_URL, and a custom domain requires SITE_BASE=/.');
  }
  return { site: site.origin, base, customDomain };
}

export function withBase(path, base) {
  if (!path.startsWith('/') || path.startsWith('//')) throw new Error('Expected a site-relative path.');
  return `${base === '/' ? '' : base}${path}`;
}

export const siteConfig = resolveSiteConfig();
