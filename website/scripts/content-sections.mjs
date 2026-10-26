import GithubSlugger from 'github-slugger';

export function headings(markdown) {
  const slugger = new GithubSlugger();
  return [...markdown.replace(/^---\n[\s\S]*?\n---\n/, '').replace(/```[\s\S]*?```/g, '').matchAll(/^(#{2,6}) (.+)$/gm)].map(match => ({
    depth: match[1].length,
    id: slugger.slug(match[2].replace(/`|\*|<[^>]+>/g, '')),
  }));
}

export function sectionMap(english, translated, legacy = {}) {
  const en = headings(english);
  const zh = headings(translated);
  if (JSON.stringify(en.map(h => h.depth)) !== JSON.stringify(zh.map(h => h.depth))) throw new Error('Translated section structure differs');
  return { ...Object.fromEntries(en.map((heading, i) => [zh[i].id, heading.id])), ...legacy };
}

export function addSectionAliases(markdown, mapping) {
  const slugger = new GithubSlugger();
  return markdown.split(/(```[\s\S]*?```)/g).map((part, i) => i % 2 ? part : part.replace(/^(#{2,6}) (.+)$/gm, (heading, _depth, title) => {
    const id = slugger.slug(title.replace(/`|\*|<[^>]+>/g, ''));
    const canonical = mapping[id] ?? id;
    const aliases = new Set([canonical, ...Object.entries(mapping).filter(([, target]) => target === canonical).map(([alias]) => alias)]);
    aliases.delete(id);
    return [...aliases].map(alias => `<span id="${alias}" data-section-alias="${canonical}"></span>\n\n`).join('') + heading;
  })).join('');
}
