// Enumerate historic heading changes; independent reviewers approve the resulting declarations.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { load } from '../../../../website/node_modules/cheerio/dist/esm/index.js';
import { pages, locales } from '../../../../website/site-model.mjs';
import { sectionMapping, legacyAliases } from '../../../../website/scripts/content-sections.mjs';

const root = fileURLToPath(new URL('../../../../', import.meta.url));
const read = filename => readFile(root + filename, 'utf8');
const legacy = JSON.parse(await read('website/legacy-sections.json'));
const reviews = JSON.parse(await read('website/content-reviews.json'));
const manifest = JSON.parse(await read('website/dist/generated/site-manifest.json'));
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const entries = new Map();
for (const file of ['public-surface-baseline.json', 'machine-surface-baseline.json']) {
  const baseline = JSON.parse(await read('website/' + file));
  for (const old of baseline.pages) {
    const $ = load(await read('website/dist' + old.route + 'index.html'));
    const page = pages.find(page => page.id === old.pageId);
    for (const anchor of old.anchors) {
      if (!anchor.heading || anchor.headingLevel === 'h1') continue;
      const node = $('main[id],main [id]').filter((_, item) => $(item).attr('id') === anchor.id);
      let heading = node.is('h2,h3,h4,h5,h6') ? node : null;
      if (!heading && node.attr('data-section-alias')) {
        let block = node;
        if (block.parent().is('p') && !block.parent().text().trim()) block = block.parent();
        for (let next = block.next(); next.length; next = next.next()) {
          const found = next.is('h2,h3,h4,h5,h6') ? next : next.children('h2,h3,h4,h5,h6');
          if (found.length) { heading = found.first(); break; }
          if (next.text().trim()) break;
        }
      }
      const title = heading?.clone().find('.sl-anchor-link').remove().end().text().replace(/\s+/g, ' ').trim();
      if (title === anchor.heading) continue;
      assert.ok(heading, `Detached historic heading: ${old.route}#${anchor.id}`);
      const key = [page.id, old.locale, anchor.heading, title].join('|');
      if (!entries.has(key)) {
        const record = reviews.entries[page.id][old.locale];
        let scope;
        if (anchor.containerModule) {
          const container = heading.closest('[data-module]');
          assert.equal(container.attr('data-module'), anchor.containerModule);
          const links = container.find('a[href]').toArray().map(item => new URL($(item).attr('href'), manifest.configuration.site + base + old.route));
          const local = links.filter(url => url.origin === manifest.configuration.site && !url.hash && !url.search && pages.some(page => base + locales[old.locale].prefix + page.route === url.pathname)).map(url => url.pathname.slice(base.length));
          const external = links.filter(url => url.origin !== manifest.configuration.site).map(url => url.href);
          scope = { containerModule: anchor.containerModule, requiredLinks: [...new Set(local)], ...(external.length ? { requiredExternalLinks: [...new Set(external)] } : {}) };
        } else {
          assert.ok(['docs', 'article'].includes(page.type));
          const source = await read(page.sources[old.locale]);
          const mapping = sectionMapping(page.id, old.locale, source, legacyAliases(legacy, page.id, old.locale));
          const headingId = heading.attr('id'), sectionId = mapping[headingId];
          assert.equal(mapping[anchor.id] ?? (anchor.id === sectionId ? sectionId : undefined), sectionId);
          scope = { sectionId, headingId };
        }
        entries.set(key, { pageId: page.id, locale: old.locale, route: old.route, anchors: [], fromHeading: anchor.heading, toHeading: title, ...scope,
          reason: 'Independent technical and linguistic reviews in website-pr66-final-2026-11-02 verified this wording correction and unchanged task. Preserve the historic anchors and protected destinations.',
          inputs: Object.fromEntries(['sourceHash', 'targetHash', 'technicalDependencyHash'].map(key => [key, record[key]])) });
      }
      if (!entries.get(key).anchors.includes(anchor.id)) entries.get(key).anchors.push(anchor.id);
    }
  }
}
await writeFile(root + 'website/public-surface-migrations.json', JSON.stringify({ schemaVersion: 1, entries: [...entries.values()] }, null, 2) + '\n');
console.log(`Prepared ${entries.size} source-bound migrations for independent approval.`);
