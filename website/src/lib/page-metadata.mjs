import { configuredLocales, localeIds } from '../../locale-model.mjs';
import { withBase } from '../../site-paths.mjs';
import { canonicalUrl, llmsUrl, markdownUrl } from './machine-urls.mjs';

/** One descriptor serves ordinary Astro pages and Starlight's document adapter. */
export function createPageMetadata(page, locale, config, overrides = {}, buildLocales = localeIds) {
  if (!page || !buildLocales.includes(locale) || !configuredLocales[locale]) throw new Error(`Unregistered page locale: ${page?.id}/${locale}`);
  const metadata = {
    id: page.id, type: page.type, locale,
    canonical: canonicalUrl(page.id, locale, config, buildLocales),
    title: overrides.title ?? page.title?.[locale],
    description: overrides.description ?? page.description?.[locale],
    ...(overrides.publishedAt ? { publishedAt: overrides.publishedAt } : {}),
    ...(overrides.category ? { category: overrides.category } : {}),
  };
  if (!metadata.title?.trim() || !metadata.description?.trim()) throw new Error(`Missing page metadata: ${page.id}/${locale}`);
  return metadata;
}

/** Language alternatives, machine representations and RSS have distinct keys. */
export function metadataLinks(metadata, config, buildLocales = localeIds) {
  if (!buildLocales.includes(metadata.locale) || !buildLocales.includes('en')) throw new Error('Metadata language set must include its page locale and English');
  const markdown = markdownUrl(metadata);
  return [
    { rel: 'canonical', href: metadata.canonical },
    ...buildLocales.map(locale => ({ rel: 'alternate', hreflang: locale, href: canonicalUrl(metadata.id, locale, config, buildLocales) })),
    { rel: 'alternate', hreflang: 'x-default', href: canonicalUrl(metadata.id, 'en', config, buildLocales) },
    { rel: 'describedby', href: llmsUrl(metadata.locale, config) },
    ...(markdown ? [{ rel: 'alternate', type: 'text/markdown', href: markdown }] : []),
  ];
}

export function socialImageUrl(locale, config) {
  const prefix = configuredLocales[locale]?.prefix;
  if (prefix === undefined) throw new Error(`Unknown locale: ${locale}`);
  return new URL(withBase(`/generated/og${prefix ? `-${prefix.slice(1)}` : ''}.png`, config.base), config.site).href;
}

// Replace only the metadata owned by this adapter; never remove an unrelated
// alternate such as a feed when adding a language or Markdown representation.
export function mergeMetadataHead(head, additions) {
  const key = item => {
    const attrs = item.attrs || {};
    if (item.tag === 'link') return `link:${attrs.rel}:${attrs.hreflang || attrs.type || ''}`;
    if (item.tag === 'meta') return `meta:${attrs.property || attrs.name}`;
    if (item.tag === 'script' && Object.hasOwn(attrs, 'data-anki-forge-schema')) return 'managed-schema';
    if (item.tag === 'script' && Object.hasOwn(attrs, 'data-anki-forge-messages')) return 'client-messages';
    return null;
  };
  const replacing = new Set(additions.map(key).filter(Boolean));
  const replacesLanguages = additions.some(item => item.tag === 'link' && item.attrs?.hreflang);
  return [...head.filter(item => !(replacesLanguages && item.tag === 'link' && item.attrs?.hreflang) && !replacing.has(key(item))), ...additions];
}
