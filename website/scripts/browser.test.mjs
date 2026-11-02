import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
import { serveBuiltSite } from './serve-built-site.mjs';
import { siteConfig } from '../site.config.mjs';
import { pagePath, resolvePage, localeIds } from '../site-model.mjs';
import { locales, localePath, configuredLocales } from '../locale-model.mjs';
import searchQueries from '../search-queries.json' with { type: 'json' };
import { publishedPages } from './published-pages.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';

const publicTopics = await publishedPages();
const hasTopic = id => publicTopics.some(page => page.id === id);
let server, browser;
before(async () => {
  server = process.env.BROWSER_SITE ? { origin: process.env.BROWSER_SITE, url: route => new URL(route, process.env.BROWSER_SITE).href, close: async () => {} } : await serveBuiltSite();
  browser = await chromium.launch({ executablePath: process.env.CHROME_EXECUTABLE || undefined, headless: true });
});
after(async () => { await browser?.close(); await server?.close(); });
async function session(t, options = {}) {
  const context = await browser.newContext({ reducedMotion: 'reduce', ...options });
  context.setDefaultTimeout(10_000);
  context.setDefaultNavigationTimeout(20_000);
  // Keep tests local. Clipboard and Pagefind still use real browser APIs.
  await context.route('https://static.cloudflareinsights.com/**', route => route.abort());
  t.after(() => context.close());
  return { context, page: await context.newPage() };
}
const visible = locator => locator.waitFor({ state: 'visible' });
async function languageSwitch(page, targetLocale, { keyboard = false } = {}) {
  const header = page.locator('.site-header');
  const currentLocale = await page.locator('html').getAttribute('lang');
  const links = header.locator('[data-language-switch]');
  assert.deepEqual((await links.evaluateAll(anchors => anchors.map(anchor => anchor.hreflang))).sort(), localeIds.filter(id => id !== currentLocale).sort(), 'Language choices must be the other release languages');
  const menu = header.locator('details.language-menu');
  assert.equal(await menu.count(), localeIds.length > 2 ? 1 : 0, 'Three or more languages must use the language menu');
  const link = header.locator(`[data-language-switch][hreflang="${targetLocale}"]`);
  assert.equal(await link.count(), 1, `Missing explicit language target ${targetLocale}`);
  assert.equal(await link.getAttribute('lang'), targetLocale);
  assert.equal((await link.textContent()).trim(), locales[targetLocale].label);
  if (await menu.count()) {
    const summary = menu.locator('summary');
    if (!await menu.evaluate(element => element.open)) {
      if (keyboard) { await summary.focus(); await page.keyboard.press('Enter'); }
      else await summary.click();
    }
    if (keyboard) {
      await summary.focus();
      for (let step = 0; step < localeIds.length; step++) {
        await page.keyboard.press('Tab');
        if (await link.evaluate(element => element === document.activeElement)) break;
      }
      assert.equal(await link.evaluate(element => element === document.activeElement), true, 'Target language must be reachable by Tab from the menu summary');
    }
  } else if (keyboard) await link.focus();
  await visible(link);
  return link;
}
const follow = async (page, link) => { await link.click(); await page.waitForLoadState('domcontentloaded'); };
const eventually = async (operation, message) => {
  for (let attempt = 0; attempt < 60; attempt++) {
    if (await operation()) return;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.fail(message);
};

test('all built content URLs and assets are reachable over HTTP', async () => {
  for (const topic of publicTopics) for (const locale of localeIds) {
    const response = await fetch(server.url(pagePath(topic.id, locale)));
    assert.equal(response.status, 200);
    assert.match(await response.text(), new RegExp(`<html[^>]+lang="${locale}"`));
  }
  for (const route of localeIds.map(locale => localePath('/missing-topic-62/', locale))) assert.equal((await fetch(server.url(route))).status, 404);
});

test('localized search retains native word-form matches beyond the frozen query set', async t => {
  const { page } = await session(t);
  for (const [locale, query, expected] of [['es', 'reutilización', 'templates'], ['de', 'Nebenläufig', 'python-api'], ['de', 'Zeichenfolge', 'python-api']]) {
    if (!localeIds.includes(locale) || !hasTopic(expected)) continue;
    await page.goto(server.url(pagePath('quickstart', locale)));
    await page.keyboard.press('Control+k');
    await visible(page.locator('site-search dialog'));
    await page.locator('.pagefind-ui__search-input').fill(query);
    await visible(page.locator(`.pagefind-ui__result-link[href="${server.url(pagePath(expected, locale))}"]`));
    const results = await page.locator('.pagefind-ui__result-link').evaluateAll(links => links.map(link => link.href));
    assert.ok(results.every(url => resolvePage(new URL(url).pathname, siteConfig.base)?.locale === locale));
  }
});

test('machine links and structured data work without JavaScript', async t => {
  const { page } = await session(t, { javaScriptEnabled: false });
  const examples = [publicTopics.find(topic => topic.type === 'docs'), publicTopics.find(topic => topic.type === 'article')].filter(Boolean);
  for (const locale of localeIds) for (const { id } of examples) {
    const route = pagePath(id, locale);
    await page.goto(server.url(route));
    assert.equal(await page.locator('h1').count(), 1);
    assert.equal(await page.locator('link[rel="describedby"]').count(), 1);
    assert.equal(await page.locator('script[type="application/ld+json"][data-anki-forge-schema]').count(), 1);
    const markdown = await page.locator('link[rel="alternate"][type="text/markdown"]').getAttribute('href');
    assert.equal(await page.locator(`a[href="${markdown}"]`).count(), 1);
    if (!process.env.BROWSER_SITE) {
      const response = await fetch(server.url(`${route}index.md`), { redirect: 'manual' });
      assert.equal(response.status, 200);
      assert.match(response.headers.get('content-type'), /^text\/(?:plain|markdown)/);
      assert.match(await response.text(), /^# /);
    }
  }
  if (!process.env.BROWSER_SITE) {
    for (const locale of localeIds) {
      const index = await fetch(server.url(localePath('/llms.txt', locale)), { redirect: 'manual' });
      assert.equal(index.status, 200);
      assert.match(index.headers.get('content-type'), /^text\/plain/);
      for (const route of ['/docs/missing-machine-62/index.md', '/missing-llms-62.txt']) assert.equal((await fetch(server.url(localePath(route, locale)), { redirect: 'manual' })).status, 404);
    }
  }
});

test('all configured language choices stay visible when focused on a short screen without scripts', async t => {
  const { page } = await session(t, { javaScriptEnabled: false });
  const choices = Object.entries(configuredLocales).filter(([id]) => id !== 'en');
  for (const width of [320, 390]) for (const scale of [1, 2]) {
    await page.setViewportSize({ width, height: 360 });
    await page.goto(server.url(pagePath('quickstart', 'en')));
    // Only this browser fixture exposes planned languages; production routes stay unchanged.
    await page.locator('.site-header .language-menu ul').evaluate((list, entries) => {
      const original = list.firstElementChild;
      const items = entries.map(([id, locale]) => {
        const item = original.cloneNode(true);
        const link = item.querySelector('a');
        link.textContent = locale.label;
        link.hreflang = id;
        return item;
      });
      list.replaceChildren(...items);
    }, choices);
    if (scale === 2) await page.evaluate(() => {
      const sizes = [...document.querySelectorAll('*')].map(element => [element, parseFloat(getComputedStyle(element).fontSize)]);
      for (const [element, size] of sizes) element.style.fontSize = `${size * 2}px`;
    });
    const summary = page.locator('.site-header .language-menu summary');
    await summary.focus();
    await summary.press('Enter');
    for (const [id] of choices) {
      await page.keyboard.press('Tab');
      const reachable = await page.evaluate(() => {
        const element = document.activeElement;
        const box = element.getBoundingClientRect();
        return { locale: element.getAttribute('hreflang'), visible: box.x >= 0 && box.right <= innerWidth + 1 && box.y >= 0 && box.bottom <= innerHeight + 1 && element.contains(document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2)) };
      });
      assert.equal(reachable.locale, id);
      assert.ok(reachable.visible, `${id}: focused language is clipped or covered at ${width}px/${scale * 100}% text`);
    }
  }
});

for (const locale of localeIds) {
  const tr = createTranslator(locale);
  const framework = JSON.parse(await readFile(new URL(`../src/content/i18n/${locale}.json`, import.meta.url), 'utf8'));
  const targetLocale = localeIds.find(other => other !== locale);
  const mediaQuery = searchQueries.locales[locale]?.find(query => query.kind === 'core' && query.expectedPageIds.includes('media'))?.query;
  assert.ok(targetLocale && mediaQuery, `${locale}: another language and an existing frozen media search fixture are required`);
  const route = (id, section) => server.url(pagePath(id, locale, section));
  test(`${locale}: card tabs, keyboard, flip, audio, updates and downloads`, async t => {
    const { page } = await session(t);
    await page.goto(route('home'));
    assert.equal(await page.locator('[data-forge-studio]').evaluate(element => element.getAnimations({ subtree: true }).length), 0, 'Hero ignores reduced motion');
    const tabs = page.locator('[data-workbench] [role="tab"]');
    await tabs.first().focus();
    await page.keyboard.press('End');
    assert.equal(await tabs.last().getAttribute('aria-selected'), 'true');
    await page.keyboard.press('Home');
    await page.keyboard.press('ArrowRight');
    assert.equal(await tabs.nth(1).getAttribute('aria-selected'), 'true');
    const cloze = page.locator('[data-example="cloze"]');
    await cloze.locator('[data-flip]').click();
    assert.equal(await cloze.locator('[data-card-status]').textContent(), tr('card.answer', { answer: (await cloze.locator('[data-side="back"]').textContent()).trim() }));
    await cloze.locator('[data-flip]').click();
    assert.equal(await cloze.locator('[data-card-status]').textContent(), tr('card.questionShown'));
    await tabs.nth(2).click();
    const audio = page.locator('[data-example="media"] audio');
    await audio.evaluate(element => element.play());
    assert.equal(await audio.evaluate(element => element.paused), false);
    await tabs.first().click();
    assert.equal(await audio.evaluate(element => element.paused), true);
    await page.locator('[data-update]').click();
    assert.equal(await page.locator('[data-update-status]').textContent(), tr('update.changed'));
    assert.equal(await page.locator('[data-update-value]').textContent(), 'hello; hi');
    await page.locator('[data-update]').click();
    assert.equal(await page.locator('[data-update-value]').textContent(), 'hello');
    const urls = await page.locator('[download]').evaluateAll(links => links.map(link => link.href));
    assert.equal(urls.length, 6);
    for (const url of urls) {
      const response = await fetch(url);
      assert.equal(response.status, 200);
      assert.equal(Buffer.from(await response.arrayBuffer()).subarray(0, 2).toString(), 'PK');
    }
  });

  test(`${locale}: real clipboard, denial fallback, AI task and localized guides`, async t => {
    const { page, context } = await session(t, { permissions: ['clipboard-read', 'clipboard-write'] });
    await page.goto(route('home'));
    const tabs = page.locator('[data-ai-language]');
    await tabs.first().focus();
    await page.keyboard.press('End');
    assert.equal(await tabs.last().getAttribute('aria-selected'), 'true');
    await page.locator('[data-ai-request]').fill('Learn biology <script>local only</script>');
    for (const sdk of ['rust', 'node', 'python']) {
      await page.locator(`[data-ai-language="${sdk}"]`).click();
      const panel = page.locator(`[data-ai-panel="${sdk}"]`);
      const prompt = await panel.locator('[data-ai-prompt]').textContent();
      assert.ok(prompt.includes('Learn biology <script>local only</script>'));
      assert.ok(prompt.includes(pagePath(sdk === 'rust' ? 'quickstart' : `${sdk}-quickstart`, locale)));
      await panel.locator('[data-copy]').first().click();
      await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()) === tr('ai.copied'), 'Clipboard success was not announced');
      assert.equal(await page.evaluate(() => navigator.clipboard.readText()), prompt);
    }
    // Deny at the browser permission boundary rather than replacing writeText with a mock.
    const cdp = await context.newCDPSession(page);
    const { targetInfo } = await cdp.send('Target.getTargetInfo');
    const permission = { name: 'clipboard-write', allowWithoutSanitization: false };
    await cdp.send('Browser.setPermission', { permission, setting: 'denied', origin: server.origin, browserContextId: targetInfo.browserContextId });
    const panel = page.locator('[data-ai-panel="python"]');
    await panel.locator('[data-copy]').first().click();
    await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()) === tr('ai.fallback'), 'Clipboard denial did not select the prompt');
    assert.equal(await page.evaluate(() => getSelection().toString()), await panel.locator('[data-ai-prompt]').textContent());
    await cdp.send('Browser.setPermission', { permission, setting: 'granted', origin: server.origin, browserContextId: targetInfo.browserContextId });
    await panel.locator('[data-copy]').first().click();
    await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()) === tr('ai.copied'), 'Clipboard did not recover');
    await page.reload();
    assert.equal(await page.locator('[data-ai-request]').inputValue(), '');
  });

  test(`${locale}: docs search, section navigation, copy and theme continuity`, { skip: !hasTopic('quickstart') }, async t => {
    const { page } = await session(t, { permissions: ['clipboard-read', 'clipboard-write'] });
    await page.goto(route('quickstart'));
    assert.equal(await page.locator('a[href*="/edit/main/"]').count(), 1);
    await page.keyboard.press('Control+k');
    await visible(page.locator('site-search dialog'));
    const input = page.locator('.pagefind-ui__search-input');
    await input.fill(mediaQuery);
    const results = page.locator('.pagefind-ui__result-link');
    await visible(results.first());
    const links = await results.evaluateAll(links => links.map(link => link.href));
    assert.ok(links.length);
    for (const href of links) {
      assert.equal(resolvePage(new URL(href).pathname, siteConfig.base)?.locale, locale, `Search result must belong to ${locale}: ${href}`);
      const response = await fetch(href);
      assert.equal(response.status, 200);
      const html = await response.text();
      const fragment = decodeURIComponent(new URL(href).hash.slice(1));
      if (fragment) assert.ok(html.includes(`id="${fragment}"`), `Search result section missing: ${href}`);
      assert.ok(!href.includes('404'), 'Error page entered search results');
    }
    const noMatch = 'zzzznonexistentterm62zzzz';
    await input.fill(noMatch);
    await eventually(async () => await results.count() === 0 && (await page.locator('.pagefind-ui__message').textContent()) === framework['pagefind.zero_results'].replaceAll('[SEARCH_TERM]', () => noMatch), 'Localized empty search missing');
    await page.keyboard.press('Escape');
    await eventually(() => page.locator('site-search [data-open-modal]').evaluate(element => element === document.activeElement), 'Search did not restore focus');
    assert.equal(await page.locator('site-search dialog').evaluate(dialog => dialog.open), false);
    const customCode = page.locator('.code-block[data-copy-block]').first();
    if (await customCode.count()) {
      await customCode.locator('[data-copy]').click();
      await eventually(async () => (await page.evaluate(() => navigator.clipboard.readText())) === (await customCode.locator('[data-copy-content]').textContent()), 'Documentation code was not copied');
    }
    const nativeCode = page.locator('.expressive-code').first();
    await nativeCode.locator('.copy button').click();
    await eventually(async () => (await page.evaluate(() => navigator.clipboard.readText())).trim() === (await nativeCode.locator('pre').innerText()).trim(), 'Native documentation code was not copied');
    await page.locator('.site-header .theme-option:has([value="dark"])').click();
    await page.goto(route('quickstart') + '?ref=parity#export-your-first-deck');
    const outward = await languageSwitch(page, targetLocale, { keyboard: true });
    await eventually(async () => (await outward.getAttribute('href')).includes('#export-your-first-deck'), 'Language switch did not preserve the valid section');
    await outward.press('Enter');
    await page.waitForLoadState('domcontentloaded');
    assert.equal(await page.locator('html').getAttribute('lang'), targetLocale);
    assert.equal(new URL(page.url()).pathname, new URL(server.url(pagePath('quickstart', targetLocale))).pathname);
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
    assert.equal(new URL(page.url()).search, '?ref=parity');
    await page.locator('[id="export-your-first-deck"]').waitFor({ state: 'attached' });
    const returning = await languageSwitch(page, locale, { keyboard: true });
    await eventually(async () => (await returning.getAttribute('href')).includes('#export-your-first-deck'), 'Return language switch lost the section');
    await returning.press('Enter');
    await page.waitForLoadState('domcontentloaded');
    assert.equal(await page.locator('html').getAttribute('lang'), locale);
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
    assert.equal(new URL(page.url()).search, '?ref=parity');
    await page.goto(route('quickstart') + '#nonexistent-section-62');
    await page.waitForTimeout(150);
    for (const href of await page.locator('.site-header [data-language-switch]').evaluateAll(links => links.map(link => link.href))) assert.equal(new URL(href).hash, '', 'Unknown fragment must not leak to any language');
  });

  test(`${locale}: native documentation code-copy denial and recovery`, { skip: !hasTopic('node-quickstart') }, async t => {
    const { page, context } = await session(t, { permissions: ['clipboard-read', 'clipboard-write'] });
    await page.goto(route('node-quickstart'));
    const block = page.locator('.expressive-code').filter({ hasText: 'import {' }).first();
    const button = block.locator('.copy button');
    await button.click();
    const code = (await button.getAttribute('data-code')).replace(/\u007f/g, '\n');
    await eventually(async () => (await page.evaluate(() => navigator.clipboard.readText())) === code, 'Native complete program was not copied');
    const cdp = await context.newCDPSession(page);
    const { targetInfo } = await cdp.send('Target.getTargetInfo');
    const permission = { name: 'clipboard-write', allowWithoutSanitization: false };
    await cdp.send('Browser.setPermission', { permission, setting: 'denied', origin: server.origin, browserContextId: targetInfo.browserContextId });
    await button.click();
    await eventually(async () => (await block.locator('[data-copy-status]').textContent()) === tr('copy.codeFallback'), 'Native copy failure has no localized manual recovery');
    assert.equal((await page.evaluate(() => getSelection().toString())).trim(), code.trim());
    await cdp.send('Browser.setPermission', { permission, setting: 'granted', origin: server.origin, browserContextId: targetInfo.browserContextId });
    await button.click();
    await eventually(async () => (await block.locator('[data-copy-status]').textContent()) === tr('copy.codeSuccess'), 'Native copy did not recover after permission returned');
  });

  test(`${locale}: mobile navigation, native sidebar, system theme and legacy sections`, { skip: !hasTopic('quickstart') || !hasTopic('concepts') }, async t => {
    const { page } = await session(t, { viewport: { width: 390, height: 844 }, colorScheme: 'dark' });
    await page.goto(route('home') + '#workflow-heading');
    assert.equal(await page.locator('#workflow-heading').count(), 1);
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    const systemOption = page.getByRole('radio', { name: tr('nav.auto'), exact: true });
    assert.equal(await systemOption.isChecked(), true);
    await systemOption.focus();
    await page.keyboard.press('ArrowRight');
    assert.equal(await page.getByRole('radio', { name: tr('nav.light'), exact: true }).isChecked(), true);
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'light');
    await page.keyboard.press('ArrowLeft');
    assert.equal(await systemOption.isChecked(), true);
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    await page.locator('.mobile-menu summary').click();
    await follow(page, page.locator('.mobile-menu [data-section="examples"]'));
    assert.equal(new URL(page.url()).pathname, new URL(route('examples')).pathname);
    await page.locator('.site-header .theme-option:has([value="light"])').click();
    await page.locator('.mobile-menu summary').click();
    await follow(page, page.locator('.mobile-menu [data-section="docs"]'));
    assert.equal(new URL(page.url()).pathname, new URL(route('docs')).pathname);
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'light');
    const checkThemeGroups = async value => {
      const groups = await page.locator('[data-theme-control]').evaluateAll(controls => controls.map(control => {
        const inputs = [...control.querySelectorAll('input[type="radio"]')];
        return { names: [...new Set(inputs.map(input => input.name))], selected: inputs.filter(input => input.checked).map(input => input.value), size: inputs.length };
      }));
      assert.equal(groups.length, 2, 'Documentation must retain both header and native theme controls');
      assert.equal(new Set(groups.flatMap(group => group.names)).size, groups.length, 'Separate controls must not share a native radio group');
      for (const group of groups) {
        assert.equal(group.names.length, 1);
        assert.equal(group.size, 3);
        assert.deepEqual(group.selected, [value], 'Every theme control retains its own checked option');
      }
    };
    await checkThemeGroups('light');
    const menu = page.locator('button[popovertarget="starlight__sidebar"]');
    await menu.click();
    await visible(page.locator('#starlight__sidebar'));
    const concepts = page.locator(`#starlight__sidebar a[href="${new URL(route('concepts')).pathname}"]`);
    assert.equal((await concepts.textContent()).trim(), publicTopics.find(topic => topic.id === 'concepts').title[locale]);
    await follow(page, concepts);
    assert.equal(new URL(page.url()).pathname, new URL(route('concepts')).pathname);
    await page.locator('.site-header .theme-option:has([value="auto"])').click();
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    await checkThemeGroups('auto');
    await page.emulateMedia({ colorScheme: 'light' });
    await eventually(async () => (await page.locator('html').getAttribute('data-theme')) === 'light', 'System theme changes were ignored');
    await page.goto(server.url(pagePath('quickstart', 'zh-CN')) + '#生成-apkg');
    const legacyTarget = await languageSwitch(page, 'en');
    await eventually(async () => (await legacyTarget.getAttribute('href')).endsWith('#export-your-first-deck'), 'Legacy Chinese section was not mapped');
    await follow(page, legacyTarget);
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
  });

  test(`${locale}: search loading failure has localized feedback and a close action`, { skip: !hasTopic('quickstart') }, async t => {
    const { page, context } = await session(t);
    await context.route('**/pagefind/**', route => route.abort());
    await page.goto(route('quickstart'));
    await page.locator('site-search [data-open-modal]').click();
    await eventually(async () => (await page.locator('site-search [data-search-feedback]').textContent()) === tr('search.error'), 'Search failure was not localized');
    await page.locator('site-search [data-close-modal]').click();
    assert.equal(await page.locator('site-search dialog').evaluate(dialog => dialog.open), false);
    await eventually(() => page.locator('site-search [data-open-modal]').evaluate(element => element === document.activeElement), 'Search did not restore focus');
  });

  test(`${locale}: header controls stay reachable at narrow 200% text zoom`, async t => {
    const { page } = await session(t);
    for (const width of [320, 390]) {
      await page.setViewportSize({ width, height: 600 });
      for (const id of ['home', 'quickstart'].filter(hasTopic)) {
        await page.goto(route(id));
        await page.evaluate(() => document.documentElement.style.fontSize = '200%');
        for (const control of await page.locator('.site-header .wordmark, .header-tools > :visible').all()) {
          const box = await control.boundingBox();
          assert.ok(box && box.x >= 0 && box.x + box.width <= width + 1, `${locale}/${id} clips a header control at ${width}px with 200% text`);
          assert.ok(await control.evaluate(element => {
            const box = element.getBoundingClientRect();
            return element.contains(document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2));
          }), `${locale}/${id}: another control obscures a header action`);
        }
        const menu = page.locator('.site-header .mobile-menu summary');
        await menu.focus();
        await menu.press('Enter');
        const link = page.locator('.site-header .mobile-menu nav a').first();
        await visible(link);
        const box = await link.boundingBox();
        assert.ok(box.x >= 0 && box.x + box.width <= width + 1, 'Opened navigation must stay inside the narrow viewport');
      }
    }
  });

  test(`${locale}: narrow layouts, no script content and real missing-page recovery`, async t => {
    const { page } = await session(t);
    const configuration = process.env.BROWSER_SITE ? 'production' : siteConfig.base === '/' ? 'root' : 'prefix';
    const output = new URL(`../../target/website-i18n-62/${configuration}/`, import.meta.url).pathname;
    await mkdir(output, { recursive: true });
    for (const width of [320, 390, 768, 1440]) {
      await page.setViewportSize({ width, height: 1000 });
      for (const id of ['home', 'examples', 'blog', 'blog:your-first-anki-deck', 'quickstart'].filter(hasTopic)) {
        await page.goto(route(id));
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        assert.ok(overflow <= 1, `${locale}/${id} overflows ${overflow}px at ${width}px`);
        const brand = await page.locator('.site-header .brand-mark svg').boundingBox();
        assert.ok(brand && brand.width > 0 && brand.height > 0, `${id} hides the home/brand action at ${width}px`);
        const tools = await page.locator('.header-tools').boundingBox();
        assert.ok(tools.x >= 0 && tools.x + tools.width <= width + 1, `${id} clips tools at ${width}px`);
        if (width === 320 && ['home', 'quickstart'].includes(id)) await page.screenshot({ path: `${output}${locale}-${id}-${width}.png`, fullPage: true });
      }
    }
    await page.setViewportSize({ width: 768, height: 900 });
    await page.goto(route(hasTopic('quickstart') ? 'quickstart' : 'docs'));
    await page.evaluate(() => document.documentElement.style.fontSize = '200%');
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), 'Text zoom caused page overflow');
    const { page: plain } = await session(t, { javaScriptEnabled: false });
    await plain.goto(route('home'));
    assert.equal(await plain.locator('[data-ai-panel]:visible').count(), 3);
    assert.equal(await plain.locator('[download]:visible').count(), 6);
    assert.ok(await plain.locator('noscript').count() > 0);
    const noScriptLanguage = await languageSwitch(plain, targetLocale, { keyboard: true });
    assert.equal(new URL(await noScriptLanguage.getAttribute('href'), plain.url()).pathname, new URL(server.url(pagePath('home', targetLocale))).pathname);
    await noScriptLanguage.press('Enter');
    await plain.waitForLoadState('domcontentloaded');
    assert.equal(await plain.locator('html').getAttribute('lang'), targetLocale, 'Native language menu must navigate without JavaScript');
    await plain.goto(route(hasTopic('quickstart') ? 'quickstart' : 'docs'));
    assert.equal(await plain.locator('site-search [data-open-modal]').isDisabled(), true, 'Search stays disabled without JavaScript');
    const missing = await page.goto(server.url(localePath('/missing-topic-62/', locale)));
    assert.equal(missing.status(), 404);
    assert.equal(await page.locator('html').getAttribute('lang'), locale);
    await visible(page.getByRole('link', { name: tr('error.home'), exact: true }));
    await plain.goto(server.url(localePath('/missing-topic-62/', locale)));
    // GitHub Pages serves one static 404 document; every release language has a no-script recovery link.
    for (const recoveryLocale of localeIds) await visible(plain.getByRole('link', { name: createTranslator(recoveryLocale)('error.home'), exact: true }));
  });
}
