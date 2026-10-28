import en from './en.json' with { type: 'json' };
import zh from './zh-CN.json' with { type: 'json' };
import { messageParameters, translatorFor } from './format-message.mjs';
import { clientMessageKeys } from './client-messages.mjs';

export const catalogs = { en, 'zh-CN': zh };
function forms(message, locale, key) {
  if (typeof message === 'string' && message.trim()) return [message];
  if (!message || message.type !== 'plural' || typeof message.variable !== 'string' || !message.forms) throw new Error(`${locale}: empty or invalid message ${key}`);
  const categories = new Intl.PluralRules(locale).resolvedOptions().pluralCategories;
  for (const category of categories) if (typeof message.forms[category] !== 'string' || !message.forms[category].trim()) throw new Error(`${locale}: missing plural form ${key}/${category}`);
  for (const [category, text] of Object.entries(message.forms)) {
    if (!['zero', 'one', 'two', 'few', 'many', 'other'].includes(category) || typeof text !== 'string' || !text.trim()) throw new Error(`${locale}: invalid plural form ${key}/${category}`);
    if (!messageParameters(text).includes(message.variable)) throw new Error(`${locale}: plural count missing in ${key}/${category}`);
  }
  return Object.values(message.forms);
}

export function validateMessages(messages = catalogs) {
  const keys = Object.keys(messages.en).sort();
  for (const [locale, catalog] of Object.entries(messages)) {
    if (JSON.stringify(Object.keys(catalog).sort()) !== JSON.stringify(keys)) throw new Error(`${locale}: missing or extra message keys`);
    for (const key of keys) {
      const expected = messageParameters(forms(messages.en[key], 'en', key)[0]);
      for (const text of forms(catalog[key], locale, key)) if (JSON.stringify(messageParameters(text)) !== JSON.stringify(expected)) throw new Error(`${locale}: mismatched message parameters: ${key}`);
    }
  }
}

export function createTranslator(locale = 'en') {
  return translatorFor(catalogs[locale], locale);
}

export function createClientMessages(locale) {
  const catalog = catalogs[locale];
  if (!catalog) throw new Error(`Unknown message locale: ${locale}`);
  return Object.fromEntries(clientMessageKeys.map(key => [key, catalog[key]]));
}

export function formatCounts(locale, counts) {
  const t = createTranslator(locale);
  return Object.fromEntries(['notes', 'cards', 'media'].map(key => [key, t(`count.${key}`, { count: counts[key] })]));
}
