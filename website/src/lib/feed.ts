import rss from '@astrojs/rss';
import { getCollection } from 'astro:content';
import { sitePath } from './site';
import { locales, pagePath } from '../../site-model.mjs';
import { createTranslator } from '../i18n/messages.mjs';
import type { APIContext } from 'astro';

export async function feed(context: APIContext, locale: 'en' | 'zh-CN') {
  const t = createTranslator(locale);
  const posts = (await getCollection('blog', ({ data }) => !data.draft && data.locale === locale)).sort((a, b) => b.data.date.valueOf() - a.data.date.valueOf());
  return rss({
    title: t('blog.feedTitle'), description: t('blog.description'),
    site: new URL(sitePath(`${locales[locale].prefix}/`), context.site),
    items: posts.map(post => ({ title: post.data.title, description: post.data.description, pubDate: post.data.date, link: sitePath(pagePath(`blog:${post.data.translationKey}`, locale)) })),
    customData: `<language>${locale}</language>`,
  });
}
