import rss from '@astrojs/rss';
import { getCollection } from 'astro:content';
import { sitePath } from '../lib/site';
import type { APIContext } from 'astro';

export async function GET(context: APIContext) {
  const posts = (await getCollection('blog', ({ data }) => !data.draft)).sort((a, b) => b.data.date.valueOf() - a.data.date.valueOf());
  return rss({
    title: 'Anki Forge Blog',
    description: 'Practical guides and engineering decisions behind Anki Forge.',
    site: new URL(sitePath('/'), context.site),
    items: posts.map(post => ({ title: post.data.title, description: post.data.description, pubDate: post.data.date, link: sitePath(`/blog/${post.id}/`) })),
    customData: '<language>en</language>',
  });
}
