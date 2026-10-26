import { createTranslator } from './i18n/messages.mjs';
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';
import { pagePair, locales } from '../site-model.mjs';
import { siteConfig, withBase } from '../site.config.mjs';

export const onRequest = defineRouteMiddleware(context => {
  const route = context.locals.starlightRoute;
  if (route.isFallback) throw new Error(`Untranslated documentation: ${route.id}`);
  const page = pagePair(context.url.pathname, siteConfig.base);
  const locale = route.lang === 'zh-CN' ? 'zh-CN' : 'en';
  const t = createTranslator(locale);
  if (!page) throw new Error(`Unregistered documentation: ${route.id}`);
  route.head = route.head.filter(item => !(item.tag === 'meta' && ['og:image', 'og:image:alt', 'og:locale'].includes(String(item.attrs?.property))));
  route.head.push(
    { tag: 'meta', attrs: { property: 'og:locale', content: locales[locale].og } },
    { tag: 'meta', attrs: { property: 'og:image', content: new URL(withBase(locale === 'en' ? '/generated/og.png' : '/generated/og-zh-cn.png', siteConfig.base), siteConfig.site).href } },
    { tag: 'meta', attrs: { property: 'og:image:alt', content: t('site.socialAlt') } },
    { tag: 'meta', attrs: { name: 'anki-forge:page-id', content: page.id } },
    { tag: 'meta', attrs: { name: 'anki-forge:content-source', content: page.sources[locale] } },
    { tag: 'link', attrs: { rel: 'alternate', type: 'application/rss+xml', title: t('blog.feedTitle'), href: withBase(`${locales[locale].prefix}/rss.xml`, siteConfig.base) } },
  );
});
