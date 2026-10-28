import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { chromium } from 'playwright';
import { serveBuiltSite } from './serve-built-site.mjs';
import { siteConfig } from '../site.config.mjs';
import { pages, pagePath, localeIds } from '../site-model.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';

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
const follow = async (page, link) => { await link.click(); await page.waitForLoadState('domcontentloaded'); };
const eventually = async (operation, message) => {
  for (let attempt = 0; attempt < 60; attempt++) {
    if (await operation()) return;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.fail(message);
};

test('all built content URLs and assets are reachable over HTTP', async () => {
  for (const topic of pages) for (const locale of localeIds) {
    const response = await fetch(server.url(pagePath(topic.id, locale)));
    assert.equal(response.status, 200);
    assert.match(await response.text(), new RegExp(`<html[^>]+lang="${locale}"`));
  }
  for (const route of ['/missing-topic-62/', '/zh-cn/missing-topic-62/']) assert.equal((await fetch(server.url(route))).status, 404);
});

for (const locale of localeIds) {
  const tr = createTranslator(locale);
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
    assert.match(await cloze.locator('[data-card-status]').textContent(), locale === 'en' ? /Answer:/ : /答案：/);
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
      await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()).includes(locale === 'en' ? 'copied' : '已复制'), 'Clipboard success was not announced');
      assert.equal(await page.evaluate(() => navigator.clipboard.readText()), prompt);
    }
    // Deny at the browser permission boundary rather than replacing writeText with a mock.
    const cdp = await context.newCDPSession(page);
    const { targetInfo } = await cdp.send('Target.getTargetInfo');
    const permission = { name: 'clipboard-write', allowWithoutSanitization: false };
    await cdp.send('Browser.setPermission', { permission, setting: 'denied', origin: server.origin, browserContextId: targetInfo.browserContextId });
    const panel = page.locator('[data-ai-panel="python"]');
    await panel.locator('[data-copy]').first().click();
    await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()).includes(locale === 'en' ? 'selected' : '已选中'), 'Clipboard denial did not select the prompt');
    assert.equal(await page.evaluate(() => getSelection().toString()), await panel.locator('[data-ai-prompt]').textContent());
    await cdp.send('Browser.setPermission', { permission, setting: 'granted', origin: server.origin, browserContextId: targetInfo.browserContextId });
    await panel.locator('[data-copy]').first().click();
    await eventually(async () => (await panel.locator('[data-copy-status]').first().textContent()).includes(locale === 'en' ? 'copied' : '已复制'), 'Clipboard did not recover');
    await page.reload();
    assert.equal(await page.locator('[data-ai-request]').inputValue(), '');
  });

  test(`${locale}: docs search, section navigation, copy and theme continuity`, async t => {
    const { page } = await session(t, { permissions: ['clipboard-read', 'clipboard-write'] });
    await page.goto(route('quickstart'));
    assert.equal(await page.locator('a[href*="/edit/main/"]').count(), 1);
    await page.keyboard.press('Control+k');
    await visible(page.locator('site-search dialog'));
    const input = page.locator('.pagefind-ui__search-input');
    await input.fill(locale === 'en' ? 'media' : '媒体');
    const results = page.locator('.pagefind-ui__result-link');
    await visible(results.first());
    const links = await results.evaluateAll(links => links.map(link => link.href));
    assert.ok(links.length);
    for (const href of links) {
      assert.equal(new URL(href).pathname.includes('/zh-cn/'), locale === 'zh-CN');
      const response = await fetch(href);
      assert.equal(response.status, 200);
      const html = await response.text();
      const fragment = decodeURIComponent(new URL(href).hash.slice(1));
      if (fragment) assert.ok(html.includes(`id="${fragment}"`), `Search result section missing: ${href}`);
      assert.ok(!href.includes('404'), 'Error page entered search results');
    }
    await input.fill('zzzznonexistentterm62zzzz');
    await eventually(async () => await results.count() === 0 && (await page.locator('.pagefind-ui__message').textContent()).includes(locale === 'en' ? 'No results' : '没有'), 'Localized empty search missing');
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
    await eventually(async () => (await page.locator('.site-header [data-language-switch]').getAttribute('href')).includes('#export-your-first-deck'), 'Language switch did not preserve the valid section');
    await follow(page, page.locator('.site-header [data-language-switch]'));
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
    assert.equal(new URL(page.url()).search, '?ref=parity');
    await page.locator('[id="export-your-first-deck"]').waitFor({ state: 'attached' });
    await eventually(async () => (await page.locator('.site-header [data-language-switch]').getAttribute('href')).includes('#export-your-first-deck'), 'Return language switch lost the section');
    await follow(page, page.locator('.site-header [data-language-switch]'));
    assert.equal(await page.locator('html').getAttribute('lang'), locale);
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
    await page.goto(route('quickstart') + '#nonexistent-section-62');
    await page.waitForTimeout(150);
    assert.equal(new URL(await page.locator('.site-header [data-language-switch]').getAttribute('href'), page.url()).hash, '');
  });

  test(`${locale}: native documentation code-copy denial and recovery`, async t => {
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

  test(`${locale}: mobile navigation, native sidebar, system theme and legacy sections`, async t => {
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
    const menu = page.locator('button[popovertarget="starlight__sidebar"]');
    await menu.click();
    await visible(page.locator('#starlight__sidebar'));
    await follow(page, page.locator('#starlight__sidebar a').filter({ hasText: locale === 'en' ? 'Core concepts' : '核心概念' }));
    assert.equal(new URL(page.url()).pathname, new URL(route('concepts')).pathname);
    await page.locator('.site-header .theme-option:has([value="auto"])').click();
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    await page.emulateMedia({ colorScheme: 'light' });
    await eventually(async () => (await page.locator('html').getAttribute('data-theme')) === 'light', 'System theme changes were ignored');
    await page.goto(server.url(pagePath('quickstart', 'zh-CN')) + '#生成-apkg');
    await eventually(async () => (await page.locator('.site-header [data-language-switch]').getAttribute('href')).endsWith('#export-your-first-deck'), 'Legacy Chinese section was not mapped');
    await follow(page, page.locator('.site-header [data-language-switch]'));
    assert.equal(new URL(page.url()).hash, '#export-your-first-deck');
  });

  test(`${locale}: search loading failure has localized feedback and a close action`, async t => {
    const { page, context } = await session(t);
    await context.route('**/pagefind/**', route => route.abort());
    await page.goto(route('quickstart'));
    await page.locator('site-search [data-open-modal]').click();
    await eventually(async () => (await page.locator('site-search [data-search-feedback]').textContent()).includes(locale === 'en' ? 'could not load' : '无法加载'), 'Search failure was not localized');
    await page.locator('site-search [data-close-modal]').click();
    assert.equal(await page.locator('site-search dialog').evaluate(dialog => dialog.open), false);
    await eventually(() => page.locator('site-search [data-open-modal]').evaluate(element => element === document.activeElement), 'Search did not restore focus');
  });

  test(`${locale}: narrow layouts, no script content and real missing-page recovery`, async t => {
    const { page } = await session(t);
    const configuration = process.env.BROWSER_SITE ? 'production' : siteConfig.base === '/' ? 'root' : 'prefix';
    const output = new URL(`../../target/website-i18n-62/${configuration}/`, import.meta.url).pathname;
    await mkdir(output, { recursive: true });
    for (const width of [320, 390, 768, 1440]) {
      await page.setViewportSize({ width, height: 1000 });
      for (const id of ['home', 'examples', 'blog', 'blog:your-first-anki-deck', 'quickstart']) {
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
    await page.goto(route('quickstart'));
    await page.evaluate(() => document.documentElement.style.fontSize = '200%');
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), 'Text zoom caused page overflow');
    const { page: plain } = await session(t, { javaScriptEnabled: false });
    await plain.goto(route('home'));
    assert.equal(await plain.locator('[data-ai-panel]:visible').count(), 3);
    assert.equal(await plain.locator('[download]:visible').count(), 6);
    assert.ok(await plain.locator('noscript').count() > 0);
    const missing = await page.goto(server.url(locale === 'en' ? '/missing-topic-62/' : '/zh-cn/missing-topic-62/'));
    assert.equal(missing.status(), 404);
    assert.equal(await page.locator('html').getAttribute('lang'), locale);
    await visible(page.getByRole('link', { name: tr('error.home'), exact: true }));
    await plain.goto(server.url('/zh-cn/missing-topic-62/'));
    // GitHub Pages serves one static 404 document: the honest no-script fallback is bilingual.
    await visible(plain.getByRole('link', { name: '返回首页', exact: true }));
    await visible(plain.getByRole('link', { name: 'Back home', exact: true }));
  });
}
