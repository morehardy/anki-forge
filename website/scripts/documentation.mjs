import assert from 'node:assert/strict';
import { readFile, writeFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { importedDocs } from './content-links.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const read = filename => readFile(path.join(root, filename), 'utf8');

export async function packageVersions() {
  const field = (source, section, key) => {
    const body = source.split(`[${section}]`)[1]?.split(/\n\[/)[0];
    const value = body?.match(new RegExp(`^${key} = "([^"]+)"`, 'm'))?.[1];
    assert.ok(value, `Missing ${section}.${key}`);
    return value;
  };
  return {
    rust: field(await read('anki_forge/Cargo.toml'), 'package', 'version'),
    node: JSON.parse(await read('bindings/node/package.json')).version,
    python: field(await read('bindings/python/pyproject.toml'), 'project', 'version'),
  };
}

export async function sourceSnippet(reference) {
  const [filename, region] = reference.split('#');
  assert.ok(!path.isAbsolute(filename) && !filename.split('/').includes('..'), `Use a repository path: ${reference}`);
  let source = await read(filename);
  if (region) {
    const key = region.startsWith('website:') ? region : `docs:${region}`;
    const start = `// ${key}:start\n`;
    const end = `// ${key}:end`;
    assert.equal(source.split(start).length, 2, `Missing/duplicate start: ${reference}`);
    source = source.split(start)[1];
    assert.ok(source.includes(end), `Missing end: ${reference}`);
    source = source.split(end)[0].replace(/^\n/, '').trimEnd();
    const lines = source.split('\n');
    const indent = Math.min(...lines.filter(line => line.trim()).map(line => line.match(/^ */)[0].length));
    source = lines.map(line => line.slice(indent)).join('\n');
  }
  return source.trimEnd();
}

export async function checkDocumentation({ sync = false } = {}) {
  const sources = new Set(importedDocs.map(doc => doc.source));
  let count = 0;
  for (const filename of sources) {
    let markdown = await read(filename);
    const pattern = /<!-- source: ([^\n]+) -->\n```([^\n]+)\n([\s\S]*?)\n```\n<!-- \/source -->/g;
    for (const match of [...markdown.matchAll(pattern)]) {
      const expected = await sourceSnippet(match[1]);
      if (sync) markdown = markdown.replace(match[0], `<!-- source: ${match[1]} -->\n\`\`\`${match[2]}\n${expected}\n\`\`\`\n<!-- /source -->`);
      else assert.equal(match[3], expected, `${filename}: stale example ${match[1]}; run npm run sync:docs`);
      count++;
    }
    if (sync) await writeFile(path.join(root, filename), markdown);
    const prose = markdown.replace(/```[\s\S]*?```/g, '');
    for (const [, link] of prose.matchAll(/\]\(([^\s)]+)\)/g)) {
      if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(link)) continue;
      const target = path.resolve(root, path.dirname(filename), decodeURIComponent(link.split(/[?#]/)[0]));
      assert.ok(target.startsWith(root), `${filename}: link leaves repository ${link}`);
      await stat(target).catch(() => assert.fail(`${filename}: missing repository link ${link}`));
    }
  }
  const versions = await packageVersions();
  const compatibility = await read('docs/compatibility.md');
  for (const [label, key] of [['Rust', 'rust'], ['Node / TypeScript', 'node'], ['Python', 'python']]) {
    assert.ok(compatibility.includes(`| ${label} | ${versions[key]} |`), `Compatibility table has stale ${label} version`);
  }
  for (const filename of ['README.md', 'README.zh-CN.md']) {
    const content = (await read(filename)).replace(/\s+/g, ' ');
    for (const [label, key] of [['Rust', 'rust'], ['Node', 'node'], ['Python', 'python']]) {
      assert.ok(content.includes(`${label} \`${versions[key]}\``), `${filename}: stale ${label} source version`);
    }
  }
  console.log(`Checked ${sources.size} source documents, ${count} source excerpts, repository links and SDK versions.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await checkDocumentation({ sync: process.argv.includes('--write') });
}
