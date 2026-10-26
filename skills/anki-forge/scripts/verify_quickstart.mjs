import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { importedDocs } from '../../../website/scripts/content-links.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));

/** Contributor check: run in the Anki Forge repository, not a user's app. */
export async function verifySkill({ directory = root, data } = {}) {
  const read = filename => readFile(path.join(directory, filename), 'utf8');
  data ??= JSON.parse(await read('docs/ai-quickstart.json'));
  const skill = 'skills/anki-forge';
  const main = await read(`${skill}/SKILL.md`);
  assert.ok(main.split('\n').length <= 100, 'Keep SKILL.md below 100 lines');
  assert.match(main, /^---\nname: anki-forge\ndescription: /);
  assert.ok(main.includes(`Anki Forge ${data.publicVersion}`), 'Skill metadata version drift');
  const guides = { rust: 'anki_forge/examples/target_api_basic.rs', node: 'docs/node/quick-start.md', python: 'docs/python/quick-start.md' };
  for (const [key, guide] of Object.entries(guides)) {
    const reference = await read(`${skill}/references/${key}.md`);
    assert.ok(reference.includes(data.publicVersion) && reference.includes(data.languages[key].install), `${key}: Skill installation drift`);
    assert.ok(reference.includes(data.languages[key].runtime), `${key}: Skill runtime drift`);
    assert.ok(reference.includes(data.languages[key].quickstart), `${key}: Skill quickstart URL drift`);
    const pattern = /<!-- source: ([^\n]+) -->\n```[^\n]+\n([\s\S]*?)\n```\n<!-- \/source -->/;
    const example = reference.match(pattern);
    const canonical = key === 'rust' ? [undefined, guide, (await read(guide)).trimEnd()] : (await read(guide)).match(pattern);
    assert.ok(example && canonical, `${key}: missing checked complete program`);
    assert.equal(example[1], canonical[1], `${key}: incorrect quickstart source`);
    assert.equal(example[2], canonical[2], `${key}: quickstart differs from public guide`);
    assert.equal(example[2], (await read(example[1])).trimEnd(), `${key}: stale quickstart source`);
    if (key === 'node') assert.match(example[2], /await output\.artifact\.close\(\)/);
  }
  const files = ['SKILL.md', ...['rust', 'node', 'python', 'workflows', 'troubleshooting'].map(key => `references/${key}.md`)];
  for (const filename of files) {
    const markdown = await read(`${skill}/${filename}`);
    for (const [version] of markdown.matchAll(/\b\d+\.\d+\.\d+\b/g)) {
      assert.equal(version, data.publicVersion, `${filename}: stale Skill version`);
    }
    const installs = [...markdown.matchAll(/ankiforge(?:@|==)(\d+\.\d+\.\d+)/g)];
    for (const [, version] of installs) assert.equal(version, data.publicVersion, `${filename}: stale install version`);
    for (const [, code] of markdown.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g)) {
      assert.doesNotMatch(code, /internal-tools|path\s*=|maturin develop|git clone|\.tgz/, `${filename}: source-only installation/example`);
    }
    for (const [, link] of markdown.matchAll(/\]\(([^\s)]+)\)/g)) {
      if (link.startsWith('https://github.com/morehardy/anki-forge/blob/main/')) {
        await stat(path.join(directory, new URL(link).pathname.split('/blob/main/')[1]));
      } else if (link.startsWith(new URL(data.sitePromptUrl).origin + '/docs/')) {
        const route = new URL(link).pathname;
        const doc = importedDocs.find(entry => entry.route === route);
        await stat(path.join(directory, doc ? doc.source : `website/src/content/docs${route.replace(/\/$/, '')}.mdx`));
      } else if (!/^[a-z]+:|^#/i.test(link)) {
        await stat(path.resolve(directory, skill, path.dirname(filename), link.split('#')[0]));
      }
    }
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await verifySkill();
  console.log('Checked Skill metadata, installations, complete quickstarts and source links.');
}
