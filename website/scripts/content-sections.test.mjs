import test from 'node:test';
import assert from 'node:assert/strict';
import { headings, sectionMapping, addSectionAliases, legacyAliases } from './content-sections.mjs';
import legacy from '../legacy-sections.json' with { type: 'json' };

const registry = { entries: { example: { requiredSections: ['install', 'update'], locales: { en: { install: 'install', update: 'update' }, 'zh-CN': { '安装': 'install', '更新': 'update' } } } } };
test('semantic section identity permits translated reordering and heading depth changes', () => {
  const source = '## 更新\n更新说明\n\n#### 安装\n安装说明\n';
  const mapping = sectionMapping('example', 'zh-CN', source, { '原安装': 'install' }, registry);
  const result = addSectionAliases(source, mapping);
  assert.ok(result.indexOf('id="update"') < result.indexOf('## 更新'));
  assert.ok(result.indexOf('id="install"') > result.indexOf('更新说明'));
  assert.ok(result.indexOf('id="原安装"') < result.indexOf('#### 安装'));
  assert.throws(() => sectionMapping('example', 'zh-CN', '## 安装\n', {}, registry), /missing task section update/);
});
test('semantic markers allow natural title changes and merge task contexts', () => {
  const source = '<!-- section: install -->\n<!-- section: update -->\n## 从这里完成任务\nInstructions.\n';
  const mapping = sectionMapping('example', 'ja', source, {}, registry);
  const result = addSectionAliases(source, mapping);
  assert.match(result, /id="install"/);
  assert.match(result, /id="update"/);
  assert.throws(() => headings('<!-- section: install -->\nParagraph\n## A'), /immediately precede/);
  assert.throws(() => headings('<!-- section: install -->\n## A\n<!-- section: install -->\n## B'), /Duplicate/);
});
test('fenced code markers and headings do not become real sections or aliases', () => {
  const source = '~~~md\n<!-- section: update -->\n## Example code\n~~~\n\n<!-- section: install -->\n## Setup\n\n<!-- section: update -->\n## Change an answer\n';
  assert.deepEqual(headings(source).map(heading => heading.id), ['setup', 'change-an-answer']);
  const result = addSectionAliases(source, sectionMapping('example', 'es', source, {}, registry));
  assert.match(result, /~~~md\n<!-- section: update -->\n## Example code\n~~~/);
  assert.equal((result.match(/id="update"/g) || []).length, 1);
});
test('an existing heading cannot silently capture another task stable alias', () => {
  assert.throws(() => addSectionAliases('## Install\n\n## Update\n', { install: 'update', update: 'install' }), /conflicts with another task/);
});
test('historic section aliases belong to the locale that published them', () => {
  const registry = { 'zh-CN': { example: { '原安装': 'install' } } };
  assert.deepEqual(legacyAliases(registry, 'example', 'zh-CN'), { '原安装': 'install' });
  assert.deepEqual(legacyAliases(registry, 'example', 'es'), {});
  assert.deepEqual(legacyAliases(registry, 'other', 'zh-CN'), {});
  assert.throws(() => legacyAliases({ 'zh-cn': {} }, 'example', 'zh-CN'), /Unknown legacy section locale: zh-cn/);
  assert.equal(legacyAliases(legacy, 'quickstart', 'zh-CN')['生成-apkg'], 'export-your-first-deck');
  assert.deepEqual(legacyAliases(legacy, 'quickstart', 'en'), {});
});
