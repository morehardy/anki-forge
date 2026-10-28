import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { glob } from 'astro/loaders';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';
import { localeIds, locales } from '../locale-model.mjs';

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
  i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
  blog: defineCollection({
    loader: glob({ pattern: localeIds.map(locale => `${locales[locale].prefix ? locales[locale].prefix.slice(1) + '/' : ''}*.{md,mdx}`), base: './src/content/blog' }),
    schema: z.object({
      locale: z.enum(localeIds as [string, ...string[]]),
      translationKey: z.string(),
      title: z.string(),
      description: z.string(),
      date: z.coerce.date(),
      category: z.enum(['Tutorial', 'Engineering']),
      draft: z.boolean().default(false),
    }),
  }),
};
