import test from 'node:test';
import assert from 'node:assert/strict';
import { load } from 'cheerio';
import { assertIndexable } from './indexability.mjs';

test('normal rendered pages and unrestricted actual headers remain indexable', () => {
  assert.doesNotThrow(() => assertIndexable(load('<main>Install the public SDK.</main>'), { headers: new Headers({ 'X-Robots-Tag': 'max-snippet:-1' }) }));
});
for (const directive of ['noindex', 'none', 'nosnippet', 'max-snippet:0', 'googlebot: max-snippet: 0']) {
  test(`indexability rejects ${directive} from HTML and real HTTP headers`, () => {
    assert.throws(() => assertIndexable(load(`<meta name="googlebot" content="${directive}"><main>SDK</main>`)), /restriction/);
    assert.throws(() => assertIndexable(load('<main>SDK</main>'), { headers: new Headers({ 'X-Robots-Tag': directive }) }), /X-Robots-Tag/);
  });
}
test('excluding the instructions from snippets fails independently of robots', () => {
  assert.throws(() => assertIndexable(load('<main><section data-nosnippet>Required original APKG baseline.</section></main>')), /data-nosnippet/);
});
test('a snippet exclusion on an ancestor cannot hide the entire main body', () => {
  assert.throws(() => assertIndexable(load('<div data-nosnippet><main>Required original APKG baseline.</main></div>')), /data-nosnippet/);
  assert.doesNotThrow(() => assertIndexable(load('<div data-nosnippet>Unrelated navigation</div><main>Required original APKG baseline.</main>')));
});
