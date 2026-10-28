import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

export const digest = value => createHash('sha256').update(value).digest('hex');
export const rubricVersion = 'anki-forge-language-quality-v1';
export const glossaryVersion = 'anki-forge-technical-terms-v1';
const templates = {
  home: ['HomePage', 'HomeShowcase', 'AiQuickstart', 'InstallCommand', 'PerformanceEvidence', 'ExampleWorkbench', 'CardPreview', 'UpdateDemo', 'CopyBlock'],
  examples: ['ExamplesPage', 'CardPreview', 'CodeBlock', 'UpdateDemo'],
  'blog-list': ['BlogPage', 'PostCard'],
  article: ['ArticlePage', 'CodeBlock'],
  docs: ['DocsHeader', 'DocsFooter', 'DocsSearch', 'CodeBlock', 'CopyBlock'],
};
const technicalFiles = ['docs/ai-quickstart.json', 'anki_forge/examples/website_showcase.rs', 'anki_forge/examples/docs_workflow.rs', 'anki_forge/examples/json_to_anki.rs', 'bindings/python/examples/json_to_anki.py', 'bindings/node/examples/json_to_anki.mts'];
const renderFiles = ['website/src/i18n/messages.mjs', 'website/src/i18n/format-message.mjs', 'website/src/i18n/client-messages.mjs', 'website/src/lib/ai-quickstart.mjs', 'website/src/lib/examples.ts', 'website/src/lib/search-terms.mjs', 'website/src/scripts/site.ts'];
const searchFiles = ['website/src/scripts/docs-search.ts', 'website/src/lib/search.mjs', 'website/search.config.mjs', 'website/scripts/search-documents.mjs'];
const repository = fileURLToPath(new URL('../../', import.meta.url));

// Read literal model data from the requested root, without executing its modules
// or reusing the main checkout's cached imports. Comments are not review inputs.
function modelDeclarations(source, filename) {
  const parsed = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
  if (parsed.statements.some(statement => ![
    ts.isImportDeclaration, ts.isExportDeclaration, ts.isVariableStatement, ts.isFunctionDeclaration, ts.isEmptyStatement,
  ].some(allowed => allowed(statement)))) throw new Error(`Review model must not mutate declaration data outside supported declarations: ${filename}`);
  return new Map(parsed.statements.filter(ts.isVariableStatement).flatMap(statement =>
    statement.declarationList.declarations.map(declaration => [declaration.name.getText(parsed), declaration.initializer])));
}

function modelImplementation(source, filename, projectedNames) {
  const parsed = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
  const printer = ts.createPrinter({ removeComments: true });
  return parsed.statements.flatMap(statement => {
    if (ts.isVariableStatement(statement)) return statement.declarationList.declarations
      .filter(declaration => !projectedNames.includes(declaration.name.getText(parsed)))
      .map(declaration => printer.printNode(ts.EmitHint.Unspecified, declaration, parsed));
    if (ts.isFunctionDeclaration(statement) || ts.isImportDeclaration(statement) || ts.isExportDeclaration(statement)) return [printer.printNode(ts.EmitHint.Unspecified, statement, parsed)];
    return [];
  });
}

