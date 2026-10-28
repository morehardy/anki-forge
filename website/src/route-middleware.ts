import { createTranslator } from './i18n/messages.mjs';
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';
import { pagePair, locales } from '../site-model.mjs';
import { siteConfig, withBase } from '../site.config.mjs';
import { repository } from '../site.config.mjs';
import { llmsUrl, markdownUrl } from './lib/machine-urls.mjs';
import { buildStructuredData, serializeJsonLd } from './lib/structured-data.mjs';
import publicInstall from './generated/ai-quickstart.json';

export const onRequest = defineRouteMiddleware(context => {
  const route = context.locals.starlightRoute;
  if (route.isFallback) throw new Error(`Untranslated documentation: ${route.id}`);
  const page = pagePair(context.url.pathname, siteConfig.base);
  const locale = route.lang === 'zh-CN' ? 'zh-CN' : 'en';
  const t = createTranslator(locale);
  if (!page) throw new Error(`Unregistered documentation: ${route.id}`);
  const metadata = { id: page.id, type: 'docs', locale, canonical: new URL(context.url.pathname, siteConfig.site).href, title: route.entry.data.title, description: route.entry.data.description || page.description?.[locale] || '' };
  const markdown = markdownUrl(metadata);
  if (!markdown || !metadata.description) throw new Error(`Missing documentation metadata: ${page.id}/${locale}`);
  const noindex = route.head.some(item => item.tag === 'meta' && item.attrs?.name === 'robots' && /\bnoindex\b/i.test(String(item.attrs?.content || '')));
  route.head = route.head.filter(item => !(item.tag === 'meta' && ['og:image', 'og:image:alt', 'og:locale'].includes(String(item.attrs?.property))));
  route.head.push(
    { tag: 'meta', attrs: { property: 'og:locale', content: locales[locale].og } },
    { tag: 'meta', attrs: { property: 'og:image', content: new URL(withBase(locale === 'en' ? '/generated/og.png' : '/generated/og-zh-cn.png', siteConfig.base), siteConfig.site).href } },
    { tag: 'meta', attrs: { property: 'og:image:alt', content: t('site.socialAlt') } },
    { tag: 'meta', attrs: { name: 'anki-forge:page-id', content: page.id } },
    { tag: 'meta', attrs: { name: 'anki-forge:content-source', content: page.sources[locale] } },
    { tag: 'link', attrs: { rel: 'alternate', type: 'application/rss+xml', title: t('blog.feedTitle'), href: withBase(`${locales[locale].prefix}/rss.xml`, siteConfig.base) } },
    { tag: 'link', attrs: { rel: 'describedby', href: llmsUrl(locale, siteConfig) } },
    { tag: 'link', attrs: { rel: 'alternate', type: 'text/markdown', href: markdown } },
  );
  if (!noindex) route.head.push({ tag: 'script', attrs: { type: 'application/ld+json', 'data-anki-forge-schema': '' }, content: serializeJsonLd(buildStructuredData(metadata, { siteConfig, repository, publicInstall, homeDescription: t('home.intro') })) });
});
