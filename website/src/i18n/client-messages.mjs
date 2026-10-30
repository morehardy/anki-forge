// This module is safe to import in a browser: it never imports a locale catalog.
export const clientMessageKeys = [
  'card.showQuestion', 'card.showAnswer', 'card.answer', 'card.questionShown',
  'update.reset', 'update.change', 'update.changed', 'update.try',
  'copy.codeSuccess', 'copy.codeFallback', 'copy.success', 'copy.copied', 'copy.fallback',
];

export function serializeClientMessages(messages) {
  return JSON.stringify(messages).replace(/</g, '\\u003c').replace(/\u2028/g, '\\u2028').replace(/\u2029/g, '\\u2029');
}
