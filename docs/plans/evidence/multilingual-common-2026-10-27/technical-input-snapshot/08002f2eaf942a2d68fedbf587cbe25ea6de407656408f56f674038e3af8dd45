import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pagePath, releaseLocales } from '../site-model.mjs';
import { renderPrompt } from '../src/lib/ai-quickstart.mjs';
import { importedDocs } from './content-links.mjs';
import { verifySkill } from '../../skills/anki-forge/scripts/verify_quickstart.mjs';

export const root = fileURLToPath(new URL('../../', import.meta.url));
const start = '<!-- ai-quickstart:start -->';
const end = '<!-- ai-quickstart:end -->';
const read = (filename, directory) => readFile(path.join(directory, filename), 'utf8');

export async function loadAiQuickstart(directory = root) {
  const data = JSON.parse(await read('docs/ai-quickstart.json', directory));
  assert.equal(data.schemaVersion, 1, 'Unsupported AI quickstart schema');
  assert.match(data.publicVersion, /^\d+\.\d+\.\d+$/, 'Use a public package version');
  assert.deepEqual(Object.keys(data.languages), ['rust', 'node', 'python']);
  for (const field of ['prompts', 'requestPlaceholders', 'promptContracts']) {
    assert.deepEqual(Object.keys(data[field] ?? {}), [...releaseLocales], `${field} must cover exactly the release locales`);
  }
  const origin = new URL(data.sitePromptUrl);
  assert.equal(origin.protocol, 'https:');
  assert.equal(origin.hash, '#ai-quickstart');
  assert.equal(origin.search + origin.username + origin.password, '');
  const installs = {
    rust: `cargo add ankiforge@${data.publicVersion}`,
    node: `npm install --include=optional ankiforge@${data.publicVersion}`,
    python: `python -m pip install ankiforge==${data.publicVersion}`,
  };
  for (const [key, language] of Object.entries(data.languages)) {
    assert.equal(language.install, installs[key], `${key}: incorrect public install command`);
    assert.ok(language.label && language.runtime, `${key}: missing language/runtime`);
    const url = new URL(language.quickstart);
    assert.equal(url.origin, origin.origin, `${key}: inconsistent public docs origin`);
    assert.equal(url.search + url.hash + url.username + url.password, '');
    assert.ok(url.pathname.startsWith('/docs/'), `${key}: expected a public docs URL`);
    const route = `/${url.pathname.slice(origin.pathname.length).replace(/^\//, '')}`;
    const doc = importedDocs.find(entry => entry.route === route);
    await stat(path.join(directory, doc ? doc.source : `website/src/content/docs${route.replace(/\/$/, '')}.mdx`));
  }
  for (const [locale, template] of Object.entries(data.prompts)) {
    const contract = data.promptContracts[locale];
    assert.ok(typeof data.requestPlaceholders[locale] === 'string' && data.requestPlaceholders[locale].trim(), `${locale}: missing request placeholder`);
    assert.doesNotMatch(data.requestPlaceholders[locale], /[{}]/, `${locale}: Unsafe request placeholder: template syntax`);
    assert.ok(Array.isArray(contract.required) && contract.required.length && contract.required.every(text => typeof text === 'string' && text.trim()), `${locale}: missing prompt requirements`);
    assert.ok(Number.isInteger(contract.maxWords) !== Number.isInteger(contract.maxCharacters), `${locale}: choose one prompt length limit`);
    const tokens = [...template.matchAll(/\{([^{}]+)\}/g)].map(match => match[1]);
    assert.deepEqual([...new Set(tokens)].sort(), ['install', 'language', 'quickstart', 'request', 'runtime', 'version']);
    assert.equal(tokens.filter(token => token === 'request').length, 1, 'Render the request once');
    assert.doesNotMatch(template, /<\/?[a-z][^>]*>|sk-[A-Za-z0-9]+|Copy prompt for AI|Copied|maturin develop|internal-tools/, 'Unsafe prompt template: markup, credentials or UI text');
    for (const language of Object.keys(data.languages)) {
      const prompt = renderPrompt(data, language, '', locale);
      for (const requirement of contract.required) assert.ok(prompt.includes(requirement), `${language}/${locale}: incomplete prompt contract`);
      assert.ok(prompt.includes(data.languages[language].install) && prompt.includes(new URL(pagePath(language === 'rust' ? 'quickstart' : `${language}-quickstart`, locale), data.languages[language].quickstart).href));
      const length = contract.maxWords ? prompt.split(/\s+/).length : prompt.length;
      assert.ok(length <= (contract.maxWords ?? contract.maxCharacters), `${language}/${locale}: prompt too long`);
    }
  }
  return data;
}

