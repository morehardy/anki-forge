import { defineRouteMiddleware } from '@astrojs/starlight/route-data';
import { pagePair } from '../scripts/core-pages.mjs';
import { siteConfig, withBase } from '../site.config.mjs';

export const onRequest = defineRouteMiddleware((context) => {
  const pair = pagePair(context.url.pathname, siteConfig.base);
  if (pair) {
    for (const [hreflang, path] of [['en', pair.en], ['zh-CN', pair.zh]]) {
      context.locals.starlightRoute.head.push({ tag: 'link', attrs: {
        rel: 'alternate', hreflang, href: new URL(withBase(path, siteConfig.base), siteConfig.site).href,
      } });
    }
  }
});
