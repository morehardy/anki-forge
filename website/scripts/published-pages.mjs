import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import yaml from 'js-yaml';
import { pages, localeIds } from '../site-model.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
export function frontmatter(source) {
  const match = source.match(/^---\s*\n([\s\S]*?)\n---(?:\n|$)/);
  return match ? (yaml.load(match[1]) || {}) : {};
}

// The same frontmatter consumed by Astro determines whether a registered topic is public.
export async function publishedPages(directory = root, registry = pages, buildLocales = localeIds) {
  if (!buildLocales.length || new Set(buildLocales).size !== buildLocales.length) throw new Error('Build locales must be nonempty and unique');
  const published = [];
  for (const page of registry) {
    if (!['docs', 'article'].includes(page.type)) { published.push(page); continue; }
    const draft = [];
    for (const locale of buildLocales) {
      if (!page.sources?.[locale]) throw new Error(`${page.id}/${locale}: missing published source`);
      const source = await readFile(path.join(directory, page.sources[locale]), 'utf8');
      const data = frontmatter(source);
      draft.push(data?.draft === true);
    }
    if (draft.some(value => value !== draft[0])) throw new Error(`${page.id}: draft status must match for all build languages`);
    if (!draft[0]) published.push(page);
  }
  return published;
}
