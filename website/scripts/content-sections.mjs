import GithubSlugger from 'github-slugger';
import contracts from '../section-contracts.json' with { type: 'json' };
import { configuredLocales } from '../locale-model.mjs';

const marker = /^\s*(?:<!--\s*section:\s*([a-z0-9][a-z0-9-]*)\s*-->|\{\/\*\s*section:\s*([a-z0-9][a-z0-9-]*)\s*\*\/\})\s*$/;
const safeId = value => typeof value === 'string' && /^[^\s"'<>]+$/u.test(value);

// Parse actual ATX headings, never headings or markers inside source examples.
export function headings(markdown) {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  const slugger = new GithubSlugger();
  const result = [];
  let fence, pending = [], frontmatter = lines[0] === '---';
  for (let line = 0; line < lines.length; line++) {
    const text = lines[line];
    if (frontmatter) { if (line && text === '---') frontmatter = false; continue; }
    const delimiter = text.match(/^ {0,3}(`{3,}|~{3,})/);
    if (delimiter) {
      if (!fence) fence = delimiter[1];
      else if (delimiter[1][0] === fence[0] && delimiter[1].length >= fence.length && /^ {0,3}(?:`+|~+)\s*$/.test(text)) fence = undefined;
      continue;
    }
    if (fence) continue;
    const annotation = text.match(marker);
    if (annotation) { pending.push(annotation[1] || annotation[2]); continue; }
    const heading = text.match(/^ {0,3}(#{2,6})\s+(.+?)(?:\s+#+)?\s*$/);
    if (heading) {
      const title = heading[2];
      result.push({ depth: heading[1].length, id: slugger.slug(title.replace(/`|\*|<[^>]+>/g, '')), title, line, sections: pending });
      pending = [];
    } else if (pending.length && text.trim()) throw new Error(`Section marker must immediately precede a heading (line ${line + 1})`);
  }
  if (pending.length) throw new Error('Section marker has no heading');
  const declared = result.flatMap(heading => heading.sections);
  if (new Set(declared).size !== declared.length) throw new Error('Duplicate semantic section marker');
  return result;
}

export function sectionMapping(pageId, locale, markdown, legacy = {}, registry = contracts) {
  const contract = registry.entries?.[pageId];
  if (!contract) throw new Error(`${pageId}: missing section contract`);
  const translated = contract.locales?.[locale];
  const actual = headings(markdown);
  const mapping = {};
  const covered = new Set();
  for (const heading of actual) {
    const stable = heading.sections[0] || translated?.[heading.id] || heading.id;
    if (!safeId(stable)) throw new Error(`${pageId}/${locale}: invalid semantic section ${stable}`);
    mapping[heading.id] = stable;
    covered.add(stable);
    for (const section of heading.sections.slice(1)) { mapping[section] = stable; covered.add(section); }
  }
  for (const required of contract.requiredSections) {
    if (!covered.has(required)) throw new Error(`${pageId}/${locale}: missing task section ${required}; use an explicit section marker or reviewed heading mapping`);
  }
  for (const [alias, target] of Object.entries(legacy)) {
    if (!safeId(alias) || !safeId(target)) throw new Error(`${pageId}/${locale}: invalid section alias`);
    if (target !== '_top' && !covered.has(target)) throw new Error(`${pageId}/${locale}: legacy alias ${alias} lost its task ${target}`);
    if (mapping[alias] && mapping[alias] !== target) throw new Error(`${pageId}/${locale}: conflicting legacy alias ${alias}`);
    mapping[alias] = target;
  }
  return mapping;
}

// Historic heading slugs exist only for the locale that published them.
export function legacyAliases(registry, pageId, locale) {
  for (const id of Object.keys(registry)) if (!Object.hasOwn(configuredLocales, id)) throw new Error(`Unknown legacy section locale: ${id}`);
  return registry[locale]?.[pageId] ?? {};
}

export function addSectionAliases(markdown, mapping) {
  const list = headings(markdown);
  const nativeTargets = new Map(list.map(heading => [heading.id, mapping[heading.id] || heading.sections[0] || heading.id]));
  const byLine = new Map(list.map(heading => [heading.line, heading]));
  const inserted = new Set(list.map(heading => heading.id));
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  return lines.map((text, line) => {
    const heading = byLine.get(line);
    if (!heading) return text;
    const canonical = mapping[heading.id] || heading.sections[0] || heading.id;
    const aliases = new Set([canonical, ...Object.entries(mapping).filter(([, target]) => target === canonical).map(([alias]) => alias)]);
    const output = [];
    for (const alias of aliases) {
      if (!safeId(alias) || !safeId(canonical)) throw new Error('Invalid section alias');
      if (nativeTargets.has(alias) && nativeTargets.get(alias) !== canonical) throw new Error(`Section alias ${alias} conflicts with another task heading`);
      if (!inserted.has(alias)) { output.push(`<span id="${alias}" data-section-alias="${canonical}"></span>\n`); inserted.add(alias); }
    }
    return output.length ? `${output.join('\n')}\n${text}` : text;
  }).join('\n');
}
