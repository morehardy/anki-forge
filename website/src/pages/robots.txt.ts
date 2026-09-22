import type { APIContext } from 'astro';
import { sitePath } from '../lib/site';

export function GET(context: APIContext) {
  const sitemap = new URL(sitePath('/sitemap-index.xml'), context.site);
  return new Response(`User-agent: *\nAllow: /\n\nSitemap: ${sitemap}\n`, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
}
