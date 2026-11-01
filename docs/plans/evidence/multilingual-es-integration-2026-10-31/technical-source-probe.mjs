import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pages, docGroups, navigation, pagePath, localeIds } from '../../../../website/site-model.mjs';
import { catalogs, validateMessages, formatCounts } from '../../../../website/src/i18n/messages.mjs';
import { loadAiQuickstart } from '../../../../website/scripts/ai-quickstart.mjs';
import { renderPrompt } from '../../../../website/src/lib/ai-quickstart.mjs';
import { reviewInputs, reviewInputFiles } from '../../../../website/scripts/content-review-inputs.mjs';

// Evidence probe only: assertions do not issue content-review approval.
const root = process.cwd();
const evidence = 'docs/plans/evidence/multilingual-es-integration-2026-10-31';
const candidateBytes = await readFile(`${evidence}/source-review-candidate.json`);
const candidate = JSON.parse(candidateBytes);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const at = (ref, filename) => execFileSync('git', ['show', `${ref}:${filename}`], { encoding: 'utf8' });
const old = filename => at('origin/main', filename);
const spanish = filename => at('codex/multilingual-spanish', filename);
const mainText = old('website/site-model.mjs');
const modelStart = mainText.indexOf('const documents = ') + 'const documents = '.length;
const mainDocuments = JSON.parse(mainText.slice(modelStart, mainText.indexOf('];', modelStart) + 1));
const esMetadata = JSON.parse(spanish('website/locale-metadata.json')).locales.es;
const reviewedDescriptionUpdates = {
  cards: 'Genera desde código tarjetas Basic de Anki de pregunta y respuesta y tarjetas Cloze con huecos. Un ejemplo verificado de Rust crea dos notas y tres tarjetas.',
  'image-occlusion': 'Crea tarjetas de oclusión de imágenes de Anki desde código con Rust, Python o Node.js: proporciona la imagen y las máscaras rectangulares y elige uno de los dos modos de oclusión.',
  media: 'Añade imágenes y audio a las tarjetas Anki que generes con Rust, Python o Node.js. Importa archivos, combina contenido y gestiona nombres de exportación, tipos MIME y límites de importación.',
  templates: 'Carga un tipo de nota de Anki reutilizable desde un paquete de plantillas, CSS y recursos multimedia, para tarjetas normales o Cloze, con Rust, Python o Node.js.',
  updates: 'Publica una nueva versión de un mazo de Anki generado: conserva los ID de nota estables, parte del APKG original anterior y revisa la comparación antes de distribuirlo.',
};
const metadata = [];
for (const document of mainDocuments) {
  const page = pages.find(page => page.id === document.name);
  assert.ok(page);
  for (const locale of ['en', 'zh-CN']) {
    assert.equal(page.title[locale], document.title[locale]);
    assert.equal(page.description[locale], document.description[locale]);
    assert.equal(await readFile(page.sources[locale], 'utf8'), old(page.sources[locale]));
  }
  assert.equal(page.title.es, esMetadata[document.name].title);
  assert.equal(page.description.es, reviewedDescriptionUpdates[document.name] ?? esMetadata[document.name].description);
  metadata.push({ id: page.id, title: page.title, description: page.description, source: page.sources.es });
}
for (const page of pages.filter(page => page.type !== 'docs')) {
  for (const locale of ['en', 'zh-CN']) assert.equal(await readFile(page.sources[locale], 'utf8'), old(page.sources[locale]));
}
for (const [filename, key] of [['website/search-terms.json', 'locales'], ['website/search-concepts.json', 'locales'], ['website/language-briefs.json', null]]) {
  const before = JSON.parse(old(filename));
  const current = JSON.parse(await readFile(filename, 'utf8'));
  for (const locale of ['en', 'zh-CN']) assert.deepEqual(key ? current[key][locale] : current[locale], key ? before[key][locale] : before[locale]);
}
const ai = await loadAiQuickstart(root);
const mainAi = JSON.parse(old('docs/ai-quickstart.json'));
assert.deepEqual(ai.languages, mainAi.languages);
assert.equal(ai.publicVersion, mainAi.publicVersion);
for (const field of ['prompts', 'requestPlaceholders', 'promptContracts']) {
  for (const locale of ['en', 'zh-CN']) assert.deepEqual(ai[field][locale], mainAi[field][locale]);
}
assert.equal(ai.prompts.es, JSON.parse(spanish('docs/ai-quickstart.json')).prompts.es);
const prompts = [];
for (const locale of localeIds) for (const sdk of ['rust', 'node', 'python']) {
  const result = renderPrompt(ai, sdk, '', locale);
  assert.ok(result.includes(ai.requestPlaceholders[locale]));
  assert.ok(result.includes(ai.languages[sdk].install));
  const route = pagePath(sdk === 'rust' ? 'quickstart' : `${sdk}-quickstart`, locale);
  assert.ok(result.includes(`https://ankiforge.dev${route}`));
  const custom = 'literal {version} <unsafe-looking-text>';
  assert.ok(renderPrompt(ai, sdk, custom, locale).includes(custom));
  prompts.push({ locale, sdk, sha256: sha(result), quickstart: route, install: ai.languages[sdk].install, words: result.split(/\s+/).length });
}
validateMessages();
validateMessages(Object.fromEntries(await Promise.all(localeIds.map(async locale => [locale, JSON.parse(await readFile(`website/src/content/i18n/${locale}.json`, 'utf8'))]))));
const sourceChecks = [];
for (const page of pages) for (const locale of localeIds) {
  const actual = await reviewInputs(page, locale, root);
  const files = await reviewInputFiles(page, locale, root);
  const entry = candidate.entries[page.id][locale];
  for (const [key, value] of Object.entries(actual)) assert.equal(entry[key], value, `${page.id}/${locale}/${key}`);
  assert.deepEqual(files, entry.inputFiles);
  sourceChecks.push({ id: page.id, locale, ...actual });
}
const codeChecks = [];
const fences = text => [...text.matchAll(/^```([^\n]*)\n([\s\S]*?)^```/gm)].map(match => ({ language: match[1], body: match[2] }));
for (const page of pages.filter(page => page.type === 'docs')) {
  const english = await readFile(page.sources.en, 'utf8');
  const translated = await readFile(page.sources.es, 'utf8');
  if (page.id === 'quickstart') {
    const code = fences(translated).find(fence => fence.language === 'rust');
    assert.equal(code.body.trim(), (await readFile('anki_forge/examples/target_api_basic.rs', 'utf8')).trim());
    assert.deepEqual(fences(translated).filter(fence => fence.language !== 'rust'), fences(english));
    assert.ok(english.includes('<CodeBlock code={quickstartSource} />'));
    codeChecks.push({ id: page.id, fences: fences(translated).length, equal: true, mode: 'MDX embeds the same canonical Rust program' });
  } else {
    assert.deepEqual(fences(translated), fences(english), page.id);
    codeChecks.push({ id: page.id, fences: fences(english).length, equal: true, mode: 'exact fenced-code pair' });
  }
}
const result = {
  reviewerRun: '/root/spanish_standards/independent-technical-source-2026-10-31',
  status: 'passed',
  scope: 'Read-only source/data invariants, not content-review approval or SDK/build/browser/live execution.',
  candidateSha256: sha(candidateBytes), locales: localeIds, topics: pages.length,
  sourceChecks, metadata, codeChecks, prompts,
  counts: [0, 1, 2, 5, 21, 1000000].map(n => ({ n, es: formatCounts('es', { notes: n, cards: n, media: n }) })),
  catalogKeys: Object.fromEntries(Object.entries(catalogs).map(([locale, catalog]) => [locale, Object.keys(catalog).length])),
  docGroups, navigation,
};
await writeFile(`${evidence}/technical-structural-probe.json`, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ status: result.status, topics: pages.length, entries: sourceChecks.length, metadata: metadata.length, codeFencePairs: codeChecks.reduce((count, check) => count + check.fences, 0), promptCases: prompts.length }));
