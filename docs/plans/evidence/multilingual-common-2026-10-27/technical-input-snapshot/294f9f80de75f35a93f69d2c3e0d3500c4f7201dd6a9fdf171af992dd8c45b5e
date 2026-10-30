import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { load } from 'cheerio';
import { assertIndexable } from './indexability.mjs';
import { assertValidatedHeadingMigrations } from './heading-migrations.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const text = value => value.replace(/\s+/g, ' ').trim();
const selectors = { downloads: '[download]', search: 'site-search', codeCopy: '[data-copy], .expressive-code .copy button', flips: '[data-flip]', audio: 'audio', updates: '[data-update]' };
export function artifactPath(dist, route, base = '/') {
  const prefix = base === '/' ? '' : base.replace(/\/$/, '');
  assert.ok(route.startsWith(prefix + '/'), `Route outside configured base: ${route}`);
  const relative = decodeURIComponent(route.slice(prefix.length));
  const file = path.resolve(dist, `.${relative}${relative.endsWith('/') ? 'index.html' : ''}`);
  assert.ok(file.startsWith(path.resolve(dist) + path.sep), `Route outside artifact: ${route}`);
  return file;
}
function headingText($, node) {
  return text($(node).clone().find('.sl-anchor-link').remove().end().text());
}
// An alias belongs next to its task heading, rather than in a collection at the top.
function aliasHeadingNode($, node) {
  let block = $(node);
  if (block.parent().is('p') && !block.parent().text().trim()) block = block.parent();
  for (let next = block.next(); next.length; next = next.next()) {
    const heading = next.is('h2,h3,h4,h5,h6') ? next : next.children('h2,h3,h4,h5,h6');
    if (heading.length) return heading.first();
    if (next.text().trim()) return null;
  }
  return null;
}
function aliasHeading($, node) {
  const heading = aliasHeadingNode($, node);
  return heading ? headingText($, heading) : null;
}
function anchors($) {
  return $('main[id], main [id]').toArray().map(node => {
    const alias = $(node).attr('data-section-alias');
    const heading = $(node).is('h1,h2,h3,h4,h5,h6') ? headingText($, node) : alias ? aliasHeading($, node) : undefined;
    const containerModule = $(node).closest('[data-module]').attr('data-module');
    return { id: $(node).attr('id'), ...(alias ? { task: alias } : {}), ...(heading !== undefined ? { heading } : {}), ...($(node).is('h1,h2,h3,h4,h5,h6') ? { headingLevel: node.tagName.toLowerCase() } : {}), ...(containerModule ? { containerModule } : {}) };
  });
}
function entityIds($) {
  const graphs = $('script[type="application/ld+json"][data-anki-forge-schema]').toArray();
  return graphs.flatMap(node => (JSON.parse($(node).text())['@graph'] || []).map(entity => entity['@id'])).filter(Boolean).sort();
}

/** Explicit baseline capture only: never invoked by a normal build. */
export async function captureSurface(dist, verifiedManifest, provenance) {
  const manifestBytes = await readFile(verifiedManifest);
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.configuration.base, '/', 'Capture the published root artifact, not a preview');
  const pages = [];
  for (const entry of manifest.pages.filter(page => page.id !== '404')) {
    const bytes = await readFile(artifactPath(dist, entry.route));
    assert.equal(hash(bytes), entry.sha256, `${entry.route}: baseline does not match independent verified manifest`);
    const $ = load(bytes.toString());
    assertIndexable($, { route: entry.route });
    pages.push({ route: entry.route, pageId: entry.id, locale: entry.locale, anchors: anchors($), capabilities: Object.fromEntries(Object.entries(selectors).map(([name, selector]) => [name, $(selector).length])), entities: entityIds($), ...( $('link[rel="describedby"]').length ? { llms: new URL($('link[rel="describedby"]').attr('href')).pathname } : {}), ...( $('link[rel="alternate"][type="text/markdown"]').length ? { markdown: new URL($('link[rel="alternate"][type="text/markdown"]').attr('href')).pathname } : {}) });
  }
  for (const entry of manifest.resources) {
    assert.equal(hash(await readFile(artifactPath(dist, entry.route))), entry.sha256, `${entry.route}: baseline resource differs from verified manifest`);
  }
  return { schemaVersion: 1, source: { ...provenance, commit: manifest.commit, inputSha256: manifest.inputSha256, manifestSha256: hash(manifestBytes) }, locales: [...new Set(pages.map(page => page.locale))], pages, machineResources: manifest.resources.filter(resource => /\/(?:index\.md|llms\.txt)$/.test(resource.route)).map(resource => ({ route: resource.route, kind: resource.route.endsWith('llms.txt') ? 'llms' : 'markdown' })) };
}

