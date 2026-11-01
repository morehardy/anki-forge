export function messageParameters(message) {
  if (typeof message !== 'string') throw new Error('A message form must be a string');
  return [...new Set([...message.matchAll(/\{(\w+)\}|\[([A-Z_]+)\]/g)].map(match => match[1] ?? match[2]))].sort();
}

export function formatMessage(message, key, values = {}, locale = 'en') {
  let text = message;
  if (typeof message !== 'string') {
    if (!message || message.type !== 'plural' || !message.forms) throw new Error(`${key}: invalid message`);
    const count = values[message.variable];
    if (typeof count !== 'number' || !Number.isFinite(count)) throw new Error(`${key}: ${message.variable} must be a finite number`);
    const category = new Intl.PluralRules(locale).select(count);
    text = message.forms[category] ?? message.forms.other;
    if (typeof text !== 'string') throw new Error(`${key}: missing plural category ${category}`);
    values = { ...values, [message.variable]: new Intl.NumberFormat(locale).format(count) };
  }
  return text.replace(/\{(\w+)\}/g, (_, parameter) => {
    if (!(parameter in values)) throw new Error(`${key}: missing parameter ${parameter}`);
    return String(values[parameter]);
  });
}

export function translatorFor(catalog, locale) {
  if (!catalog || typeof catalog !== 'object') throw new Error(`Unknown message locale: ${locale}`);
  return (key, values = {}) => {
    if (!(key in catalog)) throw new Error(`${locale}: missing message ${key}`);
    return formatMessage(catalog[key], key, values, locale);
  };
}
