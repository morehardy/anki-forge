import { pagePath } from '../../site-model.mjs';
/** @typedef {{label: string, runtime: string, install: string, quickstart: string}} Language */
/** @typedef {{publicVersion: string, languages: Record<string, Language>, prompts: Record<string, string>, requestPlaceholders: Record<string, string>}} AiQuickstart */

/**
 * Render once so README, server previews, and clipboard text share a contract.
 * Replacement values are never interpreted as markup or template syntax.
 * @param {AiQuickstart} data
 * @param {string} language
 * @param {string} [request]
 * @param {string} [locale]
 */
export function renderPrompt(data, language, request = '', locale = 'en') {
  const selected = data.languages[language];
  const template = data.prompts[locale];
  const placeholder = data.requestPlaceholders?.[locale];
  if (!selected || !template || !placeholder) throw new Error('Unknown AI quickstart language or locale.');
  const values = {
    version: data.publicVersion,
    language: selected.label,
    runtime: selected.runtime,
    install: selected.install,
    quickstart: new URL(pagePath(language === 'rust' ? 'quickstart' : `${language}-quickstart`, locale), selected.quickstart).href,
    request: request.trim() ? request : placeholder,
  };
  return template.replace(/\{(version|language|runtime|install|quickstart|request)\}/g,
    (_, key) => values[/** @type {keyof typeof values} */ (key)]);
}
