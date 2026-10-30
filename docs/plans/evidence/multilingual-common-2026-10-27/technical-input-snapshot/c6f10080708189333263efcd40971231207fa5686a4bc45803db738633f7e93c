import vocabulary from '../../search-terms.json' with { type: 'json' };
import { configuredLocales } from '../../locale-model.mjs';

export const searchEnginePath = '/generated/search-engine.js';
export function searchIndexPath(locale) {
  if (!Object.hasOwn(configuredLocales, locale)) throw new Error(`Unknown search locale: ${locale}`);
  return `/generated/search-${locale.toLowerCase()}.json`;
}

/** Search concepts describe the page; they never rewrite visible copy or queries. */
export function searchTermsFor(page, locale, collection = vocabulary) {
  if (!['docs', 'article'].includes(page.type)) return '';
  const terms = collection.locales?.[locale]?.[page.id];
  if (!Array.isArray(terms) || !terms.length || terms.some(term => typeof term !== 'string' || !term.trim())) throw new Error(`Missing local search vocabulary: ${page.id}/${locale}`);
  return terms.join(' ');
}
