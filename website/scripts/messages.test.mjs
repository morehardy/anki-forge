import test from 'node:test';
import assert from 'node:assert/strict';
import { catalogs, createTranslator, createClientMessages, formatCounts, validateMessages } from '../src/i18n/messages.mjs';
import { formatMessage } from '../src/i18n/format-message.mjs';
import { serializeClientMessages } from '../src/i18n/client-messages.mjs';

test('complete count phrases follow the locale plural system', () => {
  validateMessages();
  const en = createTranslator('en'), zh = createTranslator('zh-CN');
  for (const count of [0, 1, 2, 5, 21]) {
    assert.equal(en('count.notes', { count }), `${count} ${count === 1 ? 'note' : 'notes'}`);
    assert.equal(zh('count.cards', { count }), `${count} 张卡片`);
  }
  assert.equal(en('examples.counts', formatCounts('en', { notes: 1, cards: 2, media: 0 })), '1 note · 2 cards · 0 media files');
  assert.equal(zh('hero.counts', formatCounts('zh-CN', { notes: 1, cards: 2, media: 5 })), '2 张卡片 / 5 个媒体文件');
  assert.throws(() => formatMessage(catalogs.en['count.notes'], 'count.notes', { count: NaN }, 'en'), /finite number/);
});
test('different locale plural branches are valid and missing native categories fail', () => {
  const english = { count: { type: 'plural', variable: 'count', forms: { one: '{count} item', other: '{count} items' } } };
  const russian = { count: { type: 'plural', variable: 'count', forms: { one: '{count} элемент', few: '{count} элемента', many: '{count} элементов', other: '{count} элемента' } } };
  validateMessages({ en: english, ru: russian });
  assert.equal(formatMessage(russian.count, 'count', { count: 21 }, 'ru'), '21 элемент');
  assert.equal(formatMessage(russian.count, 'count', { count: 2 }, 'ru'), '2 элемента');
  assert.throws(() => validateMessages({ en: english, ru: english }), /missing plural form/);
});
test('browser messages only contain current locale runtime keys and serialize safely', () => {
  const messages = createClientMessages('zh-CN');
  assert.equal(messages['copy.success'], catalogs['zh-CN']['copy.success']);
  assert.ok(!('home.intro' in messages));
  const serialized = serializeClientMessages({ value: '</script>\u2028\u2029' });
  assert.ok(!serialized.includes('</script>'));
  assert.equal(JSON.parse(serialized).value, '</script>\u2028\u2029');
});
