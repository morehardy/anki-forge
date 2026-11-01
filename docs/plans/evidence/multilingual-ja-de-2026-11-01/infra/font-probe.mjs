import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';

const root = process.cwd();
const output = path.join(root, 'docs/plans/evidence/multilingual-ja-de-2026-11-01/infra');
const scratch = path.join(root, 'target/remaining-locales/ja-de-infra');
const { chromium } = await import(pathToFileURL(path.join(root, 'website/node_modules/playwright/index.mjs')));
const sharp = createRequire(path.join(root, 'website/package.json'))('sharp');
const source = await readFile(path.join(root, 'website/scripts/social-image.mjs'), 'utf8');
const tokens = await readFile(path.join(root, 'website/src/styles/tokens.css'), 'utf8');
const family = source.match(/<g font-family="([^"]+)"/)[1];
const ui = JSON.parse(await readFile(path.join(root, 'docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/home-ui.json'), 'utf8'));
const phrase = ui['home.hero2'];
const escape = value => value.replace(/[&<>]/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[char]);
await mkdir(path.join(scratch, 'font-cache'), { recursive: true });
const fc = path.join(scratch, 'fonts.conf');
await writeFile(fc, `<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig><dir>/Library/Fonts</dir><dir>/System/Library/Fonts</dir><cachedir>${scratch}/font-cache</cachedir></fontconfig>`);
process.env.FONTCONFIG_FILE = fc;
const fontmatches = {};
for (const query of [`${family}:charset=9aa8`, 'Noto Sans CJK JP:charset=9aa8', 'Hiragino Sans:charset=9aa8', 'Noto Serif JP:charset=9aa8']) {
  fontmatches[query] = execFileSync('/opt/homebrew/bin/fc-match', ['-f', '%{family}\n%{file}\n', query], { env: process.env, encoding: 'utf8' }).trim();
}
const images = [];
for (const [label, fonts, language] of [['current-og-family', family, ''], ['explicit-jp-control', 'Hiragino Sans, sans-serif', 'ja']]) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="300" ${language ? `xml:lang="${language}"` : ''}><rect width="1200" height="300" fill="white"/><g font-family="${fonts}" fill="#142d29"><text x="40" y="90" font-size="52">${escape(phrase)}</text><text x="40" y="190" font-size="52">画像と音声・配布済みデッキの更新</text><text x="40" y="268" font-size="42">字形比較用: 骨 直 令 返 述</text></g></svg>`;
  await writeFile(path.join(scratch, `${label}.svg`), svg);
  const bytes = await sharp(Buffer.from(svg)).png().toBuffer();
  const file = `${label}.png`;
  await writeFile(path.join(output, file), bytes);
  images.push({ file, sha256: createHash('sha256').update(bytes).digest('hex') });
}
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const observations = [];
try {
  const page = await browser.newPage({ viewport: { width: 1000, height: 500 } });
  await page.setContent(`<!doctype html><html lang="ja"><head><meta charset="utf-8"><style>${tokens}body{font-family:var(--af-font);padding:20px}p{font-size:36px}code{font-family:var(--af-mono)}</style></head><body><p id="system">${escape(phrase)} 画像と音声</p><p id="glyphs">骨 直 令 返 述</p><p><code id="code">BuildOptions.temporary 日本語カード</code></p></body></html>`);
  await page.evaluate(() => document.fonts.ready);
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('DOM.enable');
  await cdp.send('CSS.enable');
  const { root: document } = await cdp.send('DOM.getDocument');
  for (const selector of ['#system', '#glyphs', '#code']) {
    const { nodeId } = await cdp.send('DOM.querySelector', { nodeId: document.nodeId, selector });
    const fonts = await cdp.send('CSS.getPlatformFontsForNode', { nodeId });
    observations.push({ selector, ...fonts, style: await page.locator(selector).evaluate(node => ({ fontFamily: getComputedStyle(node).fontFamily, lang: node.closest('[lang]').lang, text: node.textContent })) });
  }
  await page.screenshot({ path: path.join(output, 'browser-ja-fonts.png') });
  await writeFile(path.join(output, 'font-observations.json'), JSON.stringify({ schemaVersion: 1, scope: 'macOS-only isolated glyph/font fixture using actual Japanese sample headline and current CSS; not a full OG product render and not Linux CI validation', node: process.version, chrome: browser.version(), currentOgFontFamily: family, svgHasLanguage: /(?:xml:lang|lang)=/.test(source), fontmatches, observations, images }, null, 2) + '\n');
  console.log(JSON.stringify({ fontmatches, observations }, null, 2));
} finally { await browser.close(); }