export function renderReadme(data, locale) {
  const chinese = locale === 'zh-CN';
  const intro = chinese
    ? '## 与 AI 编码助手一起使用\n\n将下方提示复制到 ChatGPT、Cursor、Claude Code 或其他编码助手。\n这是 Rust 模板；选择 Node / TypeScript 或 Python 时，请使用官网生成对应语言的提示。\n替换需求占位符后粘贴给 AI。'
    : '## Use with an AI coding assistant\n\nCopy this brief into ChatGPT, Cursor, Claude Code, or another coding assistant.\nThis template uses Rust; use the website to generate a Node / TypeScript or Python brief.\nReplace the request placeholder before pasting it into your assistant.';
  const builder = chinese ? '打开交互式提示生成器' : 'Open the interactive prompt builder';
  const skill = chinese ? '查看 Anki Forge Skill' : 'Open the Anki Forge Skill';
  const setup = chinese ? 'Skill 加载说明' : 'Skill setup instructions';
  const description = chinese
    ? '支持 Skill 的编码助手可按需加载语言安装、公开 API 规则、任务流程和验证指南：'
    : 'Coding assistants that support Skills can load language-specific setup, public API rules, task workflows, and verification guidance:';
  return `${start}\n${intro}\n\n\`\`\`text\n${renderPrompt(data, 'rust', '', locale)}\n\`\`\`\n\n[${builder}](${chinese ? data.sitePromptUrl.replace('/#', '/zh-cn/#') : data.sitePromptUrl})\n\n${description}\n\n[${skill}](skills/anki-forge/SKILL.md) · [${setup}](docs/ai-assistants.md)\n${end}`;
}

function replaceBlock(markdown, expected, filename) {
  assert.equal(markdown.split(start).length, 2, `${filename}: expected one AI start marker`);
  assert.equal(markdown.split(end).length, 2, `${filename}: expected one AI end marker`);
  const from = markdown.indexOf(start);
  const to = markdown.indexOf(end);
  assert.ok(to > from, `${filename}: reversed AI markers`);
  return markdown.slice(0, from) + expected + markdown.slice(to + end.length);
}

export async function checkAiQuickstart({ sync = false, directory = root } = {}) {
  const data = await loadAiQuickstart(directory);
  for (const [filename, locale] of [['README.md', 'en'], ['README.zh-CN.md', 'zh-CN']]) {
    const markdown = await read(filename, directory);
    const updated = replaceBlock(markdown, renderReadme(data, locale), filename);
    if (sync) await writeFile(path.join(directory, filename), updated);
    else assert.equal(markdown, updated, `${filename}: stale AI brief; run npm run sync:ai`);
  }
  const guides = { rust: 'docs/installation.md', node: 'docs/node/quick-start.md', python: 'docs/python/quick-start.md' };
  for (const [key, filename] of Object.entries(guides)) {
    const markdown = await read(filename, directory);
    assert.ok(markdown.includes(data.languages[key].install), `${filename}: public installation drift`);
    for (const [, version] of markdown.matchAll(/ankiforge(?:@|==)(\d+\.\d+\.\d+)/g)) {
      assert.equal(version, data.publicVersion, `${filename}: stale public version`);
    }
  }
  for (const filename of ['README.md', 'README.zh-CN.md']) {
    const markdown = await read(filename, directory);
    for (const language of Object.values(data.languages)) assert.ok(markdown.includes(language.install), `${filename}: public installation drift`);
    for (const [, version] of markdown.matchAll(/ankiforge(?:@|==| )(\d+\.\d+\.\d+)/g)) {
      assert.equal(version, data.publicVersion, `${filename}: stale public version`);
    }
  }
  for (const [, version] of (await read('docs/compatibility.md', directory)).matchAll(/ankiforge (\d+\.\d+\.\d+)/g)) {
    assert.equal(version, data.publicVersion, 'docs/compatibility.md: stale public version');
  }
  await verifySkill({ directory, data });
  return data;
}

export async function generateAiQuickstart(data, directory = root) {
  const generated = path.join(directory, 'website/src/generated');
  await mkdir(generated, { recursive: true });
  await writeFile(path.join(generated, 'ai-quickstart.json'), JSON.stringify(data, null, 2) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const sync = process.argv.includes('--write');
  const data = await checkAiQuickstart({ sync });
  if (sync) await generateAiQuickstart(data);
  console.log(`${sync ? 'Synchronized' : 'Checked'} AI briefs, public installations and Skill references.`);
}
