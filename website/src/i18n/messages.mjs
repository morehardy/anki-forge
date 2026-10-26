import en from './en.json' with { type: 'json' };
import zh from './zh-CN.json' with { type: 'json' };

export const catalogs = { en, 'zh-CN': zh };
const parameters = text => [...text.matchAll(/\{(\w+)\}|\[([A-Z_]+)\]/g)].map(match => match[1] ?? match[2]).sort();

export function validateMessages(messages = catalogs) {
  const keys = Object.keys(messages.en).sort();
  for (const [locale, catalog] of Object.entries(messages)) {
    if (JSON.stringify(Object.keys(catalog).sort()) !== JSON.stringify(keys)) throw new Error(`${locale}: missing or extra message keys`);
    for (const key of keys) {
      if (typeof catalog[key] !== 'string' || !catalog[key].trim()) throw new Error(`${locale}: empty message ${key}`);
      if (JSON.stringify(parameters(catalog[key])) !== JSON.stringify(parameters(messages.en[key]))) throw new Error(`${locale}: mismatched message parameters: ${key}`);
    }
  }
}

export function createTranslator(locale = 'en') {
  const catalog = catalogs[locale];
  if (!catalog) throw new Error(`Unknown message locale: ${locale}`);
  return (key, values = {}) => {
    const message = catalog[key];
    if (!message) throw new Error(`${locale}: missing message ${key}`);
    return message.replace(/\{(\w+)\}/g, (_, parameter) => {
      if (!(parameter in values)) throw new Error(`${key}: missing parameter ${parameter}`);
      return String(values[parameter]);
    });
  };
}
