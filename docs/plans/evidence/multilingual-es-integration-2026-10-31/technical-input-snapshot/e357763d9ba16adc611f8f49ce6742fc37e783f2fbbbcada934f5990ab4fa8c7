import release from './release-locales.json' with { type: 'json' };

// Configuration is deliberately separate from the set selected for a release.
export const configuredLocales = {
  en: { prefix: '', lang: 'en', og: 'en_US', label: 'English', format: 'en-US', starlightKey: 'root', dir: 'ltr' },
  'zh-CN': { prefix: '/zh-cn', lang: 'zh-CN', og: 'zh_CN', label: '简体中文', format: 'zh-CN', starlightKey: 'zh-cn', dir: 'ltr' },
  es: { prefix: '/es', lang: 'es', og: 'es_ES', label: 'Español', format: 'es', starlightKey: 'es', dir: 'ltr' },
  'pt-BR': { prefix: '/pt-br', lang: 'pt-BR', og: 'pt_BR', label: 'Português do Brasil', format: 'pt-BR', starlightKey: 'pt-br', dir: 'ltr' },
  ja: { prefix: '/ja', lang: 'ja', og: 'ja_JP', label: '日本語', format: 'ja-JP', starlightKey: 'ja', dir: 'ltr' },
  de: { prefix: '/de', lang: 'de', og: 'de_DE', label: 'Deutsch', format: 'de-DE', starlightKey: 'de', dir: 'ltr' },
  fr: { prefix: '/fr', lang: 'fr', og: 'fr_FR', label: 'Français', format: 'fr', starlightKey: 'fr', dir: 'ltr' },
  ru: { prefix: '/ru', lang: 'ru', og: 'ru_RU', label: 'Русский', format: 'ru-RU', starlightKey: 'ru', dir: 'ltr' },
};

export function validateReleaseLocales(snapshot, registry = configuredLocales, published = []) {
  if (snapshot?.schemaVersion !== 1 || !Array.isArray(snapshot.locales) || snapshot.locales[0] !== 'en') throw new Error('Release locales must use schemaVersion 1 and start with en');
  if (new Set(snapshot.locales).size !== snapshot.locales.length) throw new Error('Duplicate release locale');
  for (const id of snapshot.locales) if (!Object.hasOwn(registry, id)) throw new Error(`Unknown release locale: ${id}`);
  for (const id of published) if (!snapshot.locales.includes(id)) throw new Error(`Release cannot remove published locale: ${id}`);
  return Object.freeze([...snapshot.locales]);
}

export const releaseLocales = validateReleaseLocales(release);
export const localeIds = releaseLocales;
export const locales = Object.freeze(Object.fromEntries(releaseLocales.map(id => [id, configuredLocales[id]])));
export const starlightLocales = Object.fromEntries(releaseLocales.map(id => [configuredLocales[id].starlightKey, { label: configuredLocales[id].label, lang: id }]));

export function localePath(route, locale, registry = configuredLocales) {
  if (!Object.hasOwn(registry, locale)) throw new Error(`Unknown locale: ${locale}`);
  if (!route.startsWith('/') || route.startsWith('//')) throw new Error(`Expected site-relative route: ${route}`);
  return `${registry[locale].prefix}${route}`;
}

export function stripBase(pathname, base = '/') {
  const prefix = base.replace(/\/$/, '');
  if (!prefix) return pathname;
  if (pathname === prefix) return '/';
  return pathname.startsWith(`${prefix}/`) ? pathname.slice(prefix.length) : null;
}

export function resolveLocalePath(pathname, base = '/', active = releaseLocales, registry = configuredLocales) {
  const route = stripBase(pathname, base);
  if (route === null) return null;
  const translated = Object.entries(registry).find(([, locale]) => locale.prefix && (route === locale.prefix || route.startsWith(`${locale.prefix}/`)));
  if (translated) {
    const [locale, config] = translated;
    if (!active.includes(locale)) return null;
    return { locale, route: route.slice(config.prefix.length) || '/' };
  }
  return active.includes('en') ? { locale: 'en', route } : null;
}

export function socialImagePath(locale) {
  if (!Object.hasOwn(locales, locale)) throw new Error(`Unknown release locale: ${locale}`);
  return `/generated/og${configuredLocales[locale].prefix.replaceAll('/', '-')}.png`;
}
