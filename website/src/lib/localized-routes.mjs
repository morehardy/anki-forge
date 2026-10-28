import { localeIds, locales } from '../../locale-model.mjs';

// Root English routes are authored once; every translated route uses this set.
export function translatedPagePaths() {
  return localeIds.filter(locale => locales[locale].prefix).map(locale => ({
    params: { locale: locales[locale].prefix.slice(1) }, props: { locale },
  }));
}