/** Compare with an independent historic surface, not today's topic registry. */
export async function checkSurface(dist, baseline, configuration, headingMigrations = []) {
  assertValidatedHeadingMigrations(headingMigrations);
  assert.equal(baseline.schemaVersion, 1, 'Unknown public surface schema');
  for (const expected of baseline.pages) {
    const route = (configuration.base === '/' ? '' : configuration.base) + expected.route;
    const $ = load(await readFile(artifactPath(dist, route, configuration.base), 'utf8'));
    assert.equal($('meta[name="anki-forge:page-id"]').attr('content'), expected.pageId, `${route}: historic page identity changed`);
    assert.equal($('html').attr('lang'), expected.locale, `${route}: historic language changed`);
    assert.equal($('link[rel="canonical"]').length, 1, `${route}: canonical count`);
    assert.equal($('link[rel="canonical"]').attr('href'), new URL(route, configuration.site).href, `${route}: historic canonical changed`);
    assertIndexable($, { route });
    const actual = new Map(anchors($).map(anchor => [anchor.id, anchor]));
    function checkHeading(anchor, found, message) {
      if (found.heading === anchor.heading) return;
      const migration = headingMigrations.find(entry => entry.pageId === expected.pageId && entry.locale === expected.locale && entry.route === expected.route && entry.anchors.includes(anchor.id) && entry.fromHeading === anchor.heading && entry.toHeading === found.heading);
      assert.ok(migration, message);
      assert.equal(anchor.containerModule, migration.containerModule, `${route}#${anchor.id}: migration changed its historic module`);
      const nodes = $('main[id], main [id]').filter((_, node) => $(node).attr('id') === anchor.id);
      assert.equal(nodes.length, 1, `${route}#${anchor.id}: migrated anchor must remain unique`);
      const container = nodes.closest('[data-module]');
      assert.equal(container.attr('data-module'), migration.containerModule, `${route}#${anchor.id}: migrated anchor moved to another module`);
      const modules = $('[data-module]').filter((_, node) => $(node).attr('data-module') === migration.containerModule);
      assert.equal(modules.length, 1, `${route}#${anchor.id}: migrated module must remain unique`);
      const heading = nodes.is('h2,h3,h4,h5,h6') ? nodes : nodes.attr('data-section-alias') ? aliasHeadingNode($, nodes) : null;
      assert.ok(heading && heading.closest('[data-module]')[0] === container[0], `${route}#${anchor.id}: migrated alias is detached from its task heading`);
      for (const required of migration.requiredLinks) {
        const destination = new URL((configuration.base === '/' ? '' : configuration.base) + required, configuration.site);
        assert.ok(container.find('a[href]').toArray().some(node => {
          const target = new URL($(node).attr('href'), new URL(route, configuration.site));
          return target.href === destination.href;
        }), `${route}#${anchor.id}: migrated module lost protected local link ${required}`);
      }
    }
    for (const anchor of expected.anchors) {
      assert.ok(actual.has(anchor.id), `${route}: historic anchor missing: ${anchor.id}`);
      if (anchor.containerModule) assert.equal(actual.get(anchor.id).containerModule, anchor.containerModule, `${route}#${anchor.id}: historic anchor moved to another module`);
      if (anchor.heading && !anchor.task) {
        if (anchor.headingLevel === 'h1') {
          // The page's top anchor keeps its role even when an independently
          // reviewed SEO title changes. It cannot move to a loose span or task.
          assert.equal(actual.get(anchor.id).headingLevel, 'h1', `${route}#${anchor.id}: historic page-title anchor moved away from its H1`);
        } else {
          checkHeading(anchor, actual.get(anchor.id), `${route}#${anchor.id}: historic anchor moved away from its task heading; independently review a heading migration`);
        }
      }
      if (anchor.task) {
        const found = actual.get(anchor.id);
        assert.equal(found.task, anchor.task, `${route}#${anchor.id}: historic task changed`);
        if (anchor.task === '_top') {
          const order = $('main *').toArray();
          const anchorIndex = order.findIndex(node => $(node).attr('id') === anchor.id);
          const firstTask = order.findIndex(node => $(node).is('h2,h3,h4,h5,h6'));
          assert.ok(firstTask === -1 || anchorIndex < firstTask, `${route}#${anchor.id}: historic introductory task moved below another task`);
        } else {
          assert.ok(found.heading, `${route}#${anchor.id}: alias is detached from a real task heading`);
          checkHeading(anchor, found, `${route}#${anchor.id}: historic alias moved to another task; independently review a heading migration`);
        }
      }
    }
    for (const [name, count] of Object.entries(expected.capabilities)) assert.ok($(selectors[name]).length >= count, `${route}: historic ${name} capability removed`);
    for (const [key, selector] of [['llms', 'link[rel="describedby"]'], ['markdown', 'link[rel="alternate"][type="text/markdown"]']]) {
      if (expected[key]) assert.equal($(selector).attr('href'), new URL((configuration.base === '/' ? '' : configuration.base) + expected[key], configuration.site).href, `${route}: historic ${key} discovery removed`);
    }
    const actualEntities = entityIds($);
    for (const id of expected.entities) {
      const url = new URL(id);
      const localized = new URL((configuration.base === '/' ? '' : configuration.base) + url.pathname + url.hash, configuration.site).href;
      assert.ok(actualEntities.includes(localized), `${route}: historic entity ID changed: ${id}`);
    }
  }
  for (const resource of baseline.machineResources) {
    const route = (configuration.base === '/' ? '' : configuration.base) + resource.route;
    assert.ok((await stat(artifactPath(dist, route, configuration.base))).size > 0, `${route}: historic machine resource missing`);
  }
  return { pages: baseline.pages.length, machineResources: baseline.machineResources.length, source: baseline.source.commit };
}

export function releasedLanguages(baseline) { return [...baseline.locales]; }
