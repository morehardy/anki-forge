/** Browser-compatible, catalog-scoped retrieval. Vocabulary is supplied by locale. */
const HAN = /\p{Script=Han}/u;
const LATIN = /^[a-z]+$/;

function normalize(value) {
  return String(value ?? '').normalize('NFKC').toLowerCase().replace(/node\.js/g, 'nodejs')
    .normalize('NFD').replace(/\p{M}/gu, '').replace(/[’']/g, '');
}

function stem(value, locale) {
  if (!String(locale).startsWith('en') || !LATIN.test(value) || value.length < 5) return value;
  let base = value;
  if (base.endsWith('ies')) return `${base.slice(0, -3)}y`;
  if (base.endsWith('ing') && base.length > 6) base = base.slice(0, -3);
  else if (base.endsWith('ed') && base.length > 5) base = base.slice(0, -2);
  else if (base.endsWith('s') && !/(ss|us|is)$/.test(base)) base = base.slice(0, -1);
  // The same terminal-e rule on source and inflection pairs creates/creating and
  // updates/updated without a vocabulary of exceptional query strings.
  if (base.endsWith('e') && !base.endsWith('ee') && base.length > 4) base = base.slice(0, -1);
  if (base !== value && /([b-df-hj-np-tv-z])\1$/.test(base) && !/(ss|ll)$/.test(base)) base = base.slice(0, -1);
  return base;
}

function tokens(value, locale, forms) {
  const input = normalize(value);
  const words = typeof Intl.Segmenter === 'function'
    ? [...new Intl.Segmenter(locale, { granularity: 'word' }).segment(input)].filter(part => part.isWordLike).map(part => part.segment)
    : input.match(/[\p{L}\p{N}_]+/gu) ?? [];
  const output = new Set();
  for (const word of words) {
    // ASCII identifiers remain searchable both whole and by snake_case parts.
    for (const part of [word, ...word.split('_')]) {
      if (!/\p{L}/u.test(part)) continue; // Numeric coincidence is never retrieval evidence.
      const token = stem(part, locale);
      output.add(token);
      forms?.set(token, part);
      if (HAN.test(part)) {
        const chars = [...part];
        // Compound overlap supplements Segmenter; single Han characters are too broad.
        for (let index = 0; index < chars.length - 1; index++) {
          const compound = chars.slice(index, index + 2).join('');
          output.add(compound);
          forms?.set(compound, compound);
        }
      }
    }
  }
  return output;
}

/** Strip Pagefind highlight markup. Callers must still render text with textContent. */
export function plainSearchText(value) {
  return String(value ?? '').replace(/<[^>]*>/g, '').replace(/&(?:#(x[\da-f]+|\d+)|amp|lt|gt|quot|apos|nbsp);/gi, (entity, numeric) => {
    if (numeric) {
      const code = numeric[0].toLowerCase() === 'x' ? parseInt(numeric.slice(1), 16) : Number(numeric);
      return code > 0 && code <= 0x10ffff ? String.fromCodePoint(code) : '';
    }
    return ({ '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"', '&apos;': "'", '&nbsp;': ' ' })[entity.toLowerCase()] ?? entity;
  }).replace(/\s+/g, ' ').trim();
}

function pagePath(url) {
  try { return new URL(url, 'https://search.invalid').pathname.replace(/\/$/, '') || '/'; }
  catch { return ''; }
}

/** Stable content roles, independent of URL layout, language and deployment base. */
export function documentRole(document) {
  if (document.type === 'home') return 'overview';
  if (document.type === 'examples') return 'examples';
  if (document.type === 'blog-list') return 'collection';
  const id = document.id;
  if (id === 'anki-forge-vs-genanki') return 'comparison';
  if (id?.endsWith('migration')) return 'migration';
  if (id === 'api' || id?.endsWith('-api')) return 'api';
  if (id === 'quickstart' || id?.endsWith('-quickstart') || id === 'blog:your-first-anki-deck') return 'quickstart';
  if (id === 'installation') return 'installation';
  if (id === 'development') return 'development';
  if (id === 'troubleshooting' || id?.endsWith('-diagnostics')) return 'diagnostics';
  if (id === 'updates' || id === 'blog:stable-ids-and-deck-updates') return 'updates';
  if (id === 'languages' || id === 'docs') return 'overview';
  if (id === 'concepts') return 'concepts';
  if (id === 'cards') return 'cards';
  if (id === 'image-occlusion') return 'occlusion';
  if (id === 'media') return 'media';
  if (id === 'templates') return 'templates';
  if (id === 'custom-notetypes') return 'note-types';
  if (id === 'compatibility') return 'compatibility';
  return 'guide';
}

export function documentSdk(document) {
  if (/^python-/.test(document.id)) return 'python';
  if (/^node-/.test(document.id)) return 'node';
  if (/^rust-/.test(document.id) || ['quickstart', 'installation', 'blog:your-first-anki-deck'].includes(document.id)) return 'rust';
  return null;
}

function aliasPattern(value) {
  const escaped = normalize(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  // Han boundaries do not follow whitespace; Latin words require lexical boundaries.
  const start = /^[\p{Script=Latin}\p{N}_]/u.test(escaped) ? '(?<![\\p{Script=Latin}\\p{N}_])' : '';
  const end = /[\p{Script=Latin}\p{N}_]$/u.test(escaped) ? '(?![\\p{Script=Latin}\\p{N}_])' : '';
  return new RegExp(`${start}${escaped}${end}`, 'gu');
}

function conceptDefinitions(concepts) {
  return Object.entries(concepts).filter(([id, value]) => !id.startsWith('__') && Array.isArray(value?.terms))
    .map(([id, value]) => ({ id, ...value,
      context: (value.requiresAny ?? []).map(aliasPattern),
      contextGroups: (value.requiresAll ?? []).map(group => group.map(aliasPattern)),
      aliases: value.terms.filter(term => typeof term === 'string' && term.trim()).map(term => ({ term: normalize(term), pattern: aliasPattern(term) })) }));
}

function includesConcept(text, definition) {
  if (!hasContext(text, definition)) return false;
  return definition.aliases.some(alias => { alias.pattern.lastIndex = 0; return alias.pattern.test(text); });
}

function hasContext(text, definition) {
  const matches = pattern => { pattern.lastIndex = 0; return pattern.test(text); };
  return (!definition.context.length || definition.context.some(matches))
    && definition.contextGroups.every(group => group.some(matches));
}

function queryFeatures(query, definitions, stopwords, locale) {
  let remaining = normalize(query);
  const fullQuery = remaining;
  const concepts = [];
  // Consume longer aliases first so "image occlusion" is not reduced to "image".
  const matches = definitions.flatMap(definition => definition.aliases.map(alias => ({ definition, alias })))
    .sort((left, right) => right.alias.term.length - left.alias.term.length
      || (right.definition.context.length + right.definition.contextGroups.length) - (left.definition.context.length + left.definition.contextGroups.length));
  for (const { definition, alias } of matches) {
    if (!hasContext(fullQuery, definition)) continue;
    alias.pattern.lastIndex = 0;
    if (!alias.pattern.test(remaining)) continue;
    if (!concepts.some(item => item.id === definition.id)) concepts.push(definition);
    remaining = remaining.replace(alias.pattern, ' ');
  }
  // Some predicates and their subject form one semantic feature (e.g. package
  // structure). Counting the subject again would overweight recipe keywords and
  // penalize an abstract concepts title for expressing that topic differently.
  for (const definition of concepts.filter(item => item.consumeContext)) {
    for (const pattern of [...definition.context, ...definition.contextGroups.flat()]) remaining = remaining.replace(pattern, ' ');
  }
  const forms = new Map();
  const lexical = [...tokens(remaining, locale, forms)].filter(token => !stopwords.has(token) && (!HAN.test(token) || [...token].length > 1));
  return [...concepts.map(definition => ({ key: `concept:${definition.id}`, label: definition.terms[0], definition })),
    ...lexical.map(token => ({ key: `word:${token}`, label: forms.get(token), token }))];
}

/**
 * Rank only the supplied locale's real documents. Pagefind contributes body evidence
 * and snippets; it cannot introduce URLs, and fuzzy/numeric-only hits are rejected.
 * Score = field evidence × inverse document frequency × matched query coverage,
 * with stable task-role and SDK priors. Unknown words are ignored, never AND gates.
 */
export function rankSearchDocuments(query, documents, locale, { pagefindResults = [], concepts = {} } = {}) {
  if (typeof query !== 'string' || !query.trim() || !Array.isArray(documents) || !documents.length) return [];
  const definitions = conceptDefinitions(concepts);
  const stopwords = new Set((concepts.__language?.stopWords ?? []).map(word => stem(normalize(word), locale)));
  const features = queryFeatures(query, definitions, stopwords, locale);
  if (!features.length) return [];
  const pagefind = new Map();
  for (const result of pagefindResults) {
    const key = pagePath(result?.url);
    if (key && !pagefind.has(key)) pagefind.set(key, result);
  }
  const records = documents.map(document => {
    const body = pagefind.get(pagePath(document.url));
    const fields = [document.title, document.description, (document.terms ?? []).join(' '), plainSearchText(body?.content ?? body?.excerpt ?? ''), document.pageTitle].map(normalize);
    const tokenSets = fields.map(field => tokens(field, locale));
    const evidence = features.map(feature => fields.reduce((best, field, index) => {
      const matched = feature.definition ? includesConcept(field, feature.definition) : tokenSets[index].has(feature.token);
      // HTML titles recover navigation names even when the visible H1 is editorial.
      // Their repeated site branding stays weaker than a document's own H1.
      return matched ? Math.max(best, [5, 3, 2.5, 0.9, 2.5][index]) : best;
    // A curated semantic role is weak evidence, not a fabricated text match.
    // It lets definition/structure intent find concepts whose title is abstract.
    }, feature.definition?.roles?.includes(documentRole(document)) ? 1.5 : 0));
    return { document, body, evidence };
  });
  const frequencies = features.map((_, index) => records.filter(record => record.evidence[index] > 0).length);
  const idfs = frequencies.map((frequency, index) => Math.log(1 + (records.length + 1) / (frequency + 0.5)) * (features[index].definition?.weight ?? 1));
  // Only corpus-supported features enter coverage. A novel adjective cannot hide a
  // good exact match, but novel strings without any useful evidence return nothing.
  const known = features.map((_, index) => index).filter(index => frequencies[index] > 0);
  if (!known.length) return [];
  const totalWeight = known.reduce((sum, index) => sum + idfs[index], 0);
  const intentFeatures = features.filter(feature => feature.definition?.intent);
  const roles = new Set((intentFeatures.length ? intentFeatures : features).flatMap(feature => feature.definition?.roles ?? []));
  const sdks = new Set(features.map(feature => feature.definition?.sdk).filter(Boolean));
  return records.flatMap(({ document, body, evidence }) => {
    const matched = known.filter(index => evidence[index] > 0);
    if (!matched.length) return [];
    let score = matched.reduce((sum, index) => sum + evidence[index] * idfs[index], 0);
    const coverage = matched.reduce((sum, index) => sum + idfs[index], 0) / totalWeight;
    score *= 0.2 + 0.8 * coverage ** 2;
    const role = documentRole(document);
    const sdk = documentSdk(document);
    if (roles.size) score *= roles.has(role) ? (intentFeatures.length ? 2.4 : 1.8) : (intentFeatures.length ? 0.7 : 0.85);
    // Generic questions should lead to the cross-SDK troubleshooting guide.
    // Naming a language leaves the existing SDK prior in charge instead.
    if (!sdks.size && roles.has('diagnostics') && role === 'diagnostics') score *= sdk ? 0.45 : 1.8;
    if (sdks.size === 1) {
      const desired = [...sdks][0];
      if (sdk === desired) score *= 1.6;
      else if (sdk) score *= 0.4;
      // A bare language/SDK lookup normally starts at that SDK's quickstart.
      if (!roles.size && role === 'quickstart' && sdk === desired) score *= 1.35;
    }
    const excerpt = plainSearchText(body?.excerpt || document.description);
    return [{ ...document, score, matchedTerms: matched.map(index => features[index].label), excerpt }];
  }).sort((left, right) => right.score - left.score || left.url.localeCompare(right.url));
}