function literal(node) {
  if (node && (ts.isStringLiteral(node) || ts.isNumericLiteral(node))) return ts.isStringLiteral(node) ? node.text : Number(node.text);
  if (node?.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (node?.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (node?.kind === ts.SyntaxKind.NullKeyword) return null;
  if (node && ts.isArrayLiteralExpression(node)) return node.elements.map(literal);
  if (node && ts.isObjectLiteralExpression(node)) return Object.fromEntries(node.properties.map(property => {
    if (!ts.isPropertyAssignment(property) || (!ts.isIdentifier(property.name) && !ts.isStringLiteral(property.name))) throw new Error('Review model data must use literal property names');
    return [property.name.text, literal(property.initializer)];
  }));
  throw new Error('Review model data must use literals');
}

function property(node, name) {
  if (!node || !ts.isObjectLiteralExpression(node)) throw new Error(`Missing review model object: ${name}`);
  return node.properties.find(item => ts.isPropertyAssignment(item) && item.name.text === name)?.initializer;
}

function modelArray(declarations, name, allowTransform = false) {
  let node = declarations.get(name);
  let transform, mapper;
  if (node && ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && node.expression.name.text === 'map') {
    if (!allowTransform) throw new Error(`Review model array must remain literal: ${name}`);
    transform = ts.createPrinter({ removeComments: true }).printList(ts.ListFormat.CommaListElements, node.arguments, node.getSourceFile());
    mapper = node.arguments[0];
    node = node.expression.expression;
  }
  if (!node || !ts.isArrayLiteralExpression(node)) throw new Error(`Missing literal review model array: ${name}`);
  return { nodes: node.elements, transform, mapper };
}

function renderedModelInputs(files, page, locale) {
  const model = modelDeclarations(files['website/site-model.mjs'], 'site-model.mjs');
  const registry = modelDeclarations(files['website/locale-model.mjs'], 'locale-model.mjs').get('configuredLocales');
  const selected = JSON.parse(files['website/release-locales.json']).locales;
  const localeData = id => literal(property(registry, id));
  const current = localeData(locale);
  const documents = modelArray(model, 'documents').nodes.map(literal);
  const common = modelArray(model, 'common').nodes.map(node => ({ id: literal(property(node, 'id')), type: literal(property(node, 'type')), route: literal(property(node, 'route')) }));
  const routes = new Map([...common.map(item => [item.id, item.route]), ...documents.map(item => [item.name, item.route])]);
  const articles = modelArray(model, 'articles', true);
  const destination = id => {
    const route = routes.get(id);
    if (route) return { id, route: current.prefix + route };
    if (articles.nodes.some(node => `blog:${literal(node)}` === id)) return { id, prefix: current.prefix, articleRouteDefinition: articles.transform };
    throw new Error(`Unknown review model destination: ${id}`);
  };
  const values = name => modelArray(model, name).nodes.map(literal);
  const ownDocument = documents.find(document => document.name === page.id);
  const ownCommon = common.find(item => item.id === page.id);
  const catalog = JSON.parse(files[`website/src/i18n/${locale}.json`]);
  const localized = (document, field) => locale === 'en' ? document[field]
    : locale === 'zh-CN' ? document[`zh${field[0].toUpperCase()}${field.slice(1)}`]
      : document[`${field}s`]?.[locale];
  const result = {
    implementation: {
      site: modelImplementation(files['website/site-model.mjs'], 'site-model.mjs', ['documents', 'common', 'navigation', 'footerNavigation', 'nextTasks', 'docGroups', 'articles']),
      locales: modelImplementation(files['website/locale-model.mjs'], 'locale-model.mjs', ['configuredLocales']),
    },
    metadata: {
      id: page.id, type: ownDocument ? 'docs' : ownCommon?.type ?? 'article', destination: destination(page.id),
      // Article metadata is already part of its exact source/frontmatter bytes.
      title: ownDocument ? localized(ownDocument, 'title') : ownCommon ? catalog[`${page.id}.title`] : undefined,
      description: ownDocument ? localized(ownDocument, 'description') : ownCommon ? catalog[`${page.id}.description`] : undefined,
    },
    locale: { current, menu: selected.map(id => { const entry = localeData(id); return { id, label: entry.label, prefix: entry.prefix, lang: entry.lang, dir: entry.dir }; }) },
    wordmark: destination('home'),
    navigation: values('navigation').map(item => ({ ...destination(item.id), label: item.label[locale] })),
    footer: values('footerNavigation').map(item => ({ ...item, ...(item.pageId ? { destination: destination(item.pageId) } : {}) })),
  };
  if (page.type === 'home') result.nextTasks = values('nextTasks').map(item => ({ ...item, destination: destination(item.pageId) }));
  if (page.type === 'docs') {
    const groups = modelArray(model, 'docGroups', true);
    const body = groups.mapper?.body;
    const labels = property(body && ts.isParenthesizedExpression(body) ? body.expression : body, 'labels');
    const label = property(labels, locale);
    if (!label) throw new Error(`Missing review sidebar label mapping: ${locale}`);
    result.sidebar = { transform: groups.transform, groups: groups.nodes.map(literal).map(group => ({
      label: ts.isPropertyAccessExpression(label) && ts.isIdentifier(label.expression) && label.expression.text === groups.mapper.parameters[0]?.name.text ? group[label.name.text] : literal(label),
      collapsed: group.collapsed ?? false,
      items: group.ids.map(id => {
        const document = documents.find(item => item.name === id);
        if (!document) throw new Error(`Unknown review sidebar document: ${id}`);
        return { ...destination(id), label: locale === 'en' ? document.title : locale === 'zh-CN' ? document.zhTitle : document.titles?.[locale] };
      }),
    })) };
  }
  return result;
}

function repositoryReference(value) {
  const filename = value.split('#')[0];
  if (path.isAbsolute(filename) || filename.split('/').includes('..') || !filename) throw new Error(`Review input must be a repository file: ${value}`);
  return filename;
}

function sourceReferences(source) {
  const references = [];
  let fence;
  for (const line of source.split(/\r?\n/)) {
    const delimiter = line.match(/^ {0,3}(`{3,}|~{3,})/);
    if (delimiter) {
      if (!fence) fence = delimiter[1];
      else if (delimiter[1][0] === fence[0] && delimiter[1].length >= fence.length && /^ {0,3}(?:`+|~+)\s*$/.test(line)) fence = undefined;
      continue;
    }
    if (!fence) {
      const directive = line.match(/^\s*<!-- source: ([^\s]+) -->\s*$/);
      if (directive) references.push(repositoryReference(directive[1]));
    }
  }
  return references;
}

async function astroClosure(initial, root) {
  const seen = new Set();
  async function visit(filename) {
    if (seen.has(filename)) return;
    seen.add(filename);
    const source = await readFile(path.join(root, filename), 'utf8');
    for (const match of source.matchAll(/\bimport\s+(?:[^;\n]+?\s+from\s+)?['"]([^'"]+\.astro)['"]/g)) {
      if (!match[1].startsWith('.')) continue;
      const target = path.posix.normalize(path.posix.join(path.posix.dirname(filename), match[1]));
      repositoryReference(target);
      await visit(target);
    }
  }
  for (const filename of initial) await visit(filename);
  return [...seen];
}

export async function reviewInputFiles(page, locale, root = repository) {
  const sources = [...new Set([page.sources.en, page.sources[locale]])];
  const references = (await Promise.all(sources.map(async filename => sourceReferences(await readFile(path.join(root, filename), 'utf8'))))).flat();
  const templateFiles = await astroClosure([
    'website/src/layouts/SiteLayout.astro', 'website/src/components/Header.astro', 'website/src/components/Footer.astro',
    ...(templates[page.type] || []).map(name => `website/src/components/${name}.astro`),
  ], root);
  const targetedTechnical = [];
  if (['quickstart', 'blog:your-first-anki-deck'].includes(page.id)) targetedTechnical.push('anki_forge/examples/target_api_basic.rs');
  if (page.type === 'home') {
    const benchmark = JSON.parse(await readFile(path.join(root, 'docs/benchmark-presentation.json'), 'utf8'));
    if (!/^docs\/performance\/\d{8}\/comparison\.csv$/.test(benchmark.dataFile)) throw new Error('Invalid benchmark review data file');
    targetedTechnical.push('docs/benchmark-presentation.json', benchmark.dataFile, 'website/scripts/benchmark-presentation.mjs');
  }
  return [...new Set([
    ...sources, 'website/src/i18n/en.json', `website/src/i18n/${locale}.json`,
    'website/site-model.mjs', 'website/locale-model.mjs', 'website/release-locales.json', 'website/search-terms.json', 'website/section-contracts.json', 'website/legacy-sections.json', 'website/language-briefs.json',
    ...templateFiles,
    ...(page.type === 'docs' ? ['website/src/route-middleware.ts', 'website/src/content/i18n/en.json', `website/src/content/i18n/${locale}.json`, 'website/search-concepts.json', ...searchFiles] : []),
    ...technicalFiles, ...renderFiles, ...targetedTechnical, ...references,
  ])].sort();
}

// Source review hashes intentionally exclude review records and build outputs.
// Artifact-specific search/browser/machine evidence is attached after building.
export async function reviewInputs(page, locale, root) {
  const read = name => readFile(path.join(root, name), 'utf8');
  const files = Object.fromEntries(await Promise.all((await reviewInputFiles(page, locale, root)).map(async name => [name, await read(name)])));
  const sections = JSON.parse(files['website/section-contracts.json']).entries[page.id] || null;
  const sourceModel = renderedModelInputs(files, page, 'en');
  const targetModel = locale === 'en' ? sourceModel : renderedModelInputs(files, page, locale);
  const searchTerms = JSON.parse(files['website/search-terms.json']);
  const searchConcepts = files['website/search-concepts.json'] ? JSON.parse(files['website/search-concepts.json']).locales : {};
  const sourceHash = digest(JSON.stringify({ source: files[page.sources.en], metadata: sourceModel.metadata, model: sourceModel, catalog: files['website/src/i18n/en.json'], frameworkCatalog: files['website/src/content/i18n/en.json'], searchTerms: searchTerms.locales.en?.[page.id], searchConcepts: searchConcepts.en }));
  const targetHash = digest(JSON.stringify({ source: files[page.sources[locale]], metadata: targetModel.metadata, model: targetModel, catalog: files[`website/src/i18n/${locale}.json`], frameworkCatalog: files[`website/src/content/i18n/${locale}.json`], searchTerms: searchTerms.locales[locale]?.[page.id], searchConcepts: searchConcepts[locale], templates: Object.entries(files).filter(([name]) => name.endsWith('.astro') || name.endsWith('route-middleware.ts') || renderFiles.includes(name) || searchFiles.includes(name)) }));
  const briefs = JSON.parse(files['website/language-briefs.json']);
  if (!briefs[locale] || briefs.version !== glossaryVersion) throw new Error(`${locale}: missing or stale language brief`);
  const references = new Set([...sourceReferences(files[page.sources.en]), ...sourceReferences(files[page.sources[locale]])]);
  const technicalInputs = Object.keys(files).filter(name => technicalFiles.includes(name) || name === 'anki_forge/examples/target_api_basic.rs' || name === 'docs/benchmark-presentation.json' || name === 'website/scripts/benchmark-presentation.mjs' || /^docs\/performance\//.test(name) || references.has(name));
  const technicalDependencyHash = digest(JSON.stringify({ sections, legacy: JSON.parse(files['website/legacy-sections.json'])[page.id] || null, brief: briefs[locale], technical: technicalInputs.map(name => [name, digest(files[name])]) }));
  return { sourceHash, targetHash, technicalDependencyHash };
}
